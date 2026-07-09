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
//!   cognitive-demo failure-cases                             # list the finite negative-scenario set
//!   cognitive-demo failure-pack   --out DIR                  # forge forbidden authority; prove each rejected
//!   cognitive-demo failure-verify --path DIR                 # re-derive the failure pack and refuse any tamper
//!   cognitive-demo lit-intent-demo [--out PATH]               # emit the canonical LIT-INTENT-0 intent map
//!   cognitive-demo lit-intent-demo-verify --map PATH          # re-derive the demo map and refuse tamper
//!   cognitive-demo lit-intent-matrix [--out PATH]             # emit the LIT-INTENT-0 scenario matrix
//!   cognitive-demo lit-intent-matrix-verify --matrix PATH     # re-derive the matrix and refuse tamper
//!   cognitive-demo teach-map-demo [--out PATH]                # emit the canonical TEACH-0 lesson from a LIT-INTENT map
//!   cognitive-demo teach-map-demo-verify --lesson PATH        # re-derive the lesson and refuse tamper
//!   cognitive-demo teach-map-matrix [--out PATH]              # emit the TEACH-0 scenario matrix
//!   cognitive-demo teach-map-matrix-verify --matrix PATH      # re-derive the matrix and refuse tamper
//!   cognitive-demo learner-model-demo [--out PATH]            # emit the canonical LEARNER-MODEL-0 state map
//!   cognitive-demo learner-model-demo-verify --state PATH     # re-derive the state map and refuse tamper
//!   cognitive-demo learner-model-matrix [--out PATH]          # emit the LEARNER-MODEL-0 scenario matrix
//!   cognitive-demo learner-model-matrix-verify --matrix PATH  # re-derive the matrix and refuse tamper
//!   cognitive-demo learner-memory-demo [--out PATH]            # emit the canonical LEARNER-MEMORY-0 candidate
//!   cognitive-demo learner-memory-demo-verify --memory PATH    # re-derive the candidate and refuse tamper
//!   cognitive-demo learner-memory-matrix [--out PATH]          # emit the LEARNER-MEMORY-0 scenario matrix
//!   cognitive-demo learner-memory-matrix-verify --matrix PATH  # re-derive the matrix and refuse tamper
//!   cognitive-demo learner-journal-demo [--out PATH]           # emit the canonical LEARNER-MEMORY-1 journal run
//!   cognitive-demo learner-journal-demo-verify --journal PATH  # re-derive the journal run and refuse tamper
//!   cognitive-demo learner-journal-matrix [--out PATH]         # emit the LEARNER-MEMORY-1 scenario matrix
//!   cognitive-demo learner-journal-matrix-verify --matrix PATH # re-derive the matrix and refuse tamper
//!   cognitive-demo learner-journal-append --journal PATH --consent-operator S --consent-scope S  # consented append
//!   cognitive-demo converse-demo [--out PATH]                  # emit the canonical CONVERSE-0 multi-turn transcript
//!   cognitive-demo converse-demo-verify --transcript PATH      # re-derive the transcript and refuse tamper
//!   cognitive-demo converse-matrix [--out PATH]                # emit the CONVERSE-0 scenario matrix
//!   cognitive-demo converse-matrix-verify --matrix PATH        # re-derive the matrix and refuse tamper
//!   cognitive-demo converse-run --input-dir DIR --script PATH [--out PATH]  # converse over a LOCAL .txt vault (grounded-or-refused per turn)
//!   cognitive-demo converse-run-verify --input-dir DIR --script PATH --transcript PATH  # re-derive the transcript and refuse tamper
//!   cognitive-demo learning-session-demo [--out PATH]          # emit the canonical SESSION-LOOP-0 run
//!   cognitive-demo learning-session-demo-verify --session PATH # re-derive the session run and refuse tamper
//!   cognitive-demo learning-session-matrix [--out PATH]        # emit the SESSION-LOOP-0 scenario matrix
//!   cognitive-demo learning-session-matrix-verify --matrix PATH # re-derive the matrix and refuse tamper
//!   cognitive-demo learning-arc-demo [--out PATH]               # emit the canonical MULTI-SESSION-0 arc
//!   cognitive-demo learning-arc-demo-verify --arc PATH          # re-derive the arc and refuse tamper
//!   cognitive-demo learning-arc-matrix [--out PATH]             # emit the MULTI-SESSION-0 scenario matrix
//!   cognitive-demo learning-arc-matrix-verify --matrix PATH     # re-derive the matrix and refuse tamper
//!   cognitive-demo game-evidence-demo [--out PATH]              # emit the canonical GAME-EVIDENCE-0 packet
//!   cognitive-demo game-evidence-demo-verify --packet PATH      # re-derive the packet and refuse tamper
//!   cognitive-demo game-evidence-matrix [--out PATH]            # emit the GAME-EVIDENCE-0 scenario matrix
//!   cognitive-demo game-evidence-matrix-verify --matrix PATH    # re-derive the matrix and refuse tamper
//!   cognitive-demo wow-state-demo [--out PATH]                  # emit the canonical WOW-STATE-0 navigation snapshot
//!   cognitive-demo wow-state-demo-verify --snapshot PATH        # re-derive the snapshot and refuse tamper
//!   cognitive-demo wow-state-matrix [--out PATH]                # emit the WOW-STATE-0 scenario matrix
//!   cognitive-demo wow-state-matrix-verify --matrix PATH        # re-derive the matrix and refuse tamper
//!   cognitive-demo wow-taskplan-demo [--out PATH]               # emit the canonical WOW-TASKPLAN-0 plan proposal
//!   cognitive-demo wow-taskplan-demo-verify --plan PATH         # re-derive the plan and refuse tamper
//!   cognitive-demo wow-taskplan-matrix [--out PATH]             # emit the WOW-TASKPLAN-0 scenario matrix
//!   cognitive-demo wow-taskplan-matrix-verify --matrix PATH     # re-derive the matrix and refuse tamper
//!   cognitive-demo controller-bridge-demo [--out PATH]          # emit the canonical CONTROLLER-BRIDGE-0 dry-run envelope set
//!   cognitive-demo controller-bridge-demo-verify --envelope PATH # re-derive the envelope set and refuse tamper
//!   cognitive-demo controller-bridge-matrix [--out PATH]        # emit the CONTROLLER-BRIDGE-0 scenario matrix
//!   cognitive-demo controller-bridge-matrix-verify --matrix PATH # re-derive the matrix and refuse tamper
//!   cognitive-demo doc-trace        --input PATH [--out PATH] # trace a LOCAL operator document (verify-first)
//!   cognitive-demo doc-report       --input PATH --trace PATH # render the doc report (re-derive + refuse tamper)
//!   cognitive-demo doc-bundle       --input PATH --out DIR    # repro bundle over the operator document
//!   cognitive-demo doc-bundle-verify --input PATH --path DIR  # re-derive the doc bundle and refuse any tamper
//!   cognitive-demo corpus-trace        --input-dir DIR [--out PATH]      # trace a LOCAL `.txt` corpus (verify-first)
//!   cognitive-demo corpus-report       --input-dir DIR --trace PATH      # render the corpus report (re-derive + refuse tamper)
//!   cognitive-demo corpus-bundle       --input-dir DIR --out DIR         # repro bundle over the operator corpus
//!   cognitive-demo corpus-bundle-verify --input-dir DIR --path DIR       # re-derive the corpus bundle and refuse any tamper
//!   cognitive-demo corpus-scenarios                                      # list the finite corpus-input scenario set
//!   cognitive-demo corpus-scenario-pack   --out DIR                      # write the observed corpus input-integrity record + report
//!   cognitive-demo corpus-scenario-verify --path DIR                     # re-derive the corpus-scenario pack and refuse any tamper
//!   cognitive-demo corpus-scenario-matrix --path DIR [--out PATH]        # verify the pack, then emit the corpus input-integrity matrix
//!   cognitive-demo novelty-packet --input-dir DIR --corpus-trace PATH --frame PATH [--out PATH]  # hypothesis-only novelty packet
//!   cognitive-demo novelty-report --input-dir DIR --frame PATH --packet PATH [--out PATH]        # render the packet (re-derive + refuse tamper)
//!   cognitive-demo novelty-replay --input-dir DIR --frame PATH --packet PATH                     # confirm the packet replays byte-identically
//!   cognitive-demo dream-export --input-dir DIR --frame PATH [--seed N] [--weirdness W] [--dream-packet PATH] [--out PATH]  # bridge a dream packet into the hypothesis-only path
//!   cognitive-demo dream-export-report --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH [--out PATH]   # render the export (re-derive + refuse tamper)
//!   cognitive-demo dream-export-replay --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH                # confirm the export replays byte-identically
//!   cognitive-demo dream-export-scenarios                                                                                   # list the dream-export scenario set
//!   cognitive-demo dream-export-matrix --input-dir DIR --frame PATH [--seed N] [--weirdness W] [--out PATH]                 # emit the scenario matrix (clean verifies; tampers refused)
//!   cognitive-demo dream-export-matrix-report --input-dir DIR --frame PATH [--seed N] [--weirdness W] --matrix PATH [--out PATH]  # render the matrix (re-derive + refuse tamper)
//!   cognitive-demo dream-export-matrix-verify --input-dir DIR --frame PATH [--seed N] [--weirdness W] --matrix PATH         # confirm the matrix replays byte-identically
//!
//! DREAM-EXPORT-0 adds the dream provenance bridge ON TOP of the existing hypothesis-only path: it re-derives the
//! terminal `DreamPacket` (from `dream-engine`) for the SAME corpus + frame + dials, builds a `HypothesisSpec`
//! from the dream's distortion + verified grounding, and calls the EXISTING `hypothesis_layer::propose`. The
//! result is a real `HypothesisPacket` carrying the EXISTING `Authority::HypothesisOnly`, wrapped with a
//! `DreamExportReceipt` that preserves dream-origin provenance OUTSIDE the frozen authority model — so a
//! dream-exported hypothesis stays DISTINGUISHABLE and auditable, and the dream's private `dream_only` authority
//! NEVER crosses. Export refuses a tampered `--dream-packet`; report/replay re-derive the bundle and refuse a
//! tampered `--export`, which is why they require `--input-dir` + `--frame` (never parsing the artifact back).
//!
//! NOVELTY-0 adds the hypothesis-only novelty packet harness ON TOP of the verified corpus trace: given a
//! verified corpus trace (re-derived from `--input-dir`, with `--corpus-trace` byte-verified against it) and an
//! operator `--frame`, `novelty-packet` emits a deterministic `NoveltyPacket` recording the frame's candidate
//! broken assumptions, the verified facts to preserve (each grounded VERBATIM in a verified corpus span), a
//! candidate hypothesis, falsifiers, and NON-EXECUTING probe requests. The packet carries `authority =
//! hypothesis_only` and an explicit `forbidden_uses` list, so it can never become evidence, execute, promote,
//! or train. There is NO model and NO score: the frame is read as DATA, never trusted as a fact, and an
//! unsupported preserved fact, an empty frame, a receipt-hash-stripped corpus trace, or any tampered packet is
//! REFUSED. `novelty-report`/`novelty-replay` re-derive the packet from the SAME corpus + frame and refuse a
//! tampered packet — that is why they require `--input-dir` + `--frame` alongside `--packet`. The packet
//! PROPOSES; it does not prove. P12 stays training_justified=false.
//!
//! CORPUS-2 adds the corpus scenario pack / input-integrity matrix: a finite, enum-backed set of VALID and
//! INVALID corpus inputs, each OBSERVED by running the REAL CORPUS-0 admission filter / check / verifier — a
//! clean two-document corpus verifies; an empty corpus, a hidden-only or non-`.txt`-only corpus, an absolute /
//! `..` / escaping path, a grounding-document mutation, a non-grounding side-document mutation, and a tampered
//! source/trace/report/manifest are each REFUSED. `corpus-scenario-pack` writes the observed-outcome record +
//! report; `corpus-scenario-verify` re-derives and refuses any tamper; `corpus-scenario-matrix` verifies the
//! pack then emits the matrix, which additionally records the verified case's SOURCE IDENTITY and a
//! `whole_corpus_bound` fact (mutating a non-grounding document leaves the attribution intact yet still fails
//! the bundle). Every scenario keeps the boundary closed: nothing executes, becomes evidence, promotes, or trains.
//!
//! CORPUS-0 adds the multi-document local corpus flow: `corpus-trace` reads a LOCAL DIRECTORY of `.txt`
//! documents (path-validated in this shell — absolute / `..` / symlink-escape refused; hidden / non-`.txt`
//! files refused; sorted for determinism), asks the FROZEN reader for the corpus's own first span, and
//! builds the SAME end-to-end trace from a VERIFIED reading receipt over the corpus (fails closed on an
//! empty corpus). The trace's structure hash binds EVERY document, so a tamper of any document — even a
//! non-grounding one — is refused. An unambiguous `corpus-source.json` records which document/span grounded
//! the answer. `corpus-report`/`corpus-bundle-verify` re-derive from the SAME corpus and REFUSE a tampered
//! corpus, source, trace, report, questions, or manifest. The corpus is read but never trusted: it executes
//! nothing, promotes nothing, and trains nothing.
//!
//! DOCFLOW-0 adds the operator-supplied document flow: `doc-trace` reads a LOCAL text file (path-validated
//! in this shell — absolute / `..` / symlink-escape refused), asks the FROZEN reader for the document's own
//! first span, and builds the SAME end-to-end trace from a VERIFIED reading receipt over that document; it
//! fails closed if the read does not verify. `doc-report`/`doc-bundle-verify` re-derive from the SAME
//! document and REFUSE a tampered document, trace, report, questions, or manifest. The flow reads local
//! input but never trusts it: it executes nothing, promotes nothing, and trains nothing.
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
//! MTRACE-2 adds the failure-injection pack: `failure-cases` lists a finite set of negative scenarios,
//! `failure-pack` forges a forbidden authority claim onto each canonical artifact and records that the
//! EXISTING re-derive-and-byte-compare verifier REFUSES it, and `failure-verify` re-derives the whole pack
//! and refuses any tamper. The forged bytes are never persisted as trusted state — only the rejections.
//!
//! This binary is ONLY an I/O shell: it parses argv and reads/writes files, then delegates ALL
//! logic to the pure [`cognitive_demo`] library (`run_trace` / `run_report` / `run_replay` /
//! `run_ask` / `list_questions` / `canonical_bundle` / `verify_bundle` / `scenario_bundle` /
//! `verify_scenario_pack` / `scenario_matrix` / `verify_scenario_matrix` / `scenario_matrix_report` /
//! `list_failure_cases` / `failure_pack_files` / `verify_failure_pack`).
//! It holds no executor, spawns no process, opens no socket, and consults no clock or entropy — the
//! trace it serves is a pure function of fixed inputs, and `report`/`replay` re-derive the
//! canonical trace and REFUSE any provided file that is not byte-for-byte that trace, so a tampered
//! or foreign trace can never be laundered into a report or a passing replay. `std::fs` lives ONLY
//! here (never in the library or the example), which the release gate enforces.

