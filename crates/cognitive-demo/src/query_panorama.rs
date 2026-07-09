//! PANORAMA-0 — deterministic, no-model PER-NOTE BREADTH answerer.
//!
//! The frozen answerer (QFLOW / `query_select`) keeps the GLOBAL top-N spans by
//! lexical score with no per-document cap, so a question whose true answer needs
//! one sentence from each of several notes can come back as N fragments of the
//! single densest note — the other notes never surface — and the join is emitted
//! in score-descending order, so a multi-part answer reads scrambled.
//!
//! PANORAMA-0 adds a deterministic BREADTH layer ON TOP of the frozen organs,
//! editing none of them:
//!
//!   1. Run the FROZEN `query_select::select_default` ONCE. If it refuses, forward.
//!   2. Harvest the PUBLIC per-span score vector (`run.receipt.scores`) and the
//!      FROZEN selector's OWN chosen top-N (`run.receipt.candidates`).
//!   3. Take each eligible NOTE's single best span (breadth), admit notes by
//!      winner-score DESC up to a hard cap, each non-top note gated by an INTEGER
//!      precision floor. UNION that with the selector's own candidates so the
//!      result is a SPAN-SET SUPERSET of QFLOW (never a within-note depth
//!      regression), then sort into READING order (document, span).
//!   4. Re-authorize the assembled multi-span answer through the UNMODIFIED
//!      `reading_substrate::execute` + `verify`; emit ONLY if `report.passed`.
//!
//! The law is unchanged: PANORAMA PROPOSES which spans; the frozen verifier
//! AUTHORIZES support. It invents NO relevance (it never re-scores — it reuses the
//! frozen scorer's own ordering), runs no model, trains nothing, and never retags.
//! Every emitted sentence is the verbatim text of a real corpus span the frozen
//! verifier individually grounded; the answer is their space-join and nothing else.
//!
//! HONEST BOUNDARY: a CERTIFY-shaped PANORAMA answer asserts verbatim PROVENANCE +
//! lexical relevance of each span, NOT the answer's internal consistency or
//! correctness. Because the proposer contributes no meaning, two notes with
//! contradictory verbatim sentences can both be admitted (breadth), producing a
//! composite the frozen verifier still accepts — the same exposure QFLOW's top-N
//! already carries, amplified by guaranteed breadth. Detecting contradiction needs
//! meaning, which is forbidden here.
//!
//! Report types are `Serialize` but never `Deserialize`: a serialized flow/matrix is
//! re-derived and byte-compared, so a tampered artifact is refused (`ReplayMismatch`).

use serde::Serialize;

use reading_cli::corpus_from_documents;
use reading_substrate::{execute, verify, Corpus, ReadingAction, ReadingTrace, SpanId};

use crate::query_select::{
    select_default, QuerySelectionDecision, QuerySelectionRefusal, QuerySpanScore,
    SelectedEvidenceCandidate,
};
use crate::vault_norm::normalize_markdown;

/// Structural invariant: PANORAMA-0 runs no model and no training. Every forbidden
/// flag is sourced from this single `false` so no path can flip one true.
const PANORAMA_USES_MODEL: bool = false;

const SCHEMA_FLOW: &str = "panorama-flow-v0.1";
const SCHEMA_MATRIX: &str = "panorama-matrix-v0.1";

/// Longest accepted question (mirrors the CONVERSE-0 bound; refuse-over-truncate).
const MAX_QUESTION_LEN: usize = 512;
/// Hard cap on the number of distinct notes admitted — an A3 upper bound; a
/// genuinely 20-note answer truncates here (the same residual class as QFLOW's
/// fixed budget, just larger).
const DEFAULT_MAX_NOTES: usize = 8;
/// Integer precision floor: a non-top note's winning span is admitted only when
/// `winner.score * FLOOR_DEN >= top.score * FLOOR_NUM`, i.e. `winner >= top/3`.
/// Cross-multiplied in `u128` so it is float-free and cannot overflow.
const DEFAULT_FLOOR_NUM: u64 = 1;
const DEFAULT_FLOOR_DEN: u64 = 3;

// ---------------------------------------------------------------------------
// Config + boundary
// ---------------------------------------------------------------------------

/// PANORAMA configuration. `uses_model`/`uses_training` are sourced from the single
/// `false` invariant; a non-false value is refused before any work.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct PanoramaConfig {
    pub max_notes: usize,
    pub floor_num: u64,
    pub floor_den: u64,
    pub uses_model: bool,
    pub uses_training: bool,
}

impl PanoramaConfig {
    pub fn default_config() -> Self {
        PanoramaConfig {
            max_notes: DEFAULT_MAX_NOTES,
            floor_num: DEFAULT_FLOOR_NUM,
            floor_den: DEFAULT_FLOOR_DEN,
            uses_model: PANORAMA_USES_MODEL,
            uses_training: PANORAMA_USES_MODEL,
        }
    }
}

/// Inert forbidden-behavior flags, every one sourced from `PANORAMA_USES_MODEL`.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct PanoramaBoundary {
    pub is_model: bool,
    pub trains: bool,
    pub generates_prose: bool,
    pub re_scores_spans: bool,
    pub overrides_frozen_ordering: bool,
    pub creates_evidence: bool,
    pub changes_grounding_rules: bool,
    pub infers_relevance_by_meaning: bool,
    pub retags_release: bool,
}

