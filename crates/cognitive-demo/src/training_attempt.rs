//! TRAIN-0 — the first gated, deterministic local training-ATTEMPT harness.
//!
//! This sprint answers exactly ONE operational question: *given a proven model need, may a
//! reproducible local training run be PREPARED — and, only under a two-key authorization, may it
//! yield a CANDIDATE model artifact?* It is a harness, not a trainer. In this sprint it performs no
//! real weight mutation: a `dry_run_only` invocation builds a plan that touches no weights, and an
//! `authorized_local_attempt` invocation (only when fully authorized) prepares a deterministic
//! CANDIDATE descriptor — never an accepted, promoted, or deployed model. Actual weight mutation is
//! deferred to an external authorized runner governed by the runbook; the harness only proves the
//! project can SAFELY refuse or prepare such a run.
//!
//! It CONSUMES the REAL TRAIN-GATE-0 report: [`run_training_attempt`] runs
//! [`evaluate_training_gate`] itself over the supplied gate input (which in turn runs the REAL P11
//! `evaluate_model_need`, which consumes REAL FAIL-0 candidates over REAL SCORE-0 failures — the
//! SCORE-0 -> FAIL-0 -> MODEL-EVAL -> TRAIN-GATE -> TRAIN-ATTEMPT chain). The gate decision is
//! therefore DERIVED, never handed in as a trusted (forgeable) report.
//!
//! TWO KEYS, closed by default. An `authorized_local_attempt` may prepare a candidate ONLY when
//! BOTH hold together:
//!
//!   1. the consumed TRAIN-GATE-0 report is exactly [`TrainingGateDecision::TrainingAttemptAllowed`],
//!   2. a SEPARATE explicit operator authorization receipt for the ATTEMPT is present,
//!
//! AND every reproducibility prerequisite is satisfied: a deterministic hash-pinned run config, a
//! curated hash-pinned dataset bundle (uncontaminated), a present hash-pinned holdout bundle (no
//! leakage), a hash-pinned baseline artifact, a hash-pinned rollback artifact, and an affirmative
//! authority-drift check. `TrainingAttemptAllowed` ALONE is insufficient; operator authorization
//! ALONE is insufficient; any missing or unclean prerequisite refuses the attempt.
//!
//! Crucially, a produced [`TrainingCandidateArtifact`] is `CandidateOnly` at the type level: it is
//! not promoted, not deployed, not evidence, creates no memory, grants no authority, and does NOT
//! replace the baseline. It MUST be evaluated later (S8) before any promotion. Every forbidden-action
//! flag is sourced from the structural const [`ATTEMPT_CREATES_ACCEPTED_MODEL`] (`false`): no path can
//! set one true, and the deeper P12 gate (`reading_train_gate::decide`) stays `training_justified =
//! false` regardless. Receipts are `Serialize` but NEVER `Deserialize`: a serialized receipt is
//! re-derived from the same input and byte-compared, so tampering is refused.
//!
//! The boundary, recorded verbatim in [`TRAINING_ATTEMPT_BOUNDARY_LINES`]:
//!
//!   The training attempt path may create a candidate model artifact only after gate approval and
//!   explicit operator authorization.
//!   It does not promote models.
//!   It does not deploy models.
//!   It does not create truth.
//!   It does not create memory.
//!   It does not create evidence.
//!   It does not grant new authority.
//!   A candidate model is not an accepted model.
//!   A candidate model must pass later evaluation before promotion.

use crate::{
    detect_failures, evaluate_training_gate, verifier_score_matrix, AuthorityDriftCheck,
    ContaminationReportReceipt, DatasetReadinessReceipt, EvalComparison, EvalCondition, EvalRun,
    FailureClass, FailureContext, FailureObservation, FailureSignal, HoldoutReadinessReceipt,
    ModelEvalBattery, ModelNeedCandidate, OperatorAuthorizationReceipt,
    ProductionSafetyPlanReceipt, RollbackPlanReceipt, ScoreClass, ScoreReason,
    TrainingGateDecision, TrainingGateInput, TrainingGateReport, RECURRENCE_THRESHOLD,
};
use serde::Serialize;

/// The schema tag stamped on every serialized training-attempt artifact.
const SCHEMA: &str = "training-attempt-v0.1";

/// THE structural invariant: preparing a training ATTEMPT (or even a candidate descriptor) is not, by
/// itself, the creation of an ACCEPTED model. Every forbidden-action flag is sourced from this const,
/// so no code path — not even a fully authorized attempt — can set one true.
const ATTEMPT_CREATES_ACCEPTED_MODEL: bool = false;

/// Exactly two invocation modes.
pub const TRAIN_ATTEMPT_MODE_COUNT: usize = 2;

/// The two mode slugs, in canonical order.
pub const TRAIN_ATTEMPT_MODE_NAMES: [&str; TRAIN_ATTEMPT_MODE_COUNT] =
    ["dry_run_only", "authorized_local_attempt"];

/// Exactly twelve refusal reasons.
pub const TRAIN_ATTEMPT_REFUSAL_COUNT: usize = 12;

/// The twelve refusal-reason slugs, in canonical order.
pub const TRAIN_ATTEMPT_REFUSAL_NAMES: [&str; TRAIN_ATTEMPT_REFUSAL_COUNT] = [
    "missing_training_gate_allow",
    "missing_explicit_operator_authorization",
    "missing_training_run_config",
    "missing_curated_dataset_bundle",
    "missing_baseline_artifact",
    "missing_holdout_bundle",
    "missing_rollback_artifact",
    "contaminated_dataset_refused",
    "holdout_leakage_refused",
    "authority_drift_refused",
    "non_reproducible_config_refused",
    "training_attempt_serialized_tamper_refused",
];

/// The fixed training-attempt scenario matrix size.
pub const TRAIN_ATTEMPT_SCENARIO_COUNT: usize = 20;

/// The cannot-bypass boundary, recorded verbatim.
pub const TRAINING_ATTEMPT_BOUNDARY_LINES: [&str; 9] = [
    "The training attempt path may create a candidate model artifact only after gate approval and explicit operator authorization.",
    "It does not promote models.",
    "It does not deploy models.",
    "It does not create truth.",
    "It does not create memory.",
    "It does not create evidence.",
    "It does not grant new authority.",
    "A candidate model is not an accepted model.",
    "A candidate model must pass later evaluation before promotion.",
];

// --- mode / outcome / requirement / refusal taxonomies ---

/// How the harness was invoked. `DryRunOnly` always prepares a plan and never touches weights;
/// `AuthorizedLocalAttempt` may prepare a candidate, but only under the two-key authorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingAttemptMode {
    /// Build a plan describing what a run would consume. Touches no weights, produces no candidate.
    DryRunOnly,
    /// Attempt to prepare a candidate locally. Refused unless every prerequisite is present.
    AuthorizedLocalAttempt,
}

impl TrainingAttemptMode {
    /// Every mode, in canonical order.
    pub const ALL: [TrainingAttemptMode; TRAIN_ATTEMPT_MODE_COUNT] = [
        TrainingAttemptMode::DryRunOnly,
        TrainingAttemptMode::AuthorizedLocalAttempt,
    ];

    /// The stable slug for this mode.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingAttemptMode::DryRunOnly => "dry_run_only",
            TrainingAttemptMode::AuthorizedLocalAttempt => "authorized_local_attempt",
        }
    }
}

/// The terminal outcome of an invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingAttemptOutcome {
    /// A dry-run plan was prepared (no weights touched, no candidate produced).
    DryRunPlanPrepared,
    /// An authorized attempt was refused (at least one prerequisite unmet).
    AttemptRefused,
    /// An authorized attempt prepared a CANDIDATE-ONLY artifact (still not promoted/deployed).
    CandidatePrepared,
}

impl TrainingAttemptOutcome {
    /// The stable slug for this outcome.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingAttemptOutcome::DryRunPlanPrepared => "dry_run_plan_prepared",
            TrainingAttemptOutcome::AttemptRefused => "attempt_refused",
            TrainingAttemptOutcome::CandidatePrepared => "candidate_prepared",
        }
    }
}

