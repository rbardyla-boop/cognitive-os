//! Deterministic HYP-5 demo: record promotion requests from a fixed set of quarantined observations
//! and print them as pretty JSON. Pure function of fixed inputs, so two runs are byte-identical (the
//! release gate diffs them to prove promotion determinism). NOTHING is promoted — every request is
//! `rejected` (its source observation is not promotable), no request grants a promotion, and an
//! observation never becomes evidence just because it exists.

use hypothesis_layer::{
    propose, HypothesisSpec, ProbeExecutionIntent, ProbeObservationReceipt, ProbeRequest,
    PromotionRequest, PromotionTarget, ReviewDecision, ReviewReceipt, ReviewerAuthority,
};

fn observation(authority: ReviewerAuthority, decision: ReviewDecision) -> ProbeObservationReceipt {
    let spec = HypothesisSpec {
        statement: "Bridge B reopened.".to_string(),
        prior: 500,
        uncertainty: 600,
        test_cost: 50,
        risk: 100,
        reversibility: 900,
        evidence_inputs: vec![],
        probe_description: "Re-read the maintenance log span.".to_string(),
    };
    let probe = ProbeRequest::from_hypothesis(&propose(spec).unwrap());
    let intent = ProbeExecutionIntent::from_review(
        &ReviewReceipt::decide(&probe, authority, decision).unwrap(),
    );
    ProbeObservationReceipt::from_intent(&intent, "observed: maintenance log span re-read")
}

fn request(
    authority: ReviewerAuthority,
    decision: ReviewDecision,
    target: PromotionTarget,
) -> PromotionRequest {
    PromotionRequest::from_observation(&observation(authority, decision), target)
}

fn main() {
    // Fixed promotion requests spanning the reachable observation dispositions and all three targets:
    // a rejected observation (not_executed intent) requesting a claim, a requires_review observation
    // (requires_operator intent) requesting evidence, and a rejected observation (blocked intent)
    // requesting a memory note. EVERY one is `rejected` at the source — NONE is promoted.
    let requests = [
        request(
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
            PromotionTarget::Claim,
        ),
        request(
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
            PromotionTarget::Evidence,
        ),
        request(
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
            PromotionTarget::MemoryNote,
        ),
    ];
    let promoted = requests.iter().filter(|r| r.grants_promotion()).count();

    // BEHAVIORAL still-no-evidence assertions, exercised at gate-run time (the release gate diffs two
    // runs AND greps these as expected). They call the REAL from_observation() on the boundary paths,
    // so the gate verifies the boundary by BEHAVIOUR — independent of the unit tests, which a
    // `#[ignore]` or a gutted body could otherwise silently disable. If a rejected/requires_review
    // observation became promotable, or an evidence target were granted, these flip and the gate fails.
    let rejected_observation_not_promoted = !request(
        ReviewerAuthority::Automated,
        ReviewDecision::Approved,
        PromotionTarget::Claim,
    )
    .grants_promotion();
    let requires_review_observation_not_promoted = !request(
        ReviewerAuthority::Human,
        ReviewDecision::Approved,
        PromotionTarget::MemoryNote,
    )
    .grants_promotion();
    let evidence_target_not_granted = !request(
        ReviewerAuthority::Human,
        ReviewDecision::Approved,
        PromotionTarget::Evidence,
    )
    .grants_promotion();
    // STILL NO EVIDENCE: none of the produced requests grants a promotion — at HYP-5 nothing becomes
    // evidence.
    let no_promotion = requests.iter().all(|r| !r.grants_promotion());

    let report = serde_json::json!({
        "total": requests.len(),
        "promoted": promoted,
        "policy_rejected_observation_not_promoted": rejected_observation_not_promoted,
        "policy_requires_review_observation_not_promoted": requires_review_observation_not_promoted,
        "policy_evidence_target_not_granted": evidence_target_not_granted,
        "policy_no_promotion_at_hyp5": no_promotion,
        "requests": requests,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
