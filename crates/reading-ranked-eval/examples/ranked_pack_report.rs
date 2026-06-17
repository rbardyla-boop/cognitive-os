//! READ-9 — run the title-ranked reader across the READ-4 corpus pack and print
//! the no-regression comparison against the budgeted reader, plus the measured
//! title-priority win on a constructed scenario.
//!
//! `cargo run --example ranked_pack_report -p reading-ranked-eval`. For every
//! fixture it prints the ranked outcome and whether it matches the budgeted answer.
//! Exits non-zero if any ranked answer is false-grounded OR any fixture regresses
//! vs budgeted (ranking must only reorder on the committed pack).

use reading_autonomy::ReaderBounds;
use reading_ranked_eval::{evaluate_ranked_pack, title_priority_demo, RankedOutcome};
use std::process::ExitCode;

fn main() -> ExitCode {
    let bounds = ReaderBounds::default();
    let report = evaluate_ranked_pack(bounds);

    println!(
        "ranked pack over {} READ-4 fixtures (bounds: max_steps={} max_spans={} max_finalize_attempts={})",
        report.total, bounds.max_steps, bounds.max_spans, bounds.max_finalize_attempts
    );
    println!(
        "{} answered, {} coverage misses, {} regressions vs budgeted, {} false-grounded",
        report.answered,
        report.coverage_misses.len(),
        report.regressions.len(),
        report.false_grounded.len()
    );
    for score in &report.scores {
        let same = if score.matches_budgeted {
            "==budgeted"
        } else {
            "!=BUDGETED"
        };
        match &score.outcome {
            RankedOutcome::Answered {
                answer,
                trace_hash,
                claims,
            } => println!(
                "  {} ANSWERED claims={claims} {same} trace_hash={trace_hash:#018x} answer={answer:?}",
                score.name
            ),
            RankedOutcome::CoverageMiss { reason } => println!(
                "  {} COVERAGE_MISS {same} reason={reason}",
                score.name
            ),
        }
    }

    let demo = title_priority_demo();
    println!(
        "title-priority demo (relevant doc filed second, 1-span budget): budgeted_answered={} ranked_answered={} ranked_answer={:?} false_grounded={} stable_across_file_order={}",
        demo.budgeted_answered,
        demo.ranked_answered,
        demo.ranked_answer,
        demo.ranked_false_grounded,
        demo.stable_across_file_order
    );

    if !report.false_grounded.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} false-grounded ranked answer(s)",
            report.false_grounded.len()
        );
        return ExitCode::FAILURE;
    }
    if !report.regressions.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} fixture(s) regressed vs budgeted (ranking must only reorder)",
            report.regressions.len()
        );
        return ExitCode::FAILURE;
    }
    if demo.ranked_false_grounded || !demo.ranked_answered || demo.budgeted_answered {
        eprintln!("ACCEPTANCE FAIL: title-priority demo did not show the expected ranking win");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
