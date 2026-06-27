//! RELEASE-1 — the FINAL local release gate for Cognitive OS prototype v0.1.
//!
//! This sprint answers exactly ONE question: *is the local prototype RELEASE-READY?* — and it may
//! declare so ONLY after proving the committed chain, the local production package, the production
//! smoke, the operator runbook, the rollback path, the release artifacts, and the boundary locks are
//! all intact. It does NOT deploy externally, serve traffic, train, mutate weights, open production,
//! or claim cloud/public release. The correct final claim is **local prototype release-ready**, never
//! "live production".
//!
//! It CONSUMES the REAL prior layers: [`evaluate_release_gate`] re-runs PROD-SMOKE-0's
//! [`run_production_smoke`] itself (requiring a `Passed` outcome) and PROD-0's
//! [`package_production_runtime`] itself (requiring a `Packaged` outcome), and corroborates the
//! operator-supplied smoke/package hashes against those re-derivations. It verifies the committed
//! chain head ([`EXPECTED_CHAIN_HEAD`]) and the full required lineage ([`REQUIRED_LINEAGE`]) by
//! hash-pinned receipt (the REAL git ancestry check lives in `scripts/release_check.sh`, which can run
//! git; this pure library never shells out).
//!
//! It is CLOSED BY DEFAULT and refuses TWENTY-FOUR ways: a missing release input / smoke report /
//! runtime package / artifact manifest / release notes / release runbook / operator runbook / rollback
//! receipt / chain receipt / boundary lock, a not-passed smoke, a tampered package, a chain-head
//! mismatch or missing required commit, a non-green `release_check` or `operator_smoke`, a unit-count
//! mismatch, a detected training / deployment / production-traffic / baseline-replacement intent,
//! unchecked authority drift, a dirty release scope, or a tampered serialized report.
//!
//! Crucially, `local_release_ready` is NOT production: every forbidden-action flag on the report and
//! the sealed [`ReleaseGate`] readiness receipt (`trains`, `mutates_weights`, `deploys_externally`,
//! `starts_public_production`, `serves_production_traffic`, `replaces_baseline`, `creates_truth`,
//! `creates_memory`, `creates_evidence`, `grants_authority`, `training_justified`,
//! `is_cloud_or_public_deployment`, `claims_public_release`) is sourced from the structural const
//! [`RELEASE_IS_PUBLIC`] (`false`). The deeper P12 gate (`reading_train_gate::decide`) stays
//! `training_justified = false`; P13–P15 remain closed. Reports are `Serialize` but never
//! `Deserialize`: a serialized report is re-derived from the same input and byte-compared, so
//! tampering is refused.
//!
//! The boundary, recorded verbatim in [`RELEASE_BOUNDARY_LINES`]:
//!
//!   The release gate declares local prototype release readiness only.
//!   It does not train.
//!   It does not mutate weights.
//!   It does not deploy externally.
//!   It does not start public production.
//!   It does not serve production traffic.
//!   It does not replace the baseline.
//!   It does not create truth, memory, or evidence.
//!   It does not grant new authority.
//!   LocalReleaseReady is not cloud or public deployment.

use crate::{
    package_production_runtime, package_production_runtime_json, run_production_smoke,
    run_production_smoke_json, AuthorityDriftCheck, FreshRuntimeContext, OperatorRunbookReceipt,
    OperatorSmokeReceipt, ProductionRuntimeConfig, ProductionRuntimeInput, ProductionRuntimeMode,
    ProductionRuntimeOutcome, ProductionSmokeConfig, ProductionSmokeOutcome, ProductionSmokeRun,
    ReleaseCheckReceipt, RuntimeRollbackReceipt, RuntimeVersionReceipt,
};
use serde::Serialize;

/// A non-cryptographic, dependency-free FNV-1a content hash over a consumed report's canonical JSON.
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

/// The content-hash pin for a consumed prior-layer report.
fn release_hash(s: &str) -> String {
    format!("release-{:016x}", fnv1a64(s))
}

/// The schema tag stamped on every serialized release artifact.
const SCHEMA: &str = "release-gate-v0.1";

/// THE structural invariant: declaring local release readiness is not, by itself, production — not
/// external deployment, public production, served traffic, a baseline replacement, training, an
/// authority grant, or a cloud/public release. Every forbidden-action flag is sourced from this const.
const RELEASE_IS_PUBLIC: bool = false;

/// The expected committed chain head — the PROD-SMOKE-0 commit (the releasable state).
pub const EXPECTED_CHAIN_HEAD: &str = "b653dd3";

/// The required commit lineage, in canonical order: every model-readiness layer that must be in
/// history before the local prototype can be declared release-ready.
pub const REQUIRED_LINEAGE: [(&str, &str); 9] = [
    ("score-0", "e30176e"),
    ("fail-0", "f6fd0d8"),
    ("p11-model-eval", "187466c"),
    ("train-gate-0", "2e438c4"),
    ("train-0", "72adfe4"),
    ("model-eval-1", "9597c49"),
    ("model-promote-0", "e33701b"),
    ("prod-0", "fc57104"),
    ("prod-smoke-0", "b653dd3"),
];

/// The unit-test count the local release requires (the release_check pin must match this exactly).
pub const EXPECTED_RELEASE_UNIT_COUNT: usize = 439;

/// Exactly two release decisions.
pub const RELEASE_DECISION_COUNT: usize = 2;

/// The two decision slugs, in canonical order.
pub const RELEASE_DECISION_NAMES: [&str; RELEASE_DECISION_COUNT] =
    ["release_denied", "local_release_ready"];

/// Exactly twenty-four refusal reasons.
pub const RELEASE_REFUSAL_COUNT: usize = 24;

/// The twenty-four refusal-reason slugs, in canonical order.
pub const RELEASE_REFUSAL_NAMES: [&str; RELEASE_REFUSAL_COUNT] = [
    "missing_release_input",
    "missing_prod_smoke_report",
    "prod_smoke_not_passed",
    "missing_prod_runtime_package",
    "prod_runtime_package_tampered",
    "missing_release_artifact_manifest",
    "missing_release_notes",
    "missing_release_runbook",
    "missing_operator_runbook",
    "missing_rollback_receipt",
    "missing_chain_receipt",
    "chain_head_mismatch",
    "missing_required_commit",
    "release_check_failed",
    "operator_smoke_failed",
    "unit_count_mismatch",
    "boundary_lock_missing",
    "training_detected",
    "deployment_detected",
    "production_traffic_detected",
    "baseline_replacement_detected",
    "authority_drift_detected",
    "untracked_release_scope_dirty",
    "serialized_release_report_tamper_refused",
];

/// The fixed release scenario matrix size.
pub const RELEASE_SCENARIO_COUNT: usize = 29;

/// The cannot-bypass boundary, recorded verbatim.
pub const RELEASE_BOUNDARY_LINES: [&str; 10] = [
    "The release gate declares local prototype release readiness only.",
    "It does not train.",
    "It does not mutate weights.",
    "It does not deploy externally.",
    "It does not start public production.",
    "It does not serve production traffic.",
    "It does not replace the baseline.",
    "It does not create truth, memory, or evidence.",
    "It does not grant new authority.",
    "LocalReleaseReady is not cloud or public deployment.",
];

// --- decision / refusal taxonomies ---

/// The terminal release decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReleaseDecision {
    /// The local release was denied (at least one prerequisite unmet).
    ReleaseDenied,
    /// The local prototype is release-ready (all prerequisites met). NOT external deployment.
    LocalReleaseReady,
}

impl ReleaseDecision {
    /// Every decision, in canonical order.
    pub const ALL: [ReleaseDecision; RELEASE_DECISION_COUNT] = [
        ReleaseDecision::ReleaseDenied,
        ReleaseDecision::LocalReleaseReady,
    ];

    /// The stable slug for this decision.
    pub fn tag(&self) -> &'static str {
        match self {
            ReleaseDecision::ReleaseDenied => "release_denied",
            ReleaseDecision::LocalReleaseReady => "local_release_ready",
        }
    }
}

