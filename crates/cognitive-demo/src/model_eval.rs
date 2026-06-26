//! model_eval — P11-MODEL-EVAL, the honest fork.
//!
//! It consumes FAIL-0 [`crate::ModelNeedCandidate`] records and a battery of comparison
//! observations, and emits a deterministic [`ModelNeedVerdict`] — WITHOUT opening training,
//! touching weights, promoting a model, or treating model need as authorization. It answers
//! "what is the next action?" — one of four verdicts — never "train now".
//!
//! The genuine chain is SCORE-0 -> FAIL-0 -> MODEL-EVAL: the candidates it weighs are produced
//! by the REAL [`crate::detect_failures`] over REAL SCORE-0 failure observations (it does not
//! fabricate a candidate). For each candidate it reads the comparison observations
//! ([`EvalComparison`] under [`EvalCondition`]s — baseline / prompt / retrieval / horizon /
//! substrate improved) plus the holdout and memorization signals, and decides whether the
//! failure is:
//!   - RESOLVED by a non-weight fix (prompt / retrieval / horizon improvement) -> no model need,
//!   - SUBSTRATE-levered (a substrate fix removes it) -> improve the substrate first, or
//!   - a RESIDUAL clean model failure that survives ALL cleanup AND a trustworthy holdout.
//!
//! The four verdicts:
//!   - `no_training_needed`        — no recurring clean model need remains,
//!   - `improve_substrate_first`   — substrate-caused failures dominate,
//!   - `collect_more_data`         — evidence is insufficient, unstable, contaminated, or leaked,
//!   - `training_candidate_only`   — residual clean model failures survive cleanup.
//!
//! `training_candidate_only` is NOT training authorization: a candidacy flag for a LATER explicit
//! gate. The report's `training_justified` / `opens_training` / `authorizes_training` and the
//! [`TrainingCandidateSignal`]'s flags are ALL sourced from the const
//! [`TRAINING_CANDIDATE_IS_AUTHORIZATION`] (`false`); no path sets any true. Training stays closed
//! even when the verdict is `training_candidate_only`.
//!
//! The boundary, recorded verbatim in [`MODEL_EVAL_BOUNDARY_LINES`]:
//!
//!   The model-need evaluation compares residual clean failures.
//!   It does not create truth.
//!   It does not create memory.
//!   It does not create evidence.
//!   It does not train.
//!   It does not execute external actions.
//!   It does not promote models.
//!   It does not grant new authority.
//!   TrainingCandidateOnly is not training authorization.
//!
//! Determinism: runs are weighed in input order; no clock / entropy / float / IO. Reports derive
//! `Serialize` but NOT `Deserialize` — a serialized report is never trusted as input; it is
//! re-derived and byte-compared ([`verify_model_eval_report_json`] /
//! [`verify_model_eval_matrix_json`]), so any tampering is refused.

use crate::{
    detect_failures, verifier_score_matrix, FailureClass, FailureContext, FailureObservation,
    FailureSignal, ModelNeedCandidate, ScoreClass, ScoreReason,
};
use serde::Serialize;

const SCHEMA: &str = "model-need-eval-v0.1";

/// The number of allowed verdicts. Pinned by the gate.
pub const VERDICT_COUNT: usize = 4;

/// The number of observed scenario cells in [`model_eval_matrix`]. `training_never_opens` is the
/// matrix-level conjunction (the 16th rubric line), not a cell.
pub const MODEL_EVAL_SCENARIO_COUNT: usize = 15;

/// The minimum number of trustworthy residual clean failures required before the verdict may be
/// `training_candidate_only`. A SINGLE residual is never enough (it falls to `collect_more_data`).
pub const MODEL_NEED_MIN_RESIDUALS: usize = 2;

/// A `training_candidate_only` verdict is a candidacy flag for a LATER explicit gate, NEVER
/// training authorization. This structural `const` is the single source of the report's and the
/// signal's `training_justified` / `opens_training` / `authorizes_training` flags.
const TRAINING_CANDIDATE_IS_AUTHORIZATION: bool = false;

/// The MODEL-EVAL boundary, recorded verbatim and pinned by the release gate.
pub const MODEL_EVAL_BOUNDARY_LINES: [&str; 9] = [
    "The model-need evaluation compares residual clean failures.",
    "It does not create truth.",
    "It does not create memory.",
    "It does not create evidence.",
    "It does not train.",
    "It does not execute external actions.",
    "It does not promote models.",
    "It does not grant new authority.",
    "TrainingCandidateOnly is not training authorization.",
];

// --- the four verdicts ---

/// The deterministic next-action verdict. None of these is training authorization;
/// `TrainingCandidateOnly` is a candidacy flag for a later explicit gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ModelNeedVerdict {
    NoTrainingNeeded,
    ImproveSubstrateFirst,
    CollectMoreData,
    TrainingCandidateOnly,
}

