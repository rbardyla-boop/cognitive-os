//! failure_detector — FAIL-0, the recurring-clean-failure detector.
//!
//! It answers ONE narrow question: did the system observe the SAME clean failure enough
//! times to become a [`ModelNeedCandidate`]? It does NOT answer "should we train?" — a
//! candidate is a flag for the later P11 model-need eval, NEVER training authorization.
//!
//! The detector CONSUMES SCORE-0 [`crate::FailureObservation`] values — it cannot fabricate
//! one (SCORE-0's constructor is private), so every signal's evidence is a real verifier
//! failure pulled from [`crate::verifier_score_matrix`]. For each signal it decides, from the
//! verified surrounding [`FailureContext`], whether the failure is:
//!   - EXCLUDED — missing context / bad retrieval / uncurated data / bad prompt-or-schema /
//!     invalid test / stale artifact / unverified replay / quarantined candidate (8 reasons),
//!   - SUBSTRATE — a replay / trace-integrity failure (fix the substrate, NOT a model need), or
//!   - a CLEAN MODEL failure — curation passed (or a valid refusal context) AND replay/integrity
//!     verified AND no exclusion applies.
//! Clean model failures are grouped by class; a [`ModelNeedCandidate`] is emitted ONLY when the
//! clean occurrences reach the explicit, deterministic [`RECURRENCE_THRESHOLD`] AND the class +
//! SCORE-0 reason are stable across the repeats. A single failure can never emit a candidate.
//!
//! What the detector CANNOT do: it creates no truth / memory / evidence, promotes nothing,
//! grants no authority, executes nothing, and CANNOT open training. A [`ModelNeedCandidate`]
//! is structurally `training_justified=false` / `opens_training=false` /
//! `authorizes_training=false` (all sourced from the const [`MODEL_NEED_IS_TRAINING_AUTHORIZATION`]).
//!
//! The boundary, recorded verbatim in [`FAIL_BOUNDARY_LINES`]:
//!
//!   The failure detector observes recurring clean failures.
//!   It does not create truth.
//!   It does not create memory.
//!   It does not create evidence.
//!   It does not train.
//!   It does not execute external actions.
//!   It does not promote hypotheses.
//!   It does not grant new authority.
//!   ModelNeedCandidate is not training authorization.
//!
//! Determinism: classes are iterated in fixed order; no clock / entropy / float / IO. Reports
//! derive `Serialize` but NOT `Deserialize` — a serialized report is never trusted as input; it
//! is re-derived and byte-compared ([`verify_failure_report_json`] /
//! [`verify_failure_detector_matrix_json`]), so any tampering is refused.

use crate::{verifier_score_matrix, FailureObservation, ScoreClass, ScoreReason};
use serde::Serialize;

const SCHEMA: &str = "failure-detector-v0.1";

/// The number of model-failure classes the detector recognizes. Pinned by the gate.
pub const FAILURE_CLASS_COUNT: usize = 10;

/// The number of observed scenario cells in [`failure_detector_matrix`]. `training_never_opens`
/// is the matrix-level conjunction (the 17th rubric line), not a cell.
pub const FAILURE_SCENARIO_COUNT: usize = 16;

/// The explicit, deterministic recurrence threshold: a clean failure must be observed at least
/// this many times (with a stable class + reason) before it can become a [`ModelNeedCandidate`].
/// A single failure (< threshold) never emits a candidate.
pub const RECURRENCE_THRESHOLD: usize = 2;

/// A [`ModelNeedCandidate`] is a flag for further EVAL, NEVER training authorization. This
/// structural `const` is the single source of its `training_justified` / `opens_training` /
/// `authorizes_training` flags — there is no code path that constructs any of them `true`.
const MODEL_NEED_IS_TRAINING_AUTHORIZATION: bool = false;

/// The FAIL-0 boundary, recorded verbatim and pinned by the release gate.
pub const FAIL_BOUNDARY_LINES: [&str; 9] = [
    "The failure detector observes recurring clean failures.",
    "It does not create truth.",
    "It does not create memory.",
    "It does not create evidence.",
    "It does not train.",
    "It does not execute external actions.",
    "It does not promote hypotheses.",
    "It does not grant new authority.",
    "ModelNeedCandidate is not training authorization.",
];

// --- the ten model-failure classes ---

/// A model-failure taxonomy slot. Most classes are model-attributable (a clean, recurring one
/// can become a candidate); [`FailureClass::ReplayInconsistency`] is a SUBSTRATE/TRACE class
/// that can NEVER be a model need (a replay/integrity failure is a trace failure to fix, not a
/// reason to train).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FailureClass {
    ReadingMisgrounding,
    SourceSelectionFailure,
    MultiDocSynthesisFailure,
    HorizonPlanFailure,
    ToolUseSchemaFailure,
    RefusalBoundaryFailure,
    MemoryRetrievalFailure,
    InstructionFollowingFailure,
    CodingPatchFailure,
    ReplayInconsistency,
}

impl FailureClass {
    /// The ten classes in canonical order.
    pub const ALL: [FailureClass; FAILURE_CLASS_COUNT] = [
        FailureClass::ReadingMisgrounding,
        FailureClass::SourceSelectionFailure,
        FailureClass::MultiDocSynthesisFailure,
        FailureClass::HorizonPlanFailure,
        FailureClass::ToolUseSchemaFailure,
        FailureClass::RefusalBoundaryFailure,
        FailureClass::MemoryRetrievalFailure,
        FailureClass::InstructionFollowingFailure,
        FailureClass::CodingPatchFailure,
        FailureClass::ReplayInconsistency,
    ];

