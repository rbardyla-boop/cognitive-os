//! Approved-probe execution stub / non-execution boundary (HYP-3).
//!
//! HYP-2 records a governance decision on a probe as a [`ReviewReceipt`]. HYP-3 converts that
//! review into a deterministic [`ProbeExecutionIntent`] — and proves that NO probe execution
//! occurs. Approval is a record for a human/operator to act on LATER; it is never execution.
//!
//!   Hypothesis proposes.  Probe queue classifies.  Governance reviews.  HYP-3 records intent.
//!   Nothing executes.  Nothing becomes evidence.
//!
//! The execution disposition is machine-checkable, not prose. Only an APPROVED review yields a
//! cleared intent (one a human/operator may run later): an approval within automated scope is
//! recorded `not_executed`; an approval that required a human/governance authority is recorded
//! `requires_operator`; a rejected or deferred review yields a `blocked` intent that must never
//! run. A blocked probe can never be approved (HYP-2 enforces that), so it can never reach the
//! cleared path. A `ProbeExecutionIntent` follows the same structural quarantine as a receipt:
//! private fields, read-only accessors, derives `Serialize` but NOT `Deserialize` (the compiler
//! enforces this — see the `compile_fail` doctest), and is minted ONLY by
//! [`ProbeExecutionIntent::from_review`]. So a forged intent cannot be hand-set or deserialized
//! off the wire, and it can never become evidence. There is no probe-execution code here or in
//! the crate — the release gate's crate-wide no-process/filesystem/network scan proves it.

use serde::Serialize;

use crate::{
    fnv_str, fnv_u64, EvidenceRef, ReviewDecision, ReviewReceipt, ReviewerAuthority, FNV_OFFSET,
    FORBIDDEN_USES,
};

/// The execution disposition recorded on an intent — the MACHINE-CHECKABLE record of what may
/// happen to the probe NEXT, never prose. Every value is a non-executing state: HYP-3 runs
/// nothing, so there is deliberately no `executed` variant. Derived (output), so it does NOT
/// derive `Deserialize`; that also keeps [`ProbeExecutionIntent`] structurally non-deserializable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ExecutionStatus {
    /// The probe was approved within automated scope (a low-risk, reversible queued probe) and is
    /// recorded as NOT executed. HYP-3 ran nothing; a human/operator may pick it up later.
    #[serde(rename = "not_executed")]
    NotExecuted,
    /// The review did not clear the probe for execution (it was rejected or deferred): it must
    /// never be run.
    #[serde(rename = "blocked")]
    Blocked,
    /// The probe was approved by a human/governance authority: any execution REQUIRES a human
    /// operator (never an automated executor). A machine-checkable status, never prose.
    #[serde(rename = "requires_operator")]
    RequiresOperator,
}

impl ExecutionStatus {
    /// The disposition implied by an execution reason. Total and exhaustive (no wildcard), so a
    /// new reason variant forces an explicit mapping here (E0004) rather than silently defaulting
    /// to a cleared status.
    fn from_reason(reason: ExecutionReason) -> Self {
        match reason {
            ExecutionReason::ApprovedAutomatedScopeNotExecuted => ExecutionStatus::NotExecuted,
            ExecutionReason::ApprovedRequiresOperator => ExecutionStatus::RequiresOperator,
            ExecutionReason::RejectedNotExecutable | ExecutionReason::DeferredNotExecutable => {
                ExecutionStatus::Blocked
            }
        }
    }

    /// A machine-checkable token (never prose) for the status.
    pub fn token(self) -> &'static str {
        match self {
            ExecutionStatus::NotExecuted => "not_executed",
            ExecutionStatus::Blocked => "blocked",
            ExecutionStatus::RequiresOperator => "requires_operator",
        }
    }
}

