//! MULTI-SESSION-0: the canonical multi-session learning arc.
//!
//! This module composes the two canonical committed learning sessions into ONE
//! receipt-linked arc, proving the safe multi-session shape:
//!
//! ```text
//! session 1 (candidate A, from the empty journal)
//!   -> journal head 1
//! session 2 (candidate B, starting FROM journal head 1)
//!   -> journal head 2
//! ```
//!
//! Canonical-only law (Option A): the arc threads the CANONICAL journal states
//! (`learner_journal_at(n)`) between sessions and verifies head continuity at
//! every seam. It does not generalize arbitrary journals — `LearningSessionRun`
//! deliberately does not return the appended journal value, and this gate does
//! not change that.
//!
//! Non-adaptation law: session N+1's content is UNCHANGED by session N. Both
//! canonical sessions run the identical frozen evidence/intent/teach chain —
//! the arc records growth ACROSS sessions (the journal grows) without adapting
//! anything BETWEEN sessions. A test pins the content anchors equal.
//!
//! Authority law: the arc composes session receipts and adds no authority. It
//! does not grade, personalize, recall, or adapt. Consent stays per-session and
//! scope-bound; the arc never synthesizes a consent. No durable I/O anywhere —
//! file persistence remains solely behind the `learner-journal-append` verb.
//!
//! Propagation honesty: a session refused at its journal-append stage maps to
//! [`LearningArcRefusal::DuplicateSessionRefused`] because on CANONICAL
//! journals the only reachable append failure is a duplicate candidate (chain
//! violations cannot occur in re-derived canonical states). Consent failures
//! map distinctly; every other session refusal maps to `SessionRefused` with
//! the failing session index recorded on the receipt.

use serde::Serialize;

use crate::{
    empty_learner_journal, journal_scope_for_candidate, learner_journal_at, learner_model_demo,
    learning_session_demo_request, literature_intent_demo, run_learner_memory_default,
    run_learner_model_default, run_learning_session_default, teach_map_demo, ConfidenceMarker,
    LearnerJournalConsent, LearnerModelObservation, LearnerQuizAnswerObservation,
    LearningSessionRefusal, LearningSessionRequest, LearningSessionRun, CANONICAL_CONSENT_OPERATOR,
};

const SCHEMA_RECEIPT: &str = "learning-arc-receipt-v0.1";
const SCHEMA_STEP: &str = "learning-arc-step-v0.1";
const SCHEMA_MATRIX: &str = "learning-arc-matrix-v0.1";

const ARC_USES_MODEL: bool = false;
const ARC_USES_TRAINING: bool = false;
const ARC_PERSONALIZES: bool = false;
const ARC_AUTONOMOUSLY_ADAPTS: bool = false;

/// The canonical arc runs exactly the two canonical sessions.
pub const LEARNING_ARC_SESSION_COUNT: usize = 2;

/// The canonical arc stage order; a completed arc's steps must match exactly.
pub const LEARNING_ARC_STAGES: [&str; LEARNING_ARC_SESSION_COUNT] = ["session_1", "session_2"];

pub const LEARNING_ARC_BOUNDARY_LINES: [&str; 9] = [
    "MULTI-SESSION-0 records a deterministic multi-session learning arc.",
    "It proves journal continuity across sessions.",
    "It does not personalize generation.",
    "It does not adapt behavior across sessions.",
    "It does not recall autonomously.",
    "It does not write durable memory.",
    "It does not generalize arbitrary journals yet.",
    "It does not train or run a model.",
    "It does not retag v0.1.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearningArcDecision {
    ArcCompleted,
    ArcRefused,
}

impl LearningArcDecision {
    pub fn slug(&self) -> &'static str {
        match self {
            LearningArcDecision::ArcCompleted => "arc_completed",
            LearningArcDecision::ArcRefused => "arc_refused",
        }
    }
}

/// Every way an arc can be refused. Each variant is CONSTRUCTED in a reachable
/// production path (the A3 fail-closed-debris law).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearningArcRefusal {
    SessionRefused,
    SessionConsentRefused,
    DuplicateSessionRefused,
    ArcChainBreak,
    ArcStepReorder,
    UnsupportedArcStep,
    SerializedLearningArcTamper,
    ModelSignalDetected,
    TrainingSignalDetected,
    PersonalizationSignalDetected,
    AutonomousAdaptationSignalDetected,
}

