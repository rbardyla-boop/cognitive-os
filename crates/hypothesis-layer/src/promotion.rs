//! Observation promotion gate / still-no-evidence boundary (HYP-5).
//!
//! HYP-4 quarantines a [`ProbeObservationReceipt`] but cannot record anything — `recorded` is its
//! FUTURE-reserved disposition. HYP-5 adds the next boundary: a deterministic [`PromotionRequest`]
//! that records a REQUEST to promote a quarantined observation toward a claim, evidence, or a memory
//! note — while refusing to promote ANYTHING to evidence until a future verifier-backed path exists.
//!
//!   Hypothesis proposes.  Probe queue classifies.  Governance reviews.  HYP-3 records intent.
//!   HYP-4 quarantines observations.  HYP-5 records promotion requests.  Nothing becomes evidence.
//!
//! A promotion request holds no authority. Its disposition is DERIVED from the observation it is built
//! from, never supplied. A `rejected` or `requires_review` observation cannot be promoted at all — the
//! request is recorded `rejected`, independent of the requested target. The future-reserved `recorded`
//! observation (no HYP-4 intent yields it yet) still cannot become evidence here: a claim or evidence
//! target derives `requires_verifier` (it waits on a verifier that does not exist), and a memory-note
//! target derives `unsupported` (the layer may never mutate reading memory). So NO target ever yields a
//! grant — [`PromotionStatus::grants_promotion`] is always `false` at HYP-5, and is exhaustive so a
//! future promoting status cannot be added without an explicit, review-evident change.
//!
//! A `PromotionRequest` follows the same structural quarantine as the upstream receipts: private
//! fields, read-only accessors, derives `Serialize` but NOT `Deserialize` (the compiler enforces this
//! — see the `compile_fail` doctest; the derived `PromotionStatus`/`PromotionReason` enums are
//! `Serialize`-only, which is what keeps the whole request non-deserializable), and is minted ONLY by
//! [`PromotionRequest::from_observation`]. So a forged "promoted" request cannot be hand-set or
//! deserialized off the wire, and an observation can never become evidence.

use serde::{Deserialize, Serialize};

use crate::{
    fnv_str, fnv_u64, EvidenceRef, ObservationStatus, ProbeObservationReceipt, FNV_OFFSET,
    FORBIDDEN_USES,
};

/// What a caller is REQUESTING an observation be promoted to. This is an INPUT (the only governed-by
/// the-caller field), so unlike the derived enums below it derives `Deserialize` — a replay trace
/// carries the requested target as a deserializable input, exactly like the reviewer authority and
/// decision upstream. It grants nothing on its own; the OUTCOME is always re-derived.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromotionTarget {
    /// Promote the observation into a grounded claim.
    #[serde(rename = "claim")]
    Claim,
    /// Promote the observation into evidence (the exact authority leak HYP-5 refuses).
    #[serde(rename = "evidence")]
    Evidence,
    /// Promote the observation into a written memory note.
    #[serde(rename = "memory_note")]
    MemoryNote,
}

impl PromotionTarget {
    /// A machine-checkable token (never prose) for the requested target.
    pub fn token(self) -> &'static str {
        match self {
            PromotionTarget::Claim => "claim",
            PromotionTarget::Evidence => "evidence",
            PromotionTarget::MemoryNote => "memory_note",
        }
    }
}

/// The outcome of a promotion request — the MACHINE-CHECKABLE record of what a promotion is allowed to
/// be, never prose. Every value is a NON-promoting state: HYP-5 promotes nothing, so there is
/// deliberately no `promoted` / `granted` variant. Derived (output), so it does NOT derive
/// `Deserialize`; that also keeps [`PromotionRequest`] structurally non-deserializable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PromotionStatus {
    /// The source observation is not promotable (it was `rejected` or still `requires_review`): the
    /// promotion is refused at the source, for any target.
    #[serde(rename = "rejected")]
    Rejected,
    /// The source observation could in principle be promoted (a future `recorded` observation), but
    /// the requested target needs a verifier-backed path that does not exist yet. Deferred, never
    /// granted.
    #[serde(rename = "requires_verifier")]
    RequiresVerifier,
    /// The requested target is not a supported promotion in this architecture — a memory note, which
    /// the hypothesis layer may never write (`mutate_reading_memory` is forbidden).
    #[serde(rename = "unsupported")]
    Unsupported,
}

