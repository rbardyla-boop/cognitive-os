//! Probe queue / human-review boundary (HYP-1).
//!
//! HYP-0 lets a hypothesis RECOMMEND a probe. HYP-1 turns that recommendation into a
//! deterministic, inert [`ProbeRequest`] queue item with an explicit, machine-checkable
//! review [`ProbeStatus`] — WITHOUT executing the probe or mutating anything.
//!
//!   Hypothesis proposes a probe.  HYP-1 queues or blocks it.  Human/governance decides
//!   execution.  Nothing executes automatically.
//!
//! A `ProbeRequest` follows the same structural quarantine as a `HypothesisPacket`: its
//! fields are private with read-only accessors, it derives `Serialize` but NOT
//! `Deserialize` (the *compiler* enforces this — see the `compile_fail` doctests), and it
//! is minted ONLY by [`ProbeRequest::from_hypothesis`]. So its risk/reversibility-derived
//! status cannot be hand-set, it cannot be forged off the wire, and it cannot become
//! evidence. It executes nothing — there is no probe-execution code in this module or crate.

use serde::Serialize;

use crate::{
    fnv_i64, fnv_str, fnv_u64, EvidenceRef, HypothesisPacket, ProbeClearance, FNV_OFFSET,
    FORBIDDEN_USES, HIGH_RISK, LOW_REVERSIBILITY,
};

/// The review status of a queued probe — the MACHINE-CHECKABLE gate on execution, never
/// prose. Derived 1:1 from the probe's [`ProbeClearance`], so it inherits the same
/// deterministic risk/reversibility decision the packet already made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ProbeStatus {
    /// Low-risk and reversible: ELIGIBLE to be executed once a human/governance picks it
    /// up. Eligible is not executed — this layer runs nothing.
    #[serde(rename = "queued")]
    Queued,
    /// High-risk OR hard-to-reverse: a human must review before it can be considered.
    #[serde(rename = "human_review_required")]
    HumanReviewRequired,
    /// High-risk AND irreversible: must never be queued for execution.
    #[serde(rename = "blocked")]
    Blocked,
}

impl ProbeStatus {
    /// Map a probe clearance to a queue status. Total and exhaustive (no wildcard), so a new
    /// `ProbeClearance` variant forces an explicit mapping here (E0004) rather than silently
    /// defaulting to an executable status.
    fn from_clearance(clearance: ProbeClearance) -> Self {
        match clearance {
            ProbeClearance::Allowed => ProbeStatus::Queued,
            ProbeClearance::HumanReviewRequired => ProbeStatus::HumanReviewRequired,
            ProbeClearance::Blocked => ProbeStatus::Blocked,
        }
    }

    /// Whether this status permits execution WITHOUT a human first acting. Only a plain
    /// `Queued` probe is eligible; review-required and blocked are not. The match is
    /// exhaustive with NO wildcard, so adding a status variant cannot silently become
    /// eligible — it stops compiling until the eligibility decision is made explicit.
    pub fn is_execution_eligible(self) -> bool {
        match self {
            ProbeStatus::Queued => true,
            ProbeStatus::HumanReviewRequired => false,
            ProbeStatus::Blocked => false,
        }
    }

    /// A machine-checkable token (never prose) for the status.
    pub fn token(self) -> &'static str {
        match self {
            ProbeStatus::Queued => "queued",
            ProbeStatus::HumanReviewRequired => "human_review_required",
            ProbeStatus::Blocked => "blocked",
        }
    }
}

/// Why a probe got its status — a machine-checkable classification, never prose. It carries
/// the high-risk-vs-irreversible detail that the three-way status collapses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ProbeReason {
    /// Risk below threshold and reversible: safe to queue.
    #[serde(rename = "low_risk_reversible")]
    LowRiskReversible,
    /// Risk at/above threshold (but reversible).
    #[serde(rename = "high_risk")]
    HighRisk,
    /// Hard to reverse (but not high-risk).
    #[serde(rename = "hard_to_reverse")]
    HardToReverse,
    /// High-risk AND hard to reverse: the blocked case.
    #[serde(rename = "high_risk_and_irreversible")]
    HighRiskAndIrreversible,
}