impl LearningArcRefusal {
    pub const ALL: [LearningArcRefusal; 11] = [
        LearningArcRefusal::SessionRefused,
        LearningArcRefusal::SessionConsentRefused,
        LearningArcRefusal::DuplicateSessionRefused,
        LearningArcRefusal::ArcChainBreak,
        LearningArcRefusal::ArcStepReorder,
        LearningArcRefusal::UnsupportedArcStep,
        LearningArcRefusal::SerializedLearningArcTamper,
        LearningArcRefusal::ModelSignalDetected,
        LearningArcRefusal::TrainingSignalDetected,
        LearningArcRefusal::PersonalizationSignalDetected,
        LearningArcRefusal::AutonomousAdaptationSignalDetected,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            LearningArcRefusal::SessionRefused => "session_refused",
            LearningArcRefusal::SessionConsentRefused => "session_consent_refused",
            LearningArcRefusal::DuplicateSessionRefused => "duplicate_session_refused",
            LearningArcRefusal::ArcChainBreak => "arc_chain_break_refused",
            LearningArcRefusal::ArcStepReorder => "arc_step_reorder_refused",
            LearningArcRefusal::UnsupportedArcStep => "unsupported_arc_step_refused",
            LearningArcRefusal::SerializedLearningArcTamper => {
                "serialized_learning_arc_tamper_refused"
            }
            LearningArcRefusal::ModelSignalDetected => "model_signal_detected_refused",
            LearningArcRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            LearningArcRefusal::PersonalizationSignalDetected => {
                "personalization_signal_detected_refused"
            }
            LearningArcRefusal::AutonomousAdaptationSignalDetected => {
                "autonomous_adaptation_signal_detected_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningArcError {
    ReplayMismatch,
}

/// Closed-gate config: any true flag refuses before any session runs.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearningArcConfig {
    pub uses_model: bool,
    pub uses_training: bool,
    pub personalizes: bool,
    pub autonomously_adapts: bool,
}

impl LearningArcConfig {
    pub fn default_config() -> Self {
        LearningArcConfig {
            uses_model: ARC_USES_MODEL,
            uses_training: ARC_USES_TRAINING,
            personalizes: ARC_PERSONALIZES,
            autonomously_adapts: ARC_AUTONOMOUSLY_ADAPTS,
        }
    }
}

/// Structural boundary flags — every flag names a forbidden behavior and must
/// stay false.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct LearningArcBoundary {
    pub personalizes_generation: bool,
    pub adapts_across_sessions: bool,
    pub autonomously_recalls: bool,
    pub writes_durable_memory: bool,
    pub generalizes_arbitrary_journals: bool,
    pub creates_new_authority: bool,
    pub grades_content_itself: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub retags_v01: bool,
}