impl ModelNeedVerdict {
    /// The four verdicts in canonical order.
    pub const ALL: [ModelNeedVerdict; VERDICT_COUNT] = [
        ModelNeedVerdict::NoTrainingNeeded,
        ModelNeedVerdict::ImproveSubstrateFirst,
        ModelNeedVerdict::CollectMoreData,
        ModelNeedVerdict::TrainingCandidateOnly,
    ];

    /// The stable, snake_case verdict name pinned by the release gate.
    pub fn tag(self) -> &'static str {
        match self {
            ModelNeedVerdict::NoTrainingNeeded => "no_training_needed",
            ModelNeedVerdict::ImproveSubstrateFirst => "improve_substrate_first",
            ModelNeedVerdict::CollectMoreData => "collect_more_data",
            ModelNeedVerdict::TrainingCandidateOnly => "training_candidate_only",
        }
    }
}

/// The four verdict names in canonical order — pinned, in source, by the release gate.
pub const VERDICT_NAMES: [&str; VERDICT_COUNT] = [
    "no_training_needed",
    "improve_substrate_first",
    "collect_more_data",
    "training_candidate_only",
];

// --- comparison observations ---

/// A dimension that could remove a model need WITHOUT new weights: the baseline plus the four
/// non-weight improvements the eval compares against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EvalCondition {
    Baseline,
    PromptImproved,
    RetrievalImproved,
    HorizonImproved,
    SubstrateImproved,
}

impl EvalCondition {
    pub fn tag(self) -> &'static str {
        match self {
            EvalCondition::Baseline => "baseline",
            EvalCondition::PromptImproved => "prompt_improved",
            EvalCondition::RetrievalImproved => "retrieval_improved",
            EvalCondition::HorizonImproved => "horizon_improved",
            EvalCondition::SubstrateImproved => "substrate_improved",
        }
    }
}

/// One comparison observation: under `condition`, did the candidate's failure PERSIST? A
/// `failure_persisted == false` means that improvement REMOVED the failure (so it was not a
/// genuine model gap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct EvalComparison {
    pub condition: EvalCondition,
    pub failure_persisted: bool,
}

/// One eval run: a REAL FAIL-0 candidate, its comparison observations, and the holdout /
/// memorization / stability signals that decide whether the residual can be trusted. This is the
/// per-candidate input — the eval never invents a candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalRun {
    pub candidate: ModelNeedCandidate,
    pub comparisons: Vec<EvalComparison>,
    pub holdout_present: bool,
    pub holdout_contaminated: bool,
    pub memorization_leaked: bool,
    pub stable: bool,
}

impl EvalRun {
    fn persisted_under(&self, cond: EvalCondition) -> bool {
        self.comparisons
            .iter()
            .find(|c| c.condition == cond)
            .map(|c| c.failure_persisted)
            // An absent comparison means that improvement was not shown to remove the failure.
            .unwrap_or(true)
    }

    fn substrate_removes(&self) -> bool {
        !self.persisted_under(EvalCondition::SubstrateImproved)
    }

    fn harness_removes(&self) -> bool {
        !self.persisted_under(EvalCondition::PromptImproved)
            || !self.persisted_under(EvalCondition::RetrievalImproved)
            || !self.persisted_under(EvalCondition::HorizonImproved)
    }

    /// The failure persists across the baseline AND every improvement — no non-weight fix removes it.
    fn persists_all(&self) -> bool {
        self.persisted_under(EvalCondition::Baseline)
            && !self.substrate_removes()
            && !self.harness_removes()
    }

    /// The result can be trusted: a clean present holdout, no memorization leakage, and stable.
    fn trustworthy(&self) -> bool {
        self.holdout_present
            && !self.holdout_contaminated
            && !self.memorization_leaked
            && self.stable
    }

    /// A residual clean model failure: persists everywhere AND trustworthy.
    fn is_residual(&self) -> bool {
        self.persists_all() && self.trustworthy()
    }
}

/// Which lever, if any, removes a run's failure (substrate has priority — it is the
/// improve-substrate-first signal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lever {
    Substrate,
    Harness,
    None,
}

fn lever_of(run: &EvalRun) -> Lever {
    if run.substrate_removes() {
        Lever::Substrate
    } else if run.harness_removes() {
        Lever::Harness
    } else {
        Lever::None
    }
}

/// The whole eval input: the per-candidate runs. The recurrence/residual policy is the const
/// [`MODEL_NEED_MIN_RESIDUALS`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelEvalBattery {
    pub runs: Vec<EvalRun>,
}

impl ModelEvalBattery {
    pub fn new(runs: Vec<EvalRun>) -> Self {
        Self { runs }
    }
}

// --- output records (Serialize, never Deserialize) ---

