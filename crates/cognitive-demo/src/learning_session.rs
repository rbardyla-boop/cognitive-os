//! SESSION-LOOP-0: the receipt-linked learning session composer.
//!
//! This module composes the six committed organs into ONE deterministic
//! learning session:
//!
//! ```text
//! question -> verified evidence (QFLOW, run inside LIT-INTENT)
//!          -> literature intent map (LIT-INTENT)
//!          -> supported lesson (TEACH)
//!          -> quiz/check result (LEARNER-MODEL, exact-match law)
//!          -> memory candidate (LEARNER-MEMORY-0)
//!          -> consented journal append (LEARNER-MEMORY-1, in-memory fold)
//! ```
//!
//! Authority law: the session ADDS NO NEW AUTHORITY. It does not score, rank,
//! select, grade, or verify content itself — every step's authority string is
//! the frozen organ's own (the journal step's authority IS the scope-bound
//! consent string). A refusal anywhere in the chain refuses the whole session
//! with a refusal naming the failing stage.
//!
//! Grading law: quiz answers are judged ONLY by the frozen LEARNER-MODEL
//! exact-match law. An incorrect answer COMPLETES the session (wrong is a
//! recorded outcome, not a refusal); an unrecognized quiz id REFUSES.
//!
//! Memory law: the session never writes memory silently. Journal append runs
//! only when the request asks for it AND carries explicit scope-bound consent;
//! the append itself is the pure LEARNER-MEMORY-1 fold. Durable file
//! persistence stays exclusively behind the existing `learner-journal-append`
//! CLI verb — this module performs no I/O.
//!
//! The two `*_uses_model` organ passthrough flags exist ONLY so the matrix can
//! construct the corresponding propagated refusals; a true flag always refuses
//! through the frozen organ's own signal gate. They grant nothing.

use serde::Serialize;

use crate::{
    append_learner_journal_default, journal_scope_for_candidate, learner_memory_demo,
    learner_model_demo, literature_intent_demo, run_learner_memory, run_learner_model_default,
    run_literature_intent_map_default, run_teach_map, ConfidenceMarker, LearnerJournal,
    LearnerJournalConsent, LearnerJournalRefusal, LearnerMemoryConfig, LearnerMemoryRun,
    LearnerMisconceptionObservation, LearnerModelObservation, LearnerModelRefusal, LearnerModelRun,
    LearnerQuizAnswerObservation, LiteratureIntentRefusal, LiteratureIntentRun, TeachMapConfig,
    TeachMapRun, CANONICAL_CONSENT_OPERATOR,
};

const SCHEMA_RECEIPT: &str = "learning-session-receipt-v0.1";
const SCHEMA_STEP: &str = "learning-session-step-v0.1";
const SCHEMA_MATRIX: &str = "learning-session-matrix-v0.1";

const SESSION_USES_MODEL: bool = false;
const SESSION_USES_TRAINING: bool = false;
const SESSION_PERSONALIZES: bool = false;
const SESSION_ACTS_AUTONOMOUSLY: bool = false;

/// The canonical stage order. A session's steps must follow this sequence
/// exactly (the journal stage is present only when an append was consented).
pub const LEARNING_SESSION_STAGES: [&str; 6] = [
    "query_flow",
    "literature_intent",
    "teach_map",
    "learner_model",
    "memory_candidate",
    "journal_append",
];

/// Each stage's authority string — the frozen organ's OWN authority, copied
/// verbatim, never invented here. The journal stage's authority is the
/// scope-bound consent string itself.
const STAGE_AUTHORITIES: [&str; 5] = [
    "verified_candidate_support",
    "intent_map_from_verified_span",
    "teach_from_span_backed_intent_map",
    "learner_state_from_supported_teach_map",
    "memory_candidate_from_learner_state",
];

pub const LEARNING_SESSION_BOUNDARY_LINES: [&str; 10] = [
    "SESSION-LOOP-0 composes existing verified organs.",
    "It adds no new authority.",
    "It does not score, rank, select, grade, or verify content itself.",
    "It does not personalize.",
    "It does not autonomously recall/adapt.",
    "It does not schedule, daemonize, or write new durable state beyond the existing journal append path.",
    "It does not profile health, psych, identity, or hidden diagnosis.",
    "It does not train or run a model.",
    "It does not create truth.",
    "It does not retag v0.1.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearningSessionDecision {
    SessionCompleted,
    SessionRefused,
}

impl LearningSessionDecision {
    pub fn slug(&self) -> &'static str {
        match self {
            LearningSessionDecision::SessionCompleted => "session_completed",
            LearningSessionDecision::SessionRefused => "session_refused",
        }
    }
}

/// Every way a session can be refused. Each variant is CONSTRUCTED in a
/// reachable production path (the A3 fail-closed-debris law).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearningSessionRefusal {
    QueryFlowRefused,
    IntentMapRefused,
    TeachMapRefused,
    LearnerStateRefused,
    MemoryCandidateRefused,
    JournalConsentRefused,
    JournalAppendRefused,
    QuizAnswerUnrecognized,
    UnsupportedSessionStep,
    SessionChainTamper,
    StepReorder,
    ModelSignalDetected,
    TrainingSignalDetected,
    PersonalizationSignalDetected,
    AutonomousAgentSignalDetected,
    MemoryWriteWithoutConsent,
}