/// Why the local release was denied. The first twenty-three are release-path reasons; the
/// twenty-fourth (`SerializedReleaseReportTamperRefused`) is emitted only by the serialized-report
/// re-derivation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReleaseRefusal {
    /// No release request.
    MissingReleaseInput,
    /// No PROD-SMOKE-0 report.
    MissingProdSmokeReport,
    /// The consumed production smoke did not pass.
    ProdSmokeNotPassed,
    /// No PROD-0 runtime package.
    MissingProdRuntimePackage,
    /// The supplied runtime package hash did not match its re-derivation.
    ProdRuntimePackageTampered,
    /// No release artifact manifest.
    MissingReleaseArtifactManifest,
    /// No release notes.
    MissingReleaseNotes,
    /// No release runbook.
    MissingReleaseRunbook,
    /// No operator runbook.
    MissingOperatorRunbook,
    /// No rollback receipt (or it was not verified).
    MissingRollbackReceipt,
    /// No chain receipt.
    MissingChainReceipt,
    /// The chain head did not match the expected releasable commit.
    ChainHeadMismatch,
    /// A required lineage commit was missing or did not match.
    MissingRequiredCommit,
    /// `release_check` did not record a green receipt.
    ReleaseCheckFailed,
    /// `operator_smoke` did not record a green receipt.
    OperatorSmokeFailed,
    /// The unit-count receipt did not match the required count.
    UnitCountMismatch,
    /// No boundary-lock receipt (or the locks were not intact).
    BoundaryLockMissing,
    /// A training intent was detected (refused — the release does not train).
    TrainingDetected,
    /// A deployment intent was detected.
    DeploymentDetected,
    /// A production-traffic intent was detected.
    ProductionTrafficDetected,
    /// A baseline-replacement intent was detected.
    BaselineReplacementDetected,
    /// The authority-drift check was not run, or it detected drift.
    AuthorityDriftDetected,
    /// The release scope carried unrelated dirt.
    UntrackedReleaseScopeDirty,
    /// A serialized release report did not match its re-derivation and was refused.
    SerializedReleaseReportTamperRefused,
}

impl ReleaseRefusal {
    /// Every refusal reason, in canonical order.
    pub const ALL: [ReleaseRefusal; RELEASE_REFUSAL_COUNT] = [
        ReleaseRefusal::MissingReleaseInput,
        ReleaseRefusal::MissingProdSmokeReport,
        ReleaseRefusal::ProdSmokeNotPassed,
        ReleaseRefusal::MissingProdRuntimePackage,
        ReleaseRefusal::ProdRuntimePackageTampered,
        ReleaseRefusal::MissingReleaseArtifactManifest,
        ReleaseRefusal::MissingReleaseNotes,
        ReleaseRefusal::MissingReleaseRunbook,
        ReleaseRefusal::MissingOperatorRunbook,
        ReleaseRefusal::MissingRollbackReceipt,
        ReleaseRefusal::MissingChainReceipt,
        ReleaseRefusal::ChainHeadMismatch,
        ReleaseRefusal::MissingRequiredCommit,
        ReleaseRefusal::ReleaseCheckFailed,
        ReleaseRefusal::OperatorSmokeFailed,
        ReleaseRefusal::UnitCountMismatch,
        ReleaseRefusal::BoundaryLockMissing,
        ReleaseRefusal::TrainingDetected,
        ReleaseRefusal::DeploymentDetected,
        ReleaseRefusal::ProductionTrafficDetected,
        ReleaseRefusal::BaselineReplacementDetected,
        ReleaseRefusal::AuthorityDriftDetected,
        ReleaseRefusal::UntrackedReleaseScopeDirty,
        ReleaseRefusal::SerializedReleaseReportTamperRefused,
    ];

    /// The stable slug for this refusal reason.
    pub fn tag(&self) -> &'static str {
        match self {
            ReleaseRefusal::MissingReleaseInput => "missing_release_input",
            ReleaseRefusal::MissingProdSmokeReport => "missing_prod_smoke_report",
            ReleaseRefusal::ProdSmokeNotPassed => "prod_smoke_not_passed",
            ReleaseRefusal::MissingProdRuntimePackage => "missing_prod_runtime_package",
            ReleaseRefusal::ProdRuntimePackageTampered => "prod_runtime_package_tampered",
            ReleaseRefusal::MissingReleaseArtifactManifest => "missing_release_artifact_manifest",
            ReleaseRefusal::MissingReleaseNotes => "missing_release_notes",
            ReleaseRefusal::MissingReleaseRunbook => "missing_release_runbook",
            ReleaseRefusal::MissingOperatorRunbook => "missing_operator_runbook",
            ReleaseRefusal::MissingRollbackReceipt => "missing_rollback_receipt",
            ReleaseRefusal::MissingChainReceipt => "missing_chain_receipt",
            ReleaseRefusal::ChainHeadMismatch => "chain_head_mismatch",
            ReleaseRefusal::MissingRequiredCommit => "missing_required_commit",
            ReleaseRefusal::ReleaseCheckFailed => "release_check_failed",
            ReleaseRefusal::OperatorSmokeFailed => "operator_smoke_failed",
            ReleaseRefusal::UnitCountMismatch => "unit_count_mismatch",
            ReleaseRefusal::BoundaryLockMissing => "boundary_lock_missing",
            ReleaseRefusal::TrainingDetected => "training_detected",
            ReleaseRefusal::DeploymentDetected => "deployment_detected",
            ReleaseRefusal::ProductionTrafficDetected => "production_traffic_detected",
            ReleaseRefusal::BaselineReplacementDetected => "baseline_replacement_detected",
            ReleaseRefusal::AuthorityDriftDetected => "authority_drift_detected",
            ReleaseRefusal::UntrackedReleaseScopeDirty => "untracked_release_scope_dirty",
            ReleaseRefusal::SerializedReleaseReportTamperRefused => {
                "serialized_release_report_tamper_refused"
            }
        }
    }
}

// --- inputs (never trusted off-wire: Debug + Clone, no Serialize, no Deserialize) ---

/// One pinned commit in the required lineage.
#[derive(Debug, Clone)]
pub struct ReleaseCommitPin {
    /// The layer name.
    pub name: String,
    /// The commit hash.
    pub commit: String,
}

/// The committed-chain receipt: the observed head + the lineage pins. The REAL git ancestry check is
/// performed by `scripts/release_check.sh`; this receipt records what it observed.
#[derive(Debug, Clone)]
pub struct ReleaseChainReceipt {
    /// The observed chain head.
    pub chain_head: String,
    /// The observed lineage pins.
    pub lineage: Vec<ReleaseCommitPin>,
}

/// A PROD-SMOKE-0 receipt: the smoke passed, pinned by its report hash.
#[derive(Debug, Clone)]
pub struct ReleaseSmokeReceipt {
    /// Whether the production smoke passed.
    pub smoke_passed: bool,
    /// The content hash pinning the smoke report.
    pub report_hash: String,
}

/// A hash-pinned rollback receipt for the release.
#[derive(Debug, Clone)]
pub struct ReleaseRollbackReceipt {
    /// The content hash pinning the rollback target.
    pub rollback_hash: String,
    /// Whether the rollback path was verified.
    pub verified: bool,
}

/// The boundary-lock receipt: the release_check boundary locks are intact.
#[derive(Debug, Clone)]
pub struct ReleaseBoundaryReceipt {
    /// The boundary identifier.
    pub boundary_id: String,
    /// Whether the boundary locks are intact.
    pub locks_intact: bool,
}

/// The operator-runbook receipt — the operator has the release operator runbook in hand.
#[derive(Debug, Clone)]
pub struct ReleaseOperatorRunbookReceipt {
    /// The runbook identifier.
    pub runbook_id: String,
}

/// The release-notes receipt.
#[derive(Debug, Clone)]
pub struct ReleaseNotesReceipt {
    /// The notes identifier.
    pub notes_id: String,
    /// The released version.
    pub version: String,
}

