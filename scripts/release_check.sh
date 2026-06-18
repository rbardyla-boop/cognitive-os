#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
./scripts/lint.sh
./scripts/test.sh
./scripts/dashboard_smoke.py
test -f MVP_SCOPE.md
test -f NON_GOALS.md
test -f RISK_REGISTER.md
test -f QA_PLAN.md
test -f SPRINT_9_PLAN.md
test -f SPRINT_10_PLAN.md
test -f SPRINT_11_PLAN.md
test -f SPRINT_12_PLAN.md
test -f SPRINT_13_PLAN.md
test -f SPRINT_14_PLAN.md
test -f SPRINT_15_PLAN.md
test -f SPRINT_16_PLAN.md
test -f SPRINT_17_PLAN.md
test -f SPRINT_18_PLAN.md
test -f SPRINT_19_PLAN.md
test -f SPRINT_20_PLAN.md
test -f SPRINT_21_PLAN.md
test -f SPRINT_22_PLAN.md
test -f SPRINT_23_PLAN.md
test -f SPRINT_24_PLAN.md
test -f SPRINT_25_PLAN.md
test -f SPRINT_26_PLAN.md
test -f SPRINT_27_PLAN.md
test -f SPRINT_28_PLAN.md
test -f SPRINT_29_PLAN.md
test -f SPRINT_30_PLAN.md
test -f SPRINT_31_PLAN.md
test -f SPRINT_32_PLAN.md
test -f MUTATION_AUTHORITY.md
test -f CORRECTION_LOOPS.md
test -f CONTRADICTION_REPAIR.md
test -f EPISTEMIC_SNAPSHOT.md
test -f PLANNER_REGRET.md
test -f ATTENTION_REVIEW.md
test -f RECOVERY_REPLAY.md
test -f FAILURE_LEDGER.md
test -f DESIGN_REVIEW_NOTES.md
test -f RELEASE_REVIEW.md
test -f IMPLEMENTATION_NOTES.md
test -f MIGRATION_NOTES.md
test -f QA_REPORT.md
test -f RELEASE_NOTES.md
test -f KNOWN_LIMITATIONS.md
test -f CHANGELOG.md
# v0.1 consolidation: environment lock + frozen governance milestone.
test -f requirements.txt
test -f ENVIRONMENT.md
test -f GOVERNANCE_MILESTONE.md
grep -q 'cryptography==41.0.7' requirements.txt
grep -q 'PATH=/usr/bin' ENVIRONMENT.md
grep -q 'FROZEN' GOVERNANCE_MILESTONE.md
# ADR-002 (runtime-engine replay contract) is recorded — it was a dangling reference across the
# S28-32 plans and a.md before being written — and the architectural decision is in the charter.
test -f ADR-002-runtime-engine-replay-contract.md
grep -q 'Runtime Engine Replay Contract' ADR-002-runtime-engine-replay-contract.md
test -f docs/PROJECT_CHARTER.md
# a.md adopts the prototype-first track and its backlog sprint numbering is unambiguous: the
# delivered S31/S32 governance sprints keep their numbers (exactly one header each); the LLM and
# lifecycle backlog sprints are 31i/32i, so there are no duplicate Sprint 31/32 headers.
grep -q 'Prototype-First Track' a.md
grep -q '^## Sprint 31i ' a.md
grep -q '^## Sprint 32i ' a.md
test "$(grep -c '^## Sprint 31 ' a.md)" -eq 1
test "$(grep -c '^## Sprint 32 ' a.md)" -eq 1
# P1 — vibe-core: the ADR-002 L0 deterministic replay kernel. cargo test runs the
# determinism + kernel-boundary suite; output is silenced so release_check stays byte-silent.
cargo test --offline --quiet --manifest-path crates/vibe-core/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/vibe-core/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/vibe-core/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# no_wall_clock_in_core / no external entropy / kernel_has_no_backend_dependencies: the kernel
# source (kernel.rs) carries none of these tokens (sabotage-detectable, independent of the test).
test "$(grep -cE 'std::time|SystemTime|Instant|std::thread|thread::sleep|use rand|rand::|extern crate rand|thread_rng|getrandom|std::fs|std::net|tokio|async fn|\.await|reqwest|sqlx|rusqlite|ed25519|openssl|::ring|serde' crates/vibe-core/src/kernel.rs)" -eq 0
# vibe-core declares zero dependencies: cargo tree is exactly the crate itself (one line; also
# fails closed if cargo tree cannot run, so the no-backend proof is never vacuous).
test "$(cargo tree --offline --manifest-path crates/vibe-core/Cargo.toml --edges normal 2>/dev/null | wc -l)" -eq 1
# P2 — vibe-ingress: ADR-002 L1 admission control (ObservationEnvelope + IngressGate).
cargo test --offline --quiet --manifest-path crates/vibe-ingress/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/vibe-ingress/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/vibe-ingress/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# ingress_does_not_call_evaluate_tick + cannot touch engine state: the ingress source (gate.rs)
# references no engine type and no backend / wall-clock / entropy token (sabotage-detectable).
test "$(grep -cE 'evaluate_tick|EngineState|VibeEngine|std::fs|std::net|tokio|async fn|\.await|reqwest|sqlx|rusqlite|serde|rand::|use rand|SystemTime|Instant|std::time' crates/vibe-ingress/src/gate.rs)" -eq 0
# vibe-ingress depends ONLY on vibe-core (no backend crates): the tree is exactly two lines, one of
# which is vibe-core (fails closed if cargo tree cannot run, so the proof is never vacuous).
test "$(cargo tree --offline --manifest-path crates/vibe-ingress/Cargo.toml --edges normal 2>/dev/null | wc -l)" -eq 2
test "$(cargo tree --offline --manifest-path crates/vibe-ingress/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-core')" -eq 1
# P3 — vibe-scheduler: ADR-002 L1 deterministic tick scheduling (TickScheduler + ScheduledObservation).
cargo test --offline --quiet --manifest-path crates/vibe-scheduler/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/vibe-scheduler/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/vibe-scheduler/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# scheduler_does_not_call_evaluate_tick + no engine-state mutation + no wall-clock / backend tokens
# in the scheduler source (sabotage-detectable).
test "$(grep -cE 'evaluate_tick|EngineState|VibeEngine|std::fs|std::net|tokio|async fn|\.await|reqwest|sqlx|rusqlite|serde|rand::|use rand|SystemTime|Instant|std::time' crates/vibe-scheduler/src/scheduler.rs)" -eq 0
# vibe-scheduler depends only on workspace crates (vibe-core + vibe-ingress): no foreign/backend
# crate appears in the tree, and the root is present (fails closed if cargo tree cannot run).
test "$(cargo tree --offline --manifest-path crates/vibe-scheduler/Cargo.toml --edges normal 2>/dev/null | grep -vcE 'vibe-core|vibe-ingress|vibe-scheduler')" -eq 0
test "$(cargo tree --offline --manifest-path crates/vibe-scheduler/Cargo.toml --edges normal 2>/dev/null | grep -c 'vibe-scheduler')" -eq 1
# P4 — vibe-frame: ADR-002 L1 frame collection (FrameCollector + canonical ObservationFrame).
cargo test --offline --quiet --manifest-path crates/vibe-frame/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/vibe-frame/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/vibe-frame/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# collector_does_not_call_evaluate_tick + no engine-state mutation + no wall-clock / backend tokens.
test "$(grep -cE 'evaluate_tick|EngineState|VibeEngine|std::fs|std::net|tokio|async fn|\.await|reqwest|sqlx|rusqlite|serde|rand::|use rand|SystemTime|Instant|std::time' crates/vibe-frame/src/collector.rs)" -eq 0
# vibe-frame depends only on workspace crates: no foreign/backend crate, root present (fails closed).
test "$(cargo tree --offline --manifest-path crates/vibe-frame/Cargo.toml --edges normal 2>/dev/null | grep -vcE 'vibe-core|vibe-ingress|vibe-scheduler|vibe-frame')" -eq 0
test "$(cargo tree --offline --manifest-path crates/vibe-frame/Cargo.toml --edges normal 2>/dev/null | grep -c 'vibe-frame')" -eq 1
# P5 — VibeEngine evaluation loop: the canonical ObservationFrame is promoted into vibe-core (L0)
# and the engine consumes it. The P1/P4 cargo-test gates above already run the P5 evaluation +
# boundary tests (engine_consumes_canonical_frame, output_hash_changes_when_frame_changes, ...).
# Lock the reconciliation: EXACTLY ONE ObservationFrame definition (the L0 kernel); the L1 frame
# crate defines none (it re-exports the L0 type), so no two competing frame definitions remain.
test "$(grep -c '^pub struct ObservationFrame' crates/vibe-core/src/kernel.rs)" -eq 1
test "$(grep -rh 'struct ObservationFrame' crates/vibe-frame/src/ | wc -l)" -eq 0
# Positive single-definition signal: the L1 frame crate RE-EXPORTS the one L0 frame type (it uses
# the canonical definition rather than declaring its own). The behavioral cargo tests above
# (engine_consumes_canonical_frame, collected_frame_is_consumable_by_engine) are the authoritative
# proof that the engine consumes this exact type.
grep -q 'pub use vibe_core::{[^}]*ObservationFrame' crates/vibe-frame/src/lib.rs
grep -q 'pub struct StateTransition' crates/vibe-core/src/kernel.rs
grep -q 'fn output_hash' crates/vibe-core/src/kernel.rs
grep -q 'fn evaluate_tick(' crates/vibe-core/src/kernel.rs
# P6 — vibe-run: ADR-002 L2 deterministic record/replay (RunScript + RunRecorder + ReplayRunner).
cargo test --offline --quiet --manifest-path crates/vibe-run/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/vibe-run/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/vibe-run/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# "neither becomes a second engine": vibe-run DRIVES the engine (calls evaluate_tick) but defines
# no evaluate_tick and reimplements no engine internals (split_mix64) — sabotage-detectable.
test "$(grep -c 'fn evaluate_tick' crates/vibe-run/src/runner.rs)" -eq 0
test "$(grep -c 'split_mix64' crates/vibe-run/src/runner.rs)" -eq 0
grep -q 'evaluate_tick' crates/vibe-run/src/runner.rs
# vibe-run depends only on workspace crates: no foreign/backend crate, root present (fails closed).
test "$(cargo tree --offline --manifest-path crates/vibe-run/Cargo.toml --edges normal 2>/dev/null | grep -vcE 'vibe-core|vibe-ingress|vibe-scheduler|vibe-frame|vibe-run')" -eq 0
test "$(cargo tree --offline --manifest-path crates/vibe-run/Cargo.toml --edges normal 2>/dev/null | grep -c 'vibe-run v')" -eq 1
# P7 — vibe-cli: the local operator CLI (vibe run / replay / verify), incl. the `vibe` binary.
cargo test --offline --quiet --manifest-path crates/vibe-cli/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/vibe-cli/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/vibe-cli/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
cargo build --offline --quiet --manifest-path crates/vibe-cli/Cargo.toml >/dev/null 2>&1
test -f crates/vibe-cli/src/main.rs
# serde is confined to the CLI (IO layer): it must NOT appear in any engine crate's manifest, so
# the deterministic engine stays dependency-free.
test "$(grep -lE '^serde' crates/vibe-core/Cargo.toml crates/vibe-ingress/Cargo.toml crates/vibe-scheduler/Cargo.toml crates/vibe-frame/Cargo.toml crates/vibe-run/Cargo.toml 2>/dev/null | wc -l)" -eq 0
# the CLI re-derives runs through vibe-run; it reimplements no engine internals.
test "$(grep -cE 'fn evaluate_tick|split_mix64' crates/vibe-cli/src/lib.rs)" -eq 0
# ---------------------------------------------------------------------------------------------------
# P8 — Prototype Release Gate. The checks above already run the P1-P7 Rust suite, the Python
# governance gates, serde confinement, and the dependency boundaries. P8 consolidates the proof
# surface by adding (a) an end-to-end CLI binary smoke that exercises replay determinism through the
# recorded-run path and proves tamper is rejected, and (b) a no-secrets scan. No engine behavior is
# added here.
# ---------------------------------------------------------------------------------------------------
# (a) CLI binary smoke: vibe run -> replay (MATCH) -> verify (authentic) -> a tampered run MUST fail.
cargo build --offline --quiet --manifest-path crates/vibe-cli/Cargo.toml >/dev/null 2>&1
_p8_dir="$(mktemp -d)"
cat > "$_p8_dir/scenario.json" <<'P8_SCENARIO'
{ "schema":"vibe-scenario-v1","seed":7,"scheduler":{"horizon":10,"max_per_tick":8},"now":0,"run_ticks":3,
  "observations":[
    {"event_id":1,"source":"s","session":1,"source_sequence":0,"target_tick":1,"signal_micros":10000000},
    {"event_id":2,"source":"s","session":1,"source_sequence":1,"target_tick":2,"signal_micros":20000000} ] }