impl PromotionStatus {
    /// The outcome implied by a promotion reason. Total and exhaustive (no wildcard), so a new reason
    /// forces an explicit mapping here (E0004) rather than silently defaulting to a grant.
    fn from_reason(reason: PromotionReason) -> Self {
        match reason {
            PromotionReason::ObservationRejectedNotPromotable
            | PromotionReason::ObservationRequiresReviewNotPromotable => PromotionStatus::Rejected,
            PromotionReason::ClaimRequiresVerifier | PromotionReason::EvidenceRequiresVerifier => {
                PromotionStatus::RequiresVerifier
            }
            PromotionReason::MemoryNoteUnsupported => PromotionStatus::Unsupported,
        }
    }

    /// Whether this status PROMOTES the observation out of quarantine (grants it claim/evidence/memory
    /// authority). At HYP-5 this is ALWAYS `false`: no verifier-backed promotion path exists yet, so
    /// every status is a refusal or a deferral — "still no evidence". The match is exhaustive with NO
    /// wildcard, so a future promoting variant cannot be added without an explicit `true` here (E0004),
    /// making the regression review-evident.
    pub fn grants_promotion(self) -> bool {
        match self {
            PromotionStatus::Rejected
            | PromotionStatus::RequiresVerifier
            | PromotionStatus::Unsupported => false,
        }
    }

    /// A machine-checkable token (never prose) for the status.
    pub fn token(self) -> &'static str {
        match self {
            PromotionStatus::Rejected => "rejected",
            PromotionStatus::RequiresVerifier => "requires_verifier",
            PromotionStatus::Unsupported => "unsupported",
        }
    }
}

/// Why a promotion request got its outcome — a machine-checkable classification, never prose. Derived
/// only (output), so it does NOT derive `Deserialize`; like [`PromotionStatus`] this keeps the request
/// structurally non-deserializable (a forged request cannot be built off the wire).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum PromotionReason {
    /// The source observation was `rejected`: there is nothing promotable.
    #[serde(rename = "observation_rejected_not_promotable")]
    ObservationRejectedNotPromotable,
    /// The source observation still `requires_review`: it cannot be promoted before that review.
    #[serde(rename = "observation_requires_review_not_promotable")]
    ObservationRequiresReviewNotPromotable,
    /// A claim target for a (future) recorded observation: requires a verifier that does not exist.
    #[serde(rename = "claim_requires_verifier")]
    ClaimRequiresVerifier,
    /// An evidence target for a (future) recorded observation: requires a verifier that does not exist.
    #[serde(rename = "evidence_requires_verifier")]
    EvidenceRequiresVerifier,
    /// A memory-note target: unsupported, because the layer may never mutate reading memory.
    #[serde(rename = "memory_note_unsupported")]
    MemoryNoteUnsupported,
}

impl PromotionReason {
    /// Deterministic classification from the observation's disposition and the requested target.
    /// Exhaustive at both levels, no wildcard: a non-promotable observation (`rejected`/
    /// `requires_review`) yields a refusal for ANY target, and only the future-reserved `recorded`
    /// observation consults the target — where claim/evidence defer to a verifier and a memory note is
    /// unsupported. NO arm grants a promotion.
    fn derive(observation_status: ObservationStatus, requested_target: PromotionTarget) -> Self {
        match observation_status {
            ObservationStatus::Rejected => PromotionReason::ObservationRejectedNotPromotable,
            ObservationStatus::RequiresReview => {
                PromotionReason::ObservationRequiresReviewNotPromotable
            }
            ObservationStatus::Recorded => match requested_target {
                PromotionTarget::Claim => PromotionReason::ClaimRequiresVerifier,
                PromotionTarget::Evidence => PromotionReason::EvidenceRequiresVerifier,
                PromotionTarget::MemoryNote => PromotionReason::MemoryNoteUnsupported,
            },
        }
    }

