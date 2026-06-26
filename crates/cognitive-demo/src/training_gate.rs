//! TRAIN-GATE-0 — the explicit, closed-by-default gate that stands between a proven model need
//! and any attempt to change weights.
//!
//! This sprint answers exactly ONE question: *are the prerequisites complete enough to allow a
//! FUTURE training attempt?* It does not answer "what should we train?" or "is the trained model
//! acceptable?". It trains nothing. It modifies no weights. It promotes and deploys nothing.
//!
//! It CONSUMES the REAL P11-MODEL-EVAL evaluation (which itself consumes FAIL-0 candidates built by
//! the REAL `detect_failures` over REAL SCORE-0 failures — the SCORE-0 -> FAIL-0 -> MODEL-EVAL ->
//! TRAIN-GATE chain). The gate runs [`evaluate_model_need`] itself over the supplied battery, so the
//! verdict is genuinely derived, never hand-set. The gate may emit [`TrainingGateDecision::
//! TrainingAttemptAllowed`] ONLY when EVERY requirement is satisfied:
//!
//!   1. a model-need verdict is present (the battery was evaluated),
//!   2. the verdict is EXACTLY `training_candidate_only` (a candidacy flag from P11),
//!   3. recurring clean-failure evidence (>= [`MIN_RECURRING_FAILURES`] residuals),
//!   4. an explicit operator authorization receipt,
//!   5. curated dataset readiness receipts,
//!   6. a clean, present holdout receipt,
//!   7. a clean contamination report (no memorization leakage),
//!   8. a rollback plan receipt,
//!   9. a production safety plan receipt,
//!  10. an affirmative authority-drift check (clean).
//!
//! It is CLOSED BY DEFAULT: any missing or unproven prerequisite denies the attempt. A
//! `training_candidate_only` verdict ALONE is insufficient; operator authorization ALONE is
//! insufficient; every requirement must hold together.
//!
//! Crucially, `TrainingAttemptAllowed` is ONLY permission to ATTEMPT a later training run. Every
//! forbidden-action flag on the report (`trains`, `modifies_weights`, `promotes_model`,
//! `deploys_model`, `training_justified`, `opens_training`) is sourced from the structural const
//! [`ALLOWED_ATTEMPT_AUTHORIZES_TRAINING`] (`false`): no path can set any true. The deeper P12 gate
//! (`reading_train_gate::decide`) stays `training_justified = false` regardless of this gate's
//! decision. Reports are `Serialize` but NEVER `Deserialize`: a serialized report is re-derived from
//! the same input and byte-compared, so tampering is refused.
//!
//! The boundary, recorded verbatim in [`TRAINING_GATE_BOUNDARY_LINES`]:
//!
//!   The training gate evaluates whether a training attempt may be authorized.
//!   It does not train.
//!   It does not modify weights.
//!   It does not create truth.
//!   It does not create memory.
//!   It does not create evidence.
//!   It does not promote models.
//!   It does not deploy models.
//!   TrainingAttemptAllowed is not model promotion.

use crate::{
    detect_failures, evaluate_model_need, verifier_score_matrix, EvalComparison, EvalCondition,
    EvalRun, FailureClass, FailureContext, FailureObservation, FailureSignal, ModelEvalBattery,
    ModelNeedCandidate, ModelNeedVerdict, ScoreClass, ScoreReason, RECURRENCE_THRESHOLD,
};
use serde::Serialize;

/// The schema tag stamped on every serialized training-gate report.
const SCHEMA: &str = "training-gate-v0.1";

/// THE structural invariant: a granted training ATTEMPT is not, by itself, training authorization,
/// execution, promotion, or deployment. Every forbidden-action flag is sourced from this const, so
/// no code path — not even an `TrainingAttemptAllowed` decision — can set one true.
const ALLOWED_ATTEMPT_AUTHORIZES_TRAINING: bool = false;

/// Recurring clean-failure evidence threshold: the consumed P11 evaluation must carry at least this
/// many residual clean failures (mirrors P11's `MODEL_NEED_MIN_RESIDUALS`).
pub const MIN_RECURRING_FAILURES: usize = 2;

/// Exactly two decision states.
pub const TRAIN_GATE_DECISION_COUNT: usize = 2;

/// The two decision-state slugs, in canonical order.
pub const TRAIN_GATE_DECISION_NAMES: [&str; TRAIN_GATE_DECISION_COUNT] =
    ["training_attempt_denied", "training_attempt_allowed"];

/// Exactly twelve refusal reasons.
pub const TRAIN_GATE_REFUSAL_COUNT: usize = 12;

/// The twelve refusal-reason slugs, in canonical order.
pub const TRAIN_GATE_REFUSAL_NAMES: [&str; TRAIN_GATE_REFUSAL_COUNT] = [
    "missing_model_need_verdict",
    "verdict_not_training_candidate",
    "missing_operator_authorization",
    "missing_curated_dataset_receipts",
    "missing_clean_holdout",
    "holdout_contaminated",
    "memorization_leakage_detected",
    "missing_recurring_failure_evidence",
    "missing_rollback_plan",
    "missing_production_safety_plan",
    "authority_drift_detected",
    "training_gate_serialized_tamper_refused",
];

/// The fixed training-gate scenario matrix size.
pub const TRAIN_GATE_SCENARIO_COUNT: usize = 19;

/// The cannot-bypass boundary, recorded verbatim.
pub const TRAINING_GATE_BOUNDARY_LINES: [&str; 9] = [
    "The training gate evaluates whether a training attempt may be authorized.",
    "It does not train.",
    "It does not modify weights.",
    "It does not create truth.",
    "It does not create memory.",
    "It does not create evidence.",
    "It does not promote models.",
    "It does not deploy models.",
    "TrainingAttemptAllowed is not model promotion.",
];

// --- decision / requirement / refusal taxonomies ---

/// The two terminal decisions of the gate. `TrainingAttemptDenied` is the closed-by-default state;
/// `TrainingAttemptAllowed` is permission to ATTEMPT a later training run — nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingGateDecision {
    /// At least one requirement is unmet — no training attempt may proceed.
    TrainingAttemptDenied,
    /// Every requirement is satisfied — a future training attempt is authorized (but not executed,
    /// and weights remain untouched).
    TrainingAttemptAllowed,
}