/// One release artifact, pinned by content hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseArtifact {
    /// The artifact name.
    pub name: String,
    /// The content hash pinning the artifact.
    pub content_hash: String,
    /// The artifact path.
    pub path: String,
}

/// The release artifact manifest: the artifacts the local release bundles (sources, docs, runbooks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseArtifactManifest {
    /// The schema tag.
    pub schema: String,
    /// The released version.
    pub version: String,
    /// The release artifacts.
    pub artifacts: Vec<ReleaseArtifact>,
}

/// The full set of inputs the release gate weighs. INPUT type (never `Serialize`): it re-runs the real
/// PROD-SMOKE-0 and PROD-0 functions and corroborates the supplied hashes. Closed by default.
#[derive(Debug, Clone)]
pub struct ReleaseGateInput {
    /// The release request identifier (absent -> the input is missing).
    pub release_request_id: Option<String>,
    /// The PROD-SMOKE-0 receipt (smoke passed + report hash).
    pub prod_smoke: Option<ReleaseSmokeReceipt>,
    /// The PROD-0 runtime package hash (corroborated against the re-derived package).
    pub prod_runtime_package_hash: Option<String>,
    /// The committed-chain receipt.
    pub chain: Option<ReleaseChainReceipt>,
    /// The release artifact manifest.
    pub artifact_manifest: Option<ReleaseArtifactManifest>,
    /// The release-notes receipt.
    pub release_notes: Option<ReleaseNotesReceipt>,
    /// The release-runbook identifier.
    pub release_runbook_id: Option<String>,
    /// The operator-runbook receipt.
    pub operator_runbook: Option<ReleaseOperatorRunbookReceipt>,
    /// The rollback receipt.
    pub rollback: Option<ReleaseRollbackReceipt>,
    /// The boundary-lock receipt.
    pub boundary: Option<ReleaseBoundaryReceipt>,
    /// The recorded `release_check` green receipt.
    pub release_check: Option<ReleaseCheckReceipt>,
    /// The recorded `operator_smoke` green receipt.
    pub operator_smoke: Option<OperatorSmokeReceipt>,
    /// The observed unit-test count (must equal `EXPECTED_RELEASE_UNIT_COUNT`).
    pub unit_count: Option<usize>,
    /// Whether the release scope is clean (no unrelated dirt staged).
    pub release_scope_clean: Option<bool>,
    /// A training intent (must be false — the release does not train).
    pub training_requested: bool,
    /// A deployment intent (must be false).
    pub deployment_requested: bool,
    /// A production-traffic intent (must be false).
    pub production_traffic_requested: bool,
    /// A baseline-replacement intent (must be false).
    pub baseline_replacement_requested: bool,
    /// The authority-drift check (unchecked by default).
    pub authority_drift: AuthorityDriftCheck,
}

impl ReleaseGateInput {
    /// The closed-by-default input: nothing supplied, drift unchecked. The release is denied.
    pub fn closed_by_default() -> Self {
        Self {
            release_request_id: None,
            prod_smoke: None,
            prod_runtime_package_hash: None,
            chain: None,
            artifact_manifest: None,
            release_notes: None,
            release_runbook_id: None,
            operator_runbook: None,
            rollback: None,
            boundary: None,
            release_check: None,
            operator_smoke: None,
            unit_count: None,
            release_scope_clean: None,
            training_requested: false,
            deployment_requested: false,
            production_traffic_requested: false,
            baseline_replacement_requested: false,
            authority_drift: AuthorityDriftCheck::unchecked(),
        }
    }
}

// --- the boundary record + sealed readiness receipt ---

/// The inert boundary: every forbidden action is `false`. Stamped on every report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReleaseGateBoundary {
    /// The release never trains.
    pub trains: bool,
    /// The release never mutates weights.
    pub mutates_weights: bool,
    /// The release never deploys externally.
    pub deploys_externally: bool,
    /// The release never starts public production.
    pub starts_public_production: bool,
    /// The release never serves production traffic.
    pub serves_production_traffic: bool,
    /// The release never replaces the baseline.
    pub replaces_baseline: bool,
    /// The release never creates truth.
    pub creates_truth: bool,
    /// The release never creates memory.
    pub creates_memory: bool,
    /// The release never creates evidence.
    pub creates_evidence: bool,
    /// The release never grants new authority.
    pub grants_authority: bool,
}

impl ReleaseGateBoundary {
    fn inert() -> Self {
        Self {
            trains: RELEASE_IS_PUBLIC,
            mutates_weights: RELEASE_IS_PUBLIC,
            deploys_externally: RELEASE_IS_PUBLIC,
            starts_public_production: RELEASE_IS_PUBLIC,
            serves_production_traffic: RELEASE_IS_PUBLIC,
            replaces_baseline: RELEASE_IS_PUBLIC,
            creates_truth: RELEASE_IS_PUBLIC,
            creates_memory: RELEASE_IS_PUBLIC,
            creates_evidence: RELEASE_IS_PUBLIC,
            grants_authority: RELEASE_IS_PUBLIC,
        }
    }

    /// True iff every forbidden action is inert.
    pub fn all_inert(&self) -> bool {
        !self.trains
            && !self.mutates_weights
            && !self.deploys_externally
            && !self.starts_public_production
            && !self.serves_production_traffic
            && !self.replaces_baseline
            && !self.creates_truth
            && !self.creates_memory
            && !self.creates_evidence
            && !self.grants_authority
    }
}

/// The SEALED local-release-ready receipt produced ONLY on a successful release decision. It records
/// the local prototype is release-ready — it deploys nothing, starts no public production, serves no
/// traffic, and is NOT a cloud or public deployment. `Serialize` but never `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseGate {
    /// The schema tag.
    pub schema: &'static str,
    /// The decision slug — always `local_release_ready` when sealed.
    pub decision: &'static str,
    /// Always `true`: the local prototype is release-ready.
    pub local_release_ready: bool,
    /// The pinned releasable chain head.
    pub chain_head: &'static str,
    /// Always `false`: this readiness is NOT a cloud or public deployment.
    pub is_cloud_or_public_deployment: bool,
    /// Always `false`: the release claims no public release.
    pub claims_public_release: bool,
    /// Always `false`: the release trains nothing.
    pub trains: bool,
    /// Always `false`: the release mutates no weights.
    pub mutates_weights: bool,
    /// Always `false`: the release deploys nothing externally.
    pub deploys_externally: bool,
    /// Always `false`: the release starts no public production.
    pub starts_public_production: bool,
    /// Always `false`: the release serves no production traffic.
    pub serves_production_traffic: bool,
    /// Always `false`: the release replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the release creates no truth.
    pub creates_truth: bool,
    /// Always `false`: the release creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the release creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the release grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the release does not set P12 `training_justified`.
    pub training_justified: bool,
}

// --- the report (top-level) ---

/// The release gate's verdict on whether the local prototype is release-ready. `Serialize` but never
/// `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseGateReport {
    /// The schema tag.
    pub schema: &'static str,
    /// The terminal decision.
    pub decision: ReleaseDecision,
    /// Why the release was denied (empty iff release-ready).
    pub refusals: Vec<ReleaseRefusal>,
    /// The pinned releasable chain head.
    pub chain_head: &'static str,
    /// Whether the consumed PROD-SMOKE-0 smoke passed.
    pub prod_smoke_passed: bool,
    /// Whether the consumed PROD-0 runtime package packaged.
    pub prod_runtime_packaged: bool,
    /// The sealed readiness receipt (present ONLY when release-ready).
    pub seal: Option<ReleaseGate>,
    /// Always `false`: this readiness is NOT a cloud or public deployment.
    pub is_cloud_or_public_deployment: bool,
    /// Always `false`: the release claims no public release.
    pub claims_public_release: bool,
    /// Always `false`: the release trains nothing.
    pub trains: bool,
    /// Always `false`: the release mutates no weights.
    pub mutates_weights: bool,
    /// Always `false`: the release deploys nothing externally.
    pub deploys_externally: bool,
    /// Always `false`: the release starts no public production.
    pub starts_public_production: bool,
    /// Always `false`: the release serves no production traffic.
    pub serves_production_traffic: bool,
    /// Always `false`: the release replaces no baseline.
    pub replaces_baseline: bool,
    /// Always `false`: the release creates no truth.
    pub creates_truth: bool,
    /// Always `false`: the release creates no memory.
    pub creates_memory: bool,
    /// Always `false`: the release creates no evidence.
    pub creates_evidence: bool,
    /// Always `false`: the release grants no authority.
    pub grants_authority: bool,
    /// Always `false`: the release does not set P12 `training_justified`.
    pub training_justified: bool,
    /// The inert boundary.
    pub boundary: ReleaseGateBoundary,
}

