//! LEARNER-MEMORY-1: the consented append-only learner-journal receipt layer.
//!
//! This module persists POINTER ENTRIES only — receipt hashes, counts, and
//! consent fields — never memory content. Every append requires an explicit,
//! scope-bound consent affirmation; every append verifies the existing chain
//! first; every verification re-derives the canonical journal and byte-compares
//! (Serialize-but-NOT-Deserialize: a stored journal is untrusted input that is
//! byte-verified, never parsed).
//!
//! Authority law: the journal AUTHORIZES nothing. It records that a verified
//! LEARNER-MEMORY-0 candidate (itself candidate_only) was consented into an
//! append-only pointer log. The canonical journal pins the consent operator to
//! [`CANONICAL_CONSENT_OPERATOR`] so the derivation stays fully deterministic;
//! the CLI consent flags must re-affirm exactly the canonical consent (operator
//! plus per-candidate scope) or the append refuses.
//!
//! Chain law: entry N carries `seq == N` (1-based), `prev_entry_hash` equal to
//! entry N-1's `entry_hash` (or the genesis hash for entry 1), and an
//! `entry_hash` folded over every field. [`journal_entries_are_chain_linked`]
//! walks the whole journal before any append and maps each failure to a
//! DISTINCT refusal: recompute mismatch -> tamper, non-monotonic seq ->
//! reorder, seq gap -> deletion, forged prev pointer -> chain break, repeated
//! candidate hash -> duplicate, root count/head mismatch -> tamper.
//!
//! Boundary: no personalization, no autonomous recall, no behavior adaptation,
//! no trait inference, no diagnosis, no model, no training, no truth creation,
//! no v0.1 retag. All filesystem I/O for the live append verb lives in main.rs;
//! this module is a pure fold.

use serde::Serialize;

use crate::{
    learner_memory_demo, literature_intent_demo, run_learner_memory, run_learner_memory_default,
    run_learner_model_default, teach_map_demo, ConfidenceMarker, LearnerMemoryConfig,
    LearnerMemoryDecision, LearnerMemoryRun, LearnerModelObservation,
};

const SCHEMA_JOURNAL: &str = "learner-journal-v0.1";
const SCHEMA_ENTRY: &str = "learner-journal-entry-v0.1";
const SCHEMA_RECEIPT: &str = "learner-journal-receipt-v0.1";
const SCHEMA_MATRIX: &str = "learner-journal-matrix-v0.1";

/// The pinned consent operator for the canonical deterministic journal.
pub const CANONICAL_CONSENT_OPERATOR: &str = "operator";

/// The number of distinct canonical demo candidates the journal can hold.
pub const LEARNER_JOURNAL_DEMO_CANDIDATES: usize = 2;

const LEARNER_JOURNAL_USES_MODEL: bool = false;
const LEARNER_JOURNAL_USES_TRAINING: bool = false;
const LEARNER_JOURNAL_PERSONALIZES: bool = false;
const LEARNER_JOURNAL_AUTONOMOUSLY_RECALLS: bool = false;
const LEARNER_JOURNAL_PROFILES_LEARNER: bool = false;
const LEARNER_JOURNAL_INFERS_DIAGNOSIS: bool = false;

pub const LEARNER_JOURNAL_BOUNDARY_LINES: [&str; 12] = [
    "LEARNER-MEMORY-1 persists pointer entries only.",
    "It requires explicit consent per append.",
    "It verifies the append-only chain before writing.",
    "It refuses invalid or tampered journals.",
    "It does not store rich personal memory content.",
    "It does not personalize generation.",
    "It does not recall autonomously.",
    "It does not adapt behavior.",
    "It does not infer traits.",
    "It does not diagnose.",
    "It does not train or run a model.",
    "It does not retag v0.1.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearnerJournalDecision {
    JournalAppended,
    JournalRefused,
}

impl LearnerJournalDecision {
    pub fn slug(&self) -> &'static str {
        match self {
            LearnerJournalDecision::JournalAppended => "journal_appended",
            LearnerJournalDecision::JournalRefused => "journal_refused",
        }
    }
}

/// Every way an append or a supplied journal can be refused. Each variant is
/// CONSTRUCTED in a reachable production path (the A3 fail-closed-debris law).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LearnerJournalRefusal {
    MissingConsent,
    InvalidConsent,
    MemoryCandidateRefused,
    JournalEntryTamper,
    JournalChainBreak,
    JournalReorder,
    JournalDeletion,
    DuplicateEntry,
    UnsupportedSourceReceipt,
    PersonalizationSignalDetected,
    AutonomousRecallSignalDetected,
    ProfilingSignalDetected,
    DiagnosisSignalDetected,
    ModelSignalDetected,
    TrainingSignalDetected,
    SerializedLearnerJournalTamper,
}

impl LearnerJournalRefusal {
    pub const ALL: [LearnerJournalRefusal; 16] = [
        LearnerJournalRefusal::MissingConsent,
        LearnerJournalRefusal::InvalidConsent,
        LearnerJournalRefusal::MemoryCandidateRefused,
        LearnerJournalRefusal::JournalEntryTamper,
        LearnerJournalRefusal::JournalChainBreak,
        LearnerJournalRefusal::JournalReorder,
        LearnerJournalRefusal::JournalDeletion,
        LearnerJournalRefusal::DuplicateEntry,
        LearnerJournalRefusal::UnsupportedSourceReceipt,
        LearnerJournalRefusal::PersonalizationSignalDetected,
        LearnerJournalRefusal::AutonomousRecallSignalDetected,
        LearnerJournalRefusal::ProfilingSignalDetected,
        LearnerJournalRefusal::DiagnosisSignalDetected,
        LearnerJournalRefusal::ModelSignalDetected,
        LearnerJournalRefusal::TrainingSignalDetected,
        LearnerJournalRefusal::SerializedLearnerJournalTamper,
    ];

