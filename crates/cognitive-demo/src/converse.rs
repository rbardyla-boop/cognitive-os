//! CONVERSE-0 — the verifier-grounded, no-model multi-turn conversation.
//!
//! QFLOW-0 turned the substrate into "can answer ONE local question by returning a
//! verified evidence packet, or a typed refusal". CONVERSE-0 wraps that answerer in
//! a MULTI-TURN loop WITHOUT adding a language model, learned scoring, training, or
//! any semantic claim:
//!
//!   a fixed vault of local docs + a typed script of turns
//!     → per turn: resolve the DECLARED scope to a document sub-slice
//!     → call the FROZEN `run_query_default` (QFLOW-0) over that slice
//!     → record either a verbatim verified answer summary OR a typed refusal
//!     → carry context to the next turn as document NAMES only
//!     → a hash-linked, byte-replayable `ConversationTranscript`.
//!
//! THE LAW (load-bearing):
//!   An answering turn IS a QFLOW verifier-authorized packet — nothing more.
//!   CONVERSE assembles a transcript; it does NOT score, ground, or verify anything
//!   itself, never generates or paraphrases prose, and never infers what a turn means
//!   from its words. "Context" across turns is the DECLARED [`TurnScope`] plus the
//!   document NAMES cited by prior VERIFIED answers, resolved by set membership over
//!   the FIXED vault — never by matching question words to content. An ungroundable
//!   turn is an honest typed refusal; a follow-up with no prior answer to point at
//!   REFUSES rather than silently widening to the whole vault (that would be guessing).
//!
//! Report types are `Serialize` but never `Deserialize`: a serialized transcript is
//! re-derived from the same vault + script and byte-compared, so a tampered artifact
//! is refused. The turn chain is FNV-1a hash-linked (the LEARNER-MEMORY-1 precedent):
//! per-turn recompute, seq monotonicity, seq continuity, and prev-pointer walks each
//! map to a DISTINCT refusal. Everything is integer-only and pure — no clock, no
//! entropy, no filesystem; the CLI shell (main.rs) owns all file access.

use serde::Serialize;

use crate::{run_query_default, VerifiedEvidencePacket};

/// Structural invariant: CONVERSE-0 runs no model and no training. Every forbidden
/// flag is sourced from this single `false` so no path can flip one true.
const CONVERSE_USES_MODEL: bool = false;

const SCHEMA_TRANSCRIPT: &str = "converse-transcript-v0.1";
const SCHEMA_TURN: &str = "converse-turn-v0.1";
const SCHEMA_MATRIX: &str = "converse-matrix-v0.1";

/// The most turns one conversation may hold. A longer script is refused before any
/// turn runs (`TooManyTurns`), so a conversation can never grow without bound.
pub const MAX_TURNS: usize = 64;

/// The most bytes a single question may carry. A longer question is refused per-turn
/// (`QuestionTooLong`) before it reaches the frozen answerer.
pub const MAX_QUESTION_LEN: usize = 512;

/// The authority boundary, verbatim (10 lines). CONVERSE-0 carries context; it never
/// interprets, generates, or elevates.
pub const CONVERSE_BOUNDARY_LINES: [&str; 10] = [
    "CONVERSE-0 assembles a multi-turn transcript only.",
    "An answering turn is exactly a QFLOW verified evidence packet.",
    "It does not score, ground, or verify anything itself.",
    "It does not generate, paraphrase, or invent prose.",
    "It does not infer a turn's scope from its words.",
    "It carries context as prior-answer document names only.",
    "It refuses a follow-up that has no prior answer to point at.",
    "It does not train or run a model.",
    "It does not mutate the vault across turns.",
    "It does not retag v0.1.",
];

// ---------------------------------------------------------------------------
// Turn scope — the CLOSED context selector (authored, never inferred)
// ---------------------------------------------------------------------------

/// Where a turn's question is allowed to look. Authored in the input; the engine
/// NEVER derives it from the question's words. Closed set — fail closed on anything
/// else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TurnScope {
    /// Search every document in the vault.
    WholeVault,
    /// Search only the documents cited by the immediately-preceding verified answer.
    PriorAnswer,
    /// Search every document cited by any verified answer so far in this conversation.
    ConversationSoFar,
}

impl TurnScope {
    pub const ALL: [TurnScope; 3] = [
        TurnScope::WholeVault,
        TurnScope::PriorAnswer,
        TurnScope::ConversationSoFar,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            TurnScope::WholeVault => "whole_vault",
            TurnScope::PriorAnswer => "prior_answer",
            TurnScope::ConversationSoFar => "conversation_so_far",
        }
    }

    /// Map exactly the three scope tokens; anything else is `None` (closed lookup,
    /// zero semantics). Used only by the strict script parser in the shell path.
    pub fn parse(token: &str) -> Option<TurnScope> {
        match token {
            "whole_vault" => Some(TurnScope::WholeVault),
            "prior_answer" => Some(TurnScope::PriorAnswer),
            "conversation_so_far" => Some(TurnScope::ConversationSoFar),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// One turn: a question plus the DECLARED scope it may look in. Never free text the
/// engine interprets — the scope carries all routing power, the words carry none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationTurnInput {
    pub question: String,
    pub scope: TurnScope,
}

impl ConversationTurnInput {
    pub fn new(question: impl Into<String>, scope: TurnScope) -> Self {
        ConversationTurnInput {
            question: question.into(),
            scope,
        }
    }
}

/// Closed-gate config: any true flag refuses before any conversation work happens.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ConverseConfig {
    pub uses_model: bool,
    pub uses_training: bool,
}

impl ConverseConfig {
    pub fn default_config() -> Self {
        ConverseConfig {
            uses_model: CONVERSE_USES_MODEL,
            uses_training: CONVERSE_USES_MODEL,
        }
    }
}

// ---------------------------------------------------------------------------
// Decisions + refusals
// ---------------------------------------------------------------------------

/// The two terminal decisions of a whole conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConversationDecision {
    /// The script ran to the end (individual turns may still have refused honestly).
    ConversationCompleted,
    /// A preflight guard refused the whole run before any turn executed.
    ConversationRefused,
}

impl ConversationDecision {
    pub fn slug(self) -> &'static str {
        match self {
            ConversationDecision::ConversationCompleted => "conversation_completed",
            ConversationDecision::ConversationRefused => "conversation_refused",
        }
    }
}

/// The two terminal decisions of a single turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConversationTurnDecision {
    TurnAnswered,
    TurnRefused,
}

impl ConversationTurnDecision {
    pub fn slug(self) -> &'static str {
        match self {
            ConversationTurnDecision::TurnAnswered => "turn_answered",
            ConversationTurnDecision::TurnRefused => "turn_refused",
        }
    }
}

/// Every reason a conversation or a turn can refuse. Closed enum — fail closed.
/// Each variant is CONSTRUCTED in a reachable production OR matrix path (the A3
/// fail-closed-debris law). `SerializedTranscriptTamper` and `VaultBindingMismatch`
/// are produced by PURE functions the matrix constructs (not by the shell verify
/// path, which returns [`ConverseError::ReplayMismatch`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ConverseRefusal {
    ModelSignalDetected,
    TrainingSignalDetected,
    EmptyVault,
    DuplicateVaultDocName,
    EmptyConversation,
    TooManyTurns,
    QuestionTooLong,
    ScriptParseRefused,
    NoPriorAnswer,
    NoConversationContext,
    QueryFlowRefused,
    TurnChainTamper,
    TurnReorder,
    TurnDeletion,
    TurnChainBreak,
    VaultBindingMismatch,
    SerializedTranscriptTamper,
}