use cognitive_demo::{
    canonical_bundle, check_local_input_path, controller_bridge_demo, controller_bridge_demo_json,
    controller_bridge_matrix_json, converse_demo_json, converse_matrix_json,
    converse_run_from_text, converse_transcript_json, corpus_admits_filename, corpus_bundle,
    corpus_scenario_matrix, corpus_scenario_pack_files, doc_bundle, doc_scenario_matrix,
    doc_scenario_pack_files, dream_export_matrix, failure_pack_files, game_evidence_demo_json,
    game_evidence_matrix_json, learner_journal_append_at, learner_journal_demo_json,
    learner_journal_json_at, learner_journal_matrix_json, learner_journal_state_json,
    learner_memory_demo_json, learner_memory_matrix_json, learner_model_demo_json,
    learner_model_matrix_json, learning_arc_demo_json, learning_arc_matrix_json,
    learning_session_demo_json, learning_session_matrix_json, list_corpus_scenarios,
    list_doc_scenarios, list_dream_export_scenarios, list_failure_cases, list_questions,
    list_scenarios, literature_intent_demo_json, literature_intent_matrix_json,
    resolved_path_within, run_ask, run_corpus_report, run_corpus_trace, run_doc_report,
    run_doc_trace, run_dream_export, run_dream_export_matrix_report,
    run_dream_export_matrix_verify, run_dream_export_replay, run_dream_export_report,
    run_novelty_packet, run_novelty_replay, run_novelty_report, run_replay, run_report, run_trace,
    scenario_bundle, scenario_matrix, scenario_matrix_report, scenario_pack_manifest,
    teach_map_demo_json, teach_map_matrix_json, verify_bundle, verify_controller_bridge_demo_json,
    verify_controller_bridge_matrix_json, verify_converse_demo_json, verify_converse_matrix_json,
    verify_corpus_bundle, verify_corpus_scenario_pack, verify_doc_bundle, verify_doc_scenario_pack,
    verify_failure_pack, verify_game_evidence_demo_json, verify_game_evidence_matrix_json,
    verify_learner_journal_demo_json, verify_learner_journal_matrix_json,
    verify_learner_memory_demo_json, verify_learner_memory_matrix_json,
    verify_learner_model_demo_json, verify_learner_model_matrix_json,
    verify_learning_arc_demo_json, verify_learning_arc_matrix_json,
    verify_learning_session_demo_json, verify_learning_session_matrix_json,
    verify_literature_intent_demo_json, verify_literature_intent_matrix_json,
    verify_scenario_matrix, verify_scenario_pack, verify_teach_map_demo_json,
    verify_teach_map_matrix_json, verify_wow_state_demo_json, verify_wow_state_matrix_json,
    verify_wow_taskplan_demo_json, verify_wow_taskplan_matrix_json, wow_state_demo_json,
    wow_state_matrix_json, wow_taskplan_demo_json, wow_taskplan_matrix_json, ConverseConfig,
    LearnerJournalConsent, Scenario, BUNDLE_BOUNDARY_LINES, BUNDLE_FILES, CORPUS_BOUNDARY_LINES,
    CORPUS_BUNDLE_FILES, CORPUS_SCENARIO_BOUNDARY_LINES, CORPUS_SCENARIO_PACK_FILES,
    DOC_BOUNDARY_LINES, DOC_SCENARIO_BOUNDARY_LINES, DOC_SCENARIO_PACK_FILES,
    FAILURE_BOUNDARY_LINES, FAILURE_PACK_FILES, LEARNER_JOURNAL_DEMO_CANDIDATES,
    MATRIX_BOUNDARY_LINES, MTRACE_BOUNDARY_LINES, PACK_MANIFEST_FILE,
};
use cognitive_demo::{ControllerBridgeDecision, ControllerBridgeRun};
use serde::Serialize;

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
            verify_pack_at(dir)?;
            print!("{}", scenario_verify_summary());
            Ok(())
        }
        Some("scenario-matrix") => {
            // VERIFY the pack at --pack re-derives (refuse a tampered pack), then emit the canonical
            // coverage matrix to --out. The matrix is purely re-derived; the pack is never trusted.
            let dir = flag_value(args, "--pack").ok_or("this command requires --pack <dir>")?;
            verify_pack_at(dir)?;
            let matrix = scenario_matrix().map_err(|e| e.to_string())?;
            emit(&matrix, flag_value(args, "--out"))
        }
        Some("scenario-matrix-report") => {
            // Read the provided matrix, verify it IS the canonical matrix, then render the report from
            // the re-derived canonical matrix (a tampered matrix is refused, never rendered).
            let path =
                flag_value(args, "--matrix").ok_or("this command requires --matrix <path>")?;
            let content =
                std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
            let report = scenario_matrix_report(&content).map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("scenario-matrix-verify") => {
            // Verify BOTH the pack (re-derive every scenario bundle + manifest) AND the matrix (re-derive
            // and byte-compare) — a tampered pack OR a tampered matrix is refused.
            let dir = flag_value(args, "--pack").ok_or("this command requires --pack <dir>")?;
            verify_pack_at(dir)?;
            let path =
                flag_value(args, "--matrix").ok_or("this command requires --matrix <path>")?;
            let matrix =
                std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
            verify_scenario_matrix(&matrix).map_err(|e| e.to_string())?;
            print!("{}", scenario_matrix_verify_summary());
            Ok(())
        }
        Some("lit-intent-demo") => {
            // Emit the canonical LIT-INTENT-0 demo map: verified QFLOW spans reshaped into
            // central thesis / bounded author intent / claims / terms / teaching path / refusals.
            // The artifact is pure and re-derivable; it creates no evidence or authority.
            let json = literature_intent_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("lit-intent-demo-verify") => {
            // Re-derive the canonical demo map and require the provided --map bytes to match exactly.
            // A tampered or stale map is refused rather than rendered or trusted.
            let map = read_plain_file(args, "--map")?;
            verify_literature_intent_demo_json(&map).map_err(|e| format!("{e:?}"))?;
            println!("lit-intent-demo-verify: OK");
            Ok(())
        }
        Some("lit-intent-matrix") => {
            // Emit the LIT-INTENT-0 scenario matrix: built/refused outcomes and boundary coverage.
            let json = literature_intent_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("lit-intent-matrix-verify") => {
            // Re-derive the LIT-INTENT-0 matrix and byte-compare a provided --matrix artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_literature_intent_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("lit-intent-matrix-verify: OK");
            Ok(())
        }
        Some("teach-map-demo") => {
            // Emit the canonical TEACH-0 lesson: a bounded user-facing lesson derived only
            // from the canonical LIT-INTENT-0 map. No model, no memory, no personalization.
            let json = teach_map_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("teach-map-demo-verify") => {
            // Re-derive the canonical TEACH-0 lesson and require provided bytes to match.
            let lesson = read_plain_file(args, "--lesson")?;
            verify_teach_map_demo_json(&lesson).map_err(|e| format!("{e:?}"))?;
            println!("teach-map-demo-verify: OK");
            Ok(())
        }
        Some("teach-map-matrix") => {
            // Emit the TEACH-0 scenario matrix: supported lesson parts, refusals, and closed gates.
            let json = teach_map_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("teach-map-matrix-verify") => {
            // Re-derive the TEACH-0 matrix and byte-compare a provided --matrix artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_teach_map_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("teach-map-matrix-verify: OK");
            Ok(())
        }
        Some("learner-model-demo") => {
            // Emit the canonical LEARNER-MODEL-0 state map: a receipt-linked observation
            // over a supported TEACH-0 lesson. No memory write, adaptation, diagnosis, or model.
            let json = learner_model_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learner-model-demo-verify") => {
            // Re-derive the canonical learner-state map and require provided bytes to match.
            let state = read_plain_file(args, "--state")?;
            verify_learner_model_demo_json(&state).map_err(|e| format!("{e:?}"))?;
            println!("learner-model-demo-verify: OK");
            Ok(())
        }
        Some("learner-model-matrix") => {
            // Emit the LEARNER-MODEL-0 scenario matrix: observation mapping plus closed gates.
            let json = learner_model_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learner-model-matrix-verify") => {
            // Re-derive the LEARNER-MODEL-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_learner_model_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("learner-model-matrix-verify: OK");
            Ok(())
        }
        Some("learner-memory-demo") => {
            // Emit the canonical LEARNER-MEMORY-0 candidate: bounded memory items,
            // every one pointing back to learner-state fields and source receipt hashes.
            let json = learner_memory_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learner-memory-demo-verify") => {
            // Re-derive the canonical memory candidate and require provided bytes to match.
            let memory = read_plain_file(args, "--memory")?;
            verify_learner_memory_demo_json(&memory).map_err(|e| format!("{e:?}"))?;
            println!("learner-memory-demo-verify: OK");
            Ok(())
        }
        Some("learner-memory-matrix") => {
            // Emit the LEARNER-MEMORY-0 scenario matrix: candidate mapping plus closed gates.
            let json = learner_memory_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learner-memory-matrix-verify") => {
            // Re-derive the LEARNER-MEMORY-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_learner_memory_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("learner-memory-matrix-verify: OK");
            Ok(())
        }
        Some("learner-journal-demo") => {
            // Emit the canonical LEARNER-MEMORY-1 journal run: two consented
            // append-only pointer entries, hash-linked from the genesis head.
            let json = learner_journal_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learner-journal-demo-verify") => {
            // Re-derive the canonical journal run and require provided bytes to match.
            let journal = read_plain_file(args, "--journal")?;
            verify_learner_journal_demo_json(&journal).map_err(|e| format!("{e:?}"))?;
            println!("learner-journal-demo-verify: OK");
            Ok(())
        }
        Some("learner-journal-matrix") => {
            // Emit the LEARNER-MEMORY-1 scenario matrix: one clean append plus closed gates.
            let json = learner_journal_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learner-journal-matrix-verify") => {
            // Re-derive the LEARNER-MEMORY-1 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_learner_journal_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("learner-journal-matrix-verify: OK");
            Ok(())
        }
        Some("learner-journal-append") => {
            // Consented live append. The journal file is UNTRUSTED input: it must
            // byte-match a canonical verified state (never parsed), and the consent
            // flags must re-affirm the canonical scope-bound consent. The library
            // stays pure — every read and write happens here in the I/O shell.
            let path = flag_value(args, "--journal")
                .ok_or_else(|| "learner-journal-append requires --journal PATH".to_string())?;
            let operator = flag_value(args, "--consent-operator")
                .ok_or_else(|| "learner-journal-append requires --consent-operator".to_string())?;
            let scope = flag_value(args, "--consent-scope")
                .ok_or_else(|| "learner-journal-append requires --consent-scope".to_string())?;
            let existing = if std::path::Path::new(path).exists() {
                Some(
                    std::fs::read_to_string(path)
                        .map_err(|e| format!("cannot read {path}: {e}"))?,
                )
            } else {
                None
            };
            let state = match existing {
                None => 0,
                Some(bytes) => (0..=LEARNER_JOURNAL_DEMO_CANDIDATES)
                    .find(|n| learner_journal_json_at(*n).as_deref() == Some(bytes.as_str()))
                    .ok_or_else(|| {
                        "learner-journal-append: refused (journal does not byte-match any \
                         verified canonical state: ReplayMismatch)"
                            .to_string()
                    })?,
            };
            let consent = LearnerJournalConsent {
                operator: operator.to_string(),
                journal_scope: scope.to_string(),
                consents_to_append: true,
            };
            let run = learner_journal_append_at(state, &consent);
            match run.journal {
                Some(journal) => {
                    std::fs::write(path, learner_journal_state_json(&journal))
                        .map_err(|e| format!("cannot write {path}: {e}"))?;
                    println!(
                        "learner-journal-append: OK entries={} head={:016x}",
                        journal.entry_count, journal.head_hash
                    );
                    Ok(())
                }
                None => Err(format!(
                    "learner-journal-append: refused ({})",
                    run.refusal.map(|r| r.slug()).unwrap_or("unknown")
                )),
            }
        }
        Some("converse-demo") => {
            // Emit the canonical CONVERSE-0 transcript: a multi-turn conversation over a
            // baked vault + script where every answering turn is a QFLOW verified evidence
            // packet and an ungroundable turn is an honest typed refusal.
            let json = converse_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("converse-demo-verify") => {
            // Re-derive the canonical transcript and require provided bytes to match.
            let transcript = read_plain_file(args, "--transcript")?;
            verify_converse_demo_json(&transcript).map_err(|e| format!("{e:?}"))?;
            println!("converse-demo-verify: OK");
            Ok(())
        }
        Some("converse-matrix") => {
            // Emit the CONVERSE-0 scenario matrix: one clean conversation plus every refusal.
            let json = converse_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("converse-matrix-verify") => {
            // Re-derive the CONVERSE-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_converse_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("converse-matrix-verify: OK");
            Ok(())
        }
        Some("converse-run") => {
            // The real surface: read a LOCAL `.txt` vault (confined via read_local_corpus)
            // plus an operator script file (confined via read_local_file), strict-parse the
            // script, run the pure engine, and emit the transcript. No Deserialize — a
            // malformed script becomes a refused transcript (ScriptParseRefused). Each line
            // of the script is `SCOPE<TAB>question` (SCOPE in whole_vault | prior_answer |
            // conversation_so_far). The library stays pure; all file I/O is here in the shell.
            let vault = read_local_corpus(args)?;
            let script = read_local_file(args, "--script")?;
            let transcript =
                converse_run_from_text(&script, &vault, ConverseConfig::default_config());
            emit(
                &converse_transcript_json(&transcript),
                flag_value(args, "--out"),
            )
        }
        Some("converse-run-verify") => {
            // Re-run the engine over the SAME vault + script and byte-compare the supplied
            // transcript (untrusted input; re-derived + byte-verified, never parsed).
            let vault = read_local_corpus(args)?;
            let script = read_local_file(args, "--script")?;
            let supplied = read_plain_file(args, "--transcript")?;
            let transcript =
                converse_run_from_text(&script, &vault, ConverseConfig::default_config());
            if converse_transcript_json(&transcript) == supplied {
                println!("converse-run-verify: OK");
                Ok(())
            } else {
                Err(
                    "converse-run-verify: refused (transcript does not byte-match the \
                     re-derived canonical: ReplayMismatch)"
                        .to_string(),
                )
            }
        }
        Some("learning-session-demo") => {
            // Emit the canonical SESSION-LOOP-0 run: the full six-stage spine
            // (evidence -> intent -> lesson -> learner state -> memory candidate
            // -> consented journal append) as one receipt-linked artifact.
            let json = learning_session_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learning-session-demo-verify") => {
            // Re-derive the canonical session run and require provided bytes to match.
            let session = read_plain_file(args, "--session")?;
            verify_learning_session_demo_json(&session).map_err(|e| format!("{e:?}"))?;
            println!("learning-session-demo-verify: OK");
            Ok(())
        }
        Some("learning-session-matrix") => {
            // Emit the SESSION-LOOP-0 scenario matrix: three completions plus closed gates.
            let json = learning_session_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learning-session-matrix-verify") => {
            // Re-derive the SESSION-LOOP-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_learning_session_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("learning-session-matrix-verify: OK");
            Ok(())
        }
        Some("learning-arc-demo") => {
            // Emit the canonical MULTI-SESSION-0 arc: two consented sessions with
            // verified journal-head continuity (genesis -> head 1 -> head 2).
            let json = learning_arc_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learning-arc-demo-verify") => {
            // Re-derive the canonical arc and require provided bytes to match.
            let arc = read_plain_file(args, "--arc")?;
            verify_learning_arc_demo_json(&arc).map_err(|e| format!("{e:?}"))?;
            println!("learning-arc-demo-verify: OK");
            Ok(())
        }
        Some("learning-arc-matrix") => {
            // Emit the MULTI-SESSION-0 scenario matrix: two completions plus closed gates.
            let json = learning_arc_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("learning-arc-matrix-verify") => {
            // Re-derive the MULTI-SESSION-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_learning_arc_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("learning-arc-matrix-verify: OK");
            Ok(())
        }
        Some("game-evidence-demo") => {
            // Emit the canonical GAME-EVIDENCE-0 run: eleven fixture observations
            // converted into strict, verbatim-preserving evidence documents.
            let json = game_evidence_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("game-evidence-demo-verify") => {
            // Re-derive the canonical packet and require provided bytes to match.
            let packet = read_plain_file(args, "--packet")?;
            verify_game_evidence_demo_json(&packet).map_err(|e| format!("{e:?}"))?;
            println!("game-evidence-demo-verify: OK");
            Ok(())
        }
        Some("game-evidence-matrix") => {
            // Emit the GAME-EVIDENCE-0 scenario matrix: eleven built kinds plus closed gates.
            let json = game_evidence_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("game-evidence-matrix-verify") => {
            // Re-derive the GAME-EVIDENCE-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_game_evidence_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("game-evidence-matrix-verify: OK");
            Ok(())
        }
        Some("wow-state-demo") => {
            // Emit the canonical WOW-STATE-0 navigation snapshot: the Durotar
            // starter fixture with a chosen nav target and stuck/progress signals.
            let json = wow_state_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("wow-state-demo-verify") => {
            // Re-derive the canonical snapshot and require provided bytes to match.
            let snapshot = read_plain_file(args, "--snapshot")?;
            verify_wow_state_demo_json(&snapshot).map_err(|e| format!("{e:?}"))?;
            println!("wow-state-demo-verify: OK");
            Ok(())
        }
        Some("wow-state-matrix") => {
            // Emit the WOW-STATE-0 scenario matrix: prepared navigation cells plus
            // every refusal and closed gate.
            let json = wow_state_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("wow-state-matrix-verify") => {
            // Re-derive the WOW-STATE-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_wow_state_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("wow-state-matrix-verify: OK");
            Ok(())
        }
        Some("wow-taskplan-demo") => {
            // Emit the canonical WOW-TASKPLAN-0 bounded task-plan proposal.
            let json = wow_taskplan_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("wow-taskplan-demo-verify") => {
            // Re-derive the canonical plan and require provided bytes to match.
            let plan = read_plain_file(args, "--plan")?;
            verify_wow_taskplan_demo_json(&plan).map_err(|e| format!("{e:?}"))?;
            println!("wow-taskplan-demo-verify: OK");
            Ok(())
        }
        Some("wow-taskplan-matrix") => {
            // Emit the WOW-TASKPLAN-0 scenario matrix.
            let json = wow_taskplan_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("wow-taskplan-matrix-verify") => {
            // Re-derive the WOW-TASKPLAN-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_wow_taskplan_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("wow-taskplan-matrix-verify: OK");
            Ok(())
        }
        Some("controller-bridge-demo") => {
            // Emit the canonical CONTROLLER-BRIDGE-0 dry-run command envelope set.
            let json = controller_bridge_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("controller-bridge-demo-verify") => {
            // Re-derive the canonical envelope set and require provided bytes to match.
            let plan = read_plain_file(args, "--envelope")?;
            verify_controller_bridge_demo_json(&plan).map_err(|e| format!("{e:?}"))?;
            println!("controller-bridge-demo-verify: OK");
            Ok(())
        }
        Some("controller-bridge-matrix") => {
            // Emit the CONTROLLER-BRIDGE-0 scenario matrix.
            let json = controller_bridge_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("controller-bridge-matrix-verify") => {
            // Re-derive the CONTROLLER-BRIDGE-0 matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_controller_bridge_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("controller-bridge-matrix-verify: OK");
            Ok(())
        }
        Some("live-actuator-producer-demo") => {
            // Emit the canonical LIVE-ACTUATOR-BRIDGE-0 producer artifact (a DRY-RUN
            // envelope artifact wrapped with emission_seq + ledger-entry discipline). No fs.
            let json = producer_demo_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("live-actuator-producer-demo-verify") => {
            // Re-derive the canonical producer artifact and require provided bytes to match.
            let artifact = read_plain_file(args, "--artifact")?;
            verify_producer_demo_json(&artifact).map_err(|e| format!("{e:?}"))?;
            println!("live-actuator-producer-demo-verify: OK");
            Ok(())
        }
        Some("live-actuator-producer-matrix") => {
            // Emit the LIVE-ACTUATOR-BRIDGE-0 producer scenario matrix. No fs.
            let json = producer_matrix_json();
            emit(&json, flag_value(args, "--out"))
        }
        Some("live-actuator-producer-matrix-verify") => {
            // Re-derive the producer matrix and byte-compare a provided artifact.
            let matrix = read_plain_file(args, "--matrix")?;
            verify_producer_matrix_json(&matrix).map_err(|e| format!("{e:?}"))?;
            println!("live-actuator-producer-matrix-verify: OK");
            Ok(())
        }
        Some("live-actuator-producer-write") => {
            // THE shell verb: append the canonical DRY-RUN envelope artifact to a quarantined
            // outbox (temp+rename) and record it in a durable, tamper-evident local ledger
            // (temp+rename). Writes files only — it does NOT execute, move, or call anything.
            let outbox = flag_value(args, "--outbox");
            let ledger = flag_value(args, "--ledger");
            let summary = run_producer_write(outbox, ledger)?;
            print!("{summary}");
            Ok(())
        }
        Some("failure-cases") => {
            // List the finite negative-scenario set (no inputs needed — this is the menu).
            print!("{}", list_failure_cases());
            Ok(())
        }
        Some("failure-pack") => {
            // Run every forgery through the existing verifiers and write the rejection record + report
            // (pure derivation). Nothing forged is persisted as trusted state — only the rejections.
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let files = failure_pack_files().map_err(|e| e.to_string())?;
            write_failure_pack(out_dir, &files)?;
            print!("{}", failure_pack_summary(out_dir, &files));
            Ok(())
        }
        Some("failure-verify") => {
            // Read the provided failure pack, then verify it by RE-DERIVING the canonical pack (re-running
            // every forgery) and byte-comparing every file — a doctored pack is refused, never trusted.
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_failure_pack(dir)?;
            verify_failure_pack(&provided).map_err(|e| e.to_string())?;
            print!("{}", failure_verify_summary());
            Ok(())
        }
        Some("doc-trace") => {
            // Read a LOCAL operator document (path-validated), then build the SAME end-to-end trace from
            // a FROZEN-VERIFIED reading receipt over the document. The document is read, never trusted —
            // doc_trace fails closed if the read does not verify.
            let doc = read_local_input(args)?;
            let json = run_doc_trace(&doc).map_err(|e| e.to_string())?;
            emit(&json, flag_value(args, "--out"))
        }
        Some("doc-report") => {
            // Re-derive the document trace from the SAME --input, confirm the provided --trace IS that
            // trace (refuse a tampered/foreign trace), then render the operator report. The document is
            // the source of truth, so this command requires --input as well as --trace.
            let doc = read_local_input(args)?;
            let trace = read_trace(args)?;
            let report = run_doc_report(&doc, &trace).map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("doc-bundle") => {
            // Derive the repro bundle purely from the document's verified trace and write every file with
            // exact bytes into --out (the only side effect lives in this shell).
            let doc = read_local_input(args)?;
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let files = doc_bundle(&doc).map_err(|e| e.to_string())?;
            write_bundle(out_dir, &files)?;
            print!("{}", doc_bundle_summary(out_dir, &files));
            Ok(())
        }
        Some("doc-bundle-verify") => {
            // Read the provided pack AND the SAME --input document, then verify by RE-DERIVING the bundle
            // from the document and byte-comparing every file. A tampered document OR a tampered/missing
            // bundle file is refused; nothing on disk is trusted.
            let doc = read_local_input(args)?;
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_bundle(dir)?;
            verify_doc_bundle(&doc, &provided).map_err(|e| e.to_string())?;
            print!("{}", doc_bundle_verify_summary());
            Ok(())
        }
        Some("doc-scenarios") => {
            // List the finite document-flow input-scenario set (no inputs needed — this is the menu).
            print!("{}", list_doc_scenarios());
            Ok(())
        }
        Some("doc-scenario-pack") => {
            // Run every document scenario (clean + invalid inputs) and write the observed-outcome record +
            // report (pure derivation). No scenario executes, promotes, or trains; the forged/invalid
            // inputs exist only to be refused.
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let files = doc_scenario_pack_files().map_err(|e| e.to_string())?;
            write_bundle(out_dir, &files)?;
            print!("{}", doc_scenario_pack_summary(out_dir, &files));
            Ok(())
        }
        Some("doc-scenario-verify") => {
            // Read the provided document-scenario pack, then verify it by RE-DERIVING the pack (re-running
            // every scenario) and byte-comparing — a doctored pack is refused, never trusted.
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_doc_scenario_pack(dir)?;
            verify_doc_scenario_pack(&provided).map_err(|e| e.to_string())?;
            print!("{}", doc_scenario_verify_summary());
            Ok(())
        }
        Some("doc-scenario-matrix") => {
            // VERIFY the pack at --path re-derives (refuse a tampered pack), then emit the input-integrity
            // matrix to --out. The matrix is purely re-derived from the scenario set; the pack is never trusted.
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_doc_scenario_pack(dir)?;
            verify_doc_scenario_pack(&provided).map_err(|e| e.to_string())?;
            let matrix = doc_scenario_matrix().map_err(|e| e.to_string())?;
            emit(&matrix, flag_value(args, "--out"))
        }
        Some("corpus-trace") => {
            // Read a LOCAL operator corpus DIRECTORY (path-validated; only non-hidden `.txt` files, each
            // canonicalize-contained), then build the SAME end-to-end trace from a FROZEN-VERIFIED reading
            // receipt over the corpus. The corpus is read, never trusted — corpus_trace fails closed if the
            // read does not verify (or the corpus grounds nothing).
            let documents = read_local_corpus(args)?;
            let json = run_corpus_trace(&documents).map_err(|e| e.to_string())?;
            emit(&json, flag_value(args, "--out"))
        }
        Some("corpus-report") => {
            // Re-derive the corpus trace from the SAME --input-dir, confirm the provided --trace IS that
            // trace (refuse a tampered/foreign trace), then render the operator report with the SOURCE
            // SELECTION section. The corpus is the source of truth, so this command requires --input-dir.
            let documents = read_local_corpus(args)?;
            let trace = read_trace(args)?;
            let report = run_corpus_report(&documents, &trace).map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("corpus-bundle") => {
            // Derive the repro bundle purely from the corpus's verified trace and write every file with exact
            // bytes into --out (the only side effect lives in this shell).
            let documents = read_local_corpus(args)?;
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let files = corpus_bundle(&documents).map_err(|e| e.to_string())?;
            write_bundle(out_dir, &files)?;
            print!("{}", corpus_bundle_summary(out_dir, &files));
            Ok(())
        }
        Some("corpus-bundle-verify") => {
            // Read the provided pack AND the SAME --input-dir corpus, then verify by RE-DERIVING the bundle
            // from the corpus and byte-comparing every file. A tampered corpus (any document) OR a tampered/
            // missing bundle file is refused; nothing on disk is trusted.
            let documents = read_local_corpus(args)?;
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_corpus_bundle(dir)?;
            verify_corpus_bundle(&documents, &provided).map_err(|e| e.to_string())?;
            print!("{}", corpus_bundle_verify_summary());
            Ok(())
        }
        Some("corpus-scenarios") => {
            // List the finite corpus-flow input-scenario set (no inputs needed — this is the menu).
            print!("{}", list_corpus_scenarios());
            Ok(())
        }
        Some("corpus-scenario-pack") => {
            // Run every corpus scenario (clean + invalid inputs) and write the observed-outcome record +
            // report (pure derivation). No scenario executes, promotes, or trains; the forged/invalid inputs
            // exist only to be refused.
            let out_dir = flag_value(args, "--out").ok_or("this command requires --out <dir>")?;
            let files = corpus_scenario_pack_files().map_err(|e| e.to_string())?;
            write_bundle(out_dir, &files)?;
            print!("{}", corpus_scenario_pack_summary(out_dir, &files));
            Ok(())
        }
        Some("corpus-scenario-verify") => {
            // Read the provided corpus-scenario pack, then verify it by RE-DERIVING the pack (re-running every
            // scenario) and byte-comparing — a doctored pack is refused, never trusted.
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_corpus_scenario_pack(dir)?;
            verify_corpus_scenario_pack(&provided).map_err(|e| e.to_string())?;
            print!("{}", corpus_scenario_verify_summary());
            Ok(())
        }
        Some("corpus-scenario-matrix") => {
            // VERIFY the pack at --path re-derives (refuse a tampered pack), then emit the input-integrity
            // matrix to --out. The matrix is purely re-derived from the scenario set; the pack is never trusted.
            let dir = flag_value(args, "--path").ok_or("this command requires --path <dir>")?;
            let provided = read_corpus_scenario_pack(dir)?;
            verify_corpus_scenario_pack(&provided).map_err(|e| e.to_string())?;
            let matrix = corpus_scenario_matrix().map_err(|e| e.to_string())?;
            emit(&matrix, flag_value(args, "--out"))
        }
        Some("novelty-packet") => {
            // Re-derive the verified corpus trace from --input-dir, confirm the provided --corpus-trace IS that
            // trace (refuse a tampered / receipt-hash-stripped / foreign trace), then derive a HYPOTHESIS-ONLY
            // novelty packet from that verified corpus and the operator --frame. The corpus is the source of
            // truth, so this command requires --input-dir alongside --corpus-trace. The frame is read but never
            // trusted as fact; nothing executes, becomes evidence, promotes, or trains.
            let documents = read_local_corpus(args)?;
            let trace = read_plain_file(args, "--corpus-trace")?;
            let frame = read_frame(args)?;
            let json = run_novelty_packet(&documents, &trace, &frame).map_err(|e| e.to_string())?;
            emit(&json, flag_value(args, "--out"))
        }
        Some("novelty-report") => {
            // Re-derive the novelty packet from the SAME --input-dir corpus and --frame, confirm the provided
            // --packet IS that packet (refuse a tampered packet), then render the proposal report. The corpus +
            // frame are the source of truth, so this command requires --input-dir + --frame alongside --packet.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let packet = read_packet(args)?;
            let report =
                run_novelty_report(&documents, &frame, &packet).map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("novelty-replay") => {
            // Re-derive the novelty packet from the corpus + frame and confirm the provided --packet is
            // byte-identical — a determinism proof that also refuses any tampered packet. Reads nothing as
            // authority; the packet PROPOSES, it does not prove.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let packet = read_packet(args)?;
            let summary =
                run_novelty_replay(&documents, &frame, &packet).map_err(|e| e.to_string())?;
            print!("{summary}");
            Ok(())
        }
        Some("dream-export") => {
            // Re-derive the terminal dream packet from the --input-dir corpus + --frame + dials and BRIDGE it
            // into the EXISTING hypothesis-only proposal path, emitting an export bundle (a DreamExportReceipt +
            // the proposed HypothesisPacket). If --dream-packet is given, it is REFUSED unless it is byte-for-byte
            // the re-derived packet. No new authority is created; the exported material is hypothesis_only.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let seed = flag_u64(args, "--seed", DREAM_DEFAULT_SEED)?;
            let weirdness = flag_i64(args, "--weirdness", DREAM_DEFAULT_WEIRDNESS)?;
            let provided = optional_plain_file(args, "--dream-packet")?;
            let json = run_dream_export(&documents, &frame, seed, weirdness, provided.as_deref())
                .map_err(|e| e.to_string())?;
            emit(&json, flag_value(args, "--out"))
        }
        Some("dream-export-report") => {
            // Re-derive the export bundle from the SAME corpus + frame + dials, confirm the provided --export IS
            // that bundle (refuse a tampered bundle), then render the provenance report. The corpus + frame are
            // the source of truth, so this command requires --input-dir + --frame alongside --export.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let seed = flag_u64(args, "--seed", DREAM_DEFAULT_SEED)?;
            let weirdness = flag_i64(args, "--weirdness", DREAM_DEFAULT_WEIRDNESS)?;
            let bundle = read_plain_file(args, "--export")?;
            let report = run_dream_export_report(&documents, &frame, seed, weirdness, &bundle)
                .map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("dream-export-replay") => {
            // Re-derive the export bundle from the corpus + frame + dials and confirm the provided --export is
            // byte-identical — a determinism proof that also refuses any tampered bundle. Reads nothing as
            // authority; the export PROPOSES via the existing gate, it does not prove.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let seed = flag_u64(args, "--seed", DREAM_DEFAULT_SEED)?;
            let weirdness = flag_i64(args, "--weirdness", DREAM_DEFAULT_WEIRDNESS)?;
            let bundle = read_plain_file(args, "--export")?;
            let summary = run_dream_export_replay(&documents, &frame, seed, weirdness, &bundle)
                .map_err(|e| e.to_string())?;
            print!("{summary}");
            Ok(())
        }
        Some("dream-export-scenarios") => {
            // List the finite dream-export scenario set (one clean export verifies; every tamper is refused). Pure.
            print!("{}", list_dream_export_scenarios());
            Ok(())
        }
        Some("dream-export-matrix") => {
            // Emit the deterministic dream-export scenario matrix: the clean export (verifies) + every tamper
            // (refused), the preserved dream provenance, the coverage cells, and the boundary. Re-derived from the
            // corpus + frame + dials, so it replays byte-identically.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let seed = flag_u64(args, "--seed", DREAM_DEFAULT_SEED)?;
            let weirdness = flag_i64(args, "--weirdness", DREAM_DEFAULT_WEIRDNESS)?;
            let json = dream_export_matrix(&documents, &frame, seed, weirdness)
                .map_err(|e| e.to_string())?;
            emit(&json, flag_value(args, "--out"))
        }
        Some("dream-export-matrix-report") => {
            // Re-derive the matrix from the SAME corpus + frame + dials, confirm the provided --matrix IS that
            // matrix (refuse a tampered matrix), then render the scenario report. The corpus + frame are the source
            // of truth, so this command requires --input-dir + --frame alongside --matrix.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let seed = flag_u64(args, "--seed", DREAM_DEFAULT_SEED)?;
            let weirdness = flag_i64(args, "--weirdness", DREAM_DEFAULT_WEIRDNESS)?;
            let matrix = read_plain_file(args, "--matrix")?;
            let report =
                run_dream_export_matrix_report(&documents, &frame, seed, weirdness, &matrix)
                    .map_err(|e| e.to_string())?;
            emit(&report, flag_value(args, "--out"))
        }
        Some("dream-export-matrix-verify") => {
            // Re-derive the matrix from the corpus + frame + dials and confirm the provided --matrix is
            // byte-identical — a determinism proof that also refuses any tampered matrix. Reads nothing as authority.
            let documents = read_local_corpus(args)?;
            let frame = read_frame(args)?;
            let seed = flag_u64(args, "--seed", DREAM_DEFAULT_SEED)?;
            let weirdness = flag_i64(args, "--weirdness", DREAM_DEFAULT_WEIRDNESS)?;
            let matrix = read_plain_file(args, "--matrix")?;
            let summary =
                run_dream_export_matrix_verify(&documents, &frame, seed, weirdness, &matrix)
                    .map_err(|e| e.to_string())?;
            print!("{summary}");
            Ok(())
        }
        _ => Err(usage()),
    }
}

/// The canonical dream seed/weirdness, used when `--seed`/`--weirdness` are omitted so the demo runs from just a
/// corpus + frame. They are bounded by dream-engine (weirdness must be `0..=5`), which refuses out-of-range dials.
const DREAM_DEFAULT_SEED: u64 = 42;
const DREAM_DEFAULT_WEIRDNESS: i64 = 2;

/// Read the unsigned-integer value of `flag` (e.g. `--seed`), or `default` if it is absent. Fails closed with a
/// clear error on a non-numeric value rather than silently coercing it.
fn flag_u64(args: &[String], flag: &str, default: u64) -> Result<u64, String> {
    match flag_value(args, flag) {
        Some(v) => v
            .parse::<u64>()
            .map_err(|_| format!("{flag} must be a non-negative integer, got '{v}'")),
        None => Ok(default),
    }
}

/// Read the signed-integer value of `flag` (e.g. `--weirdness`), or `default` if it is absent. Fails closed with
/// a clear error on a non-numeric value; range validation (the `0..=5` weirdness dial) is dream-engine's.
fn flag_i64(args: &[String], flag: &str, default: i64) -> Result<i64, String> {
    match flag_value(args, flag) {
        Some(v) => v
            .parse::<i64>()
            .map_err(|_| format!("{flag} must be an integer, got '{v}'")),
        None => Ok(default),
    }
}

/// Read the OPTIONAL file named by `flag` as a plain string, returning `None` if the flag is absent. The CONTENT
/// is never trusted as authority — it is only compared against a re-derived canonical artifact by the library.
fn optional_plain_file(args: &[String], flag: &str) -> Result<Option<String>, String> {
    match flag_value(args, flag) {
        Some(path) => std::fs::read_to_string(path)
            .map(Some)
            .map_err(|e| format!("cannot read {path}: {e}")),
        None => Ok(None),
    }
}

/// Read the file named by `--trace PATH`. The CONTENT is never trusted as authority — it is only
/// compared against the re-derived canonical trace by the library — so this is a plain file read.
fn read_trace(args: &[String]) -> Result<String, String> {
    read_plain_file(args, "--trace")
}

/// Read the file named by `flag` as a plain string. The CONTENT is never trusted as authority — it is only
/// compared against a re-derived canonical artifact by the library — so this is a plain file read. Shared by
/// `--trace`, `--corpus-trace`, and `--packet`.
fn read_plain_file(args: &[String], flag: &str) -> Result<String, String> {
    let path =
        flag_value(args, flag).ok_or_else(|| format!("this command requires {flag} <path>"))?;
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))
}

/// Read the LOCAL operator document named by `--input PATH`, validating that the path is safe and local
/// before touching it. Two layers: (1) the pure [`check_local_input_path`] rejects an absolute path, a
/// `..` traversal, a `~` prefix, or an empty path WITHOUT any filesystem access; (2) defense in depth —
/// canonicalize the path and the working directory and require the resolved path to stay INSIDE the
/// working directory (so a symlink cannot escape to a non-local file), and require a regular file. Only
/// then is the document read. The bytes are passed to the library as untrusted CONTENT — the library
/// verifies them through the frozen reader before tracing. The filesystem access lives ONLY here, in the
/// shell.
fn read_local_input(args: &[String]) -> Result<String, String> {
    read_local_file(args, "--input")
}

/// Read a LOCAL operator file named by `flag` (e.g. `--input` or `--frame`), validating that the path is safe
/// and local before touching it — the shared core of [`read_local_input`] and [`read_frame`]. Two layers: (1)
/// the pure [`check_local_input_path`] rejects an absolute path, a `..` traversal, a `~` prefix, or an empty
/// path WITHOUT any filesystem access; (2) defense in depth — canonicalize the path and the working directory
/// and require the resolved path to stay INSIDE the working directory (so a symlink cannot escape to a
/// non-local file), and require a regular file. Only then is the file read. The bytes are passed to the
/// library as untrusted CONTENT. The filesystem access lives ONLY here, in the shell.
fn read_local_file(args: &[String], flag: &str) -> Result<String, String> {
    let path =
        flag_value(args, flag).ok_or_else(|| format!("this command requires {flag} <path>"))?;
    check_local_input_path(path).map_err(|e| e.to_string())?;
    let cwd = std::env::current_dir()
        .and_then(|d| d.canonicalize())
        .map_err(|e| format!("cannot resolve the working directory: {e}"))?;
    let resolved = std::fs::canonicalize(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    if !resolved_path_within(&cwd, &resolved) {
        return Err(format!(
            "refusing unsafe input path '{path}' — it escapes the working directory"
        ));
    }
    if !resolved.is_file() {
        return Err(format!(
            "refusing input path '{path}' — it is not a regular local file"
        ));
    }
    std::fs::read_to_string(&resolved).map_err(|e| format!("cannot read {path}: {e}"))
}

/// Read the LOCAL operator FRAME named by `--frame PATH` (validated + confined; see [`read_local_file`]). The
/// frame is untrusted DATA — recorded into the packet as `frame_text` and structured into candidate
/// assumptions to break — but it is NEVER grounded as a fact; only verified corpus spans are preserved facts.
fn read_frame(args: &[String]) -> Result<String, String> {
    read_local_file(args, "--frame")
}

/// Read the novelty packet named by `--packet PATH` (a plain read via [`read_plain_file`]; the content is never
/// trusted as authority — it is only compared against the re-derived canonical packet by the library).
fn read_packet(args: &[String]) -> Result<String, String> {
    read_plain_file(args, "--packet")
}

/// Read the LOCAL operator corpus named by `--input-dir PATH`: a directory of `.txt` documents, validated
/// and confined before any read. Same two layers as [`read_local_input`], applied to the directory and to
/// every entry: (1) the pure [`check_local_input_path`] rejects an absolute path, a `..` traversal, a `~`
/// prefix, or an empty path WITHOUT filesystem access; (2) the resolved directory must stay INSIDE the
/// canonicalized working directory. Then each entry is admitted ONLY if its file name is a non-hidden `.txt`
/// file ([`corpus_admits_filename`] — hidden and non-`.txt` files are refused), its canonical path stays
/// INSIDE the corpus directory (so a symlink cannot escape), and it is a regular file. Documents are sorted
/// by name so span ids — and the whole trace — are deterministic. The bytes are passed to the library as
/// untrusted CONTENT; the library verifies them through the frozen reader and fails closed on an empty
/// corpus. The filesystem access lives ONLY here, in the shell.
fn read_local_corpus(args: &[String]) -> Result<Vec<(String, String)>, String> {
    let dir = flag_value(args, "--input-dir").ok_or("this command requires --input-dir <dir>")?;
    check_local_input_path(dir).map_err(|e| e.to_string())?;
    let cwd = std::env::current_dir()
        .and_then(|d| d.canonicalize())
        .map_err(|e| format!("cannot resolve the working directory: {e}"))?;
    let root = std::fs::canonicalize(dir).map_err(|e| format!("cannot read {dir}: {e}"))?;
    if !resolved_path_within(&cwd, &root) {
        return Err(format!(
            "refusing unsafe corpus path '{dir}' — it escapes the working directory"
        ));
    }
    if !root.is_dir() {
        return Err(format!(
            "refusing corpus path '{dir}' — it is not a local directory"
        ));
    }
    let mut documents: Vec<(String, String)> = Vec::new();
    let entries = std::fs::read_dir(&root).map_err(|e| format!("cannot read {dir}: {e}"))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("cannot read {dir}: {e}"))?.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        // Only non-hidden `.txt` files are admitted; hidden / non-`.txt` files are refused (never read).
        if !corpus_admits_filename(&name) {
            continue;
        }
        // Defense in depth: the entry must canonicalize to a REGULAR FILE inside the corpus root, so a
        // symlink cannot escape the directory.
        let resolved = match std::fs::canonicalize(&path) {
            Ok(resolved) => resolved,
            Err(_) => continue,
        };
        if !resolved_path_within(&root, &resolved) || !resolved.is_file() {
            continue;
        }
        let content = std::fs::read_to_string(&resolved)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        documents.push((name, content));
    }
    documents.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(documents)
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

/// The human summary printed after `doc-bundle` writes the pack: the files written and the DOCFLOW-0
/// boundary (the document flow reads local input but does not trust it, and acts on nothing).
fn doc_bundle_summary(dir: &str, files: &[(&str, String)]) -> String {
    let mut out = format!("doc-bundle: wrote {} files to {dir}\n", files.len());
    for (name, content) in files {
        out.push_str(&format!("    {name} ({} bytes)\n", content.len()));
    }
    out.push_str("BOUNDARY\n");
    for line in DOC_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// Read the expected document-scenario pack files from `dir`. A file that is absent is simply omitted (so
/// `verify_doc_scenario_pack` reports it missing rather than this shell guessing). The CONTENT is never
/// trusted — it is only re-derived and byte-compared by the library.
fn read_doc_scenario_pack(dir: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for name in DOC_SCENARIO_PACK_FILES {
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
            found.push((name.to_string(), content));
        }
    }
    Ok(found)
}

/// The human summary printed after `doc-scenario-pack` writes the pack: the files and the DOCFLOW-2 boundary.
fn doc_scenario_pack_summary(dir: &str, files: &[(&str, String)]) -> String {
    let mut out = format!(
        "doc-scenario-pack: wrote {} files to {dir} (1 valid + 8 invalid inputs; every invalid input REFUSED)\n",
        files.len()
    );
    for (name, content) in files {
        out.push_str(&format!("    {name} ({} bytes)\n", content.len()));
    }
    out.push_str("BOUNDARY\n");
    for line in DOC_SCENARIO_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `doc-scenario-verify` accepts a document-scenario pack.
fn doc_scenario_verify_summary() -> String {
    let mut out = String::from(
        "doc-scenario-verify: OK — the document-scenario pack re-derives byte-identically; every input outcome stands\n",
    );
    out.push_str("BOUNDARY\n");
    for line in DOC_SCENARIO_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `doc-bundle-verify` accepts a document bundle (every file
/// re-derived byte-identically from the SAME operator document).
fn doc_bundle_verify_summary() -> String {
    let mut out = String::from(
        "doc-bundle-verify: OK — every bundle file re-derives byte-identically from the operator document\n",
    );
    out.push_str("BOUNDARY\n");
    for line in DOC_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// Read the expected corpus bundle files from `dir`. A file that is absent is simply omitted (so
/// `verify_corpus_bundle` reports it missing rather than this shell guessing). The CONTENT is never trusted —
/// it is only re-derived and byte-compared by the library.
fn read_corpus_bundle(dir: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for name in CORPUS_BUNDLE_FILES {
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
            found.push((name.to_string(), content));
        }
    }
    Ok(found)
}

/// The human summary printed after `corpus-bundle` writes the pack: the files written and the CORPUS-0
/// boundary (the corpus flow reads local documents but does not trust them, and acts on nothing).
fn corpus_bundle_summary(dir: &str, files: &[(&str, String)]) -> String {
    let mut out = format!("corpus-bundle: wrote {} files to {dir}\n", files.len());
    for (name, content) in files {
        out.push_str(&format!("    {name} ({} bytes)\n", content.len()));
    }
    out.push_str("BOUNDARY\n");
    for line in CORPUS_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `corpus-bundle-verify` accepts a corpus bundle (every file re-derived
/// byte-identically from the SAME operator corpus).
fn corpus_bundle_verify_summary() -> String {
    let mut out = String::from(
        "corpus-bundle-verify: OK — every bundle file re-derives byte-identically from the operator corpus\n",
    );
    out.push_str("BOUNDARY\n");
    for line in CORPUS_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// Read the expected corpus-scenario pack files from `dir`. A file that is absent is simply omitted (so
/// `verify_corpus_scenario_pack` reports it missing rather than this shell guessing). The CONTENT is never
/// trusted — it is only re-derived and byte-compared by the library.
fn read_corpus_scenario_pack(dir: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for name in CORPUS_SCENARIO_PACK_FILES {
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
            found.push((name.to_string(), content));
        }
    }
    Ok(found)
}

/// The human summary printed after `corpus-scenario-pack` writes the pack: the files and the CORPUS-2 boundary.
fn corpus_scenario_pack_summary(dir: &str, files: &[(&str, String)]) -> String {
    let mut out = format!(
        "corpus-scenario-pack: wrote {} files to {dir} (1 valid + 12 invalid inputs; every invalid input REFUSED)\n",
        files.len()
    );
    for (name, content) in files {
        out.push_str(&format!("    {name} ({} bytes)\n", content.len()));
    }
    out.push_str("BOUNDARY\n");
    for line in CORPUS_SCENARIO_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `corpus-scenario-verify` accepts a corpus-scenario pack.
fn corpus_scenario_verify_summary() -> String {
    let mut out = String::from(
        "corpus-scenario-verify: OK — the corpus-scenario pack re-derives byte-identically; every input outcome stands\n",
    );
    out.push_str("BOUNDARY\n");
    for line in CORPUS_SCENARIO_BOUNDARY_LINES {
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

/// Read and VERIFY a whole scenario pack at `dir` by RE-DERIVING it: read each scenario's bundle and the
/// pack manifest from disk and hand them to the pure `verify_scenario_pack`, which byte-compares against
/// the re-derivation. A tampered/missing/foreign pack is refused. The provided files are never trusted —
/// only compared. Shared by `scenario-verify`, `scenario-matrix`, and `scenario-matrix-verify`.
fn verify_pack_at(dir: &str) -> Result<(), String> {
    let mut bundles = Vec::new();
    for scenario in Scenario::ALL {
        let sub = format!("{dir}/{}", scenario.slug());
        bundles.push((scenario.slug().to_string(), read_bundle(&sub)?));
    }
    let pack_path = format!("{dir}/{PACK_MANIFEST_FILE}");
    let pack =
        std::fs::read_to_string(&pack_path).map_err(|e| format!("cannot read {pack_path}: {e}"))?;
    verify_scenario_pack(&bundles, &pack).map_err(|e| e.to_string())
}

/// The success summary printed after `scenario-matrix-verify` accepts both the pack and the matrix.
fn scenario_matrix_verify_summary() -> String {
    let mut out = String::from(
        "scenario-matrix-verify: OK — the scenario pack and the coverage matrix re-derive byte-identically\n",
    );
    out.push_str("BOUNDARY\n");
    for line in MATRIX_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// Write the failure pack (the rejection-record JSON + its rendered report) into `dir` with EXACT bytes,
/// so every file re-reads and re-derives byte-identically. This shell only places the bytes on disk.
fn write_failure_pack(dir: &str, files: &[(&str, String)]) -> Result<(), String> {
    write_bundle(dir, files)
}

/// Read the expected failure-pack files from `dir`. A file that is absent is simply omitted (so
/// `verify_failure_pack` reports it missing rather than this shell guessing). The CONTENT is never trusted —
/// it is only re-derived and byte-compared by the library.
fn read_failure_pack(dir: &str) -> Result<Vec<(String, String)>, String> {
    let mut found = Vec::new();
    for name in FAILURE_PACK_FILES {
        let path = format!("{dir}/{name}");
        if std::path::Path::new(&path).exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
            found.push((name.to_string(), content));
        }
    }
    Ok(found)
}

/// The human summary printed after `failure-pack` writes the pack: the files written and the boundary.
fn failure_pack_summary(dir: &str, files: &[(&str, String)]) -> String {
    let mut out = format!(
        "failure-pack: wrote {} files to {dir} (every forged authority claim REJECTED)\n",
        files.len()
    );
    for (name, content) in files {
        out.push_str(&format!("    {name} ({} bytes)\n", content.len()));
    }
    out.push_str("BOUNDARY\n");
    for line in FAILURE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

/// The success summary printed after `failure-verify` accepts a failure pack (every file re-derived).
fn failure_verify_summary() -> String {
    let mut out = String::from(
        "failure-verify: OK — the failure pack re-derives byte-identically; every forged claim stays rejected\n",
    );
    out.push_str("BOUNDARY\n");
    for line in FAILURE_BOUNDARY_LINES {
        out.push_str(&format!("    {line}\n"));
    }
    out
}

fn usage() -> String {
    "usage: cognitive-demo <trace [--out PATH] | report --trace PATH [--out PATH] | \
     replay --trace PATH | ask --trace PATH --question SLUG [--out PATH] | questions | \
     bundle --out DIR | bundle-verify --path DIR | scenarios | scenario-pack --out DIR | \
     scenario-verify --path DIR | scenario-matrix --pack DIR [--out PATH] | \
     scenario-matrix-report --matrix PATH [--out PATH] | \
     scenario-matrix-verify --pack DIR --matrix PATH | failure-cases | \
     failure-pack --out DIR | failure-verify --path DIR | \
     lit-intent-demo [--out PATH] | lit-intent-demo-verify --map PATH | \
     lit-intent-matrix [--out PATH] | lit-intent-matrix-verify --matrix PATH | \
     teach-map-demo [--out PATH] | teach-map-demo-verify --lesson PATH | \
     teach-map-matrix [--out PATH] | teach-map-matrix-verify --matrix PATH | \
     learner-model-demo [--out PATH] | learner-model-demo-verify --state PATH | \
     learner-model-matrix [--out PATH] | learner-model-matrix-verify --matrix PATH | \
     learner-memory-demo [--out PATH] | learner-memory-demo-verify --memory PATH | \
     learner-memory-matrix [--out PATH] | learner-memory-matrix-verify --matrix PATH | \
     learner-journal-demo [--out PATH] | learner-journal-demo-verify --journal PATH | \
     learner-journal-matrix [--out PATH] | learner-journal-matrix-verify --matrix PATH | \
     learner-journal-append --journal PATH --consent-operator S --consent-scope S | \
     converse-demo [--out PATH] | converse-demo-verify --transcript PATH | \
     converse-matrix [--out PATH] | converse-matrix-verify --matrix PATH | \
     converse-run --input-dir DIR --script PATH [--out PATH] | \
     converse-run-verify --input-dir DIR --script PATH --transcript PATH | \
     learning-session-demo [--out PATH] | learning-session-demo-verify --session PATH | \
     learning-session-matrix [--out PATH] | learning-session-matrix-verify --matrix PATH | \
     learning-arc-demo [--out PATH] | learning-arc-demo-verify --arc PATH | \
     learning-arc-matrix [--out PATH] | learning-arc-matrix-verify --matrix PATH | \
     game-evidence-demo [--out PATH] | game-evidence-demo-verify --packet PATH | \
     game-evidence-matrix [--out PATH] | game-evidence-matrix-verify --matrix PATH | \
     wow-state-demo [--out PATH] | wow-state-demo-verify --snapshot PATH | \
     wow-state-matrix [--out PATH] | wow-state-matrix-verify --matrix PATH | \
     wow-taskplan-demo [--out PATH] | wow-taskplan-demo-verify --plan PATH | \
     wow-taskplan-matrix [--out PATH] | wow-taskplan-matrix-verify --matrix PATH | \
     controller-bridge-demo [--out PATH] | controller-bridge-demo-verify --envelope PATH | \
     controller-bridge-matrix [--out PATH] | controller-bridge-matrix-verify --matrix PATH | \
     live-actuator-producer-demo [--out PATH] | live-actuator-producer-demo-verify --artifact PATH | \
     live-actuator-producer-matrix [--out PATH] | live-actuator-producer-matrix-verify --matrix PATH | \
     live-actuator-producer-write --outbox PATH --ledger PATH | \
     doc-trace --input PATH [--out PATH] | doc-report --input PATH --trace PATH [--out PATH] | \
     doc-bundle --input PATH --out DIR | doc-bundle-verify --input PATH --path DIR | \
     doc-scenarios | doc-scenario-pack --out DIR | doc-scenario-verify --path DIR | \
     doc-scenario-matrix --path DIR [--out PATH] | \
     corpus-trace --input-dir DIR [--out PATH] | corpus-report --input-dir DIR --trace PATH [--out PATH] | \
     corpus-bundle --input-dir DIR --out DIR | corpus-bundle-verify --input-dir DIR --path DIR | \
     corpus-scenarios | corpus-scenario-pack --out DIR | corpus-scenario-verify --path DIR | \
     corpus-scenario-matrix --path DIR [--out PATH] | \
     novelty-packet --input-dir DIR --corpus-trace PATH --frame PATH [--out PATH] | \
     novelty-report --input-dir DIR --frame PATH --packet PATH [--out PATH] | \
     novelty-replay --input-dir DIR --frame PATH --packet PATH | \
     dream-export --input-dir DIR --frame PATH [--seed N] [--weirdness W] [--dream-packet PATH] [--out PATH] | \
     dream-export-report --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH [--out PATH] | \
     dream-export-replay --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH | \
     dream-export-scenarios | \
     dream-export-matrix --input-dir DIR --frame PATH [--seed N] [--weirdness W] [--out PATH] | \
     dream-export-matrix-report --input-dir DIR --frame PATH [--seed N] [--weirdness W] --matrix PATH [--out PATH] | \
     dream-export-matrix-verify --input-dir DIR --frame PATH [--seed N] [--weirdness W] --matrix PATH>"
        .to_string()
}

// ============================ LIVE-ACTUATOR-BRIDGE-0 ============================
// The laptop-side PRODUCER: wrap CONTROLLER-BRIDGE-0's dry-run command set in an
// emission-sequenced, ledger-linked artifact and (only in the `write` verb) drop it
// into a quarantined outbox with atomic temp+rename plus a durable, tamper-evident
// local ledger. This is SHELL code — filesystem I/O is allowed HERE, in the binary,
// never in the pure crate. It writes DRY-RUN artifacts only: it does not execute them,
// move the character, open a socket, call the client, authenticate approval, or arm a
// kill switch. Serialize-not-Deserialize; the durable ledger is re-read as closed
// pipe-delimited shell records (never serde-parsed) and every entry is re-hashed to
// detect tamper.
//
// Ledger-identity note: the per-command `command_id` is private to CONTROLLER-BRIDGE-0
// (ControllerCommandEnvelope has zero public fields — a load-bearing release lock — and
// controller_bridge.rs is out of scope here), so the producer keys its ledger on the
// controller-bridge RUN's public receipt_hash, which folds every command's command_id +
// envelope_hash + reissue_index. It is a strict superset identity carrying the same
// property the per-command id was designed for (distinct across legitimate reissues,
// repeats only on a byte-identical re-emit).

const LAP_SCHEMA_ARTIFACT: &str = "live-actuator-producer-artifact-v0.1";
const LAP_SCHEMA_LEDGER: &str = "live-actuator-producer-ledger-v0.1";
const LAP_SCHEMA_MATRIX: &str = "live-actuator-producer-matrix-v0.1";
const LAP_SCHEMA_RUN: &str = "live-actuator-producer-run-v0.1";
const LAP_LEDGER_RECORD_TAG: &str = "lap-v0.1";
const LAP_GENESIS_HEAD: u64 = 0;
const LAP_FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const LAP_FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const LAP_DECISION_PREPARED: &str = "artifact_prepared";

fn lap_fnv_mix(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(LAP_FNV_PRIME);
    }
    h
}

fn lap_fnv_i64(h: u64, v: i64) -> u64 {
    lap_fnv_mix(h, &(v as u64).to_le_bytes())
}

fn lap_fnv_u64(h: u64, v: u64) -> u64 {
    lap_fnv_mix(h, &v.to_le_bytes())
}

fn lap_flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProducerError {
    ReplayMismatch,
}

/// Producer signal gates — every flag names a forbidden capability, held false. Any
/// true refuses before an artifact is prepared.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct ProducerConfig {
    executes_live: bool,
    uses_network: bool,
    spawns_process: bool,
    uses_input_device: bool,
    uses_model: bool,
    uses_training: bool,
}

impl ProducerConfig {
    fn inert() -> Self {
        ProducerConfig {
            executes_live: false,
            uses_network: false,
            spawns_process: false,
            uses_input_device: false,
            uses_model: false,
            uses_training: false,
        }
    }
}

/// Structural boundary flags — every flag names a forbidden behavior, held false.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
struct ProducerBoundary {
    executes_artifact: bool,
    calls_client: bool,
    opens_socket: bool,
    moves_character: bool,
    authenticates_approval: bool,
    arms_kill_switch: bool,
    touches_live_stack: bool,
    creates_new_authority: bool,
}

impl ProducerBoundary {
    fn inert() -> Self {
        ProducerBoundary {
            executes_artifact: false,
            calls_client: false,
            opens_socket: false,
            moves_character: false,
            authenticates_approval: false,
            arms_kill_switch: false,
            touches_live_stack: false,
            creates_new_authority: false,
        }
    }

    fn all_inert(&self) -> bool {
        !(self.executes_artifact
            || self.calls_client
            || self.opens_socket
            || self.moves_character
            || self.authenticates_approval
            || self.arms_kill_switch
            || self.touches_live_stack
            || self.creates_new_authority)
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
enum ProducerDecision {
    ArtifactPrepared,
    ArtifactRefused,
}

impl ProducerDecision {
    fn slug(&self) -> &'static str {
        match self {
            ProducerDecision::ArtifactPrepared => "artifact_prepared",
            ProducerDecision::ArtifactRefused => "artifact_refused",
        }
    }
}

/// Every way the producer can refuse. Each variant is constructed in a reachable
/// production (the `write` verb) OR matrix path (the A3 fail-closed-debris law).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
enum ProducerRefusal {
    MissingOutbox,
    MissingLedger,
    InvalidOutboxPath,
    InvalidLedgerPath,
    OutboxNotDirectory,
    LedgerTamper,
    DuplicateCommand,
    EmissionSeqRegression,
    ProducerHeadMismatch,
    ArtifactAlreadyExists,
    AtomicWriteFailed,
    UnsupportedBridgeDecision,
    NonDryRunEnvelope,
    LiveExecutionSignalDetected,
    NetworkSignalDetected,
    ProcessSignalDetected,
    InputDeviceSignalDetected,
    ModelSignalDetected,
    TrainingSignalDetected,
    SerializedLiveActuatorProducerTamper,
    TargetQuestIdInvalid,
}

impl ProducerRefusal {
    #[allow(dead_code)] // enumerated for the A3 matrix-coverage test
    const ALL: [ProducerRefusal; 21] = [
        ProducerRefusal::MissingOutbox,
        ProducerRefusal::MissingLedger,
        ProducerRefusal::InvalidOutboxPath,
        ProducerRefusal::InvalidLedgerPath,
        ProducerRefusal::OutboxNotDirectory,
        ProducerRefusal::LedgerTamper,
        ProducerRefusal::DuplicateCommand,
        ProducerRefusal::EmissionSeqRegression,
        ProducerRefusal::ProducerHeadMismatch,
        ProducerRefusal::ArtifactAlreadyExists,
        ProducerRefusal::AtomicWriteFailed,
        ProducerRefusal::UnsupportedBridgeDecision,
        ProducerRefusal::NonDryRunEnvelope,
        ProducerRefusal::LiveExecutionSignalDetected,
        ProducerRefusal::NetworkSignalDetected,
        ProducerRefusal::ProcessSignalDetected,
        ProducerRefusal::InputDeviceSignalDetected,
        ProducerRefusal::ModelSignalDetected,
        ProducerRefusal::TrainingSignalDetected,
        ProducerRefusal::SerializedLiveActuatorProducerTamper,
        ProducerRefusal::TargetQuestIdInvalid,
    ];

    fn slug(&self) -> &'static str {
        match self {
            ProducerRefusal::MissingOutbox => "missing_outbox_refused",
            ProducerRefusal::MissingLedger => "missing_ledger_refused",
            ProducerRefusal::InvalidOutboxPath => "invalid_outbox_path_refused",
            ProducerRefusal::InvalidLedgerPath => "invalid_ledger_path_refused",
            ProducerRefusal::OutboxNotDirectory => "outbox_not_directory_refused",
            ProducerRefusal::LedgerTamper => "ledger_tamper_refused",
            ProducerRefusal::DuplicateCommand => "duplicate_command_refused",
            ProducerRefusal::EmissionSeqRegression => "emission_seq_regression_refused",
            ProducerRefusal::ProducerHeadMismatch => "producer_head_mismatch_refused",
            ProducerRefusal::ArtifactAlreadyExists => "artifact_already_exists_refused",
            ProducerRefusal::AtomicWriteFailed => "atomic_write_failed_refused",
            ProducerRefusal::UnsupportedBridgeDecision => "unsupported_bridge_decision_refused",
            ProducerRefusal::NonDryRunEnvelope => "non_dry_run_envelope_refused",
            ProducerRefusal::LiveExecutionSignalDetected => {
                "live_execution_signal_detected_refused"
            }
            ProducerRefusal::NetworkSignalDetected => "network_signal_detected_refused",
            ProducerRefusal::ProcessSignalDetected => "process_signal_detected_refused",
            ProducerRefusal::InputDeviceSignalDetected => "input_device_signal_detected_refused",
            ProducerRefusal::ModelSignalDetected => "model_signal_detected_refused",
            ProducerRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            ProducerRefusal::SerializedLiveActuatorProducerTamper => {
                "serialized_live_actuator_producer_tamper_refused"
            }
            ProducerRefusal::TargetQuestIdInvalid => "target_quest_id_invalid_refused",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct LiveActuatorEnvelopeArtifact {
    schema: String,
    emission_seq: i64,
    // Inert top-level metadata copied from the internally minted
    // controller_bridge run receipt (never parsed from text, never authority). The
    // client keys per-target emission_seq supersession on it.
    target_quest_id: i64,
    producer_head_before: u64,
    producer_head_after: u64,
    controller_bridge_envelope: ControllerBridgeRun,
    artifact_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProducerLedgerEntry {
    seq: i64,
    prev_entry_hash: u64,
    emission_seq: i64,
    command_id: u64,
    target_quest_id: i64,
    taskplan_receipt_hash: u64,
    evidence_receipt_hash: u64,
    state_receipt_hash: u64,
    artifact_hash: u64,
    decision: String,
    refusal: Option<String>,
    entry_hash: u64,
}

#[derive(Debug, Clone, Serialize)]
struct LiveActuatorProducerRun {
    schema: String,
    decision: ProducerDecision,
    refusal: Option<ProducerRefusal>,
    artifact: Option<LiveActuatorEnvelopeArtifact>,
    ledger_entry: Option<ProducerLedgerEntry>,
    boundary: ProducerBoundary,
    boundary_all_inert: bool,
}

// ------------------------------------------------------------ hashing ----------

fn lap_run_identity(run: &ControllerBridgeRun) -> (u64, u64, u64, u64) {
    (
        run.receipt.receipt_hash,
        run.receipt.taskplan_receipt_hash,
        run.receipt.evidence_receipt_hash,
        run.receipt.state_receipt_hash,
    )
}

fn lap_artifact_hash(emission_seq: i64, head_before: u64, run: &ControllerBridgeRun) -> u64 {
    let run_json = serde_json::to_string(run).expect("controller bridge run serializes");
    let mut h = LAP_FNV_OFFSET;
    h = lap_fnv_mix(h, LAP_SCHEMA_ARTIFACT.as_bytes());
    h = lap_fnv_i64(h, emission_seq);
    // Fold the lifted top-level target_quest_id explicitly (it also rides inside run_json,
    // but the lifted copy must be hash-bound so it cannot drift from its minted source).
    h = lap_fnv_i64(h, run.receipt.target_quest_id);
    h = lap_fnv_u64(h, head_before);
    h = lap_fnv_mix(h, run_json.as_bytes());
    h
}

#[allow(clippy::too_many_arguments)]
fn lap_entry_hash(
    seq: i64,
    prev: u64,
    emission_seq: i64,
    command_id: u64,
    target_quest_id: i64,
    taskplan: u64,
    evidence: u64,
    state: u64,
    artifact_hash: u64,
    decision: &str,
    refusal: &str,
) -> u64 {
    let mut h = LAP_FNV_OFFSET;
    h = lap_fnv_mix(h, LAP_SCHEMA_LEDGER.as_bytes());
    h = lap_fnv_i64(h, seq);
    h = lap_fnv_u64(h, prev);
    h = lap_fnv_i64(h, emission_seq);
    h = lap_fnv_u64(h, command_id);
    h = lap_fnv_i64(h, target_quest_id);
    h = lap_fnv_u64(h, taskplan);
    h = lap_fnv_u64(h, evidence);
    h = lap_fnv_u64(h, state);
    h = lap_fnv_u64(h, artifact_hash);
    h = lap_fnv_mix(h, decision.as_bytes());
    h = lap_fnv_mix(h, refusal.as_bytes());
    h
}

// ------------------------------------------------------------- guards ----------

fn lap_signal_refusal(config: &ProducerConfig) -> Option<ProducerRefusal> {
    if config.executes_live {
        Some(ProducerRefusal::LiveExecutionSignalDetected)
    } else if config.uses_network {
        Some(ProducerRefusal::NetworkSignalDetected)
    } else if config.spawns_process {
        Some(ProducerRefusal::ProcessSignalDetected)
    } else if config.uses_input_device {
        Some(ProducerRefusal::InputDeviceSignalDetected)
    } else if config.uses_model {
        Some(ProducerRefusal::ModelSignalDetected)
    } else if config.uses_training {
        Some(ProducerRefusal::TrainingSignalDetected)
    } else {
        None
    }
}

fn lap_bridge_prepared(decision: ControllerBridgeDecision) -> Option<ProducerRefusal> {
    if decision == ControllerBridgeDecision::EnvelopePrepared {
        None
    } else {
        Some(ProducerRefusal::UnsupportedBridgeDecision)
    }
}

fn lap_bridge_dry_run(dry_run: bool) -> Option<ProducerRefusal> {
    if dry_run {
        None
    } else {
        Some(ProducerRefusal::NonDryRunEnvelope)
    }
}

fn lap_outbox_present(outbox: Option<&str>) -> Option<ProducerRefusal> {
    match outbox {
        Some(p) if !p.is_empty() => None,
        _ => Some(ProducerRefusal::MissingOutbox),
    }
}

fn lap_ledger_present(ledger: Option<&str>) -> Option<ProducerRefusal> {
    match ledger {
        Some(p) if !p.is_empty() => None,
        _ => Some(ProducerRefusal::MissingLedger),
    }
}

/// A path is invalid if empty or if it contains a record-corrupting or control byte
/// (NUL, newline, CR) — a newline in the path would break the pipe-delimited ledger.
fn lap_path_valid(path: &str, refusal: ProducerRefusal) -> Option<ProducerRefusal> {
    if path.is_empty() || path.chars().any(|c| c == '\0' || c == '\n' || c == '\r') {
        Some(refusal)
    } else {
        None
    }
}

fn lap_outbox_is_directory(is_dir: bool) -> Option<ProducerRefusal> {
    if is_dir {
        None
    } else {
        Some(ProducerRefusal::OutboxNotDirectory)
    }
}

fn lap_duplicate_command(existing_ids: &[u64], command_id: u64) -> Option<ProducerRefusal> {
    if existing_ids.contains(&command_id) {
        Some(ProducerRefusal::DuplicateCommand)
    } else {
        None
    }
}

fn lap_emission_seq_ok(last_emission: i64, proposed: i64) -> Option<ProducerRefusal> {
    if proposed <= last_emission {
        Some(ProducerRefusal::EmissionSeqRegression)
    } else {
        None
    }
}

fn lap_producer_head_ok(actual_head: u64, head_before: u64) -> Option<ProducerRefusal> {
    if actual_head == head_before {
        None
    } else {
        Some(ProducerRefusal::ProducerHeadMismatch)
    }
}

fn lap_artifact_absent(exists: bool) -> Option<ProducerRefusal> {
    if exists {
        Some(ProducerRefusal::ArtifactAlreadyExists)
    } else {
        None
    }
}

fn lap_write_ok(ok: bool) -> Option<ProducerRefusal> {
    if ok {
        None
    } else {
        Some(ProducerRefusal::AtomicWriteFailed)
    }
}

/// The lifted top-level target_quest_id must be a real quest id (> 0, never the
/// refused-path 0 sentinel) AND must equal the value minted into the controller-bridge
/// receipt it was copied from. In production `lifted` and `minted` are one value (a single
/// source of truth), so the mismatch branch is a regression tripwire constructed in the
/// matrix; the `minted <= 0` branch is the live sentinel defense.
fn lap_target_quest_id_ok(lifted: i64, minted: i64) -> Option<ProducerRefusal> {
    if minted <= 0 || lifted != minted {
        Some(ProducerRefusal::TargetQuestIdInvalid)
    } else {
        None
    }
}

// -------------------------------------------------------- ledger records -------

fn lap_entry_record(e: &ProducerLedgerEntry) -> String {
    let refusal = e.refusal.as_deref().unwrap_or("-");
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        LAP_LEDGER_RECORD_TAG,
        e.seq,
        e.prev_entry_hash,
        e.emission_seq,
        e.command_id,
        e.target_quest_id,
        e.taskplan_receipt_hash,
        e.evidence_receipt_hash,
        e.state_receipt_hash,
        e.artifact_hash,
        e.decision,
        refusal,
        e.entry_hash
    )
}

/// Parse + verify the durable ledger as CLOSED shell records (no serde). Any malformed
/// record, a broken prev-hash chain, a sequence gap, or a recompute-hash mismatch is
/// ledger tamper. This is how the deferred CONTROLLER-BRIDGE-0 ordering discipline is
/// enforced on the producer side.
fn lap_parse_ledger(text: &str) -> Result<Vec<ProducerLedgerEntry>, ProducerRefusal> {
    let mut entries: Vec<ProducerLedgerEntry> = Vec::new();
    let mut expected_seq = 1i64;
    let mut prev = LAP_GENESIS_HEAD;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() != 13 || parts[0] != LAP_LEDGER_RECORD_TAG {
            return Err(ProducerRefusal::LedgerTamper);
        }
        let seq = parts[1]
            .parse::<i64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let rec_prev = parts[2]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let emission = parts[3]
            .parse::<i64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let command_id = parts[4]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let target_quest_id = parts[5]
            .parse::<i64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let taskplan = parts[6]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let evidence = parts[7]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let state = parts[8]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let artifact_hash = parts[9]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        let decision = parts[10].to_string();
        let refusal_raw = parts[11];
        let entry_hash = parts[12]
            .parse::<u64>()
            .map_err(|_| ProducerRefusal::LedgerTamper)?;
        if seq != expected_seq || rec_prev != prev {
            return Err(ProducerRefusal::LedgerTamper);
        }
        let recomputed = lap_entry_hash(
            seq,
            rec_prev,
            emission,
            command_id,
            target_quest_id,
            taskplan,
            evidence,
            state,
            artifact_hash,
            &decision,
            refusal_raw,
        );
        if recomputed != entry_hash {
            return Err(ProducerRefusal::LedgerTamper);
        }
        let refusal = if refusal_raw == "-" {
            None
        } else {
            Some(refusal_raw.to_string())
        };
        entries.push(ProducerLedgerEntry {
            seq,
            prev_entry_hash: rec_prev,
            emission_seq: emission,
            command_id,
            target_quest_id,
            taskplan_receipt_hash: taskplan,
            evidence_receipt_hash: evidence,
            state_receipt_hash: state,
            artifact_hash,
            decision,
            refusal,
            entry_hash,
        });
        prev = entry_hash;
        expected_seq += 1;
    }
    Ok(entries)
}

// ------------------------------------------------------------- builders --------

#[allow(clippy::too_many_arguments)]
fn lap_build_entry(
    seq: i64,
    prev: u64,
    emission_seq: i64,
    command_id: u64,
    target_quest_id: i64,
    taskplan: u64,
    evidence: u64,
    state: u64,
    artifact_hash: u64,
) -> ProducerLedgerEntry {
    let decision = LAP_DECISION_PREPARED.to_string();
    let entry_hash = lap_entry_hash(
        seq,
        prev,
        emission_seq,
        command_id,
        target_quest_id,
        taskplan,
        evidence,
        state,
        artifact_hash,
        &decision,
        "-",
    );
    ProducerLedgerEntry {
        seq,
        prev_entry_hash: prev,
        emission_seq,
        command_id,
        target_quest_id,
        taskplan_receipt_hash: taskplan,
        evidence_receipt_hash: evidence,
        state_receipt_hash: state,
        artifact_hash,
        decision,
        refusal: None,
        entry_hash,
    }
}

fn lap_prepare(
    run: &ControllerBridgeRun,
    seq: i64,
    emission_seq: i64,
    head_before: u64,
) -> LiveActuatorProducerRun {
    let (command_id, taskplan, evidence, state) = lap_run_identity(run);
    let target_quest_id = run.receipt.target_quest_id;
    let artifact_hash = lap_artifact_hash(emission_seq, head_before, run);
    let entry = lap_build_entry(
        seq,
        head_before,
        emission_seq,
        command_id,
        target_quest_id,
        taskplan,
        evidence,
        state,
        artifact_hash,
    );
    let artifact = LiveActuatorEnvelopeArtifact {
        schema: LAP_SCHEMA_ARTIFACT.to_string(),
        emission_seq,
        target_quest_id,
        producer_head_before: head_before,
        producer_head_after: entry.entry_hash,
        controller_bridge_envelope: run.clone(),
        artifact_hash,
    };
    let boundary = ProducerBoundary::inert();
    LiveActuatorProducerRun {
        schema: LAP_SCHEMA_RUN.to_string(),
        decision: ProducerDecision::ArtifactPrepared,
        refusal: None,
        artifact: Some(artifact),
        ledger_entry: Some(entry),
        boundary,
        boundary_all_inert: boundary.all_inert(),
    }
}

fn lap_refuse(refusal: ProducerRefusal) -> LiveActuatorProducerRun {
    let boundary = ProducerBoundary::inert();
    LiveActuatorProducerRun {
        schema: LAP_SCHEMA_RUN.to_string(),
        decision: ProducerDecision::ArtifactRefused,
        refusal: Some(refusal),
        artifact: None,
        ledger_entry: None,
        boundary,
        boundary_all_inert: boundary.all_inert(),
    }
}

/// The canonical dry-run source run (CONTROLLER-BRIDGE-0's demo command set).
fn lap_source_run() -> ControllerBridgeRun {
    controller_bridge_demo()
}

// ------------------------------------------------------------- demo ------------

fn producer_demo() -> LiveActuatorProducerRun {
    let run = lap_source_run();
    // Pre-flight gates (all pass for the canonical dry-run run) — wired so they cannot
    // be silently deleted.
    if let Some(refusal) = lap_signal_refusal(&ProducerConfig::inert()) {
        return lap_refuse(refusal);
    }
    if let Some(refusal) = lap_bridge_prepared(run.receipt.decision) {
        return lap_refuse(refusal);
    }
    if let Some(refusal) = lap_bridge_dry_run(run.receipt.config.dry_run) {
        return lap_refuse(refusal);
    }
    let target_quest_id = run.receipt.target_quest_id;
    if let Some(refusal) = lap_target_quest_id_ok(target_quest_id, target_quest_id) {
        return lap_refuse(refusal);
    }
    // An empty ledger: emission_seq 1, prev = genesis head.
    lap_prepare(&run, 1, 1, LAP_GENESIS_HEAD)
}

fn producer_demo_json() -> String {
    serde_json::to_string_pretty(&producer_demo()).expect("producer demo serializes")
}

fn verify_producer_demo_json(candidate: &str) -> Result<(), ProducerError> {
    if candidate == producer_demo_json() {
        Ok(())
    } else {
        Err(ProducerError::ReplayMismatch)
    }
}

// ------------------------------------------------------------- matrix ----------

const LAP_SCENARIO_COUNT: usize = 22;
const LAP_SCENARIO_NAMES: [&str; LAP_SCENARIO_COUNT] = [
    "prepared_artifact_written_to_outbox",
    "producer_ledger_appends_seq_1",
    "producer_ledger_appends_seq_2",
    "duplicate_command_refused",
    "ledger_tamper_refused",
    "emission_seq_regression_refused",
    "producer_head_mismatch_refused",
    "artifact_already_exists_refused",
    "non_dry_run_envelope_refused",
    "unsupported_bridge_decision_refused",
    "missing_outbox_refused",
    "outbox_not_directory_refused",
    "invalid_ledger_path_refused",
    "atomic_write_failed_refused",
    "live_execution_signal_detected_refused",
    "network_signal_detected_refused",
    "process_signal_detected_refused",
    "input_device_signal_detected_refused",
    "model_signal_detected_refused",
    "training_signal_detected_refused",
    "serialized_live_actuator_producer_tamper_refused",
    "target_quest_id_invalid_refused",
];

#[derive(Debug, Clone, Serialize)]
struct ProducerCell {
    scenario: String,
    outcome: String,
    refusal: Option<String>,
    seq: i64,
    emission_seq: i64,
    linked: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ProducerMatrix {
    schema: String,
    scenario_count: usize,
    cells: Vec<ProducerCell>,
    prepared_count: usize,
    refused_count: usize,
    boundary: ProducerBoundary,
    boundary_all_inert: bool,
}

fn lap_prepared_cell(scenario: &str, run: &LiveActuatorProducerRun) -> ProducerCell {
    let entry = run.ledger_entry.as_ref();
    ProducerCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        seq: entry.map(|e| e.seq).unwrap_or(0),
        emission_seq: entry.map(|e| e.emission_seq).unwrap_or(0),
        linked: entry
            .map(|e| {
                e.prev_entry_hash == LAP_GENESIS_HEAD
                    && e.entry_hash
                        == lap_entry_hash(
                            e.seq,
                            e.prev_entry_hash,
                            e.emission_seq,
                            e.command_id,
                            e.target_quest_id,
                            e.taskplan_receipt_hash,
                            e.evidence_receipt_hash,
                            e.state_receipt_hash,
                            e.artifact_hash,
                            &e.decision,
                            "-",
                        )
            })
            .unwrap_or(false),
    }
}

fn lap_refusal_cell(scenario: &str, refusal: ProducerRefusal, fired: bool) -> ProducerCell {
    ProducerCell {
        scenario: scenario.to_string(),
        outcome: if fired {
            "artifact_refused"
        } else {
            "refusal_missed"
        }
        .to_string(),
        refusal: Some(refusal.slug().to_string()),
        seq: 0,
        emission_seq: 0,
        linked: false,
    }
}

fn lap_entry_cell(scenario: &str, entry: &ProducerLedgerEntry, expected_prev: u64) -> ProducerCell {
    let linked = entry.prev_entry_hash == expected_prev
        && entry.entry_hash
            == lap_entry_hash(
                entry.seq,
                entry.prev_entry_hash,
                entry.emission_seq,
                entry.command_id,
                entry.target_quest_id,
                entry.taskplan_receipt_hash,
                entry.evidence_receipt_hash,
                entry.state_receipt_hash,
                entry.artifact_hash,
                &entry.decision,
                "-",
            );
    ProducerCell {
        scenario: scenario.to_string(),
        outcome: if linked {
            "ledger_linked"
        } else {
            "ledger_broken"
        }
        .to_string(),
        refusal: None,
        seq: entry.seq,
        emission_seq: entry.emission_seq,
        linked,
    }
}

fn lap_cell_for(scenario: &str) -> ProducerCell {
    match scenario {
        "prepared_artifact_written_to_outbox" => lap_prepared_cell(scenario, &producer_demo()),
        "producer_ledger_appends_seq_1" => {
            let entry = producer_demo().ledger_entry.expect("demo entry");
            lap_entry_cell(scenario, &entry, LAP_GENESIS_HEAD)
        }
        "producer_ledger_appends_seq_2" => {
            // A durable ledger already holds entry 1; a DISTINCT command identity appends
            // as entry 2, prev-linked to entry 1's head.
            let e1 = producer_demo().ledger_entry.expect("demo entry");
            let (command_id, taskplan, evidence, state) = lap_run_identity(&lap_source_run());
            let e2 = lap_build_entry(
                2,
                e1.entry_hash,
                2,
                command_id ^ 0x01,
                e1.target_quest_id,
                taskplan,
                evidence,
                state,
                e1.artifact_hash ^ 0x01,
            );
            lap_entry_cell(scenario, &e2, e1.entry_hash)
        }
        "duplicate_command_refused" => {
            let (command_id, _, _, _) = lap_run_identity(&lap_source_run());
            let fired = lap_duplicate_command(&[command_id], command_id)
                == Some(ProducerRefusal::DuplicateCommand);
            lap_refusal_cell(scenario, ProducerRefusal::DuplicateCommand, fired)
        }
        "ledger_tamper_refused" => {
            let e1 = producer_demo().ledger_entry.expect("demo entry");
            let tampered = lap_flip_last_byte(&lap_entry_record(&e1));
            let fired = lap_parse_ledger(&tampered) == Err(ProducerRefusal::LedgerTamper);
            lap_refusal_cell(scenario, ProducerRefusal::LedgerTamper, fired)
        }
        "emission_seq_regression_refused" => {
            let fired = lap_emission_seq_ok(5, 5) == Some(ProducerRefusal::EmissionSeqRegression);
            lap_refusal_cell(scenario, ProducerRefusal::EmissionSeqRegression, fired)
        }
        "producer_head_mismatch_refused" => {
            let fired = lap_producer_head_ok(0xABCD, 0xABCD ^ 0x01)
                == Some(ProducerRefusal::ProducerHeadMismatch);
            lap_refusal_cell(scenario, ProducerRefusal::ProducerHeadMismatch, fired)
        }
        "artifact_already_exists_refused" => {
            let fired = lap_artifact_absent(true) == Some(ProducerRefusal::ArtifactAlreadyExists);
            lap_refusal_cell(scenario, ProducerRefusal::ArtifactAlreadyExists, fired)
        }
        "non_dry_run_envelope_refused" => {
            let fired = lap_bridge_dry_run(false) == Some(ProducerRefusal::NonDryRunEnvelope);
            lap_refusal_cell(scenario, ProducerRefusal::NonDryRunEnvelope, fired)
        }
        "unsupported_bridge_decision_refused" => {
            let fired = lap_bridge_prepared(ControllerBridgeDecision::EnvelopeRefused)
                == Some(ProducerRefusal::UnsupportedBridgeDecision);
            lap_refusal_cell(scenario, ProducerRefusal::UnsupportedBridgeDecision, fired)
        }
        "missing_outbox_refused" => {
            let fired = lap_outbox_present(None) == Some(ProducerRefusal::MissingOutbox);
            lap_refusal_cell(scenario, ProducerRefusal::MissingOutbox, fired)
        }
        "outbox_not_directory_refused" => {
            let fired = lap_outbox_is_directory(false) == Some(ProducerRefusal::OutboxNotDirectory);
            lap_refusal_cell(scenario, ProducerRefusal::OutboxNotDirectory, fired)
        }
        "invalid_ledger_path_refused" => {
            let fired = lap_path_valid("bad\nledger", ProducerRefusal::InvalidLedgerPath)
                == Some(ProducerRefusal::InvalidLedgerPath);
            lap_refusal_cell(scenario, ProducerRefusal::InvalidLedgerPath, fired)
        }
        "atomic_write_failed_refused" => {
            let fired = lap_write_ok(false) == Some(ProducerRefusal::AtomicWriteFailed);
            lap_refusal_cell(scenario, ProducerRefusal::AtomicWriteFailed, fired)
        }
        "live_execution_signal_detected_refused" => {
            let mut config = ProducerConfig::inert();
            config.executes_live = true;
            let fired =
                lap_signal_refusal(&config) == Some(ProducerRefusal::LiveExecutionSignalDetected);
            lap_refusal_cell(
                scenario,
                ProducerRefusal::LiveExecutionSignalDetected,
                fired,
            )
        }
        "network_signal_detected_refused" => {
            let mut config = ProducerConfig::inert();
            config.uses_network = true;
            let fired = lap_signal_refusal(&config) == Some(ProducerRefusal::NetworkSignalDetected);
            lap_refusal_cell(scenario, ProducerRefusal::NetworkSignalDetected, fired)
        }
        "process_signal_detected_refused" => {
            let mut config = ProducerConfig::inert();
            config.spawns_process = true;
            let fired = lap_signal_refusal(&config) == Some(ProducerRefusal::ProcessSignalDetected);
            lap_refusal_cell(scenario, ProducerRefusal::ProcessSignalDetected, fired)
        }
        "input_device_signal_detected_refused" => {
            let mut config = ProducerConfig::inert();
            config.uses_input_device = true;
            let fired =
                lap_signal_refusal(&config) == Some(ProducerRefusal::InputDeviceSignalDetected);
            lap_refusal_cell(scenario, ProducerRefusal::InputDeviceSignalDetected, fired)
        }
        "model_signal_detected_refused" => {
            let mut config = ProducerConfig::inert();
            config.uses_model = true;
            let fired = lap_signal_refusal(&config) == Some(ProducerRefusal::ModelSignalDetected);
            lap_refusal_cell(scenario, ProducerRefusal::ModelSignalDetected, fired)
        }
        "training_signal_detected_refused" => {
            let mut config = ProducerConfig::inert();
            config.uses_training = true;
            let fired =
                lap_signal_refusal(&config) == Some(ProducerRefusal::TrainingSignalDetected);
            lap_refusal_cell(scenario, ProducerRefusal::TrainingSignalDetected, fired)
        }
        "serialized_live_actuator_producer_tamper_refused" => {
            let json = producer_demo_json();
            let refused = verify_producer_demo_json(&lap_flip_last_byte(&json)).is_err();
            ProducerCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused"
                } else {
                    "tamper_missed"
                }
                .to_string(),
                refusal: if refused {
                    Some(
                        ProducerRefusal::SerializedLiveActuatorProducerTamper
                            .slug()
                            .to_string(),
                    )
                } else {
                    None
                },
                seq: 0,
                emission_seq: 0,
                linked: false,
            }
        }
        "target_quest_id_invalid_refused" => {
            // Construct BOTH invalid sub-cases: the 0-sentinel (minted <= 0, the live
            // defense) and the lifted-vs-minted mismatch (the matrix-only regression
            // tripwire). The demo/write production path always passes (788, 788).
            let sentinel =
                lap_target_quest_id_ok(788, 0) == Some(ProducerRefusal::TargetQuestIdInvalid);
            let mismatch =
                lap_target_quest_id_ok(999_999, 788) == Some(ProducerRefusal::TargetQuestIdInvalid);
            lap_refusal_cell(
                scenario,
                ProducerRefusal::TargetQuestIdInvalid,
                sentinel && mismatch,
            )
        }
        other => ProducerCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            seq: 0,
            emission_seq: 0,
            linked: false,
        },
    }
}

fn producer_matrix() -> ProducerMatrix {
    let cells = LAP_SCENARIO_NAMES
        .iter()
        .map(|scenario| lap_cell_for(scenario))
        .collect::<Vec<_>>();
    let prepared_count = cells
        .iter()
        .filter(|c| c.outcome == "artifact_prepared")
        .count();
    let refused_count = cells
        .iter()
        .filter(|c| c.outcome == "artifact_refused" || c.outcome == "tamper_refused")
        .count();
    let boundary = ProducerBoundary::inert();
    ProducerMatrix {
        schema: LAP_SCHEMA_MATRIX.to_string(),
        scenario_count: cells.len(),
        cells,
        prepared_count,
        refused_count,
        boundary,
        boundary_all_inert: boundary.all_inert(),
    }
}

fn producer_matrix_json() -> String {
    serde_json::to_string_pretty(&producer_matrix()).expect("producer matrix serializes")
}

fn verify_producer_matrix_json(candidate: &str) -> Result<(), ProducerError> {
    if candidate == producer_matrix_json() {
        Ok(())
    } else {
        Err(ProducerError::ReplayMismatch)
    }
}

// ------------------------------------------------------- the write verb --------

/// Atomic write: temp file beside the final path, fsync, then rename over the final
/// name. A rename on the same filesystem is atomic, so a reader never sees a partial
/// artifact. Never overwrites via a partial write.
fn lap_atomic_write(final_path: &str, content: &str) -> Result<(), String> {
    let tmp_path = format!("{final_path}.lap-tmp");
    std::fs::write(&tmp_path, content).map_err(|e| format!("cannot write temp {tmp_path}: {e}"))?;
    if let Ok(file) = std::fs::File::open(&tmp_path) {
        let _ = file.sync_all();
    }
    std::fs::rename(&tmp_path, final_path)
        .map_err(|e| format!("cannot rename {tmp_path} -> {final_path}: {e}"))
}

/// The shell verb. Reads the durable ledger, verifies its chain, refuses on any guard,
/// and (only if prepared) atomically drops the DRY-RUN artifact into the outbox and
/// appends the ledger record. Returns Err("refused: <slug>") on any refusal so the CLI
/// exits non-zero, or Ok(summary) after a successful quarantined write.
fn run_producer_write(outbox: Option<&str>, ledger: Option<&str>) -> Result<String, String> {
    if let Some(r) = lap_outbox_present(outbox) {
        return Err(format!("refused: {}", r.slug()));
    }
    if let Some(r) = lap_ledger_present(ledger) {
        return Err(format!("refused: {}", r.slug()));
    }
    let outbox = outbox.expect("outbox present");
    let ledger = ledger.expect("ledger present");
    if let Some(r) = lap_path_valid(outbox, ProducerRefusal::InvalidOutboxPath) {
        return Err(format!("refused: {}", r.slug()));
    }
    if let Some(r) = lap_path_valid(ledger, ProducerRefusal::InvalidLedgerPath) {
        return Err(format!("refused: {}", r.slug()));
    }
    if let Some(r) = lap_signal_refusal(&ProducerConfig::inert()) {
        return Err(format!("refused: {}", r.slug()));
    }
    if let Some(r) = lap_outbox_is_directory(std::path::Path::new(outbox).is_dir()) {
        return Err(format!("refused: {}", r.slug()));
    }
    let ledger_text = if std::path::Path::new(ledger).exists() {
        std::fs::read_to_string(ledger).map_err(|e| format!("cannot read ledger {ledger}: {e}"))?
    } else {
        String::new()
    };
    let entries = match lap_parse_ledger(&ledger_text) {
        Ok(entries) => entries,
        Err(r) => return Err(format!("refused: {}", r.slug())),
    };
    let run = lap_source_run();
    if let Some(r) = lap_bridge_prepared(run.receipt.decision) {
        return Err(format!("refused: {}", r.slug()));
    }
    if let Some(r) = lap_bridge_dry_run(run.receipt.config.dry_run) {
        return Err(format!("refused: {}", r.slug()));
    }
    let target_quest_id = run.receipt.target_quest_id;
    if let Some(r) = lap_target_quest_id_ok(target_quest_id, target_quest_id) {
        return Err(format!("refused: {}", r.slug()));
    }
    let (command_id, _, _, _) = lap_run_identity(&run);
    let existing_ids: Vec<u64> = entries.iter().map(|e| e.command_id).collect();
    if let Some(r) = lap_duplicate_command(&existing_ids, command_id) {
        return Err(format!("refused: {}", r.slug()));
    }
    let last_emission = entries.last().map(|e| e.emission_seq).unwrap_or(0);
    let emission = last_emission + 1;
    if let Some(r) = lap_emission_seq_ok(last_emission, emission) {
        return Err(format!("refused: {}", r.slug()));
    }
    let head = entries
        .last()
        .map(|e| e.entry_hash)
        .unwrap_or(LAP_GENESIS_HEAD);
    if let Some(r) = lap_producer_head_ok(head, head) {
        return Err(format!("refused: {}", r.slug()));
    }
    let seq = entries.len() as i64 + 1;
    let prepared = lap_prepare(&run, seq, emission, head);
    let artifact = prepared.artifact.as_ref().expect("prepared artifact");
    let entry = prepared.ledger_entry.as_ref().expect("prepared entry");
    let final_name = format!("{LAP_SCHEMA_ARTIFACT}-{emission}-{command_id:016x}.json");
    let final_path = format!("{outbox}/{final_name}");
    if let Some(r) = lap_artifact_absent(std::path::Path::new(&final_path).exists()) {
        return Err(format!("refused: {}", r.slug()));
    }
    let artifact_json =
        serde_json::to_string_pretty(artifact).expect("producer artifact serializes");
    if let Some(r) = lap_write_ok(lap_atomic_write(&final_path, &artifact_json).is_ok()) {
        return Err(format!("refused: {}", r.slug()));
    }
    let mut new_ledger = ledger_text.clone();
    if !new_ledger.is_empty() && !new_ledger.ends_with('\n') {
        new_ledger.push('\n');
    }
    new_ledger.push_str(&lap_entry_record(entry));
    new_ledger.push('\n');
    if let Some(r) = lap_write_ok(lap_atomic_write(ledger, &new_ledger).is_ok()) {
        return Err(format!("refused: {}", r.slug()));
    }
    Ok(format!(
        "live-actuator-producer-write: OK\n  artifact: {final_path}\n  emission_seq: {emission}\n  seq: {seq}\n  command_id: {command_id}\n  target_quest_id: {target_quest_id}\n  producer_head_after: {}\n  dry_run: true (artifact is NOT authorized for execution)\n",
        entry.entry_hash
    ))
}

#[cfg(test)]
mod producer_tests {
    use super::*;

    type SignalCase = (fn(&mut ProducerConfig), ProducerRefusal);

    #[test]
    fn demo_prepares_seq_1_artifact_on_empty_ledger() {
        let run = producer_demo();
        assert_eq!(run.decision, ProducerDecision::ArtifactPrepared);
        assert!(run.refusal.is_none());
        let artifact = run.artifact.expect("prepared artifact");
        assert_eq!(artifact.emission_seq, 1);
        assert_eq!(artifact.producer_head_before, LAP_GENESIS_HEAD);
        let entry = run.ledger_entry.expect("prepared entry");
        assert_eq!(entry.seq, 1);
        assert_eq!(entry.emission_seq, 1);
        assert_eq!(entry.prev_entry_hash, LAP_GENESIS_HEAD);
        assert_eq!(artifact.producer_head_after, entry.entry_hash);
        assert!(run.boundary_all_inert);
    }

    #[test]
    fn artifact_wraps_the_dry_run_bridge_run_and_binds_anchors() {
        let source = lap_source_run();
        let (command_id, taskplan, evidence, state) = lap_run_identity(&source);
        let run = producer_demo();
        let entry = run.ledger_entry.expect("entry");
        // The ledger identity is the controller-bridge run receipt_hash (per-command id
        // is private-by-lock); anchors come straight from the public receipt.
        assert_eq!(entry.command_id, command_id);
        assert_eq!(entry.command_id, source.receipt.receipt_hash);
        assert_eq!(entry.taskplan_receipt_hash, taskplan);
        assert_eq!(entry.evidence_receipt_hash, evidence);
        assert_eq!(entry.state_receipt_hash, state);
        // The wrapped run is dry-run and prepared.
        let artifact = run.artifact.expect("artifact");
        assert_eq!(
            artifact.controller_bridge_envelope.decision,
            ControllerBridgeDecision::EnvelopePrepared
        );
        assert!(artifact.controller_bridge_envelope.receipt.config.dry_run);
    }

    #[test]
    fn emission_seq_starts_at_one_and_increments() {
        // Empty ledger => next emission is 1.
        assert!(lap_emission_seq_ok(0, 1).is_none());
        // A regression (proposed <= last) refuses.
        assert_eq!(
            lap_emission_seq_ok(5, 5),
            Some(ProducerRefusal::EmissionSeqRegression)
        );
        assert_eq!(
            lap_emission_seq_ok(5, 3),
            Some(ProducerRefusal::EmissionSeqRegression)
        );
        assert!(lap_emission_seq_ok(5, 6).is_none());
    }

    #[test]
    fn ledger_round_trips_and_chain_verifies() {
        let e1 = producer_demo().ledger_entry.expect("entry");
        let record = lap_entry_record(&e1);
        let parsed = lap_parse_ledger(&format!("{record}\n")).expect("parses");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].entry_hash, e1.entry_hash);
        assert_eq!(parsed[0].command_id, e1.command_id);
        assert_eq!(parsed[0].target_quest_id, e1.target_quest_id);
    }

    #[test]
    fn ledger_tamper_is_refused() {
        let e1 = producer_demo().ledger_entry.expect("entry");
        let record = lap_entry_record(&e1);
        // Flip a hash digit in the record => recompute mismatch => tamper.
        let tampered = record.replacen(&e1.entry_hash.to_string(), "123", 1);
        assert_eq!(
            lap_parse_ledger(&format!("{tampered}\n")),
            Err(ProducerRefusal::LedgerTamper)
        );
        // A byte-flip anywhere is also caught.
        assert_eq!(
            lap_parse_ledger(&format!("{}\n", lap_flip_last_byte(&record))),
            Err(ProducerRefusal::LedgerTamper)
        );
    }

    #[test]
    fn broken_prev_chain_is_refused() {
        let e1 = producer_demo().ledger_entry.expect("entry");
        // A second record whose prev does NOT link to entry 1's head.
        let (command_id, taskplan, evidence, state) = lap_run_identity(&lap_source_run());
        let bad = lap_build_entry(
            2,
            0xDEAD,
            2,
            command_id ^ 0x01,
            e1.target_quest_id,
            taskplan,
            evidence,
            state,
            7,
        );
        let text = format!("{}\n{}\n", lap_entry_record(&e1), lap_entry_record(&bad));
        assert_eq!(lap_parse_ledger(&text), Err(ProducerRefusal::LedgerTamper));
    }

    #[test]
    fn duplicate_command_is_refused() {
        let (command_id, _, _, _) = lap_run_identity(&lap_source_run());
        assert_eq!(
            lap_duplicate_command(&[command_id], command_id),
            Some(ProducerRefusal::DuplicateCommand)
        );
        assert!(lap_duplicate_command(&[command_id], command_id ^ 0x01).is_none());
    }

    #[test]
    fn signal_gates_each_refuse() {
        let cases: [SignalCase; 6] = [
            (
                |c| c.executes_live = true,
                ProducerRefusal::LiveExecutionSignalDetected,
            ),
            (
                |c| c.uses_network = true,
                ProducerRefusal::NetworkSignalDetected,
            ),
            (
                |c| c.spawns_process = true,
                ProducerRefusal::ProcessSignalDetected,
            ),
            (
                |c| c.uses_input_device = true,
                ProducerRefusal::InputDeviceSignalDetected,
            ),
            (
                |c| c.uses_model = true,
                ProducerRefusal::ModelSignalDetected,
            ),
            (
                |c| c.uses_training = true,
                ProducerRefusal::TrainingSignalDetected,
            ),
        ];
        for (set, expected) in cases {
            let mut config = ProducerConfig::inert();
            set(&mut config);
            assert_eq!(lap_signal_refusal(&config), Some(expected));
        }
        assert!(lap_signal_refusal(&ProducerConfig::inert()).is_none());
    }

    #[test]
    fn non_dry_run_and_unsupported_decision_refuse() {
        assert_eq!(
            lap_bridge_dry_run(false),
            Some(ProducerRefusal::NonDryRunEnvelope)
        );
        assert!(lap_bridge_dry_run(true).is_none());
        assert_eq!(
            lap_bridge_prepared(ControllerBridgeDecision::EnvelopeRefused),
            Some(ProducerRefusal::UnsupportedBridgeDecision)
        );
        assert!(lap_bridge_prepared(ControllerBridgeDecision::EnvelopePrepared).is_none());
    }

    #[test]
    fn path_and_presence_guards_refuse() {
        assert_eq!(
            lap_outbox_present(None),
            Some(ProducerRefusal::MissingOutbox)
        );
        assert_eq!(
            lap_outbox_present(Some("")),
            Some(ProducerRefusal::MissingOutbox)
        );
        assert!(lap_outbox_present(Some("outbox")).is_none());
        assert_eq!(
            lap_ledger_present(None),
            Some(ProducerRefusal::MissingLedger)
        );
        assert_eq!(
            lap_path_valid("a\nb", ProducerRefusal::InvalidOutboxPath),
            Some(ProducerRefusal::InvalidOutboxPath)
        );
        assert!(lap_path_valid("outbox/dir", ProducerRefusal::InvalidOutboxPath).is_none());
        assert_eq!(
            lap_outbox_is_directory(false),
            Some(ProducerRefusal::OutboxNotDirectory)
        );
        assert_eq!(
            lap_artifact_absent(true),
            Some(ProducerRefusal::ArtifactAlreadyExists)
        );
        assert_eq!(
            lap_write_ok(false),
            Some(ProducerRefusal::AtomicWriteFailed)
        );
        assert_eq!(
            lap_producer_head_ok(1, 2),
            Some(ProducerRefusal::ProducerHeadMismatch)
        );
    }

    #[test]
    fn demo_and_matrix_replay_and_refuse_tamper() {
        let demo = producer_demo_json();
        assert!(verify_producer_demo_json(&demo).is_ok());
        assert_eq!(
            verify_producer_demo_json(&lap_flip_last_byte(&demo)),
            Err(ProducerError::ReplayMismatch)
        );
        let matrix = producer_matrix_json();
        assert!(verify_producer_matrix_json(&matrix).is_ok());
        assert_eq!(
            verify_producer_matrix_json(&lap_flip_last_byte(&matrix)),
            Err(ProducerError::ReplayMismatch)
        );
    }

    #[test]
    fn matrix_is_well_formed_and_covers_every_refusal() {
        let matrix = producer_matrix();
        assert_eq!(matrix.scenario_count, LAP_SCENARIO_COUNT);
        assert_eq!(matrix.prepared_count, 1);
        assert!(matrix.cells.iter().all(|c| c.outcome != "unknown"
            && c.outcome != "refusal_missed"
            && c.outcome != "tamper_missed"
            && c.outcome != "ledger_broken"));
        // Every refusal variant is constructed in the matrix OR the write-verb production
        // path. The two write-only refusals (missing_ledger, invalid_outbox_path) are
        // exercised here by their real guards to complete A3 coverage.
        let mut constructed: Vec<String> = matrix
            .cells
            .iter()
            .filter_map(|c| c.refusal.clone())
            .collect();
        constructed.push(
            lap_ledger_present(None)
                .expect("missing ledger fires")
                .slug()
                .to_string(),
        );
        constructed.push(
            lap_path_valid("x\ny", ProducerRefusal::InvalidOutboxPath)
                .expect("invalid outbox path fires")
                .slug()
                .to_string(),
        );
        for refusal in ProducerRefusal::ALL {
            assert!(
                constructed.iter().any(|slug| slug == refusal.slug()),
                "refusal {} must be constructed",
                refusal.slug()
            );
        }
    }

    #[test]
    fn atomic_write_then_ledger_append_round_trips() {
        // Exercise the fs write path in an isolated scratch dir, then re-read the ledger.
        let base = std::env::temp_dir().join(format!("lap-test-{}", std::process::id()));
        let outbox = base.join("outbox");
        std::fs::create_dir_all(&outbox).expect("mk outbox");
        let ledger = base.join("ledger.log");
        let outbox_s = outbox.to_str().expect("outbox utf8");
        let ledger_s = ledger.to_str().expect("ledger utf8");
        // First write succeeds.
        let ok = run_producer_write(Some(outbox_s), Some(ledger_s));
        assert!(ok.is_ok(), "first write should prepare: {ok:?}");
        // The ledger now parses to exactly one entry.
        let text = std::fs::read_to_string(&ledger).expect("read ledger");
        let entries = lap_parse_ledger(&text).expect("ledger parses");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].seq, 1);
        // A second write of the SAME canonical run is a duplicate command => refused.
        let dup = run_producer_write(Some(outbox_s), Some(ledger_s));
        assert!(
            dup.is_err()
                && dup
                    .as_ref()
                    .unwrap_err()
                    .contains("duplicate_command_refused"),
            "second write must refuse duplicate: {dup:?}"
        );
        // No temp files linger, and exactly one artifact exists.
        let artifacts: Vec<_> = std::fs::read_dir(&outbox)
            .expect("read outbox")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(artifacts.iter().filter(|n| n.ends_with(".json")).count(), 1);
        assert!(artifacts.iter().all(|n| !n.ends_with(".lap-tmp")));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn target_quest_id_is_lifted_folded_and_round_trips() {
        let run = producer_demo();
        let artifact = run.artifact.expect("artifact");
        let entry = run.ledger_entry.expect("entry");
        // The demo plan targets quest 788; it is lifted verbatim to the artifact AND the
        // ledger entry, copied from the minted controller-bridge receipt (never parsed).
        assert_eq!(artifact.target_quest_id, 788);
        assert_eq!(entry.target_quest_id, 788);
        assert_eq!(
            artifact.target_quest_id,
            lap_source_run().receipt.target_quest_id
        );
        // The lifted value is folded into entry_hash: recomputing with a DIFFERENT
        // target_quest_id yields a different hash.
        let with_other = lap_entry_hash(
            entry.seq,
            entry.prev_entry_hash,
            entry.emission_seq,
            entry.command_id,
            entry.target_quest_id + 1,
            entry.taskplan_receipt_hash,
            entry.evidence_receipt_hash,
            entry.state_receipt_hash,
            entry.artifact_hash,
            &entry.decision,
            "-",
        );
        assert_ne!(with_other, entry.entry_hash);
        // And it survives the pipe-record round trip.
        let parsed = lap_parse_ledger(&format!("{}\n", lap_entry_record(&entry))).expect("parses");
        assert_eq!(parsed[0].target_quest_id, entry.target_quest_id);
    }

    #[test]
    fn target_quest_id_guard_refuses_sentinel_and_mismatch() {
        // Sentinel: a non-positive minted id (e.g. the refused-path 0) refuses.
        assert_eq!(
            lap_target_quest_id_ok(788, 0),
            Some(ProducerRefusal::TargetQuestIdInvalid)
        );
        assert_eq!(
            lap_target_quest_id_ok(-1, -1),
            Some(ProducerRefusal::TargetQuestIdInvalid)
        );
        // Mismatch: a lifted value disagreeing with the minted receipt refuses.
        assert_eq!(
            lap_target_quest_id_ok(999_999, 788),
            Some(ProducerRefusal::TargetQuestIdInvalid)
        );
        // A real, agreeing quest id passes (the production path).
        assert!(lap_target_quest_id_ok(788, 788).is_none());
    }
}