impl PanoramaBoundary {
    fn inert() -> Self {
        PanoramaBoundary {
            is_model: PANORAMA_USES_MODEL,
            trains: PANORAMA_USES_MODEL,
            generates_prose: PANORAMA_USES_MODEL,
            re_scores_spans: PANORAMA_USES_MODEL,
            overrides_frozen_ordering: PANORAMA_USES_MODEL,
            creates_evidence: PANORAMA_USES_MODEL,
            changes_grounding_rules: PANORAMA_USES_MODEL,
            infers_relevance_by_meaning: PANORAMA_USES_MODEL,
            retags_release: PANORAMA_USES_MODEL,
        }
    }

    fn all_inert(&self) -> bool {
        !(self.is_model
            || self.trains
            || self.generates_prose
            || self.re_scores_spans
            || self.overrides_frozen_ordering
            || self.creates_evidence
            || self.changes_grounding_rules
            || self.infers_relevance_by_meaning
            || self.retags_release)
    }
}

// ---------------------------------------------------------------------------
// Decision + refusal taxonomy
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum PanoramaDecision {
    PanoramaAnswered,
    PanoramaRefused,
}

impl PanoramaDecision {
    pub const ALL: [PanoramaDecision; 2] = [
        PanoramaDecision::PanoramaAnswered,
        PanoramaDecision::PanoramaRefused,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            PanoramaDecision::PanoramaAnswered => "panorama_answered",
            PanoramaDecision::PanoramaRefused => "panorama_refused",
        }
    }
}

/// Every reason a PANORAMA run can refuse. Closed enum — fail closed.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum PanoramaRefusal {
    ModelSignalDetected,
    TrainingSignalDetected,
    EmptyVault,
    DuplicateVaultDocName,
    EmptyQuestion,
    QuestionTooLong,
    SelectionRefused,
    CombinedVerificationFailed,
    NonDeterministicChosenOrder,
    VaultBindingMismatch,
    SerializedFlowTamper,
}

impl PanoramaRefusal {
    pub const ALL: [PanoramaRefusal; 11] = [
        PanoramaRefusal::ModelSignalDetected,
        PanoramaRefusal::TrainingSignalDetected,
        PanoramaRefusal::EmptyVault,
        PanoramaRefusal::DuplicateVaultDocName,
        PanoramaRefusal::EmptyQuestion,
        PanoramaRefusal::QuestionTooLong,
        PanoramaRefusal::SelectionRefused,
        PanoramaRefusal::CombinedVerificationFailed,
        PanoramaRefusal::NonDeterministicChosenOrder,
        PanoramaRefusal::VaultBindingMismatch,
        PanoramaRefusal::SerializedFlowTamper,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            PanoramaRefusal::ModelSignalDetected => "model_signal_detected_refused",
            PanoramaRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            PanoramaRefusal::EmptyVault => "empty_vault_refused",
            PanoramaRefusal::DuplicateVaultDocName => "duplicate_vault_doc_name_refused",
            PanoramaRefusal::EmptyQuestion => "empty_question_refused",
            PanoramaRefusal::QuestionTooLong => "question_too_long_refused",
            PanoramaRefusal::SelectionRefused => "selection_refused",
            PanoramaRefusal::CombinedVerificationFailed => "combined_verification_failed_refused",
            PanoramaRefusal::NonDeterministicChosenOrder => {
                "non_deterministic_chosen_order_refused"
            }
            PanoramaRefusal::VaultBindingMismatch => "vault_binding_mismatch_refused",
            PanoramaRefusal::SerializedFlowTamper => "serialized_flow_tamper_refused",
        }
    }
}

/// Re-derivation failure for a serialized flow/matrix (never trusted off-wire).
/// Distinct from [`PanoramaRefusal`]: the byte-compare verify path returns this
/// ERROR, while the matrix CONSTRUCTS the `SerializedFlowTamper` refusal by flipping
/// a byte and re-deriving (the QFLOW / CONVERSE-0 precedent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanoramaError {
    ReplayMismatch,
}

// ---------------------------------------------------------------------------
// Report objects — Serialize but NEVER Deserialize
// ---------------------------------------------------------------------------

/// One chosen, frozen-verified span, lifted VERBATIM from the corpus. `span_id` is
/// provenance only; `document_name` is the stable identity.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PanoramaSpan {
    pub document_id: u64,
    pub document_name: String,
    pub span_id: u64,
    pub verified_text: String,
}

/// The whole replayable PANORAMA flow: the vault fingerprint, the config, the chosen
/// span set (reading order), the answer, and the frozen verifier's verdict. Serialize
/// only; re-derived + byte-compared.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PanoramaFlow {
    pub schema: String,
    pub vault_snapshot_hash: u64,
    pub question: String,
    pub config: PanoramaConfig,
    pub decision: PanoramaDecision,
    pub refusal: Option<PanoramaRefusal>,
    /// When `SelectionRefused`, the forwarded frozen-selector refusal slug.
    pub select_refusal: Option<String>,
    pub eligible_span_count: usize,
    pub note_count: usize,
    pub chosen: Vec<PanoramaSpan>,
    pub answer_text: Option<String>,
    pub answer_supported: bool,
    pub verified: bool,
    pub answer_hash: Option<u64>,
    pub flow_hash: u64,
    pub boundary: PanoramaBoundary,
    pub boundary_all_inert: bool,
}

