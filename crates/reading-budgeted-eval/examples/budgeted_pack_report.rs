//! READ-8 — run the budgeted selective reader across the READ-4 corpus pack and
//! print the focus comparison against the blunt READ-6 reader.
//!
//! `cargo run --example budgeted_pack_report -p reading-budgeted-eval`. For every
//! fixture it prints blunt-vs-budgeted claim counts and the budgeted outcome
//! (answered with its focused answer, or a classified coverage miss). Exits
//! non-zero if any budgeted answer is false-grounded (the unsafe class is zero).

use reading_autonomy::ReaderBounds;
use reading_budgeted_eval::{evaluate_budgeted_pack, BudgetedOutcome};
use std::process::ExitCode;

fn main() -> ExitCode {
    let bounds = ReaderBounds::default();
    let report = evaluate_budgeted_pack(bounds);

    println!(
        "budgeted pack over {} READ-4 fixtures (bounds: max_steps={} max_spans={} max_finalize_attempts={})",
        report.total, bounds.max_steps, bounds.max_spans, bounds.max_finalize_attempts
    );
    println!(
        "claims: blunt {} -> budgeted {}  ({} fixtures more focused, {} answered, {} coverage misses, {} false-grounded)",
        report.total_blunt_claims,
        report.total_budgeted_claims,
        report.more_focused,
        report.answered,
        report.coverage_misses.len(),
        report.false_grounded.len()
    );
    for score in &report.scores {
        let focus = if score.more_focused { " (focused)" } else { "" };
        match &score.outcome {
            BudgetedOutcome::Answered { answer, trace_hash, claims } => println!(
                "  {} ANSWERED blunt_claims={} budgeted_claims={claims}{focus} trace_hash={trace_hash:#018x} answer={answer:?}",
                score.name, score.blunt_claims
            ),
            BudgetedOutcome::CoverageMiss { reason } => println!(
                "  {} COVERAGE_MISS blunt_claims={} reason={reason}",
                score.name, score.blunt_claims
            ),
        }
    }

    if !report.false_grounded.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} false-grounded budgeted answer(s)",
            report.false_grounded.len()
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
