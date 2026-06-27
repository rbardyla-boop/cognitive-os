//! PROD-0 — the deterministic, local PRODUCTION RUNTIME PACKAGE.
//!
//! This sprint answers exactly ONE question: *can the verified Cognitive OS path be PACKAGED for
//! local runtime use — complete, pinned, reversible, no-training, and smoke-ready?* It does NOT claim
//! live production, external deployment, user traffic, or a successful production smoke. It packages a
//! local runtime artifact; it does not run one. S11 proves runtime execution AFTER S10.
//!
//! It CONSUMES the REAL MODEL-PROMOTE-0 evaluation: for a model-bearing package it runs
//! [`evaluate_model_promotion`] itself over the supplied [`ModelPromotionInput`] (the full SCORE-0 ->
//! ... -> PROMOTE chain, so the decision is DERIVED, never handed in), and the
//! `local_promoted_ready_runtime` mode requires that decision be exactly
//! [`ModelPromotionDecision::PromotionReady`]. A `local_no_model_runtime` packages the substrate
//! runtime with no model slot. Every package is no-training by default
//! ([`RuntimeNoTrainingMode::NoTraining`] is the only representable training state) and offline by
//! default (an enabled training mode or network is REFUSED).
//!
//! It is CLOSED BY DEFAULT: a missing runtime config / version / rollback / runbook / receipt-output /
//! replay-output, an unpinned or uncorroborated model/baseline hash, an enabled training mode or
//! network, or unchecked authority drift each REFUSE the package (14 [`ProductionRuntimeRefusal`]
//! reasons).
//!
//! Crucially, a packaged runtime is NOT production: every forbidden-action flag on the package and the
//! sealed [`ProductionRuntimeReceipt`] (`deploys_model`, `starts_production_service`,
//! `replaces_baseline`, `trains`, `mutates_weights`, `creates_evidence`, `creates_memory`,
//! `grants_authority`, `opens_p12`, `claims_production`, `serves_traffic`) is sourced from the
//! structural const [`PACKAGE_IS_PRODUCTION`] (`false`). The package `requires_s11_smoke` before any
//! production claim, records local/offline mode, and the deeper P12 gate
//! (`reading_train_gate::decide`) stays `training_justified = false`. Reports are `Serialize` but never
//! `Deserialize`: a serialized package is re-derived from the same input and byte-compared, so
//! tampering is refused.
//!
//! The boundary, recorded verbatim in [`PRODUCTION_RUNTIME_BOUNDARY_LINES`]:
//!
//!   The production runtime package prepares a local runtime artifact.
//!   It does not train.
//!   It does not mutate weights.
//!   It does not deploy models.
//!   It does not start production service.
//!   It does not replace the baseline.
//!   It does not create truth, memory, or evidence.
//!   It does not grant new authority.
//!   ProductionRuntimePackage is not production smoke.

use crate::{
    detect_failures, evaluate_candidate, evaluate_candidate_json, evaluate_model_promotion,
    run_training_attempt, verifier_score_matrix, AttemptAuthorizationReceipt, AuthorityDriftCheck,
    BaselineModelRef, CandidateEvalBattery, CandidateEvalComparison, CandidateEvalInput,
    ContaminationReportReceipt, DatasetReadinessReceipt, EvalComparison, EvalCondition,
    EvalDimension, EvalRun, FailureClass, FailureContext, FailureObservation, FailureSignal,
    HoldoutReadinessReceipt, HoldoutReport, ModelEvalBattery, ModelNeedCandidate,
    ModelPromotionDecision, ModelPromotionInput, OperatorAuthorizationReceipt,
    ProductionSafetyPlanReceipt, PromotionCandidateReceipt, PromotionEvalReceipt,
    PromotionOperatorApprovalReceipt, PromotionRollbackReceipt, PromotionRuntimeConfigReceipt,
    RollbackPlanReceipt, SafetyBoundaryReport, ScoreClass, ScoreReason, TrainingAttemptInput,
    TrainingAttemptMode, TrainingBaselineArtifact, TrainingDatasetBundle, TrainingGateInput,
    TrainingHoldoutBundle, TrainingRollbackArtifact, TrainingRunConfig, RECURRENCE_THRESHOLD,
};
use serde::Serialize;

/// A non-cryptographic, dependency-free FNV-1a content pin — byte-identical to MODEL-PROMOTE-0's own
/// `fnv1a_hex`, so an eval-report hash computed here corroborates the one the promotion gate re-derives.
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

/// The schema tag stamped on every serialized runtime artifact.
const SCHEMA: &str = "production-runtime-v0.1";

/// THE structural invariant: packaging a local runtime is not, by itself, production — not a
/// deployment, a running service, served traffic, a baseline replacement, training, or any authority
/// grant. Every forbidden-action flag is sourced from this const, so no code path can set one true.
const PACKAGE_IS_PRODUCTION: bool = false;

/// Exactly three runtime modes.
pub const PROD_RUNTIME_MODE_COUNT: usize = 3;

/// The three runtime-mode slugs, in canonical order.
pub const PROD_RUNTIME_MODE_NAMES: [&str; PROD_RUNTIME_MODE_COUNT] = [
    "local_no_model_runtime",
    "local_candidate_ready_runtime",
    "local_promoted_ready_runtime",
];

/// Exactly fourteen refusal reasons.
pub const PROD_RUNTIME_REFUSAL_COUNT: usize = 14;

/// The fourteen refusal-reason slugs, in canonical order.
pub const PROD_RUNTIME_REFUSAL_NAMES: [&str; PROD_RUNTIME_REFUSAL_COUNT] = [
    "missing_runtime_config",
    "missing_promotion_report",
    "promotion_not_ready",
    "missing_model_artifact_hash",
    "missing_baseline_hash",
    "missing_rollback_artifact",
    "missing_version_receipt",
    "missing_operator_runbook",
    "training_mode_enabled",
    "unauthorized_network_enabled",
    "missing_receipt_output_path",
    "missing_replay_output_path",
    "authority_drift_detected",
    "serialized_runtime_package_tamper_refused",
];

/// The fixed runtime scenario matrix size.
pub const PROD_RUNTIME_SCENARIO_COUNT: usize = 20;

/// The full verified path the runtime package DESCRIBES (it does not execute it here).
pub const PROD_RUNTIME_VERIFIED_PATH: [&str; 11] = [
    "curate",
    "read",
    "corpus",
    "score",
    "fail_detect",
    "model_eval",
    "training_gate",
    "training_attempt",
    "candidate_eval",
    "promotion_gate",
    "runtime_receipt",
];

/// The cannot-bypass boundary, recorded verbatim.
pub const PRODUCTION_RUNTIME_BOUNDARY_LINES: [&str; 9] = [
    "The production runtime package prepares a local runtime artifact.",
    "It does not train.",
    "It does not mutate weights.",
    "It does not deploy models.",
    "It does not start production service.",
    "It does not replace the baseline.",
    "It does not create truth, memory, or evidence.",
    "It does not grant new authority.",
    "ProductionRuntimePackage is not production smoke.",
];

// --- mode / no-training / outcome / refusal taxonomies ---

/// The runtime mode the package was requested in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductionRuntimeMode {
    /// The substrate runtime with no model slot.
    LocalNoModelRuntime,
    /// A model slot holding an evaluated candidate (not necessarily promotion-ready).
    LocalCandidateReadyRuntime,
    /// A model slot holding a promotion-ready model — packaged for a LATER smoke/deploy decision, NOT
    /// deployed or serving traffic.
    LocalPromotedReadyRuntime,
}

