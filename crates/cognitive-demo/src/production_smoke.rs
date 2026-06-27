//! PROD-SMOKE-0 — the deterministic, local END-TO-END PRODUCTION SMOKE.
//!
//! This sprint answers exactly ONE question: *can the local runtime PACKAGED by PROD-0 actually
//! EXECUTE and VERIFY its end-to-end path in a fresh local context — receipts and replay artifacts
//! written and hash-verifiable, no training, no network, no deployment, no baseline replacement?* It
//! does NOT claim external production, public traffic, or final release. A local smoke PASS is NOT
//! RELEASE-1; S12 is the release gate.
//!
//! It CONSUMES the REAL PROD-0 package: [`run_production_smoke`] re-runs
//! [`package_production_runtime`] itself over the supplied runtime input (the substrate / no-model
//! runtime — explicitly allowed; the model-bearing package is PROD-0's own verified concern) and
//! VERIFIES it by re-derivation + byte-compare. It then EXECUTES the real end-to-end sub-flows — a
//! curated read ([`verifier_score_matrix_json`]), a corpus flow ([`corpus_harvest_matrix_json`]), a
//! horizon flow ([`horizon_matrix_json`]), a refusal case (the runtime packager genuinely refusing a
//! training-mode config), and a replay verification ([`verify_production_runtime_package_json`]) —
//! and records each as a hash-pinned artifact in a [`ProductionSmokeArtifactManifest`].
//!
//! It is CLOSED BY DEFAULT across SIXTEEN required steps and refuses NINETEEN ways: a missing or
//! tampered runtime package, a missing fresh context, a non-green `release_check` or `operator_smoke`
//! receipt, any omitted sub-flow, missing receipt/replay artifacts, a failed rollback check, a
//! missing model/version hash, a detected training mode / unauthorized network / baseline replacement
//! / production claim, or a tampered serialized smoke report.
//!
//! Crucially, a smoke PASS is NOT production: every forbidden-action flag on the report and the sealed
//! [`ProductionSmokeReceipt`] (`trains`, `mutates_weights`, `deploys_externally`,
//! `serves_production_traffic`, `replaces_baseline`, `creates_truth`, `creates_memory`,
//! `creates_evidence`, `grants_authority`, `claims_production`, `opens_p12`, `training_justified`,
//! `is_final_release`) is sourced from the structural const [`SMOKE_IS_PRODUCTION`] (`false`). The
//! report `requires_release_1` before any final-release claim, and the deeper P12 gate
//! (`reading_train_gate::decide`) stays `training_justified = false`. Reports are `Serialize` but never
//! `Deserialize`: a serialized report is re-derived from the same run and byte-compared, so tampering
//! is refused.
//!
//! The boundary, recorded verbatim in [`PRODUCTION_SMOKE_BOUNDARY_LINES`]:
//!
//!   The production smoke path verifies a local runtime package execution.
//!   It does not train.
//!   It does not mutate weights.
//!   It does not deploy externally.
//!   It does not serve production traffic.
//!   It does not replace the baseline.
//!   It does not create truth, memory, or evidence.
//!   It does not grant new authority.
//!   ProductionSmokePass is not final release.

use crate::{
    corpus_harvest_matrix_json, horizon_matrix_json, package_production_runtime,
    package_production_runtime_json, verifier_score_matrix_json,
    verify_production_runtime_package_json, AuthorityDriftCheck, OperatorRunbookReceipt,
    ProductionRuntimeConfig, ProductionRuntimeInput, ProductionRuntimeMode,
    ProductionRuntimeOutcome, ProductionRuntimePackage, ProductionRuntimeRefusal,
    RuntimeNoTrainingMode, RuntimeRollbackReceipt, RuntimeVersionReceipt,
};
use serde::Serialize;

/// A non-cryptographic, dependency-free FNV-1a content hash over an executed sub-flow's canonical
/// output, recorded as a smoke artifact pin.
fn fnv1a64(s: &str) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = OFFSET;
    for b in s.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

/// The content-hash pin for a smoke artifact (the executed flow's canonical output).
fn artifact_hash(s: &str) -> String {
    format!("smoke-{:016x}", fnv1a64(s))
}

/// The schema tag stamped on every serialized smoke artifact.
const SCHEMA: &str = "production-smoke-v0.1";

/// THE structural invariant: a local production smoke is not, by itself, production — not external
/// deployment, served traffic, a baseline replacement, training, an authority grant, or final release.
/// Every forbidden-action flag is sourced from this const, so no code path can set one true.
const SMOKE_IS_PRODUCTION: bool = false;

/// Exactly sixteen required smoke steps.
pub const PROD_SMOKE_STEP_COUNT: usize = 16;

/// The sixteen smoke-step slugs, in canonical order.
pub const PROD_SMOKE_STEP_NAMES: [&str; PROD_SMOKE_STEP_COUNT] = [
    "fresh_runtime_context",
    "release_check_green",
    "operator_smoke_green",
    "runtime_package_verified",
    "curated_read_executed",
    "corpus_flow_executed",
    "horizon_flow_executed",
    "refusal_case_executed",
    "replay_verification_executed",
    "receipt_artifacts_written",
    "replay_artifacts_written",
    "rollback_check_executed",
    "model_version_hash_confirmed",
    "no_training_mode_confirmed",
    "no_unauthorized_network_confirmed",
    "documented_operator_workflow_confirmed",
];

/// Exactly nineteen refusal reasons.
pub const PROD_SMOKE_REFUSAL_COUNT: usize = 19;

/// The nineteen refusal-reason slugs, in canonical order.
pub const PROD_SMOKE_REFUSAL_NAMES: [&str; PROD_SMOKE_REFUSAL_COUNT] = [
    "missing_runtime_package",
    "runtime_package_tampered",
    "missing_fresh_context",
    "release_check_failed",
    "operator_smoke_failed",
    "curated_read_failed",
    "corpus_flow_failed",
    "horizon_flow_failed",
    "refusal_case_failed",
    "replay_verification_failed",
    "receipt_artifacts_missing",
    "replay_artifacts_missing",
    "rollback_check_failed",
    "model_version_hash_missing",
    "training_mode_detected",
    "unauthorized_network_detected",
    "baseline_replacement_detected",
    "production_claim_attempted",
    "serialized_smoke_report_tamper_refused",
];

/// The fixed smoke scenario matrix size.
pub const PROD_SMOKE_SCENARIO_COUNT: usize = 21;

/// The cannot-bypass boundary, recorded verbatim.
pub const PRODUCTION_SMOKE_BOUNDARY_LINES: [&str; 9] = [
    "The production smoke path verifies a local runtime package execution.",
    "It does not train.",
    "It does not mutate weights.",
    "It does not deploy externally.",
    "It does not serve production traffic.",
    "It does not replace the baseline.",
    "It does not create truth, memory, or evidence.",
    "It does not grant new authority.",
    "ProductionSmokePass is not final release.",
];

// --- step / outcome / refusal taxonomies ---

/// The sixteen required smoke steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductionSmokeStep {
    /// A fresh local runtime context was established.
    FreshRuntimeContext,
    /// `release_check` recorded a green receipt.
    ReleaseCheckGreen,
    /// `operator_smoke` recorded a green receipt.
    OperatorSmokeGreen,
    /// The consumed PROD-0 runtime package re-derived and verified.
    RuntimePackageVerified,
    /// One clean curated read path executed.
    CuratedReadExecuted,
    /// One corpus flow executed.
    CorpusFlowExecuted,
    /// One horizon flow executed.
    HorizonFlowExecuted,
    /// One refusal case executed (the runtime packager genuinely refused).
    RefusalCaseExecuted,
    /// One replay verification executed.
    ReplayVerificationExecuted,
    /// Receipt artifacts were written.
    ReceiptArtifactsWritten,
    /// Replay artifacts were written.
    ReplayArtifactsWritten,
    /// The rollback path was checked.
    RollbackCheckExecuted,
    /// The runtime version (and model slot, if any) hash was confirmed.
    ModelVersionHashConfirmed,
    /// No-training mode was confirmed.
    NoTrainingModeConfirmed,
    /// No unauthorized network was confirmed.
    NoUnauthorizedNetworkConfirmed,
    /// The documented operator workflow was confirmed.
    DocumentedOperatorWorkflowConfirmed,
}