    pub fn slug(&self) -> &'static str {
        match self {
            LearnerJournalRefusal::MissingConsent => "missing_consent",
            LearnerJournalRefusal::InvalidConsent => "invalid_consent",
            LearnerJournalRefusal::MemoryCandidateRefused => "memory_candidate_refused",
            LearnerJournalRefusal::JournalEntryTamper => "journal_entry_tamper",
            LearnerJournalRefusal::JournalChainBreak => "journal_chain_break",
            LearnerJournalRefusal::JournalReorder => "journal_reorder",
            LearnerJournalRefusal::JournalDeletion => "journal_deletion",
            LearnerJournalRefusal::DuplicateEntry => "duplicate_entry",
            LearnerJournalRefusal::UnsupportedSourceReceipt => "unsupported_source_receipt",
            LearnerJournalRefusal::PersonalizationSignalDetected => {
                "personalization_signal_detected"
            }
            LearnerJournalRefusal::AutonomousRecallSignalDetected => {
                "autonomous_recall_signal_detected"
            }
            LearnerJournalRefusal::ProfilingSignalDetected => "profiling_signal_detected",
            LearnerJournalRefusal::DiagnosisSignalDetected => "diagnosis_signal_detected",
            LearnerJournalRefusal::ModelSignalDetected => "model_signal_detected",
            LearnerJournalRefusal::TrainingSignalDetected => "training_signal_detected",
            LearnerJournalRefusal::SerializedLearnerJournalTamper => {
                "serialized_learner_journal_tamper"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearnerJournalError {
    ReplayMismatch,
}

/// Closed-gate config: any true flag refuses before any journal work happens.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct LearnerJournalConfig {
    pub uses_model: bool,
    pub uses_training: bool,
    pub personalizes: bool,
    pub autonomously_recalls: bool,
    pub profiles_learner: bool,
    pub infers_diagnosis: bool,
}

impl LearnerJournalConfig {
    pub fn default_config() -> Self {
        LearnerJournalConfig {
            uses_model: LEARNER_JOURNAL_USES_MODEL,
            uses_training: LEARNER_JOURNAL_USES_TRAINING,
            personalizes: LEARNER_JOURNAL_PERSONALIZES,
            autonomously_recalls: LEARNER_JOURNAL_AUTONOMOUSLY_RECALLS,
            profiles_learner: LEARNER_JOURNAL_PROFILES_LEARNER,
            infers_diagnosis: LEARNER_JOURNAL_INFERS_DIAGNOSIS,
        }
    }
}

/// An explicit, scope-bound consent affirmation for ONE append (the
/// model_promote `PromotionOperatorApprovalReceipt` precedent, strengthened:
/// the scope pins the exact memory-candidate receipt this consent authorizes).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerJournalConsent {
    pub operator: String,
    pub journal_scope: String,
    pub consents_to_append: bool,
}

/// The exact scope string a consent must carry to authorize appending this
/// candidate — one consent cannot be replayed against a different candidate.
pub fn journal_scope_for_candidate(candidate: &LearnerMemoryRun) -> String {
    format!(
        "learner_memory_receipt:{:016x}",
        candidate.receipt.receipt_hash
    )
}

/// Structural boundary flags. Every flag names a forbidden behavior and must
/// stay false; `all_inert` is re-derived, never trusted from input.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct LearnerJournalBoundary {
    pub stores_memory_content: bool,
    pub skips_consent_check: bool,
    pub mutates_prior_entries: bool,
    pub allows_chain_breaks: bool,
    pub personalizes_generation: bool,
    pub autonomously_recalls: bool,
    pub adapts_behavior: bool,
    pub infers_traits: bool,
    pub diagnoses: bool,
    pub uses_model: bool,
    pub uses_training: bool,
    pub retags_v01: bool,
}

impl LearnerJournalBoundary {
    pub fn inert() -> Self {
        LearnerJournalBoundary {
            stores_memory_content: false,
            skips_consent_check: false,
            mutates_prior_entries: false,
            allows_chain_breaks: false,
            personalizes_generation: false,
            autonomously_recalls: LEARNER_JOURNAL_AUTONOMOUSLY_RECALLS,
            adapts_behavior: false,
            infers_traits: LEARNER_JOURNAL_PROFILES_LEARNER,
            diagnoses: LEARNER_JOURNAL_INFERS_DIAGNOSIS,
            uses_model: LEARNER_JOURNAL_USES_MODEL,
            uses_training: LEARNER_JOURNAL_USES_TRAINING,
            retags_v01: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !(self.stores_memory_content
            || self.skips_consent_check
            || self.mutates_prior_entries
            || self.allows_chain_breaks
            || self.personalizes_generation
            || self.autonomously_recalls
            || self.adapts_behavior
            || self.infers_traits
            || self.diagnoses
            || self.uses_model
            || self.uses_training
            || self.retags_v01)
    }
}

/// One append-only pointer entry. Hashes and counts only — no memory content.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerJournalEntry {
    pub schema: String,
    pub seq: u64,
    pub prev_entry_hash: u64,
    pub candidate_receipt_hash: u64,
    pub source_learner_receipt_hash: u64,
    pub source_teach_receipt_hash: u64,
    pub source_intent_receipt_hash: u64,
    pub source_qflow_receipt_hash: u64,
    pub document: String,
    pub item_count: usize,
    pub consent_operator: String,
    pub consent_scope: String,
    pub entry_hash: u64,
}

