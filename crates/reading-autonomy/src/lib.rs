//! reading-autonomy — READ-6, reader autonomy v0.
//!
//! A deterministic, BOUNDED reader generates a reading plan from corpus metadata
//! and routes every proposed action through the hardened P9 codec. The reader
//! proposes; the codec validates; the substrate executes; the verifier
//! authorizes; replay records. Weights remain untouched — no model, no training.

#![forbid(unsafe_code)]

mod budgeted;
mod reader;

pub use budgeted::read_budgeted;
pub use reader::{read, ReaderBounds, ReaderOutcome};

#[cfg(test)]
mod tests {
    use super::*;
    use reading_codec::{decode, CodecPolicy, RejectKind};
    use reading_substrate::fixture;

    fn corpus_and_question() -> (reading_substrate::Corpus, String) {
        let (corpus, question, _) = fixture();
        (corpus, question)
    }

    #[test]
    fn reader_sees_metadata_before_spans() {
        let (corpus, question) = corpus_and_question();
        let outcome = read(&corpus, &question, ReaderBounds::default());
        let plan: serde_json::Value = serde_json::from_str(&outcome.plan).unwrap();
        let first = &plan.as_array().unwrap()[0];
        assert_eq!(
            first["action"], "inspect_corpus",
            "the reader inspects metadata first"
        );
        // every read_span appears after the inspect_corpus.
        let actions = plan.as_array().unwrap();
        let inspect_idx = actions
            .iter()
            .position(|a| a["action"] == "inspect_corpus")
            .unwrap();
        for (i, a) in actions.iter().enumerate() {
            if a["action"] == "read_span" {
                assert!(i > inspect_idx, "read_span must follow inspect_corpus");
            }
        }
    }

    #[test]
    fn reader_proposes_untrusted_codec_input_and_finalizes_through_it() {
        let (corpus, question) = corpus_and_question();
        let outcome = read(&corpus, &question, ReaderBounds::default());
        // The reader's only output is the untrusted plan; the decision is the
        // codec's, not the reader's.
        let _: serde_json::Value = serde_json::from_str(&outcome.plan).expect("plan is JSON");
        assert!(
            outcome.finalized(),
            "the grounded plan finalizes through the codec"
        );
    }

    #[test]
    fn claims_are_sentence_grounded() {
        // The default read finalizes only because each claim is a span sentence
        // (READ-2 grounded) — the codec/verifier authorized it.
        let (corpus, question) = corpus_and_question();
        let outcome = read(&corpus, &question, ReaderBounds::default());
        let run = match &outcome.decision {
            Ok(d) => d.finalized.as_ref().unwrap(),
            Err(e) => panic!("expected finalize, got {e:?}"),
        };
        assert!(!run.memory.claims.is_empty());
        for c in &run.memory.claims {
            assert!(!c.source_spans.is_empty(), "every claim is source-linked");
        }
    }

    #[test]
    fn bounded_max_spans() {
        let (corpus, question) = corpus_and_question();
        let two = read(
            &corpus,
            &question,
            ReaderBounds {
                max_spans: 2,
                ..Default::default()
            },
        );
        assert_eq!(two.spans_read, 2, "reads no more than max_spans");
        assert!(two.finalized());

        let none = read(
            &corpus,
            &question,
            ReaderBounds {
                max_spans: 0,
                ..Default::default()
            },
        );
        assert_eq!(none.spans_read, 0);
        assert!(
            !none.finalized(),
            "no spans read ⇒ nothing grounded to finalize"
        );
    }

    #[test]
    fn bounded_max_steps() {
        let (corpus, question) = corpus_and_question();
        // Budget for inspect + (read + extract) + synthesize = 4 → exactly 1 span.
        let tight = read(
            &corpus,
            &question,
            ReaderBounds {
                max_steps: 4,
                max_spans: 8,
                max_finalize_attempts: 1,
            },
        );
        assert_eq!(tight.spans_read, 1, "the step budget bounds the read");
        assert!(tight.steps <= 4);
        assert!(tight.finalized());
    }

    #[test]
    fn bounded_max_finalize_attempts() {
        let (corpus, question) = corpus_and_question();
        let no_finalize = read(
            &corpus,
            &question,
            ReaderBounds {
                max_finalize_attempts: 0,
                ..Default::default()
            },
        );
        assert_eq!(no_finalize.finalize_attempts, 0);
        assert!(!no_finalize.finalized(), "no finalize attempt ⇒ no answer");
    }