    /// The stable, snake_case class name pinned by the release gate.
    pub fn tag(self) -> &'static str {
        match self {
            FailureClass::ReadingMisgrounding => "reading_misgrounding",
            FailureClass::SourceSelectionFailure => "source_selection_failure",
            FailureClass::MultiDocSynthesisFailure => "multi_doc_synthesis_failure",
            FailureClass::HorizonPlanFailure => "horizon_plan_failure",
            FailureClass::ToolUseSchemaFailure => "tool_use_schema_failure",
            FailureClass::RefusalBoundaryFailure => "refusal_boundary_failure",
            FailureClass::MemoryRetrievalFailure => "memory_retrieval_failure",
            FailureClass::InstructionFollowingFailure => "instruction_following_failure",
            FailureClass::CodingPatchFailure => "coding_patch_failure",
            FailureClass::ReplayInconsistency => "replay_inconsistency",
        }
    }

    /// Whether this class is inherently a SUBSTRATE/TRACE failure (a replay/integrity failure) —
    /// such a failure is fixed in the substrate and can never be a model need.
    fn is_substrate_class(self) -> bool {
        matches!(self, FailureClass::ReplayInconsistency)
    }
}

/// The ten class names in canonical order — pinned, in source, by the release gate.
pub const FAILURE_CLASS_NAMES: [&str; FAILURE_CLASS_COUNT] = [
    "reading_misgrounding",
    "source_selection_failure",
    "multi_doc_synthesis_failure",
    "horizon_plan_failure",
    "tool_use_schema_failure",
    "refusal_boundary_failure",
    "memory_retrieval_failure",
    "instruction_following_failure",
    "coding_patch_failure",
    "replay_inconsistency",
];

/// Why a failure is NOT a clean model failure — the eight exclusion reasons. Each excludes the
/// failure from recurrence counting (it is a substrate / data / harness problem, not a model need).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FailureExclusion {
    MissingContext,
    BadRetrieval,
    UncuratedData,
    BadPromptSchema,
    InvalidTest,
    StaleArtifact,
    UnverifiedReplay,
    QuarantinedCandidate,
}

impl FailureExclusion {
    pub fn label(self) -> &'static str {
        match self {
            FailureExclusion::MissingContext => "missing_context",
            FailureExclusion::BadRetrieval => "bad_retrieval",
            FailureExclusion::UncuratedData => "uncurated_data",
            FailureExclusion::BadPromptSchema => "bad_prompt_schema",
            FailureExclusion::InvalidTest => "invalid_test",
            FailureExclusion::StaleArtifact => "stale_artifact",
            FailureExclusion::UnverifiedReplay => "unverified_replay",
            FailureExclusion::QuarantinedCandidate => "quarantined_candidate",
        }
    }
}

/// The high-level attribution of a failure: a clean model failure (model-attributable, clean
/// inputs), a substrate failure (fix the substrate, not a model need), or excluded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FailureCause {
    CleanModel,
    Substrate,
    Excluded,
}

/// The per-observation cleanliness verdict: a clean model failure, a substrate failure, or
/// excluded (carrying which exclusion applied).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CleanFailureStatus {
    Clean,
    Substrate,
    Excluded(FailureExclusion),
}

/// The verified surrounding context of a failure observation — the signals that decide whether
/// it is a CLEAN model failure or an excluded / substrate one. Every field is a verified fact
/// about the run that produced the SCORE-0 observation, not a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FailureContext {
    /// The candidate data passed the DATA-0 curator (clean data).
    pub curation_passed: bool,
    /// The replay / integrity verification was performed and is trustworthy.
    pub replay_verified: bool,
    /// The required context was present (not a missing-context failure).
    pub context_present: bool,
    /// Retrieval was valid (not a bad-retrieval / infra failure).
    pub retrieval_valid: bool,
    /// The prompt / tool / schema was valid (not a harness failure).
    pub prompt_schema_valid: bool,
    /// The test / eval was valid (not an invalid-test failure).
    pub test_valid: bool,
    /// The artifact was fresh (not stale).
    pub artifact_fresh: bool,
    /// The candidate was quarantined (held) — quarantine excludes it from model-need counting.
    pub quarantined: bool,
    /// For refusal failures, the refusal context is valid (an alternative to curation_passed).
    pub refusal_context_valid: bool,
}

impl FailureContext {
    /// A fully clean context: every gate passes, nothing quarantined.
    pub fn clean() -> Self {
        Self {
            curation_passed: true,
            replay_verified: true,
            context_present: true,
            retrieval_valid: true,
            prompt_schema_valid: true,
            test_valid: true,
            artifact_fresh: true,
            quarantined: false,
            refusal_context_valid: true,
        }
    }
}

/// Decide a single observation's cleanliness from its class + context. Exclusions are checked
/// first, in a fixed deterministic precedence; then substrate classes; then a clean model failure.
fn clean_status(class: FailureClass, ctx: &FailureContext) -> CleanFailureStatus {
    if ctx.quarantined {
        return CleanFailureStatus::Excluded(FailureExclusion::QuarantinedCandidate);
    }
    if !ctx.context_present {
        return CleanFailureStatus::Excluded(FailureExclusion::MissingContext);
    }
    if !ctx.retrieval_valid {
        return CleanFailureStatus::Excluded(FailureExclusion::BadRetrieval);
    }
    if !ctx.prompt_schema_valid {
        return CleanFailureStatus::Excluded(FailureExclusion::BadPromptSchema);
    }
    if !ctx.test_valid {
        return CleanFailureStatus::Excluded(FailureExclusion::InvalidTest);
    }
    if !ctx.artifact_fresh {
        return CleanFailureStatus::Excluded(FailureExclusion::StaleArtifact);
    }
    if !ctx.replay_verified {
        return CleanFailureStatus::Excluded(FailureExclusion::UnverifiedReplay);
    }
    // Clean data requirement: curation passed OR (for refusal failures) a valid refusal context.
    if !(ctx.curation_passed || ctx.refusal_context_valid) {
        return CleanFailureStatus::Excluded(FailureExclusion::UncuratedData);
    }
    if class.is_substrate_class() {
        return CleanFailureStatus::Substrate;
    }
    CleanFailureStatus::Clean
}