impl ProbeReason {
    /// Deterministic classification from the same thresholds [`ProbeClearance::classify`]
    /// uses (integer compare only — no floats, no model). Exhaustive, no wildcard.
    fn classify(risk: i64, reversibility: i64) -> Self {
        let high_risk = risk >= HIGH_RISK;
        let irreversible = reversibility <= LOW_REVERSIBILITY;
        match (high_risk, irreversible) {
            (true, true) => ProbeReason::HighRiskAndIrreversible,
            (true, false) => ProbeReason::HighRisk,
            (false, true) => ProbeReason::HardToReverse,
            (false, false) => ProbeReason::LowRiskReversible,
        }
    }

    /// The status this reason implies. Lets a caller (or a cross-check test) confirm the
    /// reason detail and the status (derived independently from the packet's clearance)
    /// agree. Exhaustive, no wildcard.
    pub fn status(self) -> ProbeStatus {
        match self {
            ProbeReason::LowRiskReversible => ProbeStatus::Queued,
            ProbeReason::HighRisk | ProbeReason::HardToReverse => ProbeStatus::HumanReviewRequired,
            ProbeReason::HighRiskAndIrreversible => ProbeStatus::Blocked,
        }
    }

    /// A machine-checkable token (never prose) for the reason.
    pub fn token(self) -> &'static str {
        match self {
            ProbeReason::LowRiskReversible => "low_risk_reversible",
            ProbeReason::HighRisk => "high_risk",
            ProbeReason::HardToReverse => "hard_to_reverse",
            ProbeReason::HighRiskAndIrreversible => "high_risk_and_irreversible",
        }
    }
}

/// A queued probe derived from a [`HypothesisPacket`]. It records WHAT to test, the source
/// hypothesis and receipts it came from, and WHETHER a human must review it — and nothing
/// else. It is inert: it executes no probe, holds no authority, and cannot become evidence.
///
/// Minted ONLY by [`ProbeRequest::from_hypothesis`]; its fields are private and read-only,
/// and it derives `Serialize` but NOT `Deserialize`. The first example reaches a real
/// request through `propose` + `from_hypothesis`; the `compile_fail` example proves the type
/// cannot be deserialized — so a forged request with a hand-set `Queued` status on a
/// high-risk probe cannot enter the system off the wire. If either property regresses,
/// `cargo test` fails.
///
/// ```
/// let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str(
///     r#"{"statement":"s","prior":1,"uncertainty":1,"test_cost":0,"risk":1,"reversibility":1,"evidence_inputs":[],"probe_description":"p"}"#
/// ).unwrap();
/// let packet = hypothesis_layer::propose(spec).unwrap();
/// let req = hypothesis_layer::ProbeRequest::from_hypothesis(&packet);
/// let _id: u64 = req.probe_id();
/// ```
///
/// ```compile_fail
/// // A ProbeRequest implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::ProbeRequest = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProbeRequest {
    // Private + read-only: a request is built ONLY by `from_hypothesis`, so its status is
    // always DERIVED from the source packet's risk/reversibility — never hand-set.
    probe_id: u64,
    hypothesis_id: u64,
    evidence_refs: Vec<EvidenceRef>,
    probe_text: String,
    risk: i64,
    reversibility: i64,
    status: ProbeStatus,
    reason: ProbeReason,
    created_from_trace: bool,
}

