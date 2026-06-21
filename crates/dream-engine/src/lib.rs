//! dream-engine — DREAM-0, the seeded deterministic distortion substrate.
//!
//! The dream engine DISTORTS verified corpus material into weird,
//! assumption-breaking but receipt-grounded candidate structures. It is ALIEN
//! internally and LAWFUL at the boundary: a [`DreamPacket`] is terminal and
//! inert. It carries a crate-private [`DreamAuthority::DreamOnly`] that is NEVER a
//! public authority and NEVER enters the hypothesis layer — DREAM-0 has no
//! dependency on it and no export path. Every packet is `Serialize` but NOT
//! `Deserialize`: it is re-derived from its inputs and byte-compared, never parsed
//! back into authority.
//!
//! Grounding is rebuilt on `reading-substrate` only (its own narrow canonical
//! corpus read), so the engine is structurally independent of the integration
//! crate. Ids are FNV-1a over the defining inputs; there is no entropy, no clock,
//! and no floating point, so a packet re-derives byte-for-byte from its seed.

#![forbid(unsafe_code)]

use reading_substrate::{
    execute, split_sentences, verify, Corpus, ReadingAction, ReadingTrace, SpanId,
};
use serde::Serialize;

// ── FNV-1a (64-bit) — the only hashing in the crate, deterministic across hosts ──
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The nine-line DREAM-0 boundary, embedded verbatim in every packet so the
/// refusal is machine-checkable from the packet's own bytes.
pub const DREAM_BOUNDARY_LINES: [&str; 9] = [
    "The dream engine distorts.",
    "It does not certify.",
    "Dream authority is private to dream-engine.",
    "No dream output enters the hypothesis layer in DREAM-0.",
    "Dream packets are terminal and inert.",
    "Probe requests do not execute.",
    "Nothing becomes evidence.",
    "Nothing promotes.",
    "Nothing trains.",
];

/// The canonical six uses a dream packet is FORBIDDEN from ever acquiring,
/// recorded explicitly in every packet (not the weaker four-entry novelty list).
pub const DREAM_FORBIDDEN_USES: [&str; 6] = [
    "ground_claim",
    "serve_as_evidence",
    "mutate_reading_memory",
    "alter_verifier_receipt",
    "change_training_gate",
    "bypass_codec_or_governance",
];

const PACKET_SCHEMA: &str = "dream-packet-v0.1";
const PROBE_SCHEMA: &str = "dream-probe-request-v0.1";
const MAX_WEIRDNESS: i64 = 5;

fn fnv_bytes(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn fnv_u64(h: u64, value: u64) -> u64 {
    fnv_bytes(h, &value.to_le_bytes())
}

fn fnv_i64(h: u64, value: i64) -> u64 {
    fnv_bytes(h, &value.to_le_bytes())
}

/// Length-prefixed string mixing so re-grouping cannot collide two inputs.
fn fnv_str(h: u64, s: &str) -> u64 {
    let h = fnv_u64(h, s.len() as u64);
    fnv_bytes(h, s.as_bytes())
}

/// Counter-mode FNV seed expansion with a splitmix64 finalizer — deterministic,
/// no entropy source. The finalizer avalanches the FNV output so the low bits are
/// well distributed (FNV-1a's low bits alone are too weak for `% n` selection).
fn seed_stream(seed: u64, counter: u64) -> u64 {
    let mut x = fnv_u64(fnv_u64(FNV_OFFSET, seed), counter);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// Why building or verifying a dream packet failed — every refusal is explicit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DreamError {
    /// The operator frame has no non-empty assumption line.
    EmptyFrame,
    /// The admitted documents yield no readable spans.
    EmptyCorpus,
    /// The weirdness dial is outside `0..=5`.
    WeirdnessOutOfRange(i64),
    /// The crate's own canonical corpus read does not verify (fails closed).
    CorpusDoesNotVerify,
    /// A preserved fact is not VERBATIM a verified corpus span.
    UnsupportedPreservedFact,
    /// An anti-degeneracy gate refused the packet.
    DegenerateDream(&'static str),
    /// A provided packet is not the byte-identical re-derived packet.
    DreamPacketMismatch,
}

impl std::fmt::Display for DreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DreamError::EmptyFrame => write!(f, "the operator frame has no assumption lines"),
            DreamError::EmptyCorpus => write!(f, "the admitted documents yield no spans"),
            DreamError::WeirdnessOutOfRange(w) => write!(f, "weirdness {w} is outside 0..=5"),
            DreamError::CorpusDoesNotVerify => {
                write!(f, "the canonical corpus read does not verify")
            }
            DreamError::UnsupportedPreservedFact => {
                write!(f, "a preserved fact is not a verified corpus span")
            }
            DreamError::DegenerateDream(why) => write!(f, "degenerate dream refused: {why}"),
            DreamError::DreamPacketMismatch => {
                write!(f, "the provided dream packet is not the re-derived packet")
            }
        }
    }
}