// --- input ---

/// One failure signal offered to the detector: a model-failure class, the REAL SCORE-0
/// [`FailureObservation`] that evidences it, and the verified [`FailureContext`]. This is the
/// ONLY input — the detector never invents a failure; it groups and classifies the signals it
/// is given, where each signal's evidence is a real verifier failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureSignal {
    pub class: FailureClass,
    pub observation: FailureObservation,
    pub context: FailureContext,
}

impl FailureSignal {
    pub fn new(
        class: FailureClass,
        observation: FailureObservation,
        context: FailureContext,
    ) -> Self {
        Self {
            class,
            observation,
            context,
        }
    }
}

// --- output records (Serialize, never Deserialize) ---

/// The explicit, deterministic recurrence policy. Recorded in every report so the threshold is
/// auditable and cannot drift silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FailureRecurrencePolicy {
    pub threshold: usize,
}

impl FailureRecurrencePolicy {
    fn standard() -> Self {
        Self {
            threshold: RECURRENCE_THRESHOLD,
        }
    }
}

/// A model-need candidate: a recurring, clean, model-attributable failure. It is a flag for the
/// later P11 eval, NOT training authorization. `training_justified` / `opens_training` /
/// `authorizes_training` are all structurally `false`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelNeedCandidate {
    pub class: FailureClass,
    pub reason: String,
    pub recurrences: usize,
    pub source_hashes: Vec<String>,
    /// Always `false`: a candidate is not training justification.
    pub training_justified: bool,
    /// Always `false`: a candidate cannot open training eligibility.
    pub opens_training: bool,
    /// Always `false`: a candidate does not authorize training.
    pub authorizes_training: bool,
}

impl ModelNeedCandidate {
    fn new(
        class: FailureClass,
        reason: String,
        recurrences: usize,
        source_hashes: Vec<String>,
    ) -> Self {
        Self {
            class,
            reason,
            recurrences,
            source_hashes,
            training_justified: MODEL_NEED_IS_TRAINING_AUTHORIZATION,
            opens_training: MODEL_NEED_IS_TRAINING_AUTHORIZATION,
            authorizes_training: MODEL_NEED_IS_TRAINING_AUTHORIZATION,
        }
    }
}

/// One detected failure case: a class group, its representative SCORE-0 reason, how many CLEAN
/// occurrences it has, the cause, whether the clean occurrences are class+reason stable, the
/// provenance hashes, and whether it emits a candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FailureCase {
    pub class: FailureClass,
    pub reason: String,
    pub total_signals: usize,
    pub clean_occurrences: usize,
    pub cause: FailureCause,
    pub clean_status: CleanFailureStatus,
    pub stable: bool,
    pub source_hashes: Vec<String>,
    pub emits_candidate: bool,
}

/// The inert invariants every detector run upholds — every field `false` by construction. The
/// detector has NO code that could set any true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FailureDetectorBoundary {
    pub created_truth: bool,
    pub created_memory: bool,
    pub created_evidence: bool,
    pub promoted_hypothesis: bool,
    pub granted_authority: bool,
    pub executed_external: bool,
    pub opened_training: bool,
    pub authorizes_training: bool,
}

impl FailureDetectorBoundary {
    fn inert() -> Self {
        Self {
            created_truth: false,
            created_memory: false,
            created_evidence: false,
            promoted_hypothesis: false,
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
            && !self.promoted_hypothesis
            && !self.granted_authority
            && !self.executed_external
            && !self.opened_training
            && !self.authorizes_training
    }
}

/// The complete, deterministic output of a detector run: the per-class cases, the emitted
/// model-need candidates, the recurrence policy, the always-`false` training flags, and the
/// inert boundary. `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FailureDetectorReport {
    pub schema: &'static str,
    pub cases: Vec<FailureCase>,
    pub candidates: Vec<ModelNeedCandidate>,
    pub policy: FailureRecurrencePolicy,
    /// Always `false`: the detector cannot justify training.
    pub training_justified: bool,
    /// Always `false`: the detector cannot open training eligibility.
    pub opens_training: bool,
    pub boundary: FailureDetectorBoundary,
}

// --- the detector ---

