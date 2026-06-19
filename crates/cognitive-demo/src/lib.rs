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
    /// A provided trace JSON is not byte-for-byte the canonical re-derived trace (tampered, stale,
    /// or foreign) — so it is refused for report/replay rather than laundered into authority.
    TraceMismatch,
    /// The `ask` surface was given a question slug that is not in the finite, enumerated
    /// [`TraceQuestion`] set — there is no free-form / natural-language path, so an unrecognized slug
    /// fails closed rather than being interpreted.
    UnknownQuestion(String),
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
            TraceError::TraceMismatch => write!(
                f,
                "the provided trace is not the canonical trace (tampered, stale, or foreign)"
            ),
            TraceError::UnknownQuestion(slug) => write!(
                f,
                "unknown question '{slug}' — run `cognitive-demo questions` for the finite set"
            ),
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

    /// Render a PLAIN-TEXT operator report from this trace — a readable view of the same
    /// machine-checkable record, for a human who should not have to read Rust structs or test
    /// output. The report is pure PROSE *about* the trace: it computes no new verdict, calls no
    /// frozen API, and grants no authority — every line is formatted directly from this trace's
    /// already-recorded fields (read via private access in this module), so the report can never
    /// disagree with, or be more permissive than, the canonical trace it describes. It shows each
    /// stage (reading → hypothesis → probe queue → review → intent → observation → promotion) with
    /// the ids/hashes needed to audit and replay, states explicitly that nothing executed, nothing
    /// became evidence, and training stayed false, and ends with the frozen authority boundary.
    pub fn to_report(&self) -> String {
        let mut out = String::new();
        out.push_str("COGNITIVE OS — END-TO-END TRACE REPORT\n");
        out.push_str(&format!("schema: {}\n", self.schema));
        out.push_str(
            "(a readable view of one canonical CognitiveTrace; it records, it does not act)\n\n",
        );

        out.push_str("[1] READING — verifies\n");
        out.push_str(&format!("    question:        {}\n", self.reading_question));
        out.push_str(&format!("    answer:          {}\n", self.reading_answer));
        out.push_str(&format!(
            "    answer_hash:     {}\n",
            self.reading_answer_hash
        ));
        out.push_str(&format!(
            "    memory_hash:     {}\n",
            self.reading_memory_hash
        ));
        out.push_str(&format!(
            "    structure_hash:  {}\n",
            self.reading_structure_hash
                .map(|h| h.to_string())
                .unwrap_or_else(|| "none".to_string())
        ));
        out.push_str(&format!(
            "    integrity:       {}\n",
            self.reading_integrity
        ));
        out.push_str(&format!("    verified:        {}\n\n", self.reading_passed));

        out.push_str("[2] HYPOTHESIS — proposes\n");
        out.push_str(&format!("    id:              {}\n", self.hypothesis_id));
        out.push_str(&format!(
            "    statement:       {}\n",
            self.hypothesis_statement
        ));
        out.push_str(&format!(
            "    authority:       {}\n",
            self.hypothesis_authority
        ));
        out.push_str(&format!(
            "    cites receipt:   answer_hash={} memory_hash={} (matches reading: {})\n",
            self.cited_answer_hash, self.cited_memory_hash, self.hypothesis_cites_receipt
        ));
        out.push_str(&format!(
            "    expected_util:   {}\n\n",
            self.hypothesis_expected_utility
        ));

        out.push_str("[3] PROBE QUEUE — classifies\n");
        out.push_str(&format!("    id:              {}\n", self.probe_id));
        out.push_str(&format!("    status:          {}\n", self.probe_status));
        out.push_str(&format!("    reason:          {}\n\n", self.probe_reason));

        out.push_str("[4] GOVERNANCE REVIEW — reviews (a decision, not execution)\n");
        out.push_str(&format!("    id:              {}\n", self.review_id));
        out.push_str(&format!("    decision:        {}\n", self.review_decision));
        out.push_str(&format!("    authority:       {}\n", self.review_authority));
        out.push_str(&format!("    reason:          {}\n", self.review_reason));
        out.push_str(&format!(
            "    integrity_hash:  {}\n\n",
            self.review_integrity_hash
        ));

        out.push_str("[5] EXECUTION INTENT — records intent (executes nothing)\n");
        out.push_str(&format!("    id:              {}\n", self.intent_id));
        out.push_str(&format!("    status:          {}\n", self.execution_status));
        out.push_str(&format!("    reason:          {}\n", self.execution_reason));
        out.push_str(&format!(
            "    requires_operator:{}\n",
            self.intent_requires_operator
        ));
        out.push_str(&format!(
            "    integrity_hash:  {}\n\n",
            self.intent_integrity_hash
        ));

        out.push_str("[6] OBSERVATION — quarantines (never recorded, never evidence)\n");
        out.push_str(&format!("    id:              {}\n", self.observation_id));
        out.push_str(&format!(
            "    status:          {}\n",
            self.observation_status
        ));
        out.push_str(&format!(
            "    authority:       {}\n",
            self.observation_authority
        ));
        out.push_str(&format!(
            "    integrity_hash:  {}\n\n",
            self.observation_integrity_hash
        ));

        out.push_str("[7] PROMOTION REQUEST — refuses (promotes nothing)\n");
        out.push_str(&format!("    id:              {}\n", self.promotion_id));
        out.push_str(&format!("    target:          {}\n", self.promotion_target));
        out.push_str(&format!("    status:          {}\n", self.promotion_status));
        out.push_str(&format!("    reason:          {}\n", self.promotion_reason));
        out.push_str(&format!("    grants_promotion:{}\n", self.grants_promotion));
        out.push_str(&format!(
            "    integrity_hash:  {}\n\n",
            self.promotion_integrity_hash
        ));

        out.push_str("VERDICTS\n");
        out.push_str(&format!(
            "    starts_from_verified_receipt: {}\n",
            self.starts_from_verified_receipt
        ));
        out.push_str(&format!(
            "    hypothesis_cites_receipt:     {}\n",
            self.hypothesis_cites_receipt
        ));
        out.push_str(&format!(
            "    chain_linked:                 {}\n",
            self.chain_linked
        ));
        out.push_str(&format!(
            "    nothing_executed:             {}\n",
            self.nothing_executed
        ));
        out.push_str(&format!(
            "    observation_quarantined:      {}\n",
            self.observation_quarantined
        ));
        out.push_str(&format!(
            "    promotion_refused:            {}\n",
            self.promotion_refused
        ));
        out.push_str(&format!(
            "    nothing_becomes_evidence:     {}\n",
            self.nothing_becomes_evidence
        ));
        out.push_str(&format!(
            "    training_gate_unchanged:      {}\n",
            self.training_gate_unchanged
        ));
        out.push_str(&format!(
            "    training_justified:           {}\n\n",
            self.training_justified
        ));

        out.push_str("SUMMARY\n");
        out.push_str("    Nothing executed. Nothing became evidence. Nothing was promoted.\n");
        out.push_str(&format!(
            "    The P12 training verdict stayed false (training_justified={}).\n\n",
            self.training_justified
        ));

        out.push_str("BOUNDARY\n");
        for line in BOUNDARY_LINES {
            out.push_str(&format!("    {line}\n"));
        }
        out
    }
}

