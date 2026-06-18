//! Deterministic HYP-3 demo: record execution intents from a fixed set of governance reviews and
//! print them as pretty JSON. Pure function of fixed inputs, so two runs are byte-identical (the
//! release gate diffs them to prove intent determinism). It EXECUTES no probe — an approval only
//! records an intent for a human/operator to run LATER, and a rejected/deferred review is recorded
//! `blocked`.

use hypothesis_layer::{
    propose, HypothesisSpec, ProbeExecutionIntent, ProbeRequest, ReviewDecision, ReviewReceipt,
    ReviewerAuthority,
};

fn probe(statement: &str, risk: i64, reversibility: i64) -> ProbeRequest {
    let spec = HypothesisSpec {
        statement: statement.to_string(),
        prior: 500,
        uncertainty: 600,
        test_cost: 50,
        risk,
        reversibility,
        evidence_inputs: vec![],
        probe_description: format!("probe for: {statement}"),
    };
    ProbeRequest::from_hypothesis(&propose(spec).unwrap())
}

fn intent(
    statement: &str,
    risk: i64,
    reversibility: i64,
    authority: ReviewerAuthority,
    decision: ReviewDecision,
) -> ProbeExecutionIntent {
    let receipt =
        ReviewReceipt::decide(&probe(statement, risk, reversibility), authority, decision).unwrap();
    ProbeExecutionIntent::from_review(&receipt)
}

fn main() {
    // Fixed reviews spanning the disposition: a queued probe auto-approved within scope
    // (not_executed), a review-required probe approved by a human (requires_operator), a queued
    // probe rejected (blocked), and a queued probe deferred (blocked).
    let intents = [
        intent(
            "queued auto-approve",
            100,
            900,
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
        ),
        intent(
            "review-required human-approve",
            800,
            800,
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        ),
        intent(
            "queued reject",
            100,
            900,
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
        ),
        intent(
            "queued defer",
            200,
            950,
            ReviewerAuthority::Automated,
            ReviewDecision::Deferred,
        ),
    ];
    let cleared = intents.iter().filter(|i| !i.is_blocked()).count();

    // BEHAVIORAL policy assertions, exercised at gate-run time (the release gate diffs two runs AND
    // greps these as `true`). They call the REAL from_review() / decide() on the boundary paths, so
    // the gate verifies the policy by BEHAVIOUR — independent of the unit tests, which a `#[ignore]`
    // or a gutted body could otherwise silently disable. If the disposition derivation regresses (a
    // rejected/deferred review becoming cleared, or an approval being marked executed), these flip
    // and the gate fails.
    let rejected_review_blocked = intent(
        "rej",
        100,
        900,
        ReviewerAuthority::Governance,
        ReviewDecision::Rejected,
    )
    .is_blocked();
    let deferred_review_blocked = intent(
        "def",
        100,
        900,
        ReviewerAuthority::Automated,
        ReviewDecision::Deferred,
    )
    .is_blocked();
    // A blocked probe can never be approved (HYP-2), so it can never reach a cleared intent.
    let blocked_probe_never_approved = ReviewReceipt::decide(
        &probe("blk", 900, 100),
        ReviewerAuthority::Governance,
        ReviewDecision::Approved,
    )
    .is_err();
    // An approval RECORDS an intent; it executes nothing — recorded `not_executed`, never executed.
    let approved_records_not_executed = intent(
        "apv",
        100,
        900,
        ReviewerAuthority::Automated,
        ReviewDecision::Approved,
    )
    .execution_status()
    .token()
        == "not_executed";

    let report = serde_json::json!({
        "total": intents.len(),
        "cleared": cleared,
        "policy_rejected_review_blocked": rejected_review_blocked,
        "policy_deferred_review_blocked": deferred_review_blocked,
        "policy_blocked_probe_never_approved": blocked_probe_never_approved,
        "policy_approved_records_not_executed": approved_records_not_executed,
        "intents": intents,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
