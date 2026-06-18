//! Deterministic HYP-1 demo: derive a probe queue from a fixed set of hypotheses and print
//! it as pretty JSON. Pure function of fixed inputs, so two runs are byte-identical (the
//! release gate diffs them to prove queue determinism / replay). It EXECUTES no probe — it
//! only classifies each into queued / human_review_required / blocked and reports which are
//! eligible for execution (only the plain `queued` ones).

use hypothesis_layer::{propose, HypothesisSpec, ProbeQueue};

fn spec(statement: &str, risk: i64, reversibility: i64) -> HypothesisSpec {
    HypothesisSpec {
        statement: statement.to_string(),
        prior: 500,
        uncertainty: 600,
        test_cost: 50,
        risk,
        reversibility,
        evidence_inputs: vec![],
        probe_description: format!("probe for: {statement}"),
    }
}

fn main() {
    // Fixed inputs spanning every clearance: queued, human_review (high-risk), human_review
    // (irreversible), and blocked (high-risk AND irreversible).
    let packets: Vec<_> = [
        ("low-risk reversible", 100, 900),
        ("high-risk reversible", 800, 800),
        ("low-risk irreversible", 200, 100),
        ("high-risk irreversible", 900, 100),
    ]
    .into_iter()
    .map(|(statement, risk, reversibility)| propose(spec(statement, risk, reversibility)).unwrap())
    .collect();

    let queue = ProbeQueue::from_hypotheses(&packets);
    let report = serde_json::json!({
        "total": queue.requests().len(),
        "execution_eligible": queue.execution_eligible().len(),
        "queue": queue,
    });
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