/// A prerequisite the harness checks for an authorized attempt. Recorded (in the receipt's
/// `satisfied` list) when met.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingAttemptRequirement {
    /// The consumed TRAIN-GATE-0 report is `TrainingAttemptAllowed`.
    TrainingGateAllowed,
    /// A SEPARATE explicit operator authorization for the attempt is present.
    ExplicitOperatorAuthorization,
    /// A run config is present.
    TrainingRunConfig,
    /// A curated dataset bundle is present.
    CuratedDatasetBundle,
    /// A baseline artifact is present.
    BaselineArtifact,
    /// A holdout bundle is present.
    HoldoutBundle,
    /// A rollback artifact is present.
    RollbackArtifact,
    /// The dataset bundle is uncontaminated.
    DatasetContaminationClean,
    /// The holdout bundle has no leakage.
    HoldoutLeakageClean,
    /// The authority-drift check is affirmative and clean.
    AuthorityDriftClean,
    /// The run config is deterministic (reproducible).
    ReproducibleConfig,
}

impl TrainingAttemptRequirement {
    /// Every requirement, in check order.
    pub const ALL: [TrainingAttemptRequirement; 11] = [
        TrainingAttemptRequirement::TrainingGateAllowed,
        TrainingAttemptRequirement::ExplicitOperatorAuthorization,
        TrainingAttemptRequirement::TrainingRunConfig,
        TrainingAttemptRequirement::CuratedDatasetBundle,
        TrainingAttemptRequirement::BaselineArtifact,
        TrainingAttemptRequirement::HoldoutBundle,
        TrainingAttemptRequirement::RollbackArtifact,
        TrainingAttemptRequirement::DatasetContaminationClean,
        TrainingAttemptRequirement::HoldoutLeakageClean,
        TrainingAttemptRequirement::AuthorityDriftClean,
        TrainingAttemptRequirement::ReproducibleConfig,
    ];

    /// The stable slug for this requirement.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingAttemptRequirement::TrainingGateAllowed => "training_gate_allowed",
            TrainingAttemptRequirement::ExplicitOperatorAuthorization => {
                "explicit_operator_authorization"
            }
            TrainingAttemptRequirement::TrainingRunConfig => "training_run_config",
            TrainingAttemptRequirement::CuratedDatasetBundle => "curated_dataset_bundle",
            TrainingAttemptRequirement::BaselineArtifact => "baseline_artifact",
            TrainingAttemptRequirement::HoldoutBundle => "holdout_bundle",
            TrainingAttemptRequirement::RollbackArtifact => "rollback_artifact",
            TrainingAttemptRequirement::DatasetContaminationClean => "dataset_contamination_clean",
            TrainingAttemptRequirement::HoldoutLeakageClean => "holdout_leakage_clean",
            TrainingAttemptRequirement::AuthorityDriftClean => "authority_drift_clean",
            TrainingAttemptRequirement::ReproducibleConfig => "reproducible_config",
        }
    }
}

/// Why the harness refused an attempt. The first eleven are prerequisite-path reasons; the twelfth
/// (`TrainingAttemptSerializedTamperRefused`) is emitted only by the serialized-receipt
/// re-derivation path (a tampered receipt is never trusted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrainingAttemptRefusal {
    /// The consumed gate report is not `TrainingAttemptAllowed`.
    MissingTrainingGateAllow,
    /// No separate explicit operator authorization for the attempt.
    MissingExplicitOperatorAuthorization,
    /// No run config.
    MissingTrainingRunConfig,
    /// No curated dataset bundle.
    MissingCuratedDatasetBundle,
    /// No baseline artifact.
    MissingBaselineArtifact,
    /// No holdout bundle (or the holdout is absent).
    MissingHoldoutBundle,
    /// No rollback artifact.
    MissingRollbackArtifact,
    /// The dataset bundle is present but contaminated.
    ContaminatedDatasetRefused,
    /// The holdout bundle is present but leaked into training.
    HoldoutLeakageRefused,
    /// The authority-drift check was not run, or it detected drift.
    AuthorityDriftRefused,
    /// The run config is present but non-deterministic (not reproducible).
    NonReproducibleConfigRefused,
    /// A serialized attempt receipt did not match its re-derivation and was refused.
    TrainingAttemptSerializedTamperRefused,
}

impl TrainingAttemptRefusal {
    /// Every refusal reason, in canonical order.
    pub const ALL: [TrainingAttemptRefusal; TRAIN_ATTEMPT_REFUSAL_COUNT] = [
        TrainingAttemptRefusal::MissingTrainingGateAllow,
        TrainingAttemptRefusal::MissingExplicitOperatorAuthorization,
        TrainingAttemptRefusal::MissingTrainingRunConfig,
        TrainingAttemptRefusal::MissingCuratedDatasetBundle,
        TrainingAttemptRefusal::MissingBaselineArtifact,
        TrainingAttemptRefusal::MissingHoldoutBundle,
        TrainingAttemptRefusal::MissingRollbackArtifact,
        TrainingAttemptRefusal::ContaminatedDatasetRefused,
        TrainingAttemptRefusal::HoldoutLeakageRefused,
        TrainingAttemptRefusal::AuthorityDriftRefused,
        TrainingAttemptRefusal::NonReproducibleConfigRefused,
        TrainingAttemptRefusal::TrainingAttemptSerializedTamperRefused,
    ];

    /// The stable slug for this refusal reason.
    pub fn tag(&self) -> &'static str {
        match self {
            TrainingAttemptRefusal::MissingTrainingGateAllow => "missing_training_gate_allow",
            TrainingAttemptRefusal::MissingExplicitOperatorAuthorization => {
                "missing_explicit_operator_authorization"
            }
            TrainingAttemptRefusal::MissingTrainingRunConfig => "missing_training_run_config",
            TrainingAttemptRefusal::MissingCuratedDatasetBundle => "missing_curated_dataset_bundle",
            TrainingAttemptRefusal::MissingBaselineArtifact => "missing_baseline_artifact",
            TrainingAttemptRefusal::MissingHoldoutBundle => "missing_holdout_bundle",
            TrainingAttemptRefusal::MissingRollbackArtifact => "missing_rollback_artifact",
            TrainingAttemptRefusal::ContaminatedDatasetRefused => "contaminated_dataset_refused",
            TrainingAttemptRefusal::HoldoutLeakageRefused => "holdout_leakage_refused",
            TrainingAttemptRefusal::AuthorityDriftRefused => "authority_drift_refused",
            TrainingAttemptRefusal::NonReproducibleConfigRefused => {
                "non_reproducible_config_refused"
            }
            TrainingAttemptRefusal::TrainingAttemptSerializedTamperRefused => {
                "training_attempt_serialized_tamper_refused"
            }
        }
    }
}

// --- attempt INPUTS (never trusted off-wire: Debug + Clone, no Serialize, no Deserialize) ---

/// The SEPARATE explicit operator authorization for the ATTEMPT itself — a distinct key from the
/// gate's own operator authorization. Both keys must turn for a candidate to be prepared.
#[derive(Debug, Clone)]
pub struct AttemptAuthorizationReceipt {
    /// Who authorized the attempt.
    pub operator: String,
    /// The narrow scope of the authorized attempt.
    pub attempt_scope: String,
    /// The operator's explicit acknowledgement that this prepares a CANDIDATE only.
    pub acknowledges_candidate_only: bool,
}

/// A deterministic, hash-pinned training run configuration. Non-`Serialize` input. A run is only
/// reproducible when `deterministic` is set (a fixed seed and a content-pinned config).
#[derive(Debug, Clone)]
pub struct TrainingRunConfig {
    /// The content hash pinning this configuration.
    pub config_hash: String,
    /// Whether the configuration is deterministic (fixed seed, no nondeterministic ops).
    pub deterministic: bool,
    /// The fixed RNG seed (recorded for the runbook; the harness performs no real training).
    pub seed: u64,
    /// The bounded number of steps a real run would take.
    pub max_steps: u64,
}

/// A curated, hash-pinned training dataset bundle (produced upstream by the corpus harvest).
#[derive(Debug, Clone)]
pub struct TrainingDatasetBundle {
    /// The content hash of the curated corpus.
    pub curated_corpus_hash: String,
    /// How many curated items it contains.
    pub item_count: usize,
    /// Whether contamination was detected in the curated bundle.
    pub contaminated: bool,
}

/// A hash-pinned baseline model artifact — the model a candidate is derived from and must NOT replace.
#[derive(Debug, Clone)]
pub struct TrainingBaselineArtifact {
    /// The content hash pinning the baseline.
    pub baseline_hash: String,
}