// ---------------------------------------------------------------------------
// Hashing (deterministic; FNV-1a; integer only — the CONVERSE-0 constants)
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
/// their lengths, so a boundary shift cannot collide). Binds a flow to the exact
/// vault it ran over — a same-name content edit is caught by [`panorama_binds_vault`].
fn vault_snapshot_hash(vault: &[(String, String)]) -> u64 {
    let mut h = fnv_mix(0xcbf2_9ce4_8422_2325, SCHEMA_FLOW.as_bytes());
    h = fnv_u64(h, vault.len() as u64);
    for (name, content) in vault {
        h = fnv_u64(h, name.len() as u64);
        h = fnv_mix(h, name.as_bytes());
        h = fnv_u64(h, content.len() as u64);
        h = fnv_mix(h, content.as_bytes());
    }
    h
}

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

// ---------------------------------------------------------------------------
// Preflight
// ---------------------------------------------------------------------------

fn preflight_refusal(
    documents: &[(String, String)],
    question: &str,
    config: &PanoramaConfig,
) -> Option<PanoramaRefusal> {
    if config.uses_model {
        return Some(PanoramaRefusal::ModelSignalDetected);
    }
    if config.uses_training {
        return Some(PanoramaRefusal::TrainingSignalDetected);
    }
    if documents.is_empty() {
        return Some(PanoramaRefusal::EmptyVault);
    }
    if vault_has_duplicate_names(documents) {
        return Some(PanoramaRefusal::DuplicateVaultDocName);
    }
    if question.trim().is_empty() {
        return Some(PanoramaRefusal::EmptyQuestion);
    }
    if question.len() > MAX_QUESTION_LEN {
        return Some(PanoramaRefusal::QuestionTooLong);
    }
    None
}

// ---------------------------------------------------------------------------
// Breadth selection (composes the frozen scorer's OWN output — no re-scoring)
// ---------------------------------------------------------------------------

fn passes_floor(winner_score: usize, top_score: usize, config: &PanoramaConfig) -> bool {
    (winner_score as u128) * (config.floor_den as u128)
        >= (top_score as u128) * (config.floor_num as u128)
}

/// Returns `(chosen spans in reading order, eligible_span_count, note_count)`.
/// Chosen = UNION of (a) the frozen selector's own candidates (superset guarantee)
/// and (b) each admitted note's best span (breadth), deduped by byte-equal text.
fn select_panorama_spans(
    corpus: &Corpus,
    scores: &[QuerySpanScore],
    candidates: &[SelectedEvidenceCandidate],
    config: &PanoramaConfig,
) -> (Vec<PanoramaSpan>, usize, usize) {
    let eligible: Vec<&QuerySpanScore> = scores
        .iter()
        .filter(|s| s.term_coverage > 0 && s.score > 0)
        .collect();
    let eligible_count = eligible.len();
    if eligible.is_empty() {
        return (Vec::new(), 0, 0);
    }

    // Distinct notes, ascending.
    let mut doc_ids: Vec<u64> = eligible.iter().map(|s| s.document_id).collect();
    doc_ids.sort_unstable();
    doc_ids.dedup();
    let note_count = doc_ids.len();

    // Per-note winner: highest score, tie-break SMALLEST span_id (the frozen order).
    let mut winners: Vec<&QuerySpanScore> = doc_ids
        .iter()
        .map(|&d| {
            eligible
                .iter()
                .copied()
                .filter(|s| s.document_id == d)
                .max_by(|a, b| {
                    a.score
                        .cmp(&b.score)
                        .then_with(|| b.span_id.cmp(&a.span_id))
                })
                .expect("each doc_id came from an eligible span")
        })
        .collect();
    // Admit notes by winner-score DESC, doc ASC.
    winners.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.document_id.cmp(&b.document_id))
    });
    let top_score = winners[0].score;

    // (document_id, span_id) pairs. Start from the admitted breadth winners.
    let mut chosen_pairs: Vec<(u64, u64)> = Vec::new();
    for (i, w) in winners.iter().enumerate() {
        if chosen_pairs.len() >= config.max_notes {
            break;
        }
        if i == 0 || passes_floor(w.score, top_score, config) {
            chosen_pairs.push((w.document_id, w.span_id));
        }
    }
    // Superset: UNION with the frozen selector's OWN candidates (its exact top-N).
    for c in candidates {
        if !chosen_pairs.iter().any(|(_, sid)| *sid == c.span_id) {
            chosen_pairs.push((c.document_id, c.span_id));
        }
    }
    // Reading order: document_id ASC, span_id ASC.
    chosen_pairs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    // Materialize with names + VERBATIM span text; dedup byte-equal text.
    let mut out: Vec<PanoramaSpan> = Vec::new();
    for (document_id, span_id) in chosen_pairs {
        let document_name = corpus
            .metadata()
            .iter()
            .find(|m| m.document_id == document_id)
            .map(|m| m.title.clone())
            .unwrap_or_default();
        let verified_text = corpus
            .read_span(SpanId(span_id))
            .map(|sp| sp.text().to_string())
            .unwrap_or_default();
        if out.iter().any(|o| o.verified_text == verified_text) {
            continue;
        }
        out.push(PanoramaSpan {
            document_id,
            document_name,
            span_id,
            verified_text,
        });
    }
    (out, eligible_count, note_count)
}