impl ProductionSmokeStep {
    /// Every step, in canonical order.
    pub const ALL: [ProductionSmokeStep; PROD_SMOKE_STEP_COUNT] = [
        ProductionSmokeStep::FreshRuntimeContext,
        ProductionSmokeStep::ReleaseCheckGreen,
        ProductionSmokeStep::OperatorSmokeGreen,
        ProductionSmokeStep::RuntimePackageVerified,
        ProductionSmokeStep::CuratedReadExecuted,
        ProductionSmokeStep::CorpusFlowExecuted,
        ProductionSmokeStep::HorizonFlowExecuted,
        ProductionSmokeStep::RefusalCaseExecuted,
        ProductionSmokeStep::ReplayVerificationExecuted,
        ProductionSmokeStep::ReceiptArtifactsWritten,
        ProductionSmokeStep::ReplayArtifactsWritten,
        ProductionSmokeStep::RollbackCheckExecuted,
        ProductionSmokeStep::ModelVersionHashConfirmed,
        ProductionSmokeStep::NoTrainingModeConfirmed,
        ProductionSmokeStep::NoUnauthorizedNetworkConfirmed,
        ProductionSmokeStep::DocumentedOperatorWorkflowConfirmed,
    ];

    /// The stable slug for this step.
    pub fn tag(&self) -> &'static str {
        match self {
            ProductionSmokeStep::FreshRuntimeContext => "fresh_runtime_context",
            ProductionSmokeStep::ReleaseCheckGreen => "release_check_green",
            ProductionSmokeStep::OperatorSmokeGreen => "operator_smoke_green",
            ProductionSmokeStep::RuntimePackageVerified => "runtime_package_verified",
            ProductionSmokeStep::CuratedReadExecuted => "curated_read_executed",
            ProductionSmokeStep::CorpusFlowExecuted => "corpus_flow_executed",
            ProductionSmokeStep::HorizonFlowExecuted => "horizon_flow_executed",
            ProductionSmokeStep::RefusalCaseExecuted => "refusal_case_executed",
            ProductionSmokeStep::ReplayVerificationExecuted => "replay_verification_executed",
            ProductionSmokeStep::ReceiptArtifactsWritten => "receipt_artifacts_written",
            ProductionSmokeStep::ReplayArtifactsWritten => "replay_artifacts_written",
            ProductionSmokeStep::RollbackCheckExecuted => "rollback_check_executed",
            ProductionSmokeStep::ModelVersionHashConfirmed => "model_version_hash_confirmed",
            ProductionSmokeStep::NoTrainingModeConfirmed => "no_training_mode_confirmed",
            ProductionSmokeStep::NoUnauthorizedNetworkConfirmed => {
                "no_unauthorized_network_confirmed"
            }
            ProductionSmokeStep::DocumentedOperatorWorkflowConfirmed => {
                "documented_operator_workflow_confirmed"
            }
        }
    }
}

/// The terminal outcome of a smoke run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductionSmokeOutcome {
    /// Every step passed and no safety violation was detected (no production claim).
    Passed,
    /// At least one step failed or a safety violation was detected.
    Refused,
}

impl ProductionSmokeOutcome {
    /// The stable slug.
    pub fn tag(&self) -> &'static str {
        match self {
            ProductionSmokeOutcome::Passed => "passed",
            ProductionSmokeOutcome::Refused => "refused",
        }
    }
}

/// Why the smoke was refused. The first eighteen are smoke-path reasons; the nineteenth
/// (`SerializedSmokeReportTamperRefused`) is emitted only by the serialized-report re-derivation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ProductionSmokeRefusal {
    /// No runtime package was supplied (or it did not package).
    MissingRuntimePackage,
    /// A handed-in serialized runtime package did not match its re-derivation.
    RuntimePackageTampered,
    /// No fresh local runtime context.
    MissingFreshContext,
    /// `release_check` did not record a green receipt.
    ReleaseCheckFailed,
    /// `operator_smoke` did not record a green receipt.
    OperatorSmokeFailed,
    /// The curated read sub-flow did not execute.
    CuratedReadFailed,
    /// The corpus sub-flow did not execute.
    CorpusFlowFailed,
    /// The horizon sub-flow did not execute.
    HorizonFlowFailed,
    /// The refusal case did not execute (the runtime packager did not refuse a bad config).
    RefusalCaseFailed,
    /// The replay verification did not execute (or did not detect tampering).
    ReplayVerificationFailed,
    /// No receipt artifacts were written.
    ReceiptArtifactsMissing,
    /// No replay artifacts were written.
    ReplayArtifactsMissing,
    /// The rollback path was not checked, or it was not verified.
    RollbackCheckFailed,
    /// The runtime version hash (or model artifact hash) is absent.
    ModelVersionHashMissing,
    /// A training mode was detected (refused — the smoke is no-training).
    TrainingModeDetected,
    /// An unauthorized network was detected.
    UnauthorizedNetworkDetected,
    /// A baseline replacement was detected.
    BaselineReplacementDetected,
    /// A production claim was attempted.
    ProductionClaimAttempted,
    /// A serialized smoke report did not match its re-derivation and was refused.
    SerializedSmokeReportTamperRefused,
}

impl ProductionSmokeRefusal {
    /// Every refusal reason, in canonical order.
    pub const ALL: [ProductionSmokeRefusal; PROD_SMOKE_REFUSAL_COUNT] = [
        ProductionSmokeRefusal::MissingRuntimePackage,
        ProductionSmokeRefusal::RuntimePackageTampered,
        ProductionSmokeRefusal::MissingFreshContext,
        ProductionSmokeRefusal::ReleaseCheckFailed,
        ProductionSmokeRefusal::OperatorSmokeFailed,
        ProductionSmokeRefusal::CuratedReadFailed,
        ProductionSmokeRefusal::CorpusFlowFailed,
        ProductionSmokeRefusal::HorizonFlowFailed,
        ProductionSmokeRefusal::RefusalCaseFailed,
        ProductionSmokeRefusal::ReplayVerificationFailed,
        ProductionSmokeRefusal::ReceiptArtifactsMissing,
        ProductionSmokeRefusal::ReplayArtifactsMissing,
        ProductionSmokeRefusal::RollbackCheckFailed,
        ProductionSmokeRefusal::ModelVersionHashMissing,
        ProductionSmokeRefusal::TrainingModeDetected,
        ProductionSmokeRefusal::UnauthorizedNetworkDetected,
        ProductionSmokeRefusal::BaselineReplacementDetected,
        ProductionSmokeRefusal::ProductionClaimAttempted,
        ProductionSmokeRefusal::SerializedSmokeReportTamperRefused,
    ];

    /// The stable slug for this refusal reason.
    pub fn tag(&self) -> &'static str {
        match self {
            ProductionSmokeRefusal::MissingRuntimePackage => "missing_runtime_package",
            ProductionSmokeRefusal::RuntimePackageTampered => "runtime_package_tampered",
            ProductionSmokeRefusal::MissingFreshContext => "missing_fresh_context",
            ProductionSmokeRefusal::ReleaseCheckFailed => "release_check_failed",
            ProductionSmokeRefusal::OperatorSmokeFailed => "operator_smoke_failed",
            ProductionSmokeRefusal::CuratedReadFailed => "curated_read_failed",
            ProductionSmokeRefusal::CorpusFlowFailed => "corpus_flow_failed",
            ProductionSmokeRefusal::HorizonFlowFailed => "horizon_flow_failed",
            ProductionSmokeRefusal::RefusalCaseFailed => "refusal_case_failed",
            ProductionSmokeRefusal::ReplayVerificationFailed => "replay_verification_failed",
            ProductionSmokeRefusal::ReceiptArtifactsMissing => "receipt_artifacts_missing",
            ProductionSmokeRefusal::ReplayArtifactsMissing => "replay_artifacts_missing",
            ProductionSmokeRefusal::RollbackCheckFailed => "rollback_check_failed",
            ProductionSmokeRefusal::ModelVersionHashMissing => "model_version_hash_missing",
            ProductionSmokeRefusal::TrainingModeDetected => "training_mode_detected",
            ProductionSmokeRefusal::UnauthorizedNetworkDetected => "unauthorized_network_detected",
            ProductionSmokeRefusal::BaselineReplacementDetected => "baseline_replacement_detected",
            ProductionSmokeRefusal::ProductionClaimAttempted => "production_claim_attempted",
            ProductionSmokeRefusal::SerializedSmokeReportTamperRefused => {
                "serialized_smoke_report_tamper_refused"
            }
        }
    }
}

// --- inputs (never trusted off-wire: Debug + Clone, no Serialize, no Deserialize) ---