impl ProductionRuntimeMode {
    /// Every mode, in canonical order.
    pub const ALL: [ProductionRuntimeMode; PROD_RUNTIME_MODE_COUNT] = [
        ProductionRuntimeMode::LocalNoModelRuntime,
        ProductionRuntimeMode::LocalCandidateReadyRuntime,
        ProductionRuntimeMode::LocalPromotedReadyRuntime,
    ];

    /// The stable slug for this mode.
    pub fn tag(&self) -> &'static str {
        match self {
            ProductionRuntimeMode::LocalNoModelRuntime => "local_no_model_runtime",
            ProductionRuntimeMode::LocalCandidateReadyRuntime => "local_candidate_ready_runtime",
            ProductionRuntimeMode::LocalPromotedReadyRuntime => "local_promoted_ready_runtime",
        }
    }

    /// Whether this mode uses a model slot (and so consumes a promotion report).
    pub fn uses_model(&self) -> bool {
        !matches!(self, ProductionRuntimeMode::LocalNoModelRuntime)
    }

    /// Whether this mode requires the consumed promotion to be `PromotionReady`.
    pub fn requires_promotion_ready(&self) -> bool {
        matches!(self, ProductionRuntimeMode::LocalPromotedReadyRuntime)
    }
}

/// The training mode of the runtime. A SINGLE variant by design: the only representable training state
/// is `NoTraining` — a training runtime cannot be constructed, and a request to enable training is
/// refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimeNoTrainingMode {
    /// Training is disabled — the runtime never trains.
    NoTraining,
}

impl RuntimeNoTrainingMode {
    /// The stable slug.
    pub fn tag(&self) -> &'static str {
        match self {
            RuntimeNoTrainingMode::NoTraining => "no_training",
        }
    }
}

/// The terminal outcome of a packaging attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductionRuntimeOutcome {
    /// The runtime artifact was packaged (no production claim).
    Packaged,
    /// The packaging was refused (at least one prerequisite unmet).
    Refused,
}

impl ProductionRuntimeOutcome {
    /// The stable slug.
    pub fn tag(&self) -> &'static str {
        match self {
            ProductionRuntimeOutcome::Packaged => "packaged",
            ProductionRuntimeOutcome::Refused => "refused",
        }
    }
}

/// Why the package was refused. The first thirteen are packaging-path reasons; the fourteenth
/// (`SerializedRuntimePackageTamperRefused`) is emitted only by the serialized-package re-derivation
/// path (a tampered package is never trusted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductionRuntimeRefusal {
    /// No deterministic runtime config.
    MissingRuntimeConfig,
    /// A model mode was requested but no promotion report was supplied.
    MissingPromotionReport,
    /// `local_promoted_ready_runtime` was requested but the consumed promotion is not ready.
    PromotionNotReady,
    /// A model slot was requested but its artifact hash is absent or uncorroborated.
    MissingModelArtifactHash,
    /// A model slot was requested but its baseline hash is absent or uncorroborated.
    MissingBaselineHash,
    /// No rollback artifact.
    MissingRollbackArtifact,
    /// No runtime version receipt.
    MissingVersionReceipt,
    /// No operator runbook.
    MissingOperatorRunbook,
    /// The config requested training mode (refused — the runtime is no-training).
    TrainingModeEnabled,
    /// The config enabled an unauthorized network.
    UnauthorizedNetworkEnabled,
    /// No receipt output path.
    MissingReceiptOutputPath,
    /// No replay output path.
    MissingReplayOutputPath,
    /// The authority-drift check was not run, or it detected drift.
    AuthorityDriftDetected,
    /// A serialized runtime package did not match its re-derivation and was refused.
    SerializedRuntimePackageTamperRefused,
}

impl ProductionRuntimeRefusal {
    /// Every refusal reason, in canonical order.
    pub const ALL: [ProductionRuntimeRefusal; PROD_RUNTIME_REFUSAL_COUNT] = [
        ProductionRuntimeRefusal::MissingRuntimeConfig,
        ProductionRuntimeRefusal::MissingPromotionReport,
        ProductionRuntimeRefusal::PromotionNotReady,
        ProductionRuntimeRefusal::MissingModelArtifactHash,
        ProductionRuntimeRefusal::MissingBaselineHash,
        ProductionRuntimeRefusal::MissingRollbackArtifact,
        ProductionRuntimeRefusal::MissingVersionReceipt,
        ProductionRuntimeRefusal::MissingOperatorRunbook,
        ProductionRuntimeRefusal::TrainingModeEnabled,
        ProductionRuntimeRefusal::UnauthorizedNetworkEnabled,
        ProductionRuntimeRefusal::MissingReceiptOutputPath,
        ProductionRuntimeRefusal::MissingReplayOutputPath,
        ProductionRuntimeRefusal::AuthorityDriftDetected,
        ProductionRuntimeRefusal::SerializedRuntimePackageTamperRefused,
    ];

    /// The stable slug for this refusal reason.
    pub fn tag(&self) -> &'static str {
        match self {
            ProductionRuntimeRefusal::MissingRuntimeConfig => "missing_runtime_config",
            ProductionRuntimeRefusal::MissingPromotionReport => "missing_promotion_report",
            ProductionRuntimeRefusal::PromotionNotReady => "promotion_not_ready",
            ProductionRuntimeRefusal::MissingModelArtifactHash => "missing_model_artifact_hash",
            ProductionRuntimeRefusal::MissingBaselineHash => "missing_baseline_hash",
            ProductionRuntimeRefusal::MissingRollbackArtifact => "missing_rollback_artifact",
            ProductionRuntimeRefusal::MissingVersionReceipt => "missing_version_receipt",
            ProductionRuntimeRefusal::MissingOperatorRunbook => "missing_operator_runbook",
            ProductionRuntimeRefusal::TrainingModeEnabled => "training_mode_enabled",
            ProductionRuntimeRefusal::UnauthorizedNetworkEnabled => "unauthorized_network_enabled",
            ProductionRuntimeRefusal::MissingReceiptOutputPath => "missing_receipt_output_path",
            ProductionRuntimeRefusal::MissingReplayOutputPath => "missing_replay_output_path",
            ProductionRuntimeRefusal::AuthorityDriftDetected => "authority_drift_detected",
            ProductionRuntimeRefusal::SerializedRuntimePackageTamperRefused => {
                "serialized_runtime_package_tamper_refused"
            }
        }
    }
}

// --- inputs (never trusted off-wire: Debug + Clone, no Serialize, no Deserialize) ---

/// A deterministic, hash-pinned runtime configuration. No-training and offline by default: an enabled
/// training mode or network is REFUSED.
#[derive(Debug, Clone)]
pub struct ProductionRuntimeConfig {
    /// The content hash pinning this configuration.
    pub config_hash: String,
    /// Whether the configuration is deterministic.
    pub deterministic: bool,
    /// Whether the config requested training mode (must be false — the runtime is no-training).
    pub training_mode_requested: bool,
    /// Whether the config enabled a network (must be false — offline by default).
    pub network_enabled: bool,
    /// Whether the runtime runs local/offline.
    pub local_offline: bool,
}

/// A pinned runtime version receipt.
#[derive(Debug, Clone)]
pub struct RuntimeVersionReceipt {
    /// The runtime version string.
    pub runtime_version: String,
    /// The content hash pinning the version.
    pub version_hash: String,
}

