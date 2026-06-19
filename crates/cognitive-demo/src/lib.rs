//! cognitive-demo — INT-0, the end-to-end prototype trace demo.
//!
//! This is the FIRST integration layer over the two frozen tracks. It connects the
//! frozen reading receipt (`reading-cli` @ `reading-track-v0.1`) to the frozen
//! hypothesis/probe/review/intent/observation/promotion-refusal chain
//! (`hypothesis-layer` @ `hypothesis-track-v0.1`) and produces ONE auditable,
//! deterministic, replayable [`CognitiveTrace`].
//!
//!   Reading verifies.  Hypothesis proposes.  Probe queue classifies.  Governance
//!   reviews.  Execution intent records.  Observation quarantines.  Promotion refuses.
//!   Nothing becomes evidence.  Nothing trains.
//!
//! The trace is a TYPED, REPLAYABLE, VERIFIER-CHECKED record — not a hidden
//! chain-of-thought. Every step is a typed object with its own authority limits, its own
//! deterministic content id, and (downstream of the hypothesis) an integrity hash; the
//! whole flow is a pure function of fixed inputs, so re-running it reproduces the trace
//! byte-for-byte. The trace answers, machine-checkably: what did it read, what did it
//! verify, what did it guess, why, what probe it recommended, whether the probe was
//! approved, whether anything executed, whether anything became evidence, and whether
//! training opened — and the honest answers are *no execution, no evidence, no training*.
//!
//! INT-0 GRANTS NO NEW AUTHORITY. It calls only the public, inert APIs of the frozen
//! crates; it edits neither. The reading receipt is consulted by HASH (an [`EvidenceRef`],
//! never a handle into memory); the hypothesis layer can only propose; the promotion
//! request is refused. The demo holds no executor, writes no probe result, mutates no
//! memory, and moves no training verdict — the release gate proves each refusal from the
//! trace's own serialized output.

#![forbid(unsafe_code)]

use hypothesis_layer::{
    propose, Authority, EvidenceRef, HypothesisError, HypothesisSpec, ProbeExecutionIntent,
    ProbeObservationReceipt, ProbeRequest, PromotionRequest, PromotionTarget, ReviewDecision,
    ReviewError, ReviewReceipt, ReviewerAuthority,
};
use reading_cli::{produce_run, verify_file, CliError};
use reading_train_gate::decide;
use serde::Serialize;

/// What can go wrong building the end-to-end trace. Every failure is explicit; nothing is
/// silently coerced or fabricated. The first three wrap a frozen-crate error; the last two
/// are INT-0's own provenance invariants (a trace that did not start from a verified receipt,
/// or whose hypothesis did not cite that receipt, is not a faithful end-to-end trace).
#[derive(Debug)]
pub enum TraceError {
    /// The reading pipeline rejected the (fixed, valid) inputs — produce_run / verify_file.
    Reading(CliError),
    /// The hypothesis layer rejected the (fixed, valid) spec.
    Hypothesis(HypothesisError),
    /// The governance review could not be recorded (policy refused the decision).
    Review(ReviewError),
    /// The reading receipt did not pass verification, so the trace cannot start from it.
    VerifierRejected,
    /// The hypothesis did not cite the reading receipt by hash (the provenance invariant).
    CitationMismatch,
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceError::Reading(e) => write!(f, "reading pipeline error: {e}"),
            TraceError::Hypothesis(e) => write!(f, "hypothesis error: {e}"),
            TraceError::Review(e) => write!(f, "review error: {e}"),
            TraceError::VerifierRejected => {
                write!(f, "the reading receipt did not pass verification")
            }
            TraceError::CitationMismatch => {
                write!(f, "the hypothesis did not cite the reading receipt by hash")
            }
        }
    }
}

impl std::error::Error for TraceError {}

/// The fixed reading inputs the demo runs: a real (in-memory) document folder, a question,
/// and an untrusted reading plan that finalizes a verifier-approved answer. Exposed so a
/// consumer/test can re-run the SAME reading receipt independently (e.g. to prove the trace
/// leaves the verifier receipt byte-identical). Pure: it returns owned literals only.
pub fn demo_inputs() -> (Vec<(String, String)>, String, String) {
    let documents = vec![(
        "report.txt".to_string(),
        "Bridge A was damaged. Bridge B stayed open.".to_string(),
    )];
    let question = "Which bridge is open?".to_string();
    let plan = r#"[
        {"action":"inspect_corpus"},
        {"action":"read_span","span_id":1},
        {"action":"extract_claim","statement":"Bridge B stayed open.","source_span_ids":[1]},
        {"action":"synthesize","answer_text":"Bridge B stayed open.","supporting_claims":[0]}
    ]"#
    .to_string();
    (documents, question, plan)
}