impl TrainingGateDecision {
    /// Every decision, in canonical order.
    pub const ALL: [TrainingGateDecision; TRAIN_GATE_DECISION_COUNT] = [
        TrainingGateDecision::TrainingAttemptDenied,
        TrainingGateDecision::TrainingAttemptAllowed,
    ];

    /// The stable slug for this decision.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingGateDecision::TrainingAttemptDenied => "training_attempt_denied",
            TrainingGateDecision::TrainingAttemptAllowed => "training_attempt_allowed",
        }
    }
}

/// A prerequisite the gate checks. Recorded (in the report's `satisfied` list) when met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingGateRequirement {
    /// The model-need battery was supplied and evaluated.
    ModelNeedVerdictPresent,
    /// The evaluated verdict is exactly `training_candidate_only`.
    VerdictIsTrainingCandidate,
    /// An explicit operator authorization receipt is present.
    OperatorAuthorization,
    /// Curated dataset readiness receipts are present.
    CuratedDataset,
    /// A present, uncontaminated holdout receipt.
    CleanHoldout,
    /// A clean contamination report.
    ContaminationClean,
    /// No memorization leakage.
    MemorizationClean,
    /// Recurring clean-failure evidence (>= `MIN_RECURRING_FAILURES`).
    RecurringFailureEvidence,
    /// A rollback plan receipt.
    RollbackPlan,
    /// A production safety plan receipt.
    ProductionSafetyPlan,
    /// An affirmative, clean authority-drift check.
    AuthorityDriftClean,
}

impl TrainingGateRequirement {
    /// Every requirement, in check order.
    pub const ALL: [TrainingGateRequirement; 11] = [
        TrainingGateRequirement::ModelNeedVerdictPresent,
        TrainingGateRequirement::VerdictIsTrainingCandidate,
        TrainingGateRequirement::OperatorAuthorization,
        TrainingGateRequirement::CuratedDataset,
        TrainingGateRequirement::CleanHoldout,
        TrainingGateRequirement::ContaminationClean,
        TrainingGateRequirement::MemorizationClean,
        TrainingGateRequirement::RecurringFailureEvidence,
        TrainingGateRequirement::RollbackPlan,
        TrainingGateRequirement::ProductionSafetyPlan,
        TrainingGateRequirement::AuthorityDriftClean,
    ];

    /// The stable slug for this requirement.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingGateRequirement::ModelNeedVerdictPresent => "model_need_verdict_present",
            TrainingGateRequirement::VerdictIsTrainingCandidate => "verdict_is_training_candidate",
            TrainingGateRequirement::OperatorAuthorization => "operator_authorization",
            TrainingGateRequirement::CuratedDataset => "curated_dataset",
            TrainingGateRequirement::CleanHoldout => "clean_holdout",
            TrainingGateRequirement::ContaminationClean => "contamination_clean",
            TrainingGateRequirement::MemorizationClean => "memorization_clean",
            TrainingGateRequirement::RecurringFailureEvidence => "recurring_failure_evidence",
            TrainingGateRequirement::RollbackPlan => "rollback_plan",
            TrainingGateRequirement::ProductionSafetyPlan => "production_safety_plan",
            TrainingGateRequirement::AuthorityDriftClean => "authority_drift_clean",
        }
    }
}

/// Why the gate refused. The first eleven are decision-path reasons; the twelfth
/// (`TrainingGateSerializedTamperRefused`) is emitted only by the serialized-report re-derivation
/// path (a tampered report is never trusted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingGateRefusal {
    /// No model-need battery was supplied — the verdict is unknown.
    MissingModelNeedVerdict,
    /// A verdict was evaluated but it is not `training_candidate_only`.
    VerdictNotTrainingCandidate,
    /// No explicit operator authorization receipt.
    MissingOperatorAuthorization,
    /// No curated dataset readiness receipts.
    MissingCuratedDatasetReceipts,
    /// No present holdout receipt (or the holdout is absent).
    MissingCleanHoldout,
    /// The holdout receipt is present but contaminated.
    HoldoutContaminated,
    /// Memorization leakage detected (or not proven clean).
    MemorizationLeakageDetected,
    /// Fewer than `MIN_RECURRING_FAILURES` recurring clean failures.
    MissingRecurringFailureEvidence,
    /// No rollback plan receipt.
    MissingRollbackPlan,
    /// No production safety plan receipt.
    MissingProductionSafetyPlan,
    /// The authority-drift check was not run, or it detected drift.
    AuthorityDriftDetected,
    /// A serialized gate report did not match its re-derivation and was refused.
    TrainingGateSerializedTamperRefused,
}

impl TrainingGateRefusal {
    /// Every refusal reason, in canonical order.
    pub const ALL: [TrainingGateRefusal; TRAIN_GATE_REFUSAL_COUNT] = [
        TrainingGateRefusal::MissingModelNeedVerdict,
        TrainingGateRefusal::VerdictNotTrainingCandidate,
        TrainingGateRefusal::MissingOperatorAuthorization,
        TrainingGateRefusal::MissingCuratedDatasetReceipts,
        TrainingGateRefusal::MissingCleanHoldout,
        TrainingGateRefusal::HoldoutContaminated,
        TrainingGateRefusal::MemorizationLeakageDetected,
        TrainingGateRefusal::MissingRecurringFailureEvidence,
        TrainingGateRefusal::MissingRollbackPlan,
        TrainingGateRefusal::MissingProductionSafetyPlan,
        TrainingGateRefusal::AuthorityDriftDetected,
        TrainingGateRefusal::TrainingGateSerializedTamperRefused,
    ];

