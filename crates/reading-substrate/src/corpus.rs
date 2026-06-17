//! READ-0 — external text corpus as addressable spans.
//!
//! The corpus is the reading ENVIRONMENT, not context pasted into a model. The
//! reader sees document METADATA first (titles + span ids) and reads span text
//! only by id — never the whole corpus at once.

use std::collections::BTreeMap;

/// Stable identifier for a span of source text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpanId(pub u64);

/// A contiguous span of a document's source text, addressed by a stable id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub id: SpanId,
    pub document_id: u64,
    pub byte_start: usize,
    pub byte_end: usize,
    text: String,
}

impl Span {
    /// The span's source text (read deliberately, by id, through the corpus).
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Metadata about one document — exposed BEFORE any span text is read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentMeta {
    pub document_id: u64,
    pub title: String,
    pub span_ids: Vec<SpanId>,
}

/// An external text corpus: documents made of addressable spans.
#[derive(Clone, Debug, Default)]
pub struct Corpus {
    documents: Vec<DocumentMeta>,
    spans: BTreeMap<SpanId, Span>,
}

impl Corpus {
    pub fn new() -> Self {
        Corpus::default()
    }

    /// Add a document as a sequence of spans; returns the new document id. Span
    /// ids are assigned sequentially across the whole corpus (stable + unique).
    pub fn add_document(&mut self, title: &str, span_texts: &[&str]) -> u64 {
        let document_id = self.documents.len() as u64;
        let mut span_ids = Vec::new();
        let mut offset = 0usize;
        for text in span_texts {
            let id = SpanId(self.spans.len() as u64);
            let byte_start = offset;
            let byte_end = offset + text.len();
            offset = byte_end + 1;
            self.spans.insert(
                id,
                Span {
                    id,
                    document_id,
                    byte_start,
                    byte_end,
                    text: (*text).to_string(),
                },
            );
            span_ids.push(id);
        }
        self.documents.push(DocumentMeta {
            document_id,
            title: title.to_string(),
            span_ids,
        });
        document_id
    }

    /// Metadata first: document titles + span ids, with NO span text.
    pub fn metadata(&self) -> &[DocumentMeta] {
        &self.documents
    }

    /// Total number of spans (a metadata-level fact).
    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// Read one span's text by id (None if the id is unknown).
    pub fn read_span(&self, id: SpanId) -> Option<&Span> {
        self.spans.get(&id)
    }

    /// Whether a span id exists in the corpus.
    pub fn contains(&self, id: SpanId) -> bool {
        self.spans.contains_key(&id)
    }
}

/// Split `text` into sentence-like units, each ending at a sentence terminator
/// (`.`/`!`/`?`), plus a trailing unit if the text does not end on one. Each unit
/// is trimmed; empty units are dropped. Purely lexical (no normalization, no
/// semantics). This is the SINGLE source of sentence boundaries shared by the
/// corpus builder (one sentence per span) and the verifier (sentence-fidelity
/// grounding), so the spans a reader cites and the units the verifier checks can
/// never drift apart.
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut units = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let unit = current.trim();
            if !unit.is_empty() {
                units.push(unit.to_string());
            }
            current.clear();
        }
    }
    let tail = current.trim();
    if !tail.is_empty() {
        units.push(tail.to_string());
    }
    units
}
