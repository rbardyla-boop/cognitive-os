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
/// is trimmed; empty units are dropped.
///
/// READ-5 hardening — `.` is treated as a real sentence boundary using only
/// DETERMINISTIC, LEXICAL signals (no dictionaries of meaning, no entailment, no
/// model). A period is NOT a boundary when it is (a) inside a decimal/version
/// (`3.14`, `v1.2.3` — digit before and after), (b) part of a known abbreviation
/// (`Dr.`, `Mr.`, `etc.` — a small fixed list), or (c)/(d) a single-letter token
/// (an acronym letter) immediately followed by a letter (`U.S`, `e.g`) or by a
/// lowercase continuation (`U.S. economy`). A genuine multi-letter word always
/// ends the sentence, so `attempt. and ...` still splits. `!`/`?` are always
/// boundaries. This is the SINGLE source of sentence boundaries shared by the
/// corpus builder (one sentence per span) and the verifier (sentence-fidelity
/// grounding), so spans and grounding can never drift apart.
pub fn split_sentences(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut units = Vec::new();
    let mut start = 0usize;
    for i in 0..chars.len() {
        let is_boundary = match chars[i] {
            '!' | '?' => true,
            '.' => is_period_boundary(&chars, i),
            _ => false,
        };
        if is_boundary {
            push_unit(&chars, start, i + 1, &mut units);
            start = i + 1;
        }
    }
    push_unit(&chars, start, chars.len(), &mut units);
    units
}

/// Push `chars[from..to]` as a trimmed, non-empty unit.
fn push_unit(chars: &[char], from: usize, to: usize, units: &mut Vec<String>) {
    if from >= to {
        return;
    }
    let unit: String = chars[from..to].iter().collect();
    let unit = unit.trim();
    if !unit.is_empty() {
        units.push(unit.to_string());
    }
}

/// Whether the `.` at index `i` is a real sentence boundary (deterministic).
fn is_period_boundary(chars: &[char], i: usize) -> bool {
    let prev = if i > 0 {
        chars.get(i - 1).copied()
    } else {
        None
    };
    let next = chars.get(i + 1).copied();

    // (a) decimal / version: a period between two digits.
    if let (Some(p), Some(n)) = (prev, next) {
        if p.is_ascii_digit() && n.is_ascii_digit() {
            return false;
        }
    }

    // The alphabetic word ending immediately before the period.
    let mut j = i;
    while j > 0 && chars[j - 1].is_ascii_alphabetic() {
        j -= 1;
    }
    let word: String = chars[j..i].iter().collect();

    // (b) a known abbreviation.
    if !word.is_empty() && is_abbreviation(&word.to_ascii_lowercase()) {
        return false;
    }

    // (c)/(d) a single-letter token — an initial or the trailing letter of an
    //     acronym. It is NOT a boundary when (c) immediately followed by a letter
    //     ("U.S", "e.g") or (d) the next non-space char is a lowercase continuation
    //     ("U.S. economy"). It IS a boundary before a capitalised next sentence
    //     ("Cross Bridge A. Avoid ..."). Rule (d) is scoped to single-letter tokens
    //     ON PURPOSE: a genuine multi-letter word always ends the sentence, so a
    //     real boundary before a lowercase word ("attempt. and try again.") still
    //     splits — only known abbreviations (b) and acronym letters are held back.
    if word.chars().count() == 1 {
        if let Some(n) = next {
            if n.is_ascii_alphabetic() {
                return false;
            }
        }
        let mut k = i + 1;
        while k < chars.len() && chars[k].is_whitespace() {
            k += 1;
        }
        if let Some(&n) = chars.get(k) {
            if n.is_lowercase() {
                return false;
            }
        }
    }

    true
}

/// A small, fixed, deterministic list of abbreviations whose trailing period is
/// not a sentence boundary. Deliberately excludes ambiguous words (e.g. "no")
/// that are also common sentence-ending words. Lexical only — not semantic.
fn is_abbreviation(word_lower: &str) -> bool {
    matches!(
        word_lower,
        "dr" | "mr"
            | "mrs"
            | "ms"
            | "prof"
            | "st"
            | "jr"
            | "sr"
            | "etc"
            | "vs"
            | "inc"
            | "ltd"
            | "co"
            | "mt"
            | "rev"
            | "gen"
            | "capt"
            | "lt"
            | "sgt"
            | "dept"
            | "fig"
            | "vol"
            | "ave"
    )
}

#[cfg(test)]
mod tests {
    use super::split_sentences;

    #[test]
    fn normal_sentences_still_split() {
        assert_eq!(
            split_sentences("First sentence. Second sentence."),
            vec!["First sentence.", "Second sentence."]
        );
    }

    #[test]
    fn abbreviation_us_does_not_split() {
        assert_eq!(
            split_sentences("The U.S. economy is strong this year."),
            vec!["The U.S. economy is strong this year."]
        );
    }

    #[test]
    fn titles_dr_mr_do_not_split() {
        assert_eq!(
            split_sentences("Dr. Smith met Mr. Jones today."),
            vec!["Dr. Smith met Mr. Jones today."]
        );
    }

    #[test]
    fn eg_and_ie_do_not_split() {
        assert_eq!(
            split_sentences("Bring gear, e.g. a rope, i.e. something strong."),
            vec!["Bring gear, e.g. a rope, i.e. something strong."]
        );
    }

    #[test]
    fn decimals_and_versions_do_not_split() {
        assert_eq!(
            split_sentences("It rose 3.5 percent."),
            vec!["It rose 3.5 percent."]
        );
        assert_eq!(
            split_sentences("Release v1.2.3 shipped today."),
            vec!["Release v1.2.3 shipped today."]
        );
    }

    #[test]
    fn single_letter_sentence_end_before_capital_still_splits() {
        // "Bridge A." ends a sentence (next token is a capitalized new sentence),
        // so it must still split — the single-letter rule only suppresses an
        // initial/acronym immediately followed by a letter.
        assert_eq!(
            split_sentences("Cross Bridge A. Avoid Bridge B."),
            vec!["Cross Bridge A.", "Avoid Bridge B."]
        );
    }

    #[test]
    fn exclamation_and_question_always_split() {
        assert_eq!(
            split_sentences("Is it safe? Yes! Cross now."),
            vec!["Is it safe?", "Yes!", "Cross now."]
        );
    }

    #[test]
    fn real_boundary_before_a_lowercase_word_still_splits() {
        // A multi-letter word ends the sentence even when the next word is
        // lowercase — rule (d) only holds back single-letter acronym tails, so a
        // genuine boundary is never merged away.
        assert_eq!(
            split_sentences("Do not attempt. and try again."),
            vec!["Do not attempt.", "and try again."]
        );
    }

    #[test]
    fn acronym_tail_before_lowercase_stays_merged() {
        // The trailing acronym letter ("S." in "U.S.") followed by a lowercase
        // continuation is held back, keeping the sentence whole.
        assert_eq!(
            split_sentences("The U.S. dollar fell."),
            vec!["The U.S. dollar fell."]
        );
    }
}
