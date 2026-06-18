//! Governance review receipt boundary (HYP-2).
//!
//! HYP-1 produces an inert [`ProbeRequest`] queue item with a review status. HYP-2 records the
//! GOVERNANCE DECISION on that request as a deterministic [`ReviewReceipt`] — approved, rejected,
//! or deferred — WITHOUT executing the probe or mutating anything.
//!
//!   Hypothesis proposes.  Probe queue classifies.  Governance reviews.  Nothing executes.
//!   Nothing becomes evidence.
//!
//! The policy is machine-checkable, not prose: a `blocked` probe can never be approved by ANY
//! authority; a `human_review_required` probe can be approved only by a human/governance authority
//! (never automated); a `queued` probe may be approved or rejected — but approval still executes
//! nothing. A `ReviewReceipt` follows the same structural quarantine as a packet/request: private
//! fields, read-only accessors, derives `Serialize` but NOT `Deserialize` (the compiler enforces
//! this — see the `compile_fail` doctests), and is minted ONLY by [`ReviewReceipt::decide`]. So a
//! forged decision cannot be hand-set or deserialized off the wire, and it can never become evidence.

use serde::{Deserialize, Serialize};

use crate::{fnv_str, fnv_u64, EvidenceRef, ProbeRequest, ProbeStatus, FNV_OFFSET, FORBIDDEN_USES};

/// The governance decision recorded on a review receipt. Also the reviewer's REQUESTED decision
/// passed into [`ReviewReceipt::decide`] — a deserializable input value. Machine-checkable, never
/// prose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewDecision {
    /// The probe is cleared (by policy + the reviewer's authority) for a human/governance to run
    /// LATER. Approval is not execution — this layer runs nothing.
    #[serde(rename = "approved")]
    Approved,
    /// The probe is declined.
    #[serde(rename = "rejected")]
    Rejected,
    /// The decision is postponed (e.g. an automated reviewer punts a review-required probe upward).
    #[serde(rename = "deferred")]
    Deferred,
}

impl ReviewDecision {
    /// A machine-checkable token (never prose).
    pub fn token(self) -> &'static str {
        match self {
            ReviewDecision::Approved => "approved",
            ReviewDecision::Rejected => "rejected",
            ReviewDecision::Deferred => "deferred",
        }
    }
}

/// The authority/scope of the reviewer. A CHECKED ENUM, never a free string — so "who may approve
/// what" is a machine-checkable scope, not unverifiable prose. A deserializable input value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewerAuthority {
    /// A deterministic automated reviewer: may approve only `queued` (low-risk, reversible) probes.
    #[serde(rename = "automated")]
    Automated,
    /// A human reviewer: may also approve `human_review_required` probes.
    #[serde(rename = "human")]
    Human,
    /// A governance authority: may also approve `human_review_required` probes.
    #[serde(rename = "governance")]
    Governance,
}

impl ReviewerAuthority {
    /// Whether this authority may APPROVE a `human_review_required` probe. Automated may not; a
    /// human or governance authority may. Exhaustive, no wildcard — a new authority variant forces
    /// an explicit scope decision (E0004) rather than silently gaining approval power.
    pub fn can_approve_review_required(self) -> bool {
        match self {
            ReviewerAuthority::Automated => false,
            ReviewerAuthority::Human => true,
            ReviewerAuthority::Governance => true,
        }
    }

    /// A machine-checkable token (never prose).
    pub fn token(self) -> &'static str {
        match self {
            ReviewerAuthority::Automated => "automated",
            ReviewerAuthority::Human => "human",
            ReviewerAuthority::Governance => "governance",
        }
    }
}

