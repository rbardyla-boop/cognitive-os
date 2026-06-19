//! Observation receipt quarantine (HYP-4).
//!
//! HYP-3 records an inert [`ProbeExecutionIntent`] but still executes nothing. HYP-4 adds the
//! quarantine FORMAT for a FUTURE probe result: a typed [`ProbeObservationReceipt`] that can record
//! "something was observed" WITHOUT letting that observation become evidence, a claim, verifier
//! input, or a memory mutation — and without implying the probe actually ran.
//!
//!   Hypothesis proposes.  Probe queue classifies.  Governance reviews.  HYP-3 records intent.
//!   HYP-4 quarantines observations.  Nothing becomes evidence.
//!
//! An observation is `observation_only`: it holds no authority. Its disposition is DERIVED from the
//! intent — a `not_executed` or `blocked` intent yields a `rejected` observation (there is nothing
//! legitimate to record), and a `requires_operator` intent yields a `requires_review` observation
//! (a human/governance must review it). The third status, `recorded`, is the FUTURE-reserved
//! promotion target: NO current intent disposition produces it, so at HYP-4 nothing can be recorded
//! — the quarantine holds until a future verifier/governance promotion path exists (enforced by the
//! `no_intent_disposition_yields_recorded` test). A `ProbeObservationReceipt` follows the same
//! structural quarantine as the upstream receipts: private fields, read-only accessors, derives
//! `Serialize` but NOT `Deserialize` (the compiler enforces this — see the `compile_fail` doctest),
//! and is minted ONLY by [`ProbeObservationReceipt::from_intent`]. So a forged observation cannot be
//! hand-set or deserialized off the wire, and it can never become evidence.

use serde::Serialize;

use crate::{
    fnv_str, fnv_u64, EvidenceRef, ExecutionStatus, ProbeExecutionIntent, FNV_OFFSET,
    FORBIDDEN_USES,
};

/// The quarantine disposition of an observation — the MACHINE-CHECKABLE record of what an observation
/// is allowed to be, never prose. Derived (output), so it does NOT derive `Deserialize`; that also
/// keeps [`ProbeObservationReceipt`] structurally non-deserializable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ObservationStatus {
    /// The FUTURE-reserved promotion target: an observation that has been promoted out of quarantine
    /// by a verifier/governance path. NO HYP-4 intent disposition produces this — at HYP-4 nothing
    /// can be recorded, because no promotion path exists yet (the quarantine).
    #[serde(rename = "recorded")]
    Recorded,
    /// The intent did not clear the probe for a result (it was `not_executed` or `blocked`): the
    /// observation is refused. There is nothing legitimate to record.
    #[serde(rename = "rejected")]
    Rejected,
    /// The intent requires a human operator: the observation must be reviewed by human/governance
    /// before any disposition. It is quarantined pending review, never recorded automatically.
    #[serde(rename = "requires_review")]
    RequiresReview,
}

impl ObservationStatus {
    /// The quarantine disposition implied by an intent's execution status. Total and exhaustive (no
    /// wildcard), so a new [`ExecutionStatus`] variant forces an explicit mapping here (E0004)
    /// rather than silently becoming recordable. Crucially, NO arm yields `Recorded`: an observation
    /// can never be recorded at HYP-4 — the promotion path is future work.
    fn from_execution_status(status: ExecutionStatus) -> Self {
        match status {
            ExecutionStatus::NotExecuted => ObservationStatus::Rejected,
            ExecutionStatus::Blocked => ObservationStatus::Rejected,
            ExecutionStatus::RequiresOperator => ObservationStatus::RequiresReview,
        }
    }

    /// A machine-checkable token (never prose) for the status.
    pub fn token(self) -> &'static str {
        match self {
            ObservationStatus::Recorded => "recorded",
            ObservationStatus::Rejected => "rejected",
            ObservationStatus::RequiresReview => "requires_review",
        }
    }
}

/// The authority an observation can hold. There is exactly ONE variant: an observation is ALWAYS and
/// ONLY `observation_only`. Any other authority is unrepresentable, so an observation can never be
/// marked as carrying claim/evidence/verifier/governance authority. Derived (output), `Serialize`
/// only — which also keeps [`ProbeObservationReceipt`] structurally non-deserializable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ObservationAuthority {
    #[serde(rename = "observation_only")]
    ObservationOnly,
}

impl ObservationAuthority {
    /// A machine-checkable token (never prose).
    pub fn token(self) -> &'static str {
        match self {
            ObservationAuthority::ObservationOnly => "observation_only",
        }
    }
}