impl ConverseRefusal {
    pub const ALL: [ConverseRefusal; 17] = [
        ConverseRefusal::ModelSignalDetected,
        ConverseRefusal::TrainingSignalDetected,
        ConverseRefusal::EmptyVault,
        ConverseRefusal::DuplicateVaultDocName,
        ConverseRefusal::EmptyConversation,
        ConverseRefusal::TooManyTurns,
        ConverseRefusal::QuestionTooLong,
        ConverseRefusal::ScriptParseRefused,
        ConverseRefusal::NoPriorAnswer,
        ConverseRefusal::NoConversationContext,
        ConverseRefusal::QueryFlowRefused,
        ConverseRefusal::TurnChainTamper,
        ConverseRefusal::TurnReorder,
        ConverseRefusal::TurnDeletion,
        ConverseRefusal::TurnChainBreak,
        ConverseRefusal::VaultBindingMismatch,
        ConverseRefusal::SerializedTranscriptTamper,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            ConverseRefusal::ModelSignalDetected => "model_signal_detected_refused",
            ConverseRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            ConverseRefusal::EmptyVault => "empty_vault_refused",
            ConverseRefusal::DuplicateVaultDocName => "duplicate_vault_doc_name_refused",
            ConverseRefusal::EmptyConversation => "empty_conversation_refused",
            ConverseRefusal::TooManyTurns => "too_many_turns_refused",
            ConverseRefusal::QuestionTooLong => "question_too_long_refused",
            ConverseRefusal::ScriptParseRefused => "script_parse_refused",
            ConverseRefusal::NoPriorAnswer => "no_prior_answer_refused",
            ConverseRefusal::NoConversationContext => "no_conversation_context_refused",
            ConverseRefusal::QueryFlowRefused => "query_flow_refused",
            ConverseRefusal::TurnChainTamper => "turn_chain_tamper_refused",
            ConverseRefusal::TurnReorder => "turn_reorder_refused",
            ConverseRefusal::TurnDeletion => "turn_deletion_refused",
            ConverseRefusal::TurnChainBreak => "turn_chain_break_refused",
            ConverseRefusal::VaultBindingMismatch => "vault_binding_mismatch_refused",
            ConverseRefusal::SerializedTranscriptTamper => "serialized_transcript_tamper_refused",
        }
    }
}

/// Re-derivation failure for a serialized transcript/matrix (never trusted off-wire).
/// Distinct from [`ConverseRefusal`]: the byte-compare verify path returns this ERROR,
/// while the matrix CONSTRUCTS the `SerializedTranscriptTamper` refusal by flipping a
/// byte and re-deriving (the QFLOW / LEARNER-MEMORY-1 precedent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConverseError {
    ReplayMismatch,
}

// ---------------------------------------------------------------------------
// Report objects — Serialize but NEVER Deserialize
// ---------------------------------------------------------------------------

/// One source-linked verified span, lifted VERBATIM from the QFLOW packet. `span_id`
/// is provenance only — it is NEVER a context-carry key (it is positional and rebased
/// on every scoped corpus rebuild; only `document_name` is stable identity).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TurnSource {
    pub document_name: String,
    pub span_id: u64,
    pub verified_text: String,
}

/// The verbatim answer of one answered turn: the QFLOW answer text plus its sources.
/// `answer_text` is the frozen verifier's verbatim join of the verified span texts —
/// CONVERSE composes nothing.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TurnAnswerSummary {
    pub answer_text: String,
    pub sources: Vec<TurnSource>,
}

/// One turn's replayable record. Folded into the hash chain. Carries the verified
/// packet summary on the answered path OR a typed refusal on the refused path — never
/// both, never neither.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConversationTurnRecord {
    pub schema: String,
    pub seq: u64,
    pub prev_turn_hash: u64,
    pub scope: TurnScope,
    pub question: String,
    pub resolved_doc_count: usize,
    /// FNV over the resolved document NAMES in vault order — binds the carried
    /// context into the turn hash so a focus drift moves the chain.
    pub resolved_focus_digest: u64,
    pub decision: ConversationTurnDecision,
    pub qflow_receipt_hash: u64,
    pub answer_hash: u64,
    pub evidence_item_count: usize,
    pub answer_summary: Option<TurnAnswerSummary>,
    pub refusal: Option<ConverseRefusal>,
    pub qflow_refusal: Option<String>,
    pub turn_hash: u64,
}

/// Inert forbidden-behavior flags, every one sourced from `CONVERSE_USES_MODEL`.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ConverseBoundary {
    pub is_model: bool,
    pub trains: bool,
    pub generates_prose: bool,
    pub infers_scope_from_words: bool,
    pub resolves_references_by_meaning: bool,
    pub carries_positional_id_key: bool,
    pub falls_back_on_empty_focus: bool,
    pub mutates_vault_across_turns: bool,
    pub adds_grounding_authority: bool,
    pub retags_v01: bool,
}

impl ConverseBoundary {
    fn inert() -> Self {
        ConverseBoundary {
            is_model: CONVERSE_USES_MODEL,
            trains: CONVERSE_USES_MODEL,
            generates_prose: CONVERSE_USES_MODEL,
            infers_scope_from_words: CONVERSE_USES_MODEL,
            resolves_references_by_meaning: CONVERSE_USES_MODEL,
            carries_positional_id_key: CONVERSE_USES_MODEL,
            falls_back_on_empty_focus: CONVERSE_USES_MODEL,
            mutates_vault_across_turns: CONVERSE_USES_MODEL,
            adds_grounding_authority: CONVERSE_USES_MODEL,
            retags_v01: CONVERSE_USES_MODEL,
        }
    }

    fn all_inert(&self) -> bool {
        !(self.is_model
            || self.trains
            || self.generates_prose
            || self.infers_scope_from_words
            || self.resolves_references_by_meaning
            || self.carries_positional_id_key
            || self.falls_back_on_empty_focus
            || self.mutates_vault_across_turns
            || self.adds_grounding_authority
            || self.retags_v01)
    }
}

/// The whole conversation: the vault fingerprint, the config, the hash-linked turns,
/// and the terminal decision. A run mixing answered and refused turns still COMPLETES;
/// only a preflight guard sets the top-level `refusal` with zero turns.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConversationTranscript {
    pub schema: String,
    pub vault_snapshot_hash: u64,
    pub config: ConverseConfig,
    pub turn_count: usize,
    pub head_hash: u64,
    pub turns: Vec<ConversationTurnRecord>,
    pub decision: ConversationDecision,
    pub refusal: Option<ConverseRefusal>,
    pub boundary: ConverseBoundary,
    pub boundary_all_inert: bool,
    pub transcript_hash: u64,
}

// `ConverseConfig` needs value equality for transcript `PartialEq`/`Eq`.
impl PartialEq for ConverseConfig {
    fn eq(&self, other: &Self) -> bool {
        self.uses_model == other.uses_model && self.uses_training == other.uses_training
    }
}
impl Eq for ConverseConfig {}

// ---------------------------------------------------------------------------
// Carried context — the CRUX, name-only, zero semantics
// ---------------------------------------------------------------------------

/// The conversation's carried focus: document NAMES only, never ids or spans.
/// `last_verified` = the immediately-preceding verified turn's cited doc names (vault
/// order); `union_seen` = the vault-order union of every verified turn's cited names.
/// Threaded through the fold; NOT serialized (it is derivation state, re-derived).
#[derive(Debug, Clone, Default)]
struct ConversationFocus {
    last_verified: Vec<String>,
    union_seen: Vec<String>,
}

// ---------------------------------------------------------------------------
// Hashing (deterministic; FNV-1a; integer only — the LEARNER-MEMORY-1 constants)
// ---------------------------------------------------------------------------

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