/// The frozen authority boundary, printed verbatim in every operator report. These nine lines are
/// the integration surface the whole prototype holds: each layer records or refuses, and nothing
/// executes, becomes evidence, or trains. Pinned as data so a test can assert every line is present.
pub const BOUNDARY_LINES: [&str; 9] = [
    "Reading verifies.",
    "Hypothesis proposes.",
    "Probe queue classifies.",
    "Governance reviews.",
    "Execution intent records.",
    "Observation quarantines.",
    "Promotion refuses.",
    "Nothing becomes evidence.",
    "Nothing trains.",
];

/// The canonical end-to-end trace as pretty JSON — the single deterministic record the whole demo
/// is about. Pure: it builds the trace via [`CognitiveTrace::demo`] (no I/O) and serializes it.
/// This is what the `trace` CLI command writes.
pub fn run_trace() -> Result<String, TraceError> {
    Ok(CognitiveTrace::demo()?.to_json())
}

/// Re-derive the canonical trace and confirm the PROVIDED trace JSON is byte-for-byte that trace.
/// This is the trust boundary for the `report`/`replay` commands: because [`CognitiveTrace`] is
/// `Serialize` but NOT `Deserialize`, a provided trace is never parsed back into authority — it is
/// only COMPARED against the freshly, purely re-derived canonical trace. A tampered, stale, or
/// foreign trace fails to match and is REFUSED ([`TraceError::TraceMismatch`]), so it can never be
/// laundered into a report or a passing replay. Returns the canonical (trusted) trace on a match.
pub fn verify_trace_json(provided: &str) -> Result<CognitiveTrace, TraceError> {
    let canonical = CognitiveTrace::demo()?;
    if provided == canonical.to_json() {
        Ok(canonical)
    } else {
        Err(TraceError::TraceMismatch)
    }
}

/// Render the operator report for a provided trace JSON — but only after [`verify_trace_json`]
/// confirms it IS the canonical trace, so the report always describes the real, deterministic,
/// frozen-track-derived trace and never an untrusted file's claims. This is what the `report`
/// command writes. Pure (no I/O).
pub fn run_report(trace_json: &str) -> Result<String, TraceError> {
    Ok(verify_trace_json(trace_json)?.to_report())
}

/// Confirm a provided trace JSON replays to the byte-identical canonical trace. This is what the
/// `replay` command checks: it re-derives the canonical trace and requires an exact match, so a
/// tampered or non-canonical trace is rejected. Pure (no I/O).
pub fn run_replay(trace_json: &str) -> Result<(), TraceError> {
    verify_trace_json(trace_json).map(|_| ())
}

