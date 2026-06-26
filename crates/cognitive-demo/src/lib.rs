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
    propose, Authority, EvidenceRef, HypothesisError, HypothesisPacket, HypothesisSpec,
    ProbeExecutionIntent, ProbeObservationReceipt, ProbeRequest, PromotionRequest, PromotionTarget,
    ReviewDecision, ReviewError, ReviewReceipt, ReviewerAuthority,
};
use reading_cli::{corpus_from_documents, produce_run, verify_file, CliError};
use reading_substrate::SpanId;
use reading_train_gate::decide;
use serde::Serialize;

/// HORIZON-0 — the staged interaction harness. Composes the existing verified-read,
/// DATA-0 curation, dream-packet, and dream-export flows into bounded horizons
/// `H0..H5` and proves longer horizons cannot bypass earlier gates. See
/// [`horizon`] for the boundary and invariants.
mod horizon;
pub use horizon::{
    horizon_failure_matrix, horizon_failure_matrix_json, horizon_matrix, horizon_matrix_json,
    run_horizon, run_horizon_json, verify_horizon_json, verify_horizon_matrix_json, FailureCell,
    HorizonError, HorizonLevel, HorizonStep, HorizonTrace, Module, RefusalMechanism,
    FAILURE_SCENARIO_COUNT, HORIZON_BOUNDARY_LINES, HORIZON_FAILURE_BOUNDARY_LINES,
};

/// CORPUS-HARVEST-0 — the first model-readiness corpus-harvest pipeline. Collects
/// already-verified substrate artifacts into deterministic harvest receipts, routing
/// every candidate through the DATA-0 `data_curator::curate()` gate first. No training,
/// no memory write, no new authority. See [`corpus_harvest`] for the boundary.
mod corpus_harvest;
pub use corpus_harvest::{
    corpus_harvest_matrix, corpus_harvest_matrix_json, harvest_corpus, harvest_corpus_json,
    verify_harvest_json, CorpusHarvestManifest, CorpusHarvestMatrix, CuratedCorpusReceipt,
    HarvestBoundaryChecks, HarvestDisposition, HarvestError, HarvestItem, HarvestOutcome,
    HarvestScenarioCell, HarvestSource, QuarantineReport, QuarantinedHarvestItem,
    RejectedHarvestItem, RejectedItemsReport, SourceCurationSummary, SplitIntegrityReport,
    HARVEST_BOUNDARY_LINES, HARVEST_SCENARIO_COUNT,
};

/// SCORE-0 — the verifier-as-scorer. Turns the EXISTING verifier outcomes (curation,
/// harvest replay, horizon gates, trace verification) into deterministic score receipts —
/// but a score is an OBSERVATION, never authority. No score promotes evidence, creates
/// memory, grants authority, or opens training; failures are recorded as
/// `FailureObservation`s, never training examples. See [`score`] for the boundary.
mod score;
pub use score::{
    canonical_score_receipt, score_answer_support, score_curation, score_grounding,
    score_horizon_boundary, score_receipt_json, score_refusal, score_replay, score_trace_integrity,
    verifier_score_matrix, verifier_score_matrix_json, verify_score_matrix_json,
    verify_score_receipt_json, FailureObservation, GroundingInput, HorizonScoreInput, ScoreCell,
    ScoreClass, ScoreError, ScoreReason, ScoreReceipt, ScoreState, ScoringBoundary,
    VerifierScoreMatrix, SCORE_BOUNDARY_LINES, SCORE_CLASS_COUNT, SCORE_CLASS_NAMES,
    SCORE_SCENARIO_COUNT,
};

/// FAIL-0 — the recurring-clean-failure detector. Consumes SCORE-0 `FailureObservation`s and
/// answers ONLY "did the same clean failure recur enough to be a `ModelNeedCandidate`?" — never
/// "should we train?". It separates clean model failures from substrate failures and the eight
/// exclusion causes, requires an explicit recurrence threshold, and emits candidates that are
/// structurally NOT training authorization (no score/threshold opens training). See
/// [`failure_detector`] for the boundary.
mod failure_detector;
// NOTE: `FailureCase` and `FAILURE_SCENARIO_COUNT` are intentionally re-exported under FAIL-0-prefixed
// aliases — both bare names are already taken at the crate root (the INT-0 `FailureCase` enum and the
// horizon `FAILURE_SCENARIO_COUNT`). The canonical names remain `pub` in `failure_detector` (and pinned
// by the gate); the aliases keep them reachable so they are not dead code.
pub use failure_detector::{
    canonical_failure_report, detect_failures, detect_failures_json, failure_detector_matrix,
    failure_detector_matrix_json, failure_report_json, verify_failure_detector_matrix_json,
    verify_failure_report_json, CleanFailureStatus, FailureCase as ModelFailureCase, FailureCause,
    FailureClass, FailureContext, FailureDetectorBoundary, FailureDetectorError,
    FailureDetectorMatrix, FailureDetectorReport, FailureExclusion, FailureRecurrencePolicy,
    FailureScenarioCell, FailureSignal, ModelNeedCandidate, FAILURE_CLASS_COUNT,
    FAILURE_CLASS_NAMES, FAILURE_SCENARIO_COUNT as FAILURE_DETECTOR_SCENARIO_COUNT,
    FAIL_BOUNDARY_LINES, RECURRENCE_THRESHOLD,
};

/// P11-MODEL-EVAL — the honest fork. Consumes FAIL-0 `ModelNeedCandidate`s plus baseline /
/// prompt / retrieval / horizon / substrate comparison observations and emits a deterministic
/// `ModelNeedVerdict` (no_training_needed / improve_substrate_first / collect_more_data /
/// training_candidate_only) WITHOUT opening training, touching weights, or promoting a model.
/// `training_candidate_only` is a candidacy flag for a later explicit gate, never authorization.
/// See [`model_eval`] for the boundary.
mod model_eval;
pub use model_eval::{
    evaluate_model_need, evaluate_model_need_json, model_eval_matrix, model_eval_matrix_json,
    verify_model_eval_matrix_json, verify_model_eval_report_json, EvalComparison, EvalCondition,
    EvalRun, ModelEvalBattery, ModelEvalBoundary, ModelEvalError, ModelEvalMatrix,
    ModelEvalScenarioCell, ModelNeedEvalReport, ModelNeedEvidence, ModelNeedVerdict,
    ResidualFailure, TrainingCandidateSignal, MODEL_EVAL_BOUNDARY_LINES, MODEL_EVAL_SCENARIO_COUNT,
    MODEL_NEED_MIN_RESIDUALS, VERDICT_COUNT, VERDICT_NAMES,
};

/// TRAIN-GATE-0 — the explicit, closed-by-default gate before any weight change. It CONSUMES the
/// real P11-MODEL-EVAL verdict (running `evaluate_model_need` itself over the supplied battery) and
/// emits `TrainingAttemptAllowed` ONLY when the verdict is `training_candidate_only` AND every
/// requirement holds: operator authorization, curated dataset, clean holdout, clean contamination,
/// recurring-failure evidence, rollback plan, production safety plan, and an affirmative
/// authority-drift check. `TrainingAttemptAllowed` is only permission to ATTEMPT a later run — it
/// trains nothing, modifies no weights, promotes/deploys nothing, and leaves P12
/// `training_justified = false`. Reports are `Serialize` but never `Deserialize`. See
/// [`training_gate`] for the boundary.
mod training_gate;
pub use training_gate::{
    evaluate_training_gate, evaluate_training_gate_json, training_gate_matrix,
    training_gate_matrix_json, verify_training_gate_matrix_json, verify_training_gate_report_json,
    AuthorityDriftCheck, ContaminationReportReceipt, DatasetReadinessReceipt,
    HoldoutReadinessReceipt, OperatorAuthorizationReceipt, ProductionSafetyPlanReceipt,
    RollbackPlanReceipt, TrainingGateBoundary, TrainingGateDecision, TrainingGateError,
    TrainingGateInput, TrainingGateMatrix, TrainingGateRefusal, TrainingGateReport,
    TrainingGateRequirement, TrainingGateScenarioCell, MIN_RECURRING_FAILURES,
    TRAINING_GATE_BOUNDARY_LINES, TRAIN_GATE_DECISION_COUNT, TRAIN_GATE_DECISION_NAMES,
    TRAIN_GATE_REFUSAL_COUNT, TRAIN_GATE_REFUSAL_NAMES, TRAIN_GATE_SCENARIO_COUNT,
};

/// TRAIN-0 — the first gated, deterministic local training-ATTEMPT harness. It CONSUMES the real
/// TRAIN-GATE-0 report (running `evaluate_training_gate` itself over the supplied gate input, which
/// re-runs P11) and enforces TWO keys before preparing anything: the gate must emit
/// `TrainingAttemptAllowed` AND a SEPARATE explicit operator authorization for the attempt must be
/// present — neither alone suffices. A `dry_run_only` invocation prepares a plan that touches no
/// weights and yields no candidate; an `authorized_local_attempt` prepares a `CandidateOnly`,
/// hash-pinned candidate descriptor ONLY when both keys turn and every reproducibility prerequisite
/// (deterministic config, curated uncontaminated dataset, present non-leaking holdout, baseline,
/// rollback, clean authority-drift) is satisfied. A candidate is never promoted, deployed, made
/// evidence, written to memory, granted authority, or used to replace the baseline; it MUST be
/// evaluated later by S8. The harness performs no real weight mutation and leaves P12
/// `training_justified = false`. Receipts are `Serialize` but never `Deserialize`. See
/// [`training_attempt`] for the boundary.
mod training_attempt;
pub use training_attempt::{
    run_training_attempt, run_training_attempt_json, training_attempt_matrix,
    training_attempt_matrix_json, verify_training_attempt_matrix_json,
    verify_training_attempt_receipt_json, AttemptAuthorizationReceipt, CandidateAcceptance,
    TrainingAttemptBoundary, TrainingAttemptError, TrainingAttemptInput, TrainingAttemptMatrix,
    TrainingAttemptMode, TrainingAttemptOutcome, TrainingAttemptPlan, TrainingAttemptReceipt,
    TrainingAttemptRefusal, TrainingAttemptRequirement, TrainingAttemptScenarioCell,
    TrainingBaselineArtifact, TrainingCandidateArtifact, TrainingDatasetBundle,
    TrainingHoldoutBundle, TrainingRollbackArtifact, TrainingRunConfig,
    TRAINING_ATTEMPT_BOUNDARY_LINES, TRAIN_ATTEMPT_MODE_COUNT, TRAIN_ATTEMPT_MODE_NAMES,
    TRAIN_ATTEMPT_REFUSAL_COUNT, TRAIN_ATTEMPT_REFUSAL_NAMES, TRAIN_ATTEMPT_SCENARIO_COUNT,
};

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
    /// A required bundle file was absent from the provided bundle (named by the missing filename) —
    /// so the bundle cannot be verified and is refused.
    BundleMissingFile(String),
    /// A provided bundle file (named) did not byte-match the re-derived canonical bundle file
    /// (tampered, stale, or foreign) — so it is refused rather than trusted over the re-derivation.
    BundleMismatch(String),
    /// A provided scenario-coverage matrix did not byte-match the re-derived canonical matrix
    /// (tampered, stale, or foreign) — so it is refused rather than trusted over the re-derivation.
    MatrixMismatch,
    /// DOCFLOW-0: the operator-supplied document has no readable sentence span, so no verified
    /// reading receipt can be produced from it — the document flow fails closed rather than tracing
    /// an empty/ungrounded read.
    EmptyDocument,
    /// DOCFLOW-0: a provided document trace JSON is not byte-for-byte the trace re-derived from the
    /// SAME operator document (tampered, stale, or foreign) — so it is refused for doc-report/replay
    /// rather than laundered into authority.
    DocTraceMismatch,
    /// DOCFLOW-0: the operator-supplied input path is not a safe local path (absolute, parent-dir
    /// traversal, or otherwise escaping the working directory) — the document flow refuses to read it.
    UnsafeInputPath(String),
    /// CORPUS-0: the operator-supplied corpus directory yields no readable sentence span (no admitted
    /// `.txt` document, or only empty/heading-only documents), so no verified reading receipt can be
    /// produced — the corpus flow fails closed rather than tracing an empty/ungrounded read.
    EmptyCorpus,
    /// CORPUS-0: a provided corpus trace JSON is not byte-for-byte the trace re-derived from the SAME
    /// operator corpus (tampered, stale, or foreign) — so it is refused for corpus-report rather than
    /// laundered into authority.
    CorpusTraceMismatch,
    /// NOVELTY-0: the verified corpus trace carries no reading-receipt structure hash, so a novelty packet
    /// cannot cite the receipt it claims to be grounded in — the harness fails closed rather than emit an
    /// ungrounded packet.
    MissingReceiptHash,
    /// NOVELTY-0: the operator frame has no non-empty line, so there is no candidate assumption to break —
    /// the harness refuses to emit a hypothesis packet with nothing to challenge.
    EmptyFrame,
    /// NOVELTY-0: a preserved fact is not VERBATIM one of the corpus's verified spans, so it is not grounded
    /// in the verified read — the harness refuses it rather than laundering an unsupported fact into the packet.
    UnsupportedPreservedFact,
    /// NOVELTY-0: a provided novelty packet JSON is not byte-for-byte the packet re-derived from the SAME
    /// corpus and frame (tampered, stale, or foreign) — so it is refused for novelty-report/replay rather than
    /// trusted over the re-derivation.
    NoveltyPacketMismatch,
    /// DREAM-EXPORT-0: dream-engine refused to produce or verify a dream packet for export — the
    /// corpus did not verify, the dream was degenerate, the weirdness was out of range, or a provided
    /// `--dream-packet` was tampered. Carries the engine's own explicit refusal message.
    DreamExport(String),
    /// DREAM-EXPORT-0: a provided dream-export bundle is not byte-for-byte the canonical re-derivation
    /// (tampered, stale, or foreign) — refused rather than parsed back into authority.
    DreamExportMismatch,
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
            TraceError::BundleMissingFile(name) => {
                write!(f, "the bundle is missing required file '{name}'")
            }
            TraceError::BundleMismatch(name) => write!(
                f,
                "bundle file '{name}' is not the canonical file (tampered, stale, or foreign)"
            ),
            TraceError::MatrixMismatch => write!(
                f,
                "the provided matrix is not the canonical matrix (tampered, stale, or foreign)"
            ),
            TraceError::EmptyDocument => write!(
                f,
                "the document has no readable sentence span, so no verified reading receipt can be produced"
            ),
            TraceError::DocTraceMismatch => write!(
                f,
                "the provided trace is not the trace re-derived from this document (tampered, stale, or foreign)"
            ),
            TraceError::UnsafeInputPath(path) => write!(
                f,
                "refusing unsafe input path '{path}' — the document flow reads only a local file inside the working directory"
            ),
            TraceError::EmptyCorpus => write!(
                f,
                "the corpus has no readable sentence span, so no verified reading receipt can be produced"
            ),
            TraceError::CorpusTraceMismatch => write!(
                f,
                "the provided trace is not the trace re-derived from this corpus (tampered, stale, or foreign)"
            ),
            TraceError::MissingReceiptHash => write!(
                f,
                "the verified corpus trace carries no reading-receipt hash, so a novelty packet cannot cite it"
            ),
            TraceError::EmptyFrame => write!(
                f,
                "the operator frame has no non-empty line — there is no candidate assumption to break"
            ),
            TraceError::UnsupportedPreservedFact => write!(
                f,
                "a preserved fact is not a verified corpus span, so it is not grounded and is refused"
            ),
            TraceError::NoveltyPacketMismatch => write!(
                f,
                "the provided packet is not the packet re-derived from this corpus and frame (tampered, stale, or foreign)"
            ),
            TraceError::DreamExport(why) => write!(f, "dream export refused: {why}"),
            TraceError::DreamExportMismatch => write!(
                f,
                "the provided dream-export bundle is not the canonical re-derivation (tampered, stale, or foreign)"
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
        // The canonical INT-0 trace IS the `happy-boundary` scenario (a low-risk, reversible probe that
        // governance approves). It is preserved byte-for-byte by delegating to the scenario builder.
        Self::build_scenario(documents, question, plan, Scenario::HappyBoundary)
    }

    /// Build the end-to-end trace under a [`Scenario`] — the SAME deterministic pipeline, varying ONLY
    /// the probe's risk profile and the governance decision, never the authority boundaries. Each
    /// downstream object is still DERIVED only from its predecessor and read from the frozen crates'
    /// real outputs; every scenario preserves no-execution / no-evidence / no-promotion / no-training.
    /// (`Scenario::HappyBoundary` reproduces the canonical [`CognitiveTrace::demo`] trace exactly.)
    pub fn build_scenario(
        documents: &[(String, String)],
        question: &str,
        plan: &str,
        scenario: Scenario,
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
            risk: scenario.risk(),
            reversibility: scenario.reversibility(),
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
            scenario.review_decision(),
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

// --- INT-3: the operator repro bundle (a reproducible DEMONSTRATION pack over the canonical trace).
//     `canonical_bundle` derives a fixed set of files purely from the trace; `verify_bundle` re-derives
//     that same set and byte-compares a provided bundle, trusting NOTHING on disk. The bundle shows what
//     the prototype can do — it creates no evidence and no authority, executes nothing, promotes nothing,
//     trains nothing. The filesystem I/O (writing/reading the pack) lives only in the binary shell. ---

/// The bundle's canonical file names (content files first, then the manifest). `bundle` writes exactly
/// these and `bundle-verify` re-derives exactly these. Pinned as data so a test/gate asserts the set.
pub const BUNDLE_TRACE_FILE: &str = "trace.json";
pub const BUNDLE_REPORT_FILE: &str = "report.txt";
pub const BUNDLE_QUESTIONS_FILE: &str = "questions.txt";
pub const BUNDLE_MANIFEST_FILE: &str = "manifest.json";

/// All bundle file names, in write order. The manifest is last because it hashes the content files.
pub const BUNDLE_FILES: [&str; 4] = [
    BUNDLE_TRACE_FILE,
    BUNDLE_REPORT_FILE,
    BUNDLE_QUESTIONS_FILE,
    BUNDLE_MANIFEST_FILE,
];

/// The six-line INT-3 bundle boundary, embedded in the manifest and printed as the bundle summary.
/// Pinned as data so a test can assert every line is present.
pub const BUNDLE_BOUNDARY_LINES: [&str; 6] = [
    "The bundle demonstrates the prototype.",
    "It does not create evidence.",
    "It does not create authority.",
    "It does not execute.",
    "It does not promote.",
    "It does not train.",
];

/// One manifest entry: a content file and its deterministic content hash.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct BundleFileEntry {
    name: String,
    content_hash: String,
}

/// The replay proof recorded in the manifest: the canonical trace's own content hash plus a plain
/// statement of what it proves. Re-derivable — `verify_bundle` recomputes it as part of the manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct BundleReplayProof {
    canonical_trace_hash: String,
    replay: String,
}

/// The bundle manifest: schema, the hash algorithm (named honestly — it is Rust's `DefaultHasher`,
/// NOT a cryptographic digest), a hash of every CONTENT file, the replay proof, and the six-line
/// boundary. It is `Serialize` but NOT `Deserialize` (like every record here): it is re-derived and
/// byte-compared, never parsed back into authority. It does NOT hash itself (no fixpoint).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct BundleManifest {
    schema: String,
    hash_algorithm: String,
    files: Vec<BundleFileEntry>,
    replay_proof: BundleReplayProof,
    boundary: Vec<String>,
}

/// A deterministic, dependency-free content hash of a bundle file's bytes (Rust's `DefaultHasher`,
/// hex-encoded). This is a DEMONSTRABLE digest for the manifest; the load-bearing integrity check is
/// `verify_bundle`'s byte-for-byte re-derivation of every file, of which this hash is only a part.
fn bundle_content_hash(content: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// The canonical interrogation transcript for the bundle's `questions.txt`: the finite question menu
/// followed by every enumerated question and its answer, all derived from the canonical trace. Pure.
pub fn run_questions_doc() -> Result<String, TraceError> {
    Ok(CognitiveTrace::demo()?.questions_doc())
}

/// Build a manifest JSON from the already-derived content files, with the given replay-proof text.
/// Pure and deterministic (fixed field order, fixed file order); `serde_json::to_string_pretty` yields
/// identical bytes on every run, so the manifest re-derives byte-for-byte. (The `canonical_trace_hash`
/// field names this bundle's OWN deterministic trace hash — for a scenario bundle that is the scenario
/// trace, which is canonical for its scenario.)
fn bundle_manifest_with(content_files: &[(&'static str, String)], replay: &str) -> String {
    let files: Vec<BundleFileEntry> = content_files
        .iter()
        .map(|(name, content)| BundleFileEntry {
            name: (*name).to_string(),
            content_hash: bundle_content_hash(content),
        })
        .collect();
    let trace_json = content_files
        .iter()
        .find(|(name, _)| *name == BUNDLE_TRACE_FILE)
        .map(|(_, content)| content.as_str())
        .unwrap_or_default();
    let manifest = BundleManifest {
        schema: "cognitive-bundle-v0.1".to_string(),
        hash_algorithm: "rust-default-hasher-u64-hex".to_string(),
        files,
        replay_proof: BundleReplayProof {
            canonical_trace_hash: bundle_content_hash(trace_json),
            replay: replay.to_string(),
        },
        boundary: BUNDLE_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };
    serde_json::to_string_pretty(&manifest).expect("BundleManifest serializes")
}

/// The full canonical repro bundle as (filename, content) pairs in write order, INCLUDING the
/// manifest. Pure: every file is derived from the canonical trace via the frozen-track-backed
/// `CognitiveTrace::demo()`. This is exactly what `bundle` writes and what `bundle-verify` re-derives
/// and compares against — so the bundle is a reproducible DEMONSTRATION, never trusted as authority.
pub fn canonical_bundle() -> Result<Vec<(&'static str, String)>, TraceError> {
    Ok(trace_bundle(
        &CognitiveTrace::demo()?,
        "trace.json re-derives byte-identically from CognitiveTrace::demo()",
    ))
}

/// Verify a provided bundle (its files as (name, content) pairs read from disk) WITHOUT trusting it:
/// re-derive the canonical bundle purely and require every canonical file to be present and
/// byte-identical. A missing file is [`TraceError::BundleMissingFile`]; any tampered/stale/foreign
/// file (including the manifest) is [`TraceError::BundleMismatch`]. Returns `Ok(())` only on a full,
/// exact match — so a tampered bundle can never pass and no bundle file is ever trusted over the
/// re-derived canonical. Pure (no I/O).
pub fn verify_bundle(provided: &[(String, String)]) -> Result<(), TraceError> {
    compare_bundle(&canonical_bundle()?, provided)
}

/// Build the four-file repro bundle for ANY trace, with the given replay-proof text. Pure: every file
/// is derived from the trace itself (`to_json` / `to_report` / `questions_doc`) plus a manifest hashing
/// the three content files. This is the shared core of the canonical bundle and every scenario bundle.
fn trace_bundle(trace: &CognitiveTrace, replay: &str) -> Vec<(&'static str, String)> {
    let content: Vec<(&'static str, String)> = vec![
        (BUNDLE_TRACE_FILE, trace.to_json()),
        (BUNDLE_REPORT_FILE, trace.to_report()),
        (BUNDLE_QUESTIONS_FILE, trace.questions_doc()),
    ];
    let manifest = bundle_manifest_with(&content, replay);
    let mut files = content;
    files.push((BUNDLE_MANIFEST_FILE, manifest));
    files
}

/// Require every CANONICAL (re-derived) file to be present in `provided` and byte-identical. A missing
/// file is [`TraceError::BundleMissingFile`]; any tampered/stale/foreign file (including the manifest)
/// is [`TraceError::BundleMismatch`]. The shared comparison core of [`verify_bundle`] and
/// [`verify_scenario_bundle`] — it trusts nothing on disk, it only compares against the re-derivation.
fn compare_bundle(
    canonical: &[(&'static str, String)],
    provided: &[(String, String)],
) -> Result<(), TraceError> {
    for (name, content) in canonical {
        match provided
            .iter()
            .find(|(provided_name, _)| provided_name == name)
        {
            None => return Err(TraceError::BundleMissingFile((*name).to_string())),
            Some((_, provided_content)) => {
                if provided_content != content {
                    return Err(TraceError::BundleMismatch((*name).to_string()));
                }
            }
        }
    }
    Ok(())
}

// --- MTRACE-0: the multi-trace scenario pack. The SAME deterministic pipeline is run under several
//     scenarios that vary the probe risk and the governance decision, producing several CognitiveTrace
//     bundles — each proving the SAME authority boundary (no execution / no evidence / no promotion /
//     no training) under a different review/observation/promotion outcome. Scenarios vary the path;
//     they do not vary the authority. ---

/// The six-line MTRACE-0 boundary, embedded in the scenario-pack manifest. Pinned as data so a test
/// can assert every line is present.
pub const MTRACE_BOUNDARY_LINES: [&str; 6] = [
    "Scenarios vary the path.",
    "They do not vary the authority.",
    "Nothing executes.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The scenario-pack manifest file name (lists every scenario; re-derived and byte-compared on verify).
pub const PACK_MANIFEST_FILE: &str = "pack-manifest.json";

/// A deterministic scenario over the SAME authority chain. It varies ONLY the probe's risk profile and
/// the governance decision — never the authority boundaries — so each scenario produces a distinct
/// path (different review/intent/observation statuses and ids) that still proves no execution, no
/// evidence, no promotion, and no training. The set is finite and enum-backed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    /// Governance approves; intent `requires_operator`; observation `requires_review`; promotion
    /// rejected. This IS the canonical [`CognitiveTrace::demo`] trace, byte-for-byte.
    HappyBoundary,
    /// Governance rejects a queued probe; intent `blocked`; observation `rejected`; promotion rejected.
    ReviewRejected,
    /// Governance defers a queued probe; intent `blocked`; observation `rejected`; promotion rejected.
    ReviewDeferred,
    /// The probe is classified `blocked` (high-risk AND irreversible): there is no approval path
    /// (approving a blocked probe is refused by the frozen layer), so nothing can execute.
    HighRiskBlocked,
}

impl Scenario {
    /// Every scenario, in canonical order. Pinned as data so a test/the pack can assert the full set.
    pub const ALL: [Scenario; 4] = [
        Scenario::HappyBoundary,
        Scenario::ReviewRejected,
        Scenario::ReviewDeferred,
        Scenario::HighRiskBlocked,
    ];

    /// The stable slug for this scenario. Exhaustive match — a new variant forces a slug here.
    pub fn slug(self) -> &'static str {
        match self {
            Scenario::HappyBoundary => "happy-boundary",
            Scenario::ReviewRejected => "review-rejected",
            Scenario::ReviewDeferred => "review-deferred",
            Scenario::HighRiskBlocked => "high-risk-blocked",
        }
    }

    /// A one-line description of the scenario's path (shown by `scenarios`). Exhaustive match.
    pub fn describe(self) -> &'static str {
        match self {
            Scenario::HappyBoundary => {
                "governance approves; intent requires_operator; observation requires_review; promotion rejected"
            }
            Scenario::ReviewRejected => {
                "governance rejects; intent blocked; observation rejected; promotion rejected"
            }
            Scenario::ReviewDeferred => {
                "governance defers; intent blocked; observation rejected; promotion rejected"
            }
            Scenario::HighRiskBlocked => {
                "probe classified blocked (high-risk AND irreversible); no approval path; no execution"
            }
        }
    }

    /// Parse a slug into a scenario. Fails CLOSED: any string that is not EXACTLY a known slug is `None`.
    pub fn from_slug(slug: &str) -> Option<Scenario> {
        Scenario::ALL.into_iter().find(|s| s.slug() == slug)
    }

    /// The governance decision applied in this scenario. A `blocked` probe (high-risk-blocked) can only
    /// be rejected or deferred — never approved (the frozen layer refuses) — so it uses Rejected.
    /// Exhaustive, no wildcard: a new scenario must choose its decision explicitly.
    fn review_decision(self) -> ReviewDecision {
        match self {
            Scenario::HappyBoundary => ReviewDecision::Approved,
            Scenario::ReviewRejected => ReviewDecision::Rejected,
            Scenario::ReviewDeferred => ReviewDecision::Deferred,
            Scenario::HighRiskBlocked => ReviewDecision::Rejected,
        }
    }

    /// The probe risk for this scenario. Only high-risk-blocked is at/above the frozen `HIGH_RISK`
    /// threshold (700); the others reuse the canonical low risk so their probe stays queued.
    fn risk(self) -> i64 {
        match self {
            Scenario::HappyBoundary => 100,
            Scenario::ReviewRejected => 100,
            Scenario::ReviewDeferred => 100,
            Scenario::HighRiskBlocked => 800,
        }
    }

    /// The probe reversibility for this scenario. Only high-risk-blocked is at/below the frozen
    /// `LOW_REVERSIBILITY` threshold (300); the others reuse the canonical high reversibility.
    fn reversibility(self) -> i64 {
        match self {
            Scenario::HappyBoundary => 900,
            Scenario::ReviewRejected => 900,
            Scenario::ReviewDeferred => 900,
            Scenario::HighRiskBlocked => 100,
        }
    }
}

impl CognitiveTrace {
    /// The interrogation transcript for THIS trace: the finite question menu followed by every
    /// enumerated question and its answer (about this trace). Pure — formats only recorded fields.
    /// (`CognitiveTrace::demo().questions_doc()` is exactly the canonical INT-2/INT-3 questions doc.)
    pub fn questions_doc(&self) -> String {
        let mut out = list_questions();
        for q in TraceQuestion::ALL {
            out.push_str(&format!("\n=== {} ===\n", q.slug()));
            out.push_str(&self.answer(q));
        }
        out
    }
}

/// The deterministic trace for a scenario: run the fixed [`demo_inputs`] through the pipeline under the
/// scenario's risk profile and governance decision. `Scenario::HappyBoundary` reproduces the canonical
/// [`CognitiveTrace::demo`] trace byte-for-byte. Pure and replayable.
pub fn scenario_trace(scenario: Scenario) -> Result<CognitiveTrace, TraceError> {
    let (documents, question, plan) = demo_inputs();
    CognitiveTrace::build_scenario(&documents, &question, &plan, scenario)
}

/// The four-file repro bundle for a scenario (trace.json / report.txt / questions.txt / manifest.json),
/// purely derived from the scenario's trace. Re-derivable and byte-comparable — exactly like the
/// canonical bundle, but for this scenario's path.
pub fn scenario_bundle(scenario: Scenario) -> Result<Vec<(&'static str, String)>, TraceError> {
    let replay = format!(
        "trace.json re-derives byte-identically from CognitiveTrace::build_scenario(\"{}\")",
        scenario.slug()
    );
    Ok(trace_bundle(&scenario_trace(scenario)?, &replay))
}

/// Verify a provided scenario bundle WITHOUT trusting it: re-derive the scenario's canonical bundle and
/// byte-compare every file. A missing file is [`TraceError::BundleMissingFile`]; any tampered/stale/
/// foreign file (including the manifest) is [`TraceError::BundleMismatch`]. Pure (no I/O).
pub fn verify_scenario_bundle(
    scenario: Scenario,
    provided: &[(String, String)],
) -> Result<(), TraceError> {
    compare_bundle(&scenario_bundle(scenario)?, provided)
}

/// One row of the scenario-pack manifest: the scenario's identity plus the distinguishing statuses and
/// its trace hash, and the boundary verdicts that hold for it (always no-grant / no-execution / no-
/// training). `Serialize` but NOT `Deserialize` — re-derived and byte-compared, never parsed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ScenarioEntry {
    slug: String,
    description: String,
    review_decision: String,
    probe_status: String,
    execution_status: String,
    observation_status: String,
    promotion_status: String,
    grants_promotion: bool,
    nothing_executed: bool,
    nothing_becomes_evidence: bool,
    training_justified: bool,
    trace_hash: String,
}

/// The scenario-pack manifest: every scenario row plus the six-line boundary. `Serialize` but NOT
/// `Deserialize` — re-derived and byte-compared on verify, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ScenarioPackManifest {
    schema: String,
    scenarios: Vec<ScenarioEntry>,
    boundary: Vec<String>,
}

/// The deterministic scenario-pack manifest JSON: one row per scenario (its distinguishing statuses +
/// trace hash + the always-holding no-grant/no-execution/no-training verdicts) plus the boundary. Pure.
pub fn scenario_pack_manifest() -> Result<String, TraceError> {
    let mut scenarios = Vec::new();
    for scenario in Scenario::ALL {
        let trace = scenario_trace(scenario)?;
        scenarios.push(ScenarioEntry {
            slug: scenario.slug().to_string(),
            description: scenario.describe().to_string(),
            review_decision: trace.review_decision().to_string(),
            probe_status: trace.probe_status.clone(),
            execution_status: trace.execution_status().to_string(),
            observation_status: trace.observation_status().to_string(),
            promotion_status: trace.promotion_status().to_string(),
            grants_promotion: trace.grants_promotion(),
            nothing_executed: trace.nothing_executed(),
            nothing_becomes_evidence: trace.nothing_becomes_evidence(),
            training_justified: trace.training_justified(),
            trace_hash: bundle_content_hash(&trace.to_json()),
        });
    }
    let manifest = ScenarioPackManifest {
        schema: "cognitive-scenario-pack-v0.1".to_string(),
        scenarios,
        boundary: MTRACE_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };
    Ok(serde_json::to_string_pretty(&manifest).expect("ScenarioPackManifest serializes"))
}

/// Verify a provided scenario-pack manifest by RE-DERIVING the canonical one and byte-comparing. A
/// mismatch (tampered/stale/foreign) is refused ([`TraceError::BundleMismatch`]). Pure (no I/O).
pub fn verify_scenario_pack_manifest(provided: &str) -> Result<(), TraceError> {
    if provided == scenario_pack_manifest()? {
        Ok(())
    } else {
        Err(TraceError::BundleMismatch(PACK_MANIFEST_FILE.to_string()))
    }
}

/// The `scenarios` command: list the finite scenario set (slug + one-line path description). Pure.
pub fn list_scenarios() -> String {
    let mut out = String::from(
        "cognitive-demo — deterministic scenarios (each proves the SAME authority boundary):\n",
    );
    for s in Scenario::ALL {
        out.push_str(&format!("    {:<20} {}\n", s.slug(), s.describe()));
    }
    out
}

/// Verify a WHOLE provided scenario pack (every scenario's bundle files + the pack manifest) WITHOUT
/// trusting it: re-derive each scenario bundle and the pack manifest and byte-compare. A missing
/// scenario is [`TraceError::BundleMissingFile`]; any tampered/stale/foreign file is
/// [`TraceError::BundleMismatch`]. The pure whole-pack core the matrix commands verify against. Pure.
pub fn verify_scenario_pack(
    bundles: &[(String, Vec<(String, String)>)],
    pack_manifest: &str,
) -> Result<(), TraceError> {
    for scenario in Scenario::ALL {
        match bundles.iter().find(|(slug, _)| slug == scenario.slug()) {
            None => return Err(TraceError::BundleMissingFile(scenario.slug().to_string())),
            Some((_, files)) => verify_scenario_bundle(scenario, files)?,
        }
    }
    verify_scenario_pack_manifest(pack_manifest)
}

// --- MTRACE-1: the scenario boundary-coverage matrix. A deterministic coverage report DERIVED from the
//     scenario set: for every scenario (path) it records the path's statuses AND proves the four
//     authority boundaries (no_execution / no_evidence / no_promotion / no_training) hold, plus a
//     coverage summary. The matrix is purely re-derived (it never trusts the pack files); verify/report
//     re-derive and byte-compare, refusing any tampered matrix or pack. The matrix summarizes coverage;
//     it does not create authority. ---

/// The five-line MTRACE-1 boundary, embedded in the matrix. Pinned as data so a test can assert it.
pub const MATRIX_BOUNDARY_LINES: [&str; 5] = [
    "The matrix summarizes coverage.",
    "It does not create authority.",
    "It does not execute.",
    "It does not promote.",
    "It does not train.",
];

/// One row of the coverage matrix: a scenario's PATH (its review/probe/intent/observation/promotion
/// statuses + training verdict) and the four BOUNDARY cells it proves (always all true). `Serialize`
/// but NOT `Deserialize` — re-derived and byte-compared, never parsed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MatrixRow {
    slug: String,
    description: String,
    review_status: String,
    probe_status: String,
    intent_status: String,
    observation_status: String,
    promotion_status: String,
    training_verdict: String,
    no_execution: bool,
    no_evidence: bool,
    no_promotion: bool,
    no_training: bool,
}

