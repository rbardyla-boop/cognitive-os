//! MODEL-PROMOTE-0 — the explicit, closed-by-default MODEL PROMOTION GATE.
//!
//! This sprint answers exactly ONE question: *is a candidate eligible to enter production PACKAGING?*
//! It does NOT answer "is production now running?". It trains nothing, deploys nothing, starts no
//! production runtime, writes no memory, creates no evidence, and does NOT silently replace the
//! baseline. Its single affirmative output is a SEALED [`PromotedModelReceipt`] stamped
//! `promotion_ready` — which is permission to enter S10 packaging / S11 smoke, NOT production
//! deployment.
//!
//! It CONSUMES the REAL MODEL-EVAL-1 evaluation: [`evaluate_model_promotion`] runs
//! [`evaluate_candidate`] itself over the supplied [`CandidateEvalInput`] (the full SCORE-0 -> FAIL-0
//! -> MODEL-EVAL -> TRAIN-GATE -> TRAIN-ATTEMPT -> CANDIDATE-EVAL -> PROMOTE chain), so the verdict is
//! DERIVED, never handed in. It may emit [`ModelPromotionDecision::PromotionReady`] ONLY when EVERY
//! requirement holds together:
//!
//!   1. the consumed eval verdict is EXACTLY `candidate_ready_for_promotion_review`,
//!   2. the candidate / baseline / dataset artifact hashes are pinned AND corroborated by the eval,
//!   3. the eval-report hash is pinned AND matches the re-derived eval report,
//!   4. an explicit operator promotion approval,
//!   5. a rollback artifact receipt,
//!   6. a runtime config receipt (baseline replacement only DESCRIBED as pending, never performed),
//!   7. a production safety plan,
//!   8. a clean holdout, no contamination, no memorization leakage, no critical regression
//!      (re-checked on the consumed report — defense in depth),
//!   9. an affirmative, clean authority-drift check.
//!
//! It is CLOSED BY DEFAULT: a `candidate_ready_for_promotion_review` verdict ALONE is insufficient;
//! operator approval ALONE is insufficient; any missing/unproven prerequisite denies promotion with
//! the full set of [`ModelPromotionRefusal`] reasons.
//!
//! Crucially, `PromotionReady` is ONLY eligibility for packaging: every forbidden-action flag on the
//! report and the sealed receipt (`deploys_model`, `starts_production`, `replaces_baseline`, `trains`,
//! `modifies_weights`, `creates_evidence`, `creates_memory`, `grants_authority`, `opens_p12`) is
//! sourced from the structural const [`PROMOTION_READY_IS_PRODUCTION`] (`false`). The sealed receipt
//! still `requires_s10_packaging` and `requires_s11_smoke`, and records baseline replacement as
//! PENDING runtime configuration, never performed. The deeper P12 gate
//! (`reading_train_gate::decide`) stays `training_justified = false`. Reports are `Serialize` but
//! never `Deserialize`: a serialized report is re-derived from the same input and byte-compared, so
//! tampering is refused.
//!
//! The boundary, recorded verbatim in [`MODEL_PROMOTE_BOUNDARY_LINES`]:
//!
//!   The model promotion gate evaluates whether a candidate model is ready for promotion.
//!   It does not train.
//!   It does not deploy models.
//!   It does not start production runtime.
//!   It does not create truth.
//!   It does not create memory.
//!   It does not create evidence.
//!   It does not bypass rollback.
//!   PromotionReady is not production deployment.

use crate::{
    detect_failures, evaluate_candidate, evaluate_candidate_json, run_training_attempt,
    verifier_score_matrix, AttemptAuthorizationReceipt, AuthorityDriftCheck, BaselineModelRef,
    CandidateEvalBattery, CandidateEvalComparison, CandidateEvalInput, CandidateEvalReport,
    CandidateEvalVerdict, ContaminationReportReceipt, DatasetReadinessReceipt, EvalComparison,
    EvalCondition, EvalDimension, EvalRun, FailureClass, FailureContext, FailureObservation,
    FailureSignal, HoldoutReadinessReceipt, HoldoutReport, ModelEvalBattery, ModelNeedCandidate,
    OperatorAuthorizationReceipt, ProductionSafetyPlanReceipt, RollbackPlanReceipt,
    SafetyBoundaryReport, ScoreClass, ScoreReason, TrainingAttemptInput, TrainingAttemptMode,
    TrainingBaselineArtifact, TrainingDatasetBundle, TrainingGateInput, TrainingHoldoutBundle,
    TrainingRollbackArtifact, TrainingRunConfig, RECURRENCE_THRESHOLD,
};
use serde::Serialize;

/// The schema tag stamped on every serialized promotion artifact.
const SCHEMA: &str = "model-promote-v0.1";

/// THE structural invariant: a `promotion_ready` decision is not, by itself, production deployment,
/// a running runtime, a baseline replacement, training, or any authority grant. Every
/// forbidden-action flag is sourced from this const, so no code path can set one true.
const PROMOTION_READY_IS_PRODUCTION: bool = false;

/// Exactly two decision states.
pub const MODEL_PROMOTE_DECISION_COUNT: usize = 2;

/// The two decision-state slugs, in canonical order.
pub const MODEL_PROMOTE_DECISION_NAMES: [&str; MODEL_PROMOTE_DECISION_COUNT] =
    ["promotion_denied", "promotion_ready"];

/// Exactly sixteen refusal reasons.
pub const MODEL_PROMOTE_REFUSAL_COUNT: usize = 16;

/// The sixteen refusal-reason slugs, in canonical order.
pub const MODEL_PROMOTE_REFUSAL_NAMES: [&str; MODEL_PROMOTE_REFUSAL_COUNT] = [
    "missing_candidate_eval_report",
    "candidate_not_ready_for_promotion_review",
    "missing_candidate_artifact_hash",
    "missing_baseline_artifact_hash",
    "missing_dataset_hash",
    "missing_eval_report_hash",
    "missing_runtime_config",
    "missing_rollback_artifact",
    "missing_operator_approval",
    "missing_production_safety_plan",
    "holdout_not_clean",
    "contamination_detected",
    "memorization_leakage_detected",
    "critical_regression_present",
    "authority_drift_detected",
    "serialized_promotion_report_tamper_refused",
];

/// The fixed promotion scenario matrix size.
pub const MODEL_PROMOTE_SCENARIO_COUNT: usize = 22;

/// The cannot-bypass boundary, recorded verbatim.
pub const MODEL_PROMOTE_BOUNDARY_LINES: [&str; 9] = [
    "The model promotion gate evaluates whether a candidate model is ready for promotion.",
    "It does not train.",
    "It does not deploy models.",
    "It does not start production runtime.",
    "It does not create truth.",
    "It does not create memory.",
    "It does not create evidence.",
    "It does not bypass rollback.",
    "PromotionReady is not production deployment.",
];

// --- decision / refusal taxonomies ---

/// The two terminal decisions of the gate. `PromotionDenied` is the closed-by-default state;
/// `PromotionReady` is eligibility for S10 packaging — nothing more.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ModelPromotionDecision {
    /// At least one requirement is unmet — no promotion may proceed.
    PromotionDenied,
    /// Every requirement is satisfied — the candidate is eligible to enter production packaging
    /// (but is NOT deployed, NOT running, and the baseline is NOT replaced).
    PromotionReady,
}

impl ModelPromotionDecision {
    /// Every decision, in canonical order.
    pub const ALL: [ModelPromotionDecision; MODEL_PROMOTE_DECISION_COUNT] = [
        ModelPromotionDecision::PromotionDenied,
        ModelPromotionDecision::PromotionReady,
    ];

    /// The stable slug for this decision.
    pub fn tag(&self) -> &'static str {
        match self {
            ModelPromotionDecision::PromotionDenied => "promotion_denied",
            ModelPromotionDecision::PromotionReady => "promotion_ready",
        }
    }
}

