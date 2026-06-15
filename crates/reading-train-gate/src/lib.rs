//! reading-train-gate — P12, the training-justification gate.
//!
//! Weight training stays forbidden until the P11 eval proves a stable, recurring
//! model failure that survives fixes to task spec, schema, prompt, examples,
//! tooling, context, and verifier design. This crate is the deterministic,
//! machine-checkable gate that enforces it: it consumes the eval result and
//! per-failure diagnoses and outputs a `TrainingDecision` whose load-bearing bit
//! is `training_justified`. It trains nothing and pulls no ML dependency.
//!
//! Doctrine: no failed cases → no training; any false-accept → a verifier/safety
//! fix, never training; any fixture/schema/prompt/tooling/context/verifier defect
//! → no training; only a clean, recurring model failure can justify weights.

#![forbid(unsafe_code)]

mod decision;

pub use decision::{
    decide, decide_from_eval, decide_from_report, FailureCause, FailureDiagnosis, TrainingDecision,
    MIN_RECURRENCES,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn diag(cause: FailureCause, survived: bool, recurrences: usize) -> FailureDiagnosis {
        FailureDiagnosis::new(
            "fx_case",
            "fabricated_cited_claim",
            cause,
            survived,
            recurrences,
        )
    }

    #[test]
    fn no_failures_blocks_training() {
        let d = decide(&[], &[]);
        assert!(!d.training_justified);
        assert!(d
            .blockers
            .iter()
            .any(|b| b.contains("no_unresolved_failures")));
    }

    #[test]
    fn false_accept_blocks_training_and_requires_safety_fix() {
        let d = decide(&["fc_compound_fragments".to_string()], &[]);
        assert!(!d.training_justified);
        assert!(d.safety_fix_required);
        assert!(d.reason.contains("safety"));
        assert!(d.blockers.iter().any(|b| b.contains("harden the verifier")));
    }

    #[test]
    fn eval_design_failure_blocks_training() {
        let d = decide(&[], &[diag(FailureCause::BadFixture, false, 5)]);
        assert!(!d.training_justified);
        assert!(d.blockers.iter().any(|b| b.contains("fixture_defect")));
    }

    #[test]
    fn schema_failure_blocks_training() {
        let d = decide(&[], &[diag(FailureCause::SchemaDefect, false, 5)]);
        assert!(!d.training_justified);
        assert!(d.blockers.iter().any(|b| b.contains("schema_defect")));
    }

    #[test]
    fn verifier_weakness_blocks_training() {
        // Doctrine: a verifier defect is never a training reason.
        let d = decide(&[], &[diag(FailureCause::VerifierWeakness, false, 9)]);
        assert!(!d.training_justified);
        assert!(d.blockers.iter().any(|b| b.contains("verifier_defect")));
    }

    #[test]
    fn clean_repeated_false_rejects_can_mark_training_candidate() {
        let d = decide(
            &[],
            &[FailureDiagnosis::new(
                "fr_paraphrase_001",
                "fabricated_cited_claim",
                FailureCause::CleanModelFailure,
                true,
                MIN_RECURRENCES,
            )],
        );
        assert!(
            d.training_justified,
            "clean recurring survivor should justify training"
        );
        assert_eq!(d.cited_failures, vec!["fr_paraphrase_001".to_string()]);
        assert!(d.blockers.is_empty());
    }

    #[test]
    fn training_decision_cites_fixture_ids() {
        // Justified path cites the clean survivor's id.
        let justified = decide(
            &[],
            &[FailureDiagnosis::new(
                "fr_clean_42",
                "premature_synthesize",
                FailureCause::CleanModelFailure,
                true,
                3,
            )],
        );
        assert!(justified
            .cited_failures
            .contains(&"fr_clean_42".to_string()));
        // Blocked-by-false-accept path names the offending fixture id too.
        let blocked = decide(&["fa_inject_7".to_string()], &[]);
        assert!(blocked.blockers.iter().any(|b| b.contains("fa_inject_7")));
    }

    #[test]
    fn single_clean_failure_is_not_recurring_and_blocks() {
        let d = decide(&[], &[diag(FailureCause::CleanModelFailure, true, 1)]);
        assert!(!d.training_justified);
        assert!(d
            .blockers
            .iter()
            .any(|b| b.contains("insufficient_recurrence")));
    }

    #[test]
    fn clean_failure_not_survived_cleanup_blocks() {
        let d = decide(&[], &[diag(FailureCause::CleanModelFailure, false, 9)]);
        assert!(!d.training_justified);
        assert!(d
            .blockers
            .iter()
            .any(|b| b.contains("not_survived_cleanup")));
    }

    #[test]
    fn one_remaining_defect_blocks_even_with_a_clean_candidate() {
        // A clean survivor does NOT license training while any defect remains.
        let d = decide(
            &[],
            &[
                FailureDiagnosis::new("fr_clean", "x", FailureCause::CleanModelFailure, true, 4),
                FailureDiagnosis::new("fr_schema", "y", FailureCause::SchemaDefect, false, 4),
            ],
        );
        assert!(!d.training_justified);
        assert!(d.blockers.iter().any(|b| b.contains("schema_defect")));
    }

    #[test]
    fn current_battery_blocks_training() {
        // The decision rule: P11 is 37/37, 0 false-accepts, 0 false-rejects ⇒
        // training_justified = false, because no clean residual failure exists.
        let d = decide_from_eval();
        assert!(
            !d.training_justified,
            "0 unresolved failures must not justify training"
        );
        assert!(!d.safety_fix_required);
        assert!(d.cited_failures.is_empty());
        assert!(d.reason.contains("no unresolved failures"));
    }

    #[test]
    fn decision_is_deterministic() {
        assert_eq!(decide_from_eval(), decide_from_eval());
    }

    // --- decide_from_report: a diagnosis must correspond to a REAL residual
    //     failure in the eval; a phantom cannot unlock training ---

    /// A report carrying one residual failure (false-reject) named `id`. The
    /// disposition values are unused by `decide_from_report` (it reads name +
    /// category), so we avoid constructing a `Rejected(kind)`.
    fn report_with_residual(id: &str) -> reading_eval::EvalReport {
        use reading_eval::{Disposition, EvalReport, ScoredCase, Verdict};
        EvalReport {
            total: 1,
            correct: 0,
            false_accepts: vec![],
            false_rejects: vec![ScoredCase {
                name: id.to_string(),
                category: "fabricated_cited_claim".to_string(),
                expected: Disposition::Finalized,
                actual: Disposition::AcceptedPartial,
                verdict: Verdict::FalseReject,
                reason_mismatch: false,
            }],
            by_category: Default::default(),
            failure_categories: Default::default(),
            cases: vec![],
            next_change: String::new(),
        }
    }

    #[test]
    fn phantom_diagnosis_cannot_justify_training_on_clean_eval() {
        let clean = reading_eval::run(); // 0 false-accepts, 0 false-rejects
        let phantom = FailureDiagnosis::new(
            "fr_phantom_never_failed",
            "fabricated_cited_claim",
            FailureCause::CleanModelFailure,
            true,
            5,
        );
        let d = decide_from_report(&clean, &[phantom]);
        assert!(
            !d.training_justified,
            "a phantom diagnosis must not unlock training on a clean eval"
        );
        assert!(d.cited_failures.is_empty());
        assert!(d.blockers.iter().any(|b| b.contains("phantom_diagnosis")));
    }

    #[test]
    fn valid_diagnosis_of_a_real_residual_can_justify_via_report() {
        let report = report_with_residual("fr_real_001");
        let diag = FailureDiagnosis::new(
            "fr_real_001",
            "fabricated_cited_claim",
            FailureCause::CleanModelFailure,
            true,
            3,
        );
        let d = decide_from_report(&report, &[diag]);
        assert!(
            d.training_justified,
            "a clean recurring diagnosis of a real residual justifies"
        );
        assert_eq!(d.cited_failures, vec!["fr_real_001".to_string()]);
    }

    #[test]
    fn undiagnosed_residual_failure_blocks_via_report() {
        let report = report_with_residual("fr_real_002");
        let d = decide_from_report(&report, &[]);
        assert!(
            !d.training_justified,
            "an undiagnosed residual failure blocks until triaged"
        );
    }
}
