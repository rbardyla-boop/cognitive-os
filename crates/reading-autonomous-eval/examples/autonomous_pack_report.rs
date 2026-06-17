//! READ-7 — run the autonomous reader across the READ-4 corpus pack and print the
//! manual-plan vs autonomous-reader comparison.
//!
//! `cargo run --example autonomous_pack_report -p reading-autonomous-eval`. Drives
//! every READ-4 fixture corpus through the deterministic READ-6 reader (no
//! hand-written plans), independently re-verifies each finalized answer, and
//! prints per-fixture manual-vs-autonomous outcomes plus the aggregate. Exits
//! non-zero if ANY false-grounded answer is found (the unsafe class must be zero).

use reading_autonomous_eval::{evaluate_autonomous_pack, AutonomousOutcome, Comparison};
use reading_autonomy::ReaderBounds;
use std::process::ExitCode;

fn main() -> ExitCode {
    let bounds = ReaderBounds::default();
    let report = evaluate_autonomous_pack(bounds);

    println!(
        "autonomous pack over {} READ-4 fixtures (bounds: max_steps={} max_spans={} max_finalize_attempts={})",
        report.total, bounds.max_steps, bounds.max_spans, bounds.max_finalize_attempts
    );
    println!(
        "manual (hand-plan):     verified {}/{}  rejected {}/{}",
        report.manual_verified, report.total, report.manual_rejected, report.total
    );
    println!(
        "autonomous (reader):    verified {}/{}  rejected {}/{}",
        report.autonomous_verified, report.total, report.autonomous_rejected, report.total
    );
    println!(
        "comparison: both-verified {} | autonomous-verified-where-manual-rejected {} | false-rejects {} | false-grounded {}",
        report.both_verified,
        report.autonomous_verified_manual_rejected,
        report.false_rejects.len(),
        report.false_grounded.len()
    );
    for score in &report.scores {
        let tag = match score.comparison {
            Comparison::BothVerified => "agree:verified",
            Comparison::BothRejected => "agree:rejected",
            Comparison::AutonomousVerifiedManualRejected => "auton>manual (safe divergence)",
            Comparison::AutonomousRejectedManualVerified => "auton<manual (FALSE-REJECT)",
        };
        match &score.outcome {
            AutonomousOutcome::Verified { answer, trace_hash, spans_read } => println!(
                "  [{tag}] {} VERIFIED spans_read={spans_read} trace_hash={trace_hash:#018x} answer={answer:?}",
                score.name
            ),
            AutonomousOutcome::Rejected { reason } => {
                println!("  [{tag}] {} REJECTED reason={reason}", score.name)
            }
        }
    }

    if !report.false_grounded.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} false-grounded autonomous answer(s)",
            report.false_grounded.len()
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