/// The append-only journal: a hash-linked pointer log with a re-derivable head.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LearnerJournal {
    pub schema: String,
    pub entry_count: usize,
    pub head_hash: u64,
    pub entries: Vec<LearnerJournalEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerJournalReceipt {
    pub schema: String,
    pub config: LearnerJournalConfig,
    pub entry_count: usize,
    pub head_hash: u64,
    pub candidate_receipt_hash: u64,
    pub decision: LearnerJournalDecision,
    pub refusal: Option<LearnerJournalRefusal>,
    pub receipt_hash: u64,
    pub boundary: LearnerJournalBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerJournalRun {
    pub receipt: LearnerJournalReceipt,
    pub journal: Option<LearnerJournal>,
    pub decision: LearnerJournalDecision,
    pub refusal: Option<LearnerJournalRefusal>,
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

/// The head hash of an empty journal: folded over the journal schema only.
fn genesis_hash() -> u64 {
    fnv_mix(0xcbf2_9ce4_8422_2325, SCHEMA_JOURNAL.as_bytes())
}

fn fold_entry_hash(entry: &LearnerJournalEntry) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, entry.schema.as_bytes());
    h = fnv_u64(h, entry.seq);
    h = fnv_u64(h, entry.prev_entry_hash);
    h = fnv_u64(h, entry.candidate_receipt_hash);
    h = fnv_u64(h, entry.source_learner_receipt_hash);
    h = fnv_u64(h, entry.source_teach_receipt_hash);
    h = fnv_u64(h, entry.source_intent_receipt_hash);
    h = fnv_u64(h, entry.source_qflow_receipt_hash);
    h = fnv_mix(h, entry.document.as_bytes());
    h = fnv_u64(h, entry.item_count as u64);
    h = fnv_mix(h, entry.consent_operator.as_bytes());
    h = fnv_mix(h, entry.consent_scope.as_bytes());
    h
}

fn fold_receipt_hash(
    config: &LearnerJournalConfig,
    entry_count: usize,
    head_hash: u64,
    candidate_receipt_hash: u64,
    decision: LearnerJournalDecision,
    refusal: Option<LearnerJournalRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, SCHEMA_RECEIPT.as_bytes());
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, config.personalizes as u64);
    h = fnv_u64(h, config.autonomously_recalls as u64);
    h = fnv_u64(h, config.profiles_learner as u64);
    h = fnv_u64(h, config.infers_diagnosis as u64);
    h = fnv_u64(h, entry_count as u64);
    h = fnv_u64(h, head_hash);
    h = fnv_u64(h, candidate_receipt_hash);
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

/// An empty journal at the genesis head — the only valid starting state.
pub fn empty_learner_journal() -> LearnerJournal {
    LearnerJournal {
        schema: SCHEMA_JOURNAL.to_string(),
        entry_count: 0,
        head_hash: genesis_hash(),
        entries: Vec::new(),
    }
}

/// Walk a supplied journal and refuse the FIRST structural violation with a
/// distinct refusal. Check order is load-bearing: per-entry recompute (tamper)
/// -> seq monotonicity (reorder) -> seq continuity (deletion) -> prev pointer
/// (chain break) -> repeated candidate (duplicate) -> root count/head (tamper).
pub fn journal_entries_are_chain_linked(journal: &LearnerJournal) -> Option<LearnerJournalRefusal> {
    for entry in &journal.entries {
        if fold_entry_hash(entry) != entry.entry_hash {
            return Some(LearnerJournalRefusal::JournalEntryTamper);
        }
    }
    // Reorder must be judged over the WHOLE seq vector before continuity: a
    // swapped pair ([2, 1]) reads as a gap to a single forward walk and would
    // be mislabeled as a deletion.
    let seqs = journal
        .entries
        .iter()
        .map(|entry| entry.seq)
        .collect::<Vec<_>>();
    if seqs.windows(2).any(|pair| pair[1] <= pair[0]) {
        return Some(LearnerJournalRefusal::JournalReorder);
    }
    for (index, seq) in seqs.iter().enumerate() {
        if *seq != index as u64 + 1 {
            return Some(LearnerJournalRefusal::JournalDeletion);
        }
    }
    let mut expected_prev = genesis_hash();
    for entry in &journal.entries {
        if entry.prev_entry_hash != expected_prev {
            return Some(LearnerJournalRefusal::JournalChainBreak);
        }
        expected_prev = entry.entry_hash;
    }
    let mut seen = Vec::new();
    for entry in &journal.entries {
        if seen.contains(&entry.candidate_receipt_hash) {
            return Some(LearnerJournalRefusal::DuplicateEntry);
        }
        seen.push(entry.candidate_receipt_hash);
    }
    if journal.entry_count != journal.entries.len() || journal.head_hash != expected_prev {
        return Some(LearnerJournalRefusal::JournalEntryTamper);
    }
    None
}

fn assemble(
    config: LearnerJournalConfig,
    entry_count: usize,
    head_hash: u64,
    candidate_receipt_hash: u64,
    decision: LearnerJournalDecision,
    refusal: Option<LearnerJournalRefusal>,
    journal: Option<LearnerJournal>,
) -> LearnerJournalRun {
    let boundary = LearnerJournalBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let receipt_hash = fold_receipt_hash(
        &config,
        entry_count,
        head_hash,
        candidate_receipt_hash,
        decision,
        refusal,
    );
    LearnerJournalRun {
        receipt: LearnerJournalReceipt {
            schema: SCHEMA_RECEIPT.to_string(),
            config,
            entry_count,
            head_hash,
            candidate_receipt_hash,
            decision,
            refusal,
            receipt_hash,
            boundary,
            boundary_all_inert,
        },
        journal,
        decision,
        refusal,
    }
}