/// What can go wrong handling an observation receipt. Every failure is explicit; nothing is silently
/// coerced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObservationError {
    /// An observation's recomputed integrity hash does not match the stored one (tamper detection).
    IntegrityMismatch,
}

impl std::fmt::Display for ObservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObservationError::IntegrityMismatch => {
                write!(f, "observation receipt integrity hash mismatch")
            }
        }
    }
}

impl std::error::Error for ObservationError {}

/// A quarantined observation recorded from a [`ProbeExecutionIntent`] — the boundary that lets a
/// future probe result be captured WITHOUT becoming evidence. It is inert: it holds no claim/evidence
/// authority (`observation_only`), cannot ground an answer or mutate anything, and does not imply the
/// probe actually ran. Its disposition is derived from the intent: a `not_executed`/`blocked` intent
/// yields a `rejected` observation, a `requires_operator` intent yields `requires_review`, and no
/// intent yields `recorded` (the promotion path is future work).
///
/// Minted ONLY by [`ProbeObservationReceipt::from_intent`]; its fields are private and read-only, and
/// it derives `Serialize` but NOT `Deserialize`. The first example records a real observation; the
/// `compile_fail` example proves an observation cannot be deserialized — so a forged `recorded`
/// observation cannot enter the system off the wire. If either property regresses, `cargo test` fails.
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
/// let _id: u64 = obs.observation_id();
/// ```
///
/// ```compile_fail
/// // A ProbeObservationReceipt implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::ProbeObservationReceipt = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProbeObservationReceipt {
    // Private + read-only: an observation is built ONLY by `from_intent`, which DERIVES the
    // disposition from the intent — so a forged `recorded` observation cannot exist.
    observation_id: u64,
    intent_id: u64,
    review_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    observation_text: String,
    observation_status: ObservationStatus,
    authority: ObservationAuthority,
    evidence_refs: Vec<EvidenceRef>,
    created_from_intent_trace: bool,
    integrity_hash: u64,
}

impl ProbeObservationReceipt {
    /// Quarantine an observation recorded from a [`ProbeExecutionIntent`]. The disposition is DERIVED
    /// from the intent's execution status — a `not_executed`/`blocked` intent yields `rejected`, a
    /// `requires_operator` intent yields `requires_review`, and no intent yields `recorded`. The
    /// `observation_text` is the CLAIMED observation; it is quarantined data only and does not imply
    /// the probe ran (the only way to a `&ProbeExecutionIntent` is HYP-3's `from_review`, so an
    /// observation is, by construction, derived only from a valid intent). Pure and deterministic;
    /// the observation holds `observation_only` authority and can never become evidence.
    pub fn from_intent(
        intent: &ProbeExecutionIntent,
        observation_text: &str,
    ) -> ProbeObservationReceipt {
        let observation_status =
            ObservationStatus::from_execution_status(intent.execution_status());
        let intent_id = intent.intent_id();
        let review_id = intent.review_id();
        let probe_id = intent.probe_id();
        let hypothesis_id = intent.hypothesis_id();
        let evidence_refs = intent.evidence_refs().to_vec();
        let created_from_intent_trace = intent.created_from_review_trace();
        let observation_text = observation_text.to_string();
        let observation_id = derive_observation_id(
            intent_id,
            review_id,
            probe_id,
            hypothesis_id,
            &observation_text,
            observation_status,
        );
        // Build with a placeholder hash, then bind the integrity hash over the finished fields —
        // ONE hashing path (`compute_integrity`) is shared by `from_intent` and `verify_integrity`.
        let base = ProbeObservationReceipt {
            observation_id,
            intent_id,
            review_id,
            probe_id,
            hypothesis_id,
            observation_text,
            observation_status,
            authority: ObservationAuthority::ObservationOnly,
            evidence_refs,
            created_from_intent_trace,
            integrity_hash: 0,
        };
        ProbeObservationReceipt {
            integrity_hash: base.compute_integrity(),
            ..base
        }
    }