/// A clean model failure that SURVIVED all cleanup (every non-weight fix and a trustworthy
/// holdout) — the only thing that can support a training-candidate verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResidualFailure {
    pub class: FailureClass,
    pub reason: String,
    pub recurrences: usize,
    pub holdout_present: bool,
    pub source_hashes: Vec<String>,
}

/// The aggregate evidence behind the verdict — every count derived from the runs, never hand-set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ModelNeedEvidence {
    pub candidate_count: usize,
    pub residual_count: usize,
    pub substrate_levered_count: usize,
    pub harness_levered_count: usize,
    pub untrustworthy_count: usize,
    pub holdout_present_count: usize,
    pub holdout_contaminated_count: usize,
    pub memorization_leaked_count: usize,
}

/// Emitted ONLY on a `training_candidate_only` verdict: a candidacy flag for a later explicit
/// gate. It is NOT training authorization — every training flag is structurally `false`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingCandidateSignal {
    pub residual_count: usize,
    pub classes: Vec<String>,
    /// Always `false`: a candidate signal is not training justification.
    pub training_justified: bool,
    /// Always `false`: it cannot open training eligibility.
    pub opens_training: bool,
    /// Always `false`: it does not authorize training.
    pub authorizes_training: bool,
}

/// The inert invariants every eval run upholds — every field `false` by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ModelEvalBoundary {
    pub created_truth: bool,
    pub created_memory: bool,
    pub created_evidence: bool,
    pub promoted_model: bool,
    pub granted_authority: bool,
    pub executed_external: bool,
    pub opened_training: bool,
    pub authorizes_training: bool,
}

impl ModelEvalBoundary {
    fn inert() -> Self {
        Self {
            created_truth: false,
            created_memory: false,
            created_evidence: false,
            promoted_model: false,
            granted_authority: false,
            executed_external: false,
            opened_training: false,
            authorizes_training: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !self.created_truth
            && !self.created_memory
            && !self.created_evidence
            && !self.promoted_model
            && !self.granted_authority
            && !self.executed_external
            && !self.opened_training
            && !self.authorizes_training
    }
}

/// The complete, deterministic output of a model-need evaluation: the verdict, the evidence, the
/// surviving residuals, the (optional) training-candidate signal, the always-`false` training
/// flags, and the inert boundary. `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelNeedEvalReport {
    pub schema: &'static str,
    pub verdict: ModelNeedVerdict,
    pub evidence: ModelNeedEvidence,
    pub residuals: Vec<ResidualFailure>,
    pub training_candidate_signal: Option<TrainingCandidateSignal>,
    /// Always `false`: the evaluation cannot justify training.
    pub training_justified: bool,
    /// Always `false`: the evaluation cannot open training eligibility.
    pub opens_training: bool,
    /// Always `false`: the evaluation does not authorize training.
    pub authorizes_training: bool,
    pub boundary: ModelEvalBoundary,
}

// --- the evaluator ---