impl std::error::Error for DreamError {}

/// The single authority a dream packet may carry. The enum has ONE variant — there
/// is no evidence/promoted/truth variant to construct — and it is `Serialize` but
/// NOT `Deserialize`, so a packet structurally cannot claim authority beyond a
/// terminal dream. It is crate-private vocabulary: it is never the public
/// hypothesis-layer `Authority`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum DreamAuthority {
    #[serde(rename = "dream_only")]
    DreamOnly,
}

/// The seeded distortion operators. Each records itself on the artifact it makes,
/// so a degenerate identity output is detectable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DistortionOperator {
    RoleInversion,
    CategoryViolation,
    ConstraintRemoval,
    ContradictionBraid,
    ScaleShift,
}

/// Every registered seeded distortion operator (five — the DREAM-0 floor).
pub const OPERATORS: [DistortionOperator; 5] = [
    DistortionOperator::RoleInversion,
    DistortionOperator::CategoryViolation,
    DistortionOperator::ConstraintRemoval,
    DistortionOperator::ContradictionBraid,
    DistortionOperator::ScaleShift,
];

/// A fact the dream preserves — VERBATIM a verified corpus span, addressed by id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PreservedFact {
    pub span_id: u64,
    pub document_id: u64,
    pub text: String,
}

/// An assumption a distortion operator broke — never a verbatim frame echo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BrokenAssumption {
    pub operator: DistortionOperator,
    pub text: String,
    pub derived_from_span_ids: Vec<u64>,
}

/// An impossible link binding spans from DIFFERENT documents into one frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ImpossibleLink {
    pub operator: DistortionOperator,
    pub text: String,
    pub span_ids: Vec<u64>,
    pub document_ids: Vec<u64>,
}

/// A reference-only falsifier slot: it NAMES the preserved fact (by span id and a
/// per-fact hash) and the broken assumption it would invalidate. DREAM-0 ships no
/// falsifier GENERATOR — a deterministic layer cannot validate a refutation
/// condition, so real falsifier power is deferred to the later LLM stage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FalsifierSlot {
    pub preserved_fact_span_id: u64,
    pub preserved_fact_memory_hash: u64,
    pub broken_assumption_index: usize,
}

/// A request to TEST an assumption — recorded, never executed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DreamProbeRequest {
    pub schema: String,
    pub request_id: u64,
    pub question: String,
    pub status: String,
    pub executes: bool,
}

/// A terminal, inert dream packet. `Serialize` but NOT `Deserialize`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DreamPacket {
    pub schema: String,
    pub packet_id: String,
    pub seed: u64,
    pub weirdness: i64,
    /// Whole-input commitment: every admitted document, the spans the packet used,
    /// and the reading receipt hashes — so a side document cannot mutate silently.
    pub dream_input_hash: String,
    pub source_receipt_memory_hash: u64,
    pub source_receipt_answer_hash: u64,
    pub source_corpus_hash: u64,
    pub frame_text: String,
    pub preserved_facts: Vec<PreservedFact>,
    pub broken_assumptions: Vec<BrokenAssumption>,
    pub impossible_links: Vec<ImpossibleLink>,
    pub candidate_frames: Vec<String>,
    pub falsifiers: Vec<FalsifierSlot>,
    pub probe_requests: Vec<DreamProbeRequest>,
    pub authority: DreamAuthority,
    pub forbidden_uses: Vec<String>,
    pub boundary: Vec<String>,
}