    /// Deterministic integrity hash over every field EXCEPT `integrity_hash` itself (length-prefixed
    /// strings so distinct observations cannot collide by re-grouping bytes).
    fn compute_integrity(&self) -> u64 {
        let mut h = FNV_OFFSET;
        h = fnv_u64(h, self.observation_id);
        h = fnv_u64(h, self.intent_id);
        h = fnv_u64(h, self.review_id);
        h = fnv_u64(h, self.probe_id);
        h = fnv_u64(h, self.hypothesis_id);
        h = fnv_str(h, &self.observation_text);
        h = fnv_str(h, self.observation_status.token());
        h = fnv_str(h, self.authority.token());
        h = fnv_u64(h, self.evidence_refs.len() as u64);
        for ev in &self.evidence_refs {
            h = fnv_u64(h, ev.answer_hash);
            h = fnv_u64(h, ev.memory_hash);
            h = fnv_str(h, &ev.source_label);
        }
        h = fnv_u64(h, self.created_from_intent_trace as u64);
        h
    }

    /// Deterministic content id of the observation.
    pub fn observation_id(&self) -> u64 {
        self.observation_id
    }

    /// The id of the execution intent this observation was recorded from (provenance).
    pub fn intent_id(&self) -> u64 {
        self.intent_id
    }

    /// The id of the governance review (provenance).
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

    /// The CLAIMED observation text — quarantined data only. It does not imply the probe ran and
    /// holds no authority.
    pub fn observation_text(&self) -> &str {
        &self.observation_text
    }

    /// The machine-checkable quarantine disposition (read-only — it cannot be flipped to `recorded`
    /// after the fact).
    pub fn observation_status(&self) -> ObservationStatus {
        self.observation_status
    }

    /// Always [`ObservationAuthority::ObservationOnly`] — an observation never carries
    /// claim/evidence/verifier authority.
    pub fn authority(&self) -> ObservationAuthority {
        self.authority
    }

    /// The receipts the originating hypothesis cited (carried through as provenance, never as
    /// evidence the observation itself produces).
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// Whether the originating hypothesis was derived from a trace/receipt (carried through the
    /// hypothesis -> probe -> review -> intent -> observation chain).
    pub fn created_from_intent_trace(&self) -> bool {
        self.created_from_intent_trace
    }

    /// The deterministic integrity hash binding every field of this observation.
    pub fn integrity_hash(&self) -> u64 {
        self.integrity_hash
    }

    /// Whether this observation may be used for the given purpose. Always `false` for any forbidden
    /// use: an observation is never truth, evidence, or a mutator. It inherits the canonical
    /// [`FORBIDDEN_USES`] quarantine, so it can never become a claim or ground an answer.
    pub fn permits(&self, use_name: &str) -> bool {
        !FORBIDDEN_USES.contains(&use_name)
    }

    /// Re-derive the integrity hash from this observation's OWN fields and confirm it matches.
    /// Because an observation is born only from [`from_intent`] (private fields, no `Deserialize`),
    /// it is consistent by construction; this is an explicit, auditable assertion of that binding —
    /// used to prove a replay was faithful. It grants no authority.
    pub fn verify_integrity(&self) -> Result<(), ObservationError> {
        if self.compute_integrity() == self.integrity_hash {
            Ok(())
        } else {
            Err(ObservationError::IntegrityMismatch)
        }
    }
}

