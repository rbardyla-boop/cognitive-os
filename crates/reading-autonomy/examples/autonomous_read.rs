//! READ-6 — run the bounded autonomous reader over the canonical corpus.
//!
//! `cargo run --example autonomous_read -p reading-autonomy`. Prints the bounded
//! plan the reader proposed, the bounds it respected, and the codec's decision.
//! Exits non-zero if the grounded plan did not finalize a verifier-approved
//! answer (the autonomous loop must end in a verified answer, never a bypass).

use reading_autonomy::{read, ReaderBounds};

fn main() -> std::process::ExitCode {
    let (corpus, question, _) = reading_substrate::fixture();
    let bounds = ReaderBounds::default();
    let outcome = read(&corpus, &question, bounds);

    println!("question: {question}");
    println!(
        "bounds: max_steps={} max_spans={} max_finalize_attempts={}",
        bounds.max_steps, bounds.max_spans, bounds.max_finalize_attempts
    );
    println!(
        "used: steps={} spans_read={} finalize_attempts={}",
        outcome.steps, outcome.spans_read, outcome.finalize_attempts
    );
    match &outcome.decision {
        Ok(decoded) => match &decoded.finalized {
            Some(run) => {
                println!("decision: FINALIZED (verifier-authorized)");
                println!("answer: {}", run.proof.answer_text);
            }
            None => println!("decision: accepted partial (no finalized answer)"),
        },
        Err(error) => println!("decision: REJECTED {error:?}"),
    }

    if !outcome.finalized() {
        eprintln!("AUTONOMY FAIL: the bounded plan did not finalize a verified answer");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
