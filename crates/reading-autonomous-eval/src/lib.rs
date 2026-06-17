//! reading-autonomous-eval — READ-7, the autonomous corpus eval pack.
//!
//! Measures the deterministic READ-6 reader against the READ-4 corpus fixtures
//! WITHOUT any hand-written plan: each fixture's corpus is rebuilt and the
//! autonomous reader proposes its own plan through the hardened codec. A
//! false-grounded answer (a finalized answer the independent verifier rejects) is
//! the unsafe class and must be zero. The report compares the manual-plan score
//! to the autonomous-reader score. Autonomy underperformance is an engineering
//! signal — never a training justification; the P12 gate still owns weights.

#![forbid(unsafe_code)]

mod scorer;

pub use scorer::{
    evaluate_autonomous_pack, independently_grounded, AutonomousOutcome, AutonomousPackReport,
    Comparison, FixtureScore,
};

#[cfg(test)]
mod tests {
    use super::*;
    use reading_autonomy::ReaderBounds;
    use reading_corpus_eval::{fixtures, Expected};

    fn default_report() -> AutonomousPackReport {
        evaluate_autonomous_pack(ReaderBounds::default())
    }

    #[test]
    fn runs_on_at_least_ten_fixtures() {
        let report = default_report();
        assert!(
            report.total >= 10,
            "READ-7 needs ≥ 10 fixtures, have {}",
            report.total
        );
        assert_eq!(report.total, fixtures().len(), "every fixture is scored");
    }

