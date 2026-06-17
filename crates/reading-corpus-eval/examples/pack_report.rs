//! READ-4 — run the real-corpus eval pack and print the measured report.
//!
//! `cargo run --example pack_report -p reading-corpus-eval`. Drives every fixture
//! through read0 run → verify → replay and prints, per fixture: pass/fail verdict,
//! and either the rejection reason or the verified answer + trace hash. Exits
//! non-zero if there are fewer than 10 fixtures or any false-grounded answer (an
//! expected-rejected fixture that finalized a verified answer).

use reading_corpus_eval::{evaluate_pack, fixtures, Outcome, Workdir};
use std::process::ExitCode;

fn main() -> ExitCode {
    let total_fixtures = fixtures().len();
    let work = match Workdir::new() {
        Ok(w) => w,
        Err(e) => {
            eprintln!("could not create workdir: {e}");
            return ExitCode::FAILURE;
        }
    };
    let report = match evaluate_pack(work.path()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("eval failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "read0 corpus pack: {}/{} correct over {} fixtures  ({} false-grounded, {} false-rejects)",
        report.correct,
        report.total,
        total_fixtures,
        report.false_grounded.len(),
        report.false_rejects.len()
    );
    for result in &report.results {
        match &result.outcome {
            Outcome::Verified { answer, trace_hash } => println!(
                "  [{:?}] {} VERIFIED trace_hash={trace_hash:#018x} answer={answer:?}",
                result.verdict, result.name
            ),
            Outcome::Rejected { reason } => {
                println!(
                    "  [{:?}] {} REJECTED reason={reason}",
                    result.verdict, result.name
                )
            }
        }
    }

    if total_fixtures < 10 {
        eprintln!("ACCEPTANCE FAIL: {total_fixtures} fixtures (< 10 required)");
        return ExitCode::FAILURE;
    }
    if !report.false_grounded.is_empty() {
        eprintln!(
            "ACCEPTANCE FAIL: {} false-grounded answer(s)",
            report.false_grounded.len()
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