/// Why an intent got its disposition — a machine-checkable classification, never prose. Derived
/// only (output), so it does NOT derive `Deserialize`; like [`ExecutionStatus`] this keeps the
/// intent structurally non-deserializable (a forged intent cannot be built off the wire).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ExecutionReason {
    /// Approved within automated scope (a queued probe auto-approved): recorded not_executed.
    #[serde(rename = "approved_automated_scope_not_executed")]
    ApprovedAutomatedScopeNotExecuted,
    /// Approved by a human/governance authority: execution requires a human operator.
    #[serde(rename = "approved_requires_operator")]
    ApprovedRequiresOperator,
    /// The review rejected the probe: it is not executable.
    #[serde(rename = "rejected_not_executable")]
    RejectedNotExecutable,
    /// The review deferred the probe (no clearance to execute): it is not executable.
    #[serde(rename = "deferred_not_executable")]
    DeferredNotExecutable,
}

impl ExecutionReason {
    /// Deterministic classification from the review's decision and the authority that produced it.
    /// Exhaustive, no wildcard — a cleared (non-blocked) reason requires `Approved`, so a rejected
    /// or deferred review can never derive a cleared intent.
    fn derive(decision: ReviewDecision, authority: ReviewerAuthority) -> Self {
        match decision {
            ReviewDecision::Approved => match authority {
                ReviewerAuthority::Automated => ExecutionReason::ApprovedAutomatedScopeNotExecuted,
                ReviewerAuthority::Human | ReviewerAuthority::Governance => {
                    ExecutionReason::ApprovedRequiresOperator
                }
            },
            ReviewDecision::Rejected => ExecutionReason::RejectedNotExecutable,
            ReviewDecision::Deferred => ExecutionReason::DeferredNotExecutable,
        }
    }

    /// The disposition this reason implies (lets a cross-check confirm reason and status agree).
    pub fn status(self) -> ExecutionStatus {
        ExecutionStatus::from_reason(self)
    }

    /// A machine-checkable token (never prose) for the reason.
    pub fn token(self) -> &'static str {
        match self {
            ExecutionReason::ApprovedAutomatedScopeNotExecuted => {
                "approved_automated_scope_not_executed"
            }
            ExecutionReason::ApprovedRequiresOperator => "approved_requires_operator",
            ExecutionReason::RejectedNotExecutable => "rejected_not_executable",
            ExecutionReason::DeferredNotExecutable => "deferred_not_executable",
        }
    }
}

/// What can go wrong handling an execution intent. Every failure is explicit; nothing is silently
/// coerced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionError {
    /// An intent's recomputed integrity hash does not match the stored one (tamper detection).
    IntegrityMismatch,
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::IntegrityMismatch => {
                write!(f, "execution intent integrity hash mismatch")
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

/// An execution INTENT recorded from a [`ReviewReceipt`] — the boundary that proves approval is
/// not execution. It is inert: it runs no probe, writes no probe result, holds no claim/evidence
/// authority, and cannot become evidence. Its disposition is derived from the review (a rejected
/// or deferred review yields a `blocked` intent; only an approved review yields a cleared one),
/// so a non-approved review can never produce a runnable intent.
///
/// Minted ONLY by [`ProbeExecutionIntent::from_review`]; its fields are private and read-only, and
/// it derives `Serialize` but NOT `Deserialize`. The first example records a real intent; the
/// `compile_fail` example proves an intent cannot be deserialized — so a forged intent (e.g. a
/// `not_executed` disposition for a rejected probe) cannot enter the system off the wire. If
/// either property regresses, `cargo test` fails.
///
/// ```
/// let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str(
///     r#"{"statement":"s","prior":1,"uncertainty":1,"test_cost":0,"risk":100,"reversibility":900,"evidence_inputs":[],"probe_description":"p"}"#
/// ).unwrap();
/// let packet = hypothesis_layer::propose(spec).unwrap();
/// let probe = hypothesis_layer::ProbeRequest::from_hypothesis(&packet);
/// let receipt = hypothesis_layer::ReviewReceipt::decide(
///     &probe,
///     hypothesis_layer::ReviewerAuthority::Human,
///     hypothesis_layer::ReviewDecision::Approved,
/// ).unwrap();
/// let intent = hypothesis_layer::ProbeExecutionIntent::from_review(&receipt);
/// let _id: u64 = intent.intent_id();
/// ```
///
/// ```compile_fail
/// // A ProbeExecutionIntent implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::ProbeExecutionIntent = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProbeExecutionIntent {
    // Private + read-only: an intent is built ONLY by `from_review`, which DERIVES the disposition
    // from the review — so a forged disposition (e.g. a cleared intent for a rejected probe)
    // cannot exist.
    intent_id: u64,
    review_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    execution_status: ExecutionStatus,
    reason_code: ExecutionReason,
    evidence_refs: Vec<EvidenceRef>,
    created_from_review_trace: bool,
    integrity_hash: u64,
}