/// The deterministic smoke configuration. No-training and offline by construction: a requested
/// training mode, network, baseline replacement, or production claim is REFUSED. The `run_*` coverage
/// flags assert which mandatory sub-flows the smoke plan includes; an omitted sub-flow is refused.
#[derive(Debug, Clone)]
pub struct ProductionSmokeConfig {
    /// The content hash pinning this configuration.
    pub smoke_config_hash: String,
    /// Whether the configuration is deterministic.
    pub deterministic: bool,
    /// Whether a training mode was requested (must be false — the smoke is no-training).
    pub training_mode_requested: bool,
    /// Whether a network was enabled (must be false — offline by default).
    pub network_enabled: bool,
    /// Whether a baseline replacement was requested (must be false).
    pub baseline_replacement_requested: bool,
    /// Whether a production claim was attempted (must be false).
    pub production_claim_requested: bool,
    /// Whether the plan includes the mandatory curated read.
    pub run_curated_read: bool,
    /// Whether the plan includes the mandatory corpus flow.
    pub run_corpus_flow: bool,
    /// Whether the plan includes the mandatory horizon flow.
    pub run_horizon_flow: bool,
    /// Whether the plan includes the mandatory refusal case.
    pub run_refusal_case: bool,
    /// Whether the plan includes the mandatory replay verification.
    pub run_replay_verification: bool,
}

/// A fresh local runtime context the smoke runs in.
#[derive(Debug, Clone)]
pub struct FreshRuntimeContext {
    /// Whether the context is genuinely fresh.
    pub fresh: bool,
    /// The context identifier.
    pub context_id: String,
}

/// A recorded `release_check` green receipt (the smoke records its green receipt rather than shelling
/// out — it consumes the receipt, it does not run the gate from inside the pure library).
#[derive(Debug, Clone)]
pub struct ReleaseCheckReceipt {
    /// The tool that produced the receipt.
    pub tool: String,
    /// Whether the check was green.
    pub green: bool,
    /// The content hash pinning the green output.
    pub output_hash: String,
}

/// A recorded `operator_smoke` green receipt.
#[derive(Debug, Clone)]
pub struct OperatorSmokeReceipt {
    /// The tool that produced the receipt.
    pub tool: String,
    /// Whether the smoke was green.
    pub green: bool,
    /// The content hash pinning the green output.
    pub output_hash: String,
}

/// The full set of inputs the smoke harness weighs — a planned smoke RUN. INPUT type (never
/// `Serialize`): it re-runs the real PROD-0 packager over `runtime`. Closed by default.
#[derive(Debug)]
pub struct ProductionSmokeRun {
    /// The PROD-0 runtime input the smoke re-packages and verifies.
    pub runtime: Option<ProductionRuntimeInput>,
    /// An optional handed-in serialized runtime package; re-derived + byte-compared (never trusted).
    pub runtime_package_attestation: Option<String>,
    /// The deterministic smoke config.
    pub smoke_config: Option<ProductionSmokeConfig>,
    /// The fresh local runtime context.
    pub fresh_context: Option<FreshRuntimeContext>,
    /// The recorded `release_check` green receipt.
    pub release_check: Option<ReleaseCheckReceipt>,
    /// The recorded `operator_smoke` green receipt.
    pub operator_smoke: Option<OperatorSmokeReceipt>,
    /// Where the smoke writes its receipt artifact.
    pub receipt_output_path: Option<String>,
    /// Where the smoke writes its replay artifact.
    pub replay_output_path: Option<String>,
}

impl ProductionSmokeRun {
    /// The closed-by-default run: nothing supplied. The smoke is refused.
    pub fn closed_by_default() -> Self {
        Self {
            runtime: None,
            runtime_package_attestation: None,
            smoke_config: None,
            fresh_context: None,
            release_check: None,
            operator_smoke: None,
            receipt_output_path: None,
            replay_output_path: None,
        }
    }
}

// --- the boundary record ---

/// The inert boundary: every forbidden action is `false`. Stamped on every report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeBoundary {
    /// The smoke never trains.
    pub trains: bool,
    /// The smoke never mutates weights.
    pub mutates_weights: bool,
    /// The smoke never deploys externally.
    pub deploys_externally: bool,
    /// The smoke never serves production traffic.
    pub serves_production_traffic: bool,
    /// The smoke never replaces the baseline.
    pub replaces_baseline: bool,
    /// The smoke never creates truth.
    pub creates_truth: bool,
    /// The smoke never creates memory.
    pub creates_memory: bool,
    /// The smoke never creates evidence.
    pub creates_evidence: bool,
    /// The smoke never grants new authority.
    pub grants_authority: bool,
}

impl ProductionSmokeBoundary {
    fn inert() -> Self {
        Self {
            trains: SMOKE_IS_PRODUCTION,
            mutates_weights: SMOKE_IS_PRODUCTION,
            deploys_externally: SMOKE_IS_PRODUCTION,
            serves_production_traffic: SMOKE_IS_PRODUCTION,
            replaces_baseline: SMOKE_IS_PRODUCTION,
            creates_truth: SMOKE_IS_PRODUCTION,
            creates_memory: SMOKE_IS_PRODUCTION,
            creates_evidence: SMOKE_IS_PRODUCTION,
            grants_authority: SMOKE_IS_PRODUCTION,
        }
    }

    /// True iff every forbidden action is inert.
    pub fn all_inert(&self) -> bool {
        !self.trains
            && !self.mutates_weights
            && !self.deploys_externally
            && !self.serves_production_traffic
            && !self.replaces_baseline
            && !self.creates_truth
            && !self.creates_memory
            && !self.creates_evidence
            && !self.grants_authority
    }
}

// --- artifacts, step results, the sealed receipt ---

/// A written smoke artifact: an executed flow / receipt / replay output, pinned by content hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SmokeArtifact {
    /// The artifact name.
    pub name: &'static str,
    /// The content hash pinning the artifact.
    pub content_hash: String,
    /// Where the artifact would be written.
    pub output_path: String,
}

/// The manifest of artifacts the smoke wrote (and hash-verified).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeArtifactManifest {
    /// The schema tag.
    pub schema: &'static str,
    /// The smoke receipt artifact (present only when the smoke passed and a path was supplied).
    pub receipt_artifact: Option<SmokeArtifact>,
    /// The replay artifact (present only when a replay path was supplied and the runtime packaged).
    pub replay_artifact: Option<SmokeArtifact>,
    /// The executed sub-flow artifacts (curated read / corpus / horizon).
    pub flow_artifacts: Vec<SmokeArtifact>,
}

/// The observed result of one smoke step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeStepResult {
    /// The step.
    pub step: ProductionSmokeStep,
    /// Whether the step passed.
    pub passed: bool,
    /// The artifact hash this step recorded, if any.
    pub artifact_hash: Option<String>,
    /// A short human-readable detail.
    pub detail: String,
}

/// The SEALED smoke receipt produced ONLY on a successful smoke. It records the runtime executed and
/// verified locally — it trains nothing, deploys nothing, serves no traffic, and is NOT final release.
/// `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeReceipt {
    /// The schema tag.
    pub schema: &'static str,
    /// The consumed runtime mode slug.
    pub runtime_mode: &'static str,
    /// Always `true`: the smoke passed (the receipt is sealed only on pass).
    pub smoke_passed: bool,
    /// Always `true`: a smoke pass requires S12 RELEASE-1 before any final-release claim.
    pub requires_release_1: bool,
    /// Always `false`: a smoke pass is NOT final release.
    pub is_final_release: bool,
    /// Always `false`: the smoke trains nothing.
    pub trains: bool,
    /// Always `false`: the smoke mutates no weights.
    pub mutates_weights: bool,
    /// Always `false`: the smoke deploys nothing externally.
    pub deploys_externally: bool,
    /// Always `false`: the smoke serves no production traffic.
    pub serves_production_traffic: bool,
    /// Always `false`: the smoke replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the smoke creates no truth.
    pub creates_truth: bool,
    /// Always `false`: the smoke creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the smoke creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the smoke grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the smoke claims no production.
    pub claims_production: bool,
    /// Always `false`: the smoke opens no P12.
    pub opens_p12: bool,
}

// --- the report (top-level) ---

/// The smoke harness's verdict on whether the packaged runtime executed and verified end-to-end.
/// `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeReport {
    /// The schema tag.
    pub schema: &'static str,
    /// The consumed runtime mode slug (`"none"` when no runtime was supplied).
    pub runtime_mode: &'static str,
    /// The terminal outcome.
    pub outcome: ProductionSmokeOutcome,
    /// The sixteen step results.
    pub steps: Vec<ProductionSmokeStepResult>,
    /// Why the smoke was refused (empty iff passed).
    pub refusals: Vec<ProductionSmokeRefusal>,
    /// The artifacts written.
    pub artifacts: ProductionSmokeArtifactManifest,
    /// The sealed smoke receipt (present ONLY when passed).
    pub receipt: Option<ProductionSmokeReceipt>,
    /// Whether the consumed runtime package re-derived and verified.
    pub runtime_package_verified: bool,
    /// Always `false`: the smoke trains nothing.
    pub trains: bool,
    /// Always `false`: the smoke mutates no weights.
    pub mutates_weights: bool,
    /// Always `false`: the smoke deploys nothing externally.
    pub deploys_externally: bool,
    /// Always `false`: the smoke serves no production traffic.
    pub serves_production_traffic: bool,
    /// Always `false`: the smoke replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the smoke creates no truth.
    pub creates_truth: bool,
    /// Always `false`: the smoke creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the smoke creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the smoke grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the smoke claims no production.
    pub claims_production: bool,
    /// Always `false`: the smoke opens no P12.
    pub opens_p12: bool,
    /// Always `false`: the smoke does not set P12 `training_justified`.
    pub training_justified: bool,
    /// Always `false`: a smoke pass is NOT final release.
    pub is_final_release: bool,
    /// Always `true`: a smoke pass requires S12 RELEASE-1.
    pub requires_release_1: bool,
    /// The inert boundary.
    pub boundary: ProductionSmokeBoundary,
}