    /// The stable slug for this refusal reason.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingGateRefusal::MissingModelNeedVerdict => "missing_model_need_verdict",
            TrainingGateRefusal::VerdictNotTrainingCandidate => "verdict_not_training_candidate",
            TrainingGateRefusal::MissingOperatorAuthorization => "missing_operator_authorization",
            TrainingGateRefusal::MissingCuratedDatasetReceipts => {
                "missing_curated_dataset_receipts"
            }
            TrainingGateRefusal::MissingCleanHoldout => "missing_clean_holdout",
            TrainingGateRefusal::HoldoutContaminated => "holdout_contaminated",
            TrainingGateRefusal::MemorizationLeakageDetected => "memorization_leakage_detected",
            TrainingGateRefusal::MissingRecurringFailureEvidence => {
                "missing_recurring_failure_evidence"
            }
            TrainingGateRefusal::MissingRollbackPlan => "missing_rollback_plan",
            TrainingGateRefusal::MissingProductionSafetyPlan => "missing_production_safety_plan",
            TrainingGateRefusal::AuthorityDriftDetected => "authority_drift_detected",
            TrainingGateRefusal::TrainingGateSerializedTamperRefused => {
                "training_gate_serialized_tamper_refused"
            }
        }
    }
}

// --- operator / pipeline receipts (gate INPUTS, never trusted off-wire) ---

/// An explicit operator authorization for a training ATTEMPT (not for training itself).
#[derive(Debug, Clone)]
pub struct OperatorAuthorizationReceipt {
    /// Who authorized the attempt.
    pub operator: String,
    /// The narrow scope of the authorized attempt.
    pub attempt_scope: String,
}

/// Evidence that a curated training dataset is ready (produced upstream by the corpus harvest).
#[derive(Debug, Clone)]
pub struct DatasetReadinessReceipt {
    /// The content hash of the curated corpus.
    pub curated_corpus_hash: String,
    /// How many curated items it contains.
    pub item_count: usize,
}

/// Evidence about the held-out evaluation split.
#[derive(Debug, Clone)]
pub struct HoldoutReadinessReceipt {
    /// Whether a genuine holdout exists.
    pub holdout_present: bool,
    /// Whether the holdout overlaps the training data.
    pub contaminated: bool,
    /// The content hash of the holdout.
    pub holdout_hash: String,
}

/// A contamination / memorization-leakage report on the prepared data.
#[derive(Debug, Clone)]
pub struct ContaminationReportReceipt {
    /// Whether memorization leakage was detected.
    pub memorization_leakage: bool,
    /// The content hash of the contamination report.
    pub report_hash: String,
}

/// A plan describing how to revert if a training attempt goes wrong.
#[derive(Debug, Clone)]
pub struct RollbackPlanReceipt {
    /// The snapshot/target the attempt can roll back to.
    pub rollback_target: String,
    /// Whether the rollback path was verified.
    pub verified: bool,
}

/// A production safety plan governing any eventual deployment of a trained model.
#[derive(Debug, Clone)]
pub struct ProductionSafetyPlanReceipt {
    /// The identifier of the safety plan.
    pub plan_id: String,
}

/// An affirmative authority-drift check. Closed by default: an UNCHECKED drift state is not clean.
#[derive(Debug, Clone, Copy)]
pub struct AuthorityDriftCheck {
    /// Whether the drift check was actually run.
    pub checked: bool,
    /// Whether drift was detected.
    pub drift_detected: bool,
}

impl AuthorityDriftCheck {
    /// A run check that found no drift.
    pub fn clean() -> Self {
        Self {
            checked: true,
            drift_detected: false,
        }
    }

    /// The default: no check has been run, so it is NOT clean.
    pub fn unchecked() -> Self {
        Self {
            checked: false,
            drift_detected: false,
        }
    }

    /// A run check that found drift.
    pub fn drifted() -> Self {
        Self {
            checked: true,
            drift_detected: true,
        }
    }

    /// Clean only if the check actually ran AND found no drift.
    pub fn is_clean(&self) -> bool {
        self.checked && !self.drift_detected
    }
}

/// The full set of inputs the gate weighs. This is an INPUT type (never `Serialize`): the gate
/// re-runs the real P11 evaluation over `eval` and re-checks every receipt. Closed by default.
#[derive(Debug)]
pub struct TrainingGateInput {
    /// The P11 model-eval battery to run (the SCORE-0 -> FAIL-0 -> MODEL-EVAL chain). `None` means
    /// no model-need verdict is available.
    pub eval: Option<ModelEvalBattery>,
    /// Explicit operator authorization for the attempt.
    pub operator_authorization: Option<OperatorAuthorizationReceipt>,
    /// Curated dataset readiness.
    pub dataset: Option<DatasetReadinessReceipt>,
    /// Holdout readiness.
    pub holdout: Option<HoldoutReadinessReceipt>,
    /// Contamination / memorization-leakage report.
    pub contamination: Option<ContaminationReportReceipt>,
    /// Rollback plan.
    pub rollback: Option<RollbackPlanReceipt>,
    /// Production safety plan.
    pub production_safety: Option<ProductionSafetyPlanReceipt>,
    /// Authority-drift check (unchecked by default).
    pub authority_drift: AuthorityDriftCheck,
}

impl TrainingGateInput {
    /// The closed-by-default input: nothing supplied, drift unchecked. Every requirement is unmet.
    pub fn closed_by_default() -> Self {
        Self {
            eval: None,
            operator_authorization: None,
            dataset: None,
            holdout: None,
            contamination: None,
            rollback: None,
            production_safety: None,
            authority_drift: AuthorityDriftCheck::unchecked(),
        }
    }
}

// --- the boundary record ---

/// The inert boundary: every forbidden action is `false`. Stamped on every report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TrainingGateBoundary {
    /// The gate never trains.
    pub trains: bool,
    /// The gate never modifies weights.
    pub modifies_weights: bool,
    /// The gate never creates truth.
    pub creates_truth: bool,
    /// The gate never creates memory.
    pub creates_memory: bool,
    /// The gate never creates evidence.
    pub creates_evidence: bool,
    /// The gate never promotes a model.
    pub promotes_model: bool,
    /// The gate never deploys a model.
    pub deploys_model: bool,
    /// The gate never grants new authority.
    pub grants_authority: bool,
}