impl ProbeExecutionIntent {
    /// Record an execution intent from a governance [`ReviewReceipt`]. The disposition is DERIVED
    /// from the review's decision and the authority that produced it — only an approved review
    /// yields a cleared (`not_executed` / `requires_operator`) intent; a rejected or deferred
    /// review yields a `blocked` one. A blocked probe can never carry an approved receipt (HYP-2
    /// refuses to approve it), so it can never reach the cleared path. Pure and deterministic;
    /// EXECUTES NOTHING — it records an intent for a human/operator to act on later.
    pub fn from_review(receipt: &ReviewReceipt) -> ProbeExecutionIntent {
        let reason_code = ExecutionReason::derive(receipt.decision(), receipt.reviewer_authority());
        let execution_status = ExecutionStatus::from_reason(reason_code);
        let review_id = receipt.review_id();
        let probe_id = receipt.probe_id();
        let hypothesis_id = receipt.hypothesis_id();
        let evidence_refs = receipt.evidence_refs().to_vec();
        let created_from_review_trace = receipt.created_from_queue_trace();
        let intent_id = derive_intent_id(
            review_id,
            probe_id,
            hypothesis_id,
            execution_status,
            reason_code,
        );
        // Build with a placeholder hash, then bind the integrity hash over the finished fields —
        // ONE hashing path (`compute_integrity`) is shared by `from_review` and `verify_integrity`.
        let base = ProbeExecutionIntent {
            intent_id,
            review_id,
            probe_id,
            hypothesis_id,
            execution_status,
            reason_code,
            evidence_refs,
            created_from_review_trace,
            integrity_hash: 0,
        };
        ProbeExecutionIntent {
            integrity_hash: base.compute_integrity(),
            ..base
        }
    }

    /// Deterministic integrity hash over every field EXCEPT `integrity_hash` itself (length-prefixed
    /// strings so distinct intents cannot collide by re-grouping bytes).
    fn compute_integrity(&self) -> u64 {
        let mut h = FNV_OFFSET;
        h = fnv_u64(h, self.intent_id);
        h = fnv_u64(h, self.review_id);
        h = fnv_u64(h, self.probe_id);
        h = fnv_u64(h, self.hypothesis_id);
        h = fnv_str(h, self.execution_status.token());
        h = fnv_str(h, self.reason_code.token());
        h = fnv_u64(h, self.evidence_refs.len() as u64);
        for ev in &self.evidence_refs {
            h = fnv_u64(h, ev.answer_hash);
            h = fnv_u64(h, ev.memory_hash);
            h = fnv_str(h, &ev.source_label);
        }
        h = fnv_u64(h, self.created_from_review_trace as u64);
        h
    }

    /// Deterministic content id of the execution intent.
    pub fn intent_id(&self) -> u64 {
        self.intent_id
    }

    /// The id of the review this intent was recorded from (provenance).
    pub fn review_id(&self) -> u64 {
        self.review_id
    }

    /// The id of the reviewed probe (provenance).
    pub fn probe_id(&self) -> u64 {
        self.probe_id
    }

    /// The id of the originating hypothesis (provenance).
    pub fn hypothesis_id(&self) -> u64 {
        self.hypothesis_id
    }

    /// The machine-checkable execution disposition (read-only — it cannot be flipped to a cleared
    /// status after the fact).
    pub fn execution_status(&self) -> ExecutionStatus {
        self.execution_status
    }

    /// The machine-checkable reason for the disposition.
    pub fn reason_code(&self) -> ExecutionReason {
        self.reason_code
    }