/// One auditable end-to-end trace of a single bounded cognitive path: from a verified
/// reading receipt, through the hypothesis chain, to a refused promotion. It is an inert
/// RECORD — it derives `Serialize` (to emit the trace) but NOT `Deserialize` (it is never
/// read back as authority), holds no executor/verifier/memory handle, and exposes no method
/// that returns claim/evidence authority. It is minted ONLY by [`CognitiveTrace::demo`] /
/// [`CognitiveTrace::build`], which compute every field from the frozen crates' real outputs;
/// nothing is hand-set. Its fields are private and read-only via accessors, so a trace cannot
/// be mutated into claiming an execution, an evidence promotion, or an opened training gate
/// after the fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CognitiveTrace {
    schema: String,

    // --- Reading (what it read + what it verified) ---
    reading_question: String,
    reading_answer: String,
    reading_answer_hash: u64,
    reading_memory_hash: u64,
    reading_structure_hash: Option<u64>,
    reading_passed: bool,
    reading_integrity: String,

    // --- Hypothesis (what it guessed + why, citing the receipt by hash) ---
    hypothesis_id: u64,
    hypothesis_statement: String,
    hypothesis_authority: String,
    hypothesis_created_from_trace: bool,
    hypothesis_expected_utility: i64,
    cited_answer_hash: u64,
    cited_memory_hash: u64,

    // --- Probe request (what to test) ---
    probe_id: u64,
    probe_status: String,
    probe_reason: String,
    probe_execution_eligible: bool,

    // --- Governance review (approved, but approval is not execution) ---
    review_id: u64,
    review_decision: String,
    review_authority: String,
    review_reason: String,
    review_integrity_hash: u64,

    // --- Execution intent (records intent; executes nothing) ---
    intent_id: u64,
    execution_status: String,
    execution_reason: String,
    intent_requires_operator: bool,
    intent_blocked: bool,
    intent_integrity_hash: u64,

    // --- Observation (quarantined; never recorded, never evidence) ---
    observation_id: u64,
    observation_status: String,
    observation_authority: String,
    observation_integrity_hash: u64,

    // --- Promotion request (refused; promotes nothing) ---
    promotion_id: u64,
    promotion_target: String,
    promotion_status: String,
    promotion_reason: String,
    grants_promotion: bool,
    promotion_integrity_hash: u64,

    // --- P12 training gate (unmoved by the whole flow) ---
    training_justified: bool,
    training_gate_unchanged: bool,

    // --- End-to-end verdicts (the machine-checkable answers) ---
    starts_from_verified_receipt: bool,
    hypothesis_cites_receipt: bool,
    chain_linked: bool,
    nothing_executed: bool,
    observation_quarantined: bool,
    promotion_refused: bool,
    nothing_becomes_evidence: bool,
}

impl CognitiveTrace {
    /// The canonical INT-0 demo: run the fixed [`demo_inputs`] through the full pipeline and
    /// record one end-to-end trace. Pure and deterministic — calling it twice yields an
    /// identical trace (replay). It grants no authority, executes no probe, promotes nothing.
    pub fn demo() -> Result<CognitiveTrace, TraceError> {
        let (documents, question, plan) = demo_inputs();
        Self::build(&documents, &question, &plan)
    }