impl TrainingGateBoundary {
    fn inert() -> Self {
        Self {
            trains: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            modifies_weights: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            creates_truth: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            creates_memory: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            creates_evidence: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            promotes_model: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            deploys_model: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
            grants_authority: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        }
    }

    /// True iff every forbidden action is inert.
    pub fn all_inert(&self) -> bool {
        !self.trains
            && !self.modifies_weights
            && !self.creates_truth
            && !self.creates_memory
            && !self.creates_evidence
            && !self.promotes_model
            && !self.deploys_model
            && !self.grants_authority
    }
}

/// The gate's verdict on whether a training attempt may be authorized. `Serialize` but never
/// `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingGateReport {
    /// The schema tag.
    pub schema: &'static str,
    /// The terminal decision.
    pub decision: TrainingGateDecision,
    /// The consumed P11 verdict (`None` if no battery was supplied).
    pub verdict: Option<ModelNeedVerdict>,
    /// The recurring clean-failure evidence count (residuals from the P11 evaluation).
    pub residual_count: usize,
    /// Which requirements were satisfied.
    pub satisfied: Vec<TrainingGateRequirement>,
    /// Why the gate refused (empty iff allowed).
    pub refusals: Vec<TrainingGateRefusal>,
    /// Always `false`: an allowed attempt does not train.
    pub trains: bool,
    /// Always `false`: an allowed attempt does not modify weights.
    pub modifies_weights: bool,
    /// Always `false`: an allowed attempt does not promote a model.
    pub promotes_model: bool,
    /// Always `false`: an allowed attempt does not deploy a model.
    pub deploys_model: bool,
    /// Always `false`: an allowed attempt does not set P12 `training_justified`.
    pub training_justified: bool,
    /// Always `false`: an allowed attempt does not open training eligibility.
    pub opens_training: bool,
    /// The inert boundary.
    pub boundary: TrainingGateBoundary,
}

/// Evaluate the closed-by-default training gate over `input`. Runs the REAL P11 model-need
/// evaluation over the supplied battery and re-checks every operator/pipeline receipt. Emits
/// `TrainingAttemptAllowed` only when every requirement holds; otherwise denies with the full set
/// of refusal reasons. Never trains, never opens training.
pub fn evaluate_training_gate(input: &TrainingGateInput) -> TrainingGateReport {
    let mut satisfied: Vec<TrainingGateRequirement> = Vec::new();
    let mut refusals: Vec<TrainingGateRefusal> = Vec::new();

    // Consume the REAL P11 model-need evaluation (SCORE-0 -> FAIL-0 -> MODEL-EVAL -> here). The
    // verdict and residual evidence are DERIVED, never hand-set.
    let (verdict, residual_count) = match &input.eval {
        None => {
            refusals.push(TrainingGateRefusal::MissingModelNeedVerdict);
            (None, 0)
        }
        Some(battery) => {
            let eval = evaluate_model_need(battery);
            satisfied.push(TrainingGateRequirement::ModelNeedVerdictPresent);
            (Some(eval.verdict), eval.residuals.len())
        }
    };

    // The verdict must be EXACTLY training_candidate_only.
    match &verdict {
        Some(ModelNeedVerdict::TrainingCandidateOnly) => {
            satisfied.push(TrainingGateRequirement::VerdictIsTrainingCandidate)
        }
        Some(_) => refusals.push(TrainingGateRefusal::VerdictNotTrainingCandidate),
        None => {}
    }

    // Recurring clean-failure evidence.
    if residual_count >= MIN_RECURRING_FAILURES {
        satisfied.push(TrainingGateRequirement::RecurringFailureEvidence);
    } else {
        refusals.push(TrainingGateRefusal::MissingRecurringFailureEvidence);
    }

    // Explicit operator authorization.
    match &input.operator_authorization {
        Some(_) => satisfied.push(TrainingGateRequirement::OperatorAuthorization),
        None => refusals.push(TrainingGateRefusal::MissingOperatorAuthorization),
    }

    // Curated dataset readiness.
    match &input.dataset {
        Some(_) => satisfied.push(TrainingGateRequirement::CuratedDataset),
        None => refusals.push(TrainingGateRefusal::MissingCuratedDatasetReceipts),
    }

    // Clean, present holdout.
    match &input.holdout {
        None => refusals.push(TrainingGateRefusal::MissingCleanHoldout),
        Some(h) if !h.holdout_present => refusals.push(TrainingGateRefusal::MissingCleanHoldout),
        Some(h) if h.contaminated => refusals.push(TrainingGateRefusal::HoldoutContaminated),
        Some(_) => satisfied.push(TrainingGateRequirement::CleanHoldout),
    }

    // Contamination / memorization clean — closed by default (absent report is NOT proven clean).
    match &input.contamination {
        None => refusals.push(TrainingGateRefusal::MemorizationLeakageDetected),
        Some(c) if c.memorization_leakage => {
            refusals.push(TrainingGateRefusal::MemorizationLeakageDetected)
        }
        Some(_) => {
            satisfied.push(TrainingGateRequirement::ContaminationClean);
            satisfied.push(TrainingGateRequirement::MemorizationClean);
        }
    }

    // Rollback plan.
    match &input.rollback {
        Some(_) => satisfied.push(TrainingGateRequirement::RollbackPlan),
        None => refusals.push(TrainingGateRefusal::MissingRollbackPlan),
    }

    // Production safety plan.
    match &input.production_safety {
        Some(_) => satisfied.push(TrainingGateRequirement::ProductionSafetyPlan),
        None => refusals.push(TrainingGateRefusal::MissingProductionSafetyPlan),
    }

    // Affirmative authority-drift check.
    if input.authority_drift.is_clean() {
        satisfied.push(TrainingGateRequirement::AuthorityDriftClean);
    } else {
        refusals.push(TrainingGateRefusal::AuthorityDriftDetected);
    }

    let decision = if refusals.is_empty() {
        TrainingGateDecision::TrainingAttemptAllowed
    } else {
        TrainingGateDecision::TrainingAttemptDenied
    };

    TrainingGateReport {
        schema: SCHEMA,
        decision,
        verdict,
        residual_count,
        satisfied,
        refusals,
        trains: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        modifies_weights: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        promotes_model: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        deploys_model: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        training_justified: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        opens_training: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING,
        boundary: TrainingGateBoundary::inert(),
    }
}