/// Deterministic id of the observation (FNV-1a over its defining fields, length-prefixed).
fn derive_observation_id(
    intent_id: u64,
    review_id: u64,
    probe_id: u64,
    hypothesis_id: u64,
    observation_text: &str,
    observation_status: ObservationStatus,
) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_u64(h, intent_id);
    h = fnv_u64(h, review_id);
    h = fnv_u64(h, probe_id);
    h = fnv_u64(h, hypothesis_id);
    h = fnv_str(h, observation_text);
    h = fnv_str(h, observation_status.token());
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        propose, HypothesisSpec, ProbeRequest, ReviewDecision, ReviewReceipt, ReviewerAuthority,
    };

    fn ev(label: &str) -> EvidenceRef {
        EvidenceRef {
            answer_hash: 0x1111_2222_3333_4444,
            memory_hash: 0x5555_6666_7777_8888,
            source_label: label.to_string(),
        }
    }

    fn intent_with(authority: ReviewerAuthority, decision: ReviewDecision) -> ProbeExecutionIntent {
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
        ProbeExecutionIntent::from_review(
            &ReviewReceipt::decide(&probe, authority, decision).unwrap(),
        )
    }

    // execution_status NotExecuted (queued, automated-scope approval).
    fn not_executed_intent() -> ProbeExecutionIntent {
        intent_with(ReviewerAuthority::Automated, ReviewDecision::Approved)
    }

    // execution_status RequiresOperator (approved by a human authority).
    fn requires_operator_intent() -> ProbeExecutionIntent {
        intent_with(ReviewerAuthority::Human, ReviewDecision::Approved)
    }

    // execution_status Blocked (rejected review).
    fn blocked_intent() -> ProbeExecutionIntent {
        intent_with(ReviewerAuthority::Governance, ReviewDecision::Rejected)
    }

    fn observe(intent: &ProbeExecutionIntent) -> ProbeObservationReceipt {
        ProbeObservationReceipt::from_intent(intent, "observed: something happened")
    }

    #[test]
    fn observation_derived_only_from_execution_intent() {
        // An observation is recorded from an intent and cites the intent_id, review_id, probe_id,
        // hypothesis_id, and source EvidenceRefs. (A ProbeExecutionIntent only exists via
        // from_review on a real receipt, so an observation is always derived from a valid intent.)
        let intent = requires_operator_intent();
        let obs = observe(&intent);
        assert_eq!(obs.intent_id(), intent.intent_id());
        assert_eq!(obs.review_id(), intent.review_id());
        assert_eq!(obs.probe_id(), intent.probe_id());
        assert_eq!(obs.hypothesis_id(), intent.hypothesis_id());
        assert_eq!(obs.evidence_refs(), intent.evidence_refs());
        assert_eq!(
            obs.created_from_intent_trace(),
            intent.created_from_review_trace()
        );
        assert_ne!(obs.observation_id(), 0);
        obs.verify_integrity().unwrap();
    }

    #[test]
    fn not_executed_intent_cannot_record_observation() {
        // A not_executed intent yields a `rejected` observation — there is nothing legitimate to
        // record, and it is NEVER recorded.
        let obs = observe(&not_executed_intent());
        assert_eq!(obs.observation_status(), ObservationStatus::Rejected);
        assert_ne!(obs.observation_status(), ObservationStatus::Recorded);
    }

    #[test]
    fn blocked_intent_cannot_record_observation() {
        // A blocked intent yields a `rejected` observation — a blocked probe must never produce a
        // recorded observation.
        let obs = observe(&blocked_intent());
        assert_eq!(obs.observation_status(), ObservationStatus::Rejected);
        assert_ne!(obs.observation_status(), ObservationStatus::Recorded);
    }

    #[test]
    fn requires_operator_intent_requires_review() {
        // A requires_operator intent yields a `requires_review` observation — a human/governance
        // must review it; it is never recorded automatically.
        let obs = observe(&requires_operator_intent());
        assert_eq!(obs.observation_status(), ObservationStatus::RequiresReview);
        assert_ne!(obs.observation_status(), ObservationStatus::Recorded);
    }

    #[test]
    fn no_intent_disposition_yields_recorded() {
        // THE QUARANTINE: at HYP-4 no intent disposition can produce a `recorded` observation,
        // because no verifier/governance promotion path exists yet. If from_execution_status ever
        // mapped an intent to Recorded, this fails. (A second, independent covering test of the
        // recorded-quarantine, so disabling any one test cannot silently open the promotion hole.)
        for intent in [
            not_executed_intent(),
            requires_operator_intent(),
            blocked_intent(),
        ] {
            assert_ne!(
                observe(&intent).observation_status(),
                ObservationStatus::Recorded,
                "nothing can be recorded at HYP-4"
            );
        }
    }

    #[test]
    fn observation_has_observation_only_authority() {
        // Every observation is observation_only — the only authority the type can express.
        let obs = observe(&requires_operator_intent());
        assert_eq!(obs.authority(), ObservationAuthority::ObservationOnly);
    }

    #[test]
    fn observation_authority_has_exactly_one_variant() {
        // The single-variant guarantee is enforced by the COMPILER, not a grep: this match has no
        // wildcard arm, so adding ANY second `ObservationAuthority` variant makes it non-exhaustive
        // (E0004) and the crate stops compiling. "An observation carries no other authority" can
        // therefore never silently regress.
        let a = ObservationAuthority::ObservationOnly;
        match a {
            ObservationAuthority::ObservationOnly => {}
        }
        assert_eq!(a, ObservationAuthority::ObservationOnly);
    }

    #[test]
    fn observation_cannot_be_evidence() {
        // An observation inherits the canonical forbidden-uses quarantine: it can never ground a
        // claim or serve as evidence. (Structurally there is also no API turning an observation into
        // a Claim/EvidenceRef/ProofObject, and the production crate has no verifier dependency.)
        let obs = observe(&requires_operator_intent());
        for canonical in [
            "ground_claim",
            "serve_as_evidence",
            "mutate_reading_memory",
            "alter_verifier_receipt",
            "change_training_gate",
            "bypass_codec_or_governance",
        ] {
            assert!(!obs.permits(canonical), "{canonical} must be forbidden");
        }
        assert!(obs.permits("quarantine_observation"));
    }

    #[test]
    fn observation_hash_is_deterministic() {
        // Deterministic: identical (intent, text) yield identical observation, id, and hash.
        let intent = requires_operator_intent();
        let a = ProbeObservationReceipt::from_intent(&intent, "obs text");
        let b = ProbeObservationReceipt::from_intent(&intent, "obs text");
        assert_eq!(a, b);
        assert_eq!(a.observation_id(), b.observation_id());
        assert_eq!(a.integrity_hash(), b.integrity_hash());
        // Different observation text changes the id and the hash.
        let c = ProbeObservationReceipt::from_intent(&intent, "different obs text");
        assert_ne!(a.observation_id(), c.observation_id());
        assert_ne!(a.integrity_hash(), c.integrity_hash());
        // A different intent disposition (blocked vs requires_operator) changes the status, id, hash.
        let d = observe(&blocked_intent());
        assert_ne!(a.observation_status(), d.observation_status());
        assert_ne!(a.observation_id(), d.observation_id());
    }

    #[test]
    fn observation_replay_reproduces_same_record() {
        // A trace is the INPUTS: the hypothesis spec plus the reviewer authority and decision (all
        // deserializable) plus the observation text. Replay re-derives the identical observation by
        // re-running propose -> decide -> from_review -> from_intent; the observation itself
        // serializes (to emit a trace) but is never deserialized.
        let spec = HypothesisSpec {
            statement: "replayable observation".to_string(),
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
        let text = "observed: the span was re-read";
        let build = |spec: &HypothesisSpec, a: ReviewerAuthority, d: ReviewDecision| {
            let probe = ProbeRequest::from_hypothesis(&propose(spec.clone()).unwrap());
            let intent =
                ProbeExecutionIntent::from_review(&ReviewReceipt::decide(&probe, a, d).unwrap());
            ProbeObservationReceipt::from_intent(&intent, text)
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

        assert_eq!(original, replayed, "replay reproduces the observation");
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replayed).unwrap()
        );
        replayed.verify_integrity().unwrap();
    }

    #[test]
    fn forged_observation_cannot_be_constructed() {
        // The disposition cannot be bypassed by building a raw struct: fields are private, there is
        // no public constructor or setter, and there is no Deserialize (the compile_fail doctest
        // proves it). The ONLY way to an observation is from_intent, which always DERIVES the
        // disposition from the intent and NEVER yields Recorded — so a forged `recorded` observation
        // cannot exist. (A second, independent covering test of the no-forgery property.)
        for intent in [
            not_executed_intent(),
            requires_operator_intent(),
            blocked_intent(),
        ] {
            let obs = observe(&intent);
            assert_ne!(obs.observation_status(), ObservationStatus::Recorded);
            assert_eq!(obs.authority(), ObservationAuthority::ObservationOnly);
        }
    }

    #[test]
    fn observation_does_not_change_training_gate() {
        // Quarantining an observation is orthogonal to P12: the training decision before and after
        // is identical — still training_not_justified.
        let before = reading_train_gate::decide(&[], &[]);
        let _obs = observe(&requires_operator_intent());
        let after = reading_train_gate::decide(&[], &[]);
        assert_eq!(before, after);
        assert!(
            !after.training_justified,
            "an observation cannot change the training verdict"
        );
    }

    #[test]
    fn observation_does_not_change_verifier_receipt() {
        // Recording an observation from an intent whose hypothesis cites a real receipt leaves the
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
        let intent = ProbeExecutionIntent::from_review(
            &ReviewReceipt::decide(
                &probe,
                ReviewerAuthority::Governance,
                ReviewDecision::Approved,
            )
            .unwrap(),
        );
        let obs = ProbeObservationReceipt::from_intent(&intent, "observed result");
        assert_eq!(obs.evidence_refs(), vec![cite].as_slice());

        let after = reading_cli::verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn observation_tokens_are_machine_checkable() {
        assert_eq!(ObservationStatus::Recorded.token(), "recorded");
        assert_eq!(ObservationStatus::Rejected.token(), "rejected");
        assert_eq!(ObservationStatus::RequiresReview.token(), "requires_review");
        assert_eq!(
            ObservationAuthority::ObservationOnly.token(),
            "observation_only"
        );
        assert_eq!(
            ObservationError::IntegrityMismatch.to_string(),
            "observation receipt integrity hash mismatch"
        );
    }
}