/// Fingerprint the whole vault: doc count, then each doc's NAME and CONTENT (with
/// their lengths, so a boundary shift cannot collide). Bound into the genesis head so
/// a conversation is pinned to the exact vault it ran over — a same-name content edit
/// is caught by [`transcript_binds_vault`] (`VaultBindingMismatch`), not merely by the
/// downstream byte compare.
fn vault_snapshot_hash(vault: &[(String, String)]) -> u64 {
    let mut h = fnv_mix(0xcbf2_9ce4_8422_2325, SCHEMA_TRANSCRIPT.as_bytes());
    h = fnv_u64(h, vault.len() as u64);
    for (name, content) in vault {
        h = fnv_u64(h, name.len() as u64);
        h = fnv_mix(h, name.as_bytes());
        h = fnv_u64(h, content.len() as u64);
        h = fnv_mix(h, content.as_bytes());
    }
    h
}

/// The head hash of a conversation with no turns yet — folds the schema and the vault
/// fingerprint, so turn 1 is already bound to its vault.
fn genesis_hash(vault_snapshot_hash: u64) -> u64 {
    let h = fnv_mix(0xcbf2_9ce4_8422_2325, SCHEMA_TRANSCRIPT.as_bytes());
    fnv_u64(h, vault_snapshot_hash)
}

/// FNV over resolved document NAMES in vault order (the carried-context digest).
fn focus_digest(names: &[String]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_u64(h, names.len() as u64);
    for name in names {
        h = fnv_u64(h, name.len() as u64);
        h = fnv_mix(h, name.as_bytes());
    }
    h
}

fn fold_turn_hash(record: &ConversationTurnRecord) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = fnv_mix(h, record.schema.as_bytes());
    h = fnv_u64(h, record.seq);
    h = fnv_u64(h, record.prev_turn_hash);
    h = fnv_mix(h, record.scope.slug().as_bytes());
    h = fnv_u64(h, record.question.len() as u64);
    h = fnv_mix(h, record.question.as_bytes());
    h = fnv_u64(h, record.resolved_doc_count as u64);
    h = fnv_u64(h, record.resolved_focus_digest);
    h = fnv_mix(h, record.decision.slug().as_bytes());
    h = fnv_u64(h, record.qflow_receipt_hash);
    h = fnv_u64(h, record.answer_hash);
    h = fnv_u64(h, record.evidence_item_count as u64);
    match &record.answer_summary {
        Some(summary) => {
            h = fnv_u64(h, 1);
            h = fnv_u64(h, summary.answer_text.len() as u64);
            h = fnv_mix(h, summary.answer_text.as_bytes());
            h = fnv_u64(h, summary.sources.len() as u64);
            for source in &summary.sources {
                h = fnv_u64(h, source.document_name.len() as u64);
                h = fnv_mix(h, source.document_name.as_bytes());
                h = fnv_u64(h, source.span_id);
                h = fnv_u64(h, source.verified_text.len() as u64);
                h = fnv_mix(h, source.verified_text.as_bytes());
            }
        }
        None => {
            h = fnv_u64(h, 0);
        }
    }
    h = fnv_mix(
        h,
        record
            .refusal
            .map(|r| r.slug())
            .unwrap_or("none")
            .as_bytes(),
    );
    h = fnv_mix(
        h,
        record.qflow_refusal.as_deref().unwrap_or("none").as_bytes(),
    );
    h
}

fn fold_transcript_hash(
    vault_snapshot_hash: u64,
    config: &ConverseConfig,
    turn_count: usize,
    head_hash: u64,
    decision: ConversationDecision,
    refusal: Option<ConverseRefusal>,
) -> u64 {
    let mut h = fnv_mix(0xcbf2_9ce4_8422_2325, SCHEMA_MATRIX.as_bytes());
    h = fnv_u64(h, vault_snapshot_hash);
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, turn_count as u64);
    h = fnv_u64(h, head_hash);
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

// ---------------------------------------------------------------------------
// Scope resolution (deterministic, vault-order-stable, zero semantics)
// ---------------------------------------------------------------------------

/// Filter the vault to the documents named in `names`, ALWAYS in vault order — so the
/// result is independent of `names`' order. Name identity only.
fn subset_by_names(vault: &[(String, String)], names: &[String]) -> Vec<(String, String)> {
    vault
        .iter()
        .filter(|(name, _)| names.iter().any(|wanted| wanted == name))
        .cloned()
        .collect()
}

/// Resolve a turn's DECLARED scope to a document sub-slice, or refuse. An empty
/// prior-answer / conversation-so-far focus REFUSES (it never silently widens to the
/// whole vault — that would be guessing).
fn resolve_scope(
    vault: &[(String, String)],
    scope: TurnScope,
    focus: &ConversationFocus,
) -> Result<Vec<(String, String)>, ConverseRefusal> {
    match scope {
        TurnScope::WholeVault => Ok(vault.to_vec()),
        TurnScope::PriorAnswer => {
            if focus.last_verified.is_empty() {
                Err(ConverseRefusal::NoPriorAnswer)
            } else {
                Ok(subset_by_names(vault, &focus.last_verified))
            }
        }
        TurnScope::ConversationSoFar => {
            if focus.union_seen.is_empty() {
                Err(ConverseRefusal::NoConversationContext)
            } else {
                Ok(subset_by_names(vault, &focus.union_seen))
            }
        }
    }
}

