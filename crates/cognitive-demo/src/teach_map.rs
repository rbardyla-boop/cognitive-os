//! TEACH-0 — deterministic lessons from bounded LIT-INTENT maps.
//!
//! This is a MAP-only teaching surface. It consumes a [`LiteratureIntentRun`]
//! whose map was already built from verified QFLOW spans, and turns the
//! span-backed findings into a small lesson: explanation, examples,
//! misconception checks, quiz items, and a next reading step. It does NOT
//! personalize, does NOT remember the user, does NOT call a model, and does NOT
//! teach unsupported content. Missing teaching sections become field-level
//! refusals, never invented prose.

use serde::Serialize;

use crate::{
    literature_intent_demo, run_literature_intent_map_default, IntentSpanRef,
    LiteratureIntentDecision, LiteratureIntentMap, LiteratureIntentRefusal, LiteratureIntentRun,
    SpanBackedFinding,
};

const SCHEMA: &str = "teach-map-v0.1";
const TEACH_MAP_USES_MODEL: bool = false;
const TEACH_MAP_USES_TRAINING: bool = false;
const TEACH_MAP_PERSONALIZES: bool = false;
const TEACH_MAP_WRITES_MEMORY: bool = false;
const REQUIRED_INTENT_AUTHORITY: &str = "intent_map_from_verified_span";
const AUTHORITY_TEACH_FROM_INTENT_MAP: &str = "teach_from_span_backed_intent_map";

/// The authority boundary, verbatim. TEACH-0 teaches only from a bounded map.
pub const TEACH_MAP_BOUNDARY_LINES: [&str; 8] = [
    "TEACH-0 teaches from span-backed intent-map findings only.",
    "It does not create truth.",
    "It does not claim full comprehension.",
    "It does not infer hidden author motives.",
    "It does not teach unsupported content.",
    "It does not personalize.",
    "It does not read or write learner memory.",
    "It does not train or run a model.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TeachMapDecision {
    LessonBuilt,
    LessonRefused,
}