/// One admitted document — a canonical name and its full admitted text bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DreamDocument {
    pub name: String,
    pub text: String,
}

/// The inputs to a dream: the admitted documents, the untrusted operator frame, a
/// seed, and the weirdness dial (`0..=5`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DreamInput {
    pub documents: Vec<DreamDocument>,
    pub frame_text: String,
    pub seed: u64,
    pub weirdness: i64,
}

fn build_corpus(documents: &[DreamDocument]) -> Corpus {
    let mut corpus = Corpus::new();
    for doc in documents {
        let sentences = split_sentences(&doc.text);
        let refs: Vec<&str> = sentences.iter().map(String::as_str).collect();
        corpus.add_document(&doc.name, &refs);
    }
    corpus
}

fn verified_spans(corpus: &Corpus) -> Vec<PreservedFact> {
    let mut spans: Vec<PreservedFact> = Vec::new();
    for doc in corpus.metadata() {
        for sid in &doc.span_ids {
            if let Some(span) = corpus.read_span(*sid) {
                spans.push(PreservedFact {
                    span_id: sid.0,
                    document_id: span.document_id,
                    text: span.text().to_string(),
                });
            }
        }
    }
    spans.sort_by(|a, b| a.span_id.cmp(&b.span_id));
    spans
}

fn frame_assumptions(frame_text: &str) -> Result<Vec<String>, DreamError> {
    let lines: Vec<String> = frame_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    if lines.is_empty() {
        return Err(DreamError::EmptyFrame);
    }
    Ok(lines)
}

/// The crate's OWN canonical verified read of the corpus: inspect, read every span,
/// extract one claim per span (grounded in that span), synthesize the answer from
/// all claims. Returns the reading receipt `(memory_hash, answer_hash)`; fails
/// closed if the read does not verify. Built on `reading-substrate` only.
fn canonical_read(corpus: &Corpus) -> Result<(u64, u64), DreamError> {
    let mut span_ids: Vec<SpanId> = corpus
        .metadata()
        .iter()
        .flat_map(|doc| doc.span_ids.iter().copied())
        .collect();
    span_ids.sort();
    if span_ids.is_empty() {
        return Err(DreamError::EmptyCorpus);
    }

    let mut trace = ReadingTrace::new();
    trace.push(ReadingAction::InspectCorpus);
    for id in &span_ids {
        trace.push(ReadingAction::ReadSpan(*id));
    }
    let mut statements: Vec<String> = Vec::with_capacity(span_ids.len());
    for id in &span_ids {
        let text = corpus
            .read_span(*id)
            .ok_or(DreamError::EmptyCorpus)?
            .text()
            .to_string();
        trace.push(ReadingAction::ExtractClaim {
            statement: text.clone(),
            source_spans: vec![*id],
        });
        statements.push(text);
    }
    let supporting: Vec<u64> = (0..statements.len() as u64).collect();
    trace.push(ReadingAction::Synthesize {
        answer_text: statements.join(" "),
        supporting_claims: supporting,
    });

    let run = execute(corpus, "dream-engine canonical corpus read", &trace)
        .map_err(|_| DreamError::CorpusDoesNotVerify)?;
    if !verify(corpus, &run).passed {
        return Err(DreamError::CorpusDoesNotVerify);
    }
    Ok((run.memory_hash, run.answer_hash))
}

fn reverse_words(text: &str) -> String {
    text.split_whitespace().rev().collect::<Vec<_>>().join(" ")
}

fn rotate_words(text: &str, by: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let pivot = by % words.len();
    let mut rotated: Vec<&str> = Vec::with_capacity(words.len());
    rotated.extend_from_slice(&words[pivot..]);
    rotated.extend_from_slice(&words[..pivot]);
    rotated.join(" ")
}

