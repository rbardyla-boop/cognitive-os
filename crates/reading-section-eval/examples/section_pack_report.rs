//! READ-10 — run the section-aware reader across the READ-4 corpus pack and print
//! the no-regression comparison against the budgeted reader, plus the measured
//! section-heading + multi-term win on constructed sectioned corpora.
//!
//! `cargo run --example section_pack_report -p reading-section-eval`. Exits
//! non-zero if any section answer is false-grounded, any fixture regresses vs
//! budgeted (the flat pack must only reorder), or the demo fails to show the win.

use reading_autonomy::ReaderBounds;
use reading_section_eval::{evaluate_section_pack, section_priority_demo, SectionOutcome};
use std::process::ExitCode;

fn main() -> ExitCode {
    let bounds = ReaderBounds::default();
    let report = evaluate_section_pack(bounds);

    println!(
        "section pack over {} READ-4 fixtures (bounds: max_steps={} max_spans={} max_finalize_attempts={})",
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
            SectionOutcome::Answered {
                answer,
                trace_hash,
                claims,
            } => println!(
                "  {} ANSWERED claims={claims} {same} trace_hash={trace_hash:#018x} answer={answer:?}",
                score.name
            ),
            SectionOutcome::CoverageMiss { reason } => {
                println!("  {} COVERAGE_MISS {same} reason={reason}", score.name)
            }
        }
    }

    let demo = section_priority_demo();
    println!(
        "section demo (1-span budget): heading[budgeted={} ranked={} answer={:?}] multiterm[budgeted={} ranked={} answer={:?}] false_grounded={} stable_across_section_order={}",
        demo.heading_budgeted_answered,
        demo.heading_ranked_answered,
        demo.heading_answer,
        demo.multiterm_budgeted_answered,
        demo.multiterm_ranked_answered,
        demo.multiterm_answer,
        demo.any_false_grounded,
        demo.stable_across_section_order
    );

    if !report.false_grounded.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} false-grounded section answer(s)",
            report.false_grounded.len()
        );
        return ExitCode::FAILURE;
    }
    if !report.regressions.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} fixture(s) regressed vs budgeted (flat pack must only reorder)",
            report.regressions.len()
        );
        return ExitCode::FAILURE;
    }
    let demo_ok = demo.heading_ranked_answered
        && !demo.heading_budgeted_answered
        && demo.multiterm_ranked_answered
        && !demo.multiterm_budgeted_answered
        && !demo.any_false_grounded
        && demo.stable_across_section_order;
    if !demo_ok {
        eprintln!("ACCEPTANCE FAIL: section/multi-term demo did not show the expected ranking win");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