/// Why a review got its decision — a machine-checkable classification, never prose. Derived only
/// (output), so it does NOT derive `Deserialize`; this also keeps [`ReviewReceipt`] structurally
/// non-deserializable (a receipt cannot derive `Deserialize` while this field cannot).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ReasonCode {
    /// Approved by an automated reviewer within its scope (a queued probe).
    #[serde(rename = "approved_within_automated_scope")]
    ApprovedWithinAutomatedScope,
    /// Approved by a human/governance authority.
    #[serde(rename = "approved_by_reviewer_authority")]
    ApprovedByReviewerAuthority,
    /// Rejected because the probe was blocked (high-risk AND irreversible) — its only safe
    /// dispositions are rejected or deferred.
    #[serde(rename = "rejected_blocked_probe")]
    RejectedBlockedProbe,
    /// Rejected at the reviewer's discretion.
    #[serde(rename = "rejected_by_reviewer")]
    RejectedByReviewer,
    /// Deferred for (further) review.
    #[serde(rename = "deferred_for_review")]
    DeferredForReview,
}

impl ReasonCode {
    /// Deterministic classification from the probe status, reviewer authority, and decision.
    /// Exhaustive, no wildcard.
    fn derive(status: ProbeStatus, authority: ReviewerAuthority, decision: ReviewDecision) -> Self {
        match decision {
            ReviewDecision::Approved => match authority {
                ReviewerAuthority::Automated => ReasonCode::ApprovedWithinAutomatedScope,
                ReviewerAuthority::Human | ReviewerAuthority::Governance => {
                    ReasonCode::ApprovedByReviewerAuthority
                }
            },
            ReviewDecision::Rejected => match status {
                ProbeStatus::Blocked => ReasonCode::RejectedBlockedProbe,
                ProbeStatus::Queued | ProbeStatus::HumanReviewRequired => {
                    ReasonCode::RejectedByReviewer
                }
            },
            ReviewDecision::Deferred => ReasonCode::DeferredForReview,
        }
    }

    /// A machine-checkable token (never prose).
    pub fn token(self) -> &'static str {
        match self {
            ReasonCode::ApprovedWithinAutomatedScope => "approved_within_automated_scope",
            ReasonCode::ApprovedByReviewerAuthority => "approved_by_reviewer_authority",
            ReasonCode::RejectedBlockedProbe => "rejected_blocked_probe",
            ReasonCode::RejectedByReviewer => "rejected_by_reviewer",
            ReasonCode::DeferredForReview => "deferred_for_review",
        }
    }
}

/// What can go wrong recording a review. Every failure is explicit; nothing is silently coerced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewError {
    /// An attempt to APPROVE a `blocked` (high-risk AND irreversible) probe. A blocked probe can
    /// never be approved by any authority.
    BlockedCannotBeApproved,
    /// An attempt to APPROVE a `human_review_required` probe with insufficient (automated)
    /// authority.
    AuthorityInsufficient,
    /// A receipt's recomputed integrity hash does not match the stored one (tamper detection).
    IntegrityMismatch,
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReviewError::BlockedCannotBeApproved => {
                write!(f, "a blocked probe can never be approved")
            }
            ReviewError::AuthorityInsufficient => {
                write!(
                    f,
                    "approving a review-required probe requires human/governance authority"
                )
            }
            ReviewError::IntegrityMismatch => write!(f, "review receipt integrity hash mismatch"),
        }
    }
}

impl std::error::Error for ReviewError {}

/// A governance decision recorded on a [`ProbeRequest`] — approved, rejected, or deferred. It is
/// inert: it executes nothing, holds no claim/evidence authority, and cannot become evidence. The
/// decision is policy-checked (a blocked probe is never approved; a review-required probe needs
/// human/governance authority).
///
/// Minted ONLY by [`ReviewReceipt::decide`]; its fields are private and read-only, and it derives
/// `Serialize` but NOT `Deserialize`. The first example records a real decision; the `compile_fail`
/// example proves a receipt cannot be deserialized — so a forged "approved" receipt for a blocked
/// probe cannot enter the system off the wire. If either property regresses, `cargo test` fails.
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
/// let _id: u64 = receipt.review_id();
/// ```
///
/// ```compile_fail
/// // A ReviewReceipt implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::ReviewReceipt = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReviewReceipt {
    // Private + read-only: a receipt is built ONLY by `decide`, which enforces the policy, so a
    // forged decision (e.g. an approved blocked probe) cannot exist.
    review_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    decision: ReviewDecision,
    reviewer_authority: ReviewerAuthority,
    reason_code: ReasonCode,
    evidence_refs: Vec<EvidenceRef>,
    created_from_queue_trace: bool,
    integrity_hash: u64,
}

