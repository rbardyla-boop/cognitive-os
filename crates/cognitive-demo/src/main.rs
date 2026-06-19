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
//!   cognitive-demo bundle        --out DIR                   # write a reproducible operator repro pack
//!   cognitive-demo bundle-verify --path DIR                  # re-derive the pack and refuse any tamper
//!
//! INT-2 adds the interrogation surface: `questions` lists the finite, enumerated audit-question set,
//! and `ask` answers exactly one of those questions about a provided trace — there is no free-form /
//! natural-language path (an unknown slug fails closed), and `ask` re-derives the canonical trace and
//! refuses a tampered file before answering, exactly like `report`/`replay`.
//!
//! INT-3 adds the repro bundle: `bundle` writes a fixed pack (trace.json, report.txt, questions.txt,
//! manifest.json) purely derived from the canonical trace; `bundle-verify` RE-DERIVES every file and
//! byte-compares, refusing any tampered/missing/foreign file. The bundle is a DEMONSTRATION — it
//! creates no evidence and no authority. The filesystem reads/writes for the pack live here, in the
//! shell; the library that derives and verifies the bundle stays pure.
//!
//! This binary is ONLY an I/O shell: it parses argv and reads/writes files, then delegates ALL
//! logic to the pure [`cognitive_demo`] library (`run_trace` / `run_report` / `run_replay` /
//! `run_ask` / `list_questions` / `canonical_bundle` / `verify_bundle` / `scenario_bundle` /
//! `verify_scenario_bundle` / `scenario_pack_manifest`). It
//! holds no executor, spawns no process, opens no socket, and consults no clock or entropy — the
//! trace it serves is a pure function of fixed inputs, and `report`/`replay` re-derive the
//! canonical trace and REFUSE any provided file that is not byte-for-byte that trace, so a tampered
//! or foreign trace can never be laundered into a report or a passing replay. `std::fs` lives ONLY
//! here (never in the library or the example), which the release gate enforces.

use cognitive_demo::{
    canonical_bundle, list_questions, list_scenarios, run_ask, run_replay, run_report, run_trace,
    scenario_bundle, scenario_pack_manifest, verify_bundle, verify_scenario_bundle,
    verify_scenario_pack_manifest, Scenario, BUNDLE_BOUNDARY_LINES, BUNDLE_FILES,
    MTRACE_BOUNDARY_LINES, PACK_MANIFEST_FILE,
};

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
        Some("bundle") => {
            // Derive the canonical repro pack (pure) and write every file into --out with exact bytes.
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let files = canonical_bundle().map_err(|e| e.to_string())?;
            write_bundle(out_dir, &files)?;
            print!("{}", bundle_summary(out_dir, &files));
            Ok(())
        }
        Some("bundle-verify") => {
            // Read the provided pack, then verify it by RE-DERIVING the canonical pack and byte-comparing
            // every file (the file contents are never trusted as authority).
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_bundle(dir)?;
            verify_bundle(&provided).map_err(|e| e.to_string())?;
            print!("{}", bundle_verify_summary());
            Ok(())
        }
        Some("scenarios") => {
            // List the finite scenario set (no inputs needed — this is the menu).
            print!("{}", list_scenarios());
            Ok(())
        }
        Some("scenario-pack") => {
            // Write one bundle subdirectory per scenario plus the scenario-pack manifest (pure derivation).
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let file_count = write_scenario_pack(out_dir)?;
            print!("{}", scenario_pack_summary(out_dir, file_count));
            Ok(())
        }
        Some("scenario-verify") => {
            // Verify every scenario bundle AND the pack manifest by RE-DERIVING each and byte-comparing.
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            for scenario in Scenario::ALL {
                let sub = format!("{dir}/{}", scenario.slug());
                let provided = read_bundle(&sub)?;
                verify_scenario_bundle(scenario, &provided)
                    .map_err(|e| format!("{}: {e}", scenario.slug()))?;
            }
            let pack_path = format!("{dir}/{PACK_MANIFEST_FILE}");
            let pack = std::fs::read_to_string(&pack_path)
                .map_err(|e| format!("cannot read {pack_path}: {e}"))?;
            verify_scenario_pack_manifest(&pack).map_err(|e| e.to_string())?;
            print!("{}", scenario_verify_summary());
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

/// Write each (name, content) bundle file into `dir` (created if needed) with EXACT bytes, so every
/// file re-reads and re-derives byte-identically. This is the bundle command's only side effect.
fn write_bundle(dir: &str, files: &[(&str, String)]) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {dir}: {e}"))?;
    for (name, content) in files {
        let path = format!("{dir}/{name}");
        std::fs::write(&path, content).map_err(|e| format!("cannot write {path}: {e}"))?;
    }
    Ok(())
}