// --- INT-2: the operator interrogation surface (a finite, enumerated audit-question harness over the
//     canonical trace). `ask` answers ONE enumerated question; `questions` lists the closed set. There
//     is NO free-form / natural-language path: a question is a [`TraceQuestion`] variant, an answer is
//     PROSE formatted from the trace's own recorded fields, and the trace is re-derived and confirmed
//     canonical BEFORE any answer is produced — so a question can never become authority and a tampered
//     trace can never be answered. ---

/// The INT-2 authority boundary, printed at the foot of every `ask` answer: an answer EXPLAINS the
/// trace, it does not act. Pinned as data so a test can assert every line is present in each answer.
pub const ASK_BOUNDARY_LINES: [&str; 5] = [
    "Trace questions explain the trace.",
    "They do not create authority.",
    "They do not execute.",
    "They do not promote.",
    "They do not train.",
];

/// The finite, enumerated set of audit questions an operator may ask about a [`CognitiveTrace`]. The
/// set is CLOSED — there is no free-form or natural-language path. An unrecognized slug maps to no
/// variant ([`TraceQuestion::from_slug`] returns `None`) and the `ask` surface fails closed
/// ([`TraceError::UnknownQuestion`]), so prose can never be accepted as a question and a question can
/// never grant authority. Each variant maps to one fixed answer derived only from the trace's fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceQuestion {
    /// What the reading stage read and verified.
    WhatRead,
    /// What was actually proven (exactly one thing: the reading receipt passed verification).
    WhatWasProven,
    /// What was hypothesized (a proposal, not a claim).
    WhatWasHypothesized,
    /// What probe was requested (a queued record, not an execution).
    WhatProbeWasRequested,
    /// Whether anything executed (no — approval is not execution).
    WasAnythingExecuted,
    /// Whether anything became evidence (no — the observation is quarantined, promotion refused).
    DidAnythingBecomeEvidence,
    /// Why the promotion was refused.
    WhyWasPromotionRefused,
    /// Whether training opened (no — the P12 verdict stayed false).
    DidTrainingOpen,
}

impl TraceQuestion {
    /// Every question, in canonical (chain) order. Pinned as data so a test can assert the set is
    /// finite and a `questions` listing covers it exactly.
    pub const ALL: [TraceQuestion; 8] = [
        TraceQuestion::WhatRead,
        TraceQuestion::WhatWasProven,
        TraceQuestion::WhatWasHypothesized,
        TraceQuestion::WhatProbeWasRequested,
        TraceQuestion::WasAnythingExecuted,
        TraceQuestion::DidAnythingBecomeEvidence,
        TraceQuestion::WhyWasPromotionRefused,
        TraceQuestion::DidTrainingOpen,
    ];

    /// The stable CLI slug for this question (e.g. `what-read`). Exhaustive match with no wildcard —
    /// a future variant forces a slug here (E0004) rather than silently defaulting.
    pub fn slug(self) -> &'static str {
        match self {
            TraceQuestion::WhatRead => "what-read",
            TraceQuestion::WhatWasProven => "what-was-proven",
            TraceQuestion::WhatWasHypothesized => "what-was-hypothesized",
            TraceQuestion::WhatProbeWasRequested => "what-probe-was-requested",
            TraceQuestion::WasAnythingExecuted => "was-anything-executed",
            TraceQuestion::DidAnythingBecomeEvidence => "did-anything-become-evidence",
            TraceQuestion::WhyWasPromotionRefused => "why-was-promotion-refused",
            TraceQuestion::DidTrainingOpen => "did-training-open",
        }
    }

    /// A one-line description of what the question asks (shown by `questions`). Exhaustive match.
    pub fn describe(self) -> &'static str {
        match self {
            TraceQuestion::WhatRead => "what the reading stage read and verified",
            TraceQuestion::WhatWasProven => "what was actually proven (only the reading receipt)",
            TraceQuestion::WhatWasHypothesized => "what was hypothesized (a proposal, not a claim)",
            TraceQuestion::WhatProbeWasRequested => "what probe was requested (a queued record)",
            TraceQuestion::WasAnythingExecuted => "whether anything executed (no)",
            TraceQuestion::DidAnythingBecomeEvidence => "whether anything became evidence (no)",
            TraceQuestion::WhyWasPromotionRefused => "why the promotion was refused",
            TraceQuestion::DidTrainingOpen => "whether training opened (no)",
        }
    }

    /// Parse a slug into a question. Fails CLOSED: any string that is not EXACTLY a known slug is
    /// `None` (no fuzzy match, no partial match, no free-form acceptance), so `ask` refuses it.
    pub fn from_slug(slug: &str) -> Option<TraceQuestion> {
        TraceQuestion::ALL.into_iter().find(|q| q.slug() == slug)
    }
}