/// A hash-pinned rollback artifact for the runtime.
#[derive(Debug, Clone)]
pub struct RuntimeRollbackReceipt {
    /// The content hash pinning the rollback target.
    pub rollback_hash: String,
    /// Whether the rollback path was verified.
    pub verified: bool,
}

/// A model slot: the pinned artifact + baseline hashes, corroborated against the consumed promotion
/// report.
#[derive(Debug, Clone)]
pub struct RuntimeModelSlot {
    /// The pinned model artifact hash.
    pub model_artifact_hash: String,
    /// The pinned baseline hash.
    pub baseline_hash: String,
}

/// The operator runbook receipt — the operator has the production-runtime runbook in hand.
#[derive(Debug, Clone)]
pub struct OperatorRunbookReceipt {
    /// The runbook identifier.
    pub runbook_id: String,
}

/// The full set of inputs the packager weighs. INPUT type (never `Serialize`): for a model mode it
/// re-runs the real MODEL-PROMOTE-0 evaluation. Closed by default.
#[derive(Debug)]
pub struct ProductionRuntimeInput {
    /// The requested runtime mode.
    pub mode: ProductionRuntimeMode,
    /// The MODEL-PROMOTE-0 input the packager runs `evaluate_model_promotion` over (model modes).
    pub promotion: Option<ModelPromotionInput>,
    /// The deterministic runtime config.
    pub runtime_config: Option<ProductionRuntimeConfig>,
    /// The pinned model slot (model modes).
    pub model_slot: Option<RuntimeModelSlot>,
    /// The runtime version receipt.
    pub version: Option<RuntimeVersionReceipt>,
    /// The rollback artifact.
    pub rollback: Option<RuntimeRollbackReceipt>,
    /// The operator runbook receipt.
    pub operator_runbook: Option<OperatorRunbookReceipt>,
    /// Where the runtime would write its receipt.
    pub receipt_output_path: Option<String>,
    /// Where the runtime would write its replay.
    pub replay_output_path: Option<String>,
    /// The authority-drift check (unchecked by default).
    pub authority_drift: AuthorityDriftCheck,
}

impl ProductionRuntimeInput {
    /// The closed-by-default input: nothing supplied, drift unchecked. Packaging is refused.
    pub fn closed_by_default(mode: ProductionRuntimeMode) -> Self {
        Self {
            mode,
            promotion: None,
            runtime_config: None,
            model_slot: None,
            version: None,
            rollback: None,
            operator_runbook: None,
            receipt_output_path: None,
            replay_output_path: None,
            authority_drift: AuthorityDriftCheck::unchecked(),
        }
    }
}

// --- the boundary record ---

/// The inert boundary: every forbidden action is `false`. Stamped on every package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductionRuntimeBoundary {
    /// The package never trains.
    pub trains: bool,
    /// The package never mutates weights.
    pub mutates_weights: bool,
    /// The package never deploys a model.
    pub deploys_model: bool,
    /// The package never starts a production service.
    pub starts_production_service: bool,
    /// The package never replaces the baseline.
    pub replaces_baseline: bool,
    /// The package never creates truth.
    pub creates_truth: bool,
    /// The package never creates memory.
    pub creates_memory: bool,
    /// The package never creates evidence.
    pub creates_evidence: bool,
    /// The package never grants new authority.
    pub grants_authority: bool,
}

impl ProductionRuntimeBoundary {
    fn inert() -> Self {
        Self {
            trains: PACKAGE_IS_PRODUCTION,
            mutates_weights: PACKAGE_IS_PRODUCTION,
            deploys_model: PACKAGE_IS_PRODUCTION,
            starts_production_service: PACKAGE_IS_PRODUCTION,
            replaces_baseline: PACKAGE_IS_PRODUCTION,
            creates_truth: PACKAGE_IS_PRODUCTION,
            creates_memory: PACKAGE_IS_PRODUCTION,
            creates_evidence: PACKAGE_IS_PRODUCTION,
            grants_authority: PACKAGE_IS_PRODUCTION,
        }
    }

    /// True iff every forbidden action is inert.
    pub fn all_inert(&self) -> bool {
        !self.trains
            && !self.mutates_weights
            && !self.deploys_model
            && !self.starts_production_service
            && !self.replaces_baseline
            && !self.creates_truth
            && !self.creates_memory
            && !self.creates_evidence
            && !self.grants_authority
    }
}

// --- the manifest + sealed receipt ---

/// The runtime manifest: the pinned lineage + the verified path the runtime would run. Always built
/// (describes the requested package). `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionRuntimeManifest {
    /// The schema tag.
    pub schema: &'static str,
    /// The requested runtime mode.
    pub mode: ProductionRuntimeMode,
    /// The pinned runtime version, if supplied.
    pub runtime_version: Option<String>,
    /// The pinned version hash, if supplied.
    pub version_hash: Option<String>,
    /// The pinned config hash, if supplied.
    pub config_hash: Option<String>,
    /// The pinned model artifact hash, if a model slot is used.
    pub model_artifact_hash: Option<String>,
    /// The pinned baseline hash, if a model slot is used.
    pub baseline_hash: Option<String>,
    /// The pinned rollback hash, if supplied.
    pub rollback_hash: Option<String>,
    /// The receipt output path, if supplied.
    pub receipt_output_path: Option<String>,
    /// The replay output path, if supplied.
    pub replay_output_path: Option<String>,
    /// The full verified path the runtime would run (described, not executed).
    pub verified_path: [&'static str; 11],
    /// Whether the runtime runs local/offline.
    pub local_offline: bool,
    /// The (only) training mode — always `NoTraining`.
    pub no_training_mode: RuntimeNoTrainingMode,
    /// Always `true`: the package requires S11 production smoke before any production claim.
    pub requires_s11_smoke: bool,
}

/// The SEALED runtime receipt produced ONLY on a successful package. It records the runtime is
/// smoke-ready — it deploys nothing, starts no service, claims no production. `Serialize` but never
/// `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionRuntimeReceipt {
    /// The schema tag.
    pub schema: &'static str,
    /// The runtime mode.
    pub mode: ProductionRuntimeMode,
    /// The pinned runtime version.
    pub runtime_version: String,
    /// The pinned config hash.
    pub config_hash: String,
    /// Always `true`: requires S11 production smoke before any production claim.
    pub requires_s11_smoke: bool,
    /// Always `NoTraining`.
    pub no_training_mode: RuntimeNoTrainingMode,
    /// Always `false`: the receipt deploys no model.
    pub deploys_model: bool,
    /// Always `false`: the receipt starts no production service.
    pub starts_production_service: bool,
    /// Always `false`: the receipt replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the receipt trains nothing.
    pub trains: bool,
    /// Always `false`: the receipt mutates no weights.
    pub mutates_weights: bool,
    /// Always `false`: the receipt creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the receipt creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the receipt grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the receipt opens no P12.
    pub opens_p12: bool,
    /// Always `false`: the receipt claims no production.
    pub claims_production: bool,
    /// Always `false`: the receipt serves no traffic.
    pub serves_traffic: bool,
}

// --- the package (top-level report) ---