/// The coverage summary: how many scenarios × boundaries were proven, and the DISTINCT path statuses
/// (proving the variation is real, not cosmetic). `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct MatrixCoverage {
    scenario_count: usize,
    boundary_count: usize,
    boundaries: Vec<String>,
    cells_total: usize,
    cells_proven: usize,
    all_boundaries_hold: bool,
    distinct_review_statuses: Vec<String>,
    distinct_intent_statuses: Vec<String>,
    distinct_probe_statuses: Vec<String>,
}

/// The scenario boundary-coverage matrix: one row per scenario, the coverage summary, and the boundary.
/// `Serialize` but NOT `Deserialize` — re-derived and byte-compared on verify, never parsed back into
/// authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ScenarioMatrix {
    schema: String,
    scenarios: Vec<MatrixRow>,
    coverage: MatrixCoverage,
    boundary: Vec<String>,
}

/// The training verdict token (always `training_not_justified` here — P12 stays false).
fn training_verdict_token(training_justified: bool) -> &'static str {
    if training_justified {
        "training_justified"
    } else {
        "training_not_justified"
    }
}

/// Sort + dedup a list of status tokens for a deterministic, stable distinct-set in the coverage.
fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

/// Build the canonical coverage matrix from the scenario set. Pure and deterministic: every row and the
/// coverage summary are computed from each scenario's re-derived trace via read-only accessors.
fn canonical_scenario_matrix() -> Result<ScenarioMatrix, TraceError> {
    let mut rows = Vec::new();
    let mut review_statuses = Vec::new();
    let mut intent_statuses = Vec::new();
    let mut probe_statuses = Vec::new();
    let mut cells_proven = 0usize;
    for scenario in Scenario::ALL {
        let trace = scenario_trace(scenario)?;
        let no_execution = trace.nothing_executed();
        let no_evidence = trace.nothing_becomes_evidence();
        let no_promotion = trace.promotion_refused();
        let no_training = !trace.training_justified();
        cells_proven += [no_execution, no_evidence, no_promotion, no_training]
            .iter()
            .filter(|cell| **cell)
            .count();
        review_statuses.push(trace.review_decision().to_string());
        intent_statuses.push(trace.execution_status().to_string());
        probe_statuses.push(trace.probe_status.clone());
        rows.push(MatrixRow {
            slug: scenario.slug().to_string(),
            description: scenario.describe().to_string(),
            review_status: trace.review_decision().to_string(),
            probe_status: trace.probe_status.clone(),
            intent_status: trace.execution_status().to_string(),
            observation_status: trace.observation_status().to_string(),
            promotion_status: trace.promotion_status().to_string(),
            training_verdict: training_verdict_token(trace.training_justified()).to_string(),
            no_execution,
            no_evidence,
            no_promotion,
            no_training,
        });
    }
    let scenario_count = rows.len();
    let boundary_count = 4;
    let cells_total = scenario_count * boundary_count;
    let coverage = MatrixCoverage {
        scenario_count,
        boundary_count,
        boundaries: vec![
            "no_execution".to_string(),
            "no_evidence".to_string(),
            "no_promotion".to_string(),
            "no_training".to_string(),
        ],
        cells_total,
        cells_proven,
        all_boundaries_hold: cells_proven == cells_total,
        distinct_review_statuses: sorted_unique(review_statuses),
        distinct_intent_statuses: sorted_unique(intent_statuses),
        distinct_probe_statuses: sorted_unique(probe_statuses),
    };
    Ok(ScenarioMatrix {
        schema: "cognitive-scenario-matrix-v0.1".to_string(),
        scenarios: rows,
        coverage,
        boundary: MATRIX_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// The deterministic scenario coverage matrix as pretty JSON. Pure: every cell is derived from the
/// scenario set's re-derived traces. This is what `scenario-matrix --out` writes.
pub fn scenario_matrix() -> Result<String, TraceError> {
    Ok(serde_json::to_string_pretty(&canonical_scenario_matrix()?)
        .expect("ScenarioMatrix serializes"))
}

/// Verify a provided matrix JSON by RE-DERIVING the canonical matrix and byte-comparing. A mismatch
/// (tampered/stale/foreign) is refused ([`TraceError::MatrixMismatch`]). Pure (no I/O).
pub fn verify_scenario_matrix(provided: &str) -> Result<(), TraceError> {
    if provided == scenario_matrix()? {
        Ok(())
    } else {
        Err(TraceError::MatrixMismatch)
    }
}

/// Render the plain-text coverage report for a provided matrix JSON — but only after RE-DERIVING the
/// canonical matrix and confirming the provided JSON IS it (byte-for-byte). The report is then rendered
/// from the RE-DERIVED canonical matrix struct (never the provided file's claims), so a tampered matrix
/// can never be laundered into a clean report. This is what `scenario-matrix-report` writes. Pure.
pub fn scenario_matrix_report(provided: &str) -> Result<String, TraceError> {
    let canonical = canonical_scenario_matrix()?;
    let canonical_json =
        serde_json::to_string_pretty(&canonical).expect("ScenarioMatrix serializes");
    if provided != canonical_json {
        return Err(TraceError::MatrixMismatch);
    }
    Ok(render_scenario_matrix(&canonical))
}

/// Render the coverage matrix as a plain operator report — pure FORMATTING of the matrix's recorded
/// fields (no new verdict, no authority object). Shows each scenario's path × boundary cells, the
/// coverage summary, and the boundary.
fn render_scenario_matrix(matrix: &ScenarioMatrix) -> String {
    let mut out = String::new();
    out.push_str("COGNITIVE OS — SCENARIO BOUNDARY COVERAGE MATRIX\n");
    out.push_str(&format!("schema: {}\n", matrix.schema));
    out.push_str("(a coverage view of the scenario pack; it summarizes, it does not act)\n\n");

    out.push_str("PER-SCENARIO PATH x BOUNDARY\n");
    for row in &matrix.scenarios {
        out.push_str(&format!("[{}]\n", row.slug));
        out.push_str(&format!("    review:       {}\n", row.review_status));
        out.push_str(&format!("    probe:        {}\n", row.probe_status));
        out.push_str(&format!("    intent:       {}\n", row.intent_status));
        out.push_str(&format!("    observation:  {}\n", row.observation_status));
        out.push_str(&format!("    promotion:    {}\n", row.promotion_status));
        out.push_str(&format!("    training:     {}\n", row.training_verdict));
        out.push_str(&format!(
            "    boundary:     no_execution={} no_evidence={} no_promotion={} no_training={}\n",
            row.no_execution, row.no_evidence, row.no_promotion, row.no_training
        ));
    }

    out.push_str("\nCOVERAGE\n");
    out.push_str(&format!(
        "    scenarios:           {}\n",
        matrix.coverage.scenario_count
    ));
    out.push_str(&format!(
        "    boundaries:          {} ({})\n",
        matrix.coverage.boundary_count,
        matrix.coverage.boundaries.join(", ")
    ));
    out.push_str(&format!(
        "    cells proven:        {}/{}\n",
        matrix.coverage.cells_proven, matrix.coverage.cells_total
    ));
    out.push_str(&format!(
        "    all_boundaries_hold: {}\n",
        matrix.coverage.all_boundaries_hold
    ));
    out.push_str(&format!(
        "    distinct review:     {}\n",
        matrix.coverage.distinct_review_statuses.join(", ")
    ));
    out.push_str(&format!(
        "    distinct intent:     {}\n",
        matrix.coverage.distinct_intent_statuses.join(", ")
    ));
    out.push_str(&format!(
        "    distinct probe:      {}\n",
        matrix.coverage.distinct_probe_statuses.join(", ")
    ));

    out.push_str("\nSUMMARY\n");
    out.push_str("    Every scenario varies the path but preserves the authority boundary.\n");
    out.push_str(
        "    Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.\n\n",
    );

    out.push_str("BOUNDARY\n");
    for line in MATRIX_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

// --- MTRACE-2: the scenario failure-injection / boundary-regression pack. Where MTRACE-0/1 prove the
//     GOOD paths preserve the authority boundary, MTRACE-2 proves the BAD paths cannot smuggle authority:
//     a finite, enum-backed set of negative scenarios, each of which DETERMINISTICALLY forges a forbidden
//     authority claim onto a canonical artifact (a trace, a scenario bundle, the report, or the coverage
//     matrix) and then runs the EXISTING re-derive-and-byte-compare verifier, which REFUSES it. Nothing is
//     trusted: the forged bytes are never parsed back into authority (every artifact type is Serialize-only),
//     they are only COMPARED against the freshly re-derived canonical and rejected with a typed error. The
//     pack records, per case, that the forgery genuinely altered the canonical bytes AND the exact typed
//     rejection reason. Failure cases attack the boundary; they do not weaken it. ---

/// The seven-line MTRACE-2 boundary, embedded in the failure pack. Pinned as data so a test can assert it.
pub const FAILURE_BOUNDARY_LINES: [&str; 7] = [
    "Failure cases attack the boundary.",
    "They do not weaken it.",
    "Forged authority is rejected.",
    "Nothing executes.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The failure-pack file names (the rejection record + its rendered report). Re-derived and byte-compared
/// on verify, exactly like the bundle files.
pub const FAILURE_PACK_FILE: &str = "failure-pack.json";
/// The rendered failure-pack report file name.
pub const FAILURE_REPORT_FILE: &str = "failure-report.txt";
/// Both failure-pack files, in write order (so the shell can read them back for `failure-verify`).
pub const FAILURE_PACK_FILES: [&str; 2] = [FAILURE_PACK_FILE, FAILURE_REPORT_FILE];

/// The finite, enumerated set of negative scenarios. Each forges ONE forbidden authority claim onto a
/// canonical artifact and is REFUSED by the existing re-derive-and-byte-compare verifier. The set is
/// CLOSED — there is no free-form path; an unrecognized slug maps to no variant ([`FailureCase::from_slug`]
/// returns `None`). Each variant maps to one fixed forgery and the surface that rejects it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureCase {
    /// Forge the execution intent to claim the probe ran. Refused by `verify_trace_json`.
    ForgedExecution,
    /// Forge the observation to claim evidence authority. Refused by `verify_trace_json`.
    ForgedEvidence,
    /// Forge the promotion request to grant a promotion. Refused by `verify_trace_json`.
    ForgedPromotion,
    /// Forge the P12 training gate toward justified. Refused by `verify_trace_json`.
    ForgedTraining,
    /// Forge a rejected governance review to read as approved. Refused by `verify_scenario_bundle`.
    ForgedReview,
    /// Forge the operator report to narrate that execution/evidence occurred. Refused by `verify_bundle`.
    ForgedReport,
    /// Forge the coverage matrix to hide a failed boundary cell. Refused by `verify_scenario_matrix`.
    ForgedMatrix,
}

impl FailureCase {
    /// Every failure case, in canonical order. Pinned as data so the pack can assert the full set.
    pub const ALL: [FailureCase; 7] = [
        FailureCase::ForgedExecution,
        FailureCase::ForgedEvidence,
        FailureCase::ForgedPromotion,
        FailureCase::ForgedTraining,
        FailureCase::ForgedReview,
        FailureCase::ForgedReport,
        FailureCase::ForgedMatrix,
    ];

    /// The stable slug for this case. Exhaustive match — a new variant forces a slug here.
    pub fn slug(self) -> &'static str {
        match self {
            FailureCase::ForgedExecution => "forged-execution",
            FailureCase::ForgedEvidence => "forged-evidence",
            FailureCase::ForgedPromotion => "forged-promotion",
            FailureCase::ForgedTraining => "forged-training",
            FailureCase::ForgedReview => "forged-review",
            FailureCase::ForgedReport => "forged-report",
            FailureCase::ForgedMatrix => "forged-matrix",
        }
    }

    /// A one-line description of the attack (shown by `failure-cases`). Exhaustive match.
    pub fn describe(self) -> &'static str {
        match self {
            FailureCase::ForgedExecution => "forge an executed status onto the execution intent",
            FailureCase::ForgedEvidence => {
                "forge evidence authority onto the quarantined observation"
            }
            FailureCase::ForgedPromotion => "forge the promotion request to grant a promotion",
            FailureCase::ForgedTraining => "forge the P12 training gate toward justified",
            FailureCase::ForgedReview => "forge a rejected governance review to read as approved",
            FailureCase::ForgedReport => {
                "forge the operator report to narrate execution and evidence"
            }
            FailureCase::ForgedMatrix => "forge the coverage matrix to hide a failed boundary cell",
        }
    }

    /// Parse a slug into a case. Fails CLOSED: any string that is not EXACTLY a known slug is `None`.
    pub fn from_slug(slug: &str) -> Option<FailureCase> {
        FailureCase::ALL.into_iter().find(|c| c.slug() == slug)
    }

    /// The surface (verifier) that rejects this case's forgery. Prose only — no affirmative authority
    /// token, so recording it in the pack never leaks a forged claim into trusted state. Exhaustive match.
    fn target_surface(self) -> &'static str {
        match self {
            FailureCase::ForgedExecution
            | FailureCase::ForgedEvidence
            | FailureCase::ForgedPromotion
            | FailureCase::ForgedTraining => "trace-json (verify_trace_json re-derives + byte-compares)",
            FailureCase::ForgedReview => {
                "scenario-bundle:review-rejected (verify_scenario_bundle re-derives + byte-compares)"
            }
            FailureCase::ForgedReport => "bundle (verify_bundle re-derives + byte-compares)",
            FailureCase::ForgedMatrix => {
                "matrix (verify_scenario_matrix re-derives + byte-compares)"
            }
        }
    }

    /// A prose description of the forbidden authority the forgery TRIES to inject. Deliberately prose (no
    /// affirmative-authority JSON token), so the pack records the ATTACK without ever encoding the claim as
    /// trusted state. Exhaustive match.
    fn forbidden_claim(self) -> &'static str {
        match self {
            FailureCase::ForgedExecution => {
                "the execution intent is altered to claim the probe ran"
            }
            FailureCase::ForgedEvidence => "the observation is altered to claim evidence authority",
            FailureCase::ForgedPromotion => "the promotion request is altered to grant a promotion",
            FailureCase::ForgedTraining => "the P12 training gate is altered toward justified",
            FailureCase::ForgedReview => {
                "a rejected governance review is altered to read as approved"
            }
            FailureCase::ForgedReport => {
                "the operator report is altered to narrate that execution and evidence occurred"
            }
            FailureCase::ForgedMatrix => {
                "the coverage matrix is altered to hide a failed boundary cell"
            }
        }
    }

    /// The exact AFFIRMATIVE-authority substring the forgery injects into the forged artifact — proof that
    /// the forgery genuinely encodes FORBIDDEN authority (not a benign byte-change that would also be
    /// rejected by byte-compare). Used ONLY to inspect the in-memory forged artifact at attempt time; it is
    /// never persisted into the pack (the pack records only the boolean result), so no affirmative claim
    /// becomes trusted state. Exhaustive match.
    fn forbidden_token(self) -> &'static str {
        match self {
            FailureCase::ForgedExecution => "\"execution_status\": \"executed\"",
            FailureCase::ForgedEvidence => "\"observation_authority\": \"evidence\"",
            FailureCase::ForgedPromotion => "\"grants_promotion\": true",
            FailureCase::ForgedTraining => "\"training_justified\": true",
            FailureCase::ForgedReview => "\"review_decision\": \"approved\"",
            FailureCase::ForgedReport => "The promotion was granted.",
            FailureCase::ForgedMatrix => "\"no_execution\": false",
        }
    }
}

/// The outcome of attempting one forgery: whether the forgery genuinely altered the canonical bytes, whether
/// it injected the specific FORBIDDEN authority token (not just any change), and the REAL verdict the existing
/// verifier returned for the forged artifact. All are observed, never
/// asserted — the pack records what actually happened, so a forgery that slipped through would surface as
/// `forgery_applied=false` or a non-error `verdict` and fail the rejection tests.
struct FailureAttempt {
    forgery_applied: bool,
    injects_forbidden: bool,
    verdict: Result<(), TraceError>,
}

/// Forge ONE file inside a canonical bundle (by name) via a deterministic substring replacement, returning
/// the provided-bundle-shaped copy. The other files are passed through unchanged. Pure.
fn forge_bundle_file(
    bundle: &[(&'static str, String)],
    target_file: &str,
    from: &str,
    to: &str,
) -> Vec<(String, String)> {
    bundle
        .iter()
        .map(|(name, content)| {
            if *name == target_file {
                (name.to_string(), content.replace(from, to))
            } else {
                (name.to_string(), content.clone())
            }
        })
        .collect()
}

/// The content of a named file in a provided-shaped bundle (empty if absent), for inspecting a forged file.
fn bundle_file_content(bundle: &[(String, String)], name: &str) -> String {
    bundle
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, c)| c.clone())
        .unwrap_or_default()
}

/// The content of a named file in a CANONICAL (&str-keyed) bundle (empty if absent), for the un-forged
/// baseline when computing whether a bundle forgery actually changed the target file.
fn canonical_bundle_file(bundle: &[(&'static str, String)], name: &str) -> String {
    bundle
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, c)| c.clone())
        .unwrap_or_default()
}

/// Run ONE failure case: build the canonical artifact, apply the case's deterministic forgery, and run the
/// EXISTING re-derive-and-byte-compare verifier on the forged artifact. Returns whether the forgery changed
/// the canonical bytes, whether it injected the case's specific FORBIDDEN authority token (so a benign change
/// cannot masquerade as a forbidden-authority forgery), and the verifier's real verdict (an `Err` = refused).
/// The canonical artifact is always built fresh and the forgery operates on a COPY, so this never mutates any
/// canonical data. Pure.
fn run_failure_case(case: FailureCase) -> Result<FailureAttempt, TraceError> {
    let token = case.forbidden_token();
    match case {
        FailureCase::ForgedExecution => {
            let canonical = CognitiveTrace::demo()?.to_json();
            let forged = canonical.replace(
                "\"execution_status\": \"requires_operator\"",
                "\"execution_status\": \"executed\"",
            );
            Ok(FailureAttempt {
                forgery_applied: forged != canonical,
                injects_forbidden: forged.contains(token),
                verdict: verify_trace_json(&forged).map(|_| ()),
            })
        }
        FailureCase::ForgedEvidence => {
            let canonical = CognitiveTrace::demo()?.to_json();
            let forged = canonical.replace(
                "\"observation_authority\": \"observation_only\"",
                "\"observation_authority\": \"evidence\"",
            );
            Ok(FailureAttempt {
                forgery_applied: forged != canonical,
                injects_forbidden: forged.contains(token),
                verdict: verify_trace_json(&forged).map(|_| ()),
            })
        }
        FailureCase::ForgedPromotion => {
            let canonical = CognitiveTrace::demo()?.to_json();
            let forged =
                canonical.replace("\"grants_promotion\": false", "\"grants_promotion\": true");
            Ok(FailureAttempt {
                forgery_applied: forged != canonical,
                injects_forbidden: forged.contains(token),
                verdict: verify_trace_json(&forged).map(|_| ()),
            })
        }
        FailureCase::ForgedTraining => {
            let canonical = CognitiveTrace::demo()?.to_json();
            let forged = canonical.replace(
                "\"training_justified\": false",
                "\"training_justified\": true",
            );
            Ok(FailureAttempt {
                forgery_applied: forged != canonical,
                injects_forbidden: forged.contains(token),
                verdict: verify_trace_json(&forged).map(|_| ()),
            })
        }
        FailureCase::ForgedReview => {
            // The review-rejected scenario's trace records a `rejected` decision; forge it to `approved`.
            let canonical = scenario_bundle(Scenario::ReviewRejected)?;
            let original = canonical_bundle_file(&canonical, BUNDLE_TRACE_FILE);
            let forged = forge_bundle_file(
                &canonical,
                BUNDLE_TRACE_FILE,
                "\"review_decision\": \"rejected\"",
                "\"review_decision\": \"approved\"",
            );
            let forged_trace = bundle_file_content(&forged, BUNDLE_TRACE_FILE);
            Ok(FailureAttempt {
                forgery_applied: forged_trace != original,
                injects_forbidden: forged_trace.contains(token),
                verdict: verify_scenario_bundle(Scenario::ReviewRejected, &forged),
            })
        }
        FailureCase::ForgedReport => {
            // The canonical report states the no-execution/no-evidence/no-promotion summary; forge it to
            // narrate that execution and evidence occurred.
            let canonical = canonical_bundle()?;
            let original = canonical_bundle_file(&canonical, BUNDLE_REPORT_FILE);
            let forged = forge_bundle_file(
                &canonical,
                BUNDLE_REPORT_FILE,
                "Nothing executed. Nothing became evidence. Nothing was promoted.",
                "Execution ran. The observation became evidence. The promotion was granted.",
            );
            let forged_report = bundle_file_content(&forged, BUNDLE_REPORT_FILE);
            Ok(FailureAttempt {
                forgery_applied: forged_report != original,
                injects_forbidden: forged_report.contains(token),
                verdict: verify_bundle(&forged),
            })
        }
        FailureCase::ForgedMatrix => {
            // Flip the FIRST row's no_execution cell to false while the summary still claims full coverage
            // (cells_proven 16 / all_boundaries_hold true) — a matrix that HIDES a failed boundary cell.
            let canonical = scenario_matrix()?;
            let forged = canonical.replacen("\"no_execution\": true", "\"no_execution\": false", 1);
            Ok(FailureAttempt {
                forgery_applied: forged != canonical,
                injects_forbidden: forged.contains(token),
                verdict: verify_scenario_matrix(&forged),
            })
        }
    }
}

/// One recorded rejection: the case identity, the attack and the surface that refused it, whether the
/// forgery genuinely altered the canonical bytes, whether it was rejected, and the EXACT typed rejection
/// reason. `Serialize` but NOT `Deserialize` — re-derived and byte-compared, never parsed back into
/// authority. The `forbidden_claim`/`rejection_reason` are prose; no affirmative-authority token is stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct FailureRejection {
    slug: String,
    attack: String,
    target_surface: String,
    forbidden_claim: String,
    forgery_applied: bool,
    injects_forbidden: bool,
    rejected: bool,
    rejection_reason: String,
}

/// The coverage summary for the failure pack: how many cases, whether every forgery genuinely altered the
/// canonical bytes, injected its forbidden authority token, AND was rejected, and this pack's canonical trace
/// hash (ties the pack to the real, unchanged canonical). `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct FailureSummary {
    case_count: usize,
    all_forged: bool,
    all_inject_forbidden: bool,
    all_rejected: bool,
    canonical_trace_hash: String,
}

/// The failure pack: every recorded rejection, the coverage summary, and the boundary. `Serialize` but NOT
/// `Deserialize` — re-derived and byte-compared on verify, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct FailurePack {
    schema: String,
    cases: Vec<FailureRejection>,
    summary: FailureSummary,
    boundary: Vec<String>,
}

/// Build the canonical failure pack: run every forgery through the existing verifier and record the real
/// outcome. Pure and deterministic — every case is re-run from fixed inputs; the `rejected`/`rejection_reason`
/// fields are OBSERVED from the verifier, not asserted, so a forgery that slipped through would record
/// `rejected=false` and fail the rejection tests rather than being silently laundered.
fn canonical_failure_pack() -> Result<FailurePack, TraceError> {
    let mut cases = Vec::new();
    let mut all_forged = true;
    let mut all_inject_forbidden = true;
    let mut all_rejected = true;
    for case in FailureCase::ALL {
        let attempt = run_failure_case(case)?;
        let rejected = attempt.verdict.is_err();
        let rejection_reason = match &attempt.verdict {
            Err(e) => e.to_string(),
            Ok(()) => "ACCEPTED — the forgery was NOT rejected".to_string(),
        };
        if !attempt.forgery_applied {
            all_forged = false;
        }
        if !attempt.injects_forbidden {
            all_inject_forbidden = false;
        }
        if !rejected {
            all_rejected = false;
        }
        cases.push(FailureRejection {
            slug: case.slug().to_string(),
            attack: case.describe().to_string(),
            target_surface: case.target_surface().to_string(),
            forbidden_claim: case.forbidden_claim().to_string(),
            forgery_applied: attempt.forgery_applied,
            injects_forbidden: attempt.injects_forbidden,
            rejected,
            rejection_reason,
        });
    }
    let summary = FailureSummary {
        case_count: cases.len(),
        all_forged,
        all_inject_forbidden,
        all_rejected,
        canonical_trace_hash: bundle_content_hash(&CognitiveTrace::demo()?.to_json()),
    };
    Ok(FailurePack {
        schema: "cognitive-failure-pack-v0.1".to_string(),
        cases,
        summary,
        boundary: FAILURE_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// The deterministic failure pack as pretty JSON: one entry per negative scenario recording that its forgery
/// was rejected, plus the coverage summary and the boundary. This is what `failure-pack --out` writes. Pure.
pub fn failure_pack() -> Result<String, TraceError> {
    Ok(serde_json::to_string_pretty(&canonical_failure_pack()?).expect("FailurePack serializes"))
}

/// Render the plain-text failure report from a failure pack — pure FORMATTING of its recorded fields (the
/// per-case attack, surface, whether the forgery applied, REJECTED + the exact reason), the coverage summary,
/// and the boundary. No new verdict, no authority object.
fn render_failure_pack(pack: &FailurePack) -> String {
    let mut out = String::new();
    out.push_str("COGNITIVE OS — SCENARIO FAILURE-INJECTION / BOUNDARY REGRESSION PACK\n");
    out.push_str(&format!("schema: {}\n", pack.schema));
    out.push_str(
        "(each case forges forbidden authority and is REFUSED by re-derive byte-compare)\n\n",
    );

    out.push_str("PER-CASE FORGERY x REJECTION\n");
    for case in &pack.cases {
        out.push_str(&format!("[{}]\n", case.slug));
        out.push_str(&format!("    attack:           {}\n", case.attack));
        out.push_str(&format!("    forbidden claim:  {}\n", case.forbidden_claim));
        out.push_str(&format!("    surface:          {}\n", case.target_surface));
        out.push_str(&format!("    forgery applied:  {}\n", case.forgery_applied));
        out.push_str(&format!(
            "    injects forbidden:{}\n",
            case.injects_forbidden
        ));
        out.push_str(&format!(
            "    verdict:          {}\n",
            if case.rejected {
                "REJECTED"
            } else {
                "ACCEPTED"
            }
        ));
        out.push_str(&format!(
            "    rejection reason: {}\n",
            case.rejection_reason
        ));
    }

    out.push_str("\nCOVERAGE\n");
    out.push_str(&format!(
        "    cases:                  {}\n",
        pack.summary.case_count
    ));
    out.push_str(&format!(
        "    every forgery applied:   {}\n",
        pack.summary.all_forged
    ));
    out.push_str(&format!(
        "    every forgery forbidden: {}\n",
        pack.summary.all_inject_forbidden
    ));
    out.push_str(&format!(
        "    every forgery rejected:  {}\n",
        pack.summary.all_rejected
    ));
    out.push_str(&format!(
        "    canonical trace hash:  {}\n",
        pack.summary.canonical_trace_hash
    ));

    out.push_str("\nSUMMARY\n");
    out.push_str("    Every forged authority claim is rejected by re-derive byte-compare.\n");
    out.push_str(
        "    Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.\n\n",
    );

    out.push_str("BOUNDARY\n");
    for line in FAILURE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The full failure pack as (filename, content) pairs in write order: the rejection-record JSON and its
/// rendered report. Pure: both are derived from the same canonical [`FailurePack`]. This is what
/// `failure-pack` writes and what `failure-verify` re-derives and byte-compares against.
pub fn failure_pack_files() -> Result<Vec<(&'static str, String)>, TraceError> {
    let pack = canonical_failure_pack()?;
    let json = serde_json::to_string_pretty(&pack).expect("FailurePack serializes");
    let report = render_failure_pack(&pack);
    Ok(vec![
        (FAILURE_PACK_FILE, json),
        (FAILURE_REPORT_FILE, report),
    ])
}

/// Verify a provided failure pack (its files as (name, content) pairs read from disk) WITHOUT trusting it:
/// re-derive the canonical pack (re-running every forgery) and require every file to be present and
/// byte-identical. A missing file is [`TraceError::BundleMissingFile`]; any tampered/stale/foreign file is
/// [`TraceError::BundleMismatch`]. So a doctored failure pack (e.g. one that flips a `rejected` to false to
/// claim a forgery passed) can never be laundered into a clean verification. Pure (no I/O).
pub fn verify_failure_pack(provided: &[(String, String)]) -> Result<(), TraceError> {
    compare_bundle(&failure_pack_files()?, provided)
}

/// The `failure-cases` command: list the finite negative-scenario set (slug + one-line attack). Pure.
pub fn list_failure_cases() -> String {
    let mut out = String::from(
        "cognitive-demo — deterministic failure cases (each forges forbidden authority and is rejected):\n",
    );
    for c in FailureCase::ALL {
        out.push_str(&format!("    {:<18} {}\n", c.slug(), c.describe()));
    }
    out
}

// --- DOCFLOW-0: operator-supplied document trace. The SAME end-to-end pipeline as the canonical demo,
//     but the reading corpus is ONE operator-supplied LOCAL text document instead of the fixed canonical
//     corpus. The operator's bytes are READ (the shell passes the content in as `&str`) but never
//     TRUSTED: the flow asks the FROZEN reading codec for the document's OWN first span, builds a plan
//     that grounds and synthesizes EXACTLY that span, runs the FROZEN read0 verifier, and only proceeds
//     from a PASSED receipt — so a document that cannot ground a verified read fails closed. Every
//     downstream stage and boundary is identical to `CognitiveTrace::build` (hypothesis cites the receipt
//     hash; probe queued, never executed; observation quarantined; promotion refused; P12 unmoved). The
//     bundle re-derives from the SAME document, so a tampered document OR a tampered bundle file is
//     refused. No filesystem access here — the shell reads the file and validates its path. ---

/// The seven-line DOCFLOW-0 boundary, printed as the doc-trace / doc-bundle summary and pinned as data
/// so a test/gate can assert every line is present.
pub const DOC_BOUNDARY_LINES: [&str; 7] = [
    "The document flow reads local input.",
    "It does not trust local input.",
    "It verifies before tracing.",
    "It does not create authority.",
    "It does not execute.",
    "It does not promote.",
    "It does not train.",
];

/// The fixed title the document is read under. A CONSTANT (never the operator's filename), so the doc
/// trace is a pure function of the document CONTENT alone: two runs over the same content are
/// byte-identical, and the bundle re-derives.
pub const DOC_TITLE: &str = "operator-document";

/// The fixed question the document flow asks. Constant, so the trace stays a pure function of content.
pub const DOC_QUESTION: &str = "What does the document state in its first span?";

/// Validate that an operator-supplied input path is a SAFE LOCAL path WITHOUT touching the filesystem:
/// it must be non-empty, must not start with `~`, must be relative (not absolute), and must contain no
/// parent-dir (`..`), root, or prefix component. This is a PURE, unit-testable check — it uses
/// `std::path` for parsing only, never the filesystem. The shell performs the actual read and an additional
/// canonicalize-and-contain check as defense in depth. A failing path is refused, never read.
pub fn check_local_input_path(path: &str) -> Result<(), TraceError> {
    use std::path::{Component, Path};
    let reject = || TraceError::UnsafeInputPath(path.to_string());
    if path.trim().is_empty() || path.starts_with('~') {
        return Err(reject());
    }
    let parsed = Path::new(path);
    if parsed.is_absolute() {
        return Err(reject());
    }
    for component in parsed.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(reject())
            }
            Component::Normal(_) | Component::CurDir => {}
        }
    }
    Ok(())
}

/// The reading inputs the document flow feeds to [`CognitiveTrace::build`]: the one-document corpus,
/// the fixed question, and the generated grounding plan.
type DocReadingInputs = (Vec<(String, String)>, String, String);

/// Build the `(documents, question, plan)` reading inputs for an operator-supplied document. The plan is
/// constructed from the document's OWN first span — read through the frozen corpus builder, the same one
/// `produce_run` uses — so the claim and the synthesized answer ground EXACTLY against span 0: a verified
/// read of the operator's own text, never a trusted assertion about it. Returns
/// [`TraceError::EmptyDocument`] if the document yields no span. Pure (no I/O).
fn doc_inputs(doc_text: &str) -> Result<DocReadingInputs, TraceError> {
    let documents = vec![(DOC_TITLE.to_string(), doc_text.to_string())];
    // Ask the FROZEN corpus builder for the document's first span — the exact text the verifier will
    // ground against — so the generated plan can never drift from the codec's own sentence split.
    let corpus = corpus_from_documents(&documents);
    let first_id = corpus
        .metadata()
        .first()
        .and_then(|doc| doc.span_ids.first().copied())
        .ok_or(TraceError::EmptyDocument)?;
    let first_text = corpus
        .read_span(first_id)
        .map(|span| span.text().to_string())
        .ok_or(TraceError::EmptyDocument)?;
    let plan = doc_reading_plan(first_id, &first_text);
    Ok((documents, DOC_QUESTION.to_string(), plan))
}

/// Build the deterministic reading plan that reads span `id`, extracts a claim whose statement IS that
/// span's text, and synthesizes the answer from that single claim. Built via `serde_json` so the span
/// text — operator-supplied, possibly containing quotes/backslashes — is correctly JSON-escaped. The
/// resulting receipt grounds and answer-supports exactly, so the frozen verifier passes. Pure.
fn doc_reading_plan(id: SpanId, span_text: &str) -> String {
    serde_json::json!([
        {"action": "inspect_corpus"},
        {"action": "read_span", "span_id": id.0},
        {"action": "extract_claim", "statement": span_text, "source_span_ids": [id.0]},
        {"action": "synthesize", "answer_text": span_text, "supporting_claims": [0]}
    ])
    .to_string()
}

/// Build the end-to-end trace for an operator-supplied document. Identical pipeline to
/// [`CognitiveTrace::build`] — it starts from a FROZEN-VERIFIED reading receipt over the document and
/// fails closed ([`TraceError::VerifierRejected`]) if that read does not verify. Pure (no I/O); the
/// shell reads the file and passes its content as `doc_text`.
pub fn doc_trace(doc_text: &str) -> Result<CognitiveTrace, TraceError> {
    let (documents, question, plan) = doc_inputs(doc_text)?;
    CognitiveTrace::build(&documents, &question, &plan)
}

/// The `doc-trace` command body: build the document trace and serialize it. Pure.
pub fn run_doc_trace(doc_text: &str) -> Result<String, TraceError> {
    Ok(doc_trace(doc_text)?.to_json())
}

/// Re-derive the document trace from `doc_text` and confirm the PROVIDED trace JSON is byte-for-byte
/// that trace. Like [`verify_trace_json`], the provided trace is NEVER parsed back into authority
/// (`CognitiveTrace` is `Serialize` but not `Deserialize`) — it is only COMPARED against the freshly
/// re-derived trace, so a tampered/stale/foreign trace is REFUSED ([`TraceError::DocTraceMismatch`]).
/// The document is the source of truth, which is why doc-report requires `--input`. Pure (no I/O).
pub fn verify_doc_trace_json(doc_text: &str, provided: &str) -> Result<CognitiveTrace, TraceError> {
    let canonical = doc_trace(doc_text)?;
    if provided == canonical.to_json() {
        Ok(canonical)
    } else {
        Err(TraceError::DocTraceMismatch)
    }
}

/// The `doc-report` command body: render the operator report for a provided document trace — but only
/// after [`verify_doc_trace_json`] confirms it IS the trace re-derived from `doc_text`, so the report
/// always describes the real verified trace and never an untrusted file's claims. Pure (no I/O).
pub fn run_doc_report(doc_text: &str, provided_trace_json: &str) -> Result<String, TraceError> {
    Ok(verify_doc_trace_json(doc_text, provided_trace_json)?.to_report())
}

/// The full repro bundle for an operator document as (filename, content) pairs in write order. Pure:
/// every file is derived from the document's verified trace via the shared [`trace_bundle`] core, so the
/// doc bundle is a reproducible DEMONSTRATION over the operator's own document, never trusted authority.
pub fn doc_bundle(doc_text: &str) -> Result<Vec<(&'static str, String)>, TraceError> {
    Ok(trace_bundle(
        &doc_trace(doc_text)?,
        "trace.json re-derives byte-identically from the operator document",
    ))
}

/// Verify a provided document bundle WITHOUT trusting it: re-derive the bundle from the SAME `doc_text`
/// and require every file present and byte-identical. A missing file is [`TraceError::BundleMissingFile`];
/// any tampered/stale/foreign file (including the manifest) is [`TraceError::BundleMismatch`]; and a
/// TAMPERED DOCUMENT yields a different trace, so the whole bundle fails to match. Returns `Ok(())` only
/// on a full, exact re-derivation. Pure (no I/O).
pub fn verify_doc_bundle(doc_text: &str, provided: &[(String, String)]) -> Result<(), TraceError> {
    compare_bundle(&doc_bundle(doc_text)?, provided)
}

// --- DOCFLOW-2: the document-flow scenario pack / input-integrity matrix. Where DOCFLOW-0 proves one
//     clean local-document path and DOCFLOW-1 pins the operator path, DOCFLOW-2 proves the document flow
//     holds across a finite, enum-backed set of VALID and INVALID inputs: a clean local document verifies;
//     a modified document, a tampered bundle file, an empty document, an absolute path, a `..` traversal,
//     and a path that escapes the working directory are each REFUSED. Every scenario is OBSERVED by running
//     the REAL DOCFLOW-0 check/verify (proves, not asserts), and every scenario preserves the same boundary:
//     local text is read, never trusted; nothing executes, becomes evidence, promotes, or trains. The pack
//     and matrix are `Serialize`-only and re-derived-and-byte-compared, so a tampered pack is refused. ---

/// Decide whether a RESOLVED (already canonicalized) path stays inside `working_dir`. This is the pure
/// containment decision the shell applies after `canonicalize()` to refuse a symlink that escapes the
/// working directory: a resolved path that is not within `working_dir` is rejected. Pure — it inspects
/// already-resolved [`std::path::Path`] values component-wise and never touches the filesystem, so it is
/// unit-testable and is the SINGLE source of the containment decision the shell also calls.
pub fn resolved_path_within(working_dir: &std::path::Path, resolved: &std::path::Path) -> bool {
    resolved.starts_with(working_dir)
}

/// The eight-line DOCFLOW-2 boundary, embedded in the pack and matrix. Pinned as data so a test can assert it.
pub const DOC_SCENARIO_BOUNDARY_LINES: [&str; 8] = [
    "Document scenarios vary the input.",
    "They do not vary the authority.",
    "Local text is read, not trusted.",
    "Verification comes before tracing.",
    "Nothing executes.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The document-scenario pack file names (the structured outcome record + its rendered report).
pub const DOC_SCENARIO_PACK_FILE: &str = "doc-scenario-pack.json";
pub const DOC_SCENARIO_REPORT_FILE: &str = "doc-scenario-report.txt";
pub const DOC_SCENARIO_PACK_FILES: [&str; 2] = [DOC_SCENARIO_PACK_FILE, DOC_SCENARIO_REPORT_FILE];

/// The clean operator document every scenario derives from (a CONSTANT, so the pack is reproducible). Its
/// first span is the verified reading answer.
const DOC_SCENARIO_SAMPLE: &str = "The east bridge reopened today. Traffic resumed by noon.";
/// A genuinely different document used by the modified-document scenario (its first span differs, so the
/// re-derived trace differs and the clean bundle no longer matches).
const DOC_SCENARIO_MODIFIED: &str = "The west bridge collapsed today. Traffic stopped by noon.";

/// A deterministic document-flow input scenario. The set is finite and enum-backed: one VALID input
/// (clean) and eight INVALID inputs (modified / empty / unsafe / escaping / tampered), each of which the
/// DOCFLOW-0 check or verifier must refuse. Each scenario proves an input-integrity property while keeping
/// the authority boundary closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocScenario {
    /// A clean local document: its bundle re-derives byte-identically and verifies.
    CleanLocalDocument,
    /// A modified document: the clean bundle no longer matches the trace re-derived from the new text.
    ModifiedDocument,
    /// An empty document: no readable span, so the flow fails closed before tracing.
    EmptyDocument,
    /// An absolute input path: refused by the pure path check before any read.
    AbsolutePath,
    /// A `..` parent-traversal input path: refused by the pure path check before any read.
    ParentTraversal,
    /// A path whose resolved target escapes the working directory (e.g. a symlink): refused by the
    /// containment decision after canonicalize.
    SymlinkEscape,
    /// A tampered `trace.json` in an otherwise-clean bundle: refused by re-derivation.
    TamperedTrace,
    /// A tampered `report.txt` in an otherwise-clean bundle: refused by re-derivation.
    TamperedReport,
    /// A tampered `manifest.json` in an otherwise-clean bundle: refused by re-derivation.
    TamperedManifest,
}

impl DocScenario {
    /// Every scenario, in canonical order. Pinned as data so a test/the pack can assert the full set.
    pub const ALL: [DocScenario; 9] = [
        DocScenario::CleanLocalDocument,
        DocScenario::ModifiedDocument,
        DocScenario::EmptyDocument,
        DocScenario::AbsolutePath,
        DocScenario::ParentTraversal,
        DocScenario::SymlinkEscape,
        DocScenario::TamperedTrace,
        DocScenario::TamperedReport,
        DocScenario::TamperedManifest,
    ];

    /// The stable slug for this scenario. Exhaustive match — a new variant forces a slug here.
    pub fn slug(self) -> &'static str {
        match self {
            DocScenario::CleanLocalDocument => "clean-local-document",
            DocScenario::ModifiedDocument => "modified-document",
            DocScenario::EmptyDocument => "empty-document",
            DocScenario::AbsolutePath => "absolute-path",
            DocScenario::ParentTraversal => "parent-traversal",
            DocScenario::SymlinkEscape => "symlink-escape",
            DocScenario::TamperedTrace => "tampered-trace",
            DocScenario::TamperedReport => "tampered-report",
            DocScenario::TamperedManifest => "tampered-manifest",
        }
    }

    /// A one-line description of the scenario (shown by `doc-scenarios`). Exhaustive match.
    pub fn describe(self) -> &'static str {
        match self {
            DocScenario::CleanLocalDocument => {
                "a clean local document verifies (its bundle re-derives byte-identically)"
            }
            DocScenario::ModifiedDocument => {
                "a modified document invalidates the clean bundle (re-derivation no longer matches)"
            }
            DocScenario::EmptyDocument => {
                "an empty document fails closed (no readable span, no verified receipt)"
            }
            DocScenario::AbsolutePath => "an absolute input path is refused before any read",
            DocScenario::ParentTraversal => {
                "a `..` traversal input path is refused before any read"
            }
            DocScenario::SymlinkEscape => {
                "a path that escapes the working directory is refused after canonicalize"
            }
            DocScenario::TamperedTrace => "a tampered trace.json is refused by re-derivation",
            DocScenario::TamperedReport => "a tampered report.txt is refused by re-derivation",
            DocScenario::TamperedManifest => "a tampered manifest.json is refused by re-derivation",
        }
    }

    /// Parse a slug into a scenario. Fails CLOSED: any string that is not EXACTLY a known slug is `None`.
    pub fn from_slug(slug: &str) -> Option<DocScenario> {
        DocScenario::ALL.into_iter().find(|s| s.slug() == slug)
    }

    /// The class of input this scenario varies. Exhaustive match.
    fn input_kind(self) -> &'static str {
        match self {
            DocScenario::CleanLocalDocument => "clean",
            DocScenario::ModifiedDocument => "modified",
            DocScenario::EmptyDocument => "empty",
            DocScenario::AbsolutePath | DocScenario::ParentTraversal => "unsafe-path",
            DocScenario::SymlinkEscape => "escaping-path",
            DocScenario::TamperedTrace
            | DocScenario::TamperedReport
            | DocScenario::TamperedManifest => "tampered-artifact",
        }
    }

    /// The expected outcome: only the clean document verifies; every other input is refused. Exhaustive.
    fn expectation(self) -> &'static str {
        match self {
            DocScenario::CleanLocalDocument => "verifies",
            _ => "refused",
        }
    }
}