/// The number of distinct cross-document span pairs available.
fn max_cross_pairs(spans: &[PreservedFact]) -> usize {
    let mut count = 0usize;
    for (i, a) in spans.iter().enumerate() {
        for b in &spans[i + 1..] {
            if a.document_id != b.document_id {
                count += 1;
            }
        }
    }
    count
}

/// Seeded selection of `want` distinct cross-document index pairs (deterministic).
/// The walk is seed-driven, so `cross_doc_pairs(.., k)` is a prefix of
/// `cross_doc_pairs(.., k+1)` — selection is monotone in the weirdness dial.
fn cross_doc_pairs(spans: &[PreservedFact], seed: u64, want: usize) -> Vec<(usize, usize)> {
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    if spans.len() < 2 || want == 0 {
        return pairs;
    }
    let n = spans.len() as u64;
    let mut counter = 0u64;
    let mut guard = 0u64;
    while pairs.len() < want && guard < 1_000_000 {
        let i = (seed_stream(seed, counter) % n) as usize;
        counter += 1;
        let j = (seed_stream(seed, counter) % n) as usize;
        counter += 1;
        guard += 1;
        if spans[i].document_id != spans[j].document_id {
            let pair = if i < j { (i, j) } else { (j, i) };
            if !pairs.contains(&pair) {
                pairs.push(pair);
            }
        }
    }
    pairs
}

fn source_corpus_hash(documents: &[DreamDocument]) -> u64 {
    let mut h = fnv_str(FNV_OFFSET, "dream-corpus-identity-v0.1");
    h = fnv_u64(h, documents.len() as u64);
    for (doc_id, doc) in documents.iter().enumerate() {
        h = fnv_u64(h, doc_id as u64);
        h = fnv_str(h, &doc.name);
        h = fnv_str(h, &doc.text);
    }
    h
}

/// The whole-input commitment: sorted document id + canonical name + full admitted
/// text bytes, then the spans the packet used, then the reading receipt hashes.
fn compute_dream_input_hash(
    documents: &[DreamDocument],
    used: &[PreservedFact],
    memory_hash: u64,
    answer_hash: u64,
) -> u64 {
    let mut h = fnv_str(FNV_OFFSET, "dream-input-hash-v0.1");
    h = fnv_u64(h, documents.len() as u64);
    for (doc_id, doc) in documents.iter().enumerate() {
        h = fnv_u64(h, doc_id as u64);
        h = fnv_str(h, &doc.name);
        h = fnv_str(h, &doc.text);
    }
    h = fnv_u64(h, used.len() as u64);
    for fact in used {
        h = fnv_u64(h, fact.span_id);
        h = fnv_u64(h, fact.document_id);
        h = fnv_str(h, &fact.text);
    }
    h = fnv_u64(h, memory_hash);
    fnv_u64(h, answer_hash)
}

/// Grounding primitive: a candidate fact is supported only if it is VERBATIM a
/// verified span of the admitted documents. Reusable and testable on its own.
pub fn ground_preserved_fact(documents: &[DreamDocument], text: &str) -> Result<(), DreamError> {
    let corpus = build_corpus(documents);
    if verified_spans(&corpus).iter().any(|span| span.text == text) {
        Ok(())
    } else {
        Err(DreamError::UnsupportedPreservedFact)
    }
}