/// Run the model-need evaluation over `battery`. Deterministic precedence:
/// 1. no runs -> `NoTrainingNeeded`;
/// 2. any contaminated holdout or memorization leakage -> `CollectMoreData` (cannot be trusted);
/// 3. >= [`MODEL_NEED_MIN_RESIDUALS`] trustworthy residuals -> `TrainingCandidateOnly`;
/// 4. 1 residual -> `CollectMoreData` (a single candidate is not enough);
/// 5. substrate-levered runs dominate -> `ImproveSubstrateFirst`;
/// 6. any untrustworthy run (no/unstable holdout) with no residual -> `CollectMoreData`;
/// 7. otherwise (all resolved by non-substrate fixes, or none) -> `NoTrainingNeeded`.
pub fn evaluate_model_need(battery: &ModelEvalBattery) -> ModelNeedEvalReport {
    let runs = &battery.runs;

    let any_tainted = runs
        .iter()
        .any(|r| r.holdout_contaminated || r.memorization_leaked);
    let residual_runs: Vec<&EvalRun> = runs.iter().filter(|r| r.is_residual()).collect();
    let substrate_levered = runs
        .iter()
        .filter(|r| lever_of(r) == Lever::Substrate)
        .count();
    let harness_levered = runs
        .iter()
        .filter(|r| lever_of(r) == Lever::Harness)
        .count();
    let untrustworthy = runs.iter().filter(|r| !r.trustworthy()).count();

    let verdict = if runs.is_empty() {
        ModelNeedVerdict::NoTrainingNeeded
    } else if any_tainted {
        ModelNeedVerdict::CollectMoreData
    } else if residual_runs.len() >= MODEL_NEED_MIN_RESIDUALS {
        ModelNeedVerdict::TrainingCandidateOnly
    } else if !residual_runs.is_empty() {
        // 1..MIN_RESIDUALS residuals — a single candidate is not enough on its own.
        ModelNeedVerdict::CollectMoreData
    } else if substrate_levered > 0 && substrate_levered >= harness_levered {
        ModelNeedVerdict::ImproveSubstrateFirst
    } else if untrustworthy > 0 {
        ModelNeedVerdict::CollectMoreData
    } else {
        ModelNeedVerdict::NoTrainingNeeded
    };

    let residuals: Vec<ResidualFailure> = residual_runs
        .iter()
        .map(|r| ResidualFailure {
            class: r.candidate.class,
            reason: r.candidate.reason.clone(),
            recurrences: r.candidate.recurrences,
            holdout_present: r.holdout_present,
            source_hashes: r.candidate.source_hashes.clone(),
        })
        .collect();

    let evidence = ModelNeedEvidence {
        candidate_count: runs.len(),
        residual_count: residual_runs.len(),
        substrate_levered_count: substrate_levered,
        harness_levered_count: harness_levered,
        untrustworthy_count: untrustworthy,
        holdout_present_count: runs.iter().filter(|r| r.holdout_present).count(),
        holdout_contaminated_count: runs.iter().filter(|r| r.holdout_contaminated).count(),
        memorization_leaked_count: runs.iter().filter(|r| r.memorization_leaked).count(),
    };

    let training_candidate_signal = if verdict == ModelNeedVerdict::TrainingCandidateOnly {
        Some(TrainingCandidateSignal {
            residual_count: residuals.len(),
            classes: residuals
                .iter()
                .map(|r| r.class.tag().to_string())
                .collect(),
            training_justified: TRAINING_CANDIDATE_IS_AUTHORIZATION,
            opens_training: TRAINING_CANDIDATE_IS_AUTHORIZATION,
            authorizes_training: TRAINING_CANDIDATE_IS_AUTHORIZATION,
        })
    } else {
        None
    };

    ModelNeedEvalReport {
        schema: SCHEMA,
        verdict,
        evidence,
        residuals,
        training_candidate_signal,
        training_justified: TRAINING_CANDIDATE_IS_AUTHORIZATION,
        opens_training: TRAINING_CANDIDATE_IS_AUTHORIZATION,
        authorizes_training: TRAINING_CANDIDATE_IS_AUTHORIZATION,
        boundary: ModelEvalBoundary::inert(),
    }
}

/// The eval report serialized to canonical JSON (for an operator gate to emit).
pub fn evaluate_model_need_json(battery: &ModelEvalBattery) -> String {
    serde_json::to_string(&evaluate_model_need(battery)).expect("model-need eval report serializes")
}

/// What can go wrong verifying a serialized eval artifact.
#[derive(Debug, PartialEq, Eq)]
pub enum ModelEvalError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the report from the SAME battery and byte-compare against `candidate`. The report is
/// `Serialize` but never `Deserialize`: a serialized report is NOT trusted as input — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_model_eval_report_json(
    battery: &ModelEvalBattery,
    candidate: &str,
) -> Result<(), ModelEvalError> {
    if candidate == evaluate_model_need_json(battery) {
        Ok(())
    } else {
        Err(ModelEvalError::ReplayMismatch)
    }
}

// --- consuming REAL FAIL-0 candidates (SCORE-0 -> FAIL-0 -> MODEL-EVAL) ---

/// Produce a REAL FAIL-0 [`ModelNeedCandidate`] by running the REAL [`detect_failures`] over `n`
/// repeats of a real SCORE-0 failure observation. The eval never fabricates a candidate — the
/// whole chain (SCORE-0 verifier failure -> FAIL-0 recurrence -> eval) is exercised.
fn real_candidate(
    failures: &[FailureObservation],
    class: FailureClass,
    sc: ScoreClass,
    reason: ScoreReason,
    n: usize,
) -> ModelNeedCandidate {
    let obs = failures
        .iter()
        .find(|f| f.class == sc && f.reason == reason)
        .cloned()
        .expect("the SCORE-0 matrix yields the expected failure observation");
    let signals: Vec<FailureSignal> = (0..n)
        .map(|_| FailureSignal::new(class, obs.clone(), FailureContext::clean()))
        .collect();
    detect_failures(&signals)
        .candidates
        .into_iter()
        .find(|c| c.class == class)
        .expect("FAIL-0 emits a candidate for the recurring clean failure")
}

// --- run builders ---

fn all_persist() -> Vec<EvalComparison> {
    [
        EvalCondition::Baseline,
        EvalCondition::PromptImproved,
        EvalCondition::RetrievalImproved,
        EvalCondition::HorizonImproved,
        EvalCondition::SubstrateImproved,
    ]
    .iter()
    .map(|&condition| EvalComparison {
        condition,
        failure_persisted: true,
    })
    .collect()
}

/// A residual run: the failure persists across every condition, with a clean trustworthy holdout.
fn residual_run(candidate: ModelNeedCandidate) -> EvalRun {
    EvalRun {
        candidate,
        comparisons: all_persist(),
        holdout_present: true,
        holdout_contaminated: false,
        memorization_leaked: false,
        stable: true,
    }
}

