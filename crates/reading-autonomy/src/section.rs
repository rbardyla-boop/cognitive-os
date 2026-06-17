//! READ-10 — section-aware, multi-term deterministic relevance ranking.
//!
//! `read_section_ranked` extends READ-9's title ranking with two richer, still
//! purely lexical signals: it ranks at the SECTION granularity and scores each
//! section by how many DISTINCT question terms are covered by the document TITLE
//! *or* the section HEADING together. So a section whose heading answers more of a
//! multi-term question is read before a section that only shares a single token,
//! and under a tight budget the relevant section is reached instead of missed.
//!
//! Both signals are **metadata-only**: titles and section headings are exposed
//! before any span text is read (a heading is never a span — no claim can cite
//! one), so the ranking never previews a span's text and a ranking score can never
//! become evidence. The claim FILTER is unchanged: reads still route through the
//! shared `read_selecting` core, so a span is claimed only if its OWN text is
//! lexically relevant AND grounds verbatim through the codec/verifier. Ranking
//! orders reads; it never grounds a claim. No model, semantics, entailment, or
//! paraphrase. On a flat corpus (one headingless section) the score reduces to the
//! title score, so this degrades cleanly to READ-9. Deterministic ⇒ replayable.

use crate::budgeted::{content_terms, prefix_overlap, read_selecting};
use crate::reader::{ReaderBounds, ReaderOutcome};
use reading_substrate::{Corpus, SpanId};

/// Autonomously read `corpus` for `question` within `bounds`, visiting spans in
/// SECTION-relevance order (title + heading, multi-term) so a budget reaches the
/// most relevant section first. Same selection, budget, and codec path as
/// `read_budgeted`; only the order differs. Deterministic.
pub fn read_section_ranked(corpus: &Corpus, question: &str, bounds: ReaderBounds) -> ReaderOutcome {
    let query = content_terms(question);
    let order = section_ranked_order(corpus, &query);
    read_selecting(corpus, question, &order, bounds)
}

/// One section's ranking record — borrows its metadata from the corpus. Holds only
/// metadata (title, heading, ids); never any span text.
struct RankedSection<'a> {
    score: usize,
    title: &'a str,
    heading: &'a str,
    document_id: u64,
    section_index: usize,
    span_ids: &'a [SpanId],
}

/// Order the corpus's span ids by SECTION relevance to `query`, then read each
/// section's spans in order. Metadata only — titles and section headings are known
/// before any span text is read, so this never previews text.
///
/// Sort key: `(combined_relevance DESC, title ASC, heading ASC, document_id ASC,
/// section_index ASC)`. The leading keys are independent of insertion order, so for
/// distinct (title, heading) pairs the ranking — and the selection — is stable
/// across any permutation of documents or sections; `document_id`/`section_index`
/// are only the final tiebreaks for otherwise-identical sections, keeping the
/// result deterministic/replayable.
fn section_ranked_order(corpus: &Corpus, query: &[String]) -> Vec<u64> {
    let mut sections: Vec<RankedSection> = Vec::new();
    for doc in corpus.metadata() {
        for (section_index, section) in doc.sections.iter().enumerate() {
            sections.push(RankedSection {
                score: combined_relevance(&doc.title, &section.heading, query),
                title: &doc.title,
                heading: &section.heading,
                document_id: doc.document_id,
                section_index,
                span_ids: &section.span_ids,
            });
        }
    }
    sections.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.title.cmp(b.title))
            .then_with(|| a.heading.cmp(b.heading))
            .then_with(|| a.document_id.cmp(&b.document_id))
            .then_with(|| a.section_index.cmp(&b.section_index))
    });
    sections
        .into_iter()
        .flat_map(|s| s.span_ids.iter().map(|id| id.0))
        .collect()
}

/// Multi-term, section-aware relevance: the number of query content terms that
/// share a deterministic word-prefix overlap with a term of the document TITLE or
/// the section HEADING. Purely lexical, computed from metadata only (never span
/// text). Because it counts the matched terms across title + heading, a section
/// covering more of a multi-term question scores higher than one sharing a single
/// token, so it is read earlier.
fn combined_relevance(title: &str, heading: &str, query: &[String]) -> usize {
    let mut meta_terms = content_terms(title);
    meta_terms.extend(content_terms(heading));
    query
        .iter()
        .filter(|q| meta_terms.iter().any(|t| prefix_overlap(q, t)))
        .count()
}
