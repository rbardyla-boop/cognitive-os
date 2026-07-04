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
    canonical_bundle, check_local_input_path, controller_bridge_demo_json,
    controller_bridge_matrix_json, corpus_admits_filename, corpus_bundle, corpus_scenario_matrix,
    corpus_scenario_pack_files, doc_bundle, doc_scenario_matrix, doc_scenario_pack_files,
    dream_export_matrix, failure_pack_files, game_evidence_demo_json, game_evidence_matrix_json,
    learner_journal_append_at, learner_journal_demo_json, learner_journal_json_at,
    learner_journal_matrix_json, learner_journal_state_json, learner_memory_demo_json,
    learner_memory_matrix_json, learner_model_demo_json, learner_model_matrix_json,
    learning_arc_demo_json, learning_arc_matrix_json, learning_session_demo_json,
    learning_session_matrix_json, list_corpus_scenarios, list_doc_scenarios,
    list_dream_export_scenarios, list_failure_cases, list_questions, list_scenarios,
    literature_intent_demo_json, literature_intent_matrix_json, resolved_path_within, run_ask,
    run_corpus_report, run_corpus_trace, run_doc_report, run_doc_trace, run_dream_export,
    run_dream_export_matrix_report, run_dream_export_matrix_verify, run_dream_export_replay,
    run_dream_export_report, run_novelty_packet, run_novelty_replay, run_novelty_report,
    run_replay, run_report, run_trace, scenario_bundle, scenario_matrix, scenario_matrix_report,
    scenario_pack_manifest, teach_map_demo_json, teach_map_matrix_json, verify_bundle,
    verify_controller_bridge_demo_json, verify_controller_bridge_matrix_json, verify_corpus_bundle,
    verify_corpus_scenario_pack, verify_doc_bundle, verify_doc_scenario_pack, verify_failure_pack,
    verify_game_evidence_demo_json, verify_game_evidence_matrix_json,
    verify_learner_journal_demo_json, verify_learner_journal_matrix_json,
    verify_learner_memory_demo_json, verify_learner_memory_matrix_json,
    verify_learner_model_demo_json, verify_learner_model_matrix_json,
    verify_learning_arc_demo_json, verify_learning_arc_matrix_json,
    verify_learning_session_demo_json, verify_learning_session_matrix_json,
    verify_literature_intent_demo_json, verify_literature_intent_matrix_json,
    verify_scenario_matrix, verify_scenario_pack, verify_teach_map_demo_json,
    verify_teach_map_matrix_json, verify_wow_state_demo_json, verify_wow_state_matrix_json,
    verify_wow_taskplan_demo_json, verify_wow_taskplan_matrix_json, wow_state_demo_json,
    wow_state_matrix_json, wow_taskplan_demo_json, wow_taskplan_matrix_json, LearnerJournalConsent,
    Scenario, BUNDLE_BOUNDARY_LINES, BUNDLE_FILES, CORPUS_BOUNDARY_LINES, CORPUS_BUNDLE_FILES,
    CORPUS_SCENARIO_BOUNDARY_LINES, CORPUS_SCENARIO_PACK_FILES, DOC_BOUNDARY_LINES,
    DOC_SCENARIO_BOUNDARY_LINES, DOC_SCENARIO_PACK_FILES, FAILURE_BOUNDARY_LINES,
    FAILURE_PACK_FILES, LEARNER_JOURNAL_DEMO_CANDIDATES, MATRIX_BOUNDARY_LINES,
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
