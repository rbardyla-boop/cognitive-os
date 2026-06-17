//! reading-section-eval — READ-10, the section-aware / multi-term ranking eval.
//!
//! Measures the deterministic SECTION-RANKED reader (`read_section_ranked`) against
//! the READ-8 budgeted reader. On the flat committed READ-4 pack section ranking
//! reduces to title ranking and only REORDERS, so the eval proves NO-REGRESSION
//! (section answer == budgeted answer, 0 regressions) and 0 false-grounded
//! (cross-validated). The section-heading + multi-term WIN — reaching a
//! heading-relevant or more-terms-covered section first under a tight budget,
//! deterministically and stably across order — is measured by
//! `section_priority_demo`. Coverage misses are an engineering signal; the P12 gate
//! still owns weights. No model, no training.

#![forbid(unsafe_code)]

mod scorer;

pub use scorer::{
    evaluate_section_pack, section_priority_demo, SectionDemo, SectionOutcome, SectionReport,
    SectionScore,
};

#[cfg(test)]
mod tests {
    use super::*;
    use reading_autonomy::ReaderBounds;
    use reading_corpus_eval::fixtures;

    fn default_report() -> SectionReport {
        evaluate_section_pack(ReaderBounds::default())
    }

    #[test]
    fn runs_on_at_least_ten_fixtures() {
        let report = default_report();
        assert!(
            report.total >= 10,
            "READ-10 needs ≥ 10 fixtures, have {}",
            report.total
        );
        assert_eq!(report.total, fixtures().len());
    }

    #[test]
    fn zero_false_grounded() {
        let report = default_report();
        assert!(
            report.false_grounded.is_empty(),
            "a false-grounded section answer is the unsafe class: {:?}",
            report
                .false_grounded
                .iter()
                .map(|s| &s.name)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn no_regression_vs_budgeted_on_the_flat_pack() {
        // The committed pack is flat (one headingless section per document), so
        // section ranking reduces to title ranking and only reorders — the section
        // answer equals the budgeted answer on every fixture.
        let report = default_report();
        assert!(
            report.regressions.is_empty(),
            "section ranking must not change the answer on the flat pack: {:?}",
            report
                .regressions
                .iter()
                .map(|s| &s.name)
                .collect::<Vec<_>>()
        );
        for score in &report.scores {
            assert!(
                score.matches_budgeted,
                "{} section answer diverged from budgeted",
                score.name
            );
        }
    }

    #[test]
    fn every_answered_run_is_cross_validated() {
        let report = default_report();
        for score in &report.scores {
            if matches!(score.outcome, SectionOutcome::Answered { .. }) {
                assert!(
                    !score.false_grounded,
                    "{} answered but not grounded",
                    score.name
                );
            }
        }
    }

    #[test]
    fn coverage_misses_are_classified_not_hidden() {
        let report = default_report();
        let listed: Vec<&String> = report.coverage_misses.iter().map(|s| &s.name).collect();
        let actual: Vec<&String> = report
            .scores
            .iter()
            .filter(|s| matches!(s.outcome, SectionOutcome::CoverageMiss { .. }))
            .map(|s| &s.name)
            .collect();
        assert_eq!(listed, actual, "coverage misses are explicitly classified");
    }

    #[test]
    fn section_heading_priority_recovers_a_miss() {
        // Heading-relevant section filed second + 1-span budget: the budgeted reader
        // misses, the section-aware reader answers, with 0 false-grounded.
        let demo = section_priority_demo();
        assert!(
            !demo.heading_budgeted_answered,
            "metadata order misses the heading-relevant second section"
        );
        assert!(
            demo.heading_ranked_answered,
            "section ranking reaches the heading-relevant section under the same budget"
        );
        assert_eq!(
            demo.heading_answer.as_deref(),
            Some("Winds will reach forty miles per hour.")
        );
        assert!(!demo.any_false_grounded, "recovered answers are grounded");
    }

    #[test]
    fn multi_term_ranking_beats_single_token_overlap() {
        // Both sections' headings share the token "wind"; the section covering more
        // distinct query terms ("storm wind warning") is read first and answers,
        // which a single-token ranker could not have selected.
        let demo = section_priority_demo();
        assert!(
            !demo.multiterm_budgeted_answered,
            "metadata order reads the single-term section ⇒ miss"
        );
        assert!(
            demo.multiterm_ranked_answered,
            "the more-terms section answers"
        );
        assert_eq!(
            demo.multiterm_answer.as_deref(),
            Some("A severe storm wind warning is in effect tonight.")
        );
    }

    #[test]
    fn section_priority_is_stable_across_section_order() {
        let demo = section_priority_demo();
        assert!(
            demo.stable_across_section_order,
            "distinct headings ⇒ the section answer is identical across section order"
        );
    }

    #[test]
    fn report_and_demo_are_deterministic() {
        assert_eq!(
            default_report(),
            default_report(),
            "fixed inputs ⇒ identical report"
        );
        assert_eq!(section_priority_demo(), section_priority_demo());
    }

    #[test]
    fn section_ranked_read0_recovers_heading_relevant_answer() {
        // READ-11: a REAL Markdown document built by read0's loader. The
        // wind-relevant content lives under a "## Wind Forecast" heading filed
        // second; the heading becomes section metadata (not a span). Under a 1-span
        // budget the budgeted reader (metadata order) reads the first section's
        // sentence and misses, while the section-aware reader uses the PARSED
        // heading to read the relevant section first and answers.
        use reading_autonomy::{read_budgeted, read_section_ranked};
        use reading_cli::corpus_from_documents;
        let content = "# Daily Notes\nThe office opened at nine.\n## Wind Forecast\nWinds will reach forty miles per hour.".to_string();
        let corpus = corpus_from_documents(&[("bulletin.txt".to_string(), content)]);
        // The ATX headings became section metadata, not spans.
        let headings: Vec<&str> = corpus.metadata()[0]
            .sections
            .iter()
            .map(|s| s.heading.as_str())
            .collect();
        assert_eq!(headings, vec!["Daily Notes", "Wind Forecast"]);
        let tight = ReaderBounds {
            max_spans: 1,
            ..Default::default()
        };
        let question = "What is the wind forecast?";
        assert!(
            !read_budgeted(&corpus, question, tight).finalized(),
            "metadata order reads the first section's sentence ⇒ miss"
        );
        assert_eq!(
            read_section_ranked(&corpus, question, tight).answer(),
            Some("Winds will reach forty miles per hour."),
            "the parsed heading lets section ranking reach the relevant section first"
        );
    }
}