/// One observed row of the document-scenario pack: the scenario's identity, the input class, the expected
/// and OBSERVED outcome, whether the input genuinely differed from the clean input (anti-vacuity), the
/// typed rejection reason (observed, empty for the clean case), and the four boundary cells (always all
/// true). `Serialize` but NOT `Deserialize` — re-derived and byte-compared, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DocScenarioEntry {
    slug: String,
    description: String,
    input_kind: String,
    expectation: String,
    produced_trace: bool,
    verified: bool,
    refused: bool,
    input_changed: bool,
    rejection_reason: String,
    no_execution: bool,
    no_evidence: bool,
    no_promotion: bool,
    no_training: bool,
}

/// The coverage summary over the scenario set: counts of verified/refused, the boundary cells proven, and
/// the distinct input kinds and rejection reasons (proving the variation is real). Shared by the pack and
/// the matrix. `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DocScenarioCoverage {
    scenario_count: usize,
    verified_count: usize,
    refused_count: usize,
    boundary_count: usize,
    cells_total: usize,
    cells_proven: usize,
    all_expectations_met: bool,
    all_boundaries_hold: bool,
    distinct_input_kinds: Vec<String>,
    distinct_rejection_reasons: Vec<String>,
}

/// The document-scenario pack manifest: every observed scenario row, the coverage summary, and the
/// eight-line boundary. `Serialize` but NOT `Deserialize` — re-derived and byte-compared on verify.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DocScenarioPack {
    schema: String,
    scenarios: Vec<DocScenarioEntry>,
    coverage: DocScenarioCoverage,
    boundary: Vec<String>,
}

/// One row of the input-integrity matrix: a projection of an entry onto the input class, the observed
/// outcome, and the four boundary cells. `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DocMatrixRow {
    slug: String,
    input_kind: String,
    expectation: String,
    outcome: String,
    rejection_reason: String,
    no_execution: bool,
    no_evidence: bool,
    no_promotion: bool,
    no_training: bool,
}

/// The input-integrity matrix: one row per scenario, the coverage summary, and the boundary. `Serialize`
/// but NOT `Deserialize` — re-derived and byte-compared, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DocScenarioMatrix {
    schema: String,
    scenarios: Vec<DocMatrixRow>,
    coverage: DocScenarioCoverage,
    boundary: Vec<String>,
}

/// A stable, deterministic token for an OBSERVED rejection. Derived from the typed error variant (never the
/// variable path/byte contents), so the pack stays reproducible while still recording WHY each input was
/// refused.
fn doc_rejection_token(err: &TraceError) -> String {
    match err {
        TraceError::EmptyDocument => "empty-document".to_string(),
        TraceError::EmptyCorpus => "empty-corpus".to_string(),
        TraceError::CorpusTraceMismatch => "corpus-trace-mismatch".to_string(),
        TraceError::UnsafeInputPath(_) => "unsafe-input-path".to_string(),
        TraceError::BundleMismatch(name) => format!("bundle-file-mismatch:{name}"),
        TraceError::BundleMissingFile(name) => format!("bundle-missing-file:{name}"),
        TraceError::DocTraceMismatch => "doc-trace-mismatch".to_string(),
        TraceError::VerifierRejected => "verifier-rejected".to_string(),
        TraceError::CitationMismatch => "citation-mismatch".to_string(),
        TraceError::TraceMismatch => "trace-mismatch".to_string(),
        TraceError::MatrixMismatch => "matrix-mismatch".to_string(),
        TraceError::UnknownQuestion(_) => "unknown-question".to_string(),
        TraceError::Reading(_) => "reading-error".to_string(),
        TraceError::Hypothesis(_) => "hypothesis-error".to_string(),
        TraceError::Review(_) => "review-error".to_string(),
        TraceError::MissingReceiptHash => "missing-receipt-hash".to_string(),
        TraceError::EmptyFrame => "empty-frame".to_string(),
        TraceError::UnsupportedPreservedFact => "unsupported-preserved-fact".to_string(),
        TraceError::NoveltyPacketMismatch => "novelty-packet-mismatch".to_string(),
        TraceError::DreamExport(_) => "dream-export-refused".to_string(),
        TraceError::DreamExportMismatch => "dream-export-mismatch".to_string(),
    }
}

/// Clone a `(name, content)` bundle into owned pairs (as a provided pack would arrive from disk).
fn owned_pairs(files: &[(&'static str, String)]) -> Vec<(String, String)> {
    files
        .iter()
        .map(|(name, content)| (name.to_string(), content.clone()))
        .collect()
}

/// Forge one named file in an otherwise-clean document bundle by appending a tamper marker. Returns the
/// forged provided bundle and whether the named file was found and genuinely changed (anti-vacuity: a
/// no-op cannot masquerade as a caught tamper). The forged bytes are never persisted as trusted state —
/// they exist only to be REFUSED by re-derivation.
fn forge_doc_bundle_file(file: &str) -> Result<(Vec<(String, String)>, bool), TraceError> {
    let mut provided = owned_pairs(&doc_bundle(DOC_SCENARIO_SAMPLE)?);
    let mut changed = false;
    for (name, content) in provided.iter_mut() {
        if name == file {
            let before = content.clone();
            content.push_str("\n{tampered}");
            changed = *content != before;
        }
    }
    Ok((provided, changed))
}

/// Run ONE document scenario by exercising the REAL DOCFLOW-0 check/verifier over the input variation and
/// recording the OBSERVED outcome (verified vs refused + the typed reason), never an asserted one. A
/// refused scenario produces no trace, so its four boundary cells hold trivially (nothing was minted); the
/// clean scenario reads its cells from the real verified trace.
fn run_doc_scenario(scenario: DocScenario) -> Result<DocScenarioEntry, TraceError> {
    // (produced_trace, verified, refused, input_changed, rejection_reason, [no_exec, no_evid, no_promo, no_train])
    let (produced_trace, verified, refused, input_changed, rejection_reason, cells) = match scenario
    {
        DocScenario::CleanLocalDocument => {
            let provided = owned_pairs(&doc_bundle(DOC_SCENARIO_SAMPLE)?);
            let verified = verify_doc_bundle(DOC_SCENARIO_SAMPLE, &provided).is_ok();
            let trace = doc_trace(DOC_SCENARIO_SAMPLE)?;
            let cells = [
                trace.nothing_executed(),
                trace.nothing_becomes_evidence(),
                trace.promotion_refused(),
                !trace.training_justified(),
            ];
            (true, verified, !verified, false, String::new(), cells)
        }
        DocScenario::ModifiedDocument => {
            let provided = owned_pairs(&doc_bundle(DOC_SCENARIO_SAMPLE)?);
            let changed = DOC_SCENARIO_MODIFIED != DOC_SCENARIO_SAMPLE;
            let err = verify_doc_bundle(DOC_SCENARIO_MODIFIED, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        DocScenario::EmptyDocument => {
            let err = doc_trace("").err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                true,
                reason,
                [true, true, true, true],
            )
        }
        DocScenario::AbsolutePath => {
            let err = check_local_input_path("/etc/passwd").err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                true,
                reason,
                [true, true, true, true],
            )
        }
        DocScenario::ParentTraversal => {
            let err = check_local_input_path("../escape.txt").err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                true,
                reason,
                [true, true, true, true],
            )
        }
        DocScenario::SymlinkEscape => {
            // The containment decision the shell applies to a canonicalized path: a resolved target
            // outside the working directory (e.g. a symlink pointing at /etc) is refused. The library is
            // filesystem-free, so it observes the SAME pure decision the shell calls
            // (`resolved_path_within`, exercised on both an escaping and a contained path by the unit
            // test); the end-to-end refusal of a REAL filesystem symlink is proven by the shell and by the
            // release gate's end-to-end `doc-trace --input <symlink>` smoke.
            let working_dir = std::path::Path::new("/work/project");
            let escaped = std::path::Path::new("/etc/hostname");
            let within = resolved_path_within(working_dir, escaped);
            (
                false,
                false,
                !within,
                true,
                "escapes-working-directory".to_string(),
                [true, true, true, true],
            )
        }
        DocScenario::TamperedTrace => {
            let (provided, changed) = forge_doc_bundle_file("trace.json")?;
            let err = verify_doc_bundle(DOC_SCENARIO_SAMPLE, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        DocScenario::TamperedReport => {
            let (provided, changed) = forge_doc_bundle_file("report.txt")?;
            let err = verify_doc_bundle(DOC_SCENARIO_SAMPLE, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        DocScenario::TamperedManifest => {
            let (provided, changed) = forge_doc_bundle_file("manifest.json")?;
            let err = verify_doc_bundle(DOC_SCENARIO_SAMPLE, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
    };
    Ok(DocScenarioEntry {
        slug: scenario.slug().to_string(),
        description: scenario.describe().to_string(),
        input_kind: scenario.input_kind().to_string(),
        expectation: scenario.expectation().to_string(),
        produced_trace,
        verified,
        refused,
        input_changed,
        rejection_reason,
        no_execution: cells[0],
        no_evidence: cells[1],
        no_promotion: cells[2],
        no_training: cells[3],
    })
}

/// Run every document scenario, in canonical order, recording each observed outcome. Pure (no I/O).
fn canonical_doc_scenario_entries() -> Result<Vec<DocScenarioEntry>, TraceError> {
    DocScenario::ALL.into_iter().map(run_doc_scenario).collect()
}

/// Compute the coverage summary over the observed entries: counts, boundary cells proven, whether every
/// observed outcome met its expectation, and the distinct input kinds / rejection reasons.
fn doc_scenario_coverage(entries: &[DocScenarioEntry]) -> DocScenarioCoverage {
    let scenario_count = entries.len();
    let verified_count = entries.iter().filter(|e| e.verified).count();
    let refused_count = entries.iter().filter(|e| e.refused).count();
    let boundary_count = 4;
    let cells_total = scenario_count * boundary_count;
    let cells_proven: usize = entries
        .iter()
        .map(|e| {
            [e.no_execution, e.no_evidence, e.no_promotion, e.no_training]
                .iter()
                .filter(|cell| **cell)
                .count()
        })
        .sum();
    let all_expectations_met = entries.iter().all(|e| {
        if e.expectation == "verifies" {
            e.verified && !e.refused
        } else {
            e.refused && !e.verified
        }
    });
    DocScenarioCoverage {
        scenario_count,
        verified_count,
        refused_count,
        boundary_count,
        cells_total,
        cells_proven,
        all_expectations_met,
        all_boundaries_hold: cells_proven == cells_total,
        distinct_input_kinds: sorted_unique(entries.iter().map(|e| e.input_kind.clone()).collect()),
        distinct_rejection_reasons: sorted_unique(
            entries
                .iter()
                .filter(|e| !e.rejection_reason.is_empty())
                .map(|e| e.rejection_reason.clone())
                .collect(),
        ),
    }
}

fn doc_scenario_boundary() -> Vec<String> {
    DOC_SCENARIO_BOUNDARY_LINES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The document-scenario pack manifest JSON: every observed scenario row + coverage + boundary. Pure.
pub fn doc_scenario_pack_manifest() -> Result<String, TraceError> {
    let scenarios = canonical_doc_scenario_entries()?;
    let coverage = doc_scenario_coverage(&scenarios);
    let pack = DocScenarioPack {
        schema: "cognitive-doc-scenario-pack-v0.1".to_string(),
        scenarios,
        coverage,
        boundary: doc_scenario_boundary(),
    };
    Ok(serde_json::to_string_pretty(&pack).expect("DocScenarioPack serializes"))
}

/// Render the plain-text document-scenario report: each scenario's input class, expected/observed outcome,
/// rejection reason, and boundary cells, plus the coverage summary and boundary. Pure FORMATTING.
pub fn doc_scenario_report() -> Result<String, TraceError> {
    let entries = canonical_doc_scenario_entries()?;
    let coverage = doc_scenario_coverage(&entries);
    let mut out = String::new();
    out.push_str("COGNITIVE OS — DOCUMENT FLOW INPUT-INTEGRITY SCENARIOS\n");
    out.push_str("schema: cognitive-doc-scenario-pack-v0.1\n");
    out.push_str("(each scenario varies the INPUT and observes the real check; it records, it does not act)\n\n");
    for e in &entries {
        out.push_str(&format!("[{}]  ({})\n", e.slug, e.input_kind));
        out.push_str(&format!("    {}\n", e.description));
        out.push_str(&format!(
            "    expected: {}    observed: {}\n",
            e.expectation,
            if e.verified { "verified" } else { "refused" }
        ));
        if !e.rejection_reason.is_empty() {
            out.push_str(&format!("    rejection: {}\n", e.rejection_reason));
        }
        out.push_str(&format!(
            "    boundary: no_execution={} no_evidence={} no_promotion={} no_training={}\n",
            e.no_execution, e.no_evidence, e.no_promotion, e.no_training
        ));
    }
    out.push_str("\nCOVERAGE\n");
    out.push_str(&format!(
        "    scenarios:           {}\n",
        coverage.scenario_count
    ));
    out.push_str(&format!(
        "    verified:            {}\n",
        coverage.verified_count
    ));
    out.push_str(&format!(
        "    refused:             {}\n",
        coverage.refused_count
    ));
    out.push_str(&format!(
        "    cells proven:        {}/{}\n",
        coverage.cells_proven, coverage.cells_total
    ));
    out.push_str(&format!(
        "    all_expectations_met: {}\n",
        coverage.all_expectations_met
    ));
    out.push_str(&format!(
        "    all_boundaries_hold: {}\n",
        coverage.all_boundaries_hold
    ));
    out.push_str(&format!(
        "    distinct input kinds: {}\n",
        coverage.distinct_input_kinds.join(", ")
    ));
    out.push_str(&format!(
        "    distinct rejections:  {}\n",
        coverage.distinct_rejection_reasons.join(", ")
    ));
    out.push_str("\nBOUNDARY\n");
    for line in DOC_SCENARIO_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    Ok(out)
}

/// The document-scenario pack as `(filename, content)` pairs in write order: the structured manifest and
/// its rendered report. Pure: both are derived from the observed scenario set.
pub fn doc_scenario_pack_files() -> Result<Vec<(&'static str, String)>, TraceError> {
    Ok(vec![
        (DOC_SCENARIO_PACK_FILE, doc_scenario_pack_manifest()?),
        (DOC_SCENARIO_REPORT_FILE, doc_scenario_report()?),
    ])
}

/// Verify a provided document-scenario pack WITHOUT trusting it: re-derive both files (re-running every
/// scenario) and byte-compare. A missing file is [`TraceError::BundleMissingFile`]; any tampered/stale/
/// foreign file is [`TraceError::BundleMismatch`]. Pure (no I/O).
pub fn verify_doc_scenario_pack(provided: &[(String, String)]) -> Result<(), TraceError> {
    compare_bundle(&doc_scenario_pack_files()?, provided)
}

/// The `doc-scenarios` command: list the finite document-scenario set (slug + one-line description). Pure.
pub fn list_doc_scenarios() -> String {
    let mut out = String::from(
        "cognitive-demo — document-flow input scenarios (each proves an input-integrity property):\n",
    );
    for s in DocScenario::ALL {
        out.push_str(&format!("    {:<22} {}\n", s.slug(), s.describe()));
    }
    out
}

/// The input-integrity matrix JSON: one row per scenario (input class × observed outcome × boundary
/// cells) plus the coverage summary, re-derived from the scenario set. Pure: it never trusts the pack
/// files; the matrix command verifies the pack separately before emitting this.
pub fn doc_scenario_matrix() -> Result<String, TraceError> {
    let entries = canonical_doc_scenario_entries()?;
    let coverage = doc_scenario_coverage(&entries);
    let rows = entries
        .iter()
        .map(|e| DocMatrixRow {
            slug: e.slug.clone(),
            input_kind: e.input_kind.clone(),
            expectation: e.expectation.clone(),
            outcome: if e.verified { "verified" } else { "refused" }.to_string(),
            rejection_reason: e.rejection_reason.clone(),
            no_execution: e.no_execution,
            no_evidence: e.no_evidence,
            no_promotion: e.no_promotion,
            no_training: e.no_training,
        })
        .collect();
    let matrix = DocScenarioMatrix {
        schema: "cognitive-doc-scenario-matrix-v0.1".to_string(),
        scenarios: rows,
        coverage,
        boundary: doc_scenario_boundary(),
    };
    Ok(serde_json::to_string_pretty(&matrix).expect("DocScenarioMatrix serializes"))
}

// --- CORPUS-0: multi-document local corpus trace / source-selection boundary. Where DOCFLOW-0 traces ONE
//     operator document, CORPUS-0 traces a small LOCAL CORPUS DIRECTORY of `.txt` documents through the SAME
//     end-to-end pipeline. The shell enumerates the directory (path-validated, only non-hidden `.txt` files,
//     each canonicalize-contained so no symlink escapes, sorted by name for determinism) and passes the
//     `(title, content)` documents to the pure library; the library asks the FROZEN corpus builder for the
//     corpus's OWN first span, builds a grounding plan over it, and starts the trace from a VERIFIED read0
//     receipt (fails closed with `EmptyCorpus` if the corpus grounds nothing). The trace's structure hash
//     binds EVERY document (title + spans + sections), so a tamper of ANY document — even a non-grounding
//     one — re-derives a different trace and is refused. Source selection is made UNAMBIGUOUS by an explicit
//     `CorpusSource` attribution (which document index/title/span/text grounded the answer), re-derived and
//     byte-compared in the bundle. The corpus is READ, never TRUSTED: nothing executes, becomes evidence,
//     promotes, or trains; P12 stays training_justified=false. No filesystem access here — the shell reads
//     the directory and validates every path. ---

/// The eight-line CORPUS-0 boundary, printed as the corpus-bundle / corpus-report summary and pinned as data
/// so a test/gate can assert every line is present.
pub const CORPUS_BOUNDARY_LINES: [&str; 8] = [
    "The corpus flow reads local documents.",
    "It does not trust local documents.",
    "Source selection is verified and replayable.",
    "Verification comes before tracing.",
    "Nothing executes.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The fixed question the corpus flow asks. Constant (never derived from the corpus), so the trace stays a
/// pure function of the corpus CONTENT and document NAMES alone.
pub const CORPUS_QUESTION: &str = "What does the corpus state in its first span?";

/// The source-attribution file name in the corpus bundle (re-derived and byte-compared on verify).
pub const CORPUS_SOURCE_FILE: &str = "corpus-source.json";

/// The corpus bundle file names, in write order: the unambiguous source attribution, then the same
/// trace/report/questions/manifest as the canonical bundle.
pub const CORPUS_BUNDLE_FILES: [&str; 5] = [
    CORPUS_SOURCE_FILE,
    BUNDLE_TRACE_FILE,
    BUNDLE_REPORT_FILE,
    BUNDLE_QUESTIONS_FILE,
    BUNDLE_MANIFEST_FILE,
];

/// Decide whether a directory entry's FILE NAME is admitted into the corpus: it must be a plain, non-hidden
/// `.txt` file. A leading `.` (hidden file, including a bare `.txt`) is refused, and only a non-empty stem
/// followed by the exact `.txt` suffix is accepted. PURE and unit-testable — the shell applies it to filter
/// entries before any read, so a hidden or non-`.txt` file never becomes a trusted document. (The shell
/// adds canonicalize-and-contain as defense in depth, so a symlink cannot escape the directory either.)
pub fn corpus_admits_filename(name: &str) -> bool {
    if name.starts_with('.') {
        return false;
    }
    match name.strip_suffix(".txt") {
        Some(stem) => !stem.is_empty(),
        None => false,
    }
}

/// Which document and span grounded the corpus answer: the document's index and title in the sorted corpus,
/// the grounding span's id, and that span's verbatim text. Derived from the FROZEN corpus metadata, so source
/// identity is unambiguous and replayable. `Serialize` but NOT `Deserialize` — re-derived and byte-compared,
/// never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CorpusSource {
    schema: String,
    document_index: usize,
    document_title: String,
    span_id: u64,
    span_text: String,
}

/// Build the `(documents, question, plan)` reading inputs for an operator-supplied corpus. The plan grounds
/// on the corpus's OWN first span — read through the frozen corpus builder (the same one `produce_run` uses)
/// — so the claim and synthesized answer ground EXACTLY against the first span of the first document that has
/// one. Returns [`TraceError::EmptyCorpus`] if the corpus yields no span. Pure (no I/O).
fn corpus_inputs(documents: &[(String, String)]) -> Result<DocReadingInputs, TraceError> {
    let corpus = corpus_from_documents(documents);
    let first_id = corpus
        .metadata()
        .iter()
        .flat_map(|doc| doc.span_ids.iter().copied())
        .next()
        .ok_or(TraceError::EmptyCorpus)?;
    let first_text = corpus
        .read_span(first_id)
        .map(|span| span.text().to_string())
        .ok_or(TraceError::EmptyCorpus)?;
    let plan = doc_reading_plan(first_id, &first_text);
    Ok((documents.to_vec(), CORPUS_QUESTION.to_string(), plan))
}

/// Re-derive the unambiguous source attribution for the corpus: find the FIRST document (in sorted order)
/// that owns a span, and record its index/title and that span's id/text. This is the SAME span the trace
/// grounds on (the globally-first span id), so the attribution and the trace cannot disagree. Returns
/// [`TraceError::EmptyCorpus`] if no document grounds a span. Pure (no I/O).
fn corpus_source(documents: &[(String, String)]) -> Result<CorpusSource, TraceError> {
    let corpus = corpus_from_documents(documents);
    for (index, doc) in corpus.metadata().iter().enumerate() {
        if let Some(&span_id) = doc.span_ids.first() {
            let span_text = corpus
                .read_span(span_id)
                .map(|span| span.text().to_string())
                .ok_or(TraceError::EmptyCorpus)?;
            return Ok(CorpusSource {
                schema: "cognitive-corpus-source-v0.1".to_string(),
                document_index: index,
                document_title: doc.title.clone(),
                span_id: span_id.0,
                span_text,
            });
        }
    }
    Err(TraceError::EmptyCorpus)
}

/// The source attribution as pretty JSON (the `corpus-source.json` bundle file). Pure.
fn corpus_source_json(documents: &[(String, String)]) -> Result<String, TraceError> {
    Ok(serde_json::to_string_pretty(&corpus_source(documents)?).expect("CorpusSource serializes"))
}

/// Build the end-to-end trace for an operator-supplied corpus. Identical pipeline to
/// [`CognitiveTrace::build`] — it starts from a FROZEN-VERIFIED reading receipt over the corpus and fails
/// closed ([`TraceError::VerifierRejected`]/[`TraceError::EmptyCorpus`]) if that read does not verify. The
/// receipt's structure hash binds every document, so the trace re-derives differently if ANY document
/// changes. Pure (no I/O); the shell reads the directory and passes its documents as `documents`.
pub fn corpus_trace(documents: &[(String, String)]) -> Result<CognitiveTrace, TraceError> {
    let (documents, question, plan) = corpus_inputs(documents)?;
    CognitiveTrace::build(&documents, &question, &plan)
}

/// The `corpus-trace` command body: build the corpus trace and serialize it. Pure.
pub fn run_corpus_trace(documents: &[(String, String)]) -> Result<String, TraceError> {
    Ok(corpus_trace(documents)?.to_json())
}

/// Re-derive the corpus trace from `documents` and confirm the PROVIDED trace JSON is byte-for-byte that
/// trace. Like [`verify_doc_trace_json`], the provided trace is NEVER parsed back into authority
/// (`CognitiveTrace` is `Serialize` but not `Deserialize`) — it is only COMPARED against the freshly
/// re-derived trace, so a tampered/stale/foreign trace is REFUSED ([`TraceError::CorpusTraceMismatch`]). The
/// corpus is the source of truth, which is why corpus-report requires `--input-dir`. Pure (no I/O).
pub fn verify_corpus_trace_json(
    documents: &[(String, String)],
    provided: &str,
) -> Result<CognitiveTrace, TraceError> {
    let canonical = corpus_trace(documents)?;
    if provided == canonical.to_json() {
        Ok(canonical)
    } else {
        Err(TraceError::CorpusTraceMismatch)
    }
}

/// Render the corpus operator report: the trace report, then a SOURCE SELECTION section that names the
/// grounded document (index + title), its span id and text, and lists every corpus document (title + span
/// count) so source identity is unambiguous, then the eight-line CORPUS-0 boundary. Pure FORMATTING derived
/// from the corpus and its verified trace.
fn corpus_report_body(
    documents: &[(String, String)],
    trace: &CognitiveTrace,
) -> Result<String, TraceError> {
    let source = corpus_source(documents)?;
    let corpus = corpus_from_documents(documents);
    let mut out = trace.to_report();
    out.push_str("\nSOURCE SELECTION\n");
    out.push_str(&format!(
        "    grounded document:  [{}] {}\n",
        source.document_index, source.document_title
    ));
    out.push_str(&format!("    grounded span:      {}\n", source.span_id));
    out.push_str(&format!("    grounded text:      {}\n", source.span_text));
    out.push_str(&format!(
        "    corpus documents:   {}\n",
        corpus.metadata().len()
    ));
    for (index, doc) in corpus.metadata().iter().enumerate() {
        out.push_str(&format!(
            "      [{index}] {} ({} spans)\n",
            doc.title,
            doc.span_ids.len()
        ));
    }
    out.push_str("\nBOUNDARY\n");
    for line in CORPUS_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    Ok(out)
}

/// The `corpus-report` command body: render the corpus report for a provided trace — but only after
/// [`verify_corpus_trace_json`] confirms it IS the trace re-derived from `documents`, so the report always
/// describes the real verified trace and never an untrusted file's claims. Pure (no I/O).
pub fn run_corpus_report(
    documents: &[(String, String)],
    provided_trace_json: &str,
) -> Result<String, TraceError> {
    let trace = verify_corpus_trace_json(documents, provided_trace_json)?;
    corpus_report_body(documents, &trace)
}

/// The full repro bundle for an operator corpus as (filename, content) pairs in write order: the unambiguous
/// source attribution, then the verified trace, its report (with the source-selection section), the
/// questions transcript, and a manifest hashing all four content files. Pure: every file is derived from the
/// corpus's verified trace, so the corpus bundle is a reproducible DEMONSTRATION, never trusted authority.
pub fn corpus_bundle(
    documents: &[(String, String)],
) -> Result<Vec<(&'static str, String)>, TraceError> {
    let trace = corpus_trace(documents)?;
    let content: Vec<(&'static str, String)> = vec![
        (CORPUS_SOURCE_FILE, corpus_source_json(documents)?),
        (BUNDLE_TRACE_FILE, trace.to_json()),
        (BUNDLE_REPORT_FILE, corpus_report_body(documents, &trace)?),
        (BUNDLE_QUESTIONS_FILE, trace.questions_doc()),
    ];
    let manifest = bundle_manifest_with(
        &content,
        "trace.json + corpus-source.json re-derive byte-identically from the operator corpus",
    );
    let mut files = content;
    files.push((BUNDLE_MANIFEST_FILE, manifest));
    Ok(files)
}

/// Verify a provided corpus bundle WITHOUT trusting it: re-derive the bundle from the SAME `documents` and
/// require every file present and byte-identical. A missing file is [`TraceError::BundleMissingFile`]; any
/// tampered/stale/foreign file (including the source attribution or manifest) is [`TraceError::BundleMismatch`];
/// and a TAMPERED CORPUS (any document changed, even a non-grounding one) yields a different trace structure
/// hash, so the whole bundle fails to match. Returns `Ok(())` only on a full, exact re-derivation. Pure (no I/O).
pub fn verify_corpus_bundle(
    documents: &[(String, String)],
    provided: &[(String, String)],
) -> Result<(), TraceError> {
    compare_bundle(&corpus_bundle(documents)?, provided)
}

// --- CORPUS-2: corpus scenario pack / input-integrity matrix. Where CORPUS-0 traces ONE clean corpus and
//     CORPUS-1 documents the operator path, CORPUS-2 makes corpus behavior AUDITABLE across a finite,
//     enum-backed matrix of valid and invalid corpus inputs — the corpus analog of DOCFLOW-2. Each scenario
//     varies the CORPUS INPUT (clean two-document / empty / hidden-only / non-`.txt`-only / unsafe path /
//     escaping path / grounding-document mutation / non-grounding side-document mutation / tampered artifact)
//     and OBSERVES the REAL CORPUS-0 check or verifier, recording the outcome it actually produced — never an
//     asserted one. Exactly one input (clean) verifies; every other is REFUSED. The matrix additionally
//     records the verified case's SOURCE IDENTITY (which document/span grounded the answer) and a
//     `whole_corpus_bound` fact proven by the side-document scenario: mutating a non-grounding document leaves
//     the source attribution byte-identical yet still fails the bundle (the structure hash binds the WHOLE
//     corpus). Every scenario keeps the authority boundary closed: nothing executes, becomes evidence,
//     promotes, or trains; P12 stays training_justified=false. The library is filesystem-free — the path/escape
//     scenarios observe the SAME pure decisions the shell calls; the gate proves the end-to-end refusals. ---

/// The nine-line CORPUS-2 boundary, embedded in the pack and matrix. Pinned as data so a test can assert it.
pub const CORPUS_SCENARIO_BOUNDARY_LINES: [&str; 9] = [
    "Corpus scenarios vary the corpus input.",
    "They do not vary the authority.",
    "Source selection is verified and replayable.",
    "The whole corpus is hash-bound.",
    "Verification comes before tracing.",
    "Nothing executes.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The corpus-scenario pack file names (the structured outcome record + its rendered report).
pub const CORPUS_SCENARIO_PACK_FILE: &str = "corpus-scenario-pack.json";
pub const CORPUS_SCENARIO_REPORT_FILE: &str = "corpus-scenario-report.txt";
pub const CORPUS_SCENARIO_PACK_FILES: [&str; 2] =
    [CORPUS_SCENARIO_PACK_FILE, CORPUS_SCENARIO_REPORT_FILE];

/// The clean two-document corpus every scenario derives from (CONSTANTS, so the pack is reproducible), in the
/// SORTED order the shell loader produces — so `a-east.txt` owns the globally-first span and grounds the answer.
const CORPUS_SCENARIO_DOC_A: (&str, &str) = (
    "a-east.txt",
    "The east bridge reopened today. Traffic resumed by noon.",
);
const CORPUS_SCENARIO_DOC_B: (&str, &str) = (
    "b-west.txt",
    "The west tunnel remains closed. Crews continue repairs.",
);
/// A genuinely different GROUNDING document (changes the first span, so the source attribution AND the trace
/// re-derive differently): the grounding-mutation scenario.
const CORPUS_GROUNDING_MUTATION: &str = "The east bridge collapsed today. Traffic stopped by noon.";
/// A genuinely different NON-GROUNDING side document (leaves the grounding document — and thus the source
/// attribution — untouched, yet the structure hash binds it, so the trace still re-derives differently): the
/// side-document-mutation scenario, which proves whole-corpus binding.
const CORPUS_SIDE_MUTATION: &str = "The west tunnel reopened early. Crews departed.";

/// Candidate file names for the hidden-only scenario: every entry is a HIDDEN file, so the admission filter
/// admits NONE and the corpus is empty before any read.
const CORPUS_HIDDEN_ONLY_NAMES: [&str; 2] = [".secret.txt", ".hidden.txt"];
/// Candidate file names for the non-`.txt`-only scenario: every entry is a NON-`.txt` file, so the admission
/// filter admits NONE and the corpus is empty before any read.
const CORPUS_NON_TXT_ONLY_NAMES: [&str; 3] = ["notes.md", "data.json", "README"];

/// The clean two-document corpus, owned, in sorted loader order. The fixture every CORPUS-2 scenario derives
/// from. Pure (no I/O).
fn corpus_scenario_sample() -> Vec<(String, String)> {
    vec![
        (
            CORPUS_SCENARIO_DOC_A.0.to_string(),
            CORPUS_SCENARIO_DOC_A.1.to_string(),
        ),
        (
            CORPUS_SCENARIO_DOC_B.0.to_string(),
            CORPUS_SCENARIO_DOC_B.1.to_string(),
        ),
    ]
}

/// A deterministic corpus-flow input scenario. The set is finite and enum-backed: one VALID input (a clean
/// two-document corpus) and twelve INVALID inputs (empty / hidden-only / non-`.txt`-only / unsafe / escaping /
/// grounding-mutated / side-mutated / tampered), each of which the CORPUS-0 admission filter, check, or verifier
/// must refuse. Each scenario proves an input-integrity property while keeping the authority boundary closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorpusScenario {
    /// A clean two-document corpus: its bundle re-derives byte-identically and verifies.
    CleanTwoDocument,
    /// An empty corpus: no document grounds a span, so the flow fails closed before tracing.
    EmptyCorpus,
    /// A corpus of only hidden files: the admission filter admits none, so the corpus is empty.
    HiddenOnly,
    /// A corpus of only non-`.txt` files: the admission filter admits none, so the corpus is empty.
    NonTxtOnly,
    /// An absolute corpus path: refused by the pure path check before any read.
    AbsolutePath,
    /// A `..` parent-traversal corpus path: refused by the pure path check before any read.
    ParentTraversal,
    /// A corpus entry whose resolved target escapes the directory (e.g. a symlink): refused by the
    /// containment decision after canonicalize.
    SymlinkEscape,
    /// A mutated GROUNDING document: the source attribution and the trace re-derive differently, so the
    /// clean bundle no longer matches.
    GroundingMutation,
    /// A mutated NON-GROUNDING side document: the source attribution is byte-identical, yet the whole-corpus
    /// structure hash re-derives a different trace, so the clean bundle still fails to match.
    SideDocumentMutation,
    /// A tampered `corpus-source.json` in an otherwise-clean bundle: refused by re-derivation.
    TamperedSource,
    /// A tampered `trace.json` in an otherwise-clean bundle: refused by re-derivation.
    TamperedTrace,
    /// A tampered `report.txt` in an otherwise-clean bundle: refused by re-derivation.
    TamperedReport,
    /// A tampered `manifest.json` in an otherwise-clean bundle: refused by re-derivation.
    TamperedManifest,
}

impl CorpusScenario {
    /// Every scenario, in canonical order. Pinned as data so a test/the pack can assert the full set.
    pub const ALL: [CorpusScenario; 13] = [
        CorpusScenario::CleanTwoDocument,
        CorpusScenario::EmptyCorpus,
        CorpusScenario::HiddenOnly,
        CorpusScenario::NonTxtOnly,
        CorpusScenario::AbsolutePath,
        CorpusScenario::ParentTraversal,
        CorpusScenario::SymlinkEscape,
        CorpusScenario::GroundingMutation,
        CorpusScenario::SideDocumentMutation,
        CorpusScenario::TamperedSource,
        CorpusScenario::TamperedTrace,
        CorpusScenario::TamperedReport,
        CorpusScenario::TamperedManifest,
    ];

    /// The stable slug for this scenario. Exhaustive match — a new variant forces a slug here.
    pub fn slug(self) -> &'static str {
        match self {
            CorpusScenario::CleanTwoDocument => "clean-two-document",
            CorpusScenario::EmptyCorpus => "empty-corpus",
            CorpusScenario::HiddenOnly => "hidden-only",
            CorpusScenario::NonTxtOnly => "non-txt-only",
            CorpusScenario::AbsolutePath => "absolute-path",
            CorpusScenario::ParentTraversal => "parent-traversal",
            CorpusScenario::SymlinkEscape => "symlink-escape",
            CorpusScenario::GroundingMutation => "grounding-mutation",
            CorpusScenario::SideDocumentMutation => "side-document-mutation",
            CorpusScenario::TamperedSource => "tampered-source",
            CorpusScenario::TamperedTrace => "tampered-trace",
            CorpusScenario::TamperedReport => "tampered-report",
            CorpusScenario::TamperedManifest => "tampered-manifest",
        }
    }

    /// A one-line description of the scenario (shown by `corpus-scenarios`). Exhaustive match.
    pub fn describe(self) -> &'static str {
        match self {
            CorpusScenario::CleanTwoDocument => {
                "a clean two-document corpus verifies (its bundle re-derives byte-identically)"
            }
            CorpusScenario::EmptyCorpus => {
                "an empty corpus fails closed (no document grounds a span)"
            }
            CorpusScenario::HiddenOnly => {
                "a corpus of only hidden files is refused (no file is admitted)"
            }
            CorpusScenario::NonTxtOnly => {
                "a corpus of only non-.txt files is refused (no file is admitted)"
            }
            CorpusScenario::AbsolutePath => "an absolute corpus path is refused before any read",
            CorpusScenario::ParentTraversal => {
                "a `..` traversal corpus path is refused before any read"
            }
            CorpusScenario::SymlinkEscape => {
                "a corpus entry that escapes the directory is refused after canonicalize"
            }
            CorpusScenario::GroundingMutation => {
                "mutating the grounding document invalidates the bundle"
            }
            CorpusScenario::SideDocumentMutation => {
                "mutating a non-grounding side document invalidates the bundle (whole-corpus binding)"
            }
            CorpusScenario::TamperedSource => {
                "a tampered corpus-source.json is refused by re-derivation"
            }
            CorpusScenario::TamperedTrace => "a tampered trace.json is refused by re-derivation",
            CorpusScenario::TamperedReport => "a tampered report.txt is refused by re-derivation",
            CorpusScenario::TamperedManifest => {
                "a tampered manifest.json is refused by re-derivation"
            }
        }
    }

    /// Parse a slug into a scenario. Fails CLOSED: any string that is not EXACTLY a known slug is `None`.
    pub fn from_slug(slug: &str) -> Option<CorpusScenario> {
        CorpusScenario::ALL.into_iter().find(|s| s.slug() == slug)
    }

    /// The class of corpus input this scenario varies. Exhaustive match.
    fn input_kind(self) -> &'static str {
        match self {
            CorpusScenario::CleanTwoDocument => "clean",
            CorpusScenario::EmptyCorpus => "empty-corpus",
            CorpusScenario::HiddenOnly => "hidden-only",
            CorpusScenario::NonTxtOnly => "non-txt-only",
            CorpusScenario::AbsolutePath | CorpusScenario::ParentTraversal => "unsafe-path",
            CorpusScenario::SymlinkEscape => "escaping-path",
            CorpusScenario::GroundingMutation => "grounding-mutation",
            CorpusScenario::SideDocumentMutation => "side-document-mutation",
            CorpusScenario::TamperedSource
            | CorpusScenario::TamperedTrace
            | CorpusScenario::TamperedReport
            | CorpusScenario::TamperedManifest => "tampered-artifact",
        }
    }

    /// The expected outcome: only the clean corpus verifies; every other input is refused. Exhaustive.
    fn expectation(self) -> &'static str {
        match self {
            CorpusScenario::CleanTwoDocument => "verifies",
            _ => "refused",
        }
    }
}

/// One observed row of the corpus-scenario pack: the scenario's identity, the input class, the expected and
/// OBSERVED outcome, whether the input genuinely differed from the clean input (anti-vacuity), the typed
/// rejection reason (observed, empty for the clean case), and the four boundary cells (always all true).
/// `Serialize` but NOT `Deserialize` — re-derived and byte-compared, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CorpusScenarioEntry {
    slug: String,
    description: String,
    input_kind: String,
    expectation: String,
    produced_trace: bool,
    verified: bool,
    refused: bool,
    input_changed: bool,
    rejection_reason: String,
    no_execution: bool,
    no_evidence: bool,
    no_promotion: bool,
    no_training: bool,
}

/// The coverage summary over the corpus-scenario set: counts of verified/refused, the boundary cells proven,
/// whether the whole corpus is hash-bound (the side-document fact), and the distinct input kinds and rejection
/// reasons (proving the variation is real). Shared by the pack and the matrix. `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CorpusScenarioCoverage {
    scenario_count: usize,
    verified_count: usize,
    refused_count: usize,
    boundary_count: usize,
    cells_total: usize,
    cells_proven: usize,
    all_expectations_met: bool,
    all_boundaries_hold: bool,
    whole_corpus_bound: bool,
    distinct_input_kinds: Vec<String>,
    distinct_rejection_reasons: Vec<String>,
}

/// The corpus-scenario pack manifest: every observed scenario row, the coverage summary, the verified case's
/// SOURCE IDENTITY, and the nine-line boundary. `Serialize` but NOT `Deserialize` — re-derived and byte-compared.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CorpusScenarioPack {
    schema: String,
    scenarios: Vec<CorpusScenarioEntry>,
    coverage: CorpusScenarioCoverage,
    source: CorpusSource,
    boundary: Vec<String>,
}

/// One row of the corpus input-integrity matrix: a projection of an entry onto the input class, the observed
/// outcome, and the four boundary cells. `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CorpusMatrixRow {
    slug: String,
    input_kind: String,
    expectation: String,
    outcome: String,
    rejection_reason: String,
    no_execution: bool,
    no_evidence: bool,
    no_promotion: bool,
    no_training: bool,
}