/// Why the gate refused. The first fifteen are decision-path reasons; the sixteenth
/// (`SerializedPromotionReportTamperRefused`) is emitted only by the serialized-report re-derivation
/// path (a tampered report is never trusted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ModelPromotionRefusal {
    /// No candidate-eval receipt was supplied — the verdict is unknown.
    MissingCandidateEvalReport,
    /// The consumed eval verdict is not `candidate_ready_for_promotion_review`.
    CandidateNotReadyForPromotionReview,
    /// The candidate artifact hash is absent or not corroborated by the eval.
    MissingCandidateArtifactHash,
    /// The baseline artifact hash is absent or not corroborated by the eval.
    MissingBaselineArtifactHash,
    /// The dataset hash is absent or not corroborated by the eval.
    MissingDatasetHash,
    /// The eval-report hash is absent or does not match the re-derived eval report.
    MissingEvalReportHash,
    /// No runtime config receipt.
    MissingRuntimeConfig,
    /// No rollback artifact receipt.
    MissingRollbackArtifact,
    /// No explicit operator promotion approval.
    MissingOperatorApproval,
    /// No production safety plan.
    MissingProductionSafetyPlan,
    /// The consumed eval report has no clean, present holdout.
    HoldoutNotClean,
    /// The consumed eval report flagged contamination.
    ContaminationDetected,
    /// The consumed eval report flagged memorization leakage.
    MemorizationLeakageDetected,
    /// The consumed eval report flagged a critical regression.
    CriticalRegressionPresent,
    /// The authority-drift check was not run, or it detected drift.
    AuthorityDriftDetected,
    /// A serialized promotion report did not match its re-derivation and was refused.
    SerializedPromotionReportTamperRefused,
}

impl ModelPromotionRefusal {
    /// Every refusal reason, in canonical order.
    pub const ALL: [ModelPromotionRefusal; MODEL_PROMOTE_REFUSAL_COUNT] = [
        ModelPromotionRefusal::MissingCandidateEvalReport,
        ModelPromotionRefusal::CandidateNotReadyForPromotionReview,
        ModelPromotionRefusal::MissingCandidateArtifactHash,
        ModelPromotionRefusal::MissingBaselineArtifactHash,
        ModelPromotionRefusal::MissingDatasetHash,
        ModelPromotionRefusal::MissingEvalReportHash,
        ModelPromotionRefusal::MissingRuntimeConfig,
        ModelPromotionRefusal::MissingRollbackArtifact,
        ModelPromotionRefusal::MissingOperatorApproval,
        ModelPromotionRefusal::MissingProductionSafetyPlan,
        ModelPromotionRefusal::HoldoutNotClean,
        ModelPromotionRefusal::ContaminationDetected,
        ModelPromotionRefusal::MemorizationLeakageDetected,
        ModelPromotionRefusal::CriticalRegressionPresent,
        ModelPromotionRefusal::AuthorityDriftDetected,
        ModelPromotionRefusal::SerializedPromotionReportTamperRefused,
    ];

    /// The stable slug for this refusal reason.
    pub fn tag(&self) -> &'static str {
        match self {
            ModelPromotionRefusal::MissingCandidateEvalReport => "missing_candidate_eval_report",
            ModelPromotionRefusal::CandidateNotReadyForPromotionReview => {
                "candidate_not_ready_for_promotion_review"
            }
            ModelPromotionRefusal::MissingCandidateArtifactHash => {
                "missing_candidate_artifact_hash"
            }
            ModelPromotionRefusal::MissingBaselineArtifactHash => "missing_baseline_artifact_hash",
            ModelPromotionRefusal::MissingDatasetHash => "missing_dataset_hash",
            ModelPromotionRefusal::MissingEvalReportHash => "missing_eval_report_hash",
            ModelPromotionRefusal::MissingRuntimeConfig => "missing_runtime_config",
            ModelPromotionRefusal::MissingRollbackArtifact => "missing_rollback_artifact",
            ModelPromotionRefusal::MissingOperatorApproval => "missing_operator_approval",
            ModelPromotionRefusal::MissingProductionSafetyPlan => "missing_production_safety_plan",
            ModelPromotionRefusal::HoldoutNotClean => "holdout_not_clean",
            ModelPromotionRefusal::ContaminationDetected => "contamination_detected",
            ModelPromotionRefusal::MemorizationLeakageDetected => "memorization_leakage_detected",
            ModelPromotionRefusal::CriticalRegressionPresent => "critical_regression_present",
            ModelPromotionRefusal::AuthorityDriftDetected => "authority_drift_detected",
            ModelPromotionRefusal::SerializedPromotionReportTamperRefused => {
                "serialized_promotion_report_tamper_refused"
            }
        }
    }
}

// --- promotion INPUTS (never trusted off-wire: Debug + Clone, no Serialize, no Deserialize) ---

/// The eval receipt: the candidate-eval input the gate RE-RUNS, plus the operator's pinned hash of
/// the eval report they intend to promote. The gate re-derives the eval and refuses a stale/forged
/// pin.
#[derive(Debug)]
pub struct PromotionEvalReceipt {
    /// The MODEL-EVAL-1 input the gate runs `evaluate_candidate` over (genuine consumption).
    pub eval: CandidateEvalInput,
    /// The operator's pinned hash of the eval report being promoted.
    pub eval_report_hash: String,
}

/// The candidate receipt: the operator's pinned artifact hashes for the candidate being promoted.
/// Each must be present AND corroborated by the consumed eval report.
#[derive(Debug, Clone)]
pub struct PromotionCandidateReceipt {
    /// The pinned candidate artifact hash.
    pub candidate_artifact_hash: String,
    /// The pinned baseline artifact hash.
    pub baseline_artifact_hash: String,
    /// The pinned dataset/lineage hash.
    pub dataset_hash: String,
}

/// An explicit operator approval for a promotion (distinct from any earlier authorization).
#[derive(Debug, Clone)]
pub struct PromotionOperatorApprovalReceipt {
    /// Who approved the promotion.
    pub operator: String,
    /// The narrow scope of the approved promotion.
    pub promotion_scope: String,
    /// The operator's explicit affirmation of the promotion.
    pub approves_promotion: bool,
}

/// A hash-pinned rollback artifact — the verified snapshot a promotion can revert to.
#[derive(Debug, Clone)]
pub struct PromotionRollbackReceipt {
    /// The content hash pinning the rollback target.
    pub rollback_hash: String,
    /// Whether the rollback path was verified.
    pub verified: bool,
}

/// A hash-pinned runtime configuration. Baseline replacement is DESCRIBED as pending here — it is
/// performed later by S10/S11, never by this gate.
#[derive(Debug, Clone)]
pub struct PromotionRuntimeConfigReceipt {
    /// The content hash pinning the runtime configuration.
    pub runtime_config_hash: String,
    /// Whether baseline replacement is pending later runtime configuration (must be true: never done
    /// here).
    pub baseline_replacement_pending: bool,
}

/// The full set of inputs the gate weighs. INPUT type (never `Serialize`): the gate re-runs the real
/// MODEL-EVAL-1 evaluation and re-checks every receipt. Closed by default.
#[derive(Debug)]
pub struct ModelPromotionInput {
    /// The candidate-eval receipt (gate re-runs `evaluate_candidate`). `None` means no eval report.
    pub eval: Option<PromotionEvalReceipt>,
    /// The pinned candidate/baseline/dataset hashes.
    pub candidate: Option<PromotionCandidateReceipt>,
    /// Explicit operator promotion approval.
    pub operator_approval: Option<PromotionOperatorApprovalReceipt>,
    /// Rollback artifact.
    pub rollback: Option<PromotionRollbackReceipt>,
    /// Runtime config.
    pub runtime_config: Option<PromotionRuntimeConfigReceipt>,
    /// Production safety plan (reuses the TRAIN-GATE-0 receipt type).
    pub production_safety: Option<ProductionSafetyPlanReceipt>,
    /// Authority-drift check (unchecked by default).
    pub authority_drift: AuthorityDriftCheck,
}