/// Anti-degeneracy gates, enforced as refusals (not merely tested).
fn enforce_gates(
    preserved: &[PreservedFact],
    broken: &[BrokenAssumption],
    links: &[ImpossibleLink],
    frames: &[String],
    frame_lines: &[String],
) -> Result<(), DreamError> {
    // G1 — every produced artifact is byte-distinct from every preserved span.
    let fact_texts: Vec<&str> = preserved.iter().map(|f| f.text.as_str()).collect();
    let any_identity = broken
        .iter()
        .map(|b| b.text.as_str())
        .chain(links.iter().map(|l| l.text.as_str()))
        .chain(frames.iter().map(String::as_str))
        .any(|text| fact_texts.contains(&text));
    if any_identity {
        return Err(DreamError::DegenerateDream(
            "a produced artifact is byte-identical to a preserved span (no distortion applied)",
        ));
    }

    // G2 — at least one link combines two spans from DIFFERENT documents.
    let has_cross = links
        .iter()
        .any(|l| l.document_ids.len() == 2 && l.document_ids[0] != l.document_ids[1]);
    if !has_cross {
        return Err(DreamError::DegenerateDream(
            "no impossible link combines two distinct documents",
        ));
    }

    // G3 — at least one broken assumption is operator output, byte-distinct from
    //      every frame line and every preserved span.
    let has_broken = broken.iter().any(|b| {
        !frame_lines.iter().any(|line| line == &b.text)
            && !preserved.iter().any(|f| f.text == b.text)
    });
    if !has_broken {
        return Err(DreamError::DegenerateDream(
            "no broken assumption is distinct from the frame and the preserved spans",
        ));
    }
    Ok(())
}