/// A hash-pinned held-out evaluation bundle.
#[derive(Debug, Clone)]
pub struct TrainingHoldoutBundle {
    /// Whether a genuine holdout exists.
    pub holdout_present: bool,
    /// Whether the holdout leaked into the training data.
    pub leaked: bool,
    /// The content hash of the holdout.
    pub holdout_hash: String,
}

/// A hash-pinned rollback artifact — the verified snapshot a failed attempt reverts to.
#[derive(Debug, Clone)]
pub struct TrainingRollbackArtifact {
    /// The content hash pinning the rollback target.
    pub rollback_hash: String,
    /// Whether the rollback path was verified.
    pub verified: bool,
}

/// The full set of inputs the harness weighs. INPUT type (never `Serialize`): the harness re-runs the
/// REAL gate over `gate_input` and re-checks every artifact. Closed by default.
#[derive(Debug)]
pub struct TrainingAttemptInput {
    /// The requested invocation mode.
    pub mode: TrainingAttemptMode,
    /// The TRAIN-GATE-0 input the harness RUNS [`evaluate_training_gate`] over (genuine consumption;
    /// the gate report is derived, never handed in).
    pub gate_input: TrainingGateInput,
    /// The SEPARATE explicit operator authorization for the attempt (the second key).
    pub operator_authorization: Option<AttemptAuthorizationReceipt>,
    /// The deterministic, hash-pinned run config.
    pub run_config: Option<TrainingRunConfig>,
    /// The curated, hash-pinned dataset bundle.
    pub dataset: Option<TrainingDatasetBundle>,
    /// The hash-pinned baseline artifact.
    pub baseline: Option<TrainingBaselineArtifact>,
    /// The hash-pinned holdout bundle.
    pub holdout: Option<TrainingHoldoutBundle>,
    /// The hash-pinned rollback artifact.
    pub rollback: Option<TrainingRollbackArtifact>,
    /// The authority-drift check (unchecked by default).
    pub authority_drift: AuthorityDriftCheck,
}

impl TrainingAttemptInput {
    /// The closed-by-default dry-run input: a closed gate, nothing supplied, drift unchecked.
    pub fn closed_dry_run() -> Self {
        Self {
            mode: TrainingAttemptMode::DryRunOnly,
            gate_input: TrainingGateInput::closed_by_default(),
            operator_authorization: None,
            run_config: None,
            dataset: None,
            baseline: None,
            holdout: None,
            rollback: None,
            authority_drift: AuthorityDriftCheck::unchecked(),
        }
    }
}

// --- the boundary record ---

/// The inert boundary: every forbidden action is `false`. Stamped on every artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TrainingAttemptBoundary {
    /// The harness never promotes a model.
    pub promotes_model: bool,
    /// The harness never deploys a model.
    pub deploys_model: bool,
    /// The harness never replaces the baseline.
    pub replaces_baseline: bool,
    /// The harness never creates truth.
    pub creates_truth: bool,
    /// The harness never creates memory.
    pub creates_memory: bool,
    /// The harness never creates evidence.
    pub creates_evidence: bool,
    /// The harness never grants new authority.
    pub grants_authority: bool,
    /// The harness never mutates weights (real training is deferred to an external runner).
    pub modifies_weights: bool,
}

impl TrainingAttemptBoundary {
    fn inert() -> Self {
        Self {
            promotes_model: ATTEMPT_CREATES_ACCEPTED_MODEL,
            deploys_model: ATTEMPT_CREATES_ACCEPTED_MODEL,
            replaces_baseline: ATTEMPT_CREATES_ACCEPTED_MODEL,
            creates_truth: ATTEMPT_CREATES_ACCEPTED_MODEL,
            creates_memory: ATTEMPT_CREATES_ACCEPTED_MODEL,
            creates_evidence: ATTEMPT_CREATES_ACCEPTED_MODEL,
            grants_authority: ATTEMPT_CREATES_ACCEPTED_MODEL,
            modifies_weights: ATTEMPT_CREATES_ACCEPTED_MODEL,
        }
    }

    /// True iff every forbidden action is inert.
    pub fn all_inert(&self) -> bool {
        !self.promotes_model
            && !self.deploys_model
            && !self.replaces_baseline
            && !self.creates_truth
            && !self.creates_memory
            && !self.creates_evidence
            && !self.grants_authority
            && !self.modifies_weights
    }
}

// --- the candidate artifact ---

/// A candidate's acceptance status. A SINGLE variant by design: a candidate produced by this harness
/// is `CandidateOnly` and can never be represented as accepted — acceptance is S8's job, not S7's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CandidateAcceptance {
    /// The artifact is a candidate only — not accepted, not promoted, not deployed.
    CandidateOnly,
}

impl CandidateAcceptance {
    /// The stable slug.
    pub fn tag(&self) -> &'static str {
        match self {
            CandidateAcceptance::CandidateOnly => "candidate_only",
        }
    }
}

/// A prepared candidate model descriptor. Hash-pinned and reproducible (a deterministic function of
/// the pinned baseline/dataset/config hashes). It is `CandidateOnly`: not promoted, not deployed, not
/// evidence; it creates no memory and grants no authority, does not replace the baseline, and MUST be
/// evaluated by S8 before any promotion. `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingCandidateArtifact {
    /// The schema tag.
    pub schema: &'static str,
    /// The hash-pinned candidate descriptor (derived from baseline/dataset/config).
    pub candidate_hash: String,
    /// The baseline this candidate descends from (lineage).
    pub baseline_hash: String,
    /// The dataset the candidate would be trained on.
    pub dataset_hash: String,
    /// The run config the candidate would use.
    pub config_hash: String,
    /// The acceptance status — always `CandidateOnly`.
    pub acceptance: CandidateAcceptance,
    /// Always `true`: the candidate must pass later S8 evaluation before any promotion.
    pub requires_s8_evaluation: bool,
    /// Always `false`: the candidate is not promoted.
    pub promoted: bool,
    /// Always `false`: the candidate is not deployed.
    pub deployed: bool,
    /// Always `false`: the candidate is not evidence.
    pub is_evidence: bool,
    /// Always `false`: the candidate creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the candidate grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the candidate does not replace the baseline.
    pub replaces_baseline: bool,
}

/// A non-cryptographic, dependency-free FNV-1a content pin over the candidate's lineage. Deterministic
/// and portable (pure integer arithmetic), so the candidate descriptor re-derives byte-identically.
fn fnv1a_hex(s: &str) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for b in s.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("cand-{hash:016x}")
}

fn build_candidate(
    config: &TrainingRunConfig,
    dataset: &TrainingDatasetBundle,
    baseline: &TrainingBaselineArtifact,
) -> TrainingCandidateArtifact {
    let lineage = format!(
        "{}|{}|{}",
        baseline.baseline_hash, dataset.curated_corpus_hash, config.config_hash
    );
    TrainingCandidateArtifact {
        schema: SCHEMA,
        candidate_hash: fnv1a_hex(&lineage),
        baseline_hash: baseline.baseline_hash.clone(),
        dataset_hash: dataset.curated_corpus_hash.clone(),
        config_hash: config.config_hash.clone(),
        acceptance: CandidateAcceptance::CandidateOnly,
        requires_s8_evaluation: true,
        promoted: ATTEMPT_CREATES_ACCEPTED_MODEL,
        deployed: ATTEMPT_CREATES_ACCEPTED_MODEL,
        is_evidence: ATTEMPT_CREATES_ACCEPTED_MODEL,
        creates_memory: ATTEMPT_CREATES_ACCEPTED_MODEL,
        grants_authority: ATTEMPT_CREATES_ACCEPTED_MODEL,
        replaces_baseline: ATTEMPT_CREATES_ACCEPTED_MODEL,
    }
}

// --- the plan ---