/// A run whose failure is REMOVED by improving `cond` (so it is not a genuine model gap).
fn resolved_by(candidate: ModelNeedCandidate, cond: EvalCondition) -> EvalRun {
    EvalRun {
        candidate,
        comparisons: vec![
            EvalComparison {
                condition: EvalCondition::Baseline,
                failure_persisted: true,
            },
            EvalComparison {
                condition: cond,
                failure_persisted: false,
            },
        ],
        holdout_present: true,
        holdout_contaminated: false,
        memorization_leaked: false,
        stable: true,
    }
}

// --- the eval scenario matrix (observes the real evaluator over REAL FAIL-0 candidates) ---

/// One scenario cell: the OBSERVED verdict of running the real evaluator over a constructed
/// battery whose candidates are REAL FAIL-0 records. Never asserted — recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelEvalScenarioCell {
    pub name: &'static str,
    pub verdict: ModelNeedVerdict,
    pub candidate_count: usize,
    pub residual_count: usize,
    pub opens_training: bool,
    pub detail: String,
}

/// The fixed eval scenario matrix. Every cell runs the real evaluator and records what it
/// observed; `training_never_opens` is the conjunction across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelEvalMatrix {
    pub schema: &'static str,
    pub scenarios: Vec<ModelEvalScenarioCell>,
    pub verdicts: [&'static str; VERDICT_COUNT],
    pub min_residuals: usize,
    pub training_never_opens: bool,
    pub boundary: ModelEvalBoundary,
}