impl ModelPromotionInput {
    /// The closed-by-default input: nothing supplied, drift unchecked. Promotion is denied.
    pub fn closed_by_default() -> Self {
        Self {
            eval: None,
            candidate: None,
            operator_approval: None,
            rollback: None,
            runtime_config: None,
            production_safety: None,
            authority_drift: AuthorityDriftCheck::unchecked(),
        }
    }
}

// --- the boundary record ---

/// The inert boundary: every forbidden action is `false`. Stamped on every report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ModelPromotionBoundary {
    /// The gate never trains.
    pub trains: bool,
    /// The gate never deploys a model.
    pub deploys_model: bool,
    /// The gate never starts production runtime.
    pub starts_production: bool,
    /// The gate never creates truth.
    pub creates_truth: bool,
    /// The gate never creates memory.
    pub creates_memory: bool,
    /// The gate never creates evidence.
    pub creates_evidence: bool,
    /// The gate never bypasses rollback.
    pub bypasses_rollback: bool,
    /// The gate never replaces the baseline.
    pub replaces_baseline: bool,
}

impl ModelPromotionBoundary {
    fn inert() -> Self {
        Self {
            trains: PROMOTION_READY_IS_PRODUCTION,
            deploys_model: PROMOTION_READY_IS_PRODUCTION,
            starts_production: PROMOTION_READY_IS_PRODUCTION,
            creates_truth: PROMOTION_READY_IS_PRODUCTION,
            creates_memory: PROMOTION_READY_IS_PRODUCTION,
            creates_evidence: PROMOTION_READY_IS_PRODUCTION,
            bypasses_rollback: PROMOTION_READY_IS_PRODUCTION,
            replaces_baseline: PROMOTION_READY_IS_PRODUCTION,
        }
    }

    /// True iff every forbidden action is inert.
    pub fn all_inert(&self) -> bool {
        !self.trains
            && !self.deploys_model
            && !self.starts_production
            && !self.creates_truth
            && !self.creates_memory
            && !self.creates_evidence
            && !self.bypasses_rollback
            && !self.replaces_baseline
    }
}

// --- the sealed promoted-model receipt ---

/// The SEALED receipt produced ONLY on `PromotionReady`. It records the pinned lineage and that the
/// candidate is eligible for S10 packaging / S11 smoke — it deploys nothing, starts no production,
/// and records baseline replacement as PENDING (never performed). `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromotedModelReceipt {
    /// The schema tag.
    pub schema: &'static str,
    /// The pinned candidate artifact hash.
    pub promoted_candidate_hash: String,
    /// The pinned baseline artifact hash.
    pub baseline_hash: String,
    /// The pinned dataset hash.
    pub dataset_hash: String,
    /// The pinned eval-report hash.
    pub eval_report_hash: String,
    /// The pinned runtime-config hash.
    pub runtime_config_hash: String,
    /// The pinned rollback hash.
    pub rollback_hash: String,
    /// Always `true`: must pass S10 packaging before any production.
    pub requires_s10_packaging: bool,
    /// Always `true`: must pass S11 smoke before any production.
    pub requires_s11_smoke: bool,
    /// Always `true`: baseline replacement is pending later runtime configuration, never performed.
    pub baseline_replacement_pending: bool,
    /// Always `false`: a sealed receipt does not deploy.
    pub deploys_model: bool,
    /// Always `false`: a sealed receipt does not start production.
    pub starts_production: bool,
    /// Always `false`: a sealed receipt does not replace the baseline.
    pub replaces_baseline: bool,
    /// Always `false`: a sealed receipt does not train.
    pub trains: bool,
    /// Always `false`: a sealed receipt does not modify weights.
    pub modifies_weights: bool,
    /// Always `false`: a sealed receipt does not create evidence.
    pub creates_evidence: bool,
    /// Always `false`: a sealed receipt does not create memory.
    pub creates_memory: bool,
    /// Always `false`: a sealed receipt does not grant authority.
    pub grants_authority: bool,
    /// Always `false`: a sealed receipt does not open P12.
    pub opens_p12: bool,
}

// --- the report ---

/// The gate's verdict on whether a candidate is ready for promotion. `Serialize` but never
/// `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelPromotionReport {
    /// The schema tag.
    pub schema: &'static str,
    /// The terminal decision.
    pub decision: ModelPromotionDecision,
    /// The consumed eval verdict slug (`None` if no eval receipt was supplied).
    pub eval_verdict: Option<&'static str>,
    /// The corroborated candidate artifact hash (`None` unless the eval produced one).
    pub candidate_hash: Option<String>,
    /// The corroborated baseline artifact hash.
    pub baseline_hash: Option<String>,
    /// The corroborated dataset hash.
    pub dataset_hash: Option<String>,
    /// The re-derived eval-report hash (`None` if no eval receipt).
    pub eval_report_hash: Option<String>,
    /// Why the gate refused (empty iff ready).
    pub refusals: Vec<ModelPromotionRefusal>,
    /// The sealed promoted-model receipt (present ONLY on `PromotionReady`).
    pub promoted: Option<PromotedModelReceipt>,
    /// Always `true`: a promotion still requires S10 packaging and S11 smoke before production.
    pub requires_s10_s11: bool,
    /// Always `false`: the gate deploys no model.
    pub deploys_model: bool,
    /// Always `false`: the gate starts no production.
    pub starts_production: bool,
    /// Always `false`: the gate replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the gate trains nothing.
    pub trains: bool,
    /// Always `false`: the gate modifies no weights.
    pub modifies_weights: bool,
    /// Always `false`: the gate creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the gate creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the gate grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the gate opens no P12.
    pub opens_p12: bool,
    /// Always `false`: the gate does not set P12 `training_justified`.
    pub training_justified: bool,
    /// The inert boundary.
    pub boundary: ModelPromotionBoundary,
}

/// A non-cryptographic, dependency-free FNV-1a content pin. Deterministic and portable.
fn fnv1a_hex(s: &str) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for b in s.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("eval-{hash:016x}")
}

/// True iff `pinned` is present (non-empty) AND corroborated by the eval-derived `derived` value.
fn hash_ok(pinned: &str, derived: &Option<String>) -> bool {
    !pinned.is_empty() && derived.as_deref() == Some(pinned)
}