/// The corpus input-integrity matrix: one row per scenario, the coverage summary, the verified case's SOURCE
/// IDENTITY (which document/span grounded the answer), and the boundary. `Serialize` but NOT `Deserialize` —
/// re-derived and byte-compared, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct CorpusScenarioMatrix {
    schema: String,
    scenarios: Vec<CorpusMatrixRow>,
    coverage: CorpusScenarioCoverage,
    source: CorpusSource,
    boundary: Vec<String>,
}

/// Forge one named file in an otherwise-clean corpus bundle by appending a tamper marker. Returns the forged
/// provided bundle and whether the named file was found and genuinely changed (anti-vacuity). The forged bytes
/// are never persisted as trusted state — they exist only to be REFUSED by re-derivation.
fn forge_corpus_bundle_file(file: &str) -> Result<(Vec<(String, String)>, bool), TraceError> {
    let mut provided = owned_pairs(&corpus_bundle(&corpus_scenario_sample())?);
    let mut changed = false;
    for (name, content) in provided.iter_mut() {
        if name == file {
            let before = content.clone();
            content.push_str("\n{tampered}");
            changed = *content != before;
        }
    }
    Ok((provided, changed))
}

/// Whether the WHOLE corpus is hash-bound, proven structurally: mutating the NON-grounding side document leaves
/// the source attribution byte-identical (the grounding document is untouched) YET the clean bundle is still
/// refused — because the receipt's structure hash binds every document, so the re-derived trace differs. Both
/// facts must hold. Pure (no I/O).
fn corpus_whole_binding_holds() -> Result<bool, TraceError> {
    let sample = corpus_scenario_sample();
    let clean = owned_pairs(&corpus_bundle(&sample)?);
    let mut side = sample.clone();
    side[1].1 = CORPUS_SIDE_MUTATION.to_string();
    let source_unchanged = corpus_source_json(&side)? == corpus_source_json(&sample)?;
    let refused = verify_corpus_bundle(&side, &clean).is_err();
    Ok(source_unchanged && refused)
}

