//! LEARNER-MODEL-0 — map-only learner-state receipts from supported lessons.
//!
//! This is NOT companion memory yet. It consumes a built [`TeachMapRun`] and a
//! bounded observation record, then emits the first local learner-state object:
//! what lesson items were seen, what concept was taught, exact-match quiz
//! outcomes, explicit misconception flags, a self-reported confidence marker,
//! a non-adaptive next-review target, and receipt links. It does NOT
//! personalize, does NOT adapt autonomously, does NOT write long-term memory,
//! does NOT infer health/psych profiles, does NOT diagnose, and does NOT call a
//! model.

use serde::Serialize;

use crate::{teach_map_demo, TeachMapDecision, TeachMapRefusal, TeachMapRun, TeachSupportRef};

const SCHEMA: &str = "learner-model-map-v0.1";
const LEARNER_MODEL_USES_MODEL: bool = false;
const LEARNER_MODEL_USES_TRAINING: bool = false;
const LEARNER_MODEL_PERSONALIZES: bool = false;
const LEARNER_MODEL_WRITES_MEMORY: bool = false;
const LEARNER_MODEL_AUTONOMOUSLY_ADAPTS: bool = false;
const LEARNER_MODEL_INFERS_HEALTH_PROFILE: bool = false;
const LEARNER_MODEL_INFERS_HIDDEN_DIAGNOSIS: bool = false;
const REQUIRED_TEACH_AUTHORITY: &str = "teach_from_span_backed_intent_map";
const REQUIRED_INTENT_AUTHORITY: &str = "intent_map_from_verified_span";
const AUTHORITY_LEARNER_STATE_FROM_TEACH_MAP: &str = "learner_state_from_supported_teach_map";

/// The authority boundary, verbatim. LEARNER-MODEL-0 maps observations only.
pub const LEARNER_MODEL_BOUNDARY_LINES: [&str; 9] = [
    "LEARNER-MODEL-0 records a local learner-state map only.",
    "It consumes a supported TEACH-0 lesson receipt.",
    "It does not personalize generation.",
    "It does not autonomously adapt.",
    "It does not write long-term learner memory.",
    "It does not run a model.",
    "It does not train.",
    "It does not infer a health or psych profile.",
    "It does not make hidden diagnoses.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearnerModelDecision {
    LearnerStateMapped,
    LearnerStateRefused,
}