impl LearningSessionRefusal {
    pub const ALL: [LearningSessionRefusal; 16] = [
        LearningSessionRefusal::QueryFlowRefused,
        LearningSessionRefusal::IntentMapRefused,
        LearningSessionRefusal::TeachMapRefused,
        LearningSessionRefusal::LearnerStateRefused,
        LearningSessionRefusal::MemoryCandidateRefused,
        LearningSessionRefusal::JournalConsentRefused,
        LearningSessionRefusal::JournalAppendRefused,
        LearningSessionRefusal::QuizAnswerUnrecognized,
        LearningSessionRefusal::UnsupportedSessionStep,
        LearningSessionRefusal::SessionChainTamper,
        LearningSessionRefusal::StepReorder,
        LearningSessionRefusal::ModelSignalDetected,
        LearningSessionRefusal::TrainingSignalDetected,
        LearningSessionRefusal::PersonalizationSignalDetected,
        LearningSessionRefusal::AutonomousAgentSignalDetected,
        LearningSessionRefusal::MemoryWriteWithoutConsent,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            LearningSessionRefusal::QueryFlowRefused => "query_flow_refused",
            LearningSessionRefusal::IntentMapRefused => "intent_map_refused",
            LearningSessionRefusal::TeachMapRefused => "teach_map_refused",
            LearningSessionRefusal::LearnerStateRefused => "learner_state_refused",
            LearningSessionRefusal::MemoryCandidateRefused => "memory_candidate_refused",
            LearningSessionRefusal::JournalConsentRefused => "journal_consent_refused",
            LearningSessionRefusal::JournalAppendRefused => "journal_append_refused",
            LearningSessionRefusal::QuizAnswerUnrecognized => "quiz_answer_unrecognized_refused",
            LearningSessionRefusal::UnsupportedSessionStep => "unsupported_session_step_refused",
            LearningSessionRefusal::SessionChainTamper => "session_chain_tamper_refused",
            LearningSessionRefusal::StepReorder => "step_reorder_refused",
            LearningSessionRefusal::ModelSignalDetected => "model_signal_detected_refused",
            LearningSessionRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            LearningSessionRefusal::PersonalizationSignalDetected => {
                "personalization_signal_detected_refused"
            }
            LearningSessionRefusal::AutonomousAgentSignalDetected => {
                "autonomous_agent_signal_detected_refused"
            }
            LearningSessionRefusal::MemoryWriteWithoutConsent => {
                "memory_write_without_consent_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningSessionError {
    ReplayMismatch,
}

/// Closed-gate config. The four session flags refuse before any organ runs.
/// The two organ passthrough flags reach the frozen organ's OWN signal gate —
/// a true value always refuses (matrix reachability, never capability).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearningSessionConfig {
    pub uses_model: bool,
    pub uses_training: bool,
    pub personalizes: bool,
    pub acts_autonomously: bool,
    pub teach_uses_model: bool,
    pub memory_uses_model: bool,
}

impl LearningSessionConfig {
    pub fn default_config() -> Self {
        LearningSessionConfig {
            uses_model: SESSION_USES_MODEL,
            uses_training: SESSION_USES_TRAINING,
            personalizes: SESSION_PERSONALIZES,
            acts_autonomously: SESSION_ACTS_AUTONOMOUSLY,
            teach_uses_model: false,
            memory_uses_model: false,
        }
    }
}

/// One learning session request: raw documents, a focus question, the
/// learner's explicit observations (exact-match quiz answers, misconception
/// flags, self-reported confidence), and the optional consented journal ask.
#[derive(Debug, Clone, Serialize)]
pub struct LearningSessionRequest {
    pub focus_question: String,
    pub documents: Vec<(String, String)>,
    pub seen_lesson_item_ids: Vec<String>,
    pub quiz_answers: Vec<LearnerQuizAnswerObservation>,
    pub misconception_flags: Vec<LearnerMisconceptionObservation>,
    pub confidence_marker: ConfidenceMarker,
    pub append_to_journal: bool,
    pub journal_consent: Option<LearnerJournalConsent>,
}

/// Structural boundary flags — every flag names a forbidden behavior and must
/// stay false.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct LearningSessionBoundary {
    pub adds_new_authority: bool,
    pub grades_content_itself: bool,
    pub personalizes: bool,
    pub autonomously_recalls: bool,
    pub adapts_behavior: bool,
    pub schedules_or_daemonizes: bool,
    pub writes_memory_without_consent: bool,
    pub profiles_learner: bool,
    pub diagnoses: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub retags_v01: bool,
}