    #[test]
    fn fabricated_autonomous_claim_is_rejected() {
        // Even an autonomous reader cannot finalize a fabrication: a plan that
        // claims something the cited span does not support is rejected by the same
        // codec/verifier path the reader uses.
        let (corpus, question) = corpus_and_question();
        let fabricated = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":0},
            {"action":"extract_claim","statement":"Bridge A is perfectly safe to cross.","source_span_ids":[0]},
            {"action":"synthesize","answer_text":"Bridge A is perfectly safe to cross.","supporting_claims":[0]}
        ]"#;
        let err = decode(&corpus, &question, fabricated, CodecPolicy::strict()).unwrap_err();
        assert_eq!(err.kind(), RejectKind::Unverified);
    }

    // --- READ-8: the budgeted, selective reader (read_budgeted) ---

    fn weather_corpus() -> (reading_substrate::Corpus, &'static str) {
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document(
            "forecast",
            &[
                "Heavy rain is expected tonight.",
                "Winds will reach forty miles per hour.",
            ],
        );
        (corpus, "What is the wind forecast?")
    }

    #[test]
    fn budgeted_reader_is_selective_and_drops_irrelevant_spans() {
        let (corpus, question) = weather_corpus();
        let blunt = read(&corpus, question, ReaderBounds::default());
        let budgeted = read_budgeted(&corpus, question, ReaderBounds::default());
        // Blunt claims both sentences; budgeted claims only the wind-relevant one.
        assert_eq!(
            blunt.answer(),
            Some("Heavy rain is expected tonight. Winds will reach forty miles per hour.")
        );
        assert_eq!(
            budgeted.answer(),
            Some("Winds will reach forty miles per hour.")
        );
    }

    #[test]
    fn budgeted_reader_finalizes_through_the_codec() {
        let (corpus, question) = weather_corpus();
        let budgeted = read_budgeted(&corpus, question, ReaderBounds::default());
        assert!(
            budgeted.finalized(),
            "a relevant span grounds a verified answer"
        );
    }

    #[test]
    fn budgeted_read_budget_is_enforced() {
        let (corpus, question) = weather_corpus();
        // max_spans = 1: only the first span (irrelevant "Heavy rain ...") is read,
        // so nothing relevant is claimed → a coverage miss, never a false answer.
        let tight = read_budgeted(
            &corpus,
            question,
            ReaderBounds {
                max_spans: 1,
                ..Default::default()
            },
        );
        assert_eq!(tight.spans_read, 1, "the budget bounds reads");
        assert!(
            !tight.finalized(),
            "relevant span beyond budget ⇒ coverage miss"
        );
    }

    #[test]
    fn budgeted_reader_is_deterministic() {
        let (corpus, question) = weather_corpus();
        let a = read_budgeted(&corpus, question, ReaderBounds::default());
        let b = read_budgeted(&corpus, question, ReaderBounds::default());
        assert_eq!(a.plan, b.plan, "selection is deterministic ⇒ replayable");
        assert_eq!(a.answer(), b.answer());
    }

    #[test]
    fn budgeted_reader_preserves_negation_in_a_relevant_claim() {
        // The selective reader still claims WHOLE sentences verbatim, so a relevant
        // negated sentence keeps its "Do not" — selection never paraphrases.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document("advice", &["Do not cross the river during the flood."]);
        let budgeted = read_budgeted(&corpus, "Can I cross the river?", ReaderBounds::default());
        assert_eq!(
            budgeted.answer(),
            Some("Do not cross the river during the flood.")
        );
    }

    #[test]
    fn trace_replay_reproduces_the_result() {
        let (corpus, question) = corpus_and_question();
        let a = read(&corpus, &question, ReaderBounds::default());
        let b = read(&corpus, &question, ReaderBounds::default());
        assert_eq!(a.plan, b.plan, "the reader is deterministic");
        assert_eq!(a.answer(), b.answer());
        // Re-decoding the same plan reproduces the same verified run hashes.
        let ra = decode(&corpus, &question, &a.plan, CodecPolicy::strict())
            .unwrap()
            .finalized
            .unwrap();
        let rb = decode(&corpus, &question, &b.plan, CodecPolicy::strict())
            .unwrap()
            .finalized
            .unwrap();
        assert_eq!(ra.memory_hash, rb.memory_hash);
        assert_eq!(ra.answer_hash, rb.answer_hash);
    }
}