pub fn append_learner_journal_default(
    journal: &LearnerJournal,
    candidate: &LearnerMemoryRun,
    consent: Option<&LearnerJournalConsent>,
) -> LearnerJournalRun {
    append_learner_journal(
        journal,
        candidate,
        consent,
        LearnerJournalConfig::default_config(),
    )
}

/// Append one consented candidate to a verified journal, or refuse. The prior
/// journal is untrusted input: its whole chain is verified before any append.
pub fn append_learner_journal(
    journal: &LearnerJournal,
    candidate: &LearnerMemoryRun,
    consent: Option<&LearnerJournalConsent>,
    config: LearnerJournalConfig,
) -> LearnerJournalRun {
    let candidate_hash = candidate.receipt.receipt_hash;
    let refuse = |refusal: LearnerJournalRefusal| {
        assemble(
            config,
            journal.entry_count,
            journal.head_hash,
            candidate_hash,
            LearnerJournalDecision::JournalRefused,
            Some(refusal),
            None,
        )
    };
    let signal = if config.uses_model {
        Some(LearnerJournalRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(LearnerJournalRefusal::TrainingSignalDetected)
    } else if config.personalizes {
        Some(LearnerJournalRefusal::PersonalizationSignalDetected)
    } else if config.autonomously_recalls {
        Some(LearnerJournalRefusal::AutonomousRecallSignalDetected)
    } else if config.profiles_learner {
        Some(LearnerJournalRefusal::ProfilingSignalDetected)
    } else if config.infers_diagnosis {
        Some(LearnerJournalRefusal::DiagnosisSignalDetected)
    } else {
        None
    };
    if let Some(refusal) = signal {
        return refuse(refusal);
    }
    if let Some(refusal) = journal_entries_are_chain_linked(journal) {
        return refuse(refusal);
    }
    if candidate.decision != LearnerMemoryDecision::MemoryCandidateMapped {
        return refuse(LearnerJournalRefusal::MemoryCandidateRefused);
    }
    let memory = match candidate.memory.as_ref() {
        Some(memory) => memory,
        None => return refuse(LearnerJournalRefusal::MemoryCandidateRefused),
    };
    let consent = match consent {
        Some(consent) => consent,
        None => return refuse(LearnerJournalRefusal::MissingConsent),
    };
    if !consent.consents_to_append
        || consent.operator != CANONICAL_CONSENT_OPERATOR
        || consent.journal_scope != journal_scope_for_candidate(candidate)
    {
        return refuse(LearnerJournalRefusal::InvalidConsent);
    }
    let receipt = &candidate.receipt;
    let spine_matches = memory.source_learner_receipt_hash == receipt.source_learner_receipt_hash
        && memory.source_teach_receipt_hash == receipt.source_teach_receipt_hash
        && memory.source_intent_receipt_hash == receipt.source_intent_receipt_hash
        && memory.source_qflow_receipt_hash == receipt.source_qflow_receipt_hash;
    if !spine_matches {
        return refuse(LearnerJournalRefusal::UnsupportedSourceReceipt);
    }
    if journal
        .entries
        .iter()
        .any(|entry| entry.candidate_receipt_hash == candidate_hash)
    {
        return refuse(LearnerJournalRefusal::DuplicateEntry);
    }
    let mut entry = LearnerJournalEntry {
        schema: SCHEMA_ENTRY.to_string(),
        seq: journal.entry_count as u64 + 1,
        prev_entry_hash: journal.head_hash,
        candidate_receipt_hash: candidate_hash,
        source_learner_receipt_hash: receipt.source_learner_receipt_hash,
        source_teach_receipt_hash: receipt.source_teach_receipt_hash,
        source_intent_receipt_hash: receipt.source_intent_receipt_hash,
        source_qflow_receipt_hash: receipt.source_qflow_receipt_hash,
        document: memory.document.clone(),
        item_count: receipt.item_count,
        consent_operator: consent.operator.clone(),
        consent_scope: consent.journal_scope.clone(),
        entry_hash: 0,
    };
    entry.entry_hash = fold_entry_hash(&entry);
    let head_hash = entry.entry_hash;
    let mut entries = journal.entries.clone();
    entries.push(entry);
    let appended = LearnerJournal {
        schema: SCHEMA_JOURNAL.to_string(),
        entry_count: entries.len(),
        head_hash,
        entries,
    };
    if let Some(refusal) = journal_entries_are_chain_linked(&appended) {
        return refuse(refusal);
    }
    assemble(
        config,
        appended.entry_count,
        appended.head_hash,
        candidate_hash,
        LearnerJournalDecision::JournalAppended,
        None,
        Some(appended),
    )
}

/// The first canonical demo candidate: the LEARNER-MEMORY-0 demo itself.
fn demo_candidate_a() -> LearnerMemoryRun {
    learner_memory_demo()
}

/// The second canonical demo candidate: the same verified chain observed with
/// no quiz answers, no misconception flags, and an unstated confidence marker —
/// a distinct learner state, so a distinct memory-candidate receipt hash.
fn demo_candidate_b() -> LearnerMemoryRun {
    let teach = teach_map_demo();
    let base = crate::learner_model_demo();
    let seen_lesson_item_ids = base
        .learner_state
        .as_ref()
        .expect("canonical learner state")
        .seen_items
        .iter()
        .map(|item| item.item_id.clone())
        .collect::<Vec<_>>();
    let observation = LearnerModelObservation {
        seen_lesson_item_ids,
        quiz_answers: Vec::new(),
        misconception_flags: Vec::new(),
        confidence_marker: ConfidenceMarker::Unstated,
    };
    let learner = run_learner_model_default(&teach, observation);
    run_learner_memory_default(&learner, &literature_intent_demo())
}

fn demo_candidate(index: usize) -> LearnerMemoryRun {
    if index == 0 {
        demo_candidate_a()
    } else {
        demo_candidate_b()
    }
}

fn canonical_consent_for(candidate: &LearnerMemoryRun) -> LearnerJournalConsent {
    LearnerJournalConsent {
        operator: CANONICAL_CONSENT_OPERATOR.to_string(),
        journal_scope: journal_scope_for_candidate(candidate),
        consents_to_append: true,
    }
}

/// The canonical journal after the first `n` demo appends (n in 0..=2).
pub fn learner_journal_at(n: usize) -> Option<LearnerJournal> {
    if n > LEARNER_JOURNAL_DEMO_CANDIDATES {
        return None;
    }
    let mut journal = empty_learner_journal();
    for index in 0..n {
        let candidate = demo_candidate(index);
        let consent = canonical_consent_for(&candidate);
        let run = append_learner_journal_default(&journal, &candidate, Some(&consent));
        journal = run.journal?;
    }
    Some(journal)
}

/// The canonical journal state serialized for the on-disk append verb.
pub fn learner_journal_state_json(journal: &LearnerJournal) -> String {
    serde_json::to_string_pretty(journal).expect("learner journal serializes")
}

pub fn learner_journal_json_at(n: usize) -> Option<String> {
    learner_journal_at(n).map(|journal| learner_journal_state_json(&journal))
}

/// Append the (n+1)th canonical candidate to the canonical journal at `n`,
/// gated on the supplied consent. Past the last candidate this re-attempts the
/// final candidate and refuses as a duplicate — the journal is append-only and
/// finite until a later gate supplies new verified candidates.
pub fn learner_journal_append_at(n: usize, consent: &LearnerJournalConsent) -> LearnerJournalRun {
    let index = n.min(LEARNER_JOURNAL_DEMO_CANDIDATES - 1);
    let candidate = demo_candidate(index);
    let journal = match learner_journal_at(n.min(LEARNER_JOURNAL_DEMO_CANDIDATES)) {
        Some(journal) => journal,
        None => empty_learner_journal(),
    };
    append_learner_journal_default(&journal, &candidate, Some(consent))
}

/// The canonical LEARNER-MEMORY-1 demo: two consented appends from genesis.
pub fn learner_journal_demo() -> LearnerJournalRun {
    let candidate_a = demo_candidate_a();
    let consent_a = canonical_consent_for(&candidate_a);
    let first =
        append_learner_journal_default(&empty_learner_journal(), &candidate_a, Some(&consent_a));
    let journal = first.journal.expect("first canonical append succeeds");
    let candidate_b = demo_candidate_b();
    let consent_b = canonical_consent_for(&candidate_b);
    append_learner_journal_default(&journal, &candidate_b, Some(&consent_b))
}

pub fn learner_journal_demo_json() -> String {
    serde_json::to_string_pretty(&learner_journal_demo()).expect("learner journal demo serializes")
}

pub fn verify_learner_journal_demo_json(candidate: &str) -> Result<(), LearnerJournalError> {
    if candidate == learner_journal_demo_json() {
        Ok(())
    } else {
        Err(LearnerJournalError::ReplayMismatch)
    }
}

pub const LEARNER_JOURNAL_SCENARIO_COUNT: usize = 18;
pub const LEARNER_JOURNAL_SCENARIO_NAMES: [&str; LEARNER_JOURNAL_SCENARIO_COUNT] = [
    "journal_appended",
    "missing_consent_refused",
    "invalid_consent_flag_refused",
    "invalid_consent_scope_refused",
    "memory_candidate_refused",
    "journal_entry_tamper_refused",
    "journal_chain_break_refused",
    "journal_reorder_refused",
    "journal_deletion_refused",
    "duplicate_entry_refused",
    "unsupported_source_receipt_refused",
    "model_signal_refused",
    "training_signal_refused",
    "personalization_signal_refused",
    "autonomous_recall_signal_refused",
    "profiling_signal_refused",
    "diagnosis_signal_refused",
    "serialized_learner_journal_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct LearnerJournalCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub entry_count: usize,
    pub head_advanced: bool,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LearnerJournalMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<LearnerJournalCell>,
    pub appended_count: usize,
    pub refused_count: usize,
    pub boundary: LearnerJournalBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_run(scenario: &str, run: &LearnerJournalRun) -> LearnerJournalCell {
    LearnerJournalCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        entry_count: run.receipt.entry_count,
        head_advanced: run.receipt.head_hash != genesis_hash()
            && run.decision == LearnerJournalDecision::JournalAppended,
        boundary_all_inert: run.receipt.boundary_all_inert,
    }
}

fn cell_from_guard(scenario: &str, refusal: Option<LearnerJournalRefusal>) -> LearnerJournalCell {
    LearnerJournalCell {
        scenario: scenario.to_string(),
        outcome: match refusal {
            Some(_) => "journal_refused".to_string(),
            None => "violation_missed".to_string(),
        },
        refusal: refusal.map(|r| r.slug().to_string()),
        entry_count: 0,
        head_advanced: false,
        boundary_all_inert: LearnerJournalBoundary::inert().all_inert(),
    }
}

fn signal_cell(scenario: &str, set: fn(&mut LearnerJournalConfig)) -> LearnerJournalCell {
    let mut config = LearnerJournalConfig::default_config();
    set(&mut config);
    let candidate = demo_candidate_a();
    let consent = canonical_consent_for(&candidate);
    let run = append_learner_journal(&empty_learner_journal(), &candidate, Some(&consent), config);
    cell_from_run(scenario, &run)
}

fn cell_for(scenario: &str) -> LearnerJournalCell {
    match scenario {
        "journal_appended" => {
            let run = learner_journal_demo();
            cell_from_run(scenario, &run)
        }
        "missing_consent_refused" => {
            let candidate = demo_candidate_a();
            let run = append_learner_journal_default(&empty_learner_journal(), &candidate, None);
            cell_from_run(scenario, &run)
        }
        "invalid_consent_flag_refused" => {
            let candidate = demo_candidate_a();
            let mut consent = canonical_consent_for(&candidate);
            consent.consents_to_append = false;
            let run = append_learner_journal_default(
                &empty_learner_journal(),
                &candidate,
                Some(&consent),
            );
            cell_from_run(scenario, &run)
        }
        "invalid_consent_scope_refused" => {
            // Consent scoped to candidate B cannot authorize appending candidate A.
            let candidate = demo_candidate_a();
            let consent = canonical_consent_for(&demo_candidate_b());
            let run = append_learner_journal_default(
                &empty_learner_journal(),
                &candidate,
                Some(&consent),
            );
            cell_from_run(scenario, &run)
        }
        "memory_candidate_refused" => {
            let mut config = LearnerMemoryConfig::default_config();
            config.uses_model = true;
            let refused_candidate = run_learner_memory(
                &crate::learner_model_demo(),
                &literature_intent_demo(),
                config,
            );
            let consent = canonical_consent_for(&refused_candidate);
            let run = append_learner_journal_default(
                &empty_learner_journal(),
                &refused_candidate,
                Some(&consent),
            );
            cell_from_run(scenario, &run)
        }
        "journal_entry_tamper_refused" => {
            // Mutate a stored field without re-folding: recompute must mismatch.
            let mut journal = learner_journal_at(2).expect("canonical journal");
            journal.entries[0].document = "tampered.md".to_string();
            cell_from_guard(scenario, journal_entries_are_chain_linked(&journal))
        }
        "journal_chain_break_refused" => {
            // Forge the prev pointer AND re-fold the entry hash so recompute
            // passes; only the chain-link walk can catch this.
            let mut journal = learner_journal_at(2).expect("canonical journal");
            journal.entries[1].prev_entry_hash = 0xdead_beef;
            journal.entries[1].entry_hash = fold_entry_hash(&journal.entries[1]);
            journal.head_hash = journal.entries[1].entry_hash;
            cell_from_guard(scenario, journal_entries_are_chain_linked(&journal))
        }
        "journal_reorder_refused" => {
            let mut journal = learner_journal_at(2).expect("canonical journal");
            journal.entries.swap(0, 1);
            cell_from_guard(scenario, journal_entries_are_chain_linked(&journal))
        }
        "journal_deletion_refused" => {
            let mut journal = learner_journal_at(2).expect("canonical journal");
            journal.entries.remove(0);
            journal.entry_count = 1;
            cell_from_guard(scenario, journal_entries_are_chain_linked(&journal))
        }
        "duplicate_entry_refused" => {
            // Re-consent the SAME candidate through the production append path.
            let journal = learner_journal_at(1).expect("canonical journal");
            let candidate = demo_candidate_a();
            let consent = canonical_consent_for(&candidate);
            let run = append_learner_journal_default(&journal, &candidate, Some(&consent));
            cell_from_run(scenario, &run)
        }
        "unsupported_source_receipt_refused" => {
            // Forge the candidate's map spine away from its receipt spine.
            let mut candidate = demo_candidate_a();
            if let Some(memory) = candidate.memory.as_mut() {
                memory.source_qflow_receipt_hash ^= 1;
            }
            let consent = canonical_consent_for(&candidate);
            let run = append_learner_journal_default(
                &empty_learner_journal(),
                &candidate,
                Some(&consent),
            );
            cell_from_run(scenario, &run)
        }
        "model_signal_refused" => signal_cell(scenario, |c| c.uses_model = true),
        "training_signal_refused" => signal_cell(scenario, |c| c.uses_training = true),
        "personalization_signal_refused" => signal_cell(scenario, |c| c.personalizes = true),
        "autonomous_recall_signal_refused" => {
            signal_cell(scenario, |c| c.autonomously_recalls = true)
        }
        "profiling_signal_refused" => signal_cell(scenario, |c| c.profiles_learner = true),
        "diagnosis_signal_refused" => signal_cell(scenario, |c| c.infers_diagnosis = true),
        "serialized_learner_journal_tamper_refused" => {
            // Serialize the real journal artifact, flip one byte, and confirm
            // the tamper is detectable — constructing the refusal that names
            // this scenario (the QSELECT/QFLOW/LEARNER-MEMORY precedent).
            let json = learner_journal_demo_json();
            let refused = verify_learner_journal_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(LearnerJournalRefusal::SerializedLearnerJournalTamper)
            } else {
                None
            };
            LearnerJournalCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: refusal.map(|r| r.slug().to_string()),
                entry_count: 0,
                head_advanced: false,
                boundary_all_inert: LearnerJournalBoundary::inert().all_inert(),
            }
        }
        other => LearnerJournalCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            entry_count: 0,
            head_advanced: false,
            boundary_all_inert: false,
        },
    }
}