impl ProbeRequest {
    /// Derive an inert probe request from a (necessarily valid) [`HypothesisPacket`]. The
    /// only way to obtain a `&HypothesisPacket` is [`crate::propose`], which validates its
    /// inputs — so a request is, by construction, derived only from a valid hypothesis. The
    /// status is taken from the packet's already-computed clearance (the single source of
    /// truth); the reason is the finer risk/reversibility detail. Pure and deterministic.
    pub fn from_hypothesis(packet: &HypothesisPacket) -> ProbeRequest {
        let risk = packet.risk();
        let reversibility = packet.reversibility();
        // Status comes from the packet's canonical clearance — HYP-1 RESPECTS the HYP-0
        // decision, it does not recompute a competing one.
        let status = ProbeStatus::from_clearance(packet.recommended_probe().clearance());
        let reason = ProbeReason::classify(risk, reversibility);
        let probe_text = packet.recommended_probe().description().to_string();
        let hypothesis_id = packet.hypothesis_id();
        let evidence_refs = packet.evidence_inputs().to_vec();
        let probe_id = derive_probe_id(
            hypothesis_id,
            &probe_text,
            risk,
            reversibility,
            &evidence_refs,
        );
        ProbeRequest {
            probe_id,
            hypothesis_id,
            evidence_refs,
            probe_text,
            risk,
            reversibility,
            status,
            reason,
            created_from_trace: packet.created_from_trace(),
        }
    }

    /// Deterministic content id of the request (FNV-1a over its defining fields).
    pub fn probe_id(&self) -> u64 {
        self.probe_id
    }

    /// The id of the hypothesis this probe tests (provenance).
    pub fn hypothesis_id(&self) -> u64 {
        self.hypothesis_id
    }

    /// The receipts the source hypothesis cited (carried through as provenance, never as
    /// evidence the request itself produces).
    pub fn evidence_refs(&self) -> &[EvidenceRef] {
        &self.evidence_refs
    }

    /// What the probe would do (human-readable; this layer never runs it).
    pub fn probe_text(&self) -> &str {
        &self.probe_text
    }

    /// Risk of the probe, per-mille.
    pub fn risk(&self) -> i64 {
        self.risk
    }

    /// Reversibility of the probe, per-mille.
    pub fn reversibility(&self) -> i64 {
        self.reversibility
    }

    /// The machine-checkable review status (read-only — it cannot be downgraded to
    /// executable after the fact).
    pub fn status(&self) -> ProbeStatus {
        self.status
    }

    /// The machine-checkable reason for the status.
    pub fn reason(&self) -> ProbeReason {
        self.reason
    }

    /// Whether the source hypothesis was derived from a trace/receipt.
    pub fn created_from_trace(&self) -> bool {
        self.created_from_trace
    }

    /// Whether this probe may be executed WITHOUT a human first acting. True only for a
    /// `Queued` status — a blocked or review-required probe is never eligible.
    pub fn is_execution_eligible(&self) -> bool {
        self.status.is_execution_eligible()
    }

    /// Whether this request may be used for the given purpose. Always `false` for any
    /// forbidden use: a probe request is never truth, evidence, or a mutator. It inherits the
    /// canonical [`FORBIDDEN_USES`] quarantine (identity-pinned by the hypothesis tests), so
    /// it can never become a claim or ground an answer.
    pub fn permits(&self, use_name: &str) -> bool {
        !FORBIDDEN_USES.contains(&use_name)
    }
}

/// A deterministic, content-ordered queue of probe requests derived from hypotheses. The
/// order is canonical — by `probe_id`, then `hypothesis_id` — so it is INSERTION-ORDER
/// INDEPENDENT and reproduces exactly on replay (the inputs are content-addressed, never
/// wall-clock-ordered). The queue executes nothing.
///
/// Like a request, a queue is minted only by [`ProbeQueue::from_hypotheses`], has a private
/// field, and derives `Serialize` but NOT `Deserialize`.
///
/// ```
/// let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str(
///     r#"{"statement":"s","prior":1,"uncertainty":1,"test_cost":0,"risk":1,"reversibility":1,"evidence_inputs":[],"probe_description":"p"}"#
/// ).unwrap();
/// let packet = hypothesis_layer::propose(spec).unwrap();
/// let queue = hypothesis_layer::ProbeQueue::from_hypotheses(&[packet]);
/// let _n = queue.requests().len();
/// ```
///
/// ```compile_fail
/// // A ProbeQueue implements no Deserialize, so this does NOT compile.
/// let _: hypothesis_layer::ProbeQueue = serde_json::from_str("{}").unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProbeQueue {
    requests: Vec<ProbeRequest>,
}

