//! LEARNER-MEMORY-0 — replay-verifiable learner-memory receipt CANDIDATES from
//! learner state.
//!
//! This is NOT durable memory yet. It consumes a built [`LearnerModelRun`] plus
//! the [`LiteratureIntentRun`] that anchored it, cross-checks the receipt chain,
//! and emits a memory CANDIDATE object: bounded memory items (concept taught,
//! answered quiz outcomes, flagged misconceptions, the next-review pointer),
//! each pointing back to explicit learner-state fields AND the source
//! LEARNER/TEACH/LIT/QFLOW receipt hashes — or a typed refusal. It does NOT
//! persist anything to disk, does NOT mutate long-term memory, does NOT
//! personalize, does NOT autonomously recall or adapt, does NOT infer health,
//! psych, identity, or hidden-diagnosis profiles, and does NOT call a model.

use serde::Serialize;

use crate::{
    learner_model_demo, literature_intent_demo, run_literature_intent_map_default,
    LearnerModelDecision, LearnerModelRefusal, LearnerModelRun, LearnerStateMap, LearnerSupportRef,
    LiteratureIntentRun, QuizOutcome,
};

const SCHEMA: &str = "learner-memory-map-v0.1";
const LEARNER_MEMORY_USES_MODEL: bool = false;
const LEARNER_MEMORY_USES_TRAINING: bool = false;
const LEARNER_MEMORY_PERSONALIZES: bool = false;
const LEARNER_MEMORY_PERSISTS_TO_DISK: bool = false;
const LEARNER_MEMORY_AUTONOMOUSLY_RECALLS: bool = false;
const LEARNER_MEMORY_INFERS_HEALTH_PROFILE: bool = false;
const LEARNER_MEMORY_INFERS_IDENTITY_PROFILE: bool = false;
const LEARNER_MEMORY_INFERS_HIDDEN_DIAGNOSIS: bool = false;
const REQUIRED_INTENT_AUTHORITY: &str = "intent_map_from_verified_span";
const REQUIRED_TEACH_AUTHORITY: &str = "teach_from_span_backed_intent_map";
const REQUIRED_LEARNER_AUTHORITY: &str = "learner_state_from_supported_teach_map";
const AUTHORITY_MEMORY_CANDIDATE: &str = "memory_candidate_from_learner_state";

/// The authority boundary, verbatim. LEARNER-MEMORY-0 maps receipt candidates only.
pub const LEARNER_MEMORY_BOUNDARY_LINES: [&str; 10] = [
    "LEARNER-MEMORY-0 creates a replay-verifiable learner-memory receipt object from LEARNER-MODEL-0 state.",
    "Every memory item points back to learner-state receipt fields and source hashes.",
    "It does not persist to disk as memory.",
    "It does not mutate long-term memory.",
    "It does not personalize generation.",
    "It does not autonomously adapt or recall.",
    "It does not infer health, psych, identity, or hidden diagnosis.",
    "It does not train or run a model.",
    "It does not create truth.",
    "It does not retag v0.1.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearnerMemoryDecision {
    MemoryCandidateMapped,
    MemoryCandidateRefused,
}