impl ModelEvalMatrix {
    pub fn scenario(&self, name: &str) -> Option<&ModelEvalScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn eval_cell(name: &'static str, battery: ModelEvalBattery) -> ModelEvalScenarioCell {
    let report = evaluate_model_need(&battery);
    ModelEvalScenarioCell {
        name,
        verdict: report.verdict,
        candidate_count: report.evidence.candidate_count,
        residual_count: report.evidence.residual_count,
        opens_training: report.opens_training,
        detail: report.verdict.tag().to_string(),
    }
}

/// The canonical battery for the serialized-report tamper proof: two residual reading-misgrounding
/// failures (which yields `training_candidate_only`).
fn canonical_battery(failures: &[FailureObservation]) -> ModelEvalBattery {
    let c = real_candidate(
        failures,
        FailureClass::ReadingMisgrounding,
        ScoreClass::Grounding,
        ScoreReason::Ungrounded,
        MODEL_NEED_MIN_RESIDUALS,
    );
    ModelEvalBattery::new(vec![residual_run(c.clone()), residual_run(c)])
}

/// The serialized-report tamper cell: tamper a real eval report JSON and observe the re-derive
/// verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous; the
/// canonical form must itself verify.
fn tamper_cell(failures: &[FailureObservation]) -> ModelEvalScenarioCell {
    let battery = canonical_battery(failures);
    let canonical = evaluate_model_need_json(&battery);
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_model_eval_report_json(&battery, &tampered).is_err()
        && verify_model_eval_report_json(&battery, &canonical).is_ok();
    let report = evaluate_model_need(&battery);
    ModelEvalScenarioCell {
        name: "serialized_eval_report_tamper_refused",
        verdict: report.verdict,
        candidate_count: report.evidence.candidate_count,
        residual_count: report.evidence.residual_count,
        opens_training: report.opens_training,
        detail: if refused {
            "serialized_report_tamper_refused".to_string()
        } else {
            "VACUOUS: report verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 15-scenario eval matrix from the REAL evaluator over REAL FAIL-0 candidates.
pub fn model_eval_matrix() -> ModelEvalMatrix {
    // Derive the SCORE-0 failure set ONCE; every candidate reuses it (no per-build rebuild).
    let failures = verifier_score_matrix().failures;
    let reading = || {
        real_candidate(
            &failures,
            FailureClass::ReadingMisgrounding,
            ScoreClass::Grounding,
            ScoreReason::Ungrounded,
            MODEL_NEED_MIN_RESIDUALS,
        )
    };

    // a residual run with a single field flipped (no holdout / unstable / contaminated / leaked).
    let no_holdout = |c: ModelNeedCandidate| EvalRun {
        holdout_present: false,
        ..residual_run(c)
    };
    let unstable = |c: ModelNeedCandidate| EvalRun {
        stable: false,
        ..residual_run(c)
    };
    let contaminated = |c: ModelNeedCandidate| EvalRun {
        holdout_contaminated: true,
        ..residual_run(c)
    };
    let leaked = |c: ModelNeedCandidate| EvalRun {
        memorization_leaked: true,
        ..residual_run(c)
    };

    let scenarios = vec![
        eval_cell(
            "no_candidates_no_training_needed",
            ModelEvalBattery::new(vec![]),
        ),
        eval_cell(
            "substrate_failures_improve_substrate_first",
            ModelEvalBattery::new(vec![
                resolved_by(reading(), EvalCondition::SubstrateImproved),
                resolved_by(reading(), EvalCondition::SubstrateImproved),
            ]),
        ),
        eval_cell(
            "insufficient_evidence_collect_more_data",
            ModelEvalBattery::new(vec![no_holdout(reading())]),
        ),
        eval_cell(
            "unstable_candidate_collect_more_data",
            ModelEvalBattery::new(vec![unstable(reading())]),
        ),
        eval_cell(
            "residual_clean_failure_training_candidate_only",
            ModelEvalBattery::new(vec![residual_run(reading()), residual_run(reading())]),
        ),
        eval_cell(
            "prompt_fix_removes_model_need",
            ModelEvalBattery::new(vec![
                resolved_by(reading(), EvalCondition::PromptImproved),
                resolved_by(reading(), EvalCondition::PromptImproved),
            ]),
        ),
        eval_cell(
            "retrieval_fix_removes_model_need",
            ModelEvalBattery::new(vec![
                resolved_by(reading(), EvalCondition::RetrievalImproved),
                resolved_by(reading(), EvalCondition::RetrievalImproved),
            ]),
        ),
        eval_cell(
            "horizon_fix_removes_model_need",
            ModelEvalBattery::new(vec![
                resolved_by(reading(), EvalCondition::HorizonImproved),
                resolved_by(reading(), EvalCondition::HorizonImproved),
            ]),
        ),
        eval_cell(
            "substrate_fix_removes_model_need",
            ModelEvalBattery::new(vec![resolved_by(
                reading(),
                EvalCondition::SubstrateImproved,
            )]),
        ),
        eval_cell(
            "holdout_clean_recorded",
            ModelEvalBattery::new(vec![residual_run(reading()), residual_run(reading())]),
        ),
        eval_cell(
            "holdout_contamination_detected",
            ModelEvalBattery::new(vec![contaminated(reading()), residual_run(reading())]),
        ),
        eval_cell(
            "memorization_leakage_detected",
            ModelEvalBattery::new(vec![leaked(reading()), residual_run(reading())]),
        ),
        eval_cell(
            "single_candidate_not_enough",
            ModelEvalBattery::new(vec![residual_run(reading())]),
        ),
        tamper_cell(&failures),
        eval_cell(
            "training_candidate_only_not_authorization",
            ModelEvalBattery::new(vec![residual_run(reading()), residual_run(reading())]),
        ),
    ];

    let training_never_opens = scenarios.iter().all(|c| !c.opens_training);
    ModelEvalMatrix {
        schema: SCHEMA,
        scenarios,
        verdicts: VERDICT_NAMES,
        min_residuals: MODEL_NEED_MIN_RESIDUALS,
        training_never_opens,
        boundary: ModelEvalBoundary::inert(),
    }
}

/// The eval matrix serialized to canonical JSON.
pub fn model_eval_matrix_json() -> String {
    serde_json::to_string(&model_eval_matrix()).expect("model eval matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`:
/// a serialized matrix is NOT trusted — it is re-derived and compared, so any tampering is refused.
pub fn verify_model_eval_matrix_json(candidate: &str) -> Result<(), ModelEvalError> {
    if candidate == model_eval_matrix_json() {
        Ok(())
    } else {
        Err(ModelEvalError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failures() -> Vec<FailureObservation> {
        verifier_score_matrix().failures
    }
    fn reading_candidate(n: usize) -> ModelNeedCandidate {
        real_candidate(
            &failures(),
            FailureClass::ReadingMisgrounding,
            ScoreClass::Grounding,
            ScoreReason::Ungrounded,
            n,
        )
    }

    // --- structure ---

    #[test]
    fn there_are_exactly_four_verdicts_with_stable_names() {
        assert_eq!(VERDICT_COUNT, 4);
        assert_eq!(ModelNeedVerdict::ALL.len(), 4);
        let tags: Vec<&str> = ModelNeedVerdict::ALL.iter().map(|v| v.tag()).collect();
        assert_eq!(tags, VERDICT_NAMES.to_vec());
    }

    #[test]
    fn boundary_lines_are_the_nine_and_inert() {
        assert_eq!(MODEL_EVAL_BOUNDARY_LINES.len(), 9);
        assert_eq!(
            MODEL_EVAL_BOUNDARY_LINES[0],
            "The model-need evaluation compares residual clean failures."
        );
        assert_eq!(
            MODEL_EVAL_BOUNDARY_LINES[8],
            "TrainingCandidateOnly is not training authorization."
        );
        assert!(ModelEvalBoundary::inert().all_inert());
    }

    // --- the chain: real FAIL-0 candidates ---

    #[test]
    fn candidates_come_from_the_real_fail0_detector() {
        let c = reading_candidate(MODEL_NEED_MIN_RESIDUALS);
        assert_eq!(c.class, FailureClass::ReadingMisgrounding);
        assert_eq!(c.recurrences, MODEL_NEED_MIN_RESIDUALS);
        // A FAIL-0 candidate is structurally not training authorization.
        assert!(!c.training_justified);
        assert!(!c.opens_training);
        assert!(!c.authorizes_training);
    }

    // --- verdicts ---

    #[test]
    fn no_candidates_yields_no_training_needed() {
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![]));
        assert_eq!(r.verdict, ModelNeedVerdict::NoTrainingNeeded);
        assert!(r.training_candidate_signal.is_none());
    }

    #[test]
    fn two_residual_clean_failures_yield_training_candidate_only_not_authorization() {
        let c = reading_candidate(2);
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![
            residual_run(c.clone()),
            residual_run(c),
        ]));
        assert_eq!(r.verdict, ModelNeedVerdict::TrainingCandidateOnly);
        assert_eq!(r.residuals.len(), 2);
        // The candidacy signal is emitted, but it is NOT training authorization.
        let sig = r.training_candidate_signal.expect("signal emitted");
        assert!(!sig.training_justified);
        assert!(!sig.opens_training);
        assert!(!sig.authorizes_training);
        assert!(!r.training_justified);
        assert!(!r.opens_training);
        assert!(!r.authorizes_training);
    }

    #[test]
    fn single_residual_is_not_enough() {
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![residual_run(
            reading_candidate(2),
        )]));
        assert_eq!(r.verdict, ModelNeedVerdict::CollectMoreData);
        assert!(r.training_candidate_signal.is_none());
    }

    #[test]
    fn substrate_levered_failures_yield_improve_substrate_first() {
        let c = reading_candidate(2);
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![
            resolved_by(c.clone(), EvalCondition::SubstrateImproved),
            resolved_by(c, EvalCondition::SubstrateImproved),
        ]));
        assert_eq!(r.verdict, ModelNeedVerdict::ImproveSubstrateFirst);
        assert_eq!(r.evidence.substrate_levered_count, 2);
        assert_eq!(r.residuals.len(), 0);
    }