/// True iff `pinned` is present (non-empty) AND equals `derived`.
fn hash_ok(pinned: &str, derived: &str) -> bool {
    !pinned.is_empty() && pinned == derived
}

/// Evaluate the final local release gate over `input`. It re-runs the REAL PROD-SMOKE-0 and PROD-0
/// functions, verifies the committed chain + lineage, requires every release receipt, refuses any
/// training / deployment / production-traffic / baseline-replacement intent and any dirty scope, and
/// declares `local_release_ready` only when every requirement holds. Deploys nothing, starts no public
/// production, serves no traffic, claims no public release.
pub fn evaluate_release_gate(input: &ReleaseGateInput) -> ReleaseGateReport {
    let mut refusals: Vec<ReleaseRefusal> = Vec::new();

    // CONSUME PROD-SMOKE-0 and PROD-0: re-run the real functions and derive their canonical hashes.
    let smoke = run_production_smoke(&release_smoke_run());
    let smoke_passed = smoke.outcome == ProductionSmokeOutcome::Passed;
    let smoke_hash = release_hash(&run_production_smoke_json(&release_smoke_run()));
    let package = package_production_runtime(&release_runtime());
    let prod_runtime_packaged = package.outcome == ProductionRuntimeOutcome::Packaged;
    let package_hash = release_hash(&package_production_runtime_json(&release_runtime()));

    // The release request itself.
    if input.release_request_id.as_deref().unwrap_or("").is_empty() {
        refusals.push(ReleaseRefusal::MissingReleaseInput);
    }

    // PROD-SMOKE-0: a passed smoke, corroborated by its report hash.
    match &input.prod_smoke {
        None => refusals.push(ReleaseRefusal::MissingProdSmokeReport),
        Some(r) => {
            if !r.smoke_passed || !smoke_passed || !hash_ok(&r.report_hash, &smoke_hash) {
                refusals.push(ReleaseRefusal::ProdSmokeNotPassed);
            }
        }
    }

    // PROD-0: a packaged runtime, corroborated by its package hash.
    match input.prod_runtime_package_hash.as_deref() {
        None | Some("") => refusals.push(ReleaseRefusal::MissingProdRuntimePackage),
        Some(h) => {
            if !prod_runtime_packaged {
                refusals.push(ReleaseRefusal::MissingProdRuntimePackage);
            } else if !hash_ok(h, &package_hash) {
                refusals.push(ReleaseRefusal::ProdRuntimePackageTampered);
            }
        }
    }

    // Release artifacts, notes, runbooks.
    if input.artifact_manifest.is_none() {
        refusals.push(ReleaseRefusal::MissingReleaseArtifactManifest);
    }
    if input.release_notes.is_none() {
        refusals.push(ReleaseRefusal::MissingReleaseNotes);
    }
    if input.release_runbook_id.as_deref().unwrap_or("").is_empty() {
        refusals.push(ReleaseRefusal::MissingReleaseRunbook);
    }
    if input.operator_runbook.is_none() {
        refusals.push(ReleaseRefusal::MissingOperatorRunbook);
    }

    // Rollback + boundary locks.
    if !input.rollback.as_ref().map(|r| r.verified).unwrap_or(false) {
        refusals.push(ReleaseRefusal::MissingRollbackReceipt);
    }
    if !input
        .boundary
        .as_ref()
        .map(|b| b.locks_intact)
        .unwrap_or(false)
    {
        refusals.push(ReleaseRefusal::BoundaryLockMissing);
    }

    // The committed chain head + required lineage.
    match &input.chain {
        None => refusals.push(ReleaseRefusal::MissingChainReceipt),
        Some(c) => {
            if c.chain_head != EXPECTED_CHAIN_HEAD {
                refusals.push(ReleaseRefusal::ChainHeadMismatch);
            }
            let lineage_complete = REQUIRED_LINEAGE.iter().all(|(name, commit)| {
                c.lineage
                    .iter()
                    .any(|p| p.name == *name && p.commit == *commit)
            });
            if !lineage_complete {
                refusals.push(ReleaseRefusal::MissingRequiredCommit);
            }
        }
    }

    // Green signals + unit-count pin.
    if !input
        .release_check
        .as_ref()
        .map(|r| r.green && !r.output_hash.is_empty())
        .unwrap_or(false)
    {
        refusals.push(ReleaseRefusal::ReleaseCheckFailed);
    }
    if !input
        .operator_smoke
        .as_ref()
        .map(|r| r.green && !r.output_hash.is_empty())
        .unwrap_or(false)
    {
        refusals.push(ReleaseRefusal::OperatorSmokeFailed);
    }
    if input.unit_count != Some(EXPECTED_RELEASE_UNIT_COUNT) {
        refusals.push(ReleaseRefusal::UnitCountMismatch);
    }

    // Safety intents + drift + scope.
    if input.training_requested {
        refusals.push(ReleaseRefusal::TrainingDetected);
    }
    if input.deployment_requested {
        refusals.push(ReleaseRefusal::DeploymentDetected);
    }
    if input.production_traffic_requested {
        refusals.push(ReleaseRefusal::ProductionTrafficDetected);
    }
    if input.baseline_replacement_requested {
        refusals.push(ReleaseRefusal::BaselineReplacementDetected);
    }
    if !input.authority_drift.is_clean() {
        refusals.push(ReleaseRefusal::AuthorityDriftDetected);
    }
    if !input.release_scope_clean.unwrap_or(false) {
        refusals.push(ReleaseRefusal::UntrackedReleaseScopeDirty);
    }

    let decision = if refusals.is_empty() {
        ReleaseDecision::LocalReleaseReady
    } else {
        ReleaseDecision::ReleaseDenied
    };

    let seal = if decision == ReleaseDecision::LocalReleaseReady {
        Some(ReleaseGate {
            schema: SCHEMA,
            decision: ReleaseDecision::LocalReleaseReady.tag(),
            local_release_ready: true,
            chain_head: EXPECTED_CHAIN_HEAD,
            is_cloud_or_public_deployment: RELEASE_IS_PUBLIC,
            claims_public_release: RELEASE_IS_PUBLIC,
            trains: RELEASE_IS_PUBLIC,
            mutates_weights: RELEASE_IS_PUBLIC,
            deploys_externally: RELEASE_IS_PUBLIC,
            starts_public_production: RELEASE_IS_PUBLIC,
            serves_production_traffic: RELEASE_IS_PUBLIC,
            replaces_baseline: RELEASE_IS_PUBLIC,
            creates_truth: RELEASE_IS_PUBLIC,
            creates_memory: RELEASE_IS_PUBLIC,
            creates_evidence: RELEASE_IS_PUBLIC,
            grants_authority: RELEASE_IS_PUBLIC,
            training_justified: RELEASE_IS_PUBLIC,
        })
    } else {
        None
    };

    ReleaseGateReport {
        schema: SCHEMA,
        decision,
        refusals,
        chain_head: EXPECTED_CHAIN_HEAD,
        prod_smoke_passed: smoke_passed,
        prod_runtime_packaged,
        seal,
        is_cloud_or_public_deployment: RELEASE_IS_PUBLIC,
        claims_public_release: RELEASE_IS_PUBLIC,
        trains: RELEASE_IS_PUBLIC,
        mutates_weights: RELEASE_IS_PUBLIC,
        deploys_externally: RELEASE_IS_PUBLIC,
        starts_public_production: RELEASE_IS_PUBLIC,
        serves_production_traffic: RELEASE_IS_PUBLIC,
        replaces_baseline: RELEASE_IS_PUBLIC,
        creates_truth: RELEASE_IS_PUBLIC,
        creates_memory: RELEASE_IS_PUBLIC,
        creates_evidence: RELEASE_IS_PUBLIC,
        grants_authority: RELEASE_IS_PUBLIC,
        training_justified: RELEASE_IS_PUBLIC,
        boundary: ReleaseGateBoundary::inert(),
    }
}

