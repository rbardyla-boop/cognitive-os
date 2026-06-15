//! P10 — run the baseline failure-profile eval and print the report.
//!
//! `cargo run --example baseline_report -p reading-adapter`. Records the baseline
//! model's score and failure categories against the hardened codec + READ-1
//! verifier. Exits non-zero if the safety boundary does not hold (a verbatim
//! grounded sequence must finalize; the fabricated-but-cited claim must be
//! rejected as Unverified) — so the runnable artifact is also a gate.

use reading_adapter::{baseline_outputs, baseline_report, Outcome};
use reading_codec::RejectKind;

fn main() {
    let (corpus, question, _) = reading_substrate::fixture();
    let report = baseline_report(&corpus, &question, &baseline_outputs());

    println!(
        "baseline profile: {} finalized / {} total  ({} accepted-partial, {} rejected)",
        report.finalized, report.total, report.accepted_partial, report.rejected
    );
    println!("failure categories:");
    for (category, count) in &report.by_category {
        println!("  {category}: {count}");
    }
    for entry in &report.entries {
        println!("  [{}] {:?}", entry.name, entry.outcome);
    }

    let verbatim_ok = report
        .entries
        .iter()
        .any(|e| e.name == "verbatim_grounded_full_sequence" && e.outcome == Outcome::Finalized);
    let fabricated_blocked = report.entries.iter().any(|e| {
        e.name == "fabricated_supported_claim"
            && e.outcome == Outcome::Rejected(RejectKind::Unverified)
    });
    if !verbatim_ok || !fabricated_blocked {
        eprintln!("SAFETY BOUNDARY VIOLATED: verbatim_ok={verbatim_ok} fabricated_blocked={fabricated_blocked}");
        std::process::exit(1);
    }
}