pub fn learner_journal_matrix() -> LearnerJournalMatrix {
    let cells = LEARNER_JOURNAL_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let appended_count = cells
        .iter()
        .filter(|cell| cell.outcome == "journal_appended")
        .count();
    let refused_count = cells.len() - appended_count;
    let boundary = LearnerJournalBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    LearnerJournalMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        appended_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn learner_journal_matrix_json() -> String {
    serde_json::to_string_pretty(&learner_journal_matrix())
        .expect("learner journal matrix serializes")
}

pub fn verify_learner_journal_matrix_json(candidate: &str) -> Result<(), LearnerJournalError> {
    if candidate == learner_journal_matrix_json() {
        Ok(())
    } else {
        Err(LearnerJournalError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type SignalCase = (fn(&mut LearnerJournalConfig), LearnerJournalRefusal);

    #[test]
    fn demo_appends_two_entries_from_genesis() {
        let run = learner_journal_demo();
        assert_eq!(run.decision, LearnerJournalDecision::JournalAppended);
        assert!(run.refusal.is_none());
        let journal = run.journal.as_ref().expect("demo journal");
        assert_eq!(journal.entry_count, 2);
        assert_eq!(journal.entries.len(), 2);
        assert_ne!(journal.head_hash, genesis_hash());
        assert!(run.receipt.boundary_all_inert);
    }

    #[test]
    fn head_advances_and_links_per_append() {
        let one = learner_journal_at(1).expect("journal at 1");
        let two = learner_journal_at(2).expect("journal at 2");
        assert_ne!(one.head_hash, two.head_hash);
        assert_eq!(two.entries[0].prev_entry_hash, genesis_hash());
        assert_eq!(two.entries[1].prev_entry_hash, two.entries[0].entry_hash);
        assert_eq!(two.head_hash, two.entries[1].entry_hash);
    }

    #[test]
    fn demo_candidates_are_distinct() {
        assert_ne!(
            demo_candidate_a().receipt.receipt_hash,
            demo_candidate_b().receipt.receipt_hash,
            "the duplicate-entry law needs two distinct canonical candidates"
        );
        assert_eq!(
            demo_candidate_b().decision,
            LearnerMemoryDecision::MemoryCandidateMapped
        );
    }

    #[test]
    fn entries_point_at_candidate_receipts() {
        let candidate = demo_candidate_a();
        let journal = learner_journal_at(1).expect("journal at 1");
        let entry = &journal.entries[0];
        assert_eq!(entry.candidate_receipt_hash, candidate.receipt.receipt_hash);
        assert_eq!(
            entry.source_learner_receipt_hash,
            candidate.receipt.source_learner_receipt_hash
        );
        assert_eq!(
            entry.source_teach_receipt_hash,
            candidate.receipt.source_teach_receipt_hash
        );
        assert_eq!(
            entry.source_intent_receipt_hash,
            candidate.receipt.source_intent_receipt_hash
        );
        assert_eq!(
            entry.source_qflow_receipt_hash,
            candidate.receipt.source_qflow_receipt_hash
        );
        assert_eq!(entry.item_count, candidate.receipt.item_count);
        assert_eq!(entry.consent_scope, journal_scope_for_candidate(&candidate));
        assert_eq!(fold_entry_hash(entry), entry.entry_hash);
    }

    #[test]
    fn entries_carry_no_memory_content() {
        let candidate = demo_candidate_a();
        let journal = learner_journal_at(1).expect("journal at 1");
        let serialized = learner_journal_state_json(&journal);
        for item in &candidate.memory.as_ref().expect("demo memory").items {
            assert!(
                !serialized.contains(&item.content),
                "journal must not persist memory item content"
            );
        }
    }

    #[test]
    fn missing_consent_is_refused() {
        let run =
            append_learner_journal_default(&empty_learner_journal(), &demo_candidate_a(), None);
        assert_eq!(run.refusal, Some(LearnerJournalRefusal::MissingConsent));
        assert!(run.journal.is_none());
    }

    #[test]
    fn consent_without_affirmation_is_refused() {
        let candidate = demo_candidate_a();
        let mut consent = canonical_consent_for(&candidate);
        consent.consents_to_append = false;
        let run =
            append_learner_journal_default(&empty_learner_journal(), &candidate, Some(&consent));
        assert_eq!(run.refusal, Some(LearnerJournalRefusal::InvalidConsent));
    }

    #[test]
    fn consent_scope_mismatch_is_refused() {
        let candidate = demo_candidate_a();
        let consent = canonical_consent_for(&demo_candidate_b());
        let run =
            append_learner_journal_default(&empty_learner_journal(), &candidate, Some(&consent));
        assert_eq!(run.refusal, Some(LearnerJournalRefusal::InvalidConsent));
    }

    #[test]
    fn consent_operator_mismatch_is_refused() {
        let candidate = demo_candidate_a();
        let mut consent = canonical_consent_for(&candidate);
        consent.operator = "someone_else".to_string();
        let run =
            append_learner_journal_default(&empty_learner_journal(), &candidate, Some(&consent));
        assert_eq!(run.refusal, Some(LearnerJournalRefusal::InvalidConsent));
    }

    #[test]
    fn refused_candidate_cannot_be_journaled() {
        let mut config = LearnerMemoryConfig::default_config();
        config.uses_model = true;
        let refused = run_learner_memory(
            &crate::learner_model_demo(),
            &literature_intent_demo(),
            config,
        );
        let consent = canonical_consent_for(&refused);
        let run =
            append_learner_journal_default(&empty_learner_journal(), &refused, Some(&consent));
        assert_eq!(
            run.refusal,
            Some(LearnerJournalRefusal::MemoryCandidateRefused)
        );
    }

    #[test]
    fn tampered_entry_is_refused() {
        let mut journal = learner_journal_at(2).expect("canonical journal");
        journal.entries[0].item_count += 1;
        assert_eq!(
            journal_entries_are_chain_linked(&journal),
            Some(LearnerJournalRefusal::JournalEntryTamper)
        );
    }

    #[test]
    fn chain_break_is_refused() {
        let mut journal = learner_journal_at(2).expect("canonical journal");
        journal.entries[1].prev_entry_hash = 0xdead_beef;
        journal.entries[1].entry_hash = fold_entry_hash(&journal.entries[1]);
        journal.head_hash = journal.entries[1].entry_hash;
        assert_eq!(
            journal_entries_are_chain_linked(&journal),
            Some(LearnerJournalRefusal::JournalChainBreak)
        );
    }

    #[test]
    fn reordered_entries_are_refused() {
        let mut journal = learner_journal_at(2).expect("canonical journal");
        journal.entries.swap(0, 1);
        assert_eq!(
            journal_entries_are_chain_linked(&journal),
            Some(LearnerJournalRefusal::JournalReorder)
        );
    }

    #[test]
    fn deleted_entry_is_refused() {
        let mut journal = learner_journal_at(2).expect("canonical journal");
        journal.entries.remove(0);
        journal.entry_count = 1;
        assert_eq!(
            journal_entries_are_chain_linked(&journal),
            Some(LearnerJournalRefusal::JournalDeletion)
        );
    }

    #[test]
    fn forged_root_is_refused() {
        let mut journal = learner_journal_at(2).expect("canonical journal");
        journal.head_hash ^= 1;
        assert_eq!(
            journal_entries_are_chain_linked(&journal),
            Some(LearnerJournalRefusal::JournalEntryTamper)
        );
    }

    #[test]
    fn duplicate_append_is_refused() {
        let journal = learner_journal_at(1).expect("canonical journal");
        let candidate = demo_candidate_a();
        let consent = canonical_consent_for(&candidate);
        let run = append_learner_journal_default(&journal, &candidate, Some(&consent));
        assert_eq!(run.refusal, Some(LearnerJournalRefusal::DuplicateEntry));
    }

    #[test]
    fn forged_candidate_spine_is_refused() {
        let mut candidate = demo_candidate_a();
        candidate
            .memory
            .as_mut()
            .expect("demo memory")
            .source_qflow_receipt_hash ^= 1;
        let consent = canonical_consent_for(&candidate);
        let run =
            append_learner_journal_default(&empty_learner_journal(), &candidate, Some(&consent));
        assert_eq!(
            run.refusal,
            Some(LearnerJournalRefusal::UnsupportedSourceReceipt)
        );
    }

    #[test]
    fn every_signal_config_refuses_before_journal_work() {
        let cases: [SignalCase; 6] = [
            (
                |c| c.uses_model = true,
                LearnerJournalRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                LearnerJournalRefusal::TrainingSignalDetected,
            ),
            (
                |c| c.personalizes = true,
                LearnerJournalRefusal::PersonalizationSignalDetected,
            ),
            (
                |c| c.autonomously_recalls = true,
                LearnerJournalRefusal::AutonomousRecallSignalDetected,
            ),
            (
                |c| c.profiles_learner = true,
                LearnerJournalRefusal::ProfilingSignalDetected,
            ),
            (
                |c| c.infers_diagnosis = true,
                LearnerJournalRefusal::DiagnosisSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = LearnerJournalConfig::default_config();
            set(&mut config);
            let candidate = demo_candidate_a();
            let consent = canonical_consent_for(&candidate);
            let run = append_learner_journal(
                &empty_learner_journal(),
                &candidate,
                Some(&consent),
                config,
            );
            assert_eq!(run.refusal, Some(expected));
            assert!(run.journal.is_none());
        }
    }

    #[test]
    fn append_at_progression_matches_canonical_states() {
        let candidate = demo_candidate_a();
        let consent = canonical_consent_for(&candidate);
        let run = learner_journal_append_at(0, &consent);
        assert_eq!(run.decision, LearnerJournalDecision::JournalAppended);
        assert_eq!(
            run.journal.expect("appended journal"),
            learner_journal_at(1).expect("journal at 1")
        );
        let candidate_b = demo_candidate_b();
        let consent_b = canonical_consent_for(&candidate_b);
        let run = learner_journal_append_at(1, &consent_b);
        assert_eq!(
            run.journal.expect("appended journal"),
            learner_journal_at(2).expect("journal at 2")
        );
        let exhausted = learner_journal_append_at(2, &consent_b);
        assert_eq!(
            exhausted.refusal,
            Some(LearnerJournalRefusal::DuplicateEntry)
        );
    }

    #[test]
    fn receipt_folds_head_hash() {
        let one = learner_journal_append_at(0, &canonical_consent_for(&demo_candidate_a()));
        let two = learner_journal_append_at(1, &canonical_consent_for(&demo_candidate_b()));
        assert_ne!(one.receipt.head_hash, two.receipt.head_hash);
        assert_ne!(one.receipt.receipt_hash, two.receipt.receipt_hash);
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = learner_journal_demo_json();
        assert!(verify_learner_journal_demo_json(&json).is_ok());
        assert_eq!(
            verify_learner_journal_demo_json(&flip_last_byte(&json)),
            Err(LearnerJournalError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = learner_journal_matrix_json();
        assert!(verify_learner_journal_matrix_json(&json).is_ok());
        assert_eq!(
            verify_learner_journal_matrix_json(&flip_last_byte(&json)),
            Err(LearnerJournalError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = learner_journal_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|cell| cell.scenario == "serialized_learner_journal_tamper_refused")
            .expect("tamper scenario present");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_learner_journal_tamper")
        );
    }

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = learner_journal_matrix();
        assert_eq!(matrix.scenario_count, LEARNER_JOURNAL_SCENARIO_COUNT);
        assert_eq!(matrix.appended_count, 1);
        let constructed = matrix
            .cells
            .iter()
            .filter_map(|cell| cell.refusal.clone())
            .collect::<Vec<_>>();
        for refusal in LearnerJournalRefusal::ALL {
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
        assert_eq!(LEARNER_JOURNAL_BOUNDARY_LINES.len(), 12);
        let boundary = LearnerJournalBoundary::inert();
        assert!(boundary.all_inert());
        let mut broken = boundary;
        broken.autonomously_recalls = true;
        assert!(!broken.all_inert());
    }
}
