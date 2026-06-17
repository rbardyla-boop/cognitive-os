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

/// A heading-labelled section of a document. The heading is METADATA (exposed
/// before any span text is read); it is NOT itself a span, so no claim can ever
/// cite or ground against a heading — section structure can rank reads, never
/// supply evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionMeta {
    pub heading: String,
    pub span_ids: Vec<SpanId>,
}

/// Metadata about one document — exposed BEFORE any span text is read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentMeta {
    pub document_id: u64,
    pub title: String,
    /// Every span of the document, in order (across all sections).
    pub span_ids: Vec<SpanId>,
    /// The document's sections, each a heading (metadata) plus its spans, in
    /// order. A document added without explicit sections has exactly one section
    /// with an empty heading containing every span.
    pub sections: Vec<SectionMeta>,
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

    /// Add a document as a flat sequence of spans (one headingless section);
    /// returns the new document id. Span ids are assigned sequentially across the
    /// whole corpus (stable + unique).
    pub fn add_document(&mut self, title: &str, span_texts: &[&str]) -> u64 {
        self.add_document_with_sections(title, &[("", span_texts)])
    }

    /// Add a document whose spans are grouped into heading-labelled SECTIONS;
    /// returns the new document id. The headings are METADATA only — they are
    /// never inserted as spans, so no claim can cite a heading. Span ids are
    /// assigned sequentially across the whole corpus (stable + unique), in section
    /// order then span order, so a single-section call is identical to a flat add.
    pub fn add_document_with_sections(&mut self, title: &str, sections: &[(&str, &[&str])]) -> u64 {
        let document_id = self.documents.len() as u64;
        let mut span_ids = Vec::new();
        let mut section_metas = Vec::new();
        let mut offset = 0usize;
        for (heading, span_texts) in sections {
            let mut section_span_ids = Vec::new();
            for text in *span_texts {
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
                section_span_ids.push(id);
            }
            section_metas.push(SectionMeta {
                heading: (*heading).to_string(),
                span_ids: section_span_ids,
            });
        }
        self.documents.push(DocumentMeta {
            document_id,
            title: title.to_string(),
            span_ids,
            sections: section_metas,
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
    use super::{split_sentences, Corpus, SpanId};

    #[test]
    fn plain_document_has_one_headingless_section_with_all_spans() {
        // A document added without sections still exposes a section view: exactly
        // one section, an empty heading, containing every span — so a section-aware
        // reader degrades cleanly to title-only ranking on flat corpora.
        let mut corpus = Corpus::new();
        corpus.add_document("notes", &["First sentence.", "Second sentence."]);
        let doc = &corpus.metadata()[0];
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "");
        assert_eq!(doc.sections[0].span_ids, doc.span_ids);
        assert_eq!(doc.span_ids, vec![SpanId(0), SpanId(1)]);
    }

    #[test]
    fn sectioned_document_exposes_headings_as_metadata_never_as_spans() {
        let mut corpus = Corpus::new();
        let id = corpus.add_document_with_sections(
            "report",
            &[
                ("overview", &["The bridge is open."]),
                ("wind forecast", &["Winds will reach forty miles per hour."]),
            ],
        );
        let doc = &corpus.metadata()[id as usize];
        assert_eq!(doc.sections.len(), 2);
        assert_eq!(doc.sections[0].heading, "overview");
        assert_eq!(doc.sections[1].heading, "wind forecast");
        // The flat span list is exactly the section spans, in order.
        let flat: Vec<SpanId> = doc
            .sections
            .iter()
            .flat_map(|s| s.span_ids.clone())
            .collect();
        assert_eq!(flat, doc.span_ids);
        // Headings are NOT spans: no addressable span's text equals a heading.
        for sid in &doc.span_ids {
            let text = corpus.read_span(*sid).unwrap().text();
            assert_ne!(text, "overview");
            assert_ne!(text, "wind forecast");
        }
    }

    #[test]
    fn sectioned_spans_keep_sequential_ids_like_a_flat_add() {
        // Two single-span sections produce the same span ids a two-span flat add
        // would, so sectioning is purely a metadata grouping over the same spans.
        let mut sectioned = Corpus::new();
        sectioned.add_document_with_sections("d", &[("a", &["One."]), ("b", &["Two."])]);
        let mut flat = Corpus::new();
        flat.add_document("d", &["One.", "Two."]);
        assert_eq!(
            sectioned.metadata()[0].span_ids,
            flat.metadata()[0].span_ids
        );
        assert_eq!(
            sectioned.read_span(SpanId(1)).unwrap().text(),
            flat.read_span(SpanId(1)).unwrap().text()
        );
    }

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
