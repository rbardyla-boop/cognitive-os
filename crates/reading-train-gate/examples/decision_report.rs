//! P12 — print the training-justification decision for the live P11 eval.
//!
//! `cargo run --example decision_report -p reading-train-gate`. Prints the
//! machine-checkable decision (training_justified + blockers + cited fixture
//! ids). Exits non-zero only if the decision is internally inconsistent — i.e.
//! it claims training is justified without citing any clean, recurring failure
//! (an unjustified "yes"). For the current battery the decision is a blocked
//! "no", so it exits 0.

use reading_train_gate::decide_from_eval;

fn main() {
    let decision = decide_from_eval();

    println!("training_justified: {}", decision.training_justified);
    println!("safety_fix_required: {}", decision.safety_fix_required);
    println!("reason: {}", decision.reason);
    if !decision.cited_failures.is_empty() {
        println!(
            "cited clean failures: {}",
            decision.cited_failures.join(", ")
        );
    }
    if !decision.blockers.is_empty() {
        println!("blockers:");
        for blocker in &decision.blockers {
            println!("  - {blocker}");
        }
    }

    // A "train" verdict MUST cite at least one clean, recurring failure. An
    // unjustified yes is a gate failure.
    if decision.training_justified && decision.cited_failures.is_empty() {
        eprintln!("INCONSISTENT: training_justified=true with no cited clean failures");
        std::process::exit(1);
    }
}