/// The packager's verdict on whether a local runtime artifact could be prepared. `Serialize` but never
/// `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionRuntimePackage {
    /// The schema tag.
    pub schema: &'static str,
    /// The requested runtime mode.
    pub mode: ProductionRuntimeMode,
    /// The terminal outcome.
    pub outcome: ProductionRuntimeOutcome,
    /// The consumed promotion decision slug (`None` for no-model or absent report).
    pub model_decision: Option<&'static str>,
    /// Why packaging was refused (empty iff packaged).
    pub refusals: Vec<ProductionRuntimeRefusal>,
    /// The runtime manifest (always present — describes the requested package).
    pub manifest: ProductionRuntimeManifest,
    /// The sealed runtime receipt (present ONLY when packaged).
    pub receipt: Option<ProductionRuntimeReceipt>,
    /// Always `true`: the package requires S11 production smoke before any production claim.
    pub requires_s11_smoke: bool,
    /// Always `false`: the package deploys no model.
    pub deploys_model: bool,
    /// Always `false`: the package starts no production service.
    pub starts_production_service: bool,
    /// Always `false`: the package replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the package trains nothing.
    pub trains: bool,
    /// Always `false`: the package mutates no weights.
    pub mutates_weights: bool,
    /// Always `false`: the package creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the package creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the package grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the package opens no P12.
    pub opens_p12: bool,
    /// Always `false`: the package claims no production.
    pub claims_production: bool,
    /// Always `false`: the package serves no traffic.
    pub serves_traffic: bool,
    /// Always `false`: the package does not set P12 `training_justified`.
    pub training_justified: bool,
    /// The inert boundary.
    pub boundary: ProductionRuntimeBoundary,
}

/// True iff `pinned` is present (non-empty) AND corroborated by the promotion-derived `derived` value.
fn hash_ok(pinned: &str, derived: &Option<String>) -> bool {
    !pinned.is_empty() && derived.as_deref() == Some(pinned)
}

/// Package the local production runtime over `input`. For a model mode it runs the REAL MODEL-PROMOTE-0
/// evaluation and corroborates the pinned hashes; the promoted-ready mode also requires
/// `PromotionReady`. No-training and offline are defaults (an enabled training mode or network is
/// refused). Emits a packaged artifact only when every requirement holds; otherwise refuses. Deploys
/// nothing, starts no service, claims no production.
pub fn package_production_runtime(input: &ProductionRuntimeInput) -> ProductionRuntimePackage {
    let mut refusals: Vec<ProductionRuntimeRefusal> = Vec::new();

    // Runtime config: present, no-training, offline.
    match &input.runtime_config {
        None => refusals.push(ProductionRuntimeRefusal::MissingRuntimeConfig),
        Some(c) => {
            if c.training_mode_requested {
                refusals.push(ProductionRuntimeRefusal::TrainingModeEnabled);
            }
            if c.network_enabled {
                refusals.push(ProductionRuntimeRefusal::UnauthorizedNetworkEnabled);
            }
        }
    }

    // Common receipts.
    if input.version.is_none() {
        refusals.push(ProductionRuntimeRefusal::MissingVersionReceipt);
    }
    if input.rollback.is_none() {
        refusals.push(ProductionRuntimeRefusal::MissingRollbackArtifact);
    }
    if input.operator_runbook.is_none() {
        refusals.push(ProductionRuntimeRefusal::MissingOperatorRunbook);
    }
    if input
        .receipt_output_path
        .as_deref()
        .unwrap_or("")
        .is_empty()
    {
        refusals.push(ProductionRuntimeRefusal::MissingReceiptOutputPath);
    }
    if input.replay_output_path.as_deref().unwrap_or("").is_empty() {
        refusals.push(ProductionRuntimeRefusal::MissingReplayOutputPath);
    }
    if !input.authority_drift.is_clean() {
        refusals.push(ProductionRuntimeRefusal::AuthorityDriftDetected);
    }

    // Model-slot modes: CONSUME the real MODEL-PROMOTE-0 evaluation and corroborate the pinned hashes.
    let mut model_decision: Option<&'static str> = None;
    if input.mode.uses_model() {
        let (rep_candidate_hash, rep_baseline_hash) = match &input.promotion {
            None => {
                refusals.push(ProductionRuntimeRefusal::MissingPromotionReport);
                (None, None)
            }
            Some(pi) => {
                let report = evaluate_model_promotion(pi);
                model_decision = Some(report.decision.tag());
                if input.mode.requires_promotion_ready()
                    && report.decision != ModelPromotionDecision::PromotionReady
                {
                    refusals.push(ProductionRuntimeRefusal::PromotionNotReady);
                }
                (report.candidate_hash.clone(), report.baseline_hash.clone())
            }
        };
        match &input.model_slot {
            None => {
                refusals.push(ProductionRuntimeRefusal::MissingModelArtifactHash);
                refusals.push(ProductionRuntimeRefusal::MissingBaselineHash);
            }
            Some(s) => {
                if !hash_ok(&s.model_artifact_hash, &rep_candidate_hash) {
                    refusals.push(ProductionRuntimeRefusal::MissingModelArtifactHash);
                }
                if !hash_ok(&s.baseline_hash, &rep_baseline_hash) {
                    refusals.push(ProductionRuntimeRefusal::MissingBaselineHash);
                }
            }
        }
    }

    let model_artifact_hash = input
        .model_slot
        .as_ref()
        .map(|s| s.model_artifact_hash.clone());
    let baseline_hash = input.model_slot.as_ref().map(|s| s.baseline_hash.clone());

    let manifest = ProductionRuntimeManifest {
        schema: SCHEMA,
        mode: input.mode,
        runtime_version: input.version.as_ref().map(|v| v.runtime_version.clone()),
        version_hash: input.version.as_ref().map(|v| v.version_hash.clone()),
        config_hash: input.runtime_config.as_ref().map(|c| c.config_hash.clone()),
        model_artifact_hash,
        baseline_hash,
        rollback_hash: input.rollback.as_ref().map(|r| r.rollback_hash.clone()),
        receipt_output_path: input.receipt_output_path.clone(),
        replay_output_path: input.replay_output_path.clone(),
        verified_path: PROD_RUNTIME_VERIFIED_PATH,
        local_offline: input
            .runtime_config
            .as_ref()
            .map(|c| c.local_offline)
            .unwrap_or(false),
        no_training_mode: RuntimeNoTrainingMode::NoTraining,
        requires_s11_smoke: true,
    };

    let outcome = if refusals.is_empty() {
        ProductionRuntimeOutcome::Packaged
    } else {
        ProductionRuntimeOutcome::Refused
    };

    // The sealed receipt is produced ONLY on a successful package; the unwraps are sound (an empty
    // refusal set implies the config and version are present).
    let receipt = if outcome == ProductionRuntimeOutcome::Packaged {
        let c = input
            .runtime_config
            .as_ref()
            .expect("config present when packaged");
        let v = input
            .version
            .as_ref()
            .expect("version present when packaged");
        Some(ProductionRuntimeReceipt {
            schema: SCHEMA,
            mode: input.mode,
            runtime_version: v.runtime_version.clone(),
            config_hash: c.config_hash.clone(),
            requires_s11_smoke: true,
            no_training_mode: RuntimeNoTrainingMode::NoTraining,
            deploys_model: PACKAGE_IS_PRODUCTION,
            starts_production_service: PACKAGE_IS_PRODUCTION,
            replaces_baseline: PACKAGE_IS_PRODUCTION,
            trains: PACKAGE_IS_PRODUCTION,
            mutates_weights: PACKAGE_IS_PRODUCTION,
            creates_evidence: PACKAGE_IS_PRODUCTION,
            creates_memory: PACKAGE_IS_PRODUCTION,
            grants_authority: PACKAGE_IS_PRODUCTION,
            opens_p12: PACKAGE_IS_PRODUCTION,
            claims_production: PACKAGE_IS_PRODUCTION,
            serves_traffic: PACKAGE_IS_PRODUCTION,
        })
    } else {
        None
    };

    ProductionRuntimePackage {
        schema: SCHEMA,
        mode: input.mode,
        outcome,
        model_decision,
        refusals,
        manifest,
        receipt,
        requires_s11_smoke: true,
        deploys_model: PACKAGE_IS_PRODUCTION,
        starts_production_service: PACKAGE_IS_PRODUCTION,
        replaces_baseline: PACKAGE_IS_PRODUCTION,
        trains: PACKAGE_IS_PRODUCTION,
        mutates_weights: PACKAGE_IS_PRODUCTION,
        creates_evidence: PACKAGE_IS_PRODUCTION,
        creates_memory: PACKAGE_IS_PRODUCTION,
        grants_authority: PACKAGE_IS_PRODUCTION,
        opens_p12: PACKAGE_IS_PRODUCTION,
        claims_production: PACKAGE_IS_PRODUCTION,
        serves_traffic: PACKAGE_IS_PRODUCTION,
        training_justified: PACKAGE_IS_PRODUCTION,
        boundary: ProductionRuntimeBoundary::inert(),
    }
}