/// The distinct document names cited by a verified packet, in VAULT order.
fn packet_doc_names(vault: &[(String, String)], packet: &VerifiedEvidencePacket) -> Vec<String> {
    vault
        .iter()
        .map(|(name, _)| name)
        .filter(|name| packet.items.iter().any(|item| &item.document_name == *name))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// The conversation
// ---------------------------------------------------------------------------

fn vault_has_duplicate_names(vault: &[(String, String)]) -> bool {
    let mut seen: Vec<&str> = Vec::new();
    for (name, _) in vault {
        if seen.contains(&name.as_str()) {
            return true;
        }
        seen.push(name.as_str());
    }
    false
}

fn preflight_refusal(
    vault: &[(String, String)],
    script: &[ConversationTurnInput],
    config: ConverseConfig,
) -> Option<ConverseRefusal> {
    if config.uses_model {
        return Some(ConverseRefusal::ModelSignalDetected);
    }
    if config.uses_training {
        return Some(ConverseRefusal::TrainingSignalDetected);
    }
    if vault.is_empty() {
        return Some(ConverseRefusal::EmptyVault);
    }
    if vault_has_duplicate_names(vault) {
        return Some(ConverseRefusal::DuplicateVaultDocName);
    }
    if script.is_empty() {
        return Some(ConverseRefusal::EmptyConversation);
    }
    if script.len() > MAX_TURNS {
        return Some(ConverseRefusal::TooManyTurns);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn build_turn(
    seq: u64,
    prev_turn_hash: u64,
    scope: TurnScope,
    question: &str,
    resolved_doc_count: usize,
    resolved_focus_digest: u64,
    decision: ConversationTurnDecision,
    qflow_receipt_hash: u64,
    answer_hash: u64,
    evidence_item_count: usize,
    answer_summary: Option<TurnAnswerSummary>,
    refusal: Option<ConverseRefusal>,
    qflow_refusal: Option<String>,
) -> ConversationTurnRecord {
    let mut record = ConversationTurnRecord {
        schema: SCHEMA_TURN.to_string(),
        seq,
        prev_turn_hash,
        scope,
        question: question.to_string(),
        resolved_doc_count,
        resolved_focus_digest,
        decision,
        qflow_receipt_hash,
        answer_hash,
        evidence_item_count,
        answer_summary,
        refusal,
        qflow_refusal,
        turn_hash: 0,
    };
    record.turn_hash = fold_turn_hash(&record);
    record
}

fn assemble_transcript(
    vault_snapshot_hash: u64,
    config: ConverseConfig,
    turns: Vec<ConversationTurnRecord>,
    decision: ConversationDecision,
    refusal: Option<ConverseRefusal>,
) -> ConversationTranscript {
    let head_hash = turns
        .last()
        .map(|turn| turn.turn_hash)
        .unwrap_or_else(|| genesis_hash(vault_snapshot_hash));
    let boundary = ConverseBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    let turn_count = turns.len();
    let transcript_hash = fold_transcript_hash(
        vault_snapshot_hash,
        &config,
        turn_count,
        head_hash,
        decision,
        refusal,
    );
    ConversationTranscript {
        schema: SCHEMA_TRANSCRIPT.to_string(),
        vault_snapshot_hash,
        config,
        turn_count,
        head_hash,
        turns,
        decision,
        refusal,
        boundary,
        boundary_all_inert,
        transcript_hash,
    }
}

/// Run a whole conversation over a FIXED `vault` (`(name, raw_text)`) and a typed
/// `script`. Preflight guards refuse the whole run before any turn; otherwise each
/// turn resolves its declared scope and calls the FROZEN answerer, carrying context
/// forward as document names. Deterministic ⇒ replayable.
pub fn run_conversation(
    vault: &[(String, String)],
    script: &[ConversationTurnInput],
    config: ConverseConfig,
) -> ConversationTranscript {
    let snapshot = vault_snapshot_hash(vault);
    if let Some(refusal) = preflight_refusal(vault, script, config) {
        return assemble_transcript(
            snapshot,
            config,
            Vec::new(),
            ConversationDecision::ConversationRefused,
            Some(refusal),
        );
    }

    let mut focus = ConversationFocus::default();
    let mut turns: Vec<ConversationTurnRecord> = Vec::with_capacity(script.len());
    let mut prev_hash = genesis_hash(snapshot);

    for (index, input) in script.iter().enumerate() {
        let seq = index as u64 + 1;

        // Per-turn input guard: an oversized question is refused before the answerer.
        if input.question.len() > MAX_QUESTION_LEN {
            let record = build_turn(
                seq,
                prev_hash,
                input.scope,
                &input.question,
                0,
                focus_digest(&[]),
                ConversationTurnDecision::TurnRefused,
                0,
                0,
                0,
                None,
                Some(ConverseRefusal::QuestionTooLong),
                None,
            );
            prev_hash = record.turn_hash;
            turns.push(record);
            continue;
        }

        // Resolve the DECLARED scope to a document slice, or refuse honestly.
        let subset = match resolve_scope(vault, input.scope, &focus) {
            Ok(subset) => subset,
            Err(refusal) => {
                let record = build_turn(
                    seq,
                    prev_hash,
                    input.scope,
                    &input.question,
                    0,
                    focus_digest(&[]),
                    ConversationTurnDecision::TurnRefused,
                    0,
                    0,
                    0,
                    None,
                    Some(refusal),
                    None,
                );
                prev_hash = record.turn_hash;
                turns.push(record);
                continue;
            }
        };

        let resolved_names: Vec<String> = subset.iter().map(|(name, _)| name.clone()).collect();
        let resolved_doc_count = subset.len();
        let resolved_focus_digest = focus_digest(&resolved_names);

        // Call the FROZEN answerer. It owns grounding; CONVERSE records the outcome.
        let flow = run_query_default(&subset, &input.question);
        let record = match flow.packet.as_ref() {
            Some(packet) => {
                let sources: Vec<TurnSource> = packet
                    .items
                    .iter()
                    .map(|item| TurnSource {
                        document_name: item.document_name.clone(),
                        span_id: item.span_id,
                        verified_text: item.verified_text.clone(),
                    })
                    .collect();
                let summary = TurnAnswerSummary {
                    answer_text: packet.answer_text.clone(),
                    sources,
                };
                // Carry context forward: name-only, vault order.
                let cited = packet_doc_names(vault, packet);
                for name in &cited {
                    if !focus.union_seen.iter().any(|seen| seen == name) {
                        focus.union_seen.push(name.clone());
                    }
                }
                focus.last_verified = cited;
                build_turn(
                    seq,
                    prev_hash,
                    input.scope,
                    &input.question,
                    resolved_doc_count,
                    resolved_focus_digest,
                    ConversationTurnDecision::TurnAnswered,
                    flow.receipt.receipt_hash,
                    packet.answer_hash,
                    packet.items.len(),
                    Some(summary),
                    None,
                    None,
                )
            }
            None => {
                // A refused turn leaves the focus UNCHANGED (a follow-up still points
                // at the last thing actually answered).
                build_turn(
                    seq,
                    prev_hash,
                    input.scope,
                    &input.question,
                    resolved_doc_count,
                    resolved_focus_digest,
                    ConversationTurnDecision::TurnRefused,
                    flow.receipt.receipt_hash,
                    0,
                    0,
                    None,
                    Some(ConverseRefusal::QueryFlowRefused),
                    flow.refusal.map(|r| r.slug().to_string()),
                )
            }
        };
        prev_hash = record.turn_hash;
        turns.push(record);
    }

    assemble_transcript(
        snapshot,
        config,
        turns,
        ConversationDecision::ConversationCompleted,
        None,
    )
}

/// Run a conversation with the default (all-inert) config.
pub fn run_conversation_default(
    vault: &[(String, String)],
    script: &[ConversationTurnInput],
) -> ConversationTranscript {
    run_conversation(vault, script, ConverseConfig::default_config())
}

// ---------------------------------------------------------------------------
// Strict script parsing (shell path) — Deserialize-free
// ---------------------------------------------------------------------------

/// Parse a plain-text script into typed turns. One line per turn: `SCOPE<TAB>question`.
/// The scope token must be one of the three [`TurnScope`] tokens; the question must be
/// non-empty. A malformed line (no tab, unknown scope token, or empty question) is
/// `ScriptParseRefused`. Blank lines are skipped. The typed turns are CONSTRUCTED
/// here — never serde-parsed. (`MAX_TURNS` is enforced downstream by the conversation
/// preflight, so `too_many_turns` stays a distinct refusal from a malformed line.)
pub fn parse_script(text: &str) -> Result<Vec<ConversationTurnInput>, ConverseRefusal> {
    let mut turns = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.trim().is_empty() {
            continue;
        }
        let (scope_token, question) = match line.split_once('\t') {
            Some(pair) => pair,
            None => return Err(ConverseRefusal::ScriptParseRefused),
        };
        let scope = match TurnScope::parse(scope_token) {
            Some(scope) => scope,
            None => return Err(ConverseRefusal::ScriptParseRefused),
        };
        let question = question.trim();
        if question.is_empty() {
            return Err(ConverseRefusal::ScriptParseRefused);
        }
        turns.push(ConversationTurnInput::new(question, scope));
    }
    Ok(turns)
}

/// Run a conversation from raw script TEXT over a vault. Strict-parses the text (a
/// malformed script becomes a preflight `ScriptParseRefused` transcript), then runs
/// the pure engine. The shell hands untrusted `--script` bytes here.
pub fn converse_run_from_text(
    text: &str,
    vault: &[(String, String)],
    config: ConverseConfig,
) -> ConversationTranscript {
    match parse_script(text) {
        Ok(script) => run_conversation(vault, &script, config),
        Err(refusal) => assemble_transcript(
            vault_snapshot_hash(vault),
            config,
            Vec::new(),
            ConversationDecision::ConversationRefused,
            Some(refusal),
        ),
    }
}

// ---------------------------------------------------------------------------
// Structural guards on a supplied transcript (never trusted off-wire)
// ---------------------------------------------------------------------------

/// Walk a supplied transcript's turn chain and refuse the FIRST structural violation
/// with a distinct refusal. Check order is load-bearing (the LEARNER-MEMORY-1 walk
/// MINUS the duplicate step — repeat questions are LEGITIMATE and `seq` makes every
/// turn hash distinct, so a duplicate-turn refusal would be dead/wrong A3 debris):
/// per-turn recompute (tamper) → seq monotonicity (reorder) → seq continuity
/// (deletion) → prev pointer (chain break) → root count/head (tamper).
pub fn conversation_turns_are_chain_linked(
    transcript: &ConversationTranscript,
) -> Option<ConverseRefusal> {
    for turn in &transcript.turns {
        if fold_turn_hash(turn) != turn.turn_hash {
            return Some(ConverseRefusal::TurnChainTamper);
        }
    }
    // Reorder is judged over the WHOLE seq vector before continuity: a swapped pair
    // ([2, 1]) reads as a gap to a single forward walk and would mislabel a deletion.
    let seqs: Vec<u64> = transcript.turns.iter().map(|turn| turn.seq).collect();
    if seqs.windows(2).any(|pair| pair[1] <= pair[0]) {
        return Some(ConverseRefusal::TurnReorder);
    }
    for (index, seq) in seqs.iter().enumerate() {
        if *seq != index as u64 + 1 {
            return Some(ConverseRefusal::TurnDeletion);
        }
    }
    let mut expected_prev = genesis_hash(transcript.vault_snapshot_hash);
    for turn in &transcript.turns {
        if turn.prev_turn_hash != expected_prev {
            return Some(ConverseRefusal::TurnChainBreak);
        }
        expected_prev = turn.turn_hash;
    }
    if transcript.turn_count != transcript.turns.len() || transcript.head_hash != expected_prev {
        return Some(ConverseRefusal::TurnChainTamper);
    }
    None
}

/// Refuse a transcript whose bound vault fingerprint disagrees with the supplied
/// vault (tampered, stale, or a different corpus). Pure — the matrix constructs this;
/// the shell verify path surfaces it.
pub fn transcript_binds_vault(
    transcript: &ConversationTranscript,
    vault: &[(String, String)],
) -> Option<ConverseRefusal> {
    if transcript.vault_snapshot_hash != vault_snapshot_hash(vault) {
        Some(ConverseRefusal::VaultBindingMismatch)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// The canonical demo conversation
// ---------------------------------------------------------------------------

/// The baked demo vault: three short local notes with unique names.
pub fn converse_demo_vault() -> Vec<(String, String)> {
    vec![
        (
            "bridge.txt".to_string(),
            "The bridge is open today. The status is green.".to_string(),
        ),
        (
            "reactor.txt".to_string(),
            "The reactor hums quietly. Coolant is low.".to_string(),
        ),
        (
            "weather.txt".to_string(),
            "The weather looks calm and clear.".to_string(),
        ),
    ]
}

/// The baked demo script: a WholeVault answer, a second WholeVault answer, a
/// PriorAnswer follow-up that narrows to the last answer's doc, a ConversationSoFar
/// turn that widens to every answered doc, and a WholeVault turn that cannot ground
/// (an honest QueryFlowRefused).
pub fn converse_demo_script() -> Vec<ConversationTurnInput> {
    vec![
        ConversationTurnInput::new("reactor", TurnScope::WholeVault),
        ConversationTurnInput::new("bridge", TurnScope::WholeVault),
        ConversationTurnInput::new("status", TurnScope::PriorAnswer),
        ConversationTurnInput::new("coolant", TurnScope::ConversationSoFar),
        ConversationTurnInput::new("xylophone", TurnScope::WholeVault),
    ]
}

pub fn converse_demo() -> ConversationTranscript {
    run_conversation_default(&converse_demo_vault(), &converse_demo_script())
}

pub fn converse_demo_json() -> String {
    serde_json::to_string_pretty(&converse_demo()).expect("converse demo serializes")
}

/// Serialize any transcript for the CLI shell (the `converse-run` surface). Serialize
/// only — a stored transcript is re-derived from the same vault + script and
/// byte-compared, never parsed back.
pub fn converse_transcript_json(transcript: &ConversationTranscript) -> String {
    serde_json::to_string_pretty(transcript).expect("converse transcript serializes")
}

/// Re-derive the canonical demo transcript and byte-compare; a tampered/foreign
/// transcript is refused (Serialize only, no Deserialize).
pub fn verify_converse_demo_json(candidate: &str) -> Result<(), ConverseError> {
    if candidate == converse_demo_json() {
        Ok(())
    } else {
        Err(ConverseError::ReplayMismatch)
    }
}

// ---------------------------------------------------------------------------
// Coverage matrix (every refusal constructed — the A3 law)
// ---------------------------------------------------------------------------

pub const CONVERSE_SCENARIO_COUNT: usize = 19;
pub const CONVERSE_SCENARIO_NAMES: [&str; CONVERSE_SCENARIO_COUNT] = [
    "conversation_completed",
    "model_signal_refused",
    "training_signal_refused",
    "empty_vault_refused",
    "duplicate_vault_doc_name_refused",
    "empty_conversation_refused",
    "too_many_turns_refused",
    "question_too_long_refused",
    "script_parse_refused",
    "no_prior_answer_refused",
    "no_conversation_context_refused",
    "query_flow_refused",
    "turn_chain_tamper_refused",
    "turn_root_tamper_refused",
    "turn_reorder_refused",
    "turn_deletion_refused",
    "turn_chain_break_refused",
    "vault_binding_mismatch_refused",
    "serialized_transcript_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct ConverseCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub turn_count: usize,
    pub answered_turns: usize,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConverseMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<ConverseCell>,
    pub completed_count: usize,
    pub refused_count: usize,
    pub boundary: ConverseBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_transcript(scenario: &str, transcript: &ConversationTranscript) -> ConverseCell {
    let answered_turns = transcript
        .turns
        .iter()
        .filter(|turn| turn.decision == ConversationTurnDecision::TurnAnswered)
        .count();
    // The outcome is the CONVERSATION-level decision: a preflight guard refuses the
    // whole run; otherwise it completed (a per-turn refusal is honest, not fatal). The
    // refusal COLUMN still records a representative refusal — the preflight one, or the
    // first refused turn's — so a per-turn-refusal scenario populates the A3 coverage.
    let outcome = if transcript.refusal.is_some() {
        "conversation_refused".to_string()
    } else {
        transcript.decision.slug().to_string()
    };
    let refusal = transcript
        .refusal
        .map(|r| r.slug().to_string())
        .or_else(|| {
            transcript
                .turns
                .iter()
                .find(|turn| turn.decision == ConversationTurnDecision::TurnRefused)
                .and_then(|turn| turn.refusal.map(|r| r.slug().to_string()))
        });
    ConverseCell {
        scenario: scenario.to_string(),
        outcome,
        refusal,
        turn_count: transcript.turn_count,
        answered_turns,
        boundary_all_inert: transcript.boundary_all_inert,
    }
}

fn cell_from_guard(scenario: &str, refusal: Option<ConverseRefusal>) -> ConverseCell {
    ConverseCell {
        scenario: scenario.to_string(),
        outcome: match refusal {
            Some(_) => "conversation_refused".to_string(),
            None => "violation_missed".to_string(),
        },
        refusal: refusal.map(|r| r.slug().to_string()),
        turn_count: 0,
        answered_turns: 0,
        boundary_all_inert: ConverseBoundary::inert().all_inert(),
    }
}

fn cell_for(scenario: &str) -> ConverseCell {
    match scenario {
        "conversation_completed" => cell_from_transcript(scenario, &converse_demo()),
        "model_signal_refused" => {
            let mut config = ConverseConfig::default_config();
            config.uses_model = true;
            let transcript =
                run_conversation(&converse_demo_vault(), &converse_demo_script(), config);
            cell_from_transcript(scenario, &transcript)
        }
        "training_signal_refused" => {
            let mut config = ConverseConfig::default_config();
            config.uses_training = true;
            let transcript =
                run_conversation(&converse_demo_vault(), &converse_demo_script(), config);
            cell_from_transcript(scenario, &transcript)
        }
        "empty_vault_refused" => {
            let vault: Vec<(String, String)> = Vec::new();
            let transcript = run_conversation_default(&vault, &converse_demo_script());
            cell_from_transcript(scenario, &transcript)
        }
        "duplicate_vault_doc_name_refused" => {
            let vault = vec![
                ("dup.txt".to_string(), "The bridge is open.".to_string()),
                ("dup.txt".to_string(), "The reactor is stable.".to_string()),
            ];
            let script = vec![ConversationTurnInput::new("bridge", TurnScope::WholeVault)];
            let transcript = run_conversation_default(&vault, &script);
            cell_from_transcript(scenario, &transcript)
        }
        "empty_conversation_refused" => {
            let script: Vec<ConversationTurnInput> = Vec::new();
            let transcript = run_conversation_default(&converse_demo_vault(), &script);
            cell_from_transcript(scenario, &transcript)
        }
        "too_many_turns_refused" => {
            let script: Vec<ConversationTurnInput> = (0..=MAX_TURNS)
                .map(|_| ConversationTurnInput::new("bridge", TurnScope::WholeVault))
                .collect();
            let transcript = run_conversation_default(&converse_demo_vault(), &script);
            cell_from_transcript(scenario, &transcript)
        }
        "question_too_long_refused" => {
            let long = "x".repeat(MAX_QUESTION_LEN + 1);
            let script = vec![ConversationTurnInput::new(long, TurnScope::WholeVault)];
            let transcript = run_conversation_default(&converse_demo_vault(), &script);
            cell_from_transcript(scenario, &transcript)
        }
        "script_parse_refused" => {
            // A line with no tab is malformed — the strict parser refuses it.
            let transcript = converse_run_from_text(
                "bridge is open",
                &converse_demo_vault(),
                ConverseConfig::default_config(),
            );
            cell_from_transcript(scenario, &transcript)
        }
        "no_prior_answer_refused" => {
            let script = vec![ConversationTurnInput::new("bridge", TurnScope::PriorAnswer)];
            let transcript = run_conversation_default(&converse_demo_vault(), &script);
            cell_from_transcript(scenario, &transcript)
        }
        "no_conversation_context_refused" => {
            let script = vec![ConversationTurnInput::new(
                "bridge",
                TurnScope::ConversationSoFar,
            )];
            let transcript = run_conversation_default(&converse_demo_vault(), &script);
            cell_from_transcript(scenario, &transcript)
        }
        "query_flow_refused" => {
            let script = vec![ConversationTurnInput::new(
                "xylophone",
                TurnScope::WholeVault,
            )];
            let transcript = run_conversation_default(&converse_demo_vault(), &script);
            cell_from_transcript(scenario, &transcript)
        }
        "turn_chain_tamper_refused" => {
            // Mutate a stored field without re-folding: per-turn recompute must mismatch.
            let mut transcript = converse_demo();
            transcript.turns[0].question = "tampered".to_string();
            cell_from_guard(scenario, conversation_turns_are_chain_linked(&transcript))
        }
        "turn_root_tamper_refused" => {
            // Forge the head hash: the root count/head check must mismatch (the SECOND
            // TurnChainTamper trigger site).
            let mut transcript = converse_demo();
            transcript.head_hash ^= 1;
            cell_from_guard(scenario, conversation_turns_are_chain_linked(&transcript))
        }
        "turn_reorder_refused" => {
            let mut transcript = converse_demo();
            transcript.turns.swap(0, 1);
            cell_from_guard(scenario, conversation_turns_are_chain_linked(&transcript))
        }
        "turn_deletion_refused" => {
            let mut transcript = converse_demo();
            transcript.turns.remove(0);
            transcript.turn_count -= 1;
            cell_from_guard(scenario, conversation_turns_are_chain_linked(&transcript))
        }
        "turn_chain_break_refused" => {
            // Forge the prev pointer AND re-fold the turn hash so recompute passes;
            // only the chain-link walk can catch this.
            let mut transcript = converse_demo();
            transcript.turns[1].prev_turn_hash = 0xdead_beef;
            transcript.turns[1].turn_hash = fold_turn_hash(&transcript.turns[1]);
            cell_from_guard(scenario, conversation_turns_are_chain_linked(&transcript))
        }
        "vault_binding_mismatch_refused" => {
            // Re-derive against a vault whose fingerprint differs (content edited).
            let transcript = converse_demo();
            let other = vec![
                (
                    "bridge.txt".to_string(),
                    "The bridge is CLOSED today. The status is red.".to_string(),
                ),
                (
                    "reactor.txt".to_string(),
                    "The reactor hums quietly. Coolant is low.".to_string(),
                ),
                (
                    "weather.txt".to_string(),
                    "The weather looks calm and clear.".to_string(),
                ),
            ];
            cell_from_guard(scenario, transcript_binds_vault(&transcript, &other))
        }
        "serialized_transcript_tamper_refused" => {
            // Serialize the real transcript, flip one byte, confirm the tamper is
            // detectable — constructing the refusal that names this scenario.
            let json = converse_demo_json();
            let refused = verify_converse_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(ConverseRefusal::SerializedTranscriptTamper)
            } else {
                None
            };
            ConverseCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused".to_string()
                } else {
                    "tamper_missed".to_string()
                },
                refusal: refusal.map(|r| r.slug().to_string()),
                turn_count: 0,
                answered_turns: 0,
                boundary_all_inert: ConverseBoundary::inert().all_inert(),
            }
        }
        other => ConverseCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            turn_count: 0,
            answered_turns: 0,
            boundary_all_inert: false,
        },
    }
}

pub fn converse_matrix() -> ConverseMatrix {
    let cells: Vec<ConverseCell> = CONVERSE_SCENARIO_NAMES
        .iter()
        .map(|n| cell_for(n))
        .collect();
    let completed_count = cells
        .iter()
        .filter(|cell| cell.outcome == "conversation_completed")
        .count();
    let refused_count = cells.len() - completed_count;
    let boundary = ConverseBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    ConverseMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        completed_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn converse_matrix_json() -> String {
    serde_json::to_string_pretty(&converse_matrix()).expect("converse matrix serializes")
}

pub fn verify_converse_matrix_json(candidate: &str) -> Result<(), ConverseError> {
    if candidate == converse_matrix_json() {
        Ok(())
    } else {
        Err(ConverseError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- criterion 1: grounded-only answers --------------------------------

    #[test]
    fn answered_turns_carry_a_verified_packet_summary() {
        let transcript = converse_demo();
        assert_eq!(
            transcript.decision,
            ConversationDecision::ConversationCompleted
        );
        let answered: Vec<&ConversationTurnRecord> = transcript
            .turns
            .iter()
            .filter(|t| t.decision == ConversationTurnDecision::TurnAnswered)
            .collect();
        assert!(!answered.is_empty(), "demo must answer at least one turn");
        for turn in answered {
            let summary = turn.answer_summary.as_ref().expect("answered => summary");
            assert!(turn.evidence_item_count >= 1);
            assert_ne!(turn.qflow_receipt_hash, 0);
            assert_ne!(turn.answer_hash, 0);
            assert!(!summary.sources.is_empty());
            assert!(turn.refusal.is_none());
        }
    }

    // -- no invented words: answer_text is the verbatim join of the sources --

    #[test]
    fn answer_text_is_the_verbatim_join_of_its_sources() {
        let transcript = converse_demo();
        for turn in &transcript.turns {
            if let Some(summary) = &turn.answer_summary {
                let rebuilt = summary
                    .sources
                    .iter()
                    .map(|s| s.verified_text.clone())
                    .collect::<Vec<_>>()
                    .join(" ");
                assert_eq!(rebuilt, summary.answer_text);
            }
        }
    }

    #[test]
    fn source_text_is_verbatim_from_the_cited_vault_doc() {
        let vault = converse_demo_vault();
        let transcript = converse_demo();
        for turn in &transcript.turns {
            if let Some(summary) = &turn.answer_summary {
                for source in &summary.sources {
                    let (_, content) = vault
                        .iter()
                        .find(|(name, _)| name == &source.document_name)
                        .expect("source cites a real vault doc");
                    assert!(
                        content.contains(&source.verified_text),
                        "verified text must be a verbatim substring of its source doc"
                    );
                }
            }
        }
    }

    // -- criterion 2: honest refusal ---------------------------------------

    #[test]
    fn refused_turns_carry_no_answer_and_a_typed_refusal() {
        let transcript = converse_demo();
        for turn in &transcript.turns {
            if turn.decision == ConversationTurnDecision::TurnRefused {
                assert!(turn.answer_summary.is_none());
                assert_eq!(turn.answer_hash, 0);
                assert_eq!(turn.evidence_item_count, 0);
                assert!(turn.refusal.is_some());
            }
        }
    }

    #[test]
    fn ungroundable_turn_is_a_query_flow_refusal_with_propagated_slug() {
        let vault = converse_demo_vault();
        let script = vec![ConversationTurnInput::new(
            "xylophone",
            TurnScope::WholeVault,
        )];
        let transcript = run_conversation_default(&vault, &script);
        let turn = &transcript.turns[0];
        assert_eq!(turn.decision, ConversationTurnDecision::TurnRefused);
        assert_eq!(turn.refusal, Some(ConverseRefusal::QueryFlowRefused));
        assert!(turn.qflow_refusal.is_some(), "the QFLOW slug is propagated");
    }

    // -- criterion 3a: deterministic context carry -------------------------

    #[test]
    fn first_turn_prior_answer_refuses_no_prior_answer() {
        let vault = converse_demo_vault();
        let script = vec![ConversationTurnInput::new("bridge", TurnScope::PriorAnswer)];
        let transcript = run_conversation_default(&vault, &script);
        assert_eq!(
            transcript.turns[0].refusal,
            Some(ConverseRefusal::NoPriorAnswer)
        );
    }

    #[test]
    fn first_turn_conversation_so_far_refuses_no_context() {
        let vault = converse_demo_vault();
        let script = vec![ConversationTurnInput::new(
            "bridge",
            TurnScope::ConversationSoFar,
        )];
        let transcript = run_conversation_default(&vault, &script);
        assert_eq!(
            transcript.turns[0].refusal,
            Some(ConverseRefusal::NoConversationContext)
        );
    }

    #[test]
    fn prior_answer_resolves_to_exactly_the_last_answered_docs() {
        let vault = converse_demo_vault();
        // Turn 1 answers over the whole vault; turn 2 (PriorAnswer) must resolve to
        // exactly the doc-name set turn 1 cited — proven by the focus digest, not by
        // any word matching.
        let script = vec![
            ConversationTurnInput::new("bridge", TurnScope::WholeVault),
            ConversationTurnInput::new("status", TurnScope::PriorAnswer),
        ];
        let transcript = run_conversation_default(&vault, &script);
        let first = &transcript.turns[0];
        let first_names: Vec<String> = first
            .answer_summary
            .as_ref()
            .expect("turn 1 answered")
            .sources
            .iter()
            .map(|s| s.document_name.clone())
            // dedup to the vault-order distinct set
            .fold(Vec::new(), |mut acc, name| {
                if !acc.contains(&name) {
                    acc.push(name);
                }
                acc
            });
        let expected = super::focus_digest(&first_names);
        assert_eq!(transcript.turns[1].resolved_focus_digest, expected);
        assert_eq!(transcript.turns[1].resolved_doc_count, first_names.len());
    }

    // -- criterion 3b: scope is not inferred from words --------------------

    #[test]
    fn scope_routes_docs_not_the_question_words() {
        let vault = converse_demo_vault();
        // Same question word, different declared scope after a priming answer =>
        // different resolved doc slice. The words never change routing; the scope does.
        let prime = ConversationTurnInput::new("bridge", TurnScope::WholeVault);
        let whole = run_conversation_default(
            &vault,
            &[
                prime.clone(),
                ConversationTurnInput::new("status", TurnScope::WholeVault),
            ],
        );
        let prior = run_conversation_default(
            &vault,
            &[
                prime,
                ConversationTurnInput::new("status", TurnScope::PriorAnswer),
            ],
        );
        // WholeVault sees every doc; PriorAnswer sees only bridge.txt.
        assert_eq!(whole.turns[1].resolved_doc_count, vault.len());
        assert_eq!(prior.turns[1].resolved_doc_count, 1);
        assert_ne!(
            whole.turns[1].resolved_focus_digest,
            prior.turns[1].resolved_focus_digest
        );
    }

    // -- criterion 4: determinism ------------------------------------------

    #[test]
    fn same_input_same_transcript() {
        let a = converse_demo();
        let b = converse_demo();
        assert_eq!(a, b);
        assert_eq!(a.transcript_hash, b.transcript_hash);
        assert_eq!(converse_demo_json(), converse_demo_json());
    }

    // -- tamper A/B --------------------------------------------------------

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = converse_demo_json();
        assert!(verify_converse_demo_json(&json).is_ok());
        assert_eq!(
            verify_converse_demo_json(&flip_last_byte(&json)),
            Err(ConverseError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = converse_matrix_json();
        assert!(verify_converse_matrix_json(&json).is_ok());
        assert_eq!(
            verify_converse_matrix_json(&flip_last_byte(&json)),
            Err(ConverseError::ReplayMismatch)
        );
    }

    // -- chain integrity ---------------------------------------------------

    #[test]
    fn canonical_chain_links_and_head_matches() {
        let transcript = converse_demo();
        assert_eq!(conversation_turns_are_chain_linked(&transcript), None);
        assert_eq!(
            transcript.turns[0].prev_turn_hash,
            super::genesis_hash(transcript.vault_snapshot_hash)
        );
        for window in transcript.turns.windows(2) {
            assert_eq!(window[1].prev_turn_hash, window[0].turn_hash);
        }
        assert_eq!(
            transcript.head_hash,
            transcript.turns.last().unwrap().turn_hash
        );
    }

    #[test]
    fn each_chain_mutation_maps_to_its_distinct_refusal() {
        let mut tamper = converse_demo();
        tamper.turns[0].question = "tampered".to_string();
        assert_eq!(
            conversation_turns_are_chain_linked(&tamper),
            Some(ConverseRefusal::TurnChainTamper)
        );

        let mut root = converse_demo();
        root.head_hash ^= 1;
        assert_eq!(
            conversation_turns_are_chain_linked(&root),
            Some(ConverseRefusal::TurnChainTamper)
        );

        let mut reorder = converse_demo();
        reorder.turns.swap(0, 1);
        assert_eq!(
            conversation_turns_are_chain_linked(&reorder),
            Some(ConverseRefusal::TurnReorder)
        );

        let mut deletion = converse_demo();
        deletion.turns.remove(0);
        deletion.turn_count -= 1;
        assert_eq!(
            conversation_turns_are_chain_linked(&deletion),
            Some(ConverseRefusal::TurnDeletion)
        );

        let mut chain_break = converse_demo();
        chain_break.turns[1].prev_turn_hash = 0xdead_beef;
        chain_break.turns[1].turn_hash = fold_turn_hash(&chain_break.turns[1]);
        assert_eq!(
            conversation_turns_are_chain_linked(&chain_break),
            Some(ConverseRefusal::TurnChainBreak)
        );
    }

    #[test]
    fn vault_binding_mismatch_is_refused() {
        let transcript = converse_demo();
        assert_eq!(
            transcript_binds_vault(&transcript, &converse_demo_vault()),
            None
        );
        let mut other = converse_demo_vault();
        other[0].1 = "The bridge is CLOSED. The status is red.".to_string();
        assert_eq!(
            transcript_binds_vault(&transcript, &other),
            Some(ConverseRefusal::VaultBindingMismatch)
        );
    }

    // -- preflight gates ---------------------------------------------------

    #[test]
    fn model_and_training_signals_refuse_before_any_turn() {
        let mut model = ConverseConfig::default_config();
        model.uses_model = true;
        let t = run_conversation(&converse_demo_vault(), &converse_demo_script(), model);
        assert_eq!(t.refusal, Some(ConverseRefusal::ModelSignalDetected));
        assert!(t.turns.is_empty());

        let mut training = ConverseConfig::default_config();
        training.uses_training = true;
        let t = run_conversation(&converse_demo_vault(), &converse_demo_script(), training);
        assert_eq!(t.refusal, Some(ConverseRefusal::TrainingSignalDetected));
    }

    #[test]
    fn empty_vault_and_empty_conversation_and_too_many_turns_refuse() {
        let empty_vault: Vec<(String, String)> = Vec::new();
        assert_eq!(
            run_conversation_default(&empty_vault, &converse_demo_script()).refusal,
            Some(ConverseRefusal::EmptyVault)
        );
        let empty_script: Vec<ConversationTurnInput> = Vec::new();
        assert_eq!(
            run_conversation_default(&converse_demo_vault(), &empty_script).refusal,
            Some(ConverseRefusal::EmptyConversation)
        );
        let too_many: Vec<ConversationTurnInput> = (0..=MAX_TURNS)
            .map(|_| ConversationTurnInput::new("bridge", TurnScope::WholeVault))
            .collect();
        assert_eq!(
            run_conversation_default(&converse_demo_vault(), &too_many).refusal,
            Some(ConverseRefusal::TooManyTurns)
        );
    }

    #[test]
    fn duplicate_vault_doc_name_refuses() {
        let vault = vec![
            ("dup.txt".to_string(), "The bridge is open.".to_string()),
            ("dup.txt".to_string(), "The reactor is stable.".to_string()),
        ];
        let script = vec![ConversationTurnInput::new("bridge", TurnScope::WholeVault)];
        assert_eq!(
            run_conversation_default(&vault, &script).refusal,
            Some(ConverseRefusal::DuplicateVaultDocName)
        );
    }

    #[test]
    fn oversized_question_is_refused_per_turn() {
        let long = "x".repeat(MAX_QUESTION_LEN + 1);
        let script = vec![ConversationTurnInput::new(long, TurnScope::WholeVault)];
        let transcript = run_conversation_default(&converse_demo_vault(), &script);
        assert_eq!(
            transcript.decision,
            ConversationDecision::ConversationCompleted
        );
        assert_eq!(
            transcript.turns[0].refusal,
            Some(ConverseRefusal::QuestionTooLong)
        );
    }

    // -- strict script parser ----------------------------------------------

    #[test]
    fn parse_script_accepts_well_formed_lines() {
        let text = "whole_vault\treactor\nprior_answer\tstatus\nconversation_so_far\tcoolant\n";
        let script = parse_script(text).expect("well-formed script parses");
        assert_eq!(script.len(), 3);
        assert_eq!(script[0].scope, TurnScope::WholeVault);
        assert_eq!(script[1].scope, TurnScope::PriorAnswer);
        assert_eq!(script[2].scope, TurnScope::ConversationSoFar);
        assert_eq!(script[0].question, "reactor");
    }

    #[test]
    fn parse_script_refuses_malformed_lines() {
        assert_eq!(
            parse_script("no tab here"),
            Err(ConverseRefusal::ScriptParseRefused)
        );
        assert_eq!(
            parse_script("unknown_scope\tquestion"),
            Err(ConverseRefusal::ScriptParseRefused)
        );
        assert_eq!(
            parse_script("whole_vault\t   "),
            Err(ConverseRefusal::ScriptParseRefused)
        );
    }

    #[test]
    fn converse_run_from_text_refuses_a_malformed_script() {
        let transcript = converse_run_from_text(
            "not a scope line",
            &converse_demo_vault(),
            ConverseConfig::default_config(),
        );
        assert_eq!(
            transcript.refusal,
            Some(ConverseRefusal::ScriptParseRefused)
        );
        assert!(transcript.turns.is_empty());
    }

    #[test]
    fn converse_run_from_text_matches_the_typed_engine() {
        let text = "reactor\treactor\n"; // one WholeVault turn (scope token is "reactor"? no)
                                         // Build the same run two ways: from text and from typed inputs.
        let typed = run_conversation_default(
            &converse_demo_vault(),
            &[ConversationTurnInput::new("reactor", TurnScope::WholeVault)],
        );
        let from_text = converse_run_from_text(
            "whole_vault\treactor\n",
            &converse_demo_vault(),
            ConverseConfig::default_config(),
        );
        assert_eq!(typed, from_text);
        // (the first `text` binding documents that the scope token must be a real
        // scope, not a question word — parse would refuse "reactor" as a scope.)
        assert_eq!(parse_script(text), Err(ConverseRefusal::ScriptParseRefused));
    }

    // -- matrix / A3 coverage ----------------------------------------------

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = converse_matrix();
        assert_eq!(matrix.scenario_count, CONVERSE_SCENARIO_COUNT);
        // The clean showcase conversation completed (per-turn refusals inside a
        // completed run also read "conversation_completed" — the conversation still
        // ran to the end).
        let demo_cell = matrix
            .cells
            .iter()
            .find(|cell| cell.scenario == "conversation_completed")
            .expect("demo scenario present");
        assert_eq!(demo_cell.outcome, "conversation_completed");
        assert!(demo_cell.answered_turns >= 1);
        assert!(matrix.completed_count >= 1);
        assert_eq!(
            matrix.completed_count + matrix.refused_count,
            CONVERSE_SCENARIO_COUNT
        );
        let constructed: Vec<String> = matrix
            .cells
            .iter()
            .filter_map(|cell| cell.refusal.clone())
            .collect();
        for refusal in ConverseRefusal::ALL {
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
    fn scenario_names_are_unique_and_counted() {
        let mut names: Vec<&str> = CONVERSE_SCENARIO_NAMES.to_vec();
        assert_eq!(names.len(), CONVERSE_SCENARIO_COUNT);
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "scenario names are unique");
    }

    #[test]
    fn refusal_slugs_are_unique() {
        let mut slugs: Vec<&str> = ConverseRefusal::ALL.iter().map(|r| r.slug()).collect();
        assert_eq!(ConverseRefusal::ALL.len(), 17);
        slugs.sort_unstable();
        let n = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), n);
    }

    // -- boundary ----------------------------------------------------------

    #[test]
    fn boundary_is_inert_and_recorded() {
        let transcript = converse_demo();
        assert!(transcript.boundary_all_inert);
        assert_eq!(CONVERSE_BOUNDARY_LINES.len(), 10);
        let mut broken = ConverseBoundary::inert();
        broken.generates_prose = true;
        assert!(!broken.all_inert());
    }

    #[test]
    fn scope_tokens_round_trip_and_reject_unknown() {
        for scope in TurnScope::ALL {
            assert_eq!(TurnScope::parse(scope.slug()), Some(scope));
        }
        assert_eq!(TurnScope::parse("sideways"), None);
        assert_eq!(TurnScope::ALL.len(), 3);
    }
}