    /// The receipts the originating hypothesis cited (carried through as provenance, never as
    /// evidence the intent itself produces).
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// Whether the originating hypothesis was derived from a trace/receipt (carried through the
    /// hypothesis -> probe -> review -> intent chain).
    pub fn created_from_review_trace(&self) -> bool {
        self.created_from_review_trace
    }

    /// The deterministic integrity hash binding every field of this intent.
    pub fn integrity_hash(&self) -> u64 {
        self.integrity_hash
    }

    /// Whether this intent is BLOCKED from execution (rejected or deferred review). A blocked
    /// intent must never be run.
    pub fn is_blocked(&self) -> bool {
        self.execution_status == ExecutionStatus::Blocked
    }

    /// Whether execution of this (approved) intent REQUIRES a human operator. True only for an
    /// approval that needed a human/governance authority — never for a blocked intent. HYP-3 still
    /// runs nothing; this records who may run it later.
    pub fn requires_operator(&self) -> bool {
        self.execution_status == ExecutionStatus::RequiresOperator
    }

    /// Whether this intent may be used for the given purpose. Always `false` for any forbidden
    /// use: an execution intent is never truth, evidence, or a mutator. It inherits the canonical
    /// [`FORBIDDEN_USES`] quarantine, so it can never become a claim or ground an answer.
    pub fn permits(&self, use_name: &str) -> bool {
        !FORBIDDEN_USES.contains(&use_name)
    }

    /// Re-derive the integrity hash from this intent's OWN fields and confirm it matches. Because
    /// an intent is born only from [`from_review`] (private fields, no `Deserialize`), it is
    /// consistent by construction; this is an explicit, auditable assertion of that binding — used
    /// to prove a replay was faithful. It grants no authority and runs no probe.
    pub fn verify_integrity(&self) -> Result<(), ExecutionError> {
        if self.compute_integrity() == self.integrity_hash {
            Ok(())
        } else {
            Err(ExecutionError::IntegrityMismatch)
        }
    }
}