impl LearningSessionBoundary {
    pub fn inert() -> Self {
        LearningSessionBoundary {
            adds_new_authority: false,
            grades_content_itself: false,
            personalizes: SESSION_PERSONALIZES,
            autonomously_recalls: false,
            adapts_behavior: false,
            schedules_or_daemonizes: SESSION_ACTS_AUTONOMOUSLY,
            writes_memory_without_consent: false,
            profiles_learner: false,
            diagnoses: false,
            uses_model: SESSION_USES_MODEL,
            uses_training: SESSION_USES_TRAINING,
            retags_v01: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !(self.adds_new_authority
            || self.grades_content_itself
            || self.personalizes
            || self.autonomously_recalls
            || self.adapts_behavior
            || self.schedules_or_daemonizes
            || self.writes_memory_without_consent
            || self.profiles_learner
            || self.diagnoses
            || self.uses_model
            || self.uses_training
            || self.retags_v01)
    }
}

/// One chained stage record: the organ's receipt hash, its own decision slug,
/// and its own authority string — nothing re-derived, nothing invented.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearningSessionStep {
    pub schema: String,
    pub step_id: u64,
    pub stage: String,
    pub receipt_hash: u64,
    pub decision: String,
    pub authority: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningSessionReceipt {
    pub schema: String,
    pub config: LearningSessionConfig,
    pub question_digest: u64,
    pub step_count: usize,
    pub qflow_receipt_hash: u64,
    pub intent_receipt_hash: u64,
    pub teach_receipt_hash: u64,
    pub learner_receipt_hash: u64,
    pub memory_receipt_hash: u64,
    pub journal_head_before: u64,
    pub journal_head_after: u64,
    pub journal_appended: bool,
    pub quiz_correct: usize,
    pub quiz_incorrect: usize,
    pub quiz_unanswered: usize,
    pub decision: LearningSessionDecision,
    pub refusal: Option<LearningSessionRefusal>,
    pub receipt_hash: u64,
    pub boundary: LearningSessionBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningSessionRun {
    pub receipt: LearningSessionReceipt,
    pub steps: Vec<LearningSessionStep>,
    pub decision: LearningSessionDecision,
    pub refusal: Option<LearningSessionRefusal>,
}

fn fnv_mix(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn fnv_u64(h: u64, v: u64) -> u64 {
    fnv_mix(h, &v.to_le_bytes())
}

fn flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn question_digest(request: &LearningSessionRequest) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, request.focus_question.as_bytes());
    for (name, content) in &request.documents {
        h = fnv_mix(h, name.as_bytes());
        h = fnv_mix(h, content.as_bytes());
    }
    h
}

/// The receipt anchors gathered stage by stage; zeros mean "stage not reached".
#[derive(Debug, Clone, Copy, Default)]
struct StageAnchors {
    qflow: u64,
    intent: u64,
    teach: u64,
    learner: u64,
    memory: u64,
}

struct QuizCounts {
    correct: usize,
    incorrect: usize,
    unanswered: usize,
}

fn quiz_counts(learner: &LearnerModelRun) -> QuizCounts {
    match learner.learner_state.as_ref() {
        Some(state) => QuizCounts {
            correct: state.quiz_result.correct_exact_match,
            incorrect: state.quiz_result.incorrect_exact_mismatch,
            unanswered: state.quiz_result.unanswered,
        },
        None => QuizCounts {
            correct: 0,
            incorrect: 0,
            unanswered: 0,
        },
    }
}

fn fold_step(h: u64, step: &LearningSessionStep) -> u64 {
    let mut h = fnv_mix(h, step.schema.as_bytes());
    h = fnv_u64(h, step.step_id);
    h = fnv_mix(h, step.stage.as_bytes());
    h = fnv_u64(h, step.receipt_hash);
    h = fnv_mix(h, step.decision.as_bytes());
    h = fnv_mix(h, step.authority.as_bytes());
    h
}

#[allow(clippy::too_many_arguments)]
fn fold_receipt_hash(
    config: &LearningSessionConfig,
    digest: u64,
    anchors: &StageAnchors,
    journal_head_before: u64,
    journal_head_after: u64,
    journal_appended: bool,
    quiz: &QuizCounts,
    steps: &[LearningSessionStep],
    decision: LearningSessionDecision,
    refusal: Option<LearningSessionRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_RECEIPT.as_bytes());
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, config.personalizes as u64);
    h = fnv_u64(h, config.acts_autonomously as u64);
    h = fnv_u64(h, config.teach_uses_model as u64);
    h = fnv_u64(h, config.memory_uses_model as u64);
    h = fnv_u64(h, digest);
    h = fnv_u64(h, anchors.qflow);
    h = fnv_u64(h, anchors.intent);
    h = fnv_u64(h, anchors.teach);
    h = fnv_u64(h, anchors.learner);
    h = fnv_u64(h, anchors.memory);
    h = fnv_u64(h, journal_head_before);
    h = fnv_u64(h, journal_head_after);
    h = fnv_u64(h, journal_appended as u64);
    h = fnv_u64(h, quiz.correct as u64);
    h = fnv_u64(h, quiz.incorrect as u64);
    h = fnv_u64(h, quiz.unanswered as u64);
    h = fnv_u64(h, steps.len() as u64);
    for step in steps {
        h = fold_step(h, step);
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

/// Walk a session's steps and refuse the first structural violation: an
/// unknown stage name refuses as an unsupported step; a known stage out of
/// canonical position (or a wrong step_id) refuses as a reorder. A completed
/// session's steps are a prefix of the canonical stage order.
pub fn session_steps_are_chain_ordered(
    steps: &[LearningSessionStep],
) -> Option<LearningSessionRefusal> {
    for (index, step) in steps.iter().enumerate() {
        if !LEARNING_SESSION_STAGES.contains(&step.stage.as_str()) {
            return Some(LearningSessionRefusal::UnsupportedSessionStep);
        }
        if index >= LEARNING_SESSION_STAGES.len()
            || step.stage != LEARNING_SESSION_STAGES[index]
            || step.step_id != index as u64 + 1
        {
            return Some(LearningSessionRefusal::StepReorder);
        }
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn assemble(
    config: LearningSessionConfig,
    digest: u64,
    anchors: StageAnchors,
    journal_head_before: u64,
    journal_head_after: u64,
    journal_appended: bool,
    quiz: QuizCounts,
    steps: Vec<LearningSessionStep>,
    decision: LearningSessionDecision,
    refusal: Option<LearningSessionRefusal>,
) -> LearningSessionRun {
    let boundary = LearningSessionBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let receipt_hash = fold_receipt_hash(
        &config,
        digest,
        &anchors,
        journal_head_before,
        journal_head_after,
        journal_appended,
        &quiz,
        &steps,
        decision,
        refusal,
    );
    LearningSessionRun {
        receipt: LearningSessionReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            question_digest: digest,
            step_count: steps.len(),
            qflow_receipt_hash: anchors.qflow,
            intent_receipt_hash: anchors.intent,
            teach_receipt_hash: anchors.teach,
            learner_receipt_hash: anchors.learner,
            memory_receipt_hash: anchors.memory,
            journal_head_before,
            journal_head_after,
            journal_appended,
            quiz_correct: quiz.correct,
            quiz_incorrect: quiz.incorrect,
            quiz_unanswered: quiz.unanswered,
            decision,
            refusal,
            receipt_hash,
            boundary,
            boundary_all_inert,
        },
        steps,
        decision,
        refusal,
    }
}

fn step(
    step_id: u64,
    stage: &str,
    receipt_hash: u64,
    decision: &str,
    authority: &str,
) -> LearningSessionStep {
    LearningSessionStep {
        schema: SCHEMA_STEP.to_string(),
        step_id,
        stage: stage.to_string(),
        receipt_hash,
        decision: decision.to_string(),
        authority: authority.to_string(),
    }
}

fn build_core_steps(
    intent: &LiteratureIntentRun,
    teach: &TeachMapRun,
    learner: &LearnerModelRun,
    memory: &LearnerMemoryRun,
) -> Vec<LearningSessionStep> {
    vec![
        step(
            1,
            LEARNING_SESSION_STAGES[0],
            intent.receipt.qflow_receipt_hash,
            &intent.receipt.qflow_decision,
            STAGE_AUTHORITIES[0],
        ),
        step(
            2,
            LEARNING_SESSION_STAGES[1],
            intent.receipt.receipt_hash,
            intent.decision.slug(),
            STAGE_AUTHORITIES[1],
        ),
        step(
            3,
            LEARNING_SESSION_STAGES[2],
            teach.receipt.receipt_hash,
            teach.decision.slug(),
            STAGE_AUTHORITIES[2],
        ),
        step(
            4,
            LEARNING_SESSION_STAGES[3],
            learner.receipt.receipt_hash,
            learner.decision.slug(),
            STAGE_AUTHORITIES[3],
        ),
        step(
            5,
            LEARNING_SESSION_STAGES[4],
            memory.receipt.receipt_hash,
            memory.decision.slug(),
            STAGE_AUTHORITIES[4],
        ),
    ]
}

pub fn run_learning_session_default(
    journal: &LearnerJournal,
    request: &LearningSessionRequest,
) -> LearningSessionRun {
    run_learning_session(journal, request, LearningSessionConfig::default_config())
}

/// Compose one deterministic learning session over the frozen organ chain.
/// Pure fold: no I/O, no clock, no entropy, no model.
pub fn run_learning_session(
    journal: &LearnerJournal,
    request: &LearningSessionRequest,
    config: LearningSessionConfig,
) -> LearningSessionRun {
    let digest = question_digest(request);
    let head_before = journal.head_hash;
    let refuse = |anchors: StageAnchors, refusal: LearningSessionRefusal| {
        assemble(
            config,
            digest,
            anchors,
            head_before,
            head_before,
            false,
            QuizCounts {
                correct: 0,
                incorrect: 0,
                unanswered: 0,
            },
            Vec::new(),
            LearningSessionDecision::SessionRefused,
            Some(refusal),
        )
    };
    let signal = if config.uses_model {
        Some(LearningSessionRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(LearningSessionRefusal::TrainingSignalDetected)
    } else if config.personalizes {
        Some(LearningSessionRefusal::PersonalizationSignalDetected)
    } else if config.acts_autonomously {
        Some(LearningSessionRefusal::AutonomousAgentSignalDetected)
    } else {
        None
    };
    if let Some(refusal) = signal {
        return refuse(StageAnchors::default(), refusal);
    }
    let mut anchors = StageAnchors::default();

    // Stages 1-2: QFLOW runs inside LIT-INTENT; its receipt hash and decision
    // surface through the intent receipt.
    let intent = run_literature_intent_map_default(&request.documents, &request.focus_question);
    anchors.qflow = intent.receipt.qflow_receipt_hash;
    anchors.intent = intent.receipt.receipt_hash;
    match intent.refusal {
        Some(LiteratureIntentRefusal::QueryFlowRefused) => {
            return refuse(anchors, LearningSessionRefusal::QueryFlowRefused)
        }
        Some(_) => return refuse(anchors, LearningSessionRefusal::IntentMapRefused),
        None => {}
    }

    // Stage 3: TEACH.
    let mut teach_config = TeachMapConfig::default_config();
    teach_config.uses_model = config.teach_uses_model;
    let teach = run_teach_map(&intent, teach_config);
    anchors.teach = teach.receipt.receipt_hash;
    if teach.refusal.is_some() {
        return refuse(anchors, LearningSessionRefusal::TeachMapRefused);
    }

    // Stage 4: LEARNER-MODEL — the ONLY grader (exact-match law). An
    // unrecognized quiz id refuses distinctly; a wrong answer does not refuse.
    let observation = LearnerModelObservation {
        seen_lesson_item_ids: request.seen_lesson_item_ids.clone(),
        quiz_answers: request.quiz_answers.clone(),
        misconception_flags: request.misconception_flags.clone(),
        confidence_marker: request.confidence_marker,
    };
    let learner = run_learner_model_default(&teach, observation);
    anchors.learner = learner.receipt.receipt_hash;
    match learner.refusal {
        Some(LearnerModelRefusal::UnrecognizedQuizItem) => {
            return refuse(anchors, LearningSessionRefusal::QuizAnswerUnrecognized)
        }
        Some(_) => return refuse(anchors, LearningSessionRefusal::LearnerStateRefused),
        None => {}
    }
    let quiz = quiz_counts(&learner);

    // Stage 5: LEARNER-MEMORY-0 candidate.
    let mut memory_config = LearnerMemoryConfig::default_config();
    memory_config.uses_model = config.memory_uses_model;
    let memory = run_learner_memory(&learner, &intent, memory_config);
    anchors.memory = memory.receipt.receipt_hash;
    if memory.refusal.is_some() {
        return refuse(anchors, LearningSessionRefusal::MemoryCandidateRefused);
    }

    // Stage 6: consented journal append (in-memory fold; never silent).
    let mut steps = build_core_steps(&intent, &teach, &learner, &memory);
    let (head_after, appended) = if request.append_to_journal {
        let consent = match request.journal_consent.as_ref() {
            Some(consent) => consent,
            None => return refuse(anchors, LearningSessionRefusal::MemoryWriteWithoutConsent),
        };
        let journal_run = append_learner_journal_default(journal, &memory, Some(consent));
        match journal_run.refusal {
            Some(LearnerJournalRefusal::MissingConsent)
            | Some(LearnerJournalRefusal::InvalidConsent) => {
                return refuse(anchors, LearningSessionRefusal::JournalConsentRefused)
            }
            Some(_) => return refuse(anchors, LearningSessionRefusal::JournalAppendRefused),
            None => {}
        }
        let appended_journal = journal_run
            .journal
            .as_ref()
            .expect("appended journal present when journal append succeeds");
        steps.push(step(
            6,
            LEARNING_SESSION_STAGES[5],
            journal_run.receipt.receipt_hash,
            journal_run.decision.slug(),
            &consent.journal_scope,
        ));
        (appended_journal.head_hash, true)
    } else {
        (head_before, false)
    };

    // Self-check: the composed steps must satisfy the same chain-order law the
    // matrix uses against forged step vectors.
    if let Some(refusal) = session_steps_are_chain_ordered(&steps) {
        return refuse(anchors, refusal);
    }

    assemble(
        config,
        digest,
        anchors,
        head_before,
        head_after,
        appended,
        quiz,
        steps,
        LearningSessionDecision::SessionCompleted,
        None,
    )
}

/// The canonical demo request: the fixture documents and question (reused
/// through the LIT-INTENT demo's pub request), the canonical observations
/// re-derived from the LEARNER-MODEL demo state (so the whole chain lands on
/// the canonical candidate), and a consented journal append.
pub fn learning_session_demo_request() -> LearningSessionRequest {
    let fixture = literature_intent_demo().request;
    let learner = learner_model_demo();
    let state = learner
        .learner_state
        .as_ref()
        .expect("canonical learner state");
    let seen_lesson_item_ids = state
        .seen_items
        .iter()
        .map(|item| item.item_id.clone())
        .collect::<Vec<_>>();
    let quiz_answers = state
        .quiz_result
        .items
        .iter()
        .filter_map(|item| {
            item.observed_answer
                .as_ref()
                .map(|answer| LearnerQuizAnswerObservation {
                    quiz_id: item.quiz_id.clone(),
                    answer: answer.clone(),
                })
        })
        .collect::<Vec<_>>();
    let misconception_flags = state
        .misconception_flags
        .iter()
        .map(|flag| LearnerMisconceptionObservation {
            check_id: flag.check_id.clone(),
            flagged: flag.flagged,
        })
        .collect::<Vec<_>>();
    let consent = LearnerJournalConsent {
        operator: CANONICAL_CONSENT_OPERATOR.to_string(),
        journal_scope: journal_scope_for_candidate(&learner_memory_demo()),
        consents_to_append: true,
    };
    LearningSessionRequest {
        focus_question: fixture.focus_question,
        documents: fixture.documents,
        seen_lesson_item_ids,
        quiz_answers,
        misconception_flags,
        confidence_marker: state.confidence_marker.marker,
        append_to_journal: true,
        journal_consent: Some(consent),
    }
}

/// The canonical SESSION-LOOP-0 demo: the full six-stage spine from an empty
/// journal, ending in a consented append whose head must advance.
pub fn learning_session_demo() -> LearningSessionRun {
    run_learning_session_default(
        &crate::empty_learner_journal(),
        &learning_session_demo_request(),
    )
}

pub fn learning_session_demo_json() -> String {
    serde_json::to_string_pretty(&learning_session_demo())
        .expect("learning session demo serializes")
}

pub fn verify_learning_session_demo_json(candidate: &str) -> Result<(), LearningSessionError> {
    if candidate == learning_session_demo_json() {
        Ok(())
    } else {
        Err(LearningSessionError::ReplayMismatch)
    }
}

pub const LEARNING_SESSION_SCENARIO_COUNT: usize = 19;
pub const LEARNING_SESSION_SCENARIO_NAMES: [&str; LEARNING_SESSION_SCENARIO_COUNT] = [
    "session_completed_with_journal_append",
    "session_completed_without_journal",
    "incorrect_quiz_answer_still_completes",
    "query_flow_refused",
    "intent_map_refused",
    "teach_map_refused",
    "learner_state_refused",
    "quiz_answer_unrecognized_refused",
    "memory_candidate_refused",
    "memory_write_without_consent_refused",
    "journal_consent_refused",
    "journal_append_refused",
    "step_reorder_refused",
    "unsupported_session_step_refused",
    "session_chain_tamper_refused",
    "model_signal_refused",
    "training_signal_refused",
    "personalization_signal_refused",
    "autonomous_agent_signal_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct LearningSessionCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub step_count: usize,
    pub journal_appended: bool,
    pub journal_head_advanced: bool,
    pub quiz_correct: usize,
    pub quiz_incorrect: usize,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningSessionMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<LearningSessionCell>,
    pub completed_count: usize,
    pub refused_count: usize,
    pub boundary: LearningSessionBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_run(scenario: &str, run: &LearningSessionRun) -> LearningSessionCell {
    LearningSessionCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        step_count: run.receipt.step_count,
        journal_appended: run.receipt.journal_appended,
        journal_head_advanced: run.receipt.journal_head_after != run.receipt.journal_head_before,
        quiz_correct: run.receipt.quiz_correct,
        quiz_incorrect: run.receipt.quiz_incorrect,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn cell_from_guard(scenario: &str, refusal: Option<LearningSessionRefusal>) -> LearningSessionCell {
    LearningSessionCell {
        scenario: scenario.to_string(),
        outcome: match refusal {
            Some(_) => "session_refused".to_string(),
            None => "violation_missed".to_string(),
        },
        refusal: refusal.map(|r| r.slug().to_string()),
        step_count: 0,
        journal_appended: false,
        journal_head_advanced: false,
        quiz_correct: 0,
        quiz_incorrect: 0,
        boundary_all_inert: LearningSessionBoundary::inert().all_inert(),
    }
}

fn run_demo_variant(mutate: impl FnOnce(&mut LearningSessionRequest)) -> LearningSessionRun {
    let mut request = learning_session_demo_request();
    mutate(&mut request);
    run_learning_session_default(&crate::empty_learner_journal(), &request)
}

fn signal_cell(scenario: &str, set: fn(&mut LearningSessionConfig)) -> LearningSessionCell {
    let mut config = LearningSessionConfig::default_config();
    set(&mut config);
    let run = run_learning_session(
        &crate::empty_learner_journal(),
        &learning_session_demo_request(),
        config,
    );
    cell_from_run(scenario, &run)
}

fn cell_for(scenario: &str) -> LearningSessionCell {
    match scenario {
        "session_completed_with_journal_append" => {
            cell_from_run(scenario, &learning_session_demo())
        }
        "session_completed_without_journal" => {
            let run = run_demo_variant(|request| {
                request.append_to_journal = false;
                request.journal_consent = None;
            });
            cell_from_run(scenario, &run)
        }
        "incorrect_quiz_answer_still_completes" => {
            // Keep only the exact-mismatch answer: the session must COMPLETE
            // with the wrong answer recorded, never refuse it.
            let run = run_demo_variant(|request| {
                request.quiz_answers.retain(|a| a.quiz_id == "quiz:2");
                request.append_to_journal = false;
                request.journal_consent = None;
            });
            cell_from_run(scenario, &run)
        }
        "query_flow_refused" => {
            let run = run_demo_variant(|request| {
                request.documents = vec![("empty.md".to_string(), String::new())];
            });
            cell_from_run(scenario, &run)
        }
        "intent_map_refused" => {
            let run = run_demo_variant(|request| {
                request.focus_question = String::new();
            });
            cell_from_run(scenario, &run)
        }
        "teach_map_refused" => {
            let mut config = LearningSessionConfig::default_config();
            config.teach_uses_model = true;
            let run = run_learning_session(
                &crate::empty_learner_journal(),
                &learning_session_demo_request(),
                config,
            );
            cell_from_run(scenario, &run)
        }
        "learner_state_refused" => {
            let run = run_demo_variant(|request| {
                request.seen_lesson_item_ids = vec!["no_such_lesson_item".to_string()];
            });
            cell_from_run(scenario, &run)
        }
        "quiz_answer_unrecognized_refused" => {
            let run = run_demo_variant(|request| {
                request.quiz_answers.push(LearnerQuizAnswerObservation {
                    quiz_id: "quiz:999".to_string(),
                    answer: "anything".to_string(),
                });
            });
            cell_from_run(scenario, &run)
        }
        "memory_candidate_refused" => {
            let mut config = LearningSessionConfig::default_config();
            config.memory_uses_model = true;
            let run = run_learning_session(
                &crate::empty_learner_journal(),
                &learning_session_demo_request(),
                config,
            );
            cell_from_run(scenario, &run)
        }
        "memory_write_without_consent_refused" => {
            let run = run_demo_variant(|request| {
                request.journal_consent = None;
            });
            cell_from_run(scenario, &run)
        }
        "journal_consent_refused" => {
            let run = run_demo_variant(|request| {
                if let Some(consent) = request.journal_consent.as_mut() {
                    consent.journal_scope = "learner_memory_receipt:0000000000000000".to_string();
                }
            });
            cell_from_run(scenario, &run)
        }
        "journal_append_refused" => {
            // The starting journal already holds the canonical candidate: the
            // consented re-append must refuse structurally (duplicate entry).
            let journal = crate::learner_journal_at(1).expect("canonical journal");
            let run = run_learning_session_default(&journal, &learning_session_demo_request());
            cell_from_run(scenario, &run)
        }
        "step_reorder_refused" => {
            let mut steps = learning_session_demo().steps;
            steps.swap(0, 1);
            cell_from_guard(scenario, session_steps_are_chain_ordered(&steps))
        }
        "unsupported_session_step_refused" => {
            let mut steps = learning_session_demo().steps;
            steps[0].stage = "autonomous_planner".to_string();
            cell_from_guard(scenario, session_steps_are_chain_ordered(&steps))
        }
        "session_chain_tamper_refused" => {
            // Serialize the real session artifact, flip one byte, and confirm
            // the tamper is detectable — constructing the refusal that names
            // this scenario (the established A3 precedent).
            let json = learning_session_demo_json();
            let refused = verify_learning_session_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(LearningSessionRefusal::SessionChainTamper)
            } else {
                None
            };
            LearningSessionCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: refusal.map(|r| r.slug().to_string()),
                step_count: 0,
                journal_appended: false,
                journal_head_advanced: false,
                quiz_correct: 0,
                quiz_incorrect: 0,
                boundary_all_inert: LearningSessionBoundary::inert().all_inert(),
            }
        }
        "model_signal_refused" => signal_cell(scenario, |c| c.uses_model = true),
        "training_signal_refused" => signal_cell(scenario, |c| c.uses_training = true),
        "personalization_signal_refused" => signal_cell(scenario, |c| c.personalizes = true),
        "autonomous_agent_signal_refused" => signal_cell(scenario, |c| c.acts_autonomously = true),
        other => LearningSessionCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            step_count: 0,
            journal_appended: false,
            journal_head_advanced: false,
            quiz_correct: 0,
            quiz_incorrect: 0,
            boundary_all_inert: false,
        },
    }
}

pub fn learning_session_matrix() -> LearningSessionMatrix {
    let cells = LEARNING_SESSION_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let completed_count = cells
        .iter()
        .filter(|cell| cell.outcome == "session_completed")
        .count();
    let refused_count = cells.len() - completed_count;
    let boundary = LearningSessionBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    LearningSessionMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        completed_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn learning_session_matrix_json() -> String {
    serde_json::to_string_pretty(&learning_session_matrix())
        .expect("learning session matrix serializes")
}

pub fn verify_learning_session_matrix_json(candidate: &str) -> Result<(), LearningSessionError> {
    if candidate == learning_session_matrix_json() {
        Ok(())
    } else {
        Err(LearningSessionError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_completes_full_spine_with_journal_append() {
        let run = learning_session_demo();
        assert_eq!(run.decision, LearningSessionDecision::SessionCompleted);
        assert!(run.refusal.is_none());
        assert_eq!(run.steps.len(), 6);
        assert!(run.receipt.journal_appended);
        assert_ne!(
            run.receipt.journal_head_after, run.receipt.journal_head_before,
            "the demo journal head must advance"
        );
        assert!(run.receipt.boundary_all_inert);
    }

    #[test]
    fn demo_steps_follow_canonical_stage_order() {
        let run = learning_session_demo();
        for (index, step) in run.steps.iter().enumerate() {
            assert_eq!(step.stage, LEARNING_SESSION_STAGES[index]);
            assert_eq!(step.step_id, index as u64 + 1);
        }
        assert!(session_steps_are_chain_ordered(&run.steps).is_none());
    }

    #[test]
    fn demo_lands_on_canonical_organ_receipts() {
        // The session composes the SAME canonical chain the organs pin
        // individually — a cross-organ consistency proof.
        let run = learning_session_demo();
        let intent = literature_intent_demo();
        let learner = learner_model_demo();
        let memory = learner_memory_demo();
        assert_eq!(run.receipt.intent_receipt_hash, intent.receipt.receipt_hash);
        assert_eq!(
            run.receipt.qflow_receipt_hash,
            intent.receipt.qflow_receipt_hash
        );
        assert_eq!(
            run.receipt.learner_receipt_hash,
            learner.receipt.receipt_hash
        );
        assert_eq!(run.receipt.memory_receipt_hash, memory.receipt.receipt_hash);
        let canonical_journal = crate::learner_journal_at(1).expect("canonical journal");
        assert_eq!(run.receipt.journal_head_after, canonical_journal.head_hash);
    }

    #[test]
    fn demo_records_exact_match_quiz_counts() {
        let run = learning_session_demo();
        assert_eq!(run.receipt.quiz_correct, 1);
        assert_eq!(run.receipt.quiz_incorrect, 1);
        assert_eq!(run.receipt.quiz_unanswered, 1);
    }

    #[test]
    fn no_append_session_completes_with_unchanged_head() {
        let run = run_demo_variant(|request| {
            request.append_to_journal = false;
            request.journal_consent = None;
        });
        assert_eq!(run.decision, LearningSessionDecision::SessionCompleted);
        assert!(!run.receipt.journal_appended);
        assert_eq!(
            run.receipt.journal_head_after,
            run.receipt.journal_head_before
        );
        assert_eq!(run.steps.len(), 5);
    }

    #[test]
    fn incorrect_answer_completes_and_is_recorded() {
        let run = run_demo_variant(|request| {
            request.quiz_answers.retain(|a| a.quiz_id == "quiz:2");
            request.append_to_journal = false;
            request.journal_consent = None;
        });
        assert_eq!(run.decision, LearningSessionDecision::SessionCompleted);
        assert_eq!(run.receipt.quiz_incorrect, 1);
        assert_eq!(run.receipt.quiz_correct, 0);
        assert_eq!(run.receipt.quiz_unanswered, 2);
    }

    #[test]
    fn unknown_quiz_id_is_refused_distinctly() {
        let run = run_demo_variant(|request| {
            request.quiz_answers.push(LearnerQuizAnswerObservation {
                quiz_id: "quiz:999".to_string(),
                answer: "anything".to_string(),
            });
        });
        assert_eq!(
            run.refusal,
            Some(LearningSessionRefusal::QuizAnswerUnrecognized)
        );
    }

    #[test]
    fn query_flow_refusal_propagates_distinctly() {
        let run = run_demo_variant(|request| {
            request.documents = vec![("empty.md".to_string(), String::new())];
        });
        assert_eq!(run.refusal, Some(LearningSessionRefusal::QueryFlowRefused));
    }

    #[test]
    fn empty_question_refuses_as_intent_map() {
        let run = run_demo_variant(|request| {
            request.focus_question = String::new();
        });
        assert_eq!(run.refusal, Some(LearningSessionRefusal::IntentMapRefused));
    }

    #[test]
    fn teach_refusal_propagates() {
        let mut config = LearningSessionConfig::default_config();
        config.teach_uses_model = true;
        let run = run_learning_session(
            &crate::empty_learner_journal(),
            &learning_session_demo_request(),
            config,
        );
        assert_eq!(run.refusal, Some(LearningSessionRefusal::TeachMapRefused));
    }

    #[test]
    fn unknown_seen_item_refuses_learner_state() {
        let run = run_demo_variant(|request| {
            request.seen_lesson_item_ids = vec!["no_such_lesson_item".to_string()];
        });
        assert_eq!(
            run.refusal,
            Some(LearningSessionRefusal::LearnerStateRefused)
        );
    }

    #[test]
    fn memory_refusal_propagates() {
        let mut config = LearningSessionConfig::default_config();
        config.memory_uses_model = true;
        let run = run_learning_session(
            &crate::empty_learner_journal(),
            &learning_session_demo_request(),
            config,
        );
        assert_eq!(
            run.refusal,
            Some(LearningSessionRefusal::MemoryCandidateRefused)
        );
    }

    #[test]
    fn append_without_consent_is_refused() {
        let run = run_demo_variant(|request| {
            request.journal_consent = None;
        });
        assert_eq!(
            run.refusal,
            Some(LearningSessionRefusal::MemoryWriteWithoutConsent)
        );
        assert!(!run.receipt.journal_appended);
    }

    #[test]
    fn bad_consent_scope_is_refused() {
        let run = run_demo_variant(|request| {
            if let Some(consent) = request.journal_consent.as_mut() {
                consent.journal_scope = "learner_memory_receipt:0000000000000000".to_string();
            }
        });
        assert_eq!(
            run.refusal,
            Some(LearningSessionRefusal::JournalConsentRefused)
        );
    }

    #[test]
    fn structural_journal_refusal_propagates() {
        let journal = crate::learner_journal_at(1).expect("canonical journal");
        let run = run_learning_session_default(&journal, &learning_session_demo_request());
        assert_eq!(
            run.refusal,
            Some(LearningSessionRefusal::JournalAppendRefused)
        );
        assert_eq!(run.receipt.journal_head_after, journal.head_hash);
    }

    #[test]
    fn reordered_steps_are_refused() {
        let mut steps = learning_session_demo().steps;
        steps.swap(0, 1);
        assert_eq!(
            session_steps_are_chain_ordered(&steps),
            Some(LearningSessionRefusal::StepReorder)
        );
    }

    #[test]
    fn unknown_stage_is_refused() {
        let mut steps = learning_session_demo().steps;
        steps[0].stage = "autonomous_planner".to_string();
        assert_eq!(
            session_steps_are_chain_ordered(&steps),
            Some(LearningSessionRefusal::UnsupportedSessionStep)
        );
    }

    #[test]
    fn every_signal_config_refuses_before_any_organ_runs() {
        type SignalCase = (fn(&mut LearningSessionConfig), LearningSessionRefusal);
        let cases: [SignalCase; 4] = [
            (
                |c| c.uses_model = true,
                LearningSessionRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                LearningSessionRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.personalizes = true,
                LearningSessionRefusal::PersonalizationSignalDetected,
            ),
            (
                |c| c.acts_autonomously = true,
                LearningSessionRefusal::AutonomousAgentSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = LearningSessionConfig::default_config();
            set(&mut config);
            let run = run_learning_session(
                &crate::empty_learner_journal(),
                &learning_session_demo_request(),
                config,
            );
            assert_eq!(run.refusal, Some(expected));
            assert_eq!(run.receipt.qflow_receipt_hash, 0, "no organ may run");
        }
    }

    #[test]
    fn receipt_folds_journal_head_progression() {
        let with_append = learning_session_demo();
        let without_append = run_demo_variant(|request| {
            request.append_to_journal = false;
            request.journal_consent = None;
        });
        assert_ne!(
            with_append.receipt.receipt_hash,
            without_append.receipt.receipt_hash
        );
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = learning_session_demo_json();
        assert!(verify_learning_session_demo_json(&json).is_ok());
        assert_eq!(
            verify_learning_session_demo_json(&flip_last_byte(&json)),
            Err(LearningSessionError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = learning_session_matrix_json();
        assert!(verify_learning_session_matrix_json(&json).is_ok());
        assert_eq!(
            verify_learning_session_matrix_json(&flip_last_byte(&json)),
            Err(LearningSessionError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = learning_session_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|cell| cell.scenario == "session_chain_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("session_chain_tamper_refused")
        );
    }

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = learning_session_matrix();
        assert_eq!(matrix.scenario_count, LEARNING_SESSION_SCENARIO_COUNT);
        assert_eq!(matrix.completed_count, 3);
        let constructed = matrix
            .cells
            .iter()
            .filter_map(|cell| cell.refusal.clone())
            .collect::<Vec<_>>();
        for refusal in LearningSessionRefusal::ALL {
            assert!(
                constructed.iter().any(|slug| slug == refusal.slug()),
                "refusal {} must be constructed by a matrix scenario",
                refusal.slug()
            );
        }
        assert!(matrix
            .cells
            .iter()
            .all(|cell| cell.outcome != "unknown" && cell.outcome != "violation_missed"));
    }

    #[test]
    fn boundary_lines_and_flags_stay_inert() {
        assert_eq!(LEARNING_SESSION_BOUNDARY_LINES.len(), 10);
        let boundary = LearningSessionBoundary::inert();
        assert!(boundary.all_inert());
        let mut broken = boundary;
        broken.adds_new_authority = true;
        assert!(!broken.all_inert());
    }
}
