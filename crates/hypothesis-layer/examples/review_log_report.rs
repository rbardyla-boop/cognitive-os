//! Deterministic HYP-2 demo: record a governance review decision on a fixed set of probes and print
//! the resulting review log as pretty JSON. Pure function of fixed inputs, so two runs are
//! byte-identical (the release gate diffs them to prove receipt/log determinism). It EXECUTES no
//! probe — approving a probe only records the decision for a human to run LATER.

use hypothesis_layer::{
    propose, HypothesisSpec, ProbeRequest, ReviewDecision, ReviewLog, ReviewReceipt,
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

fn main() {
    // Fixed reviews spanning the policy: a queued probe auto-approved within scope, a review-required
    // probe approved by a human, a blocked probe rejected (it can never be approved), and a queued
    // probe deferred.
    let reviews = [
        (
            "queued auto-approve",
            100,
            900,
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
        ),
        (
            "review-required human-approve",
            800,
            800,
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
        ),
        (
            "blocked governance-reject",
            900,
            100,
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
        ),
        (
            "queued defer",
            200,
            950,
            ReviewerAuthority::Automated,
            ReviewDecision::Deferred,
        ),
    ];
    let receipts: Vec<ReviewReceipt> = reviews
        .into_iter()
        .map(|(statement, risk, reversibility, authority, decision)| {
            ReviewReceipt::decide(&probe(statement, risk, reversibility), authority, decision)
                .unwrap()
        })
        .collect();

    let log = ReviewLog::from_receipts(receipts);

    // BEHAVIORAL policy assertions, exercised at gate-run time (the release gate diffs two runs AND
    // greps these as `true`). These call the REAL decide() on the forbidden paths, so the gate verifies
    // the policy by BEHAVIOUR — independent of the unit tests, which a `#[ignore]` or a gutted body could
    // otherwise silently disable. If the Blocked guard or the authority check regresses, decide() returns
    // Ok here, these flip to false, and the gate fails.
    let blocked_approve_refused = ReviewReceipt::decide(
        &probe("blk", 900, 100),
        ReviewerAuthority::Governance,
        ReviewDecision::Approved,
    )
    .is_err();
    let automated_review_required_refused = ReviewReceipt::decide(
        &probe("rev", 800, 800),
        ReviewerAuthority::Automated,
        ReviewDecision::Approved,
    )
    .is_err();

    let report = serde_json::json!({
        "total": log.receipts().len(),
        "approved": log.approved().len(),
        "policy_blocked_approve_refused": blocked_approve_refused,
        "policy_automated_review_required_refused": automated_review_required_refused,
        "log": log,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
