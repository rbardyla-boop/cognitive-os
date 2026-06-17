//! READ-7 — drive the autonomous reader over the READ-4 corpus pack and score it.
//!
//! For every committed READ-4 fixture we build the corpus exactly as `read0` does
//! (`corpus_from_documents` — one sentence per span) from its documents, then run
//! the deterministic READ-6 reader (`reading_autonomy::read`) against the
//! fixture's question — the fixture's hand-written `plan` is IGNORED. The reader
//! routes its own proposed plan through the hardened codec, so a finalized answer
//! is verifier-approved. We then CROSS-VALIDATE it: a fresh `reading_substrate::
//! verify` pass AND a separate `independently_grounded` check with different logic
//! must BOTH agree it is grounded, else it is flagged false-grounded — so the
//! measurement catches a `verify()` bug, not just trusts it. Each autonomous
//! outcome is compared to the fixture's COMMITTED manual
//! label, so the report shows manual-plan score vs autonomous-reader score.
//! Deterministic; no model, no training.

use reading_autonomy::{read, ReaderBounds};
use reading_cli::corpus_from_documents;
use reading_corpus_eval::{fixtures, Expected};
use reading_substrate::{verify, Corpus, ReadingRun};

/// An INDEPENDENT grounding cross-check that never calls `reading_substrate::
/// verify` (or its `sentence_aligned`). For READ-7's corpora — one sentence per
/// span (`corpus_from_documents`) — a grounded answer must be exactly the join of
/// its supporting claims, and each supporting claim must be the VERBATIM text of a
/// cited span. This uses different logic from the verifier (exact whole-span
/// equality, not contiguous-sentence-unit matching), so a `verify()` bug that
/// wrongly accepted a fragment would DISAGREE here and be flagged — the
/// "0 false-grounded" claim is cross-validated, not a same-function tautology.
pub fn independently_grounded(corpus: &Corpus, run: &ReadingRun) -> bool {
    // (1) The answer is exactly the join of its supporting claims' statements.
    let mut rendered: Vec<String> = Vec::new();
    for cid in &run.proof.supporting_claims {
        match run.memory.claim(*cid) {
            Some(claim) => rendered.push(claim.statement.clone()),
            None => return false,
        }
    }
    if rendered.join(" ") != run.proof.answer_text {
        return false;
    }
    // (2) Each supporting claim is the verbatim text of one of its cited spans.
    for cid in &run.proof.supporting_claims {
        let claim = match run.memory.claim(*cid) {
            Some(claim) => claim,
            None => return false,
        };
        let grounded = claim.source_spans.iter().any(|span_id| {
            corpus
                .read_span(*span_id)
                .map(|span| span.text().trim() == claim.statement.trim())
                .unwrap_or(false)
        });
        if !grounded {
            return false;
        }
    }
    true
}

/// What the autonomous reader did with a fixture's corpus + question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutonomousOutcome {
    /// A verifier-approved, source-grounded answer was finalized (and
    /// independently re-verified).
    Verified {
        answer: String,
        trace_hash: u64,
        spans_read: usize,
    },
    /// No verified answer was produced.
    Rejected { reason: String },
}

/// How the autonomous outcome compares to the committed manual (hand-plan) label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    /// Manual verified, autonomous verified.
    BothVerified,
    /// Manual rejected, autonomous rejected.
    BothRejected,
    /// Manual rejected (an adversarial hand-plan), autonomous verified a grounded
    /// answer — a SAFE divergence: the reader is non-adversarial, it does not
    /// reproduce malformed/fabricated/fragment plans.
    AutonomousVerifiedManualRejected,
    /// Manual verified, autonomous rejected — a FALSE-REJECT: the reader failed to
    /// produce an answer the hand-plan could (an engineering signal, never a
    /// training justification).
    AutonomousRejectedManualVerified,
}

/// One scored fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureScore {
    pub name: String,
    pub manual: Expected,
    pub outcome: AutonomousOutcome,
    pub comparison: Comparison,
    /// True iff the reader finalized an answer the independent verifier does NOT
    /// support — the unsafe class. MUST be empty across the pack.
    pub false_grounded: bool,
    /// The plan the autonomous reader proposed (never the fixture's hand-plan).
    pub autonomous_plan: String,
}

