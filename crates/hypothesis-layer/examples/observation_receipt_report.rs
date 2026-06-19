//! Deterministic HYP-4 demo: quarantine observations recorded from a fixed set of execution intents
//! and print them as pretty JSON. Pure function of fixed inputs, so two runs are byte-identical (the
//! release gate diffs them to prove observation determinism). NOTHING is recorded — every observation
//! is `rejected` or `requires_review`; an observation holds `observation_only` authority and never
//! implies the probe ran.

use hypothesis_layer::{
    propose, HypothesisSpec, ObservationStatus, ProbeExecutionIntent, ProbeObservationReceipt,
    ProbeRequest, ReviewDecision, ReviewReceipt, ReviewerAuthority,
};

fn intent(authority: ReviewerAuthority, decision: ReviewDecision) -> ProbeExecutionIntent {
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
    ProbeExecutionIntent::from_review(&ReviewReceipt::decide(&probe, authority, decision).unwrap())
}

fn observe(
    authority: ReviewerAuthority,
    decision: ReviewDecision,
    text: &str,
) -> ProbeObservationReceipt {
    ProbeObservationReceipt::from_intent(&intent(authority, decision), text)
}

fn main() {
    // Fixed observations spanning the reachable dispositions: a not_executed intent (rejected), a
    // requires_operator intent (requires_review), and a blocked intent (rejected). NONE is recorded.
    let observations = [
        observe(
            ReviewerAuthority::Automated,
            ReviewDecision::Approved,
            "observed: log span re-read (not-executed intent)",
        ),
        observe(
            ReviewerAuthority::Human,
            ReviewDecision::Approved,
            "observed: operator-gated probe (requires-operator intent)",
        ),
        observe(
            ReviewerAuthority::Governance,
            ReviewDecision::Rejected,
            "observed: blocked probe (rejected intent)",
        ),
    ];
    let recorded = observations
        .iter()
        .filter(|o| o.observation_status() == ObservationStatus::Recorded)
        .count();

    // BEHAVIORAL quarantine assertions, exercised at gate-run time (the release gate diffs two runs
    // AND greps these as expected). They call the REAL from_intent() on the boundary paths, so the
    // gate verifies the quarantine by BEHAVIOUR — independent of the unit tests, which a `#[ignore]`
    // or a gutted body could otherwise silently disable. If a not_executed/blocked intent became
    // recorded, or a requires_operator intent stopped requiring review, these flip and the gate fails.
    let not_executed_rejected =
        observe(ReviewerAuthority::Automated, ReviewDecision::Approved, "ne")
            .observation_status()
            .token()
            == "rejected";
    let blocked_rejected = observe(
        ReviewerAuthority::Governance,
        ReviewDecision::Rejected,
        "bl",
    )
    .observation_status()
    .token()
        == "rejected";
    let requires_operator_requires_review =
        observe(ReviewerAuthority::Human, ReviewDecision::Approved, "ro")
            .observation_status()
            .token()
            == "requires_review";
    // THE QUARANTINE: none of the produced observations is recorded — at HYP-4 nothing can be promoted.
    let no_recorded = observations
        .iter()
        .all(|o| o.observation_status() != ObservationStatus::Recorded);

    let report = serde_json::json!({
        "total": observations.len(),
        "recorded": recorded,
        "policy_not_executed_rejected": not_executed_rejected,
        "policy_blocked_rejected": blocked_rejected,
        "policy_requires_operator_requires_review": requires_operator_requires_review,
        "policy_no_recorded_at_hyp4": no_recorded,
        "observations": observations,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