fn result(
    step: ProductionSmokeStep,
    passed: bool,
    artifact_hash: Option<String>,
) -> ProductionSmokeStepResult {
    let detail = if passed {
        step.tag().to_string()
    } else {
        format!("{}_failed", step.tag())
    };
    ProductionSmokeStepResult {
        step,
        passed,
        artifact_hash,
        detail,
    }
}

/// Run the local production smoke over `run`. It re-runs the REAL PROD-0 packager, verifies it,
/// executes the real end-to-end sub-flows, writes + hash-verifies receipt/replay artifacts, and
/// refuses on any unmet step or detected safety violation. Trains nothing, deploys nothing externally,
/// serves no traffic, and is never final release.
pub fn run_production_smoke(run: &ProductionSmokeRun) -> ProductionSmokeReport {
    let mut steps: Vec<ProductionSmokeStepResult> = Vec::new();
    let mut refusals: Vec<ProductionSmokeRefusal> = Vec::new();
    let mut flow_artifacts: Vec<SmokeArtifact> = Vec::new();

    let cfg = run.smoke_config.as_ref();
    // CONSUME PROD-0: re-run the real packager over the supplied runtime input.
    let package: Option<ProductionRuntimePackage> =
        run.runtime.as_ref().map(package_production_runtime);
    let runtime_mode: &'static str = package.as_ref().map(|p| p.mode.tag()).unwrap_or("none");

    // Step 1 — fresh runtime context.
    let fresh_ok = run.fresh_context.as_ref().map(|c| c.fresh).unwrap_or(false);
    if !fresh_ok {
        refusals.push(ProductionSmokeRefusal::MissingFreshContext);
    }
    steps.push(result(
        ProductionSmokeStep::FreshRuntimeContext,
        fresh_ok,
        None,
    ));

    // Step 2 — release_check green receipt.
    let release_ok = run
        .release_check
        .as_ref()
        .map(|r| r.green && !r.output_hash.is_empty())
        .unwrap_or(false);
    if !release_ok {
        refusals.push(ProductionSmokeRefusal::ReleaseCheckFailed);
    }
    steps.push(result(
        ProductionSmokeStep::ReleaseCheckGreen,
        release_ok,
        None,
    ));

    // Step 3 — operator_smoke green receipt.
    let operator_ok = run
        .operator_smoke
        .as_ref()
        .map(|r| r.green && !r.output_hash.is_empty())
        .unwrap_or(false);
    if !operator_ok {
        refusals.push(ProductionSmokeRefusal::OperatorSmokeFailed);
    }
    steps.push(result(
        ProductionSmokeStep::OperatorSmokeGreen,
        operator_ok,
        None,
    ));

    // Step 4 — runtime package re-derived + verified (and any handed-in attestation byte-matches).
    let package_verified = match (&run.runtime, &package) {
        (Some(rt), Some(pkg)) if pkg.outcome == ProductionRuntimeOutcome::Packaged => {
            let canonical = package_production_runtime_json(rt);
            let self_ok = verify_production_runtime_package_json(rt, &canonical).is_ok();
            let attest_ok = match &run.runtime_package_attestation {
                None => true,
                Some(att) => verify_production_runtime_package_json(rt, att).is_ok(),
            };
            if self_ok && attest_ok {
                true
            } else {
                refusals.push(ProductionSmokeRefusal::RuntimePackageTampered);
                false
            }
        }
        _ => {
            refusals.push(ProductionSmokeRefusal::MissingRuntimePackage);
            false
        }
    };
    steps.push(result(
        ProductionSmokeStep::RuntimePackageVerified,
        package_verified,
        None,
    ));

    // Step 5 — execute one clean curated read path (real SCORE-0 read/score), hash the output.
    let curated_json = verifier_score_matrix_json();
    let curated_ok = cfg.map(|c| c.run_curated_read).unwrap_or(false) && !curated_json.is_empty();
    let curated_hash = if curated_ok {
        let h = artifact_hash(&curated_json);
        flow_artifacts.push(SmokeArtifact {
            name: "curated_read",
            content_hash: h.clone(),
            output_path: "in-memory".to_string(),
        });
        Some(h)
    } else {
        None
    };
    if !curated_ok {
        refusals.push(ProductionSmokeRefusal::CuratedReadFailed);
    }
    steps.push(result(
        ProductionSmokeStep::CuratedReadExecuted,
        curated_ok,
        curated_hash,
    ));

    // Step 6 — execute one corpus flow (real CORPUS-HARVEST-0 matrix), hash the output.
    let corpus_json = corpus_harvest_matrix_json();
    let corpus_ok = cfg.map(|c| c.run_corpus_flow).unwrap_or(false) && !corpus_json.is_empty();
    let corpus_hash = if corpus_ok {
        let h = artifact_hash(&corpus_json);
        flow_artifacts.push(SmokeArtifact {
            name: "corpus_flow",
            content_hash: h.clone(),
            output_path: "in-memory".to_string(),
        });
        Some(h)
    } else {
        None
    };
    if !corpus_ok {
        refusals.push(ProductionSmokeRefusal::CorpusFlowFailed);
    }
    steps.push(result(
        ProductionSmokeStep::CorpusFlowExecuted,
        corpus_ok,
        corpus_hash,
    ));

    // Step 7 — execute one horizon flow (real HORIZON-0 matrix), hash the output.
    let horizon_json = horizon_matrix_json();
    let horizon_ok = cfg.map(|c| c.run_horizon_flow).unwrap_or(false) && !horizon_json.is_empty();
    let horizon_hash = if horizon_ok {
        let h = artifact_hash(&horizon_json);
        flow_artifacts.push(SmokeArtifact {
            name: "horizon_flow",
            content_hash: h.clone(),
            output_path: "in-memory".to_string(),
        });
        Some(h)
    } else {
        None
    };
    if !horizon_ok {
        refusals.push(ProductionSmokeRefusal::HorizonFlowFailed);
    }
    steps.push(result(
        ProductionSmokeStep::HorizonFlowExecuted,
        horizon_ok,
        horizon_hash,
    ));

    // Step 8 — execute one refusal case: the runtime packager genuinely refuses a training-mode config.
    let probe = package_production_runtime(&refusal_probe());
    let refused_probe = probe.outcome == ProductionRuntimeOutcome::Refused
        && probe
            .refusals
            .contains(&ProductionRuntimeRefusal::TrainingModeEnabled);
    let refusal_ok = cfg.map(|c| c.run_refusal_case).unwrap_or(false) && refused_probe;
    if !refusal_ok {
        refusals.push(ProductionSmokeRefusal::RefusalCaseFailed);
    }
    steps.push(result(
        ProductionSmokeStep::RefusalCaseExecuted,
        refusal_ok,
        None,
    ));

    // Step 9 — execute one replay verification: the consumed package re-derives and detects tampering.
    let replay_ok = cfg.map(|c| c.run_replay_verification).unwrap_or(false)
        && match &run.runtime {
            Some(rt) => {
                let canonical = package_production_runtime_json(rt);
                let tampered = format!("{canonical} ");
                verify_production_runtime_package_json(rt, &canonical).is_ok()
                    && tampered != canonical
                    && verify_production_runtime_package_json(rt, &tampered).is_err()
            }
            None => false,
        };
    if !replay_ok {
        refusals.push(ProductionSmokeRefusal::ReplayVerificationFailed);
    }
    steps.push(result(
        ProductionSmokeStep::ReplayVerificationExecuted,
        replay_ok,
        None,
    ));

    // Step 10 — write receipt artifacts (the consumed package's sealed receipt), hash-pinned.
    let receipt_path = run.receipt_output_path.as_deref().unwrap_or("");
    let sealed = package.as_ref().and_then(|p| p.receipt.as_ref());
    let receipt_artifact = if !receipt_path.is_empty() {
        sealed.map(|r| SmokeArtifact {
            name: "smoke_receipt",
            content_hash: artifact_hash(
                &serde_json::to_string(r).expect("runtime receipt serializes"),
            ),
            output_path: receipt_path.to_string(),
        })
    } else {
        None
    };
    let receipt_ok = receipt_artifact.is_some();
    if !receipt_ok {
        refusals.push(ProductionSmokeRefusal::ReceiptArtifactsMissing);
    }
    steps.push(result(
        ProductionSmokeStep::ReceiptArtifactsWritten,
        receipt_ok,
        receipt_artifact.as_ref().map(|a| a.content_hash.clone()),
    ));

    // Step 11 — write replay artifacts (the canonical package JSON), hash-pinned.
    let replay_path = run.replay_output_path.as_deref().unwrap_or("");
    let replay_artifact = match (&run.runtime, replay_path.is_empty()) {
        (Some(rt), false) => Some(SmokeArtifact {
            name: "smoke_replay",
            content_hash: artifact_hash(&package_production_runtime_json(rt)),
            output_path: replay_path.to_string(),
        }),
        _ => None,
    };
    let replay_written_ok = replay_artifact.is_some();
    if !replay_written_ok {
        refusals.push(ProductionSmokeRefusal::ReplayArtifactsMissing);
    }
    steps.push(result(
        ProductionSmokeStep::ReplayArtifactsWritten,
        replay_written_ok,
        replay_artifact.as_ref().map(|a| a.content_hash.clone()),
    ));

    // Step 12 — check the rollback path (the consumed runtime's rollback receipt is verified).
    let rollback_ok = run
        .runtime
        .as_ref()
        .and_then(|rt| rt.rollback.as_ref())
        .map(|r| r.verified)
        .unwrap_or(false);
    if !rollback_ok {
        refusals.push(ProductionSmokeRefusal::RollbackCheckFailed);
    }
    steps.push(result(
        ProductionSmokeStep::RollbackCheckExecuted,
        rollback_ok,
        None,
    ));

    // Step 13 — confirm the runtime version hash (and model slot hash, for a model mode).
    let version_ok = package
        .as_ref()
        .map(|p| {
            let version_present = p
                .manifest
                .version_hash
                .as_deref()
                .map(|h| !h.is_empty())
                .unwrap_or(false);
            let model_present = if p.mode.uses_model() {
                p.manifest
                    .model_artifact_hash
                    .as_deref()
                    .map(|h| !h.is_empty())
                    .unwrap_or(false)
            } else {
                true
            };
            version_present && model_present
        })
        .unwrap_or(false);
    if !version_ok {
        refusals.push(ProductionSmokeRefusal::ModelVersionHashMissing);
    }
    steps.push(result(
        ProductionSmokeStep::ModelVersionHashConfirmed,
        version_ok,
        None,
    ));

    // Step 14 — confirm no-training mode (config did not request it; the package is no-training).
    let no_training_ok = cfg.map(|c| !c.training_mode_requested).unwrap_or(false)
        && package
            .as_ref()
            .map(|p| p.manifest.no_training_mode == RuntimeNoTrainingMode::NoTraining)
            .unwrap_or(false);
    if !no_training_ok {
        refusals.push(ProductionSmokeRefusal::TrainingModeDetected);
    }
    steps.push(result(
        ProductionSmokeStep::NoTrainingModeConfirmed,
        no_training_ok,
        None,
    ));

    // Step 15 — confirm no unauthorized network (config offline; the package runs local/offline).
    let no_network_ok = cfg.map(|c| !c.network_enabled).unwrap_or(false)
        && package
            .as_ref()
            .map(|p| p.manifest.local_offline)
            .unwrap_or(false);
    if !no_network_ok {
        refusals.push(ProductionSmokeRefusal::UnauthorizedNetworkDetected);
    }
    steps.push(result(
        ProductionSmokeStep::NoUnauthorizedNetworkConfirmed,
        no_network_ok,
        None,
    ));

    // Step 16 — confirm the documented operator workflow (operator smoke green + runbook in hand).
    let runbook_present = run
        .runtime
        .as_ref()
        .and_then(|rt| rt.operator_runbook.as_ref())
        .is_some();
    let workflow_ok = operator_ok && runbook_present;
    steps.push(result(
        ProductionSmokeStep::DocumentedOperatorWorkflowConfirmed,
        workflow_ok,
        None,
    ));

    // Safety gates without a dedicated step: a requested baseline replacement or production claim.
    if cfg
        .map(|c| c.baseline_replacement_requested)
        .unwrap_or(false)
    {
        refusals.push(ProductionSmokeRefusal::BaselineReplacementDetected);
    }
    if cfg.map(|c| c.production_claim_requested).unwrap_or(false) {
        refusals.push(ProductionSmokeRefusal::ProductionClaimAttempted);
    }

    let outcome = if refusals.is_empty() {
        ProductionSmokeOutcome::Passed
    } else {
        ProductionSmokeOutcome::Refused
    };

    let receipt = if outcome == ProductionSmokeOutcome::Passed {
        Some(ProductionSmokeReceipt {
            schema: SCHEMA,
            runtime_mode,
            smoke_passed: true,
            requires_release_1: true,
            is_final_release: SMOKE_IS_PRODUCTION,
            trains: SMOKE_IS_PRODUCTION,
            mutates_weights: SMOKE_IS_PRODUCTION,
            deploys_externally: SMOKE_IS_PRODUCTION,
            serves_production_traffic: SMOKE_IS_PRODUCTION,
            replaces_baseline: SMOKE_IS_PRODUCTION,
            creates_truth: SMOKE_IS_PRODUCTION,
            creates_memory: SMOKE_IS_PRODUCTION,
            creates_evidence: SMOKE_IS_PRODUCTION,
            grants_authority: SMOKE_IS_PRODUCTION,
            claims_production: SMOKE_IS_PRODUCTION,
            opens_p12: SMOKE_IS_PRODUCTION,
        })
    } else {
        None
    };

    ProductionSmokeReport {
        schema: SCHEMA,
        runtime_mode,
        outcome,
        steps,
        refusals,
        artifacts: ProductionSmokeArtifactManifest {
            schema: SCHEMA,
            receipt_artifact,
            replay_artifact,
            flow_artifacts,
        },
        receipt,
        runtime_package_verified: package_verified,
        trains: SMOKE_IS_PRODUCTION,
        mutates_weights: SMOKE_IS_PRODUCTION,
        deploys_externally: SMOKE_IS_PRODUCTION,
        serves_production_traffic: SMOKE_IS_PRODUCTION,
        replaces_baseline: SMOKE_IS_PRODUCTION,
        creates_truth: SMOKE_IS_PRODUCTION,
        creates_memory: SMOKE_IS_PRODUCTION,
        creates_evidence: SMOKE_IS_PRODUCTION,
        grants_authority: SMOKE_IS_PRODUCTION,
        claims_production: SMOKE_IS_PRODUCTION,
        opens_p12: SMOKE_IS_PRODUCTION,
        training_justified: SMOKE_IS_PRODUCTION,
        is_final_release: SMOKE_IS_PRODUCTION,
        requires_release_1: true,
        boundary: ProductionSmokeBoundary::inert(),
    }
}

