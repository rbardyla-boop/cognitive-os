//! Deterministic HYP-0 demo: propose a fixed hypothesis from a cited trace and print
//! its packet as JSON. The output is a pure function of the fixed inputs, so running
//! it twice yields byte-identical output — the gate diffs two runs to prove the
//! trace-replay / deterministic-scoring property at the binary level.

use hypothesis_layer::{propose, EvidenceRef, HypothesisSpec};
use std::process::ExitCode;

fn main() -> ExitCode {
    let spec = HypothesisSpec {
        statement: "The outage recurs because the failover never engaged.".to_string(),
        prior: 450,
        uncertainty: 700,
        test_cost: 40,
        risk: 250,
        reversibility: 850,
        evidence_inputs: vec![EvidenceRef {
            answer_hash: 0x0123_4567_89ab_cdef,
            memory_hash: 0xfedc_ba98_7654_3210,
            source_label: "incident-receipt".to_string(),
        }],
        probe_description: "Replay the failover path against the recorded trace.".to_string(),
    };
    match propose(spec) {
        Ok(packet) => match serde_json::to_string_pretty(&packet) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("serialize error: {e}");
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("propose error: {e}");
            ExitCode::FAILURE
        }
    }
}