/// The autonomous-pack report: the autonomous score, the manual-vs-autonomous
/// comparison, and the explicit false-grounded / false-reject lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutonomousPackReport {
    pub total: usize,
    pub bounds: ReaderBounds,
    pub autonomous_verified: usize,
    pub autonomous_rejected: usize,
    pub manual_verified: usize,
    pub manual_rejected: usize,
    pub both_verified: usize,
    pub autonomous_verified_manual_rejected: usize,
    /// The unsafe class — MUST be empty.
    pub false_grounded: Vec<FixtureScore>,
    /// Allowed but classified: autonomous rejected where the hand-plan verified.
    pub false_rejects: Vec<FixtureScore>,
    pub scores: Vec<FixtureScore>,
}

/// Score the committed READ-4 fixture pack with the autonomous reader. Deterministic.
pub fn evaluate_autonomous_pack(bounds: ReaderBounds) -> AutonomousPackReport {
    let mut scores = Vec::new();
    let mut autonomous_verified = 0usize;
    let mut autonomous_rejected = 0usize;
    let mut manual_verified = 0usize;
    let mut manual_rejected = 0usize;
    let mut both_verified = 0usize;
    let mut autonomous_verified_manual_rejected = 0usize;
    let mut false_grounded = Vec::new();
    let mut false_rejects = Vec::new();

    for fixture in fixtures() {
        let docs: Vec<(String, String)> = fixture
            .documents
            .iter()
            .map(|(name, content)| (name.to_string(), content.to_string()))
            .collect();
        let corpus = corpus_from_documents(&docs);
        let result = read(&corpus, fixture.question, bounds);

        let (outcome, is_false_grounded) = match &result.decision {
            Ok(decoded) => match &decoded.finalized {
                Some(run) => {
                    // CROSS-VALIDATED false-grounded detection: a finalized answer
                    // must clear BOTH a fresh verify() pass AND an INDEPENDENT
                    // grounding check that does not reuse verify()'s logic. If they
                    // disagree (e.g. a verify() bug accepted a fragment), the
                    // answer is flagged false-grounded — the measurement is not a
                    // same-function tautology.
                    let report = verify(&corpus, run);
                    let is_false_grounded = !report.passed || !independently_grounded(&corpus, run);
                    (
                        AutonomousOutcome::Verified {
                            answer: run.proof.answer_text.clone(),
                            trace_hash: run.answer_hash,
                            spans_read: result.spans_read,
                        },
                        is_false_grounded,
                    )
                }
                None => (
                    AutonomousOutcome::Rejected {
                        reason: "accepted partial (no finalized answer)".to_string(),
                    },
                    false,
                ),
            },
            Err(error) => (
                AutonomousOutcome::Rejected {
                    reason: format!("codec rejected the autonomous plan: {error:?}"),
                },
                false,
            ),
        };

        let autonomous_is_verified = matches!(outcome, AutonomousOutcome::Verified { .. });
        let manual_is_verified = fixture.expected == Expected::Verified;
        if manual_is_verified {
            manual_verified += 1;
        } else {
            manual_rejected += 1;
        }
        if autonomous_is_verified {
            autonomous_verified += 1;
        } else {
            autonomous_rejected += 1;
        }

        let comparison = match (manual_is_verified, autonomous_is_verified) {
            (true, true) => Comparison::BothVerified,
            (false, false) => Comparison::BothRejected,
            (false, true) => Comparison::AutonomousVerifiedManualRejected,
            (true, false) => Comparison::AutonomousRejectedManualVerified,
        };
        match comparison {
            Comparison::BothVerified => both_verified += 1,
            Comparison::AutonomousVerifiedManualRejected => {
                autonomous_verified_manual_rejected += 1
            }
            _ => {}
        }

        let score = FixtureScore {
            name: fixture.name.to_string(),
            manual: fixture.expected,
            outcome,
            comparison,
            false_grounded: is_false_grounded,
            autonomous_plan: result.plan,
        };
        if is_false_grounded {
            false_grounded.push(score.clone());
        }
        if comparison == Comparison::AutonomousRejectedManualVerified {
            false_rejects.push(score.clone());
        }
        scores.push(score);
    }

    AutonomousPackReport {
        total: scores.len(),
        bounds,
        autonomous_verified,
        autonomous_rejected,
        manual_verified,
        manual_rejected,
        both_verified,
        autonomous_verified_manual_rejected,
        false_grounded,
        false_rejects,
        scores,
    }
}