    /// Build the end-to-end trace from explicit inputs. Each downstream object is DERIVED
    /// only from its immediate predecessor (reading receipt → hypothesis → probe → review →
    /// intent → observation → promotion), and every governed field is read from the frozen
    /// crates' real outputs, never supplied. Returns a [`TraceError`] if the reading receipt
    /// does not verify, the hypothesis does not cite it, or any frozen API refuses.
    pub fn build(
        documents: &[(String, String)],
        question: &str,
        plan: &str,
    ) -> Result<CognitiveTrace, TraceError> {
        // P12 verdict BEFORE the flow — the whole trace must leave it unmoved.
        let training_before = decide(&[], &[]);

        // 1. Reading: produce a run and START FROM A VERIFIED RECEIPT (verify_file re-derives
        //    the answer from the plan through the codec and re-runs the verifier).
        let file = produce_run(documents, question, plan).map_err(TraceError::Reading)?;
        let outcome = verify_file(&file).map_err(TraceError::Reading)?;
        if !outcome.receipt.passed {
            return Err(TraceError::VerifierRejected);
        }

        // 2. Hypothesis CITES THE RECEIPT BY HASH — an EvidenceRef carries only the answer +
        //    memory hashes and a label, never a handle into reading memory.
        let cite = EvidenceRef {
            answer_hash: file.answer_hash,
            memory_hash: file.memory_hash,
            source_label: "bridge-run".to_string(),
        };
        let spec = HypothesisSpec {
            statement: "Bridge B reopened because the storm weakened.".to_string(),
            prior: 500,
            uncertainty: 600,
            test_cost: 50,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![cite.clone()],
            probe_description: "Re-read the maintenance log span for Bridge B.".to_string(),
        };
        let packet = propose(spec).map_err(TraceError::Hypothesis)?;
        // Provenance invariant: the hypothesis was derived from the receipt and cites its hashes.
        let cites_receipt = packet.created_from_trace()
            && packet
                .evidence_inputs()
                .iter()
                .any(|e| e.answer_hash == file.answer_hash && e.memory_hash == file.memory_hash);
        if !cites_receipt {
            return Err(TraceError::CitationMismatch);
        }

        // 3..7. The frozen hypothesis chain — each derived ONLY from its predecessor.
        let probe = ProbeRequest::from_hypothesis(&packet);
        let review = ReviewReceipt::decide(
            &probe,
            ReviewerAuthority::Governance,
            ReviewDecision::Approved,
        )
        .map_err(TraceError::Review)?;
        let intent = ProbeExecutionIntent::from_review(&review);
        let observation = ProbeObservationReceipt::from_intent(
            &intent,
            "observed: the maintenance log span was re-read",
        );
        let promotion = PromotionRequest::from_observation(&observation, PromotionTarget::Evidence);

        // Chain linkage: every stage cites its predecessor's deterministic id.
        let chain_linked = probe.hypothesis_id() == packet.hypothesis_id()
            && review.probe_id() == probe.probe_id()
            && review.hypothesis_id() == packet.hypothesis_id()
            && intent.review_id() == review.review_id()
            && intent.probe_id() == probe.probe_id()
            && intent.hypothesis_id() == packet.hypothesis_id()
            && observation.intent_id() == intent.intent_id()
            && observation.review_id() == review.review_id()
            && observation.probe_id() == probe.probe_id()
            && observation.hypothesis_id() == packet.hypothesis_id()
            && promotion.observation_id() == observation.observation_id()
            && promotion.intent_id() == intent.intent_id()
            && promotion.probe_id() == probe.probe_id()
            && promotion.hypothesis_id() == packet.hypothesis_id();

        // P12 verdict AFTER the flow — proven equal and still false.
        let training_after = decide(&[], &[]);
        let training_gate_unchanged = training_before == training_after;

        // --- End-to-end verdicts, computed from the REAL objects (never hand-set) ---
        let starts_from_verified_receipt = outcome.receipt.passed;
        let nothing_executed = intent.execution_status().token() != "executed";
        let observation_quarantined = observation.observation_status().token() != "recorded"
            && observation.authority().token() == "observation_only";
        let promotion_refused = !promotion.grants_promotion();
        let nothing_becomes_evidence = !promotion.grants_promotion()
            && !promotion.permits("serve_as_evidence")
            && promotion.status().token() != "evidence"
            && observation.authority().token() == "observation_only";

        Ok(CognitiveTrace {
            schema: "cognitive-trace-v0.1".to_string(),

            reading_question: file.question.clone(),
            reading_answer: file.answer.clone(),
            reading_answer_hash: file.answer_hash,
            reading_memory_hash: file.memory_hash,
            reading_structure_hash: file.structure_hash,
            reading_passed: outcome.receipt.passed,
            reading_integrity: outcome.integrity.token().to_string(),

            hypothesis_id: packet.hypothesis_id(),
            hypothesis_statement: packet.statement().to_string(),
            hypothesis_authority: authority_token(packet.authority()).to_string(),
            hypothesis_created_from_trace: packet.created_from_trace(),
            hypothesis_expected_utility: packet.expected_utility(),
            cited_answer_hash: cite.answer_hash,
            cited_memory_hash: cite.memory_hash,

            probe_id: probe.probe_id(),
            probe_status: probe.status().token().to_string(),
            probe_reason: probe.reason().token().to_string(),
            probe_execution_eligible: probe.is_execution_eligible(),

            review_id: review.review_id(),
            review_decision: review.decision().token().to_string(),
            review_authority: review.reviewer_authority().token().to_string(),
            review_reason: review.reason_code().token().to_string(),
            review_integrity_hash: review.integrity_hash(),

            intent_id: intent.intent_id(),
            execution_status: intent.execution_status().token().to_string(),
            execution_reason: intent.reason_code().token().to_string(),
            intent_requires_operator: intent.requires_operator(),
            intent_blocked: intent.is_blocked(),
            intent_integrity_hash: intent.integrity_hash(),

            observation_id: observation.observation_id(),
            observation_status: observation.observation_status().token().to_string(),
            observation_authority: observation.authority().token().to_string(),
            observation_integrity_hash: observation.integrity_hash(),

            promotion_id: promotion.promotion_id(),
            promotion_target: promotion.requested_target().token().to_string(),
            promotion_status: promotion.status().token().to_string(),
            promotion_reason: promotion.reason_code().token().to_string(),
            grants_promotion: promotion.grants_promotion(),
            promotion_integrity_hash: promotion.integrity_hash(),

            training_justified: training_after.training_justified,
            training_gate_unchanged,

            starts_from_verified_receipt,
            hypothesis_cites_receipt: cites_receipt,
            chain_linked,
            nothing_executed,
            observation_quarantined,
            promotion_refused,
            nothing_becomes_evidence,
        })
    }