impl LearnerMemoryDecision {
    pub const ALL: [LearnerMemoryDecision; 2] = [
        LearnerMemoryDecision::MemoryCandidateMapped,
        LearnerMemoryDecision::MemoryCandidateRefused,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            LearnerMemoryDecision::MemoryCandidateMapped => "memory_candidate_mapped",
            LearnerMemoryDecision::MemoryCandidateRefused => "memory_candidate_refused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearnerMemoryRefusal {
    LearnerModelRefused,
    LearnerStateUnavailable,
    SourceChainMismatch,
    UnbackedMemoryItem,
    ModelSignalDetected,
    TrainingSignalDetected,
    PersonalizationSignalDetected,
    MemoryPersistenceSignalDetected,
    AutonomousRecallSignalDetected,
    HealthProfileSignalDetected,
    IdentityProfileSignalDetected,
    HiddenDiagnosisSignalDetected,
    SerializedLearnerMemoryTamper,
}

impl LearnerMemoryRefusal {
    pub const ALL: [LearnerMemoryRefusal; 13] = [
        LearnerMemoryRefusal::LearnerModelRefused,
        LearnerMemoryRefusal::LearnerStateUnavailable,
        LearnerMemoryRefusal::SourceChainMismatch,
        LearnerMemoryRefusal::UnbackedMemoryItem,
        LearnerMemoryRefusal::ModelSignalDetected,
        LearnerMemoryRefusal::TrainingSignalDetected,
        LearnerMemoryRefusal::PersonalizationSignalDetected,
        LearnerMemoryRefusal::MemoryPersistenceSignalDetected,
        LearnerMemoryRefusal::AutonomousRecallSignalDetected,
        LearnerMemoryRefusal::HealthProfileSignalDetected,
        LearnerMemoryRefusal::IdentityProfileSignalDetected,
        LearnerMemoryRefusal::HiddenDiagnosisSignalDetected,
        LearnerMemoryRefusal::SerializedLearnerMemoryTamper,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            LearnerMemoryRefusal::LearnerModelRefused => "learner_model_refused",
            LearnerMemoryRefusal::LearnerStateUnavailable => "learner_state_unavailable_refused",
            LearnerMemoryRefusal::SourceChainMismatch => "source_chain_mismatch_refused",
            LearnerMemoryRefusal::UnbackedMemoryItem => "unbacked_memory_item_refused",
            LearnerMemoryRefusal::ModelSignalDetected => "model_signal_detected_refused",
            LearnerMemoryRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            LearnerMemoryRefusal::PersonalizationSignalDetected => {
                "personalization_signal_detected_refused"
            }
            LearnerMemoryRefusal::MemoryPersistenceSignalDetected => {
                "memory_persistence_signal_detected_refused"
            }
            LearnerMemoryRefusal::AutonomousRecallSignalDetected => {
                "autonomous_recall_signal_detected_refused"
            }
            LearnerMemoryRefusal::HealthProfileSignalDetected => {
                "health_profile_signal_detected_refused"
            }
            LearnerMemoryRefusal::IdentityProfileSignalDetected => {
                "identity_profile_signal_detected_refused"
            }
            LearnerMemoryRefusal::HiddenDiagnosisSignalDetected => {
                "hidden_diagnosis_signal_detected_refused"
            }
            LearnerMemoryRefusal::SerializedLearnerMemoryTamper => {
                "serialized_learner_memory_tamper_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearnerMemoryError {
    ReplayMismatch,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearnerMemoryConfig {
    pub uses_model: bool,
    pub uses_training: bool,
    pub personalizes: bool,
    pub persists_to_disk: bool,
    pub autonomously_recalls: bool,
    pub infers_health_profile: bool,
    pub infers_identity_profile: bool,
    pub infers_hidden_diagnosis: bool,
}

impl LearnerMemoryConfig {
    pub fn default_config() -> Self {
        LearnerMemoryConfig {
            uses_model: LEARNER_MEMORY_USES_MODEL,
            uses_training: LEARNER_MEMORY_USES_TRAINING,
            personalizes: LEARNER_MEMORY_PERSONALIZES,
            persists_to_disk: LEARNER_MEMORY_PERSISTS_TO_DISK,
            autonomously_recalls: LEARNER_MEMORY_AUTONOMOUSLY_RECALLS,
            infers_health_profile: LEARNER_MEMORY_INFERS_HEALTH_PROFILE,
            infers_identity_profile: LEARNER_MEMORY_INFERS_IDENTITY_PROFILE,
            infers_hidden_diagnosis: LEARNER_MEMORY_INFERS_HIDDEN_DIAGNOSIS,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearnerMemoryBoundary {
    pub creates_truth: bool,
    pub persists_memory_to_disk: bool,
    pub mutates_long_term_memory: bool,
    pub personalizes: bool,
    pub autonomously_recalls: bool,
    pub trains: bool,
    pub is_model: bool,
    pub infers_health_profile: bool,
    pub infers_identity_profile: bool,
    pub infers_hidden_diagnosis: bool,
    pub retags_release: bool,
}

impl LearnerMemoryBoundary {
    fn inert() -> Self {
        LearnerMemoryBoundary {
            creates_truth: LEARNER_MEMORY_USES_MODEL,
            persists_memory_to_disk: LEARNER_MEMORY_PERSISTS_TO_DISK,
            mutates_long_term_memory: LEARNER_MEMORY_PERSISTS_TO_DISK,
            personalizes: LEARNER_MEMORY_PERSONALIZES,
            autonomously_recalls: LEARNER_MEMORY_AUTONOMOUSLY_RECALLS,
            trains: LEARNER_MEMORY_USES_TRAINING,
            is_model: LEARNER_MEMORY_USES_MODEL,
            infers_health_profile: LEARNER_MEMORY_INFERS_HEALTH_PROFILE,
            infers_identity_profile: LEARNER_MEMORY_INFERS_IDENTITY_PROFILE,
            infers_hidden_diagnosis: LEARNER_MEMORY_INFERS_HIDDEN_DIAGNOSIS,
            retags_release: LEARNER_MEMORY_USES_MODEL,
        }
    }

    fn all_inert(&self) -> bool {
        !self.creates_truth
            && !self.persists_memory_to_disk
            && !self.mutates_long_term_memory
            && !self.personalizes
            && !self.autonomously_recalls
            && !self.trains
            && !self.is_model
            && !self.infers_health_profile
            && !self.infers_identity_profile
            && !self.infers_hidden_diagnosis
            && !self.retags_release
    }
}

/// The authority spine of one memory item: the learner support span extended
/// with the memory-candidate authority. Memory never upgrades authority.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MemorySupportRef {
    pub document_id: u64,
    pub document_name: String,
    pub span_id: u64,
    pub text: String,
    pub intent_authority: String,
    pub teach_authority: String,
    pub learner_authority: String,
    pub memory_authority: String,
}

/// One bounded memory candidate. Every item cites the explicit learner-state
/// field it came from and carries all four source receipt hashes.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerMemoryItem {
    pub item_id: String,
    pub kind: String,
    pub source_field: String,
    pub content: String,
    pub outcome: Option<String>,
    pub source_learner_receipt_hash: u64,
    pub source_teach_receipt_hash: u64,
    pub source_intent_receipt_hash: u64,
    pub source_qflow_receipt_hash: u64,
    pub support: Vec<MemorySupportRef>,
}

/// The session confidence marker carried at map level (a session marker, not a
/// memory item): self-reported only, never inferred.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionConfidence {
    pub marker: String,
    pub source: String,
    pub inferred: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerMemoryFieldRefusal {
    pub field: String,
    pub reason: String,
}

/// The memory CANDIDATE map: bounded items plus receipt links. Not a store.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerMemoryMap {
    pub source_learner_receipt_hash: u64,
    pub source_teach_receipt_hash: u64,
    pub source_intent_receipt_hash: u64,
    pub source_qflow_receipt_hash: u64,
    pub document: String,
    pub items: Vec<LearnerMemoryItem>,
    pub session_confidence: SessionConfidence,
    pub candidate_status: String,
    pub receipt_link: String,
    pub refusals: Vec<LearnerMemoryFieldRefusal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerMemoryReceipt {
    pub schema: String,
    pub source_learner_receipt_hash: u64,
    pub source_teach_receipt_hash: u64,
    pub source_intent_receipt_hash: u64,
    pub source_qflow_receipt_hash: u64,
    pub source_learner_decision: String,
    pub source_learner_refusal: Option<String>,
    pub config: LearnerMemoryConfig,
    pub item_count: usize,
    pub concept_items: usize,
    pub quiz_items: usize,
    pub misconception_items: usize,
    pub review_items: usize,
    pub decision: LearnerMemoryDecision,
    pub refusal: Option<LearnerMemoryRefusal>,
    pub receipt_hash: u64,
    pub boundary: LearnerMemoryBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerMemoryRun {
    pub receipt: LearnerMemoryReceipt,
    pub memory: Option<LearnerMemoryMap>,
    pub decision: LearnerMemoryDecision,
    pub refusal: Option<LearnerMemoryRefusal>,
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

struct SourceHashes {
    learner: u64,
    teach: u64,
    intent: u64,
    qflow: u64,
}

fn mix_support(mut h: u64, support: &[MemorySupportRef]) -> u64 {
    h = fnv_u64(h, support.len() as u64);
    for span in support {
        h = fnv_u64(h, span.document_id);
        h = fnv_u64(h, span.span_id);
        h = fnv_mix(h, span.document_name.as_bytes());
        h = fnv_mix(h, span.text.as_bytes());
        h = fnv_mix(h, span.intent_authority.as_bytes());
        h = fnv_mix(h, span.teach_authority.as_bytes());
        h = fnv_mix(h, span.learner_authority.as_bytes());
        h = fnv_mix(h, span.memory_authority.as_bytes());
    }
    h
}

fn receipt_hash(
    hashes: &SourceHashes,
    learner_run: &LearnerModelRun,
    config: &LearnerMemoryConfig,
    memory: Option<&LearnerMemoryMap>,
    decision: LearnerMemoryDecision,
    refusal: Option<LearnerMemoryRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    h = fnv_mix(h, SCHEMA.as_bytes());
    h = fnv_u64(h, hashes.learner);
    h = fnv_u64(h, hashes.teach);
    h = fnv_u64(h, hashes.intent);
    h = fnv_u64(h, hashes.qflow);
    h = fnv_mix(h, learner_run.decision.slug().as_bytes());
    h = fnv_mix(
        h,
        learner_run
            .refusal
            .map(LearnerModelRefusal::slug)
            .unwrap_or("none")
            .as_bytes(),
    );
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, config.personalizes as u64);
    h = fnv_u64(h, config.persists_to_disk as u64);
    h = fnv_u64(h, config.autonomously_recalls as u64);
    h = fnv_u64(h, config.infers_health_profile as u64);
    h = fnv_u64(h, config.infers_identity_profile as u64);
    h = fnv_u64(h, config.infers_hidden_diagnosis as u64);
    if let Some(memory) = memory {
        h = fnv_mix(h, memory.document.as_bytes());
        for item in &memory.items {
            h = fnv_mix(h, item.item_id.as_bytes());
            h = fnv_mix(h, item.kind.as_bytes());
            h = fnv_mix(h, item.source_field.as_bytes());
            h = fnv_mix(h, item.content.as_bytes());
            h = fnv_mix(h, item.outcome.as_deref().unwrap_or("none").as_bytes());
            h = fnv_u64(h, item.source_learner_receipt_hash);
            h = fnv_u64(h, item.source_teach_receipt_hash);
            h = fnv_u64(h, item.source_intent_receipt_hash);
            h = fnv_u64(h, item.source_qflow_receipt_hash);
            h = mix_support(h, &item.support);
        }
        h = fnv_mix(h, memory.session_confidence.marker.as_bytes());
        h = fnv_mix(h, memory.session_confidence.source.as_bytes());
        h = fnv_u64(h, memory.session_confidence.inferred as u64);
        h = fnv_mix(h, memory.candidate_status.as_bytes());
        h = fnv_mix(h, memory.receipt_link.as_bytes());
        for refusal in &memory.refusals {
            h = fnv_mix(h, refusal.field.as_bytes());
            h = fnv_mix(h, refusal.reason.as_bytes());
        }
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

fn memory_support(support: &[LearnerSupportRef]) -> Vec<MemorySupportRef> {
    support
        .iter()
        .map(|span| MemorySupportRef {
            document_id: span.document_id,
            document_name: span.document_name.clone(),
            span_id: span.span_id,
            text: span.text.clone(),
            intent_authority: span.intent_authority.clone(),
            teach_authority: span.teach_authority.clone(),
            learner_authority: span.learner_authority.clone(),
            memory_authority: AUTHORITY_MEMORY_CANDIDATE.to_string(),
        })
        .collect()
}

fn memory_items(state: &LearnerStateMap, hashes: &SourceHashes) -> Vec<LearnerMemoryItem> {
    let mut items = Vec::new();
    if let Some(concept) = state.concept_taught.as_ref() {
        items.push(LearnerMemoryItem {
            item_id: "memory:concept:1".to_string(),
            kind: "concept_taught".to_string(),
            source_field: format!("concept_taught[{}]", concept.source_item_id),
            content: concept.label.clone(),
            outcome: None,
            source_learner_receipt_hash: hashes.learner,
            source_teach_receipt_hash: hashes.teach,
            source_intent_receipt_hash: hashes.intent,
            source_qflow_receipt_hash: hashes.qflow,
            support: memory_support(&concept.support),
        });
    }
    let answered = state
        .quiz_result
        .items
        .iter()
        .filter(|item| item.outcome != QuizOutcome::Unanswered);
    for (idx, quiz) in answered.enumerate() {
        items.push(LearnerMemoryItem {
            item_id: format!("memory:quiz:{}", idx + 1),
            kind: "quiz_outcome".to_string(),
            source_field: format!("quiz_result.items[{}]", quiz.quiz_id),
            content: quiz.question.clone(),
            outcome: Some(quiz.outcome.slug().to_string()),
            source_learner_receipt_hash: hashes.learner,
            source_teach_receipt_hash: hashes.teach,
            source_intent_receipt_hash: hashes.intent,
            source_qflow_receipt_hash: hashes.qflow,
            support: memory_support(&quiz.support),
        });
    }
    let flagged = state.misconception_flags.iter().filter(|flag| flag.flagged);
    for (idx, flag) in flagged.enumerate() {
        items.push(LearnerMemoryItem {
            item_id: format!("memory:misconception:{}", idx + 1),
            kind: "misconception_flagged".to_string(),
            source_field: format!("misconception_flags[{}]", flag.check_id),
            content: flag.misconception.clone(),
            outcome: None,
            source_learner_receipt_hash: hashes.learner,
            source_teach_receipt_hash: hashes.teach,
            source_intent_receipt_hash: hashes.intent,
            source_qflow_receipt_hash: hashes.qflow,
            support: memory_support(&flag.support),
        });
    }
    if let Some(target) = state.next_review_target.as_ref() {
        items.push(LearnerMemoryItem {
            item_id: "memory:review:1".to_string(),
            kind: "next_review_target".to_string(),
            source_field: "next_review_target".to_string(),
            content: target.text.clone(),
            outcome: None,
            source_learner_receipt_hash: hashes.learner,
            source_teach_receipt_hash: hashes.teach,
            source_intent_receipt_hash: hashes.intent,
            source_qflow_receipt_hash: hashes.qflow,
            support: memory_support(&target.support),
        });
    }
    items
}

fn support_is_memory_backed(support: &[MemorySupportRef]) -> bool {
    !support.is_empty()
        && support.iter().all(|span| {
            span.intent_authority == REQUIRED_INTENT_AUTHORITY
                && span.teach_authority == REQUIRED_TEACH_AUTHORITY
                && span.learner_authority == REQUIRED_LEARNER_AUTHORITY
                && span.memory_authority == AUTHORITY_MEMORY_CANDIDATE
                && span.text.chars().any(|c| c.is_alphanumeric())
        })
}

fn item_resolves_to_state(item: &LearnerMemoryItem, state: &LearnerStateMap) -> bool {
    match item.kind.as_str() {
        "concept_taught" => state.concept_taught.as_ref().is_some_and(|concept| {
            item.content == concept.label && item.source_field.contains(&concept.source_item_id)
        }),
        "quiz_outcome" => state.quiz_result.items.iter().any(|quiz| {
            quiz.outcome != QuizOutcome::Unanswered
                && item.source_field.contains(&quiz.quiz_id)
                && item.content == quiz.question
                && item.outcome.as_deref() == Some(quiz.outcome.slug())
        }),
        "misconception_flagged" => state.misconception_flags.iter().any(|flag| {
            flag.flagged
                && item.source_field.contains(&flag.check_id)
                && item.content == flag.misconception
        }),
        "next_review_target" => state
            .next_review_target
            .as_ref()
            .is_some_and(|target| item.content == target.text),
        _ => false,
    }
}

/// The pointer law, enforced: every memory item must cite an explicit
/// learner-state field it resolves to, carry the exact four source receipt
/// hashes, and keep the full four-step authority chain on non-empty verbatim
/// support — or the whole candidate is refused.
pub fn memory_items_are_receipt_backed(
    memory: &LearnerMemoryMap,
    learner_run: &LearnerModelRun,
    qflow_receipt_hash: u64,
) -> Option<LearnerMemoryRefusal> {
    let state = learner_run.learner_state.as_ref()?;
    let receipt = &learner_run.receipt;
    for item in &memory.items {
        let hashes_match = item.source_learner_receipt_hash == receipt.receipt_hash
            && item.source_teach_receipt_hash == receipt.source_teach_receipt_hash
            && item.source_intent_receipt_hash == receipt.source_intent_receipt_hash
            && item.source_qflow_receipt_hash == qflow_receipt_hash;
        if !hashes_match
            || !support_is_memory_backed(&item.support)
            || !item_resolves_to_state(item, state)
        {
            return Some(LearnerMemoryRefusal::UnbackedMemoryItem);
        }
    }
    None
}

pub fn learner_memory_demo() -> LearnerMemoryRun {
    let intent = literature_intent_demo();
    let learner = learner_model_demo();
    run_learner_memory_default(&learner, &intent)
}

pub fn learner_memory_demo_json() -> String {
    serde_json::to_string_pretty(&learner_memory_demo()).expect("learner memory demo serializes")
}

pub fn verify_learner_memory_demo_json(candidate: &str) -> Result<(), LearnerMemoryError> {
    if candidate == learner_memory_demo_json() {
        Ok(())
    } else {
        Err(LearnerMemoryError::ReplayMismatch)
    }
}

pub fn run_learner_memory_default(
    learner_run: &LearnerModelRun,
    intent_run: &LiteratureIntentRun,
) -> LearnerMemoryRun {
    run_learner_memory(
        learner_run,
        intent_run,
        LearnerMemoryConfig::default_config(),
    )
}

pub fn run_learner_memory(
    learner_run: &LearnerModelRun,
    intent_run: &LiteratureIntentRun,
    config: LearnerMemoryConfig,
) -> LearnerMemoryRun {
    let hashes = SourceHashes {
        learner: learner_run.receipt.receipt_hash,
        teach: learner_run.receipt.source_teach_receipt_hash,
        intent: learner_run.receipt.source_intent_receipt_hash,
        qflow: intent_run.receipt.qflow_receipt_hash,
    };
    let signal_refusal = if config.uses_model {
        Some(LearnerMemoryRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(LearnerMemoryRefusal::TrainingSignalDetected)
    } else if config.personalizes {
        Some(LearnerMemoryRefusal::PersonalizationSignalDetected)
    } else if config.persists_to_disk {
        Some(LearnerMemoryRefusal::MemoryPersistenceSignalDetected)
    } else if config.autonomously_recalls {
        Some(LearnerMemoryRefusal::AutonomousRecallSignalDetected)
    } else if config.infers_health_profile {
        Some(LearnerMemoryRefusal::HealthProfileSignalDetected)
    } else if config.infers_identity_profile {
        Some(LearnerMemoryRefusal::IdentityProfileSignalDetected)
    } else if config.infers_hidden_diagnosis {
        Some(LearnerMemoryRefusal::HiddenDiagnosisSignalDetected)
    } else {
        None
    };
    if let Some(refusal) = signal_refusal {
        return assemble(
            &hashes,
            learner_run,
            config,
            LearnerMemoryDecision::MemoryCandidateRefused,
            Some(refusal),
            None,
        );
    }
    if learner_run.decision == LearnerModelDecision::LearnerStateRefused {
        return assemble(
            &hashes,
            learner_run,
            config,
            LearnerMemoryDecision::MemoryCandidateRefused,
            Some(LearnerMemoryRefusal::LearnerModelRefused),
            None,
        );
    }
    let state = match learner_run.learner_state.as_ref() {
        Some(state) => state,
        None => {
            return assemble(
                &hashes,
                learner_run,
                config,
                LearnerMemoryDecision::MemoryCandidateRefused,
                Some(LearnerMemoryRefusal::LearnerStateUnavailable),
                None,
            );
        }
    };
    if intent_run.receipt.receipt_hash != learner_run.receipt.source_intent_receipt_hash {
        return assemble(
            &hashes,
            learner_run,
            config,
            LearnerMemoryDecision::MemoryCandidateRefused,
            Some(LearnerMemoryRefusal::SourceChainMismatch),
            None,
        );
    }
    let memory = build_memory(state, &hashes);
    if let Some(refusal) = memory_items_are_receipt_backed(&memory, learner_run, hashes.qflow) {
        return assemble(
            &hashes,
            learner_run,
            config,
            LearnerMemoryDecision::MemoryCandidateRefused,
            Some(refusal),
            None,
        );
    }
    assemble(
        &hashes,
        learner_run,
        config,
        LearnerMemoryDecision::MemoryCandidateMapped,
        None,
        Some(memory),
    )
}

fn build_memory(state: &LearnerStateMap, hashes: &SourceHashes) -> LearnerMemoryMap {
    let items = memory_items(state, hashes);
    let session_confidence = SessionConfidence {
        marker: state.confidence_marker.marker.slug().to_string(),
        source: state.confidence_marker.source.clone(),
        inferred: false,
    };
    let refusals = vec![
        field_refusal("disk_persistence", "not performed by LEARNER-MEMORY-0"),
        field_refusal(
            "long_term_memory_mutation",
            "not performed by LEARNER-MEMORY-0",
        ),
        field_refusal("personalization", "not generated by LEARNER-MEMORY-0"),
        field_refusal("autonomous_recall", "not performed by LEARNER-MEMORY-0"),
        field_refusal(
            "health_or_psych_profile",
            "not inferred by LEARNER-MEMORY-0",
        ),
        field_refusal("identity_profile", "not inferred by LEARNER-MEMORY-0"),
        field_refusal("hidden_diagnosis", "not inferred by LEARNER-MEMORY-0"),
    ];
    LearnerMemoryMap {
        source_learner_receipt_hash: hashes.learner,
        source_teach_receipt_hash: hashes.teach,
        source_intent_receipt_hash: hashes.intent,
        source_qflow_receipt_hash: hashes.qflow,
        document: state.document.clone(),
        items,
        session_confidence,
        candidate_status: "candidate_only".to_string(),
        receipt_link: format!("learner_receipt:{}", hashes.learner),
        refusals,
    }
}

fn assemble(
    hashes: &SourceHashes,
    learner_run: &LearnerModelRun,
    config: LearnerMemoryConfig,
    decision: LearnerMemoryDecision,
    refusal: Option<LearnerMemoryRefusal>,
    memory: Option<LearnerMemoryMap>,
) -> LearnerMemoryRun {
    let boundary = LearnerMemoryBoundary::inert();
    let count = |kind: &str| {
        memory
            .as_ref()
            .map(|m| m.items.iter().filter(|i| i.kind == kind).count())
            .unwrap_or(0)
    };
    let item_count = memory.as_ref().map(|m| m.items.len()).unwrap_or(0);
    let concept_items = count("concept_taught");
    let quiz_items = count("quiz_outcome");
    let misconception_items = count("misconception_flagged");
    let review_items = count("next_review_target");
    let receipt_hash = receipt_hash(
        hashes,
        learner_run,
        &config,
        memory.as_ref(),
        decision,
        refusal,
    );
    let receipt = LearnerMemoryReceipt {
        schema: SCHEMA.to_string(),
        source_learner_receipt_hash: hashes.learner,
        source_teach_receipt_hash: hashes.teach,
        source_intent_receipt_hash: hashes.intent,
        source_qflow_receipt_hash: hashes.qflow,
        source_learner_decision: learner_run.decision.slug().to_string(),
        source_learner_refusal: learner_run.refusal.map(|r| r.slug().to_string()),
        config,
        item_count,
        concept_items,
        quiz_items,
        misconception_items,
        review_items,
        decision,
        refusal,
        receipt_hash,
        boundary,
        boundary_all_inert: boundary.all_inert(),
    };
    LearnerMemoryRun {
        receipt,
        memory,
        decision,
        refusal,
    }
}

fn field_refusal(field: &str, reason: &str) -> LearnerMemoryFieldRefusal {
    LearnerMemoryFieldRefusal {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

pub const LEARNER_MEMORY_SCENARIO_COUNT: usize = 15;
pub const LEARNER_MEMORY_SCENARIO_NAMES: [&str; LEARNER_MEMORY_SCENARIO_COUNT] = [
    "mapped_learner_state_builds_memory_candidate",
    "memory_items_point_to_receipts",
    "concept_memory_is_span_backed",
    "quiz_outcome_memories_are_exact_match_only",
    "flagged_misconceptions_become_memory_items",
    "next_review_memory_is_non_adaptive",
    "session_confidence_is_self_reported",
    "learner_model_refusal_propagates",
    "source_chain_mismatch_refused",
    "unbacked_memory_item_refused",
    "no_model_signal_detected",
    "no_training_signal_detected",
    "personalization_signal_refused",
    "persistence_recall_identity_health_diagnosis_signals_refused",
    "serialized_learner_memory_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct LearnerMemoryCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub memory_mapped: bool,
    pub memory_items: usize,
    pub quiz_memories: usize,
    pub misconception_memories: usize,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerMemoryMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<LearnerMemoryCell>,
    pub mapped_count: usize,
    pub refused_count: usize,
    pub boundary: LearnerMemoryBoundary,
    pub boundary_all_inert: bool,
}

fn cell_for(scenario: &str) -> LearnerMemoryCell {
    match scenario {
        "mapped_learner_state_builds_memory_candidate"
        | "memory_items_point_to_receipts"
        | "concept_memory_is_span_backed"
        | "quiz_outcome_memories_are_exact_match_only"
        | "flagged_misconceptions_become_memory_items"
        | "next_review_memory_is_non_adaptive"
        | "session_confidence_is_self_reported" => {
            let run = learner_memory_demo();
            cell_from_run(scenario, &run)
        }
        "learner_model_refusal_propagates" => {
            let intent = literature_intent_demo();
            let mut config = crate::LearnerModelConfig::default_config();
            config.uses_model = true;
            let teach = crate::teach_map_demo();
            let observation = crate::LearnerModelObservation {
                seen_lesson_item_ids: vec![],
                quiz_answers: vec![],
                misconception_flags: vec![],
                confidence_marker: crate::ConfidenceMarker::Unstated,
            };
            let learner = crate::run_learner_model(&teach, observation, config);
            let run = run_learner_memory_default(&learner, &intent);
            cell_from_run(scenario, &run)
        }
        "source_chain_mismatch_refused" => {
            let foreign_intent = run_literature_intent_map_default(
                &[(
                    "other.md".to_string(),
                    "A different document about a different topic entirely.".to_string(),
                )],
                "different topic",
            );
            let learner = learner_model_demo();
            let run = run_learner_memory_default(&learner, &foreign_intent);
            cell_from_run(scenario, &run)
        }
        "unbacked_memory_item_refused" => {
            // Forge an item that cites no learner-state field and carries no
            // support; the pointer-law guard must refuse the whole candidate.
            let intent = literature_intent_demo();
            let learner = learner_model_demo();
            let run = run_learner_memory_default(&learner, &intent);
            let mut memory = run.memory.clone().expect("demo memory candidate");
            memory.items.push(LearnerMemoryItem {
                item_id: "memory:forged:1".to_string(),
                kind: "invented".to_string(),
                source_field: "no_such_field".to_string(),
                content: "smuggled memory".to_string(),
                outcome: None,
                source_learner_receipt_hash: 0,
                source_teach_receipt_hash: 0,
                source_intent_receipt_hash: 0,
                source_qflow_receipt_hash: 0,
                support: vec![],
            });
            let refusal = memory_items_are_receipt_backed(
                &memory,
                &learner,
                intent.receipt.qflow_receipt_hash,
            );
            LearnerMemoryCell {
                scenario: scenario.to_string(),
                outcome: match refusal {
                    Some(_) => "memory_candidate_refused".to_string(),
                    None => "forged_item_missed".to_string(),
                },
                refusal: refusal.map(|r| r.slug().to_string()),
                memory_mapped: false,
                memory_items: 0,
                quiz_memories: 0,
                misconception_memories: 0,
                boundary_all_inert: LearnerMemoryBoundary::inert().all_inert(),
            }
        }
        "no_model_signal_detected" => {
            let mut config = LearnerMemoryConfig::default_config();
            config.uses_model = true;
            let run = run_learner_memory(&learner_model_demo(), &literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "no_training_signal_detected" => {
            let mut config = LearnerMemoryConfig::default_config();
            config.uses_training = true;
            let run = run_learner_memory(&learner_model_demo(), &literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "personalization_signal_refused" => {
            let mut config = LearnerMemoryConfig::default_config();
            config.personalizes = true;
            let run = run_learner_memory(&learner_model_demo(), &literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "persistence_recall_identity_health_diagnosis_signals_refused" => {
            let mut config = LearnerMemoryConfig::default_config();
            config.persists_to_disk = true;
            config.autonomously_recalls = true;
            config.infers_health_profile = true;
            config.infers_identity_profile = true;
            config.infers_hidden_diagnosis = true;
            let run = run_learner_memory(&learner_model_demo(), &literature_intent_demo(), config);
            cell_from_run(scenario, &run)
        }
        "serialized_learner_memory_tamper_refused" => {
            // Serialize the real candidate, flip one byte, and confirm the
            // tamper is detectable — constructing the refusal that names this
            // scenario (the QSELECT/QFLOW/LEARNER-MODEL precedent).
            let json = learner_memory_demo_json();
            let refused = verify_learner_memory_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(LearnerMemoryRefusal::SerializedLearnerMemoryTamper)
            } else {
                None
            };
            LearnerMemoryCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: refusal.map(|r| r.slug().to_string()),
                memory_mapped: false,
                memory_items: 0,
                quiz_memories: 0,
                misconception_memories: 0,
                boundary_all_inert: LearnerMemoryBoundary::inert().all_inert(),
            }
        }
        other => LearnerMemoryCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            memory_mapped: false,
            memory_items: 0,
            quiz_memories: 0,
            misconception_memories: 0,
            boundary_all_inert: false,
        },
    }
}

fn cell_from_run(scenario: &str, run: &LearnerMemoryRun) -> LearnerMemoryCell {
    LearnerMemoryCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        memory_mapped: run.decision == LearnerMemoryDecision::MemoryCandidateMapped,
        memory_items: run.receipt.item_count,
        quiz_memories: run.receipt.quiz_items,
        misconception_memories: run.receipt.misconception_items,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

pub fn learner_memory_matrix() -> LearnerMemoryMatrix {
    let cells = LEARNER_MEMORY_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let mapped_count = cells.iter().filter(|cell| cell.memory_mapped).count();
    let refused_count = cells.iter().filter(|cell| !cell.memory_mapped).count();
    LearnerMemoryMatrix {
        schema: SCHEMA.to_string(),
        scenario_count: LEARNER_MEMORY_SCENARIO_COUNT,
        cells,
        mapped_count,
        refused_count,
        boundary: LearnerMemoryBoundary::inert(),
        boundary_all_inert: LearnerMemoryBoundary::inert().all_inert(),
    }
}

pub fn learner_memory_matrix_json() -> String {
    serde_json::to_string(&learner_memory_matrix()).expect("learner memory matrix serializes")
}

pub fn verify_learner_memory_matrix_json(candidate: &str) -> Result<(), LearnerMemoryError> {
    if candidate == learner_memory_matrix_json() {
        Ok(())
    } else {
        Err(LearnerMemoryError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_memory() -> LearnerMemoryMap {
        learner_memory_demo().memory.expect("memory candidate")
    }

    #[test]
    fn mapped_learner_state_builds_memory_candidate() {
        let run = learner_memory_demo();
        assert_eq!(run.decision, LearnerMemoryDecision::MemoryCandidateMapped);
        assert!(run.refusal.is_none());
        let learner = learner_model_demo();
        let intent = literature_intent_demo();
        assert_eq!(
            run.receipt.source_learner_receipt_hash,
            learner.receipt.receipt_hash
        );
        assert_eq!(
            run.receipt.source_teach_receipt_hash,
            learner.receipt.source_teach_receipt_hash
        );
        assert_eq!(
            run.receipt.source_intent_receipt_hash,
            intent.receipt.receipt_hash
        );
        assert_eq!(
            run.receipt.source_qflow_receipt_hash,
            intent.receipt.qflow_receipt_hash
        );
        let memory = run.memory.expect("memory candidate");
        assert_eq!(memory.document, "companion.md");
        assert_eq!(memory.candidate_status, "candidate_only");
        assert_eq!(run.receipt.concept_items, 1);
        assert_eq!(run.receipt.quiz_items, 2);
        assert_eq!(run.receipt.misconception_items, 1);
        assert_eq!(run.receipt.review_items, 1);
        assert_eq!(run.receipt.item_count, 5);
    }

    #[test]
    fn memory_items_point_back_to_receipts() {
        let memory = demo_memory();
        let learner = learner_model_demo();
        let intent = literature_intent_demo();
        assert!(memory_items_are_receipt_backed(
            &memory,
            &learner,
            intent.receipt.qflow_receipt_hash
        )
        .is_none());
        for item in &memory.items {
            assert_eq!(
                item.source_learner_receipt_hash,
                learner.receipt.receipt_hash
            );
            assert_eq!(
                item.source_qflow_receipt_hash,
                intent.receipt.qflow_receipt_hash
            );
            assert!(!item.source_field.is_empty());
        }
        assert!(memory.receipt_link.starts_with("learner_receipt:"));
    }

    #[test]
    fn unbacked_memory_item_is_refused() {
        let learner = learner_model_demo();
        let intent = literature_intent_demo();
        let mut memory = demo_memory();
        memory.items.push(LearnerMemoryItem {
            item_id: "memory:forged:1".to_string(),
            kind: "invented".to_string(),
            source_field: "no_such_field".to_string(),
            content: "smuggled memory".to_string(),
            outcome: None,
            source_learner_receipt_hash: 0,
            source_teach_receipt_hash: 0,
            source_intent_receipt_hash: 0,
            source_qflow_receipt_hash: 0,
            support: vec![],
        });
        assert_eq!(
            memory_items_are_receipt_backed(&memory, &learner, intent.receipt.qflow_receipt_hash),
            Some(LearnerMemoryRefusal::UnbackedMemoryItem)
        );
    }

    #[test]
    fn concept_memory_is_span_backed() {
        let memory = demo_memory();
        let concept = memory
            .items
            .iter()
            .find(|item| item.kind == "concept_taught")
            .expect("concept memory item");
        assert!(concept.content.contains("central thesis"));
        assert!(concept.source_field.contains("explanation:1"));
        assert!(support_is_memory_backed(&concept.support));
    }

    #[test]
    fn quiz_outcome_memories_are_exact_match_only() {
        let memory = demo_memory();
        let outcomes: Vec<&str> = memory
            .items
            .iter()
            .filter(|item| item.kind == "quiz_outcome")
            .map(|item| item.outcome.as_deref().expect("quiz outcome"))
            .collect();
        assert_eq!(outcomes.len(), 2);
        assert!(outcomes.contains(&"correct_exact_match"));
        assert!(outcomes.contains(&"incorrect_exact_mismatch"));
        assert!(!outcomes.contains(&"unanswered"));
    }

    #[test]
    fn flagged_misconceptions_become_memory_items() {
        let memory = demo_memory();
        let flagged: Vec<&LearnerMemoryItem> = memory
            .items
            .iter()
            .filter(|item| item.kind == "misconception_flagged")
            .collect();
        assert_eq!(flagged.len(), 1);
        assert!(flagged[0].source_field.contains("misconception_check:1"));
        assert!(support_is_memory_backed(&flagged[0].support));
    }

    #[test]
    fn next_review_memory_is_non_adaptive() {
        let memory = demo_memory();
        let review = memory
            .items
            .iter()
            .find(|item| item.kind == "next_review_target")
            .expect("review memory item");
        let learner = learner_model_demo();
        let state = learner.learner_state.expect("state");
        let target = state.next_review_target.expect("target");
        assert_eq!(review.content, target.text);
        assert!(!target.autonomously_adapted);
        assert!(support_is_memory_backed(&review.support));
    }

    #[test]
    fn session_confidence_is_self_reported() {
        let memory = demo_memory();
        assert_eq!(memory.session_confidence.marker, "medium");
        assert_eq!(memory.session_confidence.source, "self_reported_marker");
        assert!(!memory.session_confidence.inferred);
        assert!(!memory
            .items
            .iter()
            .any(|item| item.kind == "confidence_marker"));
    }

    #[test]
    fn learner_model_refusal_propagates() {
        let intent = literature_intent_demo();
        let mut config = crate::LearnerModelConfig::default_config();
        config.uses_model = true;
        let teach = crate::teach_map_demo();
        let observation = crate::LearnerModelObservation {
            seen_lesson_item_ids: vec![],
            quiz_answers: vec![],
            misconception_flags: vec![],
            confidence_marker: crate::ConfidenceMarker::Unstated,
        };
        let learner = crate::run_learner_model(&teach, observation, config);
        let run = run_learner_memory_default(&learner, &intent);
        assert_eq!(run.decision, LearnerMemoryDecision::MemoryCandidateRefused);
        assert_eq!(run.refusal, Some(LearnerMemoryRefusal::LearnerModelRefused));
        assert!(run.memory.is_none());
    }

    #[test]
    fn learner_state_unavailable_is_refused() {
        let intent = literature_intent_demo();
        let mut learner = learner_model_demo();
        learner.learner_state = None;
        let run = run_learner_memory_default(&learner, &intent);
        assert_eq!(
            run.refusal,
            Some(LearnerMemoryRefusal::LearnerStateUnavailable)
        );
    }

    #[test]
    fn source_chain_mismatch_is_refused() {
        let foreign_intent = run_literature_intent_map_default(
            &[(
                "other.md".to_string(),
                "A different document about a different topic entirely.".to_string(),
            )],
            "different topic",
        );
        let learner = learner_model_demo();
        let run = run_learner_memory_default(&learner, &foreign_intent);
        assert_eq!(run.decision, LearnerMemoryDecision::MemoryCandidateRefused);
        assert_eq!(run.refusal, Some(LearnerMemoryRefusal::SourceChainMismatch));
    }

    type SignalCase = (fn(&mut LearnerMemoryConfig), LearnerMemoryRefusal);

    #[test]
    fn all_signal_configs_refuse() {
        let learner = learner_model_demo();
        let intent = literature_intent_demo();
        let cases: [SignalCase; 8] = [
            (
                |c| c.uses_model = true,
                LearnerMemoryRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                LearnerMemoryRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.personalizes = true,
                LearnerMemoryRefusal::PersonalizationSignalDetected,
            ),
            (
                |c| c.persists_to_disk = true,
                LearnerMemoryRefusal::MemoryPersistenceSignalDetected,
            ),
            (
                |c| c.autonomously_recalls = true,
                LearnerMemoryRefusal::AutonomousRecallSignalDetected,
            ),
            (
                |c| c.infers_health_profile = true,
                LearnerMemoryRefusal::HealthProfileSignalDetected,
            ),
            (
                |c| c.infers_identity_profile = true,
                LearnerMemoryRefusal::IdentityProfileSignalDetected,
            ),
            (
                |c| c.infers_hidden_diagnosis = true,
                LearnerMemoryRefusal::HiddenDiagnosisSignalDetected,
            ),
        ];
        for (mutate, expected) in cases {
            let mut config = LearnerMemoryConfig::default_config();
            mutate(&mut config);
            let run = run_learner_memory(&learner, &intent, config);
            assert_eq!(run.refusal, Some(expected));
            assert!(run.memory.is_none());
        }
    }

    #[test]
    fn boundary_is_inert_and_recorded() {
        let run = learner_memory_demo();
        assert!(run.receipt.boundary_all_inert);
        assert_eq!(LEARNER_MEMORY_BOUNDARY_LINES.len(), 10);
        assert_eq!(
            LEARNER_MEMORY_BOUNDARY_LINES[2],
            "It does not persist to disk as memory."
        );
        let memory = run.memory.expect("memory candidate");
        let has = |field: &str| memory.refusals.iter().any(|r| r.field == field);
        assert!(has("disk_persistence"));
        assert!(has("long_term_memory_mutation"));
        assert!(has("personalization"));
        assert!(has("autonomous_recall"));
        assert!(has("health_or_psych_profile"));
        assert!(has("identity_profile"));
        assert!(has("hidden_diagnosis"));
    }

    #[test]
    fn memory_receipt_folds_source_hashes() {
        let learner = learner_model_demo();
        let intent_a = literature_intent_demo();
        let mut intent_b = literature_intent_demo();
        intent_b.receipt.qflow_receipt_hash ^= 0x0000_0000_0000_abcd;
        let mut config = LearnerMemoryConfig::default_config();
        config.uses_model = true;
        let run_a = run_learner_memory(&learner, &intent_a, config);
        let run_b = run_learner_memory(&learner, &intent_b, config);
        assert_ne!(
            run_a.receipt.receipt_hash, run_b.receipt.receipt_hash,
            "memory receipt_hash must fold the source qflow receipt hash"
        );
    }

    #[test]
    fn demo_json_re_derives_and_refuses_tampering() {
        let json = learner_memory_demo_json();
        assert!(json.contains("\"memory\""));
        assert!(verify_learner_memory_demo_json(&json).is_ok());
        assert_eq!(
            verify_learner_memory_demo_json(&format!("{json} ")),
            Err(LearnerMemoryError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_has_named_scenarios_and_replays() {
        let matrix = learner_memory_matrix();
        assert_eq!(matrix.scenario_count, LEARNER_MEMORY_SCENARIO_COUNT);
        assert_eq!(matrix.cells.len(), LEARNER_MEMORY_SCENARIO_COUNT);
        assert_eq!(
            matrix.mapped_count + matrix.refused_count,
            LEARNER_MEMORY_SCENARIO_COUNT
        );
        for (cell, name) in matrix
            .cells
            .iter()
            .zip(LEARNER_MEMORY_SCENARIO_NAMES.iter())
        {
            assert_eq!(&cell.scenario, name);
            assert_ne!(cell.outcome, "unknown");
        }
        let json = learner_memory_matrix_json();
        assert!(verify_learner_memory_matrix_json(&json).is_ok());
        assert_eq!(
            verify_learner_memory_matrix_json(&format!("{json} ")),
            Err(LearnerMemoryError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = learner_memory_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "serialized_learner_memory_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_learner_memory_tamper_refused")
        );
        assert!(!cell.memory_mapped);
        let json = learner_memory_demo_json();
        assert!(verify_learner_memory_demo_json(&flip_last_byte(&json)).is_err());
    }

    #[test]
    fn decisions_and_refusals_are_complete_and_slugged() {
        assert_eq!(LearnerMemoryDecision::ALL.len(), 2);
        assert_eq!(LearnerMemoryRefusal::ALL.len(), 13);
        let mut slugs = LearnerMemoryRefusal::ALL
            .iter()
            .map(|r| r.slug())
            .collect::<Vec<_>>();
        slugs.sort_unstable();
        let n = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), n);
    }
}