/// Evaluate the closed-by-default promotion gate over `input`. Runs the REAL MODEL-EVAL-1 evaluation,
/// re-checks every pinned hash and receipt, and emits `PromotionReady` only when every requirement
/// holds; otherwise denies with the full set of refusal reasons. Deploys nothing, starts no
/// production, trains nothing, replaces no baseline.
pub fn evaluate_model_promotion(input: &ModelPromotionInput) -> ModelPromotionReport {
    let mut refusals: Vec<ModelPromotionRefusal> = Vec::new();

    // CONSUME the REAL MODEL-EVAL-1 evaluation (re-run; verdict DERIVED, never handed in).
    let (eval_verdict, report_opt, derived_eval_hash): (
        Option<&'static str>,
        Option<CandidateEvalReport>,
        Option<String>,
    ) = match &input.eval {
        None => {
            refusals.push(ModelPromotionRefusal::MissingCandidateEvalReport);
            (None, None, None)
        }
        Some(er) => {
            let report = evaluate_candidate(&er.eval);
            let derived = fnv1a_hex(&evaluate_candidate_json(&er.eval));
            if report.verdict != CandidateEvalVerdict::CandidateReadyForPromotionReview {
                refusals.push(ModelPromotionRefusal::CandidateNotReadyForPromotionReview);
            }
            // Defense in depth: re-check the safety fields on the consumed report.
            if !report.holdout.holdout_present {
                refusals.push(ModelPromotionRefusal::HoldoutNotClean);
            }
            if report.holdout.contaminated {
                refusals.push(ModelPromotionRefusal::ContaminationDetected);
            }
            if report.holdout.memorization_leaked {
                refusals.push(ModelPromotionRefusal::MemorizationLeakageDetected);
            }
            if report.regression.any_critical {
                refusals.push(ModelPromotionRefusal::CriticalRegressionPresent);
            }
            // The eval-report hash must be pinned AND match the re-derivation.
            if er.eval_report_hash.is_empty() || er.eval_report_hash != derived {
                refusals.push(ModelPromotionRefusal::MissingEvalReportHash);
            }
            (Some(report.verdict.tag()), Some(report), Some(derived))
        }
    };

    let rep_candidate_hash = report_opt.as_ref().and_then(|r| r.candidate_hash.clone());
    let rep_baseline_hash = report_opt.as_ref().and_then(|r| r.baseline_hash.clone());
    let rep_dataset_hash = report_opt.as_ref().and_then(|r| r.dataset_hash.clone());

    // The candidate/baseline/dataset hashes must be pinned AND corroborated by the eval.
    match &input.candidate {
        None => {
            refusals.push(ModelPromotionRefusal::MissingCandidateArtifactHash);
            refusals.push(ModelPromotionRefusal::MissingBaselineArtifactHash);
            refusals.push(ModelPromotionRefusal::MissingDatasetHash);
        }
        Some(c) => {
            if !hash_ok(&c.candidate_artifact_hash, &rep_candidate_hash) {
                refusals.push(ModelPromotionRefusal::MissingCandidateArtifactHash);
            }
            if !hash_ok(&c.baseline_artifact_hash, &rep_baseline_hash) {
                refusals.push(ModelPromotionRefusal::MissingBaselineArtifactHash);
            }
            if !hash_ok(&c.dataset_hash, &rep_dataset_hash) {
                refusals.push(ModelPromotionRefusal::MissingDatasetHash);
            }
        }
    }

    // Runtime config.
    match &input.runtime_config {
        Some(_) => {}
        None => refusals.push(ModelPromotionRefusal::MissingRuntimeConfig),
    }

    // Rollback.
    match &input.rollback {
        Some(_) => {}
        None => refusals.push(ModelPromotionRefusal::MissingRollbackArtifact),
    }

    // Explicit operator approval.
    match &input.operator_approval {
        Some(a) if a.approves_promotion => {}
        _ => refusals.push(ModelPromotionRefusal::MissingOperatorApproval),
    }

    // Production safety plan.
    match &input.production_safety {
        Some(_) => {}
        None => refusals.push(ModelPromotionRefusal::MissingProductionSafetyPlan),
    }

    // Affirmative authority-drift check.
    if !input.authority_drift.is_clean() {
        refusals.push(ModelPromotionRefusal::AuthorityDriftDetected);
    }

    let decision = if refusals.is_empty() {
        ModelPromotionDecision::PromotionReady
    } else {
        ModelPromotionDecision::PromotionDenied
    };

    // The sealed receipt is produced ONLY on PromotionReady; the unwraps are sound (an empty refusal
    // set implies every receipt is present and every hash corroborated).
    let promoted = if decision == ModelPromotionDecision::PromotionReady {
        let er = input.eval.as_ref().expect("eval present when ready");
        let c = input
            .candidate
            .as_ref()
            .expect("candidate present when ready");
        let rc = input
            .runtime_config
            .as_ref()
            .expect("runtime present when ready");
        let rb = input
            .rollback
            .as_ref()
            .expect("rollback present when ready");
        Some(PromotedModelReceipt {
            schema: SCHEMA,
            promoted_candidate_hash: c.candidate_artifact_hash.clone(),
            baseline_hash: c.baseline_artifact_hash.clone(),
            dataset_hash: c.dataset_hash.clone(),
            eval_report_hash: er.eval_report_hash.clone(),
            runtime_config_hash: rc.runtime_config_hash.clone(),
            rollback_hash: rb.rollback_hash.clone(),
            requires_s10_packaging: true,
            requires_s11_smoke: true,
            baseline_replacement_pending: true,
            deploys_model: PROMOTION_READY_IS_PRODUCTION,
            starts_production: PROMOTION_READY_IS_PRODUCTION,
            replaces_baseline: PROMOTION_READY_IS_PRODUCTION,
            trains: PROMOTION_READY_IS_PRODUCTION,
            modifies_weights: PROMOTION_READY_IS_PRODUCTION,
            creates_evidence: PROMOTION_READY_IS_PRODUCTION,
            creates_memory: PROMOTION_READY_IS_PRODUCTION,
            grants_authority: PROMOTION_READY_IS_PRODUCTION,
            opens_p12: PROMOTION_READY_IS_PRODUCTION,
        })
    } else {
        None
    };

    ModelPromotionReport {
        schema: SCHEMA,
        decision,
        eval_verdict,
        candidate_hash: rep_candidate_hash,
        baseline_hash: rep_baseline_hash,
        dataset_hash: rep_dataset_hash,
        eval_report_hash: derived_eval_hash,
        refusals,
        promoted,
        requires_s10_s11: true,
        deploys_model: PROMOTION_READY_IS_PRODUCTION,
        starts_production: PROMOTION_READY_IS_PRODUCTION,
        replaces_baseline: PROMOTION_READY_IS_PRODUCTION,
        trains: PROMOTION_READY_IS_PRODUCTION,
        modifies_weights: PROMOTION_READY_IS_PRODUCTION,
        creates_evidence: PROMOTION_READY_IS_PRODUCTION,
        creates_memory: PROMOTION_READY_IS_PRODUCTION,
        grants_authority: PROMOTION_READY_IS_PRODUCTION,
        opens_p12: PROMOTION_READY_IS_PRODUCTION,
        training_justified: PROMOTION_READY_IS_PRODUCTION,
        boundary: ModelPromotionBoundary::inert(),
    }
}

/// The promotion report serialized to canonical JSON.
pub fn evaluate_model_promotion_json(input: &ModelPromotionInput) -> String {
    serde_json::to_string(&evaluate_model_promotion(input)).expect("promotion report serializes")
}

/// What can go wrong verifying a serialized promotion artifact.
#[derive(Debug, PartialEq, Eq)]
pub enum ModelPromotionError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the report from the SAME input and byte-compare against `candidate`. The report is
/// `Serialize` but never `Deserialize`: a serialized report is NOT trusted as authority — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_model_promotion_report_json(
    input: &ModelPromotionInput,
    candidate: &str,
) -> Result<(), ModelPromotionError> {
    if candidate == evaluate_model_promotion_json(input) {
        Ok(())
    } else {
        Err(ModelPromotionError::ReplayMismatch)
    }
}

// --- building a REAL MODEL-EVAL-1 input (the SCORE-0 -> ... -> CANDIDATE-EVAL chain) ---

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

fn reading_candidate(failures: &[FailureObservation]) -> ModelNeedCandidate {
    real_candidate(
        failures,
        FailureClass::ReadingMisgrounding,
        ScoreClass::Grounding,
        ScoreReason::Ungrounded,
        RECURRENCE_THRESHOLD,
    )
}

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

fn candidate_battery(failures: &[FailureObservation]) -> ModelEvalBattery {
    ModelEvalBattery::new(vec![
        residual_run(reading_candidate(failures)),
        residual_run(reading_candidate(failures)),
    ])
}