impl ProbeQueue {
    /// Derive one [`ProbeRequest`] per hypothesis and order them canonically. Pure and
    /// deterministic: the same hypotheses (in any input order) yield the identical queue.
    pub fn from_hypotheses(packets: &[HypothesisPacket]) -> ProbeQueue {
        let mut requests: Vec<ProbeRequest> =
            packets.iter().map(ProbeRequest::from_hypothesis).collect();
        // Canonical content order → insertion-order independent and replay-stable.
        requests.sort_by(|a, b| {
            a.probe_id
                .cmp(&b.probe_id)
                .then(a.hypothesis_id.cmp(&b.hypothesis_id))
        });
        ProbeQueue { requests }
    }

    /// The full queue in canonical order (includes blocked and review-required requests for
    /// audit/trace — their status marks them, they are simply never execution-eligible).
    pub fn requests(&self) -> &[ProbeRequest] {
        &self.requests
    }

    /// The requests eligible for execution WITHOUT human review — exactly the `Queued` ones.
    /// Blocked and human-review-required requests are intentionally excluded, so a blocked
    /// probe can never be picked up as executable.
    pub fn execution_eligible(&self) -> Vec<&ProbeRequest> {
        self.requests
            .iter()
            .filter(|r| r.is_execution_eligible())
            .collect()
    }
}

