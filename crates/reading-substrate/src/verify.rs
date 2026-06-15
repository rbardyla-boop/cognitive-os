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

    // 1. Grounding (READ-1 + READ-2): every claim cites ≥1 existing source span,
    //    AND its statement is a complete SENTENCE-LEVEL unit of a SINGLE cited
    //    span. A claim is NOT grounded merely because it points at a read span
    //    (READ-1: a fabricated statement must fail), and NOT grounded merely
    //    because it is a verbatim sub-fragment of a span (READ-2: an arbitrary
    //    fragment like "Bridge A", or a negation-adjacent fragment like
    //    "using Bridge A" lifted from "advised against using Bridge A", must
    //    fail). The statement must equal a contiguous run of one or more of a
    //    single cited span's complete sentence units; spans are checked
    //    individually (never concatenated). Deterministic floor: minimal
    //    whitespace/case normalization + sentence-boundary-aligned literal
    //    support. No semantic entailment, no paraphrase, no model judge.
    let mut grounded = true;
    for c in &run.memory.claims {
        if c.source_spans.is_empty() {
            grounded = false;
            problems.push(format!("claim {} has no source span", c.id));
            continue;
        }
        let mut supported_by_a_span = false;
        let mut all_spans_exist = true;
        for s in &c.source_spans {
            match corpus.read_span(*s) {
                Some(span) => {
                    if sentence_aligned(span.text(), &c.statement) {
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
                "claim {} is not a complete sentence-level unit of any single cited span",
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

/// Minimal, deterministic normalization for the grounding floor: collapse every
/// whitespace run to a single space, trim, and lowercase. Deliberately NOT
/// semantic — no stemming, synonyms, or paraphrase handling.
fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Split normalized `text` into complete sentence units — each ending at a
/// sentence terminator (`.`/`!`/`?`), plus a trailing unit if the text does not
/// end on one. Purely lexical; no semantics.
fn sentence_units(text: &str) -> Vec<String> {
    let normalized = normalize(text);
    let mut units = Vec::new();
    let mut current = String::new();
    for ch in normalized.chars() {
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

/// READ-2 sentence-fidelity: a claim is grounded by a span iff its normalized
/// statement equals a contiguous run of one or more of that span's complete
/// sentence units. This rejects arbitrary verbatim fragments (a claim must be a
/// sentence-level support unit, not any substring) while accepting full-sentence
/// claims. Deterministic — no entailment, no paraphrase, no model judge.
fn sentence_aligned(span_text: &str, claim: &str) -> bool {
    let needle = normalize(claim);
    if needle.is_empty() {
        return false;
    }
    let units = sentence_units(span_text);
    for start in 0..units.len() {
        let mut joined = String::new();
        for unit in &units[start..] {
            if !joined.is_empty() {
                joined.push(' ');
            }
            joined.push_str(unit);
            if joined == needle {
                return true;
            }
            if joined.len() > needle.len() {
                break;
            }
        }
    }
    false
}