/// The gate report serialized to canonical JSON (for an operator to record an authorized attempt).
pub fn evaluate_training_gate_json(input: &TrainingGateInput) -> String {
    serde_json::to_string(&evaluate_training_gate(input)).expect("training-gate report serializes")
}

/// What can go wrong verifying a serialized training-gate artifact.
#[derive(Debug, PartialEq, Eq)]
pub enum TrainingGateError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the report from the SAME input and byte-compare against `candidate`. The report is
/// `Serialize` but never `Deserialize`: a serialized report is NOT trusted as input — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_training_gate_report_json(
    input: &TrainingGateInput,
    candidate: &str,
) -> Result<(), TrainingGateError> {
    if candidate == evaluate_training_gate_json(input) {
        Ok(())
    } else {
        Err(TrainingGateError::ReplayMismatch)
    }
}

// --- building REAL P11 batteries (SCORE-0 -> FAIL-0 -> MODEL-EVAL chain) ---

/// Produce a REAL FAIL-0 [`ModelNeedCandidate`] by running the REAL [`detect_failures`] over `n`
/// repeats of a real SCORE-0 failure observation. The gate never fabricates a verdict — the whole
/// chain (SCORE-0 verifier failure -> FAIL-0 recurrence -> P11 evaluation -> this gate) is exercised.
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

/// A real reading-misgrounding candidate (the canonical recurring clean failure).
fn reading(failures: &[FailureObservation]) -> ModelNeedCandidate {
    real_candidate(
        failures,
        FailureClass::ReadingMisgrounding,
        ScoreClass::Grounding,
        ScoreReason::Ungrounded,
        RECURRENCE_THRESHOLD,
    )
}

/// The failure persists across every comparison condition (a genuine residual signal).
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

/// A residual run: persists everywhere, with a clean trustworthy holdout.
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

/// A run whose failure is REMOVED by improving `cond` (not a genuine model gap).
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

/// A battery that yields `training_candidate_only` (two residual clean failures).
fn candidate_battery(failures: &[FailureObservation]) -> ModelEvalBattery {
    ModelEvalBattery::new(vec![
        residual_run(reading(failures)),
        residual_run(reading(failures)),
    ])
}

/// A battery that yields `collect_more_data` (a single residual — not enough on its own).
fn single_residual_battery(failures: &[FailureObservation]) -> ModelEvalBattery {
    ModelEvalBattery::new(vec![residual_run(reading(failures))])
}

/// A battery that yields `improve_substrate_first` (a substrate-removable failure).
fn substrate_battery(failures: &[FailureObservation]) -> ModelEvalBattery {
    ModelEvalBattery::new(vec![resolved_by(
        reading(failures),
        EvalCondition::SubstrateImproved,
    )])
}

/// A battery that yields `no_training_needed` (no candidates at all).
fn empty_battery() -> ModelEvalBattery {
    ModelEvalBattery::new(vec![])
}

// --- receipt builders ---

fn op_auth() -> OperatorAuthorizationReceipt {
    OperatorAuthorizationReceipt {
        operator: "operator".to_string(),
        attempt_scope: "local-finetune-attempt".to_string(),
    }
}

fn dataset() -> DatasetReadinessReceipt {
    DatasetReadinessReceipt {
        curated_corpus_hash: "curated-corpus-hash".to_string(),
        item_count: 2,
    }
}

fn clean_holdout() -> HoldoutReadinessReceipt {
    HoldoutReadinessReceipt {
        holdout_present: true,
        contaminated: false,
        holdout_hash: "holdout-hash".to_string(),
    }
}

fn contaminated_holdout() -> HoldoutReadinessReceipt {
    HoldoutReadinessReceipt {
        holdout_present: true,
        contaminated: true,
        holdout_hash: "holdout-hash".to_string(),
    }
}

fn clean_contamination() -> ContaminationReportReceipt {
    ContaminationReportReceipt {
        memorization_leakage: false,
        report_hash: "contamination-report-hash".to_string(),
    }
}

fn leaked_contamination() -> ContaminationReportReceipt {
    ContaminationReportReceipt {
        memorization_leakage: true,
        report_hash: "contamination-report-hash".to_string(),
    }
}

fn rollback() -> RollbackPlanReceipt {
    RollbackPlanReceipt {
        rollback_target: "pre-train-snapshot".to_string(),
        verified: true,
    }
}

fn prod_safety() -> ProductionSafetyPlanReceipt {
    ProductionSafetyPlanReceipt {
        plan_id: "production-safety-plan-0".to_string(),
    }
}