    /// The outcome this reason implies (lets a cross-check confirm reason and status agree).
    pub fn status(self) -> PromotionStatus {
        PromotionStatus::from_reason(self)
    }

    /// A machine-checkable token (never prose) for the reason.
    pub fn token(self) -> &'static str {
        match self {
            PromotionReason::ObservationRejectedNotPromotable => {
                "observation_rejected_not_promotable"
            }
            PromotionReason::ObservationRequiresReviewNotPromotable => {
                "observation_requires_review_not_promotable"
            }
            PromotionReason::ClaimRequiresVerifier => "claim_requires_verifier",
            PromotionReason::EvidenceRequiresVerifier => "evidence_requires_verifier",
            PromotionReason::MemoryNoteUnsupported => "memory_note_unsupported",
        }
    }
}

/// What can go wrong handling a promotion request. Every failure is explicit; nothing is silently
/// coerced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromotionError {
    /// A request's recomputed integrity hash does not match the stored one (tamper detection).
    IntegrityMismatch,
}

impl std::fmt::Display for PromotionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromotionError::IntegrityMismatch => {
                write!(f, "promotion request integrity hash mismatch")
            }
        }
    }
}

impl std::error::Error for PromotionError {}

/// A request to promote a quarantined [`ProbeObservationReceipt`] — the boundary that proves an
/// observation does not become evidence just because it exists. It is inert: it promotes nothing,
/// holds no claim/evidence authority, and cannot mutate anything. Its outcome is DERIVED from the
/// observation and the requested target — a `rejected`/`requires_review` observation yields a
/// `rejected` request, and the future-reserved `recorded` observation yields `requires_verifier`
/// (claim/evidence) or `unsupported` (memory note); no path grants a promotion.
///
/// Minted ONLY by [`PromotionRequest::from_observation`]; its fields are private and read-only, and it
/// derives `Serialize` but NOT `Deserialize`. The first example records a real request; the
/// `compile_fail` example proves a request cannot be deserialized — so a forged "promoted" request
/// cannot enter the system off the wire. If either property regresses, `cargo test` fails.
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
/// let obs = hypothesis_layer::ProbeObservationReceipt::from_intent(&intent, "observed: log span re-read");
/// let req = hypothesis_layer::PromotionRequest::from_observation(
///     &obs,
///     hypothesis_layer::PromotionTarget::Evidence,
/// );
/// let _id: u64 = req.promotion_id();
/// assert!(!req.status().grants_promotion());
/// ```
///
/// ```compile_fail
/// // A PromotionRequest implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::PromotionRequest = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromotionRequest {
    // Private + read-only: a request is built ONLY by `from_observation`, which DERIVES the outcome
    // from the observation and target — so a forged "promoted" request cannot exist.
    promotion_id: u64,
    observation_id: u64,
    intent_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    requested_target: PromotionTarget,
    status: PromotionStatus,
    reason_code: PromotionReason,
    evidence_refs: Vec<EvidenceRef>,
    created_from_observation_trace: bool,
    integrity_hash: u64,
}

impl PromotionRequest {
    /// Record a promotion request from a quarantined [`ProbeObservationReceipt`]. The outcome is
    /// DERIVED from the observation's disposition and the requested target — a `rejected`/
    /// `requires_review` observation yields a `rejected` request (for any target), and the
    /// future-reserved `recorded` observation yields `requires_verifier` (claim/evidence) or
    /// `unsupported` (memory note). NO path grants a promotion. The only way to a
    /// `&ProbeObservationReceipt` is HYP-4's `from_intent`, so a request is, by construction, derived
    /// only from a valid observation. Pure and deterministic; PROMOTES NOTHING and holds no authority.
    pub fn from_observation(
        observation: &ProbeObservationReceipt,
        requested_target: PromotionTarget,
    ) -> PromotionRequest {
        let reason_code =
            PromotionReason::derive(observation.observation_status(), requested_target);
        let status = PromotionStatus::from_reason(reason_code);
        let observation_id = observation.observation_id();
        let intent_id = observation.intent_id();
        let probe_id = observation.probe_id();
        let hypothesis_id = observation.hypothesis_id();
        let evidence_refs = observation.evidence_refs().to_vec();
        let created_from_observation_trace = observation.created_from_intent_trace();
        let promotion_id = derive_promotion_id(
            observation_id,
            intent_id,
            probe_id,
            hypothesis_id,
            requested_target,
            status,
            reason_code,
        );
        // Build with a placeholder hash, then bind the integrity hash over the finished fields — ONE
        // hashing path (`compute_integrity`) is shared by `from_observation` and `verify_integrity`.
        let base = PromotionRequest {
            promotion_id,
            observation_id,
            intent_id,
            probe_id,
            hypothesis_id,
            requested_target,
            status,
            reason_code,
            evidence_refs,
            created_from_observation_trace,
            integrity_hash: 0,
        };
        PromotionRequest {
            integrity_hash: base.compute_integrity(),
            ..base
        }
    }

