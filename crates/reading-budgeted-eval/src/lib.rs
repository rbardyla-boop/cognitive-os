//! reading-budgeted-eval — READ-8, the budgeted span-selection eval.
//!
//! Measures the deterministic SELECTIVE reader (`read_budgeted`) against the blunt
//! READ-6 reader over the READ-4 corpus fixtures: the budgeted reader claims only
//! question-relevant spans, so it is more focused, and under a tight budget it can
//! miss a relevant span — a CLASSIFIED coverage miss, never a false-grounded
//! answer (which is cross-validated to zero). Coverage misses are an engineering
//! signal; the P12 gate still owns weights. No model, no training.

#![forbid(unsafe_code)]

mod scorer;

pub use scorer::{evaluate_budgeted_pack, BudgetedOutcome, BudgetedReport, BudgetedScore};

#[cfg(test)]
mod tests {
    use super::*;
    use reading_autonomy::ReaderBounds;
    use reading_corpus_eval::fixtures;

    fn default_report() -> BudgetedReport {
        evaluate_budgeted_pack(ReaderBounds::default())
    }

    #[test]
    fn runs_on_at_least_ten_fixtures() {
        let report = default_report();
        assert!(
            report.total >= 10,
            "READ-8 needs ≥ 10 fixtures, have {}",
            report.total
        );
        assert_eq!(report.total, fixtures().len());
    }

    #[test]
    fn zero_false_grounded() {
        // Cross-validated: false_grounded is set iff verify fails OR the independent
        // check disagrees. An empty list means every budgeted answer is grounded.
        let report = default_report();
        assert!(
            report.false_grounded.is_empty(),
            "a false-grounded budgeted answer is the unsafe class: {:?}",
            report
                .false_grounded
                .iter()
                .map(|s| &s.name)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn budgeted_reader_is_more_focused_than_blunt() {
        // Selection drops irrelevant spans, so the budgeted reader makes no more
        // claims than the blunt one overall, and strictly fewer on some fixture.
        let report = default_report();
        assert!(
            report.total_budgeted_claims <= report.total_blunt_claims,
            "budgeted ({}) must not exceed blunt ({})",
            report.total_budgeted_claims,
            report.total_blunt_claims
        );
        assert!(
            report.more_focused >= 1,
            "at least one fixture is more focused under budgeting"
        );
    }

    #[test]
    fn every_answered_run_is_cross_validated() {
        let report = default_report();
        for score in &report.scores {
            if matches!(score.outcome, BudgetedOutcome::Answered { .. }) {
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
        // The coverage-miss list exactly enumerates the non-finalized budgeted runs
        // (an explicit list, not buried in an aggregate score).
        let report = default_report();
        let listed: Vec<&String> = report.coverage_misses.iter().map(|s| &s.name).collect();
        let actual: Vec<&String> = report
            .scores
            .iter()
            .filter(|s| matches!(s.outcome, BudgetedOutcome::CoverageMiss { .. }))
            .map(|s| &s.name)
            .collect();
        assert_eq!(listed, actual, "coverage misses are explicitly classified");
    }

    #[test]
    fn tight_budget_forces_classified_coverage_misses_without_false_grounded() {
        // max_spans = 1: each corpus exposes only its first span to the reader, so
        // fixtures whose first sentence is not question-relevant become coverage
        // misses — classified, and still 0 false-grounded.
        let report = evaluate_budgeted_pack(ReaderBounds {
            max_spans: 1,
            ..Default::default()
        });
        assert!(
            report.false_grounded.is_empty(),
            "no false-grounded under tight budget"
        );
        assert!(
            !report.coverage_misses.is_empty(),
            "a tight budget produces at least one classified coverage miss"
        );
        for miss in &report.coverage_misses {
            assert!(matches!(miss.outcome, BudgetedOutcome::CoverageMiss { .. }));
        }
    }

    #[test]
    fn report_is_deterministic() {
        assert_eq!(
            default_report(),
            default_report(),
            "fixed inputs ⇒ identical report"
        );
    }
}
