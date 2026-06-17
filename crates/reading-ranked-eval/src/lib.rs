//! reading-ranked-eval — READ-9, the title-ranked selection eval.
//!
//! Measures the deterministic TITLE-RANKED reader (`read_ranked`) against the
//! READ-8 budgeted reader over the READ-4 corpus fixtures. On the committed pack
//! the relevant documents are already first, so ranking only REORDERS reads: the
//! eval proves NO-REGRESSION (ranked answer == budgeted answer, 0 regressions) and
//! 0 false-grounded (cross-validated). The title-ranking WIN — reaching a relevant
//! document filed second under a tight budget, deterministically and stably across
//! file order — is measured by `title_priority_demo`. Coverage misses are an
//! engineering signal; the P12 gate still owns weights. No model, no training.

#![forbid(unsafe_code)]

mod scorer;

pub use scorer::{
    evaluate_ranked_pack, title_priority_demo, RankedOutcome, RankedReport, RankedScore,
    RankingDemo,
};

#[cfg(test)]
mod tests {
    use super::*;
    use reading_autonomy::ReaderBounds;
    use reading_corpus_eval::fixtures;

    fn default_report() -> RankedReport {
        evaluate_ranked_pack(ReaderBounds::default())
    }

    #[test]
    fn runs_on_at_least_ten_fixtures() {
        let report = default_report();
        assert!(
            report.total >= 10,
            "READ-9 needs ≥ 10 fixtures, have {}",
            report.total
        );
        assert_eq!(report.total, fixtures().len());
    }

    #[test]
    fn zero_false_grounded() {
        // Cross-validated: false_grounded is set iff verify fails OR the independent
        // check disagrees. An empty list means every ranked answer is grounded.
        let report = default_report();
        assert!(
            report.false_grounded.is_empty(),
            "a false-grounded ranked answer is the unsafe class: {:?}",
            report
                .false_grounded
                .iter()
                .map(|s| &s.name)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn no_regression_vs_budgeted_on_the_committed_pack() {
        // Under the default budget ranking only reorders reads, so the ranked
        // answer equals the budgeted answer on every fixture — no regressions.
        let report = default_report();
        assert!(
            report.regressions.is_empty(),
            "ranking must not change the answer on the committed pack: {:?}",
            report
                .regressions
                .iter()
                .map(|s| &s.name)
                .collect::<Vec<_>>()
        );
        for score in &report.scores {
            assert!(
                score.matches_budgeted,
                "{} ranked answer diverged from budgeted",
                score.name
            );
        }
    }

    #[test]
    fn every_answered_run_is_cross_validated() {
        let report = default_report();
        for score in &report.scores {
            if matches!(score.outcome, RankedOutcome::Answered { .. }) {
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
        // The coverage-miss list exactly enumerates the non-finalized ranked runs.
        let report = default_report();
        let listed: Vec<&String> = report.coverage_misses.iter().map(|s| &s.name).collect();
        let actual: Vec<&String> = report
            .scores
            .iter()
            .filter(|s| matches!(s.outcome, RankedOutcome::CoverageMiss { .. }))
            .map(|s| &s.name)
            .collect();
        assert_eq!(listed, actual, "coverage misses are explicitly classified");
    }

    #[test]
    fn title_priority_recovers_a_miss_the_budgeted_reader_makes() {
        // The measured win: with the relevant document filed second and a 1-span
        // budget, the budgeted reader misses while the title-ranked reader answers,
        // with 0 false-grounded.
        let demo = title_priority_demo();
        assert!(
            !demo.budgeted_answered,
            "blunt metadata order misses the relevant second document"
        );
        assert!(
            demo.ranked_answered,
            "title ranking reaches the relevant document under the same budget"
        );
        assert_eq!(
            demo.ranked_answer.as_deref(),
            Some("Winds will reach forty miles per hour.")
        );
        assert!(
            !demo.ranked_false_grounded,
            "the recovered answer is cross-validated grounded"
        );
    }

    #[test]
    fn title_priority_is_stable_across_file_order() {
        let demo = title_priority_demo();
        assert!(
            demo.stable_across_file_order,
            "distinct titles ⇒ the ranked answer is identical across file order"
        );
    }

    #[test]
    fn report_and_demo_are_deterministic() {
        assert_eq!(
            default_report(),
            default_report(),
            "fixed inputs ⇒ identical report"
        );
        assert_eq!(title_priority_demo(), title_priority_demo());
    }
}