    /// Deterministic integrity hash over every field EXCEPT `integrity_hash` itself (length-prefixed
    /// strings so distinct requests cannot collide by re-grouping bytes).
    fn compute_integrity(&self) -> u64 {
        let mut h = FNV_OFFSET;
        h = fnv_u64(h, self.promotion_id);
        h = fnv_u64(h, self.observation_id);
        h = fnv_u64(h, self.intent_id);
        h = fnv_u64(h, self.probe_id);
        h = fnv_u64(h, self.hypothesis_id);
        h = fnv_str(h, self.requested_target.token());
        h = fnv_str(h, self.status.token());
        h = fnv_str(h, self.reason_code.token());
        h = fnv_u64(h, self.evidence_refs.len() as u64);
        for ev in &self.evidence_refs {
            h = fnv_u64(h, ev.answer_hash);
            h = fnv_u64(h, ev.memory_hash);
            h = fnv_str(h, &ev.source_label);
        }
        h = fnv_u64(h, self.created_from_observation_trace as u64);
        h
    }

    /// Deterministic content id of the promotion request.
    pub fn promotion_id(&self) -> u64 {
        self.promotion_id
    }

    /// The id of the observation this request was built from (provenance).
    pub fn observation_id(&self) -> u64 {
        self.observation_id
    }

    /// The id of the execution intent the observation came from (provenance).
    pub fn intent_id(&self) -> u64 {
        self.intent_id
    }

    /// The id of the reviewed probe (provenance).
    pub fn probe_id(&self) -> u64 {
        self.probe_id
    }

    /// The id of the originating hypothesis (provenance).
    pub fn hypothesis_id(&self) -> u64 {
        self.hypothesis_id
    }

    /// What the caller requested the observation be promoted to (read-only).
    pub fn requested_target(&self) -> PromotionTarget {
        self.requested_target
    }

    /// The machine-checkable promotion outcome (read-only — it cannot be flipped to a grant after the
    /// fact).
    pub fn status(&self) -> PromotionStatus {
        self.status
    }

    /// The machine-checkable reason for the outcome.
    pub fn reason_code(&self) -> PromotionReason {
        self.reason_code
    }

    /// The receipts the originating hypothesis cited (carried through as provenance, never as evidence
    /// the request itself produces).
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// Whether the originating hypothesis was derived from a trace/receipt (carried through the
    /// hypothesis -> probe -> review -> intent -> observation -> promotion chain).
    pub fn created_from_observation_trace(&self) -> bool {
        self.created_from_observation_trace
    }

    /// The deterministic integrity hash binding every field of this request.
    pub fn integrity_hash(&self) -> u64 {
        self.integrity_hash
    }

    /// Whether this request PROMOTES the observation — ALWAYS `false` at HYP-5 (still no evidence).
    pub fn grants_promotion(&self) -> bool {
        self.status.grants_promotion()
    }

    /// Whether the promotion was refused at the source (a non-promotable observation).
    pub fn is_rejected(&self) -> bool {
        self.status == PromotionStatus::Rejected
    }

    /// Whether the (future-reserved) outcome defers to a verifier that does not exist yet.
    pub fn requires_verifier(&self) -> bool {
        self.status == PromotionStatus::RequiresVerifier
    }

