//! INT-0 end-to-end trace demo (runnable).
//!
//! Builds the single deterministic [`cognitive_demo::CognitiveTrace`] and prints it as the
//! auditable JSON record. The trace is a pure function of fixed inputs, so two runs are
//! byte-identical (the release gate double-runs this and diffs the output to prove replay,
//! then greps the trace for the machine-checkable verdicts: verified reading start, hypothesis
//! cites the receipt by hash, linked chain, no execution, quarantine, refused promotion, no
//! evidence, training unmoved). It exits non-zero if the pipeline fails to produce a faithful
//! trace, so a broken integration trips the gate.

use cognitive_demo::CognitiveTrace;

fn main() {
    match CognitiveTrace::demo() {
        Ok(trace) => {
            println!("{}", trace.to_json());
        }
        Err(e) => {
            eprintln!("cognitive_trace_demo: failed to build the end-to-end trace: {e}");
            std::process::exit(1);
        }
    }
}