/// The dry-run proposal: what a real attempt would consume, and whether it would be allowed. A plan
/// touches no weights and produces no candidate. `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingAttemptPlan {
    /// The schema tag.
    pub schema: &'static str,
    /// The mode this plan was prepared under.
    pub mode: TrainingAttemptMode,
    /// The consumed gate decision slug.
    pub gate_decision: &'static str,
    /// Whether the consumed gate emitted `TrainingAttemptAllowed`.
    pub gate_allowed: bool,
    /// The pinned config hash, if a config was supplied.
    pub config_hash: Option<String>,
    /// The pinned dataset hash, if a dataset was supplied.
    pub dataset_hash: Option<String>,
    /// The pinned baseline hash, if a baseline was supplied.
    pub baseline_hash: Option<String>,
    /// The pinned holdout hash, if a holdout was supplied.
    pub holdout_hash: Option<String>,
    /// The pinned rollback hash, if a rollback was supplied.
    pub rollback_hash: Option<String>,
    /// Which prerequisites are still missing/unclean for a real attempt.
    pub missing: Vec<TrainingAttemptRefusal>,
    /// Whether an authorized attempt with these inputs would prepare a candidate.
    pub would_be_allowed: bool,
    /// Always `false`: preparing a plan touches no weights.
    pub touches_weights: bool,
    /// The inert boundary.
    pub boundary: TrainingAttemptBoundary,
}

// --- the receipt (top-level report) ---

/// The harness's record of an invocation. `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingAttemptReceipt {
    /// The schema tag.
    pub schema: &'static str,
    /// The invocation mode.
    pub mode: TrainingAttemptMode,
    /// The terminal outcome.
    pub outcome: TrainingAttemptOutcome,
    /// The consumed gate decision slug.
    pub gate_decision: &'static str,
    /// Whether the consumed gate emitted `TrainingAttemptAllowed`.
    pub gate_allowed: bool,
    /// Which prerequisites were satisfied.
    pub satisfied: Vec<TrainingAttemptRequirement>,
    /// Why the attempt was refused (empty for a dry run or a prepared candidate).
    pub refusals: Vec<TrainingAttemptRefusal>,
    /// The dry-run plan (present for `DryRunPlanPrepared`).
    pub plan: Option<TrainingAttemptPlan>,
    /// The prepared candidate (present ONLY for `CandidatePrepared`).
    pub candidate: Option<TrainingCandidateArtifact>,
    /// Always `false`: the harness promotes nothing.
    pub promotes_model: bool,
    /// Always `false`: the harness deploys nothing.
    pub deploys_model: bool,
    /// Always `false`: the harness replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the harness creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the harness creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the harness grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the harness mutates no weights.
    pub modifies_weights: bool,
    /// Always `false`: an attempt does not set P12 `training_justified`.
    pub training_justified: bool,
    /// The inert boundary.
    pub boundary: TrainingAttemptBoundary,
}

impl TrainingAttemptReceipt {
    /// Whether the SEPARATE explicit operator authorization (the second key) was satisfied.
    pub fn operator_was_authorized(&self) -> bool {
        self.satisfied
            .contains(&TrainingAttemptRequirement::ExplicitOperatorAuthorization)
    }
}

/// The outcome of checking every prerequisite for an authorized attempt.
struct Prerequisites {
    gate_decision: &'static str,
    gate_allowed: bool,
    satisfied: Vec<TrainingAttemptRequirement>,
    refusals: Vec<TrainingAttemptRefusal>,
}

/// Run the REAL gate and check every attempt prerequisite. The gate decision is DERIVED here (the
/// harness never trusts a handed-in gate report).
fn check_prerequisites(input: &TrainingAttemptInput) -> Prerequisites {
    let mut satisfied: Vec<TrainingAttemptRequirement> = Vec::new();
    let mut refusals: Vec<TrainingAttemptRefusal> = Vec::new();

    // Key 1: CONSUME the real TRAIN-GATE-0 report (which re-runs P11 over the battery).
    let gate: TrainingGateReport = evaluate_training_gate(&input.gate_input);
    let gate_allowed = gate.decision == TrainingGateDecision::TrainingAttemptAllowed;
    if gate_allowed {
        satisfied.push(TrainingAttemptRequirement::TrainingGateAllowed);
    } else {
        refusals.push(TrainingAttemptRefusal::MissingTrainingGateAllow);
    }

    // Key 2: the SEPARATE explicit operator authorization for the attempt.
    match &input.operator_authorization {
        Some(_) => satisfied.push(TrainingAttemptRequirement::ExplicitOperatorAuthorization),
        None => refusals.push(TrainingAttemptRefusal::MissingExplicitOperatorAuthorization),
    }

    // A present, deterministic run config.
    match &input.run_config {
        None => refusals.push(TrainingAttemptRefusal::MissingTrainingRunConfig),
        Some(c) => {
            satisfied.push(TrainingAttemptRequirement::TrainingRunConfig);
            if c.deterministic {
                satisfied.push(TrainingAttemptRequirement::ReproducibleConfig);
            } else {
                refusals.push(TrainingAttemptRefusal::NonReproducibleConfigRefused);
            }
        }
    }

    // A present, uncontaminated curated dataset bundle.
    match &input.dataset {
        None => refusals.push(TrainingAttemptRefusal::MissingCuratedDatasetBundle),
        Some(d) => {
            satisfied.push(TrainingAttemptRequirement::CuratedDatasetBundle);
            if d.contaminated {
                refusals.push(TrainingAttemptRefusal::ContaminatedDatasetRefused);
            } else {
                satisfied.push(TrainingAttemptRequirement::DatasetContaminationClean);
            }
        }
    }

    // A hash-pinned baseline.
    match &input.baseline {
        Some(_) => satisfied.push(TrainingAttemptRequirement::BaselineArtifact),
        None => refusals.push(TrainingAttemptRefusal::MissingBaselineArtifact),
    }

    // A present, non-leaking holdout bundle.
    match &input.holdout {
        None => refusals.push(TrainingAttemptRefusal::MissingHoldoutBundle),
        Some(h) if !h.holdout_present => {
            refusals.push(TrainingAttemptRefusal::MissingHoldoutBundle)
        }
        Some(h) => {
            satisfied.push(TrainingAttemptRequirement::HoldoutBundle);
            if h.leaked {
                refusals.push(TrainingAttemptRefusal::HoldoutLeakageRefused);
            } else {
                satisfied.push(TrainingAttemptRequirement::HoldoutLeakageClean);
            }
        }
    }

    // A hash-pinned rollback artifact.
    match &input.rollback {
        Some(_) => satisfied.push(TrainingAttemptRequirement::RollbackArtifact),
        None => refusals.push(TrainingAttemptRefusal::MissingRollbackArtifact),
    }

    // An affirmative authority-drift check.
    if input.authority_drift.is_clean() {
        satisfied.push(TrainingAttemptRequirement::AuthorityDriftClean);
    } else {
        refusals.push(TrainingAttemptRefusal::AuthorityDriftRefused);
    }

    Prerequisites {
        gate_decision: gate.decision.tag(),
        gate_allowed,
        satisfied,
        refusals,
    }
}

fn build_plan(input: &TrainingAttemptInput, pre: &Prerequisites) -> TrainingAttemptPlan {
    TrainingAttemptPlan {
        schema: SCHEMA,
        mode: input.mode,
        gate_decision: pre.gate_decision,
        gate_allowed: pre.gate_allowed,
        config_hash: input.run_config.as_ref().map(|c| c.config_hash.clone()),
        dataset_hash: input
            .dataset
            .as_ref()
            .map(|d| d.curated_corpus_hash.clone()),
        baseline_hash: input.baseline.as_ref().map(|b| b.baseline_hash.clone()),
        holdout_hash: input.holdout.as_ref().map(|h| h.holdout_hash.clone()),
        rollback_hash: input.rollback.as_ref().map(|r| r.rollback_hash.clone()),
        missing: pre.refusals.clone(),
        would_be_allowed: pre.refusals.is_empty(),
        touches_weights: ATTEMPT_CREATES_ACCEPTED_MODEL,
        boundary: TrainingAttemptBoundary::inert(),
    }
}