impl LearningArcBoundary {
    pub fn inert() -> Self {
        LearningArcBoundary {
            personalizes_generation: ARC_PERSONALIZES,
            adapts_across_sessions: ARC_AUTONOMOUSLY_ADAPTS,
            autonomously_recalls: false,
            writes_durable_memory: false,
            generalizes_arbitrary_journals: false,
            creates_new_authority: false,
            grades_content_itself: false,
            uses_model: ARC_USES_MODEL,
            uses_training: ARC_USES_TRAINING,
            retags_v01: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !(self.personalizes_generation
            || self.adapts_across_sessions
            || self.autonomously_recalls
            || self.writes_durable_memory
            || self.generalizes_arbitrary_journals
            || self.creates_new_authority
            || self.grades_content_itself
            || self.uses_model
            || self.uses_training
            || self.retags_v01)
    }
}

/// One arc step: a session's receipt hash plus the journal seam it crossed.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearningArcStep {
    pub schema: String,
    pub step_id: u64,
    pub stage: String,
    pub session_receipt_hash: u64,
    pub journal_head_before: u64,
    pub journal_head_after: u64,
    pub journal_appended: bool,
    pub decision: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningArcReceipt {
    pub schema: String,
    pub config: LearningArcConfig,
    pub session_count: usize,
    pub journal_head_genesis: u64,
    pub journal_head_after_session_1: u64,
    pub journal_head_after_session_2: u64,
    pub session_1_receipt_hash: u64,
    pub session_2_receipt_hash: u64,
    pub refused_session_index: Option<u64>,
    pub decision: LearningArcDecision,
    pub refusal: Option<LearningArcRefusal>,
    pub receipt_hash: u64,
    pub boundary: LearningArcBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningArcRun {
    pub receipt: LearningArcReceipt,
    pub steps: Vec<LearningArcStep>,
    pub decision: LearningArcDecision,
    pub refusal: Option<LearningArcRefusal>,
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

fn fold_step(h: u64, step: &LearningArcStep) -> u64 {
    let mut h = fnv_mix(h, step.schema.as_bytes());
    h = fnv_u64(h, step.step_id);
    h = fnv_mix(h, step.stage.as_bytes());
    h = fnv_u64(h, step.session_receipt_hash);
    h = fnv_u64(h, step.journal_head_before);
    h = fnv_u64(h, step.journal_head_after);
    h = fnv_u64(h, step.journal_appended as u64);
    h = fnv_mix(h, step.decision.as_bytes());
    h
}

struct ArcAnchors {
    genesis: u64,
    head_1: u64,
    head_2: u64,
    session_1: u64,
    session_2: u64,
}

#[allow(clippy::too_many_arguments)]
fn fold_receipt_hash(
    config: &LearningArcConfig,
    anchors: &ArcAnchors,
    refused_session_index: Option<u64>,
    steps: &[LearningArcStep],
    decision: LearningArcDecision,
    refusal: Option<LearningArcRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_RECEIPT.as_bytes());
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, config.personalizes as u64);
    h = fnv_u64(h, config.autonomously_adapts as u64);
    h = fnv_u64(h, anchors.genesis);
    h = fnv_u64(h, anchors.head_1);
    h = fnv_u64(h, anchors.head_2);
    h = fnv_u64(h, anchors.session_1);
    h = fnv_u64(h, anchors.session_2);
    h = fnv_u64(h, refused_session_index.unwrap_or(0));
    h = fnv_u64(h, steps.len() as u64);
    for step in steps {
        h = fold_step(h, step);
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

/// Walk an arc's steps and refuse the first structural violation: an unknown
/// stage refuses as an unsupported step; a known stage out of canonical order
/// (or wrong step_id) refuses as a reorder; a seam where a step's starting head
/// is not the previous step's ending head (or where the first step does not
/// start at the genesis head) refuses as a chain break.
pub fn arc_steps_are_chain_linked(steps: &[LearningArcStep]) -> Option<LearningArcRefusal> {
    for (index, step) in steps.iter().enumerate() {
        if !LEARNING_ARC_STAGES.contains(&step.stage.as_str()) {
            return Some(LearningArcRefusal::UnsupportedArcStep);
        }
        if index >= LEARNING_ARC_STAGES.len()
            || step.stage != LEARNING_ARC_STAGES[index]
            || step.step_id != index as u64 + 1
        {
            return Some(LearningArcRefusal::ArcStepReorder);
        }
    }
    let mut expected_head = empty_learner_journal().head_hash;
    for step in steps {
        if step.journal_head_before != expected_head {
            return Some(LearningArcRefusal::ArcChainBreak);
        }
        expected_head = step.journal_head_after;
    }
    None
}

fn map_session_refusal(refusal: LearningSessionRefusal) -> LearningArcRefusal {
    match refusal {
        LearningSessionRefusal::MemoryWriteWithoutConsent
        | LearningSessionRefusal::JournalConsentRefused => {
            LearningArcRefusal::SessionConsentRefused
        }
        // On canonical journals the only reachable append failure is a
        // duplicate candidate; see the module docs for the honesty caveat.
        LearningSessionRefusal::JournalAppendRefused => LearningArcRefusal::DuplicateSessionRefused,
        _ => LearningArcRefusal::SessionRefused,
    }
}

fn step_from_session(step_id: u64, stage: &str, session: &LearningSessionRun) -> LearningArcStep {
    LearningArcStep {
        schema: SCHEMA_STEP.to_string(),
        step_id,
        stage: stage.to_string(),
        session_receipt_hash: session.receipt.receipt_hash,
        journal_head_before: session.receipt.journal_head_before,
        journal_head_after: session.receipt.journal_head_after,
        journal_appended: session.receipt.journal_appended,
        decision: session.decision.slug().to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn assemble(
    config: LearningArcConfig,
    anchors: ArcAnchors,
    refused_session_index: Option<u64>,
    steps: Vec<LearningArcStep>,
    decision: LearningArcDecision,
    refusal: Option<LearningArcRefusal>,
) -> LearningArcRun {
    let boundary = LearningArcBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let receipt_hash = fold_receipt_hash(
        &config,
        &anchors,
        refused_session_index,
        &steps,
        decision,
        refusal,
    );
    LearningArcRun {
        receipt: LearningArcReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            session_count: steps.len(),
            journal_head_genesis: anchors.genesis,
            journal_head_after_session_1: anchors.head_1,
            journal_head_after_session_2: anchors.head_2,
            session_1_receipt_hash: anchors.session_1,
            session_2_receipt_hash: anchors.session_2,
            refused_session_index,
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

/// The canonical second-session request: the same fixture documents and
/// question, observed with every lesson item seen, no quiz answers, no
/// misconception flags, and an unstated confidence marker — the LEARNER-MEMORY-1
/// candidate-B observation expressed as a session request, consented to the
/// candidate-B scope. Reconstructed entirely from the pub organ surface.
pub fn learning_arc_second_request() -> LearningSessionRequest {
    let fixture = literature_intent_demo().request;
    let base = learner_model_demo();
    let seen_lesson_item_ids = base
        .learner_state
        .as_ref()
        .expect("canonical learner state")
        .seen_items
        .iter()
        .map(|item| item.item_id.clone())
        .collect::<Vec<_>>();
    let observation = LearnerModelObservation {
        seen_lesson_item_ids: seen_lesson_item_ids.clone(),
        quiz_answers: Vec::new(),
        misconception_flags: Vec::new(),
        confidence_marker: ConfidenceMarker::Unstated,
    };
    let learner_b = run_learner_model_default(&teach_map_demo(), observation);
    let candidate_b = run_learner_memory_default(&learner_b, &literature_intent_demo());
    LearningSessionRequest {
        focus_question: fixture.focus_question,
        documents: fixture.documents,
        seen_lesson_item_ids,
        quiz_answers: Vec::new(),
        misconception_flags: Vec::new(),
        confidence_marker: ConfidenceMarker::Unstated,
        append_to_journal: true,
        journal_consent: Some(LearnerJournalConsent {
            operator: CANONICAL_CONSENT_OPERATOR.to_string(),
            journal_scope: journal_scope_for_candidate(&candidate_b),
            consents_to_append: true,
        }),
    }
}

pub fn run_learning_arc_default() -> LearningArcRun {
    run_learning_arc(LearningArcConfig::default_config())
}

/// Compose the two canonical sessions into one receipt-linked arc, threading
/// the canonical journal states between them and verifying head continuity at
/// every seam. Pure fold: no I/O, no clock, no entropy, no model.
pub fn run_learning_arc(config: LearningArcConfig) -> LearningArcRun {
    run_learning_arc_over(
        &learning_session_demo_request(),
        &learning_arc_second_request(),
        config,
    )
}

fn run_learning_arc_over(
    first_request: &LearningSessionRequest,
    second_request: &LearningSessionRequest,
    config: LearningArcConfig,
) -> LearningArcRun {
    let genesis = empty_learner_journal().head_hash;
    let mut anchors = ArcAnchors {
        genesis,
        head_1: 0,
        head_2: 0,
        session_1: 0,
        session_2: 0,
    };
    let signal = if config.uses_model {
        Some(LearningArcRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(LearningArcRefusal::TrainingSignalDetected)
    } else if config.personalizes {
        Some(LearningArcRefusal::PersonalizationSignalDetected)
    } else if config.autonomously_adapts {
        Some(LearningArcRefusal::AutonomousAdaptationSignalDetected)
    } else {
        None
    };
    if let Some(refusal) = signal {
        return assemble(
            config,
            anchors,
            None,
            Vec::new(),
            LearningArcDecision::ArcRefused,
            Some(refusal),
        );
    }

    // Session 1: candidate A from the empty journal.
    let session_1 = run_learning_session_default(&empty_learner_journal(), first_request);
    anchors.session_1 = session_1.receipt.receipt_hash;
    anchors.head_1 = session_1.receipt.journal_head_after;
    if let Some(refusal) = session_1.refusal {
        return assemble(
            config,
            anchors,
            Some(1),
            Vec::new(),
            LearningArcDecision::ArcRefused,
            Some(map_session_refusal(refusal)),
        );
    }

    // Canonical-threading seam: session 1 must land exactly on the canonical
    // journal head 1, or the arc cannot safely thread the next session.
    let journal_1 = match learner_journal_at(1) {
        Some(journal) if journal.head_hash == session_1.receipt.journal_head_after => journal,
        _ => {
            return assemble(
                config,
                anchors,
                Some(1),
                Vec::new(),
                LearningArcDecision::ArcRefused,
                Some(LearningArcRefusal::ArcChainBreak),
            );
        }
    };

    // Session 2: candidate B, starting FROM journal head 1.
    let session_2 = run_learning_session_default(&journal_1, second_request);
    anchors.session_2 = session_2.receipt.receipt_hash;
    anchors.head_2 = session_2.receipt.journal_head_after;
    if let Some(refusal) = session_2.refusal {
        return assemble(
            config,
            anchors,
            Some(2),
            Vec::new(),
            LearningArcDecision::ArcRefused,
            Some(map_session_refusal(refusal)),
        );
    }
    let head_2_canonical = learner_journal_at(2)
        .map(|journal| journal.head_hash)
        .unwrap_or(0);
    if session_2.receipt.journal_head_before != journal_1.head_hash
        || session_2.receipt.journal_head_after != head_2_canonical
    {
        return assemble(
            config,
            anchors,
            Some(2),
            Vec::new(),
            LearningArcDecision::ArcRefused,
            Some(LearningArcRefusal::ArcChainBreak),
        );
    }

    let steps = vec![
        step_from_session(1, LEARNING_ARC_STAGES[0], &session_1),
        step_from_session(2, LEARNING_ARC_STAGES[1], &session_2),
    ];
    // Self-check: the composed steps must satisfy the same chain law the
    // matrix uses against forged step vectors.
    if let Some(refusal) = arc_steps_are_chain_linked(&steps) {
        return assemble(
            config,
            anchors,
            None,
            Vec::new(),
            LearningArcDecision::ArcRefused,
            Some(refusal),
        );
    }
    assemble(
        config,
        anchors,
        None,
        steps,
        LearningArcDecision::ArcCompleted,
        None,
    )
}

/// The finite-growth proof: after the canonical arc, re-consenting the SAME
/// second session onto journal head 2 must refuse as a duplicate — the arc
/// cannot grow past its verified canonical candidates.
fn duplicate_third_session_run() -> LearningArcRun {
    let config = LearningArcConfig::default_config();
    let genesis = empty_learner_journal().head_hash;
    let journal_2 = learner_journal_at(2).expect("canonical journal at 2");
    let session_3 = run_learning_session_default(&journal_2, &learning_arc_second_request());
    let anchors = ArcAnchors {
        genesis,
        head_1: learner_journal_at(1)
            .map(|journal| journal.head_hash)
            .unwrap_or(0),
        head_2: journal_2.head_hash,
        session_1: 0,
        session_2: session_3.receipt.receipt_hash,
    };
    let refusal = session_3
        .refusal
        .map(map_session_refusal)
        .unwrap_or(LearningArcRefusal::SessionRefused);
    assemble(
        config,
        anchors,
        Some(3),
        Vec::new(),
        LearningArcDecision::ArcRefused,
        Some(refusal),
    )
}

/// The canonical MULTI-SESSION-0 demo: two consented sessions, one arc.
pub fn learning_arc_demo() -> LearningArcRun {
    run_learning_arc_default()
}

pub fn learning_arc_demo_json() -> String {
    serde_json::to_string_pretty(&learning_arc_demo()).expect("learning arc demo serializes")
}

pub fn verify_learning_arc_demo_json(candidate: &str) -> Result<(), LearningArcError> {
    if candidate == learning_arc_demo_json() {
        Ok(())
    } else {
        Err(LearningArcError::ReplayMismatch)
    }
}

pub const LEARNING_ARC_SCENARIO_COUNT: usize = 13;
pub const LEARNING_ARC_SCENARIO_NAMES: [&str; LEARNING_ARC_SCENARIO_COUNT] = [
    "arc_completed_across_two_sessions",
    "content_is_non_adaptive_across_sessions",
    "duplicate_third_session_refused",
    "refused_first_session_propagates",
    "consent_refusal_propagates",
    "arc_chain_break_refused",
    "arc_step_reorder_refused",
    "unsupported_arc_step_refused",
    "serialized_learning_arc_tamper_refused",
    "model_signal_refused",
    "training_signal_refused",
    "personalization_signal_refused",
    "autonomous_adaptation_signal_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct LearningArcCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub session_count: usize,
    pub refused_session_index: Option<u64>,
    pub heads_progressed: bool,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearningArcMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<LearningArcCell>,
    pub completed_count: usize,
    pub refused_count: usize,
    pub boundary: LearningArcBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_run(scenario: &str, run: &LearningArcRun) -> LearningArcCell {
    LearningArcCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        session_count: run.receipt.session_count,
        refused_session_index: run.receipt.refused_session_index,
        heads_progressed: run.receipt.journal_head_genesis
            != run.receipt.journal_head_after_session_1
            && run.receipt.journal_head_after_session_1 != run.receipt.journal_head_after_session_2
            && run.decision == LearningArcDecision::ArcCompleted,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn cell_from_guard(scenario: &str, refusal: Option<LearningArcRefusal>) -> LearningArcCell {
    LearningArcCell {
        scenario: scenario.to_string(),
        outcome: match refusal {
            Some(_) => "arc_refused".to_string(),
            None => "violation_missed".to_string(),
        },
        refusal: refusal.map(|r| r.slug().to_string()),
        session_count: 0,
        refused_session_index: None,
        heads_progressed: false,
        boundary_all_inert: LearningArcBoundary::inert().all_inert(),
    }
}

fn signal_cell(scenario: &str, set: fn(&mut LearningArcConfig)) -> LearningArcCell {
    let mut config = LearningArcConfig::default_config();
    set(&mut config);
    cell_from_run(scenario, &run_learning_arc(config))
}

fn cell_for(scenario: &str) -> LearningArcCell {
    match scenario {
        "arc_completed_across_two_sessions" => cell_from_run(scenario, &learning_arc_demo()),
        "content_is_non_adaptive_across_sessions" => {
            // The arc completes AND both sessions ran the identical frozen
            // content chain — growth happened only in the journal.
            let run = learning_arc_demo();
            let adapted = run.decision == LearningArcDecision::ArcCompleted
                && run.steps.len() == 2
                && run.steps[0].session_receipt_hash != run.steps[1].session_receipt_hash;
            // Distinct session receipts (journal seams differ) but the arc
            // itself proves content equality in its focused test; this cell
            // records the completed, non-adapted arc.
            let mut cell = cell_from_run(scenario, &run);
            cell.outcome = if adapted {
                cell.outcome
            } else {
                "non_adaptation_unproven".to_string()
            };
            cell
        }
        "duplicate_third_session_refused" => {
            cell_from_run(scenario, &duplicate_third_session_run())
        }
        "refused_first_session_propagates" => {
            let mut request = learning_session_demo_request();
            request.quiz_answers.push(LearnerQuizAnswerObservation {
                quiz_id: "quiz:999".to_string(),
                answer: "anything".to_string(),
            });
            let run = run_learning_arc_over(
                &request,
                &learning_arc_second_request(),
                LearningArcConfig::default_config(),
            );
            cell_from_run(scenario, &run)
        }
        "consent_refusal_propagates" => {
            let mut request = learning_session_demo_request();
            request.journal_consent = None;
            let run = run_learning_arc_over(
                &request,
                &learning_arc_second_request(),
                LearningArcConfig::default_config(),
            );
            cell_from_run(scenario, &run)
        }
        "arc_chain_break_refused" => {
            let mut steps = learning_arc_demo().steps;
            steps[1].journal_head_before ^= 1;
            cell_from_guard(scenario, arc_steps_are_chain_linked(&steps))
        }
        "arc_step_reorder_refused" => {
            let mut steps = learning_arc_demo().steps;
            steps.swap(0, 1);
            cell_from_guard(scenario, arc_steps_are_chain_linked(&steps))
        }
        "unsupported_arc_step_refused" => {
            let mut steps = learning_arc_demo().steps;
            steps[0].stage = "autonomous_planner".to_string();
            cell_from_guard(scenario, arc_steps_are_chain_linked(&steps))
        }
        "serialized_learning_arc_tamper_refused" => {
            // Serialize the real arc artifact, flip one byte, and confirm the
            // tamper is detectable — constructing the refusal that names this
            // scenario (the established A3 precedent).
            let json = learning_arc_demo_json();
            let refused = verify_learning_arc_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(LearningArcRefusal::SerializedLearningArcTamper)
            } else {
                None
            };
            LearningArcCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: refusal.map(|r| r.slug().to_string()),
                session_count: 0,
                refused_session_index: None,
                heads_progressed: false,
                boundary_all_inert: LearningArcBoundary::inert().all_inert(),
            }
        }
        "model_signal_refused" => signal_cell(scenario, |c| c.uses_model = true),
        "training_signal_refused" => signal_cell(scenario, |c| c.uses_training = true),
        "personalization_signal_refused" => signal_cell(scenario, |c| c.personalizes = true),
        "autonomous_adaptation_signal_refused" => {
            signal_cell(scenario, |c| c.autonomously_adapts = true)
        }
        other => LearningArcCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            session_count: 0,
            refused_session_index: None,
            heads_progressed: false,
            boundary_all_inert: false,
        },
    }
}

pub fn learning_arc_matrix() -> LearningArcMatrix {
    let cells = LEARNING_ARC_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let completed_count = cells
        .iter()
        .filter(|cell| cell.outcome == "arc_completed")
        .count();
    let refused_count = cells.len() - completed_count;
    let boundary = LearningArcBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    LearningArcMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        completed_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn learning_arc_matrix_json() -> String {
    serde_json::to_string_pretty(&learning_arc_matrix()).expect("learning arc matrix serializes")
}

pub fn verify_learning_arc_matrix_json(candidate: &str) -> Result<(), LearningArcError> {
    if candidate == learning_arc_matrix_json() {
        Ok(())
    } else {
        Err(LearningArcError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_completes_two_sessions_with_head_progression() {
        let run = learning_arc_demo();
        assert_eq!(run.decision, LearningArcDecision::ArcCompleted);
        assert!(run.refusal.is_none());
        assert_eq!(run.receipt.session_count, 2);
        assert_eq!(run.steps.len(), 2);
        assert_ne!(
            run.receipt.journal_head_genesis,
            run.receipt.journal_head_after_session_1
        );
        assert_ne!(
            run.receipt.journal_head_after_session_1,
            run.receipt.journal_head_after_session_2
        );
        assert!(run.receipt.boundary_all_inert);
    }

    #[test]
    fn head_progression_matches_canonical_journals() {
        let run = learning_arc_demo();
        assert_eq!(
            run.receipt.journal_head_genesis,
            empty_learner_journal().head_hash
        );
        assert_eq!(
            run.receipt.journal_head_after_session_1,
            learner_journal_at(1).expect("journal at 1").head_hash
        );
        assert_eq!(
            run.receipt.journal_head_after_session_2,
            learner_journal_at(2).expect("journal at 2").head_hash
        );
        // The seam law, explicitly: session 2 STARTED from head 1.
        assert_eq!(
            run.steps[1].journal_head_before,
            run.receipt.journal_head_after_session_1
        );
        assert_eq!(
            run.steps[0].journal_head_before,
            run.receipt.journal_head_genesis
        );
    }

    #[test]
    fn receipt_folds_both_session_hashes() {
        let run = learning_arc_demo();
        assert_eq!(
            run.receipt.session_1_receipt_hash,
            run.steps[0].session_receipt_hash
        );
        assert_eq!(
            run.receipt.session_2_receipt_hash,
            run.steps[1].session_receipt_hash
        );
        assert_ne!(run.receipt.session_1_receipt_hash, 0);
        assert_ne!(run.receipt.session_2_receipt_hash, 0);
        assert_ne!(
            run.receipt.session_1_receipt_hash,
            run.receipt.session_2_receipt_hash
        );
    }

    #[test]
    fn content_is_non_adaptive_across_sessions() {
        // Both sessions run the IDENTICAL frozen content chain: the evidence,
        // intent, and teach receipt anchors are equal across sessions — only
        // the journal grew. This is the non-adaptation proof.
        let session_1 = run_learning_session_default(
            &empty_learner_journal(),
            &learning_session_demo_request(),
        );
        let session_2 = run_learning_session_default(
            &learner_journal_at(1).expect("journal at 1"),
            &learning_arc_second_request(),
        );
        assert_eq!(
            session_1.receipt.qflow_receipt_hash,
            session_2.receipt.qflow_receipt_hash
        );
        assert_eq!(
            session_1.receipt.intent_receipt_hash,
            session_2.receipt.intent_receipt_hash
        );
        assert_eq!(
            session_1.receipt.teach_receipt_hash,
            session_2.receipt.teach_receipt_hash
        );
        // The learner observations differ by explicit request, not by history.
        assert_ne!(
            session_1.receipt.learner_receipt_hash,
            session_2.receipt.learner_receipt_hash
        );
    }

    #[test]
    fn duplicate_third_session_is_refused() {
        let run = duplicate_third_session_run();
        assert_eq!(run.decision, LearningArcDecision::ArcRefused);
        assert_eq!(
            run.refusal,
            Some(LearningArcRefusal::DuplicateSessionRefused)
        );
        assert_eq!(run.receipt.refused_session_index, Some(3));
    }

    #[test]
    fn refused_first_session_propagates_with_index() {
        let mut request = learning_session_demo_request();
        request.quiz_answers.push(LearnerQuizAnswerObservation {
            quiz_id: "quiz:999".to_string(),
            answer: "anything".to_string(),
        });
        let run = run_learning_arc_over(
            &request,
            &learning_arc_second_request(),
            LearningArcConfig::default_config(),
        );
        assert_eq!(run.refusal, Some(LearningArcRefusal::SessionRefused));
        assert_eq!(run.receipt.refused_session_index, Some(1));
    }

    #[test]
    fn consent_refusal_propagates_distinctly() {
        let mut request = learning_session_demo_request();
        request.journal_consent = None;
        let run = run_learning_arc_over(
            &request,
            &learning_arc_second_request(),
            LearningArcConfig::default_config(),
        );
        assert_eq!(run.refusal, Some(LearningArcRefusal::SessionConsentRefused));
    }

    #[test]
    fn forged_seam_is_refused_as_chain_break() {
        let mut steps = learning_arc_demo().steps;
        steps[1].journal_head_before ^= 1;
        assert_eq!(
            arc_steps_are_chain_linked(&steps),
            Some(LearningArcRefusal::ArcChainBreak)
        );
    }

    #[test]
    fn forged_genesis_is_refused_as_chain_break() {
        let mut steps = learning_arc_demo().steps;
        steps[0].journal_head_before ^= 1;
        assert_eq!(
            arc_steps_are_chain_linked(&steps),
            Some(LearningArcRefusal::ArcChainBreak)
        );
    }

    #[test]
    fn reordered_steps_are_refused() {
        let mut steps = learning_arc_demo().steps;
        steps.swap(0, 1);
        assert_eq!(
            arc_steps_are_chain_linked(&steps),
            Some(LearningArcRefusal::ArcStepReorder)
        );
    }

    #[test]
    fn unknown_stage_is_refused() {
        let mut steps = learning_arc_demo().steps;
        steps[0].stage = "autonomous_planner".to_string();
        assert_eq!(
            arc_steps_are_chain_linked(&steps),
            Some(LearningArcRefusal::UnsupportedArcStep)
        );
    }

    #[test]
    fn every_signal_config_refuses_before_any_session_runs() {
        type SignalCase = (fn(&mut LearningArcConfig), LearningArcRefusal);
        let cases: [SignalCase; 4] = [
            (
                |c| c.uses_model = true,
                LearningArcRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                LearningArcRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.personalizes = true,
                LearningArcRefusal::PersonalizationSignalDetected,
            ),
            (
                |c| c.autonomously_adapts = true,
                LearningArcRefusal::AutonomousAdaptationSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = LearningArcConfig::default_config();
            set(&mut config);
            let run = run_learning_arc(config);
            assert_eq!(run.refusal, Some(expected));
            assert_eq!(run.receipt.session_1_receipt_hash, 0, "no session may run");
        }
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = learning_arc_demo_json();
        assert!(verify_learning_arc_demo_json(&json).is_ok());
        assert_eq!(
            verify_learning_arc_demo_json(&flip_last_byte(&json)),
            Err(LearningArcError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = learning_arc_matrix_json();
        assert!(verify_learning_arc_matrix_json(&json).is_ok());
        assert_eq!(
            verify_learning_arc_matrix_json(&flip_last_byte(&json)),
            Err(LearningArcError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = learning_arc_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|cell| cell.scenario == "serialized_learning_arc_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_learning_arc_tamper_refused")
        );
    }

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = learning_arc_matrix();
        assert_eq!(matrix.scenario_count, LEARNING_ARC_SCENARIO_COUNT);
        assert_eq!(matrix.completed_count, 2);
        let constructed = matrix
            .cells
            .iter()
            .filter_map(|cell| cell.refusal.clone())
            .collect::<Vec<_>>();
        for refusal in LearningArcRefusal::ALL {
            assert!(
                constructed.iter().any(|slug| slug == refusal.slug()),
                "refusal {} must be constructed by a matrix scenario",
                refusal.slug()
            );
        }
        assert!(matrix.cells.iter().all(|cell| cell.outcome != "unknown"
            && cell.outcome != "violation_missed"
            && cell.outcome != "non_adaptation_unproven"));
    }

    #[test]
    fn boundary_lines_and_flags_stay_inert() {
        assert_eq!(LEARNING_ARC_BOUNDARY_LINES.len(), 9);
        let boundary = LearningArcBoundary::inert();
        assert!(boundary.all_inert());
        let mut broken = boundary;
        broken.adapts_across_sessions = true;
        assert!(!broken.all_inert());
    }
}