/// The release report serialized to canonical JSON.
pub fn evaluate_release_gate_json(input: &ReleaseGateInput) -> String {
    serde_json::to_string(&evaluate_release_gate(input)).expect("release report serializes")
}

/// What can go wrong verifying a serialized release report.
#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseGateError {
    /// The candidate bytes do not equal the re-derived canonical artifact.
    ReplayMismatch,
}

/// Re-derive the report from the SAME input and byte-compare against `candidate`. The report is
/// `Serialize` but never `Deserialize`: a serialized report is NOT trusted as authority — it is
/// re-derived and compared, so any tampering is refused.
pub fn verify_release_gate_report_json(
    input: &ReleaseGateInput,
    candidate: &str,
) -> Result<(), ReleaseGateError> {
    if candidate == evaluate_release_gate_json(input) {
        Ok(())
    } else {
        Err(ReleaseGateError::ReplayMismatch)
    }
}

// --- internal consumption builders (the REAL PROD-SMOKE-0 smoke run + PROD-0 runtime, pub types) ---

fn release_runtime() -> ProductionRuntimeInput {
    ProductionRuntimeInput {
        mode: ProductionRuntimeMode::LocalNoModelRuntime,
        promotion: None,
        runtime_config: Some(ProductionRuntimeConfig {
            config_hash: "release-runtime-config-hash".to_string(),
            deterministic: true,
            training_mode_requested: false,
            network_enabled: false,
            local_offline: true,
        }),
        model_slot: None,
        version: Some(RuntimeVersionReceipt {
            runtime_version: "cognitive-os-runtime-0.1.0".to_string(),
            version_hash: "release-runtime-version-hash".to_string(),
        }),
        rollback: Some(RuntimeRollbackReceipt {
            rollback_hash: "release-runtime-rollback-hash".to_string(),
            verified: true,
        }),
        operator_runbook: Some(OperatorRunbookReceipt {
            runbook_id: "production-runtime-runbook-0".to_string(),
        }),
        receipt_output_path: Some("out/runtime-receipt.json".to_string()),
        replay_output_path: Some("out/runtime-replay.json".to_string()),
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

fn release_smoke_run() -> ProductionSmokeRun {
    ProductionSmokeRun {
        runtime: Some(release_runtime()),
        runtime_package_attestation: None,
        smoke_config: Some(ProductionSmokeConfig {
            smoke_config_hash: "release-smoke-config-hash".to_string(),
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
        }),
        fresh_context: Some(FreshRuntimeContext {
            fresh: true,
            context_id: "release-smoke-context-0".to_string(),
        }),
        release_check: Some(ReleaseCheckReceipt {
            tool: "release_check.sh".to_string(),
            green: true,
            output_hash: "release-check-green-hash".to_string(),
        }),
        operator_smoke: Some(OperatorSmokeReceipt {
            tool: "operator_smoke.sh".to_string(),
            green: true,
            output_hash: "operator-smoke-green-hash".to_string(),
        }),
        receipt_output_path: Some("out/smoke-receipt.json".to_string()),
        replay_output_path: Some("out/smoke-replay.json".to_string()),
    }
}

// --- release-input builders ---

fn chain_receipt() -> ReleaseChainReceipt {
    ReleaseChainReceipt {
        chain_head: EXPECTED_CHAIN_HEAD.to_string(),
        lineage: REQUIRED_LINEAGE
            .iter()
            .map(|(name, commit)| ReleaseCommitPin {
                name: name.to_string(),
                commit: commit.to_string(),
            })
            .collect(),
    }
}

fn artifact_manifest() -> ReleaseArtifactManifest {
    ReleaseArtifactManifest {
        schema: SCHEMA.to_string(),
        version: "v0.1".to_string(),
        artifacts: vec![
            ReleaseArtifact {
                name: "release_gate".to_string(),
                content_hash: "release-artifact-gate-hash".to_string(),
                path: "crates/cognitive-demo/src/release_gate.rs".to_string(),
            },
            ReleaseArtifact {
                name: "release_runbook".to_string(),
                content_hash: "release-artifact-runbook-hash".to_string(),
                path: "docs/RELEASE_RUNBOOK.md".to_string(),
            },
            ReleaseArtifact {
                name: "release_notes".to_string(),
                content_hash: "release-artifact-notes-hash".to_string(),
                path: "docs/RELEASE_NOTES_v0.1.md".to_string(),
            },
        ],
    }
}

/// A fully-met release input -> `local_release_ready`.
fn full_release_input() -> ReleaseGateInput {
    let smoke_hash = release_hash(&run_production_smoke_json(&release_smoke_run()));
    let package_hash = release_hash(&package_production_runtime_json(&release_runtime()));
    ReleaseGateInput {
        release_request_id: Some("release-v0.1-request".to_string()),
        prod_smoke: Some(ReleaseSmokeReceipt {
            smoke_passed: true,
            report_hash: smoke_hash,
        }),
        prod_runtime_package_hash: Some(package_hash),
        chain: Some(chain_receipt()),
        artifact_manifest: Some(artifact_manifest()),
        release_notes: Some(ReleaseNotesReceipt {
            notes_id: "release-notes-v0.1".to_string(),
            version: "v0.1".to_string(),
        }),
        release_runbook_id: Some("release-runbook-0".to_string()),
        operator_runbook: Some(ReleaseOperatorRunbookReceipt {
            runbook_id: "production-runtime-runbook-0".to_string(),
        }),
        rollback: Some(ReleaseRollbackReceipt {
            rollback_hash: "release-rollback-hash".to_string(),
            verified: true,
        }),
        boundary: Some(ReleaseBoundaryReceipt {
            boundary_id: "release-boundary-0".to_string(),
            locks_intact: true,
        }),
        release_check: Some(ReleaseCheckReceipt {
            tool: "release_check.sh".to_string(),
            green: true,
            output_hash: "release-check-green-hash".to_string(),
        }),
        operator_smoke: Some(OperatorSmokeReceipt {
            tool: "operator_smoke.sh".to_string(),
            green: true,
            output_hash: "operator-smoke-green-hash".to_string(),
        }),
        unit_count: Some(EXPECTED_RELEASE_UNIT_COUNT),
        release_scope_clean: Some(true),
        training_requested: false,
        deployment_requested: false,
        production_traffic_requested: false,
        baseline_replacement_requested: false,
        authority_drift: AuthorityDriftCheck::clean(),
    }
}

// --- the release scenario matrix (observes the real gate over constructed inputs) ---

/// One scenario cell: the OBSERVED outcome of running the real release gate over a constructed input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseScenarioCell {
    /// The scenario name.
    pub name: &'static str,
    /// The observed decision slug.
    pub decision: &'static str,
    /// The observed refusal-reason slugs.
    pub refusals: Vec<&'static str>,
    /// Whether a sealed readiness receipt was produced.
    pub sealed: bool,
    /// Whether the release stayed local (no forbidden flag set; not a public/cloud deployment).
    pub release_stays_local: bool,
    /// Whether the cell claimed public release (must always be false).
    pub public_release_claimed: bool,
    /// A short human-readable detail.
    pub detail: String,
}

/// The fixed release scenario matrix. Every cell runs the real gate and records what it observed;
/// `release_never_goes_public` and `public_release_never_claimed` are conjunctions across all cells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReleaseMatrix {
    /// The schema tag.
    pub schema: &'static str,
    /// The scenario cells.
    pub scenarios: Vec<ReleaseScenarioCell>,
    /// The two decision slugs.
    pub decisions: [&'static str; RELEASE_DECISION_COUNT],
    /// The twenty-four refusal-reason slugs.
    pub refusal_reasons: [&'static str; RELEASE_REFUSAL_COUNT],
    /// True iff no cell went public/external.
    pub release_never_goes_public: bool,
    /// True iff no cell claimed public release.
    pub public_release_never_claimed: bool,
    /// The inert boundary.
    pub boundary: ReleaseGateBoundary,
}

impl ReleaseMatrix {
    /// Find a scenario cell by name.
    pub fn scenario(&self, name: &str) -> Option<&ReleaseScenarioCell> {
        self.scenarios.iter().find(|c| c.name == name)
    }
}

fn claims_public(report: &ReleaseGateReport) -> bool {
    report.is_cloud_or_public_deployment
        || report.claims_public_release
        || report
            .seal
            .as_ref()
            .map(|s| s.is_cloud_or_public_deployment || s.claims_public_release)
            .unwrap_or(false)
}

fn stays_local(report: &ReleaseGateReport) -> bool {
    let seal_local = match &report.seal {
        None => true,
        Some(s) => {
            !s.is_cloud_or_public_deployment
                && !s.claims_public_release
                && !s.trains
                && !s.mutates_weights
                && !s.deploys_externally
                && !s.starts_public_production
                && !s.serves_production_traffic
                && !s.replaces_baseline
                && !s.creates_truth
                && !s.creates_memory
                && !s.creates_evidence
                && !s.grants_authority
                && !s.training_justified
                && s.local_release_ready
        }
    };
    !report.is_cloud_or_public_deployment
        && !report.claims_public_release
        && !report.trains
        && !report.mutates_weights
        && !report.deploys_externally
        && !report.starts_public_production
        && !report.serves_production_traffic
        && !report.replaces_baseline
        && !report.creates_truth
        && !report.creates_memory
        && !report.creates_evidence
        && !report.grants_authority
        && !report.training_justified
        && report.boundary.all_inert()
        && seal_local
}

fn release_cell(name: &'static str, input: ReleaseGateInput) -> ReleaseScenarioCell {
    let report = evaluate_release_gate(&input);
    ReleaseScenarioCell {
        name,
        decision: report.decision.tag(),
        refusals: report.refusals.iter().map(|r| r.tag()).collect(),
        sealed: report.seal.is_some(),
        release_stays_local: stays_local(&report),
        public_release_claimed: claims_public(&report),
        detail: report.decision.tag().to_string(),
    }
}

/// The serialized-report tamper cell: tamper a real (release-ready) report JSON and observe the
/// re-derive verifier refuse it. The `tampered != canonical` guard makes the refusal non-vacuous.
fn release_tamper_cell() -> ReleaseScenarioCell {
    let canonical = evaluate_release_gate_json(&full_release_input());
    let tampered = format!("{canonical} ");
    let refused = tampered != canonical
        && verify_release_gate_report_json(&full_release_input(), &tampered).is_err()
        && verify_release_gate_report_json(&full_release_input(), &canonical).is_ok();
    let report = evaluate_release_gate(&full_release_input());
    ReleaseScenarioCell {
        name: "serialized_release_report_tamper_refused",
        decision: report.decision.tag(),
        refusals: if refused {
            vec!["serialized_release_report_tamper_refused"]
        } else {
            vec!["VACUOUS"]
        },
        sealed: report.seal.is_some(),
        release_stays_local: stays_local(&report) && refused,
        public_release_claimed: claims_public(&report),
        detail: if refused {
            "serialized_release_report_tamper_refused".to_string()
        } else {
            "VACUOUS: release verifier did not refuse tamper".to_string()
        },
    }
}

/// Build the fixed 29-scenario release matrix from the REAL gate over constructed inputs.
pub fn release_matrix() -> ReleaseMatrix {
    let scenarios = vec![
        // 1. A fully-met input is release-ready.
        release_cell("local_release_ready", full_release_input()),
        // 2-24. Each missing/failed/detected requirement denies.
        release_cell(
            "missing_release_input_denied",
            ReleaseGateInput {
                release_request_id: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_prod_smoke_report_denied",
            ReleaseGateInput {
                prod_smoke: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "prod_smoke_not_passed_denied",
            ReleaseGateInput {
                prod_smoke: Some(ReleaseSmokeReceipt {
                    smoke_passed: false,
                    report_hash: release_hash(&run_production_smoke_json(&release_smoke_run())),
                }),
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_prod_runtime_package_denied",
            ReleaseGateInput {
                prod_runtime_package_hash: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "prod_runtime_package_tampered_denied",
            ReleaseGateInput {
                prod_runtime_package_hash: Some("wrong-package-hash".to_string()),
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_release_artifact_manifest_denied",
            ReleaseGateInput {
                artifact_manifest: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_release_notes_denied",
            ReleaseGateInput {
                release_notes: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_release_runbook_denied",
            ReleaseGateInput {
                release_runbook_id: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_operator_runbook_denied",
            ReleaseGateInput {
                operator_runbook: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_rollback_receipt_denied",
            ReleaseGateInput {
                rollback: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_chain_receipt_denied",
            ReleaseGateInput {
                chain: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "chain_head_mismatch_denied",
            ReleaseGateInput {
                chain: Some(ReleaseChainReceipt {
                    chain_head: "deadbeef".to_string(),
                    ..chain_receipt()
                }),
                ..full_release_input()
            },
        ),
        release_cell(
            "missing_required_commit_denied",
            ReleaseGateInput {
                chain: Some(ReleaseChainReceipt {
                    chain_head: EXPECTED_CHAIN_HEAD.to_string(),
                    lineage: vec![ReleaseCommitPin {
                        name: "score-0".to_string(),
                        commit: "e30176e".to_string(),
                    }],
                }),
                ..full_release_input()
            },
        ),
        release_cell(
            "release_check_failure_denied",
            ReleaseGateInput {
                release_check: Some(ReleaseCheckReceipt {
                    tool: "release_check.sh".to_string(),
                    green: false,
                    output_hash: "release-check-green-hash".to_string(),
                }),
                ..full_release_input()
            },
        ),
        release_cell(
            "operator_smoke_failure_denied",
            ReleaseGateInput {
                operator_smoke: Some(OperatorSmokeReceipt {
                    tool: "operator_smoke.sh".to_string(),
                    green: false,
                    output_hash: "operator-smoke-green-hash".to_string(),
                }),
                ..full_release_input()
            },
        ),
        release_cell(
            "unit_count_mismatch_denied",
            ReleaseGateInput {
                unit_count: Some(1),
                ..full_release_input()
            },
        ),
        release_cell(
            "boundary_lock_missing_denied",
            ReleaseGateInput {
                boundary: None,
                ..full_release_input()
            },
        ),
        release_cell(
            "training_detected_denied",
            ReleaseGateInput {
                training_requested: true,
                ..full_release_input()
            },
        ),
        release_cell(
            "deployment_detected_denied",
            ReleaseGateInput {
                deployment_requested: true,
                ..full_release_input()
            },
        ),
        release_cell(
            "production_traffic_detected_denied",
            ReleaseGateInput {
                production_traffic_requested: true,
                ..full_release_input()
            },
        ),
        release_cell(
            "baseline_replacement_detected_denied",
            ReleaseGateInput {
                baseline_replacement_requested: true,
                ..full_release_input()
            },
        ),
        release_cell(
            "authority_drift_denied",
            ReleaseGateInput {
                authority_drift: AuthorityDriftCheck::drifted(),
                ..full_release_input()
            },
        ),
        release_cell(
            "dirty_release_scope_denied",
            ReleaseGateInput {
                release_scope_clean: Some(false),
                ..full_release_input()
            },
        ),
        // 25. Serialized report tamper refused.
        release_tamper_cell(),
        // 26-29. Release-ready is NOT external deployment / public production / baseline / training.
        release_cell(
            "local_release_ready_not_external_deployment",
            full_release_input(),
        ),
        release_cell(
            "local_release_ready_not_public_production",
            full_release_input(),
        ),
        release_cell(
            "local_release_ready_not_baseline_replacement",
            full_release_input(),
        ),
        release_cell("local_release_ready_not_training", full_release_input()),
    ];

    let release_never_goes_public = scenarios.iter().all(|c| c.release_stays_local);
    let public_release_never_claimed = scenarios.iter().all(|c| !c.public_release_claimed);
    ReleaseMatrix {
        schema: SCHEMA,
        scenarios,
        decisions: RELEASE_DECISION_NAMES,
        refusal_reasons: RELEASE_REFUSAL_NAMES,
        release_never_goes_public,
        public_release_never_claimed,
        boundary: ReleaseGateBoundary::inert(),
    }
}

/// The release matrix serialized to canonical JSON.
pub fn release_matrix_json() -> String {
    serde_json::to_string(&release_matrix()).expect("release matrix serializes")
}

/// Re-derive the matrix and byte-compare against `candidate`. `Serialize` but never `Deserialize`.
pub fn verify_release_matrix_json(candidate: &str) -> Result<(), ReleaseGateError> {
    if candidate == release_matrix_json() {
        Ok(())
    } else {
        Err(ReleaseGateError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(report: &ReleaseGateReport, r: ReleaseRefusal) -> bool {
        report.refusals.contains(&r)
    }

    #[test]
    fn release_gate_consumes_the_real_prod_smoke_and_package() {
        // The gate re-runs the REAL PROD-SMOKE-0 smoke and PROD-0 packager (derived, not handed in).
        let report = evaluate_release_gate(&full_release_input());
        assert!(report.prod_smoke_passed);
        assert!(report.prod_runtime_packaged);
        // A receipt with a wrong smoke hash is not corroborated -> not passed.
        let bad = evaluate_release_gate(&ReleaseGateInput {
            prod_smoke: Some(ReleaseSmokeReceipt {
                smoke_passed: true,
                report_hash: "wrong-smoke-hash".to_string(),
            }),
            ..full_release_input()
        });
        assert!(has(&bad, ReleaseRefusal::ProdSmokeNotPassed));
    }

    #[test]
    fn local_release_ready_seals_a_readiness_receipt() {
        let report = evaluate_release_gate(&full_release_input());
        assert_eq!(report.decision, ReleaseDecision::LocalReleaseReady);
        assert!(report.refusals.is_empty());
        let seal = report.seal.as_ref().expect("sealed on ready");
        assert!(seal.local_release_ready);
        assert_eq!(seal.decision, "local_release_ready");
        assert_eq!(seal.chain_head, EXPECTED_CHAIN_HEAD);
        assert!(!seal.is_cloud_or_public_deployment);
        assert!(!seal.claims_public_release);
    }

    #[test]
    fn missing_release_input_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            release_request_id: None,
            ..full_release_input()
        });
        assert_eq!(report.decision, ReleaseDecision::ReleaseDenied);
        assert!(has(&report, ReleaseRefusal::MissingReleaseInput));
        assert!(report.seal.is_none());
    }

    #[test]
    fn missing_prod_smoke_report_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            prod_smoke: None,
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::MissingProdSmokeReport));
    }

    #[test]
    fn prod_smoke_not_passed_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            prod_smoke: Some(ReleaseSmokeReceipt {
                smoke_passed: false,
                report_hash: release_hash(&run_production_smoke_json(&release_smoke_run())),
            }),
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::ProdSmokeNotPassed));
    }

    #[test]
    fn missing_prod_runtime_package_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            prod_runtime_package_hash: None,
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::MissingProdRuntimePackage));
    }

    #[test]
    fn tampered_prod_runtime_package_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            prod_runtime_package_hash: Some("wrong-package-hash".to_string()),
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::ProdRuntimePackageTampered));
    }

    #[test]
    fn missing_chain_receipt_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            chain: None,
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::MissingChainReceipt));
    }

    #[test]
    fn chain_head_mismatch_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            chain: Some(ReleaseChainReceipt {
                chain_head: "deadbeef".to_string(),
                ..chain_receipt()
            }),
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::ChainHeadMismatch));
    }

    #[test]
    fn missing_required_commit_is_denied() {
        // A lineage missing PROD-SMOKE-0 (and the rest) is refused; a wrong commit is also refused.
        let short = evaluate_release_gate(&ReleaseGateInput {
            chain: Some(ReleaseChainReceipt {
                chain_head: EXPECTED_CHAIN_HEAD.to_string(),
                lineage: vec![ReleaseCommitPin {
                    name: "score-0".to_string(),
                    commit: "e30176e".to_string(),
                }],
            }),
            ..full_release_input()
        });
        assert!(has(&short, ReleaseRefusal::MissingRequiredCommit));

        let wrong = evaluate_release_gate(&ReleaseGateInput {
            chain: Some(ReleaseChainReceipt {
                chain_head: EXPECTED_CHAIN_HEAD.to_string(),
                lineage: REQUIRED_LINEAGE
                    .iter()
                    .map(|(name, commit)| ReleaseCommitPin {
                        name: name.to_string(),
                        commit: if *name == "prod-smoke-0" {
                            "badc0de".to_string()
                        } else {
                            commit.to_string()
                        },
                    })
                    .collect(),
            }),
            ..full_release_input()
        });
        assert!(has(&wrong, ReleaseRefusal::MissingRequiredCommit));
    }

    #[test]
    fn release_check_and_operator_smoke_must_be_green() {
        let rc = evaluate_release_gate(&ReleaseGateInput {
            release_check: Some(ReleaseCheckReceipt {
                tool: "release_check.sh".to_string(),
                green: false,
                output_hash: "h".to_string(),
            }),
            ..full_release_input()
        });
        assert!(has(&rc, ReleaseRefusal::ReleaseCheckFailed));
        let os = evaluate_release_gate(&ReleaseGateInput {
            operator_smoke: Some(OperatorSmokeReceipt {
                tool: "operator_smoke.sh".to_string(),
                green: false,
                output_hash: "h".to_string(),
            }),
            ..full_release_input()
        });
        assert!(has(&os, ReleaseRefusal::OperatorSmokeFailed));
    }

    #[test]
    fn unit_count_mismatch_is_denied() {
        let report = evaluate_release_gate(&ReleaseGateInput {
            unit_count: Some(1),
            ..full_release_input()
        });
        assert!(has(&report, ReleaseRefusal::UnitCountMismatch));
        // The expected count is the pinned release count.
        let ok = evaluate_release_gate(&ReleaseGateInput {
            unit_count: Some(EXPECTED_RELEASE_UNIT_COUNT),
            ..full_release_input()
        });
        assert!(!has(&ok, ReleaseRefusal::UnitCountMismatch));
    }

    #[test]
    fn missing_release_artifacts_notes_and_runbooks_are_denied() {
        let m = evaluate_release_gate(&ReleaseGateInput {
            artifact_manifest: None,
            ..full_release_input()
        });
        assert!(has(&m, ReleaseRefusal::MissingReleaseArtifactManifest));
        let n = evaluate_release_gate(&ReleaseGateInput {
            release_notes: None,
            ..full_release_input()
        });
        assert!(has(&n, ReleaseRefusal::MissingReleaseNotes));
        let rb = evaluate_release_gate(&ReleaseGateInput {
            release_runbook_id: None,
            ..full_release_input()
        });
        assert!(has(&rb, ReleaseRefusal::MissingReleaseRunbook));
        let orb = evaluate_release_gate(&ReleaseGateInput {
            operator_runbook: None,
            ..full_release_input()
        });
        assert!(has(&orb, ReleaseRefusal::MissingOperatorRunbook));
    }

    #[test]
    fn missing_rollback_and_boundary_lock_are_denied() {
        let rb = evaluate_release_gate(&ReleaseGateInput {
            rollback: None,
            ..full_release_input()
        });
        assert!(has(&rb, ReleaseRefusal::MissingRollbackReceipt));
        let unverified = evaluate_release_gate(&ReleaseGateInput {
            rollback: Some(ReleaseRollbackReceipt {
                rollback_hash: "h".to_string(),
                verified: false,
            }),
            ..full_release_input()
        });
        assert!(has(&unverified, ReleaseRefusal::MissingRollbackReceipt));
        let b = evaluate_release_gate(&ReleaseGateInput {
            boundary: None,
            ..full_release_input()
        });
        assert!(has(&b, ReleaseRefusal::BoundaryLockMissing));
    }

    #[test]
    fn training_deployment_traffic_and_baseline_are_refused() {
        let t = evaluate_release_gate(&ReleaseGateInput {
            training_requested: true,
            ..full_release_input()
        });
        assert!(has(&t, ReleaseRefusal::TrainingDetected));
        let d = evaluate_release_gate(&ReleaseGateInput {
            deployment_requested: true,
            ..full_release_input()
        });
        assert!(has(&d, ReleaseRefusal::DeploymentDetected));
        let p = evaluate_release_gate(&ReleaseGateInput {
            production_traffic_requested: true,
            ..full_release_input()
        });
        assert!(has(&p, ReleaseRefusal::ProductionTrafficDetected));
        let b = evaluate_release_gate(&ReleaseGateInput {
            baseline_replacement_requested: true,
            ..full_release_input()
        });
        assert!(has(&b, ReleaseRefusal::BaselineReplacementDetected));
    }

    #[test]
    fn authority_drift_and_dirty_scope_are_refused() {
        let drift = evaluate_release_gate(&ReleaseGateInput {
            authority_drift: AuthorityDriftCheck::drifted(),
            ..full_release_input()
        });
        assert!(has(&drift, ReleaseRefusal::AuthorityDriftDetected));
        let unchecked = evaluate_release_gate(&ReleaseGateInput {
            authority_drift: AuthorityDriftCheck::unchecked(),
            ..full_release_input()
        });
        assert!(has(&unchecked, ReleaseRefusal::AuthorityDriftDetected));
        let dirty = evaluate_release_gate(&ReleaseGateInput {
            release_scope_clean: Some(false),
            ..full_release_input()
        });
        assert!(has(&dirty, ReleaseRefusal::UntrackedReleaseScopeDirty));
    }

    #[test]
    fn local_release_ready_is_not_external_deployment_or_public_production() {
        let report = evaluate_release_gate(&full_release_input());
        assert_eq!(report.decision, ReleaseDecision::LocalReleaseReady);
        assert!(!report.deploys_externally);
        assert!(!report.starts_public_production);
        assert!(!report.serves_production_traffic);
        assert!(!report.is_cloud_or_public_deployment);
        assert!(!report.claims_public_release);
        assert!(report.boundary.all_inert());
    }

    #[test]
    fn local_release_ready_is_not_baseline_replacement_or_training() {
        let report = evaluate_release_gate(&full_release_input());
        assert!(!report.replaces_baseline);
        assert!(!report.trains);
        assert!(!report.mutates_weights);
        let seal = report.seal.as_ref().expect("sealed");
        assert!(!seal.replaces_baseline);
        assert!(!seal.trains);
    }

    #[test]
    fn p12_training_justified_remains_false_even_when_release_ready() {
        let report = evaluate_release_gate(&full_release_input());
        assert!(!report.training_justified);
        // The real P12 gate is unaffected by a release-ready decision.
        assert!(!reading_train_gate::decide(&[], &[]).training_justified);
    }

    #[test]
    fn decision_and_refusal_counts_match_enums() {
        assert_eq!(ReleaseDecision::ALL.len(), RELEASE_DECISION_COUNT);
        assert_eq!(ReleaseRefusal::ALL.len(), RELEASE_REFUSAL_COUNT);
        assert_eq!(RELEASE_DECISION_NAMES.len(), RELEASE_DECISION_COUNT);
        assert_eq!(RELEASE_REFUSAL_NAMES.len(), RELEASE_REFUSAL_COUNT);
        for (d, name) in ReleaseDecision::ALL.iter().zip(RELEASE_DECISION_NAMES) {
            assert_eq!(d.tag(), name);
        }
        for (r, name) in ReleaseRefusal::ALL.iter().zip(RELEASE_REFUSAL_NAMES) {
            assert_eq!(r.tag(), name);
        }
    }

    #[test]
    fn matrix_has_the_twenty_nine_named_scenarios() {
        let matrix = release_matrix();
        assert_eq!(matrix.scenarios.len(), RELEASE_SCENARIO_COUNT);
        for name in [
            "local_release_ready",
            "missing_release_input_denied",
            "missing_prod_smoke_report_denied",
            "prod_smoke_not_passed_denied",
            "missing_prod_runtime_package_denied",
            "prod_runtime_package_tampered_denied",
            "missing_release_artifact_manifest_denied",
            "missing_release_notes_denied",
            "missing_release_runbook_denied",
            "missing_operator_runbook_denied",
            "missing_rollback_receipt_denied",
            "missing_chain_receipt_denied",
            "chain_head_mismatch_denied",
            "missing_required_commit_denied",
            "release_check_failure_denied",
            "operator_smoke_failure_denied",
            "unit_count_mismatch_denied",
            "boundary_lock_missing_denied",
            "training_detected_denied",
            "deployment_detected_denied",
            "production_traffic_detected_denied",
            "baseline_replacement_detected_denied",
            "authority_drift_denied",
            "dirty_release_scope_denied",
            "serialized_release_report_tamper_refused",
            "local_release_ready_not_external_deployment",
            "local_release_ready_not_public_production",
            "local_release_ready_not_baseline_replacement",
            "local_release_ready_not_training",
        ] {
            assert!(
                matrix.scenario(name).is_some(),
                "scenario {name} is missing"
            );
        }
        assert!(matrix.release_never_goes_public);
        assert!(matrix.public_release_never_claimed);
        let ready = matrix.scenario("local_release_ready").expect("present");
        assert_eq!(ready.decision, "local_release_ready");
        assert!(ready.sealed);
    }

    #[test]
    fn every_matrix_cell_keeps_release_local_and_unclaimed() {
        let matrix = release_matrix();
        for cell in &matrix.scenarios {
            assert!(cell.release_stays_local, "cell {} went public", cell.name);
            assert!(
                !cell.public_release_claimed,
                "cell {} claimed public release",
                cell.name
            );
        }
        let tamper = matrix
            .scenario("serialized_release_report_tamper_refused")
            .expect("tamper cell present");
        assert!(tamper
            .refusals
            .contains(&"serialized_release_report_tamper_refused"));
    }

    #[test]
    fn report_is_deterministic_and_re_derives_refusing_tampering() {
        let canonical = evaluate_release_gate_json(&full_release_input());
        assert_eq!(canonical, evaluate_release_gate_json(&full_release_input()));
        assert!(verify_release_gate_report_json(&full_release_input(), &canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_release_gate_report_json(&full_release_input(), &tampered),
            Err(ReleaseGateError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_re_derives_refusing_tampering() {
        let canonical = release_matrix_json();
        assert!(verify_release_matrix_json(&canonical).is_ok());
        let tampered = format!("{canonical} ");
        assert_ne!(tampered, canonical);
        assert_eq!(
            verify_release_matrix_json(&tampered),
            Err(ReleaseGateError::ReplayMismatch)
        );
    }

    #[test]
    fn closed_by_default_denies_with_no_inputs() {
        let report = evaluate_release_gate(&ReleaseGateInput::closed_by_default());
        assert_eq!(report.decision, ReleaseDecision::ReleaseDenied);
        assert!(has(&report, ReleaseRefusal::MissingReleaseInput));
        assert!(has(&report, ReleaseRefusal::MissingChainReceipt));
        assert!(has(&report, ReleaseRefusal::AuthorityDriftDetected));
        assert!(report.seal.is_none());
        assert!(stays_local(&report));
    }
}
