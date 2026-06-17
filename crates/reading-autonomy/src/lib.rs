//! reading-autonomy — READ-6, reader autonomy v0.
//!
//! A deterministic, BOUNDED reader generates a reading plan from corpus metadata
//! and routes every proposed action through the hardened P9 codec. The reader
//! proposes; the codec validates; the substrate executes; the verifier
//! authorizes; replay records. Weights remain untouched — no model, no training.

#![forbid(unsafe_code)]

mod budgeted;
mod ranked;
mod reader;
mod section;

pub use budgeted::read_budgeted;
pub use ranked::read_ranked;
pub use reader::{read, ReaderBounds, ReaderOutcome};
pub use section::read_section_ranked;

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

    // --- READ-9: title-aware deterministic relevance ranking (read_ranked) ---

    /// A two-document corpus whose question-relevant document (its title matches
    /// the question) is filed SECOND, behind an irrelevant first document.
    fn relevant_doc_second() -> (reading_substrate::Corpus, &'static str) {
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document("daily_log", &["The office opened at nine."]);
        corpus.add_document("wind_forecast", &["Winds will reach forty miles per hour."]);
        (corpus, "What is the wind forecast?")
    }

    #[test]
    fn ranked_reader_prioritizes_a_title_relevant_document() {
        // Tight 1-span budget. The blunt budgeted reader reads in metadata order,
        // so it reads the irrelevant first document and misses. The title-aware
        // ranked reader reads the title-relevant document first and answers.
        let (corpus, question) = relevant_doc_second();
        let tight = ReaderBounds {
            max_spans: 1,
            ..Default::default()
        };
        let budgeted = read_budgeted(&corpus, question, tight);
        let ranked = read_ranked(&corpus, question, tight);
        assert!(
            !budgeted.finalized(),
            "blunt metadata order reads the irrelevant first doc ⇒ coverage miss"
        );
        assert_eq!(
            ranked.answer(),
            Some("Winds will reach forty miles per hour."),
            "title rank reaches the relevant document within the same 1-span budget"
        );
    }

    #[test]
    fn ranked_reader_is_stable_across_file_order() {
        // Distinct titles ⇒ the (relevance, title) sort key is a total order
        // independent of insertion order, so permuting the documents yields the
        // identical ranked answer.
        let question = "What is the wind forecast?";
        let mut forward = reading_substrate::Corpus::new();
        forward.add_document("daily_log", &["The office opened at nine."]);
        forward.add_document("wind_forecast", &["Winds will reach forty miles per hour."]);
        let mut reverse = reading_substrate::Corpus::new();
        reverse.add_document("wind_forecast", &["Winds will reach forty miles per hour."]);
        reverse.add_document("daily_log", &["The office opened at nine."]);

        let a = read_ranked(&forward, question, ReaderBounds::default());
        let b = read_ranked(&reverse, question, ReaderBounds::default());
        assert_eq!(
            a.answer(),
            Some("Winds will reach forty miles per hour."),
            "the title-relevant sentence is the answer"
        );
        assert_eq!(
            a.answer(),
            b.answer(),
            "ranking is stable across file order"
        );
    }

    #[test]
    fn title_match_does_not_fabricate_a_claim_from_an_irrelevant_span() {
        // The document TITLE matches the question ("wind"), so ranking reads its
        // span first — but the span TEXT is not question-relevant, so it is not
        // claimed. A title match only reorders reads; it never grounds a claim, so
        // it cannot fabricate support.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document("wind_report", &["The committee adjourned at noon."]);
        let ranked = read_ranked(
            &corpus,
            "What is the wind forecast?",
            ReaderBounds::default(),
        );
        assert_eq!(ranked.spans_read, 1, "the title-ranked span was read");
        assert!(
            !ranked.finalized(),
            "an irrelevant span is not claimed even when the title matches"
        );
        assert_eq!(ranked.answer(), None);
    }

    #[test]
    fn ranked_reader_is_deterministic() {
        let (corpus, question) = relevant_doc_second();
        let a = read_ranked(&corpus, question, ReaderBounds::default());
        let b = read_ranked(&corpus, question, ReaderBounds::default());
        assert_eq!(a.plan, b.plan, "ranking is deterministic ⇒ replayable");
        assert_eq!(a.answer(), b.answer());
    }

    #[test]
    fn ranked_reader_matches_budgeted_under_a_loose_budget() {
        // With a budget wide enough to read everything, ranking only REORDERS the
        // reads — it drops and adds nothing — so the ranked answer equals the
        // budgeted answer (no regression). Here both relevant sentences are claimed.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document("bridge_report", &["The bridge inspection found cracks."]);
        corpus.add_document("road_notice", &["The road was closed for repairs."]);
        let question = "What happened to the bridge and road?";
        let budgeted = read_budgeted(&corpus, question, ReaderBounds::default());
        let ranked = read_ranked(&corpus, question, ReaderBounds::default());
        assert_eq!(
            budgeted.answer(),
            ranked.answer(),
            "a loose budget ⇒ ranking only reorders, same claims as budgeted"
        );
        assert!(ranked.finalized());
    }

    // --- READ-10: section-aware, multi-term ranking (read_section_ranked) ---

    #[test]
    fn section_ranking_prioritizes_a_heading_relevant_section() {
        // One document, two sections; the question-relevant section's HEADING
        // matches but it is filed second. Under a 1-span budget the budgeted reader
        // reads the first (irrelevant) section's span and misses; the section-aware
        // reader reads the heading-relevant section first and answers.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document_with_sections(
            "bulletin",
            &[
                ("general notes", &["The office opened at nine."]),
                (
                    "storm wind forecast",
                    &["Winds will reach forty miles per hour."],
                ),
            ],
        );
        let question = "What is the storm wind forecast?";
        let tight = ReaderBounds {
            max_spans: 1,
            ..Default::default()
        };
        let budgeted = read_budgeted(&corpus, question, tight);
        let sectioned = read_section_ranked(&corpus, question, tight);
        assert!(
            !budgeted.finalized(),
            "metadata order reads the irrelevant first section ⇒ coverage miss"
        );
        assert_eq!(
            sectioned.answer(),
            Some("Winds will reach forty miles per hour."),
            "the heading-relevant section is reached first within the same budget"
        );
    }

    #[test]
    fn section_ranking_prefers_the_section_covering_more_query_terms() {
        // Both sections' headings share the single token "wind", so single-token
        // overlap cannot distinguish them. The multi-term score (distinct query
        // terms covered by title+heading) ranks the "storm wind warning" section
        // (3 terms) above "wind notes" (1 term), recovering an answer the blunt
        // metadata order misses.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document_with_sections(
            "alerts",
            &[
                ("wind notes", &["Breezes stayed calm all afternoon."]),
                (
                    "storm wind warning",
                    &["A severe storm wind warning is in effect tonight."],
                ),
            ],
        );
        let question = "Is there a storm wind warning?";
        let tight = ReaderBounds {
            max_spans: 1,
            ..Default::default()
        };
        let budgeted = read_budgeted(&corpus, question, tight);
        let sectioned = read_section_ranked(&corpus, question, tight);
        assert!(
            !budgeted.finalized(),
            "metadata order reads the single-term section ⇒ coverage miss"
        );
        assert_eq!(
            sectioned.answer(),
            Some("A severe storm wind warning is in effect tonight."),
            "the section covering more query terms is read first"
        );
    }

    #[test]
    fn section_ranking_is_stable_across_section_order() {
        // Distinct headings ⇒ the (score, title, heading) key is a total order
        // independent of section insertion order, so permuting the sections yields
        // the identical answer.
        let question = "What is the storm wind forecast?";
        let tight = ReaderBounds {
            max_spans: 1,
            ..Default::default()
        };
        let mut forward = reading_substrate::Corpus::new();
        forward.add_document_with_sections(
            "bulletin",
            &[
                ("general notes", &["The office opened at nine."]),
                (
                    "storm wind forecast",
                    &["Winds will reach forty miles per hour."],
                ),
            ],
        );
        let mut reverse = reading_substrate::Corpus::new();
        reverse.add_document_with_sections(
            "bulletin",
            &[
                (
                    "storm wind forecast",
                    &["Winds will reach forty miles per hour."],
                ),
                ("general notes", &["The office opened at nine."]),
            ],
        );
        let a = read_section_ranked(&forward, question, tight);
        let b = read_section_ranked(&reverse, question, tight);
        assert_eq!(
            a.answer(),
            Some("Winds will reach forty miles per hour."),
            "the heading-relevant section is the answer"
        );
        assert_eq!(a.answer(), b.answer(), "stable across section order");
    }

    #[test]
    fn section_heading_match_does_not_fabricate_a_claim() {
        // The section HEADING matches the question ("wind"), so ranking reads its
        // span first — but the span TEXT is irrelevant, so it is not claimed. A
        // heading match only reorders reads; it never grounds a claim, so a ranking
        // signal can never become evidence.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document_with_sections(
            "log",
            &[("wind report", &["The committee adjourned at noon."])],
        );
        let sectioned = read_section_ranked(
            &corpus,
            "What is the wind forecast?",
            ReaderBounds::default(),
        );
        assert_eq!(sectioned.spans_read, 1, "the heading-ranked span was read");
        assert!(
            !sectioned.finalized(),
            "an irrelevant span is not claimed even when the heading matches"
        );
        assert_eq!(sectioned.answer(), None);
    }

    #[test]
    fn section_ranking_matches_budgeted_on_a_flat_corpus() {
        // A flat corpus (one headingless section per document) gives the section
        // score = title score, so the section reader degrades to title ranking and,
        // under a loose budget, makes exactly the budgeted reader's claims.
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document("bridge_report", &["The bridge inspection found cracks."]);
        corpus.add_document("road_notice", &["The road was closed for repairs."]);
        let question = "What happened to the bridge and road?";
        let budgeted = read_budgeted(&corpus, question, ReaderBounds::default());
        let sectioned = read_section_ranked(&corpus, question, ReaderBounds::default());
        assert_eq!(
            budgeted.answer(),
            sectioned.answer(),
            "flat corpus ⇒ section ranking reduces to the budgeted claims"
        );
        assert!(sectioned.finalized());
    }

    #[test]
    fn section_reader_is_deterministic() {
        let mut corpus = reading_substrate::Corpus::new();
        corpus.add_document_with_sections(
            "bulletin",
            &[
                ("general notes", &["The office opened at nine."]),
                (
                    "storm wind forecast",
                    &["Winds will reach forty miles per hour."],
                ),
            ],
        );
        let question = "What is the storm wind forecast?";
        let a = read_section_ranked(&corpus, question, ReaderBounds::default());
        let b = read_section_ranked(&corpus, question, ReaderBounds::default());
        assert_eq!(
            a.plan, b.plan,
            "section ranking is deterministic ⇒ replayable"
        );
        assert_eq!(a.answer(), b.answer());
    }
}