/// The `questions` command: list the finite, enumerated audit-question set (slug + one-line
/// description). This IS the closed menu — there is no other way to phrase a question. Pure.
pub fn list_questions() -> String {
    let mut out = String::from(
        "cognitive-demo — audit questions (finite, enumerated; ask one with `ask --question <slug>`):\n",
    );
    for q in TraceQuestion::ALL {
        out.push_str(&format!("    {:<30} {}\n", q.slug(), q.describe()));
    }
    out
}

/// Answer ONE enumerated audit question about a provided trace JSON — but only after the trace is
/// re-derived and confirmed canonical. The flow fails closed TWICE: an unrecognized question slug is
/// refused ([`TraceError::UnknownQuestion`]) WITHOUT consulting any trace, and a non-canonical trace
/// is refused ([`TraceError::TraceMismatch`]) BEFORE any answer is produced. The returned answer is
/// PROSE about the recorded trace (formatted from its own fields, via the same private access the
/// report uses), never a new verdict and never an authority object. This is what the `ask` command
/// emits. Pure (no I/O).
pub fn run_ask(trace_json: &str, question_slug: &str) -> Result<String, TraceError> {
    // Fail closed on an unknown question BEFORE touching the trace: the question menu is the enum.
    let question = TraceQuestion::from_slug(question_slug)
        .ok_or_else(|| TraceError::UnknownQuestion(question_slug.to_string()))?;
    // Re-derive the canonical trace and refuse any tampered/stale/foreign input before answering.
    let trace = verify_trace_json(trace_json)?;
    Ok(trace.answer(question))
}

impl CognitiveTrace {
    /// Render the plain-text answer to one enumerated audit question, formatted DIRECTLY from this
    /// trace's already-recorded fields (read via private access in this module). It computes no new
    /// verdict, calls no frozen API, and returns no authority object; every answer preserves the
    /// authority boundary it is about (a hypothesis is a proposal not proof, an approval is not
    /// execution, an observation is not evidence, a refused promotion promoted nothing, and training
    /// stayed closed) and ends with the INT-2 boundary footer. Sound by construction: `ask` only ever
    /// answers the canonical trace (a tampered one is refused upstream), so the fixed yes/no headers
    /// describe the only trace that can reach here. Pure.
    fn answer(&self, question: TraceQuestion) -> String {
        let mut out = match question {
            TraceQuestion::WhatRead => self.answer_what_read(),
            TraceQuestion::WhatWasProven => self.answer_what_was_proven(),
            TraceQuestion::WhatWasHypothesized => self.answer_what_was_hypothesized(),
            TraceQuestion::WhatProbeWasRequested => self.answer_what_probe_was_requested(),
            TraceQuestion::WasAnythingExecuted => self.answer_was_anything_executed(),
            TraceQuestion::DidAnythingBecomeEvidence => self.answer_did_anything_become_evidence(),
            TraceQuestion::WhyWasPromotionRefused => self.answer_why_was_promotion_refused(),
            TraceQuestion::DidTrainingOpen => self.answer_did_training_open(),
        };
        out.push_str("\nBOUNDARY\n");
        for line in ASK_BOUNDARY_LINES {
            out.push_str(&format!("    {line}\n"));
        }
        out
    }

    fn answer_what_read(&self) -> String {
        let structure = self
            .reading_structure_hash
            .map(|h| h.to_string())
            .unwrap_or_else(|| "none".to_string());
        let mut out = String::from("READING — verifies\n");
        out.push_str(
            "This is a verified reading receipt: the only object in the trace that was PROVEN.\n",
        );
        out.push_str(&format!("    question:        {}\n", self.reading_question));
        out.push_str(&format!("    answer:          {}\n", self.reading_answer));
        out.push_str(&format!(
            "    answer_hash:     {}\n",
            self.reading_answer_hash
        ));
        out.push_str(&format!(
            "    memory_hash:     {}\n",
            self.reading_memory_hash
        ));
        out.push_str(&format!("    structure_hash:  {structure}\n"));
        out.push_str(&format!(
            "    integrity:       {}\n",
            self.reading_integrity
        ));
        out.push_str(&format!("    verified:        {}\n", self.reading_passed));
        out
    }

    fn answer_what_was_proven(&self) -> String {
        let mut out =
            String::from("PROVEN — exactly one thing: the reading receipt passed verification.\n");
        out.push_str(
            "The read0 verifier accepted the answer as grounded, supported, and replay-matched.\n",
        );
        out.push_str(&format!("    proven answer:   {}\n", self.reading_answer));
        out.push_str(&format!(
            "    answer_hash:     {}\n",
            self.reading_answer_hash
        ));
        out.push_str(&format!(
            "    memory_hash:     {}\n",
            self.reading_memory_hash
        ));
        out.push_str(&format!("    verified:        {}\n", self.reading_passed));
        out.push_str("Nothing downstream is proof:\n");
        out.push_str("    - the hypothesis only PROPOSES (it is not a claim),\n");
        out.push_str("    - the governance review only DECIDES (approval is not execution),\n");
        out.push_str("    - the observation is QUARANTINED (it is not evidence),\n");
        out.push_str("    - the promotion request was REFUSED (nothing was promoted).\n");
        out
    }