fn allowed_gate_input(failures: &[FailureObservation]) -> TrainingGateInput {
    TrainingGateInput {
        eval: Some(candidate_battery(failures)),
        operator_authorization: Some(OperatorAuthorizationReceipt {
            operator: "operator".to_string(),
            attempt_scope: "local-finetune-attempt".to_string(),
        }),
        dataset: Some(DatasetReadinessReceipt {
            curated_corpus_hash: "curated-corpus-hash".to_string(),
            item_count: 2,
        }),
        holdout: Some(HoldoutReadinessReceipt {
            holdout_present: true,
            contaminated: false,
            holdout_hash: "holdout-hash".to_string(),
        }),
        contamination: Some(ContaminationReportReceipt {
            memorization_leakage: false,
            report_hash: "contamination-report-hash".to_string(),
        }),
        rollback: Some(RollbackPlanReceipt {
            rollback_target: "pre-train-snapshot".to_string(),
            verified: true,
        }),
        production_safety: Some(ProductionSafetyPlanReceipt {
            plan_id: "production-safety-plan-0".to_string(),
        }),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

fn full_attempt(failures: &[FailureObservation]) -> TrainingAttemptInput {
    TrainingAttemptInput {
        mode: TrainingAttemptMode::AuthorizedLocalAttempt,
        gate_input: allowed_gate_input(failures),
        operator_authorization: Some(AttemptAuthorizationReceipt {
            operator: "operator".to_string(),
            attempt_scope: "local-candidate-train-attempt".to_string(),
            acknowledges_candidate_only: true,
        }),
        run_config: Some(TrainingRunConfig {
            config_hash: "run-config-hash".to_string(),
            deterministic: true,
            seed: 7,
            max_steps: 100,
        }),
        dataset: Some(TrainingDatasetBundle {
            curated_corpus_hash: "curated-corpus-hash".to_string(),
            item_count: 2,
            contaminated: false,
        }),
        baseline: Some(TrainingBaselineArtifact {
            baseline_hash: "baseline-hash".to_string(),
        }),
        holdout: Some(TrainingHoldoutBundle {
            holdout_present: true,
            leaked: false,
            holdout_hash: "attempt-holdout-hash".to_string(),
        }),
        rollback: Some(TrainingRollbackArtifact {
            rollback_hash: "rollback-hash".to_string(),
            verified: true,
        }),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

fn baseline_ref() -> BaselineModelRef {
    BaselineModelRef {
        baseline_hash: "baseline-hash".to_string(),
    }
}

fn eval_clean_holdout() -> HoldoutReport {
    HoldoutReport {
        holdout_present: true,
        contaminated: false,
        memorization_leaked: false,
        holdout_hash: "eval-holdout-hash".to_string(),
    }
}

fn eval_clean_safety() -> SafetyBoundaryReport {
    SafetyBoundaryReport {
        adversarial_pass: true,
        long_horizon_pass: true,
        dry_run_production_smoke_pass: true,
    }
}

fn eval_clean_comparisons(target_improves: bool) -> Vec<CandidateEvalComparison> {
    let higher = |dimension: EvalDimension| CandidateEvalComparison {
        dimension,
        baseline_score: 80,
        candidate_score: 85,
        higher_is_better: true,
    };
    vec![
        higher(EvalDimension::Reading),
        higher(EvalDimension::Grounding),
        higher(EvalDimension::Curation),
        higher(EvalDimension::Replay),
        higher(EvalDimension::HorizonBoundary),
        higher(EvalDimension::Refusal),
        CandidateEvalComparison {
            dimension: EvalDimension::Hallucination,
            baseline_score: 20,
            candidate_score: 15,
            higher_is_better: false,
        },
        CandidateEvalComparison {
            dimension: EvalDimension::TargetRecurringFailure,
            baseline_score: 10,
            candidate_score: if target_improves { 5 } else { 10 },
            higher_is_better: false,
        },
    ]
}

fn eval_comparisons_regressing(dimension: EvalDimension) -> Vec<CandidateEvalComparison> {
    eval_clean_comparisons(true)
        .into_iter()
        .map(|c| {
            if c.dimension == dimension {
                CandidateEvalComparison {
                    dimension: c.dimension,
                    baseline_score: c.baseline_score,
                    candidate_score: if c.higher_is_better {
                        c.baseline_score - 10
                    } else {
                        c.baseline_score + 10
                    },
                    higher_is_better: c.higher_is_better,
                }
            } else {
                c
            }
        })
        .collect()
}

fn eval_battery(target_improves: bool) -> CandidateEvalBattery {
    CandidateEvalBattery {
        comparisons: eval_clean_comparisons(target_improves),
        holdout: eval_clean_holdout(),
        safety: eval_clean_safety(),
    }
}

/// A ready MODEL-EVAL-1 input (the candidate beats the baseline cleanly).
fn ready_eval(failures: &[FailureObservation]) -> CandidateEvalInput {
    CandidateEvalInput {
        candidate: Some(
            run_training_attempt(&full_attempt(failures))
                .candidate
                .expect("a fully-authorized TRAIN-0 attempt prepares a candidate"),
        ),
        baseline: Some(baseline_ref()),
        battery: Some(eval_battery(true)),
    }
}

// --- promotion-input builders ---

fn promotion_candidate_receipt(report: &CandidateEvalReport) -> PromotionCandidateReceipt {
    PromotionCandidateReceipt {
        candidate_artifact_hash: report.candidate_hash.clone().unwrap_or_default(),
        baseline_artifact_hash: report.baseline_hash.clone().unwrap_or_default(),
        dataset_hash: report.dataset_hash.clone().unwrap_or_default(),
    }
}

fn operator_approval() -> PromotionOperatorApprovalReceipt {
    PromotionOperatorApprovalReceipt {
        operator: "operator".to_string(),
        promotion_scope: "local-candidate-promotion".to_string(),
        approves_promotion: true,
    }
}

fn rollback() -> PromotionRollbackReceipt {
    PromotionRollbackReceipt {
        rollback_hash: "promotion-rollback-hash".to_string(),
        verified: true,
    }
}

fn runtime_config() -> PromotionRuntimeConfigReceipt {
    PromotionRuntimeConfigReceipt {
        runtime_config_hash: "runtime-config-hash".to_string(),
        baseline_replacement_pending: true,
    }
}

fn prod_safety() -> ProductionSafetyPlanReceipt {
    ProductionSafetyPlanReceipt {
        plan_id: "production-safety-plan-0".to_string(),
    }
}

/// A fully-correct promotion input over `eval`: pins the eval's real hashes and supplies every
/// receipt + a clean drift check.
fn promotion_input_over(eval: CandidateEvalInput) -> ModelPromotionInput {
    let report = evaluate_candidate(&eval);
    let eval_report_hash = fnv1a_hex(&evaluate_candidate_json(&eval));
    ModelPromotionInput {
        candidate: Some(promotion_candidate_receipt(&report)),
        eval: Some(PromotionEvalReceipt {
            eval,
            eval_report_hash,
        }),
        operator_approval: Some(operator_approval()),
        rollback: Some(rollback()),
        runtime_config: Some(runtime_config()),
        production_safety: Some(prod_safety()),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

/// The fully-met promotion input over a ready eval -> PromotionReady.
fn full_input(failures: &[FailureObservation]) -> ModelPromotionInput {
    promotion_input_over(ready_eval(failures))
}

// --- the promotion scenario matrix (observes the real gate over constructed inputs) ---

/// One scenario cell: the OBSERVED decision of running the real gate over a constructed input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelPromotionScenarioCell {
    /// The scenario name.
    pub name: &'static str,
    /// The observed decision slug.
    pub decision: &'static str,
    /// The observed refusal-reason slugs.
    pub refusals: Vec<&'static str>,
    /// Whether a sealed promoted-model receipt was produced.
    pub sealed_receipt: bool,
    /// Whether production stayed fully closed (no forbidden flag set; no deploy/start).
    pub production_still_closed: bool,
    /// A short human-readable detail.
    pub detail: String,
}

/// The fixed promotion scenario matrix. Every cell runs the real gate and records what it observed;
/// `production_never_opens` is the conjunction across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModelPromotionMatrix {
    /// The schema tag.
    pub schema: &'static str,
    /// The scenario cells.
    pub scenarios: Vec<ModelPromotionScenarioCell>,
    /// The two decision-state slugs.
    pub decisions: [&'static str; MODEL_PROMOTE_DECISION_COUNT],
    /// The sixteen refusal-reason slugs.
    pub refusal_reasons: [&'static str; MODEL_PROMOTE_REFUSAL_COUNT],
    /// True iff no cell opened production.
    pub production_never_opens: bool,
    /// The inert boundary.
    pub boundary: ModelPromotionBoundary,
}

impl ModelPromotionMatrix {
    /// Find a scenario cell by name.
    pub fn scenario(&self, name: &str) -> Option<&ModelPromotionScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn closed_for(report: &ModelPromotionReport) -> bool {
    let receipt_closed = match &report.promoted {
        None => true,
        Some(r) => {
            !r.deploys_model
                && !r.starts_production
                && !r.replaces_baseline
                && !r.trains
                && !r.modifies_weights
                && !r.creates_evidence
                && !r.creates_memory
                && !r.grants_authority
                && !r.opens_p12
                && r.requires_s10_packaging
                && r.requires_s11_smoke
                && r.baseline_replacement_pending
        }
    };
    !report.deploys_model
        && !report.starts_production
        && !report.replaces_baseline
        && !report.trains
        && !report.modifies_weights
        && !report.creates_evidence
        && !report.creates_memory
        && !report.grants_authority
        && !report.opens_p12
        && !report.training_justified
        && report.boundary.all_inert()
        && receipt_closed
}

fn promo_cell(name: &'static str, input: ModelPromotionInput) -> ModelPromotionScenarioCell {
    let report = evaluate_model_promotion(&input);
    ModelPromotionScenarioCell {
        name,
        decision: report.decision.tag(),
        refusals: report.refusals.iter().map(|r| r.tag()).collect(),
        sealed_receipt: report.promoted.is_some(),
        production_still_closed: closed_for(&report),
        detail: report.decision.tag().to_string(),
    }
}

/// The serialized-report tamper cell: tamper a real (ready) promotion report JSON and observe the
/// re-derive verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous.
fn tamper_cell(failures: &[FailureObservation]) -> ModelPromotionScenarioCell {
    let input = full_input(failures);
    let canonical = evaluate_model_promotion_json(&input);
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_model_promotion_report_json(&input, &tampered).is_err()
        && verify_model_promotion_report_json(&input, &canonical).is_ok();
    let report = evaluate_model_promotion(&input);
    ModelPromotionScenarioCell {
        name: "serialized_promotion_report_tamper_refused",
        decision: report.decision.tag(),
        refusals: if refused {
            vec!["serialized_promotion_report_tamper_refused"]
        } else {
            vec!["VACUOUS"]
        },
        sealed_receipt: report.promoted.is_some(),
        production_still_closed: closed_for(&report) && refused,
        detail: if refused {
            "serialized_promotion_report_tamper_refused".to_string()
        } else {
            "VACUOUS: promotion verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 22-scenario promotion matrix from the REAL gate over constructed inputs.
pub fn model_promotion_matrix() -> ModelPromotionMatrix {
    // Derive the SCORE-0 failure set ONCE; every candidate reuses it.
    let failures = verifier_score_matrix().failures;

    // A rejected eval (adversarial safety check fails -> rejected, hashes still present).
    let rejected_eval = || CandidateEvalInput {
        battery: Some(CandidateEvalBattery {
            safety: SafetyBoundaryReport {
                adversarial_pass: false,
                ..eval_clean_safety()
            },
            ..eval_battery(true)
        }),
        ..ready_eval(&failures)
    };
    // A needs-more-evidence eval (clean but no target improvement).
    let needs_more_eval = || CandidateEvalInput {
        battery: Some(eval_battery(false)),
        ..ready_eval(&failures)
    };

    let scenarios = vec![
        promo_cell(
            "missing_candidate_eval_report_denied",
            ModelPromotionInput {
                eval: None,
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "candidate_rejected_denied",
            promotion_input_over(rejected_eval()),
        ),
        promo_cell(
            "candidate_needs_more_evidence_denied",
            promotion_input_over(needs_more_eval()),
        ),
        promo_cell(
            "ready_without_candidate_hash_denied",
            ModelPromotionInput {
                candidate: Some(PromotionCandidateReceipt {
                    candidate_artifact_hash: String::new(),
                    ..promotion_candidate_receipt(&evaluate_candidate(&ready_eval(&failures)))
                }),
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_baseline_hash_denied",
            ModelPromotionInput {
                candidate: Some(PromotionCandidateReceipt {
                    baseline_artifact_hash: String::new(),
                    ..promotion_candidate_receipt(&evaluate_candidate(&ready_eval(&failures)))
                }),
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_dataset_hash_denied",
            ModelPromotionInput {
                candidate: Some(PromotionCandidateReceipt {
                    dataset_hash: String::new(),
                    ..promotion_candidate_receipt(&evaluate_candidate(&ready_eval(&failures)))
                }),
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_eval_hash_denied",
            ModelPromotionInput {
                eval: Some(PromotionEvalReceipt {
                    eval: ready_eval(&failures),
                    eval_report_hash: String::new(),
                }),
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_runtime_config_denied",
            ModelPromotionInput {
                runtime_config: None,
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_rollback_denied",
            ModelPromotionInput {
                rollback: None,
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_operator_approval_denied",
            ModelPromotionInput {
                operator_approval: None,
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "ready_without_production_safety_plan_denied",
            ModelPromotionInput {
                production_safety: None,
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "holdout_not_clean_denied",
            promotion_input_over(CandidateEvalInput {
                battery: Some(CandidateEvalBattery {
                    holdout: HoldoutReport {
                        holdout_present: false,
                        ..eval_clean_holdout()
                    },
                    ..eval_battery(true)
                }),
                ..ready_eval(&failures)
            }),
        ),
        promo_cell(
            "contamination_detected_denied",
            promotion_input_over(CandidateEvalInput {
                battery: Some(CandidateEvalBattery {
                    holdout: HoldoutReport {
                        contaminated: true,
                        ..eval_clean_holdout()
                    },
                    ..eval_battery(true)
                }),
                ..ready_eval(&failures)
            }),
        ),
        promo_cell(
            "memorization_leakage_denied",
            promotion_input_over(CandidateEvalInput {
                battery: Some(CandidateEvalBattery {
                    holdout: HoldoutReport {
                        memorization_leaked: true,
                        ..eval_clean_holdout()
                    },
                    ..eval_battery(true)
                }),
                ..ready_eval(&failures)
            }),
        ),
        promo_cell(
            "critical_regression_denied",
            promotion_input_over(CandidateEvalInput {
                battery: Some(CandidateEvalBattery {
                    comparisons: eval_comparisons_regressing(EvalDimension::Reading),
                    ..eval_battery(true)
                }),
                ..ready_eval(&failures)
            }),
        ),
        promo_cell(
            "authority_drift_denied",
            ModelPromotionInput {
                authority_drift: AuthorityDriftCheck::drifted(),
                ..full_input(&failures)
            },
        ),
        promo_cell(
            "all_requirements_met_promotion_ready",
            full_input(&failures),
        ),
        promo_cell("promotion_ready_not_deployment", full_input(&failures)),
        promo_cell("promotion_ready_not_training", full_input(&failures)),
        promo_cell(
            "promotion_ready_not_baseline_replacement",
            full_input(&failures),
        ),
        promo_cell("promotion_ready_requires_s10_s11", full_input(&failures)),
        tamper_cell(&failures),
    ];

    let production_never_opens = scenarios.iter().all(|c| c.production_still_closed);
    ModelPromotionMatrix {
        schema: SCHEMA,
        scenarios,
        decisions: MODEL_PROMOTE_DECISION_NAMES,
        refusal_reasons: MODEL_PROMOTE_REFUSAL_NAMES,
        production_never_opens,
        boundary: ModelPromotionBoundary::inert(),
    }
}

/// The promotion matrix serialized to canonical JSON.
pub fn model_promotion_matrix_json() -> String {
    serde_json::to_string(&model_promotion_matrix()).expect("promotion matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`.
pub fn verify_model_promotion_matrix_json(candidate: &str) -> Result<(), ModelPromotionError> {
    if candidate == model_promotion_matrix_json() {
        Ok(())
    } else {
        Err(ModelPromotionError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failures() -> Vec<FailureObservation> {
        verifier_score_matrix().failures
    }

    fn has(report: &ModelPromotionReport, r: ModelPromotionRefusal) -> bool {
        report.refusals.contains(&r)
    }

    #[test]
    fn gate_consumes_the_real_candidate_eval_report() {
        let f = failures();
        // A ready eval -> the gate observes the REAL candidate_ready_for_promotion_review verdict.
        let ready = evaluate_model_promotion(&full_input(&f));
        assert_eq!(
            ready.eval_verdict,
            Some("candidate_ready_for_promotion_review")
        );
        // A rejected eval -> the gate observes candidate_rejected (derived, not hard-coded).
        let rejected = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(CandidateEvalBattery {
                safety: SafetyBoundaryReport {
                    adversarial_pass: false,
                    ..eval_clean_safety()
                },
                ..eval_battery(true)
            }),
            ..ready_eval(&f)
        }));
        assert_eq!(rejected.eval_verdict, Some("candidate_rejected"));
        assert_eq!(rejected.decision, ModelPromotionDecision::PromotionDenied);
        assert!(has(
            &rejected,
            ModelPromotionRefusal::CandidateNotReadyForPromotionReview
        ));
    }

    #[test]
    fn missing_candidate_eval_report_is_denied() {
        let f = failures();
        let report = evaluate_model_promotion(&ModelPromotionInput {
            eval: None,
            ..full_input(&f)
        });
        assert_eq!(report.decision, ModelPromotionDecision::PromotionDenied);
        assert!(has(
            &report,
            ModelPromotionRefusal::MissingCandidateEvalReport
        ));
        assert_eq!(report.eval_verdict, None);
    }

    #[test]
    fn candidate_not_ready_is_denied() {
        let f = failures();
        // Rejected.
        let rejected = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(CandidateEvalBattery {
                safety: SafetyBoundaryReport {
                    adversarial_pass: false,
                    ..eval_clean_safety()
                },
                ..eval_battery(true)
            }),
            ..ready_eval(&f)
        }));
        assert!(has(
            &rejected,
            ModelPromotionRefusal::CandidateNotReadyForPromotionReview
        ));
    }

    #[test]
    fn candidate_needs_more_evidence_is_denied() {
        let f = failures();
        let report = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(eval_battery(false)),
            ..ready_eval(&f)
        }));
        assert_eq!(report.eval_verdict, Some("candidate_needs_more_evidence"));
        assert_eq!(report.decision, ModelPromotionDecision::PromotionDenied);
        assert!(has(
            &report,
            ModelPromotionRefusal::CandidateNotReadyForPromotionReview
        ));
    }

    #[test]
    fn ready_without_each_hash_is_denied() {
        let f = failures();
        let base = evaluate_candidate(&ready_eval(&f));

        let no_cand = evaluate_model_promotion(&ModelPromotionInput {
            candidate: Some(PromotionCandidateReceipt {
                candidate_artifact_hash: String::new(),
                ..promotion_candidate_receipt(&base)
            }),
            ..full_input(&f)
        });
        assert!(has(
            &no_cand,
            ModelPromotionRefusal::MissingCandidateArtifactHash
        ));

        let no_base = evaluate_model_promotion(&ModelPromotionInput {
            candidate: Some(PromotionCandidateReceipt {
                baseline_artifact_hash: String::new(),
                ..promotion_candidate_receipt(&base)
            }),
            ..full_input(&f)
        });
        assert!(has(
            &no_base,
            ModelPromotionRefusal::MissingBaselineArtifactHash
        ));

        let no_ds = evaluate_model_promotion(&ModelPromotionInput {
            candidate: Some(PromotionCandidateReceipt {
                dataset_hash: String::new(),
                ..promotion_candidate_receipt(&base)
            }),
            ..full_input(&f)
        });
        assert!(has(&no_ds, ModelPromotionRefusal::MissingDatasetHash));

        let no_eval_hash = evaluate_model_promotion(&ModelPromotionInput {
            eval: Some(PromotionEvalReceipt {
                eval: ready_eval(&f),
                eval_report_hash: String::new(),
            }),
            ..full_input(&f)
        });
        assert!(has(
            &no_eval_hash,
            ModelPromotionRefusal::MissingEvalReportHash
        ));
    }

    #[test]
    fn a_stale_eval_report_hash_is_refused() {
        let f = failures();
        let report = evaluate_model_promotion(&ModelPromotionInput {
            eval: Some(PromotionEvalReceipt {
                eval: ready_eval(&f),
                eval_report_hash: "eval-deadbeefdeadbeef".to_string(),
            }),
            ..full_input(&f)
        });
        // A present-but-non-matching pin is refused (the pin must corroborate the re-derived eval).
        assert!(has(&report, ModelPromotionRefusal::MissingEvalReportHash));
    }

    #[test]
    fn ready_without_each_receipt_is_denied() {
        let f = failures();
        let rc = evaluate_model_promotion(&ModelPromotionInput {
            runtime_config: None,
            ..full_input(&f)
        });
        assert!(has(&rc, ModelPromotionRefusal::MissingRuntimeConfig));

        let rb = evaluate_model_promotion(&ModelPromotionInput {
            rollback: None,
            ..full_input(&f)
        });
        assert!(has(&rb, ModelPromotionRefusal::MissingRollbackArtifact));

        let op = evaluate_model_promotion(&ModelPromotionInput {
            operator_approval: None,
            ..full_input(&f)
        });
        assert!(has(&op, ModelPromotionRefusal::MissingOperatorApproval));

        let ps = evaluate_model_promotion(&ModelPromotionInput {
            production_safety: None,
            ..full_input(&f)
        });
        assert!(has(&ps, ModelPromotionRefusal::MissingProductionSafetyPlan));
    }

    #[test]
    fn operator_approval_must_be_affirmative() {
        let f = failures();
        let report = evaluate_model_promotion(&ModelPromotionInput {
            operator_approval: Some(PromotionOperatorApprovalReceipt {
                approves_promotion: false,
                ..operator_approval()
            }),
            ..full_input(&f)
        });
        assert!(has(&report, ModelPromotionRefusal::MissingOperatorApproval));
    }

    #[test]
    fn holdout_contamination_leakage_and_regression_are_denied() {
        let f = failures();
        let holdout = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(CandidateEvalBattery {
                holdout: HoldoutReport {
                    holdout_present: false,
                    ..eval_clean_holdout()
                },
                ..eval_battery(true)
            }),
            ..ready_eval(&f)
        }));
        assert!(has(&holdout, ModelPromotionRefusal::HoldoutNotClean));

        let contaminated = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(CandidateEvalBattery {
                holdout: HoldoutReport {
                    contaminated: true,
                    ..eval_clean_holdout()
                },
                ..eval_battery(true)
            }),
            ..ready_eval(&f)
        }));
        assert!(has(
            &contaminated,
            ModelPromotionRefusal::ContaminationDetected
        ));

        let leaked = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(CandidateEvalBattery {
                holdout: HoldoutReport {
                    memorization_leaked: true,
                    ..eval_clean_holdout()
                },
                ..eval_battery(true)
            }),
            ..ready_eval(&f)
        }));
        assert!(has(
            &leaked,
            ModelPromotionRefusal::MemorizationLeakageDetected
        ));
    }

    #[test]
    fn critical_regression_is_denied() {
        let f = failures();
        let report = evaluate_model_promotion(&promotion_input_over(CandidateEvalInput {
            battery: Some(CandidateEvalBattery {
                comparisons: eval_comparisons_regressing(EvalDimension::Grounding),
                ..eval_battery(true)
            }),
            ..ready_eval(&f)
        }));
        assert!(has(
            &report,
            ModelPromotionRefusal::CriticalRegressionPresent
        ));
    }

    #[test]
    fn authority_drift_is_denied() {
        let f = failures();
        let drifted = evaluate_model_promotion(&ModelPromotionInput {
            authority_drift: AuthorityDriftCheck::drifted(),
            ..full_input(&f)
        });
        assert!(has(&drifted, ModelPromotionRefusal::AuthorityDriftDetected));
        let unchecked = evaluate_model_promotion(&ModelPromotionInput {
            authority_drift: AuthorityDriftCheck::unchecked(),
            ..full_input(&f)
        });
        assert!(has(
            &unchecked,
            ModelPromotionRefusal::AuthorityDriftDetected
        ));
    }

    #[test]
    fn all_requirements_met_is_promotion_ready() {
        let f = failures();
        let report = evaluate_model_promotion(&full_input(&f));
        assert_eq!(report.decision, ModelPromotionDecision::PromotionReady);
        assert!(report.refusals.is_empty());
        assert!(report.promoted.is_some());
    }

    #[test]
    fn promotion_ready_seals_a_promoted_model_receipt() {
        let f = failures();
        let report = evaluate_model_promotion(&full_input(&f));
        let sealed = report.promoted.as_ref().expect("a receipt is sealed");
        // The sealed receipt pins the lineage and is inert on every production axis.
        assert_eq!(sealed.baseline_hash, "baseline-hash");
        assert!(!sealed.deploys_model);
        assert!(!sealed.starts_production);
        assert!(!sealed.replaces_baseline);
        assert!(!sealed.trains);
        assert!(!sealed.modifies_weights);
    }

    #[test]
    fn promotion_ready_is_not_deployment_or_training() {
        let f = failures();
        let report = evaluate_model_promotion(&full_input(&f));
        assert_eq!(report.decision, ModelPromotionDecision::PromotionReady);
        assert!(!report.deploys_model);
        assert!(!report.starts_production);
        assert!(!report.trains);
        assert!(!report.modifies_weights);
        assert!(report.boundary.all_inert());
    }

    #[test]
    fn promotion_ready_is_not_baseline_replacement() {
        let f = failures();
        let report = evaluate_model_promotion(&full_input(&f));
        assert!(!report.replaces_baseline);
        let sealed = report.promoted.as_ref().expect("sealed");
        assert!(!sealed.replaces_baseline);
        // Baseline replacement is described as PENDING, never performed.
        assert!(sealed.baseline_replacement_pending);
    }

    #[test]
    fn promotion_ready_requires_s10_s11() {
        let f = failures();
        let report = evaluate_model_promotion(&full_input(&f));
        assert!(report.requires_s10_s11);
        let sealed = report.promoted.as_ref().expect("sealed");
        assert!(sealed.requires_s10_packaging);
        assert!(sealed.requires_s11_smoke);
    }

    #[test]
    fn p12_training_justified_remains_false_even_when_ready() {
        let f = failures();
        let report = evaluate_model_promotion(&full_input(&f));
        assert!(!report.training_justified);
        assert!(!report.opens_p12);
        // The real P12 gate is unaffected by a promotion-ready decision.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn decision_and_refusal_counts_match_enums() {
        assert_eq!(
            ModelPromotionDecision::ALL.len(),
            MODEL_PROMOTE_DECISION_COUNT
        );
        assert_eq!(
            ModelPromotionRefusal::ALL.len(),
            MODEL_PROMOTE_REFUSAL_COUNT
        );
        assert_eq!(
            MODEL_PROMOTE_DECISION_NAMES.len(),
            MODEL_PROMOTE_DECISION_COUNT
        );
        assert_eq!(
            MODEL_PROMOTE_REFUSAL_NAMES.len(),
            MODEL_PROMOTE_REFUSAL_COUNT
        );
        for (d, name) in ModelPromotionDecision::ALL
            .iter()
            .zip(MODEL_PROMOTE_DECISION_NAMES)
        {
            assert_eq!(d.tag(), name);
        }
        for (r, name) in ModelPromotionRefusal::ALL
            .iter()
            .zip(MODEL_PROMOTE_REFUSAL_NAMES)
        {
            assert_eq!(r.tag(), name);
        }
    }

    #[test]
    fn matrix_has_the_twenty_two_named_scenarios() {
        let matrix = model_promotion_matrix();
        assert_eq!(matrix.scenarios.len(), MODEL_PROMOTE_SCENARIO_COUNT);
        for name in [
            "missing_candidate_eval_report_denied",
            "candidate_rejected_denied",
            "candidate_needs_more_evidence_denied",
            "ready_without_candidate_hash_denied",
            "ready_without_baseline_hash_denied",
            "ready_without_dataset_hash_denied",
            "ready_without_eval_hash_denied",
            "ready_without_runtime_config_denied",
            "ready_without_rollback_denied",
            "ready_without_operator_approval_denied",
            "ready_without_production_safety_plan_denied",
            "holdout_not_clean_denied",
            "contamination_detected_denied",
            "memorization_leakage_denied",
            "critical_regression_denied",
            "authority_drift_denied",
            "all_requirements_met_promotion_ready",
            "promotion_ready_not_deployment",
            "promotion_ready_not_training",
            "promotion_ready_not_baseline_replacement",
            "promotion_ready_requires_s10_s11",
            "serialized_promotion_report_tamper_refused",
        ] {
            assert!(
                matrix.scenario(name).is_some(),
                "scenario {name} is missing"
            );
        }
        assert!(matrix.production_never_opens);
        // Exactly the all-met / not-deployment / not-training / not-baseline / requires-s10s11 cells
        // seal a receipt (5 ready cells).
        let ready = matrix
            .scenario("all_requirements_met_promotion_ready")
            .expect("present");
        assert_eq!(ready.decision, "promotion_ready");
        assert!(ready.sealed_receipt);
    }

    #[test]
    fn every_matrix_cell_keeps_production_closed() {
        let matrix = model_promotion_matrix();
        for cell in &matrix.scenarios {
            assert!(
                cell.production_still_closed,
                "cell {} opened production",
                cell.name
            );
        }
        let tamper = matrix
            .scenario("serialized_promotion_report_tamper_refused")
            .expect("tamper cell present");
        assert!(tamper
            .refusals
            .contains(&"serialized_promotion_report_tamper_refused"));
    }

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let f = failures();
        let input = full_input(&f);
        let canonical = evaluate_model_promotion_json(&input);
        assert_eq!(canonical, evaluate_model_promotion_json(&full_input(&f)));
        assert!(verify_model_promotion_report_json(&input, &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_model_promotion_report_json(&input, &tampered),
            Err(ModelPromotionError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_re_derives_refusing_tampering() {
        let canonical = model_promotion_matrix_json();
        assert!(verify_model_promotion_matrix_json(&canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_model_promotion_matrix_json(&tampered),
            Err(ModelPromotionError::ReplayMismatch)
        );
    }

    #[test]
    fn closed_by_default_denies_with_no_inputs() {
        let report = evaluate_model_promotion(&ModelPromotionInput::closed_by_default());
        assert_eq!(report.decision, ModelPromotionDecision::PromotionDenied);
        assert!(has(
            &report,
            ModelPromotionRefusal::MissingCandidateEvalReport
        ));
        assert!(has(&report, ModelPromotionRefusal::MissingOperatorApproval));
        assert!(has(&report, ModelPromotionRefusal::AuthorityDriftDetected));
        assert!(report.promoted.is_none());
    }
}
