//! READ-6 — the deterministic, bounded reader.
//!
//! The reader PROPOSES; it never authorizes. It sees corpus METADATA first
//! (titles + span ids — not the text), then proposes a BOUNDED reading plan as
//! UNTRUSTED text, which is routed ONLY through `reading_codec::decode`. The codec
//! validates the plan, executes it through the substrate, and finalizes an answer
//! only if the READ-1/READ-2 verifier approves. The reader holds no executor or
//! verifier handle and cannot finalize on its own.
//!
//! v0 strategy (deterministic — no model, no training): inspect metadata, read up
//! to `max_spans` spans by id (bounded), claim each span's text verbatim (one
//! sentence per span ⇒ READ-2 grounded), and make one bounded finalize attempt
//! synthesizing the read sentences. A smarter reader is a future, gated step; the
//! point here is the bounded propose → codec → verifier loop, not intelligence.

use reading_codec::{decode, CodecError, CodecPolicy, Decoded};
use reading_substrate::{Corpus, SpanId};

/// Hard bounds on an autonomous read — it can never run unbounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReaderBounds {
    /// Total proposed actions.
    pub max_steps: usize,
    /// Spans the reader may read (it never inspects all text at once).
    pub max_spans: usize,
    /// Synthesize attempts.
    pub max_finalize_attempts: usize,
}

impl Default for ReaderBounds {
    fn default() -> Self {
        ReaderBounds {
            max_steps: 64,
            max_spans: 8,
            max_finalize_attempts: 1,
        }
    }
}

/// The result of one autonomous read.
#[derive(Debug)]
pub struct ReaderOutcome {
    /// The untrusted plan the reader proposed (audit trail).
    pub plan: String,
    pub steps: usize,
    pub spans_read: usize,
    pub finalize_attempts: usize,
    /// The codec's decision over the proposed plan. The reader cannot finalize
    /// except through this.
    pub decision: Result<Decoded, CodecError>,
}

impl ReaderOutcome {
    /// Whether the codec finalized a verifier-approved answer.
    pub fn finalized(&self) -> bool {
        matches!(&self.decision, Ok(d) if d.finalized.is_some())
    }

    /// The finalized answer text, if any.
    pub fn answer(&self) -> Option<&str> {
        match &self.decision {
            Ok(d) => d.finalized.as_ref().map(|r| r.proof.answer_text.as_str()),
            Err(_) => None,
        }
    }
}

/// Autonomously read `corpus` for `question` within `bounds`, routing the
/// proposed plan through the hardened codec. Deterministic: same inputs → same
/// plan and decision (so replay reproduces).
pub fn read(corpus: &Corpus, question: &str, bounds: ReaderBounds) -> ReaderOutcome {
    // Metadata first: titles + span ids, never the text.
    let span_ids: Vec<u64> = corpus
        .metadata()
        .iter()
        .flat_map(|doc| doc.span_ids.iter().map(|s| s.0))
        .collect();

    let mut actions: Vec<serde_json::Value> = Vec::new();
    actions.push(serde_json::json!({ "action": "inspect_corpus" }));
    let mut steps = 1usize;
    let mut spans_read = 0usize;
    let mut claim_statements: Vec<String> = Vec::new();

    for span_id in span_ids {
        if spans_read >= bounds.max_spans {
            break;
        }
        // Each span costs a read + an extract; keep room for those plus a
        // finalize, so we never exceed the step budget.
        if steps + 3 > bounds.max_steps {
            break;
        }
        actions.push(serde_json::json!({ "action": "read_span", "span_id": span_id }));
        steps += 1;
        spans_read += 1;

        // One sentence per span (READ-5) ⇒ the span text is a grounded claim.
        if let Some(span) = corpus.read_span(SpanId(span_id)) {
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

    // One bounded finalize attempt over the grounded claims.
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
