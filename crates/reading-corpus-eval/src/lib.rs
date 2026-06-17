//! reading-corpus-eval — READ-4, the real-corpus eval pack.
//!
//! Turns read0 from a single hand-run demo into a measured benchmark over many
//! real corpora. Each committed fixture (docs + question + plan + expected
//! verifier result) is driven through the actual read0 run → verify → replay
//! path, and scored against its COMMITTED label. The unsafe class — a
//! false-grounded answer (an expected-rejected fixture that finalized a verified
//! answer) — is surfaced explicitly and must be zero. Deterministic; no model, no
//! training. Training stays forbidden until the P12 gate is justified by clean
//! recurring failures — anecdotes here never justify weights.

#![forbid(unsafe_code)]

mod pack;
mod scorer;

pub use pack::{fixtures, CorpusFixture, Expected};
pub use scorer::{evaluate, evaluate_pack, FixtureResult, Outcome, PackReport, Verdict, Workdir};

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> PackReport {
        let work = Workdir::new().expect("workdir");
        evaluate_pack(work.path()).expect("evaluate pack")
    }

    #[test]
    fn at_least_ten_fixtures() {
        assert!(
            fixtures().len() >= 10,
            "READ-4 requires ≥ 10 fixtures, have {}",
            fixtures().len()
        );
    }

    #[test]
    fn every_fixture_matches_its_committed_expectation() {
        let r = report();
        assert_eq!(
            r.correct,
            r.total,
            "false-grounded: {:?}; false-rejects: {:?}",
            r.false_grounded.iter().map(|f| &f.name).collect::<Vec<_>>(),
            r.false_rejects.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn zero_false_grounded_answers() {
        let r = report();
        assert!(
            r.false_grounded.is_empty(),
            "a false-grounded answer is an explicit failure: {:?}",
            r.false_grounded.iter().map(|f| &f.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn every_verified_fixture_is_replayed_with_a_trace_hash() {
        // A Verified outcome is only produced after verify AND replay both pass
        // (scorer::run_one), so a present trace hash proves the run was replayed.
        let r = report();
        let verified: Vec<&FixtureResult> = r
            .results
            .iter()
            .filter(|f| matches!(f.outcome, Outcome::Verified { .. }))
            .collect();
        assert!(verified.len() >= 5, "expected several verified fixtures");
        for f in verified {
            assert!(
                f.trace_hash().is_some(),
                "{} verified but has no trace hash",
                f.name
            );
        }
    }

    #[test]
    fn every_rejection_records_a_reason() {
        let r = report();
        for f in r
            .results
            .iter()
            .filter(|f| matches!(f.outcome, Outcome::Rejected { .. }))
        {
            assert!(
                f.rejection_reason().map(|s| !s.is_empty()).unwrap_or(false),
                "{} rejected but has no reason",
                f.name
            );
        }
    }

    #[test]
    fn report_is_deterministic() {
        assert_eq!(report(), report(), "fixed content ⇒ identical pack report");
    }

    #[test]
    fn a_valid_plan_mislabelled_reject_is_flagged_false_grounded() {
        // Control: the labels are COMMITTED, not inferred. A genuinely valid plan
        // labelled as if it must be rejected must surface as a false-grounded
        // failure (proving the scorer grades actual-vs-committed, never the model).
        let work = Workdir::new().unwrap();
        let valid_plan_labelled_reject = [CorpusFixture {
            name: "control_valid_but_labelled_reject",
            documents: &[("note.txt", "An ECG was ordered immediately.")],
            question: "What test?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"An ECG was ordered immediately.","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"An ECG was ordered immediately.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Rejected,
        }];
        let r = evaluate(work.path(), &valid_plan_labelled_reject).unwrap();
        assert_eq!(r.false_grounded.len(), 1);
        assert_eq!(
            r.false_grounded[0].name,
            "control_valid_but_labelled_reject"
        );
    }
}
