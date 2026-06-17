//! READ-8 — the deterministic, budgeted, SELECTIVE reader.
//!
//! `read_budgeted` makes autonomy less blunt than READ-6's read-everything
//! strategy WITHOUT any model, semantics, entailment, or paraphrase. It still
//! sees corpus METADATA first, reads spans only by id (bounded by the budget — it
//! never previews text), and routes its proposed plan ONLY through the codec. The
//! difference: among the spans it reads, it CLAIMS only those LEXICALLY relevant
//! to the question — a deterministic word-prefix overlap against a small fixed
//! stopword list. So the answer is the question-relevant subset, not every
//! sentence. Reads are bounded by `max_spans`, so with a tight budget a relevant
//! span beyond the budget is simply never reached — a COVERAGE MISS (a classified
//! false-reject), never a false-grounded answer. Deterministic ⇒ replayable.

use crate::reader::{ReaderBounds, ReaderOutcome};
use reading_codec::{decode, CodecPolicy};
use reading_substrate::{Corpus, SpanId};

/// Autonomously read `corpus` for `question` within `bounds`, but CLAIM only the
/// spans lexically relevant to the question. Deterministic: same inputs → same
/// plan and decision.
pub fn read_budgeted(corpus: &Corpus, question: &str, bounds: ReaderBounds) -> ReaderOutcome {
    // Metadata first: titles + span ids, never the text.
    let span_ids: Vec<u64> = corpus
        .metadata()
        .iter()
        .flat_map(|doc| doc.span_ids.iter().map(|s| s.0))
        .collect();
    let query = content_terms(question);

    let mut actions: Vec<serde_json::Value> = Vec::new();
    actions.push(serde_json::json!({ "action": "inspect_corpus" }));
    let mut steps = 1usize;
    let mut spans_read = 0usize;
    let mut claim_statements: Vec<String> = Vec::new();

    for span_id in span_ids {
        // Budget on READS: the reader cannot inspect a span unless the budget
        // allows it. Keep room for a read (+ a possible extract + finalize).
        if spans_read >= bounds.max_spans {
            break;
        }
        if steps + 3 > bounds.max_steps {
            break;
        }
        actions.push(serde_json::json!({ "action": "read_span", "span_id": span_id }));
        steps += 1;
        spans_read += 1;

        // SELECT: claim the span only if it is lexically relevant to the question.
        if let Some(span) = corpus.read_span(SpanId(span_id)) {
            if !is_relevant(span.text(), &query) {
                continue;
            }
            let statement = span.text().to_string();
            actions.push(serde_json::json!({
                "action": "extract_claim",
                "statement": statement,
                "source_span_ids": [span_id],
            }));
            steps += 1;
            claim_statements.push(statement);
        }
    }

    let mut finalize_attempts = 0usize;
    if bounds.max_finalize_attempts >= 1 && !claim_statements.is_empty() && steps < bounds.max_steps
    {
        let answer = claim_statements.join(" ");
        let supporting: Vec<u64> = (0..claim_statements.len() as u64).collect();
        actions.push(serde_json::json!({
            "action": "synthesize",
            "answer_text": answer,
            "supporting_claims": supporting,
        }));
        steps += 1;
        finalize_attempts = 1;
    }

    let plan = serde_json::Value::Array(actions).to_string();
    let decision = decode(corpus, question, &plan, CodecPolicy::strict());
    ReaderOutcome {
        plan,
        steps,
        spans_read,
        finalize_attempts,
        decision,
    }
}

/// The content terms of `text`: lowercase alphanumeric words of length ≥ 3 that
/// are not common stopwords. Purely lexical — no stemming, synonyms, or meaning.
fn content_terms(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_ascii_lowercase())
        .filter(|w| w.len() >= 3 && !is_stopword(w))
        .collect()
}

/// Whether `span_text` shares a content term with the query by deterministic
/// word-prefix overlap (the shorter term, ≥ 3 chars, is a prefix of the longer —
/// so "wind" matches "winds" but "art" does not match "start"). Lexical only.
fn is_relevant(span_text: &str, query: &[String]) -> bool {
    if query.is_empty() {
        return false;
    }
    let span_terms = content_terms(span_text);
    query
        .iter()
        .any(|q| span_terms.iter().any(|s| prefix_overlap(q, s)))
}

/// True when the shorter of `a`/`b` (≥ 3 chars) is a prefix of the longer.
fn prefix_overlap(a: &str, b: &str) -> bool {
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    short.len() >= 3 && long.starts_with(short)
}

/// A small fixed list of common function words (length ≥ 3). Lexical, not
/// semantic — like READ-5's abbreviation list.
fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "and"
            | "for"
            | "was"
            | "are"
            | "will"
            | "what"
            | "which"
            | "who"
            | "whom"
            | "can"
            | "how"
            | "does"
            | "did"
            | "has"
            | "have"
            | "had"
            | "this"
            | "that"
            | "with"
            | "from"
            | "its"
            | "into"
            | "per"
            | "during"
            | "after"
            | "before"
            | "until"
            | "any"
            | "all"
            | "you"
            | "your"
            | "there"
            | "their"
            | "they"
            | "them"
            | "then"
            | "than"
            | "but"
            | "not"
            | "were"
            | "been"
            | "being"
    )
}