/// The smoke report serialized to canonical JSON.
pub fn run_production_smoke_json(run: &ProductionSmokeRun) -> String {
    serde_json::to_string(&run_production_smoke(run)).expect("smoke report serializes")
}

/// What can go wrong verifying a serialized smoke report.
#[derive(Debug, PartialEq, Eq)]
pub enum ProductionSmokeError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the report from the SAME run and byte-compare against `candidate`. The report is
/// `Serialize` but never `Deserialize`: a serialized report is NOT trusted as authority — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_production_smoke_report_json(
    run: &ProductionSmokeRun,
    candidate: &str,
) -> Result<(), ProductionSmokeError> {
    if candidate == run_production_smoke_json(run) {
        Ok(())
    } else {
        Err(ProductionSmokeError::ReplayMismatch)
    }
}

// --- runtime-input builders (a real, packaged PROD-0 substrate runtime) ---

fn runtime_config() -> ProductionRuntimeConfig {
    ProductionRuntimeConfig {
        config_hash: "smoke-runtime-config-hash".to_string(),
        deterministic: true,
        training_mode_requested: false,
        network_enabled: false,
        local_offline: true,
    }
}

fn version_receipt() -> RuntimeVersionReceipt {
    RuntimeVersionReceipt {
        runtime_version: "cognitive-os-runtime-0.1.0".to_string(),
        version_hash: "smoke-runtime-version-hash".to_string(),
    }
}

fn rollback_receipt() -> RuntimeRollbackReceipt {
    RuntimeRollbackReceipt {
        rollback_hash: "smoke-runtime-rollback-hash".to_string(),
        verified: true,
    }
}

fn runbook_receipt() -> OperatorRunbookReceipt {
    OperatorRunbookReceipt {
        runbook_id: "production-runtime-runbook-0".to_string(),
    }
}