    #[test]
    fn no_fixture_uses_its_hand_written_plan() {
        // The scorer drives the autonomous reader; the fixture's committed plan is
        // never used. Each autonomous plan is the reader's own (inspect first),
        // and differs from the hand-written plan.
        let report = default_report();
        let fxs = fixtures();
        for score in &report.scores {
            let fixture = fxs.iter().find(|f| f.name == score.name).unwrap();
            assert_ne!(
                score.autonomous_plan, fixture.plan,
                "{} must use the autonomous plan, not the hand-written one",
                score.name
            );
            assert!(
                score
                    .autonomous_plan
                    .starts_with(r#"[{"action":"inspect_corpus"}"#),
                "the autonomous plan inspects metadata first: {}",
                score.autonomous_plan
            );
        }
    }

    #[test]
    fn zero_false_grounded() {
        // false_grounded is set iff verify() fails OR the independent cross-check
        // disagrees, so an empty list means BOTH agreed every finalized answer is
        // grounded — a cross-validated zero, not a same-function tautology.
        let report = default_report();
        assert!(
            report.false_grounded.is_empty(),
            "a false-grounded autonomous answer is the unsafe class: {:?}",
            report
                .false_grounded
                .iter()
                .map(|s| &s.name)
                .collect::<Vec<_>>()
        );
    }

    fn finalized_run(name: &str) -> (reading_substrate::Corpus, reading_substrate::ReadingRun) {
        let fixture = fixtures().into_iter().find(|f| f.name == name).unwrap();
        let docs: Vec<(String, String)> = fixture
            .documents
            .iter()
            .map(|(n, c)| (n.to_string(), c.to_string()))
            .collect();
        let corpus = reading_cli::corpus_from_documents(&docs);
        let outcome = reading_autonomy::read(&corpus, fixture.question, ReaderBounds::default());
        let run = outcome.decision.unwrap().finalized.unwrap();
        (corpus, run)
    }

    #[test]
    fn independent_check_is_load_bearing_and_does_not_just_echo_verify() {
        // A genuine finalized answer passes the independent check; a tampered
        // answer (claims unchanged, answer_text altered) is rejected by it WITHOUT
        // any call to verify() — so it would catch a verify() that wrongly passed.
        let (corpus, run) = finalized_run("weather_wind_valid");
        assert!(
            independently_grounded(&corpus, &run),
            "the genuine answer is grounded"
        );
        let mut tampered = run.clone();
        tampered.proof.answer_text = "The bridge is completely safe to cross.".to_string();
        assert!(
            !independently_grounded(&corpus, &tampered),
            "the independent check catches an answer that is not its cited claims"
        );
    }

    #[test]
    fn reader_cites_every_extracted_claim_so_none_is_silently_dropped() {
        // The v0 reader's answer cites ALL extracted claims (supporting_claims ==
        // every claim id), so no grounded claim — e.g. a "Do not" warning — is
        // dropped from the answer. (A future selective reader would need its own
        // completeness check; that is out of scope for the v0 measurement.)
        let (_corpus, run) = finalized_run("multi_sentence_doc_valid");
        let all_ids: Vec<u64> = run.memory.claims.iter().map(|c| c.id).collect();
        assert_eq!(
            run.proof.supporting_claims, all_ids,
            "every extracted claim is cited in the answer"
        );
    }

    #[test]
    fn every_finalized_answer_is_independently_reverified() {
        // A Verified outcome only exists after a fresh verify() passed in the
        // scorer, so no verified fixture can be flagged false-grounded.
        let report = default_report();
        for score in &report.scores {
            if matches!(score.outcome, AutonomousOutcome::Verified { .. }) {
                assert!(
                    !score.false_grounded,
                    "{} finalized but failed independent re-verification",
                    score.name
                );
            }
        }
    }

    #[test]
    fn report_compares_manual_vs_autonomous() {
        let report = default_report();
        // Manual baseline is the READ-4 committed labels (6 verified, 9 rejected).
        assert_eq!(report.manual_verified, 6);
        assert_eq!(report.manual_rejected, 9);
        assert_eq!(
            report.manual_verified + report.manual_rejected,
            report.total
        );
        assert_eq!(
            report.autonomous_verified + report.autonomous_rejected,
            report.total
        );
        // The comparison classes partition the pack.
        let classified = report.both_verified
            + report.autonomous_verified_manual_rejected
            + report.false_rejects.len()
            + report
                .scores
                .iter()
                .filter(|s| s.comparison == Comparison::BothRejected)
                .count();
        assert_eq!(classified, report.total, "every fixture is classified");
    }

    #[test]
    fn autonomous_reader_grounds_every_corpus_under_default_bounds() {
        // The v0 reader reads spans verbatim, so it produces a grounded answer on
        // every fixture corpus — including the adversarial reject fixtures, where
        // it verifies a SAFE answer the hand-plan deliberately got rejected.
        let report = default_report();
        assert_eq!(report.autonomous_verified, report.total);
        assert_eq!(report.both_verified, report.manual_verified);
        assert_eq!(
            report.autonomous_verified_manual_rejected,
            report.manual_rejected
        );
        assert!(report.false_rejects.is_empty());
    }

    #[test]
    fn autonomy_preserves_negation_and_is_not_false_grounded() {
        // The "negation" fixture's hand-plan dropped the "Do not" and was rejected.
        // The autonomous reader claims the WHOLE sentence verbatim, so the negation
        // survives and the answer is honestly grounded — never false-grounded.
        let report = default_report();
        let negation = report
            .scores
            .iter()
            .find(|s| s.name == "negation_dropped_fragment_reject")
            .expect("negation fixture present");
        match &negation.outcome {
            AutonomousOutcome::Verified { answer, .. } => {
                assert!(answer.contains("Do not"), "negation preserved: {answer:?}");
            }
            AutonomousOutcome::Rejected { reason } => {
                panic!("expected grounded read, got {reason}")
            }
        }
        assert!(!negation.false_grounded);
    }

    #[test]
    fn tight_bounds_produce_classified_false_rejects_without_false_grounded() {
        // With no spans allowed, the reader finalizes nothing: every manual-verified
        // fixture becomes a CLASSIFIED false-reject, and there is still no
        // false-grounded answer. Exercises the false-reject classifier.
        let report = evaluate_autonomous_pack(ReaderBounds {
            max_spans: 0,
            ..Default::default()
        });
        assert_eq!(report.autonomous_verified, 0);
        assert_eq!(report.false_rejects.len(), report.manual_verified);
        assert!(report.false_grounded.is_empty());
        for score in &report.false_rejects {
            assert_eq!(
                score.comparison,
                Comparison::AutonomousRejectedManualVerified
            );
            assert_eq!(score.manual, Expected::Verified);
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