/// Run ONE corpus scenario by exercising the REAL CORPUS-0 admission filter / check / verifier over the input
/// variation and recording the OBSERVED outcome (verified vs refused + the typed reason), never an asserted one.
/// A refused scenario produces no trace, so its four boundary cells hold trivially (nothing was minted); the
/// clean scenario reads its cells from the real verified trace. Pure (no I/O).
fn run_corpus_scenario(scenario: CorpusScenario) -> Result<CorpusScenarioEntry, TraceError> {
    let sample = corpus_scenario_sample();
    // (produced_trace, verified, refused, input_changed, rejection_reason, [no_exec, no_evid, no_promo, no_train])
    let (produced_trace, verified, refused, input_changed, rejection_reason, cells) = match scenario
    {
        CorpusScenario::CleanTwoDocument => {
            let provided = owned_pairs(&corpus_bundle(&sample)?);
            let verified = verify_corpus_bundle(&sample, &provided).is_ok();
            let trace = corpus_trace(&sample)?;
            let cells = [
                trace.nothing_executed(),
                trace.nothing_becomes_evidence(),
                trace.promotion_refused(),
                !trace.training_justified(),
            ];
            (true, verified, !verified, false, String::new(), cells)
        }
        CorpusScenario::EmptyCorpus => {
            let err = corpus_trace(&[]).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                true,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::HiddenOnly => {
            // The SAME pure admission filter the shell applies: a corpus of only hidden files admits none, so
            // no document is ever read and the corpus is empty.
            let admitted = CORPUS_HIDDEN_ONLY_NAMES
                .iter()
                .filter(|n| corpus_admits_filename(n))
                .count();
            let refused = admitted == 0 && !CORPUS_HIDDEN_ONLY_NAMES.is_empty();
            (
                false,
                false,
                refused,
                true,
                "no-admitted-files".to_string(),
                [true, true, true, true],
            )
        }
        CorpusScenario::NonTxtOnly => {
            let admitted = CORPUS_NON_TXT_ONLY_NAMES
                .iter()
                .filter(|n| corpus_admits_filename(n))
                .count();
            let refused = admitted == 0 && !CORPUS_NON_TXT_ONLY_NAMES.is_empty();
            (
                false,
                false,
                refused,
                true,
                "no-admitted-files".to_string(),
                [true, true, true, true],
            )
        }
        CorpusScenario::AbsolutePath => {
            let err = check_local_input_path("/etc/passwd").err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                true,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::ParentTraversal => {
            let err = check_local_input_path("../escape").err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                true,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::SymlinkEscape => {
            // The containment decision the shell applies to a canonicalized corpus entry: a resolved target
            // outside the corpus root (e.g. a symlink pointing at /etc) is refused. The library is
            // filesystem-free, so it observes the SAME pure decision the shell calls (`resolved_path_within`,
            // exercised on both an escaping and a contained path by the unit test); the end-to-end refusal of a
            // REAL filesystem symlink is proven by the shell and by the release gate's corpus-trace smoke.
            let corpus_root = std::path::Path::new("/work/corpus");
            let escaped = std::path::Path::new("/etc/hostname");
            let within = resolved_path_within(corpus_root, escaped);
            (
                false,
                false,
                !within,
                true,
                "escapes-working-directory".to_string(),
                [true, true, true, true],
            )
        }
        CorpusScenario::GroundingMutation => {
            // Mutating the FIRST (grounding) document changes its first span, so BOTH the source attribution
            // and the trace re-derive differently — the clean bundle fails first on corpus-source.json.
            let clean = owned_pairs(&corpus_bundle(&sample)?);
            let mut mutated = sample.clone();
            mutated[0].1 = CORPUS_GROUNDING_MUTATION.to_string();
            let changed = mutated[0].1 != sample[0].1;
            let err = verify_corpus_bundle(&mutated, &clean).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::SideDocumentMutation => {
            // Mutating the SECOND (non-grounding) document leaves corpus-source.json byte-identical, but the
            // structure hash binds the WHOLE corpus, so the trace re-derives differently — the clean bundle
            // fails on trace.json. This is the whole-corpus-binding proof.
            let clean = owned_pairs(&corpus_bundle(&sample)?);
            let mut mutated = sample.clone();
            mutated[1].1 = CORPUS_SIDE_MUTATION.to_string();
            let changed = mutated[1].1 != sample[1].1;
            let err = verify_corpus_bundle(&mutated, &clean).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::TamperedSource => {
            let (provided, changed) = forge_corpus_bundle_file(CORPUS_SOURCE_FILE)?;
            let err = verify_corpus_bundle(&sample, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::TamperedTrace => {
            let (provided, changed) = forge_corpus_bundle_file(BUNDLE_TRACE_FILE)?;
            let err = verify_corpus_bundle(&sample, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::TamperedReport => {
            let (provided, changed) = forge_corpus_bundle_file(BUNDLE_REPORT_FILE)?;
            let err = verify_corpus_bundle(&sample, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
        CorpusScenario::TamperedManifest => {
            let (provided, changed) = forge_corpus_bundle_file(BUNDLE_MANIFEST_FILE)?;
            let err = verify_corpus_bundle(&sample, &provided).err();
            let reason = err.as_ref().map(doc_rejection_token).unwrap_or_default();
            (
                false,
                false,
                err.is_some(),
                changed,
                reason,
                [true, true, true, true],
            )
        }
    };
    Ok(CorpusScenarioEntry {
        slug: scenario.slug().to_string(),
        description: scenario.describe().to_string(),
        input_kind: scenario.input_kind().to_string(),
        expectation: scenario.expectation().to_string(),
        produced_trace,
        verified,
        refused,
        input_changed,
        rejection_reason,
        no_execution: cells[0],
        no_evidence: cells[1],
        no_promotion: cells[2],
        no_training: cells[3],
    })
}

/// Run every corpus scenario, in canonical order, recording each observed outcome. Pure (no I/O).
fn canonical_corpus_scenario_entries() -> Result<Vec<CorpusScenarioEntry>, TraceError> {
    CorpusScenario::ALL
        .into_iter()
        .map(run_corpus_scenario)
        .collect()
}

/// Compute the coverage summary over the observed entries: counts, boundary cells proven, whether every observed
/// outcome met its expectation, the whole-corpus-binding fact, and the distinct input kinds / rejection reasons.
fn corpus_scenario_coverage(
    entries: &[CorpusScenarioEntry],
    whole_corpus_bound: bool,
) -> CorpusScenarioCoverage {
    let scenario_count = entries.len();
    let verified_count = entries.iter().filter(|e| e.verified).count();
    let refused_count = entries.iter().filter(|e| e.refused).count();
    let boundary_count = 4;
    let cells_total = scenario_count * boundary_count;
    let cells_proven: usize = entries
        .iter()
        .map(|e| {
            [e.no_execution, e.no_evidence, e.no_promotion, e.no_training]
                .iter()
                .filter(|cell| **cell)
                .count()
        })
        .sum();
    let all_expectations_met = entries.iter().all(|e| {
        if e.expectation == "verifies" {
            e.verified && !e.refused
        } else {
            e.refused && !e.verified
        }
    });
    CorpusScenarioCoverage {
        scenario_count,
        verified_count,
        refused_count,
        boundary_count,
        cells_total,
        cells_proven,
        all_expectations_met,
        all_boundaries_hold: cells_proven == cells_total,
        whole_corpus_bound,
        distinct_input_kinds: sorted_unique(entries.iter().map(|e| e.input_kind.clone()).collect()),
        distinct_rejection_reasons: sorted_unique(
            entries
                .iter()
                .filter(|e| !e.rejection_reason.is_empty())
                .map(|e| e.rejection_reason.clone())
                .collect(),
        ),
    }
}

fn corpus_scenario_boundary() -> Vec<String> {
    CORPUS_SCENARIO_BOUNDARY_LINES
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The corpus-scenario pack manifest JSON: every observed scenario row + coverage + verified-case source
/// identity + boundary. Pure.
pub fn corpus_scenario_pack_manifest() -> Result<String, TraceError> {
    let scenarios = canonical_corpus_scenario_entries()?;
    let coverage = corpus_scenario_coverage(&scenarios, corpus_whole_binding_holds()?);
    let source = corpus_source(&corpus_scenario_sample())?;
    let pack = CorpusScenarioPack {
        schema: "cognitive-corpus-scenario-pack-v0.1".to_string(),
        scenarios,
        coverage,
        source,
        boundary: corpus_scenario_boundary(),
    };
    Ok(serde_json::to_string_pretty(&pack).expect("CorpusScenarioPack serializes"))
}

/// Render the plain-text corpus-scenario report: each scenario's input class, expected/observed outcome,
/// rejection reason, and boundary cells, plus the coverage summary, the verified case's SOURCE SELECTION, and
/// the boundary. Pure FORMATTING.
pub fn corpus_scenario_report() -> Result<String, TraceError> {
    let entries = canonical_corpus_scenario_entries()?;
    let coverage = corpus_scenario_coverage(&entries, corpus_whole_binding_holds()?);
    let source = corpus_source(&corpus_scenario_sample())?;
    let mut out = String::new();
    out.push_str("COGNITIVE OS — CORPUS FLOW INPUT-INTEGRITY SCENARIOS\n");
    out.push_str("schema: cognitive-corpus-scenario-pack-v0.1\n");
    out.push_str("(each scenario varies the CORPUS INPUT and observes the real check; it records, it does not act)\n\n");
    for e in &entries {
        out.push_str(&format!("[{}]  ({})\n", e.slug, e.input_kind));
        out.push_str(&format!("    {}\n", e.description));
        out.push_str(&format!(
            "    expected: {}    observed: {}\n",
            e.expectation,
            if e.verified { "verified" } else { "refused" }
        ));
        if !e.rejection_reason.is_empty() {
            out.push_str(&format!("    rejection: {}\n", e.rejection_reason));
        }
        out.push_str(&format!(
            "    boundary: no_execution={} no_evidence={} no_promotion={} no_training={}\n",
            e.no_execution, e.no_evidence, e.no_promotion, e.no_training
        ));
    }
    out.push_str("\nCOVERAGE\n");
    out.push_str(&format!(
        "    scenarios:            {}\n",
        coverage.scenario_count
    ));
    out.push_str(&format!(
        "    verified:             {}\n",
        coverage.verified_count
    ));
    out.push_str(&format!(
        "    refused:              {}\n",
        coverage.refused_count
    ));
    out.push_str(&format!(
        "    cells proven:         {}/{}\n",
        coverage.cells_proven, coverage.cells_total
    ));
    out.push_str(&format!(
        "    all_expectations_met: {}\n",
        coverage.all_expectations_met
    ));
    out.push_str(&format!(
        "    all_boundaries_hold:  {}\n",
        coverage.all_boundaries_hold
    ));
    out.push_str(&format!(
        "    whole_corpus_bound:   {}\n",
        coverage.whole_corpus_bound
    ));
    out.push_str(&format!(
        "    distinct input kinds: {}\n",
        coverage.distinct_input_kinds.join(", ")
    ));
    out.push_str(&format!(
        "    distinct rejections:  {}\n",
        coverage.distinct_rejection_reasons.join(", ")
    ));
    out.push_str("\nSOURCE SELECTION (verified case)\n");
    out.push_str(&format!(
        "    grounded document:  [{}] {}\n",
        source.document_index, source.document_title
    ));
    out.push_str(&format!("    grounded span:      {}\n", source.span_id));
    out.push_str(&format!("    grounded text:      {}\n", source.span_text));
    out.push_str("\nBOUNDARY\n");
    for line in CORPUS_SCENARIO_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    Ok(out)
}

/// The corpus-scenario pack as `(filename, content)` pairs in write order: the structured manifest and its
/// rendered report. Pure: both are derived from the observed scenario set.
pub fn corpus_scenario_pack_files() -> Result<Vec<(&'static str, String)>, TraceError> {
    Ok(vec![
        (CORPUS_SCENARIO_PACK_FILE, corpus_scenario_pack_manifest()?),
        (CORPUS_SCENARIO_REPORT_FILE, corpus_scenario_report()?),
    ])
}

/// Verify a provided corpus-scenario pack WITHOUT trusting it: re-derive both files (re-running every scenario)
/// and byte-compare. A missing file is [`TraceError::BundleMissingFile`]; any tampered/stale/foreign file is
/// [`TraceError::BundleMismatch`]. Pure (no I/O).
pub fn verify_corpus_scenario_pack(provided: &[(String, String)]) -> Result<(), TraceError> {
    compare_bundle(&corpus_scenario_pack_files()?, provided)
}

/// The `corpus-scenarios` command: list the finite corpus-scenario set (slug + one-line description). Pure.
pub fn list_corpus_scenarios() -> String {
    let mut out = String::from(
        "cognitive-demo — corpus-flow input scenarios (each proves an input-integrity property):\n",
    );
    for s in CorpusScenario::ALL {
        out.push_str(&format!("    {:<24} {}\n", s.slug(), s.describe()));
    }
    out
}

/// The corpus input-integrity matrix JSON: one row per scenario (input class × observed outcome × boundary
/// cells), the coverage summary, the verified case's SOURCE IDENTITY, and the boundary — re-derived from the
/// scenario set. Pure: it never trusts the pack files; the matrix command verifies the pack separately before
/// emitting this.
pub fn corpus_scenario_matrix() -> Result<String, TraceError> {
    let entries = canonical_corpus_scenario_entries()?;
    let coverage = corpus_scenario_coverage(&entries, corpus_whole_binding_holds()?);
    let source = corpus_source(&corpus_scenario_sample())?;
    let rows = entries
        .iter()
        .map(|e| CorpusMatrixRow {
            slug: e.slug.clone(),
            input_kind: e.input_kind.clone(),
            expectation: e.expectation.clone(),
            outcome: if e.verified { "verified" } else { "refused" }.to_string(),
            rejection_reason: e.rejection_reason.clone(),
            no_execution: e.no_execution,
            no_evidence: e.no_evidence,
            no_promotion: e.no_promotion,
            no_training: e.no_training,
        })
        .collect();
    let matrix = CorpusScenarioMatrix {
        schema: "cognitive-corpus-scenario-matrix-v0.1".to_string(),
        scenarios: rows,
        coverage,
        source,
        boundary: corpus_scenario_boundary(),
    };
    Ok(serde_json::to_string_pretty(&matrix).expect("CorpusScenarioMatrix serializes"))
}

// --- NOVELTY-0: hypothesis-only novelty packet harness. Where the corpus arc READS local documents into a
//     VERIFIED trace, NOVELTY-0 adds a bounded HYPOTHESIS layer ON TOP of that verified trace: given a verified
//     corpus trace and an operator-supplied FRAME, it produces a deterministic `NoveltyPacket` that records the
//     frame's candidate broken assumptions, the verified facts that must be preserved (each grounded VERBATIM in
//     a verified corpus span), a candidate hypothesis, falsifiers, and NON-EXECUTING probe requests. The packet
//     PROPOSES; it never proves. It carries `authority = hypothesis_only` (an enum with no evidence/promoted/
//     truth variant) and an explicit `forbidden_uses` list, so it can never become evidence, execute, promote,
//     or train. There is NO model and NO score: the FRAME is operator-supplied DATA (recorded, never trusted as
//     fact), and the ONLY grounded content is verified span text — an unsupported preserved fact, a corpus trace
//     missing its receipt hash, an empty frame, or any tampered packet is REFUSED by re-derivation. Like every
//     artifact here, `NoveltyPacket` is `Serialize` but NOT `Deserialize`: it is verified by re-deriving it from
//     the corpus + frame (the source of truth) and byte-comparing, which is why novelty-report/replay require
//     the same `--input-dir` + `--frame` inputs, exactly as corpus-report/corpus-bundle-verify do. P12 stays
//     training_justified=false; the library is filesystem-free (the shell reads the corpus dir + frame file and
//     validates every path). Doctrine: Novelty packets propose. They do not prove. They cite verified receipts.
//     They do not create authority. Probe requests do not execute. Nothing becomes evidence, promotes, or trains. ---

/// The eight-line NOVELTY-0 boundary, embedded in every packet and printed in the report. Pinned as data so a
/// test/gate can assert every line is present.
pub const NOVELTY_BOUNDARY_LINES: [&str; 8] = [
    "Novelty packets propose.",
    "They do not prove.",
    "They cite verified receipts.",
    "They do not create authority.",
    "Probe requests do not execute.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The four uses a novelty packet is FORBIDDEN from ever acquiring, recorded explicitly in every packet so the
/// refusal is machine-checkable from the packet's own bytes, not merely implied.
pub const NOVELTY_FORBIDDEN_USES: [&str; 4] = ["evidence", "execution", "promotion", "training"];

/// The single authority a novelty packet may carry: HYPOTHESIS-ONLY. The enum has ONE variant (there is no
/// `Evidence` / `Promoted` / `Truth` variant to construct), so a packet structurally cannot claim any authority
/// beyond proposal. `Serialize` but NOT `Deserialize` — re-derived, never parsed back into authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum NoveltyAuthority {
    #[serde(rename = "hypothesis_only")]
    HypothesisOnly,
}

/// A request to TEST a candidate assumption — recorded, never executed. `executes` is always `false` and
/// `status` is always operator-review-gated, so the packet emits only PROBE REQUESTS, never executions.
/// `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct NoveltyProbeRequest {
    schema: String,
    request_id: u64,
    question: String,
    status: String,
    executes: bool,
}

/// A deterministic, hypothesis-only novelty packet derived from a VERIFIED corpus trace and an operator frame.
/// It cites the reading receipt by hash and the corpus by identity hash, records the frame's candidate broken
/// assumptions, the verified facts to preserve (each grounded VERBATIM in a verified corpus span), a candidate
/// hypothesis, falsifiers, and non-executing probe requests. `authority` is `hypothesis_only` and
/// `forbidden_uses` lists what it may never become. `Serialize` but NOT `Deserialize` — re-derived from the
/// corpus + frame and byte-compared, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct NoveltyPacket {
    schema: String,
    packet_id: String,
    source_receipt_hash: u64,
    source_corpus_hash: u64,
    frame_text: String,
    broken_assumptions: Vec<String>,
    preserved_facts: Vec<String>,
    candidate_hypothesis: String,
    falsifiers: Vec<String>,
    probe_requests: Vec<NoveltyProbeRequest>,
    authority: NoveltyAuthority,
    forbidden_uses: Vec<String>,
    boundary: Vec<String>,
}

/// All verified span texts of the corpus, in reading order — the ONLY facts a packet may preserve. A preserved
/// fact that is not VERBATIM one of these is unsupported and refused. Pure (no I/O).
fn corpus_verified_spans(documents: &[(String, String)]) -> Vec<String> {
    let corpus = corpus_from_documents(documents);
    corpus
        .metadata()
        .iter()
        .flat_map(|doc| doc.span_ids.iter().copied())
        .filter_map(|id| corpus.read_span(id).map(|span| span.text().to_string()))
        .collect()
}

/// A deterministic, dependency-free identity hash of the WHOLE corpus (every document's title + content), so
/// the packet binds the corpus it was derived from. Distinct from the reading receipt's structure hash (a
/// different input scope); both are recorded so a packet cannot be silently re-pointed at a different corpus.
fn corpus_identity_hash(documents: &[(String, String)]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for (title, content) in documents {
        title.hash(&mut hasher);
        content.hash(&mut hasher);
    }
    hasher.finish()
}

/// Split the operator FRAME into its candidate assumptions: each non-empty, trimmed line is one assumption the
/// operator proposes to break. The frame is untrusted DATA — recorded and structured, never grounded as a fact.
/// Returns [`TraceError::EmptyFrame`] if no line carries text (a frame with nothing to break cannot produce a
/// hypothesis). Pure.
fn frame_assumptions(frame_text: &str) -> Result<Vec<String>, TraceError> {
    let lines: Vec<String> = frame_text
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();
    if lines.is_empty() {
        return Err(TraceError::EmptyFrame);
    }
    Ok(lines)
}

/// Require every preserved fact to be VERBATIM one of the corpus's verified spans. This is the grounding gate:
/// the ONLY facts a packet may carry are verified reads, so a frame claim (or any unsupported text) can never
/// be laundered into a preserved fact — it is refused with [`TraceError::UnsupportedPreservedFact`]. Pure.
fn novelty_facts_grounded(
    documents: &[(String, String)],
    facts: &[String],
) -> Result<(), TraceError> {
    let verified_spans = corpus_verified_spans(documents);
    for fact in facts {
        if !verified_spans.iter().any(|span| span == fact) {
            return Err(TraceError::UnsupportedPreservedFact);
        }
    }
    Ok(())
}

/// Derive the deterministic hypothesis-only novelty packet for an operator corpus + frame. It starts from the
/// VERIFIED corpus trace (fails closed via [`corpus_trace`] if the read does not verify, or [`TraceError::EmptyCorpus`]),
/// cites the reading receipt by hash ([`TraceError::MissingReceiptHash`] if the verified trace carries none),
/// grounds the preserved facts VERBATIM in the corpus's verified spans ([`novelty_facts_grounded`]), records the
/// frame's candidate broken assumptions, and emits non-executing probe requests. NO model, NO score: the
/// structure is deterministic and the only authority is `hypothesis_only`. Pure (no I/O).
fn novelty_packet(
    documents: &[(String, String)],
    frame_text: &str,
) -> Result<NoveltyPacket, TraceError> {
    // 1. The packet is grounded in a VERIFIED corpus trace — re-derived here, fails closed if it does not verify.
    let trace = corpus_trace(documents)?;
    let receipt_hash = trace
        .reading_structure_hash
        .ok_or(TraceError::MissingReceiptHash)?;
    let source = corpus_source(documents)?;
    let corpus_hash = corpus_identity_hash(documents);

    // 2. The operator FRAME supplies the candidate assumptions to break (untrusted data, recorded verbatim).
    let broken_assumptions = frame_assumptions(frame_text)?;

    // 3. The ONLY grounded content is verified span text. The preserved fact is the grounded source span, and
    //    the grounding gate REFUSES anything that is not a verified span — so no frame claim can be laundered in.
    let preserved_facts = vec![source.span_text.clone()];
    novelty_facts_grounded(documents, &preserved_facts)?;

    // 4. Deterministic, hypothesis-labeled candidate + falsifiers + non-executing probe requests.
    let first_assumption = &broken_assumptions[0];
    let candidate_hypothesis = format!(
        "Proposal only (hypothesis_only): if the assumption \"{first_assumption}\" is relaxed, the verified record still constrains it — \"{}\". This is a candidate to probe, not a claim.",
        source.span_text
    );
    let falsifiers: Vec<String> = preserved_facts
        .iter()
        .map(|fact| {
            format!("Falsified if an observation contradicts the verified span: \"{fact}\".")
        })
        .collect();
    let probe_requests: Vec<NoveltyProbeRequest> = broken_assumptions
        .iter()
        .enumerate()
        .map(|(index, assumption)| NoveltyProbeRequest {
            schema: "cognitive-novelty-probe-request-v0.1".to_string(),
            request_id: index as u64,
            question: format!(
                "What observation would test relaxing the assumption \"{assumption}\"?"
            ),
            status: "requires_operator_review".to_string(),
            executes: false,
        })
        .collect();

    // 5. A deterministic, replayable packet id over the receipt hash, the corpus hash, and the frame text.
    let packet_id = format!(
        "novelty-{}",
        bundle_content_hash(&format!("{receipt_hash}|{corpus_hash}|{frame_text}"))
    );

    Ok(NoveltyPacket {
        schema: "cognitive-novelty-packet-v0.1".to_string(),
        packet_id,
        source_receipt_hash: receipt_hash,
        source_corpus_hash: corpus_hash,
        frame_text: frame_text.to_string(),
        broken_assumptions,
        preserved_facts,
        candidate_hypothesis,
        falsifiers,
        probe_requests,
        authority: NoveltyAuthority::HypothesisOnly,
        forbidden_uses: NOVELTY_FORBIDDEN_USES
            .iter()
            .map(|use_| use_.to_string())
            .collect(),
        boundary: NOVELTY_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// The novelty packet as pretty JSON. Pure and deterministic (fixed field order), so it re-derives byte-for-byte.
fn novelty_packet_json(
    documents: &[(String, String)],
    frame_text: &str,
) -> Result<String, TraceError> {
    Ok(
        serde_json::to_string_pretty(&novelty_packet(documents, frame_text)?)
            .expect("NoveltyPacket serializes"),
    )
}

/// The `novelty-packet` command body: confirm the PROVIDED corpus trace IS the trace re-derived from the
/// `--input-dir` corpus (refuse a tampered / stale / foreign / receipt-hash-stripped trace via
/// [`verify_corpus_trace_json`]), then derive the hypothesis-only novelty packet from that VERIFIED corpus and
/// the operator frame. The corpus is the source of truth — that is why this command requires `--input-dir`
/// alongside `--corpus-trace`, exactly as corpus-report does. Pure (no I/O).
pub fn run_novelty_packet(
    documents: &[(String, String)],
    provided_trace_json: &str,
    frame_text: &str,
) -> Result<String, TraceError> {
    verify_corpus_trace_json(documents, provided_trace_json)?;
    novelty_packet_json(documents, frame_text)
}

/// Re-derive the novelty packet from the SAME corpus + frame and confirm the PROVIDED packet JSON is
/// byte-for-byte that packet. The provided packet is NEVER parsed back into authority (`NoveltyPacket` is
/// `Serialize` but not `Deserialize`) — it is only COMPARED against the freshly re-derived packet, so a
/// tampered / stale / foreign packet is REFUSED ([`TraceError::NoveltyPacketMismatch`]). Pure (no I/O).
pub fn verify_novelty_packet_json(
    documents: &[(String, String)],
    frame_text: &str,
    provided: &str,
) -> Result<(), TraceError> {
    if provided == novelty_packet_json(documents, frame_text)? {
        Ok(())
    } else {
        Err(TraceError::NoveltyPacketMismatch)
    }
}

/// Render the novelty operator report from the re-derived packet: the proposal banner (hypothesis_only — not
/// truth), the operator frame (recorded, never trusted), the candidate broken assumptions, the preserved
/// (verified) facts, the candidate hypothesis, the falsifiers, the probe requests (each NON-executing), the
/// forbidden uses, and the eight-line NOVELTY-0 boundary. Pure FORMATTING derived from the packet.
fn novelty_report_body(packet: &NoveltyPacket) -> String {
    let mut out = String::from("NOVELTY PACKET (PROPOSAL ONLY — hypothesis_only, not truth)\n");
    out.push_str(&format!("    packet_id:            {}\n", packet.packet_id));
    out.push_str(&format!(
        "    source_receipt_hash:  {}\n",
        packet.source_receipt_hash
    ));
    out.push_str(&format!(
        "    source_corpus_hash:   {}\n",
        packet.source_corpus_hash
    ));
    out.push_str("    authority:            hypothesis_only\n");
    out.push_str("\nFRAME (operator-supplied; recorded, never trusted as fact)\n");
    for line in packet.frame_text.lines() {
        out.push_str(&format!("    {line}\n"));
    }
    out.push_str("\nBROKEN ASSUMPTIONS (candidates to challenge — no truth claimed)\n");
    for assumption in &packet.broken_assumptions {
        out.push_str(&format!("    - {assumption}\n"));
    }
    out.push_str("\nPRESERVED FACTS (verified corpus spans — the only grounded content)\n");
    for fact in &packet.preserved_facts {
        out.push_str(&format!("    - {fact}\n"));
    }
    out.push_str("\nCANDIDATE HYPOTHESIS\n");
    out.push_str(&format!("    {}\n", packet.candidate_hypothesis));
    out.push_str("\nFALSIFIERS\n");
    for falsifier in &packet.falsifiers {
        out.push_str(&format!("    - {falsifier}\n"));
    }
    out.push_str("\nPROBE REQUESTS (recorded, NOT executed)\n");
    for probe in &packet.probe_requests {
        out.push_str(&format!(
            "    [{}] {} (status: {}, executes: {})\n",
            probe.request_id, probe.question, probe.status, probe.executes
        ));
    }
    out.push_str("\nFORBIDDEN USES (this packet may never become or do)\n");
    for use_ in &packet.forbidden_uses {
        out.push_str(&format!("    - {use_}\n"));
    }
    out.push_str("\nBOUNDARY\n");
    for line in NOVELTY_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The `novelty-report` command body: re-derive + verify the packet from the SAME corpus + frame (refuse a
/// tampered packet), then render the operator report. The corpus + frame are the source of truth, so this
/// command requires `--input-dir` + `--frame` alongside `--packet`. Pure (no I/O).
pub fn run_novelty_report(
    documents: &[(String, String)],
    frame_text: &str,
    provided: &str,
) -> Result<String, TraceError> {
    verify_novelty_packet_json(documents, frame_text, provided)?;
    Ok(novelty_report_body(&novelty_packet(documents, frame_text)?))
}

/// The `novelty-replay` command body: re-derive the packet from the corpus + frame and confirm the provided
/// packet is byte-identical — a DETERMINISM proof (re-derivation is bit-for-bit) that also REFUSES any tampered
/// packet ([`TraceError::NoveltyPacketMismatch`]). Returns the confirmation summary. Pure (no I/O).
pub fn run_novelty_replay(
    documents: &[(String, String)],
    frame_text: &str,
    provided: &str,
) -> Result<String, TraceError> {
    verify_novelty_packet_json(documents, frame_text, provided)?;
    Ok(String::from(
        "novelty-replay: OK — the packet re-derives byte-identically from the corpus and frame (deterministic). It proposes; it does not prove.\n",
    ))
}

// --- DREAM-EXPORT-0: Dream Export Receipt / Provenance Bridge. A terminal, inert `DreamPacket` (from the
//     STANDALONE dream-engine — DREAM-0, which itself has NO export path) is BRIDGED into the EXISTING
//     hypothesis-only proposal path: the bridge re-derives the canonical dream packet from the SAME corpus +
//     frame + seed + weirdness, builds a `HypothesisSpec` from the dream's distortion + verified grounding, and
//     calls the EXISTING `hypothesis_layer::propose`. The result is a real `HypothesisPacket` carrying the
//     EXISTING `Authority::HypothesisOnly` (read straight off the proposed packet — never a new authority). A
//     `DreamExportReceipt` records dream-origin provenance (dream packet id, input hash, seed, engine version,
//     operator ids, the dream's grounding receipt hashes) OUTSIDE the frozen hypothesis-layer authority model, so
//     a dream-exported hypothesis stays DISTINGUISHABLE from an ordinary one and the dream origin stays
//     auditable. The dream's private `dream_only` authority NEVER crosses the boundary — only ids/hashes/operator
//     tokens do. Like every artifact here, the receipt + bundle are `Serialize` but NOT `Deserialize`: they are
//     re-derived from the corpus + frame and byte-compared, never parsed back into authority, which is why
//     dream-export-report/replay require `--input-dir` + `--frame` exactly as novelty-report/replay do.
//     Doctrine: Dream export preserves provenance. It does not create a new authority. Exported dream material
//     remains hypothesis_only. Dream origin remains auditable. Probe requests do not execute. Nothing becomes
//     evidence, promotes, or trains. ---

/// The eight-line DREAM-EXPORT-0 boundary, embedded verbatim in every export receipt and printed in the report,
/// so the refusal is machine-checkable from the bundle's own bytes.
pub const DREAM_EXPORT_BOUNDARY_LINES: [&str; 8] = [
    "Dream export preserves provenance.",
    "It does not create a new authority.",
    "Exported dream material remains hypothesis_only.",
    "Dream origin remains auditable.",
    "Probe requests do not execute.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

// The hypothesis a dream export proposes is HIGHLY SPECULATIVE: low prior, high uncertainty, and a reversible,
// low-risk thought-probe. These are bounded per-mille values (0..=hypothesis_layer::SCALE), fixed so the
// proposal re-derives byte-identically. The dream is a candidate to probe, never a claim.
const DREAM_HYP_PRIOR: i64 = 100;
const DREAM_HYP_UNCERTAINTY: i64 = 900;
const DREAM_HYP_TEST_COST: i64 = 1;
const DREAM_HYP_RISK: i64 = 100;
const DREAM_HYP_REVERSIBILITY: i64 = 1000;

/// The stable snake_case token for a dream distortion operator — mirrors dream-engine's own serde rename, so a
/// receipt records WHICH distortions produced the dream without re-serializing the engine's private vocabulary.
fn operator_token(op: dream_engine::DistortionOperator) -> &'static str {
    use dream_engine::DistortionOperator::*;
    match op {
        RoleInversion => "role_inversion",
        CategoryViolation => "category_violation",
        ConstraintRemoval => "constraint_removal",
        ContradictionBraid => "contradiction_braid",
        ScaleShift => "scale_shift",
    }
}

/// The provenance bridge between a terminal dream packet and the EXISTING hypothesis-only proposal path. It is
/// `Serialize` but NOT `Deserialize`: it is re-derived and byte-compared, never parsed back into authority. It
/// holds NO authority of its own — `authority_after_export` is the EXISTING [`Authority::HypothesisOnly`] read
/// off the proposed packet, and the receipt only NAMES the dream by id/hash so the origin stays auditable
/// OUTSIDE the frozen authority model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DreamExportReceipt {
    pub schema: String,
    pub export_id: String,
    /// Dream provenance — the terminal packet this hypothesis descends from (named, never re-derived to authority).
    pub dream_packet_id: String,
    pub dream_input_hash: String,
    pub dream_seed: u64,
    pub dream_weirdness: i64,
    pub dream_engine_version: String,
    pub dream_operator_ids: Vec<String>,
    pub source_receipt_memory_hash: u64,
    pub source_receipt_answer_hash: u64,
    /// The deterministic content id of the proposed hypothesis (its FNV-1a hash) — binds the receipt to it.
    pub exported_hypothesis_hash: u64,
    /// Always `true`: the export went through `hypothesis_layer::propose`, the EXISTING gate — not a new path.
    pub exported_via_existing_hypothesis_gate: bool,
    /// The EXISTING authority the exported material carries, read off the proposed packet. Always `HypothesisOnly`.
    pub authority_after_export: Authority,
    /// Always `true`: this hypothesis descends from a dream, so it stays DISTINGUISHABLE from an ordinary one.
    pub dream_origin: bool,
    /// The exported hypothesis's own forbidden-uses list (the canonical hypothesis-layer quarantine).
    pub forbidden_uses: Vec<String>,
    pub export_trace_hash: String,
    pub boundary: Vec<String>,
}

/// A dream-export bundle: the provenance [`DreamExportReceipt`] plus the EXISTING-path [`HypothesisPacket`] it
/// produced. `Serialize` but NOT `Deserialize` (it embeds a `HypothesisPacket`, which is itself not
/// deserializable) — re-derived from the corpus + frame and byte-compared, never parsed back into authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DreamExportBundle {
    pub schema: String,
    pub receipt: DreamExportReceipt,
    pub hypothesis: HypothesisPacket,
}

/// Build the dream-engine input from the operator corpus + frame + dials. Pure.
fn dream_export_input(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
) -> dream_engine::DreamInput {
    dream_engine::DreamInput {
        documents: documents
            .iter()
            .map(|(name, text)| dream_engine::DreamDocument {
                name: name.clone(),
                text: text.clone(),
            })
            .collect(),
        frame_text: frame_text.to_string(),
        seed,
        weirdness,
    }
}

/// Re-derive the canonical dream packet and BRIDGE it into the existing hypothesis-only proposal path. The dream
/// packet is re-derived here (fails closed via [`TraceError::DreamExport`] if the corpus does not verify or the
/// dream is degenerate — so an export REQUIRES a valid re-derived packet), then a `HypothesisSpec` is built from
/// the dream's impossible link (the distortion) and its VERIFIED grounding receipt (cited as evidence so the
/// hypothesis is created-from-trace and the dream origin is auditable), and the EXISTING `propose` is called.
/// The receipt records dream-origin provenance and the EXISTING authority. Pure (no I/O).
fn dream_export_bundle(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
) -> Result<DreamExportBundle, TraceError> {
    // 1. Re-derive the terminal dream packet from the SAME inputs. dream-engine grounds it on a VERIFIED
    //    canonical read and refuses a degenerate dream — so this fails closed if there is nothing valid to export.
    let input = dream_export_input(documents, frame_text, seed, weirdness);
    let packet =
        dream_engine::dream_packet(&input).map_err(|e| TraceError::DreamExport(e.to_string()))?;

    // 2. Build the hypothesis from the dream's distortion + its verified grounding. The statement is explicitly a
    //    dream-origin PROPOSAL (hypothesis_only), and the evidence cites the dream's verified reading receipt —
    //    so the exported hypothesis is created-from-trace AND carries a `dream:` provenance label.
    let distortion = packet.impossible_links.first().ok_or_else(|| {
        TraceError::DreamExport("the dream packet has no impossible link to export".to_string())
    })?;
    let probe = packet.probe_requests.first().ok_or_else(|| {
        TraceError::DreamExport("the dream packet has no probe request to export".to_string())
    })?;
    let statement = format!(
        "Dream-derived proposal (hypothesis_only, dream_origin): {}. This is a candidate to probe, not a claim — its only grounding is the verified reading receipt it cites.",
        distortion.text
    );
    let evidence = vec![EvidenceRef {
        answer_hash: packet.source_receipt_answer_hash,
        memory_hash: packet.source_receipt_memory_hash,
        source_label: format!("dream:{}", packet.packet_id),
    }];
    let spec = HypothesisSpec {
        statement,
        prior: DREAM_HYP_PRIOR,
        uncertainty: DREAM_HYP_UNCERTAINTY,
        test_cost: DREAM_HYP_TEST_COST,
        risk: DREAM_HYP_RISK,
        reversibility: DREAM_HYP_REVERSIBILITY,
        evidence_inputs: evidence,
        probe_description: probe.question.clone(),
    };
    // 3. The EXISTING hypothesis-only gate. The returned packet carries the EXISTING Authority::HypothesisOnly.
    let hypothesis = propose(spec).map_err(TraceError::Hypothesis)?;

    // 4. Collect the distinct distortion operators actually recorded on the packet (canonical order), as
    //    provenance of WHICH distortions produced the dream. Scale-shift is recorded on candidate frames (text).
    let mut dream_operator_ids: Vec<String> = Vec::new();
    for op in dream_engine::OPERATORS {
        let used = packet.broken_assumptions.iter().any(|b| b.operator == op)
            || packet.impossible_links.iter().any(|l| l.operator == op)
            || (op == dream_engine::DistortionOperator::ScaleShift
                && !packet.candidate_frames.is_empty());
        if used {
            dream_operator_ids.push(operator_token(op).to_string());
        }
    }

    // 5. The receipt binds the hypothesis to its dream origin OUTSIDE the frozen authority model. The export
    //    trace hash is a deterministic digest of the dream->hypothesis binding (demonstrable; the load-bearing
    //    check is byte-for-byte re-derivation in `verify_dream_export_bundle_json`).
    let export_trace_hash = bundle_content_hash(&format!(
        "{}|{}|{}|{}",
        packet.dream_input_hash,
        packet.packet_id,
        hypothesis.hypothesis_id(),
        packet.seed
    ));
    let export_id = format!(
        "dream-export-{}",
        bundle_content_hash(&format!(
            "{}|{}",
            packet.dream_input_hash,
            hypothesis.hypothesis_id()
        ))
    );
    let receipt = DreamExportReceipt {
        schema: "dream-export-receipt-v0.1".to_string(),
        export_id,
        dream_packet_id: packet.packet_id.clone(),
        dream_input_hash: packet.dream_input_hash.clone(),
        dream_seed: packet.seed,
        dream_weirdness: packet.weirdness,
        dream_engine_version: packet.schema.clone(),
        dream_operator_ids,
        source_receipt_memory_hash: packet.source_receipt_memory_hash,
        source_receipt_answer_hash: packet.source_receipt_answer_hash,
        exported_hypothesis_hash: hypothesis.hypothesis_id(),
        exported_via_existing_hypothesis_gate: true,
        // Read the authority straight off the proposed packet — never a new or fabricated variant.
        authority_after_export: hypothesis.authority(),
        dream_origin: true,
        forbidden_uses: hypothesis.forbidden_uses().to_vec(),
        export_trace_hash,
        boundary: DREAM_EXPORT_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };

    Ok(DreamExportBundle {
        schema: "dream-export-bundle-v0.1".to_string(),
        receipt,
        hypothesis,
    })
}

/// The dream-export bundle as pretty JSON. Pure and deterministic (fixed field order), so it re-derives
/// byte-for-byte from the same corpus + frame + dials.
fn dream_export_bundle_json(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
) -> Result<String, TraceError> {
    Ok(serde_json::to_string_pretty(&dream_export_bundle(
        documents, frame_text, seed, weirdness,
    )?)
    .expect("DreamExportBundle serializes"))
}

/// The `dream-export` command body: re-derive the dream packet and bridge it into the hypothesis-only path,
/// emitting the export bundle. If a `--dream-packet` is provided, it is REFUSED unless it is byte-for-byte the
/// re-derived dream packet (a tampered / stale / foreign packet cannot be laundered into an export). Pure (no I/O).
pub fn run_dream_export(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided_dream_packet: Option<&str>,
) -> Result<String, TraceError> {
    if let Some(provided) = provided_dream_packet {
        let input = dream_export_input(documents, frame_text, seed, weirdness);
        dream_engine::verify_dream_packet_json(&input, provided)
            .map_err(|e| TraceError::DreamExport(e.to_string()))?;
    }
    dream_export_bundle_json(documents, frame_text, seed, weirdness)
}

/// Re-derive the dream-export bundle from the SAME corpus + frame + dials and confirm the PROVIDED bundle JSON is
/// byte-for-byte that bundle. The provided bundle is NEVER parsed back into authority — only COMPARED against the
/// re-derived one — so a tampered / stale / foreign bundle is REFUSED ([`TraceError::DreamExportMismatch`]). Pure.
pub fn verify_dream_export_bundle_json(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided: &str,
) -> Result<(), TraceError> {
    if provided == dream_export_bundle_json(documents, frame_text, seed, weirdness)? {
        Ok(())
    } else {
        Err(TraceError::DreamExportMismatch)
    }
}

/// Render the dream-export operator report from the re-derived bundle: the provenance banner (hypothesis_only,
/// dream_origin), the dream packet id / input hash / seed / engine version / operator ids, the EXISTING authority
/// after export, the exported hypothesis (a PROPOSAL, not truth) with its `dream:` evidence label (so the origin
/// is auditable), the note that the source dream's probe requests do not execute, the forbidden uses, and the
/// eight-line boundary. Pure FORMATTING derived from the bundle.
fn dream_export_report_body(bundle: &DreamExportBundle) -> String {
    let r = &bundle.receipt;
    let h = &bundle.hypothesis;
    let mut out =
        String::from("DREAM EXPORT (PROVENANCE BRIDGE — hypothesis_only, dream_origin)\n");
    out.push_str(&format!("    export_id:              {}\n", r.export_id));
    out.push_str(&format!(
        "    dream_packet_id:        {}\n",
        r.dream_packet_id
    ));
    out.push_str(&format!(
        "    dream_input_hash:       {}\n",
        r.dream_input_hash
    ));
    out.push_str(&format!("    dream_seed:             {}\n", r.dream_seed));
    out.push_str(&format!(
        "    dream_weirdness:        {}\n",
        r.dream_weirdness
    ));
    out.push_str(&format!(
        "    dream_engine_version:   {}\n",
        r.dream_engine_version
    ));
    out.push_str(&format!(
        "    dream_operator_ids:     {}\n",
        r.dream_operator_ids.join(", ")
    ));
    out.push_str("    authority_after_export: hypothesis_only\n");
    out.push_str(&format!("    dream_origin:           {}\n", r.dream_origin));
    out.push_str(&format!(
        "    via_existing_gate:      {}\n",
        r.exported_via_existing_hypothesis_gate
    ));
    out.push_str("\nEXPORTED HYPOTHESIS (PROPOSAL ONLY — hypothesis_only, not truth)\n");
    out.push_str(&format!(
        "    hypothesis_id:          {}\n",
        h.hypothesis_id()
    ));
    out.push_str(&format!("    statement:              {}\n", h.statement()));
    out.push_str(&format!(
        "    expected_utility:       {}\n",
        h.expected_utility()
    ));
    out.push_str("    evidence_inputs (dream provenance is auditable here):\n");
    for e in h.evidence_inputs() {
        out.push_str(&format!(
            "    - {} (answer_hash={}, memory_hash={})\n",
            e.source_label, e.answer_hash, e.memory_hash
        ));
    }
    out.push_str(&format!(
        "    recommended_probe:      {} (clearance: {:?})\n",
        h.recommended_probe().description(),
        h.recommended_probe().clearance()
    ));
    out.push_str("\nPROBE PROVENANCE\n");
    out.push_str("    the source dream's probe requests are recorded with executes: false — NEVER executed\n");
    out.push_str("\nFORBIDDEN USES (this exported hypothesis may never become or do)\n");
    for use_ in &r.forbidden_uses {
        out.push_str(&format!("    - {use_}\n"));
    }
    out.push_str("\nBOUNDARY\n");
    for line in DREAM_EXPORT_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The `dream-export-report` command body: re-derive + verify the bundle from the SAME corpus + frame + dials
/// (refuse a tampered bundle), then render the operator report from the re-derived (trusted) bundle. The corpus +
/// frame are the source of truth, so this command requires `--input-dir` + `--frame`. Pure (no I/O).
pub fn run_dream_export_report(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided_bundle: &str,
) -> Result<String, TraceError> {
    verify_dream_export_bundle_json(documents, frame_text, seed, weirdness, provided_bundle)?;
    Ok(dream_export_report_body(&dream_export_bundle(
        documents, frame_text, seed, weirdness,
    )?))
}

/// The `dream-export-replay` command body: re-derive the bundle from the corpus + frame + dials and confirm the
/// provided bundle is byte-identical — a determinism proof that also refuses any tampered bundle. Reads nothing
/// as authority; the export PROPOSES via the existing gate, it does not prove. Pure (no I/O).
pub fn run_dream_export_replay(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided_bundle: &str,
) -> Result<String, TraceError> {
    verify_dream_export_bundle_json(documents, frame_text, seed, weirdness, provided_bundle)?;
    Ok(String::from(
        "dream-export-replay: OK — the export bundle re-derives byte-identically from the corpus and frame (deterministic). Dream origin is preserved; the exported material is hypothesis_only.\n",
    ))
}

// ── DREAM-EXPORT-2 — Dream Export Scenario Matrix / Provenance Integrity ──────────────────────────────────────
// A deterministic matrix over the EXISTING dream-export bridge: one CLEAN export that must VERIFY, plus six
// tamper scenarios that must each be REFUSED (a tampered dream packet, a tampered receipt, a forged
// dream_origin=false, a mutated dream_input_hash, a mutated dream_packet_id, and a forged authority_after_export
// that injects the dream engine's private authority token). Each row records the OBSERVED outcome (never
// asserted), so a tamper that slipped through would record outcome="verifies" against expected="refused" and fail
// its test. The matrix also records the dream provenance fields, that the exported material stays hypothesis_only
// and is DISTINGUISHABLE from a plain hypothesis, that probe requests never execute, and the
// no-evidence / no-promotion / no-training boundary cells. Pure + re-derived-and-byte-compared on verify; it
// creates NO authority and the dream's private authority token is only ever FORGED-then-REFUSED, never minted.

/// The DREAM-EXPORT-2 scenario-matrix boundary. SOURCE-SAFE: the dream engine's private authority is named by its
/// lowercase serialized token `dream_only`, because a release_check gate keeps the PascalCase identifier crate-
/// private to dream-engine (so this very source file may not contain it — which is exactly what the line asserts).
pub const DREAM_EXPORT_MATRIX_BOUNDARY_LINES: [&str; 9] = [
    "Dream export scenarios vary the export artifact.",
    "They do not vary the authority.",
    "Dream provenance remains auditable.",
    "Exported material remains HypothesisOnly.",
    "dream_only remains private to dream-engine.",
    "Probe requests do not execute.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// One dream-export scenario: the CLEAN export (which must verify) or a deterministic tamper (which must be
/// refused). Each names which surface refuses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DreamExportScenario {
    CleanExport,
    TamperedDreamPacket,
    TamperedReceipt,
    ForgedDreamOriginFalse,
    MutatedDreamInputHash,
    MutatedDreamPacketId,
    ForgedAuthorityAfterExport,
}

impl DreamExportScenario {
    const ALL: [DreamExportScenario; 7] = [
        DreamExportScenario::CleanExport,
        DreamExportScenario::TamperedDreamPacket,
        DreamExportScenario::TamperedReceipt,
        DreamExportScenario::ForgedDreamOriginFalse,
        DreamExportScenario::MutatedDreamInputHash,
        DreamExportScenario::MutatedDreamPacketId,
        DreamExportScenario::ForgedAuthorityAfterExport,
    ];

    fn slug(self) -> &'static str {
        match self {
            DreamExportScenario::CleanExport => "clean-export",
            DreamExportScenario::TamperedDreamPacket => "tampered-dream-packet",
            DreamExportScenario::TamperedReceipt => "tampered-receipt",
            DreamExportScenario::ForgedDreamOriginFalse => "forged-dream-origin-false",
            DreamExportScenario::MutatedDreamInputHash => "mutated-dream-input-hash",
            DreamExportScenario::MutatedDreamPacketId => "mutated-dream-packet-id",
            DreamExportScenario::ForgedAuthorityAfterExport => "forged-authority-after-export",
        }
    }

    fn describe(self) -> &'static str {
        match self {
            DreamExportScenario::CleanExport => {
                "the clean dream export re-derives byte-identically and verifies"
            }
            DreamExportScenario::TamperedDreamPacket => {
                "a tampered source dream packet is refused before export"
            }
            DreamExportScenario::TamperedReceipt => {
                "a tampered export receipt (via-existing-gate bit) is refused"
            }
            DreamExportScenario::ForgedDreamOriginFalse => {
                "a forged dream_origin=false is refused (provenance cannot be stripped)"
            }
            DreamExportScenario::MutatedDreamInputHash => {
                "a mutated dream_input_hash is refused (provenance binding holds)"
            }
            DreamExportScenario::MutatedDreamPacketId => {
                "a mutated dream_packet_id is refused (provenance binding holds)"
            }
            DreamExportScenario::ForgedAuthorityAfterExport => {
                "a forged authority_after_export (the dream's private token) is refused"
            }
        }
    }

    /// Which surface returns the verdict for this scenario.
    fn target_surface(self) -> &'static str {
        match self {
            DreamExportScenario::CleanExport => "bundle_rederive_byte_compare",
            DreamExportScenario::TamperedDreamPacket => "dream_packet_cross_check",
            _ => "bundle_rederive_byte_compare",
        }
    }

    fn expected_verifies(self) -> bool {
        matches!(self, DreamExportScenario::CleanExport)
    }
}

/// The OBSERVED outcome of one scenario: whether the tamper genuinely changed the canonical bytes, whether it
/// injected the dream's private authority token (only the authority-forgery does), and the REAL verdict the
/// EXISTING verifier returned. All observed, never asserted — the matrix records what actually happened.
struct DreamExportAttempt {
    mutation_applied: bool,
    injects_dream_only: bool,
    verdict: Result<(), TraceError>,
}

/// Run ONE dream-export scenario: build the canonical artifact fresh, apply the scenario's deterministic mutation
/// to a COPY, and run the EXISTING verifier (the bundle re-derive byte-compare, or the dream-packet cross-check).
/// Returns whether the mutation changed the bytes, whether it injected the dream's private token, and the real
/// verdict (`Ok` = the clean export verifies; `Err` = a tamper is refused). Never mutates canonical data. Pure.
fn run_dream_export_scenario(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    scenario: DreamExportScenario,
) -> Result<DreamExportAttempt, TraceError> {
    match scenario {
        DreamExportScenario::CleanExport => {
            let bundle = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;
            Ok(DreamExportAttempt {
                mutation_applied: false,
                injects_dream_only: false,
                verdict: verify_dream_export_bundle_json(
                    documents, frame_text, seed, weirdness, &bundle,
                ),
            })
        }
        DreamExportScenario::TamperedDreamPacket => {
            let input = dream_export_input(documents, frame_text, seed, weirdness);
            let valid = dream_engine::dream_packet_json(&input)
                .map_err(|e| TraceError::DreamExport(e.to_string()))?;
            let tampered = valid.replacen("dream-packet-v0.1", "dream-packet-v9.9", 1);
            Ok(DreamExportAttempt {
                mutation_applied: tampered != valid,
                injects_dream_only: false,
                verdict: run_dream_export(documents, frame_text, seed, weirdness, Some(&tampered))
                    .map(|_| ()),
            })
        }
        DreamExportScenario::TamperedReceipt => {
            let bundle = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;
            let tampered = bundle.replacen(
                "\"exported_via_existing_hypothesis_gate\": true",
                "\"exported_via_existing_hypothesis_gate\": false",
                1,
            );
            Ok(DreamExportAttempt {
                mutation_applied: tampered != bundle,
                injects_dream_only: false,
                verdict: verify_dream_export_bundle_json(
                    documents, frame_text, seed, weirdness, &tampered,
                ),
            })
        }
        DreamExportScenario::ForgedDreamOriginFalse => {
            let bundle = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;
            let tampered = bundle.replacen("\"dream_origin\": true", "\"dream_origin\": false", 1);
            Ok(DreamExportAttempt {
                mutation_applied: tampered != bundle,
                injects_dream_only: false,
                verdict: verify_dream_export_bundle_json(
                    documents, frame_text, seed, weirdness, &tampered,
                ),
            })
        }
        DreamExportScenario::MutatedDreamInputHash => {
            let receipt = dream_export_bundle(documents, frame_text, seed, weirdness)?.receipt;
            let bundle = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;
            let from = format!("\"dream_input_hash\": \"{}\"", receipt.dream_input_hash);
            let tampered = bundle.replacen(&from, "\"dream_input_hash\": \"deadbeefdeadbeef\"", 1);
            Ok(DreamExportAttempt {
                mutation_applied: tampered != bundle,
                injects_dream_only: false,
                verdict: verify_dream_export_bundle_json(
                    documents, frame_text, seed, weirdness, &tampered,
                ),
            })
        }
        DreamExportScenario::MutatedDreamPacketId => {
            let receipt = dream_export_bundle(documents, frame_text, seed, weirdness)?.receipt;
            let bundle = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;
            let from = format!("\"dream_packet_id\": \"{}\"", receipt.dream_packet_id);
            let tampered =
                bundle.replacen(&from, "\"dream_packet_id\": \"dream-0000000000000000\"", 1);
            Ok(DreamExportAttempt {
                mutation_applied: tampered != bundle,
                injects_dream_only: false,
                verdict: verify_dream_export_bundle_json(
                    documents, frame_text, seed, weirdness, &tampered,
                ),
            })
        }
        DreamExportScenario::ForgedAuthorityAfterExport => {
            // Forge the EXISTING hypothesis_only authority to the dream engine's private serialized token. The
            // verifier re-derives and refuses it — proving the private dream authority cannot be laundered into an
            // export. The token is only ever present in this FORGED copy; the canonical export never carries it.
            let bundle = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;
            let tampered = bundle.replacen(
                "\"authority_after_export\": \"hypothesis_only\"",
                "\"authority_after_export\": \"dream_only\"",
                1,
            );
            Ok(DreamExportAttempt {
                mutation_applied: tampered != bundle,
                injects_dream_only: tampered.contains("dream_only"),
                verdict: verify_dream_export_bundle_json(
                    documents, frame_text, seed, weirdness, &tampered,
                ),
            })
        }
    }
}

/// One scenario row: the identity, the surface, whether the mutation applied / injected the dream token, the
/// expected vs OBSERVED outcome, whether they match, and the exact detail (rejection reason or clean
/// confirmation). `Serialize` but NOT `Deserialize`. No affirmative-authority token is stored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DreamExportScenarioRow {
    slug: String,
    scenario: String,
    target_surface: String,
    mutation_applied: bool,
    injects_dream_only: bool,
    expected: String,
    outcome: String,
    matches_expected: bool,
    detail: String,
}

/// The dream provenance the export preserves — recorded from the CLEAN receipt so the matrix proves provenance
/// survives export. `authority_after_export` is the EXISTING [`Authority::HypothesisOnly`], never a new variant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DreamExportProvenance {
    dream_packet_id: String,
    dream_input_hash: String,
    dream_seed: u64,
    dream_weirdness: i64,
    dream_engine_version: String,
    dream_operator_ids: Vec<String>,
    source_receipt_memory_hash: u64,
    source_receipt_answer_hash: u64,
    exported_hypothesis_hash: u64,
    authority_after_export: Authority,
    dream_origin: bool,
}

/// The matrix coverage: every outcome matched expectation, the exported material stays hypothesis_only and is
/// distinguishable from a plain hypothesis, probe requests never execute, and the no-execution / no-evidence /
/// no-promotion / no-training boundary cells. All derived from real fields. `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DreamExportMatrixCoverage {
    scenario_count: usize,
    clean_verifies: bool,
    all_tampers_refused: bool,
    all_match_expected: bool,
    exported_material_is_hypothesis_only: bool,
    dream_distinguishable_from_plain: bool,
    probe_requests_execute: bool,
    no_execution: bool,
    no_evidence: bool,
    no_promotion: bool,
    no_training: bool,
    canonical_export_hash: String,
}

/// The dream-export scenario matrix: every scenario row, the preserved dream provenance, the coverage summary,
/// and the boundary. `Serialize` but NOT `Deserialize` — re-derived and byte-compared on verify, never parsed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DreamExportScenarioMatrix {
    schema: String,
    rows: Vec<DreamExportScenarioRow>,
    provenance: DreamExportProvenance,
    coverage: DreamExportMatrixCoverage,
    boundary: Vec<String>,
}

/// Build the canonical dream-export scenario matrix: run every scenario through the EXISTING verifier and record
/// the OBSERVED outcome, then record the preserved provenance and the boundary cells from real fields. Pure and
/// deterministic — every scenario is re-run from fixed inputs; `outcome`/`matches_expected` are observed, not
/// asserted, so a tamper that slipped through records `matches_expected=false` and fails the tests.
fn canonical_dream_export_matrix(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
) -> Result<DreamExportScenarioMatrix, TraceError> {
    // The clean bundle (provenance source of truth) and the source dream packet (probe-execution truth).
    let bundle = dream_export_bundle(documents, frame_text, seed, weirdness)?;
    let receipt = bundle.receipt.clone();
    let input = dream_export_input(documents, frame_text, seed, weirdness);
    let packet =
        dream_engine::dream_packet(&input).map_err(|e| TraceError::DreamExport(e.to_string()))?;
    let bundle_json = dream_export_bundle_json(documents, frame_text, seed, weirdness)?;

    let mut rows = Vec::new();
    let mut clean_verifies = false;
    let mut all_tampers_refused = true;
    let mut all_match_expected = true;
    for scenario in DreamExportScenario::ALL {
        let attempt = run_dream_export_scenario(documents, frame_text, seed, weirdness, scenario)?;
        let verifies = attempt.verdict.is_ok();
        let outcome = if verifies { "verifies" } else { "refused" };
        let expected = if scenario.expected_verifies() {
            "verifies"
        } else {
            "refused"
        };
        let matches_expected = outcome == expected;
        let detail = match &attempt.verdict {
            Ok(()) => "clean export re-derives byte-identically".to_string(),
            Err(e) => e.to_string(),
        };
        if scenario == DreamExportScenario::CleanExport && verifies {
            clean_verifies = true;
        }
        if !scenario.expected_verifies() && verifies {
            all_tampers_refused = false;
        }
        if !matches_expected {
            all_match_expected = false;
        }
        rows.push(DreamExportScenarioRow {
            slug: scenario.slug().to_string(),
            scenario: scenario.describe().to_string(),
            target_surface: scenario.target_surface().to_string(),
            mutation_applied: attempt.mutation_applied,
            injects_dream_only: attempt.injects_dream_only,
            expected: expected.to_string(),
            outcome: outcome.to_string(),
            matches_expected,
            detail,
        });
    }

    // Distinguishability: the dream-exported hypothesis cites a `dream:` provenance label and the bundle records
    // dream_origin; a plain hypothesis cites neither. (Observed, so a regression that made them identical fails.)
    let dream_cites_dream = bundle
        .hypothesis
        .evidence_inputs()
        .iter()
        .all(|e| e.source_label.starts_with("dream:"));
    let plain = propose(HypothesisSpec {
        statement: "Plain proposal with no dream origin.".to_string(),
        prior: 500,
        uncertainty: 500,
        test_cost: 1,
        risk: 100,
        reversibility: 900,
        evidence_inputs: vec![EvidenceRef {
            answer_hash: 1,
            memory_hash: 2,
            source_label: "receipt:plain".to_string(),
        }],
        probe_description: "probe it".to_string(),
    })
    .map_err(TraceError::Hypothesis)?;
    let plain_json = serde_json::to_string_pretty(&plain).expect("plain hypothesis serializes");
    let dream_distinguishable_from_plain = dream_cites_dream
        && plain
            .evidence_inputs()
            .iter()
            .all(|e| !e.source_label.starts_with("dream:"))
        && bundle_json.contains("\"dream_origin\": true")
        && !plain_json.contains("dream_origin");

    // Boundary cells from real fields: probes never execute; the exported material stays hypothesis_only and
    // carries the hypothesis-layer quarantine (forbids serving as evidence / changing the training gate).
    let probe_requests_execute = packet.probe_requests.iter().any(|p| p.executes);
    let exported_material_is_hypothesis_only =
        matches!(receipt.authority_after_export, Authority::HypothesisOnly)
            && !bundle_json.contains("dream_only");
    let no_evidence = receipt
        .forbidden_uses
        .iter()
        .any(|u| u == "serve_as_evidence");
    let no_training = receipt
        .forbidden_uses
        .iter()
        .any(|u| u == "change_training_gate");
    let no_promotion = matches!(receipt.authority_after_export, Authority::HypothesisOnly);

    let provenance = DreamExportProvenance {
        dream_packet_id: receipt.dream_packet_id.clone(),
        dream_input_hash: receipt.dream_input_hash.clone(),
        dream_seed: receipt.dream_seed,
        dream_weirdness: receipt.dream_weirdness,
        dream_engine_version: receipt.dream_engine_version.clone(),
        dream_operator_ids: receipt.dream_operator_ids.clone(),
        source_receipt_memory_hash: receipt.source_receipt_memory_hash,
        source_receipt_answer_hash: receipt.source_receipt_answer_hash,
        exported_hypothesis_hash: receipt.exported_hypothesis_hash,
        authority_after_export: receipt.authority_after_export,
        dream_origin: receipt.dream_origin,
    };
    let coverage = DreamExportMatrixCoverage {
        scenario_count: rows.len(),
        clean_verifies,
        all_tampers_refused,
        all_match_expected,
        exported_material_is_hypothesis_only,
        dream_distinguishable_from_plain,
        probe_requests_execute,
        no_execution: !probe_requests_execute,
        no_evidence,
        no_promotion,
        no_training,
        canonical_export_hash: bundle_content_hash(&bundle_json),
    };
    Ok(DreamExportScenarioMatrix {
        schema: "dream-export-scenario-matrix-v0.1".to_string(),
        rows,
        provenance,
        coverage,
        boundary: DREAM_EXPORT_MATRIX_BOUNDARY_LINES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    })
}

/// The dream-export scenario matrix as pretty JSON. Pure and deterministic — re-derives byte-for-byte from the
/// same corpus + frame + dials. This is what `dream-export-matrix --out` writes.
pub fn dream_export_matrix(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
) -> Result<String, TraceError> {
    Ok(serde_json::to_string_pretty(&canonical_dream_export_matrix(
        documents, frame_text, seed, weirdness,
    )?)
    .expect("DreamExportScenarioMatrix serializes"))
}

/// Re-derive the matrix from the SAME corpus + frame + dials and confirm the PROVIDED matrix JSON is byte-for-byte
/// that matrix. The provided matrix is NEVER parsed back into authority — only COMPARED — so a tampered / stale /
/// foreign matrix (e.g. one that flips a refused outcome to verifies) is REFUSED. Pure.
pub fn verify_dream_export_matrix(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided: &str,
) -> Result<(), TraceError> {
    if provided == dream_export_matrix(documents, frame_text, seed, weirdness)? {
        Ok(())
    } else {
        Err(TraceError::DreamExportMismatch)
    }
}

/// Render the plain-text dream-export scenario-matrix report — pure FORMATTING of its recorded fields: the
/// per-scenario expected/outcome verdict, the preserved provenance, the coverage cells, and the boundary. No new
/// verdict, no authority object.
fn render_dream_export_matrix(matrix: &DreamExportScenarioMatrix) -> String {
    let mut out = String::from("DREAM EXPORT SCENARIO MATRIX (PROVENANCE INTEGRITY)\n");
    out.push_str(&format!("schema: {}\n", matrix.schema));
    out.push_str(
        "(one clean export VERIFIES; every tamper is REFUSED by re-derive byte-compare)\n\n",
    );

    out.push_str("PER-SCENARIO EXPECTED x OUTCOME\n");
    for row in &matrix.rows {
        out.push_str(&format!("[{}]\n", row.slug));
        out.push_str(&format!("    scenario:         {}\n", row.scenario));
        out.push_str(&format!("    surface:          {}\n", row.target_surface));
        out.push_str(&format!("    mutation applied: {}\n", row.mutation_applied));
        out.push_str(&format!("    expected:         {}\n", row.expected));
        out.push_str(&format!("    outcome:          {}\n", row.outcome));
        out.push_str(&format!("    matches expected: {}\n", row.matches_expected));
        out.push_str(&format!("    detail:           {}\n", row.detail));
    }

    out.push_str("\nDREAM PROVENANCE (preserved across export, auditable)\n");
    out.push_str(&format!(
        "    dream_packet_id:        {}\n",
        matrix.provenance.dream_packet_id
    ));
    out.push_str(&format!(
        "    dream_input_hash:       {}\n",
        matrix.provenance.dream_input_hash
    ));
    out.push_str(&format!(
        "    dream_seed:             {}\n",
        matrix.provenance.dream_seed
    ));
    out.push_str(&format!(
        "    dream_engine_version:   {}\n",
        matrix.provenance.dream_engine_version
    ));
    out.push_str(&format!(
        "    exported_hypothesis:    {}\n",
        matrix.provenance.exported_hypothesis_hash
    ));
    out.push_str("    authority_after_export: hypothesis_only\n");
    out.push_str(&format!(
        "    dream_origin:           {}\n",
        matrix.provenance.dream_origin
    ));

    out.push_str("\nCOVERAGE\n");
    out.push_str(&format!(
        "    scenarios:                       {}\n",
        matrix.coverage.scenario_count
    ));
    out.push_str(&format!(
        "    clean verifies:                  {}\n",
        matrix.coverage.clean_verifies
    ));
    out.push_str(&format!(
        "    all tampers refused:             {}\n",
        matrix.coverage.all_tampers_refused
    ));
    out.push_str(&format!(
        "    all match expected:              {}\n",
        matrix.coverage.all_match_expected
    ));
    out.push_str(&format!(
        "    exported material hypothesis_only:{}\n",
        matrix.coverage.exported_material_is_hypothesis_only
    ));
    out.push_str(&format!(
        "    dream distinguishable from plain:{}\n",
        matrix.coverage.dream_distinguishable_from_plain
    ));
    out.push_str(&format!(
        "    probe requests execute:          {}\n",
        matrix.coverage.probe_requests_execute
    ));
    out.push_str(&format!(
        "    no_execution:                    {}\n",
        matrix.coverage.no_execution
    ));
    out.push_str(&format!(
        "    no_evidence:                     {}\n",
        matrix.coverage.no_evidence
    ));
    out.push_str(&format!(
        "    no_promotion:                    {}\n",
        matrix.coverage.no_promotion
    ));
    out.push_str(&format!(
        "    no_training:                     {}\n",
        matrix.coverage.no_training
    ));
    out.push_str(&format!(
        "    canonical export hash:           {}\n",
        matrix.coverage.canonical_export_hash
    ));

    out.push_str("\nBOUNDARY\n");
    for line in DREAM_EXPORT_MATRIX_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The `dream-export-matrix-report` command body: re-derive + verify the matrix from the SAME corpus + frame +
/// dials (refuse a tampered matrix), then render the report from the re-derived (trusted) matrix. Pure (no I/O).
pub fn run_dream_export_matrix_report(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided_matrix: &str,
) -> Result<String, TraceError> {
    verify_dream_export_matrix(documents, frame_text, seed, weirdness, provided_matrix)?;
    Ok(render_dream_export_matrix(&canonical_dream_export_matrix(
        documents, frame_text, seed, weirdness,
    )?))
}

/// The `dream-export-matrix-verify` command body: re-derive the matrix and confirm the provided matrix is
/// byte-identical — a determinism proof that also refuses any tampered matrix. Reads nothing as authority. Pure.
pub fn run_dream_export_matrix_verify(
    documents: &[(String, String)],
    frame_text: &str,
    seed: u64,
    weirdness: i64,
    provided_matrix: &str,
) -> Result<String, TraceError> {
    verify_dream_export_matrix(documents, frame_text, seed, weirdness, provided_matrix)?;
    Ok(String::from(
        "dream-export-matrix-verify: OK — the scenario matrix re-derives byte-identically; the clean export verifies and every tamper stays refused. Dream provenance is preserved; the exported material is hypothesis_only.\n",
    ))
}

/// The `dream-export-scenarios` command: list the finite dream-export scenario set (slug + one-line description).
/// Pure.
pub fn list_dream_export_scenarios() -> String {
    let mut out = String::from(
        "cognitive-demo — dream-export scenarios (one clean export verifies; every tamper is refused):\n",
    );
    for s in DreamExportScenario::ALL {
        out.push_str(&format!("    {:<30} {}\n", s.slug(), s.describe()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- DREAM-EXPORT-0 fixtures + tests ---

    /// A two-document corpus + frame the dream engine grounds and distorts (one cross-document pair), with the
    /// canonical seed/weirdness. Mirrors dream-engine's own fixture so the bridge exercises a real dream.
    fn dream_export_fixture() -> (Vec<(String, String)>, String, u64, i64) {
        let documents = vec![
            (
                "bridge_report".to_string(),
                "Bridge A was reported structurally damaged after the June storm. Inspectors advised against using Bridge A until repairs are complete.".to_string(),
            ),
            (
                "weather_log".to_string(),
                "The June storm brought heavy rain and high winds overnight. Bridge B remained passable during light rain.".to_string(),
            ),
        ];
        let frame =
            "Documents are passive inputs.\nSource selection is mere retrieval.".to_string();
        (documents, frame, 42, 2)
    }

    #[test]
    fn dream_export_builds_from_verified_corpus() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("builds");
        assert_eq!(bundle.schema, "dream-export-bundle-v0.1");
        assert_eq!(bundle.receipt.schema, "dream-export-receipt-v0.1");
        // The exported material is a real hypothesis carrying the EXISTING hypothesis-only authority.
        assert_eq!(bundle.hypothesis.authority(), Authority::HypothesisOnly);
    }

    #[test]
    fn dream_export_receipt_preserves_dream_provenance() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("builds");
        // Re-derive the dream packet independently and confirm the receipt names exactly it.
        let input = dream_export_input(&docs, &frame, seed, w);
        let packet = dream_engine::dream_packet(&input).expect("packet");
        assert_eq!(bundle.receipt.dream_packet_id, packet.packet_id);
        assert_eq!(bundle.receipt.dream_input_hash, packet.dream_input_hash);
        assert_eq!(bundle.receipt.dream_seed, packet.seed);
        assert_eq!(bundle.receipt.dream_weirdness, packet.weirdness);
        assert_eq!(
            bundle.receipt.source_receipt_memory_hash,
            packet.source_receipt_memory_hash
        );
        assert!(!bundle.receipt.dream_operator_ids.is_empty());
    }

    #[test]
    fn dream_export_receipt_records_dream_origin_true() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("builds");
        assert!(bundle.receipt.dream_origin);
    }

    #[test]
    fn dream_export_authority_after_export_is_hypothesis_only() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("builds");
        // The receipt's authority is the EXISTING enum, read off the proposed packet — not a new variant.
        assert_eq!(
            bundle.receipt.authority_after_export,
            Authority::HypothesisOnly
        );
        assert_eq!(
            bundle.receipt.authority_after_export,
            bundle.hypothesis.authority()
        );
    }

    #[test]
    fn dream_export_uses_existing_hypothesis_gate() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("builds");
        assert!(bundle.receipt.exported_via_existing_hypothesis_gate);
        // The exported hypothesis carries the canonical hypothesis-layer forbidden-uses — proof it went through
        // the real `propose` path, not a hand-built impostor. It can never serve as evidence or ground a claim.
        assert!(!bundle.hypothesis.permits("serve_as_evidence"));
        assert!(!bundle.hypothesis.permits("ground_claim"));
        assert!(!bundle.hypothesis.permits("change_training_gate"));
        assert_eq!(
            bundle.receipt.forbidden_uses,
            bundle.hypothesis.forbidden_uses()
        );
    }

    #[test]
    fn dream_export_carries_no_dream_authority() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let json = dream_export_bundle_json(&docs, &frame, seed, w).expect("json");
        // The dream's private `dream_only` authority NEVER crosses the boundary; only hypothesis_only does.
        assert!(!json.contains("dream_only"));
        assert!(json.contains("hypothesis_only"));
    }

    #[test]
    fn dream_export_probe_requests_do_not_execute() {
        let (docs, frame, seed, w) = dream_export_fixture();
        // The source dream the export descends from carries only non-executing probe requests.
        let input = dream_export_input(&docs, &frame, seed, w);
        let packet = dream_engine::dream_packet(&input).expect("packet");
        assert!(!packet.probe_requests.is_empty());
        assert!(packet.probe_requests.iter().all(|p| !p.executes));
    }

    #[test]
    fn dream_export_refuses_tampered_dream_packet() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let input = dream_export_input(&docs, &frame, seed, w);
        let valid = dream_engine::dream_packet_json(&input).expect("packet json");
        // A byte-identical valid packet is accepted (and produces a bundle).
        run_dream_export(&docs, &frame, seed, w, Some(&valid)).expect("valid packet exports");
        // A tampered packet is refused — it is never laundered into an export.
        let tampered = valid.replacen("dream-packet-v0.1", "dream-packet-v9.9", 1);
        assert!(matches!(
            run_dream_export(&docs, &frame, seed, w, Some(&tampered)),
            Err(TraceError::DreamExport(_))
        ));
    }

    #[test]
    fn dream_export_replay_byte_identical() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let a = dream_export_bundle_json(&docs, &frame, seed, w).expect("a");
        let b = dream_export_bundle_json(&docs, &frame, seed, w).expect("b");
        assert_eq!(a, b);
        verify_dream_export_bundle_json(&docs, &frame, seed, w, &a).expect("verifies");
        run_dream_export_report(&docs, &frame, seed, w, &a).expect("report");
        run_dream_export_replay(&docs, &frame, seed, w, &a).expect("replay");
    }

    #[test]
    fn dream_export_tampered_bundle_refused() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle_json(&docs, &frame, seed, w).expect("bundle");
        // Flip the dream_origin flag in the serialized bundle — re-derivation no longer matches, so it is refused.
        let tampered = bundle.replacen("\"dream_origin\": true", "\"dream_origin\": false", 1);
        assert_ne!(tampered, bundle);
        assert!(matches!(
            run_dream_export_report(&docs, &frame, seed, w, &tampered),
            Err(TraceError::DreamExportMismatch)
        ));
        assert!(matches!(
            run_dream_export_replay(&docs, &frame, seed, w, &tampered),
            Err(TraceError::DreamExportMismatch)
        ));
    }

    #[test]
    fn plain_and_dream_hypothesis_distinguishable() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("builds");
        // A dream-exported hypothesis cites a `dream:` provenance label; an ordinary one does not.
        assert!(bundle
            .hypothesis
            .evidence_inputs()
            .iter()
            .all(|e| e.source_label.starts_with("dream:")));
        let plain = propose(HypothesisSpec {
            statement: "Plain proposal with no dream origin.".to_string(),
            prior: 500,
            uncertainty: 500,
            test_cost: 1,
            risk: 100,
            reversibility: 900,
            evidence_inputs: vec![EvidenceRef {
                answer_hash: 1,
                memory_hash: 2,
                source_label: "receipt:plain".to_string(),
            }],
            probe_description: "probe it".to_string(),
        })
        .expect("plain hypothesis");
        assert!(plain
            .evidence_inputs()
            .iter()
            .all(|e| !e.source_label.starts_with("dream:")));
        // The export bundle records dream origin; a bare hypothesis JSON does not.
        let bundle_json = dream_export_bundle_json(&docs, &frame, seed, w).expect("bundle json");
        let plain_json = serde_json::to_string_pretty(&plain).expect("plain json");
        assert!(bundle_json.contains("\"dream_origin\": true"));
        assert!(!plain_json.contains("dream_origin"));
    }

    #[test]
    fn dream_export_report_shows_provenance() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle_json(&docs, &frame, seed, w).expect("bundle");
        let report = run_dream_export_report(&docs, &frame, seed, w, &bundle).expect("report");
        assert!(report.contains("dream_origin:"));
        assert!(report.contains("authority_after_export: hypothesis_only"));
        assert!(report.contains("dream_packet_id:"));
        assert!(report.contains("dream:")); // the auditable provenance label
        assert!(report.contains("Dream origin remains auditable."));
    }

    #[test]
    fn dream_export_refuses_unverifiable_corpus() {
        // A single-document corpus yields no cross-document pair, so the dream is degenerate and the export
        // fails closed — an export REQUIRES a valid re-derived dream packet.
        let docs = vec![(
            "only".to_string(),
            "Bridge A was damaged. Bridge B stayed open.".to_string(),
        )];
        let frame = "Documents are passive inputs.".to_string();
        assert!(matches!(
            dream_export_bundle(&docs, &frame, 42, 2),
            Err(TraceError::DreamExport(_))
        ));
    }

    // --- DREAM-EXPORT-2 scenario matrix / provenance integrity tests ---

    #[test]
    fn dream_export_matrix_lists_all_scenarios() {
        let listed = list_dream_export_scenarios();
        for s in DreamExportScenario::ALL {
            assert!(listed.contains(s.slug()), "missing scenario: {}", s.slug());
        }
        assert_eq!(DreamExportScenario::ALL.len(), 7);
    }

    #[test]
    fn dream_export_matrix_clean_verifies() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        let clean = m
            .rows
            .iter()
            .find(|r| r.slug == "clean-export")
            .expect("clean row");
        assert_eq!(clean.outcome, "verifies");
        assert_eq!(clean.expected, "verifies");
        assert!(clean.matches_expected);
        assert!(m.coverage.clean_verifies);
    }

    #[test]
    fn dream_export_matrix_all_tampers_refused() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        for row in m.rows.iter().filter(|r| r.slug != "clean-export") {
            assert_eq!(row.outcome, "refused", "{} was not refused", row.slug);
            assert_eq!(row.expected, "refused");
            assert!(row.matches_expected);
        }
        assert!(m.coverage.all_tampers_refused);
    }

    #[test]
    fn dream_export_matrix_all_match_expected() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        assert!(m.coverage.all_match_expected);
        assert!(m.rows.iter().all(|r| r.matches_expected));
        assert_eq!(m.coverage.scenario_count, 7);
    }

    #[test]
    fn dream_export_matrix_records_dream_provenance() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let bundle = dream_export_bundle(&docs, &frame, seed, w).expect("bundle");
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        // The matrix records the SAME provenance the clean receipt carries — provenance SURVIVES export.
        assert_eq!(m.provenance.dream_packet_id, bundle.receipt.dream_packet_id);
        assert_eq!(
            m.provenance.dream_input_hash,
            bundle.receipt.dream_input_hash
        );
        assert_eq!(m.provenance.dream_seed, bundle.receipt.dream_seed);
        assert_eq!(
            m.provenance.exported_hypothesis_hash,
            bundle.receipt.exported_hypothesis_hash
        );
        assert!(m.provenance.dream_origin);
        assert!(!m.provenance.dream_operator_ids.is_empty());
    }

    #[test]
    fn dream_export_matrix_authority_remains_hypothesis_only() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        assert!(matches!(
            m.provenance.authority_after_export,
            Authority::HypothesisOnly
        ));
        assert!(m.coverage.exported_material_is_hypothesis_only);
        // The clean export bundle the matrix is built over never carries the dream's private token.
        let bundle_json = dream_export_bundle_json(&docs, &frame, seed, w).expect("bundle");
        assert!(!bundle_json.contains("dream_only"));
        assert!(bundle_json.contains("hypothesis_only"));
    }

    #[test]
    fn dream_export_matrix_distinguishes_plain_from_dream() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        assert!(m.coverage.dream_distinguishable_from_plain);
    }

    #[test]
    fn dream_export_matrix_probe_requests_do_not_execute() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        assert!(!m.coverage.probe_requests_execute);
        assert!(m.coverage.no_execution);
    }

    #[test]
    fn dream_export_matrix_records_no_evidence_promotion_training() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        assert!(m.coverage.no_evidence);
        assert!(m.coverage.no_promotion);
        assert!(m.coverage.no_training);
    }

    #[test]
    fn dream_export_matrix_replay_byte_identical() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let a = dream_export_matrix(&docs, &frame, seed, w).expect("a");
        let b = dream_export_matrix(&docs, &frame, seed, w).expect("b");
        assert_eq!(a, b);
        verify_dream_export_matrix(&docs, &frame, seed, w, &a).expect("verifies");
        run_dream_export_matrix_report(&docs, &frame, seed, w, &a).expect("report");
        run_dream_export_matrix_verify(&docs, &frame, seed, w, &a).expect("verify");
    }

    #[test]
    fn dream_export_matrix_verify_rejects_tampered_matrix() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let matrix = dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        // Flip a refused outcome to "verifies" to claim a tamper passed — re-derivation refuses the doctored matrix.
        let doctored = matrix.replacen("\"outcome\": \"refused\"", "\"outcome\": \"verifies\"", 1);
        assert_ne!(doctored, matrix);
        assert!(matches!(
            run_dream_export_matrix_report(&docs, &frame, seed, w, &doctored),
            Err(TraceError::DreamExportMismatch)
        ));
        assert!(matches!(
            run_dream_export_matrix_verify(&docs, &frame, seed, w, &doctored),
            Err(TraceError::DreamExportMismatch)
        ));
    }

    #[test]
    fn dream_export_matrix_authority_forgery_injects_dream_token_and_is_refused() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        let row = m
            .rows
            .iter()
            .find(|r| r.slug == "forged-authority-after-export")
            .expect("authority-forgery row");
        // The forgery genuinely injected the dream's private serialized token AND was refused.
        assert!(row.injects_dream_only);
        assert!(row.mutation_applied);
        assert_eq!(row.outcome, "refused");
        assert!(row.matches_expected);
    }

    #[test]
    fn dream_export_matrix_report_shows_provenance_and_outcomes() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let matrix = dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        let report =
            run_dream_export_matrix_report(&docs, &frame, seed, w, &matrix).expect("report");
        assert!(report.contains("DREAM EXPORT SCENARIO MATRIX"));
        assert!(report.contains("clean-export"));
        assert!(report.contains("verifies"));
        assert!(report.contains("refused"));
        assert!(report.contains("dream_packet_id:"));
        assert!(report.contains("authority_after_export: hypothesis_only"));
        assert!(report.contains("no_evidence:"));
        assert!(report.contains("Dream provenance remains auditable."));
    }

    #[test]
    fn dream_export_matrix_tampers_actually_mutate() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let m = canonical_dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        // Every tamper scenario must genuinely alter the canonical bytes — a no-op mutation cannot masquerade as a
        // refusal proof. (The clean scenario applies no mutation, so it is excluded.)
        for row in m.rows.iter().filter(|r| r.slug != "clean-export") {
            assert!(row.mutation_applied, "{} did not mutate", row.slug);
        }
    }

    #[test]
    fn dream_export_matrix_does_not_change_training_gate() {
        let (docs, frame, seed, w) = dream_export_fixture();
        let before = CognitiveTrace::demo().expect("trace").to_json();
        let _ = dream_export_matrix(&docs, &frame, seed, w).expect("matrix");
        let after = CognitiveTrace::demo().expect("trace").to_json();
        // Building the matrix is pure — it cannot open training; the P12 verdict is unchanged and stays false.
        assert_eq!(before, after);
        assert!(before.contains("\"training_justified\": false"));
    }

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

    // --- INT-3: the operator repro bundle (`bundle` + `bundle-verify`). The pure cores
    //     (canonical_bundle / verify_bundle) are tested here; the binary fs I/O is gated by the
    //     release_check INT-3 smoke. ---

    /// The canonical bundle as owned (name, content) pairs — the shape `verify_bundle` consumes.
    fn canonical_bundle_owned() -> Vec<(String, String)> {
        canonical_bundle()
            .unwrap()
            .into_iter()
            .map(|(name, content)| (name.to_string(), content))
            .collect()
    }

    #[test]
    fn bundle_command_writes_all_expected_files() {
        // The canonical bundle is exactly the four expected files, in write order, each non-empty, and
        // the content files are the REAL canonical artifacts (trace / report / questions transcript).
        let files = canonical_bundle().unwrap();
        let names: Vec<&str> = files.iter().map(|(name, _)| *name).collect();
        assert_eq!(names, BUNDLE_FILES.to_vec());
        for (name, content) in &files {
            assert!(!content.is_empty(), "{name} must have content");
        }
        let trace_json = run_trace().unwrap();
        assert_eq!(files[0].1, trace_json);
        assert_eq!(files[1].1, run_report(&trace_json).unwrap());
        assert_eq!(files[2].1, run_questions_doc().unwrap());
    }

    #[test]
    fn bundle_manifest_hashes_all_files() {
        // The manifest names AND records the exact content hash of every CONTENT file (not itself).
        let files = canonical_bundle().unwrap();
        let manifest = files
            .iter()
            .find(|(name, _)| *name == BUNDLE_MANIFEST_FILE)
            .unwrap()
            .1
            .clone();
        for (name, content) in files
            .iter()
            .filter(|(name, _)| *name != BUNDLE_MANIFEST_FILE)
        {
            assert!(manifest.contains(name), "manifest must name {name}");
            assert!(
                manifest.contains(&bundle_content_hash(content)),
                "manifest must record the content hash of {name}"
            );
        }
        // The manifest does NOT hash itself (no fixpoint) — its own name is absent from the file list.
        assert!(!manifest.contains("\"name\": \"manifest.json\""));
    }

    #[test]
    fn bundle_verify_rejects_tampered_trace() {
        // A tampered trace.json fails re-derivation byte-comparison (the manifest is untouched, so the
        // trace itself is caught first).
        let mut b = canonical_bundle_owned();
        let i = b
            .iter()
            .position(|(name, _)| name == BUNDLE_TRACE_FILE)
            .unwrap();
        b[i].1 = b[i]
            .1
            .replace("\"grants_promotion\": false", "\"grants_promotion\": true");
        assert!(
            matches!(verify_bundle(&b), Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_TRACE_FILE)
        );
    }

    #[test]
    fn bundle_verify_rejects_tampered_report() {
        // A tampered report.txt is refused — bundle prose cannot become authority by editing the file.
        let mut b = canonical_bundle_owned();
        let i = b
            .iter()
            .position(|(name, _)| name == BUNDLE_REPORT_FILE)
            .unwrap();
        b[i].1.push_str("\nINJECTED: promotion granted.\n");
        assert!(
            matches!(verify_bundle(&b), Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_REPORT_FILE)
        );
    }

    #[test]
    fn bundle_verify_rejects_tampered_questions() {
        // A tampered questions.txt is refused (here the promotion answer is flipped to claim it occurred).
        let mut b = canonical_bundle_owned();
        let i = b
            .iter()
            .position(|(name, _)| name == BUNDLE_QUESTIONS_FILE)
            .unwrap();
        b[i].1 = b[i].1.replace("did not occur", "DID occur");
        assert!(
            matches!(verify_bundle(&b), Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_QUESTIONS_FILE)
        );
    }

    #[test]
    fn bundle_verify_rejects_tampered_manifest() {
        // The manifest is itself re-derived and byte-compared — editing it (here the schema) is refused,
        // so a forged manifest can never vouch for a forged bundle.
        let mut b = canonical_bundle_owned();
        let i = b
            .iter()
            .position(|(name, _)| name == BUNDLE_MANIFEST_FILE)
            .unwrap();
        b[i].1 = b[i]
            .1
            .replace("cognitive-bundle-v0.1", "cognitive-bundle-v9.9");
        assert!(
            matches!(verify_bundle(&b), Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_MANIFEST_FILE)
        );
    }

    #[test]
    fn bundle_verify_rejects_missing_file() {
        // A bundle missing a required file is refused (BundleMissingFile), never silently accepted.
        let mut b = canonical_bundle_owned();
        b.retain(|(name, _)| name != BUNDLE_QUESTIONS_FILE);
        assert!(
            matches!(verify_bundle(&b), Err(TraceError::BundleMissingFile(ref f)) if f == BUNDLE_QUESTIONS_FILE)
        );
    }

    #[test]
    fn bundle_verify_rederives_canonical_trace() {
        // Verification is by RE-DERIVATION, not trust: the pristine canonical bundle passes, the
        // bundle's trace.json equals the freshly re-derived canonical trace, and a bundle with the
        // canonical file NAMES but foreign content is refused.
        let b = canonical_bundle_owned();
        assert!(verify_bundle(&b).is_ok());
        let trace = b
            .iter()
            .find(|(name, _)| name == BUNDLE_TRACE_FILE)
            .unwrap()
            .1
            .clone();
        assert_eq!(trace, run_trace().unwrap());
        let foreign: Vec<(String, String)> = BUNDLE_FILES
            .iter()
            .map(|name| (name.to_string(), "foreign".to_string()))
            .collect();
        assert!(verify_bundle(&foreign).is_err());
    }

    #[test]
    fn bundle_does_not_change_training_gate() {
        // Building AND verifying the bundle is orthogonal to P12: the training decision is unmoved, and
        // the bundle manifest itself states training does not open.
        let before = decide(&[], &[]);
        let b = canonical_bundle_owned();
        verify_bundle(&b).unwrap();
        let after = decide(&[], &[]);
        assert_eq!(before, after);
        assert!(!after.training_justified);
        let manifest = b
            .iter()
            .find(|(name, _)| name == BUNDLE_MANIFEST_FILE)
            .unwrap()
            .1
            .clone();
        assert!(manifest.contains("It does not train."));
    }

    #[test]
    fn bundle_does_not_change_verifier_receipt() {
        // Building/verifying the bundle leaves the reading receipt byte-identical — the bundle reads
        // hashes and re-derives, it never mutates the verifier or executes anything.
        let (d, q, p) = demo_inputs();
        let file = produce_run(&d, &q, &p).unwrap();
        let before = verify_file(&file).unwrap();
        let b = canonical_bundle_owned();
        verify_bundle(&b).unwrap();
        let after = verify_file(&file).unwrap();
        assert_eq!(before, after, "the verifier receipt is unchanged");
        assert!(after.receipt.passed);
    }

    #[test]
    fn bundle_boundary_lines_present() {
        // The bundle carries the six-line INT-3 boundary verbatim (in the manifest), and the const is
        // pinned line-for-line.
        let b = canonical_bundle().unwrap();
        let manifest = b
            .iter()
            .find(|(name, _)| *name == BUNDLE_MANIFEST_FILE)
            .unwrap()
            .1
            .clone();
        for line in BUNDLE_BOUNDARY_LINES {
            assert!(
                manifest.contains(line),
                "bundle must contain boundary: {line}"
            );
        }
        assert_eq!(BUNDLE_BOUNDARY_LINES.len(), 6);
        assert_eq!(
            BUNDLE_BOUNDARY_LINES[0],
            "The bundle demonstrates the prototype."
        );
        assert_eq!(BUNDLE_BOUNDARY_LINES[1], "It does not create evidence.");
        assert_eq!(BUNDLE_BOUNDARY_LINES[2], "It does not create authority.");
        assert_eq!(BUNDLE_BOUNDARY_LINES[3], "It does not execute.");
        assert_eq!(BUNDLE_BOUNDARY_LINES[4], "It does not promote.");
        assert_eq!(BUNDLE_BOUNDARY_LINES[5], "It does not train.");
    }

    #[test]
    fn bundle_output_is_not_authority() {
        // No bundle file shows an affirmative executed/promoted/granted status or a true grant; the
        // bundle DEMONSTRATES, creating no authority and no evidence, and records training stays false.
        let b = canonical_bundle().unwrap();
        for (name, content) in &b {
            assert!(
                !content.contains("\"execution_status\": \"executed\""),
                "{name} must not show an executed status"
            );
            assert!(
                !content.contains("\"promotion_status\": \"promoted\""),
                "{name} must not show a promoted status"
            );
            assert!(
                !content.contains("\"observation_status\": \"recorded\""),
                "{name} must not show a recorded observation"
            );
            assert!(
                !content.contains("\"grants_promotion\": true"),
                "{name} must not grant a promotion"
            );
        }
        let trace = b
            .iter()
            .find(|(name, _)| *name == BUNDLE_TRACE_FILE)
            .unwrap()
            .1
            .clone();
        assert!(trace.contains("\"training_justified\": false"));
    }

    // --- MTRACE-0: the multi-trace scenario pack. Each scenario varies the path (review/observation/
    //     promotion outcome) but NOT the authority boundary. The binary fs I/O is gated by the
    //     release_check MTRACE-0 smoke. ---

    /// A scenario bundle as owned (name, content) pairs — the shape `verify_scenario_bundle` consumes.
    fn scenario_bundle_owned(scenario: Scenario) -> Vec<(String, String)> {
        scenario_bundle(scenario)
            .unwrap()
            .into_iter()
            .map(|(name, content)| (name.to_string(), content))
            .collect()
    }

    #[test]
    fn happy_boundary_scenario_equals_canonical_demo() {
        // The happy-boundary scenario IS the frozen canonical demo trace, byte-for-byte — the refactor
        // (parameterizing the builder) preserved the integration-demo-v0.1 trace exactly.
        let happy = scenario_trace(Scenario::HappyBoundary).unwrap();
        let demo = CognitiveTrace::demo().unwrap();
        assert_eq!(happy, demo);
        assert_eq!(happy.to_json(), demo.to_json());
        assert_eq!(happy.execution_status(), "requires_operator");
        assert_eq!(happy.observation_status(), "requires_review");
    }

    #[test]
    fn scenario_pack_lists_all_scenarios() {
        // The scenario set is finite and enum-backed; the listing and the pack manifest name every
        // scenario, each slug round-trips, and the set is closed (a near-miss is not accepted).
        assert_eq!(Scenario::ALL.len(), 4);
        let listing = list_scenarios();
        let manifest = scenario_pack_manifest().unwrap();
        for s in Scenario::ALL {
            assert!(listing.contains(s.slug()), "listing must name {}", s.slug());
            assert!(
                manifest.contains(s.slug()),
                "pack manifest must name {}",
                s.slug()
            );
            assert_eq!(Scenario::from_slug(s.slug()), Some(s));
        }
        assert_eq!(Scenario::from_slug("happy"), None);
        assert_eq!(Scenario::from_slug(""), None);
        // The pack manifest carries the six MTRACE-0 boundary lines verbatim.
        for line in MTRACE_BOUNDARY_LINES {
            assert!(
                manifest.contains(line),
                "pack manifest must contain boundary: {line}"
            );
        }
    }

    #[test]
    fn each_scenario_replays() {
        // Every scenario trace is a pure function of fixed inputs: building it twice yields an identical
        // record AND identical serialized bytes (replay).
        for s in Scenario::ALL {
            let a = scenario_trace(s).unwrap();
            let b = scenario_trace(s).unwrap();
            assert_eq!(a, b, "{} replays to an identical record", s.slug());
            assert_eq!(
                a.to_json(),
                b.to_json(),
                "{} replays byte-identically",
                s.slug()
            );
        }
    }

    #[test]
    fn each_scenario_bundle_verifies() {
        // Every scenario's pristine bundle verifies by re-derivation (it is not trusted — it matches the
        // freshly re-derived canonical scenario bundle).
        for s in Scenario::ALL {
            let bundle = scenario_bundle_owned(s);
            assert!(
                verify_scenario_bundle(s, &bundle).is_ok(),
                "{} bundle must verify",
                s.slug()
            );
            // A scenario bundle is NOT accepted as a DIFFERENT scenario's bundle (the paths differ).
            for other in Scenario::ALL {
                if other != s {
                    assert!(
                        verify_scenario_bundle(other, &bundle).is_err(),
                        "{} bundle must not verify as {}",
                        s.slug(),
                        other.slug()
                    );
                }
            }
        }
    }

    #[test]
    fn review_rejected_scenario_blocks_intent() {
        // A rejected review yields a BLOCKED (never executable) intent — a rejected review can never
        // produce executable intent.
        let trace = scenario_trace(Scenario::ReviewRejected).unwrap();
        assert_eq!(trace.review_decision(), "rejected");
        assert_eq!(trace.execution_status(), "blocked");
        assert_ne!(trace.execution_status(), "executed");
        assert!(trace.nothing_executed());
        assert_eq!(trace.observation_status(), "rejected");
    }

    #[test]
    fn review_deferred_scenario_blocks_intent() {
        // A deferred review likewise yields a BLOCKED intent — deferral is not execution.
        let trace = scenario_trace(Scenario::ReviewDeferred).unwrap();
        assert_eq!(trace.review_decision(), "deferred");
        assert_eq!(trace.execution_status(), "blocked");
        assert_ne!(trace.execution_status(), "executed");
        assert!(trace.nothing_executed());
        assert_eq!(trace.observation_status(), "rejected");
    }

    #[test]
    fn high_risk_scenario_blocks_probe() {
        // A high-risk AND irreversible probe is classified BLOCKED and has NO approval path: the frozen
        // layer refuses to approve a blocked probe for ANY authority, so nothing can execute.
        let trace = scenario_trace(Scenario::HighRiskBlocked).unwrap();
        assert_eq!(trace.probe_status, "blocked");
        assert_eq!(trace.execution_status(), "blocked");
        assert!(trace.nothing_executed());
        // No approval path: rebuilding the same blocked probe, approving it is refused.
        let (d, q, p) = demo_inputs();
        let file = produce_run(&d, &q, &p).unwrap();
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
            risk: 800,
            reversibility: 100,
            evidence_inputs: vec![cite],
            probe_description: "Re-read the maintenance log span for Bridge B.".to_string(),
        };
        let packet = propose(spec).unwrap();
        let probe = ProbeRequest::from_hypothesis(&packet);
        assert!(
            ReviewReceipt::decide(
                &probe,
                ReviewerAuthority::Governance,
                ReviewDecision::Approved
            )
            .is_err(),
            "a blocked probe must have no approval path"
        );
    }

    #[test]
    fn no_scenario_executes() {
        // Across EVERY scenario, nothing executes: the execution status is never `executed` and the
        // nothing_executed verdict holds.
        for s in Scenario::ALL {
            let trace = scenario_trace(s).unwrap();
            assert!(trace.nothing_executed(), "{} executes nothing", s.slug());
            assert_ne!(trace.execution_status(), "executed", "{}", s.slug());
        }
    }

    #[test]
    fn no_scenario_promotes() {
        // Across EVERY scenario, nothing is promoted and nothing becomes evidence: the promotion is
        // rejected and grants nothing, regardless of the path.
        for s in Scenario::ALL {
            let trace = scenario_trace(s).unwrap();
            assert!(
                !trace.grants_promotion(),
                "{} grants no promotion",
                s.slug()
            );
            assert_eq!(trace.promotion_status(), "rejected", "{}", s.slug());
            assert!(trace.nothing_becomes_evidence(), "{}", s.slug());
        }
    }

    #[test]
    fn no_scenario_changes_training_gate() {
        // Building EVERY scenario (and the whole pack manifest) is orthogonal to P12: the training
        // decision is unmoved and every scenario records training_justified=false.
        let before = decide(&[], &[]);
        for s in Scenario::ALL {
            let trace = scenario_trace(s).unwrap();
            assert!(!trace.training_justified(), "{}", s.slug());
            assert!(trace.training_gate_unchanged(), "{}", s.slug());
        }
        let _pack = scenario_pack_manifest().unwrap();
        let after = decide(&[], &[]);
        assert_eq!(before, after);
        assert!(!after.training_justified);
    }

    #[test]
    fn tampered_scenario_bundle_is_refused() {
        // A tampered or incomplete scenario bundle is refused (never trusted over the re-derivation):
        // every content file is re-derived and byte-compared, and a missing file is reported.
        let s = Scenario::ReviewRejected;
        let mut b = scenario_bundle_owned(s);
        let i = b.iter().position(|(n, _)| n == BUNDLE_TRACE_FILE).unwrap();
        b[i].1 = b[i].1.replace(
            "\"review_decision\": \"rejected\"",
            "\"review_decision\": \"approved\"",
        );
        assert!(matches!(
            verify_scenario_bundle(s, &b),
            Err(TraceError::BundleMismatch(_))
        ));
        let mut b2 = scenario_bundle_owned(s);
        let j = b2
            .iter()
            .position(|(n, _)| n == BUNDLE_MANIFEST_FILE)
            .unwrap();
        b2[j].1 = b2[j].1.replace("cognitive-bundle-v0.1", "forged");
        assert!(matches!(
            verify_scenario_bundle(s, &b2),
            Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_MANIFEST_FILE
        ));
        let mut b3 = scenario_bundle_owned(s);
        b3.retain(|(n, _)| n != BUNDLE_QUESTIONS_FILE);
        assert!(matches!(
            verify_scenario_bundle(s, &b3),
            Err(TraceError::BundleMissingFile(ref f)) if f == BUNDLE_QUESTIONS_FILE
        ));
        let tampered_pack = scenario_pack_manifest()
            .unwrap()
            .replace("cognitive-scenario-pack-v0.1", "forged");
        assert!(verify_scenario_pack_manifest(&tampered_pack).is_err());
    }

    #[test]
    fn scenarios_are_distinguishable() {
        // The four scenario traces are pairwise distinct (distinguishable by ids/statuses), and the pack
        // manifest records the distinct paths — variation is real, not cosmetic — while every row still
        // preserves the boundary (no grant, nothing executes, training stays false).
        let traces: Vec<String> = Scenario::ALL
            .iter()
            .map(|s| scenario_trace(*s).unwrap().to_json())
            .collect();
        for a in 0..traces.len() {
            for b in (a + 1)..traces.len() {
                assert_ne!(
                    traces[a], traces[b],
                    "scenarios {a} and {b} must be distinguishable"
                );
            }
        }
        let pack = scenario_pack_manifest().unwrap();
        assert!(pack.contains("requires_operator"));
        assert!(pack.contains("\"execution_status\": \"blocked\""));
        assert!(pack.contains("\"review_decision\": \"deferred\""));
        assert!(pack.contains("\"review_decision\": \"rejected\""));
        assert!(!pack.contains("\"grants_promotion\": true"));
        assert!(!pack.contains("\"training_justified\": true"));
    }

    // --- MTRACE-1: the scenario boundary-coverage matrix. The matrix is purely re-derived from the
    //     scenario set and proves the four boundaries hold for every path; verify/report re-derive and
    //     byte-compare, refusing tampered matrices or packs. ---

    /// The canonical scenario pack as the (slug, files) shape `verify_scenario_pack` consumes.
    fn canonical_pack_owned() -> Vec<(String, Vec<(String, String)>)> {
        Scenario::ALL
            .iter()
            .map(|s| {
                let files = scenario_bundle(*s)
                    .unwrap()
                    .into_iter()
                    .map(|(name, content)| (name.to_string(), content))
                    .collect();
                (s.slug().to_string(), files)
            })
            .collect()
    }

    #[test]
    fn scenario_matrix_lists_all_scenarios() {
        // The matrix has one row per scenario and names every scenario slug.
        let matrix = scenario_matrix().unwrap();
        assert_eq!(canonical_scenario_matrix().unwrap().scenarios.len(), 4);
        for s in Scenario::ALL {
            assert!(matrix.contains(s.slug()), "matrix must list {}", s.slug());
        }
        assert!(matrix.contains("\"scenario_count\": 4"));
    }

    #[test]
    fn scenario_matrix_records_all_statuses() {
        // Every row records the review, probe, intent, observation, promotion status and the training
        // verdict (the full path), for every scenario.
        let matrix = scenario_matrix().unwrap();
        for field in [
            "review_status",
            "probe_status",
            "intent_status",
            "observation_status",
            "promotion_status",
            "training_verdict",
        ] {
            assert!(matrix.contains(field), "matrix must record {field}");
        }
        // The recorded statuses match each scenario's trace exactly.
        for s in Scenario::ALL {
            let trace = scenario_trace(s).unwrap();
            assert!(matrix.contains(&format!(
                "\"intent_status\": \"{}\"",
                trace.execution_status()
            )));
        }
    }

    #[test]
    fn scenario_matrix_proves_no_execution_for_all() {
        // Every row proves no_execution=true; the matrix never records no_execution=false.
        let m = canonical_scenario_matrix().unwrap();
        assert_eq!(m.scenarios.len(), 4);
        assert!(m.scenarios.iter().all(|r| r.no_execution));
        assert!(!scenario_matrix()
            .unwrap()
            .contains("\"no_execution\": false"));
    }

    #[test]
    fn scenario_matrix_proves_no_evidence_for_all() {
        let m = canonical_scenario_matrix().unwrap();
        assert!(m.scenarios.iter().all(|r| r.no_evidence));
        assert!(!scenario_matrix()
            .unwrap()
            .contains("\"no_evidence\": false"));
    }

    #[test]
    fn scenario_matrix_proves_no_promotion_for_all() {
        let m = canonical_scenario_matrix().unwrap();
        assert!(m.scenarios.iter().all(|r| r.no_promotion));
        assert!(!scenario_matrix()
            .unwrap()
            .contains("\"no_promotion\": false"));
    }

    #[test]
    fn scenario_matrix_proves_training_false_for_all() {
        // Every row proves no_training=true and records the training_not_justified verdict; the matrix
        // never records a training_justified verdict or no_training=false.
        let m = canonical_scenario_matrix().unwrap();
        assert!(m.scenarios.iter().all(|r| r.no_training));
        assert!(m
            .scenarios
            .iter()
            .all(|r| r.training_verdict == "training_not_justified"));
        let matrix = scenario_matrix().unwrap();
        assert!(!matrix.contains("\"no_training\": false"));
        assert!(!matrix.contains("\"training_verdict\": \"training_justified\""));
    }

    #[test]
    fn scenario_matrix_verify_rejects_tampered_matrix() {
        // A tampered matrix is refused: verify re-derives the canonical matrix and byte-compares.
        let matrix = scenario_matrix().unwrap();
        assert!(verify_scenario_matrix(&matrix).is_ok());
        let tampered = matrix.replace("\"no_execution\": true", "\"no_execution\": false");
        assert_ne!(tampered, matrix, "the tamper changed the bytes");
        assert!(matches!(
            verify_scenario_matrix(&tampered),
            Err(TraceError::MatrixMismatch)
        ));
        // A foreign matrix is likewise refused.
        assert!(verify_scenario_matrix("{\"not\":\"a matrix\"}").is_err());
    }

    #[test]
    fn scenario_matrix_verify_rejects_tampered_pack() {
        // A tampered scenario pack is refused by the whole-pack verifier (it re-derives and
        // byte-compares each scenario bundle), and a missing scenario is reported.
        let pack = canonical_pack_owned();
        assert!(verify_scenario_pack(&pack, &scenario_pack_manifest().unwrap()).is_ok());
        let mut tampered = canonical_pack_owned();
        let trace_idx = tampered[1]
            .1
            .iter()
            .position(|(n, _)| n == BUNDLE_TRACE_FILE)
            .unwrap();
        tampered[1].1[trace_idx].1 = tampered[1].1[trace_idx].1.replace(
            "\"review_decision\": \"rejected\"",
            "\"review_decision\": \"approved\"",
        );
        assert!(matches!(
            verify_scenario_pack(&tampered, &scenario_pack_manifest().unwrap()),
            Err(TraceError::BundleMismatch(_))
        ));
        // A tampered pack manifest is also refused.
        let bad_manifest = scenario_pack_manifest()
            .unwrap()
            .replace("cognitive-scenario-pack-v0.1", "forged");
        assert!(verify_scenario_pack(&pack, &bad_manifest).is_err());
    }

    #[test]
    fn scenario_matrix_report_contains_boundary_summary() {
        // The report renders the coverage and states the boundary explicitly, in prose, including all
        // five MTRACE-1 boundary lines verbatim.
        let report = scenario_matrix_report(&scenario_matrix().unwrap()).unwrap();
        assert!(report.contains("COVERAGE"));
        assert!(report.contains("cells proven:        16/16"));
        assert!(report.contains("all_boundaries_hold: true"));
        assert!(report.contains(
            "Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains."
        ));
        for line in MATRIX_BOUNDARY_LINES {
            assert!(
                report.contains(line),
                "report must contain boundary: {line}"
            );
        }
        // The report refuses a tampered matrix (it never renders an untrusted matrix's claims).
        let tampered = scenario_matrix().unwrap().replace(
            "\"all_boundaries_hold\": true",
            "\"all_boundaries_hold\": false",
        );
        assert!(matches!(
            scenario_matrix_report(&tampered),
            Err(TraceError::MatrixMismatch)
        ));
    }

    #[test]
    fn scenario_matrix_does_not_change_training_gate() {
        // Building the matrix (and verifying/reporting it) is orthogonal to P12: the training decision
        // is unmoved and the matrix records training stays false.
        let before = decide(&[], &[]);
        let matrix = scenario_matrix().unwrap();
        let _ = scenario_matrix_report(&matrix).unwrap();
        verify_scenario_matrix(&matrix).unwrap();
        let after = decide(&[], &[]);
        assert_eq!(before, after);
        assert!(!after.training_justified);
        assert!(matrix.contains("training_not_justified"));
    }

    #[test]
    fn scenario_matrix_distinguishes_all_paths() {
        // The coverage distinguishes the four paths: it records both a requires_operator and a blocked
        // intent, all three review decisions, and both a queued and a blocked probe.
        let m = canonical_scenario_matrix().unwrap();
        assert_eq!(
            m.coverage.distinct_intent_statuses,
            vec!["blocked".to_string(), "requires_operator".to_string()]
        );
        assert_eq!(
            m.coverage.distinct_review_statuses,
            vec![
                "approved".to_string(),
                "deferred".to_string(),
                "rejected".to_string()
            ]
        );
        assert_eq!(
            m.coverage.distinct_probe_statuses,
            vec!["blocked".to_string(), "queued".to_string()]
        );
        // The four scenario slugs each appear exactly once.
        for s in Scenario::ALL {
            assert_eq!(m.scenarios.iter().filter(|r| r.slug == s.slug()).count(), 1);
        }
    }

    #[test]
    fn scenario_matrix_report_is_not_authority() {
        // The matrix and its report are output, not authority: no affirmative executed/promoted/granted/
        // recorded status, no true grant, no training_justified verdict — and the frozen canonical trace
        // is still byte-identical (the matrix did not perturb it).
        let matrix = scenario_matrix().unwrap();
        let report = scenario_matrix_report(&matrix).unwrap();
        for blob in [&matrix, &report] {
            assert!(!blob.contains(": executed"));
            assert!(!blob.contains(": promoted"));
            assert!(!blob.contains(": granted"));
            assert!(!blob.contains(": recorded"));
            assert!(!blob.contains("\"grants_promotion\": true"));
        }
        assert!(!matrix.contains("\"training_verdict\": \"training_justified\""));
        assert_eq!(
            scenario_trace(Scenario::HappyBoundary).unwrap().to_json(),
            CognitiveTrace::demo().unwrap().to_json()
        );
    }

    // --- MTRACE-2: scenario failure-injection / boundary-regression pack ---

    #[test]
    fn failure_pack_lists_all_cases() {
        // The pack enumerates every negative scenario, the summary counts them, and the listing covers
        // them — and every forgery both applied and was rejected.
        let pack = canonical_failure_pack().unwrap();
        assert_eq!(pack.cases.len(), 7);
        assert_eq!(pack.summary.case_count, 7);
        for case in FailureCase::ALL {
            assert!(
                pack.cases.iter().any(|c| c.slug == case.slug()),
                "{} is recorded",
                case.slug()
            );
        }
        assert!(pack.summary.all_forged, "every forgery genuinely applied");
        assert!(
            pack.summary.all_inject_forbidden,
            "every forgery injected its specific forbidden authority token"
        );
        assert!(pack.summary.all_rejected, "every forgery was rejected");
        let listing = list_failure_cases();
        for case in FailureCase::ALL {
            assert!(
                listing.contains(case.slug()),
                "listing covers {}",
                case.slug()
            );
        }
        // The slug round-trips through the closed enum; an unknown slug fails closed.
        assert_eq!(
            FailureCase::from_slug("forged-execution"),
            Some(FailureCase::ForgedExecution)
        );
        assert_eq!(FailureCase::from_slug("forged-anything"), None);
    }

    #[test]
    fn forged_execution_is_rejected() {
        // Forging the execution intent to claim it ran is refused by re-derive byte-compare (TraceMismatch),
        // a STRUCTURAL rejection — not a prose grep — and the forgery genuinely altered the canonical bytes.
        let attempt = run_failure_case(FailureCase::ForgedExecution).unwrap();
        assert!(
            attempt.forgery_applied,
            "the forgery altered the canonical trace"
        );
        assert!(
            attempt.injects_forbidden,
            "the forgery injected the forbidden executed-status token (not a benign change)"
        );
        assert!(
            matches!(attempt.verdict, Err(TraceError::TraceMismatch)),
            "a forged execution claim is refused"
        );
    }

    #[test]
    fn forged_evidence_is_rejected() {
        let attempt = run_failure_case(FailureCase::ForgedEvidence).unwrap();
        assert!(attempt.forgery_applied);
        assert!(attempt.injects_forbidden);
        assert!(matches!(attempt.verdict, Err(TraceError::TraceMismatch)));
    }

    #[test]
    fn forged_promotion_is_rejected() {
        let attempt = run_failure_case(FailureCase::ForgedPromotion).unwrap();
        assert!(attempt.forgery_applied);
        assert!(attempt.injects_forbidden);
        assert!(matches!(attempt.verdict, Err(TraceError::TraceMismatch)));
    }

    #[test]
    fn forged_training_is_rejected() {
        let attempt = run_failure_case(FailureCase::ForgedTraining).unwrap();
        assert!(attempt.forgery_applied);
        assert!(attempt.injects_forbidden);
        assert!(matches!(attempt.verdict, Err(TraceError::TraceMismatch)));
    }

    #[test]
    fn forged_review_is_rejected() {
        // Forging a rejected scenario review to "approved" is refused by the scenario bundle verifier
        // (BundleMismatch on the trace file), again a re-derive byte-compare, not a content grep.
        let attempt = run_failure_case(FailureCase::ForgedReview).unwrap();
        assert!(attempt.forgery_applied);
        assert!(attempt.injects_forbidden);
        assert!(
            matches!(attempt.verdict, Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_TRACE_FILE)
        );
    }

    #[test]
    fn forged_report_is_rejected() {
        // Forging the report to narrate execution/evidence is refused by the bundle verifier
        // (BundleMismatch on the report file).
        let attempt = run_failure_case(FailureCase::ForgedReport).unwrap();
        assert!(attempt.forgery_applied);
        assert!(attempt.injects_forbidden);
        assert!(
            matches!(attempt.verdict, Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_REPORT_FILE)
        );
    }

    #[test]
    fn forged_matrix_is_rejected() {
        // Forging the coverage matrix to hide a failed cell is refused by the matrix verifier (MatrixMismatch).
        let attempt = run_failure_case(FailureCase::ForgedMatrix).unwrap();
        assert!(attempt.forgery_applied);
        assert!(attempt.injects_forbidden);
        assert!(matches!(attempt.verdict, Err(TraceError::MatrixMismatch)));
    }

    #[test]
    fn failure_report_contains_rejection_reasons() {
        // The rendered report records every case as REJECTED with its exact typed rejection reason —
        // the reasons come from the verifier's typed errors, never from a hand-written string.
        let pack = canonical_failure_pack().unwrap();
        let report = render_failure_pack(&pack);
        assert!(report.contains("REJECTED"));
        for case in &pack.cases {
            assert!(case.rejected, "{} is rejected", case.slug);
            assert!(report.contains(&case.slug), "report names {}", case.slug);
            assert!(
                report.contains(&case.rejection_reason),
                "report records the {} rejection reason",
                case.slug
            );
            assert!(
                !case.rejection_reason.is_empty()
                    && case.rejection_reason != "ACCEPTED — the forgery was NOT rejected",
                "{} has a real rejection reason",
                case.slug
            );
        }
        // The reasons are the real typed-error Displays (tamper/stale/foreign), not bare prose verdicts.
        assert!(report.contains("tampered, stale, or foreign"));
    }

    #[test]
    fn failure_pack_does_not_change_training_gate() {
        // Building the whole pack (running every forgery) leaves the P12 gate closed and the canonical
        // trace byte-identical, and the pack's summary ties to that real, unchanged canonical.
        let before = CognitiveTrace::demo().unwrap();
        assert!(!before.training_justified());
        let _ = failure_pack().unwrap();
        let after = CognitiveTrace::demo().unwrap();
        assert!(!after.training_justified(), "training gate stays closed");
        assert_eq!(
            before.to_json(),
            after.to_json(),
            "canonical trace byte-identical"
        );
        let pack = canonical_failure_pack().unwrap();
        assert_eq!(
            pack.summary.canonical_trace_hash,
            bundle_content_hash(&after.to_json())
        );
    }

    #[test]
    fn failure_pack_forgeries_actually_mutate_canonical() {
        // Every forgery genuinely changes the canonical bytes (so each rejection is REAL, not vacuous) and
        // is rejected — and building the pack leaves the frozen canonical trace AND the MTRACE-1 matrix
        // byte-identical (a failure case mutates no canonical data).
        for case in FailureCase::ALL {
            let attempt = run_failure_case(case).unwrap();
            assert!(
                attempt.forgery_applied,
                "{} forgery alters canonical bytes",
                case.slug()
            );
            assert!(
                attempt.injects_forbidden,
                "{} forgery injects its forbidden authority token",
                case.slug()
            );
            assert!(
                attempt.verdict.is_err(),
                "{} forgery is rejected",
                case.slug()
            );
        }
        let demo_before = CognitiveTrace::demo().unwrap().to_json();
        let matrix_before = scenario_matrix().unwrap();
        let pack_before = scenario_pack_manifest().unwrap();
        let _ = failure_pack().unwrap();
        assert_eq!(demo_before, CognitiveTrace::demo().unwrap().to_json());
        assert_eq!(matrix_before, scenario_matrix().unwrap());
        assert_eq!(pack_before, scenario_pack_manifest().unwrap());
    }

    #[test]
    fn failure_pack_verify_rejects_tampered_pack() {
        // The failure pack is itself re-derive-never-trust: a pristine pack verifies, but a pack doctored to
        // claim a forgery passed (rejected:true -> false) or with a missing file is refused.
        let files = failure_pack_files().unwrap();
        let pristine: Vec<(String, String)> = files
            .iter()
            .map(|(n, c)| (n.to_string(), c.clone()))
            .collect();
        assert!(verify_failure_pack(&pristine).is_ok());
        let tampered: Vec<(String, String)> = files
            .iter()
            .map(|(n, c)| {
                let c2 = if *n == FAILURE_PACK_FILE {
                    c.replacen("\"rejected\": true", "\"rejected\": false", 1)
                } else {
                    c.clone()
                };
                (n.to_string(), c2)
            })
            .collect();
        assert!(
            matches!(verify_failure_pack(&tampered), Err(TraceError::BundleMismatch(ref f)) if f == FAILURE_PACK_FILE)
        );
        let missing: Vec<(String, String)> = files
            .iter()
            .filter(|(n, _)| *n != FAILURE_REPORT_FILE)
            .map(|(n, c)| (n.to_string(), c.clone()))
            .collect();
        assert!(
            matches!(verify_failure_pack(&missing), Err(TraceError::BundleMissingFile(ref f)) if f == FAILURE_REPORT_FILE)
        );
    }

    // --- DOCFLOW-0: operator-supplied document trace ---

    /// A well-formed multi-sentence operator document (two sentences → two spans).
    const DOC_SAMPLE: &str = "The east bridge reopened today. Traffic resumed by noon.";

    /// The doc bundle for `doc` as owned (name, content) pairs, the shape a verifier reads from disk.
    fn doc_provided(doc: &str) -> Vec<(String, String)> {
        doc_bundle(doc)
            .unwrap()
            .iter()
            .map(|(name, content)| (name.to_string(), content.clone()))
            .collect()
    }

    #[test]
    fn doc_trace_starts_from_verified_receipt() {
        // The document flow must VERIFY before tracing: the trace starts from a passed read0 receipt.
        let trace =
            doc_trace(DOC_SAMPLE).expect("a well-formed document produces a verified trace");
        assert!(trace.starts_from_verified_receipt());
        assert!(trace.reading_passed());
    }

    #[test]
    fn doc_trace_cites_document_receipt_hash() {
        // The hypothesis cites the DOCUMENT's own receipt by hash (provenance from the verified read).
        let trace = doc_trace(DOC_SAMPLE).unwrap();
        assert!(trace.hypothesis_cites_receipt());
        assert_eq!(trace.cited_answer_hash(), trace.reading_answer_hash());
        assert_eq!(trace.cited_memory_hash(), trace.reading_memory_hash());
    }

    #[test]
    fn doc_bundle_verifies_clean_input() {
        // A bundle re-derives byte-identically from the SAME document.
        let provided = doc_provided(DOC_SAMPLE);
        assert!(verify_doc_bundle(DOC_SAMPLE, &provided).is_ok());
    }

    #[test]
    fn doc_bundle_rejects_tampered_document() {
        // A bundle built from one document must NOT verify against a DIFFERENT document — the trace
        // (and every derived file) re-derives differently, so the bundle fails to match.
        let provided = doc_provided(DOC_SAMPLE);
        let tampered_doc = "The west bridge collapsed today. Traffic stopped by noon.";
        assert!(matches!(
            verify_doc_bundle(tampered_doc, &provided),
            Err(TraceError::BundleMismatch(_))
        ));
    }

    #[test]
    fn doc_bundle_rejects_tampered_trace() {
        let mut provided = doc_provided(DOC_SAMPLE);
        for (name, content) in provided.iter_mut() {
            if name == BUNDLE_TRACE_FILE {
                content.push_str("\n{tampered}");
            }
        }
        assert!(matches!(
            verify_doc_bundle(DOC_SAMPLE, &provided),
            Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_TRACE_FILE
        ));
    }

    #[test]
    fn doc_bundle_rejects_tampered_report() {
        let mut provided = doc_provided(DOC_SAMPLE);
        for (name, content) in provided.iter_mut() {
            if name == BUNDLE_REPORT_FILE {
                content.push_str("\nexecuted: true");
            }
        }
        assert!(matches!(
            verify_doc_bundle(DOC_SAMPLE, &provided),
            Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_REPORT_FILE
        ));
    }

    #[test]
    fn doc_bundle_rejects_tampered_manifest() {
        let mut provided = doc_provided(DOC_SAMPLE);
        for (name, content) in provided.iter_mut() {
            if name == BUNDLE_MANIFEST_FILE {
                content.push_str("\n{tampered}");
            }
        }
        assert!(matches!(
            verify_doc_bundle(DOC_SAMPLE, &provided),
            Err(TraceError::BundleMismatch(ref f)) if f == BUNDLE_MANIFEST_FILE
        ));
    }

    #[test]
    fn doc_input_path_is_local_and_safe() {
        // Safe local paths are accepted.
        assert!(check_local_input_path("doc.txt").is_ok());
        assert!(check_local_input_path("sub/dir/notes.txt").is_ok());
        assert!(check_local_input_path("./notes.txt").is_ok());
        // Unsafe paths are refused (absolute, parent traversal, embedded escape, tilde, empty).
        assert!(check_local_input_path("/etc/passwd").is_err());
        assert!(check_local_input_path("../secrets.txt").is_err());
        assert!(check_local_input_path("sub/../../escape.txt").is_err());
        assert!(check_local_input_path("~/secret").is_err());
        assert!(check_local_input_path("").is_err());
    }

    #[test]
    fn doc_flow_does_not_change_training_gate() {
        let trace = doc_trace(DOC_SAMPLE).unwrap();
        assert!(trace.training_gate_unchanged());
        assert!(!trace.training_justified());
    }

    #[test]
    fn doc_flow_does_not_execute_or_promote() {
        let trace = doc_trace(DOC_SAMPLE).unwrap();
        assert!(trace.nothing_executed());
        assert!(trace.observation_quarantined());
        assert!(trace.promotion_refused());
        assert!(trace.nothing_becomes_evidence());
        assert_eq!(trace.execution_status(), "requires_operator");
        assert_eq!(trace.promotion_status(), "rejected");
    }

    // --- DOCFLOW-2: document-flow scenario pack / input-integrity matrix ---

    #[test]
    fn doc_scenarios_list_all_cases() {
        // The finite set is exactly nine: one valid (clean) + eight invalid inputs, each with a unique
        // slug, and the menu lists every one.
        assert_eq!(DocScenario::ALL.len(), 9);
        let slugs: Vec<&str> = DocScenario::ALL.iter().map(|s| s.slug()).collect();
        let mut sorted = slugs.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), slugs.len(), "every scenario slug is unique");
        let menu = list_doc_scenarios();
        for s in DocScenario::ALL {
            assert!(menu.contains(s.slug()), "menu lists {}", s.slug());
            assert_eq!(DocScenario::from_slug(s.slug()), Some(s));
        }
        // A non-slug fails closed.
        assert_eq!(DocScenario::from_slug("not-a-scenario"), None);
    }

    #[test]
    fn doc_clean_local_document_verifies() {
        // The clean scenario is OBSERVED to verify, produces a real trace, and its boundary cells come
        // from that trace (all true).
        let entry = run_doc_scenario(DocScenario::CleanLocalDocument).unwrap();
        assert!(entry.verified, "clean document verifies");
        assert!(!entry.refused);
        assert!(entry.produced_trace);
        assert!(entry.rejection_reason.is_empty());
        assert!(
            entry.no_execution && entry.no_evidence && entry.no_promotion && entry.no_training,
            "clean trace preserves the boundary"
        );
        // And the bundle really re-derives byte-identically.
        let provided: Vec<(String, String)> = doc_bundle(DOC_SCENARIO_SAMPLE)
            .unwrap()
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect();
        assert!(verify_doc_bundle(DOC_SCENARIO_SAMPLE, &provided).is_ok());
    }

    #[test]
    fn doc_modified_input_invalidates_bundle() {
        // A clean bundle verified against a DIFFERENT document is refused: the re-derived trace differs,
        // so the stale bundle no longer matches.
        let entry = run_doc_scenario(DocScenario::ModifiedDocument).unwrap();
        assert!(entry.refused, "modified input invalidates the bundle");
        assert!(!entry.verified);
        assert!(
            entry.input_changed,
            "the modified document genuinely differs"
        );
        assert!(
            entry.rejection_reason.starts_with("bundle-file-mismatch:"),
            "rejected by re-derivation, got {}",
            entry.rejection_reason
        );
        // Direct: the clean bundle does NOT verify against the modified document.
        let clean: Vec<(String, String)> = doc_bundle(DOC_SCENARIO_SAMPLE)
            .unwrap()
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect();
        assert!(verify_doc_bundle(DOC_SCENARIO_MODIFIED, &clean).is_err());
    }

    #[test]
    fn doc_empty_document_fails_closed() {
        // An empty document yields no readable span, so the flow fails closed with EmptyDocument — an
        // explicit unsupported status, never an ambiguous success or a panic.
        assert!(matches!(doc_trace(""), Err(TraceError::EmptyDocument)));
        let entry = run_doc_scenario(DocScenario::EmptyDocument).unwrap();
        assert!(entry.refused);
        assert!(!entry.verified);
        assert!(!entry.produced_trace);
        assert_eq!(entry.rejection_reason, "empty-document");
    }

    #[test]
    fn doc_absolute_path_refused() {
        // An absolute input path is refused by the pure path check before any read.
        assert!(matches!(
            check_local_input_path("/etc/passwd"),
            Err(TraceError::UnsafeInputPath(_))
        ));
        let entry = run_doc_scenario(DocScenario::AbsolutePath).unwrap();
        assert!(entry.refused);
        assert_eq!(entry.input_kind, "unsafe-path");
        assert_eq!(entry.rejection_reason, "unsafe-input-path");
    }

    #[test]
    fn doc_parent_traversal_refused() {
        // A `..` traversal input path is refused by the pure path check before any read.
        assert!(matches!(
            check_local_input_path("../escape.txt"),
            Err(TraceError::UnsafeInputPath(_))
        ));
        let entry = run_doc_scenario(DocScenario::ParentTraversal).unwrap();
        assert!(entry.refused);
        assert_eq!(entry.input_kind, "unsafe-path");
        assert_eq!(entry.rejection_reason, "unsafe-input-path");
    }

    #[test]
    fn doc_symlink_escape_refused() {
        // The containment decision refuses a resolved path that escapes the working directory, and accepts
        // one that stays inside it (so the check is discriminating, not always-false).
        let work = std::path::Path::new("/work/project");
        assert!(!resolved_path_within(
            work,
            std::path::Path::new("/etc/hostname")
        ));
        assert!(resolved_path_within(
            work,
            std::path::Path::new("/work/project/sub/doc.txt")
        ));
        let entry = run_doc_scenario(DocScenario::SymlinkEscape).unwrap();
        assert!(entry.refused, "an escaping path is refused");
        assert_eq!(entry.input_kind, "escaping-path");
        assert_eq!(entry.rejection_reason, "escapes-working-directory");
    }

    #[test]
    fn doc_tampered_artifact_refused() {
        // Each tampered bundle file (trace / report / manifest) is refused by re-derivation, and the
        // tamper genuinely changed the bytes (anti-vacuity).
        for scenario in [
            DocScenario::TamperedTrace,
            DocScenario::TamperedReport,
            DocScenario::TamperedManifest,
        ] {
            let entry = run_doc_scenario(scenario).unwrap();
            assert!(entry.refused, "{} is refused", scenario.slug());
            assert!(!entry.verified);
            assert!(
                entry.input_changed,
                "{} genuinely changed bytes",
                scenario.slug()
            );
            assert!(
                entry.rejection_reason.starts_with("bundle-file-mismatch:"),
                "{} rejected by re-derivation, got {}",
                scenario.slug(),
                entry.rejection_reason
            );
        }
    }

    #[test]
    fn doc_scenario_matrix_records_outcomes() {
        // The matrix records one row per scenario with its observed outcome and boundary cells, and the
        // coverage proves every expectation met, all boundary cells hold, and the variation is real.
        let json = doc_scenario_matrix().unwrap();
        for s in DocScenario::ALL {
            assert!(json.contains(s.slug()), "matrix records {}", s.slug());
        }
        assert!(json.contains("\"all_expectations_met\": true"));
        assert!(json.contains("\"all_boundaries_hold\": true"));
        // 9 scenarios × 4 boundary cells = 36 cells, all proven.
        assert!(json.contains("\"cells_total\": 36"));
        assert!(json.contains("\"cells_proven\": 36"));
        assert!(json.contains("\"verified_count\": 1"));
        assert!(json.contains("\"refused_count\": 8"));
        // The pack re-derives and a tampered pack is refused.
        let pack: Vec<(String, String)> = doc_scenario_pack_files()
            .unwrap()
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect();
        assert!(verify_doc_scenario_pack(&pack).is_ok());
        let mut tampered = pack.clone();
        tampered[0].1.push_str("\n{tampered}");
        assert!(verify_doc_scenario_pack(&tampered).is_err());
    }

    #[test]
    fn doc_scenarios_do_not_change_training_gate() {
        // Every scenario keeps training closed: the no_training cell holds for all, and the clean trace
        // proves the training gate is unchanged and not justified.
        let entries = canonical_doc_scenario_entries().unwrap();
        assert_eq!(entries.len(), 9);
        for e in &entries {
            assert!(e.no_training, "{} keeps training closed", e.slug);
            assert!(e.no_execution && e.no_evidence && e.no_promotion);
        }
        let clean = doc_trace(DOC_SCENARIO_SAMPLE).unwrap();
        assert!(clean.training_gate_unchanged());
        assert!(!clean.training_justified());
        // No scenario produced an executed/promoted authority claim.
        let json = doc_scenario_pack_manifest().unwrap();
        assert!(!json.contains("\"verified\": true,\n            \"refused\": true"));
    }

    // --- CORPUS-0: multi-document local corpus trace / source-selection boundary ---

    /// A small two-document corpus, in the SORTED order the shell loader produces (so the first span
    /// belongs to `a-east.txt`). Used by the CORPUS-0 tests.
    fn corpus_sample() -> Vec<(String, String)> {
        vec![
            (
                "a-east.txt".to_string(),
                "The east bridge reopened today. Traffic resumed by noon.".to_string(),
            ),
            (
                "b-west.txt".to_string(),
                "The west tunnel remains closed. Crews continue repairs.".to_string(),
            ),
        ]
    }

    /// The corpus bundle for `documents` as owned (name, content) pairs, the shape a verifier reads from disk.
    fn corpus_provided(documents: &[(String, String)]) -> Vec<(String, String)> {
        corpus_bundle(documents)
            .unwrap()
            .iter()
            .map(|(name, content)| (name.to_string(), content.clone()))
            .collect()
    }

    #[test]
    fn corpus_trace_starts_from_verified_receipt() {
        // The corpus flow must VERIFY before tracing: the trace starts from a passed read0 receipt.
        let trace =
            corpus_trace(&corpus_sample()).expect("a well-formed corpus produces a verified trace");
        assert!(trace.starts_from_verified_receipt());
        assert!(trace.reading_passed());
    }

    #[test]
    fn corpus_trace_cites_receipt_hash() {
        // The hypothesis cites the corpus's own receipt by hash (provenance from the verified read).
        let trace = corpus_trace(&corpus_sample()).unwrap();
        assert!(trace.hypothesis_cites_receipt());
        assert_eq!(trace.cited_answer_hash(), trace.reading_answer_hash());
        assert_eq!(trace.cited_memory_hash(), trace.reading_memory_hash());
    }

    #[test]
    fn corpus_trace_records_grounding_document_and_span() {
        // Source identity is UNAMBIGUOUS: the attribution names the first document (sorted), the
        // globally-first span id, and that span's verbatim text — and the trace grounds on that same text.
        let source = corpus_source(&corpus_sample()).unwrap();
        assert_eq!(source.document_index, 0);
        assert_eq!(source.document_title, "a-east.txt");
        assert_eq!(source.span_id, 0);
        assert_eq!(source.span_text, "The east bridge reopened today.");
        let trace = corpus_trace(&corpus_sample()).unwrap();
        assert!(
            trace.to_json().contains("The east bridge reopened today."),
            "the trace grounds on the recorded source span"
        );
    }

    #[test]
    fn corpus_admits_only_plain_local_txt_files() {
        // Only non-hidden `.txt` files are admitted; hidden files and non-`.txt` files are refused.
        assert!(corpus_admits_filename("report.txt"));
        assert!(corpus_admits_filename("a-east.txt"));
        assert!(corpus_admits_filename("sub.notes.txt"));
        assert!(
            !corpus_admits_filename(".secret.txt"),
            "hidden file refused"
        );
        assert!(!corpus_admits_filename(".txt"), "bare hidden .txt refused");
        assert!(!corpus_admits_filename("notes.md"), "non-txt refused");
        assert!(
            !corpus_admits_filename("archive.txt.bak"),
            "non-txt suffix refused"
        );
        assert!(!corpus_admits_filename("README"));
        assert!(!corpus_admits_filename(""));
    }

    #[test]
    fn corpus_empty_fails_closed() {
        // An empty corpus (no documents), an empty document, and a heading-only document all yield no
        // readable span, so the flow fails closed with EmptyCorpus — never an ambiguous success or a panic.
        assert!(matches!(corpus_trace(&[]), Err(TraceError::EmptyCorpus)));
        assert!(matches!(
            corpus_trace(&[("e.txt".to_string(), String::new())]),
            Err(TraceError::EmptyCorpus)
        ));
        assert!(matches!(
            corpus_trace(&[("h.txt".to_string(), "# Heading only".to_string())]),
            Err(TraceError::EmptyCorpus)
        ));
        // The source attribution fails closed identically.
        assert!(matches!(corpus_source(&[]), Err(TraceError::EmptyCorpus)));
    }

    #[test]
    fn corpus_bundle_verifies_clean_input() {
        // A bundle re-derives byte-identically from the SAME corpus.
        let provided = corpus_provided(&corpus_sample());
        assert!(verify_corpus_bundle(&corpus_sample(), &provided).is_ok());
    }

    #[test]
    fn corpus_bundle_rejects_tampered_corpus() {
        // The bundle commits to the WHOLE corpus via the receipt's structure hash: changing ANY document —
        // including the SECOND, non-grounding one — re-derives a different trace, so the clean bundle no
        // longer matches.
        let clean = corpus_provided(&corpus_sample());
        let mut tampered_second = corpus_sample();
        tampered_second[1].1 = "The west tunnel reopened early. Crews left.".to_string();
        assert!(
            matches!(
                verify_corpus_bundle(&tampered_second, &clean),
                Err(TraceError::BundleMismatch(_))
            ),
            "a non-grounding document tamper invalidates the bundle"
        );
        // Changing the FIRST (grounding) document is likewise refused.
        let mut tampered_first = corpus_sample();
        tampered_first[0].1 = "The east bridge collapsed today. Traffic stopped.".to_string();
        assert!(matches!(
            verify_corpus_bundle(&tampered_first, &clean),
            Err(TraceError::BundleMismatch(_))
        ));
    }

    #[test]
    fn corpus_bundle_rejects_tampered_artifact() {
        // Each tampered bundle file (source / trace / report / questions / manifest) is refused by
        // re-derivation, named by the file that no longer matches.
        for file in CORPUS_BUNDLE_FILES {
            let mut provided = corpus_provided(&corpus_sample());
            let mut changed = false;
            for (name, content) in provided.iter_mut() {
                if name == file {
                    content.push_str("\n{tampered}");
                    changed = true;
                }
            }
            assert!(changed, "forged {file}");
            assert!(
                matches!(
                    verify_corpus_bundle(&corpus_sample(), &provided),
                    Err(TraceError::BundleMismatch(ref f)) if f == file
                ),
                "{file} is refused by re-derivation"
            );
        }
    }

    #[test]
    fn corpus_report_records_source_selection_and_refuses_tamper() {
        // The report names the grounded document/span and lists every corpus document, and a tampered trace
        // is refused (re-derive, never trust).
        let trace = corpus_trace(&corpus_sample()).unwrap();
        let report = run_corpus_report(&corpus_sample(), &trace.to_json()).unwrap();
        assert!(report.contains("SOURCE SELECTION"));
        assert!(report.contains("[0] a-east.txt"));
        assert!(report.contains("The east bridge reopened today."));
        assert!(
            report.contains("b-west.txt"),
            "every corpus document is listed"
        );
        assert!(report.contains("Nothing trains."));
        let mut tampered = trace.to_json();
        tampered.push_str("\n{tampered}");
        assert!(matches!(
            run_corpus_report(&corpus_sample(), &tampered),
            Err(TraceError::CorpusTraceMismatch)
        ));
    }

    #[test]
    fn corpus_flow_does_not_change_training_gate() {
        let trace = corpus_trace(&corpus_sample()).unwrap();
        assert!(trace.training_gate_unchanged());
        assert!(!trace.training_justified());
    }

    #[test]
    fn corpus_flow_does_not_execute_or_promote() {
        let trace = corpus_trace(&corpus_sample()).unwrap();
        assert!(trace.nothing_executed());
        assert!(trace.observation_quarantined());
        assert!(trace.promotion_refused());
        assert!(trace.nothing_becomes_evidence());
        assert_eq!(trace.execution_status(), "requires_operator");
        assert_eq!(trace.promotion_status(), "rejected");
    }

    #[test]
    fn corpus_source_is_deterministic_and_replayable() {
        // The corpus bundle and trace are pure functions of the corpus content + document names: two runs
        // are byte-identical, so the source selection is replayable.
        assert_eq!(
            corpus_bundle(&corpus_sample()).unwrap(),
            corpus_bundle(&corpus_sample()).unwrap()
        );
        assert_eq!(
            corpus_trace(&corpus_sample()).unwrap().to_json(),
            corpus_trace(&corpus_sample()).unwrap().to_json()
        );
        // The grounded source is the globally-first span of the first document that owns one.
        let source = corpus_source(&corpus_sample()).unwrap();
        assert_eq!(source.span_id, 0);
        assert_eq!(source.document_index, 0);
    }

    // --- CORPUS-2: corpus scenario pack / input-integrity matrix ---

    #[test]
    fn corpus_scenarios_list_all_cases() {
        // The finite set is exactly thirteen: one valid (clean two-document) + twelve invalid inputs, each
        // with a unique slug, and the menu lists every one.
        assert_eq!(CorpusScenario::ALL.len(), 13);
        let slugs: Vec<&str> = CorpusScenario::ALL.iter().map(|s| s.slug()).collect();
        let mut sorted = slugs.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), slugs.len(), "every scenario slug is unique");
        let menu = list_corpus_scenarios();
        for s in CorpusScenario::ALL {
            assert!(menu.contains(s.slug()), "menu lists {}", s.slug());
            assert_eq!(CorpusScenario::from_slug(s.slug()), Some(s));
        }
        // A non-slug fails closed.
        assert_eq!(CorpusScenario::from_slug("not-a-scenario"), None);
    }

    #[test]
    fn corpus_clean_two_document_case_verifies() {
        // The clean scenario is OBSERVED to verify, produces a real trace, and its boundary cells come from
        // that trace (all true).
        let entry = run_corpus_scenario(CorpusScenario::CleanTwoDocument).unwrap();
        assert!(entry.verified, "clean corpus verifies");
        assert!(!entry.refused);
        assert!(entry.produced_trace);
        assert!(entry.rejection_reason.is_empty());
        assert!(
            entry.no_execution && entry.no_evidence && entry.no_promotion && entry.no_training,
            "clean trace preserves the boundary"
        );
        // And the bundle really re-derives byte-identically.
        let provided = corpus_provided(&corpus_scenario_sample());
        assert!(verify_corpus_bundle(&corpus_scenario_sample(), &provided).is_ok());
    }

    #[test]
    fn corpus_empty_case_fails_closed() {
        // An empty corpus yields no readable span, so the flow fails closed with EmptyCorpus — an explicit
        // unsupported status, never an ambiguous success or a panic.
        assert!(matches!(corpus_trace(&[]), Err(TraceError::EmptyCorpus)));
        let entry = run_corpus_scenario(CorpusScenario::EmptyCorpus).unwrap();
        assert!(entry.refused);
        assert!(!entry.verified);
        assert!(!entry.produced_trace);
        assert_eq!(entry.rejection_reason, "empty-corpus");
    }

    #[test]
    fn corpus_hidden_only_case_refused() {
        // A corpus of only hidden files admits NONE through the real admission filter, so it is refused before
        // any read; an empty admitted corpus then fails closed with EmptyCorpus.
        for name in CORPUS_HIDDEN_ONLY_NAMES {
            assert!(!corpus_admits_filename(name), "{name} is not admitted");
        }
        let entry = run_corpus_scenario(CorpusScenario::HiddenOnly).unwrap();
        assert!(entry.refused);
        assert!(!entry.verified);
        assert_eq!(entry.input_kind, "hidden-only");
        assert_eq!(entry.rejection_reason, "no-admitted-files");
        // An admitted corpus of zero documents fails closed end-to-end.
        assert!(matches!(corpus_trace(&[]), Err(TraceError::EmptyCorpus)));
    }

    #[test]
    fn corpus_non_txt_only_case_refused() {
        // A corpus of only non-.txt files admits NONE through the real admission filter, so it is refused.
        for name in CORPUS_NON_TXT_ONLY_NAMES {
            assert!(!corpus_admits_filename(name), "{name} is not admitted");
        }
        let entry = run_corpus_scenario(CorpusScenario::NonTxtOnly).unwrap();
        assert!(entry.refused);
        assert!(!entry.verified);
        assert_eq!(entry.input_kind, "non-txt-only");
        assert_eq!(entry.rejection_reason, "no-admitted-files");
    }

    #[test]
    fn corpus_absolute_path_refused() {
        // An absolute corpus path is refused by the pure path check before any read.
        assert!(matches!(
            check_local_input_path("/etc/passwd"),
            Err(TraceError::UnsafeInputPath(_))
        ));
        let entry = run_corpus_scenario(CorpusScenario::AbsolutePath).unwrap();
        assert!(entry.refused);
        assert_eq!(entry.input_kind, "unsafe-path");
        assert_eq!(entry.rejection_reason, "unsafe-input-path");
    }

    #[test]
    fn corpus_parent_traversal_refused() {
        // A `..` traversal corpus path is refused by the pure path check before any read.
        assert!(matches!(
            check_local_input_path("../escape"),
            Err(TraceError::UnsafeInputPath(_))
        ));
        let entry = run_corpus_scenario(CorpusScenario::ParentTraversal).unwrap();
        assert!(entry.refused);
        assert_eq!(entry.input_kind, "unsafe-path");
        assert_eq!(entry.rejection_reason, "unsafe-input-path");
    }

    #[test]
    fn corpus_symlink_escape_refused() {
        // The containment decision refuses a resolved path that escapes the corpus root, and accepts one that
        // stays inside it (so the check is discriminating, not always-false).
        let root = std::path::Path::new("/work/corpus");
        assert!(!resolved_path_within(
            root,
            std::path::Path::new("/etc/hostname")
        ));
        assert!(resolved_path_within(
            root,
            std::path::Path::new("/work/corpus/sub/doc.txt")
        ));
        let entry = run_corpus_scenario(CorpusScenario::SymlinkEscape).unwrap();
        assert!(entry.refused, "an escaping path is refused");
        assert_eq!(entry.input_kind, "escaping-path");
        assert_eq!(entry.rejection_reason, "escapes-working-directory");
    }

    #[test]
    fn corpus_grounding_doc_mutation_invalidates_bundle() {
        // Mutating the FIRST (grounding) document changes its first span, so the source attribution AND the
        // trace re-derive differently — the clean bundle fails first on corpus-source.json.
        let entry = run_corpus_scenario(CorpusScenario::GroundingMutation).unwrap();
        assert!(entry.refused, "a grounding-document mutation is refused");
        assert!(!entry.verified);
        assert!(
            entry.input_changed,
            "the grounding document genuinely differs"
        );
        assert_eq!(
            entry.rejection_reason, "bundle-file-mismatch:corpus-source.json",
            "the grounding mutation changes the source attribution first"
        );
    }

    #[test]
    fn corpus_side_doc_mutation_invalidates_bundle() {
        // Mutating the SECOND (non-grounding) document leaves corpus-source.json byte-identical, yet the
        // structure hash binds the WHOLE corpus, so the trace re-derives differently — the clean bundle fails
        // on trace.json. This is the whole-corpus-binding proof.
        let entry = run_corpus_scenario(CorpusScenario::SideDocumentMutation).unwrap();
        assert!(
            entry.refused,
            "a non-grounding side-document mutation is refused"
        );
        assert!(!entry.verified);
        assert!(entry.input_changed, "the side document genuinely differs");
        assert_eq!(
            entry.rejection_reason, "bundle-file-mismatch:trace.json",
            "the side mutation leaves the source attribution intact but breaks the whole-corpus trace"
        );
        // Direct proof of whole-corpus binding: source attribution unchanged, bundle still refused.
        let sample = corpus_scenario_sample();
        let clean = corpus_provided(&sample);
        let mut side = sample.clone();
        side[1].1 = CORPUS_SIDE_MUTATION.to_string();
        assert_eq!(
            corpus_source_json(&side).unwrap(),
            corpus_source_json(&sample).unwrap(),
            "the source attribution is identical under a non-grounding mutation"
        );
        assert!(
            verify_corpus_bundle(&side, &clean).is_err(),
            "yet the whole-corpus bundle is still refused"
        );
        assert!(corpus_whole_binding_holds().unwrap());
    }

    #[test]
    fn corpus_tampered_artifacts_refused() {
        // Each tampered bundle file (source / trace / report / manifest) is a refused scenario by re-derivation,
        // and the tamper genuinely changed the bytes (anti-vacuity).
        for scenario in [
            CorpusScenario::TamperedSource,
            CorpusScenario::TamperedTrace,
            CorpusScenario::TamperedReport,
            CorpusScenario::TamperedManifest,
        ] {
            let entry = run_corpus_scenario(scenario).unwrap();
            assert!(entry.refused, "{} is refused", scenario.slug());
            assert!(!entry.verified);
            assert!(
                entry.input_changed,
                "{} genuinely changed bytes",
                scenario.slug()
            );
            assert!(
                entry.rejection_reason.starts_with("bundle-file-mismatch:"),
                "{} rejected by re-derivation, got {}",
                scenario.slug(),
                entry.rejection_reason
            );
        }
        // Belt-and-suspenders: EVERY bundle file (including questions.txt) is tamper-sensitive directly.
        for file in CORPUS_BUNDLE_FILES {
            let mut provided = corpus_provided(&corpus_scenario_sample());
            let mut changed = false;
            for (name, content) in provided.iter_mut() {
                if name == file {
                    content.push_str("\n{tampered}");
                    changed = true;
                }
            }
            assert!(changed, "forged {file}");
            assert!(
                matches!(
                    verify_corpus_bundle(&corpus_scenario_sample(), &provided),
                    Err(TraceError::BundleMismatch(ref f)) if f == file
                ),
                "{file} is refused by re-derivation"
            );
        }
    }

    #[test]
    fn corpus_scenario_matrix_records_source_and_boundaries() {
        // The matrix records one row per scenario with its observed outcome and boundary cells; the coverage
        // proves every expectation met, all boundary cells hold, the whole corpus is hash-bound; and it records
        // the verified case's source identity. A tampered pack is refused.
        let json = corpus_scenario_matrix().unwrap();
        for s in CorpusScenario::ALL {
            assert!(json.contains(s.slug()), "matrix records {}", s.slug());
        }
        assert!(json.contains("\"all_expectations_met\": true"));
        assert!(json.contains("\"all_boundaries_hold\": true"));
        assert!(json.contains("\"whole_corpus_bound\": true"));
        // 13 scenarios × 4 boundary cells = 52 cells, all proven.
        assert!(json.contains("\"cells_total\": 52"));
        assert!(json.contains("\"cells_proven\": 52"));
        assert!(json.contains("\"verified_count\": 1"));
        assert!(json.contains("\"refused_count\": 12"));
        // The verified case's source identity is recorded (document/span that grounded the answer).
        assert!(json.contains("\"document_title\": \"a-east.txt\""));
        assert!(json.contains("\"span_id\": 0"));
        assert!(json.contains("\"span_text\": \"The east bridge reopened today.\""));
        // The pack re-derives and a tampered pack is refused.
        let pack: Vec<(String, String)> = corpus_scenario_pack_files()
            .unwrap()
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect();
        assert!(verify_corpus_scenario_pack(&pack).is_ok());
        let mut tampered = pack.clone();
        tampered[0].1.push_str("\n{tampered}");
        assert!(verify_corpus_scenario_pack(&tampered).is_err());
        // Every scenario keeps training closed (no scenario opens the gate).
        let entries = canonical_corpus_scenario_entries().unwrap();
        assert_eq!(entries.len(), 13);
        for e in &entries {
            assert!(e.no_training, "{} keeps training closed", e.slug);
        }
    }

    // --- NOVELTY-0: hypothesis-only novelty packet harness ---

    /// A two-line operator frame: each line is a candidate assumption to break. The claims are NOT in the
    /// corpus, so none of them can become a grounded preserved fact.
    fn novelty_frame() -> String {
        "The east bridge stays closed indefinitely.\nTraffic never recovers after a closure.\n"
            .to_string()
    }

    #[test]
    fn novelty_packet_requires_verified_corpus_receipt() {
        // The packet is grounded in a VERIFIED corpus trace: a corpus that grounds nothing fails closed (no
        // packet), and a well-formed corpus yields a packet whose receipt hash IS the verified trace's hash.
        assert!(matches!(
            novelty_packet(&[], &novelty_frame()),
            Err(TraceError::EmptyCorpus)
        ));
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        let trace = corpus_trace(&corpus_sample()).unwrap();
        assert_eq!(
            Some(packet.source_receipt_hash),
            trace.reading_structure_hash
        );
    }

    #[test]
    fn novelty_packet_cites_receipt_and_source_identity() {
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        let source = corpus_source(&corpus_sample()).unwrap();
        assert_ne!(packet.source_receipt_hash, 0);
        assert_eq!(
            packet.source_corpus_hash,
            corpus_identity_hash(&corpus_sample())
        );
        assert_eq!(packet.preserved_facts, vec![source.span_text.clone()]);
        assert!(packet.packet_id.starts_with("novelty-"));
    }

    #[test]
    fn novelty_packet_authority_is_hypothesis_only() {
        let json = novelty_packet_json(&corpus_sample(), &novelty_frame()).unwrap();
        assert!(json.contains("\"authority\": \"hypothesis_only\""));
        // There is NO score field (a score could be mistaken for authority) and no affirmative-authority status.
        assert!(!json.contains("\"score\""));
        assert!(!json.contains("\"executed\""));
        assert!(!json.contains("\"promoted\""));
    }

    #[test]
    fn novelty_packet_records_broken_assumptions() {
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        assert_eq!(
            packet.broken_assumptions,
            vec![
                "The east bridge stays closed indefinitely.".to_string(),
                "Traffic never recovers after a closure.".to_string(),
            ]
        );
    }

    #[test]
    fn novelty_packet_records_preserved_facts_grounded() {
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        let spans = corpus_verified_spans(&corpus_sample());
        assert!(!packet.preserved_facts.is_empty());
        for fact in &packet.preserved_facts {
            assert!(
                spans.contains(fact),
                "every preserved fact is a verified span"
            );
        }
    }

    #[test]
    fn novelty_packet_records_falsifiers() {
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        assert_eq!(packet.falsifiers.len(), packet.preserved_facts.len());
        assert!(packet.falsifiers.iter().all(|f| f.contains("Falsified if")));
    }

    #[test]
    fn novelty_probe_requests_do_not_execute() {
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        assert_eq!(packet.probe_requests.len(), packet.broken_assumptions.len());
        for probe in &packet.probe_requests {
            assert!(!probe.executes, "a probe request never executes");
            assert_eq!(probe.status, "requires_operator_review");
        }
    }

    #[test]
    fn novelty_packet_cannot_become_evidence_or_promote_or_train() {
        let packet = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        assert_eq!(
            packet.forbidden_uses,
            vec!["evidence", "execution", "promotion", "training"]
        );
        assert!(matches!(packet.authority, NoveltyAuthority::HypothesisOnly));
        // The corpus trace the packet is grounded in keeps the whole authority boundary closed.
        let trace = corpus_trace(&corpus_sample()).unwrap();
        assert!(trace.nothing_becomes_evidence());
        assert!(trace.promotion_refused());
        assert!(!trace.training_justified());
    }

    #[test]
    fn novelty_packet_replay_is_deterministic() {
        // Two derivations are byte-identical, and the canonical packet verifies (replay confirms determinism).
        let a = novelty_packet_json(&corpus_sample(), &novelty_frame()).unwrap();
        let b = novelty_packet_json(&corpus_sample(), &novelty_frame()).unwrap();
        assert_eq!(a, b);
        assert!(verify_novelty_packet_json(&corpus_sample(), &novelty_frame(), &a).is_ok());
        assert!(run_novelty_replay(&corpus_sample(), &novelty_frame(), &a).is_ok());
    }

    #[test]
    fn novelty_packet_rejects_tampered_packet() {
        let mut packet = novelty_packet_json(&corpus_sample(), &novelty_frame()).unwrap();
        packet.push_str("\n{tampered}");
        assert!(matches!(
            verify_novelty_packet_json(&corpus_sample(), &novelty_frame(), &packet),
            Err(TraceError::NoveltyPacketMismatch)
        ));
        assert!(run_novelty_report(&corpus_sample(), &novelty_frame(), &packet).is_err());
        assert!(run_novelty_replay(&corpus_sample(), &novelty_frame(), &packet).is_err());
    }

    #[test]
    fn novelty_facts_grounded_rejects_unsupported_fact() {
        // A fact that is not a verified span is refused; the grounded source span is accepted.
        let source = corpus_source(&corpus_sample()).unwrap();
        assert!(novelty_facts_grounded(&corpus_sample(), &[source.span_text]).is_ok());
        assert!(matches!(
            novelty_facts_grounded(
                &corpus_sample(),
                &["A fact not present in the corpus.".to_string()]
            ),
            Err(TraceError::UnsupportedPreservedFact)
        ));
    }

    #[test]
    fn novelty_packet_refuses_corpus_trace_missing_receipt_hash() {
        // novelty-packet verifies the provided corpus trace against the re-derivation. A trace JSON with its
        // receipt hash stripped is NOT the verified trace, so the packet refuses to ground on it.
        let trace = run_corpus_trace(&corpus_sample()).unwrap();
        assert!(run_novelty_packet(&corpus_sample(), &trace, &novelty_frame()).is_ok());
        let stripped: String = trace
            .lines()
            .filter(|line| !line.contains("structure_hash"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(stripped, trace, "the receipt hash line was removed");
        assert!(matches!(
            run_novelty_packet(&corpus_sample(), &stripped, &novelty_frame()),
            Err(TraceError::CorpusTraceMismatch)
        ));
    }

    #[test]
    fn novelty_packet_does_not_change_training_gate() {
        let before = corpus_trace(&corpus_sample()).unwrap().training_justified();
        let _ = novelty_packet(&corpus_sample(), &novelty_frame()).unwrap();
        let after = corpus_trace(&corpus_sample()).unwrap().training_justified();
        assert!(
            !before && !after,
            "the novelty harness leaves P12 training_justified=false"
        );
        assert!(corpus_trace(&corpus_sample())
            .unwrap()
            .training_gate_unchanged());
    }

    #[test]
    fn novelty_frame_text_is_not_trusted_as_fact() {
        // A frame claim that is NOT a verified corpus span can never become a preserved fact: the grounding
        // gate refuses it, and the packet's preserved facts contain only verified spans, never the frame's
        // assertions. The frame IS recorded verbatim (as data), but only as the recorded frame, not as a fact.
        let frame = "Bridges are always unsafe after rain.\n".to_string();
        let packet = novelty_packet(&corpus_sample(), &frame).unwrap();
        assert!(!packet
            .preserved_facts
            .iter()
            .any(|f| f.contains("Bridges are always unsafe")));
        assert!(matches!(
            novelty_facts_grounded(
                &corpus_sample(),
                &["Bridges are always unsafe after rain.".to_string()]
            ),
            Err(TraceError::UnsupportedPreservedFact)
        ));
        assert!(packet
            .frame_text
            .contains("Bridges are always unsafe after rain."));
    }

    #[test]
    fn novelty_empty_frame_fails_closed() {
        // A frame with no non-empty line has nothing to break, so the harness refuses to emit a packet.
        assert!(matches!(
            novelty_packet(&corpus_sample(), "\n   \n"),
            Err(TraceError::EmptyFrame)
        ));
        assert!(matches!(
            frame_assumptions("   "),
            Err(TraceError::EmptyFrame)
        ));
    }
}
