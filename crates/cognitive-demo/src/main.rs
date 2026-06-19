//! cognitive-demo — INT-1, the operator CLI / report surface for the INT-0 trace.
//!
//! A thin, deterministic command surface so a human can inspect the end-to-end cognitive trace
//! without reading Rust structs or test output:
//!
//!   cognitive-demo trace  [--out PATH]                       # write the canonical CognitiveTrace JSON
//!   cognitive-demo report --trace PATH [--out PATH]          # write a plain operator report
//!   cognitive-demo replay --trace PATH                       # confirm the trace replays byte-identically
//!   cognitive-demo ask    --trace PATH --question SLUG [...]  # answer one enumerated audit question
//!   cognitive-demo questions                                 # list the finite audit-question set
//!
//! INT-2 adds the interrogation surface: `questions` lists the finite, enumerated audit-question set,
//! and `ask` answers exactly one of those questions about a provided trace — there is no free-form /
//! natural-language path (an unknown slug fails closed), and `ask` re-derives the canonical trace and
//! refuses a tampered file before answering, exactly like `report`/`replay`.
//!
//! This binary is ONLY an I/O shell: it parses argv and reads/writes files, then delegates ALL
//! logic to the pure [`cognitive_demo`] library (`run_trace` / `run_report` / `run_replay` /
//! `run_ask` / `list_questions`). It
//! holds no executor, spawns no process, opens no socket, and consults no clock or entropy — the
//! trace it serves is a pure function of fixed inputs, and `report`/`replay` re-derive the
//! canonical trace and REFUSE any provided file that is not byte-for-byte that trace, so a tampered
//! or foreign trace can never be laundered into a report or a passing replay. `std::fs` lives ONLY
//! here (never in the library or the example), which the release gate enforces.

use cognitive_demo::{list_questions, run_ask, run_replay, run_report, run_trace};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(msg) = dispatch(&args) {
        eprintln!("cognitive-demo: {msg}");
        std::process::exit(1);
    }
}

/// Route the subcommand. Returns a human-readable error string on any failure; `main` prints it to
/// stderr and exits non-zero. No subcommand performs any action beyond reading/writing the trace
/// or report file — there is no execution, promotion, or training path here.
fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("trace") => {
            // Build the canonical trace (pure) and emit its JSON.
            let json = run_trace().map_err(|e| e.to_string())?;
            emit(&json, flag_value(args, "--out"))
        }
        Some("report") => {
            // Read the provided trace, verify it IS the canonical trace, render the operator report.
            let content = read_trace(args)?;
            let report = run_report(&content).map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("replay") => {
            // Re-derive the canonical trace and require the provided one to be byte-identical.
            let content = read_trace(args)?;
            run_replay(&content).map_err(|e| e.to_string())?;
            println!("replay: OK — the trace is the byte-identical canonical trace");
            Ok(())
        }
        Some("ask") => {
            // Answer ONE enumerated audit question. The slug is checked against the closed enum
            // (unknown → fail closed) and the trace is re-derived/verified before any answer.
            let content = read_trace(args)?;
            let question = flag_value(args, "--question").ok_or(
                "this command requires --question <slug> (see `cognitive-demo questions`)",
            )?;
            let answer = run_ask(&content, question).map_err(|e| e.to_string())?;
            emit(&answer, flag_value(args, "--out"))
        }
        Some("questions") => {
            // List the finite, enumerated audit-question set (no trace needed — this is the menu).
            print!("{}", list_questions());
            Ok(())
        }
        _ => Err(usage()),
    }
}

/// Read the file named by `--trace PATH`. The CONTENT is never trusted as authority — it is only
/// compared against the re-derived canonical trace by the library — so this is a plain file read.
fn read_trace(args: &[String]) -> Result<String, String> {
    let path = flag_value(args, "--trace").ok_or("this command requires --trace <path>")?;
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))
}

/// Write `content` to `--out PATH` if given, otherwise print it to stdout. A file write stores the
/// EXACT bytes (no extra newline), so a trace written by `trace --out` replays byte-identically.
fn emit(content: &str, out: Option<&str>) -> Result<(), String> {
    match out {
        Some(path) => {
            std::fs::write(path, content).map_err(|e| format!("cannot write {path}: {e}"))
        }
        None => {
            println!("{content}");
            Ok(())
        }
    }
}

/// The value following `flag` in argv (e.g. `--out file.json` → `Some("file.json")`), or `None`.
fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn usage() -> String {
    "usage: cognitive-demo <trace [--out PATH] | report --trace PATH [--out PATH] | \
     replay --trace PATH | ask --trace PATH --question SLUG [--out PATH] | questions>"
        .to_string()
}