impl LearnerModelDecision {
    pub const ALL: [LearnerModelDecision; 2] = [
        LearnerModelDecision::LearnerStateMapped,
        LearnerModelDecision::LearnerStateRefused,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            LearnerModelDecision::LearnerStateMapped => "learner_state_mapped",
            LearnerModelDecision::LearnerStateRefused => "learner_state_refused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearnerModelRefusal {
    TeachMapRefused,
    TeachLessonUnavailable,
    UnsupportedLessonSupport,
    NoLessonItemsSeen,
    UnrecognizedLessonItem,
    UnrecognizedQuizItem,
    UnrecognizedMisconceptionCheck,
    ModelSignalDetected,
    TrainingSignalDetected,
    PersonalizationSignalDetected,
    MemoryWriteSignalDetected,
    AutonomousAdaptationSignalDetected,
    HealthProfileSignalDetected,
    HiddenDiagnosisSignalDetected,
    SerializedLearnerModelTamper,
}

impl LearnerModelRefusal {
    pub const ALL: [LearnerModelRefusal; 15] = [
        LearnerModelRefusal::TeachMapRefused,
        LearnerModelRefusal::TeachLessonUnavailable,
        LearnerModelRefusal::UnsupportedLessonSupport,
        LearnerModelRefusal::NoLessonItemsSeen,
        LearnerModelRefusal::UnrecognizedLessonItem,
        LearnerModelRefusal::UnrecognizedQuizItem,
        LearnerModelRefusal::UnrecognizedMisconceptionCheck,
        LearnerModelRefusal::ModelSignalDetected,
        LearnerModelRefusal::TrainingSignalDetected,
        LearnerModelRefusal::PersonalizationSignalDetected,
        LearnerModelRefusal::MemoryWriteSignalDetected,
        LearnerModelRefusal::AutonomousAdaptationSignalDetected,
        LearnerModelRefusal::HealthProfileSignalDetected,
        LearnerModelRefusal::HiddenDiagnosisSignalDetected,
        LearnerModelRefusal::SerializedLearnerModelTamper,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            LearnerModelRefusal::TeachMapRefused => "teach_map_refused",
            LearnerModelRefusal::TeachLessonUnavailable => "teach_lesson_unavailable_refused",
            LearnerModelRefusal::UnsupportedLessonSupport => "unsupported_lesson_support_refused",
            LearnerModelRefusal::NoLessonItemsSeen => "no_lesson_items_seen_refused",
            LearnerModelRefusal::UnrecognizedLessonItem => "unrecognized_lesson_item_refused",
            LearnerModelRefusal::UnrecognizedQuizItem => "unrecognized_quiz_item_refused",
            LearnerModelRefusal::UnrecognizedMisconceptionCheck => {
                "unrecognized_misconception_check_refused"
            }
            LearnerModelRefusal::ModelSignalDetected => "model_signal_detected_refused",
            LearnerModelRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            LearnerModelRefusal::PersonalizationSignalDetected => {
                "personalization_signal_detected_refused"
            }
            LearnerModelRefusal::MemoryWriteSignalDetected => {
                "memory_write_signal_detected_refused"
            }
            LearnerModelRefusal::AutonomousAdaptationSignalDetected => {
                "autonomous_adaptation_signal_detected_refused"
            }
            LearnerModelRefusal::HealthProfileSignalDetected => {
                "health_profile_signal_detected_refused"
            }
            LearnerModelRefusal::HiddenDiagnosisSignalDetected => {
                "hidden_diagnosis_signal_detected_refused"
            }
            LearnerModelRefusal::SerializedLearnerModelTamper => {
                "serialized_learner_model_tamper_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearnerModelError {
    ReplayMismatch,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearnerModelConfig {
    pub uses_model: bool,
    pub uses_training: bool,
    pub personalizes: bool,
    pub writes_memory: bool,
    pub autonomously_adapts: bool,
    pub infers_health_profile: bool,
    pub infers_hidden_diagnosis: bool,
}

impl LearnerModelConfig {
    pub fn default_config() -> Self {
        LearnerModelConfig {
            uses_model: LEARNER_MODEL_USES_MODEL,
            uses_training: LEARNER_MODEL_USES_TRAINING,
            personalizes: LEARNER_MODEL_PERSONALIZES,
            writes_memory: LEARNER_MODEL_WRITES_MEMORY,
            autonomously_adapts: LEARNER_MODEL_AUTONOMOUSLY_ADAPTS,
            infers_health_profile: LEARNER_MODEL_INFERS_HEALTH_PROFILE,
            infers_hidden_diagnosis: LEARNER_MODEL_INFERS_HIDDEN_DIAGNOSIS,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearnerModelBoundary {
    pub creates_truth: bool,
    pub personalizes: bool,
    pub autonomously_adapts: bool,
    pub writes_long_term_memory: bool,
    pub trains: bool,
    pub is_model: bool,
    pub infers_health_profile: bool,
    pub infers_hidden_diagnosis: bool,
    pub retags_release: bool,
}

impl LearnerModelBoundary {
    fn inert() -> Self {
        LearnerModelBoundary {
            creates_truth: LEARNER_MODEL_USES_MODEL,
            personalizes: LEARNER_MODEL_PERSONALIZES,
            autonomously_adapts: LEARNER_MODEL_AUTONOMOUSLY_ADAPTS,
            writes_long_term_memory: LEARNER_MODEL_WRITES_MEMORY,
            trains: LEARNER_MODEL_USES_TRAINING,
            is_model: LEARNER_MODEL_USES_MODEL,
            infers_health_profile: LEARNER_MODEL_INFERS_HEALTH_PROFILE,
            infers_hidden_diagnosis: LEARNER_MODEL_INFERS_HIDDEN_DIAGNOSIS,
            retags_release: LEARNER_MODEL_USES_MODEL,
        }
    }

    fn all_inert(&self) -> bool {
        !self.creates_truth
            && !self.personalizes
            && !self.autonomously_adapts
            && !self.writes_long_term_memory
            && !self.trains
            && !self.is_model
            && !self.infers_health_profile
            && !self.infers_hidden_diagnosis
            && !self.retags_release
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConfidenceMarker {
    Unstated,
    Low,
    Medium,
    High,
}

impl ConfidenceMarker {
    pub fn slug(self) -> &'static str {
        match self {
            ConfidenceMarker::Unstated => "unstated",
            ConfidenceMarker::Low => "low",
            ConfidenceMarker::Medium => "medium",
            ConfidenceMarker::High => "high",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerQuizAnswerObservation {
    pub quiz_id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerMisconceptionObservation {
    pub check_id: String,
    pub flagged: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerModelObservation {
    pub seen_lesson_item_ids: Vec<String>,
    pub quiz_answers: Vec<LearnerQuizAnswerObservation>,
    pub misconception_flags: Vec<LearnerMisconceptionObservation>,
    pub confidence_marker: ConfidenceMarker,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerSupportRef {
    pub document_id: u64,
    pub document_name: String,
    pub span_id: u64,
    pub text: String,
    pub intent_authority: String,
    pub teach_authority: String,
    pub learner_authority: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SeenLessonItem {
    pub item_id: String,
    pub kind: String,
    pub text: String,
    pub support: Vec<LearnerSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TaughtConcept {
    pub label: String,
    pub source_item_id: String,
    pub support: Vec<LearnerSupportRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuizOutcome {
    CorrectExactMatch,
    IncorrectExactMismatch,
    Unanswered,
}

impl QuizOutcome {
    pub fn slug(self) -> &'static str {
        match self {
            QuizOutcome::CorrectExactMatch => "correct_exact_match",
            QuizOutcome::IncorrectExactMismatch => "incorrect_exact_mismatch",
            QuizOutcome::Unanswered => "unanswered",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuizItemResult {
    pub quiz_id: String,
    pub question: String,
    pub expected_answer: String,
    pub observed_answer: Option<String>,
    pub outcome: QuizOutcome,
    pub support: Vec<LearnerSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuizResultSummary {
    pub total: usize,
    pub answered: usize,
    pub correct_exact_match: usize,
    pub incorrect_exact_mismatch: usize,
    pub unanswered: usize,
    pub items: Vec<QuizItemResult>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MisconceptionFlag {
    pub check_id: String,
    pub misconception: String,
    pub flagged: bool,
    pub source: String,
    pub hidden_diagnosis: bool,
    pub support: Vec<LearnerSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfidenceState {
    pub marker: ConfidenceMarker,
    pub source: String,
    pub inferred: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NextReviewTarget {
    pub source: String,
    pub text: String,
    pub autonomously_adapted: bool,
    pub support: Vec<LearnerSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerModelFieldRefusal {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerStateMap {
    pub source_teach_receipt_hash: u64,
    pub source_intent_receipt_hash: u64,
    pub document: String,
    pub seen_items: Vec<SeenLessonItem>,
    pub concept_taught: Option<TaughtConcept>,
    pub quiz_result: QuizResultSummary,
    pub misconception_flags: Vec<MisconceptionFlag>,
    pub confidence_marker: ConfidenceState,
    pub next_review_target: Option<NextReviewTarget>,
    pub receipt_link: String,
    pub refusals: Vec<LearnerModelFieldRefusal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerModelReceipt {
    pub schema: String,
    pub source_teach_receipt_hash: u64,
    pub source_intent_receipt_hash: u64,
    pub source_teach_decision: String,
    pub source_teach_refusal: Option<String>,
    pub config: LearnerModelConfig,
    pub seen_item_count: usize,
    pub quiz_total: usize,
    pub quiz_answered: usize,
    pub misconception_flag_count: usize,
    pub decision: LearnerModelDecision,
    pub refusal: Option<LearnerModelRefusal>,
    pub receipt_hash: u64,
    pub boundary: LearnerModelBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerModelRun {
    pub observation: LearnerModelObservation,
    pub receipt: LearnerModelReceipt,
    pub learner_state: Option<LearnerStateMap>,
    pub decision: LearnerModelDecision,
    pub refusal: Option<LearnerModelRefusal>,
}

#[derive(Debug, Clone)]
struct LessonItemRef {
    id: String,
    kind: String,
    text: String,
    support: Vec<TeachSupportRef>,
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

fn receipt_hash(
    teach_run: &TeachMapRun,
    observation: &LearnerModelObservation,
    config: &LearnerModelConfig,
    state: Option<&LearnerStateMap>,
    decision: LearnerModelDecision,
    refusal: Option<LearnerModelRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    h = fnv_mix(h, SCHEMA.as_bytes());
    h = fnv_u64(h, teach_run.receipt.receipt_hash);
    h = fnv_u64(h, teach_run.receipt.source_intent_receipt_hash);
    h = fnv_mix(h, teach_run.decision.slug().as_bytes());
    h = fnv_mix(
        h,
        teach_run
            .refusal
            .map(TeachMapRefusal::slug)
            .unwrap_or("none")
            .as_bytes(),
    );
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, config.personalizes as u64);
    h = fnv_u64(h, config.writes_memory as u64);
    h = fnv_u64(h, config.autonomously_adapts as u64);
    h = fnv_u64(h, config.infers_health_profile as u64);
    h = fnv_u64(h, config.infers_hidden_diagnosis as u64);
    for seen in &observation.seen_lesson_item_ids {
        h = fnv_mix(h, seen.as_bytes());
    }
    for answer in &observation.quiz_answers {
        h = fnv_mix(h, answer.quiz_id.as_bytes());
        h = fnv_mix(h, answer.answer.as_bytes());
    }
    for flag in &observation.misconception_flags {
        h = fnv_mix(h, flag.check_id.as_bytes());
        h = fnv_u64(h, flag.flagged as u64);
    }
    h = fnv_mix(h, observation.confidence_marker.slug().as_bytes());
    if let Some(state) = state {
        h = fnv_mix(h, state.document.as_bytes());
        for seen in &state.seen_items {
            h = fnv_mix(h, seen.item_id.as_bytes());
            h = fnv_mix(h, seen.kind.as_bytes());
            h = fnv_mix(h, seen.text.as_bytes());
            h = mix_support(h, &seen.support);
        }
        if let Some(concept) = state.concept_taught.as_ref() {
            h = fnv_mix(h, concept.label.as_bytes());
            h = fnv_mix(h, concept.source_item_id.as_bytes());
            h = mix_support(h, &concept.support);
        }
        for item in &state.quiz_result.items {
            h = fnv_mix(h, item.quiz_id.as_bytes());
            h = fnv_mix(h, item.question.as_bytes());
            h = fnv_mix(h, item.expected_answer.as_bytes());
            h = fnv_mix(
                h,
                item.observed_answer.as_deref().unwrap_or("none").as_bytes(),
            );
            h = fnv_mix(h, item.outcome.slug().as_bytes());
            h = mix_support(h, &item.support);
        }
        for flag in &state.misconception_flags {
            h = fnv_mix(h, flag.check_id.as_bytes());
            h = fnv_mix(h, flag.misconception.as_bytes());
            h = fnv_u64(h, flag.flagged as u64);
            h = fnv_mix(h, flag.source.as_bytes());
            h = fnv_u64(h, flag.hidden_diagnosis as u64);
            h = mix_support(h, &flag.support);
        }
        h = fnv_mix(h, state.confidence_marker.marker.slug().as_bytes());
        h = fnv_mix(h, state.confidence_marker.source.as_bytes());
        h = fnv_u64(h, state.confidence_marker.inferred as u64);
        if let Some(target) = state.next_review_target.as_ref() {
            h = fnv_mix(h, target.source.as_bytes());
            h = fnv_mix(h, target.text.as_bytes());
            h = fnv_u64(h, target.autonomously_adapted as u64);
            h = mix_support(h, &target.support);
        }
        h = fnv_mix(h, state.receipt_link.as_bytes());
        for refusal in &state.refusals {
            h = fnv_mix(h, refusal.field.as_bytes());
            h = fnv_mix(h, refusal.reason.as_bytes());
        }
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

fn mix_support(mut h: u64, support: &[LearnerSupportRef]) -> u64 {
    h = fnv_u64(h, support.len() as u64);
    for item in support {
        h = fnv_u64(h, item.document_id);
        h = fnv_u64(h, item.span_id);
        h = fnv_mix(h, item.document_name.as_bytes());
        h = fnv_mix(h, item.text.as_bytes());
        h = fnv_mix(h, item.intent_authority.as_bytes());
        h = fnv_mix(h, item.teach_authority.as_bytes());
        h = fnv_mix(h, item.learner_authority.as_bytes());
    }
    h
}

fn flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn learner_model_demo() -> LearnerModelRun {
    let teach = teach_map_demo();
    let observation = learner_model_demo_observation(&teach);
    run_learner_model_default(&teach, observation)
}

pub fn learner_model_demo_json() -> String {
    serde_json::to_string_pretty(&learner_model_demo()).expect("learner model demo serializes")
}

pub fn verify_learner_model_demo_json(candidate: &str) -> Result<(), LearnerModelError> {
    if candidate == learner_model_demo_json() {
        Ok(())
    } else {
        Err(LearnerModelError::ReplayMismatch)
    }
}

pub fn run_learner_model_default(
    teach_run: &TeachMapRun,
    observation: LearnerModelObservation,
) -> LearnerModelRun {
    run_learner_model(teach_run, observation, LearnerModelConfig::default_config())
}

pub fn run_learner_model(
    teach_run: &TeachMapRun,
    observation: LearnerModelObservation,
    config: LearnerModelConfig,
) -> LearnerModelRun {
    if config.uses_model {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::ModelSignalDetected),
            None,
        );
    }
    if config.uses_training {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::TrainingSignalDetected),
            None,
        );
    }
    if config.personalizes {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::PersonalizationSignalDetected),
            None,
        );
    }
    if config.writes_memory {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::MemoryWriteSignalDetected),
            None,
        );
    }
    if config.autonomously_adapts {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::AutonomousAdaptationSignalDetected),
            None,
        );
    }
    if config.infers_health_profile {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::HealthProfileSignalDetected),
            None,
        );
    }
    if config.infers_hidden_diagnosis {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::HiddenDiagnosisSignalDetected),
            None,
        );
    }
    if teach_run.decision == TeachMapDecision::LessonRefused {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::TeachMapRefused),
            None,
        );
    }
    let lesson = match teach_run.lesson.as_ref() {
        Some(lesson) => lesson,
        None => {
            return assemble(
                teach_run,
                observation,
                config,
                LearnerModelDecision::LearnerStateRefused,
                Some(LearnerModelRefusal::TeachLessonUnavailable),
                None,
            );
        }
    };
    let lesson_items = lesson_item_refs(lesson);
    if lesson_items
        .iter()
        .any(|item| !valid_teach_support(&item.support))
    {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::UnsupportedLessonSupport),
            None,
        );
    }
    if observation.seen_lesson_item_ids.is_empty() {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::NoLessonItemsSeen),
            None,
        );
    }
    if observation
        .seen_lesson_item_ids
        .iter()
        .any(|id| lesson_item(&lesson_items, id).is_none())
    {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::UnrecognizedLessonItem),
            None,
        );
    }
    if observation
        .quiz_answers
        .iter()
        .any(|answer| quiz_item(lesson, &answer.quiz_id).is_none())
    {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::UnrecognizedQuizItem),
            None,
        );
    }
    if observation
        .misconception_flags
        .iter()
        .any(|flag| misconception_check(lesson, &flag.check_id).is_none())
    {
        return assemble(
            teach_run,
            observation,
            config,
            LearnerModelDecision::LearnerStateRefused,
            Some(LearnerModelRefusal::UnrecognizedMisconceptionCheck),
            None,
        );
    }

    let state = build_state(teach_run, &observation, &lesson_items);
    assemble(
        teach_run,
        observation,
        config,
        LearnerModelDecision::LearnerStateMapped,
        None,
        Some(state),
    )
}

fn assemble(
    teach_run: &TeachMapRun,
    observation: LearnerModelObservation,
    config: LearnerModelConfig,
    decision: LearnerModelDecision,
    refusal: Option<LearnerModelRefusal>,
    state: Option<LearnerStateMap>,
) -> LearnerModelRun {
    let boundary = LearnerModelBoundary::inert();
    let seen_item_count = state.as_ref().map(|s| s.seen_items.len()).unwrap_or(0);
    let quiz_total = state.as_ref().map(|s| s.quiz_result.total).unwrap_or(0);
    let quiz_answered = state.as_ref().map(|s| s.quiz_result.answered).unwrap_or(0);
    let misconception_flag_count = state
        .as_ref()
        .map(|s| s.misconception_flags.len())
        .unwrap_or(0);
    let receipt_hash = receipt_hash(
        teach_run,
        &observation,
        &config,
        state.as_ref(),
        decision,
        refusal,
    );
    let receipt = LearnerModelReceipt {
        schema: SCHEMA.to_string(),
        source_teach_receipt_hash: teach_run.receipt.receipt_hash,
        source_intent_receipt_hash: teach_run.receipt.source_intent_receipt_hash,
        source_teach_decision: teach_run.decision.slug().to_string(),
        source_teach_refusal: teach_run.refusal.map(|r| r.slug().to_string()),
        config,
        seen_item_count,
        quiz_total,
        quiz_answered,
        misconception_flag_count,
        decision,
        refusal,
        receipt_hash,
        boundary,
        boundary_all_inert: boundary.all_inert(),
    };
    LearnerModelRun {
        observation,
        receipt,
        learner_state: state,
        decision,
        refusal,
    }
}

fn build_state(
    teach_run: &TeachMapRun,
    observation: &LearnerModelObservation,
    lesson_items: &[LessonItemRef],
) -> LearnerStateMap {
    let lesson = teach_run.lesson.as_ref().expect("lesson already checked");
    let seen_items = observation
        .seen_lesson_item_ids
        .iter()
        .filter_map(|id| lesson_item(lesson_items, id))
        .map(|item| SeenLessonItem {
            item_id: item.id.clone(),
            kind: item.kind.clone(),
            text: item.text.clone(),
            support: learner_support(&item.support),
        })
        .collect::<Vec<_>>();

    let concept_taught = seen_items.first().map(|item| TaughtConcept {
        label: concept_label(&item.text),
        source_item_id: item.item_id.clone(),
        support: item.support.clone(),
    });
    let quiz_result = quiz_result(lesson, &observation.quiz_answers);
    let misconception_flags = misconception_flags(lesson, &observation.misconception_flags);
    let confidence_marker = ConfidenceState {
        marker: observation.confidence_marker,
        source: match observation.confidence_marker {
            ConfidenceMarker::Unstated => "not_reported".to_string(),
            _ => "self_reported_marker".to_string(),
        },
        inferred: false,
    };
    let next_review_target = lesson
        .next_reading_step
        .as_ref()
        .map(|target| NextReviewTarget {
            source: "teach_map_next_reading_step".to_string(),
            text: target.text.clone(),
            autonomously_adapted: false,
            support: learner_support(&target.support),
        });
    let mut refusals = vec![
        field_refusal("personalization", "not generated by LEARNER-MODEL-0"),
        field_refusal("long_term_memory", "not written by LEARNER-MODEL-0"),
        field_refusal("autonomous_adaptation", "not performed by LEARNER-MODEL-0"),
        field_refusal("health_or_psych_profile", "not inferred by LEARNER-MODEL-0"),
        field_refusal("hidden_diagnosis", "not inferred by LEARNER-MODEL-0"),
    ];
    if next_review_target.is_none() {
        refusals.push(field_refusal(
            "next_review_target",
            "no supported TEACH-0 next reading step was available",
        ));
    }
    LearnerStateMap {
        source_teach_receipt_hash: teach_run.receipt.receipt_hash,
        source_intent_receipt_hash: teach_run.receipt.source_intent_receipt_hash,
        document: lesson.document.clone(),
        seen_items,
        concept_taught,
        quiz_result,
        misconception_flags,
        confidence_marker,
        next_review_target,
        receipt_link: format!("teach_receipt:{}", teach_run.receipt.receipt_hash),
        refusals,
    }
}

fn lesson_item_refs(lesson: &crate::TeachLesson) -> Vec<LessonItemRef> {
    let mut items = Vec::new();
    if let Some(explanation) = lesson.explanation.as_ref() {
        items.push(LessonItemRef {
            id: "explanation:1".to_string(),
            kind: "explanation".to_string(),
            text: explanation.text.clone(),
            support: explanation.support.clone(),
        });
    }
    for (idx, example) in lesson.examples.iter().enumerate() {
        items.push(LessonItemRef {
            id: format!("example:{}", idx + 1),
            kind: "example".to_string(),
            text: example.text.clone(),
            support: example.support.clone(),
        });
    }
    for (idx, check) in lesson.misconception_checks.iter().enumerate() {
        items.push(LessonItemRef {
            id: format!("misconception_check:{}", idx + 1),
            kind: "misconception_check".to_string(),
            text: check.misconception.clone(),
            support: check.support.clone(),
        });
    }
    for (idx, item) in lesson.quiz.iter().enumerate() {
        items.push(LessonItemRef {
            id: format!("quiz:{}", idx + 1),
            kind: "quiz".to_string(),
            text: item.question.clone(),
            support: item.support.clone(),
        });
    }
    if let Some(target) = lesson.next_reading_step.as_ref() {
        items.push(LessonItemRef {
            id: "next_review_target:1".to_string(),
            kind: "next_review_target".to_string(),
            text: target.text.clone(),
            support: target.support.clone(),
        });
    }
    items
}

fn lesson_item<'a>(items: &'a [LessonItemRef], id: &str) -> Option<&'a LessonItemRef> {
    items.iter().find(|item| item.id == id)
}

fn quiz_item<'a>(lesson: &'a crate::TeachLesson, id: &str) -> Option<(usize, &'a crate::QuizItem)> {
    lesson
        .quiz
        .iter()
        .enumerate()
        .find(|(idx, _)| id == format!("quiz:{}", idx + 1))
}

fn misconception_check<'a>(
    lesson: &'a crate::TeachLesson,
    id: &str,
) -> Option<(usize, &'a crate::MisconceptionCheck)> {
    lesson
        .misconception_checks
        .iter()
        .enumerate()
        .find(|(idx, _)| id == format!("misconception_check:{}", idx + 1))
}

fn quiz_result(
    lesson: &crate::TeachLesson,
    answers: &[LearnerQuizAnswerObservation],
) -> QuizResultSummary {
    let items = lesson
        .quiz
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let quiz_id = format!("quiz:{}", idx + 1);
            let observed_answer = answers
                .iter()
                .find(|answer| answer.quiz_id == quiz_id)
                .map(|answer| answer.answer.clone());
            let outcome = match observed_answer.as_ref() {
                Some(answer) if answer == &item.expected_answer => QuizOutcome::CorrectExactMatch,
                Some(_) => QuizOutcome::IncorrectExactMismatch,
                None => QuizOutcome::Unanswered,
            };
            QuizItemResult {
                quiz_id,
                question: item.question.clone(),
                expected_answer: item.expected_answer.clone(),
                observed_answer,
                outcome,
                support: learner_support(&item.support),
            }
        })
        .collect::<Vec<_>>();
    let answered = items
        .iter()
        .filter(|item| item.outcome != QuizOutcome::Unanswered)
        .count();
    let correct_exact_match = items
        .iter()
        .filter(|item| item.outcome == QuizOutcome::CorrectExactMatch)
        .count();
    let incorrect_exact_mismatch = items
        .iter()
        .filter(|item| item.outcome == QuizOutcome::IncorrectExactMismatch)
        .count();
    let unanswered = items
        .iter()
        .filter(|item| item.outcome == QuizOutcome::Unanswered)
        .count();
    QuizResultSummary {
        total: items.len(),
        answered,
        correct_exact_match,
        incorrect_exact_mismatch,
        unanswered,
        items,
    }
}

