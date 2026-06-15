//! READ-0 — the answer verifier (the authority boundary).
//!
//! An answer is not authoritative because the reader produced it; it is
//! authoritative only if it is grounded in source spans, every answer statement
//! is a cited grounded claim, and the reading trace replays to the same memory
//! and answer. Authority comes from verified source-linked memory, not from
//! reader confidence.

use crate::corpus::Corpus;
use crate::trace::{execute, ReadingRun};

/// The result of verifying a reading run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyReport {
    /// Every claim in memory cites ≥1 source span, and each span exists.
    pub grounded: bool,
    /// The answer text is exactly its cited (grounded) claims' statements.
    pub answer_supported: bool,
    /// Re-executing the saved trace reproduces the same memory and answer.
    pub replay_matches: bool,
    /// All three hold — the answer is authoritative.
    pub passed: bool,
    /// Human-readable reasons for any failure.
    pub problems: Vec<String>,
}

/// Verify a reading run against the corpus it was read from.
pub fn verify(corpus: &Corpus, run: &ReadingRun) -> VerifyReport {
    let mut problems = Vec::new();

    // 1. Grounding: every claim cites ≥1 existing source span, AND its statement
    //    is literally supported by a SINGLE cited span's TEXT. A claim is NOT
    //    grounded merely because it points at a span that was read — a fabricated
    //    statement citing a real span must fail. The statement must be a literal
    //    substring of at least one cited span on its own; spans are checked
    //    individually (never concatenated), so a statement cannot be "grounded"
    //    by text that straddles the join of two spans and exists in neither.
    //    Deterministic floor: minimal whitespace/case normalization + literal
    //    substring. No semantic entailment, no paraphrase acceptance, no model
    //    judge (those are later).
    let mut grounded = true;
    for c in &run.memory.claims {
        if c.source_spans.is_empty() {
            grounded = false;
            problems.push(format!("claim {} has no source span", c.id));
            continue;
        }
        let needle = normalize(&c.statement);
        let mut supported_by_a_span = false;
        let mut all_spans_exist = true;
        for s in &c.source_spans {
            match corpus.read_span(*s) {
                Some(span) => {
                    if normalize(span.text()).contains(&needle) {
                        supported_by_a_span = true;
                    }
                }
                None => {
                    grounded = false;
                    all_spans_exist = false;
                    problems.push(format!("claim {} cites unknown span {}", c.id, s.0));
                }
            }
        }
        if all_spans_exist && !supported_by_a_span {
            grounded = false;
            problems.push(format!(
                "claim {} is not supported by any single cited span's text",
                c.id
            ));
        }
    }

    // 2. Answer support: the answer text is exactly the cited claims' statements,
    //    and every cited claim exists and is grounded.
    let mut answer_supported = true;
    let mut rendered: Vec<String> = Vec::new();
    for cid in &run.proof.supporting_claims {
        match run.memory.claim(*cid) {
            Some(c) if !c.source_spans.is_empty() => rendered.push(c.statement.clone()),
            Some(_) => {
                answer_supported = false;
                problems.push(format!("answer cites ungrounded claim {cid}"));
            }
            None => {
                answer_supported = false;
                problems.push(format!("answer cites missing claim {cid}"));
            }
        }
    }
    if rendered.join(" ") != run.proof.answer_text {
        answer_supported = false;
        problems.push(
            "answer text is not exactly its cited claims' statements (unsupported content)"
                .to_string(),
        );
    }

    // 3. Replay: re-execute the saved trace; it must reproduce the run exactly.
    let replay_matches = match execute(corpus, &run.question, &run.trace) {
        Ok(replayed) => {
            let same = replayed.memory == run.memory
                && replayed.proof == run.proof
                && replayed.memory_hash == run.memory_hash
                && replayed.answer_hash == run.answer_hash;
            if !same {
                problems.push("replay did not reproduce the recorded memory/answer".to_string());
            }
            same
        }
        Err(e) => {
            problems.push(format!("replay of the trace failed: {e:?}"));
            false
        }
    };

    let passed = grounded && answer_supported && replay_matches;
    VerifyReport {
        grounded,
        answer_supported,
        replay_matches,
        passed,
        problems,
    }
}

/// Minimal, deterministic normalization for the literal-substring grounding
/// floor: collapse every whitespace run to a single space, trim, and lowercase.
/// Deliberately NOT semantic — no stemming, synonyms, or paraphrase handling.
fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