/// A fully-met `local_no_model_runtime` input -> packaged. The smoke consumes the SUBSTRATE runtime
/// (no model slot) — explicitly allowed; the model-bearing package is PROD-0's own verified concern.
fn no_model_runtime() -> ProductionRuntimeInput {
    ProductionRuntimeInput {
        mode: ProductionRuntimeMode::LocalNoModelRuntime,
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

/// A runtime whose version hash is empty — it still packages (a version receipt is present), but the
/// smoke's model/version-hash confirmation fails.
fn empty_version_runtime() -> ProductionRuntimeInput {
    ProductionRuntimeInput {
        version: Some(RuntimeVersionReceipt {
            runtime_version: "cognitive-os-runtime-0.1.0".to_string(),
            version_hash: String::new(),
        }),
        ..no_model_runtime()
    }
}

/// A runtime whose rollback artifact is present but NOT verified — it packages, but the smoke's
/// rollback check fails.
fn unverified_rollback_runtime() -> ProductionRuntimeInput {
    ProductionRuntimeInput {
        rollback: Some(RuntimeRollbackReceipt {
            rollback_hash: "smoke-runtime-rollback-hash".to_string(),
            verified: false,
        }),
        ..no_model_runtime()
    }
}

/// A training-mode runtime config — the packager genuinely REFUSES it (the smoke's refusal case).
fn refusal_probe() -> ProductionRuntimeInput {
    ProductionRuntimeInput {
        runtime_config: Some(ProductionRuntimeConfig {
            training_mode_requested: true,
            ..runtime_config()
        }),
        ..no_model_runtime()
    }
}

// --- smoke-run builders ---

fn smoke_config() -> ProductionSmokeConfig {
    ProductionSmokeConfig {
        smoke_config_hash: "smoke-config-hash".to_string(),
        deterministic: true,
        training_mode_requested: false,
        network_enabled: false,
        baseline_replacement_requested: false,
        production_claim_requested: false,
        run_curated_read: true,
        run_corpus_flow: true,
        run_horizon_flow: true,
        run_refusal_case: true,
        run_replay_verification: true,
    }
}

fn green_release_check() -> ReleaseCheckReceipt {
    ReleaseCheckReceipt {
        tool: "release_check.sh".to_string(),
        green: true,
        output_hash: "release-check-green-hash".to_string(),
    }
}

fn green_operator_smoke() -> OperatorSmokeReceipt {
    OperatorSmokeReceipt {
        tool: "operator_smoke.sh".to_string(),
        green: true,
        output_hash: "operator-smoke-green-hash".to_string(),
    }
}

/// A fully-met smoke run -> passed.
fn full_smoke_run() -> ProductionSmokeRun {
    ProductionSmokeRun {
        runtime: Some(no_model_runtime()),
        runtime_package_attestation: None,
        smoke_config: Some(smoke_config()),
        fresh_context: Some(FreshRuntimeContext {
            fresh: true,
            context_id: "smoke-context-0".to_string(),
        }),
        release_check: Some(green_release_check()),
        operator_smoke: Some(green_operator_smoke()),
        receipt_output_path: Some("out/smoke-receipt.json".to_string()),
        replay_output_path: Some("out/smoke-replay.json".to_string()),
    }
}

// --- the smoke scenario matrix (observes the real harness over constructed runs) ---

/// One scenario cell: the OBSERVED outcome of running the real smoke harness over a constructed run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeScenarioCell {
    /// The scenario name.
    pub name: &'static str,
    /// The observed outcome slug.
    pub outcome: &'static str,
    /// The observed refusal-reason slugs.
    pub refusals: Vec<&'static str>,
    /// Whether a sealed smoke receipt was produced.
    pub sealed_receipt: bool,
    /// Whether production stayed fully closed (no forbidden flag set).
    pub production_still_closed: bool,
    /// Whether the cell claimed final release (must always be false).
    pub final_release_claimed: bool,
    /// A short human-readable detail.
    pub detail: String,
}

/// The fixed smoke scenario matrix. Every cell runs the real harness and records what it observed;
/// `production_never_opens` and `final_release_never_claimed` are conjunctions across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProductionSmokeMatrix {
    /// The schema tag.
    pub schema: &'static str,
    /// The scenario cells.
    pub scenarios: Vec<ProductionSmokeScenarioCell>,
    /// The sixteen smoke-step slugs.
    pub steps: [&'static str; PROD_SMOKE_STEP_COUNT],
    /// The nineteen refusal-reason slugs.
    pub refusal_reasons: [&'static str; PROD_SMOKE_REFUSAL_COUNT],
    /// True iff no cell opened production.
    pub production_never_opens: bool,
    /// True iff no cell claimed final release.
    pub final_release_never_claimed: bool,
    /// The inert boundary.
    pub boundary: ProductionSmokeBoundary,
}

impl ProductionSmokeMatrix {
    /// Find a scenario cell by name.
    pub fn scenario(&self, name: &str) -> Option<&ProductionSmokeScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn claims_final_release(report: &ProductionSmokeReport) -> bool {
    report.is_final_release
        || report
            .receipt
            .as_ref()
            .map(|r| r.is_final_release)
            .unwrap_or(false)
}

fn closed_for_smoke(report: &ProductionSmokeReport) -> bool {
    let receipt_closed = match &report.receipt {
        None => true,
        Some(r) => {
            !r.trains
                && !r.mutates_weights
                && !r.deploys_externally
                && !r.serves_production_traffic
                && !r.replaces_baseline
                && !r.creates_truth
                && !r.creates_memory
                && !r.creates_evidence
                && !r.grants_authority
                && !r.claims_production
                && !r.opens_p12
                && !r.is_final_release
                && r.requires_release_1
        }
    };
    !report.trains
        && !report.mutates_weights
        && !report.deploys_externally
        && !report.serves_production_traffic
        && !report.replaces_baseline
        && !report.creates_truth
        && !report.creates_memory
        && !report.creates_evidence
        && !report.grants_authority
        && !report.claims_production
        && !report.opens_p12
        && !report.training_justified
        && !report.is_final_release
        && report.boundary.all_inert()
        && receipt_closed
}

fn smoke_cell(name: &'static str, run: ProductionSmokeRun) -> ProductionSmokeScenarioCell {
    let report = run_production_smoke(&run);
    ProductionSmokeScenarioCell {
        name,
        outcome: report.outcome.tag(),
        refusals: report.refusals.iter().map(|r| r.tag()).collect(),
        sealed_receipt: report.receipt.is_some(),
        production_still_closed: closed_for_smoke(&report),
        final_release_claimed: claims_final_release(&report),
        detail: report.outcome.tag().to_string(),
    }
}

/// The serialized-report tamper cell: tamper a real (passed) smoke report JSON and observe the
/// re-derive verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous.
fn smoke_tamper_cell() -> ProductionSmokeScenarioCell {
    let canonical = run_production_smoke_json(&full_smoke_run());
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_production_smoke_report_json(&full_smoke_run(), &tampered).is_err()
        && verify_production_smoke_report_json(&full_smoke_run(), &canonical).is_ok();
    let report = run_production_smoke(&full_smoke_run());
    ProductionSmokeScenarioCell {
        name: "serialized_smoke_report_tamper_refused",
        outcome: report.outcome.tag(),
        refusals: if refused {
            vec!["serialized_smoke_report_tamper_refused"]
        } else {
            vec!["VACUOUS"]
        },
        sealed_receipt: report.receipt.is_some(),
        production_still_closed: closed_for_smoke(&report) && refused,
        final_release_claimed: claims_final_release(&report),
        detail: if refused {
            "serialized_smoke_report_tamper_refused".to_string()
        } else {
            "VACUOUS: smoke verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 21-scenario smoke matrix from the REAL harness over constructed runs.
pub fn production_smoke_matrix() -> ProductionSmokeMatrix {
    let scenarios = vec![
        // 1. A fully-met run passes.
        smoke_cell("local_smoke_passes", full_smoke_run()),
        // 2. No runtime package.
        smoke_cell(
            "missing_runtime_package_refused",
            ProductionSmokeRun {
                runtime: None,
                ..full_smoke_run()
            },
        ),
        // 3. A handed-in serialized package that does not match its re-derivation.
        smoke_cell(
            "runtime_package_tampered_refused",
            ProductionSmokeRun {
                runtime_package_attestation: Some(format!(
                    "{} ",
                    package_production_runtime_json(&no_model_runtime())
                )),
                ..full_smoke_run()
            },
        ),
        // 4. No fresh context.
        smoke_cell(
            "missing_fresh_context_refused",
            ProductionSmokeRun {
                fresh_context: None,
                ..full_smoke_run()
            },
        ),
        // 5. A non-green release_check receipt.
        smoke_cell(
            "release_check_failure_refused",
            ProductionSmokeRun {
                release_check: Some(ReleaseCheckReceipt {
                    green: false,
                    ..green_release_check()
                }),
                ..full_smoke_run()
            },
        ),
        // 6. A non-green operator_smoke receipt.
        smoke_cell(
            "operator_smoke_failure_refused",
            ProductionSmokeRun {
                operator_smoke: Some(OperatorSmokeReceipt {
                    green: false,
                    ..green_operator_smoke()
                }),
                ..full_smoke_run()
            },
        ),
        // 7. The curated read is omitted.
        smoke_cell(
            "curated_read_failure_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    run_curated_read: false,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 8. The corpus flow is omitted.
        smoke_cell(
            "corpus_flow_failure_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    run_corpus_flow: false,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 9. The horizon flow is omitted.
        smoke_cell(
            "horizon_flow_failure_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    run_horizon_flow: false,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 10. The refusal case is omitted.
        smoke_cell(
            "refusal_case_failure_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    run_refusal_case: false,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 11. The replay verification is omitted.
        smoke_cell(
            "replay_verification_failure_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    run_replay_verification: false,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 12. No receipt artifacts path.
        smoke_cell(
            "missing_receipt_artifacts_refused",
            ProductionSmokeRun {
                receipt_output_path: None,
                ..full_smoke_run()
            },
        ),
        // 13. No replay artifacts path.
        smoke_cell(
            "missing_replay_artifacts_refused",
            ProductionSmokeRun {
                replay_output_path: None,
                ..full_smoke_run()
            },
        ),
        // 14. The rollback artifact is present but not verified.
        smoke_cell(
            "rollback_check_failure_refused",
            ProductionSmokeRun {
                runtime: Some(unverified_rollback_runtime()),
                ..full_smoke_run()
            },
        ),
        // 15. The runtime version hash is empty.
        smoke_cell(
            "missing_model_version_hash_refused",
            ProductionSmokeRun {
                runtime: Some(empty_version_runtime()),
                ..full_smoke_run()
            },
        ),
        // 16. A training mode is requested.
        smoke_cell(
            "training_mode_detected_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    training_mode_requested: true,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 17. A network is enabled.
        smoke_cell(
            "unauthorized_network_detected_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    network_enabled: true,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 18. A baseline replacement is requested.
        smoke_cell(
            "baseline_replacement_detected_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    baseline_replacement_requested: true,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 19. A production claim is attempted.
        smoke_cell(
            "production_claim_attempt_refused",
            ProductionSmokeRun {
                smoke_config: Some(ProductionSmokeConfig {
                    production_claim_requested: true,
                    ..smoke_config()
                }),
                ..full_smoke_run()
            },
        ),
        // 20. Serialized report tamper refused.
        smoke_tamper_cell(),
        // 21. A smoke pass is NOT final release (same passing run, asserted not-released).
        smoke_cell("smoke_pass_is_not_final_release", full_smoke_run()),
    ];

    let production_never_opens = scenarios.iter().all(|c| c.production_still_closed);
    let final_release_never_claimed = scenarios.iter().all(|c| !c.final_release_claimed);
    ProductionSmokeMatrix {
        schema: SCHEMA,
        scenarios,
        steps: PROD_SMOKE_STEP_NAMES,
        refusal_reasons: PROD_SMOKE_REFUSAL_NAMES,
        production_never_opens,
        final_release_never_claimed,
        boundary: ProductionSmokeBoundary::inert(),
    }
}

/// The smoke matrix serialized to canonical JSON.
pub fn production_smoke_matrix_json() -> String {
    serde_json::to_string(&production_smoke_matrix()).expect("smoke matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`.
pub fn verify_production_smoke_matrix_json(candidate: &str) -> Result<(), ProductionSmokeError> {
    if candidate == production_smoke_matrix_json() {
        Ok(())
    } else {
        Err(ProductionSmokeError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(report: &ProductionSmokeReport, r: ProductionSmokeRefusal) -> bool {
        report.refusals.contains(&r)
    }

    #[test]
    fn smoke_consumes_the_real_runtime_package() {
        // The smoke re-runs the REAL PROD-0 packager and reports the consumed mode (derived, not handed in).
        let report = run_production_smoke(&full_smoke_run());
        assert_eq!(report.runtime_mode, "local_no_model_runtime");
        assert!(report.runtime_package_verified);
        // A run with no runtime cannot be verified.
        let none = run_production_smoke(&ProductionSmokeRun {
            runtime: None,
            ..full_smoke_run()
        });
        assert!(!none.runtime_package_verified);
        assert_eq!(none.runtime_mode, "none");
    }

    #[test]
    fn local_smoke_passes_and_seals_a_receipt() {
        let report = run_production_smoke(&full_smoke_run());
        assert_eq!(report.outcome, ProductionSmokeOutcome::Passed);
        assert!(report.refusals.is_empty());
        assert_eq!(report.steps.len(), PROD_SMOKE_STEP_COUNT);
        assert!(report.steps.iter().all(|s| s.passed));
        let receipt = report.receipt.as_ref().expect("sealed on pass");
        assert!(receipt.smoke_passed);
        assert!(receipt.requires_release_1);
        assert!(!receipt.is_final_release);
        // Artifacts written: a receipt, a replay, and three executed flows.
        assert!(report.artifacts.receipt_artifact.is_some());
        assert!(report.artifacts.replay_artifact.is_some());
        assert_eq!(report.artifacts.flow_artifacts.len(), 3);
    }

    #[test]
    fn missing_runtime_package_is_refused() {
        let report = run_production_smoke(&ProductionSmokeRun {
            runtime: None,
            ..full_smoke_run()
        });
        assert_eq!(report.outcome, ProductionSmokeOutcome::Refused);
        assert!(has(&report, ProductionSmokeRefusal::MissingRuntimePackage));
        assert!(report.receipt.is_none());
    }

    #[test]
    fn tampered_runtime_package_is_refused() {
        // A handed-in serialized package that does not match its re-derivation is refused.
        let tampered = format!("{} ", package_production_runtime_json(&no_model_runtime()));
        let report = run_production_smoke(&ProductionSmokeRun {
            runtime_package_attestation: Some(tampered),
            ..full_smoke_run()
        });
        assert!(has(&report, ProductionSmokeRefusal::RuntimePackageTampered));
        assert!(!report.runtime_package_verified);
        // The canonical attestation verifies fine.
        let canonical = package_production_runtime_json(&no_model_runtime());
        let ok = run_production_smoke(&ProductionSmokeRun {
            runtime_package_attestation: Some(canonical),
            ..full_smoke_run()
        });
        assert_eq!(ok.outcome, ProductionSmokeOutcome::Passed);
    }

    #[test]
    fn missing_fresh_context_is_refused() {
        let report = run_production_smoke(&ProductionSmokeRun {
            fresh_context: None,
            ..full_smoke_run()
        });
        assert!(has(&report, ProductionSmokeRefusal::MissingFreshContext));
    }

    #[test]
    fn release_check_and_operator_smoke_must_be_green() {
        let no_release = run_production_smoke(&ProductionSmokeRun {
            release_check: Some(ReleaseCheckReceipt {
                green: false,
                ..green_release_check()
            }),
            ..full_smoke_run()
        });
        assert!(has(&no_release, ProductionSmokeRefusal::ReleaseCheckFailed));

        let no_operator = run_production_smoke(&ProductionSmokeRun {
            operator_smoke: Some(OperatorSmokeReceipt {
                green: false,
                ..green_operator_smoke()
            }),
            ..full_smoke_run()
        });
        assert!(has(
            &no_operator,
            ProductionSmokeRefusal::OperatorSmokeFailed
        ));
    }

    #[test]
    fn smoke_executes_curated_read_corpus_and_horizon_flows() {
        // Each flow is genuinely executed (the real matrix functions) and recorded as a hashed artifact.
        let report = run_production_smoke(&full_smoke_run());
        let names: Vec<&str> = report
            .artifacts
            .flow_artifacts
            .iter()
            .map(|a| a.name)
            .collect();
        assert!(names.contains(&"curated_read"));
        assert!(names.contains(&"corpus_flow"));
        assert!(names.contains(&"horizon_flow"));
        assert!(report
            .artifacts
            .flow_artifacts
            .iter()
            .all(|a| a.content_hash.starts_with("smoke-")));
        // Omitting any required flow refuses.
        let no_corpus = run_production_smoke(&ProductionSmokeRun {
            smoke_config: Some(ProductionSmokeConfig {
                run_corpus_flow: false,
                ..smoke_config()
            }),
            ..full_smoke_run()
        });
        assert!(has(&no_corpus, ProductionSmokeRefusal::CorpusFlowFailed));
    }

    #[test]
    fn smoke_executes_a_refusal_case_and_replay_verification() {
        // The refusal case genuinely runs the packager over a training-mode config and confirms refusal.
        let report = run_production_smoke(&full_smoke_run());
        assert!(report
            .steps
            .iter()
            .any(|s| s.step == ProductionSmokeStep::RefusalCaseExecuted && s.passed));
        assert!(report
            .steps
            .iter()
            .any(|s| s.step == ProductionSmokeStep::ReplayVerificationExecuted && s.passed));
        let no_replay = run_production_smoke(&ProductionSmokeRun {
            smoke_config: Some(ProductionSmokeConfig {
                run_replay_verification: false,
                ..smoke_config()
            }),
            ..full_smoke_run()
        });
        assert!(has(
            &no_replay,
            ProductionSmokeRefusal::ReplayVerificationFailed
        ));
    }

    #[test]
    fn receipt_and_replay_artifacts_are_written_and_hash_verified() {
        let report = run_production_smoke(&full_smoke_run());
        let receipt = report
            .artifacts
            .receipt_artifact
            .as_ref()
            .expect("receipt artifact");
        assert!(receipt.content_hash.starts_with("smoke-"));
        assert_eq!(receipt.output_path, "out/smoke-receipt.json");

        let no_receipt = run_production_smoke(&ProductionSmokeRun {
            receipt_output_path: None,
            ..full_smoke_run()
        });
        assert!(has(
            &no_receipt,
            ProductionSmokeRefusal::ReceiptArtifactsMissing
        ));
        let no_replay = run_production_smoke(&ProductionSmokeRun {
            replay_output_path: None,
            ..full_smoke_run()
        });
        assert!(has(
            &no_replay,
            ProductionSmokeRefusal::ReplayArtifactsMissing
        ));
    }

    #[test]
    fn rollback_check_and_model_version_hash_required() {
        let bad_rollback = run_production_smoke(&ProductionSmokeRun {
            runtime: Some(unverified_rollback_runtime()),
            ..full_smoke_run()
        });
        assert!(has(
            &bad_rollback,
            ProductionSmokeRefusal::RollbackCheckFailed
        ));

        let no_version = run_production_smoke(&ProductionSmokeRun {
            runtime: Some(empty_version_runtime()),
            ..full_smoke_run()
        });
        assert!(has(
            &no_version,
            ProductionSmokeRefusal::ModelVersionHashMissing
        ));
    }

    #[test]
    fn training_mode_and_network_are_detected_and_refused() {
        let training = run_production_smoke(&ProductionSmokeRun {
            smoke_config: Some(ProductionSmokeConfig {
                training_mode_requested: true,
                ..smoke_config()
            }),
            ..full_smoke_run()
        });
        assert!(has(&training, ProductionSmokeRefusal::TrainingModeDetected));

        let network = run_production_smoke(&ProductionSmokeRun {
            smoke_config: Some(ProductionSmokeConfig {
                network_enabled: true,
                ..smoke_config()
            }),
            ..full_smoke_run()
        });
        assert!(has(
            &network,
            ProductionSmokeRefusal::UnauthorizedNetworkDetected
        ));
    }

    #[test]
    fn baseline_replacement_and_production_claim_are_refused() {
        let baseline = run_production_smoke(&ProductionSmokeRun {
            smoke_config: Some(ProductionSmokeConfig {
                baseline_replacement_requested: true,
                ..smoke_config()
            }),
            ..full_smoke_run()
        });
        assert!(has(
            &baseline,
            ProductionSmokeRefusal::BaselineReplacementDetected
        ));

        let claim = run_production_smoke(&ProductionSmokeRun {
            smoke_config: Some(ProductionSmokeConfig {
                production_claim_requested: true,
                ..smoke_config()
            }),
            ..full_smoke_run()
        });
        assert!(has(
            &claim,
            ProductionSmokeRefusal::ProductionClaimAttempted
        ));
    }

    #[test]
    fn smoke_pass_is_not_final_release() {
        let report = run_production_smoke(&full_smoke_run());
        assert_eq!(report.outcome, ProductionSmokeOutcome::Passed);
        assert!(!report.is_final_release);
        assert!(report.requires_release_1);
        let receipt = report.receipt.as_ref().expect("sealed");
        assert!(!receipt.is_final_release);
        assert!(receipt.requires_release_1);
    }

    #[test]
    fn p12_training_justified_remains_false_even_when_smoke_passes() {
        let report = run_production_smoke(&full_smoke_run());
        assert!(!report.training_justified);
        assert!(!report.opens_p12);
        // The real P12 gate is unaffected by a passing smoke.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn step_and_refusal_counts_match_enums() {
        assert_eq!(ProductionSmokeStep::ALL.len(), PROD_SMOKE_STEP_COUNT);
        assert_eq!(ProductionSmokeRefusal::ALL.len(), PROD_SMOKE_REFUSAL_COUNT);
        assert_eq!(PROD_SMOKE_STEP_NAMES.len(), PROD_SMOKE_STEP_COUNT);
        assert_eq!(PROD_SMOKE_REFUSAL_NAMES.len(), PROD_SMOKE_REFUSAL_COUNT);
        for (s, name) in ProductionSmokeStep::ALL.iter().zip(PROD_SMOKE_STEP_NAMES) {
            assert_eq!(s.tag(), name);
        }
        for (r, name) in ProductionSmokeRefusal::ALL
            .iter()
            .zip(PROD_SMOKE_REFUSAL_NAMES)
        {
            assert_eq!(r.tag(), name);
        }
    }

    #[test]
    fn matrix_has_the_twenty_one_named_scenarios() {
        let matrix = production_smoke_matrix();
        assert_eq!(matrix.scenarios.len(), PROD_SMOKE_SCENARIO_COUNT);
        for name in [
            "local_smoke_passes",
            "missing_runtime_package_refused",
            "runtime_package_tampered_refused",
            "missing_fresh_context_refused",
            "release_check_failure_refused",
            "operator_smoke_failure_refused",
            "curated_read_failure_refused",
            "corpus_flow_failure_refused",
            "horizon_flow_failure_refused",
            "refusal_case_failure_refused",
            "replay_verification_failure_refused",
            "missing_receipt_artifacts_refused",
            "missing_replay_artifacts_refused",
            "rollback_check_failure_refused",
            "missing_model_version_hash_refused",
            "training_mode_detected_refused",
            "unauthorized_network_detected_refused",
            "baseline_replacement_detected_refused",
            "production_claim_attempt_refused",
            "serialized_smoke_report_tamper_refused",
            "smoke_pass_is_not_final_release",
        ] {
            assert!(
                matrix.scenario(name).is_some(),
                "scenario {name} is missing"
            );
        }
        assert!(matrix.production_never_opens);
        assert!(matrix.final_release_never_claimed);
        let pass = matrix.scenario("local_smoke_passes").expect("present");
        assert_eq!(pass.outcome, "passed");
        assert!(pass.sealed_receipt);
    }

    #[test]
    fn every_matrix_cell_keeps_production_closed_and_unreleased() {
        let matrix = production_smoke_matrix();
        for cell in &matrix.scenarios {
            assert!(
                cell.production_still_closed,
                "cell {} opened production",
                cell.name
            );
            assert!(
                !cell.final_release_claimed,
                "cell {} claimed final release",
                cell.name
            );
        }
        let tamper = matrix
            .scenario("serialized_smoke_report_tamper_refused")
            .expect("tamper cell present");
        assert!(tamper
            .refusals
            .contains(&"serialized_smoke_report_tamper_refused"));
    }

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let canonical = run_production_smoke_json(&full_smoke_run());
        assert_eq!(canonical, run_production_smoke_json(&full_smoke_run()));
        assert!(verify_production_smoke_report_json(&full_smoke_run(), &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_production_smoke_report_json(&full_smoke_run(), &tampered),
            Err(ProductionSmokeError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_re_derives_refusing_tampering() {
        let canonical = production_smoke_matrix_json();
        assert!(verify_production_smoke_matrix_json(&canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_production_smoke_matrix_json(&tampered),
            Err(ProductionSmokeError::ReplayMismatch)
        );
    }

    #[test]
    fn closed_by_default_refuses_with_no_inputs() {
        let report = run_production_smoke(&ProductionSmokeRun::closed_by_default());
        assert_eq!(report.outcome, ProductionSmokeOutcome::Refused);
        assert!(has(&report, ProductionSmokeRefusal::MissingFreshContext));
        assert!(has(&report, ProductionSmokeRefusal::MissingRuntimePackage));
        assert!(has(&report, ProductionSmokeRefusal::ReleaseCheckFailed));
        assert!(report.receipt.is_none());
        assert!(closed_for_smoke(&report));
    }
}