fn misconception_flags(
    lesson: &crate::TeachLesson,
    observations: &[LearnerMisconceptionObservation],
) -> Vec<MisconceptionFlag> {
    lesson
        .misconception_checks
        .iter()
        .enumerate()
        .map(|(idx, check)| {
            let check_id = format!("misconception_check:{}", idx + 1);
            let observed = observations.iter().find(|flag| flag.check_id == check_id);
            MisconceptionFlag {
                check_id,
                misconception: check.misconception.clone(),
                flagged: observed.map(|flag| flag.flagged).unwrap_or(false),
                source: if observed.is_some() {
                    "explicit_marker".to_string()
                } else {
                    "not_marked".to_string()
                },
                hidden_diagnosis: false,
                support: learner_support(&check.support),
            }
        })
        .collect()
}

fn concept_label(text: &str) -> String {
    text.split(':')
        .next_back()
        .unwrap_or(text)
        .trim()
        .to_string()
}

fn valid_teach_support(support: &[TeachSupportRef]) -> bool {
    !support.is_empty()
        && support.iter().all(|span| {
            span.intent_authority == REQUIRED_INTENT_AUTHORITY
                && span.teach_authority == REQUIRED_TEACH_AUTHORITY
                && span.text.chars().any(|c| c.is_alphanumeric())
        })
}