/// Build the deterministic, terminal dream packet for the given input.
pub fn dream_packet(input: &DreamInput) -> Result<DreamPacket, DreamError> {
    if !(0..=MAX_WEIRDNESS).contains(&input.weirdness) {
        return Err(DreamError::WeirdnessOutOfRange(input.weirdness));
    }
    let frame_lines = frame_assumptions(&input.frame_text)?;

    let corpus = build_corpus(&input.documents);
    if corpus.span_count() == 0 {
        return Err(DreamError::EmptyCorpus);
    }
    let (memory_hash, answer_hash) = canonical_read(&corpus)?;
    let spans = verified_spans(&corpus);

    let level = input.weirdness as usize;
    let want_links = (level + 1).min(max_cross_pairs(&spans));
    let pairs = cross_doc_pairs(&spans, input.seed, want_links);
    if pairs.is_empty() {
        return Err(DreamError::DegenerateDream(
            "no cross-document span pair: a dream must combine at least two distinct documents",
        ));
    }

    // Impossible links — each combines two spans from DIFFERENT documents (G2).
    let mut impossible_links: Vec<ImpossibleLink> = Vec::with_capacity(pairs.len());
    for (idx, (i, j)) in pairs.iter().enumerate() {
        let a = &spans[*i];
        let b = &spans[*j];
        let (operator, text) = if idx % 2 == 0 {
            (
                DistortionOperator::CategoryViolation,
                format!(
                    "Category violation — treat the subject of \"{}\" as the process of \"{}\".",
                    a.text, b.text
                ),
            )
        } else {
            (
                DistortionOperator::ContradictionBraid,
                format!(
                    "Contradiction braid — hold jointly, against type: \"{}\" AND \"{}\".",
                    a.text, b.text
                ),
            )
        };
        impossible_links.push(ImpossibleLink {
            operator,
            text,
            span_ids: vec![a.span_id, b.span_id],
            document_ids: vec![a.document_id, b.document_id],
        });
    }

    // Preserved facts — exactly the spans the links combine (what the dream used).
    let mut preserved_facts: Vec<PreservedFact> = Vec::new();
    for (i, j) in &pairs {
        for idx in [*i, *j] {
            let fact = &spans[idx];
            if !preserved_facts.iter().any(|f| f.span_id == fact.span_id) {
                preserved_facts.push(fact.clone());
            }
        }
    }
    preserved_facts.sort_by(|a, b| a.span_id.cmp(&b.span_id));

    // Broken assumptions — constraint removal over frame lines + a role inversion.
    let mut broken_assumptions: Vec<BrokenAssumption> = Vec::new();
    let want_constraints = (level + 1).min(frame_lines.len());
    for line in frame_lines.iter().take(want_constraints) {
        broken_assumptions.push(BrokenAssumption {
            operator: DistortionOperator::ConstraintRemoval,
            text: format!(
                "Constraint removal — drop the hidden rule \"{line}\"; what still stands without it?"
            ),
            derived_from_span_ids: Vec::new(),
        });
    }
    let pivot = preserved_facts[0].clone();
    broken_assumptions.push(BrokenAssumption {
        operator: DistortionOperator::RoleInversion,
        text: format!(
            "Role inversion — if cause and effect swap: {}",
            reverse_words(&pivot.text)
        ),
        derived_from_span_ids: vec![pivot.span_id],
    });

    // Candidate frames — scale shift over preserved facts.
    let want_frames = (level + 1).min(preserved_facts.len());
    let mut candidate_frames: Vec<String> = Vec::with_capacity(want_frames);
    for (offset, fact) in preserved_facts.iter().take(want_frames).enumerate() {
        candidate_frames.push(format!(
            "Scale shift — read the local fact as a system law: {}",
            rotate_words(&fact.text, offset + 1)
        ));
    }

    enforce_gates(
        &preserved_facts,
        &broken_assumptions,
        &impossible_links,
        &candidate_frames,
        &frame_lines,
    )?;

    // Falsifier slots — reference-only (span id + per-fact hash + assumption index).
    let mut falsifiers: Vec<FalsifierSlot> = Vec::with_capacity(preserved_facts.len());
    for (offset, fact) in preserved_facts.iter().enumerate() {
        let fact_hash = fnv_str(fnv_u64(memory_hash, fact.span_id), &fact.text);
        falsifiers.push(FalsifierSlot {
            preserved_fact_span_id: fact.span_id,
            preserved_fact_memory_hash: fact_hash,
            broken_assumption_index: offset % broken_assumptions.len(),
        });
    }

    // Probe requests — recorded, never executed.
    let probe_requests: Vec<DreamProbeRequest> = frame_lines
        .iter()
        .enumerate()
        .map(|(idx, line)| DreamProbeRequest {
            schema: PROBE_SCHEMA.to_string(),
            request_id: idx as u64,
            question: format!("What observation would test relaxing the assumption \"{line}\"?"),
            status: "requires_operator_review".to_string(),
            executes: false,
        })
        .collect();

    let source_corpus = source_corpus_hash(&input.documents);
    let input_hash =
        compute_dream_input_hash(&input.documents, &preserved_facts, memory_hash, answer_hash);
    let mut id_h = fnv_u64(FNV_OFFSET, input_hash);
    id_h = fnv_u64(id_h, input.seed);
    id_h = fnv_i64(id_h, input.weirdness);
    id_h = fnv_str(id_h, &input.frame_text);
    let packet_id = format!("dream-{id_h:016x}");

    Ok(DreamPacket {
        schema: PACKET_SCHEMA.to_string(),
        packet_id,
        seed: input.seed,
        weirdness: input.weirdness,
        dream_input_hash: format!("{input_hash:016x}"),
        source_receipt_memory_hash: memory_hash,
        source_receipt_answer_hash: answer_hash,
        source_corpus_hash: source_corpus,
        frame_text: input.frame_text.clone(),
        preserved_facts,
        broken_assumptions,
        impossible_links,
        candidate_frames,
        falsifiers,
        probe_requests,
        authority: DreamAuthority::DreamOnly,
        forbidden_uses: DREAM_FORBIDDEN_USES.iter().map(|u| u.to_string()).collect(),
        boundary: DREAM_BOUNDARY_LINES.iter().map(|b| b.to_string()).collect(),
    })
}

/// The dream packet as pretty JSON — pure and deterministic (fixed field order),
/// so it re-derives byte-for-byte.
pub fn dream_packet_json(input: &DreamInput) -> Result<String, DreamError> {
    let packet = dream_packet(input)?;
    Ok(serde_json::to_string_pretty(&packet).expect("DreamPacket serializes"))
}

