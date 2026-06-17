//! READ-3 — the `read0` binary: run / verify / replay a real-corpus reading.
//!
//!   read0 run <docs_dir> <question> <plan.json> <out.json>
//!   read0 verify <out.json>
//!   read0 replay <out.json>

use reading_cli::{replay_run, run_reading, verify_run};
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match dispatch(&args) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn dispatch(args: &[String]) -> Result<String, String> {
    match args.get(1).map(String::as_str) {
        Some("run") => {
            let docs = arg(args, 2, "docs_dir")?;
            let question = arg(args, 3, "question")?;
            let plan = arg(args, 4, "plan.json")?;
            let out = arg(args, 5, "out.json")?;
            let file = run_reading(Path::new(docs), question, Path::new(plan), Path::new(out))
                .map_err(|e| e.to_string())?;
            Ok(format!("verified: {}", file.answer))
        }
        Some("verify") => {
            let out = arg(args, 2, "out.json")?;
            let r = verify_run(Path::new(out)).map_err(|e| e.to_string())?;
            Ok(format!(
                "verify: passed={} grounded={} answer_supported={} replay_matches={}",
                r.passed, r.grounded, r.answer_supported, r.replay_matches
            ))
        }
        Some("replay") => {
            let out = arg(args, 2, "out.json")?;
            replay_run(Path::new(out)).map_err(|e| e.to_string())?;
            Ok("replay: MATCH".to_string())
        }
        _ => Err(usage()),
    }
}

fn arg<'a>(args: &'a [String], index: usize, name: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing <{name}>\n{}", usage()))
}

fn usage() -> String {
    "usage:\n  read0 run <docs_dir> <question> <plan.json> <out.json>\n  read0 verify <out.json>\n  read0 replay <out.json>".to_string()
}