/// Deterministic content id over the request's defining fields (length-prefixed strings so
/// distinct requests cannot collide by re-grouping bytes). Excludes status/reason, which are
/// a pure function of risk/reversibility.
fn derive_probe_id(
    hypothesis_id: u64,
    probe_text: &str,
    risk: i64,
    reversibility: i64,
    evidence_refs: &[EvidenceRef],
) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_u64(h, hypothesis_id);
    h = fnv_str(h, probe_text);
    h = fnv_i64(h, risk);
    h = fnv_i64(h, reversibility);
    h = fnv_u64(h, evidence_refs.len() as u64);
    for ev in evidence_refs {
        h = fnv_u64(h, ev.answer_hash);
        h = fnv_u64(h, ev.memory_hash);
        h = fnv_str(h, &ev.source_label);
    }
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

    // A supported, low-risk, reversible hypothesis (probe clearance Allowed → Queued).
    fn supported_spec() -> HypothesisSpec {
        HypothesisSpec {
            statement: "Bridge B reopened because the storm weakened.".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![ev("run.json")],
            probe_description: "Re-read the maintenance log span for Bridge B.".to_string(),
        }
    }

    fn packet_with(risk: i64, reversibility: i64, statement: &str) -> HypothesisPacket {
        let mut spec = supported_spec();
        spec.risk = risk;
        spec.reversibility = reversibility;
        spec.statement = statement.to_string();
        propose(spec).unwrap()
    }

    #[test]
    fn probe_request_derived_from_hypothesis_packet() {
        // A request is derived from a valid packet and cites the hypothesis_id + the source
        // EvidenceRefs + the recommended probe text. (A packet only exists via propose(),
        // which validates — so a request is always derived from a valid hypothesis.)
        let packet = propose(supported_spec()).unwrap();
        let req = ProbeRequest::from_hypothesis(&packet);
        assert_eq!(req.hypothesis_id(), packet.hypothesis_id());
        assert_eq!(req.evidence_refs(), packet.evidence_inputs());
        assert_eq!(req.probe_text(), packet.recommended_probe().description());
        assert_eq!(req.risk(), packet.risk());
        assert_eq!(req.reversibility(), packet.reversibility());
        assert_eq!(req.created_from_trace(), packet.created_from_trace());
        assert_ne!(req.probe_id(), 0);
    }

    #[test]
    fn high_risk_probe_requires_human_review() {
        // High risk but reversible → human_review_required, reason high_risk, NOT eligible.
        let packet = packet_with(800, 800, "high-risk probe");
        let req = ProbeRequest::from_hypothesis(&packet);
        assert_eq!(req.status(), ProbeStatus::HumanReviewRequired);
        assert_eq!(req.reason(), ProbeReason::HighRisk);
        assert!(!req.is_execution_eligible());
    }

    #[test]
    fn irreversible_probe_requires_human_review() {
        // Low risk but hard to reverse → human_review_required, reason hard_to_reverse.
        let packet = packet_with(200, 100, "irreversible probe");
        let req = ProbeRequest::from_hypothesis(&packet);
        assert_eq!(req.status(), ProbeStatus::HumanReviewRequired);
        assert_eq!(req.reason(), ProbeReason::HardToReverse);
        assert!(!req.is_execution_eligible());
    }

    #[test]
    fn high_risk_and_irreversible_probe_is_blocked() {
        // High risk AND irreversible → blocked, and never queued as executable.
        let packet = packet_with(900, 100, "dangerous irreversible probe");
        let req = ProbeRequest::from_hypothesis(&packet);
        assert_eq!(req.status(), ProbeStatus::Blocked);
        assert_eq!(req.reason(), ProbeReason::HighRiskAndIrreversible);
        assert!(!req.is_execution_eligible());

        let queue = ProbeQueue::from_hypotheses(&[packet]);
        assert_eq!(queue.requests().len(), 1);
        assert!(
            queue.execution_eligible().is_empty(),
            "a blocked probe cannot be queued as executable"
        );
    }

    #[test]
    fn probe_status_matches_packet_clearance() {
        // The probe-queue status agrees with BOTH the packet's canonical clearance and the
        // independently-derived reason. If any of the three classifiers diverged, this fails.
        for (risk, reversibility) in [(100, 900), (800, 800), (200, 100), (900, 100)] {
            let packet = packet_with(risk, reversibility, "x");
            let req = ProbeRequest::from_hypothesis(&packet);
            assert_eq!(
                req.status(),
                ProbeStatus::from_clearance(packet.recommended_probe().clearance())
            );
            assert_eq!(req.reason().status(), req.status());
        }
    }

    #[test]
    fn forged_status_cannot_be_constructed() {
        // The risk/reversibility check cannot be bypassed by building a raw struct: there is
        // no public constructor or setter (fields are private) and no Deserialize (the
        // compile_fail doctest proves it). The ONLY way to a request is from_hypothesis, which
        // always DERIVES the status — so a high-risk probe can never carry a Queued status.
        let packet = packet_with(900, 100, "dangerous");
        let req = ProbeRequest::from_hypothesis(&packet);
        assert_ne!(req.status(), ProbeStatus::Queued);
        assert!(!req.is_execution_eligible());
    }

    #[test]
    fn probe_queue_order_is_deterministic() {
        // The queue is a pure function of its inputs AND insertion-order independent: the
        // same hypotheses in different input orders produce the identical queue.
        let a = packet_with(100, 900, "alpha");
        let b = packet_with(800, 800, "bravo");
        let c = packet_with(900, 100, "charlie");
        let q1 = ProbeQueue::from_hypotheses(&[a.clone(), b.clone(), c.clone()]);
        let q2 = ProbeQueue::from_hypotheses(&[c, a, b]);
        assert_eq!(q1, q2, "queue order is insertion-order independent");
        // Canonical: sorted by probe_id ascending.
        let ids: Vec<u64> = q1.requests().iter().map(ProbeRequest::probe_id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn replay_reproduces_probe_queue() {
        // A trace is the hypothesis INPUTS (specs). Replay = deserialize the specs, re-propose,
        // re-queue → the identical queue. The queue serializes (to emit a trace) but is never
        // deserialized, so replay cannot smuggle a forged request.
        let specs = vec![
            {
                let mut s = supported_spec();
                s.statement = "first".to_string();
                s
            },
            {
                let mut s = supported_spec();
                s.statement = "second".to_string();
                s.risk = 900;
                s.reversibility = 100;
                s
            },
        ];
        let build = |specs: &[HypothesisSpec]| {
            let packets: Vec<HypothesisPacket> =
                specs.iter().cloned().map(|s| propose(s).unwrap()).collect();
            ProbeQueue::from_hypotheses(&packets)
        };
        let original = build(&specs);

        // Round-trip the trace inputs, then rebuild.
        let specs_json = serde_json::to_string(&specs).unwrap();
        let restored: Vec<HypothesisSpec> = serde_json::from_str(&specs_json).unwrap();
        let replayed = build(&restored);
        assert_eq!(original, replayed, "replay reproduces the queue");

        // The queue emits a stable trace, and re-deriving yields identical bytes.
        assert_eq!(
            serde_json::to_string(&original).unwrap(),
            serde_json::to_string(&replayed).unwrap()
        );
    }

    #[test]
    fn probe_request_cannot_be_evidence() {
        // A request inherits the canonical forbidden-uses quarantine: it can never ground a
        // claim or serve as evidence. (Structurally there is also no API turning a request
        // into a Claim/EvidenceRef/ProofObject, and the production crate has no verifier dep.)
        let packet = propose(supported_spec()).unwrap();
        let req = ProbeRequest::from_hypothesis(&packet);
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
        // A non-forbidden use (e.g. scheduling the probe for review) is permitted.
        assert!(req.permits("schedule_for_review"));
    }

    #[test]
    fn probe_queue_does_not_change_training_gate() {
        // Building a probe queue is orthogonal to P12: the training decision before and after
        // is identical — still training_not_justified. (Production has no train-gate
        // dependency; this dev-only test proves the non-interference.)
        let before = reading_train_gate::decide(&[], &[]);
        let packets = vec![packet_with(100, 900, "one"), packet_with(900, 100, "two")];
        let _queue = ProbeQueue::from_hypotheses(&packets);
        let after = reading_train_gate::decide(&[], &[]);
        assert_eq!(before, after);
        assert!(
            !after.training_justified,
            "a probe queue cannot change the training verdict"
        );
    }

    #[test]
    fn probe_queue_does_not_change_verifier_receipt() {
        // Deriving a probe queue from a hypothesis that cites a real receipt leaves the
        // verifier receipt byte-identical — the layer reads hashes, never the object, and
        // executes nothing.
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
        let mut spec = supported_spec();
        spec.evidence_inputs = vec![cite.clone()];
        let packet = propose(spec).unwrap();
        let queue = ProbeQueue::from_hypotheses(&[packet]);
        // The request carries the cited receipt as provenance...
        assert_eq!(queue.requests()[0].evidence_refs(), vec![cite].as_slice());

        // ...and the receipt re-verifies identically (unchanged).
        let after = reading_cli::verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn probe_status_tokens_are_machine_checkable() {
        // The status and reason expose stable machine-checkable tokens (not prose), so
        // human_review_required is a checkable value, never free text.
        assert_eq!(ProbeStatus::Queued.token(), "queued");
        assert_eq!(
            ProbeStatus::HumanReviewRequired.token(),
            "human_review_required"
        );
        assert_eq!(ProbeStatus::Blocked.token(), "blocked");
        assert!(ProbeStatus::Queued.is_execution_eligible());
        assert!(!ProbeStatus::HumanReviewRequired.is_execution_eligible());
        assert!(!ProbeStatus::Blocked.is_execution_eligible());
    }
}
