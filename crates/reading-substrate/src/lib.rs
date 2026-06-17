//! reading-substrate — READ-0, the first reading-substrate bridge for Cognitive OS.
//!
//! A SEPARATE track from the deterministic vibe engine (it depends on no engine
//! crate). It treats external text as an addressable environment, builds
//! source-linked structured memory through a scripted (deterministic) reader,
//! verifies the answer's grounding, and replays the reading trace. No model
//! weights are trained; the scripted reader stands in for the eventual LLM
//! controller (the P9–P15 track). It reuses the project's record/replay
//! discipline without touching engine math or semantics.
//!
//! Load-bearing invariant: answer authority comes from verified source-linked
//! memory, not from reader confidence.

#![forbid(unsafe_code)]

mod corpus;
mod memory;
mod trace;
mod verify;

pub use corpus::{split_sentences, Corpus, DocumentMeta, SectionMeta, Span, SpanId};
pub use memory::{Claim, Entity, Memory, ProofObject};
pub use trace::{execute, ReadingAction, ReadingError, ReadingRun, ReadingTrace};
pub use verify::{verify, VerifyReport};

/// The READ-0 reference fixture: a small external corpus, a fixed question, and a
/// scripted deterministic reader (a `ReadingTrace`) that answers it.
pub fn fixture() -> (Corpus, String, ReadingTrace) {
    let mut corpus = Corpus::new();
    corpus.add_document(
        "bridge_safety_report",
        &[
            "Bridge A was reported structurally damaged after the June storm.",
            "Bridge B remained passable during light rain on the same day.",
            "Inspectors advised against using Bridge A until repairs are complete.",
        ],
    );
    corpus.add_document(
        "weather_log",
        &["The June storm brought heavy rain and high winds overnight."],
    );
    let question = "Which bridge is safe to cross after the storm?".to_string();

    // Claims are VERBATIM excerpts of their cited spans (the deterministic
    // grounding floor: a claim's statement must be a literal substring of the
    // cited span text). Paraphrase is intentionally not accepted yet.
    let claim_b = "Bridge B remained passable during light rain on the same day.";
    let claim_a = "Bridge A was reported structurally damaged after the June storm.";

    let mut trace = ReadingTrace::new();
    trace.push(ReadingAction::InspectCorpus);
    trace.push(ReadingAction::ReadSpan(SpanId(1)));
    trace.push(ReadingAction::ReadSpan(SpanId(0)));
    trace.push(ReadingAction::ReadSpan(SpanId(2)));
    trace.push(ReadingAction::ExtractClaim {
        statement: claim_b.to_string(),
        source_spans: vec![SpanId(1)],
    });
    trace.push(ReadingAction::ExtractClaim {
        statement: claim_a.to_string(),
        source_spans: vec![SpanId(0), SpanId(2)],
    });
    trace.push(ReadingAction::ExtractEntity {
        name: "Bridge B".to_string(),
        source_spans: vec![SpanId(1)],
    });
    trace.push(ReadingAction::ExtractEntity {
        name: "Bridge A".to_string(),
        source_spans: vec![SpanId(0), SpanId(2)],
    });
    trace.push(ReadingAction::CompareClaims { left: 0, right: 1 });
    trace.push(ReadingAction::Synthesize {
        answer_text: format!("{claim_b} {claim_a}"),
        supporting_claims: vec![0, 1],
    });

    (corpus, question, trace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read0_produces_a_verified_answer() {
        let (corpus, question, trace) = fixture();
        let run = execute(&corpus, &question, &trace).expect("scripted reader executes");
        let report = verify(&corpus, &run);
        assert!(
            report.passed,
            "answer must verify; problems: {:?}",
            report.problems
        );
        assert!(report.grounded && report.answer_supported && report.replay_matches);
    }

    #[test]
    fn metadata_is_inspected_before_any_span_is_read() {
        let (corpus, question, _) = fixture();
        let mut trace = ReadingTrace::new();
        trace.push(ReadingAction::ReadSpan(SpanId(0))); // read before inspecting metadata
        assert_eq!(
            execute(&corpus, &question, &trace),
            Err(ReadingError::MetadataNotInspectedFirst)
        );
    }

    #[test]
    fn a_claim_cannot_exist_without_a_source_span() {
        let (corpus, question, _) = fixture();
        let mut trace = ReadingTrace::new();
        trace.push(ReadingAction::InspectCorpus);
        trace.push(ReadingAction::ExtractClaim {
            statement: "ungrounded".to_string(),
            source_spans: vec![],
        });
        assert_eq!(
            execute(&corpus, &question, &trace),
            Err(ReadingError::UngroundedExtraction)
        );
    }

    #[test]
    fn a_claim_must_cite_a_span_that_was_actually_read() {
        let (corpus, question, _) = fixture();
        let mut trace = ReadingTrace::new();
        trace.push(ReadingAction::InspectCorpus);
        // cite span 0 without reading it first
        trace.push(ReadingAction::ExtractClaim {
            statement: "premature".to_string(),
            source_spans: vec![SpanId(0)],
        });
        assert_eq!(
            execute(&corpus, &question, &trace),
            Err(ReadingError::UnreadSpan(SpanId(0)))
        );
    }

    #[test]
    fn claims_are_source_linked_and_the_answer_is_from_claim_memory() {
        let (corpus, question, trace) = fixture();
        let run = execute(&corpus, &question, &trace).unwrap();
        assert!(!run.memory.claims.is_empty());
        for c in &run.memory.claims {
            assert!(!c.source_spans.is_empty(), "every claim is source-linked");
        }
        assert!(
            !run.proof.supporting_claims.is_empty(),
            "answer synthesized from claims"
        );
        // the reader selected spans by id, in reading order.
        assert_eq!(run.read_spans, vec![SpanId(1), SpanId(0), SpanId(2)]);
    }

    #[test]
    fn trace_replay_reproduces_the_same_memory_and_answer() {
        let (corpus, question, trace) = fixture();
        let a = execute(&corpus, &question, &trace).unwrap();
        let b = execute(&corpus, &question, &trace).unwrap();
        assert_eq!(a, b, "replay is deterministic");
        assert_eq!(a.memory_hash, b.memory_hash);
        assert_eq!(a.answer_hash, b.answer_hash);
    }

    // --- sabotage probes (each must fail the verifier) ---

    #[test]
    fn sabotage_removing_a_source_span_fails_grounding() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        run.memory.claims[0].source_spans.clear(); // strip the claim's evidence
        let report = verify(&corpus, &run);
        assert!(
            !report.grounded,
            "a claim with no source span must fail grounding"
        );
        assert!(!report.passed);
    }

    #[test]
    fn sabotage_reordering_the_trace_fails_replay() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        // swap the two ExtractClaim actions -> claim ids swap -> memory diverges.
        run.trace.actions.swap(4, 5);
        let report = verify(&corpus, &run);
        assert!(!report.replay_matches, "a reordered trace must fail replay");
        assert!(!report.passed);
    }

    #[test]
    fn sabotage_unsupported_answer_sentence_fails_support() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        run.proof
            .answer_text
            .push_str(" Bridge C is also perfectly safe.");
        let report = verify(&corpus, &run);
        assert!(
            !report.answer_supported,
            "an unsupported answer sentence must fail support"
        );
        assert!(!report.passed);
    }

    // --- claim fidelity (READ-1): a claim is grounded only if its statement is
    //     literally supported by its cited span text, not merely if it cites a
    //     real, read span ---

    #[test]
    fn verbatim_claim_is_grounded_by_cited_span_text() {
        let (corpus, question, trace) = fixture();
        let run = execute(&corpus, &question, &trace).unwrap();
        assert!(
            verify(&corpus, &run).grounded,
            "verbatim claims drawn from their cited spans are grounded"
        );
    }

    #[test]
    fn sabotage_fabricated_claim_citing_a_real_span_fails_fidelity() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        // Keep the real, read cited spans [0,2]; replace the statement with text
        // the cited spans do NOT support (in fact, the opposite of span 0). Under
        // the old structural-only grounding this verified; fidelity must reject.
        run.memory.claims[1].statement =
            "Bridge A is fully safe to cross after the storm.".to_string();
        let report = verify(&corpus, &run);
        assert!(
            !report.grounded,
            "a fabricated statement citing a real span must fail fidelity grounding"
        );
        assert!(!report.passed);
    }

    #[test]
    fn sabotage_cross_span_join_straddle_fails_fidelity() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        // claims[1] cites [0,2]. Replace its statement with text that straddles
        // the boundary between span 0 and span 2 — a substring of the two spans
        // CONCATENATED, but present in NO single cited span. Grounding must fail:
        // a claim must be supported by an actual span, not an artifact of joining.
        run.memory.claims[1].statement =
            "after the June storm. Inspectors advised against using Bridge A".to_string();
        let report = verify(&corpus, &run);
        assert!(
            !report.grounded,
            "a statement straddling the span join is in no single span and must fail fidelity"
        );
        assert!(!report.passed);
    }

    // --- sentence fidelity (READ-2): a claim must be a complete sentence-level
    //     unit of a cited span, not an arbitrary verbatim fragment ---

    #[test]
    fn full_sentence_claim_is_grounded() {
        let (corpus, question, trace) = fixture();
        // The canonical claims are full span sentences; they stay grounded.
        let run = execute(&corpus, &question, &trace).unwrap();
        assert!(verify(&corpus, &run).grounded);
    }

    #[test]
    fn sabotage_verbatim_fragment_fails_sentence_fidelity() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        // "Bridge A" is a verbatim substring of span 0 but not a full sentence.
        run.memory.claims[1].statement = "Bridge A".to_string();
        let report = verify(&corpus, &run);
        assert!(
            !report.grounded,
            "a sub-sentence fragment must fail sentence fidelity even though it is a substring"
        );
        assert!(!report.passed);
    }

    #[test]
    fn sabotage_negation_adjacent_fragment_fails_sentence_fidelity() {
        let (corpus, question, trace) = fixture();
        let mut run = execute(&corpus, &question, &trace).unwrap();
        // claims[1] cites [0,2]; span 2 is "Inspectors advised against using
        // Bridge A until repairs are complete." The fragment drops the negation
        // and is a mid-sentence substring — it must fail sentence fidelity.
        run.memory.claims[1].statement = "using Bridge A until repairs are complete.".to_string();
        let report = verify(&corpus, &run);
        assert!(
            !report.grounded,
            "a negation-adjacent mid-sentence fragment must fail sentence fidelity"
        );
        assert!(!report.passed);
    }
}