    /// Whether this request may be used for the given purpose. Always `false` for any forbidden use: a
    /// promotion request is never truth, evidence, or a mutator. It inherits the canonical
    /// [`FORBIDDEN_USES`] quarantine, so it can never become a claim or ground an answer.
    pub fn permits(&self, use_name: &str) -> bool {
        !FORBIDDEN_USES.contains(&use_name)
    }

    /// Re-derive the integrity hash from this request's OWN fields and confirm it matches. Because a
    /// request is born only from [`from_observation`] (private fields, no `Deserialize`), it is
    /// consistent by construction; this is an explicit, auditable assertion of that binding — used to
    /// prove a replay was faithful. It grants no authority and promotes nothing.
    pub fn verify_integrity(&self) -> Result<(), PromotionError> {
        if self.compute_integrity() == self.integrity_hash {
            Ok(())
        } else {
            Err(PromotionError::IntegrityMismatch)
        }
    }
}

/// Deterministic id of the promotion request (FNV-1a over its defining fields, length-prefixed).
fn derive_promotion_id(
    observation_id: u64,
    intent_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    requested_target: PromotionTarget,
    status: PromotionStatus,
    reason_code: PromotionReason,
) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_u64(h, observation_id);
    h = fnv_u64(h, intent_id);
    h = fnv_u64(h, probe_id);
    h = fnv_u64(h, hypothesis_id);
    h = fnv_str(h, requested_target.token());
    h = fnv_str(h, status.token());
    h = fnv_str(h, reason_code.token());
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        propose, HypothesisSpec, ProbeExecutionIntent, ProbeRequest, ReviewDecision, ReviewReceipt,
        ReviewerAuthority,
    };

    fn ev(label: &str) -> EvidenceRef {
        EvidenceRef {
            answer_hash: 0x1111_2222_3333_4444,
            memory_hash: 0x5555_6666_7777_8888,
            source_label: label.to_string(),
        }
    }

    fn observation_with(
        authority: ReviewerAuthority,
        decision: ReviewDecision,
    ) -> ProbeObservationReceipt {
        let spec = HypothesisSpec {
            statement: "Bridge B reopened.".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![ev("run.json")],
            probe_description: "Re-read the maintenance log span.".to_string(),
        };
        let probe = ProbeRequest::from_hypothesis(&propose(spec).unwrap());
        let intent = ProbeExecutionIntent::from_review(
            &ReviewReceipt::decide(&probe, authority, decision).unwrap(),
        );
        ProbeObservationReceipt::from_intent(&intent, "observed: something happened")
    }

    // observation_status Rejected (not_executed intent: automated-scope approval).
    fn rejected_obs() -> ProbeObservationReceipt {
        observation_with(ReviewerAuthority::Automated, ReviewDecision::Approved)
    }

    // observation_status RequiresReview (requires_operator intent: human approval).
    fn requires_review_obs() -> ProbeObservationReceipt {
        observation_with(ReviewerAuthority::Human, ReviewDecision::Approved)
    }

    // observation_status Rejected (blocked intent: rejected review).
    fn blocked_obs() -> ProbeObservationReceipt {
        observation_with(ReviewerAuthority::Governance, ReviewDecision::Rejected)
    }

    #[test]
    fn promotion_derived_only_from_observation_receipt() {
        // A request is recorded from an observation and cites the observation_id, intent_id, probe_id,
        // hypothesis_id, and source EvidenceRefs. (A ProbeObservationReceipt only exists via
        // from_intent on a real intent, so a request is always derived from a valid observation.)
        let obs = requires_review_obs();
        let req = PromotionRequest::from_observation(&obs, PromotionTarget::Claim);
        assert_eq!(req.observation_id(), obs.observation_id());
        assert_eq!(req.intent_id(), obs.intent_id());
        assert_eq!(req.probe_id(), obs.probe_id());
        assert_eq!(req.hypothesis_id(), obs.hypothesis_id());
        assert_eq!(req.evidence_refs(), obs.evidence_refs());
        assert_eq!(
            req.created_from_observation_trace(),
            obs.created_from_intent_trace()
        );
        assert_eq!(req.requested_target(), PromotionTarget::Claim);
        assert_ne!(req.promotion_id(), 0);
        req.verify_integrity().unwrap();
    }

    #[test]
    fn rejected_observation_cannot_promote() {
        // A `rejected` observation cannot be promoted, for ANY target: the request is `rejected` and
        // grants nothing.
        for target in [
            PromotionTarget::Claim,
            PromotionTarget::Evidence,
            PromotionTarget::MemoryNote,
        ] {
            let req = PromotionRequest::from_observation(&rejected_obs(), target);
            assert_eq!(req.status(), PromotionStatus::Rejected);
            assert_eq!(
                req.reason_code(),
                PromotionReason::ObservationRejectedNotPromotable
            );
            assert!(req.is_rejected());
            assert!(!req.grants_promotion());
        }
        // The blocked-intent observation is likewise `rejected`, so it too cannot promote.
        let req = PromotionRequest::from_observation(&blocked_obs(), PromotionTarget::Evidence);
        assert_eq!(req.status(), PromotionStatus::Rejected);
        assert!(!req.grants_promotion());
    }

    #[test]
    fn requires_review_observation_cannot_promote() {
        // A `requires_review` observation cannot be promoted, for ANY target: the request is
        // `rejected` (a human/governance must review the observation first) and grants nothing.
        for target in [
            PromotionTarget::Claim,
            PromotionTarget::Evidence,
            PromotionTarget::MemoryNote,
        ] {
            let req = PromotionRequest::from_observation(&requires_review_obs(), target);
            assert_eq!(req.status(), PromotionStatus::Rejected);
            assert_eq!(
                req.reason_code(),
                PromotionReason::ObservationRequiresReviewNotPromotable
            );
            assert!(!req.grants_promotion());
        }
    }

    #[test]
    fn recorded_observation_requires_future_verifier() {
        // `recorded` is HYP-4's FUTURE-reserved disposition (no intent yields it yet), so a recorded
        // observation cannot be built via from_intent. The promotion derivation is nonetheless TOTAL
        // over the disposition: even a recorded observation requesting a claim or evidence target
        // derives `requires_verifier`, NEVER a grant — promotion waits on a verifier-backed path that
        // does not exist at HYP-5.
        for target in [PromotionTarget::Claim, PromotionTarget::Evidence] {
            let reason = PromotionReason::derive(ObservationStatus::Recorded, target);
            let status = PromotionStatus::from_reason(reason);
            assert_eq!(status, PromotionStatus::RequiresVerifier);
            assert!(!status.grants_promotion());
        }
        // A memory-note target is structurally unsupported (the layer may never mutate reading memory).
        let mem = PromotionReason::derive(ObservationStatus::Recorded, PromotionTarget::MemoryNote);
        assert_eq!(mem, PromotionReason::MemoryNoteUnsupported);
        assert_eq!(mem.status(), PromotionStatus::Unsupported);
        assert!(!mem.status().grants_promotion());
    }

    #[test]
    fn promotion_never_yields_evidence_authority() {
        // STILL NO EVIDENCE: across EVERY (observation disposition, requested target) cell, the
        // derived status NEVER grants a promotion — there is no path by which an observation becomes
        // evidence or a claim at HYP-5. grants_promotion is exhaustive with no wildcard, so a future
        // promoting status could not be added without an explicit `true` (E0004), making the
        // regression review-evident.
        for status in [
            ObservationStatus::Recorded,
            ObservationStatus::Rejected,
            ObservationStatus::RequiresReview,
        ] {
            for target in [
                PromotionTarget::Claim,
                PromotionTarget::Evidence,
                PromotionTarget::MemoryNote,
            ] {
                let s = PromotionStatus::from_reason(PromotionReason::derive(status, target));
                assert!(!s.grants_promotion(), "no cell may grant a promotion");
                assert_ne!(s.token(), "promoted");
                assert_ne!(s.token(), "evidence");
            }
        }
        // A real evidence-target request refuses the evidence/claim uses outright.
        let req = PromotionRequest::from_observation(&rejected_obs(), PromotionTarget::Evidence);
        assert!(!req.permits("serve_as_evidence"));
        assert!(!req.permits("ground_claim"));
        assert!(!req.grants_promotion());
    }

    #[test]
    fn promotion_status_and_reason_agree() {
        // The status and the reason are derived consistently for every (disposition, target): the
        // reason's implied status always equals the request's status. If the two classifiers diverged,
        // this fails.
        let cases = [
            (rejected_obs(), PromotionTarget::Claim),
            (requires_review_obs(), PromotionTarget::Evidence),
            (blocked_obs(), PromotionTarget::MemoryNote),
        ];
        for (obs, target) in cases {
            let req = PromotionRequest::from_observation(&obs, target);
            assert_eq!(req.reason_code().status(), req.status());
        }
        // Also at the derive level for the future-reserved recorded cells.
        for target in [
            PromotionTarget::Claim,
            PromotionTarget::Evidence,
            PromotionTarget::MemoryNote,
        ] {
            let reason = PromotionReason::derive(ObservationStatus::Recorded, target);
            assert_eq!(reason.status(), PromotionStatus::from_reason(reason));
        }
    }

    #[test]
    fn forged_promotion_cannot_be_constructed() {
        // The outcome cannot be bypassed by building a raw struct: fields are private, there is no
        // public constructor or setter, and there is no Deserialize (the compile_fail doctest proves
        // it). The ONLY way to a request is from_observation, which always DERIVES the outcome and
        // NEVER grants a promotion — so a forged "promoted" request cannot exist. (A second,
        // independent covering test of the no-grant property.)
        for obs in [rejected_obs(), requires_review_obs(), blocked_obs()] {
            for target in [
                PromotionTarget::Claim,
                PromotionTarget::Evidence,
                PromotionTarget::MemoryNote,
            ] {
                let req = PromotionRequest::from_observation(&obs, target);
                assert!(!req.grants_promotion());
                assert_eq!(req.status(), PromotionStatus::Rejected);
            }
        }
    }

    #[test]
    fn promotion_hash_is_deterministic() {
        // Deterministic: identical (observation, target) yield identical request, id, and hash.
        let obs = requires_review_obs();
        let a = PromotionRequest::from_observation(&obs, PromotionTarget::Claim);
        let b = PromotionRequest::from_observation(&obs, PromotionTarget::Claim);
        assert_eq!(a, b);
        assert_eq!(a.promotion_id(), b.promotion_id());
        assert_eq!(a.integrity_hash(), b.integrity_hash());
        // A different target changes the id and the hash (the request records what was requested).
        let c = PromotionRequest::from_observation(&obs, PromotionTarget::Evidence);
        assert_ne!(a.promotion_id(), c.promotion_id());
        assert_ne!(a.integrity_hash(), c.integrity_hash());
        // A different observation disposition changes the id and the hash.
        let d = PromotionRequest::from_observation(&rejected_obs(), PromotionTarget::Claim);
        assert_ne!(a.promotion_id(), d.promotion_id());
    }

    #[test]
    fn promotion_replay_reproduces_same_record() {
        // A trace is the INPUTS: the hypothesis spec plus the reviewer authority and decision plus the
        // observation text plus the requested target (all deserializable). Replay re-derives the
        // identical request by re-running propose -> decide -> from_review -> from_intent ->
        // from_observation; the request itself serializes (to emit a trace) but is never deserialized.
        let spec = HypothesisSpec {
            statement: "replayable promotion".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![ev("run.json")],
            probe_description: "probe".to_string(),
        };
        let authority_json = "\"human\"";
        let decision_json = "\"approved\"";
        let target_json = "\"evidence\"";
        let text = "observed: the span was re-read";
        let build =
            |spec: &HypothesisSpec, a: ReviewerAuthority, d: ReviewDecision, t: PromotionTarget| {
                let probe = ProbeRequest::from_hypothesis(&propose(spec.clone()).unwrap());
                let intent = ProbeExecutionIntent::from_review(
                    &ReviewReceipt::decide(&probe, a, d).unwrap(),
                );
                let obs = ProbeObservationReceipt::from_intent(&intent, text);
                PromotionRequest::from_observation(&obs, t)
            };
        let authority: ReviewerAuthority = serde_json::from_str(authority_json).unwrap();
        let decision: ReviewDecision = serde_json::from_str(decision_json).unwrap();
        let target: PromotionTarget = serde_json::from_str(target_json).unwrap();
        let original = build(&spec, authority, decision, target);

        // Round-trip the trace inputs and rebuild.
        let spec2: HypothesisSpec =
            serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
        let authority2: ReviewerAuthority = serde_json::from_str(authority_json).unwrap();
        let decision2: ReviewDecision = serde_json::from_str(decision_json).unwrap();
        let target2: PromotionTarget =
            serde_json::from_str(&serde_json::to_string(&target).unwrap()).unwrap();
        let replayed = build(&spec2, authority2, decision2, target2);

        assert_eq!(original, replayed, "replay reproduces the request");
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replayed).unwrap()
        );
        replayed.verify_integrity().unwrap();
    }

    #[test]
    fn promotion_preserves_forbidden_uses() {
        // A request inherits the canonical forbidden-uses quarantine: it can never ground a claim or
        // serve as evidence. (Structurally there is also no API turning a request into a
        // Claim/EvidenceRef/ProofObject, and the production crate has no verifier dependency.)
        let req =
            PromotionRequest::from_observation(&requires_review_obs(), PromotionTarget::Claim);
        for canonical in [
            "ground_claim",
            "serve_as_evidence",
            "mutate_reading_memory",
            "alter_verifier_receipt",
            "change_training_gate",
            "bypass_codec_or_governance",
        ] {
            assert!(!req.permits(canonical), "{canonical} must be forbidden");
        }
        assert!(req.permits("record_promotion_request"));
    }

    #[test]
    fn promotion_does_not_change_training_gate() {
        // Recording a promotion request is orthogonal to P12: the training decision before and after
        // is identical — still training_not_justified.
        let before = reading_train_gate::decide(&[], &[]);
        let _req =
            PromotionRequest::from_observation(&requires_review_obs(), PromotionTarget::Evidence);
        let after = reading_train_gate::decide(&[], &[]);
        assert_eq!(before, after);
        assert!(
            !after.training_justified,
            "a promotion request cannot change the training verdict"
        );
    }

    #[test]
    fn promotion_does_not_change_verifier_receipt() {
        // Recording a request from an observation whose hypothesis cites a real receipt leaves the
        // verifier receipt byte-identical — the layer reads hashes, never the object, and promotes
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
        let intent = ProbeExecutionIntent::from_review(
            &ReviewReceipt::decide(
                &probe,
                ReviewerAuthority::Governance,
                ReviewDecision::Approved,
            )
            .unwrap(),
        );
        let obs = ProbeObservationReceipt::from_intent(&intent, "observed result");
        let req = PromotionRequest::from_observation(&obs, PromotionTarget::Evidence);
        assert_eq!(req.evidence_refs(), vec![cite].as_slice());
        assert!(!req.grants_promotion());

        let after = reading_cli::verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn promotion_tokens_are_machine_checkable() {
        assert_eq!(PromotionTarget::Claim.token(), "claim");
        assert_eq!(PromotionTarget::Evidence.token(), "evidence");
        assert_eq!(PromotionTarget::MemoryNote.token(), "memory_note");
        assert_eq!(PromotionStatus::Rejected.token(), "rejected");
        assert_eq!(
            PromotionStatus::RequiresVerifier.token(),
            "requires_verifier"
        );
        assert_eq!(PromotionStatus::Unsupported.token(), "unsupported");
        assert_eq!(
            PromotionReason::ObservationRejectedNotPromotable.token(),
            "observation_rejected_not_promotable"
        );
        assert_eq!(
            PromotionReason::ObservationRequiresReviewNotPromotable.token(),
            "observation_requires_review_not_promotable"
        );
        assert_eq!(
            PromotionReason::ClaimRequiresVerifier.token(),
            "claim_requires_verifier"
        );
        assert_eq!(
            PromotionReason::EvidenceRequiresVerifier.token(),
            "evidence_requires_verifier"
        );
        assert_eq!(
            PromotionReason::MemoryNoteUnsupported.token(),
            "memory_note_unsupported"
        );
        assert_eq!(
            PromotionError::IntegrityMismatch.to_string(),
            "promotion request integrity hash mismatch"
        );
    }
}