/// Detect recurring clean failures from `signals`. Groups signals by class (in canonical order),
/// classifies each via [`clean_status`], counts CLEAN-model occurrences, and emits a
/// [`ModelNeedCandidate`] ONLY when the clean occurrences reach [`RECURRENCE_THRESHOLD`] AND the
/// class + SCORE-0 reason are stable across them. Excluded and substrate signals are recorded but
/// never counted toward recurrence; a single failure never emits a candidate.
pub fn detect_failures(signals: &[FailureSignal]) -> FailureDetectorReport {
    let policy = FailureRecurrencePolicy::standard();
    let mut cases: Vec<FailureCase> = Vec::new();
    let mut candidates: Vec<ModelNeedCandidate> = Vec::new();

    for class in FailureClass::ALL {
        let group: Vec<&FailureSignal> = signals.iter().filter(|s| s.class == class).collect();
        if group.is_empty() {
            continue;
        }
        let statuses: Vec<CleanFailureStatus> = group
            .iter()
            .map(|s| clean_status(class, &s.context))
            .collect();

        // CLEAN-model occurrences only count toward recurrence.
        let clean_idx: Vec<usize> = (0..group.len())
            .filter(|&i| statuses[i] == CleanFailureStatus::Clean)
            .collect();
        let clean_reasons: Vec<&str> = clean_idx
            .iter()
            .map(|&i| group[i].observation.reason.label())
            .collect();
        let clean_occurrences = clean_idx.len();
        let stable =
            !clean_reasons.is_empty() && clean_reasons.iter().all(|r| *r == clean_reasons[0]);

        let cause = if clean_occurrences > 0 {
            FailureCause::CleanModel
        } else if class.is_substrate_class() || statuses.contains(&CleanFailureStatus::Substrate) {
            FailureCause::Substrate
        } else {
            FailureCause::Excluded
        };

        let source_hashes: Vec<String> = clean_idx
            .iter()
            .map(|&i| group[i].observation.source_hash.clone())
            .collect();
        let reason = group[0].observation.reason.label().to_string();
        let emits =
            clean_occurrences >= policy.threshold && stable && cause == FailureCause::CleanModel;

        if emits {
            candidates.push(ModelNeedCandidate::new(
                class,
                reason.clone(),
                clean_occurrences,
                source_hashes.clone(),
            ));
        }
        cases.push(FailureCase {
            class,
            reason,
            total_signals: group.len(),
            clean_occurrences,
            cause,
            clean_status: statuses[0],
            stable,
            source_hashes,
            emits_candidate: emits,
        });
    }

    FailureDetectorReport {
        schema: SCHEMA,
        cases,
        candidates,
        policy,
        training_justified: MODEL_NEED_IS_TRAINING_AUTHORIZATION,
        opens_training: MODEL_NEED_IS_TRAINING_AUTHORIZATION,
        boundary: FailureDetectorBoundary::inert(),
    }
}

/// The detector report serialized to canonical JSON (for an operator gate to emit).
pub fn detect_failures_json(signals: &[FailureSignal]) -> String {
    serde_json::to_string(&detect_failures(signals)).expect("failure report serializes")
}

/// What can go wrong verifying a serialized failure artifact.
#[derive(Debug, PartialEq, Eq)]
pub enum FailureDetectorError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the report from the SAME signals and byte-compare against `candidate`. The report is
/// `Serialize` but never `Deserialize`: a serialized report is NOT trusted as input — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_failure_report_json(
    signals: &[FailureSignal],
    candidate: &str,
) -> Result<(), FailureDetectorError> {
    if candidate == detect_failures_json(signals) {
        Ok(())
    } else {
        Err(FailureDetectorError::ReplayMismatch)
    }
}

// --- consuming REAL SCORE-0 observations ---

/// Find the REAL SCORE-0 [`FailureObservation`] for a (class, reason) in an already-derived
/// SCORE-0 failure set. The detector cannot fabricate an observation (SCORE-0's constructor is
/// private), so every signal's evidence is genuinely a SCORE-0 verifier failure.
fn find_obs(
    failures: &[FailureObservation],
    class: ScoreClass,
    reason: ScoreReason,
) -> FailureObservation {
    failures
        .iter()
        .find(|f| f.class == class && f.reason == reason)
        .cloned()
        .expect("the SCORE-0 matrix yields the expected failure observation")
}

/// Pull the REAL SCORE-0 [`FailureObservation`] for a (class, reason), deriving the SCORE-0
/// failure set fresh from [`verifier_score_matrix`]. Used by single-lookup callers; the matrix
/// derives the SCORE-0 set once and uses [`find_obs`] to avoid rebuilding it per lookup.
fn score_obs(class: ScoreClass, reason: ScoreReason) -> FailureObservation {
    find_obs(&verifier_score_matrix().failures, class, reason)
}

fn signal(class: FailureClass, obs: FailureObservation, ctx: FailureContext) -> FailureSignal {
    FailureSignal::new(class, obs, ctx)
}

// --- the detector scenario matrix (observes the real detector over real SCORE-0 failures) ---

/// One scenario cell: the OBSERVED outcome of running the real detector over a constructed signal
/// list whose evidence is REAL SCORE-0 failures. Never asserted — recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FailureScenarioCell {
    pub name: &'static str,
    pub class: FailureClass,
    pub signal_count: usize,
    pub clean_occurrences: usize,
    pub cause: FailureCause,
    pub emits_candidate: bool,
    pub opens_training: bool,
    pub detail: String,
}

/// The fixed detector scenario matrix. Every cell runs the real detector and records what it
/// observed; `training_never_opens` is the conjunction across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FailureDetectorMatrix {
    pub schema: &'static str,
    pub scenarios: Vec<FailureScenarioCell>,
    pub classes: [&'static str; FAILURE_CLASS_COUNT],
    pub recurrence_threshold: usize,
    pub training_never_opens: bool,
    pub boundary: FailureDetectorBoundary,
}