    fn answer_what_was_hypothesized(&self) -> String {
        let mut out =
            String::from("HYPOTHESIS — proposes (a proposal, NOT a claim and NOT proof)\n");
        out.push_str(&format!("    id:              {}\n", self.hypothesis_id));
        out.push_str(&format!(
            "    statement:       {}\n",
            self.hypothesis_statement
        ));
        out.push_str(&format!(
            "    authority:       {}\n",
            self.hypothesis_authority
        ));
        out.push_str(&format!(
            "    cites receipt:   answer_hash={} memory_hash={} (matches reading: {})\n",
            self.cited_answer_hash, self.cited_memory_hash, self.hypothesis_cites_receipt
        ));
        out.push_str(&format!(
            "    expected_util:   {}\n",
            self.hypothesis_expected_utility
        ));
        out.push_str("A hypothesis only proposes a test to run later; it asserts nothing as true and grants no authority.\n");
        out
    }

    fn answer_what_probe_was_requested(&self) -> String {
        let mut out = String::from(
            "PROBE REQUEST — classifies (a queued record, NOT an execution and NOT evidence)\n",
        );
        out.push_str(&format!("    id:              {}\n", self.probe_id));
        out.push_str(&format!("    status:          {}\n", self.probe_status));
        out.push_str(&format!("    reason:          {}\n", self.probe_reason));
        out.push_str(
            "A queued probe records WHAT a human could test later; this layer runs nothing.\n",
        );
        out
    }

    fn answer_was_anything_executed(&self) -> String {
        let mut out = String::from("WAS ANYTHING EXECUTED?  No.\n");
        out.push_str("Governance APPROVED the probe — but approval is a decision recorded for a human, not execution.\n");
        out.push_str(&format!(
            "    review decision:    {} (by {})\n",
            self.review_decision, self.review_authority
        ));
        out.push_str(&format!("    execution intent:   id={}\n", self.intent_id));
        out.push_str(&format!(
            "    execution status:   {} (a non-running state; never `executed`)\n",
            self.execution_status
        ));
        out.push_str(&format!(
            "    requires_operator:  {}\n",
            self.intent_requires_operator
        ));
        out.push_str(&format!(
            "    nothing_executed:   {}\n",
            self.nothing_executed
        ));
        out.push_str("No probe ran. Nothing executed.\n");
        out
    }

    fn answer_did_anything_become_evidence(&self) -> String {
        let mut out = String::from("DID ANYTHING BECOME EVIDENCE?  No.\n");
        out.push_str(&format!(
            "    observation:        id={} status={} ({})\n",
            self.observation_id, self.observation_status, self.observation_authority
        ));
        out.push_str(&format!(
            "    promotion request:  id={} target={} (the REQUEST)\n",
            self.promotion_id, self.promotion_target
        ));
        out.push_str(&format!(
            "    promotion outcome:  {}\n",
            self.promotion_status
        ));
        out.push_str(&format!(
            "    grants_promotion:   {}\n",
            self.grants_promotion
        ));
        out.push_str(&format!(
            "    nothing_becomes_evidence: {}\n",
            self.nothing_becomes_evidence
        ));
        out.push_str("The observation stayed quarantined and the promotion to evidence was refused. Nothing became evidence.\n");
        out
    }

    fn answer_why_was_promotion_refused(&self) -> String {
        let mut out = String::from("WHY WAS PROMOTION REFUSED?\n");
        out.push_str(&format!("    promotion id:    {}\n", self.promotion_id));
        out.push_str(&format!(
            "    requested target: {} (the REQUEST — to become evidence)\n",
            self.promotion_target
        ));
        out.push_str(&format!("    outcome:         {}\n", self.promotion_status));
        out.push_str(&format!("    reason:          {}\n", self.promotion_reason));
        out.push_str(&format!(
            "    grants_promotion: {}\n",
            self.grants_promotion
        ));
        out.push_str("An observation that only `requires_review` cannot be promoted to evidence: the request was\n");
        out.push_str("rejected and grants nothing. The promotion did not occur.\n");
        out
    }