impl ReviewReceipt {
    /// Record a governance decision on a (necessarily valid) [`ProbeRequest`]. The only way to a
    /// `&ProbeRequest` is HYP-1's `from_hypothesis`, so a receipt is, by construction, derived only
    /// from a valid probe. Policy is enforced here: APPROVING a `blocked` probe is rejected for any
    /// authority; APPROVING a `human_review_required` probe requires human/governance authority.
    /// Rejecting or deferring is always permitted. Pure and deterministic; executes nothing.
    pub fn decide(
        probe: &ProbeRequest,
        authority: ReviewerAuthority,
        decision: ReviewDecision,
    ) -> Result<ReviewReceipt, ReviewError> {
        let status = probe.status();
        if decision == ReviewDecision::Approved {
            // Exhaustive, no wildcard: the approval policy is decided per status explicitly.
            match status {
                ProbeStatus::Blocked => return Err(ReviewError::BlockedCannotBeApproved),
                ProbeStatus::HumanReviewRequired => {
                    if !authority.can_approve_review_required() {
                        return Err(ReviewError::AuthorityInsufficient);
                    }
                }
                ProbeStatus::Queued => {}
            }
        }
        let reason_code = ReasonCode::derive(status, authority, decision);
        let evidence_refs = probe.evidence_refs().to_vec();
        let created_from_queue_trace = probe.created_from_trace();
        let probe_id = probe.probe_id();
        let hypothesis_id = probe.hypothesis_id();
        let review_id = derive_review_id(probe_id, hypothesis_id, decision, authority);
        // Build with a placeholder hash, then bind the integrity hash over the finished fields —
        // ONE hashing path (`compute_integrity`) is shared by `decide` and `verify_integrity`.
        let base = ReviewReceipt {
            review_id,
            probe_id,
            hypothesis_id,
            decision,
            reviewer_authority: authority,
            reason_code,
            evidence_refs,
            created_from_queue_trace,
            integrity_hash: 0,
        };
        Ok(ReviewReceipt {
            integrity_hash: base.compute_integrity(),
            ..base
        })
    }

    /// Deterministic integrity hash over every field EXCEPT `integrity_hash` itself (length-prefixed
    /// strings so distinct receipts cannot collide by re-grouping bytes).
    fn compute_integrity(&self) -> u64 {
        let mut h = FNV_OFFSET;
        h = fnv_u64(h, self.review_id);
        h = fnv_u64(h, self.probe_id);
        h = fnv_u64(h, self.hypothesis_id);
        h = fnv_str(h, self.decision.token());
        h = fnv_str(h, self.reviewer_authority.token());
        h = fnv_str(h, self.reason_code.token());
        h = fnv_u64(h, self.evidence_refs.len() as u64);
        for ev in &self.evidence_refs {
            h = fnv_u64(h, ev.answer_hash);
            h = fnv_u64(h, ev.memory_hash);
            h = fnv_str(h, &ev.source_label);
        }
        h = fnv_u64(h, self.created_from_queue_trace as u64);
        h
    }

    /// Deterministic content id of the review decision.
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

    /// The recorded decision (read-only — it cannot be flipped to approved after the fact).
    pub fn decision(&self) -> ReviewDecision {
        self.decision
    }

    /// The reviewer authority that produced the decision.
    pub fn reviewer_authority(&self) -> ReviewerAuthority {
        self.reviewer_authority
    }

    /// The machine-checkable reason for the decision.
    pub fn reason_code(&self) -> ReasonCode {
        self.reason_code
    }

    /// The receipts the originating hypothesis cited (carried through as provenance, never as
    /// evidence the receipt itself produces).
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// Whether the originating hypothesis was derived from a trace/receipt.
    pub fn created_from_queue_trace(&self) -> bool {
        self.created_from_queue_trace
    }

    /// The deterministic integrity hash binding every field of this receipt.
    pub fn integrity_hash(&self) -> u64 {
        self.integrity_hash
    }