impl FailureDetectorMatrix {
    pub fn scenario(&self, name: &str) -> Option<&FailureScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

/// Run the real detector over `signals` for `class` and record the OBSERVED cell.
fn cell(
    name: &'static str,
    class: FailureClass,
    signals: Vec<FailureSignal>,
) -> FailureScenarioCell {
    let report = detect_failures(&signals);
    let case = report.cases.iter().find(|c| c.class == class);
    let emits = report.candidates.iter().any(|c| c.class == class);
    let (clean_occurrences, cause, detail) = match case {
        Some(c) => (
            c.clean_occurrences,
            c.cause,
            match c.clean_status {
                CleanFailureStatus::Excluded(x) => x.label().to_string(),
                CleanFailureStatus::Substrate => "substrate".to_string(),
                CleanFailureStatus::Clean => {
                    if c.stable {
                        "clean_stable".to_string()
                    } else {
                        "clean_unstable".to_string()
                    }
                }
            },
        ),
        None => (0, FailureCause::Excluded, String::new()),
    };
    FailureScenarioCell {
        name,
        class,
        signal_count: signals.iter().filter(|s| s.class == class).count(),
        clean_occurrences,
        cause,
        emits_candidate: emits,
        opens_training: report.opens_training,
        detail,
    }
}

/// The canonical detector signal set used to prove serialized-report re-derivation: two clean,
/// stable reading-misgrounding failures (which emit a candidate).
fn canonical_signals() -> Vec<FailureSignal> {
    let obs = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
    vec![
        signal(
            FailureClass::ReadingMisgrounding,
            obs.clone(),
            FailureContext::clean(),
        ),
        signal(
            FailureClass::ReadingMisgrounding,
            obs,
            FailureContext::clean(),
        ),
    ]
}

/// The canonical, deterministic [`FailureDetectorReport`] for the serialized-report tamper proof.
pub fn canonical_failure_report() -> FailureDetectorReport {
    detect_failures(&canonical_signals())
}

/// The canonical failure report serialized to JSON.
pub fn failure_report_json() -> String {
    serde_json::to_string(&canonical_failure_report()).expect("failure report serializes")
}

/// The serialized-report tamper cell: tamper the canonical report JSON and observe the re-derive
/// verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous; the
/// canonical form must itself verify — so a broken verifier surfaces as a non-refusal, not a
/// false refusal.
fn tamper_cell() -> FailureScenarioCell {
    let signals = canonical_signals();
    let canonical = detect_failures_json(&signals);
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_failure_report_json(&signals, &tampered).is_err()
        && verify_failure_report_json(&signals, &canonical).is_ok();
    FailureScenarioCell {
        name: "serialized_failure_report_tamper_refused",
        class: FailureClass::ReplayInconsistency,
        signal_count: 0,
        clean_occurrences: 0,
        cause: FailureCause::Substrate,
        emits_candidate: false,
        opens_training: false,
        detail: if refused {
            "serialized_report_tamper_refused".to_string()
        } else {
            "VACUOUS: report verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 16-scenario detector matrix from the REAL detector over REAL SCORE-0 failures.
pub fn failure_detector_matrix() -> FailureDetectorMatrix {
    // Derive the SCORE-0 failure set ONCE; every scenario reuses it (no per-lookup rebuild).
    let failures = verifier_score_matrix().failures;
    let grounding = || find_obs(&failures, ScoreClass::Grounding, ScoreReason::Ungrounded);
    let refusal = || find_obs(&failures, ScoreClass::Refusal, ScoreReason::RefusalAbsent);
    let answer = || {
        find_obs(
            &failures,
            ScoreClass::AnswerSupport,
            ScoreReason::AnswerUnsupported,
        )
    };
    let replay = || find_obs(&failures, ScoreClass::Replay, ScoreReason::ReplayMismatch);
    let integrity = || {
        find_obs(
            &failures,
            ScoreClass::TraceIntegrity,
            ScoreReason::IntegrityViolated,
        )
    };

    let excluded = |field: fn(&mut FailureContext)| {
        let mut c = FailureContext::clean();
        field(&mut c);
        c
    };

    let scenarios = vec![
        cell(
            "single_failure_no_candidate",
            FailureClass::ReadingMisgrounding,
            vec![signal(
                FailureClass::ReadingMisgrounding,
                grounding(),
                FailureContext::clean(),
            )],
        ),
        cell(
            "recurring_clean_model_failure_candidate",
            FailureClass::ReadingMisgrounding,
            vec![
                signal(
                    FailureClass::ReadingMisgrounding,
                    grounding(),
                    FailureContext::clean(),
                ),
                signal(
                    FailureClass::ReadingMisgrounding,
                    grounding(),
                    FailureContext::clean(),
                ),
            ],
        ),
        cell(
            "recurring_substrate_failure_no_candidate",
            FailureClass::ReplayInconsistency,
            vec![
                signal(
                    FailureClass::ReplayInconsistency,
                    replay(),
                    FailureContext::clean(),
                ),
                signal(
                    FailureClass::ReplayInconsistency,
                    replay(),
                    FailureContext::clean(),
                ),
            ],
        ),
        cell(
            "missing_context_excluded",
            FailureClass::HorizonPlanFailure,
            vec![
                signal(
                    FailureClass::HorizonPlanFailure,
                    grounding(),
                    excluded(|c| c.context_present = false),
                ),
                signal(
                    FailureClass::HorizonPlanFailure,
                    grounding(),
                    excluded(|c| c.context_present = false),
                ),
            ],
        ),
        cell(
            "bad_retrieval_excluded",
            FailureClass::MemoryRetrievalFailure,
            vec![
                signal(
                    FailureClass::MemoryRetrievalFailure,
                    grounding(),
                    excluded(|c| c.retrieval_valid = false),
                ),
                signal(
                    FailureClass::MemoryRetrievalFailure,
                    grounding(),
                    excluded(|c| c.retrieval_valid = false),
                ),
            ],
        ),
        cell(
            "uncurated_data_excluded",
            FailureClass::ReadingMisgrounding,
            vec![
                signal(
                    FailureClass::ReadingMisgrounding,
                    grounding(),
                    excluded(|c| {
                        c.curation_passed = false;
                        c.refusal_context_valid = false;
                    }),
                ),
                signal(
                    FailureClass::ReadingMisgrounding,
                    grounding(),
                    excluded(|c| {
                        c.curation_passed = false;
                        c.refusal_context_valid = false;
                    }),
                ),
            ],
        ),
        cell(
            "bad_prompt_schema_excluded",
            FailureClass::ToolUseSchemaFailure,
            vec![
                signal(
                    FailureClass::ToolUseSchemaFailure,
                    grounding(),
                    excluded(|c| c.prompt_schema_valid = false),
                ),
                signal(
                    FailureClass::ToolUseSchemaFailure,
                    grounding(),
                    excluded(|c| c.prompt_schema_valid = false),
                ),
            ],
        ),
        cell(
            "invalid_test_excluded",
            FailureClass::CodingPatchFailure,
            vec![
                signal(
                    FailureClass::CodingPatchFailure,
                    grounding(),
                    excluded(|c| c.test_valid = false),
                ),
                signal(
                    FailureClass::CodingPatchFailure,
                    grounding(),
                    excluded(|c| c.test_valid = false),
                ),
            ],
        ),
        cell(
            "stale_artifact_excluded",
            FailureClass::SourceSelectionFailure,
            vec![
                signal(
                    FailureClass::SourceSelectionFailure,
                    grounding(),
                    excluded(|c| c.artifact_fresh = false),
                ),
                signal(
                    FailureClass::SourceSelectionFailure,
                    grounding(),
                    excluded(|c| c.artifact_fresh = false),
                ),
            ],
        ),
        cell(
            "unverified_replay_excluded",
            FailureClass::MultiDocSynthesisFailure,
            vec![
                signal(
                    FailureClass::MultiDocSynthesisFailure,
                    answer(),
                    excluded(|c| c.replay_verified = false),
                ),
                signal(
                    FailureClass::MultiDocSynthesisFailure,
                    answer(),
                    excluded(|c| c.replay_verified = false),
                ),
            ],
        ),
        cell(
            "quarantined_candidate_excluded",
            FailureClass::ReadingMisgrounding,
            vec![
                signal(
                    FailureClass::ReadingMisgrounding,
                    grounding(),
                    excluded(|c| c.quarantined = true),
                ),
                signal(
                    FailureClass::ReadingMisgrounding,
                    grounding(),
                    excluded(|c| c.quarantined = true),
                ),
            ],
        ),
        cell(
            "unstable_failure_class_excluded",
            FailureClass::InstructionFollowingFailure,
            vec![
                // Same class, DIFFERENT SCORE-0 reasons ⇒ not reason-stable ⇒ no candidate.
                signal(
                    FailureClass::InstructionFollowingFailure,
                    grounding(),
                    FailureContext::clean(),
                ),
                signal(
                    FailureClass::InstructionFollowingFailure,
                    answer(),
                    FailureContext::clean(),
                ),
            ],
        ),
        cell(
            "stable_failure_class_candidate",
            FailureClass::SourceSelectionFailure,
            vec![
                signal(
                    FailureClass::SourceSelectionFailure,
                    answer(),
                    FailureContext::clean(),
                ),
                signal(
                    FailureClass::SourceSelectionFailure,
                    answer(),
                    FailureContext::clean(),
                ),
            ],
        ),
        cell(
            "refusal_boundary_recurrence_candidate",
            FailureClass::RefusalBoundaryFailure,
            vec![
                // No curation, but a valid refusal context ⇒ clean.
                signal(
                    FailureClass::RefusalBoundaryFailure,
                    refusal(),
                    excluded(|c| c.curation_passed = false),
                ),
                signal(
                    FailureClass::RefusalBoundaryFailure,
                    refusal(),
                    excluded(|c| c.curation_passed = false),
                ),
            ],
        ),
        cell(
            "trace_integrity_failure_not_model_need",
            FailureClass::ReplayInconsistency,
            vec![
                signal(
                    FailureClass::ReplayInconsistency,
                    integrity(),
                    FailureContext::clean(),
                ),
                signal(
                    FailureClass::ReplayInconsistency,
                    integrity(),
                    FailureContext::clean(),
                ),
            ],
        ),
        tamper_cell(),
    ];

    let training_never_opens = scenarios.iter().all(|c| !c.opens_training);
    FailureDetectorMatrix {
        schema: SCHEMA,
        scenarios,
        classes: FAILURE_CLASS_NAMES,
        recurrence_threshold: RECURRENCE_THRESHOLD,
        training_never_opens,
        boundary: FailureDetectorBoundary::inert(),
    }
}

/// The detector matrix serialized to canonical JSON.
pub fn failure_detector_matrix_json() -> String {
    serde_json::to_string(&failure_detector_matrix()).expect("failure detector matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`:
/// a serialized matrix is NOT trusted — it is re-derived and compared, so any tampering is refused.
pub fn verify_failure_detector_matrix_json(candidate: &str) -> Result<(), FailureDetectorError> {
    if candidate == failure_detector_matrix_json() {
        Ok(())
    } else {
        Err(FailureDetectorError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- class / threshold / boundary structure ---

    #[test]
    fn there_are_exactly_ten_failure_classes_with_stable_names() {
        assert_eq!(FAILURE_CLASS_COUNT, 10);
        assert_eq!(FailureClass::ALL.len(), 10);
        let tags: Vec<&str> = FailureClass::ALL.iter().map(|c| c.tag()).collect();
        assert_eq!(tags, FAILURE_CLASS_NAMES.to_vec());
    }

    #[test]
    fn recurrence_threshold_is_explicit_and_at_least_two() {
        assert_eq!(RECURRENCE_THRESHOLD, 2);
        assert_eq!(FailureRecurrencePolicy::standard().threshold, 2);
    }

    #[test]
    fn boundary_lines_are_the_nine_and_inert() {
        assert_eq!(FAIL_BOUNDARY_LINES.len(), 9);
        assert_eq!(
            FAIL_BOUNDARY_LINES[0],
            "The failure detector observes recurring clean failures."
        );
        assert_eq!(
            FAIL_BOUNDARY_LINES[8],
            "ModelNeedCandidate is not training authorization."
        );
        assert!(FailureDetectorBoundary::inert().all_inert());
    }

    // --- the detector consumes REAL SCORE-0 observations ---

    #[test]
    fn signals_carry_real_score0_failure_observations() {
        let obs = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
        assert_eq!(obs.class, ScoreClass::Grounding);
        assert_eq!(obs.reason, ScoreReason::Ungrounded);
        // The consumed observation carries SCORE-0's invariant: it is NOT a training example.
        assert!(!obs.is_training_example());
        assert!(!obs.training_example);
    }

    // --- recurrence ---

    #[test]
    fn single_clean_failure_emits_no_candidate() {
        let obs = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
        let report = detect_failures(&[signal(
            FailureClass::ReadingMisgrounding,
            obs,
            FailureContext::clean(),
        )]);
        assert!(report.candidates.is_empty());
        let case = &report.cases[0];
        assert_eq!(case.clean_occurrences, 1);
        assert_eq!(case.cause, FailureCause::CleanModel);
        assert!(!case.emits_candidate);
    }

    #[test]
    fn recurring_clean_model_failure_emits_candidate() {
        let obs = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
        let report = detect_failures(&[
            signal(
                FailureClass::ReadingMisgrounding,
                obs.clone(),
                FailureContext::clean(),
            ),
            signal(
                FailureClass::ReadingMisgrounding,
                obs,
                FailureContext::clean(),
            ),
        ]);
        assert_eq!(report.candidates.len(), 1);
        let cand = &report.candidates[0];
        assert_eq!(cand.class, FailureClass::ReadingMisgrounding);
        assert_eq!(cand.recurrences, 2);
        assert!(!cand.training_justified);
        assert!(!cand.opens_training);
        assert!(!cand.authorizes_training);
    }

    // --- attribution ---

    #[test]
    fn recurring_substrate_failure_is_not_a_model_need() {
        let obs = score_obs(ScoreClass::Replay, ScoreReason::ReplayMismatch);
        let report = detect_failures(&[
            signal(
                FailureClass::ReplayInconsistency,
                obs.clone(),
                FailureContext::clean(),
            ),
            signal(
                FailureClass::ReplayInconsistency,
                obs,
                FailureContext::clean(),
            ),
        ]);
        assert!(report.candidates.is_empty());
        assert_eq!(report.cases[0].cause, FailureCause::Substrate);
    }

    #[test]
    fn each_exclusion_blocks_a_candidate() {
        let obs = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
        let cases: Vec<(FailureContext, FailureExclusion)> = vec![
            (
                FailureContext {
                    quarantined: true,
                    ..FailureContext::clean()
                },
                FailureExclusion::QuarantinedCandidate,
            ),
            (
                FailureContext {
                    context_present: false,
                    ..FailureContext::clean()
                },
                FailureExclusion::MissingContext,
            ),
            (
                FailureContext {
                    retrieval_valid: false,
                    ..FailureContext::clean()
                },
                FailureExclusion::BadRetrieval,
            ),
            (
                FailureContext {
                    prompt_schema_valid: false,
                    ..FailureContext::clean()
                },
                FailureExclusion::BadPromptSchema,
            ),
            (
                FailureContext {
                    test_valid: false,
                    ..FailureContext::clean()
                },
                FailureExclusion::InvalidTest,
            ),
            (
                FailureContext {
                    artifact_fresh: false,
                    ..FailureContext::clean()
                },
                FailureExclusion::StaleArtifact,
            ),
            (
                FailureContext {
                    replay_verified: false,
                    ..FailureContext::clean()
                },
                FailureExclusion::UnverifiedReplay,
            ),
        ];
        for (ctx, expected) in cases {
            let report = detect_failures(&[
                signal(FailureClass::ReadingMisgrounding, obs.clone(), ctx),
                signal(FailureClass::ReadingMisgrounding, obs.clone(), ctx),
            ]);
            assert!(
                report.candidates.is_empty(),
                "exclusion {expected:?} still emitted a candidate"
            );
            assert_eq!(
                report.cases[0].clean_status,
                CleanFailureStatus::Excluded(expected)
            );
            assert_eq!(report.cases[0].cause, FailureCause::Excluded);
        }
    }

    #[test]
    fn uncurated_without_refusal_context_is_excluded() {
        let obs = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
        let mut ctx = FailureContext::clean();
        ctx.curation_passed = false;
        ctx.refusal_context_valid = false;
        let report = detect_failures(&[
            signal(FailureClass::ReadingMisgrounding, obs.clone(), ctx),
            signal(FailureClass::ReadingMisgrounding, obs, ctx),
        ]);
        assert!(report.candidates.is_empty());
        assert_eq!(
            report.cases[0].clean_status,
            CleanFailureStatus::Excluded(FailureExclusion::UncuratedData)
        );
    }

    #[test]
    fn refusal_failure_with_valid_refusal_context_is_clean() {
        let obs = score_obs(ScoreClass::Refusal, ScoreReason::RefusalAbsent);
        let mut ctx = FailureContext::clean();
        ctx.curation_passed = false; // refusal failures need no curation, but a valid refusal context
        let report = detect_failures(&[
            signal(FailureClass::RefusalBoundaryFailure, obs.clone(), ctx),
            signal(FailureClass::RefusalBoundaryFailure, obs, ctx),
        ]);
        assert_eq!(report.candidates.len(), 1);
        assert_eq!(report.cases[0].cause, FailureCause::CleanModel);
    }

    #[test]
    fn unstable_reasons_block_a_candidate() {
        let g = score_obs(ScoreClass::Grounding, ScoreReason::Ungrounded);
        let a = score_obs(ScoreClass::AnswerSupport, ScoreReason::AnswerUnsupported);
        let report = detect_failures(&[
            signal(
                FailureClass::InstructionFollowingFailure,
                g,
                FailureContext::clean(),
            ),
            signal(
                FailureClass::InstructionFollowingFailure,
                a,
                FailureContext::clean(),
            ),
        ]);
        assert!(
            report.candidates.is_empty(),
            "unstable reasons must not emit a candidate"
        );
        assert!(!report.cases[0].stable);
        assert_eq!(report.cases[0].clean_occurrences, 2);
    }

    // --- training closure ---

    #[test]
    fn detector_never_opens_training_even_with_candidates() {
        let report = detect_failures(&canonical_signals());
        assert!(
            !report.candidates.is_empty(),
            "this fixture emits a candidate"
        );
        assert!(!report.training_justified);
        assert!(!report.opens_training);
        assert!(report.boundary.all_inert());
        for c in &report.candidates {
            assert!(!c.training_justified);
            assert!(!c.opens_training);
            assert!(!c.authorizes_training);
        }
        // The REAL P12 verdict (decided on empty inputs) is unmoved and still closed.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    // --- determinism + re-derivation ---

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let signals = canonical_signals();
        let canonical = detect_failures_json(&signals);
        assert_eq!(canonical, detect_failures_json(&signals));
        assert!(verify_failure_report_json(&signals, &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_failure_report_json(&signals, &tampered),
            Err(FailureDetectorError::ReplayMismatch)
        );
    }

    // --- the matrix ---

    #[test]
    fn matrix_has_the_sixteen_named_scenarios() {
        let m = failure_detector_matrix();
        assert_eq!(m.scenarios.len(), FAILURE_SCENARIO_COUNT);
        let names: Vec<&str> = m.scenarios.iter().map(|c| c.name).collect();
        assert_eq!(
            names,
            vec![
                "single_failure_no_candidate",
                "recurring_clean_model_failure_candidate",
                "recurring_substrate_failure_no_candidate",
                "missing_context_excluded",
                "bad_retrieval_excluded",
                "uncurated_data_excluded",
                "bad_prompt_schema_excluded",
                "invalid_test_excluded",
                "stale_artifact_excluded",
                "unverified_replay_excluded",
                "quarantined_candidate_excluded",
                "unstable_failure_class_excluded",
                "stable_failure_class_candidate",
                "refusal_boundary_recurrence_candidate",
                "trace_integrity_failure_not_model_need",
                "serialized_failure_report_tamper_refused",
            ]
        );
    }

    #[test]
    fn matrix_records_the_observed_candidate_decisions() {
        let m = failure_detector_matrix();
        let emits = |n: &str| m.scenario(n).unwrap().emits_candidate;
        // Exactly the three candidate scenarios emit.
        assert!(emits("recurring_clean_model_failure_candidate"));
        assert!(emits("stable_failure_class_candidate"));
        assert!(emits("refusal_boundary_recurrence_candidate"));
        // Everything else does not.
        for c in &m.scenarios {
            let is_candidate_cell = matches!(
                c.name,
                "recurring_clean_model_failure_candidate"
                    | "stable_failure_class_candidate"
                    | "refusal_boundary_recurrence_candidate"
            );
            assert_eq!(c.emits_candidate, is_candidate_cell, "scenario {}", c.name);
        }
    }

    #[test]
    fn matrix_single_failure_and_substrate_and_exclusions_emit_no_candidate() {
        let m = failure_detector_matrix();
        assert!(
            !m.scenario("single_failure_no_candidate")
                .unwrap()
                .emits_candidate
        );
        let substrate = m
            .scenario("recurring_substrate_failure_no_candidate")
            .unwrap();
        assert!(!substrate.emits_candidate);
        assert_eq!(substrate.cause, FailureCause::Substrate);
        let integ = m
            .scenario("trace_integrity_failure_not_model_need")
            .unwrap();
        assert!(!integ.emits_candidate);
        assert_eq!(integ.cause, FailureCause::Substrate);
        for n in [
            "missing_context_excluded",
            "bad_retrieval_excluded",
            "uncurated_data_excluded",
            "bad_prompt_schema_excluded",
            "invalid_test_excluded",
            "stale_artifact_excluded",
            "unverified_replay_excluded",
            "quarantined_candidate_excluded",
        ] {
            let c = m.scenario(n).unwrap();
            assert!(!c.emits_candidate, "{n} emitted a candidate");
            assert_eq!(c.cause, FailureCause::Excluded, "{n} cause");
        }
    }

    #[test]
    fn matrix_serialized_report_tamper_is_refused() {
        let cell = failure_detector_matrix()
            .scenario("serialized_failure_report_tamper_refused")
            .unwrap()
            .clone();
        assert_eq!(cell.detail, "serialized_report_tamper_refused");
        assert!(!cell.emits_candidate);
    }

    #[test]
    fn matrix_opens_no_training_in_any_scenario() {
        let m = failure_detector_matrix();
        assert!(m.training_never_opens);
        assert_eq!(m.recurrence_threshold, RECURRENCE_THRESHOLD);
        assert_eq!(m.classes, FAILURE_CLASS_NAMES);
        for c in &m.scenarios {
            assert!(!c.opens_training, "scenario {} opened training", c.name);
        }
        assert!(m.boundary.all_inert());
    }

    #[test]
    fn matrix_is_deterministic_and_re_derivable() {
        assert_eq!(failure_detector_matrix(), failure_detector_matrix());
        assert_eq!(
            failure_detector_matrix_json(),
            failure_detector_matrix_json()
        );
        let canonical = failure_detector_matrix_json();
        assert!(verify_failure_detector_matrix_json(&canonical).is_ok());
        assert_eq!(
            verify_failure_detector_matrix_json(&format!("{canonical} ")),
            Err(FailureDetectorError::ReplayMismatch)
        );
    }
}