/// Run the gated, deterministic training-attempt harness over `input`. A `DryRunOnly` invocation
/// always prepares a plan (touching no weights, producing no candidate). An `AuthorizedLocalAttempt`
/// prepares a `CandidateOnly` artifact ONLY when the consumed gate is allowed, a separate explicit
/// operator authorization is present, and every reproducibility prerequisite is satisfied; otherwise
/// it is refused with the full set of reasons. Never promotes, deploys, or mutates weights.
pub fn run_training_attempt(input: &TrainingAttemptInput) -> TrainingAttemptReceipt {
    let pre = check_prerequisites(input);

    let (outcome, plan, candidate, refusals): (
        TrainingAttemptOutcome,
        Option<TrainingAttemptPlan>,
        Option<TrainingCandidateArtifact>,
        Vec<TrainingAttemptRefusal>,
    ) = match input.mode {
        // A dry run NEVER touches weights and NEVER produces a candidate. It always succeeds at
        // preparing a plan that documents what a real attempt would still need.
        TrainingAttemptMode::DryRunOnly => (
            TrainingAttemptOutcome::DryRunPlanPrepared,
            Some(build_plan(input, &pre)),
            None,
            Vec::new(),
        ),
        // An authorized attempt prepares a candidate ONLY when every prerequisite holds.
        TrainingAttemptMode::AuthorizedLocalAttempt => {
            if pre.refusals.is_empty() {
                // Both keys turned + every reproducibility prerequisite present. The unwraps are
                // sound: an empty refusal set implies all three artifacts are `Some`.
                let candidate = build_candidate(
                    input
                        .run_config
                        .as_ref()
                        .expect("config present when allowed"),
                    input
                        .dataset
                        .as_ref()
                        .expect("dataset present when allowed"),
                    input
                        .baseline
                        .as_ref()
                        .expect("baseline present when allowed"),
                );
                (
                    TrainingAttemptOutcome::CandidatePrepared,
                    Some(build_plan(input, &pre)),
                    Some(candidate),
                    Vec::new(),
                )
            } else {
                (
                    TrainingAttemptOutcome::AttemptRefused,
                    None,
                    None,
                    pre.refusals.clone(),
                )
            }
        }
    };

    TrainingAttemptReceipt {
        schema: SCHEMA,
        mode: input.mode,
        outcome,
        gate_decision: pre.gate_decision,
        gate_allowed: pre.gate_allowed,
        satisfied: pre.satisfied,
        refusals,
        plan,
        candidate,
        promotes_model: ATTEMPT_CREATES_ACCEPTED_MODEL,
        deploys_model: ATTEMPT_CREATES_ACCEPTED_MODEL,
        replaces_baseline: ATTEMPT_CREATES_ACCEPTED_MODEL,
        creates_evidence: ATTEMPT_CREATES_ACCEPTED_MODEL,
        creates_memory: ATTEMPT_CREATES_ACCEPTED_MODEL,
        grants_authority: ATTEMPT_CREATES_ACCEPTED_MODEL,
        modifies_weights: ATTEMPT_CREATES_ACCEPTED_MODEL,
        training_justified: ATTEMPT_CREATES_ACCEPTED_MODEL,
        boundary: TrainingAttemptBoundary::inert(),
    }
}

/// The attempt receipt serialized to canonical JSON (for an operator to record an invocation).
pub fn run_training_attempt_json(input: &TrainingAttemptInput) -> String {
    serde_json::to_string(&run_training_attempt(input))
        .expect("training-attempt receipt serializes")
}

/// What can go wrong verifying a serialized training-attempt artifact.
#[derive(Debug, PartialEq, Eq)]
pub enum TrainingAttemptError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the receipt from the SAME input and byte-compare against `candidate`. The receipt is
/// `Serialize` but never `Deserialize`: a serialized receipt is NOT trusted as input — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_training_attempt_receipt_json(
    input: &TrainingAttemptInput,
    candidate: &str,
) -> Result<(), TrainingAttemptError> {
    if candidate == run_training_attempt_json(input) {
        Ok(())
    } else {
        Err(TrainingAttemptError::ReplayMismatch)
    }
}

// --- building a REAL allowed/denied TRAIN-GATE-0 input (the SCORE-0 -> ... -> TRAIN-GATE chain) ---

/// Produce a REAL FAIL-0 [`ModelNeedCandidate`] by running the REAL [`detect_failures`] over `n`
/// repeats of a real SCORE-0 failure observation (mirrors the gate's own chain construction).
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

/// A battery that yields `training_candidate_only` (two residual clean failures).
fn candidate_battery(failures: &[FailureObservation]) -> ModelEvalBattery {
    ModelEvalBattery::new(vec![
        residual_run(reading(failures)),
        residual_run(reading(failures)),
    ])
}

// gate-level receipts (S6 input types) used to build an ALLOWED gate input.
fn gate_op_auth() -> OperatorAuthorizationReceipt {
    OperatorAuthorizationReceipt {
        operator: "operator".to_string(),
        attempt_scope: "local-finetune-attempt".to_string(),
    }
}

fn gate_dataset() -> DatasetReadinessReceipt {
    DatasetReadinessReceipt {
        curated_corpus_hash: "curated-corpus-hash".to_string(),
        item_count: 2,
    }
}

fn gate_clean_holdout() -> HoldoutReadinessReceipt {
    HoldoutReadinessReceipt {
        holdout_present: true,
        contaminated: false,
        holdout_hash: "holdout-hash".to_string(),
    }
}

fn gate_clean_contamination() -> ContaminationReportReceipt {
    ContaminationReportReceipt {
        memorization_leakage: false,
        report_hash: "contamination-report-hash".to_string(),
    }
}

fn gate_rollback() -> RollbackPlanReceipt {
    RollbackPlanReceipt {
        rollback_target: "pre-train-snapshot".to_string(),
        verified: true,
    }
}

fn gate_prod_safety() -> ProductionSafetyPlanReceipt {
    ProductionSafetyPlanReceipt {
        plan_id: "production-safety-plan-0".to_string(),
    }
}