P8_SCENARIO
./target/debug/vibe run "$_p8_dir/scenario.json" "$_p8_dir/run.json" >/dev/null 2>&1
./target/debug/vibe replay "$_p8_dir/run.json" >/dev/null 2>&1
./target/debug/vibe verify "$_p8_dir/run.json" >/dev/null 2>&1
sed -i 's/10000000/11111111/' "$_p8_dir/run.json"
if ./target/debug/vibe verify "$_p8_dir/run.json" >/dev/null 2>&1; then rm -rf "$_p8_dir"; exit 1; fi
rm -rf "$_p8_dir"
# (b) No-secrets scan: no committed secret files anywhere (excl. build/.git), and no key/credential
# material in the Rust tree. A planted .env/key/credential fixture fails the gate.
test "$(find . -type f \( -name '.env' -o -name '*.pem' -o -name '*.key' -o -name '*.p8' -o -name 'id_rsa' -o -name 'id_ed25519' \) -not -path './target/*' -not -path './.git/*' 2>/dev/null | wc -l)" -eq 0
test "$(grep -rlIE 'BEGIN [A-Z ]*PRIVATE KEY|AKIA[0-9A-Z]{16}|aws_secret_access_key' crates --include='*.rs' --include='*.toml' 2>/dev/null | wc -l)" -eq 0
# ---------------------------------------------------------------------------------------------------
# READ-0 — reading substrate (a SEPARATE track from the vibe engine; it must not contaminate the
# engine crates). A deterministic scripted reader treats external text as an addressable environment
# and builds source-linked structured memory; the verifier gates grounding, answer support, and trace
# replay. No trained weights. (Runs alongside the P8 gate; P8's engine checks are unaffected.)
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-substrate/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-substrate/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-substrate/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Separation: reading-substrate depends on NO vibe engine crate, and is zero-dependency.
test "$(cargo tree --offline --manifest-path crates/reading-substrate/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-substrate/Cargo.toml --edges normal 2>/dev/null | wc -l)" -eq 1
# READ-1 claim fidelity: the verifier reads cited span TEXT (not just span ids) and a claim is grounded
# only if its statement is literally supported by that text. The cargo test above runs the fidelity
# probe (fabricated claim citing a real, read span -> grounding fails); this positive signal asserts the
# verifier actually consults span text, so it cannot silently degrade to a structural id-only check.
grep -q 'read_span' crates/reading-substrate/src/verify.rs
grep -q 'fn normalize' crates/reading-substrate/src/verify.rs
# READ-2 sentence fidelity: a claim must be a complete sentence-level unit of a cited span, not an
# arbitrary verbatim fragment (kills fragment + cross-fragment-composition false-accepts). The cargo
# test above runs the fragment/negation probes; this positive signal asserts the sentence-boundary
# check exists so it cannot silently degrade back to a plain substring match.
grep -q 'fn sentence_aligned' crates/reading-substrate/src/verify.rs
# READ-5 — deterministic sentence-splitter hardening: the shared splitter recognises abbreviations /
# decimals / versions / initials so they do not mis-split, WITHOUT any semantics or model. The substrate
# corpus tests (abbreviation_us_does_not_split, decimals_and_versions_do_not_split, ...) above are the
# load-bearing checks; this positive signal asserts the deterministic boundary logic exists so the
# splitter cannot silently revert to naive period-splitting.
grep -q 'fn is_period_boundary' crates/reading-substrate/src/corpus.rs
# ---------------------------------------------------------------------------------------------------
# P9 — reading-codec: the untrained LLM codec boundary + eval harness. A strict, deterministic codec
# parses UNTRUSTED model output into typed reading actions, validates them, executes accepted actions
# ONLY through the READ-0 substrate, and finalizes an answer ONLY if the verifier approves it. No
# trained weights, no live model — model output is untrusted text (fixtures). The codec is the
# boundary/IO layer for the reading track (serde allowed here, never in the engine or substrate core).
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-codec/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-codec/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-codec/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# The eval harness is a runnable gate: it scores the 10-fixture battery and exits non-zero on any
# divergence between the codec's actual decision and the required decision.
cargo run --offline --quiet --example eval_report -p reading-codec >/dev/null 2>&1
# The substrate is the ONLY executor: the codec DRIVES reading_substrate::execute/verify and defines
# no executor/verifier/hashing internals of its own (sabotage-detectable, like the vibe-run gate).
test "$(grep -rlE 'fn execute\(|fn verify\(|fn hash_memory|fn hash_proof|fn require_grounded' crates/reading-codec/src/ | wc -l)" -eq 0
grep -q 'execute(corpus' crates/reading-codec/src/codec.rs
grep -q 'verify(corpus' crates/reading-codec/src/codec.rs
# Determinism: the decode decision is pure — no wall-clock, no entropy, no network in the codec source.
test "$(grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|tokio|\.await|reqwest' crates/reading-codec/src/ | wc -l)" -eq 0
# No model is trained or loaded: the codec manifest pulls no ML/inference/training framework.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-codec/Cargo.toml | wc -l)" -eq 0
# Separation: reading-codec depends on the reading-substrate (its only executor) and on NO vibe engine
# crate. (serde/serde_json are allowed boundary deps; fails closed if cargo tree cannot run.)
test "$(cargo tree --offline --manifest-path crates/reading-codec/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-codec/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-substrate')" -ge 1
# ---------------------------------------------------------------------------------------------------
# P10 — reading-adapter: the baseline local LLM adapter. A REPLACEABLE model backend proposes untrusted
# reading-action text routed ONLY through reading_codec::decode (validate -> substrate execute -> READ-1
# verifier finalize). The default scripted backend is deterministic; the optional `local-model` feature
# (a real local model via std::process) is OFF by default and never RUN by the gate (only compiled +
# linted), so release_check stays offline + deterministic. No training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-adapter/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-adapter/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-adapter/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# The optional real-model backend must still compile + lint clean (it is never executed by the gate).
cargo clippy --offline --manifest-path crates/reading-adapter/Cargo.toml --features local-model -- -D warnings >/dev/null 2>&1
# The runnable baseline eval records the failure profile AND fails if the safety boundary breaks (a
# verbatim grounded sequence must finalize; a fabricated-but-cited claim must be rejected Unverified).
cargo run --offline --quiet --example baseline_report -p reading-adapter >/dev/null 2>&1
# The adapter routes untrusted model text ONLY through reading_codec::decode; it calls no substrate
# executor / verifier / finalizer directly (sabotage-detectable).
grep -q 'reading_codec::decode' crates/reading-adapter/src/adapter.rs
grep -q 'decode(' crates/reading-adapter/src/adapter.rs
test "$(grep -rlE 'execute\(|verify\(|finalize\(' crates/reading-adapter/src/ | wc -l)" -eq 0
# Determinism/purity of the DEFAULT (baseline) adapter path: no clock, entropy, network, or process
# spawning. The real-model backend's std::process is isolated to the feature-gated local_backend.rs.
test "$(grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|std::process|tokio|\.await|reqwest' crates/reading-adapter/src/ --include='*.rs' --exclude='local_backend.rs' | wc -l)" -eq 0
# The real-model backend is feature-gated OFF by default, so the default build/gate never spawns it.
grep -q 'cfg(feature = "local-model")' crates/reading-adapter/src/lib.rs
# No model is trained or loaded: the adapter manifest pulls no ML/inference/training framework.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-adapter/Cargo.toml | wc -l)" -eq 0
# Separation: reading-adapter depends on reading-codec (its only path to the substrate) and on NO vibe
# engine crate. (fails closed if cargo tree cannot run.)
test "$(cargo tree --offline --manifest-path crates/reading-adapter/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-adapter/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-codec')" -ge 1
# ---------------------------------------------------------------------------------------------------
# P11 — reading-eval: the codec eval harness. 30+ committed fixtures (raw untrusted proposal text + a
# COMMITTED expected outcome) scored through the P10 adapter; the model never self-grades. The unsafe
# class — false-accepts (a should-reject output that got accepted/finalized) — is surfaced explicitly
# and MUST be zero; false-rejects are allowed but classified by cause. Deterministic; no model, no
# training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-eval/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-eval/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-eval/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# The runnable harness enforces the acceptance targets: >= 30 fixtures and 0 false-accepts (else exit 1).
cargo run --offline --quiet --example eval_report -p reading-eval >/dev/null 2>&1
# >= 30 committed fixtures (source-level floor, independent of the cargo test).
test "$(grep -c 'EvalCase {' crates/reading-eval/src/fixtures.rs)" -ge 30
# Determinism/purity: the scorer is pure — no clock, entropy, network, or process spawning in the eval.
test "$(grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|std::process|tokio|\.await|reqwest' crates/reading-eval/src/ | wc -l)" -eq 0
# No training/ML/inference dependency in the eval manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-eval/Cargo.toml | wc -l)" -eq 0
# Separation: reading-eval depends on the reading track (adapter→codec→substrate) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-eval/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-eval/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-adapter')" -ge 1
# ---------------------------------------------------------------------------------------------------
# P12 — reading-train-gate: the training-justification gate. A deterministic, machine-checkable decision
# that BLOCKS weight training unless a clean, recurring model failure survives cleanup of every fixable
# cause (fixture/schema/prompt/tooling/context/verifier). No failed cases → no training; any false-accept
# → a verifier/safety fix, never training. On the current P11 battery (0 false-accepts, 0 residual) the
# decision is training_justified=false. No training, no ML dependency.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-train-gate/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-train-gate/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-train-gate/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# The runnable decision is internally consistent (never "train" without citing a clean recurring failure).
cargo run --offline --quiet --example decision_report -p reading-train-gate >/dev/null 2>&1
# Determinism/purity: the decision is pure — no clock, entropy, network, or process in the gate source.
test "$(grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|std::process|tokio|\.await|reqwest' crates/reading-train-gate/src/ | wc -l)" -eq 0
# No model is trained or loaded: the gate manifest pulls no ML/inference/training framework.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-train-gate/Cargo.toml | wc -l)" -eq 0
# Separation: reading-train-gate depends on reading-eval (the harness it gates on) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-train-gate/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-train-gate/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-eval')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-3 — reading-cli (read0): the real-corpus reading CLI. Loads a folder of documents into a corpus of
# one sentence per span (shared splitter), runs an UNTRUSTED reading plan ONLY through reading_codec::decode
# (validate -> substrate execute -> READ-1/READ-2 verifier finalize), and emits a replayable run + proof +
# verifier receipt; verify/replay re-derive from the run file and reject tamper. The plan never reaches
# memory except via the codec; read0 calls no substrate executor directly. (serde is the IO layer here.)
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-cli/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-cli/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-cli/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
cargo build --offline --quiet --manifest-path crates/reading-cli/Cargo.toml >/dev/null 2>&1
test -f crates/reading-cli/src/main.rs
# read0_uses_codec_only: the untrusted plan is routed through reading_codec::decode; read0 calls no
# substrate executor directly (sabotage-detectable). It MAY call the verifier for the receipt.
grep -q 'decode(' crates/reading-cli/src/lib.rs
test "$(grep -rlE 'execute\(' crates/reading-cli/src/ | wc -l)" -eq 0
# No model/training dependency in the CLI manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-cli/Cargo.toml | wc -l)" -eq 0
# Separation: read0 depends on reading-codec + reading-substrate and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-cli/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-cli/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-codec')" -ge 1
# End-to-end read0 binary smoke (deterministic, offline): build a corpus from a temp folder, run a
# sentence-grounded plan -> verify (pass) -> replay (match); a fragment/fabricated plan MUST be rejected;
# a tampered run MUST fail verify.
_read3_dir="$(mktemp -d)"
mkdir -p "$_read3_dir/docs"
printf 'Bridge A was structurally damaged. Bridge B stayed open during the storm.' > "$_read3_dir/docs/report.txt"
cat > "$_read3_dir/plan.json" <<'READ3_PLAN'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":1},{"action":"extract_claim","statement":"Bridge B stayed open during the storm.","source_span_ids":[1]},{"action":"synthesize","answer_text":"Bridge B stayed open during the storm.","supporting_claims":[0]}]
READ3_PLAN
./target/debug/read0 run "$_read3_dir/docs" "Which bridge is open?" "$_read3_dir/plan.json" "$_read3_dir/out.json" >/dev/null 2>&1
./target/debug/read0 verify "$_read3_dir/out.json" >/dev/null 2>&1
./target/debug/read0 replay "$_read3_dir/out.json" >/dev/null 2>&1
cat > "$_read3_dir/bad.json" <<'READ3_BAD'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":0},{"action":"extract_claim","statement":"Bridge A","source_span_ids":[0]},{"action":"synthesize","answer_text":"Bridge A","supporting_claims":[0]}]
READ3_BAD
if ./target/debug/read0 run "$_read3_dir/docs" "Which bridge is open?" "$_read3_dir/bad.json" "$_read3_dir/bad_out.json" >/dev/null 2>&1; then rm -rf "$_read3_dir"; exit 1; fi
sed -i 's/"answer_hash": [0-9]*/"answer_hash": 0/' "$_read3_dir/out.json"
if ./target/debug/read0 verify "$_read3_dir/out.json" >/dev/null 2>&1; then rm -rf "$_read3_dir"; exit 1; fi
rm -rf "$_read3_dir"
# ---------------------------------------------------------------------------------------------------
# READ-4 — reading-corpus-eval: the real-corpus eval pack. >= 10 committed fixtures (docs folder +
# question + plan + expected verifier result) each driven through the REAL read0 run -> verify -> replay
# path. A false-grounded answer (an expected-rejected fixture that finalized a verified answer) is the
# unsafe class and MUST be zero. Expected labels are committed in source, never inferred from a model.
# No training: anecdotal failures here never justify weights (the P12 gate decides that).
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-corpus-eval/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-corpus-eval/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-corpus-eval/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Runnable pack: drives every fixture through run/verify/replay; exits non-zero on < 10 fixtures or any
# false-grounded answer.
cargo run --offline --quiet --example pack_report -p reading-corpus-eval >/dev/null 2>&1
# >= 10 committed fixtures (source-level floor, independent of the cargo test).
test "$(grep -c 'CorpusFixture {' crates/reading-corpus-eval/src/pack.rs)" -ge 10
# No model/training dependency in the manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-corpus-eval/Cargo.toml | wc -l)" -eq 0
# Separation: depends on reading-cli (the read0 pipeline it measures) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-corpus-eval/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-corpus-eval/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-cli')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-6 — reading-autonomy: reader autonomy v0. A DETERMINISTIC, BOUNDED reader proposes a reading plan
# from corpus METADATA (not all text) and routes every action ONLY through reading_codec::decode (validate
# -> substrate execute -> READ-1/READ-2 verifier finalize). It holds no executor/verifier handle, cannot
# finalize on its own, and is bounded by max steps / spans / finalize attempts. Autonomy proposes; the
# codec validates; the verifier authorizes. No model, no training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-autonomy/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-autonomy/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-autonomy/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Runnable: the bounded autonomous read must finalize a verifier-authorized answer (else exit non-zero).
cargo run --offline --quiet --example autonomous_read -p reading-autonomy >/dev/null 2>&1
# Autonomy proposes ONLY through the codec: the reader routes the plan through reading_codec::decode and
# calls no substrate executor / verifier directly (sabotage-detectable).
grep -q 'decode(' crates/reading-autonomy/src/reader.rs
test "$(grep -rlE 'execute\(|verify\(' crates/reading-autonomy/src/ | wc -l)" -eq 0
# Bounded by construction: the bounds struct exists (the reader can never run unbounded).
grep -q 'struct ReaderBounds' crates/reading-autonomy/src/reader.rs
# No model/training dependency in the manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-autonomy/Cargo.toml | wc -l)" -eq 0
# Separation: reading-autonomy depends on reading-codec (its only path to the substrate) and NO vibe crate.
test "$(cargo tree --offline --manifest-path crates/reading-autonomy/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-autonomy/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-codec')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-7 — reading-autonomous-eval: autonomous corpus eval pack. Drives the deterministic READ-6 reader
# across the READ-4 corpus fixtures (NO hand-written plans), INDEPENDENTLY re-verifies every finalized
# answer, and compares the manual-plan score to the autonomous-reader score. 0 false-grounded REQUIRED;
# false-rejects allowed but classified. Autonomy underperformance is an engineering signal, NOT a training
# justification (P12 still owns weights). No model, no training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-autonomous-eval/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-autonomous-eval/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-autonomous-eval/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Runnable: the autonomous pack must report 0 false-grounded (exits non-zero otherwise).
cargo run --offline --quiet --example autonomous_pack_report -p reading-autonomous-eval >/dev/null 2>&1
# The scorer drives the AUTONOMOUS reader and NEVER the fixture's hand-written plan.
grep -q 'use reading_autonomy' crates/reading-autonomous-eval/src/scorer.rs
test "$(grep -c 'fixture\.plan' crates/reading-autonomous-eval/src/scorer.rs)" -eq 0
# Independent re-verification of every finalized answer (false-grounded is MEASURED, not assumed).
grep -q 'verify(' crates/reading-autonomous-eval/src/scorer.rs
# No model/training dependency in the manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-autonomous-eval/Cargo.toml | wc -l)" -eq 0
# Separation: depends on reading-autonomy (the measured reader) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-autonomous-eval/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-autonomous-eval/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-autonomy')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-8 — budgeted autonomous span selection. reading_autonomy::read_budgeted makes the reader SELECTIVE
# (claims only spans LEXICALLY relevant to the question — deterministic word-prefix overlap + a fixed
# stopword list, NO model / semantics / entailment / paraphrase) while still metadata-first, budget-bounded,
# and routed ONLY through the codec (the codec-only scan over reading-autonomy/src above covers budgeted.rs).
# reading-budgeted-eval measures it vs the blunt reader, classifies coverage misses, and keeps 0
# false-grounded (cross-validated). The blunt read() is unchanged, so READ-7 stays green. No training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-budgeted-eval/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-budgeted-eval/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-budgeted-eval/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Runnable: the budgeted pack must report 0 false-grounded (exits non-zero otherwise).
cargo run --offline --quiet --example budgeted_pack_report -p reading-budgeted-eval >/dev/null 2>&1
# The budgeted reader exists and routes its proposed plan through the codec.
grep -q 'pub fn read_budgeted' crates/reading-autonomy/src/budgeted.rs
grep -q 'decode(' crates/reading-autonomy/src/budgeted.rs
# Deterministic LEXICAL selection (positive signals — word-prefix overlap + content-term tokenizer).
grep -q 'fn prefix_overlap' crates/reading-autonomy/src/budgeted.rs
grep -q 'fn content_terms' crates/reading-autonomy/src/budgeted.rs
# No model/training dependency in the eval manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-budgeted-eval/Cargo.toml | wc -l)" -eq 0
# Separation: depends on reading-autonomy (the readers it compares) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-budgeted-eval/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-budgeted-eval/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-autonomy')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-9 — title-aware deterministic relevance ranking. reading_autonomy::read_ranked orders the
# budgeted reader's span reads by DETERMINISTIC title relevance (document TITLE vs question, the same
# lexical word-prefix overlap as READ-8 — NO model / semantics / entailment / paraphrase, and NEVER a
# span-text preview before read_span), so under a tight budget a title-relevant document is reached
# first instead of missed. The claim FILTER is unchanged (a span is claimed only if its OWN text is
# relevant AND grounds verbatim through the codec), so a title match alone can never fabricate support.
# reading-ranked-eval proves no-regression vs read_budgeted on the committed pack, classifies coverage
# misses, keeps 0 false-grounded (cross-validated), and measures the tight-budget recovery + file-order
# stability the ranking buys. read_budgeted/read are behaviour-unchanged, so READ-7/READ-8 stay green.
# No training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-ranked-eval/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-ranked-eval/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-ranked-eval/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Runnable: the ranked pack must report 0 false-grounded AND 0 regressions vs budgeted, and the
# title-priority demo must show the recovery (exits non-zero otherwise).
cargo run --offline --quiet --example ranked_pack_report -p reading-ranked-eval >/dev/null 2>&1
# The ranked reader exists and reuses the shared budgeted/codec core (no second executor/verifier — the
# READ-6 codec-only scan over reading-autonomy/src above also covers ranked.rs).
grep -q 'pub fn read_ranked' crates/reading-autonomy/src/ranked.rs
grep -q 'read_selecting' crates/reading-autonomy/src/ranked.rs
# Title ranking is METADATA-ONLY (positive signals): it scores the document TITLE and orders reads by it.
grep -q 'fn title_relevance' crates/reading-autonomy/src/ranked.rs
grep -q 'fn title_ranked_order' crates/reading-autonomy/src/ranked.rs
# It NEVER previews span text for ordering: ranked.rs calls no read_span / .text() (those live only in
# the shared budget loop in budgeted.rs). A "rank by full-text preview" regression trips this.
test "$(grep -cE 'read_span|\.text\(\)' crates/reading-autonomy/src/ranked.rs)" -eq 0
# No model/training dependency in the eval manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-ranked-eval/Cargo.toml | wc -l)" -eq 0
# Separation: depends on reading-autonomy (the readers it compares) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-ranked-eval/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-ranked-eval/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-autonomy')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-10 — section-aware / multi-term deterministic relevance ranking. The substrate gains
# heading-labelled SECTIONS as METADATA (SectionMeta + add_document_with_sections; a heading is NEVER a
# span, so no claim can cite one). reading_autonomy::read_section_ranked orders the budgeted reader's
# span reads by combined TITLE + section-HEADING relevance, counting DISTINCT matched question terms
# (multi-term), so under a tight budget the most relevant section is reached first. Metadata-only (no
# span-text preview before read), no model/semantics/entailment/paraphrase, and the ranking score never
# becomes evidence (section.rs builds no claim/answer). reading-section-eval proves no-regression vs
# read_budgeted on the flat committed pack, classifies coverage misses, keeps 0 false-grounded
# (cross-validated), and measures the section-heading + multi-term recovery on constructed corpora.
# read/read_budgeted/read_ranked are behaviour-unchanged so READ-7/8/9 stay green. No training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/reading-section-eval/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/reading-section-eval/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/reading-section-eval/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Runnable: the section pack must report 0 false-grounded AND 0 regressions, and the demo must show the
# section-heading + multi-term win (exits non-zero otherwise).
cargo run --offline --quiet --example section_pack_report -p reading-section-eval >/dev/null 2>&1
# The substrate exposes section/heading METADATA (positive signals); the headings-are-never-spans test
# (sectioned_document_exposes_headings_as_metadata_never_as_spans) in the READ-0 substrate suite above
# is the load-bearing proof that a heading cannot be cited or grounded.
grep -q 'pub struct SectionMeta' crates/reading-substrate/src/corpus.rs
grep -q 'fn add_document_with_sections' crates/reading-substrate/src/corpus.rs
# The section reader exists and reuses the shared budgeted/codec core (no second executor/verifier — the
# READ-6 codec-only scan over reading-autonomy/src above also covers section.rs).
grep -q 'pub fn read_section_ranked' crates/reading-autonomy/src/section.rs
grep -q 'read_selecting' crates/reading-autonomy/src/section.rs
# Multi-term, section-aware ranking (positive signals): scores TITLE + HEADING by matched query terms.
grep -q 'fn section_ranked_order' crates/reading-autonomy/src/section.rs
grep -q 'fn combined_relevance' crates/reading-autonomy/src/section.rs
# Metadata-only: the ranker never previews span text (no read_span / .text() in section.rs)...
test "$(grep -cE 'read_span|\.text\(\)' crates/reading-autonomy/src/section.rs)" -eq 0
# ...and the ranking score never becomes evidence — the ranker builds no claim/answer (those live only
# in the shared budget loop). A "ranking confidence becomes evidence" regression trips this.
test "$(grep -cE 'extract_claim|synthesize|answer_text' crates/reading-autonomy/src/section.rs)" -eq 0
# No model/training dependency in the eval manifest.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/reading-section-eval/Cargo.toml | wc -l)" -eq 0
# Separation: depends on reading-autonomy (the readers it compares) and NO vibe engine crate.
test "$(cargo tree --offline --manifest-path crates/reading-section-eval/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/reading-section-eval/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-autonomy')" -ge 1
# ---------------------------------------------------------------------------------------------------
# READ-11 — real document section metadata ingestion. read0's corpus loader (reading-cli/corpus_load.rs)
# detects Markdown ATX headings (`# `/`## `/`### ` … up to 6, strict: hashes then whitespace then text)
# DETERMINISTICALLY and routes the body sentences through add_document_with_sections, so a heading becomes
# SectionMeta (metadata) and is NEVER split into a span — it has no SpanId and can never be cited or
# grounded. A headingless file is one default section, byte-identical to the flat build; produce_run stores
# the corpus's actual spans (heading-free) so verify/replay stay consistent. The reading-cli suite
# (markdown_heading_becomes_section_metadata, heading_is_not_a_span, sentence_under_heading_gets_section_id,
# unheaded_file_gets_default_section, non_atx_hash_lines_are_body_not_headings, claim_citing_heading_is_rejected,
# misleading_heading_without_body_support_cannot_finalize, headed_document_runs_verifies_and_replays) and the
# reading-section-eval test (section_ranked_read0_recovers_heading_relevant_answer) above are the load-bearing
# checks. No semantic heading inference, no all-caps guessing, no layout inference, no model, no training.
# ---------------------------------------------------------------------------------------------------
# The deterministic ATX heading detector + section parser exist (positive signals)...
grep -q 'fn parse_atx_heading' crates/reading-cli/src/corpus_load.rs
grep -q 'fn parse_sections' crates/reading-cli/src/corpus_load.rs
# ...and headings route ONLY through the section metadata API (so they become metadata, never spans).
grep -q 'add_document_with_sections' crates/reading-cli/src/corpus_load.rs
# produce_run stores the corpus's BUILT spans (heading-free), not a re-split of raw content: it reads
# them back from the corpus by span id (sabotage-detectable — reverting to split_sentences(content) would
# restore headings as spans and drop this token).
grep -q 'metadata span ids exist in the corpus' crates/reading-cli/src/lib.rs
# End-to-end headed-document binary smoke (comment-immune, behavioral): a real Markdown file with an ATX
# heading runs through the read0 BINARY; run -> verify -> replay must all pass, AND the heading text must
# NOT appear anywhere in the run file (a heading is metadata, never a stored span). This gates the exact
# regression a panel raised — reverting produce_run to re-split raw content would leak "# Wind Forecast"
# into a stored span and this grep would catch it, independent of any source token.
_read11_dir="$(mktemp -d)"
mkdir -p "$_read11_dir/docs"
printf '# Wind Forecast\nWinds will reach forty miles per hour.' > "$_read11_dir/docs/forecast.txt"
cat > "$_read11_dir/plan.json" <<'READ11_PLAN'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":0},{"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[0]},{"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}]
READ11_PLAN
./target/debug/read0 run "$_read11_dir/docs" "What is the wind forecast?" "$_read11_dir/plan.json" "$_read11_dir/out.json" >/dev/null 2>&1
./target/debug/read0 verify "$_read11_dir/out.json" >/dev/null 2>&1
./target/debug/read0 replay "$_read11_dir/out.json" >/dev/null 2>&1
# A heading is metadata: READ-12 persists it as a section heading ("Wind Forecast", no '#'), but the
# ATX heading LINE ("# Wind Forecast") must never appear as a stored span. So an "# Wind Forecast" token
# (with the hash) anywhere in the run file means a heading leaked into a span.
if grep -q '# Wind Forecast' "$_read11_dir/out.json"; then rm -rf "$_read11_dir"; exit 1; fi
rm -rf "$_read11_dir"
# ---------------------------------------------------------------------------------------------------
# READ-12 — persist section metadata in run receipts. The run file now carries each document's
# heading-labelled SECTIONS (DocumentDto.sections: a heading + a span COUNT — never a span), so
# section-aware autonomy can operate over a real read0 output without rebuilding a different structure.
# `spans` stays the canonical span-id source, so grounding/hash/tamper checks keep full strength. The
# shared `rebuild_corpus` (verify/replay + section consumers) rejects HEADING-AS-SPAN tamper (no stored
# span is an ATX heading) and SECTION/BODY-MISMATCH tamper (section counts must partition the body), and
# reconstructs the SAME sections the run built. Headings rank reads, never ground claims. The reading-cli
# suite (run_receipt_includes_section_metadata, rebuild_corpus_reconstructs_the_run_sections,
# heading_as_span_tamper_is_rejected, section_body_mismatch_tamper_is_rejected, headingless_document_round_trips_under_v2,
# span_text_tamper_still_caught_under_v2) and the reading-section-eval test (section_ranked_read0_uses_persisted_metadata)
# above are the load-bearing checks. Schema/receipt hardening only — no model, no training.
# ---------------------------------------------------------------------------------------------------
# The receipt schema is v2 and carries the section structure + the tamper-checking rebuild (signals).
grep -q '"read0-run-v2"' crates/reading-cli/src/lib.rs
grep -q 'struct SectionDto' crates/reading-cli/src/lib.rs
grep -q 'pub fn rebuild_corpus' crates/reading-cli/src/lib.rs
grep -q 'fn corpus_from_sections' crates/reading-cli/src/corpus_load.rs
# rebuild_corpus enforces the heading-as-span check (a stored span that is an ATX heading is tamper).
grep -q 'parse_atx_heading' crates/reading-cli/src/lib.rs
# End-to-end receipt-tamper binary smoke: a headed document's receipt carries section metadata and
# verifies; injecting an ATX heading as a body span OR corrupting the section partition MUST be rejected.
_read12_dir="$(mktemp -d)"
mkdir -p "$_read12_dir/docs"
printf '# Overview\nThe bridge is open.\n## Wind Forecast\nWinds will reach forty miles per hour.' > "$_read12_dir/docs/forecast.txt"
cat > "$_read12_dir/plan.json" <<'READ12_PLAN'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":1},{"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},{"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}]
READ12_PLAN
./target/debug/read0 run "$_read12_dir/docs" "What is the wind forecast?" "$_read12_dir/plan.json" "$_read12_dir/out.json" >/dev/null 2>&1
./target/debug/read0 verify "$_read12_dir/out.json" >/dev/null 2>&1
# the receipt persists the section heading (metadata) and a span count partition.
grep -q '"heading": "Wind Forecast"' "$_read12_dir/out.json"
grep -q '"span_count"' "$_read12_dir/out.json"
# TAMPER 1 — inject an ATX heading as the (uncited) first body span: verify MUST reject (heading-as-span).
sed 's/"The bridge is open."/"# Injected Heading"/' "$_read12_dir/out.json" > "$_read12_dir/tamper_heading.json"
if ./target/debug/read0 verify "$_read12_dir/tamper_heading.json" >/dev/null 2>&1; then rm -rf "$_read12_dir"; exit 1; fi
# TAMPER 2 — corrupt the section span counts so they no longer partition the body: verify MUST reject.
sed 's/"span_count": [0-9]*/"span_count": 9/' "$_read12_dir/out.json" > "$_read12_dir/tamper_count.json"
if ./target/debug/read0 verify "$_read12_dir/tamper_count.json" >/dev/null 2>&1; then rm -rf "$_read12_dir"; exit 1; fi
# TAMPER 3 — a usize::MAX section count (overflow attempt): verify MUST reject GRACEFULLY (no panic on a
# crafted receipt). Caught a panel finding: a plain sum could be wrapped past; the checked partition
# returns Tamper instead of an out-of-bounds panic.
sed 's/"span_count": [0-9]*/"span_count": 18446744073709551615/' "$_read12_dir/out.json" > "$_read12_dir/tamper_overflow.json"
if ./target/debug/read0 verify "$_read12_dir/tamper_overflow.json" >/dev/null 2>"$_read12_dir/of.err"; then rm -rf "$_read12_dir"; exit 1; fi
if grep -qi panic "$_read12_dir/of.err"; then rm -rf "$_read12_dir"; exit 1; fi
rm -rf "$_read12_dir"
# ---------------------------------------------------------------------------------------------------
# READ-13 — explicit receipt schema versioning / migration discipline. verify/replay now recognize the
# schema tag (`read0-run-v1` / `read0-run-v2`) explicitly and require it to AGREE with the receipt's
# content, so version handling is deterministic and the tag can never weaken tamper detection. An old v1
# receipt (no section metadata) MIGRATES forward to one default empty-heading section over all spans (the
# flat rebuild reproduces the same span ids + hashes, so it still verifies/replays). A v2 receipt MUST
# carry its sections — stripping them is rejected (closing the READ-12 hole where empty sections silently
# fell back to flat and still verified, so section metadata could DISAPPEAR unnoticed). A v1 tag wearing
# v2 sections is ambiguous and rejected; an unknown schema version is refused cleanly without panic. The
# schema tag governs STRUCTURE only, never evidence authority. The reading-cli suite
# (v1_headingless_receipt_migrates_and_verifies, v1_receipt_carrying_sections_is_rejected,
# v2_receipt_with_dropped_sections_is_rejected, unknown_schema_is_rejected) is the load-bearing check.
# Schema/receipt hardening only — no model, no training.
# ---------------------------------------------------------------------------------------------------
# Version discipline is present: an explicit version enum, the v1 tag, the unsupported-schema refusal,
# and the checked section partition (signals).
grep -q 'enum SchemaVersion' crates/reading-cli/src/lib.rs
grep -q 'UnsupportedSchema' crates/reading-cli/src/lib.rs
grep -q 'read0-run-v1' crates/reading-cli/src/lib.rs
grep -q 'fn partition_sections' crates/reading-cli/src/lib.rs
# End-to-end schema-version binary smoke: read0 writes a v3 receipt (READ-14); a faithful v1 migration of
# it verifies; and every tag/content mismatch (v2 dropped sections, v1-with-sections, unknown version) is
# rejected. Each legacy variant is built FAITHFULLY (a pre-v3 tag carries no structure hash).
_read13_dir="$(mktemp -d)"
mkdir -p "$_read13_dir/docs"
printf '# Overview\nThe bridge is open.\n## Wind Forecast\nWinds will reach forty miles per hour.' > "$_read13_dir/docs/forecast.txt"
cat > "$_read13_dir/plan.json" <<'READ13_PLAN'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":1},{"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},{"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}]
READ13_PLAN
./target/debug/read0 run "$_read13_dir/docs" "What is the wind forecast?" "$_read13_dir/plan.json" "$_read13_dir/out.json" >/dev/null 2>&1
./target/debug/read0 verify "$_read13_dir/out.json" >/dev/null 2>&1
# MIGRATE — a faithful old v1 receipt (tag read0-run-v1, NO sections, NO structure hash) MUST verify.
python3 -c "import json; d=json.load(open('$_read13_dir/out.json')); d['schema']='read0-run-v1'; d.pop('structure_hash',None); [doc.pop('sections',None) for doc in d['documents']]; json.dump(d,open('$_read13_dir/v1.json','w'))"
./target/debug/read0 verify "$_read13_dir/v1.json" >/dev/null 2>&1
# TAMPER A — a faithful v2 receipt with its sections DROPPED MUST be rejected (sections cannot silently vanish).
python3 -c "import json; d=json.load(open('$_read13_dir/out.json')); d['schema']='read0-run-v2'; d.pop('structure_hash',None); [doc.update(sections=[]) for doc in d['documents']]; json.dump(d,open('$_read13_dir/drop.json','w'))"
if ./target/debug/read0 verify "$_read13_dir/drop.json" >/dev/null 2>&1; then rm -rf "$_read13_dir"; exit 1; fi
# TAMPER B — a v1 tag still carrying sections is ambiguous and MUST be rejected.
python3 -c "import json; d=json.load(open('$_read13_dir/out.json')); d['schema']='read0-run-v1'; d.pop('structure_hash',None); json.dump(d,open('$_read13_dir/v1sec.json','w'))"
if ./target/debug/read0 verify "$_read13_dir/v1sec.json" >/dev/null 2>&1; then rm -rf "$_read13_dir"; exit 1; fi
# TAMPER C — an unknown schema version MUST be rejected GRACEFULLY (no panic).
python3 -c "import json; d=json.load(open('$_read13_dir/out.json')); d['schema']='read0-run-v9'; json.dump(d,open('$_read13_dir/unknown.json','w'))"
if ./target/debug/read0 verify "$_read13_dir/unknown.json" >/dev/null 2>"$_read13_dir/u.err"; then rm -rf "$_read13_dir"; exit 1; fi
if grep -qi panic "$_read13_dir/u.err"; then rm -rf "$_read13_dir"; exit 1; fi
rm -rf "$_read13_dir"
# ---------------------------------------------------------------------------------------------------
# READ-14 — receipt integrity hashing for structural metadata. read0 now writes `read0-run-v3`, which adds
# an explicit structural-integrity hash (FNV-1a 64-bit) binding the schema + per-document title, ordered
# span texts, and ordered sections (heading + span count). verify/replay recompute it and reject a mismatch,
# so a NON-EVIDENTIARY structural edit that the READ-12/13 consistency checks would miss — a heading or
# title string, an UNCITED span's text, a section boundary that still partitions — is now caught. The hash
# is version-gated: a v3 receipt MUST carry a matching hash; a pre-v3 (v1/v2) receipt MUST NOT (a relabel
# that keeps a stale hash is rejected). The hash is an INTEGRITY checksum, never an evidence signal — it
# never reaches the codec/grounding and never makes a heading citable; evidence authority (memory/answer
# re-derivation + cited-span grounding) is unchanged. The reading-cli suite (heading_string_tamper_is_rejected,
# title_tamper_is_rejected, uncited_span_tamper_caught_under_v3_not_v2, v3_receipt_with_missing_structure_hash_is_rejected,
# v2_receipt_carrying_structure_hash_is_rejected, structural_hash_is_deterministic_and_field_sensitive) is
# the load-bearing check. Schema/receipt hardening only — no model, no training.
# ---------------------------------------------------------------------------------------------------
# The structural binding is present: the v3 tag, the structure_hash field, and the deterministic hasher.
grep -q 'read0-run-v3' crates/reading-cli/src/lib.rs
grep -q 'structure_hash' crates/reading-cli/src/lib.rs
grep -q 'fn structural_hash' crates/reading-cli/src/lib.rs
grep -q 'fn enforce_structure_hash' crates/reading-cli/src/lib.rs
# End-to-end structural-hash binary smoke: a v3 receipt carries a structure_hash and verifies; tampering a
# heading STRING, corrupting the hash, dropping the hash, or relabel-keeping it under v2 are each rejected.
_read14_dir="$(mktemp -d)"
mkdir -p "$_read14_dir/docs"
printf '# Overview\nThe bridge is open.\n## Wind Forecast\nWinds will reach forty miles per hour.' > "$_read14_dir/docs/forecast.txt"
cat > "$_read14_dir/plan.json" <<'READ14_PLAN'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":1},{"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},{"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}]
READ14_PLAN
./target/debug/read0 run "$_read14_dir/docs" "What is the wind forecast?" "$_read14_dir/plan.json" "$_read14_dir/out.json" >/dev/null 2>&1
./target/debug/read0 verify "$_read14_dir/out.json" >/dev/null 2>&1
# the v3 receipt carries the schema tag and a structure hash.
grep -q '"read0-run-v3"' "$_read14_dir/out.json"
grep -q '"structure_hash"' "$_read14_dir/out.json"
# TAMPER A — edit a section HEADING string (a non-evidentiary label that still partitions): MUST be rejected.
python3 -c "import json; d=json.load(open('$_read14_dir/out.json')); d['documents'][0]['sections'][1]['heading']='Calm Skies'; json.dump(d,open('$_read14_dir/heading.json','w'))"
if ./target/debug/read0 verify "$_read14_dir/heading.json" >/dev/null 2>&1; then rm -rf "$_read14_dir"; exit 1; fi
# TAMPER B — corrupt the structure hash: MUST be rejected.
python3 -c "import json; d=json.load(open('$_read14_dir/out.json')); d['structure_hash']=(d['structure_hash'] ^ 0xDEADBEEF); json.dump(d,open('$_read14_dir/corrupt.json','w'))"
if ./target/debug/read0 verify "$_read14_dir/corrupt.json" >/dev/null 2>&1; then rm -rf "$_read14_dir"; exit 1; fi
# TAMPER C — a v3 receipt with the structure hash DROPPED: MUST be rejected (the binding cannot vanish).
python3 -c "import json; d=json.load(open('$_read14_dir/out.json')); d.pop('structure_hash',None); json.dump(d,open('$_read14_dir/nohash.json','w'))"
if ./target/debug/read0 verify "$_read14_dir/nohash.json" >/dev/null 2>"$_read14_dir/n.err"; then rm -rf "$_read14_dir"; exit 1; fi
if grep -qi panic "$_read14_dir/n.err"; then rm -rf "$_read14_dir"; exit 1; fi
# TAMPER D — relabel to v2 but KEEP the structure hash (a pre-v3 tag must not carry it): MUST be rejected.
python3 -c "import json; d=json.load(open('$_read14_dir/out.json')); d['schema']='read0-run-v2'; json.dump(d,open('$_read14_dir/v2hash.json','w'))"
if ./target/debug/read0 verify "$_read14_dir/v2hash.json" >/dev/null 2>&1; then rm -rf "$_read14_dir"; exit 1; fi
rm -rf "$_read14_dir"
# ---------------------------------------------------------------------------------------------------
# READ-15 — receipt downgrade policy / final receipt boundary. verify now CLASSIFIES the receipt's
# structural-integrity LEVEL (IntegrityLevel, derived from the validated schema version, never persisted so
# it cannot be forged) and surfaces it as a MACHINE-CHECKABLE token: a v3 receipt reports `structure_bound`;
# a legacy/downgraded v1/v2 receipt reports `legacy_unbound_structure` plus an explicit warning. So weaker
# integrity is never silently accepted as equivalent to current integrity — a v3→v2 stripped-hash downgrade
# still verifies (its evidence is bound) but is reported as legacy, NOT current. The integrity level
# classifies structure only and NEVER changes grounding authority (a v3 receipt and its v2 downgrade produce
# the identical verifier Receipt). The reading-cli suite (v3_receipt_reports_current_integrity,
# legacy_v2_and_v1_report_legacy_unbound_structure, v3_to_v2_downgrade_is_not_reported_as_current,
# integrity_level_does_not_change_evidence_authority, integrity_level_is_derived_from_version_not_a_stored_claim)
# is the load-bearing check. Integrity classification only — no model, no training.
# ---------------------------------------------------------------------------------------------------
# The integrity classification is present: the level type, the machine-checkable tokens, the verify outcome.
grep -q 'enum IntegrityLevel' crates/reading-cli/src/lib.rs
grep -q 'struct VerifyOutcome' crates/reading-cli/src/lib.rs
grep -q 'legacy_unbound_structure' crates/reading-cli/src/lib.rs
grep -q 'structure_bound' crates/reading-cli/src/lib.rs
# End-to-end downgrade-policy binary smoke: a v3 receipt reports structure_bound; a faithful v2 downgrade
# (relabel + strip hash) still verifies but reports legacy_unbound_structure and NEVER structure_bound.
_read15_dir="$(mktemp -d)"
mkdir -p "$_read15_dir/docs"
printf '# Overview\nThe bridge is open.\n## Wind Forecast\nWinds will reach forty miles per hour.' > "$_read15_dir/docs/forecast.txt"
cat > "$_read15_dir/plan.json" <<'READ15_PLAN'
[{"action":"inspect_corpus"},{"action":"read_span","span_id":1},{"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},{"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}]
READ15_PLAN
./target/debug/read0 run "$_read15_dir/docs" "What is the wind forecast?" "$_read15_dir/plan.json" "$_read15_dir/out.json" >/dev/null 2>&1
# A v3 receipt verifies AND reports the current, machine-checkable structure_bound integrity token.
./target/debug/read0 verify "$_read15_dir/out.json" > "$_read15_dir/v3.txt" 2>&1
grep -q 'integrity=structure_bound' "$_read15_dir/v3.txt"
# A faithful v2 downgrade (relabel + strip the structure hash) still VERIFIES (legacy evidence is bound)...
python3 -c "import json; d=json.load(open('$_read15_dir/out.json')); d['schema']='read0-run-v2'; d.pop('structure_hash',None); json.dump(d,open('$_read15_dir/v2.json','w'))"
./target/debug/read0 verify "$_read15_dir/v2.json" > "$_read15_dir/v2.txt" 2>&1
# ...but is reported as legacy_unbound_structure and is NEVER reported as current (structure_bound).
grep -q 'integrity=legacy_unbound_structure' "$_read15_dir/v2.txt"
grep -q 'warning: legacy_unbound_structure' "$_read15_dir/v2.txt"
if grep -q 'integrity=structure_bound' "$_read15_dir/v2.txt"; then rm -rf "$_read15_dir"; exit 1; fi
rm -rf "$_read15_dir"
# ---------------------------------------------------------------------------------------------------
# READ-16 — reading track milestone freeze. The READ-0 -> READ-15 arc is frozen as reading-track-v0.1. The
# milestone record (READING_TRACK_MILESTONE.md) pins the commit lineage, the boundaries that hold across the
# arc, the P12 training verdict, and the honest residuals, and is locked here so the freeze cannot silently
# drift. The pinned commit hashes are auditable against `git log`. Documentation freeze only — no model, no
# training; the milestone records training_not_justified.
# ---------------------------------------------------------------------------------------------------
test -f READING_TRACK_MILESTONE.md
grep -q 'FROZEN' READING_TRACK_MILESTONE.md
grep -q 'reading-track-v0.1' READING_TRACK_MILESTONE.md
grep -q 'READ-0' READING_TRACK_MILESTONE.md
grep -q 'READ-15' READING_TRACK_MILESTONE.md
grep -q 'training_not_justified' READING_TRACK_MILESTONE.md
# Commit-lineage endpoints + the P12 training gate are pinned (cross-checkable against git log).
grep -q 'f5b3fa9' READING_TRACK_MILESTONE.md
grep -q '3902418' READING_TRACK_MILESTONE.md
grep -q '11e9c5f' READING_TRACK_MILESTONE.md
# ---------------------------------------------------------------------------------------------------
# HYP-0 / P16 — hypothesis-only abductive layer (post-freeze track, above the frozen reading substrate,
# below human review). It may CREATE / SCORE / TRACE proposed explanations and next probes, and NOTHING
# else. Structural quarantine: hypothesis-layer depends on serde ONLY — its non-dev dependency tree contains
# no codec/substrate/engine crate and no ML crate, so it holds no handle that could mutate memory, the
# verifier, governance, receipts, or engine state (the reading crates are DEV-only, used solely to prove
# non-interference). Every HypothesisPacket carries authority hypothesis_only (the only variant that exists)
# and a baked forbidden_uses list, so it can never become a claim or evidence; a high-risk / irreversible
# probe escalates to human_review_required or is blocked. Scoring is deterministic integer math (no floats,
# no wall-clock, no entropy, no model), so trace replay reproduces the same packet. The reading-cli suite
# proves a hypothesis changes neither the verifier receipt nor the P12 training verdict. Doctrine:
# Probability proposes. Replay tests. Governance authorizes. Memory records. No model, no training.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/hypothesis-layer/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/hypothesis-layer/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Hypothesis-only structure is present (signals).
grep -q 'enum Authority' crates/hypothesis-layer/src/lib.rs
grep -q 'HypothesisOnly' crates/hypothesis-layer/src/lib.rs
grep -q 'struct HypothesisPacket' crates/hypothesis-layer/src/lib.rs
grep -q 'forbidden_uses' crates/hypothesis-layer/src/lib.rs
grep -q 'HumanReviewRequired' crates/hypothesis-layer/src/lib.rs
grep -q 'pub fn propose' crates/hypothesis-layer/src/lib.rs
# A replayed packet re-derives from its inputs: verify_consistency asserts the derivation contract.
grep -q 'fn verify_consistency' crates/hypothesis-layer/src/lib.rs
# ENCAPSULATION (structural, not by convention): a HypothesisPacket can only be MINTED by propose()
# and is read-only thereafter — its fields are private (no indented `pub ` inside the struct body) and
# it does NOT derive Deserialize, so a caller cannot mutate forbidden_uses/clearance/provenance or
# forge a packet off the wire. The deserializable trace surface is the INPUTS (HypothesisSpec); replay
# re-derives every governed field. RecommendedProbe is likewise read-only. These checks fail closed if
# the boundary regresses to public/forgeable fields.
grep -q 'pub fn recommended_probe(&self)' crates/hypothesis-layer/src/lib.rs
grep -q 'pub fn forbidden_uses(&self)' crates/hypothesis-layer/src/lib.rs
test "$(awk '/pub struct HypothesisPacket \{/,/^\}/' crates/hypothesis-layer/src/lib.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(awk '/pub struct RecommendedProbe \{/,/^\}/' crates/hypothesis-layer/src/lib.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
# Non-deserializability of BOTH inert output types is COMPILER-enforced by `compile_fail` doctests,
# NOT by grepping the derive line. A line-based `grep -B1 ... Deserialize` is dodgeable: an interposing
# comment between `#[derive(...)]` and the struct pushes the token out of the one-line window, and
# RecommendedProbe (unlike a packet, whose RecommendedProbe field blocks the derive) has only
# deserializable fields, so it would derive Deserialize cleanly. So we instead assert the compiler
# proofs EXIST and are anchored to the right types: if a proof is deleted the count check fails; if a
# type regresses to Deserialize (derive OR hand-written impl), its `compile_fail` body compiles and
# `cargo test` (above) fails. A proof cannot be both present and satisfied while its type deserializes.
grep -q 'let _: hypothesis_layer::HypothesisPacket = serde_json::from_str' crates/hypothesis-layer/src/lib.rs
grep -q 'let _: hypothesis_layer::RecommendedProbe = serde_json::from_str' crates/hypothesis-layer/src/lib.rs
test "$(grep -c 'compile_fail' crates/hypothesis-layer/src/lib.rs)" -ge 2
# The deserializable trace surface (the INPUTS) is likewise compiler-proven: a positive doctest
# round-trips a HypothesisSpec through propose(), so the replay path can't silently break.
grep -q 'let spec: hypothesis_layer::HypothesisSpec = serde_json::from_str' crates/hypothesis-layer/src/lib.rs
# Non-derive (hand-written `impl Deserialize for ...`) bypass is rejected too — whole-file scan,
# comment-insensitive — as a cheap canary on top of the compiler proofs above.
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+HypothesisPacket\b' crates/hypothesis-layer/src/lib.rs)" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+RecommendedProbe\b' crates/hypothesis-layer/src/lib.rs)" -eq 0
# Single-variant Authority is COMPILER-enforced, not just grepped: the authority_has_exactly_one_variant
# test matches Authority exhaustively with no wildcard, so adding a second variant (e.g. a Claim/Evidence
# authority) breaks compilation. "Any other authority is unrepresentable" cannot silently regress.
grep -q 'fn authority_has_exactly_one_variant' crates/hypothesis-layer/src/lib.rs
# The forbidden-uses quarantine is pinned by IDENTITY, not by a circular length/membership check:
# forbidden_uses_are_exactly_the_canonical_six asserts each of the six canonical use-names (written as
# literals, not read from FORBIDDEN_USES) is refused AND that the set has six DISTINCT entries. Replacing
# a canonical use with a duplicate (length stays 6) un-forbids it but fails the distinctness assert, and
# dropping/renaming one fails the literal assert. The other forbidden_uses tests iterate the const and so
# cannot catch a substitution; this one can.
grep -q 'fn forbidden_uses_are_exactly_the_canonical_six' crates/hypothesis-layer/src/lib.rs
# The end-to-end determinism smoke must EXERCISE the real API: the example has to CALL propose() (not
# print a hardcoded JSON), so the two-run diff + key greps below prove a genuine propose() output.
grep -q 'propose(' crates/hypothesis-layer/examples/hypothesis_report.rs
# Deterministic integer scoring: NO floats, NO wall-clock, NO entropy in the source.
test "$(grep -cE '\bf32\b|\bf64\b' crates/hypothesis-layer/src/lib.rs)" -eq 0
test "$(grep -cE 'SystemTime|Instant|rand::|rand_|random|thread_rng' crates/hypothesis-layer/src/lib.rs)" -eq 0
# Structural QUARANTINE: the production (non-dev) dependency tree holds no engine/reading crate and no ML.
test "$(cargo tree --offline --manifest-path crates/hypothesis-layer/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/hypothesis-layer/Cargo.toml --edges normal 2>/dev/null | grep -cE 'reading-')" -eq 0
test "$(grep -ciE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/hypothesis-layer/Cargo.toml)" -eq 0
# End-to-end determinism smoke: the demo packet is a pure function of fixed inputs, so two runs are
# byte-identical (trace replay reproduces the packet), and the packet carries hypothesis_only authority,
# the forbidden-uses, and a probe clearance.
cargo build --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml --example hypothesis_report >/dev/null 2>&1
_hyp_dir="$(mktemp -d)"
./target/debug/examples/hypothesis_report > "$_hyp_dir/run1.json" 2>/dev/null
./target/debug/examples/hypothesis_report > "$_hyp_dir/run2.json" 2>/dev/null
if ! cmp -s "$_hyp_dir/run1.json" "$_hyp_dir/run2.json"; then rm -rf "$_hyp_dir"; exit 1; fi
grep -q '"authority": "hypothesis_only"' "$_hyp_dir/run1.json"
grep -q '"ground_claim"' "$_hyp_dir/run1.json"
grep -q '"clearance"' "$_hyp_dir/run1.json"
rm -rf "$_hyp_dir"
# ---------------------------------------------------------------------------------------------------
# HYP-1 / Probe Queue / Human Review Boundary (crates/hypothesis-layer/src/probe.rs). Turns a
# HypothesisPacket's recommended probe into an inert, deterministic ProbeRequest queue item with a
# MACHINE-CHECKABLE review status (queued | human_review_required | blocked) — WITHOUT executing the
# probe or mutating anything. A request is minted ONLY by ProbeRequest::from_hypothesis (private fields,
# no Deserialize — compiler-enforced), so its risk/reversibility-derived status cannot be hand-set or
# forged off the wire; a high-risk/irreversible probe is human_review_required and a high-risk AND
# irreversible one is blocked and never execution-eligible; the queue is content-ordered (insertion-order
# independent) so replay reproduces it. Same quarantine as HYP-0 (serde-only production deps, asserted
# above for the whole crate), so a probe queue changes neither the verifier receipt nor the P12 verdict.
# Doctrine: Hypothesis proposes a probe. HYP-1 queues or blocks it. Human/governance decides execution.
# (test/fmt/clippy for the whole crate, and the quarantine cargo-tree + no-ML scans, run in the HYP-0
# block above and already cover probe.rs.)
# ---------------------------------------------------------------------------------------------------
# Probe-queue structure present (signals).
grep -q 'enum ProbeStatus' crates/hypothesis-layer/src/probe.rs
grep -q 'enum ProbeReason' crates/hypothesis-layer/src/probe.rs
grep -q 'struct ProbeRequest' crates/hypothesis-layer/src/probe.rs
grep -q 'struct ProbeQueue' crates/hypothesis-layer/src/probe.rs
grep -q 'pub fn from_hypothesis' crates/hypothesis-layer/src/probe.rs
grep -q 'pub fn from_hypotheses' crates/hypothesis-layer/src/probe.rs
grep -q 'fn is_execution_eligible' crates/hypothesis-layer/src/probe.rs
grep -q 'human_review_required' crates/hypothesis-layer/src/probe.rs
# ENCAPSULATION (compiler-enforced, the HYP-0 discipline): a ProbeRequest / ProbeQueue is minted ONLY by
# from_hypothesis(es), is read-only (private fields), and is NOT deserializable — so a forged request
# with a hand-set status cannot enter off the wire. Non-deserializability is proven by `compile_fail`
# doctests (the compiler enforces it against derive AND manual impl; `cargo test` runs them); we assert
# the proofs EXIST so they cannot be silently deleted, plus private-fields and whole-file manual-impl scans.
grep -q 'let _: hypothesis_layer::ProbeRequest = serde_json::from_str' crates/hypothesis-layer/src/probe.rs
grep -q 'let _: hypothesis_layer::ProbeQueue = serde_json::from_str' crates/hypothesis-layer/src/probe.rs
test "$(grep -c 'compile_fail' crates/hypothesis-layer/src/probe.rs)" -ge 2
# ...but a text grep cannot tell a LIVE `///` doctest from one converted to a `//` comment (which keeps the
# grep-visible tokens yet drops out of the doctest suite, letting the type be made Deserialize undetected).
# So we ALSO pin the doctest REALITY from cargo itself — it reports every live doctest and labels the
# compile_fail ones — so commenting out or deleting any compile_fail proof lowers these counts and fails
# here. Crate-wide: exactly 14 live doctests, 7 `compile fail`, one per inert type (HypothesisPacket,
# RecommendedProbe, ProbeRequest, ProbeQueue, ReviewReceipt, ReviewLog, ProbeExecutionIntent). (Residual: a
# decoy compile_fail deliberately planted on the same type while the real one is commented out is
# review-evident insider forgery, beyond regression scope.)
_doc_out="$(cargo test --offline --doc --manifest-path crates/hypothesis-layer/Cargo.toml 2>/dev/null)"
test "$(printf '%s\n' "$_doc_out" | grep -oE 'running [0-9]+ tests' | grep -oE '[0-9]+')" -eq 14
test "$(printf '%s\n' "$_doc_out" | grep -c 'compile fail')" -eq 7
printf '%s\n' "$_doc_out" | grep -q 'HypothesisPacket (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'RecommendedProbe (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeRequest (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeQueue (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ReviewReceipt (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ReviewLog (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeExecutionIntent (line.*compile fail'
# Likewise, the UNIT tests must actually RUN, not be silently disabled: an `#[ignore]` (or a cfg-out /
# commented-out `#[test]`) skips a test without failing `cargo test`, so a test-name grep cannot tell an
# enforced policy from a disabled one. We pin the test reality from cargo: the crate's library unit tests
# must report EXACTLY the expected passed count and ZERO ignored — ignoring or removing any test lowers the
# passed count and/or raises the ignored count and fails here. (Update the count when adding/removing tests.)
_unit_out="$(cargo test --offline --lib --manifest-path crates/hypothesis-layer/Cargo.toml 2>/dev/null)"
test "$(printf '%s\n' "$_unit_out" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')" -eq 48
test "$(printf '%s\n' "$_unit_out" | grep -oE '[0-9]+ ignored' | grep -oE '[0-9]+')" -eq 0
test "$(awk '/pub struct ProbeRequest \{/,/^\}/' crates/hypothesis-layer/src/probe.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(awk '/pub struct ProbeQueue \{/,/^\}/' crates/hypothesis-layer/src/probe.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+Probe(Request|Queue|Status|Reason)\b' crates/hypothesis-layer/src/probe.rs)" -eq 0
# The blocked/review boundary is COMPILER-pinned, not prose: is_execution_eligible matches ProbeStatus
# exhaustively with NO wildcard, so a new status variant cannot silently become executable (E0004). The
# forged_status_cannot_be_constructed + high_risk_and_irreversible_probe_is_blocked tests (run by cargo
# test above) prove a high-risk probe can never carry a Queued status and a blocked probe is never eligible.
grep -q 'fn forged_status_cannot_be_constructed' crates/hypothesis-layer/src/probe.rs
grep -q 'fn high_risk_and_irreversible_probe_is_blocked' crates/hypothesis-layer/src/probe.rs
grep -q 'fn probe_queue_does_not_change_training_gate' crates/hypothesis-layer/src/probe.rs
grep -q 'fn probe_queue_does_not_change_verifier_receipt' crates/hypothesis-layer/src/probe.rs
# Deterministic integer classification + NO PROBE EXECUTION, enforced CRATE-WIDE — over EVERY module under
# src/ (lib.rs, probe.rs, and any future file), not one named file, because the doctrine binds the whole
# crate: the hypothesis-layer is a PROPOSER that may CREATE / SCORE / TRACE and NOTHING else; it NEVER
# runs a probe. NO floats / wall-clock / entropy (non-determinism); and NO process spawn / filesystem /
# network anywhere in src/ OR the examples — a live executor (even `Command::new("sh").arg(req.probe_text())
# .spawn()`) leaves the deterministic output unchanged, so the double-run cannot catch it; these recursive
# scans do. (`std::process::ExitCode` / `process::id` are read-only and intentionally not matched.)
test "$(grep -rE '\bf32\b|\bf64\b' crates/hypothesis-layer/src | wc -l)" -eq 0
test "$(grep -rE 'SystemTime|Instant|rand::|rand_|random|thread_rng' crates/hypothesis-layer/src | wc -l)" -eq 0
test "$(grep -rE 'Command::new|process::Command|\.spawn\(|std::fs|File::create|File::open|fs::write|fs::read|OpenOptions|std::net|TcpStream|UdpSocket' crates/hypothesis-layer/src crates/hypothesis-layer/examples | wc -l)" -eq 0
# The LIBRARY (src/) performs no side-effecting I/O and carries no `#[allow(...)]` that could hide dead or
# execution code past clippy's `-D warnings` dead-code lint. (The example legitimately prints its report.)
test "$(grep -rE 'println!|eprintln!|print!|eprint!|dbg!' crates/hypothesis-layer/src | wc -l)" -eq 0
test "$(grep -rE '#\[allow\(' crates/hypothesis-layer/src | wc -l)" -eq 0
# The end-to-end determinism smoke must EXERCISE the real API: the example CALLS from_hypotheses (not a
# hardcoded queue), so the two-run diff + status greps below prove a genuine ProbeQueue output.
grep -q 'from_hypotheses' crates/hypothesis-layer/examples/probe_queue_report.rs
# End-to-end determinism smoke: the demo queue is a pure function of fixed inputs, so two runs are
# byte-identical (replay reproduces the queue); the queue carries all three review statuses, and exactly
# one probe (the low-risk reversible one) is execution-eligible — blocked + review-required are excluded.
cargo build --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml --example probe_queue_report >/dev/null 2>&1
_pq_dir="$(mktemp -d)"
./target/debug/examples/probe_queue_report > "$_pq_dir/run1.json" 2>/dev/null
./target/debug/examples/probe_queue_report > "$_pq_dir/run2.json" 2>/dev/null
if ! cmp -s "$_pq_dir/run1.json" "$_pq_dir/run2.json"; then rm -rf "$_pq_dir"; exit 1; fi
grep -q '"status": "queued"' "$_pq_dir/run1.json"
grep -q '"status": "human_review_required"' "$_pq_dir/run1.json"
grep -q '"status": "blocked"' "$_pq_dir/run1.json"
grep -q '"execution_eligible": 1' "$_pq_dir/run1.json"
rm -rf "$_pq_dir"
# ---------------------------------------------------------------------------------------------------
# HYP-2 / Governance Review Receipt Boundary (crates/hypothesis-layer/src/review.rs). Records the
# GOVERNANCE DECISION on a HYP-1 ProbeRequest as an inert, deterministic ReviewReceipt (approved /
# rejected / deferred) — WITHOUT executing the probe or mutating anything. Policy is machine-checkable: a
# BLOCKED probe can never be approved by ANY authority; a human_review_required probe can be approved only
# by a human/governance authority (ReviewerAuthority is a CHECKED ENUM, never a free string); a queued
# probe may be approved or rejected — approval still executes nothing. A receipt is minted ONLY by
# ReviewReceipt::decide (private fields, no Deserialize — compiler-enforced, and pinned LIVE by the
# crate-wide cargo doctest-reality check above), so a forged decision cannot be hand-set or deserialized
# off the wire; it carries an integrity_hash over all fields and reuses the FORBIDDEN_USES quarantine so it
# can never become evidence. The crate-wide no-execution / no-float / no-wall-clock / no-IO / no-#[allow]
# scans and the serde-only quarantine cargo-tree above already cover review.rs. Doctrine: Hypothesis
# proposes. Probe queue classifies. Governance reviews. Nothing executes. Nothing becomes evidence.
# ---------------------------------------------------------------------------------------------------
# Review-receipt structure present (signals).
grep -q 'enum ReviewDecision' crates/hypothesis-layer/src/review.rs
grep -q 'enum ReviewerAuthority' crates/hypothesis-layer/src/review.rs
grep -q 'enum ReasonCode' crates/hypothesis-layer/src/review.rs
grep -q 'struct ReviewReceipt' crates/hypothesis-layer/src/review.rs
grep -q 'struct ReviewLog' crates/hypothesis-layer/src/review.rs
grep -q 'pub fn decide' crates/hypothesis-layer/src/review.rs
grep -q 'fn can_approve_review_required' crates/hypothesis-layer/src/review.rs
grep -q 'integrity_hash' crates/hypothesis-layer/src/review.rs
# ENCAPSULATION (compiler-enforced): a ReviewReceipt / ReviewLog is minted ONLY by decide / from_receipts,
# is read-only (private fields), and is NOT deserializable — proven by compile_fail doctests whose LIVE
# presence is pinned by the cargo doctest-reality check above (the existence greps below cannot be dodged
# by a `//`-commented copy because that copy drops out of cargo's doctest run). Plus private-fields and
# whole-file manual-`impl Deserialize` scans for the inert output types.
grep -q 'let _: hypothesis_layer::ReviewReceipt = serde_json::from_str' crates/hypothesis-layer/src/review.rs
grep -q 'let _: hypothesis_layer::ReviewLog = serde_json::from_str' crates/hypothesis-layer/src/review.rs
test "$(awk '/pub struct ReviewReceipt \{/,/^\}/' crates/hypothesis-layer/src/review.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(awk '/pub struct ReviewLog \{/,/^\}/' crates/hypothesis-layer/src/review.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+(ReviewReceipt|ReviewLog|ReasonCode)\b' crates/hypothesis-layer/src/review.rs)" -eq 0
# The governance POLICY is compiler/test-enforced, not prose: the approval gate in decide() matches the
# probe status exhaustively (no wildcard → E0004 on a new status), and these tests (run by cargo test
# above) prove a blocked probe is never approved, a review-required probe needs authority, a queued probe
# is approved without execution, a receipt can't be evidence, and a review changes neither P12 nor a receipt.
grep -q 'fn blocked_probe_cannot_be_approved' crates/hypothesis-layer/src/review.rs
grep -q 'fn review_required_probe_requires_authority' crates/hypothesis-layer/src/review.rs
grep -q 'fn queued_probe_can_be_approved_without_execution' crates/hypothesis-layer/src/review.rs
grep -q 'fn review_receipt_cannot_be_evidence' crates/hypothesis-layer/src/review.rs
grep -q 'fn review_receipt_does_not_change_training_gate' crates/hypothesis-layer/src/review.rs
grep -q 'fn review_receipt_does_not_change_verifier_receipt' crates/hypothesis-layer/src/review.rs
# The end-to-end determinism smoke must EXERCISE the real API: the example CALLS decide + from_receipts.
grep -q 'ReviewReceipt::decide' crates/hypothesis-layer/examples/review_log_report.rs
grep -q 'ReviewLog::from_receipts' crates/hypothesis-layer/examples/review_log_report.rs
# End-to-end determinism smoke: the demo review log is a pure function of fixed inputs, so two runs are
# byte-identical (replay reproduces the log); it carries all three decisions, a blocked probe is rejected
# (never approved → reason rejected_blocked_probe), and exactly two of four reviews are approved.
cargo build --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml --example review_log_report >/dev/null 2>&1
_rl_dir="$(mktemp -d)"
./target/debug/examples/review_log_report > "$_rl_dir/run1.json" 2>/dev/null
./target/debug/examples/review_log_report > "$_rl_dir/run2.json" 2>/dev/null
if ! cmp -s "$_rl_dir/run1.json" "$_rl_dir/run2.json"; then rm -rf "$_rl_dir"; exit 1; fi
grep -q '"decision": "approved"' "$_rl_dir/run1.json"
grep -q '"decision": "rejected"' "$_rl_dir/run1.json"
grep -q '"decision": "deferred"' "$_rl_dir/run1.json"
grep -q '"reason_code": "rejected_blocked_probe"' "$_rl_dir/run1.json"
grep -q '"approved": 2' "$_rl_dir/run1.json"
# BEHAVIORAL policy backstop, independent of the (potentially gut-able) unit tests: the example RUNS the
# real decide() on the forbidden paths, so the gate verifies the policy by behaviour. If the Blocked guard
# or the authority check regresses, decide() returns Ok, these flip to false, and the gate fails here.
grep -q '"policy_blocked_approve_refused": true' "$_rl_dir/run1.json"
grep -q '"policy_automated_review_required_refused": true' "$_rl_dir/run1.json"
rm -rf "$_rl_dir"
# ---------------------------------------------------------------------------------------------------
# HYP-3 / Approved Probe Execution Stub — Non-Execution Boundary (crates/hypothesis-layer/src/execution.rs).
# Converts a HYP-2 ReviewReceipt into an inert, deterministic ProbeExecutionIntent that records what may
# happen to the probe NEXT — WITHOUT executing it, writing a probe result, or mutating anything. The
# disposition is machine-checkable and DERIVED from the review: only an APPROVED review yields a cleared
# intent (not_executed for an automated-scope approval, requires_operator for a human/governance approval);
# a rejected or deferred review yields a `blocked` intent that must never run. A blocked probe can never be
# approved (HYP-2), so it can never reach the cleared path. An intent is minted ONLY by
# ProbeExecutionIntent::from_review (private fields, no Deserialize — compiler-enforced and pinned LIVE by
# the crate-wide cargo doctest-reality check above), so a forged disposition cannot be hand-set or
# deserialized off the wire; it carries an integrity_hash over all fields and reuses the FORBIDDEN_USES
# quarantine so it can never become evidence. The crate-wide no-execution / no-float / no-wall-clock / no-IO
# / no-#[allow] scans and the serde-only quarantine cargo-tree above already cover execution.rs and the new
# example. Doctrine: Hypothesis proposes. Probe queue classifies. Governance reviews. HYP-3 records intent.
# Nothing executes. Nothing becomes evidence.
# ---------------------------------------------------------------------------------------------------
# Execution-intent structure present (signals).
grep -q 'enum ExecutionStatus' crates/hypothesis-layer/src/execution.rs
grep -q 'enum ExecutionReason' crates/hypothesis-layer/src/execution.rs
grep -q 'struct ProbeExecutionIntent' crates/hypothesis-layer/src/execution.rs
grep -q 'pub fn from_review' crates/hypothesis-layer/src/execution.rs
grep -q 'requires_operator' crates/hypothesis-layer/src/execution.rs
grep -q 'integrity_hash' crates/hypothesis-layer/src/execution.rs
# ENCAPSULATION (compiler-enforced): a ProbeExecutionIntent is minted ONLY by from_review, is read-only
# (private fields), and is NOT deserializable — proven by the compile_fail doctest whose LIVE presence is
# pinned by the cargo doctest-reality check above (the existence grep below cannot be dodged by a
# `//`-commented copy because that copy drops out of cargo's doctest run). Plus private-fields and whole-file
# manual-`impl Deserialize` scans for the inert output type and its derived (Serialize-only) enums.
grep -q 'let _: hypothesis_layer::ProbeExecutionIntent = serde_json::from_str' crates/hypothesis-layer/src/execution.rs
test "$(awk '/pub struct ProbeExecutionIntent \{/,/^\}/' crates/hypothesis-layer/src/execution.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+(ProbeExecutionIntent|ExecutionStatus|ExecutionReason)\b' crates/hypothesis-layer/src/execution.rs)" -eq 0
# The non-execution boundary is COMPILER/test-enforced, not prose: the reason derivation matches the review
# decision exhaustively with NO wildcard (a cleared reason requires Approved → E0004 on a new decision), and
# the status<-reason map is likewise exhaustive (E0004 on a new reason), so a rejected/deferred review can
# never derive a cleared status. These tests (run by cargo test above) prove a rejected/deferred review is
# blocked, a blocked probe never reaches a cleared intent, an approval records but does not execute, an
# intent can't be evidence, and an intent changes neither P12 nor a verifier receipt.
grep -q 'fn rejected_review_cannot_create_execution_intent' crates/hypothesis-layer/src/execution.rs
grep -q 'fn deferred_review_cannot_create_execution_intent' crates/hypothesis-layer/src/execution.rs
grep -q 'fn blocked_probe_never_reaches_cleared_intent' crates/hypothesis-layer/src/execution.rs
grep -q 'fn execution_intent_is_not_executed' crates/hypothesis-layer/src/execution.rs
grep -q 'fn intent_cannot_be_evidence' crates/hypothesis-layer/src/execution.rs
grep -q 'fn intent_does_not_change_training_gate' crates/hypothesis-layer/src/execution.rs
grep -q 'fn intent_does_not_change_verifier_receipt' crates/hypothesis-layer/src/execution.rs
# The end-to-end determinism smoke must EXERCISE the real API: the example CALLS from_review + decide.
grep -q 'ProbeExecutionIntent::from_review' crates/hypothesis-layer/examples/execution_intent_report.rs
grep -q 'ReviewReceipt::decide' crates/hypothesis-layer/examples/execution_intent_report.rs
# End-to-end determinism smoke: the demo intent set is a pure function of fixed inputs, so two runs are
# byte-identical (replay reproduces the intents); it carries all three dispositions and exactly two of four
# reviews are cleared (the approvals). The `intents` array is the LEAST-FABRICABLE behavioral surface: each
# element is a serialized REAL ProbeExecutionIntent (built by from_review, a type that is private +
# non-deserializable, so it cannot be forged off-API), so EVERY ExecutionReason token below must appear
# because the corresponding fixed review really derived it. This binds all four dispositions to the real
# from_review output, not just to the (individually hardcodable) standalone policy booleans further down — a
# rejected/deferred review becoming cleared, or an approval being marked executed, drops the matching reason
# token here and fails the gate. (Residual: fabricating the ENTIRE example output AND gutting all four
# covering unit tests is review-evident multi-file insider forgery, beyond regression scope.)
cargo build --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml --example execution_intent_report >/dev/null 2>&1
_ei_dir="$(mktemp -d)"
./target/debug/examples/execution_intent_report > "$_ei_dir/run1.json" 2>/dev/null
./target/debug/examples/execution_intent_report > "$_ei_dir/run2.json" 2>/dev/null
if ! cmp -s "$_ei_dir/run1.json" "$_ei_dir/run2.json"; then rm -rf "$_ei_dir"; exit 1; fi
grep -q '"execution_status": "not_executed"' "$_ei_dir/run1.json"
grep -q '"execution_status": "requires_operator"' "$_ei_dir/run1.json"
grep -q '"execution_status": "blocked"' "$_ei_dir/run1.json"
grep -q '"cleared": 2' "$_ei_dir/run1.json"
# All four ExecutionReason tokens, each from a real from_review disposition in the intents array above.
grep -q '"reason_code": "approved_automated_scope_not_executed"' "$_ei_dir/run1.json"
grep -q '"reason_code": "approved_requires_operator"' "$_ei_dir/run1.json"
grep -q '"reason_code": "rejected_not_executable"' "$_ei_dir/run1.json"
grep -q '"reason_code": "deferred_not_executable"' "$_ei_dir/run1.json"
# BEHAVIORAL policy backstop (a SECONDARY channel to the unit tests + the intents-array reasons above): the
# example RUNS the real from_review() / decide() on the boundary paths — including the blocked-probe path,
# which cannot appear in the intents array (an approved blocked probe is unconstructable) — and emits these.
# If a blocked probe became approvable, or any boundary path regressed, these flip to false and the gate fails.
grep -q '"policy_rejected_review_blocked": true' "$_ei_dir/run1.json"
grep -q '"policy_deferred_review_blocked": true' "$_ei_dir/run1.json"
grep -q '"policy_blocked_probe_never_approved": true' "$_ei_dir/run1.json"
grep -q '"policy_approved_records_not_executed": true' "$_ei_dir/run1.json"
rm -rf "$_ei_dir"
grep -q '"release": "cognitive-os-v0.1.0"' VERSION.json
grep -q '"cip_schema": "cip-schema-v0.1"' VERSION.json
grep -q '"memory_schema": "memory-schema-v0.1"' VERSION.json
grep -q '"simulation": "simulation-bridge-world-v0.1"' VERSION.json
grep -q "New Packet Types" CHANGELOG.md
grep -q "Schema Changes" CHANGELOG.md
grep -q "Memory Behavior Changes" CHANGELOG.md
grep -q "Known Failure Modes" CHANGELOG.md
grep -q "Test Coverage" CHANGELOG.md
grep -q "Simulation Results" CHANGELOG.md
grep -q "FAIL-0001" FAILURE_LEDGER.md
grep -q "locked" FAILURE_LEDGER.md
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
python3 scripts/mutation_audit.py --scenario direct_mutation_without_verifier >"$tmp_dir/direct_mutation.json"
python3 scripts/mutation_audit.py --scenario memory_mutation_with_low_authority_packet >"$tmp_dir/low_authority_mutation.json"
python3 scripts/mutation_audit.py --scenario valid_human_promotion_allows_invariant >"$tmp_dir/valid_promotion.json"
python3 scripts/mutation_audit.py --scenario degraded_action_success_does_not_overconfirm >"$tmp_dir/degraded_success.json"
python3 scripts/mutation_audit.py --scenario degraded_action_failure_quarantines_memory >"$tmp_dir/degraded_failure.json"
python3 scripts/mutation_audit.py --scenario degraded_action_partial_success_scopes_memory >"$tmp_dir/degraded_partial.json"
python3 scripts/contradiction_audit.py --scenario contradiction_resolved_by_new_evidence >"$tmp_dir/contradiction_resolved.json"
python3 scripts/contradiction_audit.py --scenario contradiction_scoped_by_context >"$tmp_dir/contradiction_scoped.json"
python3 scripts/contradiction_audit.py --scenario contradiction_remains_unresolved >"$tmp_dir/contradiction_unresolved.json"
python3 scripts/epistemic_snapshot.py --scenario bridge_a_safe_time_pressure --strict >"$tmp_dir/snapshot_bridge_a.json"
python3 scripts/epistemic_snapshot.py --scenario contradiction_remains_unresolved --strict >"$tmp_dir/snapshot_unresolved.json"
python3 scripts/epistemic_snapshot.py --scenario contradiction_scoped_by_context --strict >"$tmp_dir/snapshot_scoped.json"
python3 scripts/epistemic_snapshot.py --scenario valid_human_promotion_allows_invariant --strict >"$tmp_dir/snapshot_promotion.json"
python3 scripts/planner_regret_audit.py --scenario planner_correct_under_uncertainty >"$tmp_dir/planner_correct.json"
python3 scripts/planner_regret_audit.py --scenario planner_near_miss_requires_policy_review >"$tmp_dir/planner_near_miss.json"
python3 scripts/planner_regret_audit.py --scenario planner_overconservative_waits_unnecessarily >"$tmp_dir/planner_overconservative.json"
python3 scripts/epistemic_snapshot.py --scenario planner_near_miss_requires_policy_review --strict >"$tmp_dir/snapshot_planner_review.json"
python3 scripts/attention_review_audit.py --scenario reflex_mode_correctly_triggered >"$tmp_dir/attention_reflex_correct.json"
python3 scripts/attention_review_audit.py --scenario reflex_mode_false_alarm >"$tmp_dir/attention_false_reflex.json"
python3 scripts/attention_review_audit.py --scenario interrupt_storm_recovery_replay >"$tmp_dir/attention_storm_replay.json"
python3 scripts/epistemic_snapshot.py --scenario reflex_mode_false_alarm --strict >"$tmp_dir/snapshot_attention_review.json"
python3 scripts/recovery_replay.py --scenario recovery_queue_orders_mixed_jobs >"$tmp_dir/recovery_order.json"
python3 scripts/recovery_replay.py --scenario recovery_replay_resolves_jobs_through_gateway >"$tmp_dir/recovery_resolved.json"
python3 scripts/recovery_replay.py --scenario recovery_queue_bounds_deferred_work >"$tmp_dir/recovery_bounds.json"
python3 scripts/epistemic_snapshot.py --scenario recovery_queue_bounds_deferred_work --strict >"$tmp_dir/snapshot_recovery_queue.json"
recovery_ledger="$tmp_dir/recovery_ledger.json"
recovery_key="$tmp_dir/replay_key"
python3 -c "import os,binascii;open('$recovery_key','w').write(binascii.hexlify(os.urandom(32)).decode())"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-key-file "$recovery_key" --ledger "$recovery_ledger" >"$tmp_dir/recovery_idem_1.json"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-key-file "$recovery_key" --ledger "$recovery_ledger" >"$tmp_dir/recovery_idem_2.json"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger "$recovery_ledger" >"$tmp_dir/recovery_idem_nokey.json"
asym_ledger="$tmp_dir/asym_ledger.json"
asym_private="$tmp_dir/replay_ed25519_private.pem"
asym_public="$tmp_dir/replay_ed25519_public.pem"
asym_wrong_public="$tmp_dir/replay_ed25519_wrong_public.pem"
PYTHONPATH=scripts python3 -c "from replay_asymmetric_key import generate_ephemeral_private_key_pem, public_key_pem_from_private_pem; p=generate_ephemeral_private_key_pem(); open('$asym_private','w').write(p); open('$asym_public','w').write(public_key_pem_from_private_pem(p)); q=generate_ephemeral_private_key_pem(); open('$asym_wrong_public','w').write(public_key_pem_from_private_pem(q))"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-private-key-file "$asym_private" --ledger "$asym_ledger" >"$tmp_dir/asym_idem_1.json"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-public-key-file "$asym_public" --ledger "$asym_ledger" >"$tmp_dir/asym_idem_2.json"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-public-key-file "$asym_wrong_public" --ledger "$asym_ledger" >"$tmp_dir/asym_wrong_public.json"
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-public-key-file "$asym_public" >"$tmp_dir/asym_public_only_fresh.json"
python3 scripts/recovery_replay.py --scenario duplicate_correction_job_is_rejected_or_coalesced >"$tmp_dir/recovery_dedup.json"
python3 scripts/recovery_replay.py --scenario failed_job_retry_preserves_audit_lineage >"$tmp_dir/recovery_retry.json"
python3 scripts/epistemic_snapshot.py --scenario failed_job_retry_preserves_audit_lineage --strict >"$tmp_dir/snapshot_recovery_retry.json"
python3 scripts/recovery_replay.py --scenario config_priority_outside_allowlist_rejected >"$tmp_dir/config_priority.json"
python3 scripts/recovery_replay.py --scenario config_unknown_job_type_rejected >"$tmp_dir/config_unknown.json"
python3 scripts/recovery_replay.py --scenario config_attempts_authority_field_injection_rejected >"$tmp_dir/config_inject.json"
python3 scripts/recovery_replay.py --scenario config_valid_job_loads_without_mutation >"$tmp_dir/config_valid.json"
python3 scripts/epistemic_snapshot.py --scenario config_attempts_authority_field_injection_rejected --strict >"$tmp_dir/snapshot_config_inject.json"
python3 scripts/recovery_replay.py --scenario scenario_embedded_ledger_requires_trust_marker >"$tmp_dir/ledger_nomarker.json"
python3 scripts/recovery_replay.py --scenario forged_ledger_verified_idempotent_rejected >"$tmp_dir/ledger_forged.json"
python3 scripts/recovery_replay.py --scenario ledger_job_mutation_mismatch_rejected >"$tmp_dir/ledger_mismatch.json"
python3 scripts/recovery_replay.py --scenario unsigned_ledger_cannot_suppress_mutation >"$tmp_dir/ledger_unsigned.json"
python3 scripts/recovery_replay.py --scenario embedded_test_trusted_ledger_still_test_only >"$tmp_dir/ledger_marker_only.json"
python3 scripts/epistemic_snapshot.py --scenario forged_ledger_verified_idempotent_rejected --strict >"$tmp_dir/snapshot_ledger_forged.json"
python3 scripts/ingest_experience.py --scenario experience_ingest_preserves_raw_episode >"$tmp_dir/raw_ingest.json"
python3 scripts/ingest_experience.py --scenario semantic_candidate_requires_raw_episode >"$tmp_dir/raw_candidate_gate.json"
python3 scripts/ingest_experience.py --scenario raw_episode_is_append_only >"$tmp_dir/raw_append_only.json"
python3 scripts/ingest_experience.py --scenario malformed_experience_rejected_without_partial_state >"$tmp_dir/raw_malformed.json"
python3 scripts/epistemic_snapshot.py --scenario experience_ingest_preserves_raw_episode --strict >"$tmp_dir/snapshot_raw_ingest.json"
python3 scripts/semantic_candidate_extractor.py --scenario raw_episode_generates_semantic_candidates >"$tmp_dir/semantic_candidates.json"
python3 scripts/semantic_candidate_extractor.py --scenario candidate_defaults_to_hypothesis_only >"$tmp_dir/candidate_default.json"
python3 scripts/semantic_candidate_extractor.py --scenario candidate_cites_raw_episode >"$tmp_dir/candidate_cites_raw.json"
python3 scripts/semantic_candidate_extractor.py --scenario llm_output_cannot_create_authoritative_memory >"$tmp_dir/candidate_llm_boundary.json"
python3 scripts/semantic_candidate_extractor.py --scenario candidate_extraction_failure_preserves_raw_episode >"$tmp_dir/candidate_failure.json"
python3 scripts/epistemic_snapshot.py --scenario raw_episode_generates_semantic_candidates --strict >"$tmp_dir/snapshot_semantic_candidates.json"
python3 scripts/design_audit.py --scenario design_contradiction_in_sprint_plan >"$tmp_dir/design_contradiction.json"
python3 scripts/design_audit.py --scenario design_proposal_consistent_with_invariants >"$tmp_dir/design_consistent.json"
python3 scripts/decision_audit.py --project --strict >"$tmp_dir/project_audit.json"
python3 scripts/project_self_audit.py --project --strict --emit-health >"$tmp_dir/project_self_audit.json"
python3 scripts/effect_classifier.py >/dev/null
python3 scripts/design_audit.py --scenario design_effect_mislabel_attack >"$tmp_dir/design_mislabel.json"
python3 scripts/design_audit.py --scenario design_effect_derived_without_declaration >"$tmp_dir/design_no_decl.json"
python3 scripts/design_audit.py --scenario design_effect_preserve_consistent >"$tmp_dir/design_preserve.json"
python3 scripts/design_audit.py --scenario design_effect_lexicon_avoiding_weaken >"$tmp_dir/design_lexicon_evasion.json"
python3 scripts/design_audit.py --scenario design_effect_ambiguous_needs_review >"$tmp_dir/design_ambiguous.json"
grep -q '"decision": "reject"' "$tmp_dir/direct_mutation.json"
grep -q '"decision": "reject"' "$tmp_dir/low_authority_mutation.json"
grep -q '"decision": "allow"' "$tmp_dir/valid_promotion.json"
grep -q '"after": "promoted_invariant"' "$tmp_dir/valid_promotion.json"
grep -q '"after": "retest_required"' "$tmp_dir/degraded_success.json"
grep -q '"after": "quarantined"' "$tmp_dir/degraded_failure.json"
grep -q '"after": "exception_scoped"' "$tmp_dir/degraded_partial.json"
grep -q '"repair_type": "resolved_by_new_evidence"' "$tmp_dir/contradiction_resolved.json"
grep -q '"raw_episodes_preserved": true' "$tmp_dir/contradiction_resolved.json"
grep -q '"repair_type": "resolved_by_scope"' "$tmp_dir/contradiction_scoped.json"
grep -q '"unresolved_visible": true' "$tmp_dir/contradiction_unresolved.json"
grep -q '"strict_action_blocked": true' "$tmp_dir/contradiction_unresolved.json"
grep -q '"authority_license": "hazard_only"' "$tmp_dir/snapshot_bridge_a.json"
grep -q '"surface_role": "current_cognition"' "$tmp_dir/snapshot_bridge_a.json"
grep -q '"selected": "Bridge B"' "$tmp_dir/snapshot_bridge_a.json"
grep -q '"post_action_revalidation":' "$tmp_dir/snapshot_bridge_a.json"
grep -q '"status": "contradicted"' "$tmp_dir/snapshot_unresolved.json"
grep -q 'Strict/full-premise action blocked' "$tmp_dir/snapshot_unresolved.json"
grep -q '"contradiction_repair":' "$tmp_dir/snapshot_unresolved.json"
grep -q '"scope_conditions"' "$tmp_dir/snapshot_scoped.json"
grep -q '"authority_class": "promoted_invariant"' "$tmp_dir/snapshot_promotion.json"
grep -q '"after": "planner_policy_scoped_strengthened"' "$tmp_dir/planner_correct.json"
grep -q '"regret_class": "policy_success"' "$tmp_dir/planner_correct.json"
grep -q '"review_required": true' "$tmp_dir/planner_near_miss.json"
grep -q '"regret_class": "safety_near_miss"' "$tmp_dir/planner_near_miss.json"
grep -q '"review_status": "open"' "$tmp_dir/planner_near_miss.json"
grep -q '"global_rule_rewrite": false' "$tmp_dir/planner_near_miss.json"
grep -q '"policy_update_kind": "opportunity_cost_review"' "$tmp_dir/planner_overconservative.json"
grep -q '"regret_class": "opportunity_cost"' "$tmp_dir/planner_overconservative.json"
grep -q '"planner_review":' "$tmp_dir/snapshot_planner_review.json"
grep -q '"status": "open"' "$tmp_dir/snapshot_planner_review.json"
grep -q '"classification": "justified"' "$tmp_dir/attention_reflex_correct.json"
grep -q '"classification": "over_triggered"' "$tmp_dir/attention_false_reflex.json"
grep -q '"memory_authority_changed": false' "$tmp_dir/attention_false_reflex.json"
grep -q '"planner_authority_changed": false' "$tmp_dir/attention_false_reflex.json"
grep -q '"raw_packet_count": 1000' "$tmp_dir/attention_storm_replay.json"
grep -q '"coalesced_source_count": 1000' "$tmp_dir/attention_storm_replay.json"
grep -q '"attention_mode_review":' "$tmp_dir/snapshot_attention_review.json"
grep -q '"status": "open"' "$tmp_dir/snapshot_attention_review.json"
grep -q '"deterministic_order":' "$tmp_dir/recovery_order.json"
grep -q '"CJ_action_002"' "$tmp_dir/recovery_order.json"
grep -q '"audit_replayable": true' "$tmp_dir/recovery_resolved.json"
grep -q '"mutation_ids":' "$tmp_dir/recovery_resolved.json"
grep -q '"highest_priority_pending_job":' "$tmp_dir/recovery_bounds.json"
grep -q '"priority": "P0"' "$tmp_dir/recovery_bounds.json"
grep -q '"coalesced_or_deferred_count": 2' "$tmp_dir/recovery_bounds.json"
grep -q '"correction_queue":' "$tmp_dir/snapshot_recovery_queue.json"
grep -q '"deferred_correction_jobs":' "$tmp_dir/snapshot_recovery_queue.json"
grep -q '"jobs_requiring_mutation_authority":' "$tmp_dir/snapshot_recovery_queue.json"
grep -q '"decision": "allow"' "$tmp_dir/recovery_idem_1.json"
grep -q '"decision": "verify"' "$tmp_dir/recovery_idem_2.json"
grep -q '"resolution": "verified_idempotent_replay"' "$tmp_dir/recovery_idem_2.json"
grep -q '"idempotent_replay": true' "$tmp_dir/recovery_idem_2.json"
grep -q '"status": "coalesced"' "$tmp_dir/recovery_dedup.json"
grep -q '"coalesced_into": "CJ_dup_a"' "$tmp_dir/recovery_dedup.json"
grep -q '"coalesced_duplicate_count": 1' "$tmp_dir/recovery_dedup.json"
grep -q '"original_failure":' "$tmp_dir/recovery_retry.json"
grep -q '"retried_resolved_through_mutation_gateway"' "$tmp_dir/recovery_retry.json"
grep -q '"jobs_with_retry_lineage":' "$tmp_dir/snapshot_recovery_retry.json"
grep -q '"reason": "priority_not_in_allowlist"' "$tmp_dir/config_priority.json"
grep -q '"reason": "unknown_job_type"' "$tmp_dir/config_unknown.json"
grep -q '"reason": "authority_field_injection"' "$tmp_dir/config_inject.json"
grep -q '"rejected_config_attempts": \[\]' "$tmp_dir/config_valid.json"
grep -q '"resolution": "no_state_mutation_required"' "$tmp_dir/config_valid.json"
grep -q '"reason": "authority_field_injection"' "$tmp_dir/snapshot_config_inject.json"
grep -q '"run_id":' "$tmp_dir/recovery_idem_1.json"
grep -q '"scheme": "hmac-sha256"' "$tmp_dir/recovery_idem_1.json"
grep -q '"signature_status": "signed_valid"' "$tmp_dir/recovery_idem_2.json"
grep -q '"signature_status": "no_key"' "$tmp_dir/recovery_idem_nokey.json"
grep -q '"status": "audit_only"' "$tmp_dir/recovery_idem_nokey.json"
grep -q '"scheme": "ed25519"' "$tmp_dir/asym_idem_1.json"
grep -q '"asymmetric_signature_status": "asymmetric_signed_valid"' "$tmp_dir/asym_idem_2.json"
grep -q '"decision": "verify"' "$tmp_dir/asym_idem_2.json"
grep -q '"idempotent_replay": true' "$tmp_dir/asym_idem_2.json"
grep -q '"asymmetric_signature_status": "wrong_public_key"' "$tmp_dir/asym_wrong_public.json"
grep -q '"status": "audit_only"' "$tmp_dir/asym_wrong_public.json"
grep -q '"signature":' "$tmp_dir/asym_idem_1.json"
test "$(grep -c '"signature":' "$tmp_dir/asym_public_only_fresh.json")" -eq 0
grep -q '"status": "untrusted"' "$tmp_dir/ledger_nomarker.json"
grep -q '"reason": "embedded_ledger_requires_trust_marker"' "$tmp_dir/ledger_nomarker.json"
grep -q '"status": "rejected"' "$tmp_dir/ledger_forged.json"
grep -q '"reason": "ledger_job_mutation_mismatch"' "$tmp_dir/ledger_forged.json"
grep -q '"reason": "ledger_job_mutation_mismatch"' "$tmp_dir/ledger_mismatch.json"
grep -q '"status": "audit_only"' "$tmp_dir/ledger_unsigned.json"
grep -q '"signature_status": "unsigned"' "$tmp_dir/ledger_unsigned.json"
grep -q '"integrity_status": "trusted"' "$tmp_dir/ledger_marker_only.json"
grep -q '"ledger_authentication":' "$tmp_dir/snapshot_ledger_forged.json"
grep -q '"signature_status":' "$tmp_dir/snapshot_ledger_forged.json"
grep -q '"asymmetric_signature_status":' "$tmp_dir/snapshot_ledger_forged.json"
grep -q '"episode_id": "RE_bridge_a_inspection_raw"' "$tmp_dir/raw_ingest.json"
grep -q '"parsed_claims": \[\]' "$tmp_dir/raw_ingest.json"
grep -q '"raw_before_semantic": true' "$tmp_dir/raw_ingest.json"
grep -q '"source_raw_episode_id": "RE_bridge_a_inspection_raw"' "$tmp_dir/raw_ingest.json"
grep -q '"candidate_without_raw_blocked": true' "$tmp_dir/raw_candidate_gate.json"
grep -q '"append_only_replace_blocked": true' "$tmp_dir/raw_append_only.json"
grep -q '"episode_count": 0' "$tmp_dir/raw_malformed.json"
grep -q '"rejected_envelopes":' "$tmp_dir/raw_malformed.json"
grep -q '"raw_ingestion":' "$tmp_dir/snapshot_raw_ingest.json"
grep -q '"kind": "raw_episode"' "$tmp_dir/snapshot_raw_ingest.json"
grep -q '"integrity_digest":' "$tmp_dir/snapshot_raw_ingest.json"
grep -q '"memory_id": "CMN_bridge_a_standing_water"' "$tmp_dir/semantic_candidates.json"
grep -q '"candidate_count": 1' "$tmp_dir/semantic_candidates.json"
grep -q '"epistemic_license": "hypothesis_only"' "$tmp_dir/candidate_default.json"
grep -q '"status": "semantic_candidate"' "$tmp_dir/candidate_default.json"
grep -q '"source_raw_episode_id": "RE_bridge_a_audio_raw"' "$tmp_dir/candidate_cites_raw.json"
grep -q '"all_candidates_cite_raw_episode": true' "$tmp_dir/candidate_cites_raw.json"
grep -q '"non_authoritative_by_default": true' "$tmp_dir/candidate_llm_boundary.json"
grep -q '"forbidden_use":' "$tmp_dir/candidate_llm_boundary.json"
grep -q '"candidate_count": 0' "$tmp_dir/candidate_failure.json"
grep -q '"raw_episode_count": 1' "$tmp_dir/candidate_failure.json"
grep -q '"rejected_candidates":' "$tmp_dir/candidate_failure.json"
grep -q '"semantic_candidate_extraction":' "$tmp_dir/snapshot_semantic_candidates.json"
grep -q '"kind": "candidate_memory_node"' "$tmp_dir/snapshot_semantic_candidates.json"
grep -q '"authority_license": "hypothesis_only"' "$tmp_dir/snapshot_semantic_candidates.json"
grep -q '"contradiction_license": "hazard_only"' "$tmp_dir/design_contradiction.json"
grep -q '"conflict_type": "hard_contradiction"' "$tmp_dir/design_contradiction.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_contradiction.json"
grep -q '"mutation_decision": "reject"' "$tmp_dir/design_contradiction.json"
grep -q '"invariant_preserved": true' "$tmp_dir/design_contradiction.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_contradiction.json"
grep -q '"revalidation_scheduled": true' "$tmp_dir/design_contradiction.json"
grep -q '"blocks_release": true' "$tmp_dir/design_contradiction.json"
grep -q '"naked_fact": false' "$tmp_dir/design_contradiction.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_consistent.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_consistent.json"
grep -q '"contradiction_detected": false' "$tmp_dir/design_consistent.json"
grep -q '"strict_audit": "pass"' "$tmp_dir/project_audit.json"
grep -q '"project_cognitive_health": "green"' "$tmp_dir/project_audit.json"
grep -q '"project_cognitive_health_consolidated": true' "$tmp_dir/project_audit.json"
grep -q '"violations": \[\]' "$tmp_dir/project_audit.json"
grep -q '"strict_audit": "pass"' "$tmp_dir/project_self_audit.json"
grep -q '"all_non_authoritative": true' "$tmp_dir/project_self_audit.json"
grep -q '"declared_effect": "extend"' "$tmp_dir/design_mislabel.json"
grep -q '"derived_effect": "contradict"' "$tmp_dir/design_mislabel.json"
grep -q '"effect_mislabel": true' "$tmp_dir/design_mislabel.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_mislabel.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_mislabel.json"
grep -q '"contradiction_license": "hazard_only"' "$tmp_dir/design_mislabel.json"
grep -q '"declared_effect": null' "$tmp_dir/design_no_decl.json"
grep -q '"derived_effect": "contradict"' "$tmp_dir/design_no_decl.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_no_decl.json"
grep -q '"effect_mislabel": false' "$tmp_dir/design_no_decl.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_preserve.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_preserve.json"
grep -q '"effect_mislabel": false' "$tmp_dir/design_preserve.json"
grep -q '"derived_effect": "contradict"' "$tmp_dir/design_lexicon_evasion.json"
grep -q '"effect_mislabel": true' "$tmp_dir/design_lexicon_evasion.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_lexicon_evasion.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_lexicon_evasion.json"
grep -q '"derived_effect": "needs_review"' "$tmp_dir/design_ambiguous.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_ambiguous.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_ambiguous.json"
# Sprint 26: trace-grounded invariant diff. Behavior decides, not wording.
python3 scripts/trace_diff.py >/dev/null
python3 scripts/design_audit.py --scenario preserve_marker_launders_weakening_blocked >"$tmp_dir/design_launder.json"
python3 scripts/design_audit.py --scenario trace_diff_detects_hazard_gate_softening >"$tmp_dir/design_trace_hazard.json"
python3 scripts/design_audit.py --scenario trace_diff_detects_consolidation_gate_softening >"$tmp_dir/design_trace_consolidation.json"
python3 scripts/design_audit.py --scenario trace_diff_accepts_true_preserving_extension >"$tmp_dir/design_trace_accept.json"
grep -q '"lexical_effect": "preserve"' "$tmp_dir/design_launder.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_launder.json"
grep -q '"effect_authority": "trace_behavior_regression"' "$tmp_dir/design_launder.json"
grep -q '"derived_effect": "contradict"' "$tmp_dir/design_launder.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_launder.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_launder.json"
grep -q '"lexical_effect": "extend"' "$tmp_dir/design_trace_hazard.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_trace_hazard.json"
grep -q '"derived_effect": "contradict"' "$tmp_dir/design_trace_hazard.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_trace_hazard.json"
grep -q '"trace_post": "consolidated"' "$tmp_dir/design_trace_consolidation.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_trace_consolidation.json"
grep -q '"derived_effect": "weaken"' "$tmp_dir/design_trace_consolidation.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_trace_consolidation.json"
grep -q '"trace_tested": true' "$tmp_dir/design_trace_accept.json"
grep -q '"trace_regressed": false' "$tmp_dir/design_trace_accept.json"
grep -q '"effect_authority": "trace_confirmed_preservation"' "$tmp_dir/design_trace_accept.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_trace_accept.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_trace_accept.json"
# Sprint 27: complete locked-invariant probe coverage. No probe -> not eligible for accept.
python3 scripts/design_audit.py --scenario trace_diff_blocks_no_naked_facts_laundering >"$tmp_dir/design_naked.json"
python3 scripts/design_audit.py --scenario trace_diff_blocks_raw_episode_append_only_laundering >"$tmp_dir/design_append.json"
python3 scripts/design_audit.py --scenario trace_diff_blocks_llm_authority_laundering >"$tmp_dir/design_llm.json"
for f in "$tmp_dir/design_naked.json" "$tmp_dir/design_append.json" "$tmp_dir/design_llm.json"; do
  grep -q '"lexical_effect": "preserve"' "$f"
  grep -q '"trace_regressed": true' "$f"
  grep -q '"effect_authority": "trace_behavior_regression"' "$f"
  grep -q '"derived_effect": "weaken"' "$f"
  grep -q '"governance_decision": "block"' "$f"
  grep -q '"proposal_consolidated": false' "$f"
done
grep -q '"trace_post": "normal_use"' "$tmp_dir/design_naked.json"
grep -q '"trace_post": "consolidated"' "$tmp_dir/design_llm.json"
# Sprint 28: delta-to-code provenance. A delta without provenance is just a label.
python3 scripts/change_provenance.py --selftest >/dev/null
python3 scripts/design_audit.py --scenario misstated_noop_delta_with_weakening_patch_blocked >"$tmp_dir/design_misstated.json"
python3 scripts/design_audit.py --scenario derived_delta_matches_patch_accepts_preserving_change >"$tmp_dir/design_prov_accept.json"
python3 scripts/design_audit.py --scenario missing_patch_for_behavioral_delta_needs_review >"$tmp_dir/design_missing_patch.json"
python3 scripts/design_audit.py --scenario delta_provenance_required_for_locked_invariant >"$tmp_dir/design_prov_required.json"
grep -q '"delta_matches_change_set": false' "$tmp_dir/design_misstated.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_misstated.json"
grep -q '"trace_provenance": "verified"' "$tmp_dir/design_misstated.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_misstated.json"
grep -q '"trace_provenance": "verified"' "$tmp_dir/design_prov_accept.json"
grep -q '"changed_artifact": "simulations/bridge_world/control_point_policies/hazard_gate.json"' "$tmp_dir/design_prov_accept.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_prov_accept.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_prov_accept.json"
for f in "$tmp_dir/design_missing_patch.json" "$tmp_dir/design_prov_required.json"; do
  grep -q '"trace_provenance": "missing"' "$f"
  grep -q '"effect_authority": "delta_provenance_unverified"' "$f"
  grep -q '"governance_decision": "block"' "$f"
  grep -q '"proposal_consolidated": false' "$f"
done
# Sprint 29: artifact content-hash binding. A change is the before/after artifact content.
python3 scripts/design_audit.py --scenario stale_pre_image_hash_rejected >"$tmp_dir/design_stale.json"
python3 scripts/design_audit.py --scenario wrong_post_image_hash_rejected >"$tmp_dir/design_wrongpost.json"
python3 scripts/design_audit.py --scenario structured_patch_diverges_from_literal_diff_blocked >"$tmp_dir/design_diverges.json"
python3 scripts/design_audit.py --scenario literal_diff_weakening_change_blocks >"$tmp_dir/design_litweaken.json"
python3 scripts/design_audit.py --scenario literal_diff_preserving_change_accepts >"$tmp_dir/design_litpreserve.json"
grep -q '"trace_provenance": "stale_pre_image"' "$tmp_dir/design_stale.json"
grep -q '"effect_authority": "delta_provenance_unverified"' "$tmp_dir/design_stale.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_stale.json"
grep -q '"trace_provenance": "wrong_post_image"' "$tmp_dir/design_wrongpost.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_wrongpost.json"
grep -q '"trace_provenance": "structured_patch_diverges"' "$tmp_dir/design_diverges.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_diverges.json"
grep -q '"trace_provenance": "verified"' "$tmp_dir/design_litweaken.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_litweaken.json"
grep -q '"derived_effect": "contradict"' "$tmp_dir/design_litweaken.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_litweaken.json"
grep -q '"trace_provenance": "verified"' "$tmp_dir/design_litpreserve.json"
grep -q '"trace_regressed": false' "$tmp_dir/design_litpreserve.json"
grep -q '"changed_artifact": "simulations/bridge_world/control_point_policies/hazard_gate.json"' "$tmp_dir/design_litpreserve.json"
grep -q '"diff_digest":' "$tmp_dir/design_litpreserve.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_litpreserve.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_litpreserve.json"
# Sprint 30: signed change provenance. Authorship over the content digest; never overrides trace.
python3 scripts/design_signing.py --selftest >/dev/null
python3 scripts/design_audit.py --scenario signed_preserving_change_accepts >"$tmp_dir/design_signed_accept.json"
python3 scripts/design_audit.py --scenario signed_weakening_change_still_blocks >"$tmp_dir/design_signed_weaken.json"
python3 scripts/design_audit.py --scenario unsigned_content_bound_change_blocks >"$tmp_dir/design_unsigned.json"
python3 scripts/design_audit.py --scenario wrong_signer_rejected >"$tmp_dir/design_wrongsigner.json"
python3 scripts/design_audit.py --scenario signature_replay_against_different_artifact_rejected >"$tmp_dir/design_replay.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/design_signed_accept.json"
grep -q '"signer": "design_authority"' "$tmp_dir/design_signed_accept.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_signed_accept.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_signed_accept.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/design_signed_weaken.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_signed_weaken.json"
grep -q '"effect_authority": "trace_behavior_regression"' "$tmp_dir/design_signed_weaken.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_signed_weaken.json"
grep -q '"signature_status": "unsigned"' "$tmp_dir/design_unsigned.json"
grep -q '"effect_authority": "change_signature_unverified"' "$tmp_dir/design_unsigned.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_unsigned.json"
grep -q '"signature_status": "unauthorized_signer"' "$tmp_dir/design_wrongsigner.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_wrongsigner.json"
grep -q '"signature_status": "signature_payload_mismatch"' "$tmp_dir/design_replay.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_replay.json"
# Sprint 31: signer-set governance. A public key is not permanent authority; authority is evaluated
# at the decision tick (revoked/expired/out-of-scope rejected; rotated successor accepted; a valid
# governed signer never overrides a trace failure). Lifecycle is logical-tick based (deterministic).
python3 scripts/design_audit.py --scenario revoked_signer_rejected >"$tmp_dir/design_revoked.json"
python3 scripts/design_audit.py --scenario expired_signer_rejected >"$tmp_dir/design_expired.json"
python3 scripts/design_audit.py --scenario wrong_scope_signer_rejected >"$tmp_dir/design_wrongscope.json"
python3 scripts/design_audit.py --scenario rotated_successor_accepted >"$tmp_dir/design_rotated.json"
python3 scripts/design_audit.py --scenario revoked_key_cannot_replay_prior_signature >"$tmp_dir/design_replay31.json"
python3 scripts/design_audit.py --scenario signed_weakening_still_blocks_under_governance >"$tmp_dir/design_govweaken.json"
grep -q '"signature_status": "signer_revoked"' "$tmp_dir/design_revoked.json"
grep -q '"signer_status": "revoked"' "$tmp_dir/design_revoked.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_revoked.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_revoked.json"
grep -q '"signature_status": "signer_expired"' "$tmp_dir/design_expired.json"
grep -q '"signer_expires_at": 50' "$tmp_dir/design_expired.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_expired.json"
grep -q '"signature_status": "signer_wrong_scope"' "$tmp_dir/design_wrongscope.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_wrongscope.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/design_rotated.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/design_rotated.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/design_rotated.json"
grep -q '"evaluation_tick": 150' "$tmp_dir/design_rotated.json"
grep -q '"signature_status": "signer_revoked"' "$tmp_dir/design_replay31.json"
grep -q '"signer_revoked_at": 10' "$tmp_dir/design_replay31.json"
grep -q '"evaluation_tick": 20' "$tmp_dir/design_replay31.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_replay31.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/design_govweaken.json"
grep -q '"trace_regressed": true' "$tmp_dir/design_govweaken.json"
grep -q '"effect_authority": "trace_behavior_regression"' "$tmp_dir/design_govweaken.json"
grep -q '"governance_decision": "block"' "$tmp_dir/design_govweaken.json"
grep -q '"proposal_consolidated": false' "$tmp_dir/design_govweaken.json"
# The governed registry is v0.2 with scope + lifecycle fields; only public keys are committed.
grep -q '"schema": "authorized-design-signers-v0.2"' simulations/bridge_world/authorized_design_signers.json
grep -q '"valid_from_tick"' simulations/bridge_world/authorized_design_signers.json
grep -q '"revoked_at_tick"' simulations/bridge_world/authorized_design_signers.json
grep -q '"rotated_to"' simulations/bridge_world/authorized_design_signers.json
# Determinism: signer lifecycle must be logical-tick based — no wall-clock in the signing module.
test "$(grep -cE 'datetime|time\.time|time\.monotonic' scripts/design_signing.py)" -eq 0
# Sprint 32: mechanism-source content binding. A policy says what the rule is; the mechanism SOURCE
# decides whether it is enforced. The enforcement code is content-hash bound and verified before any
# decision is trusted; a gate-code weakening is caught by probe even with a clean policy + valid sig.
# (Regenerate the manifest with `python3 scripts/mechanism_provenance.py --build` after editing any
# bound enforcement file, or this verify gate fails.)
python3 scripts/mechanism_provenance.py --verify
python3 scripts/mechanism_provenance.py --selftest >/dev/null
python3 scripts/design_audit.py --scenario mechanism_source_hash_mismatch_fails_release >"$tmp_dir/mech_hash.json"
python3 scripts/design_audit.py --scenario unsigned_mechanism_source_change_blocks >"$tmp_dir/mech_unsigned.json"
python3 scripts/design_audit.py --scenario signed_mechanism_preserving_change_accepts >"$tmp_dir/mech_preserve.json"
python3 scripts/design_audit.py --scenario signed_mechanism_weakening_change_blocks_by_probe >"$tmp_dir/mech_weaken.json"
python3 scripts/design_audit.py --scenario policy_artifact_clean_but_gate_code_weakened_fails >"$tmp_dir/mech_policy_clean.json"
grep -q '"mechanism_source": true' "$tmp_dir/mech_hash.json"
grep -q '"trace_provenance": "stale_pre_image"' "$tmp_dir/mech_hash.json"
grep -q '"effect_authority": "delta_provenance_unverified"' "$tmp_dir/mech_hash.json"
grep -q '"governance_decision": "block"' "$tmp_dir/mech_hash.json"
grep -q '"mechanism_source": true' "$tmp_dir/mech_unsigned.json"
grep -q '"signature_status": "unsigned"' "$tmp_dir/mech_unsigned.json"
grep -q '"effect_authority": "change_signature_unverified"' "$tmp_dir/mech_unsigned.json"
grep -q '"governance_decision": "block"' "$tmp_dir/mech_unsigned.json"
grep -q '"mechanism_source": true' "$tmp_dir/mech_preserve.json"
grep -q '"mechanism_role": "adjudicator"' "$tmp_dir/mech_preserve.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/mech_preserve.json"
grep -q '"trace_regressed": false' "$tmp_dir/mech_preserve.json"
grep -q '"governance_decision": "accept"' "$tmp_dir/mech_preserve.json"
grep -q '"proposal_consolidated": true' "$tmp_dir/mech_preserve.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/mech_weaken.json"
grep -q '"trace_regressed": true' "$tmp_dir/mech_weaken.json"
grep -q '"effect_authority": "trace_behavior_regression"' "$tmp_dir/mech_weaken.json"
grep -q '"governance_decision": "block"' "$tmp_dir/mech_weaken.json"
grep -q '"signature_status": "signature_verified"' "$tmp_dir/mech_policy_clean.json"
grep -q '"trace_regressed": true' "$tmp_dir/mech_policy_clean.json"
grep -q '"effect_authority": "trace_behavior_regression"' "$tmp_dir/mech_policy_clean.json"
grep -q '"governance_decision": "block"' "$tmp_dir/mech_policy_clean.json"
# The mechanism-source manifest binds path + content hash + role; the project audit gates on it.
grep -q '"schema": "mechanism-source-manifest-v0.1"' simulations/bridge_world/mechanism_source_manifest.json
grep -q '"content_hash"' simulations/bridge_world/mechanism_source_manifest.json
python3 scripts/decision_audit.py --project >"$tmp_dir/mech_project.json"
grep -q '"mechanism_source_binding": "verified"' "$tmp_dir/mech_project.json"
# No private signing key may ever be committed; only the public key is in the registry.
# (Exclude the gate scripts, which legitimately contain the pattern string as a grep argument.)
test "$(grep -rl --exclude=release_check.sh --exclude=test.sh 'BEGIN PRIVATE KEY\|BEGIN OPENSSH PRIVATE KEY' simulations scripts | wc -l)" -eq 0
grep -q '"signers"' simulations/bridge_world/authorized_design_signers.json
for packet in system_state_packet intent_packet retrieval_request retrieval_result contradiction_packet backpressure_command plan_proposal action_command action_outcome memory_mutation claim_packet evidence_packet episode_packet raw_episode_packet semantic_candidate_packet rule_packet human_promotion_packet plan_regret_packet attention_mode_review_packet; do
  test -f "schemas/cip/$packet.schema.json"
done