fn learner_support(support: &[TeachSupportRef]) -> Vec<LearnerSupportRef> {
    support
        .iter()
        .map(|span| LearnerSupportRef {
            document_id: span.document_id,
            document_name: span.document_name.clone(),
            span_id: span.span_id,
            text: span.text.clone(),
            intent_authority: span.intent_authority.clone(),
            teach_authority: span.teach_authority.clone(),
            learner_authority: AUTHORITY_LEARNER_STATE_FROM_TEACH_MAP.to_string(),
        })
        .collect()
}

fn field_refusal(field: &str, reason: &str) -> LearnerModelFieldRefusal {
    LearnerModelFieldRefusal {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

fn learner_model_demo_observation(teach_run: &TeachMapRun) -> LearnerModelObservation {
    let lesson = teach_run.lesson.as_ref().expect("canonical lesson");
    let seen_lesson_item_ids = lesson_item_refs(lesson)
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let mut quiz_answers = Vec::new();
    if let Some(item) = lesson.quiz.first() {
        quiz_answers.push(LearnerQuizAnswerObservation {
            quiz_id: "quiz:1".to_string(),
            answer: item.expected_answer.clone(),
        });
    }
    if lesson.quiz.len() > 1 {
        quiz_answers.push(LearnerQuizAnswerObservation {
            quiz_id: "quiz:2".to_string(),
            answer: "Hidden motive is enough.".to_string(),
        });
    }
    let misconception_flags = lesson
        .misconception_checks
        .iter()
        .enumerate()
        .map(|(idx, _)| LearnerMisconceptionObservation {
            check_id: format!("misconception_check:{}", idx + 1),
            flagged: idx == 0,
        })
        .collect::<Vec<_>>();
    LearnerModelObservation {
        seen_lesson_item_ids,
        quiz_answers,
        misconception_flags,
        confidence_marker: ConfidenceMarker::Medium,
    }
}

pub const LEARNER_MODEL_SCENARIO_COUNT: usize = 16;
pub const LEARNER_MODEL_SCENARIO_NAMES: [&str; LEARNER_MODEL_SCENARIO_COUNT] = [
    "supported_teach_map_builds_learner_state",
    "seen_items_are_receipt_linked",
    "concept_taught_is_supported",
    "quiz_result_uses_exact_match_only",
    "misconception_flags_are_explicit",
    "confidence_marker_is_self_reported",
    "next_review_target_is_non_adaptive",
    "teach_map_refusal_propagates",
    "unknown_seen_item_refused",
    "unknown_quiz_item_refused",
    "unknown_misconception_flag_refused",
    "no_model_signal_detected",
    "no_training_signal_detected",
    "personalization_signal_refused",
    "memory_adaptation_health_diagnosis_signals_refused",
    "serialized_learner_model_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct LearnerModelCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub learner_state_mapped: bool,
    pub seen_items: usize,
    pub quiz_total: usize,
    pub quiz_answered: usize,
    pub misconception_flags: usize,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerModelMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<LearnerModelCell>,
    pub mapped_count: usize,
    pub refused_count: usize,
    pub boundary: LearnerModelBoundary,
    pub boundary_all_inert: bool,
}

fn cell_for(scenario: &str) -> LearnerModelCell {
    match scenario {
        "supported_teach_map_builds_learner_state"
        | "seen_items_are_receipt_linked"
        | "concept_taught_is_supported"
        | "quiz_result_uses_exact_match_only"
        | "misconception_flags_are_explicit"
        | "confidence_marker_is_self_reported"
        | "next_review_target_is_non_adaptive" => {
            let run = learner_model_demo();
            cell_from_run(scenario, &run)
        }
        "serialized_learner_model_tamper_refused" => {
            // Serialize a real learner-model artifact, flip one byte, and confirm the
            // tamper is detectable — constructing the refusal that names this scenario
            // (matches the QSELECT/QFLOW serialized-tamper precedent). Never a vacuous
            // replay of the successful demo.
            let json = learner_model_demo_json();
            let refused = verify_learner_model_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(LearnerModelRefusal::SerializedLearnerModelTamper)
            } else {
                None
            };
            LearnerModelCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: refusal.map(|r| r.slug().to_string()),
                learner_state_mapped: false,
                seen_items: 0,
                quiz_total: 0,
                quiz_answered: 0,
                misconception_flags: 0,
                boundary_all_inert: LearnerModelBoundary::inert().all_inert(),
            }
        }
        "teach_map_refusal_propagates" => {
            let mut config = crate::TeachMapConfig::default_config();
            config.uses_model = true;
            let teach = crate::run_teach_map(&crate::literature_intent_demo(), config);
            let run = run_learner_model_default(&teach, empty_observation());
            cell_from_run(scenario, &run)
        }
        "unknown_seen_item_refused" => {
            let teach = teach_map_demo();
            let mut observation = learner_model_demo_observation(&teach);
            observation
                .seen_lesson_item_ids
                .push("lesson_item:unknown".to_string());
            let run = run_learner_model_default(&teach, observation);
            cell_from_run(scenario, &run)
        }
        "unknown_quiz_item_refused" => {
            let teach = teach_map_demo();
            let mut observation = learner_model_demo_observation(&teach);
            observation.quiz_answers.push(LearnerQuizAnswerObservation {
                quiz_id: "quiz:999".to_string(),
                answer: "foreign".to_string(),
            });
            let run = run_learner_model_default(&teach, observation);
            cell_from_run(scenario, &run)
        }
        "unknown_misconception_flag_refused" => {
            let teach = teach_map_demo();
            let mut observation = learner_model_demo_observation(&teach);
            observation
                .misconception_flags
                .push(LearnerMisconceptionObservation {
                    check_id: "misconception_check:999".to_string(),
                    flagged: true,
                });
            let run = run_learner_model_default(&teach, observation);
            cell_from_run(scenario, &run)
        }
        "no_model_signal_detected" => {
            let teach = teach_map_demo();
            let mut config = LearnerModelConfig::default_config();
            config.uses_model = true;
            let run = run_learner_model(&teach, learner_model_demo_observation(&teach), config);
            cell_from_run(scenario, &run)
        }
        "no_training_signal_detected" => {
            let teach = teach_map_demo();
            let mut config = LearnerModelConfig::default_config();
            config.uses_training = true;
            let run = run_learner_model(&teach, learner_model_demo_observation(&teach), config);
            cell_from_run(scenario, &run)
        }
        "personalization_signal_refused" => {
            let teach = teach_map_demo();
            let mut config = LearnerModelConfig::default_config();
            config.personalizes = true;
            let run = run_learner_model(&teach, learner_model_demo_observation(&teach), config);
            cell_from_run(scenario, &run)
        }
        "memory_adaptation_health_diagnosis_signals_refused" => {
            let teach = teach_map_demo();
            let mut config = LearnerModelConfig::default_config();
            config.writes_memory = true;
            config.autonomously_adapts = true;
            config.infers_health_profile = true;
            config.infers_hidden_diagnosis = true;
            let run = run_learner_model(&teach, learner_model_demo_observation(&teach), config);
            cell_from_run(scenario, &run)
        }
        other => LearnerModelCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            learner_state_mapped: false,
            seen_items: 0,
            quiz_total: 0,
            quiz_answered: 0,
            misconception_flags: 0,
            boundary_all_inert: false,
        },
    }
}