    /// Serialize the trace as pretty JSON (the auditable, machine-checkable record). Pure.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("CognitiveTrace serializes")
    }

    // --- Read-only accessors (the trace grants no authority; these only read recorded data) ---

    /// Whether the trace started from a verifier-approved reading receipt.
    pub fn starts_from_verified_receipt(&self) -> bool {
        self.starts_from_verified_receipt
    }
    /// Whether the hypothesis cites the reading receipt by hash.
    pub fn hypothesis_cites_receipt(&self) -> bool {
        self.hypothesis_cites_receipt
    }
    /// Whether every stage cites its predecessor's id (the chain is linked end to end).
    pub fn chain_linked(&self) -> bool {
        self.chain_linked
    }
    /// Whether nothing executed (the execution intent is a non-running state).
    pub fn nothing_executed(&self) -> bool {
        self.nothing_executed
    }
    /// Whether the observation stayed quarantined (never `recorded`, `observation_only`).
    pub fn observation_quarantined(&self) -> bool {
        self.observation_quarantined
    }
    /// Whether the promotion was refused (it grants nothing).
    pub fn promotion_refused(&self) -> bool {
        self.promotion_refused
    }
    /// Whether nothing became evidence across the whole flow.
    pub fn nothing_becomes_evidence(&self) -> bool {
        self.nothing_becomes_evidence
    }
    /// Whether the P12 training verdict was unchanged by the flow.
    pub fn training_gate_unchanged(&self) -> bool {
        self.training_gate_unchanged
    }
    /// The P12 training verdict recorded in the trace (always `false` — training stays blocked).
    pub fn training_justified(&self) -> bool {
        self.training_justified
    }
    /// Whether the promotion request grants a promotion (always `false` — still no evidence).
    pub fn grants_promotion(&self) -> bool {
        self.grants_promotion
    }
    /// Whether the reading receipt passed verification.
    pub fn reading_passed(&self) -> bool {
        self.reading_passed
    }

    /// The reading receipt's answer hash (what the hypothesis must cite).
    pub fn reading_answer_hash(&self) -> u64 {
        self.reading_answer_hash
    }
    /// The reading receipt's memory hash.
    pub fn reading_memory_hash(&self) -> u64 {
        self.reading_memory_hash
    }
    /// The answer hash the hypothesis cited (equals [`Self::reading_answer_hash`]).
    pub fn cited_answer_hash(&self) -> u64 {
        self.cited_answer_hash
    }
    /// The memory hash the hypothesis cited (equals [`Self::reading_memory_hash`]).
    pub fn cited_memory_hash(&self) -> u64 {
        self.cited_memory_hash
    }

    /// The execution disposition token (a non-running state — never `executed`).
    pub fn execution_status(&self) -> &str {
        &self.execution_status
    }
    /// The observation quarantine disposition token (never `recorded`).
    pub fn observation_status(&self) -> &str {
        &self.observation_status
    }
    /// The promotion outcome token (`rejected` — promotes nothing).
    pub fn promotion_status(&self) -> &str {
        &self.promotion_status
    }
    /// The governance decision token (`approved` — approval is not execution).
    pub fn review_decision(&self) -> &str {
        &self.review_decision
    }

    /// The deterministic content ids of every stage, in chain order.
    pub fn stage_ids(&self) -> [u64; 6] {
        [
            self.hypothesis_id,
            self.probe_id,
            self.review_id,
            self.intent_id,
            self.observation_id,
            self.promotion_id,
        ]
    }
}