/// The chosen set's `(document_id, span_id)` keys must be a strict total order (no
/// duplicates, ascending) — the deterministic-replay guard.
fn chosen_order_is_total(chosen: &[PanoramaSpan]) -> bool {
    for w in chosen.windows(2) {
        let a = (w[0].document_id, w[0].span_id);
        let b = (w[1].document_id, w[1].span_id);
        if a >= b {
            return false;
        }
    }
    true
}

/// Build ONE reading trace over the chosen spans, claim each VERBATIM, synthesize the
/// space-join, and run the FROZEN `execute` + `verify`. Returns `Some((answer, hash))`
/// iff the frozen verifier accepts; `None` (→ CombinedVerificationFailed) otherwise.
fn verify_chosen(
    corpus: &Corpus,
    question: &str,
    chosen: &[(u64, String)],
) -> Option<(String, u64)> {
    let mut trace = ReadingTrace::new();
    trace.push(ReadingAction::InspectCorpus);
    let mut statements: Vec<String> = Vec::new();
    let mut supporting: Vec<u64> = Vec::new();
    for (i, (span_id, text)) in chosen.iter().enumerate() {
        trace.push(ReadingAction::ReadSpan(SpanId(*span_id)));
        trace.push(ReadingAction::ExtractClaim {
            statement: text.clone(),
            source_spans: vec![SpanId(*span_id)],
        });
        supporting.push(i as u64);
        statements.push(text.clone());
    }
    let answer = statements.join(" ");
    trace.push(ReadingAction::Synthesize {
        answer_text: answer.clone(),
        supporting_claims: supporting,
    });
    match execute(corpus, question, &trace) {
        Ok(run) => {
            let report = verify(corpus, &run);
            if report.passed {
                Some((answer, run.answer_hash))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Flow assembly
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn assemble_flow(
    question: &str,
    config: PanoramaConfig,
    vault_snapshot_hash: u64,
    decision: PanoramaDecision,
    refusal: Option<PanoramaRefusal>,
    select_refusal: Option<String>,
    eligible_span_count: usize,
    note_count: usize,
    chosen: Vec<PanoramaSpan>,
    answer_text: Option<String>,
    answer_supported: bool,
    verified: bool,
    answer_hash: Option<u64>,
) -> PanoramaFlow {
    let boundary = PanoramaBoundary::inert();
    let boundary_all_inert = boundary.all_inert();

    let mut h = fnv_mix(0xcbf2_9ce4_8422_2325, SCHEMA_FLOW.as_bytes());
    h = fnv_u64(h, vault_snapshot_hash);
    h = fnv_u64(h, question.len() as u64);
    h = fnv_mix(h, question.as_bytes());
    h = fnv_u64(h, config.max_notes as u64);
    h = fnv_u64(h, config.floor_num);
    h = fnv_u64(h, config.floor_den);
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_mix(h, decision.slug().as_bytes());
    match refusal {
        Some(r) => {
            h = fnv_u64(h, 1);
            h = fnv_mix(h, r.slug().as_bytes());
        }
        None => h = fnv_u64(h, 0),
    }
    match &select_refusal {
        Some(s) => {
            h = fnv_u64(h, 1);
            h = fnv_u64(h, s.len() as u64);
            h = fnv_mix(h, s.as_bytes());
        }
        None => h = fnv_u64(h, 0),
    }
    h = fnv_u64(h, eligible_span_count as u64);
    h = fnv_u64(h, note_count as u64);
    h = fnv_u64(h, chosen.len() as u64);
    for c in &chosen {
        h = fnv_u64(h, c.document_id);
        h = fnv_u64(h, c.document_name.len() as u64);
        h = fnv_mix(h, c.document_name.as_bytes());
        h = fnv_u64(h, c.span_id);
        h = fnv_u64(h, c.verified_text.len() as u64);
        h = fnv_mix(h, c.verified_text.as_bytes());
    }
    match &answer_text {
        Some(a) => {
            h = fnv_u64(h, 1);
            h = fnv_u64(h, a.len() as u64);
            h = fnv_mix(h, a.as_bytes());
        }
        None => h = fnv_u64(h, 0),
    }
    h = fnv_u64(h, answer_supported as u64);
    h = fnv_u64(h, verified as u64);
    match answer_hash {
        Some(v) => {
            h = fnv_u64(h, 1);
            h = fnv_u64(h, v);
        }
        None => h = fnv_u64(h, 0),
    }
    h = fnv_u64(h, boundary_all_inert as u64);

    PanoramaFlow {
        schema: SCHEMA_FLOW.to_string(),
        vault_snapshot_hash,
        question: question.to_string(),
        config,
        decision,
        refusal,
        select_refusal,
        eligible_span_count,
        note_count,
        chosen,
        answer_text,
        answer_supported,
        verified,
        answer_hash,
        flow_hash: h,
        boundary,
        boundary_all_inert,
    }
}

#[allow(clippy::too_many_arguments)]
fn refused_flow(
    question: &str,
    config: PanoramaConfig,
    vault_snapshot_hash: u64,
    refusal: PanoramaRefusal,
    select_refusal: Option<String>,
    eligible_span_count: usize,
    note_count: usize,
    chosen: Vec<PanoramaSpan>,
) -> PanoramaFlow {
    assemble_flow(
        question,
        config,
        vault_snapshot_hash,
        PanoramaDecision::PanoramaRefused,
        Some(refusal),
        select_refusal,
        eligible_span_count,
        note_count,
        chosen,
        None,
        false,
        false,
        None,
    )
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub fn answer_panorama(
    documents: &[(String, String)],
    question: &str,
    config: PanoramaConfig,
) -> PanoramaFlow {
    let vsh = vault_snapshot_hash(documents);
    if let Some(r) = preflight_refusal(documents, question, &config) {
        return refused_flow(question, config, vsh, r, None, 0, 0, Vec::new());
    }

    let normalized: Vec<(String, String)> = documents
        .iter()
        .map(|(name, content)| (name.clone(), normalize_markdown(content)))
        .collect();
    let corpus = corpus_from_documents(&normalized);

    let run = select_default(&corpus, question);
    if run.receipt.decision == QuerySelectionDecision::SelectionRefused {
        let slug = run.receipt.refusal.map(|r| r.slug().to_string());
        return refused_flow(
            question,
            config,
            vsh,
            PanoramaRefusal::SelectionRefused,
            slug,
            0,
            0,
            Vec::new(),
        );
    }

    let (chosen, eligible_count, note_count) = select_panorama_spans(
        &corpus,
        &run.receipt.scores,
        &run.receipt.candidates,
        &config,
    );
    if chosen.is_empty() {
        // Selector passed but nothing materialized — fail closed.
        return refused_flow(
            question,
            config,
            vsh,
            PanoramaRefusal::SelectionRefused,
            Some(QuerySelectionRefusal::NoCandidateSpans.slug().to_string()),
            eligible_count,
            note_count,
            Vec::new(),
        );
    }
    if !chosen_order_is_total(&chosen) {
        return refused_flow(
            question,
            config,
            vsh,
            PanoramaRefusal::NonDeterministicChosenOrder,
            None,
            eligible_count,
            note_count,
            chosen,
        );
    }

    let pairs: Vec<(u64, String)> = chosen
        .iter()
        .map(|c| (c.span_id, c.verified_text.clone()))
        .collect();
    match verify_chosen(&corpus, question, &pairs) {
        Some((answer, answer_hash)) => assemble_flow(
            question,
            config,
            vsh,
            PanoramaDecision::PanoramaAnswered,
            None,
            None,
            eligible_count,
            note_count,
            chosen,
            Some(answer),
            true,
            true,
            Some(answer_hash),
        ),
        None => refused_flow(
            question,
            config,
            vsh,
            PanoramaRefusal::CombinedVerificationFailed,
            None,
            eligible_count,
            note_count,
            chosen,
        ),
    }
}

pub fn answer_panorama_default(documents: &[(String, String)], question: &str) -> PanoramaFlow {
    answer_panorama(documents, question, PanoramaConfig::default_config())
}

/// Re-check a flow's vault binding: a same-name content edit moves the fingerprint.
pub fn panorama_binds_vault(
    flow: &PanoramaFlow,
    vault: &[(String, String)],
) -> Option<PanoramaRefusal> {
    if flow.vault_snapshot_hash != vault_snapshot_hash(vault) {
        Some(PanoramaRefusal::VaultBindingMismatch)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Demo + Serialize-not-Deserialize replay
// ---------------------------------------------------------------------------

/// A breadth demo: `bridge.txt` densely matches "bridge" (three sentences), so the
/// frozen top-N would fill entirely from it; `link.txt` also mentions the bridge but
/// would be dropped — PANORAMA surfaces it via the per-note breadth guarantee.
pub fn panorama_demo_vault() -> Vec<(String, String)> {
    vec![
        (
            "bridge.txt".to_string(),
            "The bridge is open. The bridge is safe. The bridge was inspected.".to_string(),
        ),
        (
            "link.txt".to_string(),
            "The bridge connects the north road.".to_string(),
        ),
        (
            "weather.txt".to_string(),
            "The weather is calm today.".to_string(),
        ),
    ]
}

pub fn panorama_demo_question() -> &'static str {
    "bridge"
}

pub fn panorama_demo() -> PanoramaFlow {
    answer_panorama_default(&panorama_demo_vault(), panorama_demo_question())
}

pub fn panorama_demo_json() -> String {
    serde_json::to_string_pretty(&panorama_demo()).expect("panorama demo serializes")
}

pub fn panorama_flow_json(flow: &PanoramaFlow) -> String {
    serde_json::to_string_pretty(flow).expect("panorama flow serializes")
}

pub fn verify_panorama_demo_json(candidate: &str) -> Result<(), PanoramaError> {
    if candidate == panorama_demo_json() {
        Ok(())
    } else {
        Err(PanoramaError::ReplayMismatch)
    }
}

// ---------------------------------------------------------------------------
// A3 matrix — every refusal variant on a reachable production OR matrix path
// ---------------------------------------------------------------------------

const PANORAMA_SCENARIO_NAMES: [&str; 12] = [
    "panorama_answered",
    "model_signal_detected_refused",
    "training_signal_detected_refused",
    "empty_vault_refused",
    "duplicate_vault_doc_name_refused",
    "empty_question_refused",
    "question_too_long_refused",
    "selection_refused",
    "combined_verification_failed_refused",
    "non_deterministic_chosen_order_refused",
    "vault_binding_mismatch_refused",
    "serialized_flow_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct PanoramaCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub note_count: usize,
    pub chosen_span_count: usize,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PanoramaMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<PanoramaCell>,
    pub answered_count: usize,
    pub refused_count: usize,
    pub boundary: PanoramaBoundary,
    pub boundary_all_inert: bool,
}

fn cell_from_flow(scenario: &str, flow: &PanoramaFlow) -> PanoramaCell {
    PanoramaCell {
        scenario: scenario.to_string(),
        outcome: flow.decision.slug().to_string(),
        refusal: flow.refusal.map(|r| r.slug().to_string()),
        note_count: flow.note_count,
        chosen_span_count: flow.chosen.len(),
        boundary_all_inert: flow.boundary_all_inert,
    }
}

fn cell_from_guard(scenario: &str, refusal: Option<PanoramaRefusal>) -> PanoramaCell {
    PanoramaCell {
        scenario: scenario.to_string(),
        outcome: match refusal {
            Some(_) => "panorama_refused".to_string(),
            None => "violation_missed".to_string(),
        },
        refusal: refusal.map(|r| r.slug().to_string()),
        note_count: 0,
        chosen_span_count: 0,
        boundary_all_inert: PanoramaBoundary::inert().all_inert(),
    }
}

fn cell_for(scenario: &str) -> PanoramaCell {
    match scenario {
        "panorama_answered" => cell_from_flow(scenario, &panorama_demo()),
        "model_signal_detected_refused" => {
            let mut config = PanoramaConfig::default_config();
            config.uses_model = true;
            let flow = answer_panorama(&panorama_demo_vault(), panorama_demo_question(), config);
            cell_from_flow(scenario, &flow)
        }
        "training_signal_detected_refused" => {
            let mut config = PanoramaConfig::default_config();
            config.uses_training = true;
            let flow = answer_panorama(&panorama_demo_vault(), panorama_demo_question(), config);
            cell_from_flow(scenario, &flow)
        }
        "empty_vault_refused" => {
            let vault: Vec<(String, String)> = Vec::new();
            cell_from_flow(scenario, &answer_panorama_default(&vault, "bridge"))
        }
        "duplicate_vault_doc_name_refused" => {
            let vault = vec![
                ("dup.txt".to_string(), "The bridge is open.".to_string()),
                ("dup.txt".to_string(), "The reactor is stable.".to_string()),
            ];
            cell_from_flow(scenario, &answer_panorama_default(&vault, "bridge"))
        }
        "empty_question_refused" => cell_from_flow(
            scenario,
            &answer_panorama_default(&panorama_demo_vault(), "   "),
        ),
        "question_too_long_refused" => {
            let long = "x".repeat(MAX_QUESTION_LEN + 1);
            cell_from_flow(
                scenario,
                &answer_panorama_default(&panorama_demo_vault(), &long),
            )
        }
        "selection_refused" => {
            // A content word absent from the vault → the frozen selector refuses
            // (NoCandidateSpans) → PANORAMA forwards SelectionRefused.
            cell_from_flow(
                scenario,
                &answer_panorama_default(&panorama_demo_vault(), "xylophone"),
            )
        }
        "combined_verification_failed_refused" => {
            // GENUINE negative demo: drive the FROZEN execute+verify with a FAULTED
            // claim (a foreign text for a real span) — the frozen verifier rejects it.
            let normalized: Vec<(String, String)> = panorama_demo_vault()
                .iter()
                .map(|(n, c)| (n.clone(), normalize_markdown(c)))
                .collect();
            let corpus = corpus_from_documents(&normalized);
            let real_id = corpus.metadata()[0].span_ids[0].0;
            let faulted = vec![(real_id, "The bridge is CLOSED forever.".to_string())];
            let refusal = if verify_chosen(&corpus, "bridge", &faulted).is_none() {
                Some(PanoramaRefusal::CombinedVerificationFailed)
            } else {
                None
            };
            cell_from_guard(scenario, refusal)
        }
        "non_deterministic_chosen_order_refused" => {
            // A chosen set with a duplicate (document_id, span_id) key breaks the
            // strict total order the deterministic guard requires.
            let chosen = vec![
                PanoramaSpan {
                    document_id: 0,
                    document_name: "a.txt".to_string(),
                    span_id: 1,
                    verified_text: "The bridge is open.".to_string(),
                },
                PanoramaSpan {
                    document_id: 0,
                    document_name: "a.txt".to_string(),
                    span_id: 1,
                    verified_text: "The bridge is safe.".to_string(),
                },
            ];
            let refusal = if chosen_order_is_total(&chosen) {
                None
            } else {
                Some(PanoramaRefusal::NonDeterministicChosenOrder)
            };
            cell_from_guard(scenario, refusal)
        }
        "vault_binding_mismatch_refused" => {
            let flow = panorama_demo();
            let other = vec![
                (
                    "bridge.txt".to_string(),
                    "The bridge is CLOSED. The status is red.".to_string(),
                ),
                (
                    "link.txt".to_string(),
                    "The bridge connects the north road.".to_string(),
                ),
                (
                    "weather.txt".to_string(),
                    "The weather is calm today.".to_string(),
                ),
            ];
            cell_from_guard(scenario, panorama_binds_vault(&flow, &other))
        }
        "serialized_flow_tamper_refused" => {
            let json = panorama_demo_json();
            let refused = verify_panorama_demo_json(&flip_last_byte(&json)).is_err();
            let refusal = if refused {
                Some(PanoramaRefusal::SerializedFlowTamper)
            } else {
                None
            };
            PanoramaCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused".to_string()
                } else {
                    "tamper_missed".to_string()
                },
                refusal: refusal.map(|r| r.slug().to_string()),
                note_count: 0,
                chosen_span_count: 0,
                boundary_all_inert: PanoramaBoundary::inert().all_inert(),
            }
        }
        other => PanoramaCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            note_count: 0,
            chosen_span_count: 0,
            boundary_all_inert: false,
        },
    }
}

pub fn panorama_matrix() -> PanoramaMatrix {
    let cells: Vec<PanoramaCell> = PANORAMA_SCENARIO_NAMES
        .iter()
        .map(|n| cell_for(n))
        .collect();
    let answered_count = cells
        .iter()
        .filter(|cell| cell.outcome == "panorama_answered")
        .count();
    let refused_count = cells.len() - answered_count;
    let boundary = PanoramaBoundary::inert();
    let boundary_all_inert = boundary.all_inert();
    PanoramaMatrix {
        schema: SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        answered_count,
        refused_count,
        boundary,
        boundary_all_inert,
    }
}

pub fn panorama_matrix_json() -> String {
    serde_json::to_string_pretty(&panorama_matrix()).expect("panorama matrix serializes")
}

pub fn verify_panorama_matrix_json(candidate: &str) -> Result<(), PanoramaError> {
    if candidate == panorama_matrix_json() {
        Ok(())
    } else {
        Err(PanoramaError::ReplayMismatch)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus_of(vault: &[(String, String)]) -> Corpus {
        let normalized: Vec<(String, String)> = vault
            .iter()
            .map(|(n, c)| (n.clone(), normalize_markdown(c)))
            .collect();
        corpus_from_documents(&normalized)
    }

    #[test]
    fn demo_answers_and_is_grounded() {
        let flow = panorama_demo();
        assert_eq!(flow.decision, PanoramaDecision::PanoramaAnswered);
        assert!(flow.verified);
        assert!(flow.answer_supported);
        assert!(flow.refusal.is_none());
        let answer = flow.answer_text.expect("answered");
        assert!(!answer.is_empty());
        assert!(!flow.chosen.is_empty());
        // The answer is exactly the space-join of the chosen verbatim texts.
        let join = flow
            .chosen
            .iter()
            .map(|c| c.verified_text.clone())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(answer, join);
    }

    #[test]
    fn breadth_surfaces_a_note_the_frozen_top_n_would_drop() {
        let vault = panorama_demo_vault();
        let flow = answer_panorama_default(&vault, "bridge");
        // link.txt is dominated by bridge.txt on "bridge" score but must still appear.
        assert!(flow.chosen.iter().any(|c| c.document_name == "link.txt"));
        assert!(flow.note_count >= 2);
    }

    #[test]
    fn chosen_is_a_superset_of_the_frozen_selector_candidates() {
        let vault = panorama_demo_vault();
        let corpus = corpus_of(&vault);
        let run = select_default(&corpus, "bridge");
        let qflow: Vec<u64> = run.receipt.candidates.iter().map(|c| c.span_id).collect();
        let flow = answer_panorama_default(&vault, "bridge");
        let chosen: Vec<u64> = flow.chosen.iter().map(|c| c.span_id).collect();
        for id in qflow {
            assert!(
                chosen.contains(&id),
                "chosen must contain every QFLOW candidate span"
            );
        }
    }

    #[test]
    fn single_eligible_note_chosen_set_equals_frozen_candidate_set() {
        // Only one note carries the term → PANORAMA's span SET equals QFLOW's.
        let vault = vec![
            (
                "only.txt".to_string(),
                "The bridge is open. The bridge is safe. The bridge was inspected.".to_string(),
            ),
            ("other.txt".to_string(), "The weather is calm.".to_string()),
        ];
        let corpus = corpus_of(&vault);
        let run = select_default(&corpus, "bridge");
        let mut qflow: Vec<u64> = run.receipt.candidates.iter().map(|c| c.span_id).collect();
        let flow = answer_panorama_default(&vault, "bridge");
        let mut chosen: Vec<u64> = flow.chosen.iter().map(|c| c.span_id).collect();
        qflow.sort_unstable();
        chosen.sort_unstable();
        assert_eq!(
            chosen, qflow,
            "single-note chosen set must equal the frozen candidate set"
        );
    }

    #[test]
    fn chosen_is_in_reading_order() {
        let flow = panorama_demo();
        assert!(chosen_order_is_total(&flow.chosen));
    }

    #[test]
    fn empty_vault_refuses() {
        let vault: Vec<(String, String)> = Vec::new();
        let flow = answer_panorama_default(&vault, "bridge");
        assert_eq!(flow.refusal, Some(PanoramaRefusal::EmptyVault));
        assert_eq!(flow.decision, PanoramaDecision::PanoramaRefused);
    }

    #[test]
    fn duplicate_doc_name_refuses() {
        let vault = vec![
            ("dup.txt".to_string(), "The bridge is open.".to_string()),
            ("dup.txt".to_string(), "The reactor is stable.".to_string()),
        ];
        let flow = answer_panorama_default(&vault, "bridge");
        assert_eq!(flow.refusal, Some(PanoramaRefusal::DuplicateVaultDocName));
    }

    #[test]
    fn empty_question_refuses() {
        let flow = answer_panorama_default(&panorama_demo_vault(), "   ");
        assert_eq!(flow.refusal, Some(PanoramaRefusal::EmptyQuestion));
    }

    #[test]
    fn too_long_question_refuses() {
        let long = "x".repeat(MAX_QUESTION_LEN + 1);
        let flow = answer_panorama_default(&panorama_demo_vault(), &long);
        assert_eq!(flow.refusal, Some(PanoramaRefusal::QuestionTooLong));
    }

    #[test]
    fn no_lexical_match_forwards_selection_refused() {
        let flow = answer_panorama_default(&panorama_demo_vault(), "xylophone");
        assert_eq!(flow.refusal, Some(PanoramaRefusal::SelectionRefused));
        assert!(flow.select_refusal.is_some());
    }

    #[test]
    fn model_and_training_signals_refuse() {
        let mut m = PanoramaConfig::default_config();
        m.uses_model = true;
        let fm = answer_panorama(&panorama_demo_vault(), "bridge", m);
        assert_eq!(fm.refusal, Some(PanoramaRefusal::ModelSignalDetected));

        let mut t = PanoramaConfig::default_config();
        t.uses_training = true;
        let ft = answer_panorama(&panorama_demo_vault(), "bridge", t);
        assert_eq!(ft.refusal, Some(PanoramaRefusal::TrainingSignalDetected));
    }

    #[test]
    fn vault_binding_mismatch_is_detected() {
        let flow = panorama_demo();
        let other = vec![
            (
                "bridge.txt".to_string(),
                "The bridge is CLOSED.".to_string(),
            ),
            (
                "link.txt".to_string(),
                "The bridge connects the north road.".to_string(),
            ),
            (
                "weather.txt".to_string(),
                "The weather is calm today.".to_string(),
            ),
        ];
        assert_eq!(
            panorama_binds_vault(&flow, &other),
            Some(PanoramaRefusal::VaultBindingMismatch)
        );
        // The exact same vault binds cleanly.
        assert_eq!(panorama_binds_vault(&flow, &panorama_demo_vault()), None);
    }

    #[test]
    fn faulted_claim_fails_the_frozen_verifier() {
        let corpus = corpus_of(&panorama_demo_vault());
        let real_id = corpus.metadata()[0].span_ids[0].0;
        let faulted = vec![(real_id, "The bridge is CLOSED forever.".to_string())];
        assert!(verify_chosen(&corpus, "bridge", &faulted).is_none());
        // The real text passes.
        let real_text = corpus
            .read_span(SpanId(real_id))
            .unwrap()
            .text()
            .to_string();
        assert!(verify_chosen(&corpus, "bridge", &[(real_id, real_text)]).is_some());
    }

    #[test]
    fn non_total_chosen_order_is_rejected() {
        let bad = vec![
            PanoramaSpan {
                document_id: 0,
                document_name: "a.txt".to_string(),
                span_id: 2,
                verified_text: "x".to_string(),
            },
            PanoramaSpan {
                document_id: 0,
                document_name: "a.txt".to_string(),
                span_id: 2,
                verified_text: "y".to_string(),
            },
        ];
        assert!(!chosen_order_is_total(&bad));
    }

    #[test]
    fn demo_json_replay_verifies_and_refuses_tamper() {
        let json = panorama_demo_json();
        assert!(verify_panorama_demo_json(&json).is_ok());
        assert_eq!(
            verify_panorama_demo_json(&flip_last_byte(&json)),
            Err(PanoramaError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_json_replay_verifies_and_refuses_tamper() {
        let json = panorama_matrix_json();
        assert!(verify_panorama_matrix_json(&json).is_ok());
        assert_eq!(
            verify_panorama_matrix_json(&flip_last_byte(&json)),
            Err(PanoramaError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_covers_every_refusal_variant() {
        let matrix = panorama_matrix();
        let slugs: Vec<String> = matrix
            .cells
            .iter()
            .filter_map(|c| c.refusal.clone())
            .collect();
        for refusal in PanoramaRefusal::ALL {
            assert!(
                slugs.iter().any(|s| s == refusal.slug()),
                "matrix must construct refusal variant {}",
                refusal.slug()
            );
        }
        assert!(matrix.answered_count >= 1);
        assert_eq!(matrix.scenario_count, PANORAMA_SCENARIO_NAMES.len());
    }

    #[test]
    fn boundary_is_all_inert() {
        assert!(PanoramaBoundary::inert().all_inert());
        assert!(panorama_demo().boundary_all_inert);
    }
}