/// A REAL TRAIN-GATE-0 input that the gate evaluates to `TrainingAttemptAllowed`.
fn allowed_gate_input(failures: &[FailureObservation]) -> TrainingGateInput {
    TrainingGateInput {
        eval: Some(candidate_battery(failures)),
        operator_authorization: Some(gate_op_auth()),
        dataset: Some(gate_dataset()),
        holdout: Some(gate_clean_holdout()),
        contamination: Some(gate_clean_contamination()),
        rollback: Some(gate_rollback()),
        production_safety: Some(gate_prod_safety()),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

// --- attempt-level artifact builders ---

fn attempt_auth() -> AttemptAuthorizationReceipt {
    AttemptAuthorizationReceipt {
        operator: "operator".to_string(),
        attempt_scope: "local-candidate-train-attempt".to_string(),
        acknowledges_candidate_only: true,
    }
}

fn deterministic_config() -> TrainingRunConfig {
    TrainingRunConfig {
        config_hash: "run-config-hash".to_string(),
        deterministic: true,
        seed: 7,
        max_steps: 100,
    }
}

fn nondeterministic_config() -> TrainingRunConfig {
    TrainingRunConfig {
        config_hash: "run-config-hash".to_string(),
        deterministic: false,
        seed: 7,
        max_steps: 100,
    }
}

fn clean_dataset() -> TrainingDatasetBundle {
    TrainingDatasetBundle {
        curated_corpus_hash: "curated-corpus-hash".to_string(),
        item_count: 2,
        contaminated: false,
    }
}

fn contaminated_dataset() -> TrainingDatasetBundle {
    TrainingDatasetBundle {
        curated_corpus_hash: "curated-corpus-hash".to_string(),
        item_count: 2,
        contaminated: true,
    }
}

fn baseline() -> TrainingBaselineArtifact {
    TrainingBaselineArtifact {
        baseline_hash: "baseline-hash".to_string(),
    }
}

fn clean_holdout() -> TrainingHoldoutBundle {
    TrainingHoldoutBundle {
        holdout_present: true,
        leaked: false,
        holdout_hash: "attempt-holdout-hash".to_string(),
    }
}

fn leaked_holdout() -> TrainingHoldoutBundle {
    TrainingHoldoutBundle {
        holdout_present: true,
        leaked: true,
        holdout_hash: "attempt-holdout-hash".to_string(),
    }
}

fn rollback() -> TrainingRollbackArtifact {
    TrainingRollbackArtifact {
        rollback_hash: "rollback-hash".to_string(),
        verified: true,
    }
}

/// A fully-authorized attempt input: allowed gate + the second key + every clean prerequisite.
fn full_attempt(failures: &[FailureObservation]) -> TrainingAttemptInput {
    TrainingAttemptInput {
        mode: TrainingAttemptMode::AuthorizedLocalAttempt,
        gate_input: allowed_gate_input(failures),
        operator_authorization: Some(attempt_auth()),
        run_config: Some(deterministic_config()),
        dataset: Some(clean_dataset()),
        baseline: Some(baseline()),
        holdout: Some(clean_holdout()),
        rollback: Some(rollback()),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

// --- the training-attempt scenario matrix (observes the real harness over constructed inputs) ---

/// One scenario cell: the OBSERVED outcome of running the real harness over a constructed input.
/// Never asserted — recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingAttemptScenarioCell {
    /// The scenario name.
    pub name: &'static str,
    /// The observed mode slug.
    pub mode: &'static str,
    /// The observed outcome slug.
    pub outcome: &'static str,
    /// The observed refusal-reason slugs.
    pub refusals: Vec<&'static str>,
    /// Whether a candidate descriptor was prepared.
    pub candidate_prepared: bool,
    /// Whether the prepared candidate (if any) is `CandidateOnly` and requires S8 evaluation.
    pub candidate_only: bool,
    /// Whether nothing was promoted, deployed, made evidence, or had weights mutated.
    pub training_still_closed: bool,
    /// A short human-readable detail.
    pub detail: String,
}

/// The fixed training-attempt scenario matrix. Every cell runs the real harness and records what it
/// observed; `training_never_opens` is the conjunction across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrainingAttemptMatrix {
    /// The schema tag.
    pub schema: &'static str,
    /// The scenario cells.
    pub scenarios: Vec<TrainingAttemptScenarioCell>,
    /// The two mode slugs.
    pub modes: [&'static str; TRAIN_ATTEMPT_MODE_COUNT],
    /// The twelve refusal-reason slugs.
    pub refusal_reasons: [&'static str; TRAIN_ATTEMPT_REFUSAL_COUNT],
    /// True iff no cell promoted, deployed, made evidence, or mutated weights.
    pub training_never_opens: bool,
    /// The inert boundary.
    pub boundary: TrainingAttemptBoundary,
}

impl TrainingAttemptMatrix {
    /// Find a scenario cell by name.
    pub fn scenario(&self, name: &str) -> Option<&TrainingAttemptScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn closed_for(report: &TrainingAttemptReceipt) -> bool {
    let candidate_closed = match &report.candidate {
        None => true,
        Some(c) => {
            !c.promoted
                && !c.deployed
                && !c.is_evidence
                && !c.creates_memory
                && !c.grants_authority
                && !c.replaces_baseline
                && c.requires_s8_evaluation
                && c.acceptance == CandidateAcceptance::CandidateOnly
        }
    };
    !report.promotes_model
        && !report.deploys_model
        && !report.replaces_baseline
        && !report.creates_evidence
        && !report.creates_memory
        && !report.grants_authority
        && !report.modifies_weights
        && !report.training_justified
        && report.boundary.all_inert()
        && candidate_closed
}

fn attempt_cell(name: &'static str, input: TrainingAttemptInput) -> TrainingAttemptScenarioCell {
    let report = run_training_attempt(&input);
    let candidate_only = report
        .candidate
        .as_ref()
        .map(|c| {
            c.acceptance == CandidateAcceptance::CandidateOnly
                && c.requires_s8_evaluation
                && !c.promoted
                && !c.deployed
                && !c.is_evidence
                && !c.replaces_baseline
        })
        .unwrap_or(false);
    TrainingAttemptScenarioCell {
        name,
        mode: report.mode.tag(),
        outcome: report.outcome.tag(),
        refusals: report.refusals.iter().map(|r| r.tag()).collect(),
        candidate_prepared: report.candidate.is_some(),
        candidate_only,
        training_still_closed: closed_for(&report),
        detail: report.outcome.tag().to_string(),
    }
}

/// The serialized-receipt tamper cell: tamper a real (allowed) attempt receipt JSON and observe the
/// re-derive verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous.
fn tamper_cell(failures: &[FailureObservation]) -> TrainingAttemptScenarioCell {
    let input = full_attempt(failures);
    let canonical = run_training_attempt_json(&input);
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_training_attempt_receipt_json(&input, &tampered).is_err()
        && verify_training_attempt_receipt_json(&input, &canonical).is_ok();
    let report = run_training_attempt(&input);
    TrainingAttemptScenarioCell {
        name: "serialized_training_attempt_tamper_refused",
        mode: report.mode.tag(),
        outcome: report.outcome.tag(),
        refusals: if refused {
            vec!["training_attempt_serialized_tamper_refused"]
        } else {
            vec!["VACUOUS"]
        },
        candidate_prepared: report.candidate.is_some(),
        candidate_only: report
            .candidate
            .as_ref()
            .map(|c| c.acceptance == CandidateAcceptance::CandidateOnly)
            .unwrap_or(false),
        training_still_closed: closed_for(&report) && refused,
        detail: if refused {
            "training_attempt_serialized_tamper_refused".to_string()
        } else {
            "VACUOUS: attempt receipt verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 20-scenario training-attempt matrix from the REAL harness over constructed inputs.
pub fn training_attempt_matrix() -> TrainingAttemptMatrix {
    // Derive the SCORE-0 failure set ONCE; every gate input reuses it (no per-build rebuild).
    let failures = verifier_score_matrix().failures;

    let scenarios = vec![
        // 1. A dry run with full inputs prepares a plan (no candidate, no weights).
        attempt_cell(
            "dry_run_plan_created",
            TrainingAttemptInput {
                mode: TrainingAttemptMode::DryRunOnly,
                ..full_attempt(&failures)
            },
        ),
        // 2. Authorized attempt, gate not allowed (closed gate input).
        attempt_cell(
            "missing_training_gate_allow_denied",
            TrainingAttemptInput {
                gate_input: TrainingGateInput::closed_by_default(),
                ..full_attempt(&failures)
            },
        ),
        // 3. Authorized attempt, missing the second key.
        attempt_cell(
            "missing_operator_authorization_denied",
            TrainingAttemptInput {
                operator_authorization: None,
                ..full_attempt(&failures)
            },
        ),
        // 4. Gate allowed but no operator authorization -> allow alone is insufficient.
        attempt_cell(
            "allowed_without_operator_authorization_denied",
            TrainingAttemptInput {
                operator_authorization: None,
                ..full_attempt(&failures)
            },
        ),
        // 5. Operator authorization but gate not allowed -> auth alone is insufficient.
        attempt_cell(
            "operator_authorization_without_allowed_gate_denied",
            TrainingAttemptInput {
                gate_input: TrainingGateInput::closed_by_default(),
                ..full_attempt(&failures)
            },
        ),
        // 6. Missing run config.
        attempt_cell(
            "missing_run_config_denied",
            TrainingAttemptInput {
                run_config: None,
                ..full_attempt(&failures)
            },
        ),
        // 7. Missing dataset bundle.
        attempt_cell(
            "missing_dataset_bundle_denied",
            TrainingAttemptInput {
                dataset: None,
                ..full_attempt(&failures)
            },
        ),
        // 8. Missing baseline artifact.
        attempt_cell(
            "missing_baseline_artifact_denied",
            TrainingAttemptInput {
                baseline: None,
                ..full_attempt(&failures)
            },
        ),
        // 9. Missing holdout bundle.
        attempt_cell(
            "missing_holdout_bundle_denied",
            TrainingAttemptInput {
                holdout: None,
                ..full_attempt(&failures)
            },
        ),
        // 10. Missing rollback artifact.
        attempt_cell(
            "missing_rollback_artifact_denied",
            TrainingAttemptInput {
                rollback: None,
                ..full_attempt(&failures)
            },
        ),
        // 11. Contaminated dataset.
        attempt_cell(
            "contaminated_dataset_denied",
            TrainingAttemptInput {
                dataset: Some(contaminated_dataset()),
                ..full_attempt(&failures)
            },
        ),
        // 12. Holdout leakage.
        attempt_cell(
            "holdout_leakage_denied",
            TrainingAttemptInput {
                holdout: Some(leaked_holdout()),
                ..full_attempt(&failures)
            },
        ),
        // 13. Authority drift.
        attempt_cell(
            "authority_drift_denied",
            TrainingAttemptInput {
                authority_drift: AuthorityDriftCheck::drifted(),
                ..full_attempt(&failures)
            },
        ),
        // 14. Non-reproducible config.
        attempt_cell(
            "non_reproducible_config_denied",
            TrainingAttemptInput {
                run_config: Some(nondeterministic_config()),
                ..full_attempt(&failures)
            },
        ),
        // 15. Fully authorized attempt -> CandidateOnly.
        attempt_cell("authorized_attempt_candidate_only", full_attempt(&failures)),
        // 16-19. The candidate's forbidden flags + the S8 requirement (same full run, named).
        attempt_cell("candidate_not_promoted", full_attempt(&failures)),
        attempt_cell("candidate_not_deployed", full_attempt(&failures)),
        attempt_cell("candidate_not_evidence", full_attempt(&failures)),
        attempt_cell("candidate_requires_s8_evaluation", full_attempt(&failures)),
        // 20. Serialized receipt tamper refused.
        tamper_cell(&failures),
    ];

    let training_never_opens = scenarios.iter().all(|c| c.training_still_closed);
    TrainingAttemptMatrix {
        schema: SCHEMA,
        scenarios,
        modes: TRAIN_ATTEMPT_MODE_NAMES,
        refusal_reasons: TRAIN_ATTEMPT_REFUSAL_NAMES,
        training_never_opens,
        boundary: TrainingAttemptBoundary::inert(),
    }
}

/// The training-attempt matrix serialized to canonical JSON.
pub fn training_attempt_matrix_json() -> String {
    serde_json::to_string(&training_attempt_matrix()).expect("training-attempt matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`.
pub fn verify_training_attempt_matrix_json(candidate: &str) -> Result<(), TrainingAttemptError> {
    if candidate == training_attempt_matrix_json() {
        Ok(())
    } else {
        Err(TrainingAttemptError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failures() -> Vec<FailureObservation> {
        verifier_score_matrix().failures
    }

    fn has(report: &TrainingAttemptReceipt, r: TrainingAttemptRefusal) -> bool {
        report.refusals.contains(&r)
    }

    #[test]
    fn attempt_consumes_the_real_train_gate_report() {
        let f = failures();
        // A full attempt -> the harness observes the REAL gate as allowed (derived, not handed in).
        let allowed = run_training_attempt(&full_attempt(&f));
        assert!(allowed.gate_allowed);
        assert_eq!(allowed.gate_decision, "training_attempt_allowed");
        // A closed gate input -> the harness observes the gate deny (also derived).
        let denied = run_training_attempt(&TrainingAttemptInput {
            gate_input: TrainingGateInput::closed_by_default(),
            ..full_attempt(&f)
        });
        assert!(!denied.gate_allowed);
        assert_eq!(denied.gate_decision, "training_attempt_denied");
        assert!(has(
            &denied,
            TrainingAttemptRefusal::MissingTrainingGateAllow
        ));
    }

    #[test]
    fn dry_run_builds_a_plan_without_touching_weights() {
        let f = failures();
        let report = run_training_attempt(&TrainingAttemptInput {
            mode: TrainingAttemptMode::DryRunOnly,
            ..full_attempt(&f)
        });
        assert_eq!(report.mode, TrainingAttemptMode::DryRunOnly);
        assert_eq!(report.outcome, TrainingAttemptOutcome::DryRunPlanPrepared);
        let plan = report.plan.as_ref().expect("dry run prepares a plan");
        assert!(!plan.touches_weights);
        assert!(!report.modifies_weights);
        // A dry run produces NO candidate.
        assert!(report.candidate.is_none());
        // With full inputs, the plan reports a real attempt would be allowed.
        assert!(plan.would_be_allowed);
        assert!(plan.missing.is_empty());
    }

    #[test]
    fn dry_run_records_what_a_real_attempt_would_need() {
        let f = failures();
        // A dry run over the closed input still succeeds at preparing a plan, and the plan enumerates
        // what is missing — proving the harness can SAFELY prepare/refuse.
        let report = run_training_attempt(&TrainingAttemptInput::closed_dry_run());
        assert_eq!(report.outcome, TrainingAttemptOutcome::DryRunPlanPrepared);
        let plan = report.plan.as_ref().expect("dry run prepares a plan");
        assert!(!plan.would_be_allowed);
        assert!(plan
            .missing
            .contains(&TrainingAttemptRefusal::MissingTrainingGateAllow));
        assert!(plan
            .missing
            .contains(&TrainingAttemptRefusal::MissingExplicitOperatorAuthorization));
        // Still no candidate, still no weights touched.
        assert!(report.candidate.is_none());
        assert!(!report.modifies_weights);
        let _ = f;
    }

    #[test]
    fn authorized_attempt_requires_both_gate_allow_and_operator_authorization() {
        let f = failures();
        // Both keys + every prerequisite -> a candidate is prepared.
        let ok = run_training_attempt(&full_attempt(&f));
        assert_eq!(ok.outcome, TrainingAttemptOutcome::CandidatePrepared);
        assert!(ok.candidate.is_some());
        assert!(ok.refusals.is_empty());
        assert_eq!(ok.satisfied.len(), TrainingAttemptRequirement::ALL.len());
    }

    #[test]
    fn allowed_gate_without_operator_authorization_is_refused() {
        let f = failures();
        // Gate is allowed, but the SECOND key (operator authorization) is absent.
        let report = run_training_attempt(&TrainingAttemptInput {
            operator_authorization: None,
            ..full_attempt(&f)
        });
        assert!(report.gate_allowed);
        assert_eq!(report.outcome, TrainingAttemptOutcome::AttemptRefused);
        assert!(report.candidate.is_none());
        assert!(has(
            &report,
            TrainingAttemptRefusal::MissingExplicitOperatorAuthorization
        ));
    }

    #[test]
    fn operator_authorization_without_allowed_gate_is_refused() {
        let f = failures();
        // The second key is present, but the gate is NOT allowed.
        let report = run_training_attempt(&TrainingAttemptInput {
            gate_input: TrainingGateInput::closed_by_default(),
            ..full_attempt(&f)
        });
        assert!(report.operator_was_authorized());
        assert!(!report.gate_allowed);
        assert_eq!(report.outcome, TrainingAttemptOutcome::AttemptRefused);
        assert!(report.candidate.is_none());
        assert!(has(
            &report,
            TrainingAttemptRefusal::MissingTrainingGateAllow
        ));
    }

    #[test]
    fn authorized_attempt_with_all_prerequisites_prepares_a_candidate_only_artifact() {
        let f = failures();
        let report = run_training_attempt(&full_attempt(&f));
        let candidate = report.candidate.as_ref().expect("a candidate is prepared");
        assert_eq!(candidate.acceptance, CandidateAcceptance::CandidateOnly);
        assert!(candidate.requires_s8_evaluation);
        // Hash-pinned and reproducible: the descriptor is derived from baseline/dataset/config.
        assert!(candidate.candidate_hash.starts_with("cand-"));
        assert_eq!(candidate.baseline_hash, "baseline-hash");
    }

    #[test]
    fn candidate_is_not_promoted_deployed_or_evidence() {
        let f = failures();
        let report = run_training_attempt(&full_attempt(&f));
        let candidate = report.candidate.as_ref().expect("a candidate is prepared");
        assert!(!candidate.promoted);
        assert!(!candidate.deployed);
        assert!(!candidate.is_evidence);
        assert!(!candidate.creates_memory);
        assert!(!candidate.grants_authority);
        // The receipt-level forbidden flags are inert too.
        assert!(!report.promotes_model);
        assert!(!report.deploys_model);
        assert!(!report.creates_evidence);
        assert!(report.boundary.all_inert());
    }

    #[test]
    fn candidate_does_not_replace_baseline() {
        let f = failures();
        let report = run_training_attempt(&full_attempt(&f));
        let candidate = report.candidate.as_ref().expect("a candidate is prepared");
        assert!(!candidate.replaces_baseline);
        assert!(!report.replaces_baseline);
        // The baseline lineage is recorded but the baseline is not overwritten.
        assert_eq!(candidate.baseline_hash, "baseline-hash");
    }

    #[test]
    fn missing_each_prerequisite_is_refused() {
        let f = failures();
        let cfg = run_training_attempt(&TrainingAttemptInput {
            run_config: None,
            ..full_attempt(&f)
        });
        assert!(has(&cfg, TrainingAttemptRefusal::MissingTrainingRunConfig));

        let ds = run_training_attempt(&TrainingAttemptInput {
            dataset: None,
            ..full_attempt(&f)
        });
        assert!(has(
            &ds,
            TrainingAttemptRefusal::MissingCuratedDatasetBundle
        ));

        let bl = run_training_attempt(&TrainingAttemptInput {
            baseline: None,
            ..full_attempt(&f)
        });
        assert!(has(&bl, TrainingAttemptRefusal::MissingBaselineArtifact));

        let ho = run_training_attempt(&TrainingAttemptInput {
            holdout: None,
            ..full_attempt(&f)
        });
        assert!(has(&ho, TrainingAttemptRefusal::MissingHoldoutBundle));

        let rb = run_training_attempt(&TrainingAttemptInput {
            rollback: None,
            ..full_attempt(&f)
        });
        assert!(has(&rb, TrainingAttemptRefusal::MissingRollbackArtifact));
        // Every refused authorized attempt prepares NO candidate.
        for r in [&cfg, &ds, &bl, &ho, &rb] {
            assert_eq!(r.outcome, TrainingAttemptOutcome::AttemptRefused);
            assert!(r.candidate.is_none());
        }
    }

    #[test]
    fn contaminated_dataset_and_holdout_leakage_are_refused() {
        let f = failures();
        let contaminated = run_training_attempt(&TrainingAttemptInput {
            dataset: Some(contaminated_dataset()),
            ..full_attempt(&f)
        });
        assert!(has(
            &contaminated,
            TrainingAttemptRefusal::ContaminatedDatasetRefused
        ));
        assert!(contaminated.candidate.is_none());

        let leaked = run_training_attempt(&TrainingAttemptInput {
            holdout: Some(leaked_holdout()),
            ..full_attempt(&f)
        });
        assert!(has(&leaked, TrainingAttemptRefusal::HoldoutLeakageRefused));
        assert!(leaked.candidate.is_none());
    }

    #[test]
    fn non_reproducible_config_is_refused() {
        let f = failures();
        let report = run_training_attempt(&TrainingAttemptInput {
            run_config: Some(nondeterministic_config()),
            ..full_attempt(&f)
        });
        assert!(has(
            &report,
            TrainingAttemptRefusal::NonReproducibleConfigRefused
        ));
        assert!(report.candidate.is_none());
    }

    #[test]
    fn authority_drift_is_refused() {
        let f = failures();
        let drifted = run_training_attempt(&TrainingAttemptInput {
            authority_drift: AuthorityDriftCheck::drifted(),
            ..full_attempt(&f)
        });
        assert!(has(&drifted, TrainingAttemptRefusal::AuthorityDriftRefused));
        // An unchecked drift state is ALSO not clean (closed by default).
        let unchecked = run_training_attempt(&TrainingAttemptInput {
            authority_drift: AuthorityDriftCheck::unchecked(),
            ..full_attempt(&f)
        });
        assert!(has(
            &unchecked,
            TrainingAttemptRefusal::AuthorityDriftRefused
        ));
    }

    #[test]
    fn candidate_requires_s8_evaluation_and_is_not_accepted() {
        let f = failures();
        let report = run_training_attempt(&full_attempt(&f));
        let candidate = report.candidate.as_ref().expect("a candidate is prepared");
        // A candidate is CandidateOnly and MUST be evaluated later (S8) before promotion.
        assert!(candidate.requires_s8_evaluation);
        assert_eq!(candidate.acceptance, CandidateAcceptance::CandidateOnly);
        // There is exactly one acceptance state — a candidate can never be represented as accepted.
        assert_eq!(candidate.acceptance.tag(), "candidate_only");
    }

    #[test]
    fn p12_training_justified_remains_false_even_when_candidate_prepared() {
        let f = failures();
        let report = run_training_attempt(&full_attempt(&f));
        assert_eq!(report.outcome, TrainingAttemptOutcome::CandidatePrepared);
        assert!(!report.training_justified);
        // The deeper P12 gate is unaffected by a prepared candidate.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn matrix_has_the_twenty_named_scenarios() {
        let matrix = training_attempt_matrix();
        assert_eq!(matrix.scenarios.len(), TRAIN_ATTEMPT_SCENARIO_COUNT);
        for name in [
            "dry_run_plan_created",
            "missing_training_gate_allow_denied",
            "missing_operator_authorization_denied",
            "allowed_without_operator_authorization_denied",
            "operator_authorization_without_allowed_gate_denied",
            "missing_run_config_denied",
            "missing_dataset_bundle_denied",
            "missing_baseline_artifact_denied",
            "missing_holdout_bundle_denied",
            "missing_rollback_artifact_denied",
            "contaminated_dataset_denied",
            "holdout_leakage_denied",
            "authority_drift_denied",
            "non_reproducible_config_denied",
            "authorized_attempt_candidate_only",
            "candidate_not_promoted",
            "candidate_not_deployed",
            "candidate_not_evidence",
            "candidate_requires_s8_evaluation",
            "serialized_training_attempt_tamper_refused",
        ] {
            assert!(
                matrix.scenario(name).is_some(),
                "scenario {name} is missing"
            );
        }
        // Only the authorized full-input cells prepare a candidate; training never opens anywhere.
        assert!(matrix.training_never_opens);
        let allowed = matrix
            .scenario("authorized_attempt_candidate_only")
            .expect("present");
        assert!(allowed.candidate_prepared);
        assert!(allowed.candidate_only);
        let dry = matrix.scenario("dry_run_plan_created").expect("present");
        assert!(!dry.candidate_prepared);
    }

    #[test]
    fn every_matrix_cell_keeps_training_closed() {
        let matrix = training_attempt_matrix();
        for cell in &matrix.scenarios {
            assert!(
                cell.training_still_closed,
                "cell {} opened training",
                cell.name
            );
        }
        // The tamper cell genuinely refused (its slug is recorded, not VACUOUS).
        let tamper = matrix
            .scenario("serialized_training_attempt_tamper_refused")
            .expect("tamper cell present");
        assert!(tamper
            .refusals
            .contains(&"training_attempt_serialized_tamper_refused"));
    }

    #[test]
    fn mode_and_refusal_counts_match_enums() {
        assert_eq!(TrainingAttemptMode::ALL.len(), TRAIN_ATTEMPT_MODE_COUNT);
        assert_eq!(
            TrainingAttemptRefusal::ALL.len(),
            TRAIN_ATTEMPT_REFUSAL_COUNT
        );
        assert_eq!(TRAIN_ATTEMPT_MODE_NAMES.len(), TRAIN_ATTEMPT_MODE_COUNT);
        assert_eq!(
            TRAIN_ATTEMPT_REFUSAL_NAMES.len(),
            TRAIN_ATTEMPT_REFUSAL_COUNT
        );
        for (m, name) in TrainingAttemptMode::ALL
            .iter()
            .zip(TRAIN_ATTEMPT_MODE_NAMES)
        {
            assert_eq!(m.tag(), name);
        }
        for (r, name) in TrainingAttemptRefusal::ALL
            .iter()
            .zip(TRAIN_ATTEMPT_REFUSAL_NAMES)
        {
            assert_eq!(r.tag(), name);
        }
    }

    #[test]
    fn receipt_is_deterministic_and_re_derives_refusing_tampering() {
        let f = failures();
        let input = full_attempt(&f);
        let canonical = run_training_attempt_json(&input);
        // Deterministic.
        assert_eq!(canonical, run_training_attempt_json(&full_attempt(&f)));
        // Canonical verifies; a tampered (non-equal) artifact is refused.
        assert!(verify_training_attempt_receipt_json(&input, &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_training_attempt_receipt_json(&input, &tampered),
            Err(TrainingAttemptError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_re_derives_refusing_tampering() {
        let canonical = training_attempt_matrix_json();
        assert!(verify_training_attempt_matrix_json(&canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_training_attempt_matrix_json(&tampered),
            Err(TrainingAttemptError::ReplayMismatch)
        );
    }

    #[test]
    fn candidate_hash_is_reproducible_and_lineage_bound() {
        let f = failures();
        let a = run_training_attempt(&full_attempt(&f));
        let b = run_training_attempt(&full_attempt(&f));
        let ca = a.candidate.as_ref().expect("candidate");
        let cb = b.candidate.as_ref().expect("candidate");
        // Same inputs -> same descriptor (reproducible, hash-pinned).
        assert_eq!(ca.candidate_hash, cb.candidate_hash);
        // A different baseline -> a different descriptor (lineage-bound).
        let mut other = full_attempt(&f);
        other.baseline = Some(TrainingBaselineArtifact {
            baseline_hash: "other-baseline-hash".to_string(),
        });
        let c = run_training_attempt(&other);
        let cc = c.candidate.as_ref().expect("candidate");
        assert_ne!(ca.candidate_hash, cc.candidate_hash);
    }
}