fn empty_observation() -> LearnerModelObservation {
    LearnerModelObservation {
        seen_lesson_item_ids: vec![],
        quiz_answers: vec![],
        misconception_flags: vec![],
        confidence_marker: ConfidenceMarker::Unstated,
    }
}

fn cell_from_run(scenario: &str, run: &LearnerModelRun) -> LearnerModelCell {
    LearnerModelCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        learner_state_mapped: run.decision == LearnerModelDecision::LearnerStateMapped,
        seen_items: run.receipt.seen_item_count,
        quiz_total: run.receipt.quiz_total,
        quiz_answered: run.receipt.quiz_answered,
        misconception_flags: run.receipt.misconception_flag_count,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

pub fn learner_model_matrix() -> LearnerModelMatrix {
    let cells = LEARNER_MODEL_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let mapped_count = cells
        .iter()
        .filter(|cell| cell.learner_state_mapped)
        .count();
    let refused_count = cells
        .iter()
        .filter(|cell| !cell.learner_state_mapped)
        .count();
    LearnerModelMatrix {
        schema: SCHEMA.to_string(),
        scenario_count: LEARNER_MODEL_SCENARIO_COUNT,
        cells,
        mapped_count,
        refused_count,
        boundary: LearnerModelBoundary::inert(),
        boundary_all_inert: LearnerModelBoundary::inert().all_inert(),
    }
}

pub fn learner_model_matrix_json() -> String {
    serde_json::to_string(&learner_model_matrix()).expect("learner model matrix serializes")
}

pub fn verify_learner_model_matrix_json(candidate: &str) -> Result<(), LearnerModelError> {
    if candidate == learner_model_matrix_json() {
        Ok(())
    } else {
        Err(LearnerModelError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_state() -> LearnerStateMap {
        learner_model_demo().learner_state.expect("learner state")
    }

    fn support_is_learner_backed(support: &[LearnerSupportRef]) -> bool {
        !support.is_empty()
            && support.iter().all(|span| {
                span.intent_authority == REQUIRED_INTENT_AUTHORITY
                    && span.teach_authority == REQUIRED_TEACH_AUTHORITY
                    && span.learner_authority == AUTHORITY_LEARNER_STATE_FROM_TEACH_MAP
            })
    }

    fn has_refusal(state: &LearnerStateMap, field: &str) -> bool {
        state.refusals.iter().any(|r| r.field == field)
    }

    #[test]
    fn supported_teach_map_builds_learner_state() {
        let run = learner_model_demo();
        assert_eq!(run.decision, LearnerModelDecision::LearnerStateMapped);
        assert!(run.refusal.is_none());
        let state = run.learner_state.expect("state");
        assert_eq!(state.document, "companion.md");
        assert!(!state.seen_items.is_empty());
        assert!(state.concept_taught.is_some());
        assert_eq!(state.quiz_result.total, 3);
        assert!(!state.misconception_flags.is_empty());
        assert!(state.next_review_target.is_some());
        assert_eq!(
            state.source_teach_receipt_hash,
            run.receipt.source_teach_receipt_hash
        );
    }

    #[test]
    fn seen_items_are_receipt_linked() {
        let state = demo_state();
        assert!(state.receipt_link.starts_with("teach_receipt:"));
        for item in &state.seen_items {
            assert!(support_is_learner_backed(&item.support));
        }
    }

    #[test]
    fn concept_taught_is_supported() {
        let state = demo_state();
        let concept = state.concept_taught.expect("concept");
        assert!(concept.label.contains("central thesis"));
        assert_eq!(concept.source_item_id, "explanation:1");
        assert!(support_is_learner_backed(&concept.support));
    }

    #[test]
    fn quiz_result_uses_exact_match_only() {
        let state = demo_state();
        assert_eq!(state.quiz_result.total, 3);
        assert_eq!(state.quiz_result.answered, 2);
        assert_eq!(state.quiz_result.correct_exact_match, 1);
        assert_eq!(state.quiz_result.incorrect_exact_mismatch, 1);
        assert_eq!(state.quiz_result.unanswered, 1);
        assert!(state
            .quiz_result
            .items
            .iter()
            .all(|item| support_is_learner_backed(&item.support)));
    }

    #[test]
    fn misconception_flags_are_explicit_not_diagnostic() {
        let state = demo_state();
        assert!(state.misconception_flags.iter().any(|flag| flag.flagged));
        for flag in &state.misconception_flags {
            assert!(flag.source == "explicit_marker" || flag.source == "not_marked");
            assert!(!flag.hidden_diagnosis);
            assert!(support_is_learner_backed(&flag.support));
        }
    }

    #[test]
    fn confidence_marker_is_self_reported_not_inferred() {
        let state = demo_state();
        assert_eq!(state.confidence_marker.marker, ConfidenceMarker::Medium);
        assert_eq!(state.confidence_marker.source, "self_reported_marker");
        assert!(!state.confidence_marker.inferred);
    }

    #[test]
    fn next_review_target_is_non_adaptive() {
        let state = demo_state();
        let target = state.next_review_target.expect("target");
        assert_eq!(target.source, "teach_map_next_reading_step");
        assert!(!target.autonomously_adapted);
        assert!(support_is_learner_backed(&target.support));
    }

    #[test]
    fn teach_map_refusal_propagates() {
        let mut config = crate::TeachMapConfig::default_config();
        config.uses_model = true;
        let teach = crate::run_teach_map(&crate::literature_intent_demo(), config);
        let run = run_learner_model_default(&teach, empty_observation());
        assert_eq!(run.decision, LearnerModelDecision::LearnerStateRefused);
        assert_eq!(run.refusal, Some(LearnerModelRefusal::TeachMapRefused));
        assert!(run.learner_state.is_none());
    }

    #[test]
    fn unknown_seen_quiz_and_misconception_inputs_are_refused() {
        let teach = teach_map_demo();

        let mut observation = learner_model_demo_observation(&teach);
        observation
            .seen_lesson_item_ids
            .push("lesson_item:unknown".to_string());
        let run = run_learner_model_default(&teach, observation);
        assert_eq!(
            run.refusal,
            Some(LearnerModelRefusal::UnrecognizedLessonItem)
        );

        let mut observation = learner_model_demo_observation(&teach);
        observation.quiz_answers.push(LearnerQuizAnswerObservation {
            quiz_id: "quiz:999".to_string(),
            answer: "foreign".to_string(),
        });
        let run = run_learner_model_default(&teach, observation);
        assert_eq!(run.refusal, Some(LearnerModelRefusal::UnrecognizedQuizItem));

        let mut observation = learner_model_demo_observation(&teach);
        observation
            .misconception_flags
            .push(LearnerMisconceptionObservation {
                check_id: "misconception_check:999".to_string(),
                flagged: true,
            });
        let run = run_learner_model_default(&teach, observation);
        assert_eq!(
            run.refusal,
            Some(LearnerModelRefusal::UnrecognizedMisconceptionCheck)
        );
    }

    #[test]
    fn no_seen_items_is_refused() {
        let teach = teach_map_demo();
        let mut observation = learner_model_demo_observation(&teach);
        observation.seen_lesson_item_ids.clear();
        let run = run_learner_model_default(&teach, observation);
        assert_eq!(run.refusal, Some(LearnerModelRefusal::NoLessonItemsSeen));
    }

    #[test]
    fn model_training_personalization_memory_adaptation_health_and_diagnosis_signals_refuse() {
        let teach = teach_map_demo();

        let mut config = LearnerModelConfig::default_config();
        config.uses_model = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::ModelSignalDetected)
        );

        let mut config = LearnerModelConfig::default_config();
        config.uses_training = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::TrainingSignalDetected)
        );

        let mut config = LearnerModelConfig::default_config();
        config.personalizes = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::PersonalizationSignalDetected)
        );

        let mut config = LearnerModelConfig::default_config();
        config.writes_memory = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::MemoryWriteSignalDetected)
        );

        let mut config = LearnerModelConfig::default_config();
        config.autonomously_adapts = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::AutonomousAdaptationSignalDetected)
        );

        let mut config = LearnerModelConfig::default_config();
        config.infers_health_profile = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::HealthProfileSignalDetected)
        );

        let mut config = LearnerModelConfig::default_config();
        config.infers_hidden_diagnosis = true;
        assert_eq!(
            run_learner_model(&teach, learner_model_demo_observation(&teach), config).refusal,
            Some(LearnerModelRefusal::HiddenDiagnosisSignalDetected)
        );
    }

    #[test]
    fn boundary_is_inert_and_recorded() {
        let run = learner_model_demo();
        assert!(run.receipt.boundary_all_inert);
        assert_eq!(LEARNER_MODEL_BOUNDARY_LINES.len(), 9);
        assert_eq!(
            LEARNER_MODEL_BOUNDARY_LINES[0],
            "LEARNER-MODEL-0 records a local learner-state map only."
        );
        let state = run.learner_state.expect("state");
        assert!(has_refusal(&state, "personalization"));
        assert!(has_refusal(&state, "long_term_memory"));
        assert!(has_refusal(&state, "autonomous_adaptation"));
        assert!(has_refusal(&state, "health_or_psych_profile"));
        assert!(has_refusal(&state, "hidden_diagnosis"));
    }

    #[test]
    fn demo_json_re_derives_and_refuses_tampering() {
        let json = learner_model_demo_json();
        assert!(json.contains("\"learner_state\""));
        assert!(verify_learner_model_demo_json(&json).is_ok());
        assert_eq!(
            verify_learner_model_demo_json(&format!("{json} ")),
            Err(LearnerModelError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_has_named_scenarios_and_replays() {
        let matrix = learner_model_matrix();
        assert_eq!(matrix.scenario_count, LEARNER_MODEL_SCENARIO_COUNT);
        assert_eq!(matrix.cells.len(), LEARNER_MODEL_SCENARIO_COUNT);
        assert_eq!(
            matrix.mapped_count + matrix.refused_count,
            LEARNER_MODEL_SCENARIO_COUNT
        );
        for (cell, name) in matrix.cells.iter().zip(LEARNER_MODEL_SCENARIO_NAMES.iter()) {
            assert_eq!(&cell.scenario, name);
            assert_ne!(cell.outcome, "unknown");
        }
        let json = learner_model_matrix_json();
        assert!(verify_learner_model_matrix_json(&json).is_ok());
        assert_eq!(
            verify_learner_model_matrix_json(&format!("{json} ")),
            Err(LearnerModelError::ReplayMismatch)
        );
    }

    #[test]
    fn decisions_and_refusals_are_complete_and_slugged() {
        assert_eq!(LearnerModelDecision::ALL.len(), 2);
        assert_eq!(LearnerModelRefusal::ALL.len(), 15);
        let mut slugs = LearnerModelRefusal::ALL
            .iter()
            .map(|r| r.slug())
            .collect::<Vec<_>>();
        slugs.sort_unstable();
        let n = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), n);
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        // The matrix scenario must actually construct the tamper refusal from a
        // byte-flipped artifact, not vacuously replay the successful demo. This is
        // the QFLOW-0 A3 lesson: a slugged variant that is never constructed is debris.
        let matrix = learner_model_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "serialized_learner_model_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_learner_model_tamper_refused")
        );
        assert!(!cell.learner_state_mapped);
        let json = learner_model_demo_json();
        assert!(verify_learner_model_demo_json(&flip_last_byte(&json)).is_err());
    }

    #[test]
    fn unsupported_lesson_support_is_refused() {
        // A lesson item whose support loses the required intent authority must be
        // refused, never mapped into learner state. Guards the "everything span-backed"
        // invariant against silent regression.
        let mut teach = teach_map_demo();
        {
            let lesson = teach.lesson.as_mut().expect("demo lesson present");
            let block = lesson
                .explanation
                .as_mut()
                .expect("demo explanation present");
            let span = block
                .support
                .first_mut()
                .expect("demo explanation support present");
            span.intent_authority = "wrong_authority".to_string();
        }
        let observation = learner_model_demo_observation(&teach);
        let run = run_learner_model_default(&teach, observation);
        assert_eq!(run.decision, LearnerModelDecision::LearnerStateRefused);
        assert_eq!(
            run.refusal,
            Some(LearnerModelRefusal::UnsupportedLessonSupport)
        );
        assert!(run.learner_state.is_none());
    }

    #[test]
    fn learner_receipt_folds_source_intent_hash() {
        // receipt_hash must fold the TEACH-0/intent source hashes, so changing only
        // the source intent receipt hash changes the learner receipt hash.
        let teach = teach_map_demo();
        let observation = learner_model_demo_observation(&teach);
        let run_a = run_learner_model_default(&teach, observation.clone());

        let mut teach2 = teach_map_demo();
        teach2.receipt.source_intent_receipt_hash ^= 0x0000_0000_0000_abcd;
        let run_b = run_learner_model_default(&teach2, observation);

        assert_ne!(
            run_a.receipt.receipt_hash, run_b.receipt.receipt_hash,
            "learner receipt_hash must fold the source intent receipt hash"
        );
    }
}
