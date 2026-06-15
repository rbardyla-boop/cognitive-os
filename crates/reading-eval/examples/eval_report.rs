//! P11 — run the codec eval harness and print the full report.
//!
//! `cargo run --example eval_report -p reading-eval`. Prints the score, the
//! explicit false-accept / false-reject lists, the per-category tallies, the
//! failure-category histogram, and the single recommended next change. Exits
//! non-zero if the acceptance targets are not met: ≥ 30 fixtures and 0
//! false-accepts (a should-reject output must never be accepted).

use reading_eval::{cases, run, Verdict};

fn main() {
    let total_fixtures = cases().len();
    let report = run();

    println!(
        "codec eval: {}/{} correct over {} fixtures  ({} false-accepts, {} false-rejects)",
        report.correct,
        report.total,
        total_fixtures,
        report.false_accepts.len(),
        report.false_rejects.len()
    );

    println!("per-category:");
    for (category, tally) in &report.by_category {
        println!(
            "  {category}: {}/{} correct  (fa={}, fr={})",
            tally.correct, tally.total, tally.false_accepts, tally.false_rejects
        );
    }

    println!("failure categories (actual rejections):");
    for (category, count) in &report.failure_categories {
        println!("  {category}: {count}");
    }

    if !report.false_accepts.is_empty() {
        println!("FALSE-ACCEPTS (unsafe):");
        for c in &report.false_accepts {
            println!(
                "  [{}] expected {:?}, got {:?}",
                c.name, c.expected, c.actual
            );
        }
    }
    if !report.false_rejects.is_empty() {
        println!("false-rejects (classified):");
        for c in &report.false_rejects {
            println!(
                "  [{}] expected {:?}, got {:?}",
                c.name, c.expected, c.actual
            );
        }
    }

    let reason_mismatches: Vec<&str> = report
        .cases
        .iter()
        .filter(|c| c.reason_mismatch && c.verdict == Verdict::Correct)
        .map(|c| c.name.as_str())
        .collect();
    if !reason_mismatches.is_empty() {
        println!(
            "rejected-as-expected but wrong reason: {}",
            reason_mismatches.join(", ")
        );
    }

    println!("next change: {}", report.next_change);

    if total_fixtures < 30 {
        eprintln!("ACCEPTANCE FAIL: {total_fixtures} fixtures (< 30 required)");
        std::process::exit(1);
    }
    if !report.false_accepts.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} false-accept(s) (0 required)",
            report.false_accepts.len()
        );
        std::process::exit(1);
    }
}
