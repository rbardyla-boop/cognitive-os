//! READ-9 — title-aware deterministic relevance ranking.
//!
//! `read_ranked` is READ-8's budgeted selective reader with one change: instead of
//! visiting spans in raw metadata order, it ORDERS the documents by deterministic
//! TITLE relevance to the question first, then reads their spans in that order. So
//! under a tight budget a title-relevant document is reached BEFORE an irrelevant
//! one, instead of being missed because it happened to be filed later.
//!
//! The ranking is METADATA-ONLY: it scores the document TITLE (exposed before any
//! span text) with the SAME lexical word-prefix overlap READ-8 uses — no model, no
//! semantics, no entailment, no paraphrase, and crucially it never previews a
//! span's text before that span is read by id. The claim FILTER is unchanged: a
//! span is claimed only if its own text is relevant AND grounds verbatim through
//! the codec/verifier. A title match only changes READING ORDER, never whether
//! something becomes a claim, so a title match alone can never fabricate support.
//! Reads still route ONLY through the shared `read_selecting` core, so budget,
//! selection, and codec behaviour are identical to `read_budgeted`. Deterministic
//! ⇒ replayable.

use crate::budgeted::{content_terms, prefix_overlap, read_selecting};
use crate::reader::{ReaderBounds, ReaderOutcome};
use reading_substrate::{Corpus, DocumentMeta};

/// Autonomously read `corpus` for `question` within `bounds`, visiting spans in
/// TITLE-relevance order so a budget reaches the relevant document first. Same
/// selection, budget, and codec path as `read_budgeted`; only the order differs.
/// Deterministic: same inputs → same plan and decision.
pub fn read_ranked(corpus: &Corpus, question: &str, bounds: ReaderBounds) -> ReaderOutcome {
    let query = content_terms(question);
    let order = title_ranked_order(corpus, &query);
    read_selecting(corpus, question, &order, bounds)
}

/// Order the corpus's span ids by document TITLE relevance to `query`, then read
/// each document's spans in their existing order. Metadata only — the document
/// title is known before any span text is read, so this never previews text.
///
/// Sort key: `(title_relevance DESC, title ASC, document_id ASC)`. The primary and
/// secondary keys are independent of insertion order, so for DISTINCT titles the
/// ranking — and therefore the selection — is stable across any permutation of the
/// input documents. `document_id` is only the final tiebreak for two documents that
/// share both a title and a score, which keeps the result deterministic/replayable.
fn title_ranked_order(corpus: &Corpus, query: &[String]) -> Vec<u64> {
    let mut docs: Vec<&DocumentMeta> = corpus.metadata().iter().collect();
    docs.sort_by(|a, b| {
        let score_a = title_relevance(&a.title, query);
        let score_b = title_relevance(&b.title, query);
        score_b
            .cmp(&score_a)
            .then_with(|| a.title.cmp(&b.title))
            .then_with(|| a.document_id.cmp(&b.document_id))
    });
    docs.into_iter()
        .flat_map(|doc| doc.span_ids.iter().map(|s| s.0))
        .collect()
}

/// How relevant a document `title` is to the question: the number of query content
/// terms that share a deterministic word-prefix overlap with a title term. Purely
/// lexical, computed from the TITLE only (never span text). A higher score means
/// "read this document's spans earlier".
fn title_relevance(title: &str, query: &[String]) -> usize {
    let title_terms = content_terms(title);
    query
        .iter()
        .filter(|q| title_terms.iter().any(|t| prefix_overlap(q, t)))
        .count()
}
