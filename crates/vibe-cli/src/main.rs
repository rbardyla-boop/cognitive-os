//! `vibe` — the local operator command for Cognitive OS.
//!
//!   vibe run <scenario.json> [out.json]   record a deterministic run
//!   vibe replay <recorded_run.json>       re-derive the run, report the hash
//!   vibe verify <recorded_run.json>       check the run is authentic (exit 1 if not)

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match dispatch(&args) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str =
    "usage: vibe <run <scenario.json> [out.json] | replay <run.json> | verify <run.json>>";

fn dispatch(args: &[String]) -> Result<String, String> {
    match args.get(1).map(String::as_str) {
        Some("run") => {
            let scenario_path = args
                .get(2)
                .ok_or("usage: vibe run <scenario.json> [out.json]")?;
            let scenario = read(scenario_path)?;
            let outcome = vibe_cli::run_scenario(&scenario)?;
            match args.get(3) {
                Some(out_path) => {
                    std::fs::write(out_path, &outcome.recorded_run_json)
                        .map_err(|e| format!("write {out_path}: {e}"))?;
                    Ok(format!(
                        "recorded {} ticks (final vibe {}, run_hash {:016x}) -> {out_path}",
                        outcome.ticks, outcome.final_vibe_micros, outcome.run_hash
                    ))
                }
                None => Ok(outcome.recorded_run_json),
            }
        }
        Some("replay") => {
            let path = args
                .get(2)
                .ok_or("usage: vibe replay <recorded_run.json>")?;
            let recorded = read(path)?;
            let r = vibe_cli::replay_run(&recorded)?;
            Ok(format!(
                "replay {} ticks: run_hash {:016x} (recorded {:016x}) -> {}",
                r.ticks,
                r.recomputed_run_hash,
                r.expected_run_hash,
                if r.matches { "MATCH" } else { "MISMATCH" }
            ))
        }
        Some("verify") => {
            let path = args
                .get(2)
                .ok_or("usage: vibe verify <recorded_run.json>")?;
            let recorded = read(path)?;
            if vibe_cli::verify_run(&recorded)? {
                Ok("verify: OK (authentic, reproducible)".to_string())
            } else {
                Err("verify: FAILED — recorded run is tampered or corrupt".to_string())
            }
        }
        _ => Err(USAGE.to_string()),
    }
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))
}