/// The (single) token for a hypothesis authority. `Authority` has exactly one variant, so this
/// match is exhaustive with no wildcard — a future authority variant forces an explicit token
/// here (E0004) rather than silently serializing as something else.
fn authority_token(authority: Authority) -> &'static str {
    match authority {
        Authority::HypothesisOnly => "hypothesis_only",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_trace_replays() {
        // The trace is a pure function of fixed inputs: building it twice yields an identical
        // record AND identical serialized bytes (replay reproduces the end-to-end trace).
        let a = CognitiveTrace::demo().expect("the demo trace builds");
        let b = CognitiveTrace::demo().expect("the demo trace rebuilds");
        assert_eq!(a, b, "two demo runs produce the identical trace");
        assert_eq!(
            a.to_json(),
            b.to_json(),
            "the serialized trace is byte-identical on replay"
        );
    }

    #[test]
    fn trace_starts_from_verified_reading_receipt() {
        // The trace records a passing reading receipt, AND that receipt independently
        // re-verifies through the read0 verifier (the start of the chain is genuinely verified).
        let trace = CognitiveTrace::demo().unwrap();
        assert!(trace.starts_from_verified_receipt());
        assert!(trace.reading_passed());
        let (d, q, p) = demo_inputs();
        let file = produce_run(&d, &q, &p).unwrap();
        let outcome = verify_file(&file).unwrap();
        assert!(outcome.receipt.passed, "the reading receipt verifies");
        assert_eq!(file.answer_hash, trace.reading_answer_hash());
        assert_eq!(file.memory_hash, trace.reading_memory_hash());
    }

    #[test]
    fn hypothesis_cites_receipt_hash() {
        // The hypothesis cites the reading receipt by hash: the cited answer/memory hashes in
        // the trace equal the reading receipt's hashes (citation by content, not by handle).
        let trace = CognitiveTrace::demo().unwrap();
        assert!(trace.hypothesis_cites_receipt());
        assert_eq!(trace.cited_answer_hash(), trace.reading_answer_hash());
        assert_eq!(trace.cited_memory_hash(), trace.reading_memory_hash());
        assert_ne!(trace.cited_answer_hash(), 0);
    }

    #[test]
    fn probe_request_is_inert() {
        // The probe request is a classified record, never evidence: it carries a machine-checkable
        // status/reason and is wired to the hypothesis, but it is just a queued record — it executes
        // nothing and cannot become evidence. (A queued probe is "execution-eligible" only in the
        // sense that a human MAY later pick it up; this layer runs nothing.)
        let trace = CognitiveTrace::demo().unwrap();
        assert_eq!(trace.probe_status, "queued");
        assert_eq!(trace.probe_reason, "low_risk_reversible");
        // It is wired to the originating hypothesis (provenance), never minted free-standing.
        assert_eq!(trace.probe_id, trace.stage_ids()[1]);
        assert_ne!(trace.probe_id, 0);
    }

    #[test]
    fn review_does_not_execute() {
        // Governance APPROVED the probe — yet approval is a record for a human to act on later, not
        // execution. The review decision is `approved` by `governance`, and nothing in the trace
        // downstream is an executed/promoted/granted state.
        let trace = CognitiveTrace::demo().unwrap();
        assert_eq!(trace.review_decision(), "approved");
        assert_eq!(trace.review_authority, "governance");
        assert_eq!(trace.review_reason, "approved_by_reviewer_authority");
        assert_ne!(trace.execution_status(), "executed");
    }

    #[test]
    fn execution_intent_is_not_executed() {
        // The approved review yields an execution INTENT in a non-running state. There is no
        // `executed` disposition; an approval by governance is recorded `requires_operator` (a human
        // must run it later), and `nothing_executed` holds.
        let trace = CognitiveTrace::demo().unwrap();
        assert_eq!(trace.execution_status(), "requires_operator");
        assert!(trace.intent_requires_operator);
        assert!(!trace.intent_blocked);
        assert!(trace.nothing_executed());
        assert_ne!(trace.execution_status(), "executed");
    }

    #[test]
    fn observation_is_quarantined() {
        // The observation is quarantined: its disposition is `requires_review` (never `recorded`),
        // it holds only `observation_only` authority, and it does not become evidence.
        let trace = CognitiveTrace::demo().unwrap();
        assert_eq!(trace.observation_status(), "requires_review");
        assert_eq!(trace.observation_authority, "observation_only");
        assert_ne!(trace.observation_status(), "recorded");
        assert!(trace.observation_quarantined());
    }

    #[test]
    fn promotion_request_does_not_promote() {
        // The promotion request to EVIDENCE is refused: the requested target is recorded, but the
        // outcome is `rejected`, it grants no promotion, and nothing becomes evidence.
        let trace = CognitiveTrace::demo().unwrap();
        assert_eq!(trace.promotion_target, "evidence");
        assert_eq!(trace.promotion_status(), "rejected");
        assert!(!trace.grants_promotion());
        assert!(trace.promotion_refused());
        assert!(trace.nothing_becomes_evidence());
    }

    #[test]
    fn trace_does_not_change_training_gate() {
        // The whole end-to-end flow is orthogonal to P12: the training decision before and after
        // building the trace is identical — still training_not_justified.
        let before = decide(&[], &[]);
        let trace = CognitiveTrace::demo().unwrap();
        let after = decide(&[], &[]);
        assert_eq!(before, after);
        assert!(!after.training_justified);
        assert!(trace.training_gate_unchanged());
        assert!(!trace.training_justified());
    }

    #[test]
    fn trace_does_not_change_verifier_receipt() {
        // Building the entire trace from a reading receipt leaves that receipt byte-identical — the
        // integration reads hashes, never the object, and executes/promotes nothing.
        let (d, q, p) = demo_inputs();
        let file = produce_run(&d, &q, &p).unwrap();
        let before = verify_file(&file).unwrap();
        let _trace = CognitiveTrace::demo().unwrap();
        let after = verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn trace_records_every_stage_id_and_links_the_chain() {
        // Every component id/hash is recorded in one trace, all ids are non-zero, and each stage
        // cites its predecessor's id (chain_linked) — the end-to-end provenance is auditable.
        let trace = CognitiveTrace::demo().unwrap();
        assert!(trace.chain_linked());
        for id in trace.stage_ids() {
            assert_ne!(id, 0, "every stage records a deterministic id");
        }
        // The integrity hashes of the downstream records are bound (non-zero).
        assert_ne!(trace.review_integrity_hash, 0);
        assert_ne!(trace.intent_integrity_hash, 0);
        assert_ne!(trace.observation_integrity_hash, 0);
        assert_ne!(trace.promotion_integrity_hash, 0);
    }

    #[test]
    fn trace_grants_no_new_authority() {
        // INT-0 adds no authority: the serialized trace never reports an executed / promoted /
        // granted state, the hypothesis is `hypothesis_only`, the observation is `observation_only`,
        // and the promotion grants nothing. (The requested target token `evidence` is the REQUEST,
        // which is refused — so it must never coincide with a grant.)
        let trace = CognitiveTrace::demo().unwrap();
        assert_eq!(trace.hypothesis_authority, "hypothesis_only");
        assert_eq!(trace.observation_authority, "observation_only");
        assert!(!trace.grants_promotion());
        for status in [
            trace.execution_status(),
            trace.observation_status(),
            trace.promotion_status(),
        ] {
            assert_ne!(status, "executed");
            assert_ne!(status, "promoted");
            assert_ne!(status, "granted");
        }
        // The full verdict set holds: verified start, cited receipt, linked chain, no execution,
        // quarantine, refusal, no evidence, training unmoved.
        assert!(
            trace.starts_from_verified_receipt()
                && trace.hypothesis_cites_receipt()
                && trace.chain_linked()
                && trace.nothing_executed()
                && trace.observation_quarantined()
                && trace.promotion_refused()
                && trace.nothing_becomes_evidence()
                && trace.training_gate_unchanged()
                && !trace.training_justified()
        );
    }
}