/// An input where EVERY requirement is satisfied (candidate verdict + all receipts + clean drift).
fn full_input(failures: &[FailureObservation]) -> TrainingGateInput {
    TrainingGateInput {
        eval: Some(candidate_battery(failures)),
        operator_authorization: Some(op_auth()),
        dataset: Some(dataset()),
        holdout: Some(clean_holdout()),
        contamination: Some(clean_contamination()),
        rollback: Some(rollback()),
        production_safety: Some(prod_safety()),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

// --- the training-gate scenario matrix (observes the real gate over constructed inputs) ---

/// One scenario cell: the OBSERVED decision of running the real gate over a constructed input.
/// Never asserted — recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingGateScenarioCell {
    /// The scenario name.
    pub name: &'static str,
    /// The observed decision slug.
    pub decision: &'static str,
    /// The observed refusal-reason slugs.
    pub refusals: Vec<&'static str>,
    /// Whether training stayed fully closed for this cell (no forbidden flag set).
    pub training_still_closed: bool,
    /// A short human-readable detail.
    pub detail: String,
}

/// The fixed training-gate scenario matrix. Every cell runs the real gate and records what it
/// observed; `training_never_opens` is the conjunction across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingGateMatrix {
    /// The schema tag.
    pub schema: &'static str,
    /// The scenario cells.
    pub scenarios: Vec<TrainingGateScenarioCell>,
    /// The two decision-state slugs.
    pub decisions: [&'static str; TRAIN_GATE_DECISION_COUNT],
    /// The twelve refusal-reason slugs.
    pub refusal_reasons: [&'static str; TRAIN_GATE_REFUSAL_COUNT],
    /// True iff no cell opened training.
    pub training_never_opens: bool,
    /// The inert boundary.
    pub boundary: TrainingGateBoundary,
}

impl TrainingGateMatrix {
    /// Find a scenario cell by name.
    pub fn scenario(&self, name: &str) -> Option<&TrainingGateScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn closed_for(report: &TrainingGateReport) -> bool {
    !report.trains
        && !report.modifies_weights
        && !report.promotes_model
        && !report.deploys_model
        && !report.training_justified
        && !report.opens_training
        && report.boundary.all_inert()
}

fn gate_cell(name: &'static str, input: TrainingGateInput) -> TrainingGateScenarioCell {
    let report = evaluate_training_gate(&input);
    TrainingGateScenarioCell {
        name,
        decision: report.decision.tag(),
        refusals: report.refusals.iter().map(|r| r.tag()).collect(),
        training_still_closed: closed_for(&report),
        detail: report.decision.tag().to_string(),
    }
}

/// The serialized-report tamper cell: tamper a real (allowed) gate report JSON and observe the
/// re-derive verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous.
fn tamper_cell(failures: &[FailureObservation]) -> TrainingGateScenarioCell {
    let input = full_input(failures);
    let canonical = evaluate_training_gate_json(&input);
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_training_gate_report_json(&input, &tampered).is_err()
        && verify_training_gate_report_json(&input, &canonical).is_ok();
    let report = evaluate_training_gate(&input);
    TrainingGateScenarioCell {
        name: "serialized_gate_report_tamper_refused",
        decision: report.decision.tag(),
        refusals: if refused {
            vec!["training_gate_serialized_tamper_refused"]
        } else {
            vec!["VACUOUS"]
        },
        training_still_closed: closed_for(&report),
        detail: if refused {
            "training_gate_serialized_tamper_refused".to_string()
        } else {
            "VACUOUS: gate report verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 19-scenario training-gate matrix from the REAL gate over constructed inputs.
pub fn training_gate_matrix() -> TrainingGateMatrix {
    // Derive the SCORE-0 failure set ONCE; every candidate reuses it (no per-build rebuild).
    let failures = verifier_score_matrix().failures;

    let scenarios = vec![
        gate_cell(
            "closed_by_default_denied",
            TrainingGateInput::closed_by_default(),
        ),
        gate_cell(
            "missing_model_need_verdict_denied",
            TrainingGateInput {
                eval: None,
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "no_training_needed_denied",
            TrainingGateInput {
                eval: Some(empty_battery()),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "improve_substrate_first_denied",
            TrainingGateInput {
                eval: Some(substrate_battery(&failures)),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "collect_more_data_denied",
            TrainingGateInput {
                eval: Some(single_residual_battery(&failures)),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "training_candidate_without_operator_auth_denied",
            TrainingGateInput {
                operator_authorization: None,
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "training_candidate_without_dataset_denied",
            TrainingGateInput {
                dataset: None,
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "training_candidate_without_holdout_denied",
            TrainingGateInput {
                holdout: None,
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "holdout_contaminated_denied",
            TrainingGateInput {
                holdout: Some(contaminated_holdout()),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "memorization_leakage_denied",
            TrainingGateInput {
                contamination: Some(leaked_contamination()),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "missing_recurring_failure_evidence_denied",
            TrainingGateInput {
                eval: Some(single_residual_battery(&failures)),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "missing_rollback_plan_denied",
            TrainingGateInput {
                rollback: None,
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "missing_production_safety_plan_denied",
            TrainingGateInput {
                production_safety: None,
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "authority_drift_denied",
            TrainingGateInput {
                authority_drift: AuthorityDriftCheck::drifted(),
                ..full_input(&failures)
            },
        ),
        gate_cell(
            "all_requirements_met_training_attempt_allowed",
            full_input(&failures),
        ),
        gate_cell("allowed_is_not_training_execution", full_input(&failures)),
        gate_cell("allowed_is_not_model_promotion", full_input(&failures)),
        tamper_cell(&failures),
        gate_cell("training_justified_remains_false", full_input(&failures)),
    ];

    let training_never_opens = scenarios.iter().all(|c| c.training_still_closed);
    TrainingGateMatrix {
        schema: SCHEMA,
        scenarios,
        decisions: TRAIN_GATE_DECISION_NAMES,
        refusal_reasons: TRAIN_GATE_REFUSAL_NAMES,
        training_never_opens,
        boundary: TrainingGateBoundary::inert(),
    }
}

/// The training-gate matrix serialized to canonical JSON.
pub fn training_gate_matrix_json() -> String {
    serde_json::to_string(&training_gate_matrix()).expect("training-gate matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`.
pub fn verify_training_gate_matrix_json(candidate: &str) -> Result<(), TrainingGateError> {
    if candidate == training_gate_matrix_json() {
        Ok(())
    } else {
        Err(TrainingGateError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failures() -> Vec<FailureObservation> {
        verifier_score_matrix().failures
    }

    fn has(report: &TrainingGateReport, r: TrainingGateRefusal) -> bool {
        report.refusals.contains(&r)
    }

    #[test]
    fn gate_consumes_the_real_p11_verdict() {
        let f = failures();
        // A candidate battery -> the gate observes the REAL training_candidate_only verdict.
        let allowed = evaluate_training_gate(&full_input(&f));
        assert_eq!(
            allowed.verdict,
            Some(ModelNeedVerdict::TrainingCandidateOnly)
        );
        // A substrate-removable battery -> the gate observes improve_substrate_first (not hard-coded).
        let substrate = evaluate_training_gate(&TrainingGateInput {
            eval: Some(substrate_battery(&f)),
            ..full_input(&f)
        });
        assert_eq!(
            substrate.verdict,
            Some(ModelNeedVerdict::ImproveSubstrateFirst)
        );
        assert_eq!(
            substrate.decision,
            TrainingGateDecision::TrainingAttemptDenied
        );
    }

    #[test]
    fn closed_by_default_denies_with_no_inputs() {
        let report = evaluate_training_gate(&TrainingGateInput::closed_by_default());
        assert_eq!(report.decision, TrainingGateDecision::TrainingAttemptDenied);
        assert!(has(&report, TrainingGateRefusal::MissingModelNeedVerdict));
        assert!(has(
            &report,
            TrainingGateRefusal::MissingOperatorAuthorization
        ));
        assert!(has(&report, TrainingGateRefusal::MissingRollbackPlan));
        assert!(has(&report, TrainingGateRefusal::AuthorityDriftDetected));
        // closed by default: nothing satisfied.
        assert!(report.satisfied.is_empty());
    }

    #[test]
    fn training_candidate_alone_is_insufficient() {
        let f = failures();
        // The verdict is training_candidate_only, but NO receipts are supplied.
        let report = evaluate_training_gate(&TrainingGateInput {
            eval: Some(candidate_battery(&f)),
            ..TrainingGateInput::closed_by_default()
        });
        assert_eq!(
            report.verdict,
            Some(ModelNeedVerdict::TrainingCandidateOnly)
        );
        assert_eq!(report.decision, TrainingGateDecision::TrainingAttemptDenied);
        assert!(has(
            &report,
            TrainingGateRefusal::MissingOperatorAuthorization
        ));
        assert!(has(
            &report,
            TrainingGateRefusal::MissingCuratedDatasetReceipts
        ));
        assert!(has(&report, TrainingGateRefusal::MissingCleanHoldout));
        assert!(has(&report, TrainingGateRefusal::MissingRollbackPlan));
        assert!(has(
            &report,
            TrainingGateRefusal::MissingProductionSafetyPlan
        ));
    }

    #[test]
    fn all_requirements_met_allows_a_training_attempt() {
        let f = failures();
        let report = evaluate_training_gate(&full_input(&f));
        assert_eq!(
            report.decision,
            TrainingGateDecision::TrainingAttemptAllowed
        );
        assert!(report.refusals.is_empty());
        assert_eq!(report.satisfied.len(), TrainingGateRequirement::ALL.len());
    }

    #[test]
    fn allowed_attempt_is_not_training_authorization() {
        let f = failures();
        let report = evaluate_training_gate(&full_input(&f));
        assert_eq!(
            report.decision,
            TrainingGateDecision::TrainingAttemptAllowed
        );
        // An allowed ATTEMPT trains nothing, touches no weights, promotes/deploys nothing.
        assert!(!report.trains);
        assert!(!report.modifies_weights);
        assert!(!report.promotes_model);
        assert!(!report.deploys_model);
        assert!(!report.training_justified);
        assert!(!report.opens_training);
        assert!(report.boundary.all_inert());
        // The deeper P12 gate stays closed regardless of this gate's decision.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn contaminated_holdout_and_leakage_are_denied() {
        let f = failures();
        let contaminated = evaluate_training_gate(&TrainingGateInput {
            holdout: Some(contaminated_holdout()),
            ..full_input(&f)
        });
        assert_eq!(
            contaminated.decision,
            TrainingGateDecision::TrainingAttemptDenied
        );
        assert!(has(&contaminated, TrainingGateRefusal::HoldoutContaminated));

        let leaked = evaluate_training_gate(&TrainingGateInput {
            contamination: Some(leaked_contamination()),
            ..full_input(&f)
        });
        assert_eq!(leaked.decision, TrainingGateDecision::TrainingAttemptDenied);
        assert!(has(
            &leaked,
            TrainingGateRefusal::MemorizationLeakageDetected
        ));
    }

    #[test]
    fn matrix_has_the_nineteen_named_scenarios() {
        let matrix = training_gate_matrix();
        assert_eq!(matrix.scenarios.len(), TRAIN_GATE_SCENARIO_COUNT);
        for name in [
            "closed_by_default_denied",
            "missing_model_need_verdict_denied",
            "no_training_needed_denied",
            "improve_substrate_first_denied",
            "collect_more_data_denied",
            "training_candidate_without_operator_auth_denied",
            "training_candidate_without_dataset_denied",
            "training_candidate_without_holdout_denied",
            "holdout_contaminated_denied",
            "memorization_leakage_denied",
            "missing_recurring_failure_evidence_denied",
            "missing_rollback_plan_denied",
            "missing_production_safety_plan_denied",
            "authority_drift_denied",
            "all_requirements_met_training_attempt_allowed",
            "allowed_is_not_training_execution",
            "allowed_is_not_model_promotion",
            "serialized_gate_report_tamper_refused",
            "training_justified_remains_false",
        ] {
            assert!(
                matrix.scenario(name).is_some(),
                "scenario {name} is missing"
            );
        }
        // Exactly one cell is allowed-shaped per its name; the gate stays closed everywhere.
        assert!(matrix.training_never_opens);
    }

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let f = failures();
        let input = full_input(&f);
        let canonical = evaluate_training_gate_json(&input);
        // Deterministic.
        assert_eq!(canonical, evaluate_training_gate_json(&full_input(&f)));
        // Canonical verifies; a tampered (non-equal) artifact is refused.
        assert!(verify_training_gate_report_json(&input, &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_training_gate_report_json(&input, &tampered),
            Err(TrainingGateError::ReplayMismatch)
        );
    }

    #[test]
    fn missing_model_need_verdict_is_denied() {
        let f = failures();
        let report = evaluate_training_gate(&TrainingGateInput {
            eval: None,
            ..full_input(&f)
        });
        assert_eq!(report.verdict, None);
        assert_eq!(report.decision, TrainingGateDecision::TrainingAttemptDenied);
        assert!(has(&report, TrainingGateRefusal::MissingModelNeedVerdict));
    }

    #[test]
    fn no_training_needed_verdict_is_denied() {
        let f = failures();
        let report = evaluate_training_gate(&TrainingGateInput {
            eval: Some(empty_battery()),
            ..full_input(&f)
        });
        assert_eq!(report.verdict, Some(ModelNeedVerdict::NoTrainingNeeded));
        assert!(has(
            &report,
            TrainingGateRefusal::VerdictNotTrainingCandidate
        ));
    }

    #[test]
    fn improve_substrate_first_verdict_is_denied() {
        let f = failures();
        let report = evaluate_training_gate(&TrainingGateInput {
            eval: Some(substrate_battery(&f)),
            ..full_input(&f)
        });
        assert_eq!(
            report.verdict,
            Some(ModelNeedVerdict::ImproveSubstrateFirst)
        );
        assert!(has(
            &report,
            TrainingGateRefusal::VerdictNotTrainingCandidate
        ));
    }

    #[test]
    fn collect_more_data_verdict_is_denied() {
        let f = failures();
        let report = evaluate_training_gate(&TrainingGateInput {
            eval: Some(single_residual_battery(&f)),
            ..full_input(&f)
        });
        assert_eq!(report.verdict, Some(ModelNeedVerdict::CollectMoreData));
        assert!(has(
            &report,
            TrainingGateRefusal::VerdictNotTrainingCandidate
        ));
    }

    #[test]
    fn missing_recurring_failure_evidence_is_denied() {
        let f = failures();
        // A single residual -> CollectMoreData verdict AND insufficient recurring evidence.
        let report = evaluate_training_gate(&TrainingGateInput {
            eval: Some(single_residual_battery(&f)),
            ..full_input(&f)
        });
        assert_eq!(report.residual_count, 1);
        assert!(has(
            &report,
            TrainingGateRefusal::MissingRecurringFailureEvidence
        ));
    }

    #[test]
    fn missing_each_receipt_is_denied() {
        let f = failures();
        let op = evaluate_training_gate(&TrainingGateInput {
            operator_authorization: None,
            ..full_input(&f)
        });
        assert!(has(&op, TrainingGateRefusal::MissingOperatorAuthorization));

        let ds = evaluate_training_gate(&TrainingGateInput {
            dataset: None,
            ..full_input(&f)
        });
        assert!(has(&ds, TrainingGateRefusal::MissingCuratedDatasetReceipts));

        let ho = evaluate_training_gate(&TrainingGateInput {
            holdout: None,
            ..full_input(&f)
        });
        assert!(has(&ho, TrainingGateRefusal::MissingCleanHoldout));

        let rb = evaluate_training_gate(&TrainingGateInput {
            rollback: None,
            ..full_input(&f)
        });
        assert!(has(&rb, TrainingGateRefusal::MissingRollbackPlan));

        let ps = evaluate_training_gate(&TrainingGateInput {
            production_safety: None,
            ..full_input(&f)
        });
        assert!(has(&ps, TrainingGateRefusal::MissingProductionSafetyPlan));
    }

    #[test]
    fn authority_drift_is_denied() {
        let f = failures();
        let drifted = evaluate_training_gate(&TrainingGateInput {
            authority_drift: AuthorityDriftCheck::drifted(),
            ..full_input(&f)
        });
        assert!(has(&drifted, TrainingGateRefusal::AuthorityDriftDetected));
        // An unchecked drift state is ALSO not clean (closed by default).
        let unchecked = evaluate_training_gate(&TrainingGateInput {
            authority_drift: AuthorityDriftCheck::unchecked(),
            ..full_input(&f)
        });
        assert!(has(&unchecked, TrainingGateRefusal::AuthorityDriftDetected));
    }

    #[test]
    fn allowed_does_not_promote_or_deploy() {
        let f = failures();
        let report = evaluate_training_gate(&full_input(&f));
        assert_eq!(
            report.decision,
            TrainingGateDecision::TrainingAttemptAllowed
        );
        assert!(!report.promotes_model);
        assert!(!report.deploys_model);
        assert!(!report.boundary.promotes_model);
        assert!(!report.boundary.deploys_model);
    }

    #[test]
    fn training_justified_remains_false_even_when_allowed() {
        let f = failures();
        let report = evaluate_training_gate(&full_input(&f));
        assert!(!report.training_justified);
        // The real P12 gate is unaffected by an allowed attempt.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn every_matrix_cell_keeps_training_closed() {
        let matrix = training_gate_matrix();
        for cell in &matrix.scenarios {
            assert!(
                cell.training_still_closed,
                "cell {} opened training",
                cell.name
            );
        }
        // The tamper cell genuinely refused (its slug is recorded, not VACUOUS).
        let tamper = matrix
            .scenario("serialized_gate_report_tamper_refused")
            .expect("tamper cell present");
        assert!(tamper
            .refusals
            .contains(&"training_gate_serialized_tamper_refused"));
    }

    #[test]
    fn decision_and_refusal_counts_match_enums() {
        assert_eq!(TrainingGateDecision::ALL.len(), TRAIN_GATE_DECISION_COUNT);
        assert_eq!(TrainingGateRefusal::ALL.len(), TRAIN_GATE_REFUSAL_COUNT);
        assert_eq!(TRAIN_GATE_DECISION_NAMES.len(), TRAIN_GATE_DECISION_COUNT);
        assert_eq!(TRAIN_GATE_REFUSAL_NAMES.len(), TRAIN_GATE_REFUSAL_COUNT);
        // Every decision/refusal slug array entry matches its enum tag, in order.
        for (d, name) in TrainingGateDecision::ALL
            .iter()
            .zip(TRAIN_GATE_DECISION_NAMES)
        {
            assert_eq!(d.tag(), name);
        }
        for (r, name) in TrainingGateRefusal::ALL
            .iter()
            .zip(TRAIN_GATE_REFUSAL_NAMES)
        {
            assert_eq!(r.tag(), name);
        }
    }

    #[test]
    fn matrix_re_derives_refusing_tampering() {
        let canonical = training_gate_matrix_json();
        assert!(verify_training_gate_matrix_json(&canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_training_gate_matrix_json(&tampered),
            Err(TrainingGateError::ReplayMismatch)
        );
    }
}