    fn answer_did_training_open(&self) -> String {
        let mut out = String::from("DID TRAINING OPEN?  No.\n");
        out.push_str(&format!(
            "    training_justified:      {} (P12 verdict — still false)\n",
            self.training_justified
        ));
        out.push_str(&format!(
            "    training_gate_unchanged: {}\n",
            self.training_gate_unchanged
        ));
        out.push_str(
            "The P12 training verdict stayed false; nothing in this trace opens a training path.\n",
        );
        out
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

    // --- INT-1: the operator CLI / report surface (pure cores; the binary fs shell is gated by the
    //     release_check INT-1 binary smoke). ---

    #[test]
    fn trace_command_writes_json() {
        // The `trace` command's pure core produces the canonical CognitiveTrace JSON — identical to
        // the trace the demo builds, and valid pretty JSON carrying the recorded fields.
        let json = run_trace().expect("trace command core produces json");
        assert_eq!(json, CognitiveTrace::demo().unwrap().to_json());
        assert!(json.starts_with('{') && json.trim_end().ends_with('}'));
        assert!(json.contains("\"schema\": \"cognitive-trace-v0.1\""));
        assert!(json.contains("\"promotion_status\": \"rejected\""));
        assert!(json.contains("\"training_justified\": false"));
    }

    #[test]
    fn report_command_writes_operator_summary() {
        // The `report` command's pure core verifies the trace, then renders a readable operator
        // report that walks every stage with its ids/hashes — no Rust structs or test output needed.
        let json = run_trace().unwrap();
        let report = run_report(&json).expect("report command core renders a summary");
        for stage in [
            "[1] READING",
            "[2] HYPOTHESIS",
            "[3] PROBE QUEUE",
            "[4] GOVERNANCE REVIEW",
            "[5] EXECUTION INTENT",
            "[6] OBSERVATION",
            "[7] PROMOTION REQUEST",
        ] {
            assert!(report.contains(stage), "report must show stage {stage}");
        }
        // It includes the ids/hashes needed to audit/replay (every stage id appears).
        let trace = CognitiveTrace::demo().unwrap();
        for id in trace.stage_ids() {
            assert!(
                report.contains(&id.to_string()),
                "report must include stage id {id}"
            );
        }
        // It states the refusals explicitly, in prose, for a human.
        assert!(report.contains("Nothing executed."));
        assert!(report.contains("Nothing became evidence."));
        assert!(report.contains("training_justified=false"));
    }

    #[test]
    fn report_contains_all_boundary_lines() {
        // The report prints the frozen authority boundary verbatim — all nine lines, in order.
        let report = run_report(&run_trace().unwrap()).unwrap();
        for line in BOUNDARY_LINES {
            assert!(
                report.contains(line),
                "report must contain boundary: {line}"
            );
        }
        assert_eq!(BOUNDARY_LINES.len(), 9);
        assert_eq!(BOUNDARY_LINES[0], "Reading verifies.");
        assert_eq!(BOUNDARY_LINES[8], "Nothing trains.");
    }

    #[test]
    fn replay_reproduces_trace() {
        // Replay re-derives the canonical trace and confirms a provided trace is BYTE-IDENTICAL.
        // The canonical trace replays; a tampered trace is REFUSED (TraceMismatch), so replay can
        // never silently accept a forged record.
        let json = run_trace().unwrap();
        assert!(run_replay(&json).is_ok(), "the canonical trace replays");
        let tampered = json.replace("\"grants_promotion\": false", "\"grants_promotion\": true");
        assert_ne!(tampered, json, "the tamper actually changed the bytes");
        assert!(
            matches!(run_replay(&tampered), Err(TraceError::TraceMismatch)),
            "a tampered trace must be refused, not replayed"
        );
    }

    #[test]
    fn report_does_not_change_trace() {
        // Rendering a report is read-only: the canonical trace is byte-identical before and after,
        // and a forged trace can never be laundered into a report (it is refused).
        let before = run_trace().unwrap();
        let _report = run_report(&before).unwrap();
        let after = run_trace().unwrap();
        assert_eq!(
            before, after,
            "rendering a report does not change the trace"
        );
        let forged = before.replace(
            "\"promotion_status\": \"rejected\"",
            "\"promotion_status\": \"promoted\"",
        );
        assert!(
            matches!(run_report(&forged), Err(TraceError::TraceMismatch)),
            "a forged trace cannot be laundered into a report"
        );
    }

    #[test]
    fn cli_does_not_execute_probe() {
        // The CLI surface runs no probe: the report it renders shows the execution intent in a
        // non-running state (never `executed`) and asserts nothing executed — the report describes a
        // record, it never triggers one.
        let report = run_report(&run_trace().unwrap()).unwrap();
        assert!(report.contains("EXECUTION INTENT — records intent (executes nothing)"));
        assert!(report.contains("requires_operator"));
        assert!(report.contains("Nothing executed."));
        // The report never claims an executed disposition (the status is requires_operator).
        assert!(!report.contains("executed_at"));
        assert!(!report.contains(": executed"));
        // The pure trace underneath the CLI confirms it structurally.
        assert!(CognitiveTrace::demo().unwrap().nothing_executed());
    }

    #[test]
    fn cli_does_not_change_training_gate() {
        // Running the CLI cores (trace + report) is orthogonal to P12: the training decision before
        // and after is identical — still training_not_justified.
        let before = decide(&[], &[]);
        let json = run_trace().unwrap();
        let _report = run_report(&json).unwrap();
        run_replay(&json).unwrap();
        let after = decide(&[], &[]);
        assert_eq!(before, after);
        assert!(!after.training_justified);
        assert!(run_report(&json)
            .unwrap()
            .contains("training_justified=false"));
    }

    #[test]
    fn cli_does_not_change_verifier_receipt() {
        // Producing the trace/report/replay from a reading receipt leaves that receipt byte-identical
        // — the CLI reads hashes and re-derives, it never mutates the verifier or executes anything.
        let (d, q, p) = demo_inputs();
        let file = produce_run(&d, &q, &p).unwrap();
        let before = verify_file(&file).unwrap();
        let json = run_trace().unwrap();
        let _report = run_report(&json).unwrap();
        run_replay(&json).unwrap();
        let after = verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    // --- INT-2: the operator interrogation surface (`ask` + `questions`). The answers are pure prose
    //     about the SAME canonical trace; the binary `ask`/`questions` commands are gated by the
    //     release_check INT-2 smoke. ---

    #[test]
    fn questions_command_lists_finite_question_set() {
        // The question set is finite and enum-backed; `questions` lists every slug, each slug
        // round-trips through from_slug, and the set is CLOSED — a near-miss / free-form string is not
        // accepted (no fuzzy or partial match).
        assert_eq!(TraceQuestion::ALL.len(), 8);
        let listing = list_questions();
        for q in TraceQuestion::ALL {
            assert!(
                listing.contains(q.slug()),
                "questions must list {}",
                q.slug()
            );
            assert_eq!(
                TraceQuestion::from_slug(q.slug()),
                Some(q),
                "slug round-trips"
            );
        }
        assert_eq!(TraceQuestion::from_slug("what_read"), None);
        assert_eq!(TraceQuestion::from_slug("what-read "), None);
        assert_eq!(TraceQuestion::from_slug("tell me what you read"), None);
        assert_eq!(TraceQuestion::from_slug(""), None);
    }

    #[test]
    fn ask_refuses_unknown_question() {
        // An unknown question fails CLOSED — UnknownQuestion, never an answer — and does so without
        // even requiring a valid trace (the question menu is the enum, checked first).
        let json = run_trace().unwrap();
        assert!(matches!(
            run_ask(&json, "explain-everything"),
            Err(TraceError::UnknownQuestion(_))
        ));
        // Even with a valid trace, only EXACT enum slugs answer; a plausible-looking miss is refused.
        assert!(run_ask(&json, "what-was-promoted").is_err());
    }

    #[test]
    fn ask_refuses_tampered_trace() {
        // A tampered trace is refused BEFORE any answer is produced: run_ask re-derives the canonical
        // trace and byte-compares, so a forged file maps to TraceMismatch, not a (laundered) answer.
        let json = run_trace().unwrap();
        let tampered = json.replace("\"grants_promotion\": false", "\"grants_promotion\": true");
        assert_ne!(tampered, json, "the tamper changed the bytes");
        assert!(matches!(
            run_ask(&tampered, "did-anything-become-evidence"),
            Err(TraceError::TraceMismatch)
        ));
        // A wholly foreign trace is likewise refused.
        assert!(matches!(
            run_ask("{\"not\":\"a trace\"}", "what-read"),
            Err(TraceError::TraceMismatch)
        ));
    }

    #[test]
    fn ask_what_read_reports_receipt_hash() {
        // `what-read` reports the verified reading receipt, including its answer/memory hashes — the
        // exact values recorded in the trace (so the operator can audit/replay).
        let trace = CognitiveTrace::demo().unwrap();
        let answer = run_ask(&run_trace().unwrap(), "what-read").unwrap();
        assert!(answer.contains("READING"));
        assert!(answer.contains("answer_hash"));
        assert!(answer.contains(&trace.reading_answer_hash().to_string()));
        assert!(answer.contains(&trace.reading_memory_hash().to_string()));
        assert!(answer.contains("verified:"));
    }

    #[test]
    fn ask_what_proven_reports_verified_reading_result() {
        // `what-was-proven` reports that EXACTLY the reading receipt was proven (verified), and is
        // explicit that nothing downstream is proof (the hypothesis proposes, the observation is
        // quarantined, etc.).
        let trace = CognitiveTrace::demo().unwrap();
        let answer = run_ask(&run_trace().unwrap(), "what-was-proven").unwrap();
        assert!(answer.contains("PROVEN"));
        assert!(trace.reading_passed());
        assert!(answer.contains(&trace.reading_answer_hash().to_string()));
        assert!(answer.contains("PROPOSES"));
        assert!(answer.contains("QUARANTINED"));
    }

    #[test]
    fn ask_hypothesis_distinguishes_hypothesis_from_claim() {
        // `what-was-hypothesized` makes the hypothesis/claim distinction explicit: it is a proposal
        // with `hypothesis_only` authority, NOT a claim and NOT proof, and it cites the receipt by hash.
        let answer = run_ask(&run_trace().unwrap(), "what-was-hypothesized").unwrap();
        assert!(answer.contains("HYPOTHESIS"));
        assert!(answer.contains("hypothesis_only"));
        assert!(answer.contains("NOT a claim"));
        assert!(answer.contains("proposes") || answer.contains("proposal"));
        let trace = CognitiveTrace::demo().unwrap();
        assert!(answer.contains(&trace.cited_answer_hash().to_string()));
    }

    #[test]
    fn ask_execution_question_returns_no_execution() {
        // `was-anything-executed` answers No: the approved review yields a non-running execution intent
        // (`requires_operator`, never `executed`) and the answer never shows an executed status.
        let answer = run_ask(&run_trace().unwrap(), "was-anything-executed").unwrap();
        assert!(answer.contains("No"));
        assert!(answer.contains("requires_operator"));
        assert!(answer.contains("Nothing executed."));
        assert!(!answer.contains(": executed"));
        assert!(CognitiveTrace::demo().unwrap().nothing_executed());
    }

    #[test]
    fn ask_evidence_question_returns_no_evidence() {
        // `did-anything-become-evidence` answers No: the observation is quarantined and the promotion
        // was refused (grants_promotion=false), so nothing became evidence.
        let answer = run_ask(&run_trace().unwrap(), "did-anything-become-evidence").unwrap();
        assert!(answer.contains("No"));
        assert!(answer.contains("Nothing became evidence."));
        assert!(answer.contains("rejected"));
        let trace = CognitiveTrace::demo().unwrap();
        assert!(!trace.grants_promotion());
        assert!(trace.nothing_becomes_evidence());
    }

    #[test]
    fn ask_training_question_returns_training_false() {
        // `did-training-open` answers No: the P12 verdict stayed false (and unchanged).
        let answer = run_ask(&run_trace().unwrap(), "did-training-open").unwrap();
        assert!(answer.contains("No"));
        assert!(answer.contains("training_justified"));
        let trace = CognitiveTrace::demo().unwrap();
        assert!(!trace.training_justified());
        assert!(trace.training_gate_unchanged());
    }

    #[test]
    fn ask_does_not_change_trace_or_training_gate() {
        // Asking every question is read-only: the canonical trace is byte-identical before and after,
        // and the P12 decision is unmoved (still training_not_justified).
        let before = run_trace().unwrap();
        let before_gate = decide(&[], &[]);
        for q in TraceQuestion::ALL {
            let _answer = run_ask(&before, q.slug()).unwrap();
        }
        let after = run_trace().unwrap();
        let after_gate = decide(&[], &[]);
        assert_eq!(before, after, "asking questions does not change the trace");
        assert_eq!(before_gate, after_gate);
        assert!(!after_gate.training_justified);
    }

    #[test]
    fn ask_answer_preserves_authority_boundary() {
        // EVERY answer ends with the INT-2 authority boundary (the answer explains; it does not act),
        // and every enumerated question produces an answer (the set is fully covered).
        let json = run_trace().unwrap();
        for q in TraceQuestion::ALL {
            let answer = run_ask(&json, q.slug()).unwrap();
            for line in ASK_BOUNDARY_LINES {
                assert!(
                    answer.contains(line),
                    "answer to {} must contain boundary: {line}",
                    q.slug()
                );
            }
        }
        assert_eq!(ASK_BOUNDARY_LINES.len(), 5);
        assert_eq!(ASK_BOUNDARY_LINES[0], "Trace questions explain the trace.");
        assert_eq!(ASK_BOUNDARY_LINES[1], "They do not create authority.");
        assert_eq!(ASK_BOUNDARY_LINES[2], "They do not execute.");
        assert_eq!(ASK_BOUNDARY_LINES[3], "They do not promote.");
        assert_eq!(ASK_BOUNDARY_LINES[4], "They do not train.");
    }

    #[test]
    fn ask_answer_is_not_authority() {
        // No answer ever shows an affirmative executed/promoted/granted/recorded STATUS, no grant reads
        // true, and no answer claims an execution/promotion occurred — `ask` output describes the
        // trace, it is not authority.
        let json = run_trace().unwrap();
        for q in TraceQuestion::ALL {
            let answer = run_ask(&json, q.slug()).unwrap();
            for line in answer.lines() {
                let l = line.trim_end();
                assert!(!l.ends_with(": executed"), "no executed status: {line}");
                assert!(!l.ends_with(": promoted"), "no promoted status: {line}");
                assert!(!l.ends_with(": granted"), "no granted status: {line}");
                assert!(!l.ends_with(": recorded"), "no recorded status: {line}");
            }
            assert!(!answer.contains("promotion occurred"));
            assert!(!answer.contains("was promoted to evidence"));
            assert!(!answer.contains("grants_promotion: true"));
            assert!(!answer.contains("grants_promotion:true"));
        }
    }
}
