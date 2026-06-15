//! P9 — the codec boundary: untrusted text in, verifier-gated decision out.
//!
//! The codec is the ONLY thing between a future model and the reading substrate,
//! and it holds three rules the model cannot talk its way past:
//!   1. parse + validate untrusted text into typed actions (no prose, no repair);
//!   2. execute accepted actions ONLY through `reading_substrate::execute` — the
//!      codec never mutates memory itself;
//!   3. finalize a synthesized answer ONLY if `reading_substrate::verify`
//!      approves it (grounded, supported, replayable).
//!
//! Determinism: `decode` is pure — no clock, no entropy, no IO — so the same
//! untrusted text always yields the same decision, and `Decoded` carries the
//! exact typed trace that was run (decisions are traceable).

use crate::error::CodecError;
use crate::parse::parse;
use crate::policy::CodecPolicy;
use reading_substrate::{execute, verify, Corpus, ReadingAction, ReadingRun, ReadingTrace};

/// A codec acceptance: the validated typed actions, plus the finalized run when
/// the sequence synthesized a verifier-approved answer.
#[derive(Clone, Debug)]
pub struct Decoded {
    /// The typed actions the model proposed, in order (the audit trail).
    pub actions: Vec<ReadingAction>,
    /// `Some` only when the sequence synthesized an answer the verifier approved.
    pub finalized: Option<ReadingRun>,
}

/// Decode untrusted model output against `corpus` for `question` under `policy`.
///
/// Returns the accepted decision, or the specific reason it was rejected. The
/// model text can never reach memory except as a validated trace executed by the
/// substrate, and can never finalize an answer the verifier rejects.
pub fn decode(
    corpus: &Corpus,
    question: &str,
    untrusted: &str,
    policy: CodecPolicy,
) -> Result<Decoded, CodecError> {
    let actions = parse(untrusted, &policy)?;
    validate(corpus, &actions, &policy)?;

    let mut trace = ReadingTrace::new();
    let mut synthesizes = false;
    for action in &actions {
        if matches!(action, ReadingAction::Synthesize { .. }) {
            synthesizes = true;
        }
        trace.push(action.clone());
    }

    // A sequence that does not synthesize proposes no answer to finalize; it is
    // accepted as a legal partial trace (the substrate stays the executor for any
    // later run). A sequence that DOES synthesize must clear the verifier gate.
    if !synthesizes {
        return Ok(Decoded {
            actions,
            finalized: None,
        });
    }

    let run = execute(corpus, question, &trace)
        .map_err(|e| CodecError::SubstrateRejected(format!("{e:?}")))?;
    let report = verify(corpus, &run);
    if policy.require_verified_finalize && !report.passed {
        return Err(CodecError::UnverifiedAnswer(report.problems));
    }
    Ok(Decoded {
        actions,
        finalized: Some(run),
    })
}

/// Boundary validation that runs BEFORE the substrate executes: every referenced
/// span id must exist in the corpus, and every extracted claim/entity must cite
/// at least one source span. These are defense-in-depth (the substrate also
/// enforces grounding during execution), giving a pre-execution rejection with a
/// precise reason.
fn validate(
    corpus: &Corpus,
    actions: &[ReadingAction],
    policy: &CodecPolicy,
) -> Result<(), CodecError> {
    for action in actions {
        match action {
            ReadingAction::ReadSpan(id) => {
                if !corpus.contains(*id) {
                    return Err(CodecError::UnknownSpan(id.0));
                }
            }
            ReadingAction::ExtractClaim { source_spans, .. }
            | ReadingAction::ExtractEntity { source_spans, .. } => {
                if policy.require_source_spans && source_spans.is_empty() {
                    return Err(CodecError::UngroundedProposal);
                }
                for id in source_spans {
                    if !corpus.contains(*id) {
                        return Err(CodecError::UnknownSpan(id.0));
                    }
                }
            }
            ReadingAction::InspectCorpus
            | ReadingAction::CompareClaims { .. }
            | ReadingAction::Synthesize { .. } => {}
        }
    }
    Ok(())
}