/// Deterministic id of the execution intent (FNV-1a over its defining fields, length-prefixed).
fn derive_intent_id(
    review_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    execution_status: ExecutionStatus,
    reason_code: ExecutionReason,
) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_u64(h, review_id);
    h = fnv_u64(h, probe_id);
    h = fnv_u64(h, hypothesis_id);
    h = fnv_str(h, execution_status.token());
    h = fnv_str(h, reason_code.token());
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{propose, HypothesisSpec, ProbeRequest, ReviewError};

    fn ev(label: &str) -> EvidenceRef {
        EvidenceRef {
            answer_hash: 0x1111_2222_3333_4444,
            memory_hash: 0x5555_6666_7777_8888,
            source_label: label.to_string(),
        }
    }

    fn probe_with(risk: i64, reversibility: i64, statement: &str) -> ProbeRequest {
        let spec = HypothesisSpec {
            statement: statement.to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk,
            reversibility,
            evidence_inputs: vec![ev("run.json")],
            probe_description: "Re-read the maintenance log span.".to_string(),
        };
        ProbeRequest::from_hypothesis(&propose(spec).unwrap())
    }

    fn queued() -> ProbeRequest {
        probe_with(100, 900, "queued probe")
    }

    fn review_required() -> ProbeRequest {
        probe_with(800, 800, "high-risk probe")
    }

    fn blocked() -> ProbeRequest {
        probe_with(900, 100, "dangerous irreversible probe")
    }

    fn decide(
        probe: &ProbeRequest,
        authority: ReviewerAuthority,
        decision: ReviewDecision,
    ) -> ReviewReceipt {
        ReviewReceipt::decide(probe, authority, decision).unwrap()
    }

    #[test]
    fn intent_derived_only_from_approved_review() {
        // An intent is recorded from a review and cites the review_id, probe_id, hypothesis_id, and
        // source EvidenceRefs. Only an APPROVED review yields a cleared (non-blocked) intent; a
        // rejected or deferred review yields a `blocked` one.
        let probe = queued();
        let receipt = decide(&probe, ReviewerAuthority::Human, ReviewDecision::Approved);
        let intent = ProbeExecutionIntent::from_review(&receipt);
        assert_eq!(intent.review_id(), receipt.review_id());
        assert_eq!(intent.probe_id(), receipt.probe_id());
        assert_eq!(intent.hypothesis_id(), receipt.hypothesis_id());
        assert_eq!(intent.evidence_refs(), receipt.evidence_refs());
        assert_ne!(intent.intent_id(), 0);
        intent.verify_integrity().unwrap();
        // A cleared (non-blocked) disposition arises ONLY from an approved review.
        assert!(!intent.is_blocked());
        for decision in [ReviewDecision::Rejected, ReviewDecision::Deferred] {
            let r = decide(&queued(), ReviewerAuthority::Governance, decision);
            assert!(
                ProbeExecutionIntent::from_review(&r).is_blocked(),
                "a non-approved review can never yield a cleared intent"
            );
        }
    }

    #[test]
    fn rejected_review_cannot_create_execution_intent() {
        // A rejected review yields a `blocked`, non-executable intent (never a cleared one).
        let receipt = decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Rejected,
        );
        let intent = ProbeExecutionIntent::from_review(&receipt);
        assert_eq!(intent.execution_status(), ExecutionStatus::Blocked);
        assert_eq!(intent.reason_code(), ExecutionReason::RejectedNotExecutable);
        assert!(intent.is_blocked());
        assert!(!intent.requires_operator());
    }

    #[test]
    fn deferred_review_cannot_create_execution_intent() {
        // A deferred review yields a `blocked`, non-executable intent (no clearance to run).
        let receipt = decide(
            &queued(),
            ReviewerAuthority::Automated,
            ReviewDecision::Deferred,
        );
        let intent = ProbeExecutionIntent::from_review(&receipt);
        assert_eq!(intent.execution_status(), ExecutionStatus::Blocked);
        assert_eq!(intent.reason_code(), ExecutionReason::DeferredNotExecutable);
        assert!(intent.is_blocked());
    }

    #[test]
    fn execution_intent_is_not_executed() {
        // An approval RECORDS an intent; it executes nothing. There is no `executed` status — the
        // disposition is always one of the three non-running states. An automated-scope approval is
        // recorded `not_executed`; a human/governance approval is `requires_operator` (a human must
        // run it later). Neither runs a probe.
        let auto = ProbeExecutionIntent::from_review(&decide(
            &queued(),
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
        ));
        assert_eq!(auto.execution_status(), ExecutionStatus::NotExecuted);
        assert_eq!(
            auto.reason_code(),
            ExecutionReason::ApprovedAutomatedScopeNotExecuted
        );
        assert!(!auto.is_blocked());
        assert!(!auto.requires_operator());

        let human = ProbeExecutionIntent::from_review(&decide(
            &review_required(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        ));
        assert_eq!(human.execution_status(), ExecutionStatus::RequiresOperator);
        assert_eq!(
            human.reason_code(),
            ExecutionReason::ApprovedRequiresOperator
        );
        assert!(human.requires_operator());

        // Every disposition is a non-executing state — none is an "executed" token.
        for status in [
            ExecutionStatus::NotExecuted,
            ExecutionStatus::Blocked,
            ExecutionStatus::RequiresOperator,
        ] {
            assert_ne!(status.token(), "executed");
        }
    }

    #[test]
    fn blocked_probe_never_reaches_cleared_intent() {
        // A blocked probe can never be approved (HYP-2 refuses), so no approved receipt — and thus
        // no cleared intent — can exist for it. Its only dispositions (rejected/deferred) are
        // blocked intents.
        for authority in [
            ReviewerAuthority::Automated,
            ReviewerAuthority::Human,
            ReviewerAuthority::Governance,
        ] {
            assert_eq!(
                ReviewReceipt::decide(&blocked(), authority, ReviewDecision::Approved),
                Err(ReviewError::BlockedCannotBeApproved),
                "a blocked probe can never produce an approved receipt"
            );
        }
        // The reachable dispositions of a blocked probe are blocked intents.
        let rejected = decide(
            &blocked(),
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
        );
        assert!(ProbeExecutionIntent::from_review(&rejected).is_blocked());
    }

    #[test]
    fn forged_intent_cannot_be_constructed() {
        // The disposition cannot be bypassed by building a raw struct: fields are private, there is
        // no public constructor or setter, and there is no Deserialize (the compile_fail doctest
        // proves it). The ONLY way to an intent is from_review, which always DERIVES the
        // disposition from the review — so a rejected/deferred review can never carry a cleared
        // status. (A second, independent covering test of the policy, so disabling any one test
        // cannot silently open the cleared-from-non-approved hole.)
        for decision in [ReviewDecision::Rejected, ReviewDecision::Deferred] {
            let intent = ProbeExecutionIntent::from_review(&decide(
                &queued(),
                ReviewerAuthority::Governance,
                decision,
            ));
            assert_eq!(intent.execution_status(), ExecutionStatus::Blocked);
            assert!(intent.is_blocked());
            assert!(!intent.requires_operator());
        }
    }

    #[test]
    fn intent_status_and_reason_agree() {
        // The status and the reason are derived consistently for every decision/authority: the
        // reason's implied status always equals the intent's status. If the two classifiers
        // diverged, this fails.
        let cases = [
            (
                ReviewerAuthority::Automated,
                ReviewDecision::Approved,
                ExecutionStatus::NotExecuted,
            ),
            (
                ReviewerAuthority::Human,
                ReviewDecision::Approved,
                ExecutionStatus::RequiresOperator,
            ),
            (
                ReviewerAuthority::Governance,
                ReviewDecision::Approved,
                ExecutionStatus::RequiresOperator,
            ),
            (
                ReviewerAuthority::Human,
                ReviewDecision::Rejected,
                ExecutionStatus::Blocked,
            ),
            (
                ReviewerAuthority::Automated,
                ReviewDecision::Deferred,
                ExecutionStatus::Blocked,
            ),
        ];
        for (authority, decision, want) in cases {
            // Use a queued probe so every authority can legitimately approve it.
            let intent = ProbeExecutionIntent::from_review(&decide(&queued(), authority, decision));
            assert_eq!(intent.execution_status(), want);
            assert_eq!(intent.reason_code().status(), intent.execution_status());
        }
    }

    #[test]
    fn intent_hash_is_deterministic() {
        // Deterministic: identical reviews yield identical intent, intent_id, and integrity_hash.
        let receipt = decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        );
        let a = ProbeExecutionIntent::from_review(&receipt);
        let b = ProbeExecutionIntent::from_review(&receipt);
        assert_eq!(a, b);
        assert_eq!(a.intent_id(), b.intent_id());
        assert_eq!(a.integrity_hash(), b.integrity_hash());
        // A different decision changes the disposition, the id, and the hash.
        let rejected = decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Rejected,
        );
        let c = ProbeExecutionIntent::from_review(&rejected);
        assert_ne!(a.intent_id(), c.intent_id());
        assert_ne!(a.integrity_hash(), c.integrity_hash());
        assert_ne!(a.execution_status(), c.execution_status());
    }

    #[test]
    fn intent_replay_reproduces_same_record() {
        // A trace is the INPUTS: the hypothesis spec plus the reviewer authority and decision (all
        // deserializable). Replay re-derives the identical intent by re-running propose -> decide
        // -> from_review; the intent itself serializes (to emit a trace) but is never deserialized.
        let spec = HypothesisSpec {
            statement: "replayable intent".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![ev("run.json")],
            probe_description: "probe".to_string(),
        };
        let authority_json = "\"governance\"";
        let decision_json = "\"approved\"";
        let build = |spec: &HypothesisSpec, a: ReviewerAuthority, d: ReviewDecision| {
            let probe = ProbeRequest::from_hypothesis(&propose(spec.clone()).unwrap());
            ProbeExecutionIntent::from_review(&ReviewReceipt::decide(&probe, a, d).unwrap())
        };
        let authority: ReviewerAuthority = serde_json::from_str(authority_json).unwrap();
        let decision: ReviewDecision = serde_json::from_str(decision_json).unwrap();
        let original = build(&spec, authority, decision);

        // Round-trip the trace inputs and rebuild.
        let spec2: HypothesisSpec =
            serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
        let authority2: ReviewerAuthority = serde_json::from_str(authority_json).unwrap();
        let decision2: ReviewDecision = serde_json::from_str(decision_json).unwrap();
        let replayed = build(&spec2, authority2, decision2);

        assert_eq!(original, replayed, "replay reproduces the intent");
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replayed).unwrap()
        );
        replayed.verify_integrity().unwrap();
    }

    #[test]
    fn intent_cannot_be_evidence() {
        // An intent inherits the canonical forbidden-uses quarantine: it can never ground a claim
        // or serve as evidence. (Structurally there is also no API turning an intent into a
        // Claim/EvidenceRef/ProofObject, and the production crate has no verifier dependency.)
        let intent = ProbeExecutionIntent::from_review(&decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        ));
        for canonical in [
            "ground_claim",
            "serve_as_evidence",
            "mutate_reading_memory",
            "alter_verifier_receipt",
            "change_training_gate",
            "bypass_codec_or_governance",
        ] {
            assert!(!intent.permits(canonical), "{canonical} must be forbidden");
        }
        assert!(intent.permits("record_execution_intent"));
    }

    #[test]
    fn intent_does_not_change_training_gate() {
        // Recording an execution intent is orthogonal to P12: the training decision before and
        // after is identical — still training_not_justified.
        let before = reading_train_gate::decide(&[], &[]);
        let _intent = ProbeExecutionIntent::from_review(&decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        ));
        let after = reading_train_gate::decide(&[], &[]);
        assert_eq!(before, after);
        assert!(
            !after.training_justified,
            "an execution intent cannot change the training verdict"
        );
    }

    #[test]
    fn intent_does_not_change_verifier_receipt() {
        // Recording an intent from a review whose hypothesis cites a real receipt leaves the
        // verifier receipt byte-identical — the layer reads hashes, never the object, and executes
        // nothing.
        let docs = vec![(
            "report.txt".to_string(),
            "Bridge A was damaged. Bridge B stayed open.".to_string(),
        )];
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":1},
            {"action":"extract_claim","statement":"Bridge B stayed open.","source_span_ids":[1]},
            {"action":"synthesize","answer_text":"Bridge B stayed open.","supporting_claims":[0]}
        ]"#;
        let file = reading_cli::produce_run(&docs, "Which bridge is open?", plan).unwrap();
        let before = reading_cli::verify_file(&file).unwrap();

        let cite = EvidenceRef {
            answer_hash: file.answer_hash,
            memory_hash: file.memory_hash,
            source_label: "bridge-run".to_string(),
        };
        let spec = HypothesisSpec {
            statement: "Bridge B reopened.".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![cite.clone()],
            probe_description: "probe".to_string(),
        };
        let probe = ProbeRequest::from_hypothesis(&propose(spec).unwrap());
        let receipt = decide(
            &probe,
            ReviewerAuthority::Governance,
            ReviewDecision::Approved,
        );
        let intent = ProbeExecutionIntent::from_review(&receipt);
        assert_eq!(intent.evidence_refs(), vec![cite].as_slice());

        let after = reading_cli::verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn execution_tokens_are_machine_checkable() {
        assert_eq!(ExecutionStatus::NotExecuted.token(), "not_executed");
        assert_eq!(ExecutionStatus::Blocked.token(), "blocked");
        assert_eq!(
            ExecutionStatus::RequiresOperator.token(),
            "requires_operator"
        );
        assert_eq!(
            ExecutionReason::ApprovedAutomatedScopeNotExecuted.token(),
            "approved_automated_scope_not_executed"
        );
        assert_eq!(
            ExecutionReason::ApprovedRequiresOperator.token(),
            "approved_requires_operator"
        );
        assert_eq!(
            ExecutionReason::RejectedNotExecutable.token(),
            "rejected_not_executable"
        );
        assert_eq!(
            ExecutionReason::DeferredNotExecutable.token(),
            "deferred_not_executable"
        );
        assert_eq!(
            ExecutionError::IntegrityMismatch.to_string(),
            "execution intent integrity hash mismatch"
        );
    }
}