/// Re-derive the packet from the SAME input and confirm the PROVIDED JSON is
/// byte-for-byte that packet. The provided packet is never parsed back into
/// authority — only compared — so a tampered/foreign packet is REFUSED.
pub fn verify_dream_packet_json(input: &DreamInput, provided: &str) -> Result<(), DreamError> {
    if provided == dream_packet_json(input)? {
        Ok(())
    } else {
        Err(DreamError::DreamPacketMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(name: &str, text: &str) -> DreamDocument {
        DreamDocument {
            name: name.to_string(),
            text: text.to_string(),
        }
    }

    fn fixture() -> DreamInput {
        DreamInput {
            documents: vec![
                doc(
                    "bridge_report",
                    "Bridge A was reported structurally damaged after the June storm. \
                     Inspectors advised against using Bridge A until repairs are complete.",
                ),
                doc(
                    "weather_log",
                    "The June storm brought heavy rain and high winds overnight. \
                     Bridge B remained passable during light rain.",
                ),
            ],
            frame_text: "Documents are passive inputs.\nSource selection is mere retrieval."
                .to_string(),
            seed: 42,
            weirdness: 2,
        }
    }

    #[test]
    fn dream_packet_builds_from_verified_corpus() {
        let p = dream_packet(&fixture()).expect("builds");
        assert_eq!(p.authority, DreamAuthority::DreamOnly);
        assert_eq!(p.schema, "dream-packet-v0.1");
        assert!(!p.impossible_links.is_empty());
        assert!(!p.preserved_facts.is_empty());
        assert!(p.packet_id.starts_with("dream-"));
    }

    #[test]
    fn dream_input_hash_changes_when_side_document_changes() {
        let mut with_side = fixture();
        with_side.documents.push(doc(
            "side_note",
            "Original side content, unrelated to the bridges.",
        ));
        let p1 = dream_packet(&with_side).unwrap();

        let mut mutated = with_side.clone();
        mutated.documents[2].text =
            "Mutated side content, silently altered after admission.".to_string();
        let p2 = dream_packet(&mutated).unwrap();

        assert_ne!(
            p1.dream_input_hash, p2.dream_input_hash,
            "the input commitment must cover every admitted document, side documents included"
        );
    }

    #[test]
    fn dream_refuses_degenerate_single_span_reformat() {
        let input = DreamInput {
            documents: vec![doc(
                "only",
                "A single lonely sentence with no second document.",
            )],
            frame_text: "One assumption.".to_string(),
            seed: 1,
            weirdness: 3,
        };
        assert!(matches!(
            dream_packet(&input),
            Err(DreamError::DegenerateDream(_))
        ));
    }

    #[test]
    fn dream_links_two_distinct_document_ids_into_one_frame() {
        let p = dream_packet(&fixture()).unwrap();
        assert!(p
            .impossible_links
            .iter()
            .any(|l| l.document_ids.len() == 2 && l.document_ids[0] != l.document_ids[1]));
    }

    #[test]
    fn dream_broken_assumption_is_operator_output_not_frame_echo() {
        let input = fixture();
        let frame_lines: Vec<String> = input
            .frame_text
            .lines()
            .map(|l| l.trim().to_string())
            .collect();
        let p = dream_packet(&input).unwrap();
        assert!(p.broken_assumptions.iter().any(|b| {
            !frame_lines.contains(&b.text) && !p.preserved_facts.iter().any(|f| f.text == b.text)
        }));
    }

    #[test]
    fn dream_falsifier_slot_well_formed_by_reference() {
        let p = dream_packet(&fixture()).unwrap();
        assert!(!p.falsifiers.is_empty());
        let mut seen = std::collections::BTreeSet::new();
        for fs in &p.falsifiers {
            assert!(p
                .preserved_facts
                .iter()
                .any(|f| f.span_id == fs.preserved_fact_span_id));
            assert!(fs.broken_assumption_index < p.broken_assumptions.len());
            assert!(
                seen.insert((fs.preserved_fact_span_id, fs.broken_assumption_index)),
                "each (fact, assumption) pair is unique"
            );
        }
    }

    #[test]
    fn dream_replay_byte_identical_two_processes() {
        let input = fixture();
        let a = dream_packet_json(&input).unwrap();
        let b = dream_packet_json(&input).unwrap();
        assert_eq!(a, b, "re-derivation is byte-identical");
        assert!(verify_dream_packet_json(&input, &a).is_ok());
    }

    #[test]
    fn dream_packet_tamper_refused() {
        let input = fixture();
        let json = dream_packet_json(&input).unwrap();
        let tampered = json.replacen("dream_only", "evidence", 1);
        assert_ne!(tampered, json);
        assert_eq!(
            verify_dream_packet_json(&input, &tampered),
            Err(DreamError::DreamPacketMismatch)
        );
    }

    #[test]
    fn dream_unsupported_preserved_fact_refused() {
        let input = fixture();
        assert!(ground_preserved_fact(
            &input.documents,
            "Bridge B remained passable during light rain."
        )
        .is_ok());
        assert_eq!(
            ground_preserved_fact(&input.documents, "Bridge Z is perfectly safe to cross."),
            Err(DreamError::UnsupportedPreservedFact)
        );
    }

    #[test]
    fn dream_preserved_facts_are_verified_spans() {
        let input = fixture();
        let p = dream_packet(&input).unwrap();
        for fact in &p.preserved_facts {
            assert!(ground_preserved_fact(&input.documents, &fact.text).is_ok());
        }
    }

    #[test]
    fn dream_packet_is_terminal_no_export() {
        let json = dream_packet_json(&fixture()).unwrap();
        assert!(json.contains("\"dream_only\""));
        // the hypothesis-layer authority token must never appear in a dream packet
        assert!(!json.contains("hypothesis_only"));
        let p = dream_packet(&fixture()).unwrap();
        assert_eq!(p.forbidden_uses.len(), 6);
    }

    #[test]
    fn dream_authority_has_exactly_one_variant() {
        let a = DreamAuthority::DreamOnly;
        match a {
            DreamAuthority::DreamOnly => {}
        }
    }

    #[test]
    fn dream_forbidden_uses_are_canonical_six() {
        assert_eq!(
            DREAM_FORBIDDEN_USES,
            [
                "ground_claim",
                "serve_as_evidence",
                "mutate_reading_memory",
                "alter_verifier_receipt",
                "change_training_gate",
                "bypass_codec_or_governance",
            ]
        );
    }

    #[test]
    fn dream_boundary_lines_present() {
        let p = dream_packet(&fixture()).unwrap();
        assert_eq!(p.boundary.len(), 9);
        assert_eq!(p.boundary[0], "The dream engine distorts.");
        assert_eq!(p.boundary[8], "Nothing trains.");
    }

    #[test]
    fn dream_weirdness_dial_is_monotone() {
        let mut prev_links = 0usize;
        let mut prev_broken = 0usize;
        for w in 0..=MAX_WEIRDNESS {
            let mut input = fixture();
            input.weirdness = w;
            let p = dream_packet(&input).unwrap();
            assert!(p.impossible_links.len() >= prev_links);
            assert!(p.broken_assumptions.len() >= prev_broken);
            prev_links = p.impossible_links.len();
            prev_broken = p.broken_assumptions.len();
        }
    }

    #[test]
    fn dream_weirdness_out_of_range_refused() {
        let mut input = fixture();
        input.weirdness = 6;
        assert_eq!(
            dream_packet(&input),
            Err(DreamError::WeirdnessOutOfRange(6))
        );
    }

    #[test]
    fn dream_empty_frame_fails_closed() {
        let mut input = fixture();
        input.frame_text = "   \n  \n".to_string();
        assert_eq!(dream_packet(&input), Err(DreamError::EmptyFrame));
    }

    #[test]
    fn dream_empty_corpus_fails_closed() {
        let input = DreamInput {
            documents: vec![doc("blank", "   "), doc("blank2", "")],
            frame_text: "An assumption.".to_string(),
            seed: 1,
            weirdness: 0,
        };
        assert_eq!(dream_packet(&input), Err(DreamError::EmptyCorpus));
    }

    #[test]
    fn dream_probe_requests_do_not_execute() {
        let p = dream_packet(&fixture()).unwrap();
        assert!(!p.probe_requests.is_empty());
        for pr in &p.probe_requests {
            assert!(!pr.executes);
            assert_eq!(pr.status, "requires_operator_review");
        }
    }

    #[test]
    fn dream_all_five_operators_are_registered() {
        assert_eq!(OPERATORS.len(), 5);
    }
}