    /// Whether this receipt may be used for the given purpose. Always `false` for any forbidden
    /// use: a review receipt is never truth, evidence, or a mutator. It inherits the canonical
    /// [`FORBIDDEN_USES`] quarantine, so it can never become a claim or ground an answer.
    pub fn permits(&self, use_name: &str) -> bool {
        !FORBIDDEN_USES.contains(&use_name)
    }

    /// Re-derive the integrity hash from this receipt's OWN fields and confirm it matches. Because a
    /// receipt is born only from [`decide`] (private fields, no `Deserialize`), it is consistent by
    /// construction; this is an explicit, auditable assertion of that binding — used to prove a
    /// replay was faithful. It grants no authority.
    pub fn verify_integrity(&self) -> Result<(), ReviewError> {
        if self.compute_integrity() == self.integrity_hash {
            Ok(())
        } else {
            Err(ReviewError::IntegrityMismatch)
        }
    }
}

/// A deterministic, content-ordered log of review receipts. The order is canonical — by
/// `review_id`, then `probe_id` — so it is INSERTION-ORDER INDEPENDENT and reproduces exactly on
/// replay. The log is an audit record; it executes nothing.
///
/// Like a receipt, a log is minted only by [`ReviewLog::from_receipts`], has a private field, and
/// derives `Serialize` but NOT `Deserialize`.
///
/// ```
/// let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str(
///     r#"{"statement":"s","prior":1,"uncertainty":1,"test_cost":0,"risk":100,"reversibility":900,"evidence_inputs":[],"probe_description":"p"}"#
/// ).unwrap();
/// let packet = hypothesis_layer::propose(spec).unwrap();
/// let probe = hypothesis_layer::ProbeRequest::from_hypothesis(&packet);
/// let receipt = hypothesis_layer::ReviewReceipt::decide(
///     &probe,
///     hypothesis_layer::ReviewerAuthority::Automated,
///     hypothesis_layer::ReviewDecision::Approved,
/// ).unwrap();
/// let log = hypothesis_layer::ReviewLog::from_receipts(vec![receipt]);
/// let _n = log.receipts().len();
/// ```
///
/// ```compile_fail
/// // A ReviewLog implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::ReviewLog = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ReviewLog {
    receipts: Vec<ReviewReceipt>,
}

impl ReviewLog {
    /// Order a set of review receipts canonically. Pure and deterministic: the same receipts in any
    /// input order yield the identical log.
    pub fn from_receipts(receipts: Vec<ReviewReceipt>) -> ReviewLog {
        let mut receipts = receipts;
        receipts.sort_by(|a, b| {
            a.review_id
                .cmp(&b.review_id)
                .then(a.probe_id.cmp(&b.probe_id))
        });
        ReviewLog { receipts }
    }

    /// The log in canonical order.
    pub fn receipts(&self) -> &[ReviewReceipt] {
        &self.receipts
    }

    /// The approved receipts. Approval is a governance disposition for a human to execute LATER —
    /// this returns records, it runs nothing.
    pub fn approved(&self) -> Vec<&ReviewReceipt> {
        self.receipts
            .iter()
            .filter(|r| r.decision() == ReviewDecision::Approved)
            .collect()
    }
}