impl TeachMapDecision {
    pub const ALL: [TeachMapDecision; 2] = [
        TeachMapDecision::LessonBuilt,
        TeachMapDecision::LessonRefused,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            TeachMapDecision::LessonBuilt => "lesson_built",
            TeachMapDecision::LessonRefused => "lesson_refused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TeachMapRefusal {
    IntentMapRefused,
    IntentMapUnavailable,
    NoSpanBackedFindings,
    UnsupportedTeachingContent,
    ModelSignalDetected,
    TrainingSignalDetected,
    PersonalizationSignalDetected,
    MemorySignalDetected,
    SerializedTeachMapTamper,
}

impl TeachMapRefusal {
    pub const ALL: [TeachMapRefusal; 9] = [
        TeachMapRefusal::IntentMapRefused,
        TeachMapRefusal::IntentMapUnavailable,
        TeachMapRefusal::NoSpanBackedFindings,
        TeachMapRefusal::UnsupportedTeachingContent,
        TeachMapRefusal::ModelSignalDetected,
        TeachMapRefusal::TrainingSignalDetected,
        TeachMapRefusal::PersonalizationSignalDetected,
        TeachMapRefusal::MemorySignalDetected,
        TeachMapRefusal::SerializedTeachMapTamper,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            TeachMapRefusal::IntentMapRefused => "intent_map_refused",
            TeachMapRefusal::IntentMapUnavailable => "intent_map_unavailable_refused",
            TeachMapRefusal::NoSpanBackedFindings => "no_span_backed_findings_refused",
            TeachMapRefusal::UnsupportedTeachingContent => "unsupported_teaching_content_refused",
            TeachMapRefusal::ModelSignalDetected => "model_signal_detected_refused",
            TeachMapRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            TeachMapRefusal::PersonalizationSignalDetected => {
                "personalization_signal_detected_refused"
            }
            TeachMapRefusal::MemorySignalDetected => "memory_signal_detected_refused",
            TeachMapRefusal::SerializedTeachMapTamper => "serialized_teach_map_tamper_refused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeachMapError {
    ReplayMismatch,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TeachMapConfig {
    pub uses_model: bool,
    pub uses_training: bool,
    pub personalizes: bool,
    pub writes_memory: bool,
}

impl TeachMapConfig {
    pub fn default_config() -> Self {
        TeachMapConfig {
            uses_model: TEACH_MAP_USES_MODEL,
            uses_training: TEACH_MAP_USES_TRAINING,
            personalizes: TEACH_MAP_PERSONALIZES,
            writes_memory: TEACH_MAP_WRITES_MEMORY,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TeachMapBoundary {
    pub creates_truth: bool,
    pub claims_full_comprehension: bool,
    pub infers_hidden_author_motives: bool,
    pub teaches_unsupported_content: bool,
    pub personalizes: bool,
    pub reads_or_writes_memory: bool,
    pub trains: bool,
    pub is_model: bool,
}

impl TeachMapBoundary {
    fn inert() -> Self {
        TeachMapBoundary {
            creates_truth: TEACH_MAP_USES_MODEL,
            claims_full_comprehension: TEACH_MAP_USES_MODEL,
            infers_hidden_author_motives: TEACH_MAP_USES_MODEL,
            teaches_unsupported_content: TEACH_MAP_USES_MODEL,
            personalizes: TEACH_MAP_PERSONALIZES,
            reads_or_writes_memory: TEACH_MAP_WRITES_MEMORY,
            trains: TEACH_MAP_USES_TRAINING,
            is_model: TEACH_MAP_USES_MODEL,
        }
    }

    fn all_inert(&self) -> bool {
        !self.creates_truth
            && !self.claims_full_comprehension
            && !self.infers_hidden_author_motives
            && !self.teaches_unsupported_content
            && !self.personalizes
            && !self.reads_or_writes_memory
            && !self.trains
            && !self.is_model
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TeachSupportRef {
    pub document_id: u64,
    pub document_name: String,
    pub span_id: u64,
    pub text: String,
    pub intent_authority: String,
    pub teach_authority: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LessonBlock {
    pub text: String,
    pub support: Vec<TeachSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MisconceptionCheck {
    pub misconception: String,
    pub correction: String,
    pub support: Vec<TeachSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct QuizItem {
    pub question: String,
    pub expected_answer: String,
    pub support: Vec<TeachSupportRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TeachFieldRefusal {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TeachLesson {
    pub document: String,
    pub explanation: Option<LessonBlock>,
    pub examples: Vec<LessonBlock>,
    pub misconception_checks: Vec<MisconceptionCheck>,
    pub quiz: Vec<QuizItem>,
    pub next_reading_step: Option<LessonBlock>,
    pub refusals: Vec<TeachFieldRefusal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeachMapReceipt {
    pub schema: String,
    pub source_intent_receipt_hash: u64,
    pub source_intent_decision: String,
    pub source_intent_refusal: Option<String>,
    pub config: TeachMapConfig,
    pub lesson_item_count: usize,
    pub field_refusals: usize,
    pub decision: TeachMapDecision,
    pub refusal: Option<TeachMapRefusal>,
    pub receipt_hash: u64,
    pub boundary: TeachMapBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeachMapRun {
    pub receipt: TeachMapReceipt,
    pub lesson: Option<TeachLesson>,
    pub decision: TeachMapDecision,
    pub refusal: Option<TeachMapRefusal>,
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
    intent_run: &LiteratureIntentRun,
    config: &TeachMapConfig,
    lesson: Option<&TeachLesson>,
    decision: TeachMapDecision,
    refusal: Option<TeachMapRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    h = fnv_mix(h, SCHEMA.as_bytes());
    h = fnv_u64(h, intent_run.receipt.receipt_hash);
    h = fnv_mix(h, intent_run.decision.slug().as_bytes());
    h = fnv_mix(
        h,
        intent_run
            .refusal
            .map(LiteratureIntentRefusal::slug)
            .unwrap_or("none")
            .as_bytes(),
    );
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, config.personalizes as u64);
    h = fnv_u64(h, config.writes_memory as u64);
    if let Some(lesson) = lesson {
        h = fnv_mix(h, lesson.document.as_bytes());
        for block in lesson_blocks(lesson) {
            h = fnv_mix(h, block.text.as_bytes());
            h = mix_support(h, &block.support);
        }
        for check in &lesson.misconception_checks {
            h = fnv_mix(h, check.misconception.as_bytes());
            h = fnv_mix(h, check.correction.as_bytes());
            h = mix_support(h, &check.support);
        }
        for item in &lesson.quiz {
            h = fnv_mix(h, item.question.as_bytes());
            h = fnv_mix(h, item.expected_answer.as_bytes());
            h = mix_support(h, &item.support);
        }
        for refusal in &lesson.refusals {
            h = fnv_mix(h, refusal.field.as_bytes());
            h = fnv_mix(h, refusal.reason.as_bytes());
        }
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

fn mix_support(mut h: u64, support: &[TeachSupportRef]) -> u64 {
    h = fnv_u64(h, support.len() as u64);
    for item in support {
        h = fnv_u64(h, item.document_id);
        h = fnv_u64(h, item.span_id);
        h = fnv_mix(h, item.document_name.as_bytes());
        h = fnv_mix(h, item.text.as_bytes());
        h = fnv_mix(h, item.intent_authority.as_bytes());
        h = fnv_mix(h, item.teach_authority.as_bytes());
    }
    h
}

pub fn teach_map_demo() -> TeachMapRun {
    run_teach_map_default(&literature_intent_demo())
}

pub fn teach_map_demo_json() -> String {
    serde_json::to_string_pretty(&teach_map_demo()).expect("teach map demo serializes")
}

pub fn verify_teach_map_demo_json(candidate: &str) -> Result<(), TeachMapError> {
    if candidate == teach_map_demo_json() {
        Ok(())
    } else {
        Err(TeachMapError::ReplayMismatch)
    }
}

fn flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last = last.wrapping_add(1);
    }
    String::from_utf8(bytes).expect("json stays utf8 after single-byte flip")
}

pub fn run_teach_map_default(intent_run: &LiteratureIntentRun) -> TeachMapRun {
    run_teach_map(intent_run, TeachMapConfig::default_config())
}

pub fn run_teach_map(intent_run: &LiteratureIntentRun, config: TeachMapConfig) -> TeachMapRun {
    if config.uses_model {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::ModelSignalDetected),
            None,
        );
    }
    if config.uses_training {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::TrainingSignalDetected),
            None,
        );
    }
    if config.personalizes {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::PersonalizationSignalDetected),
            None,
        );
    }
    if config.writes_memory {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::MemorySignalDetected),
            None,
        );
    }
    if intent_run.decision == LiteratureIntentDecision::IntentMapRefused {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::IntentMapRefused),
            None,
        );
    }
    let map = match intent_run.map.as_ref() {
        Some(map) => map,
        None => {
            return assemble(
                intent_run,
                config,
                TeachMapDecision::LessonRefused,
                Some(TeachMapRefusal::IntentMapUnavailable),
                None,
            );
        }
    };
    if !map_has_span_backed_finding(map) {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::NoSpanBackedFindings),
            None,
        );
    }

    let lesson = build_lesson(map);
    if !lesson_items_are_supported(&lesson) {
        return assemble(
            intent_run,
            config,
            TeachMapDecision::LessonRefused,
            Some(TeachMapRefusal::UnsupportedTeachingContent),
            None,
        );
    }
    assemble(
        intent_run,
        config,
        TeachMapDecision::LessonBuilt,
        None,
        Some(lesson),
    )
}

fn assemble(
    intent_run: &LiteratureIntentRun,
    config: TeachMapConfig,
    decision: TeachMapDecision,
    refusal: Option<TeachMapRefusal>,
    lesson: Option<TeachLesson>,
) -> TeachMapRun {
    let boundary = TeachMapBoundary::inert();
    let lesson_item_count = lesson.as_ref().map(lesson_item_count).unwrap_or(0);
    let field_refusals = lesson.as_ref().map(|l| l.refusals.len()).unwrap_or(0);
    let receipt_hash = receipt_hash(intent_run, &config, lesson.as_ref(), decision, refusal);
    let receipt = TeachMapReceipt {
        schema: SCHEMA.to_string(),
        source_intent_receipt_hash: intent_run.receipt.receipt_hash,
        source_intent_decision: intent_run.decision.slug().to_string(),
        source_intent_refusal: intent_run.refusal.map(|r| r.slug().to_string()),
        config,
        lesson_item_count,
        field_refusals,
        decision,
        refusal,
        receipt_hash,
        boundary,
        boundary_all_inert: boundary.all_inert(),
    };
    TeachMapRun {
        receipt,
        lesson,
        decision,
        refusal,
    }
}

fn map_has_span_backed_finding(map: &LiteratureIntentMap) -> bool {
    map.central_thesis
        .as_ref()
        .map(|finding| valid_intent_support(&finding.support))
        .unwrap_or(false)
        || map
            .author_intent
            .as_ref()
            .map(|finding| valid_intent_support(&finding.support))
            .unwrap_or(false)
        || map
            .core_claims
            .iter()
            .any(|finding| valid_intent_support(&finding.support))
        || map
            .key_terms
            .iter()
            .any(|finding| valid_intent_support(&finding.support))
        || map
            .assumptions
            .iter()
            .any(|finding| valid_intent_support(&finding.support))
        || map
            .tensions_or_contradictions
            .iter()
            .any(|finding| valid_intent_support(&finding.support))
}

fn build_lesson(map: &LiteratureIntentMap) -> TeachLesson {
    let mut refusals = vec![
        field_refusal("hidden_author_motives", "not inferred by TEACH-0"),
        field_refusal("full_comprehension", "not claimed by TEACH-0"),
        field_refusal("personalization", "not performed by TEACH-0"),
        field_refusal("learner_memory", "not read or written by TEACH-0"),
    ];

    let explanation = explanation_block(map);
    if explanation.is_none() {
        refusals.push(field_refusal(
            "explanation",
            "no span-backed thesis or claim was available for an explanation",
        ));
    }

    let examples = example_blocks(map);
    if examples.is_empty() {
        refusals.push(field_refusal(
            "examples",
            "no span-backed term, assumption, or claim was available for examples",
        ));
    }

    let misconception_checks = misconception_checks(map);
    if misconception_checks.is_empty() {
        refusals.push(field_refusal(
            "misconception_checks",
            "no span-backed intent or tension was available for a misconception check",
        ));
    }

    let quiz = quiz_items(map);
    if quiz.is_empty() {
        refusals.push(field_refusal(
            "quiz",
            "no span-backed thesis, term, or tension was available for a quiz item",
        ));
    }

    let next_reading_step = next_reading_step(map);
    if next_reading_step.is_none() {
        refusals.push(field_refusal(
            "next_reading_step",
            "no span-backed tension, assumption, thesis, or claim was available for a next step",
        ));
    }

    TeachLesson {
        document: map.document.clone(),
        explanation,
        examples,
        misconception_checks,
        quiz,
        next_reading_step,
        refusals,
    }
}

fn explanation_block(map: &LiteratureIntentMap) -> Option<LessonBlock> {
    if let Some(thesis) = map.central_thesis.as_ref() {
        return block_from_finding(
            format!(
                "Explain the text from its verified central thesis: {}",
                thesis.statement
            ),
            thesis,
        );
    }
    map.core_claims.first().and_then(|claim| {
        block_from_finding(
            format!(
                "Explain only the first verified claim because no central thesis was mapped: {}",
                claim.statement
            ),
            claim,
        )
    })
}

fn example_blocks(map: &LiteratureIntentMap) -> Vec<LessonBlock> {
    let mut examples = Vec::new();
    if let Some(term) = map.key_terms.first() {
        if valid_intent_support(&term.support) {
            examples.push(LessonBlock {
                text: format!(
                    "Example: use '{}' only as the verified wording uses it: {}",
                    term.term, term.usage_or_definition
                ),
                support: teach_support(&term.support),
            });
        }
    }
    if let Some(assumption) = map.assumptions.first() {
        if let Some(block) = block_from_finding(
            format!(
                "Example: treat this as a required condition, not as extra proof: {}",
                assumption.statement
            ),
            assumption,
        ) {
            examples.push(block);
        }
    }
    if examples.is_empty() {
        if let Some(claim) = map.core_claims.first() {
            if let Some(block) = block_from_finding(
                format!(
                    "Example: keep the lesson attached to this verified claim: {}",
                    claim.statement
                ),
                claim,
            ) {
                examples.push(block);
            }
        }
    }
    examples
}

fn misconception_checks(map: &LiteratureIntentMap) -> Vec<MisconceptionCheck> {
    let mut checks = Vec::new();
    if let Some(tension) = map.tensions_or_contradictions.first() {
        if valid_intent_support(&tension.support) {
            checks.push(MisconceptionCheck {
                misconception: "The lesson can ignore the text's warning or tension.".to_string(),
                correction: format!("Use the verified tension instead: {}", tension.statement),
                support: teach_support(&tension.support),
            });
        }
    }
    if let Some(intent) = map.author_intent.as_ref() {
        if valid_intent_support(&intent.support) {
            checks.push(MisconceptionCheck {
                misconception: "The lesson can replace explicit purpose with hidden motive."
                    .to_string(),
                correction: format!(
                    "Stay with the bounded explicit intent: {}",
                    intent.statement
                ),
                support: teach_support(&intent.support),
            });
        }
    }
    checks
}

fn quiz_items(map: &LiteratureIntentMap) -> Vec<QuizItem> {
    let mut items = Vec::new();
    if let Some(thesis) = map.central_thesis.as_ref() {
        if valid_intent_support(&thesis.support) {
            items.push(QuizItem {
                question: "What central thesis can be taught from the verified map?".to_string(),
                expected_answer: thesis.statement.clone(),
                support: teach_support(&thesis.support),
            });
        }
    }
    if let Some(term) = map.key_terms.first() {
        if valid_intent_support(&term.support) {
            items.push(QuizItem {
                question: format!("How does the text use '{}'?", term.term),
                expected_answer: term.usage_or_definition.clone(),
                support: teach_support(&term.support),
            });
        }
    }
    if let Some(tension) = map.tensions_or_contradictions.first() {
        if valid_intent_support(&tension.support) {
            items.push(QuizItem {
                question: "What warning or tension should the learner keep in view?".to_string(),
                expected_answer: tension.statement.clone(),
                support: teach_support(&tension.support),
            });
        }
    }
    if items.is_empty() {
        if let Some(claim) = map.core_claims.first() {
            if valid_intent_support(&claim.support) {
                items.push(QuizItem {
                    question: "Which verified claim can the learner repeat without adding unsupported content?"
                        .to_string(),
                    expected_answer: claim.statement.clone(),
                    support: teach_support(&claim.support),
                });
            }
        }
    }
    items
}

fn next_reading_step(map: &LiteratureIntentMap) -> Option<LessonBlock> {
    if let Some(tension) = map.tensions_or_contradictions.first() {
        return block_from_finding(
            format!(
                "Next, reread the verified tension before expanding the lesson: {}",
                tension.statement
            ),
            tension,
        );
    }
    if let Some(assumption) = map.assumptions.first() {
        return block_from_finding(
            format!(
                "Next, inspect the verified assumption and decide what evidence would test it: {}",
                assumption.statement
            ),
            assumption,
        );
    }
    if let Some(thesis) = map.central_thesis.as_ref() {
        return block_from_finding(
            format!(
                "Next, compare every later claim against the verified thesis: {}",
                thesis.statement
            ),
            thesis,
        );
    }
    map.core_claims.first().and_then(|claim| {
        block_from_finding(
            format!(
                "Next, find whether the text states a central thesis for this verified claim: {}",
                claim.statement
            ),
            claim,
        )
    })
}

fn block_from_finding(text: String, finding: &SpanBackedFinding) -> Option<LessonBlock> {
    if valid_intent_support(&finding.support) {
        Some(LessonBlock {
            text,
            support: teach_support(&finding.support),
        })
    } else {
        None
    }
}

fn valid_intent_support(support: &[IntentSpanRef]) -> bool {
    !support.is_empty()
        && support.iter().all(|span| {
            span.authority == REQUIRED_INTENT_AUTHORITY
                && span.text.chars().any(|c| c.is_alphanumeric())
        })
}

fn teach_support(support: &[IntentSpanRef]) -> Vec<TeachSupportRef> {
    support
        .iter()
        .map(|span| TeachSupportRef {
            document_id: span.document_id,
            document_name: span.document_name.clone(),
            span_id: span.span_id,
            text: span.text.clone(),
            intent_authority: span.authority.clone(),
            teach_authority: AUTHORITY_TEACH_FROM_INTENT_MAP.to_string(),
        })
        .collect()
}

fn lesson_items_are_supported(lesson: &TeachLesson) -> bool {
    lesson
        .explanation
        .as_ref()
        .map(supported_block)
        .unwrap_or(true)
        && lesson.examples.iter().all(supported_block)
        && lesson
            .misconception_checks
            .iter()
            .all(|check| supported_teach_support(&check.support))
        && lesson
            .quiz
            .iter()
            .all(|item| supported_teach_support(&item.support))
        && lesson
            .next_reading_step
            .as_ref()
            .map(supported_block)
            .unwrap_or(true)
}

fn supported_block(block: &LessonBlock) -> bool {
    supported_teach_support(&block.support)
}

fn supported_teach_support(support: &[TeachSupportRef]) -> bool {
    !support.is_empty()
        && support.iter().all(|span| {
            span.intent_authority == REQUIRED_INTENT_AUTHORITY
                && span.teach_authority == AUTHORITY_TEACH_FROM_INTENT_MAP
                && span.text.chars().any(|c| c.is_alphanumeric())
        })
}

fn lesson_blocks(lesson: &TeachLesson) -> Vec<&LessonBlock> {
    let mut blocks = Vec::new();
    if let Some(explanation) = lesson.explanation.as_ref() {
        blocks.push(explanation);
    }
    blocks.extend(lesson.examples.iter());
    if let Some(next) = lesson.next_reading_step.as_ref() {
        blocks.push(next);
    }
    blocks
}

fn lesson_item_count(lesson: &TeachLesson) -> usize {
    lesson.explanation.iter().count()
        + lesson.examples.len()
        + lesson.misconception_checks.len()
        + lesson.quiz.len()
        + lesson.next_reading_step.iter().count()
}

fn field_refusal(field: &str, reason: &str) -> TeachFieldRefusal {
    TeachFieldRefusal {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

pub const TEACH_MAP_SCENARIO_COUNT: usize = 13;
pub const TEACH_MAP_SCENARIO_NAMES: [&str; TEACH_MAP_SCENARIO_COUNT] = [
    "verified_intent_map_builds_teach_map",
    "explanation_is_span_backed",
    "examples_are_span_backed",
    "misconception_checks_are_span_backed",
    "quiz_is_span_backed",
    "next_reading_step_is_span_backed",
    "unsupported_teaching_content_is_refused",
    "intent_map_refusal_propagates",
    "no_model_signal_detected",
    "no_training_signal_detected",
    "personalization_signal_refused",
    "memory_signal_refused",
    "serialized_teach_map_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct TeachMapCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub lesson_built: bool,
    pub lesson_items: usize,
    pub field_refusals: usize,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeachMapMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<TeachMapCell>,
    pub built_count: usize,
    pub refused_count: usize,
    pub boundary: TeachMapBoundary,
    pub boundary_all_inert: bool,
}

fn cell_for(scenario: &str) -> TeachMapCell {
    match scenario {
        "verified_intent_map_builds_teach_map"
        | "explanation_is_span_backed"
        | "examples_are_span_backed"
        | "misconception_checks_are_span_backed"
        | "quiz_is_span_backed"
        | "next_reading_step_is_span_backed" => {
            let run = teach_map_demo();
            cell_from_run(scenario, &run)
        }
        "unsupported_teaching_content_is_refused" => {
            let intent = central_only_intent_run();
            let run = run_teach_map_default(&intent);
            cell_from_run(scenario, &run)
        }
        "intent_map_refusal_propagates" => {
            let docs = vec![("plain.md".to_string(), "The bridge is open.".to_string())];
            let intent = run_literature_intent_map_default(&docs, "reactor turbine");
            let run = run_teach_map_default(&intent);
            cell_from_run(scenario, &run)
        }
        "no_model_signal_detected" => {
            let mut config = TeachMapConfig::default_config();
            config.uses_model = true;
            let run = run_teach_map(&literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "no_training_signal_detected" => {
            let mut config = TeachMapConfig::default_config();
            config.uses_training = true;
            let run = run_teach_map(&literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "personalization_signal_refused" => {
            let mut config = TeachMapConfig::default_config();
            config.personalizes = true;
            let run = run_teach_map(&literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "memory_signal_refused" => {
            let mut config = TeachMapConfig::default_config();
            config.writes_memory = true;
            let run = run_teach_map(&literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "serialized_teach_map_tamper_refused" => {
            let json = teach_map_demo_json();
            let refused = verify_teach_map_demo_json(&flip_last_byte(&json)).is_err();
            TeachMapCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused".to_string()
                } else {
                    "tamper_not_refused".to_string()
                },
                refusal: refused
                    .then_some(TeachMapRefusal::SerializedTeachMapTamper)
                    .map(|r| r.slug().to_string()),
                lesson_built: false,
                lesson_items: 0,
                field_refusals: 0,
                boundary_all_inert: false,
            }
        }
        other => TeachMapCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            lesson_built: false,
            lesson_items: 0,
            field_refusals: 0,
            boundary_all_inert: false,
        },
    }
}

fn cell_from_run(scenario: &str, run: &TeachMapRun) -> TeachMapCell {
    TeachMapCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        lesson_built: run.decision == TeachMapDecision::LessonBuilt,
        lesson_items: run.receipt.lesson_item_count,
        field_refusals: run.receipt.field_refusals,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn central_only_intent_run() -> LiteratureIntentRun {
    let docs = vec![(
        "plain.md".to_string(),
        "The central thesis is that grounded lessons need receipts.".to_string(),
    )];
    run_literature_intent_map_default(&docs, "central thesis grounded lessons receipts")
}

pub fn teach_map_matrix() -> TeachMapMatrix {
    let cells = TEACH_MAP_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let built_count = cells.iter().filter(|cell| cell.lesson_built).count();
    let refused_count = cells.iter().filter(|cell| !cell.lesson_built).count();
    TeachMapMatrix {
        schema: SCHEMA.to_string(),
        scenario_count: TEACH_MAP_SCENARIO_COUNT,
        cells,
        built_count,
        refused_count,
        boundary: TeachMapBoundary::inert(),
        boundary_all_inert: TeachMapBoundary::inert().all_inert(),
    }
}

pub fn teach_map_matrix_json() -> String {
    serde_json::to_string(&teach_map_matrix()).expect("teach map matrix serializes")
}

pub fn verify_teach_map_matrix_json(candidate: &str) -> Result<(), TeachMapError> {
    if candidate == teach_map_matrix_json() {
        Ok(())
    } else {
        Err(TeachMapError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_lesson() -> TeachLesson {
        teach_map_demo().lesson.expect("lesson")
    }

    fn all_support_is_teach_backed(support: &[TeachSupportRef]) -> bool {
        !support.is_empty()
            && support.iter().all(|span| {
                span.intent_authority == REQUIRED_INTENT_AUTHORITY
                    && span.teach_authority == AUTHORITY_TEACH_FROM_INTENT_MAP
            })
    }

    fn has_refusal(lesson: &TeachLesson, field: &str) -> bool {
        lesson.refusals.iter().any(|r| r.field == field)
    }

    #[test]
    fn verified_intent_map_builds_teach_map() {
        let run = teach_map_demo();
        assert_eq!(run.decision, TeachMapDecision::LessonBuilt);
        assert!(run.refusal.is_none());
        let lesson = run.lesson.expect("lesson");
        assert_eq!(lesson.document, "companion.md");
        assert!(lesson.explanation.is_some());
        assert!(!lesson.examples.is_empty());
        assert!(!lesson.misconception_checks.is_empty());
        assert!(!lesson.quiz.is_empty());
        assert!(lesson.next_reading_step.is_some());
    }

    #[test]
    fn explanation_is_span_backed() {
        let lesson = demo_lesson();
        let explanation = lesson.explanation.expect("explanation");
        assert!(explanation.text.contains("verified central thesis"));
        assert!(all_support_is_teach_backed(&explanation.support));
    }

    #[test]
    fn examples_are_span_backed() {
        let lesson = demo_lesson();
        assert!(lesson
            .examples
            .iter()
            .any(|example| example.text.contains("Symbiosis")));
        for example in &lesson.examples {
            assert!(all_support_is_teach_backed(&example.support));
        }
    }

    #[test]
    fn misconception_checks_are_span_backed() {
        let lesson = demo_lesson();
        assert!(lesson
            .misconception_checks
            .iter()
            .any(|check| check.misconception.contains("hidden motive")));
        for check in &lesson.misconception_checks {
            assert!(all_support_is_teach_backed(&check.support));
            assert!(!check.correction.contains("hidden author motive inferred"));
        }
    }

    #[test]
    fn quiz_is_span_backed() {
        let lesson = demo_lesson();
        assert!(lesson.quiz.len() >= 2);
        for item in &lesson.quiz {
            assert!(item.question.ends_with('?'));
            assert!(all_support_is_teach_backed(&item.support));
        }
    }

    #[test]
    fn next_reading_step_is_span_backed() {
        let lesson = demo_lesson();
        let next = lesson.next_reading_step.expect("next reading step");
        assert!(next.text.contains("Next,"));
        assert!(all_support_is_teach_backed(&next.support));
    }

    #[test]
    fn unsupported_teaching_content_is_refused() {
        let intent = central_only_intent_run();
        let run = run_teach_map_default(&intent);
        assert_eq!(run.decision, TeachMapDecision::LessonBuilt);
        let lesson = run.lesson.expect("lesson");
        assert!(has_refusal(&lesson, "misconception_checks"));
        assert!(lesson.misconception_checks.is_empty());
        assert!(lesson_items_are_supported(&lesson));
    }

    #[test]
    fn intent_map_refusal_propagates() {
        let docs = vec![("plain.md".to_string(), "The bridge is open.".to_string())];
        let intent = run_literature_intent_map_default(&docs, "reactor turbine");
        assert_eq!(intent.decision, LiteratureIntentDecision::IntentMapRefused);
        let run = run_teach_map_default(&intent);
        assert_eq!(run.decision, TeachMapDecision::LessonRefused);
        assert_eq!(run.refusal, Some(TeachMapRefusal::IntentMapRefused));
        assert!(run.lesson.is_none());
    }

    #[test]
    fn model_training_personalization_and_memory_signals_are_refused() {
        let intent = literature_intent_demo();

        let mut config = TeachMapConfig::default_config();
        config.uses_model = true;
        assert_eq!(
            run_teach_map(&intent, config).refusal,
            Some(TeachMapRefusal::ModelSignalDetected)
        );

        let mut config = TeachMapConfig::default_config();
        config.uses_training = true;
        assert_eq!(
            run_teach_map(&intent, config).refusal,
            Some(TeachMapRefusal::TrainingSignalDetected)
        );

        let mut config = TeachMapConfig::default_config();
        config.personalizes = true;
        assert_eq!(
            run_teach_map(&intent, config).refusal,
            Some(TeachMapRefusal::PersonalizationSignalDetected)
        );

        let mut config = TeachMapConfig::default_config();
        config.writes_memory = true;
        assert_eq!(
            run_teach_map(&intent, config).refusal,
            Some(TeachMapRefusal::MemorySignalDetected)
        );
    }

    #[test]
    fn boundary_is_inert_and_recorded() {
        let run = teach_map_demo();
        assert!(run.receipt.boundary_all_inert);
        assert_eq!(TEACH_MAP_BOUNDARY_LINES.len(), 8);
        assert_eq!(
            TEACH_MAP_BOUNDARY_LINES[0],
            "TEACH-0 teaches from span-backed intent-map findings only."
        );
        let lesson = run.lesson.expect("lesson");
        assert!(has_refusal(&lesson, "personalization"));
        assert!(has_refusal(&lesson, "learner_memory"));
        assert!(has_refusal(&lesson, "hidden_author_motives"));
        assert!(has_refusal(&lesson, "full_comprehension"));
    }

    #[test]
    fn demo_json_re_derives_and_refuses_tampering() {
        let json = teach_map_demo_json();
        assert!(json.contains("\"misconception_checks\""));
        assert!(verify_teach_map_demo_json(&json).is_ok());
        assert_eq!(
            verify_teach_map_demo_json(&format!("{json} ")),
            Err(TeachMapError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_has_named_scenarios_and_replays() {
        let matrix = teach_map_matrix();
        assert_eq!(matrix.scenario_count, TEACH_MAP_SCENARIO_COUNT);
        assert_eq!(matrix.cells.len(), TEACH_MAP_SCENARIO_COUNT);
        assert_eq!(
            matrix.built_count + matrix.refused_count,
            TEACH_MAP_SCENARIO_COUNT
        );
        for (cell, name) in matrix.cells.iter().zip(TEACH_MAP_SCENARIO_NAMES.iter()) {
            assert_eq!(&cell.scenario, name);
            assert_ne!(cell.outcome, "unknown");
        }
        let json = teach_map_matrix_json();
        assert!(verify_teach_map_matrix_json(&json).is_ok());
        assert_eq!(
            verify_teach_map_matrix_json(&format!("{json} ")),
            Err(TeachMapError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = teach_map_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "serialized_teach_map_tamper_refused")
            .expect("tamper scenario");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_teach_map_tamper_refused")
        );

        let json = teach_map_demo_json();
        assert!(verify_teach_map_demo_json(&flip_last_byte(&json)).is_err());
    }

    #[test]
    fn decisions_and_refusals_are_complete_and_slugged() {
        assert_eq!(TeachMapDecision::ALL.len(), 2);
        assert_eq!(TeachMapRefusal::ALL.len(), 9);
        let mut slugs = TeachMapRefusal::ALL
            .iter()
            .map(|r| r.slug())
            .collect::<Vec<_>>();
        slugs.sort_unstable();
        let n = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), n);
    }
}
