//! P9 — run the codec eval battery and print the pass/fail report.
//!
//! `cargo run --example eval_report -p reading-codec`. Exits non-zero if any
//! fixture's actual codec decision diverges from its required decision. This is
//! the deterministic gate the eventual model output must pass before training.

use reading_codec::{evaluate, CodecPolicy};

fn main() {
    let report = evaluate(CodecPolicy::strict());
    println!(
        "codec eval (strict): {}/{} passed, {} failed",
        report.passed, report.total, report.failed
    );
    for result in &report.results {
        println!(
            "  [{}] {} — {}",
            if result.matched { "PASS" } else { "FAIL" },
            result.name,
            result.detail
        );
    }
    if report.failed != 0 {
        std::process::exit(1);
    }
}