    #[test]
    fn harness_fixes_yield_no_training_needed() {
        let c = reading_candidate(2);
        for cond in [
            EvalCondition::PromptImproved,
            EvalCondition::RetrievalImproved,
            EvalCondition::HorizonImproved,
        ] {
            let r = evaluate_model_need(&ModelEvalBattery::new(vec![
                resolved_by(c.clone(), cond),
                resolved_by(c.clone(), cond),
            ]));
            assert_eq!(
                r.verdict,
                ModelNeedVerdict::NoTrainingNeeded,
                "condition {cond:?}"
            );
        }
    }

    #[test]
    fn contaminated_holdout_never_passes() {
        let c = reading_candidate(2);
        let tainted = EvalRun {
            holdout_contaminated: true,
            ..residual_run(c.clone())
        };
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![tainted, residual_run(c)]));
        // Even with an otherwise-residual run, contamination forces collect_more_data.
        assert_eq!(r.verdict, ModelNeedVerdict::CollectMoreData);
        assert_eq!(r.evidence.holdout_contaminated_count, 1);
        assert!(r.training_candidate_signal.is_none());
    }

    #[test]
    fn memorization_leakage_never_passes() {
        let c = reading_candidate(2);
        let leaky = EvalRun {
            memorization_leaked: true,
            ..residual_run(c.clone())
        };
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![leaky, residual_run(c)]));
        assert_eq!(r.verdict, ModelNeedVerdict::CollectMoreData);
        assert_eq!(r.evidence.memorization_leaked_count, 1);
    }

    #[test]
    fn unstable_or_missing_holdout_yields_collect_more_data() {
        let c = reading_candidate(2);
        let no_holdout = EvalRun {
            holdout_present: false,
            ..residual_run(c.clone())
        };
        assert_eq!(
            evaluate_model_need(&ModelEvalBattery::new(vec![no_holdout])).verdict,
            ModelNeedVerdict::CollectMoreData
        );
        let unstable = EvalRun {
            stable: false,
            ..residual_run(c)
        };
        assert_eq!(
            evaluate_model_need(&ModelEvalBattery::new(vec![unstable])).verdict,
            ModelNeedVerdict::CollectMoreData
        );
    }

    // --- training closure ---

    #[test]
    fn evaluation_never_opens_training_even_for_training_candidate_only() {
        let c = reading_candidate(2);
        let r = evaluate_model_need(&ModelEvalBattery::new(vec![
            residual_run(c.clone()),
            residual_run(c),
        ]));
        assert_eq!(r.verdict, ModelNeedVerdict::TrainingCandidateOnly);
        assert!(!r.training_justified && !r.opens_training && !r.authorizes_training);
        assert!(r.boundary.all_inert());
        // The REAL P12 verdict (decided on empty inputs) is unmoved and still closed.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    // --- re-derivation ---

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let c = reading_candidate(2);
        let battery = ModelEvalBattery::new(vec![residual_run(c.clone()), residual_run(c)]);
        let canonical = evaluate_model_need_json(&battery);
        assert_eq!(canonical, evaluate_model_need_json(&battery));
        assert!(verify_model_eval_report_json(&battery, &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_model_eval_report_json(&battery, &tampered),
            Err(ModelEvalError::ReplayMismatch)
        );
    }

    // --- the matrix ---

    #[test]
    fn matrix_has_the_fifteen_named_scenarios() {
        let m = model_eval_matrix();
        assert_eq!(m.scenarios.len(), MODEL_EVAL_SCENARIO_COUNT);
        let names: Vec<&str> = m.scenarios.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            vec![
                "no_candidates_no_training_needed",
                "substrate_failures_improve_substrate_first",
                "insufficient_evidence_collect_more_data",
                "unstable_candidate_collect_more_data",
                "residual_clean_failure_training_candidate_only",
                "prompt_fix_removes_model_need",
                "retrieval_fix_removes_model_need",
                "horizon_fix_removes_model_need",
                "substrate_fix_removes_model_need",
                "holdout_clean_recorded",
                "holdout_contamination_detected",
                "memorization_leakage_detected",
                "single_candidate_not_enough",
                "serialized_eval_report_tamper_refused",
                "training_candidate_only_not_authorization",
            ]
        );
    }

    #[test]
    fn matrix_records_the_observed_verdicts() {
        let m = model_eval_matrix();
        let v = |n: &str| m.scenario(n).unwrap().verdict;
        assert_eq!(
            v("no_candidates_no_training_needed"),
            ModelNeedVerdict::NoTrainingNeeded
        );
        assert_eq!(
            v("substrate_failures_improve_substrate_first"),
            ModelNeedVerdict::ImproveSubstrateFirst
        );
        assert_eq!(
            v("insufficient_evidence_collect_more_data"),
            ModelNeedVerdict::CollectMoreData
        );
        assert_eq!(
            v("unstable_candidate_collect_more_data"),
            ModelNeedVerdict::CollectMoreData
        );
        assert_eq!(
            v("residual_clean_failure_training_candidate_only"),
            ModelNeedVerdict::TrainingCandidateOnly
        );
        assert_eq!(
            v("prompt_fix_removes_model_need"),
            ModelNeedVerdict::NoTrainingNeeded
        );
        assert_eq!(
            v("retrieval_fix_removes_model_need"),
            ModelNeedVerdict::NoTrainingNeeded
        );
        assert_eq!(
            v("horizon_fix_removes_model_need"),
            ModelNeedVerdict::NoTrainingNeeded
        );
        assert_eq!(
            v("substrate_fix_removes_model_need"),
            ModelNeedVerdict::ImproveSubstrateFirst
        );
        assert_eq!(
            v("holdout_clean_recorded"),
            ModelNeedVerdict::TrainingCandidateOnly
        );
        assert_eq!(
            v("holdout_contamination_detected"),
            ModelNeedVerdict::CollectMoreData
        );
        assert_eq!(
            v("memorization_leakage_detected"),
            ModelNeedVerdict::CollectMoreData
        );
        assert_eq!(
            v("single_candidate_not_enough"),
            ModelNeedVerdict::CollectMoreData
        );
        assert_eq!(
            v("training_candidate_only_not_authorization"),
            ModelNeedVerdict::TrainingCandidateOnly
        );
    }

    #[test]
    fn matrix_serialized_report_tamper_is_refused() {
        let cell = model_eval_matrix()
            .scenario("serialized_eval_report_tamper_refused")
            .unwrap()
            .clone();
        assert_eq!(cell.detail, "serialized_report_tamper_refused");
    }

    #[test]
    fn matrix_opens_no_training_in_any_scenario() {
        let m = model_eval_matrix();
        assert!(m.training_never_opens);
        assert_eq!(m.min_residuals, MODEL_NEED_MIN_RESIDUALS);
        assert_eq!(m.verdicts, VERDICT_NAMES);
        for c in &m.scenarios {
            assert!(!c.opens_training, "scenario {} opened training", c.name);
        }
        assert!(m.boundary.all_inert());
    }

    #[test]
    fn matrix_is_deterministic_and_re_derivable() {
        assert_eq!(model_eval_matrix(), model_eval_matrix());
        assert_eq!(model_eval_matrix_json(), model_eval_matrix_json());
        let canonical = model_eval_matrix_json();
        assert!(verify_model_eval_matrix_json(&canonical).is_ok());
        assert_eq!(
            verify_model_eval_matrix_json(&format!("{canonical} ")),
            Err(ModelEvalError::ReplayMismatch)
        );
    }
}