/// Deterministic id of the review decision (FNV-1a over its defining inputs, length-prefixed).
fn derive_review_id(
    probe_id: u64,
    hypothesis_id: u64,
    decision: ReviewDecision,
    authority: ReviewerAuthority,
) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_u64(h, probe_id);
    h = fnv_u64(h, hypothesis_id);
    h = fnv_str(h, decision.token());
    h = fnv_str(h, authority.token());
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{propose, HypothesisSpec};

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

    #[test]
    fn review_receipt_derived_from_probe_request() {
        // A receipt is derived from a valid probe and cites the probe_id, hypothesis_id, and the
        // source EvidenceRefs. (A ProbeRequest only exists via from_hypothesis on a validated
        // packet, so a receipt is always derived from a valid probe.)
        let probe = queued();
        let receipt = ReviewReceipt::decide(
            &probe,
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
        )
        .unwrap();
        assert_eq!(receipt.probe_id(), probe.probe_id());
        assert_eq!(receipt.hypothesis_id(), probe.hypothesis_id());
        assert_eq!(receipt.evidence_refs(), probe.evidence_refs());
        assert_eq!(
            receipt.created_from_queue_trace(),
            probe.created_from_trace()
        );
        assert_ne!(receipt.review_id(), 0);
        receipt.verify_integrity().unwrap();
    }

    #[test]
    fn blocked_probe_cannot_be_approved() {
        // No authority — not even governance — can approve a blocked probe.
        for authority in [
            ReviewerAuthority::Automated,
            ReviewerAuthority::Human,
            ReviewerAuthority::Governance,
        ] {
            assert_eq!(
                ReviewReceipt::decide(&blocked(), authority, ReviewDecision::Approved),
                Err(ReviewError::BlockedCannotBeApproved)
            );
        }
        // A blocked probe CAN be rejected or deferred (its safe dispositions).
        let rejected = ReviewReceipt::decide(
            &blocked(),
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
        )
        .unwrap();
        assert_eq!(rejected.decision(), ReviewDecision::Rejected);
        assert_eq!(rejected.reason_code(), ReasonCode::RejectedBlockedProbe);
        ReviewReceipt::decide(
            &blocked(),
            ReviewerAuthority::Automated,
            ReviewDecision::Deferred,
        )
        .unwrap();
    }

    #[test]
    fn review_required_probe_requires_authority() {
        // Automated authority cannot APPROVE a human_review_required probe...
        assert_eq!(
            ReviewReceipt::decide(
                &review_required(),
                ReviewerAuthority::Automated,
                ReviewDecision::Approved
            ),
            Err(ReviewError::AuthorityInsufficient)
        );
        // ...but human and governance authority can.
        for authority in [ReviewerAuthority::Human, ReviewerAuthority::Governance] {
            let r = ReviewReceipt::decide(&review_required(), authority, ReviewDecision::Approved)
                .unwrap();
            assert_eq!(r.decision(), ReviewDecision::Approved);
            assert_eq!(r.reason_code(), ReasonCode::ApprovedByReviewerAuthority);
        }
        // Automated MAY still reject or defer a review-required probe (safe dispositions).
        ReviewReceipt::decide(
            &review_required(),
            ReviewerAuthority::Automated,
            ReviewDecision::Deferred,
        )
        .unwrap();
    }

    #[test]
    fn queued_probe_can_be_approved_without_execution() {
        // A queued probe may be approved by any authority, and the receipt is inert: it records the
        // decision but executes nothing (there is no execution code in this crate — gate-enforced).
        let receipt = ReviewReceipt::decide(
            &queued(),
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
        )
        .unwrap();
        assert_eq!(receipt.decision(), ReviewDecision::Approved);
        assert_eq!(
            receipt.reason_code(),
            ReasonCode::ApprovedWithinAutomatedScope
        );
        // It may also be rejected.
        let rejected = ReviewReceipt::decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Rejected,
        )
        .unwrap();
        assert_eq!(rejected.decision(), ReviewDecision::Rejected);
        assert_eq!(rejected.reason_code(), ReasonCode::RejectedByReviewer);
    }

    #[test]
    fn review_receipt_order_and_hash_are_deterministic() {
        // Deterministic: identical inputs yield identical receipt, review_id, and integrity_hash.
        let p = queued();
        let a =
            ReviewReceipt::decide(&p, ReviewerAuthority::Human, ReviewDecision::Approved).unwrap();
        let b =
            ReviewReceipt::decide(&p, ReviewerAuthority::Human, ReviewDecision::Approved).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.review_id(), b.review_id());
        assert_eq!(a.integrity_hash(), b.integrity_hash());
        // A different decision changes the id and the hash.
        let c =
            ReviewReceipt::decide(&p, ReviewerAuthority::Human, ReviewDecision::Rejected).unwrap();
        assert_ne!(a.review_id(), c.review_id());
        assert_ne!(a.integrity_hash(), c.integrity_hash());
        // The log order is canonical / insertion-order independent.
        let r1 = ReviewReceipt::decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        )
        .unwrap();
        let r2 = ReviewReceipt::decide(
            &review_required(),
            ReviewerAuthority::Governance,
            ReviewDecision::Deferred,
        )
        .unwrap();
        let r3 = ReviewReceipt::decide(
            &blocked(),
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
        )
        .unwrap();
        let log1 = ReviewLog::from_receipts(vec![r1.clone(), r2.clone(), r3.clone()]);
        let log2 = ReviewLog::from_receipts(vec![r3, r1, r2]);
        assert_eq!(log1, log2, "log order is insertion-order independent");
        let ids: Vec<u64> = log1
            .receipts()
            .iter()
            .map(ReviewReceipt::review_id)
            .collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn replay_reproduces_review_receipt() {
        // A review trace is the INPUTS: the hypothesis spec (deserializable) plus the reviewer
        // authority and decision (also deserializable). Replay re-derives the identical receipt; the
        // receipt itself serializes (to emit a trace) but is never deserialized.
        let spec = HypothesisSpec {
            statement: "replayable review".to_string(),
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
            ReviewReceipt::decide(&probe, a, d).unwrap()
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

        assert_eq!(original, replayed, "replay reproduces the receipt");
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replayed).unwrap()
        );
        replayed.verify_integrity().unwrap();
    }

    #[test]
    fn forged_decision_cannot_be_constructed() {
        // The policy cannot be bypassed by building a raw struct: fields are private, there is no
        // public constructor or setter, and there is no Deserialize (the compile_fail doctest proves
        // it). The ONLY way to a receipt is `decide`, which enforces the policy — so an "approved
        // blocked probe" receipt cannot exist.
        assert!(matches!(
            ReviewReceipt::decide(
                &blocked(),
                ReviewerAuthority::Governance,
                ReviewDecision::Approved
            ),
            Err(ReviewError::BlockedCannotBeApproved)
        ));
    }

    #[test]
    fn review_receipt_cannot_be_evidence() {
        // A receipt inherits the canonical forbidden-uses quarantine: it can never ground a claim or
        // serve as evidence. (Structurally there is also no API turning a receipt into a
        // Claim/EvidenceRef/ProofObject, and the production crate has no verifier dependency.)
        let receipt = ReviewReceipt::decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        )
        .unwrap();
        for canonical in [
            "ground_claim",
            "serve_as_evidence",
            "mutate_reading_memory",
            "alter_verifier_receipt",
            "change_training_gate",
            "bypass_codec_or_governance",
        ] {
            assert!(!receipt.permits(canonical), "{canonical} must be forbidden");
        }
        assert!(receipt.permits("record_governance_decision"));
    }

    #[test]
    fn review_receipt_does_not_change_training_gate() {
        // Recording a review is orthogonal to P12: the training decision before and after is
        // identical — still training_not_justified.
        let before = reading_train_gate::decide(&[], &[]);
        let _r = ReviewReceipt::decide(
            &queued(),
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        )
        .unwrap();
        let after = reading_train_gate::decide(&[], &[]);
        assert_eq!(before, after);
        assert!(
            !after.training_justified,
            "a review receipt cannot change the training verdict"
        );
    }

    #[test]
    fn review_receipt_does_not_change_verifier_receipt() {
        // Reviewing a probe whose hypothesis cites a real receipt leaves the verifier receipt
        // byte-identical — the layer reads hashes, never the object, and executes nothing.
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
        let receipt = ReviewReceipt::decide(
            &probe,
            ReviewerAuthority::Governance,
            ReviewDecision::Approved,
        )
        .unwrap();
        assert_eq!(receipt.evidence_refs(), vec![cite].as_slice());

        let after = reading_cli::verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn review_tokens_are_machine_checkable() {
        assert_eq!(ReviewDecision::Approved.token(), "approved");
        assert_eq!(ReviewDecision::Rejected.token(), "rejected");
        assert_eq!(ReviewDecision::Deferred.token(), "deferred");
        assert_eq!(ReviewerAuthority::Automated.token(), "automated");
        assert_eq!(ReviewerAuthority::Governance.token(), "governance");
        assert!(!ReviewerAuthority::Automated.can_approve_review_required());
        assert!(ReviewerAuthority::Human.can_approve_review_required());
    }
}