/// Read the expected bundle files from `dir`. A file that is absent is simply omitted (so
/// `verify_bundle` reports it missing rather than this shell guessing); any other read error is
/// propagated. The CONTENT is never trusted — it is only re-derived and byte-compared by the library.
fn read_bundle(dir: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for name in BUNDLE_FILES {
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
            found.push((name.to_string(), content));
        }
    }
    Ok(found)
}

/// The human summary printed after `bundle` writes the pack: the files written and the boundary.
fn bundle_summary(dir: &str, files: &[(&str, String)]) -> String {
    let mut out = format!("bundle: wrote {} files to {dir}\n", files.len());
    for (name, content) in files {
        out.push_str(&format!("    {name} ({} bytes)\n", content.len()));
    }
    out.push_str("BOUNDARY\n");
    for line in BUNDLE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `bundle-verify` accepts a bundle (every file re-derived).
fn bundle_verify_summary() -> String {
    let mut out = String::from(
        "bundle-verify: OK — every bundle file re-derives byte-identically from the canonical trace\n",
    );
    out.push_str("BOUNDARY\n");
    for line in BUNDLE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// Write the scenario pack: one bundle subdirectory per scenario (each with the four bundle files) plus
/// the scenario-pack manifest. Returns the number of bundle files written. The bundle CONTENT is a pure
/// derivation from the frozen tracks; this shell only places the bytes on disk.
fn write_scenario_pack(dir: &str) -> Result<usize, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {dir}: {e}"))?;
    let mut file_count = 0;
    for scenario in Scenario::ALL {
        let sub = format!("{dir}/{}", scenario.slug());
        let files = scenario_bundle(scenario).map_err(|e| e.to_string())?;
        write_bundle(&sub, &files)?;
        file_count += files.len();
    }
    let pack = scenario_pack_manifest().map_err(|e| e.to_string())?;
    let pack_path = format!("{dir}/{PACK_MANIFEST_FILE}");
    std::fs::write(&pack_path, &pack).map_err(|e| format!("cannot write {pack_path}: {e}"))?;
    Ok(file_count)
}

/// The human summary printed after `scenario-pack` writes the pack: the scenarios and the boundary.
fn scenario_pack_summary(dir: &str, file_count: usize) -> String {
    let mut out = format!(
        "scenario-pack: wrote {} scenarios ({file_count} bundle files) + {PACK_MANIFEST_FILE} to {dir}\n",
        Scenario::ALL.len()
    );
    for s in Scenario::ALL {
        out.push_str(&format!("    {}/\n", s.slug()));
    }
    out.push_str("BOUNDARY\n");
    for line in MTRACE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `scenario-verify` accepts the whole pack.
fn scenario_verify_summary() -> String {
    let mut out = String::from(
        "scenario-verify: OK — every scenario bundle and the pack manifest re-derive byte-identically\n",
    );
    out.push_str("BOUNDARY\n");
    for line in MTRACE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

fn usage() -> String {
    "usage: cognitive-demo <trace [--out PATH] | report --trace PATH [--out PATH] | \
     replay --trace PATH | ask --trace PATH --question SLUG [--out PATH] | questions | \
     bundle --out DIR | bundle-verify --path DIR | scenarios | scenario-pack --out DIR | \
     scenario-verify --path DIR>"
        .to_string()
}