/// The runtime package serialized to canonical JSON.
pub fn package_production_runtime_json(input: &ProductionRuntimeInput) -> String {
    serde_json::to_string(&package_production_runtime(input)).expect("runtime package serializes")
}

/// What can go wrong verifying a serialized runtime package.
#[derive(Debug, PartialEq, Eq)]
pub enum ProductionRuntimeError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the package from the SAME input and byte-compare against `candidate`. The package is
/// `Serialize` but never `Deserialize`: a serialized package is NOT trusted as authority — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_production_runtime_package_json(
    input: &ProductionRuntimeInput,
    candidate: &str,
) -> Result<(), ProductionRuntimeError> {
    if candidate == package_production_runtime_json(input) {
        Ok(())
    } else {
        Err(ProductionRuntimeError::ReplayMismatch)
    }
}

// --- building a REAL MODEL-PROMOTE-0 input (the SCORE-0 -> ... -> PROMOTE chain) ---

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

fn eval_comparisons(target_improves: bool) -> Vec<CandidateEvalComparison> {
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

fn eval_input(target_improves: bool, failures: &[FailureObservation]) -> CandidateEvalInput {
    CandidateEvalInput {
        candidate: Some(
            run_training_attempt(&full_attempt(failures))
                .candidate
                .expect("a fully-authorized TRAIN-0 attempt prepares a candidate"),
        ),
        baseline: Some(BaselineModelRef {
            baseline_hash: "baseline-hash".to_string(),
        }),
        battery: Some(CandidateEvalBattery {
            comparisons: eval_comparisons(target_improves),
            holdout: eval_clean_holdout(),
            safety: eval_clean_safety(),
        }),
    }
}

/// A MODEL-PROMOTE-0 input that evaluates to `PromotionReady` (target improves -> ready eval -> full
/// promotion).
fn ready_promotion(failures: &[FailureObservation]) -> ModelPromotionInput {
    promotion_over(eval_input(true, failures))
}

/// A MODEL-PROMOTE-0 input whose eval is `needs_more_evidence` (target does not improve) -> the report
/// is NOT ready but still carries the candidate/baseline hashes.
fn not_ready_promotion(failures: &[FailureObservation]) -> ModelPromotionInput {
    promotion_over(eval_input(false, failures))
}

fn promotion_over(eval: CandidateEvalInput) -> ModelPromotionInput {
    let report = evaluate_candidate(&eval);
    let eval_report_hash = fnv1a_hex(&evaluate_candidate_json(&eval));
    ModelPromotionInput {
        candidate: Some(PromotionCandidateReceipt {
            candidate_artifact_hash: report.candidate_hash.clone().unwrap_or_default(),
            baseline_artifact_hash: report.baseline_hash.clone().unwrap_or_default(),
            dataset_hash: report.dataset_hash.clone().unwrap_or_default(),
        }),
        eval: Some(PromotionEvalReceipt {
            eval,
            eval_report_hash,
        }),
        operator_approval: Some(PromotionOperatorApprovalReceipt {
            operator: "operator".to_string(),
            promotion_scope: "local-candidate-promotion".to_string(),
            approves_promotion: true,
        }),
        rollback: Some(PromotionRollbackReceipt {
            rollback_hash: "promotion-rollback-hash".to_string(),
            verified: true,
        }),
        runtime_config: Some(PromotionRuntimeConfigReceipt {
            runtime_config_hash: "runtime-config-hash".to_string(),
            baseline_replacement_pending: true,
        }),
        production_safety: Some(ProductionSafetyPlanReceipt {
            plan_id: "production-safety-plan-0".to_string(),
        }),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

/// The model artifact / baseline hashes the promotion report corroborates (for the model slot pins).
fn promotion_hashes(pi: &ModelPromotionInput) -> (String, String) {
    let report = evaluate_model_promotion(pi);
    (
        report.candidate_hash.clone().unwrap_or_default(),
        report.baseline_hash.clone().unwrap_or_default(),
    )
}

// --- runtime-input builders ---

fn runtime_config() -> ProductionRuntimeConfig {
    ProductionRuntimeConfig {
        config_hash: "runtime-pkg-config-hash".to_string(),
        deterministic: true,
        training_mode_requested: false,
        network_enabled: false,
        local_offline: true,
    }
}

fn version_receipt() -> RuntimeVersionReceipt {
    RuntimeVersionReceipt {
        runtime_version: "cognitive-os-runtime-0.1.0".to_string(),
        version_hash: "runtime-version-hash".to_string(),
    }
}

fn rollback_receipt() -> RuntimeRollbackReceipt {
    RuntimeRollbackReceipt {
        rollback_hash: "runtime-rollback-hash".to_string(),
        verified: true,
    }
}

fn runbook_receipt() -> OperatorRunbookReceipt {
    OperatorRunbookReceipt {
        runbook_id: "production-runtime-runbook-0".to_string(),
    }
}

/// The common (model-independent) receipts present + clean drift, for a given mode.
fn base_input(mode: ProductionRuntimeMode) -> ProductionRuntimeInput {
    ProductionRuntimeInput {
        mode,
        promotion: None,
        runtime_config: Some(runtime_config()),
        model_slot: None,
        version: Some(version_receipt()),
        rollback: Some(rollback_receipt()),
        operator_runbook: Some(runbook_receipt()),
        receipt_output_path: Some("out/runtime-receipt.json".to_string()),
        replay_output_path: Some("out/runtime-replay.json".to_string()),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

/// A fully-met `local_no_model_runtime` input -> packaged.
fn no_model_input() -> ProductionRuntimeInput {
    base_input(ProductionRuntimeMode::LocalNoModelRuntime)
}

/// A fully-met `local_promoted_ready_runtime` input -> packaged (ready promotion + corroborated slot).
fn promoted_input(failures: &[FailureObservation]) -> ProductionRuntimeInput {
    let pi = ready_promotion(failures);
    let (model_artifact_hash, baseline_hash) = promotion_hashes(&pi);
    ProductionRuntimeInput {
        promotion: Some(pi),
        model_slot: Some(RuntimeModelSlot {
            model_artifact_hash,
            baseline_hash,
        }),
        ..base_input(ProductionRuntimeMode::LocalPromotedReadyRuntime)
    }
}

/// A fully-met `local_candidate_ready_runtime` input -> packaged (not-ready promotion + corroborated
/// slot; candidate-ready does NOT require PromotionReady). Test-only: the fixed matrix covers only the
/// no-model and promoted-ready packaged cells, so the candidate-ready mode is exercised by unit tests.
#[cfg(test)]
fn candidate_input(failures: &[FailureObservation]) -> ProductionRuntimeInput {
    let pi = not_ready_promotion(failures);
    let (model_artifact_hash, baseline_hash) = promotion_hashes(&pi);
    ProductionRuntimeInput {
        promotion: Some(pi),
        model_slot: Some(RuntimeModelSlot {
            model_artifact_hash,
            baseline_hash,
        }),
        ..base_input(ProductionRuntimeMode::LocalCandidateReadyRuntime)
    }
}

// --- the runtime scenario matrix (observes the real packager over constructed inputs) ---

/// One scenario cell: the OBSERVED outcome of running the real packager over a constructed input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionRuntimeScenarioCell {
    /// The scenario name.
    pub name: &'static str,
    /// The observed mode slug.
    pub mode: &'static str,
    /// The observed outcome slug.
    pub outcome: &'static str,
    /// The observed refusal-reason slugs.
    pub refusals: Vec<&'static str>,
    /// Whether a sealed runtime receipt was produced.
    pub sealed_receipt: bool,
    /// Whether production stayed fully closed (no forbidden flag set; no deploy/service/traffic).
    pub production_still_closed: bool,
    /// A short human-readable detail.
    pub detail: String,
}

/// The fixed runtime scenario matrix. Every cell runs the real packager and records what it observed;
/// `production_never_opens` is the conjunction across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionRuntimeMatrix {
    /// The schema tag.
    pub schema: &'static str,
    /// The scenario cells.
    pub scenarios: Vec<ProductionRuntimeScenarioCell>,
    /// The three runtime-mode slugs.
    pub modes: [&'static str; PROD_RUNTIME_MODE_COUNT],
    /// The fourteen refusal-reason slugs.
    pub refusal_reasons: [&'static str; PROD_RUNTIME_REFUSAL_COUNT],
    /// True iff no cell opened production.
    pub production_never_opens: bool,
    /// The inert boundary.
    pub boundary: ProductionRuntimeBoundary,
}

impl ProductionRuntimeMatrix {
    /// Find a scenario cell by name.
    pub fn scenario(&self, name: &str) -> Option<&ProductionRuntimeScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn closed_for(package: &ProductionRuntimePackage) -> bool {
    let receipt_closed = match &package.receipt {
        None => true,
        Some(r) => {
            !r.deploys_model
                && !r.starts_production_service
                && !r.replaces_baseline
                && !r.trains
                && !r.mutates_weights
                && !r.creates_evidence
                && !r.creates_memory
                && !r.grants_authority
                && !r.opens_p12
                && !r.claims_production
                && !r.serves_traffic
                && r.requires_s11_smoke
        }
    };
    !package.deploys_model
        && !package.starts_production_service
        && !package.replaces_baseline
        && !package.trains
        && !package.mutates_weights
        && !package.creates_evidence
        && !package.creates_memory
        && !package.grants_authority
        && !package.opens_p12
        && !package.claims_production
        && !package.serves_traffic
        && !package.training_justified
        && package.boundary.all_inert()
        && receipt_closed
}

fn runtime_cell(
    name: &'static str,
    input: ProductionRuntimeInput,
) -> ProductionRuntimeScenarioCell {
    let package = package_production_runtime(&input);
    ProductionRuntimeScenarioCell {
        name,
        mode: package.mode.tag(),
        outcome: package.outcome.tag(),
        refusals: package.refusals.iter().map(|r| r.tag()).collect(),
        sealed_receipt: package.receipt.is_some(),
        production_still_closed: closed_for(&package),
        detail: package.outcome.tag().to_string(),
    }
}

/// The serialized-package tamper cell: tamper a real (packaged) runtime package JSON and observe the
/// re-derive verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous.
fn tamper_cell(failures: &[FailureObservation]) -> ProductionRuntimeScenarioCell {
    let input = promoted_input(failures);
    let canonical = package_production_runtime_json(&input);
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_production_runtime_package_json(&input, &tampered).is_err()
        && verify_production_runtime_package_json(&input, &canonical).is_ok();
    let package = package_production_runtime(&input);
    ProductionRuntimeScenarioCell {
        name: "serialized_runtime_package_tamper_refused",
        mode: package.mode.tag(),
        outcome: package.outcome.tag(),
        refusals: if refused {
            vec!["serialized_runtime_package_tamper_refused"]
        } else {
            vec!["VACUOUS"]
        },
        sealed_receipt: package.receipt.is_some(),
        production_still_closed: closed_for(&package) && refused,
        detail: if refused {
            "serialized_runtime_package_tamper_refused".to_string()
        } else {
            "VACUOUS: runtime verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 20-scenario runtime matrix from the REAL packager over constructed inputs.
pub fn production_runtime_matrix() -> ProductionRuntimeMatrix {
    // Derive the SCORE-0 failure set ONCE; every promotion reuses it.
    let failures = verifier_score_matrix().failures;

    let scenarios = vec![
        // 1. A no-model runtime packages with the common receipts.
        runtime_cell("local_no_model_runtime_packaged", no_model_input()),
        // 2-14. Each missing/disallowed requirement refuses (over the promoted-ready full input).
        runtime_cell(
            "missing_runtime_config_refused",
            ProductionRuntimeInput {
                runtime_config: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_promotion_report_refused",
            ProductionRuntimeInput {
                promotion: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "promotion_not_ready_refused",
            ProductionRuntimeInput {
                promotion: Some(not_ready_promotion(&failures)),
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_model_artifact_hash_refused",
            ProductionRuntimeInput {
                model_slot: Some(RuntimeModelSlot {
                    model_artifact_hash: String::new(),
                    baseline_hash: promotion_hashes(&ready_promotion(&failures)).1,
                }),
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_baseline_hash_refused",
            ProductionRuntimeInput {
                model_slot: Some(RuntimeModelSlot {
                    model_artifact_hash: promotion_hashes(&ready_promotion(&failures)).0,
                    baseline_hash: String::new(),
                }),
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_rollback_artifact_refused",
            ProductionRuntimeInput {
                rollback: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_version_receipt_refused",
            ProductionRuntimeInput {
                version: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_operator_runbook_refused",
            ProductionRuntimeInput {
                operator_runbook: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "training_mode_enabled_refused",
            ProductionRuntimeInput {
                runtime_config: Some(ProductionRuntimeConfig {
                    training_mode_requested: true,
                    ..runtime_config()
                }),
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "unauthorized_network_enabled_refused",
            ProductionRuntimeInput {
                runtime_config: Some(ProductionRuntimeConfig {
                    network_enabled: true,
                    ..runtime_config()
                }),
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_receipt_output_path_refused",
            ProductionRuntimeInput {
                receipt_output_path: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "missing_replay_output_path_refused",
            ProductionRuntimeInput {
                replay_output_path: None,
                ..promoted_input(&failures)
            },
        ),
        runtime_cell(
            "authority_drift_refused",
            ProductionRuntimeInput {
                authority_drift: AuthorityDriftCheck::drifted(),
                ..promoted_input(&failures)
            },
        ),
        // 15. A promoted-ready runtime packages.
        runtime_cell("promoted_ready_runtime_packaged", promoted_input(&failures)),
        // 16-19. Packaging is NOT deployment / service start / baseline replacement / smoke (same run).
        runtime_cell("package_is_not_deployment", promoted_input(&failures)),
        runtime_cell("package_is_not_service_start", promoted_input(&failures)),
        runtime_cell(
            "package_is_not_baseline_replacement",
            promoted_input(&failures),
        ),
        runtime_cell("package_requires_s11_smoke", promoted_input(&failures)),
        // 20. Serialized package tamper refused.
        tamper_cell(&failures),
    ];

    let production_never_opens = scenarios.iter().all(|c| c.production_still_closed);
    ProductionRuntimeMatrix {
        schema: SCHEMA,
        scenarios,
        modes: PROD_RUNTIME_MODE_NAMES,
        refusal_reasons: PROD_RUNTIME_REFUSAL_NAMES,
        production_never_opens,
        boundary: ProductionRuntimeBoundary::inert(),
    }
}

/// The runtime matrix serialized to canonical JSON.
pub fn production_runtime_matrix_json() -> String {
    serde_json::to_string(&production_runtime_matrix()).expect("runtime matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`.
pub fn verify_production_runtime_matrix_json(
    candidate: &str,
) -> Result<(), ProductionRuntimeError> {
    if candidate == production_runtime_matrix_json() {
        Ok(())
    } else {
        Err(ProductionRuntimeError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failures() -> Vec<FailureObservation> {
        verifier_score_matrix().failures
    }

    fn has(package: &ProductionRuntimePackage, r: ProductionRuntimeRefusal) -> bool {
        package.refusals.contains(&r)
    }

    #[test]
    fn package_consumes_the_real_promotion_report() {
        let f = failures();
        // A promoted-ready package observes the REAL PromotionReady decision (derived, not handed in).
        let promoted = package_production_runtime(&promoted_input(&f));
        assert_eq!(promoted.model_decision, Some("promotion_ready"));
        assert_eq!(promoted.outcome, ProductionRuntimeOutcome::Packaged);
        // A not-ready promotion in promoted-ready mode is refused (decision derived as denied).
        let not_ready = package_production_runtime(&ProductionRuntimeInput {
            promotion: Some(not_ready_promotion(&f)),
            ..promoted_input(&f)
        });
        assert_eq!(not_ready.model_decision, Some("promotion_denied"));
        assert!(has(&not_ready, ProductionRuntimeRefusal::PromotionNotReady));
    }

    #[test]
    fn local_no_model_runtime_is_packaged() {
        let package = package_production_runtime(&no_model_input());
        assert_eq!(package.mode, ProductionRuntimeMode::LocalNoModelRuntime);
        assert_eq!(package.outcome, ProductionRuntimeOutcome::Packaged);
        assert!(package.refusals.is_empty());
        assert!(package.receipt.is_some());
        // No model slot for the substrate runtime.
        assert!(package.manifest.model_artifact_hash.is_none());
        assert_eq!(package.model_decision, None);
    }

    #[test]
    fn candidate_ready_runtime_does_not_require_promotion_ready() {
        let f = failures();
        // A not-ready promotion is fine for the CANDIDATE-ready mode (no PromotionReady requirement).
        let package = package_production_runtime(&candidate_input(&f));
        assert_eq!(
            package.mode,
            ProductionRuntimeMode::LocalCandidateReadyRuntime
        );
        assert_eq!(package.outcome, ProductionRuntimeOutcome::Packaged);
        assert!(!has(&package, ProductionRuntimeRefusal::PromotionNotReady));
        assert!(package.manifest.model_artifact_hash.is_some());
    }

    #[test]
    fn promoted_ready_runtime_requires_promotion_ready() {
        let f = failures();
        let ready = package_production_runtime(&promoted_input(&f));
        assert_eq!(ready.outcome, ProductionRuntimeOutcome::Packaged);
        // Swapping in a not-ready promotion flips it to refused.
        let denied = package_production_runtime(&ProductionRuntimeInput {
            promotion: Some(not_ready_promotion(&f)),
            ..promoted_input(&f)
        });
        assert_eq!(denied.outcome, ProductionRuntimeOutcome::Refused);
        assert!(has(&denied, ProductionRuntimeRefusal::PromotionNotReady));
    }

    #[test]
    fn missing_promotion_report_and_hashes_are_refused() {
        let f = failures();
        let no_report = package_production_runtime(&ProductionRuntimeInput {
            promotion: None,
            ..promoted_input(&f)
        });
        assert!(has(
            &no_report,
            ProductionRuntimeRefusal::MissingPromotionReport
        ));

        let no_slot = package_production_runtime(&ProductionRuntimeInput {
            model_slot: None,
            ..promoted_input(&f)
        });
        assert!(has(
            &no_slot,
            ProductionRuntimeRefusal::MissingModelArtifactHash
        ));
        assert!(has(&no_slot, ProductionRuntimeRefusal::MissingBaselineHash));

        // An uncorroborated (wrong) model hash is refused.
        let wrong = package_production_runtime(&ProductionRuntimeInput {
            model_slot: Some(RuntimeModelSlot {
                model_artifact_hash: "wrong-hash".to_string(),
                baseline_hash: promotion_hashes(&ready_promotion(&f)).1,
            }),
            ..promoted_input(&f)
        });
        assert!(has(
            &wrong,
            ProductionRuntimeRefusal::MissingModelArtifactHash
        ));
    }

    #[test]
    fn missing_each_common_requirement_is_refused() {
        let f = failures();
        let cfg = package_production_runtime(&ProductionRuntimeInput {
            runtime_config: None,
            ..promoted_input(&f)
        });
        assert!(has(&cfg, ProductionRuntimeRefusal::MissingRuntimeConfig));

        let ver = package_production_runtime(&ProductionRuntimeInput {
            version: None,
            ..promoted_input(&f)
        });
        assert!(has(&ver, ProductionRuntimeRefusal::MissingVersionReceipt));

        let rb = package_production_runtime(&ProductionRuntimeInput {
            rollback: None,
            ..promoted_input(&f)
        });
        assert!(has(&rb, ProductionRuntimeRefusal::MissingRollbackArtifact));

        let runbook = package_production_runtime(&ProductionRuntimeInput {
            operator_runbook: None,
            ..promoted_input(&f)
        });
        assert!(has(
            &runbook,
            ProductionRuntimeRefusal::MissingOperatorRunbook
        ));

        let rcpt = package_production_runtime(&ProductionRuntimeInput {
            receipt_output_path: None,
            ..promoted_input(&f)
        });
        assert!(has(
            &rcpt,
            ProductionRuntimeRefusal::MissingReceiptOutputPath
        ));

        let replay = package_production_runtime(&ProductionRuntimeInput {
            replay_output_path: Some(String::new()),
            ..promoted_input(&f)
        });
        assert!(has(
            &replay,
            ProductionRuntimeRefusal::MissingReplayOutputPath
        ));
    }

    #[test]
    fn training_mode_and_network_are_refused() {
        let f = failures();
        let training = package_production_runtime(&ProductionRuntimeInput {
            runtime_config: Some(ProductionRuntimeConfig {
                training_mode_requested: true,
                ..runtime_config()
            }),
            ..promoted_input(&f)
        });
        assert!(has(
            &training,
            ProductionRuntimeRefusal::TrainingModeEnabled
        ));

        let network = package_production_runtime(&ProductionRuntimeInput {
            runtime_config: Some(ProductionRuntimeConfig {
                network_enabled: true,
                ..runtime_config()
            }),
            ..promoted_input(&f)
        });
        assert!(has(
            &network,
            ProductionRuntimeRefusal::UnauthorizedNetworkEnabled
        ));
    }

    #[test]
    fn no_training_mode_is_the_default_and_only_state() {
        let f = failures();
        let package = package_production_runtime(&promoted_input(&f));
        assert_eq!(
            package.manifest.no_training_mode,
            RuntimeNoTrainingMode::NoTraining
        );
        let receipt = package.receipt.as_ref().expect("sealed");
        assert_eq!(receipt.no_training_mode, RuntimeNoTrainingMode::NoTraining);
        assert_eq!(RuntimeNoTrainingMode::NoTraining.tag(), "no_training");
    }

    #[test]
    fn authority_drift_is_refused() {
        let f = failures();
        let drifted = package_production_runtime(&ProductionRuntimeInput {
            authority_drift: AuthorityDriftCheck::drifted(),
            ..promoted_input(&f)
        });
        assert!(has(
            &drifted,
            ProductionRuntimeRefusal::AuthorityDriftDetected
        ));
        let unchecked = package_production_runtime(&ProductionRuntimeInput {
            authority_drift: AuthorityDriftCheck::unchecked(),
            ..promoted_input(&f)
        });
        assert!(has(
            &unchecked,
            ProductionRuntimeRefusal::AuthorityDriftDetected
        ));
    }

    #[test]
    fn packaged_runtime_is_not_deployment_or_service() {
        let f = failures();
        let package = package_production_runtime(&promoted_input(&f));
        assert_eq!(package.outcome, ProductionRuntimeOutcome::Packaged);
        assert!(!package.deploys_model);
        assert!(!package.starts_production_service);
        assert!(!package.claims_production);
        assert!(!package.serves_traffic);
        assert!(!package.replaces_baseline);
        assert!(!package.trains);
        assert!(!package.mutates_weights);
        assert!(package.boundary.all_inert());
        let receipt = package.receipt.as_ref().expect("sealed");
        assert!(!receipt.deploys_model);
        assert!(!receipt.starts_production_service);
        assert!(!receipt.claims_production);
    }

    #[test]
    fn packaged_runtime_requires_s11_smoke() {
        let f = failures();
        let package = package_production_runtime(&promoted_input(&f));
        assert!(package.requires_s11_smoke);
        assert!(package.manifest.requires_s11_smoke);
        let receipt = package.receipt.as_ref().expect("sealed");
        assert!(receipt.requires_s11_smoke);
    }

    #[test]
    fn manifest_describes_the_full_verified_path() {
        let f = failures();
        let package = package_production_runtime(&promoted_input(&f));
        assert_eq!(package.manifest.verified_path, PROD_RUNTIME_VERIFIED_PATH);
        assert_eq!(package.manifest.verified_path.len(), 11);
        assert_eq!(package.manifest.verified_path[0], "curate");
        assert_eq!(package.manifest.verified_path[10], "runtime_receipt");
        assert!(package.manifest.local_offline);
    }

    #[test]
    fn p12_training_justified_remains_false_even_when_packaged() {
        let f = failures();
        let package = package_production_runtime(&promoted_input(&f));
        assert!(!package.training_justified);
        assert!(!package.opens_p12);
        // The real P12 gate is unaffected by a packaged runtime.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn mode_and_refusal_counts_match_enums() {
        assert_eq!(ProductionRuntimeMode::ALL.len(), PROD_RUNTIME_MODE_COUNT);
        assert_eq!(
            ProductionRuntimeRefusal::ALL.len(),
            PROD_RUNTIME_REFUSAL_COUNT
        );
        assert_eq!(PROD_RUNTIME_MODE_NAMES.len(), PROD_RUNTIME_MODE_COUNT);
        assert_eq!(PROD_RUNTIME_REFUSAL_NAMES.len(), PROD_RUNTIME_REFUSAL_COUNT);
        for (m, name) in ProductionRuntimeMode::ALL
            .iter()
            .zip(PROD_RUNTIME_MODE_NAMES)
        {
            assert_eq!(m.tag(), name);
        }
        for (r, name) in ProductionRuntimeRefusal::ALL
            .iter()
            .zip(PROD_RUNTIME_REFUSAL_NAMES)
        {
            assert_eq!(r.tag(), name);
        }
    }

    #[test]
    fn matrix_has_the_twenty_named_scenarios() {
        let matrix = production_runtime_matrix();
        assert_eq!(matrix.scenarios.len(), PROD_RUNTIME_SCENARIO_COUNT);
        for name in [
            "local_no_model_runtime_packaged",
            "missing_runtime_config_refused",
            "missing_promotion_report_refused",
            "promotion_not_ready_refused",
            "missing_model_artifact_hash_refused",
            "missing_baseline_hash_refused",
            "missing_rollback_artifact_refused",
            "missing_version_receipt_refused",
            "missing_operator_runbook_refused",
            "training_mode_enabled_refused",
            "unauthorized_network_enabled_refused",
            "missing_receipt_output_path_refused",
            "missing_replay_output_path_refused",
            "authority_drift_refused",
            "promoted_ready_runtime_packaged",
            "package_is_not_deployment",
            "package_is_not_service_start",
            "package_is_not_baseline_replacement",
            "package_requires_s11_smoke",
            "serialized_runtime_package_tamper_refused",
        ] {
            assert!(
                matrix.scenario(name).is_some(),
                "scenario {name} is missing"
            );
        }
        assert!(matrix.production_never_opens);
        let packaged = matrix
            .scenario("promoted_ready_runtime_packaged")
            .expect("present");
        assert_eq!(packaged.outcome, "packaged");
        assert!(packaged.sealed_receipt);
        let no_model = matrix
            .scenario("local_no_model_runtime_packaged")
            .expect("present");
        assert_eq!(no_model.mode, "local_no_model_runtime");
    }

    #[test]
    fn every_matrix_cell_keeps_production_closed() {
        let matrix = production_runtime_matrix();
        for cell in &matrix.scenarios {
            assert!(
                cell.production_still_closed,
                "cell {} opened production",
                cell.name
            );
        }
        let tamper = matrix
            .scenario("serialized_runtime_package_tamper_refused")
            .expect("tamper cell present");
        assert!(tamper
            .refusals
            .contains(&"serialized_runtime_package_tamper_refused"));
    }

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let f = failures();
        let input = promoted_input(&f);
        let canonical = package_production_runtime_json(&input);
        assert_eq!(
            canonical,
            package_production_runtime_json(&promoted_input(&f))
        );
        assert!(verify_production_runtime_package_json(&input, &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_production_runtime_package_json(&input, &tampered),
            Err(ProductionRuntimeError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_re_derives_refusing_tampering() {
        let canonical = production_runtime_matrix_json();
        assert!(verify_production_runtime_matrix_json(&canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_production_runtime_matrix_json(&tampered),
            Err(ProductionRuntimeError::ReplayMismatch)
        );
    }

    #[test]
    fn closed_by_default_refuses_with_no_inputs() {
        let package = package_production_runtime(&ProductionRuntimeInput::closed_by_default(
            ProductionRuntimeMode::LocalPromotedReadyRuntime,
        ));
        assert_eq!(package.outcome, ProductionRuntimeOutcome::Refused);
        assert!(has(
            &package,
            ProductionRuntimeRefusal::MissingRuntimeConfig
        ));
        assert!(has(
            &package,
            ProductionRuntimeRefusal::MissingPromotionReport
        ));
        assert!(has(
            &package,
            ProductionRuntimeRefusal::AuthorityDriftDetected
        ));
        assert!(package.receipt.is_none());
    }
}
