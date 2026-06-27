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
# here. Crate-wide: exactly 18 live doctests, 9 `compile fail`, one per inert type (HypothesisPacket,
# RecommendedProbe, ProbeRequest, ProbeQueue, ReviewReceipt, ReviewLog, ProbeExecutionIntent,
# ProbeObservationReceipt, PromotionRequest). (Residual: a decoy compile_fail deliberately planted on the
# same type while the real one is commented out is review-evident insider forgery, beyond regression scope.)
_doc_out="$(cargo test --offline --doc --manifest-path crates/hypothesis-layer/Cargo.toml 2>/dev/null)"
test "$(printf '%s\n' "$_doc_out" | grep -oE 'running [0-9]+ tests' | grep -oE '[0-9]+')" -eq 18
test "$(printf '%s\n' "$_doc_out" | grep -c 'compile fail')" -eq 9
printf '%s\n' "$_doc_out" | grep -q 'HypothesisPacket (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'RecommendedProbe (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeRequest (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeQueue (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ReviewReceipt (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ReviewLog (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeExecutionIntent (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'ProbeObservationReceipt (line.*compile fail'
printf '%s\n' "$_doc_out" | grep -q 'PromotionRequest (line.*compile fail'
# Likewise, the UNIT tests must actually RUN, not be silently disabled: an `#[ignore]` (or a cfg-out /
# commented-out `#[test]`) skips a test without failing `cargo test`, so a test-name grep cannot tell an
# enforced policy from a disabled one. We pin the test reality from cargo: the crate's library unit tests
# must report EXACTLY the expected passed count and ZERO ignored — ignoring or removing any test lowers the
# passed count and/or raises the ignored count and fails here. (Update the count when adding/removing tests.)
_unit_out="$(cargo test --offline --lib --manifest-path crates/hypothesis-layer/Cargo.toml 2>/dev/null)"
test "$(printf '%s\n' "$_unit_out" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')" -eq 75
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
# ---------------------------------------------------------------------------------------------------
# HYP-4 / Observation Receipt Quarantine (crates/hypothesis-layer/src/observation.rs). Defines the
# quarantine FORMAT for a future probe result: a ProbeObservationReceipt derived from a HYP-3
# ProbeExecutionIntent that records "something was observed" WITHOUT letting it become evidence, a claim,
# verifier input, or a memory mutation — and without implying the probe ran. The disposition is DERIVED
# from the intent: a not_executed/blocked intent yields `rejected`, a requires_operator intent yields
# `requires_review`, and NO intent yields `recorded` (the FUTURE-reserved promotion target) — so at HYP-4
# nothing can be recorded; the quarantine holds until a future verifier/governance promotion path exists.
# Every observation carries authority `observation_only` (a single-variant enum). A receipt is minted ONLY
# by ProbeObservationReceipt::from_intent (private fields, no Deserialize — compiler-enforced and pinned
# LIVE by the cargo doctest-reality check above), so a forged `recorded` observation cannot be hand-set or
# deserialized off the wire; it carries an integrity_hash over all fields and reuses the FORBIDDEN_USES
# quarantine so it can never become evidence. The crate-wide no-execution / no-float / no-wall-clock / no-IO
# / no-#[allow] scans and the serde-only quarantine cargo-tree above already cover observation.rs and the
# new example. Doctrine: Hypothesis proposes. Probe queue classifies. Governance reviews. HYP-3 records
# intent. HYP-4 quarantines observations. Nothing becomes evidence.
# ---------------------------------------------------------------------------------------------------
# Observation-receipt structure present (signals).
grep -q 'enum ObservationStatus' crates/hypothesis-layer/src/observation.rs
grep -q 'enum ObservationAuthority' crates/hypothesis-layer/src/observation.rs
grep -q 'struct ProbeObservationReceipt' crates/hypothesis-layer/src/observation.rs
grep -q 'pub fn from_intent' crates/hypothesis-layer/src/observation.rs
grep -q 'observation_only' crates/hypothesis-layer/src/observation.rs
grep -q 'integrity_hash' crates/hypothesis-layer/src/observation.rs
# ENCAPSULATION (compiler-enforced): a ProbeObservationReceipt is minted ONLY by from_intent, is read-only
# (private fields), and is NOT deserializable — proven by the compile_fail doctest whose LIVE presence is
# pinned by the cargo doctest-reality check above (the existence grep below cannot be dodged by a
# `//`-commented copy because that copy drops out of cargo's doctest run). Plus private-fields and whole-file
# manual-`impl Deserialize` scans for the inert output type and its derived (Serialize-only) enums.
grep -q 'let _: hypothesis_layer::ProbeObservationReceipt = serde_json::from_str' crates/hypothesis-layer/src/observation.rs
test "$(awk '/pub struct ProbeObservationReceipt \{/,/^\}/' crates/hypothesis-layer/src/observation.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+(ProbeObservationReceipt|ObservationStatus|ObservationAuthority)\b' crates/hypothesis-layer/src/observation.rs)" -eq 0
# The quarantine is COMPILER/test-enforced, not prose: from_execution_status matches the intent disposition
# exhaustively with NO wildcard (E0004 on a new ExecutionStatus), no arm yields Recorded, and the
# single-variant ObservationAuthority is matched with no wildcard (E0004 on a 2nd variant). These tests (run
# by cargo test above) prove not_executed/blocked intents cannot record, a requires_operator intent requires
# review, NO disposition yields recorded, the observation is observation_only, it can't be evidence, and it
# changes neither P12 nor a verifier receipt.
grep -q 'fn not_executed_intent_cannot_record_observation' crates/hypothesis-layer/src/observation.rs
grep -q 'fn blocked_intent_cannot_record_observation' crates/hypothesis-layer/src/observation.rs
grep -q 'fn requires_operator_intent_requires_review' crates/hypothesis-layer/src/observation.rs
grep -q 'fn no_intent_disposition_yields_recorded' crates/hypothesis-layer/src/observation.rs
grep -q 'fn observation_authority_has_exactly_one_variant' crates/hypothesis-layer/src/observation.rs
grep -q 'fn observation_cannot_be_evidence' crates/hypothesis-layer/src/observation.rs
grep -q 'fn observation_does_not_change_training_gate' crates/hypothesis-layer/src/observation.rs
grep -q 'fn observation_does_not_change_verifier_receipt' crates/hypothesis-layer/src/observation.rs
# The end-to-end determinism smoke must EXERCISE the real API: the example CALLS from_intent.
grep -q 'ProbeObservationReceipt::from_intent' crates/hypothesis-layer/examples/observation_receipt_report.rs
# End-to-end determinism smoke: the demo observation set is a pure function of fixed inputs, so two runs are
# byte-identical (replay reproduces the observations). The observations array is the least-fabricable
# behavioral surface SHORT OF editing the example source: each element is a serialized REAL
# ProbeObservationReceipt (a private, non-deserializable type minted only by from_intent), so a forged status
# cannot be injected off-API — the reachable status tokens and the observation_only authority appear ONLY
# because the fixed intents really derived them. A not_executed/blocked intent becoming recordable or a
# requires_operator intent losing its review requirement drops the matching token here and fails the gate,
# independent of (and in a different file from) the unit tests — so gutting a unit-test body cannot hide a
# real ->recorded regression; this behavioral channel still catches it. (Residual, explicitly scoped out
# since HYP-0: fabricating the ENTIRE example output as literal JSON AND gutting every covering unit-test
# body is review-evident multi-file insider forgery, beyond regression scope.)
cargo build --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml --example observation_receipt_report >/dev/null 2>&1
_or_dir="$(mktemp -d)"
./target/debug/examples/observation_receipt_report > "$_or_dir/run1.json" 2>/dev/null
./target/debug/examples/observation_receipt_report > "$_or_dir/run2.json" 2>/dev/null
if ! cmp -s "$_or_dir/run1.json" "$_or_dir/run2.json"; then rm -rf "$_or_dir"; exit 1; fi
grep -q '"observation_status": "rejected"' "$_or_dir/run1.json"
grep -q '"observation_status": "requires_review"' "$_or_dir/run1.json"
grep -q '"authority": "observation_only"' "$_or_dir/run1.json"
# THE QUARANTINE, enforced behaviorally: no observation is `recorded` at HYP-4. The real serialized
# observations array must contain NO `recorded` status token, and recorded == 0.
if grep -q '"observation_status": "recorded"' "$_or_dir/run1.json"; then rm -rf "$_or_dir"; exit 1; fi
grep -q '"recorded": 0' "$_or_dir/run1.json"
# BEHAVIORAL quarantine backstop (a SECONDARY channel to the unit tests + the observations array above): the
# example RUNS the real from_intent() on each boundary path and emits these. If a not_executed/blocked intent
# became recordable, a requires_operator intent stopped requiring review, or ANY observation became recorded,
# these flip and the gate fails.
grep -q '"policy_not_executed_rejected": true' "$_or_dir/run1.json"
grep -q '"policy_blocked_rejected": true' "$_or_dir/run1.json"
grep -q '"policy_requires_operator_requires_review": true' "$_or_dir/run1.json"
grep -q '"policy_no_recorded_at_hyp4": true' "$_or_dir/run1.json"
rm -rf "$_or_dir"
# ---------------------------------------------------------------------------------------------------
# HYP-5 / Observation Promotion Gate — Still-No-Evidence Boundary (crates/hypothesis-layer/src/promotion.rs).
# Converts a HYP-4 ProbeObservationReceipt into an inert, deterministic PromotionRequest that RECORDS a
# request to promote a quarantined observation toward a claim/evidence/memory_note — WITHOUT promoting
# anything to evidence, mutating anything, or implying the probe ran. The outcome is machine-checkable and
# DERIVED from the observation and the requested target: a rejected/requires_review observation yields a
# `rejected` request (for ANY target), and the FUTURE-reserved `recorded` observation yields
# `requires_verifier` (claim/evidence) or `unsupported` (memory_note, which the layer may never write). NO
# path grants a promotion — PromotionStatus::grants_promotion is exhaustive and ALWAYS false at HYP-5, so a
# future promoting status cannot be added without an explicit, review-evident change. A request is minted
# ONLY by PromotionRequest::from_observation (private fields, no Deserialize — compiler-enforced and pinned
# LIVE by the cargo doctest-reality check above; the requested_target INPUT enum is deserializable, but the
# derived PromotionStatus/PromotionReason are Serialize-only, which keeps the request non-deserializable),
# so a forged "promoted" request cannot be hand-set or deserialized off the wire; it carries an
# integrity_hash over all fields and reuses the FORBIDDEN_USES quarantine so it can never become evidence.
# The crate-wide no-execution / no-float / no-wall-clock / no-IO / no-#[allow] scans and the serde-only
# quarantine cargo-tree above already cover promotion.rs and the new example. Doctrine: Hypothesis
# proposes. Probe queue classifies. Governance reviews. HYP-3 records intent. HYP-4 quarantines
# observations. HYP-5 records promotion requests. Nothing becomes evidence.
# ---------------------------------------------------------------------------------------------------
# Promotion-request structure present (signals).
grep -q 'enum PromotionTarget' crates/hypothesis-layer/src/promotion.rs
grep -q 'enum PromotionStatus' crates/hypothesis-layer/src/promotion.rs
grep -q 'enum PromotionReason' crates/hypothesis-layer/src/promotion.rs
grep -q 'struct PromotionRequest' crates/hypothesis-layer/src/promotion.rs
grep -q 'pub fn from_observation' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn grants_promotion' crates/hypothesis-layer/src/promotion.rs
grep -q 'integrity_hash' crates/hypothesis-layer/src/promotion.rs
# ENCAPSULATION (compiler-enforced): a PromotionRequest is minted ONLY by from_observation, is read-only
# (private fields), and is NOT deserializable — proven by the compile_fail doctest whose LIVE presence is
# pinned by the cargo doctest-reality check above (the existence grep below cannot be dodged by a
# `//`-commented copy because that copy drops out of cargo's doctest run). Plus private-fields and whole-file
# manual-`impl Deserialize` scans for the inert output type and its DERIVED (Serialize-only) enums — note
# PromotionTarget is the deserializable INPUT enum and is deliberately NOT in this set.
grep -q 'let _: hypothesis_layer::PromotionRequest = serde_json::from_str' crates/hypothesis-layer/src/promotion.rs
test "$(awk '/pub struct PromotionRequest \{/,/^\}/' crates/hypothesis-layer/src/promotion.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for[[:space:]]+(PromotionRequest|PromotionStatus|PromotionReason)\b' crates/hypothesis-layer/src/promotion.rs)" -eq 0
# Correct-if 1 — from_observation is the SOLE minting path. from_observation always DERIVES a NON-granting
# status, so the ONLY way to mint a granting request is to ADD a second construction site. Because the crate
# is `#![forbid(unsafe_code)]` (no transmute), PromotionRequest has NO Deserialize, and its fields are
# private with no setter, the ONLY way to construct one is a literal `PromotionRequest { ... }` — whatever
# the enclosing function's return type (PromotionRequest, Option<…>, a tuple, Self, or a fresh type alias).
# So we pin the count of that construction literal: exactly 5 occurrences (the `struct` def, the `impl`
# header, from_observation's `-> PromotionRequest {` body brace, and its TWO real constructions `let base =
# PromotionRequest {` and the `..base` return). ANY added construction site raises this to >= 6 and fails
# here, so a backdoor constructor of any return-type shape cannot be added without ALSO editing this gate.
# (Residual, scoped out since HYP-0: a token-obfuscating macro that emits the struct without the literal
# text, plus editing this count, is review-evident multi-file insider forgery, beyond regression scope — the
# structural quarantine defends against off-wire forgery and accidental regression, not against an insider
# who also rewrites the gate.)
test "$(grep -cE 'PromotionRequest \{' crates/hypothesis-layer/src/promotion.rs)" -eq 5
# The still-no-evidence boundary is COMPILER/test-enforced, not prose: the reason derivation matches the
# observation disposition exhaustively with NO wildcard (E0004 on a new ObservationStatus), the status<-reason
# map is likewise exhaustive (E0004 on a new reason), and grants_promotion matches every status with no
# wildcard returning false (a future promoting variant forces an explicit `true` → E0004). These tests (run
# by cargo test above) prove a rejected/requires_review observation cannot promote, a recorded observation
# only defers to a future verifier, no path grants evidence authority, and a request changes neither P12 nor
# a verifier receipt.
grep -q 'fn rejected_observation_cannot_promote' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn requires_review_observation_cannot_promote' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn recorded_observation_requires_future_verifier' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn promotion_never_yields_evidence_authority' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn promotion_preserves_forbidden_uses' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn promotion_does_not_change_training_gate' crates/hypothesis-layer/src/promotion.rs
grep -q 'fn promotion_does_not_change_verifier_receipt' crates/hypothesis-layer/src/promotion.rs
# The end-to-end determinism smoke must EXERCISE the real API: the example CALLS from_observation.
grep -q 'PromotionRequest::from_observation' crates/hypothesis-layer/examples/promotion_request_report.rs
# End-to-end determinism smoke: the demo request set is a pure function of fixed inputs, so two runs are
# byte-identical (replay reproduces the requests). The requests array is the least-fabricable behavioral
# surface SHORT OF editing the example source: each element is a serialized REAL PromotionRequest (a private,
# non-deserializable type minted only by from_observation), so a forged status cannot be injected off-API —
# the `rejected` status, all three requested targets, and the two reachable reason tokens appear ONLY because
# the fixed observations really derived them. A non-promotable observation becoming promotable, or any
# request granting a promotion, drops a token / raises `promoted` here and fails the gate, independent of (and
# in a different file from) the unit tests — so gutting a unit-test body cannot hide a real grant; this
# behavioral channel still catches it. (Residual, explicitly scoped out since HYP-0: fabricating the ENTIRE
# example output as literal JSON AND gutting every covering unit-test body is review-evident multi-file
# insider forgery, beyond regression scope.)
cargo build --offline --quiet --manifest-path crates/hypothesis-layer/Cargo.toml --example promotion_request_report >/dev/null 2>&1
_pr_dir="$(mktemp -d)"
./target/debug/examples/promotion_request_report > "$_pr_dir/run1.json" 2>/dev/null
./target/debug/examples/promotion_request_report > "$_pr_dir/run2.json" 2>/dev/null
if ! cmp -s "$_pr_dir/run1.json" "$_pr_dir/run2.json"; then rm -rf "$_pr_dir"; exit 1; fi
grep -q '"status": "rejected"' "$_pr_dir/run1.json"
grep -q '"requested_target": "claim"' "$_pr_dir/run1.json"
grep -q '"requested_target": "evidence"' "$_pr_dir/run1.json"
grep -q '"requested_target": "memory_note"' "$_pr_dir/run1.json"
grep -q '"reason_code": "observation_rejected_not_promotable"' "$_pr_dir/run1.json"
grep -q '"reason_code": "observation_requires_review_not_promotable"' "$_pr_dir/run1.json"
# THE STILL-NO-EVIDENCE BOUNDARY, enforced behaviorally: nothing is promoted at HYP-5. The real serialized
# requests array must contain NO granting status token, and promoted == 0.
if grep -qE '"status": "(promoted|granted|evidence)"' "$_pr_dir/run1.json"; then rm -rf "$_pr_dir"; exit 1; fi
grep -q '"promoted": 0' "$_pr_dir/run1.json"
# BEHAVIORAL still-no-evidence backstop (a SECONDARY channel to the unit tests + the requests array above):
# the example RUNS the real from_observation() on each boundary path — including the evidence-target path,
# the exact "observation exists therefore evidence" leak — and emits these. If a non-promotable observation
# became promotable, or an evidence target were granted, these flip and the gate fails.
grep -q '"policy_rejected_observation_not_promoted": true' "$_pr_dir/run1.json"
grep -q '"policy_requires_review_observation_not_promoted": true' "$_pr_dir/run1.json"
grep -q '"policy_evidence_target_not_granted": true' "$_pr_dir/run1.json"
grep -q '"policy_no_promotion_at_hyp5": true' "$_pr_dir/run1.json"
rm -rf "$_pr_dir"
# ---------------------------------------------------------------------------------------------------
# HYP-6 — hypothesis track milestone freeze. The HYP-0 -> HYP-5 arc is frozen as hypothesis-track-v0.1. The
# milestone record (HYPOTHESIS_TRACK_MILESTONE.md) pins the commit lineage, the authority boundary that holds
# across the arc, the P12 training verdict, and the honest residuals, and is locked here so the freeze cannot
# silently drift. The pinned commit hashes are auditable against `git log`. Documentation freeze only — no
# code crate changes, no model, no training; the milestone records training_not_justified.
# ---------------------------------------------------------------------------------------------------
test -f HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'FROZEN' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'hypothesis-track-v0.1' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'HYP-0' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'HYP-5' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'training_not_justified' HYPOTHESIS_TRACK_MILESTONE.md
# Full HYP commit lineage (HYP-0..HYP-5) + the charter status snapshot are pinned (cross-checkable against git log).
grep -q 'f19a998' HYPOTHESIS_TRACK_MILESTONE.md
grep -q '4b47736' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'cb68a73' HYPOTHESIS_TRACK_MILESTONE.md
grep -q '6cbb3a8' HYPOTHESIS_TRACK_MILESTONE.md
grep -q '7703e2e' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'cef91db' HYPOTHESIS_TRACK_MILESTONE.md
grep -q 'd899a61' HYPOTHESIS_TRACK_MILESTONE.md
# ---------------------------------------------------------------------------------------------------
# INT-0 — End-to-End Prototype Trace Demo (crates/cognitive-demo). The FIRST integration layer: a thin,
# deterministic demo that CONSUMES the two frozen tracks through their PUBLIC APIs and records ONE auditable
# CognitiveTrace connecting a VERIFIED reading receipt to the full hypothesis chain
# (hypothesis -> probe -> review -> intent -> observation -> promotion-refusal). It adds NO capability and
# grants NO authority: it reads the receipt by HASH (an EvidenceRef, never a memory handle), proposes,
# classifies, reviews, records intent, quarantines, and REFUSES promotion. The trace is a TYPED, REPLAYABLE,
# VERIFIER-CHECKED record (not a hidden chain-of-thought): a pure function of fixed inputs, so two runs are
# byte-identical, and every refusal is proven from the trace's OWN serialized output. No probe executes, no
# observation becomes evidence, no memory is mutated, and the P12 verdict stays training_not_justified.
# Doctrine: Reading verifies. Hypothesis proposes. Probe queue classifies. Governance reviews. Execution
# intent records. Observation quarantines. Promotion refuses. Nothing becomes evidence. Nothing trains.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/cognitive-demo/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/cognitive-demo/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Trace-type structure present (signals).
grep -q 'struct CognitiveTrace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn demo' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn build' crates/cognitive-demo/src/lib.rs
# ENCAPSULATION: the trace is an inert RECORD — minted ONLY by demo/build, read-only (private fields), and
# NOT deserializable (it is never read back as authority). The word `Deserialize` never appears in the demo
# source (the trace derives Serialize only), and the struct exposes no `pub` field, so a forged/mutated trace
# cannot enter the system or be hand-set to claim an execution / evidence promotion / opened training gate.
# The two greps catch the ONLY ways the type could become deserializable — a `derive(...Deserialize...)` or a
# manual `impl Deserialize for ...` — and are immune to the prose "NOT Deserialize" in the doc comment.
test "$(grep -cE 'derive\([^)]*Deserialize' crates/cognitive-demo/src/lib.rs)" -eq 0
test "$(grep -cE 'impl([[:space:]]|<).*Deserialize.*for' crates/cognitive-demo/src/lib.rs)" -eq 0
test "$(awk '/pub struct CognitiveTrace \{/,/^\}/' crates/cognitive-demo/src/lib.rs | grep -cE '^[[:space:]]+pub ')" -eq 0
# The trace REALLY wires the two frozen tracks (not a hardcoded JSON): it starts from a read0 receipt
# (produce_run) that is VERIFIED (verify_file), then walks the real hypothesis chain end to end. If any of
# these calls were dropped (faking the trace), the grep trips.
grep -q 'produce_run(' crates/cognitive-demo/src/lib.rs
grep -q 'verify_file(' crates/cognitive-demo/src/lib.rs
grep -q 'propose(' crates/cognitive-demo/src/lib.rs
grep -q 'ProbeRequest::from_hypothesis' crates/cognitive-demo/src/lib.rs
grep -q 'ReviewReceipt::decide' crates/cognitive-demo/src/lib.rs
grep -q 'ProbeExecutionIntent::from_review' crates/cognitive-demo/src/lib.rs
grep -q 'ProbeObservationReceipt::from_intent' crates/cognitive-demo/src/lib.rs
grep -q 'PromotionRequest::from_observation' crates/cognitive-demo/src/lib.rs
# The 10 INT-0 first-tests + the no-new-authority backstop exist (a gutted/deleted test drops the unit count
# pinned below; these name-greps additionally pin WHICH behaviours are covered).
grep -q 'fn end_to_end_trace_replays' crates/cognitive-demo/src/lib.rs
grep -q 'fn trace_starts_from_verified_reading_receipt' crates/cognitive-demo/src/lib.rs
grep -q 'fn hypothesis_cites_receipt_hash' crates/cognitive-demo/src/lib.rs
grep -q 'fn probe_request_is_inert' crates/cognitive-demo/src/lib.rs
grep -q 'fn review_does_not_execute' crates/cognitive-demo/src/lib.rs
grep -q 'fn execution_intent_is_not_executed' crates/cognitive-demo/src/lib.rs
grep -q 'fn observation_is_quarantined' crates/cognitive-demo/src/lib.rs
grep -q 'fn promotion_request_does_not_promote' crates/cognitive-demo/src/lib.rs
grep -q 'fn trace_does_not_change_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn trace_does_not_change_verifier_receipt' crates/cognitive-demo/src/lib.rs
grep -q 'fn trace_records_every_stage_id_and_links_the_chain' crates/cognitive-demo/src/lib.rs
grep -q 'fn trace_grants_no_new_authority' crates/cognitive-demo/src/lib.rs
# Unit-test REALITY pin: exactly the 394 = INT-0 (12) + INT-1 (8) + INT-2 (12) + INT-3 (12) + MTRACE-0 (12) +
# MTRACE-1 (12) + MTRACE-2 (12) + DOCFLOW-0 (10) + DOCFLOW-2 (10) + CORPUS-0 (12) + CORPUS-2 (12) + NOVELTY-0 (15) +
# DREAM-EXPORT-0 (13) + DREAM-EXPORT-2 (15) + HORIZON-0 (23) + HORIZON-2 (16) + CORPUS-HARVEST-0 (26) + SCORE-0 (20) + FAIL-0 (19) + P11-MODEL-EVAL (18) + TRAIN-GATE-0 (20) + TRAIN-0 (21) + MODEL-EVAL-1 (22) + MODEL-PROMOTE-0 (23) + PROD-0 (19) tests pass, zero ignored (so gutting/disabling one is caught, independent of the channels below).
_int0_unit="$(cargo test --offline --lib --manifest-path crates/cognitive-demo/Cargo.toml 2>/dev/null)"
test "$(printf '%s\n' "$_int0_unit" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')" -eq 394
test "$(printf '%s\n' "$_int0_unit" | grep -oE '[0-9]+ ignored' | grep -oE '[0-9]+')" -eq 0
# Determinism / no side effects: the trace is a pure, in-memory function — no clock, entropy, or network
# anywhere in src/, and no floats anywhere in the crate. (`std::process::exit` in the CLI shell is a clean
# exit code, not nondeterminism, and is intentionally not matched here.)
test "$(grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|tokio|\.await|reqwest' crates/cognitive-demo/src | wc -l)" -eq 0
test "$(grep -rE '\bf32\b|\bf64\b' crates/cognitive-demo/src | wc -l)" -eq 0
# NO PROBE EXECUTION / NO NETWORK anywhere (src + example): no process spawn, no socket — a live executor
# (even one driven off the recorded probe text) leaves the deterministic trace unchanged, so the double-run
# cannot catch it; this recursive scan does.
test "$(grep -rE 'Command::new|process::Command|\.spawn\(|std::net|TcpStream|UdpSocket' crates/cognitive-demo/src crates/cognitive-demo/examples | wc -l)" -eq 0
# fs is CONFINED to the binary I/O shell (src/main.rs): the trace CORE (lib.rs) and the example are PURE and
# touch NO filesystem, so the trace RESULT can never depend on disk — only the operator CLI reads/writes files.
# (INT-1 adds the CLI; the library that builds/verifies the trace stays fs-free.)
test "$(grep -rE 'std::fs|File::create|File::open|fs::write|fs::read|OpenOptions' crates/cognitive-demo/src/lib.rs crates/cognitive-demo/examples | wc -l)" -eq 0
# No model is trained or loaded: the demo manifest pulls no ML/inference/training framework.
test "$(grep -riE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/cognitive-demo/Cargo.toml | wc -l)" -eq 0
# Separation: cognitive-demo INTEGRATES the two frozen tracks (reading-cli + hypothesis-layer in its tree) and
# touches NO vibe engine crate (fails closed if cargo tree cannot run, so the boundary proof is never vacuous).
test "$(cargo tree --offline --manifest-path crates/cognitive-demo/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-')" -eq 0
test "$(cargo tree --offline --manifest-path crates/cognitive-demo/Cargo.toml --edges normal 2>/dev/null | grep -c 'reading-cli')" -ge 1
test "$(cargo tree --offline --manifest-path crates/cognitive-demo/Cargo.toml --edges normal 2>/dev/null | grep -c 'hypothesis-layer')" -ge 1
# The example must EXERCISE the real API: it calls CognitiveTrace::demo (not a hardcoded string).
grep -q 'CognitiveTrace::demo' crates/cognitive-demo/examples/cognitive_trace_demo.rs
# End-to-end determinism smoke: the demo trace is a pure function of fixed inputs, so two runs are
# byte-identical (replay reproduces the end-to-end trace).
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --example cognitive_trace_demo >/dev/null 2>&1
_int0_dir="$(mktemp -d)"
./target/debug/examples/cognitive_trace_demo > "$_int0_dir/run1.json" 2>/dev/null
./target/debug/examples/cognitive_trace_demo > "$_int0_dir/run2.json" 2>/dev/null
if ! cmp -s "$_int0_dir/run1.json" "$_int0_dir/run2.json"; then rm -rf "$_int0_dir"; exit 1; fi
# THE END-TO-END BOUNDARY, enforced behaviorally from the trace's OWN serialized output: the trace starts
# from a VERIFIED reading receipt, the hypothesis cites it by hash, the chain is linked, the approved probe is
# NOT executed (requires_operator, never `executed`), the observation is quarantined (requires_review, never
# `recorded`), the promotion to evidence is REFUSED (rejected, grants nothing), nothing becomes evidence, and
# the P12 verdict is unmoved (training_justified=false). A real regression flips one of these and fails here.
grep -q '"reading_passed": true' "$_int0_dir/run1.json"
grep -q '"starts_from_verified_receipt": true' "$_int0_dir/run1.json"
grep -q '"hypothesis_cites_receipt": true' "$_int0_dir/run1.json"
grep -q '"chain_linked": true' "$_int0_dir/run1.json"
grep -q '"review_decision": "approved"' "$_int0_dir/run1.json"
grep -q '"execution_status": "requires_operator"' "$_int0_dir/run1.json"
grep -q '"observation_status": "requires_review"' "$_int0_dir/run1.json"
grep -q '"promotion_status": "rejected"' "$_int0_dir/run1.json"
grep -q '"grants_promotion": false' "$_int0_dir/run1.json"
grep -q '"promotion_refused": true' "$_int0_dir/run1.json"
grep -q '"nothing_becomes_evidence": true' "$_int0_dir/run1.json"
grep -q '"training_justified": false' "$_int0_dir/run1.json"
grep -q '"training_gate_unchanged": true' "$_int0_dir/run1.json"
# NO-GRANT GUARD (precise, so the legitimate requested target `"promotion_target": "evidence"` — a REQUEST
# that is refused — never false-positives): no STATUS field may ever read executed/promoted/granted/recorded,
# and the boolean grants must never be true. A future leak (a status promoting, a grant flipping) trips this.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|promoted|granted|recorded)"' "$_int0_dir/run1.json"; then rm -rf "$_int0_dir"; exit 1; fi
if grep -q '"grants_promotion": true' "$_int0_dir/run1.json"; then rm -rf "$_int0_dir"; exit 1; fi
if grep -q '"training_justified": true' "$_int0_dir/run1.json"; then rm -rf "$_int0_dir"; exit 1; fi
rm -rf "$_int0_dir"
# ---------------------------------------------------------------------------------------------------
# INT-1 — End-to-End Trace CLI / Operator Report (crates/cognitive-demo, the `cognitive-demo` binary). A thin,
# deterministic operator surface over the INT-0 trace: `trace` writes the canonical CognitiveTrace JSON,
# `report` renders a plain operator report, `replay` confirms a byte-identical reproduction. It adds NO
# authority and NO cognition — the binary is ONLY an I/O shell (std::fs lives in src/main.rs alone; the scan
# above proves the lib + example stay fs-free), and ALL logic stays in the pure library. Because the trace is
# Serialize-but-not-Deserialize, `report`/`replay` RE-DERIVE the canonical trace and REFUSE any provided file
# that is not byte-for-byte that trace, so a tampered/foreign trace can never be laundered into a report or a
# passing replay. Doctrine: Reading verifies. Hypothesis proposes. Probe queue classifies. Governance reviews.
# Execution intent records. Observation quarantines. Promotion refuses. Nothing becomes evidence. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# The pure CLI cores + report surface exist (signals).
grep -q 'pub fn run_trace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_report' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_replay' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_trace_json' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn to_report' crates/cognitive-demo/src/lib.rs
grep -q 'BOUNDARY_LINES' crates/cognitive-demo/src/lib.rs
# The report/replay TRUST BOUNDARY is real: verify_trace_json RE-DERIVES the canonical trace and COMPARES it —
# it does NOT deserialize the provided trace (CognitiveTrace stays Serialize-only, pinned above), so a mismatch
# is refused (TraceMismatch) and a forged file is never read back as authority.
grep -q 'CognitiveTrace::demo()' crates/cognitive-demo/src/lib.rs
grep -q 'TraceMismatch' crates/cognitive-demo/src/lib.rs
# The 8 INT-1 first-tests exist (a gutted/deleted test drops the unit count pinned above).
grep -q 'fn trace_command_writes_json' crates/cognitive-demo/src/lib.rs
grep -q 'fn report_command_writes_operator_summary' crates/cognitive-demo/src/lib.rs
grep -q 'fn report_contains_all_boundary_lines' crates/cognitive-demo/src/lib.rs
grep -q 'fn replay_reproduces_trace' crates/cognitive-demo/src/lib.rs
grep -q 'fn report_does_not_change_trace' crates/cognitive-demo/src/lib.rs
grep -q 'fn cli_does_not_execute_probe' crates/cognitive-demo/src/lib.rs
grep -q 'fn cli_does_not_change_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn cli_does_not_change_verifier_receipt' crates/cognitive-demo/src/lib.rs
# End-to-end BINARY smoke (the operator surface, with real files): build the CLI, then drive the three commands
# against temp files and prove (a) `trace` is deterministic byte-identical across runs and emits the real
# canonical content, (b) `report` shows every stage + all nine boundary lines + the explicit refusals and never
# claims an executed/promoted/granted status, (c) `replay` accepts the canonical trace, and (d) a TAMPERED trace
# is REFUSED by BOTH replay and report (so the CLI can never hide a failed boundary or launder a forged trace).
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo >/dev/null 2>&1
test -f crates/cognitive-demo/src/main.rs
_int1_dir="$(mktemp -d)"
./target/debug/cognitive-demo trace --out "$_int1_dir/t1.json" >/dev/null 2>&1
./target/debug/cognitive-demo trace --out "$_int1_dir/t2.json" >/dev/null 2>&1
if ! cmp -s "$_int1_dir/t1.json" "$_int1_dir/t2.json"; then rm -rf "$_int1_dir"; exit 1; fi
# The CLI trace emits the REAL canonical record (not a stub): the same machine-checkable markers the INT-0
# example double-run already validated.
grep -q '"promotion_status": "rejected"' "$_int1_dir/t1.json"
grep -q '"grants_promotion": false' "$_int1_dir/t1.json"
grep -q '"execution_status": "requires_operator"' "$_int1_dir/t1.json"
grep -q '"training_justified": false' "$_int1_dir/t1.json"
./target/debug/cognitive-demo report --trace "$_int1_dir/t1.json" --out "$_int1_dir/r.txt" >/dev/null 2>&1
# All seven stages appear in the report.
for _stage in '[1] READING' '[2] HYPOTHESIS' '[3] PROBE QUEUE' '[4] GOVERNANCE REVIEW' '[5] EXECUTION INTENT' '[6] OBSERVATION' '[7] PROMOTION REQUEST'; do
  if ! grep -qF "$_stage" "$_int1_dir/r.txt"; then rm -rf "$_int1_dir"; exit 1; fi
done
# All nine boundary lines appear, verbatim.
for _line in 'Reading verifies.' 'Hypothesis proposes.' 'Probe queue classifies.' 'Governance reviews.' 'Execution intent records.' 'Observation quarantines.' 'Promotion refuses.' 'Nothing becomes evidence.' 'Nothing trains.'; do
  if ! grep -qF "$_line" "$_int1_dir/r.txt"; then rm -rf "$_int1_dir"; exit 1; fi
done
# The report states the refusals explicitly, in prose, for a human.
grep -qF 'Nothing executed.' "$_int1_dir/r.txt"
grep -qF 'Nothing became evidence.' "$_int1_dir/r.txt"
grep -qF 'training_justified=false' "$_int1_dir/r.txt"
# The report never claims an executed/promoted/granted status value (it describes a record, never triggers one).
if grep -qE '(executed|promoted|granted)$' "$_int1_dir/r.txt"; then rm -rf "$_int1_dir"; exit 1; fi
# replay accepts the canonical trace...
./target/debug/cognitive-demo replay --trace "$_int1_dir/t1.json" >/dev/null 2>&1
# ...and a TAMPERED trace is refused by BOTH replay and report (exit non-zero — a failed boundary is not hideable).
sed 's/"grants_promotion": false/"grants_promotion": true/' "$_int1_dir/t1.json" > "$_int1_dir/tampered.json"
if ./target/debug/cognitive-demo replay --trace "$_int1_dir/tampered.json" >/dev/null 2>&1; then rm -rf "$_int1_dir"; exit 1; fi
if ./target/debug/cognitive-demo report --trace "$_int1_dir/tampered.json" >/dev/null 2>&1; then rm -rf "$_int1_dir"; exit 1; fi
rm -rf "$_int1_dir"
# ---------------------------------------------------------------------------------------------------
# INT-2 — Trace Question Harness / Operator Interrogation Surface (crates/cognitive-demo, the
# `cognitive-demo` binary's `ask` + `questions` commands). A deterministic, FINITE, enum-backed question
# surface over the canonical trace so an operator can ask what happened, what did not, and why authority
# was refused — with NO LLM, NO natural-language parser, NO new authority. A question is a `TraceQuestion`
# enum variant (an unknown slug fails closed → UnknownQuestion); an answer is PROSE formatted from the
# trace's own recorded fields; and `ask` RE-DERIVES the canonical trace and REFUSES any tampered/foreign
# input BEFORE answering, so a question can never become authority and a tampered trace can never be
# answered. Doctrine: Trace questions explain the trace. They do not create authority. They do not
# execute. They do not promote. They do not train.
# ---------------------------------------------------------------------------------------------------
# The interrogation surface exists (signals): the closed enum, its fail-closed parser, the two pure cores,
# and the INT-2 boundary data.
grep -q 'enum TraceQuestion' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn from_slug' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_ask' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn list_questions' crates/cognitive-demo/src/lib.rs
grep -q 'ASK_BOUNDARY_LINES' crates/cognitive-demo/src/lib.rs
grep -q 'UnknownQuestion' crates/cognitive-demo/src/lib.rs
# The fail-closed + re-derive ordering is real: run_ask parses the question via from_slug (UnknownQuestion
# on miss) and then verify_trace_json RE-DERIVES + byte-compares (TraceMismatch on a tampered file) — it
# never deserializes the provided trace (CognitiveTrace stays Serialize-only, pinned far above).
grep -q 'TraceQuestion::from_slug' crates/cognitive-demo/src/lib.rs
# The 12 INT-2 tests exist (a gutted/deleted test drops the unit count pinned above).
grep -q 'fn questions_command_lists_finite_question_set' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_refuses_unknown_question' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_refuses_tampered_trace' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_what_read_reports_receipt_hash' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_what_proven_reports_verified_reading_result' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_hypothesis_distinguishes_hypothesis_from_claim' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_execution_question_returns_no_execution' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_evidence_question_returns_no_evidence' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_training_question_returns_training_false' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_does_not_change_trace_or_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_answer_preserves_authority_boundary' crates/cognitive-demo/src/lib.rs
grep -q 'fn ask_answer_is_not_authority' crates/cognitive-demo/src/lib.rs
# End-to-end BINARY smoke for the interrogation surface (real files; the CLI was built in the INT-1 smoke):
# drive `questions` + `ask` against a freshly written canonical trace and prove (a) `questions` lists the
# finite set, (b) each answer is derived from the canonical trace (carries its real hashes) and ends with
# the INT-2 boundary, (c) the no/refusal answers never claim an executed/promoted/granted status, and
# (d) an UNKNOWN question and a TAMPERED trace are BOTH refused (exit non-zero, fail closed).
_int2_dir="$(mktemp -d)"
./target/debug/cognitive-demo trace --out "$_int2_dir/t.json" >/dev/null 2>&1
./target/debug/cognitive-demo questions > "$_int2_dir/q.txt" 2>/dev/null
for _slug in what-read what-was-proven what-was-hypothesized what-probe-was-requested was-anything-executed did-anything-become-evidence why-was-promotion-refused did-training-open; do
  if ! grep -qF "$_slug" "$_int2_dir/q.txt"; then rm -rf "$_int2_dir"; exit 1; fi
done
# `what-read` carries the canonical receipt's real answer_hash and ends with the INT-2 boundary.
_int2_ah="$(grep -oE '"reading_answer_hash": [0-9]+' "$_int2_dir/t.json" | grep -oE '[0-9]+')"
./target/debug/cognitive-demo ask --trace "$_int2_dir/t.json" --question what-read > "$_int2_dir/a_read.txt" 2>/dev/null
grep -qF "$_int2_ah" "$_int2_dir/a_read.txt"
grep -qF 'READING' "$_int2_dir/a_read.txt"
# All five INT-2 boundary lines appear in the answer, verbatim (a drift in any line trips here).
for _bl in 'Trace questions explain the trace.' 'They do not create authority.' 'They do not execute.' 'They do not promote.' 'They do not train.'; do
  if ! grep -qF "$_bl" "$_int2_dir/a_read.txt"; then rm -rf "$_int2_dir"; exit 1; fi
done
# The execution / evidence / promotion / training answers say No / refuse explicitly.
./target/debug/cognitive-demo ask --trace "$_int2_dir/t.json" --question was-anything-executed > "$_int2_dir/a_exec.txt" 2>/dev/null
grep -qF 'Nothing executed.' "$_int2_dir/a_exec.txt"
grep -qF 'requires_operator' "$_int2_dir/a_exec.txt"
./target/debug/cognitive-demo ask --trace "$_int2_dir/t.json" --question did-anything-become-evidence > "$_int2_dir/a_evid.txt" 2>/dev/null
grep -qF 'Nothing became evidence.' "$_int2_dir/a_evid.txt"
grep -qF 'rejected' "$_int2_dir/a_evid.txt"
./target/debug/cognitive-demo ask --trace "$_int2_dir/t.json" --question why-was-promotion-refused > "$_int2_dir/a_promo.txt" 2>/dev/null
grep -qF 'did not occur' "$_int2_dir/a_promo.txt"
./target/debug/cognitive-demo ask --trace "$_int2_dir/t.json" --question did-training-open > "$_int2_dir/a_train.txt" 2>/dev/null
grep -qF 'training_justified' "$_int2_dir/a_train.txt"
# NO-AUTHORITY guard over every answer: no answer line may end in an affirmative executed/promoted/granted/
# recorded status, and no grant may read true (the answers describe a record, they never affirm authority).
cat "$_int2_dir"/a_*.txt > "$_int2_dir/all_answers.txt"
if grep -qE ': (executed|promoted|granted|recorded)$' "$_int2_dir/all_answers.txt"; then rm -rf "$_int2_dir"; exit 1; fi
if grep -qE 'grants_promotion:[[:space:]]*true' "$_int2_dir/all_answers.txt"; then rm -rf "$_int2_dir"; exit 1; fi
# An UNKNOWN question fails closed (no answer, non-zero exit).
if ./target/debug/cognitive-demo ask --trace "$_int2_dir/t.json" --question explain-everything >/dev/null 2>&1; then rm -rf "$_int2_dir"; exit 1; fi
# A TAMPERED trace is refused BEFORE answering (non-zero exit — a tampered trace can never be answered).
sed 's/"grants_promotion": false/"grants_promotion": true/' "$_int2_dir/t.json" > "$_int2_dir/tampered.json"
if ./target/debug/cognitive-demo ask --trace "$_int2_dir/tampered.json" --question did-anything-become-evidence >/dev/null 2>&1; then rm -rf "$_int2_dir"; exit 1; fi
rm -rf "$_int2_dir"
# ---------------------------------------------------------------------------------------------------
# INT-3 — Prototype Demo Bundle / Operator Repro Pack (crates/cognitive-demo, the `cognitive-demo` binary's
# `bundle` + `bundle-verify` commands). One reproducible operator pack over the canonical trace: `bundle` writes
# a fixed set of files (trace.json, report.txt, questions.txt, manifest.json) PURELY derived from the canonical
# trace; `bundle-verify` RE-DERIVES every file and byte-compares, trusting NOTHING on disk. The manifest hashes
# every content file and records a replay proof + the six-line boundary. It adds NO authority and NO cognition —
# the bundle is a DEMONSTRATION: it creates no evidence, executes nothing, promotes nothing, trains nothing. The
# only filesystem I/O is the binary shell (src/main.rs); the library that derives/verifies the bundle stays pure,
# which the fs-confined scan above proves. Doctrine: The bundle demonstrates the prototype. It does not create
# evidence. It does not create authority. It does not execute. It does not promote. It does not train.
# ---------------------------------------------------------------------------------------------------
# The bundle surface exists (signals): the pure builder, the re-deriving verifier, the questions transcript, and
# the bundle data (file set + boundary), plus the two refusal error variants.
grep -q 'pub fn canonical_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_questions_doc' crates/cognitive-demo/src/lib.rs
grep -q 'BUNDLE_BOUNDARY_LINES' crates/cognitive-demo/src/lib.rs
grep -q 'BUNDLE_FILES' crates/cognitive-demo/src/lib.rs
grep -q 'BundleMismatch' crates/cognitive-demo/src/lib.rs
grep -q 'BundleMissingFile' crates/cognitive-demo/src/lib.rs
grep -q 'struct BundleManifest' crates/cognitive-demo/src/lib.rs
# Re-derive (not trust): verify_bundle compares the provided files against the RE-DERIVED canonical_bundle
# (which builds from the pure CognitiveTrace::demo path) via the shared compare_bundle core — it does NOT
# parse/deserialize the provided files (CognitiveTrace + the manifest stay Serialize-only, pinned far above).
grep -q 'compare_bundle(&canonical_bundle()?' crates/cognitive-demo/src/lib.rs
# compare_bundle is the re-derive-not-trust comparator: it returns BundleMismatch on any byte difference and
# BundleMissingFile on an absent file, never trusting provided content over the re-derivation.
grep -q 'fn compare_bundle(' crates/cognitive-demo/src/lib.rs
# The 12 INT-3 tests exist (a gutted/deleted test drops the unit count pinned above).
grep -q 'fn bundle_command_writes_all_expected_files' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_manifest_hashes_all_files' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_verify_rejects_tampered_trace' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_verify_rejects_tampered_report' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_verify_rejects_tampered_questions' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_verify_rejects_tampered_manifest' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_verify_rejects_missing_file' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_verify_rederives_canonical_trace' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_does_not_change_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_does_not_change_verifier_receipt' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_boundary_lines_present' crates/cognitive-demo/src/lib.rs
grep -q 'fn bundle_output_is_not_authority' crates/cognitive-demo/src/lib.rs
# End-to-end BINARY smoke (real files; the CLI was built in the INT-1 smoke): write a bundle, prove it is the
# full 4-file pack with a hashing manifest + boundary, is byte-identical across runs, verifies clean, and that a
# TAMPER of EACH file (trace/report/questions/manifest), a MISSING file, and a FOREIGN bundle are ALL refused.
_int3_dir="$(mktemp -d)"
./target/debug/cognitive-demo bundle --out "$_int3_dir/pack" >/dev/null 2>&1
# All four files are present.
for _f in trace.json report.txt questions.txt manifest.json; do
  if [ ! -f "$_int3_dir/pack/$_f" ]; then rm -rf "$_int3_dir"; exit 1; fi
done
# The manifest hashes every CONTENT file (names each + records a content_hash) and carries the six boundary lines.
grep -q '"name": "trace.json"' "$_int3_dir/pack/manifest.json"
grep -q '"name": "report.txt"' "$_int3_dir/pack/manifest.json"
grep -q '"name": "questions.txt"' "$_int3_dir/pack/manifest.json"
test "$(grep -c '"content_hash"' "$_int3_dir/pack/manifest.json")" -ge 3
# The hashes are CONTENT-DEPENDENT, not a constant stand-in: three distinct content files yield distinct hashes.
test "$(grep -oE '"content_hash": "[0-9a-f]+"' "$_int3_dir/pack/manifest.json" | sort -u | wc -l)" -ge 2
for _bl in 'The bundle demonstrates the prototype.' 'It does not create evidence.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_bl" "$_int3_dir/pack/manifest.json"; then rm -rf "$_int3_dir"; exit 1; fi
done
# Determinism: a second bundle is byte-identical, file for file (a pure function of fixed inputs).
./target/debug/cognitive-demo bundle --out "$_int3_dir/pack2" >/dev/null 2>&1
for _f in trace.json report.txt questions.txt manifest.json; do
  if ! cmp -s "$_int3_dir/pack/$_f" "$_int3_dir/pack2/$_f"; then rm -rf "$_int3_dir"; exit 1; fi
done
# NO-AUTHORITY guard over the bundle files: no content file shows an affirmative executed/promoted/granted status
# or a true grant, and the trace records training stays false (the precise status grep avoids the legitimate
# `"promotion_target": "evidence"` REQUEST).
if grep -rqE '"(execution_status|observation_status|promotion_status)": "(executed|promoted|granted|recorded)"' "$_int3_dir/pack"; then rm -rf "$_int3_dir"; exit 1; fi
if grep -rq '"grants_promotion": true' "$_int3_dir/pack"; then rm -rf "$_int3_dir"; exit 1; fi
grep -q '"training_justified": false' "$_int3_dir/pack/trace.json"
# bundle-verify accepts the canonical pack...
./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/pack" >/dev/null 2>&1
# ...and refuses a TAMPER of EACH file, a MISSING file, and a FOREIGN bundle (exit non-zero — no tamper passes).
cp -r "$_int3_dir/pack" "$_int3_dir/tt"; sed -i 's/"grants_promotion": false/"grants_promotion": true/' "$_int3_dir/tt/trace.json"
if ./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/tt" >/dev/null 2>&1; then rm -rf "$_int3_dir"; exit 1; fi
cp -r "$_int3_dir/pack" "$_int3_dir/tr"; printf '\nINJECTED\n' >> "$_int3_dir/tr/report.txt"
if ./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/tr" >/dev/null 2>&1; then rm -rf "$_int3_dir"; exit 1; fi
cp -r "$_int3_dir/pack" "$_int3_dir/tq"; sed -i 's/did not occur/DID occur/' "$_int3_dir/tq/questions.txt"
if ./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/tq" >/dev/null 2>&1; then rm -rf "$_int3_dir"; exit 1; fi
cp -r "$_int3_dir/pack" "$_int3_dir/tm"; sed -i 's/cognitive-bundle-v0.1/cognitive-bundle-v9.9/' "$_int3_dir/tm/manifest.json"
if ./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/tm" >/dev/null 2>&1; then rm -rf "$_int3_dir"; exit 1; fi
cp -r "$_int3_dir/pack" "$_int3_dir/tx"; rm "$_int3_dir/tx/questions.txt"
if ./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/tx" >/dev/null 2>&1; then rm -rf "$_int3_dir"; exit 1; fi
mkdir "$_int3_dir/tf"; for _f in trace.json report.txt questions.txt manifest.json; do echo foreign > "$_int3_dir/tf/$_f"; done
if ./target/debug/cognitive-demo bundle-verify --path "$_int3_dir/tf" >/dev/null 2>&1; then rm -rf "$_int3_dir"; exit 1; fi
rm -rf "$_int3_dir"
# ---------------------------------------------------------------------------------------------------
# INT-4 — integration track milestone freeze. The INT-0 -> INT-3 integration-demo arc (the cognitive-demo crate
# over the two frozen tracks) is frozen as integration-demo-v0.1. The milestone record
# (INTEGRATION_DEMO_MILESTONE.md) pins the commit lineage, the frozen dependency tracks, the
# output-not-authority boundary, the P12 training verdict, and the honest residuals, and is locked here so the
# freeze cannot silently drift. The pinned commit hashes are auditable against `git log`. Documentation freeze
# only — no code crate changes, no model, no training; the milestone records training_not_justified. Doctrine:
# The integration demo shows the prototype. The trace is output, not authority. The report is output, not
# authority. Questions explain the trace. The bundle demonstrates the prototype. Nothing executes. Nothing
# becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
test -f INTEGRATION_DEMO_MILESTONE.md
grep -q 'FROZEN' INTEGRATION_DEMO_MILESTONE.md
grep -q 'integration-demo-v0.1' INTEGRATION_DEMO_MILESTONE.md
grep -q 'INT-0' INTEGRATION_DEMO_MILESTONE.md
grep -q 'INT-3' INTEGRATION_DEMO_MILESTONE.md
grep -q 'training_not_justified' INTEGRATION_DEMO_MILESTONE.md
grep -q 'training_justified=false' INTEGRATION_DEMO_MILESTONE.md
# Full INT-0..INT-3 commit lineage is pinned (cross-checkable against git log).
grep -q '2330f7c' INTEGRATION_DEMO_MILESTONE.md
grep -q '92c0692' INTEGRATION_DEMO_MILESTONE.md
grep -q 'b5bcf66' INTEGRATION_DEMO_MILESTONE.md
grep -q 'f451c39' INTEGRATION_DEMO_MILESTONE.md
# The two frozen dependency tracks are referenced as frozen deps (tag + commit).
grep -q 'reading-track-v0.1' INTEGRATION_DEMO_MILESTONE.md
grep -q 'hypothesis-track-v0.1' INTEGRATION_DEMO_MILESTONE.md
grep -q 'f6fa55a' INTEGRATION_DEMO_MILESTONE.md
grep -q 'bb20acf' INTEGRATION_DEMO_MILESTONE.md
# The output-not-authority boundary is recorded verbatim (all nine lines).
for _bl in 'The integration demo shows the prototype.' 'The trace is output, not authority.' 'The report is output, not authority.' 'Questions explain the trace.' 'The bundle demonstrates the prototype.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" INTEGRATION_DEMO_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' INTEGRATION_DEMO_MILESTONE.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# MTRACE-0 — Multi-Trace Scenario Pack (crates/cognitive-demo, the `cognitive-demo` binary's `scenarios` +
# `scenario-pack` + `scenario-verify` commands). The SAME deterministic pipeline is run under several finite,
# enum-backed scenarios that vary ONLY the probe risk and the governance decision, producing several
# CognitiveTrace bundles — each proving the SAME authority boundary (no execution / no evidence / no promotion /
# no training) under a different review/observation/promotion outcome. `scenario_bundle`/`scenario_pack_manifest`
# are PURELY derived; `verify_scenario_bundle`/`verify_scenario_pack_manifest` RE-DERIVE and byte-compare,
# trusting nothing on disk. The happy-boundary scenario IS the frozen canonical demo trace, byte-for-byte.
# Doctrine: Scenarios vary the path. They do not vary the authority. Nothing executes. Nothing becomes evidence.
# Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# The scenario surface exists (signals): the finite enum, the parameterized builder, the scenario trace/bundle/
# pack builders, the re-deriving verifiers, and the boundary data.
grep -q 'pub enum Scenario' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn build_scenario' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn scenario_trace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn scenario_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_scenario_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn scenario_pack_manifest' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_scenario_pack_manifest' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn list_scenarios' crates/cognitive-demo/src/lib.rs
grep -q 'MTRACE_BOUNDARY_LINES' crates/cognitive-demo/src/lib.rs
# Re-derive (not trust): the scenario verifiers compare against the freshly re-derived scenario bundle / pack
# manifest via the same compare_bundle core (pinned above) and a byte-compare — they parse/trust no file.
grep -q 'compare_bundle(&scenario_bundle(scenario)?' crates/cognitive-demo/src/lib.rs
grep -q 'provided == scenario_pack_manifest()?' crates/cognitive-demo/src/lib.rs
# The happy-boundary scenario IS the frozen canonical demo (the parameterized builder preserves it) — pinned by
# the named test below and by the canonical-marker greps far above.
grep -q 'fn happy_boundary_scenario_equals_canonical_demo' crates/cognitive-demo/src/lib.rs
# The 12 MTRACE-0 tests exist (a gutted/deleted test drops the unit count pinned above).
grep -q 'fn scenario_pack_lists_all_scenarios' crates/cognitive-demo/src/lib.rs
grep -q 'fn each_scenario_replays' crates/cognitive-demo/src/lib.rs
grep -q 'fn each_scenario_bundle_verifies' crates/cognitive-demo/src/lib.rs
grep -q 'fn review_rejected_scenario_blocks_intent' crates/cognitive-demo/src/lib.rs
grep -q 'fn review_deferred_scenario_blocks_intent' crates/cognitive-demo/src/lib.rs
grep -q 'fn high_risk_scenario_blocks_probe' crates/cognitive-demo/src/lib.rs
grep -q 'fn no_scenario_executes' crates/cognitive-demo/src/lib.rs
grep -q 'fn no_scenario_promotes' crates/cognitive-demo/src/lib.rs
grep -q 'fn no_scenario_changes_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn tampered_scenario_bundle_is_refused' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenarios_are_distinguishable' crates/cognitive-demo/src/lib.rs
# End-to-end BINARY smoke (real files; the CLI was built in the INT-1 smoke): write the scenario pack, prove it
# is 4 scenario subdirs (4 files each) + a pack manifest, is byte-identical across runs, the scenarios are
# DISTINGUISHABLE (a requires_operator AND blocked execution status; a blocked probe), every file preserves the
# no-authority boundary, scenario-verify accepts the pristine pack, and a TAMPER of a scenario trace / a scenario
# manifest / the pack manifest, a MISSING file, and a FOREIGN scenario are ALL refused.
_mt_dir="$(mktemp -d)"
./target/debug/cognitive-demo scenarios > "$_mt_dir/list.txt" 2>/dev/null
for _s in happy-boundary review-rejected review-deferred high-risk-blocked; do
  if ! grep -qF "$_s" "$_mt_dir/list.txt"; then rm -rf "$_mt_dir"; exit 1; fi
done
./target/debug/cognitive-demo scenario-pack --out "$_mt_dir/pack" >/dev/null 2>&1
# All four scenario subdirs with all four files, plus the pack manifest.
for _s in happy-boundary review-rejected review-deferred high-risk-blocked; do
  for _f in trace.json report.txt questions.txt manifest.json; do
    if [ ! -f "$_mt_dir/pack/$_s/$_f" ]; then rm -rf "$_mt_dir"; exit 1; fi
  done
done
test -f "$_mt_dir/pack/pack-manifest.json"
# The pack manifest names every scenario and carries the six boundary lines verbatim.
for _s in happy-boundary review-rejected review-deferred high-risk-blocked; do
  if ! grep -qF "$_s" "$_mt_dir/pack/pack-manifest.json"; then rm -rf "$_mt_dir"; exit 1; fi
done
for _bl in 'Scenarios vary the path.' 'They do not vary the authority.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" "$_mt_dir/pack/pack-manifest.json"; then rm -rf "$_mt_dir"; exit 1; fi
done
# Determinism: a second pack is byte-identical, file for file.
./target/debug/cognitive-demo scenario-pack --out "$_mt_dir/pack2" >/dev/null 2>&1
if ! diff -rq "$_mt_dir/pack" "$_mt_dir/pack2" >/dev/null 2>&1; then rm -rf "$_mt_dir"; exit 1; fi
# Scenarios are DISTINGUISHABLE by status: happy-boundary is requires_operator (never executed), the review
# scenarios are blocked, and the high-risk probe is blocked.
grep -q '"execution_status": "requires_operator"' "$_mt_dir/pack/happy-boundary/trace.json"
grep -q '"execution_status": "blocked"' "$_mt_dir/pack/review-rejected/trace.json"
grep -q '"execution_status": "blocked"' "$_mt_dir/pack/review-deferred/trace.json"
grep -q '"probe_status": "blocked"' "$_mt_dir/pack/high-risk-blocked/trace.json"
# FREEZE PIN: the happy-boundary scenario IS the frozen canonical trace, so its FNV-derived hypothesis id is the
# frozen value. A drift in the happy-boundary risk/reversibility would change this id even though the path and
# statuses are unchanged — so this catches a silent canonical-trace drift that the status greps above cannot. The
# id is a STABLE FNV hash from the frozen hypothesis-layer (not a DefaultHasher value), safe to pin literally.
grep -q '"hypothesis_id": 16880898425785712701' "$_mt_dir/pack/happy-boundary/trace.json"
# NO-AUTHORITY guard over the WHOLE pack: no file shows an affirmative executed/promoted/granted/recorded status
# or a true grant, and training stays false everywhere.
if grep -rqE '"(execution_status|observation_status|promotion_status)": "(executed|promoted|granted|recorded)"' "$_mt_dir/pack"; then rm -rf "$_mt_dir"; exit 1; fi
if grep -rq '"grants_promotion": true' "$_mt_dir/pack"; then rm -rf "$_mt_dir"; exit 1; fi
if grep -rq '"training_justified": true' "$_mt_dir/pack"; then rm -rf "$_mt_dir"; exit 1; fi
# scenario-verify accepts the pristine pack...
./target/debug/cognitive-demo scenario-verify --path "$_mt_dir/pack" >/dev/null 2>&1
# ...and refuses a TAMPER of a scenario trace, a scenario manifest, the pack manifest, a MISSING file, and a
# FOREIGN scenario (each exit non-zero — no tampered scenario passes).
cp -r "$_mt_dir/pack" "$_mt_dir/tt"; sed -i 's/"review_decision": "rejected"/"review_decision": "approved"/' "$_mt_dir/tt/review-rejected/trace.json"
if ./target/debug/cognitive-demo scenario-verify --path "$_mt_dir/tt" >/dev/null 2>&1; then rm -rf "$_mt_dir"; exit 1; fi
cp -r "$_mt_dir/pack" "$_mt_dir/tm"; sed -i 's/cognitive-bundle-v0.1/forged/' "$_mt_dir/tm/happy-boundary/manifest.json"
if ./target/debug/cognitive-demo scenario-verify --path "$_mt_dir/tm" >/dev/null 2>&1; then rm -rf "$_mt_dir"; exit 1; fi
cp -r "$_mt_dir/pack" "$_mt_dir/tp"; sed -i 's/cognitive-scenario-pack-v0.1/forged/' "$_mt_dir/tp/pack-manifest.json"
if ./target/debug/cognitive-demo scenario-verify --path "$_mt_dir/tp" >/dev/null 2>&1; then rm -rf "$_mt_dir"; exit 1; fi
cp -r "$_mt_dir/pack" "$_mt_dir/tx"; rm "$_mt_dir/tx/review-deferred/questions.txt"
if ./target/debug/cognitive-demo scenario-verify --path "$_mt_dir/tx" >/dev/null 2>&1; then rm -rf "$_mt_dir"; exit 1; fi
mkdir -p "$_mt_dir/tf"; cp -r "$_mt_dir/pack/." "$_mt_dir/tf/"; for _f in trace.json report.txt questions.txt manifest.json; do echo foreign > "$_mt_dir/tf/high-risk-blocked/$_f"; done
if ./target/debug/cognitive-demo scenario-verify --path "$_mt_dir/tf" >/dev/null 2>&1; then rm -rf "$_mt_dir"; exit 1; fi
rm -rf "$_mt_dir"
# ---------------------------------------------------------------------------------------------------
# MTRACE-1 — Scenario Matrix / Boundary Coverage Report (crates/cognitive-demo, the `cognitive-demo` binary's
# `scenario-matrix` + `scenario-matrix-report` + `scenario-matrix-verify` commands). A deterministic coverage
# matrix DERIVED from the scenario set: for every scenario (path) it records the path's statuses AND proves the
# four authority boundaries (no_execution / no_evidence / no_promotion / no_training) hold, plus a coverage
# summary. The matrix is purely RE-DERIVED (it never trusts the pack files); `scenario-matrix` verifies the pack
# before emitting; verify/report re-derive and byte-compare, refusing any tampered matrix or pack. The matrix
# summarizes coverage; it does not create authority, execute, promote, or train.
# ---------------------------------------------------------------------------------------------------
# The matrix surface exists (signals): the pure builder, the re-deriving verifier, the pure renderer, the
# whole-pack verifier, and the boundary data + the mismatch error.
grep -q 'pub fn scenario_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_scenario_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn scenario_matrix_report' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_scenario_pack' crates/cognitive-demo/src/lib.rs
grep -q 'MATRIX_BOUNDARY_LINES' crates/cognitive-demo/src/lib.rs
grep -q 'MatrixMismatch' crates/cognitive-demo/src/lib.rs
# Re-derive (not trust): verify/report compare against the freshly re-derived canonical matrix; the matrix is
# Serialize-only (no Deserialize, pinned far above), so a provided matrix is never parsed into authority.
grep -q 'provided == scenario_matrix()?' crates/cognitive-demo/src/lib.rs
grep -q 'struct ScenarioMatrix' crates/cognitive-demo/src/lib.rs
# The 12 MTRACE-1 tests exist (a gutted/deleted test drops the unit count pinned above).
grep -q 'fn scenario_matrix_lists_all_scenarios' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_records_all_statuses' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_proves_no_execution_for_all' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_proves_no_evidence_for_all' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_proves_no_promotion_for_all' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_proves_training_false_for_all' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_verify_rejects_tampered_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_verify_rejects_tampered_pack' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_report_contains_boundary_summary' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_does_not_change_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_distinguishes_all_paths' crates/cognitive-demo/src/lib.rs
grep -q 'fn scenario_matrix_report_is_not_authority' crates/cognitive-demo/src/lib.rs
# End-to-end BINARY smoke (real files; the CLI was built in the INT-1 smoke): build a scenario pack, emit the
# matrix, prove it records every scenario with all statuses + the four boundary cells (all true for every
# scenario), proves the coverage (16/16, all_boundaries_hold), is byte-identical across runs, the report contains
# the boundary summary, and that a TAMPERED matrix and a TAMPERED pack are BOTH refused by verify AND report.
_m1_dir="$(mktemp -d)"
./target/debug/cognitive-demo scenario-pack --out "$_m1_dir/pack" >/dev/null 2>&1
./target/debug/cognitive-demo scenario-matrix --pack "$_m1_dir/pack" --out "$_m1_dir/matrix.json" >/dev/null 2>&1
test -f "$_m1_dir/matrix.json"
# The matrix lists every scenario and records all status fields.
for _s in happy-boundary review-rejected review-deferred high-risk-blocked; do
  if ! grep -qF "$_s" "$_m1_dir/matrix.json"; then rm -rf "$_m1_dir"; exit 1; fi
done
for _field in '"review_status"' '"probe_status"' '"intent_status"' '"observation_status"' '"promotion_status"' '"training_verdict"'; do
  if ! grep -qF "$_field" "$_m1_dir/matrix.json"; then rm -rf "$_m1_dir"; exit 1; fi
done
# EVERY scenario proves the four boundaries: the matrix never records a false boundary cell or a granted/justified.
grep -q '"cells_proven": 16' "$_m1_dir/matrix.json"
grep -q '"all_boundaries_hold": true' "$_m1_dir/matrix.json"
if grep -qE '"(no_execution|no_evidence|no_promotion|no_training)": false' "$_m1_dir/matrix.json"; then rm -rf "$_m1_dir"; exit 1; fi
if grep -q '"training_verdict": "training_justified"' "$_m1_dir/matrix.json"; then rm -rf "$_m1_dir"; exit 1; fi
# The matrix distinguishes the paths (a requires_operator AND a blocked intent; a queued AND a blocked probe).
grep -q '"intent_status": "requires_operator"' "$_m1_dir/matrix.json"
grep -q '"intent_status": "blocked"' "$_m1_dir/matrix.json"
grep -q '"probe_status": "blocked"' "$_m1_dir/matrix.json"
# NO-AUTHORITY guard over the matrix: no affirmative executed/promoted/granted/recorded status, no true grant.
if grep -qE '"(intent_status|execution_status|observation_status|promotion_status)": "(executed|promoted|granted|recorded)"' "$_m1_dir/matrix.json"; then rm -rf "$_m1_dir"; exit 1; fi
if grep -q '"grants_promotion": true' "$_m1_dir/matrix.json"; then rm -rf "$_m1_dir"; exit 1; fi
# Determinism: a second matrix is byte-identical.
./target/debug/cognitive-demo scenario-matrix --pack "$_m1_dir/pack" --out "$_m1_dir/matrix2.json" >/dev/null 2>&1
if ! cmp -s "$_m1_dir/matrix.json" "$_m1_dir/matrix2.json"; then rm -rf "$_m1_dir"; exit 1; fi
# The report renders the coverage + the five boundary lines verbatim + the explicit no-execution prose.
./target/debug/cognitive-demo scenario-matrix-report --matrix "$_m1_dir/matrix.json" --out "$_m1_dir/matrix.txt" >/dev/null 2>&1
grep -qF 'cells proven:        16/16' "$_m1_dir/matrix.txt"
grep -qF 'Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.' "$_m1_dir/matrix.txt"
for _bl in 'The matrix summarizes coverage.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_bl" "$_m1_dir/matrix.txt"; then rm -rf "$_m1_dir"; exit 1; fi
done
# scenario-matrix-verify accepts the pristine pack + matrix...
./target/debug/cognitive-demo scenario-matrix-verify --pack "$_m1_dir/pack" --matrix "$_m1_dir/matrix.json" >/dev/null 2>&1
# ...and a TAMPERED matrix is refused by BOTH verify and report (a tampered matrix is never laundered)...
sed 's/"all_boundaries_hold": true/"all_boundaries_hold": false/' "$_m1_dir/matrix.json" > "$_m1_dir/tm.json"
if ./target/debug/cognitive-demo scenario-matrix-verify --pack "$_m1_dir/pack" --matrix "$_m1_dir/tm.json" >/dev/null 2>&1; then rm -rf "$_m1_dir"; exit 1; fi
if ./target/debug/cognitive-demo scenario-matrix-report --matrix "$_m1_dir/tm.json" >/dev/null 2>&1; then rm -rf "$_m1_dir"; exit 1; fi
# ...and a TAMPERED pack is refused by BOTH scenario-matrix (won't emit) and scenario-matrix-verify.
cp -r "$_m1_dir/pack" "$_m1_dir/tp"; sed -i 's/"review_decision": "rejected"/"review_decision": "approved"/' "$_m1_dir/tp/review-rejected/trace.json"
if ./target/debug/cognitive-demo scenario-matrix --pack "$_m1_dir/tp" --out "$_m1_dir/x.json" >/dev/null 2>&1; then rm -rf "$_m1_dir"; exit 1; fi
if ./target/debug/cognitive-demo scenario-matrix-verify --pack "$_m1_dir/tp" --matrix "$_m1_dir/matrix.json" >/dev/null 2>&1; then rm -rf "$_m1_dir"; exit 1; fi
rm -rf "$_m1_dir"
# ---------------------------------------------------------------------------------------------------
# MTRACE-2 — Scenario Failure Injection / Boundary Regression Pack (crates/cognitive-demo, the
# `cognitive-demo` binary's `failure-cases` + `failure-pack` + `failure-verify` commands). A finite, enum-
# backed set of NEGATIVE scenarios: each DETERMINISTICALLY forges a forbidden authority claim onto a canonical
# artifact (a trace / scenario bundle / report / coverage matrix) and runs the EXISTING re-derive-and-byte-
# compare verifier, which REFUSES it with a typed error. Nothing forged is trusted: every artifact type is
# Serialize-only, so the forged bytes are never parsed back into authority — only COMPARED against the freshly
# re-derived canonical and rejected. The pack records, per case, that the forgery genuinely altered the
# canonical bytes AND the exact typed rejection reason. Failure cases attack the boundary; they do not weaken it.
# ---------------------------------------------------------------------------------------------------
# The failure surface exists (signals): the pure pack builder, the re-deriving verifier, the pack files, the
# listing, the boundary data, and the closed failure-case enum.
grep -q 'pub fn failure_pack' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_failure_pack' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn failure_pack_files' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn list_failure_cases' crates/cognitive-demo/src/lib.rs
grep -q 'FAILURE_BOUNDARY_LINES' crates/cognitive-demo/src/lib.rs
grep -q 'enum FailureCase' crates/cognitive-demo/src/lib.rs
# Re-derive (not trust): verify_failure_pack byte-compares against the re-derived canonical pack; the pack is
# Serialize-only (no Deserialize, pinned far above), so a provided pack is never parsed into authority.
grep -q 'compare_bundle(&failure_pack_files()?' crates/cognitive-demo/src/lib.rs
grep -q 'struct FailurePack' crates/cognitive-demo/src/lib.rs
# ANTI-VACUITY: each forgery's verdict is the REAL result of an EXISTING re-derive verifier (observed, never a
# hardcoded bool) — so a forgery's "rejected" can only be true because a genuine verifier refused the forged
# artifact. All four surfaces are exercised (trace / scenario-bundle / bundle / matrix).
grep -q 'verdict: verify_trace_json' crates/cognitive-demo/src/lib.rs
grep -q 'verdict: verify_scenario_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'verdict: verify_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'verdict: verify_scenario_matrix' crates/cognitive-demo/src/lib.rs
# ANTI-VACUITY (forgery is FORBIDDEN authority, not a benign change): each forgery is checked to inject its
# specific affirmative-authority token, so a forgery that merely perturbed a byte (also rejected) cannot
# masquerade as a forbidden-authority forgery.
grep -q 'fn forbidden_token' crates/cognitive-demo/src/lib.rs
grep -q 'injects_forbidden: forged.contains(token)' crates/cognitive-demo/src/lib.rs
# The 12 MTRACE-2 tests exist (a gutted/deleted test drops the unit count pinned above).
grep -q 'fn failure_pack_lists_all_cases' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_execution_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_evidence_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_promotion_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_training_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_review_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_report_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn forged_matrix_is_rejected' crates/cognitive-demo/src/lib.rs
grep -q 'fn failure_report_contains_rejection_reasons' crates/cognitive-demo/src/lib.rs
grep -q 'fn failure_pack_does_not_change_training_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn failure_pack_forgeries_actually_mutate_canonical' crates/cognitive-demo/src/lib.rs
grep -q 'fn failure_pack_verify_rejects_tampered_pack' crates/cognitive-demo/src/lib.rs
# End-to-end BINARY smoke (real files; the CLI was built in the INT-1 smoke): emit the failure pack and prove
# every negative scenario is recorded with its forgery APPLIED and REJECTED, no forgery slipped through, no
# affirmative authority leaked into the pack files, the report records the exact typed rejection reasons + the
# boundary verbatim, the pack is deterministic, a doctored/missing pack is refused, and the frozen canonical
# trace is unperturbed.
_m2_dir="$(mktemp -d)"
./target/debug/cognitive-demo failure-pack --out "$_m2_dir/fp" >/dev/null 2>&1
test -f "$_m2_dir/fp/failure-pack.json"
test -f "$_m2_dir/fp/failure-report.txt"
# Every negative scenario is listed and recorded.
for _c in forged-execution forged-evidence forged-promotion forged-training forged-review forged-report forged-matrix; do
  if ! grep -qF "$_c" "$_m2_dir/fp/failure-pack.json"; then rm -rf "$_m2_dir"; exit 1; fi
done
# Every forgery genuinely applied, injected its forbidden token, AND was rejected (exactly 7 each); summary agrees.
test "$(grep -c '"forgery_applied": true' "$_m2_dir/fp/failure-pack.json")" -eq 7
test "$(grep -c '"injects_forbidden": true' "$_m2_dir/fp/failure-pack.json")" -eq 7
test "$(grep -c '"rejected": true' "$_m2_dir/fp/failure-pack.json")" -eq 7
grep -q '"all_forged": true' "$_m2_dir/fp/failure-pack.json"
grep -q '"all_inject_forbidden": true' "$_m2_dir/fp/failure-pack.json"
grep -q '"all_rejected": true' "$_m2_dir/fp/failure-pack.json"
# NO forgery slipped through or was benign (not a single applied:false, injects_forbidden:false, or rejected:false).
if grep -qE '"(forgery_applied|injects_forbidden|rejected)": false' "$_m2_dir/fp/failure-pack.json"; then rm -rf "$_m2_dir"; exit 1; fi
# NO-AUTHORITY guard: no affirmative authority JSON leaked into EITHER pack file — the forged bytes are never
# persisted as trusted state; only the (prose) rejection record is.
if grep -qE '"(execution_status|observation_status|promotion_status|intent_status)": "(executed|promoted|granted|recorded)"' "$_m2_dir/fp/"*; then rm -rf "$_m2_dir"; exit 1; fi
if grep -qE '"(grants_promotion|training_justified)": true' "$_m2_dir/fp/"*; then rm -rf "$_m2_dir"; exit 1; fi
if grep -qE '"no_(execution|evidence|promotion|training)": false' "$_m2_dir/fp/"*; then rm -rf "$_m2_dir"; exit 1; fi
# The report records each case as REJECTED (never ACCEPTED) with the EXACT typed-error reason — the rejections
# are structural re-derive byte-compare refusals, not a prose grep.
grep -qF 'verdict:          REJECTED' "$_m2_dir/fp/failure-report.txt"
if grep -qF 'verdict:          ACCEPTED' "$_m2_dir/fp/failure-report.txt"; then rm -rf "$_m2_dir"; exit 1; fi
grep -qF 'the provided trace is not the canonical trace (tampered, stale, or foreign)' "$_m2_dir/fp/failure-report.txt"
grep -qF "bundle file 'report.txt' is not the canonical file (tampered, stale, or foreign)" "$_m2_dir/fp/failure-report.txt"
grep -qF 'the provided matrix is not the canonical matrix (tampered, stale, or foreign)' "$_m2_dir/fp/failure-report.txt"
# The seven-line boundary appears verbatim.
for _bl in 'Failure cases attack the boundary.' 'They do not weaken it.' 'Forged authority is rejected.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" "$_m2_dir/fp/failure-report.txt"; then rm -rf "$_m2_dir"; exit 1; fi
done
# Determinism: a second pack is byte-identical.
./target/debug/cognitive-demo failure-pack --out "$_m2_dir/fp2" >/dev/null 2>&1
if ! cmp -s "$_m2_dir/fp/failure-pack.json" "$_m2_dir/fp2/failure-pack.json"; then rm -rf "$_m2_dir"; exit 1; fi
if ! cmp -s "$_m2_dir/fp/failure-report.txt" "$_m2_dir/fp2/failure-report.txt"; then rm -rf "$_m2_dir"; exit 1; fi
# failure-verify accepts the pristine pack...
./target/debug/cognitive-demo failure-verify --path "$_m2_dir/fp" >/dev/null 2>&1
# ...a doctored pack that claims a forgery PASSED (rejected:true -> false) is REFUSED (re-derive-never-trust)...
mkdir -p "$_m2_dir/td"
sed 's/"rejected": true/"rejected": false/' "$_m2_dir/fp/failure-pack.json" > "$_m2_dir/td/failure-pack.json"
cp "$_m2_dir/fp/failure-report.txt" "$_m2_dir/td/failure-report.txt"
if ./target/debug/cognitive-demo failure-verify --path "$_m2_dir/td" >/dev/null 2>&1; then rm -rf "$_m2_dir"; exit 1; fi
# ...and a missing file is refused.
mkdir -p "$_m2_dir/tm"; cp "$_m2_dir/fp/failure-pack.json" "$_m2_dir/tm/failure-pack.json"
if ./target/debug/cognitive-demo failure-verify --path "$_m2_dir/tm" >/dev/null 2>&1; then rm -rf "$_m2_dir"; exit 1; fi
# The FROZEN canonical is unperturbed: the happy-boundary scenario trace still equals demo() byte-for-byte.
./target/debug/cognitive-demo scenario-pack --out "$_m2_dir/pack" >/dev/null 2>&1
./target/debug/cognitive-demo trace --out "$_m2_dir/demo.json" >/dev/null 2>&1
if ! cmp -s "$_m2_dir/pack/happy-boundary/trace.json" "$_m2_dir/demo.json"; then rm -rf "$_m2_dir"; exit 1; fi
rm -rf "$_m2_dir"
# ---------------------------------------------------------------------------------------------------
# MTRACE-3 — multi-trace validation milestone freeze. The MTRACE-0 -> MTRACE-2 multi-trace arc (the cognitive-demo
# crate's scenario pack, boundary-coverage matrix, and failure-injection pack, all over the frozen
# integration-demo-v0.1 canonical trace) is frozen as multi-trace-validation-v0.1. The milestone record
# (MULTI_TRACE_VALIDATION_MILESTONE.md) pins the commit lineage, the frozen base, the scenario-variation /
# coverage / failure-injection boundary, the P12 training verdict, and the honest residuals, and is locked here so
# the freeze cannot silently drift. The pinned commit hashes are auditable against `git log`; this lock stays
# git-free and does NOT require the tag to exist (the tag is created only after a clean tree + green gate).
# Documentation freeze only — no code crate changes, no model, no training; the milestone records
# training_not_justified. Doctrine: Scenarios vary the path. They do not vary the authority. The matrix summarizes
# coverage. Failure cases attack the boundary. Forged authority is rejected. Nothing executes. Nothing becomes
# evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
test -f MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'FROZEN' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'multi-trace-validation-v0.1' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'MTRACE-0' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'MTRACE-2' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'training_not_justified' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'training_justified=false' MULTI_TRACE_VALIDATION_MILESTONE.md
# Full MTRACE-0..MTRACE-2 commit lineage is pinned (cross-checkable against git log).
grep -q 'aee733f' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q '91189f2' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'be6909f' MULTI_TRACE_VALIDATION_MILESTONE.md
# The frozen integration base is referenced as a frozen dep (tag + commit), as are the two deeper frozen tracks.
grep -q 'integration-demo-v0.1' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q '95b586d' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'reading-track-v0.1' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'hypothesis-track-v0.1' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'f6fa55a' MULTI_TRACE_VALIDATION_MILESTONE.md
grep -q 'bb20acf' MULTI_TRACE_VALIDATION_MILESTONE.md
# The nine-line scenario / matrix / failure boundary is recorded verbatim (all nine lines).
for _bl in 'Scenarios vary the path.' 'They do not vary the authority.' 'The matrix summarizes coverage.' 'Failure cases attack the boundary.' 'Forged authority is rejected.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" MULTI_TRACE_VALIDATION_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' MULTI_TRACE_VALIDATION_MILESTONE.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# OPS-0 — operator manual / prototype capability guide. OPERATOR_MANUAL.md is the plain operator guide to the
# frozen prototype: what it is/is not, the five frozen milestone tags + recovery/verify commands, the exact
# cognitive-demo commands to reproduce every demo (trace/report/replay/questions/bundle/scenario/matrix/
# failure-pack, with the real flags + the eight audit-question slugs), the authority boundaries that stay
# closed, and the P12 training verdict. A comprehension/reproducibility sprint — no code crate change, no new
# behavior, no model, no training; the manual records training_not_justified. This lock pins the manual's
# existence, the five frozen tag names it must list, the documented command surface (every subcommand by name
# + the recovery and verify commands + a real question slug), the training verdict, and the six boundary lines
# verbatim, and guards against any manual that falsely claims training has opened. Doctrine: The manual explains
# the prototype. It does not expand the prototype. It does not create authority. It does not execute. It does
# not promote. It does not train.
# ---------------------------------------------------------------------------------------------------
test -f OPERATOR_MANUAL.md
# The manual lists all five frozen milestone tags (the recovery markers).
grep -q 'cognitive-os-governance-v0.1' OPERATOR_MANUAL.md
grep -q 'reading-track-v0.1' OPERATOR_MANUAL.md
grep -q 'hypothesis-track-v0.1' OPERATOR_MANUAL.md
grep -q 'integration-demo-v0.1' OPERATOR_MANUAL.md
grep -q 'multi-trace-validation-v0.1' OPERATOR_MANUAL.md
# The manual documents the real recovery + verify commands.
grep -qF 'git checkout' OPERATOR_MANUAL.md
grep -qF './scripts/release_check.sh' OPERATOR_MANUAL.md
# The manual documents the real command surface, by exact invocation (not vacuous prose) — every subcommand.
for _cmd in 'trace --out' 'report --trace' 'replay --trace' 'ask --trace' 'questions' 'bundle --out' 'bundle-verify --path' 'scenarios' 'scenario-pack --out' 'scenario-verify --path' 'scenario-matrix --pack' 'scenario-matrix-report --matrix' 'scenario-matrix-verify --pack' 'failure-cases' 'failure-pack --out' 'failure-verify --path'; do
  if ! grep -qF "$_cmd" OPERATOR_MANUAL.md; then exit 1; fi
done
# At least one real, enumerated audit-question slug is documented (the interrogation surface is real).
grep -qF 'was-anything-executed' OPERATOR_MANUAL.md
# The training verdict is stated and P13-P15 are recorded closed.
grep -q 'training_not_justified' OPERATOR_MANUAL.md
grep -q 'training_justified=false' OPERATOR_MANUAL.md
grep -qF 'P13' OPERATOR_MANUAL.md
# The six-line manual boundary is recorded verbatim (all six lines).
for _bl in 'The manual explains the prototype.' 'It does not expand the prototype.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_bl" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' OPERATOR_MANUAL.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# OPS-1 — operator smoke script / manual drift guard. scripts/operator_smoke.sh runs the WHOLE documented
# operator path end-to-end against the built cognitive-demo binary (trace --out, report, replay, questions,
# ask, bundle+bundle-verify, scenario-pack+scenario-verify, scenario-matrix+report+verify, failure-pack+
# failure-verify) in a throwaway temp dir, and fails closed if any documented command, boundary line, or
# verify step has drifted from OPERATOR_MANUAL.md. It re-derives every generated artifact through the
# binary's OWN verify subcommands (never trusts the bytes), proves tamper is still refused (re-derive is
# load-bearing), uses --out (never a shell redirect), and leaves no repo debris. This lock RUNS the smoke
# (a failed operator path aborts the gate) and pins the script's load-bearing properties by source
# inspection so it cannot silently degrade. No code crate change, no new authority, no execution, no
# training. Doctrine: The smoke test verifies the operator path. It does not create authority. It does not
# execute. It does not promote. It does not train.
# ---------------------------------------------------------------------------------------------------
test -f scripts/operator_smoke.sh
test -x scripts/operator_smoke.sh
# RUN the operator smoke: it exercises the documented path end-to-end and fails closed on any drift.
# Capture combined output (so release_check stays byte-silent on the GREEN path) and REQUIRE the
# completion sentinel: the OK line is printed only if the whole script ran, so a short-circuited /
# early-`exit 0` smoke that runs no commands is caught here even though it exits 0. On ANY failure (the
# smoke exits non-zero OR the sentinel is missing) the captured output — including the smoke's drift
# reason — is surfaced to stderr before aborting, so the operator sees WHY without re-running. The gate
# is already failing at that point, so this never breaks green-silence.
if _ops1_smoke_out="$(./scripts/operator_smoke.sh 2>&1)"; then
  case "$_ops1_smoke_out" in
    *'operator-smoke: OK — the documented operator path runs and the manual matches the binary'*) : ;;
    *) printf '%s\n' "$_ops1_smoke_out" >&2; exit 1 ;;
  esac
else
  printf '%s\n' "$_ops1_smoke_out" >&2; exit 1
fi
# Source pins (sabotage-detectable — the smoke cannot silently weaken):
# Fail-closed shell, and the canonical trace is written with --out, NEVER a shell redirect (`trace > FILE`
# appends a newline and is correctly refused by re-derive; `--out` writes exact replayable bytes).
grep -q 'set -eu' scripts/operator_smoke.sh
grep -qF 'trace --out' scripts/operator_smoke.sh
test "$(grep -cE 'trace[[:space:]]*>' scripts/operator_smoke.sh)" -eq 0
# Temp-dir only with cleanup (no repo debris).
grep -qF 'mktemp -d' scripts/operator_smoke.sh
grep -q "trap 'rm -rf" scripts/operator_smoke.sh
# The smoke runs every documented operator command (it cannot silently drop one).
for _cmd in 'trace --out' 'report --trace' 'replay --trace' 'questions' 'ask --trace' 'bundle --out' 'bundle-verify --path' 'scenario-pack --out' 'scenario-verify --path' 'scenario-matrix --pack' 'scenario-matrix-report --matrix' 'scenario-matrix-verify --pack' 'failure-pack --out' 'failure-verify --path'; do
  if ! grep -qF "$_cmd" scripts/operator_smoke.sh; then exit 1; fi
done
# It re-derives generated artifacts through the binary's OWN verify subcommands (never trusts the bytes),
# and proves tamper is refused (so the re-derive is load-bearing, not vacuous).
grep -qF 'replay --trace' scripts/operator_smoke.sh
grep -qF 'accepted a tampered trace' scripts/operator_smoke.sh
grep -qF 'accepted a tampered bundle' scripts/operator_smoke.sh
# Boundary drift guard: the smoke embeds the BINARY's report boundary and the MANUAL's boundary to compare.
grep -qF 'binary report boundary line drifted' scripts/operator_smoke.sh
grep -qF 'manual boundary line drifted' scripts/operator_smoke.sh
grep -qF 'The manual explains the prototype.' scripts/operator_smoke.sh
grep -qF 'Nothing trains.' scripts/operator_smoke.sh
# The OPS-1 five-line boundary is recorded verbatim in the smoke (all five lines).
for _bl in 'The smoke test verifies the operator path.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_bl" scripts/operator_smoke.sh; then exit 1; fi
done
# The smoke makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' scripts/operator_smoke.sh; then exit 1; fi
# The manual records the operator smoke self-check (a short reference, so the smoke is discoverable).
grep -qF 'scripts/operator_smoke.sh' OPERATOR_MANUAL.md
# ---------------------------------------------------------------------------------------------------
# DOCFLOW-1 — document flow operator guard. The operator manual (OPERATOR_MANUAL.md §11) documents the four
# DOCFLOW-0 commands (doc-trace / doc-report / doc-bundle / doc-bundle-verify) and states the document is READ
# but NOT trusted; the operator smoke (scripts/operator_smoke.sh §10) runs the whole doc flow end-to-end
# against a LOCAL sample document, proving the trace starts from the document's OWN verified read and that a
# tampered document / trace / report / manifest is refused. This is a documentation + drift-guard sprint — no
# code crate change, no new behavior. The smoke is already RUN by the OPS-1 lock above (a doc-flow drift makes
# it fail closed and aborts the gate); the pins below stop the doc-flow coverage from being silently dropped
# from the smoke or the manual. Doctrine: The document operator path explains and verifies local-document
# tracing. It does not trust local input. It does not create authority. It does not execute. It does not
# promote. It does not train.
# ---------------------------------------------------------------------------------------------------
# The manual documents the four operator-document commands (manual surface == binary surface).
for _dc in 'doc-trace --input' 'doc-report --input' 'doc-bundle --input' 'doc-bundle-verify --input'; do
  if ! grep -qF "$_dc" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual states local input is read but NOT trusted, and records the DOCFLOW-1 six-line boundary verbatim.
grep -qF 'read but not trusted' OPERATOR_MANUAL.md
for _dbl in 'The document operator path explains and verifies local-document tracing.' 'It does not trust local input.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_dbl" OPERATOR_MANUAL.md; then exit 1; fi
done
# The smoke creates the doc under target/ (relative local path) and removes both temp dirs on exit (no debris).
grep -qF 'target/.docflow_smoke' scripts/operator_smoke.sh
grep -qF 'rm -rf "$work" "$docwork"' scripts/operator_smoke.sh
# Its §10 doc-flow run additionally asserts NO affirmative-authority status leaked into the doc trace. This
# fail string is UNIQUE to the §10 block, so removing the doc-flow run (not just the target/ setup line above)
# is caught here too — the doc-flow coverage cannot be silently dropped from any single reference point.
grep -qF 'doc trace claims an executed/recorded/promoted/granted status' scripts/operator_smoke.sh
# The smoke exercises all four doc commands with --input, and doc-trace writes with --out (never a redirect).
for _dc in 'doc-trace --input' 'doc-report --input' 'doc-bundle --input' 'doc-bundle-verify --input'; do
  if ! grep -qF "$_dc" scripts/operator_smoke.sh; then exit 1; fi
done
grep -qF 'doc-trace --input "$docrel/doc.txt" --out' scripts/operator_smoke.sh
# The smoke proves the trace read the OPERATOR's own text (its first span), not the canonical corpus.
grep -qF '"reading_answer": "The east bridge reopened today."' scripts/operator_smoke.sh
# The smoke proves re-derive is load-bearing over operator input: a tampered document, each tampered bundle
# file (trace / report / questions / manifest), and a tampered standalone trace are all refused.
grep -qF 'accepted a tampered document' scripts/operator_smoke.sh
grep -qF 'for _bf in trace.json report.txt questions.txt manifest.json' scripts/operator_smoke.sh
grep -qF 'accepted a tampered trace' scripts/operator_smoke.sh
# The smoke records the DOCFLOW-1 six-line boundary verbatim (the OPS-1 lock above already pins the smoke
# makes no false training claim).
for _dbl in 'The document operator path explains and verifies local-document tracing.' 'It does not trust local input.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_dbl" scripts/operator_smoke.sh; then exit 1; fi
done
# ---------------------------------------------------------------------------------------------------
# CORPUS-1 — corpus flow operator guard. The operator manual (OPERATOR_MANUAL.md §12) documents the four
# CORPUS-0 commands (corpus-trace / corpus-report / corpus-bundle / corpus-bundle-verify), states the corpus is
# READ but NOT trusted, that source selection is verified and replayable, and that the WHOLE corpus is
# hash-bound; the operator smoke (scripts/operator_smoke.sh §11) runs the whole corpus flow end-to-end against a
# LOCAL directory of .txt documents, proving the directory filter matches CORPUS-0 (hidden / non-.txt /
# symlink-escape refused), the trace starts from the corpus's OWN verified first span, and that mutating the
# grounding document OR a non-grounding SIDE document — and tampering the source / trace / report / questions /
# manifest — is refused. A documentation + drift-guard sprint — no code crate change, no new behavior (the unit
# count pinned at 112 above is unchanged). The smoke is already RUN by the OPS-1 lock above (a corpus-flow drift
# makes it fail closed and aborts the gate); the pins below stop the corpus coverage from being silently dropped
# from the smoke or the manual. Doctrine: The corpus operator path reads local documents. It does not trust local
# documents. Source selection is verified and replayable. The whole corpus is hash-bound. Verification comes
# before tracing. Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# The manual documents the four corpus commands (manual surface == binary surface).
for _cc in 'corpus-trace --input-dir' 'corpus-report --input-dir' 'corpus-bundle --input-dir' 'corpus-bundle-verify --input-dir'; do
  if ! grep -qF "$_cc" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual states the corpus is read-but-not-trusted, hash-bound as a whole, and source selection is
# verified and replayable; and records the CORPUS-1 nine-line boundary verbatim.
grep -qF 'read but not trusted' OPERATOR_MANUAL.md
grep -qF 'hash-bound as a whole' OPERATOR_MANUAL.md
grep -qF 'Source selection is verified and replayable.' OPERATOR_MANUAL.md
for _cbl in 'The corpus operator path reads local documents.' 'It does not trust local documents.' 'Source selection is verified and replayable.' 'The whole corpus is hash-bound.' 'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_cbl" OPERATOR_MANUAL.md; then exit 1; fi
done
# The smoke creates the corpus under target/ (relative local path) and removes all temp dirs on exit (no debris).
grep -qF 'target/.corpus_smoke' scripts/operator_smoke.sh
grep -qF 'rm -rf "$work" "$docwork" "$corpuswork"' scripts/operator_smoke.sh
# Its §11 corpus run additionally asserts NO affirmative-authority status leaked into the corpus trace. This
# fail string is UNIQUE to the §11 block, so removing the corpus-flow run (not just the target/ setup line
# above) is caught here too — the corpus coverage cannot be silently dropped from any single reference point.
grep -qF 'corpus trace claims an executed/recorded/promoted/granted status' scripts/operator_smoke.sh
# The smoke exercises all four corpus commands with --input-dir, and corpus-trace writes with --out (never a redirect).
for _cc in 'corpus-trace --input-dir' 'corpus-report --input-dir' 'corpus-bundle --input-dir' 'corpus-bundle-verify --input-dir'; do
  if ! grep -qF "$_cc" scripts/operator_smoke.sh; then exit 1; fi
done
grep -qF 'corpus-trace --input-dir "$corpusrel/corpus" --out' scripts/operator_smoke.sh
# The smoke proves the trace read the corpus's OWN first span, and proves the directory filter (exactly two
# admitted documents — hidden / non-.txt / symlink excluded, matching CORPUS-0).
grep -qF '"reading_answer": "The east bridge reopened today."' scripts/operator_smoke.sh
grep -qF 'corpus documents:   2' scripts/operator_smoke.sh
# The smoke proves re-derive is load-bearing over the WHOLE corpus: mutating the GROUNDING document AND a
# non-grounding SIDE document are BOTH refused (the corpus-specific binding a single-document guard cannot show),
# and tampered bundle files (incl. corpus-source.json) and a tampered standalone trace are refused.
grep -qF 'accepted a mutated grounding document' scripts/operator_smoke.sh
grep -qF 'accepted a mutated non-grounding side document' scripts/operator_smoke.sh
grep -qF 'for _cf in corpus-source.json trace.json report.txt questions.txt manifest.json' scripts/operator_smoke.sh
grep -qF 'corpus-report accepted a tampered trace' scripts/operator_smoke.sh
# The smoke records the CORPUS-1 nine-line boundary verbatim (the OPS-1 lock above already pins the smoke
# makes no false training claim).
for _cbl in 'The corpus operator path reads local documents.' 'It does not trust local documents.' 'Source selection is verified and replayable.' 'The whole corpus is hash-bound.' 'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_cbl" scripts/operator_smoke.sh; then exit 1; fi
done
# ---------------------------------------------------------------------------------------------------
# NOVELTY-1 — novelty flow operator guard. The operator manual (OPERATOR_MANUAL.md §13) documents the three
# NOVELTY-0 commands (novelty-packet / novelty-report / novelty-replay), states that novelty packets PROPOSE but
# do NOT prove, that the operator frame is recorded but never grounded as fact, that preserved facts come only
# from verified corpus spans, that probe requests do not execute, and that a packet can never become evidence /
# promotion / training; the operator smoke (scripts/operator_smoke.sh §12) runs the whole novelty flow end-to-end
# against a LOCAL corpus + frame — corpus-trace FIRST (a packet is only produced on top of a VERIFIED trace),
# then novelty-packet / novelty-report / novelty-replay — proving the packet's authority is hypothesis_only with
# every probe request non-executing, the only grounded content is the VERIFIED corpus span, and every refusal:
# an empty frame, an UNSUPPORTED preserved fact (the frame's own claim swapped in), a tampered packet, and a
# receipt-hash-stripped corpus trace are each refused. A documentation + drift-guard sprint — no code crate
# change, no new behavior (the unit count pinned above is unchanged). The smoke is already RUN by the OPS-1 lock
# above (a novelty-flow drift makes it fail closed and aborts the gate); the pins below stop the novelty coverage
# from being silently dropped from the smoke or the manual. Doctrine: The novelty operator path proposes. It does
# not prove. It cites verified receipts. The operator frame is not a preserved fact. Probe requests do not
# execute. Nothing becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# The manual documents the three novelty commands (manual surface == binary surface).
for _nc in 'novelty-packet --input-dir' 'novelty-report --input-dir' 'novelty-replay --input-dir'; do
  if ! grep -qF "$_nc" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual states the novelty doctrine verbatim: propose-not-prove, frame-not-grounded-as-fact, preserved
# facts come only from verified spans, and never evidence / promotion / training.
for _ns in 'propose but do not prove' 'never grounded as fact' 'come only from verified corpus spans' 'can never become evidence, a promotion, or training'; do
  if ! grep -qF "$_ns" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual records the NOVELTY-1 eight-line novelty-operator-path boundary verbatim.
for _nbl in 'The novelty operator path proposes.' 'It does not prove.' 'It cites verified receipts.' 'The operator frame is not a preserved fact.' 'Probe requests do not execute.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_nbl" OPERATOR_MANUAL.md; then exit 1; fi
done
# The smoke creates the novelty corpus + frame under target/ (relative local paths) and removes the temp dir on
# exit (the OPS-1 lock above already pins the four-dir trap line, which contains "$noveltywork").
grep -qF 'target/.novelty_smoke' scripts/operator_smoke.sh
grep -qF 'rm -rf "$work" "$docwork" "$corpuswork" "$noveltywork"' scripts/operator_smoke.sh
# The smoke runs corpus-trace FIRST (the packet is only produced on top of a verified trace) with --out (never a
# redirect), then exercises all three novelty commands with --input-dir.
grep -qF 'corpus-trace --input-dir "$noveltyrel/corpus" --out' scripts/operator_smoke.sh
for _nc in 'novelty-packet --input-dir' 'novelty-report --input-dir' 'novelty-replay --input-dir'; do
  if ! grep -qF "$_nc" scripts/operator_smoke.sh; then exit 1; fi
done
# The smoke proves the packet is hypothesis_only, preserves the VERIFIED corpus span (not the frame's claim),
# and that no affirmative-authority status leaked. This fail string is UNIQUE to the §12 novelty block, so
# removing the novelty run (not just the target/ setup line above) is caught here too.
grep -qF 'novelty-packet did not record hypothesis_only authority' scripts/operator_smoke.sh
grep -qF 'novelty packet did not preserve the verified corpus span' scripts/operator_smoke.sh
grep -qF 'novelty packet claims an executed/recorded/promoted/granted status' scripts/operator_smoke.sh
# The smoke proves re-derive is load-bearing over the novelty packet: an empty frame, an UNSUPPORTED preserved
# fact, a tampered packet, and a receipt-hash-stripped corpus trace are EACH refused end-to-end.
grep -qF 'novelty-packet accepted an empty frame' scripts/operator_smoke.sh
grep -qF 'novelty-report accepted an unsupported preserved fact' scripts/operator_smoke.sh
grep -qF 'novelty-replay accepted an unsupported preserved fact' scripts/operator_smoke.sh
grep -qF 'novelty-report accepted a tampered packet' scripts/operator_smoke.sh
grep -qF 'novelty-replay accepted a tampered packet' scripts/operator_smoke.sh
grep -qF 'novelty-packet accepted a receipt-hash-stripped corpus trace' scripts/operator_smoke.sh
# The smoke records the NOVELTY-1 eight-line boundary verbatim (the OPS-1 lock above already pins the smoke
# makes no false training claim).
for _nbl in 'The novelty operator path proposes.' 'It does not prove.' 'It cites verified receipts.' 'The operator frame is not a preserved fact.' 'Probe requests do not execute.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_nbl" scripts/operator_smoke.sh; then exit 1; fi
done
# ---------------------------------------------------------------------------------------------------
# DREAM-EXPORT-1 — dream export operator guard. The operator manual (OPERATOR_MANUAL.md §14) documents the three
# DREAM-EXPORT-0 commands (dream-export / dream-export-report / dream-export-replay), states that the bridge
# PRESERVES dream provenance and creates NO new authority type, that exported material stays hypothesis_only, that
# the dream engine's private dream_only/DreamOnly authority never crosses, that probe requests do not execute, and
# that a dream-exported hypothesis can never become evidence / promotion / training; the operator smoke
# (scripts/operator_smoke.sh §13) runs the whole dream export flow end-to-end against a LOCAL corpus + frame —
# dream-export FIRST (which re-derives/GENERATES the terminal dream packet and bridges it through the EXISTING
# hypothesis gate; dream-engine is a quarantined library with no standalone packet emitter), then
# dream-export-report / dream-export-replay — proving the export carries the EXISTING hypothesis_only authority,
# records dream_origin (auditable), emits NO dream_only/DreamOnly token, and that every tamper is refused: a
# foreign/tampered --dream-packet, a tampered DreamExportReceipt, and a receipt forged to dream_origin=false are
# each refused. A documentation + drift-guard sprint — no code crate change, no new behavior (the unit count
# pinned above is unchanged). The smoke is already RUN by the OPS-1 lock above (a dream-export drift makes it fail
# closed and aborts the gate); the pins below stop the dream export coverage from being silently dropped from the
# smoke or the manual. Doctrine: The dream export operator path preserves provenance. It does not create a new
# authority. Exported dream material remains HypothesisOnly. Dream origin remains auditable. DreamOnly remains
# private to dream-engine. Probe requests do not execute. Nothing becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# The manual documents the three dream-export commands (manual surface == binary surface).
for _dxc in 'dream-export --input-dir' 'dream-export-report --input-dir' 'dream-export-replay --input-dir'; do
  if ! grep -qF "$_dxc" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual states the dream-export doctrine verbatim: preserves provenance, NO new authority type, the private
# dream_only authority never crosses, and never evidence / promotion / training.
for _dxd in 'preserves dream provenance' 'without creating a new authority type' 'never crosses' 'can never become evidence, a promotion, or training'; do
  if ! grep -qF "$_dxd" OPERATOR_MANUAL.md; then exit 1; fi
done
# The manual records the DREAM-EXPORT-1 nine-line dream-export-operator-path boundary verbatim.
for _dxb in 'The dream export operator path preserves provenance.' 'It does not create a new authority.' 'Exported dream material remains HypothesisOnly.' 'Dream origin remains auditable.' 'DreamOnly remains private to dream-engine.' 'Probe requests do not execute.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_dxb" OPERATOR_MANUAL.md; then exit 1; fi
done
# The smoke creates the dream corpus + frame under target/ (relative local paths) and removes the temp dir on exit
# (the OPS-1 lock above already pins the trap line, whose pinned prefix this extends with "$dreamwork").
grep -qF 'target/.dream_smoke' scripts/operator_smoke.sh
grep -qF '"$dreamwork"' scripts/operator_smoke.sh
# The smoke runs dream-export FIRST with --out (never a redirect) — dream packet GENERATION happens here, inside
# dream-export — then exercises all three dream-export commands with --input-dir.
grep -qF 'dream-export --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --out' scripts/operator_smoke.sh
for _dxc in 'dream-export --input-dir' 'dream-export-report --input-dir' 'dream-export-replay --input-dir'; do
  if ! grep -qF "$_dxc" scripts/operator_smoke.sh; then exit 1; fi
done
# The smoke proves the export carries the EXISTING hypothesis_only authority, records dream_origin, and that the
# private dream_only/DreamOnly authority does NOT cross into the emitted export. These fail strings are UNIQUE to
# the §13 dream block, so removing the dream-export run (not just the target/ setup line) is caught here too.
grep -qF 'dream-export did not record hypothesis_only authority_after_export' scripts/operator_smoke.sh
grep -qF 'dream-export did not record dream_origin true' scripts/operator_smoke.sh
grep -qF 'dream-export leaked a dream_only authority' scripts/operator_smoke.sh
grep -qF 'dream-export leaked a DreamOnly authority' scripts/operator_smoke.sh
# The smoke proves re-derive is load-bearing over the dream export: a foreign/tampered --dream-packet, a tampered
# DreamExportReceipt, and a receipt forged to dream_origin=false are EACH refused end-to-end.
grep -qF 'dream-export accepted a foreign/tampered dream packet' scripts/operator_smoke.sh
grep -qF 'dream-export-replay accepted a tampered receipt' scripts/operator_smoke.sh
grep -qF 'dream-export-report accepted dream_origin=false' scripts/operator_smoke.sh
grep -qF 'dream-export-replay accepted dream_origin=false' scripts/operator_smoke.sh
# The smoke records the DREAM-EXPORT-1 nine-line boundary verbatim (the OPS-1 lock above already pins the smoke
# makes no false training claim).
for _dxbl in 'The dream export operator path preserves provenance.' 'It does not create a new authority.' 'Exported dream material remains HypothesisOnly.' 'Dream origin remains auditable.' 'DreamOnly remains private to dream-engine.' 'Probe requests do not execute.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_dxbl" scripts/operator_smoke.sh; then exit 1; fi
done
# ---------------------------------------------------------------------------------------------------
# OPS-2 — operator release snapshot / local archive manifest. OPERATOR_RELEASE_SNAPSHOT.md is a docs-only
# local snapshot of the prototype state after OPS-1: the current HEAD commit (c33dea7), every frozen tag +
# its commit, the recovery commands, the release_check + operator_smoke verification commands, what the
# prototype can and cannot do, and the P12 training verdict. A snapshot/reproducibility sprint — no code
# crate change, no new behavior, no model, no training, and NO remote release (nothing pushed/published/
# uploaded). This lock pins the snapshot's existence, the HEAD commit it records, the five frozen tag names
# + their commits, the recovery + verify commands, the training verdict, P13-P15 closed, and the six
# boundary lines verbatim, and guards against any snapshot that falsely claims training has opened.
# Doctrine: The snapshot records the prototype state. It does not release remotely. It does not create
# authority. It does not execute. It does not promote. It does not train.
# ---------------------------------------------------------------------------------------------------
test -f OPERATOR_RELEASE_SNAPSHOT.md
# The snapshot records the post-OPS-1 HEAD commit (the state it snapshots).
grep -qF 'c33dea7' OPERATOR_RELEASE_SNAPSHOT.md
# The snapshot lists all five frozen milestone tags AND their commits (the recovery markers).
grep -qF 'cognitive-os-governance-v0.1' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF 'reading-track-v0.1' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF 'hypothesis-track-v0.1' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF 'integration-demo-v0.1' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF 'multi-trace-validation-v0.1' OPERATOR_RELEASE_SNAPSHOT.md
for _sha in bbd1113 f6fa55a bb20acf 95b586d 460be0c; do
  if ! grep -qF "$_sha" OPERATOR_RELEASE_SNAPSHOT.md; then exit 1; fi
done
# The snapshot records the recovery + verification commands.
grep -qF 'git checkout' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF './scripts/release_check.sh' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF './scripts/operator_smoke.sh' OPERATOR_RELEASE_SNAPSHOT.md
# The training verdict is stated and P13-P15 are recorded closed.
grep -q 'training_not_justified' OPERATOR_RELEASE_SNAPSHOT.md
grep -q 'training_justified=false' OPERATOR_RELEASE_SNAPSHOT.md
grep -qF 'P13' OPERATOR_RELEASE_SNAPSHOT.md
# The snapshot records, verbatim, that it does NOT release remotely (the no-remote-release disclaimer).
grep -qF 'It does not release remotely.' OPERATOR_RELEASE_SNAPSHOT.md
# The six-line snapshot boundary is recorded verbatim (all six lines).
for _bl in 'The snapshot records the prototype state.' 'It does not release remotely.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_bl" OPERATOR_RELEASE_SNAPSHOT.md; then exit 1; fi
done
# The snapshot makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' OPERATOR_RELEASE_SNAPSHOT.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# OPS-3 — operator controls milestone freeze. The OPS-0 -> OPS-2 operator-controls arc (the operator manual,
# the executable smoke / manual drift guard, and the local release snapshot, all over the frozen prototype)
# is frozen as operator-controls-v0.1. The milestone record (OPERATOR_CONTROLS_MILESTONE.md) pins the
# OPS-0..OPS-2 commit lineage, the frozen base (the five prior milestone tags + commits), the release
# snapshot reference, the manual + smoke controls, the explain-and-verify boundary, the P12 training verdict,
# and the honest residuals, and is locked here so the freeze cannot silently drift. The pinned commit hashes
# are auditable against `git log`; this lock stays git-free and does NOT require the tag to exist (the tag is
# created only after a clean tree + green gate). Documentation freeze only — no code crate changes, no model,
# no training, NO remote release; the milestone records training_not_justified. Doctrine: The operator
# controls explain and verify the prototype. They do not release remotely. They do not create authority. They
# do not execute. They do not promote. They do not train.
# ---------------------------------------------------------------------------------------------------
test -f OPERATOR_CONTROLS_MILESTONE.md
grep -q 'FROZEN' OPERATOR_CONTROLS_MILESTONE.md
grep -q 'operator-controls-v0.1' OPERATOR_CONTROLS_MILESTONE.md
grep -q 'OPS-0' OPERATOR_CONTROLS_MILESTONE.md
grep -q 'OPS-2' OPERATOR_CONTROLS_MILESTONE.md
grep -q 'training_not_justified' OPERATOR_CONTROLS_MILESTONE.md
grep -q 'training_justified=false' OPERATOR_CONTROLS_MILESTONE.md
# Full OPS-0..OPS-2 commit lineage is pinned (cross-checkable against git log).
grep -qF '7aa17ec' OPERATOR_CONTROLS_MILESTONE.md
grep -qF 'c33dea7' OPERATOR_CONTROLS_MILESTONE.md
grep -qF '0876ba0' OPERATOR_CONTROLS_MILESTONE.md
# The five frozen base milestones are referenced as frozen deps (tag + commit).
for _t in cognitive-os-governance-v0.1 reading-track-v0.1 hypothesis-track-v0.1 integration-demo-v0.1 multi-trace-validation-v0.1; do
  if ! grep -qF "$_t" OPERATOR_CONTROLS_MILESTONE.md; then exit 1; fi
done
for _sha in bbd1113 f6fa55a bb20acf 95b586d 460be0c; do
  if ! grep -qF "$_sha" OPERATOR_CONTROLS_MILESTONE.md; then exit 1; fi
done
# The frozen operator controls are referenced by name (manual, smoke guard, release snapshot @ 0876ba0).
grep -qF 'OPERATOR_MANUAL.md' OPERATOR_CONTROLS_MILESTONE.md
grep -qF 'operator_smoke.sh' OPERATOR_CONTROLS_MILESTONE.md
grep -qF 'OPERATOR_RELEASE_SNAPSHOT.md' OPERATOR_CONTROLS_MILESTONE.md
# The six-line operator-controls boundary is recorded verbatim (all six lines).
for _bl in 'The operator controls explain and verify the prototype.' 'They do not release remotely.' 'They do not create authority.' 'They do not execute.' 'They do not promote.' 'They do not train.'; do
  if ! grep -qF "$_bl" OPERATOR_CONTROLS_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' OPERATOR_CONTROLS_MILESTONE.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# DOCFLOW-0 — operator-supplied document trace (crates/cognitive-demo). The doc flow runs the SAME
# end-to-end pipeline from a LOCAL operator-supplied text document instead of the fixed canonical corpus:
# the shell reads the file (path-validated — absolute / `..` / symlink-escape refused), the library asks the
# FROZEN reader for the document's OWN first span (corpus_from_documents — the same builder produce_run uses),
# builds a grounding plan, and starts the trace from a VERIFIED read0 receipt (fails closed if the read does
# not verify). The hypothesis cites the document receipt by hash; the probe is queued never executed; the
# observation is quarantined; promotion is refused; P12 stays training_justified=false. doc-bundle /
# doc-bundle-verify re-derive from the SAME document, so a tampered document, trace, report, questions, or
# manifest is refused. The document is READ, never TRUSTED — no std::fs in the library (already pinned). It
# adds NO Deserialize (already pinned), executes nothing, promotes nothing, trains nothing. Doctrine: The
# document flow reads local input. It does not trust local input. It verifies before tracing. It does not
# create authority. It does not execute. It does not promote. It does not train.
# ---------------------------------------------------------------------------------------------------
# Surface signals: the doc-flow API + commands exist and go through the frozen reader (not a hardcoded trace).
grep -q 'pub fn doc_trace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_doc_trace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_doc_report' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn doc_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_doc_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_doc_trace_json' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn check_local_input_path' crates/cognitive-demo/src/lib.rs
# The doc trace REALLY goes through the frozen pipeline: it builds via CognitiveTrace::build (which runs
# produce_run + verify_file) and reads the document's own span via the frozen corpus builder, so it cannot
# fake a receipt. A new direct dependency on the frozen reading-substrate gives the canonical sentence split.
grep -q 'CognitiveTrace::build(' crates/cognitive-demo/src/lib.rs
grep -q 'corpus_from_documents(' crates/cognitive-demo/src/lib.rs
grep -q 'reading-substrate' crates/cognitive-demo/Cargo.toml
# The 10 DOCFLOW-0 first-tests exist by name (a gutted/deleted test also drops the unit count pinned at 90 above).
for _dt in doc_trace_starts_from_verified_receipt doc_trace_cites_document_receipt_hash doc_bundle_verifies_clean_input doc_bundle_rejects_tampered_document doc_bundle_rejects_tampered_trace doc_bundle_rejects_tampered_report doc_bundle_rejects_tampered_manifest doc_input_path_is_local_and_safe doc_flow_does_not_change_training_gate doc_flow_does_not_execute_or_promote; do
  if ! grep -q "fn $_dt" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The seven-line DOCFLOW-0 boundary is recorded verbatim in the source (all seven lines).
for _dbl in 'The document flow reads local input.' 'It does not trust local input.' 'It verifies before tracing.' 'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  if ! grep -qF "$_dbl" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The shell validates the input path before reading (defense in depth: pure check + canonicalize+contain).
grep -q 'fn read_local_input' crates/cognitive-demo/src/main.rs
grep -q 'check_local_input_path' crates/cognitive-demo/src/main.rs
grep -q 'canonicalize' crates/cognitive-demo/src/main.rs
grep -q 'escapes the working directory' crates/cognitive-demo/src/main.rs
# BEHAVIORAL smoke: run the WHOLE doc flow end-to-end against a real local document, prove the boundary from
# the trace's OWN serialized output, and prove every tamper / unsafe-path is refused. The input doc lives
# under target/ (gitignored, inside the working dir) so the local-only path check accepts a relative path and
# no git debris is left. Fail-closed: any unexpected success removes the dir and aborts the gate.
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo >/dev/null 2>&1
_doc_dir="$(mktemp -d "$PWD/target/.docflow_gate.XXXXXX")"
_doc_rel="target/$(basename "$_doc_dir")"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$_doc_dir/doc.txt"
# doc-trace from a LOCAL relative path: writes the trace; the trace carries the document's own verified read
# and every boundary marker (verified receipt, cited hash, requires_operator, rejected, no evidence, training false).
./target/debug/cognitive-demo doc-trace --input "$_doc_rel/doc.txt" --out "$_doc_dir/trace.json" >/dev/null 2>&1 || { rm -rf "$_doc_dir"; exit 1; }
for _m in '"starts_from_verified_receipt": true' '"hypothesis_cites_receipt": true' '"reading_passed": true' '"nothing_executed": true' '"observation_quarantined": true' '"promotion_refused": true' '"nothing_becomes_evidence": true' '"execution_status": "requires_operator"' '"promotion_status": "rejected"' '"training_justified": false' '"training_gate_unchanged": true'; do
  if ! grep -qF "$_m" "$_doc_dir/trace.json"; then rm -rf "$_doc_dir"; exit 1; fi
done
# The trace REALLY read the operator's own text (not the canonical corpus): the answer is the document's first span.
grep -qF '"reading_answer": "The east bridge reopened today."' "$_doc_dir/trace.json" || { rm -rf "$_doc_dir"; exit 1; }
# No affirmative-authority status leaked into the doc trace.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$_doc_dir/trace.json"; then rm -rf "$_doc_dir"; exit 1; fi
# doc-report re-derives from the SAME input and renders (with the 9-line trace boundary).
./target/debug/cognitive-demo doc-report --input "$_doc_rel/doc.txt" --trace "$_doc_dir/trace.json" --out "$_doc_dir/report.txt" >/dev/null 2>&1 || { rm -rf "$_doc_dir"; exit 1; }
grep -qF 'Nothing trains.' "$_doc_dir/report.txt" || { rm -rf "$_doc_dir"; exit 1; }
# doc-bundle + doc-bundle-verify (clean) re-derive byte-identically and print the seven-line DOCFLOW boundary.
./target/debug/cognitive-demo doc-bundle --input "$_doc_rel/doc.txt" --out "$_doc_dir/pack" >/dev/null 2>&1 || { rm -rf "$_doc_dir"; exit 1; }
_doc_verify_out="$(./target/debug/cognitive-demo doc-bundle-verify --input "$_doc_rel/doc.txt" --path "$_doc_dir/pack" 2>/dev/null)" || { rm -rf "$_doc_dir"; exit 1; }
case "$_doc_verify_out" in *'doc-bundle-verify: OK'*) : ;; *) rm -rf "$_doc_dir"; exit 1 ;; esac
case "$_doc_verify_out" in *'The document flow reads local input.'*) : ;; *) rm -rf "$_doc_dir"; exit 1 ;; esac
# RE-DERIVE IS LOAD-BEARING: a tampered DOCUMENT must be refused (different doc -> different trace -> mismatch).
printf 'The west bridge collapsed today. Traffic stopped by noon.' > "$_doc_dir/doc2.txt"
if ./target/debug/cognitive-demo doc-bundle-verify --input "$_doc_rel/doc2.txt" --path "$_doc_dir/pack" >/dev/null 2>&1; then rm -rf "$_doc_dir"; exit 1; fi
# A tampered BUNDLE FILE must be refused.
printf '\n{tampered}' >> "$_doc_dir/pack/trace.json"
if ./target/debug/cognitive-demo doc-bundle-verify --input "$_doc_rel/doc.txt" --path "$_doc_dir/pack" >/dev/null 2>&1; then rm -rf "$_doc_dir"; exit 1; fi
# A tampered TRACE must be refused by doc-report.
printf '\n{tampered}' >> "$_doc_dir/trace.json"
if ./target/debug/cognitive-demo doc-report --input "$_doc_rel/doc.txt" --trace "$_doc_dir/trace.json" >/dev/null 2>&1; then rm -rf "$_doc_dir"; exit 1; fi
# UNSAFE input paths must be refused: absolute, parent traversal, and a symlink that escapes the working dir.
if ./target/debug/cognitive-demo doc-trace --input /etc/hostname >/dev/null 2>&1; then rm -rf "$_doc_dir"; exit 1; fi
if ./target/debug/cognitive-demo doc-trace --input "../etc-escape.txt" >/dev/null 2>&1; then rm -rf "$_doc_dir"; exit 1; fi
ln -s /etc/hostname "$_doc_dir/link.txt" 2>/dev/null
if ./target/debug/cognitive-demo doc-trace --input "$_doc_rel/link.txt" >/dev/null 2>&1; then rm -rf "$_doc_dir"; exit 1; fi
rm -rf "$_doc_dir"
# ---------------------------------------------------------------------------------------------------
# DOCFLOW-2 — document flow scenario pack / input-integrity matrix (crates/cognitive-demo). A finite,
# enum-backed set of VALID and INVALID document inputs, each OBSERVED by running the REAL DOCFLOW-0 check or
# verifier: a clean local document verifies; a modified document, a tampered bundle file (trace/report/
# manifest), an empty document, an absolute path, a `..` traversal, and a path that escapes the working
# directory are each REFUSED. doc-scenario-pack writes the observed-outcome record + report; doc-scenario-
# verify re-derives and refuses any tamper; doc-scenario-matrix verifies the pack then emits the coverage
# matrix. Every scenario keeps the boundary closed: local text is read never trusted, nothing executes,
# becomes evidence, promotes, or trains; P12 stays training_justified=false. No frozen crate edit. Doctrine:
# Document scenarios vary the input. They do not vary the authority. Local text is read, not trusted.
# Verification comes before tracing. Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# Surface signals: the DOCFLOW-2 API + commands exist and the containment decision is a shared pure fn.
grep -q 'pub enum DocScenario' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn doc_scenario_pack_files' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_doc_scenario_pack' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn doc_scenario_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn list_doc_scenarios' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn resolved_path_within' crates/cognitive-demo/src/lib.rs
# The shell's symlink-containment check goes through the SAME pure decision (single source of truth).
grep -q 'resolved_path_within(&cwd, &resolved)' crates/cognitive-demo/src/main.rs
# Each scenario OBSERVES the real check (proves not asserts): the pack runs the DOCFLOW-0 verifier/checks.
grep -q 'fn run_doc_scenario' crates/cognitive-demo/src/lib.rs
grep -q 'verify_doc_bundle(DOC_SCENARIO_SAMPLE' crates/cognitive-demo/src/lib.rs
# The 10 DOCFLOW-2 first-tests exist by name (a gutted/deleted test also drops the unit count pinned at 100 above).
for _dt in doc_scenarios_list_all_cases doc_clean_local_document_verifies doc_modified_input_invalidates_bundle doc_empty_document_fails_closed doc_absolute_path_refused doc_parent_traversal_refused doc_symlink_escape_refused doc_tampered_artifact_refused doc_scenario_matrix_records_outcomes doc_scenarios_do_not_change_training_gate; do
  if ! grep -q "fn $_dt" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The eight-line DOCFLOW-2 boundary is recorded verbatim in the source (all eight lines).
for _sbl in 'Document scenarios vary the input.' 'They do not vary the authority.' 'Local text is read, not trusted.' 'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_sbl" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# BEHAVIORAL smoke: run the WHOLE doc-scenario flow end-to-end, prove the coverage from the matrix's OWN
# serialized output, and prove a tampered pack is refused by BOTH verify and matrix. The pack lives under
# target/ (gitignored, inside the working dir) so no git debris is left. Fail-closed on any unexpected success.
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo >/dev/null 2>&1
_scn_dir="$(mktemp -d "$PWD/target/.docflow2_gate.XXXXXX")"
# doc-scenarios lists all nine input scenarios.
_scn_menu="$(./target/debug/cognitive-demo doc-scenarios)"
for _slug in clean-local-document modified-document empty-document absolute-path parent-traversal symlink-escape tampered-trace tampered-report tampered-manifest; do
  case "$_scn_menu" in *"$_slug"*) : ;; *) rm -rf "$_scn_dir"; exit 1 ;; esac
done
# doc-scenario-pack writes the pack; doc-scenario-verify accepts the clean pack and prints the boundary.
./target/debug/cognitive-demo doc-scenario-pack --out "$_scn_dir/pack" >/dev/null 2>&1 || { rm -rf "$_scn_dir"; exit 1; }
_scn_verify="$(./target/debug/cognitive-demo doc-scenario-verify --path "$_scn_dir/pack" 2>/dev/null)" || { rm -rf "$_scn_dir"; exit 1; }
case "$_scn_verify" in *'doc-scenario-verify: OK'*) : ;; *) rm -rf "$_scn_dir"; exit 1 ;; esac
case "$_scn_verify" in *'Local text is read, not trusted.'*) : ;; *) rm -rf "$_scn_dir"; exit 1 ;; esac
# doc-scenario-matrix verifies the pack then emits the coverage matrix; its OWN bytes prove the coverage.
./target/debug/cognitive-demo doc-scenario-matrix --path "$_scn_dir/pack" --out "$_scn_dir/matrix.json" >/dev/null 2>&1 || { rm -rf "$_scn_dir"; exit 1; }
for _m in '"verified_count": 1' '"refused_count": 8' '"cells_total": 36' '"cells_proven": 36' '"all_expectations_met": true' '"all_boundaries_hold": true'; do
  if ! grep -qF "$_m" "$_scn_dir/matrix.json"; then rm -rf "$_scn_dir"; exit 1; fi
done
# Every scenario slug appears in the matrix (it records all outcomes).
for _slug in clean-local-document modified-document empty-document absolute-path parent-traversal symlink-escape tampered-trace tampered-report tampered-manifest; do
  if ! grep -qF "\"$_slug\"" "$_scn_dir/matrix.json"; then rm -rf "$_scn_dir"; exit 1; fi
done
# No scenario produced an affirmative-authority status in the pack manifest.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$_scn_dir/pack/doc-scenario-pack.json"; then rm -rf "$_scn_dir"; exit 1; fi
# RE-DERIVE IS LOAD-BEARING: a tampered pack file must be refused by BOTH verify AND matrix.
printf '\n{tampered}' >> "$_scn_dir/pack/doc-scenario-pack.json"
if ./target/debug/cognitive-demo doc-scenario-verify --path "$_scn_dir/pack" >/dev/null 2>&1; then rm -rf "$_scn_dir"; exit 1; fi
if ./target/debug/cognitive-demo doc-scenario-matrix --path "$_scn_dir/pack" >/dev/null 2>&1; then rm -rf "$_scn_dir"; exit 1; fi
# END-TO-END input safety: the matrix records absolute-path / parent-traversal / symlink-escape as REFUSED.
# Prove those outcomes end-to-end through the binary (not only via the pure containment decision in the lib),
# so a regression in the shell's path validation cannot leave the matrix asserting a refusal that no longer
# happens. The doc commands read only a LOCAL relative path, so the escaping symlink lives under the scenario
# dir (inside target/, gitignored). Each unexpected success removes the dir and aborts the gate.
_scn_rel="target/$(basename "$_scn_dir")"
if ./target/debug/cognitive-demo doc-trace --input /etc/hostname >/dev/null 2>&1; then rm -rf "$_scn_dir"; exit 1; fi
if ./target/debug/cognitive-demo doc-trace --input "../etc-escape.txt" >/dev/null 2>&1; then rm -rf "$_scn_dir"; exit 1; fi
ln -s /etc/hostname "$_scn_dir/escape.txt" 2>/dev/null
if ./target/debug/cognitive-demo doc-trace --input "$_scn_rel/escape.txt" >/dev/null 2>&1; then rm -rf "$_scn_dir"; exit 1; fi
rm -rf "$_scn_dir"
# ---------------------------------------------------------------------------------------------------
# DOCFLOW-3 — document flow milestone freeze. The DOCFLOW-0 -> DOCFLOW-2 local-document-flow arc (the
# operator-supplied document trace, the operator manual + smoke guard for the doc commands, and the
# input-integrity scenario pack / matrix) is frozen as document-flow-v0.1. The milestone record
# (DOCUMENT_FLOW_MILESTONE.md) pins the DOCFLOW-0..DOCFLOW-2 commit lineage, the frozen base
# (operator-controls-v0.1 + the five deeper milestone tags + commits), the demonstrated capability, the
# read-not-trust boundary, the P12 training verdict, and the honest residuals, and is locked here so the
# freeze cannot silently drift. The pinned commit hashes are auditable against `git log`; this lock stays
# git-free and does NOT require the tag to exist (the tag is created only after a clean tree + green gate).
# Documentation freeze only — no code crate change, no model, no training; the milestone records
# training_not_justified. Doctrine: The document flow reads local input. It does not trust local input.
# Document scenarios vary the input. They do not vary the authority. Verification comes before tracing.
# Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
test -f DOCUMENT_FLOW_MILESTONE.md
grep -q 'FROZEN' DOCUMENT_FLOW_MILESTONE.md
grep -q 'document-flow-v0.1' DOCUMENT_FLOW_MILESTONE.md
grep -q 'DOCFLOW-0' DOCUMENT_FLOW_MILESTONE.md
grep -q 'DOCFLOW-2' DOCUMENT_FLOW_MILESTONE.md
grep -q 'training_not_justified' DOCUMENT_FLOW_MILESTONE.md
grep -q 'training_justified=false' DOCUMENT_FLOW_MILESTONE.md
# Full DOCFLOW-0..DOCFLOW-2 commit lineage is pinned (cross-checkable against git log).
grep -qF 'c9bd1e5' DOCUMENT_FLOW_MILESTONE.md
grep -qF 'b288196' DOCUMENT_FLOW_MILESTONE.md
grep -qF '4a04759' DOCUMENT_FLOW_MILESTONE.md
# The frozen base (operator-controls-v0.1) and the five deeper frozen milestones are referenced (tag + commit).
for _t in operator-controls-v0.1 multi-trace-validation-v0.1 integration-demo-v0.1 hypothesis-track-v0.1 reading-track-v0.1 cognitive-os-governance-v0.1; do
  if ! grep -qF "$_t" DOCUMENT_FLOW_MILESTONE.md; then exit 1; fi
done
for _sha in 34b4f47 460be0c 95b586d bb20acf f6fa55a bbd1113; do
  if ! grep -qF "$_sha" DOCUMENT_FLOW_MILESTONE.md; then exit 1; fi
done
# The three frozen document-flow capabilities are referenced by name (capability, operator guard, scenarios).
grep -qF 'doc-trace' DOCUMENT_FLOW_MILESTONE.md
grep -qF 'OPERATOR_MANUAL.md' DOCUMENT_FLOW_MILESTONE.md
grep -qF 'doc-scenario-matrix' DOCUMENT_FLOW_MILESTONE.md
# The nine-line document-flow boundary is recorded verbatim (all nine lines).
for _bl in 'The document flow reads local input.' 'It does not trust local input.' 'Document scenarios vary the input.' 'They do not vary the authority.' 'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" DOCUMENT_FLOW_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' DOCUMENT_FLOW_MILESTONE.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# CORPUS-0 — multi-document local corpus trace / source-selection boundary (crates/cognitive-demo). Where
# DOCFLOW-0 traces ONE operator document, CORPUS-0 traces a small LOCAL CORPUS DIRECTORY of `.txt` documents
# through the SAME end-to-end pipeline: the shell enumerates the directory (path-validated — absolute / `..` /
# symlink-escape refused; only non-hidden `.txt` files admitted; sorted for determinism), the library asks the
# FROZEN reader for the corpus's OWN first span (corpus_from_documents) and starts the trace from a VERIFIED
# read0 receipt (fails closed with EmptyCorpus if the corpus grounds nothing). The receipt's structure hash
# binds EVERY document, so a tamper of any document — even a non-grounding one — re-derives a different trace
# and is refused. An unambiguous corpus-source.json records which document/span grounded the answer. The
# corpus is READ, never TRUSTED: nothing executes, becomes evidence, promotes, or trains; P12 stays
# training_justified=false. No frozen crate edit; the library stays fs-free (pinned above). Doctrine: The
# corpus flow reads local documents. It does not trust local documents. Source selection is verified and
# replayable. Verification comes before tracing. Nothing executes. Nothing becomes evidence. Nothing
# promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# Surface signals: the corpus-flow API + commands exist and go through the frozen reader (not a hardcoded trace).
grep -q 'pub fn corpus_trace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_corpus_trace' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_corpus_report' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn corpus_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_corpus_bundle' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_corpus_trace_json' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn corpus_admits_filename' crates/cognitive-demo/src/lib.rs
# The corpus trace REALLY goes through the frozen pipeline (build + frozen corpus builder), and the source
# attribution + empty-corpus fail-closed are derived from the frozen metadata, never asserted.
grep -q 'fn corpus_inputs' crates/cognitive-demo/src/lib.rs
grep -q 'fn corpus_source' crates/cognitive-demo/src/lib.rs
grep -q 'EmptyCorpus' crates/cognitive-demo/src/lib.rs
# The shell reads the DIRECTORY with path validation + the admits filter + per-entry canonicalize-and-contain.
grep -q 'fn read_local_corpus' crates/cognitive-demo/src/main.rs
grep -q 'corpus_admits_filename' crates/cognitive-demo/src/main.rs
grep -q 'resolved_path_within(&root, &resolved)' crates/cognitive-demo/src/main.rs
# The 12 CORPUS-0 first-tests exist by name (a gutted/deleted test also drops the unit count pinned at 139 above).
for _ct in corpus_trace_starts_from_verified_receipt corpus_trace_cites_receipt_hash corpus_trace_records_grounding_document_and_span corpus_admits_only_plain_local_txt_files corpus_empty_fails_closed corpus_bundle_verifies_clean_input corpus_bundle_rejects_tampered_corpus corpus_bundle_rejects_tampered_artifact corpus_report_records_source_selection_and_refuses_tamper corpus_flow_does_not_change_training_gate corpus_flow_does_not_execute_or_promote corpus_source_is_deterministic_and_replayable; do
  if ! grep -q "fn $_ct" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The eight-line CORPUS-0 boundary is recorded verbatim in the source (all eight lines).
for _cbl in 'The corpus flow reads local documents.' 'It does not trust local documents.' 'Source selection is verified and replayable.' 'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_cbl" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# BEHAVIORAL smoke: run the WHOLE corpus flow end-to-end against a real local corpus directory, prove the
# boundary + source selection from the OWN serialized output, prove the directory filter (hidden / non-txt /
# symlink excluded), and prove every tamper / empty / unsafe-path is refused. The corpus lives under target/
# (gitignored, inside the working dir) so the local-only path check accepts a relative path and no git debris
# is left. Fail-closed: any unexpected success removes the dir and aborts the gate.
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo >/dev/null 2>&1
_cor_dir="$(mktemp -d "$PWD/target/.corpus_gate.XXXXXX")"
_cor_rel="target/$(basename "$_cor_dir")"
mkdir -p "$_cor_dir/corpus"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$_cor_dir/corpus/a-east.txt"
printf 'The west tunnel remains closed. Crews continue repairs.' > "$_cor_dir/corpus/b-west.txt"
printf 'hidden secret.' > "$_cor_dir/corpus/.hidden.txt"
printf 'ignored note.' > "$_cor_dir/corpus/notes.md"
ln -s /etc/hostname "$_cor_dir/corpus/escape.txt" 2>/dev/null
# corpus-trace from a LOCAL relative dir: writes the trace; it carries the corpus's own verified read and
# every boundary marker (verified receipt, cited hash, requires_operator, rejected, no evidence, training false).
./target/debug/cognitive-demo corpus-trace --input-dir "$_cor_rel/corpus" --out "$_cor_dir/trace.json" >/dev/null 2>&1 || { rm -rf "$_cor_dir"; exit 1; }
for _m in '"starts_from_verified_receipt": true' '"hypothesis_cites_receipt": true' '"reading_passed": true' '"nothing_executed": true' '"observation_quarantined": true' '"promotion_refused": true' '"nothing_becomes_evidence": true' '"execution_status": "requires_operator"' '"promotion_status": "rejected"' '"training_justified": false' '"training_gate_unchanged": true'; do
  if ! grep -qF "$_m" "$_cor_dir/trace.json"; then rm -rf "$_cor_dir"; exit 1; fi
done
# The trace REALLY read the corpus's own first span (the grounding document's first sentence).
grep -qF '"reading_answer": "The east bridge reopened today."' "$_cor_dir/trace.json" || { rm -rf "$_cor_dir"; exit 1; }
# No affirmative-authority status leaked into the corpus trace.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$_cor_dir/trace.json"; then rm -rf "$_cor_dir"; exit 1; fi
# corpus-report re-derives from the SAME corpus and renders the SOURCE SELECTION (grounded document/span,
# unambiguous), lists exactly the TWO admitted documents (hidden / non-txt / symlink were refused), and the boundary.
./target/debug/cognitive-demo corpus-report --input-dir "$_cor_rel/corpus" --trace "$_cor_dir/trace.json" --out "$_cor_dir/report.txt" >/dev/null 2>&1 || { rm -rf "$_cor_dir"; exit 1; }
for _rm in 'SOURCE SELECTION' 'grounded document:  [0] a-east.txt' 'corpus documents:   2' 'Nothing trains.'; do
  if ! grep -qF "$_rm" "$_cor_dir/report.txt"; then rm -rf "$_cor_dir"; exit 1; fi
done
# The refused entries never became documents: their names/content do not appear in the report.
if grep -qE 'hidden|notes\.md|escape\.txt' "$_cor_dir/report.txt"; then rm -rf "$_cor_dir"; exit 1; fi
# corpus-bundle + corpus-bundle-verify (clean) re-derive byte-identically; the source attribution is unambiguous.
./target/debug/cognitive-demo corpus-bundle --input-dir "$_cor_rel/corpus" --out "$_cor_dir/pack" >/dev/null 2>&1 || { rm -rf "$_cor_dir"; exit 1; }
for _sm in '"document_title": "a-east.txt"' '"span_id": 0' '"span_text": "The east bridge reopened today."'; do
  if ! grep -qF "$_sm" "$_cor_dir/pack/corpus-source.json"; then rm -rf "$_cor_dir"; exit 1; fi
done
_cor_verify_out="$(./target/debug/cognitive-demo corpus-bundle-verify --input-dir "$_cor_rel/corpus" --path "$_cor_dir/pack" 2>/dev/null)" || { rm -rf "$_cor_dir"; exit 1; }
case "$_cor_verify_out" in *'corpus-bundle-verify: OK'*) : ;; *) rm -rf "$_cor_dir"; exit 1 ;; esac
case "$_cor_verify_out" in *'The corpus flow reads local documents.'*) : ;; *) rm -rf "$_cor_dir"; exit 1 ;; esac
# RE-DERIVE IS LOAD-BEARING: a tampered CORPUS must be refused. Change the NON-grounding second document — the
# structure hash binds every document, so even a non-grounding edit re-derives a different trace -> mismatch.
printf 'The west tunnel reopened early. Crews left.' > "$_cor_dir/corpus/b-west.txt"
if ./target/debug/cognitive-demo corpus-bundle-verify --input-dir "$_cor_rel/corpus" --path "$_cor_dir/pack" >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
printf 'The west tunnel remains closed. Crews continue repairs.' > "$_cor_dir/corpus/b-west.txt"
# A tampered BUNDLE FILE must be refused.
printf '\n{tampered}' >> "$_cor_dir/pack/trace.json"
if ./target/debug/cognitive-demo corpus-bundle-verify --input-dir "$_cor_rel/corpus" --path "$_cor_dir/pack" >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
# A tampered TRACE must be refused by corpus-report.
printf '\n{tampered}' >> "$_cor_dir/trace.json"
if ./target/debug/cognitive-demo corpus-report --input-dir "$_cor_rel/corpus" --trace "$_cor_dir/trace.json" >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
# An EMPTY corpus fails closed (no admitted document grounds a span).
mkdir -p "$_cor_dir/empty"
if ./target/debug/cognitive-demo corpus-trace --input-dir "$_cor_rel/empty" >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
# UNSAFE corpus paths must be refused: absolute, parent traversal, and a symlink DIR that escapes the working dir.
if ./target/debug/cognitive-demo corpus-trace --input-dir /etc >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
if ./target/debug/cognitive-demo corpus-trace --input-dir "../etc-escape" >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
ln -s /etc "$_cor_dir/linkdir" 2>/dev/null
if ./target/debug/cognitive-demo corpus-trace --input-dir "$_cor_rel/linkdir" >/dev/null 2>&1; then rm -rf "$_cor_dir"; exit 1; fi
rm -rf "$_cor_dir"
# ---------------------------------------------------------------------------------------------------
# CORPUS-2 — corpus scenario pack / input-integrity matrix (crates/cognitive-demo). Where CORPUS-0 traces ONE
# clean corpus and CORPUS-1 documents the operator path, CORPUS-2 makes corpus behavior AUDITABLE across a
# finite, enum-backed matrix of VALID and INVALID corpus inputs (the corpus analog of DOCFLOW-2), each OBSERVED
# by running the REAL CORPUS-0 admission filter / check / verifier: a clean two-document corpus verifies; an
# empty corpus, a hidden-only or non-.txt-only corpus, an absolute / `..` / escaping path, a grounding-document
# mutation, a non-grounding side-document mutation, and a tampered source/trace/report/manifest are each REFUSED.
# corpus-scenario-pack writes the observed-outcome record + report; corpus-scenario-verify re-derives and refuses
# any tamper; corpus-scenario-matrix verifies the pack then emits the matrix, which ALSO records the verified
# case's SOURCE IDENTITY and a whole_corpus_bound fact (mutating a non-grounding document leaves the attribution
# byte-identical yet still fails the bundle on trace.json — the structure hash binds the WHOLE corpus). Every
# scenario keeps the boundary closed: nothing executes, becomes evidence, promotes, or trains; P12 stays
# training_justified=false. No frozen crate edit; the library stays fs-free (pinned above). Doctrine: Corpus
# scenarios vary the corpus input. They do not vary the authority. Source selection is verified and replayable.
# The whole corpus is hash-bound. Verification comes before tracing. Nothing executes. Nothing becomes evidence.
# Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
# Surface signals: the CORPUS-2 API + commands exist and the shared pure decisions back the path scenarios.
grep -q 'pub enum CorpusScenario' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn corpus_scenario_pack_files' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_corpus_scenario_pack' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn corpus_scenario_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn list_corpus_scenarios' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn resolved_path_within' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn corpus_admits_filename' crates/cognitive-demo/src/lib.rs
# Each scenario OBSERVES the real check (proves not asserts): the pack runs the CORPUS-0 verifier/checks, and the
# whole-corpus binding is proven structurally (source unchanged, bundle still refused), never asserted.
grep -q 'fn run_corpus_scenario' crates/cognitive-demo/src/lib.rs
grep -q 'fn corpus_whole_binding_holds' crates/cognitive-demo/src/lib.rs
# The 12 CORPUS-2 first-tests exist by name (a gutted/deleted test also drops the unit count pinned at 139 above).
for _c2t in corpus_scenarios_list_all_cases corpus_clean_two_document_case_verifies corpus_empty_case_fails_closed corpus_hidden_only_case_refused corpus_non_txt_only_case_refused corpus_absolute_path_refused corpus_parent_traversal_refused corpus_symlink_escape_refused corpus_grounding_doc_mutation_invalidates_bundle corpus_side_doc_mutation_invalidates_bundle corpus_tampered_artifacts_refused corpus_scenario_matrix_records_source_and_boundaries; do
  if ! grep -q "fn $_c2t" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The nine-line CORPUS-2 boundary is recorded verbatim in the source (all nine lines).
for _c2bl in 'Corpus scenarios vary the corpus input.' 'They do not vary the authority.' 'Source selection is verified and replayable.' 'The whole corpus is hash-bound.' 'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_c2bl" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# BEHAVIORAL smoke: run the WHOLE corpus-scenario flow end-to-end, prove the coverage + source identity from the
# matrix's OWN serialized output, prove the whole-corpus-binding distinction is genuinely demonstrated, and prove
# a tampered pack is refused by BOTH verify and matrix. The pack lives under target/ (gitignored) so no git debris
# is left. Fail-closed on any unexpected success.
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo >/dev/null 2>&1
_c2_dir="$(mktemp -d "$PWD/target/.corpus2_gate.XXXXXX")"
_c2_rel="target/$(basename "$_c2_dir")"
# corpus-scenarios lists all thirteen input scenarios.
_c2_menu="$(./target/debug/cognitive-demo corpus-scenarios)"
for _slug in clean-two-document empty-corpus hidden-only non-txt-only absolute-path parent-traversal symlink-escape grounding-mutation side-document-mutation tampered-source tampered-trace tampered-report tampered-manifest; do
  case "$_c2_menu" in *"$_slug"*) : ;; *) rm -rf "$_c2_dir"; exit 1 ;; esac
done
# corpus-scenario-pack writes the pack; corpus-scenario-verify accepts the clean pack and prints the boundary.
./target/debug/cognitive-demo corpus-scenario-pack --out "$_c2_dir/pack" >/dev/null 2>&1 || { rm -rf "$_c2_dir"; exit 1; }
_c2_verify="$(./target/debug/cognitive-demo corpus-scenario-verify --path "$_c2_dir/pack" 2>/dev/null)" || { rm -rf "$_c2_dir"; exit 1; }
case "$_c2_verify" in *'corpus-scenario-verify: OK'*) : ;; *) rm -rf "$_c2_dir"; exit 1 ;; esac
case "$_c2_verify" in *'The whole corpus is hash-bound.'*) : ;; *) rm -rf "$_c2_dir"; exit 1 ;; esac
# corpus-scenario-matrix verifies the pack then emits the coverage matrix; its OWN bytes prove the coverage.
./target/debug/cognitive-demo corpus-scenario-matrix --path "$_c2_dir/pack" --out "$_c2_dir/matrix.json" >/dev/null 2>&1 || { rm -rf "$_c2_dir"; exit 1; }
for _m in '"verified_count": 1' '"refused_count": 12' '"cells_total": 52' '"cells_proven": 52' '"all_expectations_met": true' '"all_boundaries_hold": true' '"whole_corpus_bound": true'; do
  if ! grep -qF "$_m" "$_c2_dir/matrix.json"; then rm -rf "$_c2_dir"; exit 1; fi
done
# The matrix records the verified case's SOURCE IDENTITY (which document/span grounded the answer).
for _s in '"document_title": "a-east.txt"' '"span_id": 0' '"span_text": "The east bridge reopened today."'; do
  if ! grep -qF "$_s" "$_c2_dir/matrix.json"; then rm -rf "$_c2_dir"; exit 1; fi
done
# WHOLE-CORPUS BINDING is genuinely demonstrated (not just a boolean): the grounding mutation breaks the source
# attribution first, while the non-grounding side mutation leaves it intact yet still breaks the whole-corpus trace.
if ! grep -qF '"rejection_reason": "bundle-file-mismatch:corpus-source.json"' "$_c2_dir/matrix.json"; then rm -rf "$_c2_dir"; exit 1; fi
if ! grep -qF '"rejection_reason": "bundle-file-mismatch:trace.json"' "$_c2_dir/matrix.json"; then rm -rf "$_c2_dir"; exit 1; fi
# Every scenario slug appears in the matrix (it records all outcomes).
for _slug in clean-two-document empty-corpus hidden-only non-txt-only absolute-path parent-traversal symlink-escape grounding-mutation side-document-mutation tampered-source tampered-trace tampered-report tampered-manifest; do
  if ! grep -qF "\"$_slug\"" "$_c2_dir/matrix.json"; then rm -rf "$_c2_dir"; exit 1; fi
done
# No scenario produced an affirmative-authority status in the pack manifest.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$_c2_dir/pack/corpus-scenario-pack.json"; then rm -rf "$_c2_dir"; exit 1; fi
# RE-DERIVE IS LOAD-BEARING: a tampered pack file must be refused by BOTH verify AND matrix.
printf '\n{tampered}' >> "$_c2_dir/pack/corpus-scenario-pack.json"
if ./target/debug/cognitive-demo corpus-scenario-verify --path "$_c2_dir/pack" >/dev/null 2>&1; then rm -rf "$_c2_dir"; exit 1; fi
if ./target/debug/cognitive-demo corpus-scenario-matrix --path "$_c2_dir/pack" >/dev/null 2>&1; then rm -rf "$_c2_dir"; exit 1; fi
# END-TO-END input safety: the matrix records hidden-only and non-.txt-only corpora as REFUSED. Prove those
# outcomes end-to-end through the binary (not only via the pure admission filter in the lib), so a regression in
# the shell's directory enumeration cannot leave the matrix asserting a refusal that no longer happens. A corpus
# of only hidden files and a corpus of only non-.txt files each admit ZERO documents -> corpus-trace fails closed.
mkdir -p "$_c2_dir/hidden_only" "$_c2_dir/non_txt_only"
printf 'hidden a.' > "$_c2_dir/hidden_only/.secret.txt"
printf 'hidden b.' > "$_c2_dir/hidden_only/.hidden.txt"
printf 'note.' > "$_c2_dir/non_txt_only/notes.md"
printf '{}' > "$_c2_dir/non_txt_only/data.json"
if ./target/debug/cognitive-demo corpus-trace --input-dir "$_c2_rel/hidden_only" >/dev/null 2>&1; then rm -rf "$_c2_dir"; exit 1; fi
if ./target/debug/cognitive-demo corpus-trace --input-dir "$_c2_rel/non_txt_only" >/dev/null 2>&1; then rm -rf "$_c2_dir"; exit 1; fi
rm -rf "$_c2_dir"
# ---------------------------------------------------------------------------------------------------
# CORPUS-3 — corpus flow milestone freeze. The CORPUS-0 -> CORPUS-2 multi-document local-corpus arc (the
# multi-document corpus trace + source-selection boundary, the operator manual + smoke guard for the corpus
# commands, and the input-integrity scenario pack / matrix) is frozen as corpus-flow-v0.1. The milestone
# record (CORPUS_FLOW_MILESTONE.md) pins the CORPUS-0..CORPUS-2 commit lineage, the frozen base
# (document-flow-v0.1 as the prior frozen local-document base + the six deeper milestone tags + commits), the
# demonstrated capability, the read-not-trust boundary, the whole-corpus binding + non-grounding side-document
# mutation behavior, the matrix source identity, the P12 training verdict, and the honest residuals, and is
# locked here so the freeze cannot silently drift. The pinned commit hashes are auditable against `git log`;
# this lock stays git-free and does NOT require the tag to exist (the tag is created only after a clean tree +
# green gate). Documentation freeze only — no code crate change, no model, no training; the milestone records
# training_not_justified. Doctrine: The corpus flow reads local documents. It does not trust local documents.
# Source selection is verified and replayable. The whole corpus is hash-bound. Corpus scenarios vary the input.
# They do not vary the authority. Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
test -f CORPUS_FLOW_MILESTONE.md
grep -q 'FROZEN' CORPUS_FLOW_MILESTONE.md
grep -q 'corpus-flow-v0.1' CORPUS_FLOW_MILESTONE.md
grep -q 'CORPUS-0' CORPUS_FLOW_MILESTONE.md
grep -q 'CORPUS-1' CORPUS_FLOW_MILESTONE.md
grep -q 'CORPUS-2' CORPUS_FLOW_MILESTONE.md
grep -q 'training_not_justified' CORPUS_FLOW_MILESTONE.md
grep -q 'training_justified=false' CORPUS_FLOW_MILESTONE.md
# Full CORPUS-0..CORPUS-2 commit lineage is pinned (cross-checkable against git log).
grep -qF 'b19dc47' CORPUS_FLOW_MILESTONE.md
grep -qF 'ae58b99' CORPUS_FLOW_MILESTONE.md
grep -qF 'e0791ed' CORPUS_FLOW_MILESTONE.md
# document-flow-v0.1 is named as the prior frozen local-document base, and the six deeper frozen milestones are
# referenced (tag + commit).
grep -qF 'prior frozen local-document base' CORPUS_FLOW_MILESTONE.md
for _t in document-flow-v0.1 operator-controls-v0.1 multi-trace-validation-v0.1 integration-demo-v0.1 hypothesis-track-v0.1 reading-track-v0.1 cognitive-os-governance-v0.1; do
  if ! grep -qF "$_t" CORPUS_FLOW_MILESTONE.md; then exit 1; fi
done
for _sha in 0cc7399 34b4f47 460be0c 95b586d bb20acf f6fa55a bbd1113; do
  if ! grep -qF "$_sha" CORPUS_FLOW_MILESTONE.md; then exit 1; fi
done
# The three frozen corpus capabilities are referenced by name (capability, operator guard, scenarios).
grep -qF 'corpus-trace' CORPUS_FLOW_MILESTONE.md
grep -qF 'OPERATOR_MANUAL.md' CORPUS_FLOW_MILESTONE.md
grep -qF 'corpus-scenario-matrix' CORPUS_FLOW_MILESTONE.md
# The CORPUS-2-specific properties the rubric requires are recorded: matrix source identity AND whole-corpus
# binding, including the non-grounding side-document mutation behavior (a side doc cannot silently pass).
grep -qF 'source identity' CORPUS_FLOW_MILESTONE.md
grep -qF 'whole_corpus_bound' CORPUS_FLOW_MILESTONE.md
grep -qF 'The whole corpus is hash-bound.' CORPUS_FLOW_MILESTONE.md
grep -qF 'non-grounding side-document mutation' CORPUS_FLOW_MILESTONE.md
# The ten-line corpus-flow boundary is recorded verbatim (all ten lines).
for _bl in 'The corpus flow reads local documents.' 'It does not trust local documents.' 'Source selection is verified and replayable.' 'The whole corpus is hash-bound.' 'Corpus scenarios vary the input.' 'They do not vary the authority.' 'Nothing executes.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" CORPUS_FLOW_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' CORPUS_FLOW_MILESTONE.md; then exit 1; fi
# ---------------------------------------------------------------------------------------------------
# NOVELTY-0 — hypothesis-only novelty packet harness (crates/cognitive-demo). ON TOP of the verified corpus
# trace, NOVELTY-0 adds a bounded HYPOTHESIS layer: given a verified corpus trace (re-derived from --input-dir,
# with --corpus-trace byte-verified against it) and an operator --frame, `novelty-packet` emits a deterministic
# NoveltyPacket recording the frame's candidate broken assumptions, the verified facts to preserve (each
# grounded VERBATIM in a verified corpus span), a candidate hypothesis, falsifiers, and NON-EXECUTING probe
# requests. The packet carries authority=hypothesis_only (an enum with no evidence/promoted/truth variant) and
# an explicit forbidden_uses list, so it can never become evidence, execute, promote, or train. NO model, NO
# score: the frame is read as DATA (never grounded as a fact), and an unsupported preserved fact, an empty
# frame, a receipt-hash-stripped corpus trace, or any tampered packet is REFUSED by re-derivation. P12 stays
# training_justified=false. Doctrine: Novelty packets propose. They do not prove. They cite verified receipts.
# They do not create authority. Probe requests do not execute. Nothing becomes evidence, promotes, or trains.
# ---------------------------------------------------------------------------------------------------
# Surface signals: the NOVELTY-0 API + the three commands exist (lib + shell).
grep -q 'struct NoveltyPacket' crates/cognitive-demo/src/lib.rs
grep -q 'struct NoveltyProbeRequest' crates/cognitive-demo/src/lib.rs
grep -q 'enum NoveltyAuthority' crates/cognitive-demo/src/lib.rs
grep -q 'rename = "hypothesis_only"' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_novelty_packet' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_novelty_report' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_novelty_replay' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_novelty_packet_json' crates/cognitive-demo/src/lib.rs
grep -q '"novelty-packet"' crates/cognitive-demo/src/main.rs
grep -q '"novelty-report"' crates/cognitive-demo/src/main.rs
grep -q '"novelty-replay"' crates/cognitive-demo/src/main.rs
grep -q 'fn read_frame' crates/cognitive-demo/src/main.rs
grep -qF -- '--corpus-trace' crates/cognitive-demo/src/main.rs
grep -qF -- '--frame' crates/cognitive-demo/src/main.rs
grep -qF -- '--packet' crates/cognitive-demo/src/main.rs
# GROUNDED-IN-A-VERIFIED-TRACE + GROUNDING-GATE (proves, not asserts): the packet is derived from the VERIFIED
# corpus trace, and a preserved fact MUST be a verified corpus span (an unsupported fact is refused).
grep -q 'fn novelty_packet(' crates/cognitive-demo/src/lib.rs
grep -q 'let trace = corpus_trace(documents)?;' crates/cognitive-demo/src/lib.rs
grep -q 'fn novelty_facts_grounded(' crates/cognitive-demo/src/lib.rs
grep -q 'fn corpus_verified_spans(' crates/cognitive-demo/src/lib.rs
grep -q 'TraceError::UnsupportedPreservedFact' crates/cognitive-demo/src/lib.rs
grep -q 'TraceError::MissingReceiptHash' crates/cognitive-demo/src/lib.rs
grep -q 'TraceError::EmptyFrame' crates/cognitive-demo/src/lib.rs
grep -q 'TraceError::NoveltyPacketMismatch' crates/cognitive-demo/src/lib.rs
# The 15 NOVELTY-0 first-tests exist by name (a gutted/deleted test also drops the unit count pinned at 139 above).
for _t in \
  novelty_packet_requires_verified_corpus_receipt \
  novelty_packet_cites_receipt_and_source_identity \
  novelty_packet_authority_is_hypothesis_only \
  novelty_packet_records_broken_assumptions \
  novelty_packet_records_preserved_facts_grounded \
  novelty_packet_records_falsifiers \
  novelty_probe_requests_do_not_execute \
  novelty_packet_cannot_become_evidence_or_promote_or_train \
  novelty_packet_replay_is_deterministic \
  novelty_packet_rejects_tampered_packet \
  novelty_facts_grounded_rejects_unsupported_fact \
  novelty_packet_refuses_corpus_trace_missing_receipt_hash \
  novelty_packet_does_not_change_training_gate \
  novelty_frame_text_is_not_trusted_as_fact \
  novelty_empty_frame_fails_closed; do
  if ! grep -q "fn $_t(" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The eight-line NOVELTY-0 boundary is recorded verbatim in the source (all eight lines).
for _bl in \
  'Novelty packets propose.' \
  'They do not prove.' \
  'They cite verified receipts.' \
  'They do not create authority.' \
  'Probe requests do not execute.' \
  'Nothing becomes evidence.' \
  'Nothing promotes.' \
  'Nothing trains.'; do
  if ! grep -qF "$_bl" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# BINARY SMOKE: run the whole NOVELTY-0 flow against a REAL local corpus + frame under the gitignored target/
# directory (relative path, since the corpus/frame commands only read inside the working dir), and prove the
# hypothesis-only boundary from the packet's OWN bytes, plus every refusal end-to-end through the binary.
_nv_dir="target/.novelty_gate.$$"
_nv_rel="$_nv_dir"
mkdir -p "$_nv_dir/corpus"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$_nv_dir/corpus/a-east.txt"
printf 'The west tunnel remains closed. Crews continue repairs.' > "$_nv_dir/corpus/b-west.txt"
printf 'The east bridge stays closed indefinitely.\nTraffic never recovers after a closure.\n' > "$_nv_dir/frame.txt"
./target/debug/cognitive-demo corpus-trace --input-dir "$_nv_rel/corpus" --out "$_nv_dir/trace.json" >/dev/null 2>&1 || { rm -rf "$_nv_dir"; exit 1; }
./target/debug/cognitive-demo novelty-packet --input-dir "$_nv_rel/corpus" --corpus-trace "$_nv_dir/trace.json" --frame "$_nv_rel/frame.txt" --out "$_nv_dir/novelty.json" >/dev/null 2>&1 || { rm -rf "$_nv_dir"; exit 1; }
# Authority is hypothesis_only; there is no score and no affirmative-authority status.
grep -q '"authority": "hypothesis_only"' "$_nv_dir/novelty.json" || { rm -rf "$_nv_dir"; exit 1; }
if grep -q '"score"' "$_nv_dir/novelty.json"; then rm -rf "$_nv_dir"; exit 1; fi
# Every probe request is NON-executing (executes:false), and none executes.
grep -q '"executes": false' "$_nv_dir/novelty.json" || { rm -rf "$_nv_dir"; exit 1; }
if grep -q '"executes": true' "$_nv_dir/novelty.json"; then rm -rf "$_nv_dir"; exit 1; fi
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$_nv_dir/novelty.json"; then rm -rf "$_nv_dir"; exit 1; fi
# forbidden_uses records exactly the four refused uses.
for _fu in evidence execution promotion training; do
  if ! grep -q "\"$_fu\"" "$_nv_dir/novelty.json"; then rm -rf "$_nv_dir"; exit 1; fi
done
# The eight boundary lines are present in the packet's own bytes.
for _bl in 'Novelty packets propose.' 'They do not prove.' 'Nothing becomes evidence.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" "$_nv_dir/novelty.json"; then rm -rf "$_nv_dir"; exit 1; fi
done
# THE LOAD-BEARING GROUNDING PROPERTY: the preserved fact is the VERIFIED corpus span, NOT the operator frame's
# claim. The verified span is preserved; the frame's claim is a broken-assumption candidate, never a fact.
grep -q '"The east bridge reopened today."' "$_nv_dir/novelty.json" || { rm -rf "$_nv_dir"; exit 1; }
./target/debug/cognitive-demo novelty-report --input-dir "$_nv_rel/corpus" --frame "$_nv_rel/frame.txt" --packet "$_nv_dir/novelty.json" > "$_nv_dir/report.txt" 2>&1 || { rm -rf "$_nv_dir"; exit 1; }
grep -q 'PROPOSAL ONLY' "$_nv_dir/report.txt" || { rm -rf "$_nv_dir"; exit 1; }
grep -q 'PRESERVED FACTS' "$_nv_dir/report.txt" || { rm -rf "$_nv_dir"; exit 1; }
# Replay confirms deterministic re-derivation.
./target/debug/cognitive-demo novelty-replay --input-dir "$_nv_rel/corpus" --frame "$_nv_rel/frame.txt" --packet "$_nv_dir/novelty.json" > "$_nv_dir/replay.txt" 2>&1 || { rm -rf "$_nv_dir"; exit 1; }
grep -q 'does not prove' "$_nv_dir/replay.txt" || { rm -rf "$_nv_dir"; exit 1; }
# RE-DERIVE IS LOAD-BEARING: a tampered packet is refused by BOTH replay and report.
cp "$_nv_dir/novelty.json" "$_nv_dir/tampered.json"
printf '\n{tampered}' >> "$_nv_dir/tampered.json"
if ./target/debug/cognitive-demo novelty-replay --input-dir "$_nv_rel/corpus" --frame "$_nv_rel/frame.txt" --packet "$_nv_dir/tampered.json" >/dev/null 2>&1; then rm -rf "$_nv_dir"; exit 1; fi
if ./target/debug/cognitive-demo novelty-report --input-dir "$_nv_rel/corpus" --frame "$_nv_rel/frame.txt" --packet "$_nv_dir/tampered.json" >/dev/null 2>&1; then rm -rf "$_nv_dir"; exit 1; fi
# A corpus trace with its receipt hash stripped is NOT the verified trace -> novelty-packet refuses to ground on it.
grep -v structure_hash "$_nv_dir/trace.json" > "$_nv_dir/trace_nohash.json"
if ./target/debug/cognitive-demo novelty-packet --input-dir "$_nv_rel/corpus" --corpus-trace "$_nv_dir/trace_nohash.json" --frame "$_nv_rel/frame.txt" >/dev/null 2>&1; then rm -rf "$_nv_dir"; exit 1; fi
# An empty frame (no candidate assumption) fails closed.
printf '\n   \n' > "$_nv_dir/empty_frame.txt"
if ./target/debug/cognitive-demo novelty-packet --input-dir "$_nv_rel/corpus" --corpus-trace "$_nv_dir/trace.json" --frame "$_nv_rel/empty_frame.txt" >/dev/null 2>&1; then rm -rf "$_nv_dir"; exit 1; fi
# END-TO-END input safety: an absolute corpus dir, and a frame that escapes the working directory via symlink,
# are each refused through the binary (not only via the pure path checks in the lib).
if ./target/debug/cognitive-demo novelty-packet --input-dir "/etc" --corpus-trace "$_nv_dir/trace.json" --frame "$_nv_rel/frame.txt" >/dev/null 2>&1; then rm -rf "$_nv_dir"; exit 1; fi
ln -s /etc/hostname "$_nv_dir/escape_frame.txt" 2>/dev/null
if ./target/debug/cognitive-demo novelty-packet --input-dir "$_nv_rel/corpus" --corpus-trace "$_nv_dir/trace.json" --frame "$_nv_rel/escape_frame.txt" >/dev/null 2>&1; then rm -rf "$_nv_dir"; exit 1; fi
rm -rf "$_nv_dir"
# ---------------------------------------------------------------------------------------------------
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

# ── DREAM-0 — Seeded Deterministic Distortion Engine (crates/dream-engine) ───────────────────────────────────
# A STANDALONE dream track that DISTORTS verified corpus material into terminal, inert DreamPackets. It carries a
# crate-PRIVATE dream authority (never the public hypothesis-layer Authority), has NO export path and NO
# hypothesis-layer dependency, rebuilds grounding on reading-substrate only, and is deterministic (FNV-1a ids,
# splitmix64 selection, no entropy/clock/floats). Nothing executes, promotes, trains, or becomes evidence.
cargo test --offline --quiet --manifest-path crates/dream-engine/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/dream-engine/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/dream-engine/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# Unit-test REALITY pin: exactly the 20 DREAM-0 tests pass, zero ignored (gutting/disabling one is caught).
_dream_unit="$(cargo test --offline --lib --manifest-path crates/dream-engine/Cargo.toml 2>/dev/null)"
test "$(printf '%s\n' "$_dream_unit" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')" -eq 20
test "$(printf '%s\n' "$_dream_unit" | grep -oE '[0-9]+ ignored' | grep -oE '[0-9]+')" -eq 0
# Structural QUARANTINE (no export path): the production tree holds NO hypothesis-layer, no engine crate, no
# codec, and no integration crate — so "no dream output enters the hypothesis layer in DREAM-0" is a
# gate-enforced invariant, not a promise. The root crate is present (fails closed if cargo tree cannot run).
test "$(cargo tree --offline --manifest-path crates/dream-engine/Cargo.toml --edges normal 2>/dev/null | grep -cE 'hypothesis-layer|vibe-|cognitive-demo|reading-codec')" -eq 0
test "$(cargo tree --offline --manifest-path crates/dream-engine/Cargo.toml --edges normal 2>/dev/null | grep -c 'dream-engine v')" -eq 1
# No model is trained or loaded: the manifest pulls no ML/inference/training framework.
test "$(grep -ciE 'torch|tensorflow|candle|onnx|tract|\bburn\b|llama|inference' crates/dream-engine/Cargo.toml)" -eq 0
# Determinism / no side effects: no clock, entropy, network, DefaultHasher, or floats anywhere in src/.
test "$(grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|tokio|\.await|reqwest|DefaultHasher' crates/dream-engine/src | wc -l)" -eq 0
test "$(grep -rE '\bf32\b|\bf64\b' crates/dream-engine/src | wc -l)" -eq 0
# DreamOnly is crate-PRIVATE vocabulary: the token appears ONLY under crates/dream-engine, never elsewhere.
test "$(grep -rl 'DreamOnly' crates --include=*.rs | grep -vc '^crates/dream-engine/')" -eq 0
# The frozen hypothesis-layer Authority is UNCHANGED: exactly one Authority enum, no DreamOnly leaked into it.
test "$(grep -c 'pub enum Authority' crates/hypothesis-layer/src/lib.rs)" -eq 1
test "$(grep -cE 'DreamOnly' crates/hypothesis-layer/src/lib.rs)" -eq 0
# The nine-line DREAM-0 boundary is recorded verbatim in the source.
grep -q 'The dream engine distorts.' crates/dream-engine/src/lib.rs
grep -q 'No dream output enters the hypothesis layer in DREAM-0.' crates/dream-engine/src/lib.rs
grep -q 'Dream packets are terminal and inert.' crates/dream-engine/src/lib.rs
grep -q 'Nothing becomes evidence.' crates/dream-engine/src/lib.rs
grep -q 'Nothing promotes.' crates/dream-engine/src/lib.rs
grep -q 'Nothing trains.' crates/dream-engine/src/lib.rs
# The canonical six forbidden uses + the named anti-degeneracy / terminal regression scenarios exist by name.
grep -q 'pub const DREAM_FORBIDDEN_USES' crates/dream-engine/src/lib.rs
grep -q 'fn dream_input_hash_changes_when_side_document_changes' crates/dream-engine/src/lib.rs
grep -q 'fn dream_refuses_degenerate_single_span_reformat' crates/dream-engine/src/lib.rs
grep -q 'fn dream_links_two_distinct_document_ids_into_one_frame' crates/dream-engine/src/lib.rs
grep -q 'fn dream_broken_assumption_is_operator_output_not_frame_echo' crates/dream-engine/src/lib.rs
grep -q 'fn dream_falsifier_slot_well_formed_by_reference' crates/dream-engine/src/lib.rs
grep -q 'fn dream_replay_byte_identical_two_processes' crates/dream-engine/src/lib.rs
grep -q 'fn dream_packet_tamper_refused' crates/dream-engine/src/lib.rs
grep -q 'fn dream_unsupported_preserved_fact_refused' crates/dream-engine/src/lib.rs
grep -q 'fn dream_packet_is_terminal_no_export' crates/dream-engine/src/lib.rs

# ── DREAM-EXPORT-0 — Dream Export Receipt / Provenance Bridge (crates/cognitive-demo) ─────────────────────────
# A terminal, inert DreamPacket (from the STANDALONE dream-engine, which itself has NO export path) is BRIDGED
# into the EXISTING hypothesis-only proposal path with a DreamExportReceipt that preserves dream-origin
# provenance OUTSIDE the frozen hypothesis-layer Authority. The correct shape is
# DreamPacket -> DreamExportReceipt -> existing HypothesisOnly proposal path; the FORBIDDEN shape is
# DreamPacket -> new Authority::DreamOnly. No new authority is created, exported material stays hypothesis_only,
# dream origin stays auditable, and probe requests never execute. (The behavioural pins live in the INT-0 unit
# count above — exactly the 13 DREAM-EXPORT-0 tests pass — and the name-greps below pin WHICH behaviours.)
# The dependency arrow is demo -> engine: cognitive-demo CONSUMES dream-engine's public terminal packet, while
# dream-engine's own quarantine tree (no hypothesis-layer / no integration crate) is UNCHANGED — asserted above.
grep -q 'dream-engine' crates/cognitive-demo/Cargo.toml
test "$(cargo tree --offline --manifest-path crates/cognitive-demo/Cargo.toml --edges normal 2>/dev/null | grep -c 'dream-engine')" -ge 1
# The bridge feeds the EXISTING path: it records the authority READ OFF the proposed packet (never a fabricated
# or new variant), marks the export as going through the existing gate, and preserves dream origin.
grep -q 'authority_after_export: hypothesis.authority()' crates/cognitive-demo/src/lib.rs
grep -q 'exported_via_existing_hypothesis_gate: true' crates/cognitive-demo/src/lib.rs
grep -q 'dream_origin: true' crates/cognitive-demo/src/lib.rs
grep -q 'propose(spec)' crates/cognitive-demo/src/lib.rs
# FORBIDDEN SHAPE is structurally impossible: cognitive-demo introduces NO new authority enum and NEVER writes
# the dream authority token — the receipt carries the EXISTING Authority::HypothesisOnly; dream_only stays in
# dream-engine (asserted crate-wide in the DREAM-0 block) and never crosses into the integration crate.
test "$(grep -cE 'DreamOnly' crates/cognitive-demo/src/lib.rs)" -eq 0
test "$(grep -cE 'enum (Dream|Export)[A-Za-z]*Authority' crates/cognitive-demo/src/lib.rs)" -eq 0
# The FROZEN hypothesis-layer Authority is UNCHANGED by the bridge: exactly one Authority enum, no DreamOnly.
test "$(grep -c 'pub enum Authority' crates/hypothesis-layer/src/lib.rs)" -eq 1
test "$(grep -cE 'DreamOnly' crates/hypothesis-layer/src/lib.rs)" -eq 0
# The export bridge stays PURE in the library (no fs/clock/entropy/net/floats — already scanned above for all of
# src/) and the receipt/bundle are Serialize-only (the no-Deserialize pin above covers the whole demo lib).
# The eight-line DREAM-EXPORT-0 boundary is recorded verbatim in the source.
grep -q 'Dream export preserves provenance.' crates/cognitive-demo/src/lib.rs
grep -q 'It does not create a new authority.' crates/cognitive-demo/src/lib.rs
grep -q 'Exported dream material remains hypothesis_only.' crates/cognitive-demo/src/lib.rs
grep -q 'Dream origin remains auditable.' crates/cognitive-demo/src/lib.rs
grep -q 'Probe requests do not execute.' crates/cognitive-demo/src/lib.rs
grep -q 'Nothing becomes evidence.' crates/cognitive-demo/src/lib.rs
grep -q 'Nothing promotes.' crates/cognitive-demo/src/lib.rs
grep -q 'Nothing trains.' crates/cognitive-demo/src/lib.rs
# The named DREAM-EXPORT-0 tests exist (a gutted/deleted test drops the unit count pinned above; these greps
# additionally pin WHICH behaviours are covered — provenance, distinguishability, tamper-refusal, no-execution).
grep -q 'fn dream_export_builds_from_verified_corpus' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_receipt_preserves_dream_provenance' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_receipt_records_dream_origin_true' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_authority_after_export_is_hypothesis_only' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_uses_existing_hypothesis_gate' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_carries_no_dream_authority' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_probe_requests_do_not_execute' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_refuses_tampered_dream_packet' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_replay_byte_identical' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_tampered_bundle_refused' crates/cognitive-demo/src/lib.rs
grep -q 'fn plain_and_dream_hypothesis_distinguishable' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_report_shows_provenance' crates/cognitive-demo/src/lib.rs
grep -q 'fn dream_export_refuses_unverifiable_corpus' crates/cognitive-demo/src/lib.rs

# ── DREAM-EXPORT-2 — Dream Export Scenario Matrix / Provenance Integrity (crates/cognitive-demo) ──────────────
# A deterministic scenario matrix over the EXISTING dream-export bridge: ONE clean export that VERIFIES, plus SIX
# tamper scenarios that are each REFUSED (a tampered source dream packet, a tampered receipt, a forged
# dream_origin=false, a mutated dream_input_hash, a mutated dream_packet_id, and a forged authority_after_export
# that injects the dream engine's private serialized token). Each row records the OBSERVED outcome, the matrix
# records the preserved dream provenance fields, that the exported material stays hypothesis_only and is
# DISTINGUISHABLE from a plain hypothesis, that probe requests never execute, and the no-evidence / no-promotion /
# no-training boundary cells. Pure + re-derived-and-byte-compared on verify; it creates NO authority. The
# behavioural pins live in the INT-0 unit count above (167 = +15 DREAM-EXPORT-2 tests); the name-greps + binary
# smoke below pin WHICH behaviours are covered. The dream's PascalCase private authority token NEVER appears in the
# demo source (gated crate-wide above) — the matrix names it only by its lowercase serialized form `dream_only`.
# Doctrine: Dream export scenarios vary the export artifact. They do not vary the authority. Dream provenance
# remains auditable. Exported material remains HypothesisOnly. dream_only remains private to dream-engine. Probe
# requests do not execute. Nothing becomes evidence. Nothing promotes. Nothing trains.
# Surface: the matrix API + the four commands exist (lib + shell).
grep -q 'enum DreamExportScenario' crates/cognitive-demo/src/lib.rs
grep -q 'fn run_dream_export_scenario' crates/cognitive-demo/src/lib.rs
grep -q 'fn canonical_dream_export_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn dream_export_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn verify_dream_export_matrix' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_dream_export_matrix_report' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn run_dream_export_matrix_verify' crates/cognitive-demo/src/lib.rs
grep -q 'pub fn list_dream_export_scenarios' crates/cognitive-demo/src/lib.rs
grep -q '"dream-export-scenarios"' crates/cognitive-demo/src/main.rs
grep -q '"dream-export-matrix"' crates/cognitive-demo/src/main.rs
grep -q '"dream-export-matrix-report"' crates/cognitive-demo/src/main.rs
grep -q '"dream-export-matrix-verify"' crates/cognitive-demo/src/main.rs
# The matrix verifies by RE-DERIVE byte-compare (no Deserialize) and refuses tamper via DreamExportMismatch.
grep -q 'provided == dream_export_matrix' crates/cognitive-demo/src/lib.rs
# The seven scenarios exist by name (clean + six tampers).
for _dxs in clean-export tampered-dream-packet tampered-receipt forged-dream-origin-false mutated-dream-input-hash mutated-dream-packet-id forged-authority-after-export; do
  if ! grep -qF "\"$_dxs\"" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# FORBIDDEN SHAPE stays impossible: no new authority enum; the PascalCase dream token never appears in the demo
# (the crate-wide DreamOnly==0 pins above cover it); the matrix names the private token only by its lowercase form.
test "$(grep -cE 'enum (Dream|Export)[A-Za-z]*Authority' crates/cognitive-demo/src/lib.rs)" -eq 0
# The source-safe nine-line DREAM-EXPORT-2 matrix boundary is recorded verbatim in the source.
for _dxmb in 'Dream export scenarios vary the export artifact.' 'They do not vary the authority.' 'Dream provenance remains auditable.' 'Exported material remains HypothesisOnly.' 'dream_only remains private to dream-engine.' 'Probe requests do not execute.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_dxmb" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# The 15 DREAM-EXPORT-2 tests exist by name (a gutted/deleted test also drops the unit count pinned at 167 above).
for _dxt in \
  dream_export_matrix_lists_all_scenarios \
  dream_export_matrix_clean_verifies \
  dream_export_matrix_all_tampers_refused \
  dream_export_matrix_all_match_expected \
  dream_export_matrix_records_dream_provenance \
  dream_export_matrix_authority_remains_hypothesis_only \
  dream_export_matrix_distinguishes_plain_from_dream \
  dream_export_matrix_probe_requests_do_not_execute \
  dream_export_matrix_records_no_evidence_promotion_training \
  dream_export_matrix_replay_byte_identical \
  dream_export_matrix_verify_rejects_tampered_matrix \
  dream_export_matrix_authority_forgery_injects_dream_token_and_is_refused \
  dream_export_matrix_report_shows_provenance_and_outcomes \
  dream_export_matrix_tampers_actually_mutate \
  dream_export_matrix_does_not_change_training_gate; do
  if ! grep -q "fn $_dxt(" crates/cognitive-demo/src/lib.rs; then exit 1; fi
done
# BINARY SMOKE (proves, not asserts): run the matrix CLI against a REAL local corpus + frame under the gitignored
# target/ (relative paths), prove the clean export verifies, the coverage cells hold, the canonical matrix carries
# NO PascalCase dream token, and a tampered matrix (a refused outcome flipped to verifies) is REFUSED end-to-end.
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo
_dxm_dir="$(mktemp -d "$PWD/target/.dxm_gate.XXXXXX")"
_dxm_rel="target/$(basename "$_dxm_dir")"
mkdir -p "$_dxm_dir/corpus"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$_dxm_dir/corpus/a-east.txt"
printf 'The west tunnel remains closed. Crews continue repairs.' > "$_dxm_dir/corpus/b-west.txt"
printf 'The east bridge stays closed indefinitely.\nTraffic never recovers after a closure.\n' > "$_dxm_dir/frame.txt"
./target/debug/cognitive-demo dream-export-matrix --input-dir "$_dxm_rel/corpus" --frame "$_dxm_rel/frame.txt" --out "$_dxm_dir/matrix.json" >/dev/null 2>&1 || { rm -rf "$_dxm_dir"; exit 1; }
./target/debug/cognitive-demo dream-export-matrix-verify --input-dir "$_dxm_rel/corpus" --frame "$_dxm_rel/frame.txt" --matrix "$_dxm_dir/matrix.json" >/dev/null 2>&1 || { rm -rf "$_dxm_dir"; exit 1; }
for _dxc in '"clean_verifies": true' '"all_tampers_refused": true' '"all_match_expected": true' '"exported_material_is_hypothesis_only": true' '"dream_distinguishable_from_plain": true' '"probe_requests_execute": false' '"no_execution": true' '"no_evidence": true' '"no_promotion": true' '"no_training": true' '"authority_after_export": "hypothesis_only"' '"dream_origin": true'; do
  if ! grep -qF "$_dxc" "$_dxm_dir/matrix.json"; then rm -rf "$_dxm_dir"; exit 1; fi
done
if grep -qF 'DreamOnly' "$_dxm_dir/matrix.json"; then rm -rf "$_dxm_dir"; exit 1; fi
sed 's/"outcome": "refused"/"outcome": "verifies"/' "$_dxm_dir/matrix.json" > "$_dxm_dir/tampered.json"
if cmp -s "$_dxm_dir/matrix.json" "$_dxm_dir/tampered.json"; then rm -rf "$_dxm_dir"; exit 1; fi
if ./target/debug/cognitive-demo dream-export-matrix-verify --input-dir "$_dxm_rel/corpus" --frame "$_dxm_rel/frame.txt" --matrix "$_dxm_dir/tampered.json" >/dev/null 2>&1; then rm -rf "$_dxm_dir"; exit 1; fi
if ./target/debug/cognitive-demo dream-export-matrix-report --input-dir "$_dxm_rel/corpus" --frame "$_dxm_rel/frame.txt" --matrix "$_dxm_dir/tampered.json" >/dev/null 2>&1; then rm -rf "$_dxm_dir"; exit 1; fi
rm -rf "$_dxm_dir"

# ---------------------------------------------------------------------------------------------------
# DREAM-EXPORT-3 — dream export milestone freeze. The DREAM-0 -> DREAM-EXPORT-2 dream-provenance arc (the terminal
# seeded distortion engine, the provenance bridge into the hypothesis-only path, the operator guard, and the
# scenario matrix) is frozen as dream-export-v0.1. The milestone record (DREAM_EXPORT_MILESTONE.md) pins the
# DREAM-0..DREAM-EXPORT-2 commit lineage, the frozen bases (corpus-flow-v0.1 + document-flow-v0.1 + the deeper
# milestone tags + commits), the demonstrated capability, the preserve-provenance-not-authority boundary, the
# private DreamOnly confinement, the single-variant hypothesis-layer Authority, the auditable dream_origin, the
# P12 training verdict, and the honest residuals, and is locked here so the freeze cannot silently drift. The
# pinned commit hashes are auditable against `git log`; this lock stays git-free and does NOT require the tag to
# exist (the tag is created only after a clean tree + green gate). Documentation freeze only — no code crate
# change, no model, no training; the milestone records training_not_justified. Doctrine: Dream export preserves
# provenance. It does not create a new authority. Exported dream material remains HypothesisOnly. Dream origin
# remains auditable. DreamOnly remains private to dream-engine. Probe requests do not execute. Nothing becomes
# evidence. Nothing promotes. Nothing trains.
# ---------------------------------------------------------------------------------------------------
test -f DREAM_EXPORT_MILESTONE.md
grep -q 'FROZEN' DREAM_EXPORT_MILESTONE.md
grep -q 'dream-export-v0.1' DREAM_EXPORT_MILESTONE.md
grep -q 'DREAM-0' DREAM_EXPORT_MILESTONE.md
grep -q 'DREAM-EXPORT-0' DREAM_EXPORT_MILESTONE.md
grep -q 'DREAM-EXPORT-1' DREAM_EXPORT_MILESTONE.md
grep -q 'DREAM-EXPORT-2' DREAM_EXPORT_MILESTONE.md
grep -q 'training_not_justified' DREAM_EXPORT_MILESTONE.md
grep -q 'training_justified=false' DREAM_EXPORT_MILESTONE.md
# Full DREAM-0..DREAM-EXPORT-2 commit lineage is pinned (cross-checkable against git log).
grep -qF '290abee' DREAM_EXPORT_MILESTONE.md
grep -qF 'd3af869' DREAM_EXPORT_MILESTONE.md
grep -qF '076277d' DREAM_EXPORT_MILESTONE.md
grep -qF 'ac03327' DREAM_EXPORT_MILESTONE.md
# corpus-flow-v0.1 + document-flow-v0.1 are named as the frozen bases, and the deeper frozen milestones are
# referenced (tag + commit).
grep -qF 'corpus-flow-v0.1' DREAM_EXPORT_MILESTONE.md
grep -qF 'document-flow-v0.1' DREAM_EXPORT_MILESTONE.md
for _t in corpus-flow-v0.1 document-flow-v0.1 operator-controls-v0.1 multi-trace-validation-v0.1 integration-demo-v0.1 hypothesis-track-v0.1 reading-track-v0.1 cognitive-os-governance-v0.1; do
  if ! grep -qF "$_t" DREAM_EXPORT_MILESTONE.md; then exit 1; fi
done
for _sha in b8577fe 0cc7399 34b4f47 460be0c 95b586d bb20acf f6fa55a bbd1113; do
  if ! grep -qF "$_sha" DREAM_EXPORT_MILESTONE.md; then exit 1; fi
done
# The four frozen dream capabilities are referenced by name (terminal packet engine, provenance bridge receipt,
# operator guard manual, scenario matrix).
grep -qF 'dream-engine' DREAM_EXPORT_MILESTONE.md
grep -qF 'DreamExportReceipt' DREAM_EXPORT_MILESTONE.md
grep -qF 'OPERATOR_MANUAL.md' DREAM_EXPORT_MILESTONE.md
grep -qF 'dream-export-matrix' DREAM_EXPORT_MILESTONE.md
# The dream-export-specific invariants the rubric requires are recorded: private DreamOnly confinement, exported
# material stays hypothesis_only, single-variant hypothesis-layer Authority, auditable dream-origin provenance.
grep -qF 'DreamOnly remains private to dream-engine.' DREAM_EXPORT_MILESTONE.md
grep -qF 'hypothesis_only' DREAM_EXPORT_MILESTONE.md
grep -qF 'single-variant' DREAM_EXPORT_MILESTONE.md
grep -qF 'dream_origin' DREAM_EXPORT_MILESTONE.md
# The nine-line dream-export boundary is recorded verbatim (all nine lines).
for _bl in 'Dream export preserves provenance.' 'It does not create a new authority.' 'Exported dream material remains HypothesisOnly.' 'Dream origin remains auditable.' 'DreamOnly remains private to dream-engine.' 'Probe requests do not execute.' 'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  if ! grep -qF "$_bl" DREAM_EXPORT_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' DREAM_EXPORT_MILESTONE.md; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# DATA-0 — dataset curation manifest / ingestion gate. `crates/data-curator` is a STANDALONE,
# deterministic admissibility gate: it classifies a caller-supplied CandidateManifest into admitted /
# rejected / quarantined records and emits a CurationReceipt BEFORE any ingestion, memory, horizon, or
# training path may use the data. It mints no authority, creates no evidence, promotes nothing, executes
# nothing, ingests nothing, and trains nothing; training eligibility defaults closed and the crate carries
# NO value that permits training. Doctrine: Data curation admits, rejects, or quarantines candidate data.
# It does not create truth. It does not create memory. It does not train. It does not execute. It does not
# promote. Training eligibility remains closed unless a later gate explicitly opens it. The cargo test
# suite runs the admit/reject/quarantine/leakage/determinism/never-eligible/inert battery; the source
# scans below are sabotage-detectable and independent of the tests.
# ---------------------------------------------------------------------------------------------------
cargo test --offline --quiet --manifest-path crates/data-curator/Cargo.toml >/dev/null 2>&1
cargo fmt --manifest-path crates/data-curator/Cargo.toml --check >/dev/null 2>&1
cargo clippy --offline --manifest-path crates/data-curator/Cargo.toml --all-targets -- -D warnings >/dev/null 2>&1
# The curator is pure: its source carries no filesystem / network / process / clock / entropy / async
# token, and never references the hypothesis-layer Authority or the training gate (sabotage-detectable).
test "$(grep -rE 'std::fs|std::net|std::process|std::time|std::thread|thread::sleep|SystemTime|Instant|tokio|async fn|\.await|reqwest|sqlx|rusqlite|use rand|rand::|getrandom|hypothesis_layer|DreamAuthority|reading_train_gate|reading_eval' crates/data-curator/src/ | wc -l)" -eq 0
# Dependency boundary: the normal dep tree contains NO workspace crate (no vibe-*, reading-*,
# hypothesis-layer, cognitive-demo, dream-engine) — so it cannot reach the Authority type, the training
# gate, or any engine/memory crate — while the crate root is present (fails closed if cargo tree cannot run).
test "$(cargo tree --offline --manifest-path crates/data-curator/Cargo.toml --edges normal 2>/dev/null | grep -cE 'vibe-|reading-|hypothesis-layer|cognitive-demo|dream-engine')" -eq 0
test "$(cargo tree --offline --manifest-path crates/data-curator/Cargo.toml --edges normal 2>/dev/null | grep -c 'data-curator')" -eq 1
# Training eligibility can NEVER be true: the single source of truth is pinned to false, and no source
# asserts an eligible / justified-true path (sabotage-detectable; the never-eligible test also runs above).
grep -qF 'const TRAINING_PERMITTED: bool = false;' crates/data-curator/src/types.rs
test "$(grep -rE 'TRAINING_PERMITTED: bool = true|training_justified[[:space:]]*[=:][[:space:]]*true' crates/data-curator/src/ | wc -l)" -eq 0
# Positive structure pins: the curation entrypoint, the receipt, the default-closed eligibility, the
# inert authority-boundary, and quarantine-not-delete for BOTH quarantine reasons are present; unsafe is
# forbidden and the only input is a CandidateManifest (no implicit filesystem-blob ingestion).
grep -qF 'pub fn curate(' crates/data-curator/src/curate.rs
grep -qF 'pub struct CurationReceipt' crates/data-curator/src/types.rs
grep -qF '#[default]' crates/data-curator/src/types.rs
grep -qF 'BoundaryChecks::inert' crates/data-curator/src/curate.rs
grep -qF 'QuarantineReason::PromptInjection' crates/data-curator/src/curate.rs
grep -qF 'QuarantineReason::SplitLeakage' crates/data-curator/src/curate.rs
grep -qF '#![forbid(unsafe_code)]' crates/data-curator/src/lib.rs

# ---------------------------------------------------------------------------------------------------
# DATA-1 — data curation operator guard. The operator manual (OPERATOR_MANUAL.md §15) documents the
# data-curation operator path: it states the curator ADMITS / REJECTS / QUARANTINES candidate data, that a
# prompt-injection marker is QUARANTINED (not deleted) and train/holdout leakage is quarantined, that
# duplicate ids and missing provenance are REJECTED, and that training eligibility remains structurally closed
# (no code path returns training-eligible=true). The operator smoke (scripts/operator_smoke.sh) runs the REAL
# curate() over candidate manifests via its named tests (clean->admitted, missing-provenance->rejected,
# duplicate->rejected, prompt-injection->quarantined, train/holdout-leakage->quarantined,
# eligibility->never-eligible), each with --exact so a dropped outcome is caught as vacuous. A documentation +
# drift-guard sprint — NO code crate change (the DATA-0 gate above is unchanged; data-curator src is
# byte-identical). The smoke is already RUN by the OPS-1 lock above (a curation drift makes it fail closed and
# aborts the gate); the pins below stop the curation coverage from being silently dropped from the smoke or the
# manual. Doctrine: The curation operator path classifies candidate data. It admits, rejects, or quarantines. It
# does not create truth. It does not create memory. It does not train. It does not execute. It does not promote.
# Training eligibility remains closed.
# ---------------------------------------------------------------------------------------------------
# The manual documents how to exercise the real curator (the cargo test command over data-curator).
grep -qF 'cargo test --offline --manifest-path crates/data-curator/Cargo.toml' OPERATOR_MANUAL.md
# It states the admit/reject/quarantine doctrine, quarantine-not-delete, dup/missing-provenance rejection,
# train/holdout leakage quarantine, and that eligibility can never be true.
for _dcd in 'admits, rejects, or quarantines' 'quarantined, not deleted' 'missing provenance' 'duplicate id' \
            'train/holdout leakage' 'no code path can return training-eligible'; do
  if ! grep -qF "$_dcd" OPERATOR_MANUAL.md; then exit 1; fi
done
# It records the DATA-1 eight-line curation-operator-path boundary verbatim.
for _dcb in 'The curation operator path classifies candidate data.' 'It admits, rejects, or quarantines.' \
            'It does not create truth.' 'It does not create memory.' 'It does not train.' \
            'It does not execute.' 'It does not promote.' 'Training eligibility remains closed.'; do
  if ! grep -qF "$_dcb" OPERATOR_MANUAL.md; then exit 1; fi
done
# The smoke creates the curation temp dir + manifest illustration under target/ and removes it on exit (the
# OPS-1 lock above already pins the trap-line prefix, which now also carries "$curatework").
grep -qF 'target/.curate_smoke' scripts/operator_smoke.sh
grep -qF 'candidate_manifest.txt' scripts/operator_smoke.sh
grep -qF '"$curatework"' scripts/operator_smoke.sh
# The smoke runs the REAL curator over EACH required outcome via --exact named tests (cannot silently drop one).
grep -qF -- '--exact "tests::' scripts/operator_smoke.sh
for _dct in clean_document_span_is_admitted_but_only_candidate missing_provenance_is_rejected \
            duplicate_id_is_rejected_and_recorded_as_contamination \
            prompt_injection_is_quarantined_not_deleted_or_admitted \
            train_holdout_leakage_is_detected_and_quarantined training_eligibility_is_never_eligible; do
  if ! grep -qF "$_dct" scripts/operator_smoke.sh; then exit 1; fi
done
# The named-test runs are NON-VACUOUS: each asserts exactly one test ran, and a dropped outcome fails closed.
grep -qF '1 passed' scripts/operator_smoke.sh
grep -qF 'curation outcome did not run (vacuous)' scripts/operator_smoke.sh
# The smoke makes NO false training claim (re-asserted for the curation additions).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' scripts/operator_smoke.sh; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# DATA-2 — curation scenario matrix. crates/data-curator/src/matrix.rs adds a FIXED, named set of 12
# candidate-data scenarios; each constructs a real CandidateManifest and runs the REAL curate() over it,
# recording the OBSERVED CurationReceipt disposition (admitted/rejected/quarantined + reason + eligibility +
# per-scenario hashes). The matrix only OBSERVES — it creates no truth/memory/authority, executes nothing,
# promotes nothing, and opens no training: every cell's opens_training is is_eligible() == false, and
# training_never_opens holds. The cells derive Serialize but NOT Deserialize and are PartialEq, so the matrix
# is re-derived and compared (the lib.rs tests run curation_matrix() twice and assert equality + per-scenario
# hash determinism). The cargo test/clippy gate above already RUNS the matrix battery (matrix_* tests) and
# compiles matrix.rs; the pins below stop the scenario set, the outcome cells, or the no-training invariant
# from being silently dropped. A capability sprint that ADDS the matrix module + tests — no other crate
# changes; the DATA-0/DATA-1 gates above are unchanged. Doctrine: The curation scenario matrix observes curation
# outcomes. It does not create truth. It does not create memory. It does not train. It does not execute. It does
# not promote. Training eligibility remains closed in every scenario.
# ---------------------------------------------------------------------------------------------------
_M2=crates/data-curator/src/matrix.rs
# The matrix exists, runs the REAL curator (not hard-coded), and pins a fixed scenario count of 12.
grep -qF 'pub fn curation_matrix(' "$_M2"
grep -qF 'let receipt = curate(manifest);' "$_M2"
grep -qF 'pub const SCENARIO_COUNT: usize = 12;' "$_M2"
# The matrix is observed-not-trusted: Serialize but NEVER derived Deserialize (re-derived and compared).
grep -qF 'Serialize' "$_M2"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_M2")" -eq 0
# Every required scenario is present (the matrix cannot silently drop a cell).
for _s2 in clean_document_admitted missing_provenance_rejected duplicate_id_rejected empty_content_rejected \
           unsupported_artifact_rejected prompt_injection_quarantined split_leakage_quarantined \
           ungrounded_durable_rejected trace_without_replay_rejected valid_split_admitted \
           invalid_split_rejected training_eligibility_never_opens; do
  if ! grep -qF "\"$_s2\"" "$_M2"; then exit 1; fi
done
# The observed outcome cells: every reject/quarantine reason label the classifier can emit is present.
for _r2 in missing_provenance duplicate_id empty_content unsupported_artifact missing_grounding \
           missing_replay_receipt invalid_split prompt_injection split_leakage; do
  if ! grep -qF "\"$_r2\"" "$_M2"; then exit 1; fi
done
# No scenario opens training: opens_training is is_eligible() (pinned false by the DATA-0 gate), and the
# matrix-level invariant is computed, not asserted true.
grep -qF 'opens_training: receipt.training_eligibility.is_eligible()' "$_M2"
grep -qF 'training_never_opens' "$_M2"
# The lib.rs matrix tests assert the count, the observed cells, the no-training invariant, and determinism
# (so the matrix coverage cannot be silently removed from the test battery the gate runs above).
for _t2 in 'fn matrix_has_the_fixed_named_scenarios' 'fn matrix_cells_record_the_observed_curation_outcomes' \
           'fn matrix_opens_no_training_in_any_scenario' 'fn matrix_is_deterministic_and_re_derivable'; do
  if ! grep -qF "$_t2" crates/data-curator/src/lib.rs; then exit 1; fi
done
# The DATA-2 seven-line scenario-matrix boundary is recorded verbatim in matrix.rs.
for _b2 in 'The curation scenario matrix observes curation outcomes.' 'It does not create truth.' \
           'It does not create memory.' 'It does not train.' 'It does not execute.' 'It does not promote.' \
           'Training eligibility remains closed in every scenario.'; do
  if ! grep -qF "$_b2" "$_M2"; then exit 1; fi
done
# DATA-2 makes NO false training claim in the matrix source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_M2"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# DATA-3 — data curation milestone freeze. The DATA-0 -> DATA-2 dataset-curation arc (the standalone
# ingestion gate, the operator guard, and the scenario matrix) is frozen as data-curation-v0.1. The
# milestone record (DATA_CURATION_MILESTONE.md) pins the DATA-0..DATA-2 commit lineage, the prior frozen
# substrate base (dream-export-v0.1) and the deeper frozen milestone tags + commits, the demonstrated
# capability, the classification-not-evidence boundary, the quarantine-not-deletion invariant, the
# structurally-closed training eligibility (Closed/CandidateOnly only, no Eligible/TrainingEligible variant,
# TRAINING_PERMITTED=false), the P12 verdict, and the honest residuals, and is locked here so the freeze
# cannot silently drift. The pinned commit hashes are auditable against `git log`; this lock stays git-free
# and does NOT require the tag to exist (the tag is created only after a clean tree + green gate).
# Documentation freeze only — no code crate change, no model, no training; the milestone records
# training_not_justified. Doctrine: Data curation classifies candidate data. It admits, rejects, or
# quarantines. It does not create truth. It does not create memory. It does not train. It does not execute.
# It does not promote. Training eligibility remains closed.
# ---------------------------------------------------------------------------------------------------
test -f DATA_CURATION_MILESTONE.md
grep -q 'FROZEN' DATA_CURATION_MILESTONE.md
grep -qF 'data-curation-v0.1' DATA_CURATION_MILESTONE.md
grep -qF 'DATA-0' DATA_CURATION_MILESTONE.md
grep -qF 'DATA-1' DATA_CURATION_MILESTONE.md
grep -qF 'DATA-2' DATA_CURATION_MILESTONE.md
grep -qF 'training_not_justified' DATA_CURATION_MILESTONE.md
grep -qF 'training_justified=false' DATA_CURATION_MILESTONE.md
# Full DATA-0..DATA-2 commit lineage is pinned (cross-checkable against git log).
grep -qF '2a3e6aa' DATA_CURATION_MILESTONE.md
grep -qF 'a0bfd04' DATA_CURATION_MILESTONE.md
grep -qF 'c84233a' DATA_CURATION_MILESTONE.md
# dream-export-v0.1 is named as the prior frozen substrate milestone (tag + commit), and the deeper frozen
# milestones are referenced (tag + commit).
grep -qF 'dream-export-v0.1' DATA_CURATION_MILESTONE.md
for _t in dream-export-v0.1 corpus-flow-v0.1 document-flow-v0.1 operator-controls-v0.1 multi-trace-validation-v0.1 integration-demo-v0.1 hypothesis-track-v0.1 reading-track-v0.1 cognitive-os-governance-v0.1; do
  if ! grep -qF "$_t" DATA_CURATION_MILESTONE.md; then exit 1; fi
done
for _sha in 5238fe8 b8577fe 0cc7399 34b4f47 460be0c 95b586d bb20acf f6fa55a bbd1113; do
  if ! grep -qF "$_sha" DATA_CURATION_MILESTONE.md; then exit 1; fi
done
# The three frozen curation capabilities are referenced by name (the ingestion gate, the operator guard
# manual, the scenario matrix).
grep -qF 'data-curator' DATA_CURATION_MILESTONE.md
grep -qF 'CurationReceipt' DATA_CURATION_MILESTONE.md
grep -qF 'OPERATOR_MANUAL.md' DATA_CURATION_MILESTONE.md
grep -qF 'curation_matrix' DATA_CURATION_MILESTONE.md
# All 12 DATA-2 scenario names are listed in the freeze record.
for _s3 in clean_document_admitted missing_provenance_rejected duplicate_id_rejected empty_content_rejected \
           unsupported_artifact_rejected prompt_injection_quarantined split_leakage_quarantined \
           ungrounded_durable_rejected trace_without_replay_rejected valid_split_admitted \
           invalid_split_rejected training_eligibility_never_opens; do
  if ! grep -qF "$_s3" DATA_CURATION_MILESTONE.md; then exit 1; fi
done
# The curation-specific invariants the rubric requires are recorded: classification-not-evidence;
# quarantine holds (not deletion); training eligibility cannot open (Closed/CandidateOnly only,
# is_eligible() == false, TRAINING_PERMITTED pinned false, no Eligible/TrainingEligible variant).
grep -qF 'classification, not evidence' DATA_CURATION_MILESTONE.md
grep -qF 'quarantined, not deleted' DATA_CURATION_MILESTONE.md
grep -qF 'it does not delete' DATA_CURATION_MILESTONE.md
grep -qF 'Closed' DATA_CURATION_MILESTONE.md
grep -qF 'CandidateOnly' DATA_CURATION_MILESTONE.md
grep -qF 'is_eligible() == false' DATA_CURATION_MILESTONE.md
grep -qF 'TRAINING_PERMITTED' DATA_CURATION_MILESTONE.md
grep -qF 'or `TrainingEligible` variant' DATA_CURATION_MILESTONE.md
# The eight-line curation boundary is recorded verbatim (all eight lines).
for _bl in 'Data curation classifies candidate data.' 'It admits, rejects, or quarantines.' 'It does not create truth.' 'It does not create memory.' 'It does not train.' 'It does not execute.' 'It does not promote.' 'Training eligibility remains closed.'; do
  if ! grep -qF "$_bl" DATA_CURATION_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' DATA_CURATION_MILESTONE.md; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# HORIZON-0 — staged interaction harness. crates/cognitive-demo/src/horizon.rs composes the EXISTING
# verified-read, DATA-0 curation, dream-packet, and dream-export flows into bounded horizons H0..H5 and
# records a HorizonTrace per level. It is a HARNESS, not intelligence: every turn is a REAL call into a
# frozen flow and each step RECORDS that flow's receipt (input/output hashes, authority state, curation
# status where candidate data is used, replay status where a trace-derived artifact is re-derived). The
# invariants are COMPUTED from the real receipts: longer horizons cannot skip curation / grounding /
# replay, cannot promote hypothesis/dream material to evidence, and cannot open training — the train gate
# is decided before AND after every horizon and proven unmoved. The HorizonTrace is Serialize but NOT
# Deserialize (re-derived and byte-compared, never trusted from bytes). The unit-count pin above already
# RUNS the 23 HORIZON-0 tests and the recursive purity/float/process scans already cover horizon.rs; the
# pins below stop the harness from silently faking a flow, dropping an invariant, or exposing the trace's
# fields. Doctrine: The horizon harness measures bounded interaction depth. It does not train. It does not
# execute external actions. It does not create truth. It does not create memory. It does not promote
# hypotheses. It does not grant new authority. Longer horizons cannot bypass earlier gates. Training
# eligibility remains closed.
# ---------------------------------------------------------------------------------------------------
_HZ=crates/cognitive-demo/src/horizon.rs
test -f "$_HZ"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod horizon;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use horizon::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn run_horizon(' "$_HZ"
grep -qF 'pub fn horizon_matrix(' "$_HZ"
grep -qF 'pub fn verify_horizon_json(' "$_HZ"
grep -qF 'pub fn verify_horizon_matrix_json(' "$_HZ"
# The harness DRIVES the real frozen flows — it cannot fabricate a horizon from a hand-written table.
# (A faked flow would drop one of these real calls; the source scan is independent of the cargo tests.)
grep -qF 'curate(' "$_HZ"
grep -qF 'dream_engine::dream_packet(' "$_HZ"
grep -qF 'produce_run(' "$_HZ"
grep -qF 'verify_file(' "$_HZ"
grep -qF 'run_dream_export(' "$_HZ"
grep -qF 'dream_export_matrix(' "$_HZ"
# Training is OBSERVED before AND after every horizon (decided on empty inputs, proven unmoved).
grep -qF 'decide(&[], &[])' "$_HZ"
# The six gate invariants are recorded as fields computed from the real receipts.
for _hz_inv in curation_never_skipped grounding_never_skipped replay_never_skipped no_promotion_to_evidence training_never_opens forbidden_escalation_refused; do
  if ! grep -qF "$_hz_inv" "$_HZ"; then exit 1; fi
done
# The six levels H0..H5 are all defined.
for _hz_lvl in 'HorizonLevel::H0' 'HorizonLevel::H1' 'HorizonLevel::H2' 'HorizonLevel::H3' 'HorizonLevel::H4' 'HorizonLevel::H5'; do
  if ! grep -qF "$_hz_lvl" "$_HZ"; then exit 1; fi
done
# The HorizonTrace is re-derived, never trusted: Serialize but NOT Deserialize, and its fields are PRIVATE
# (the record is inert — read through accessors, never reconstructed off the wire with public fields).
test "$(grep -cE 'derive\([^)]*Deserialize' "$_HZ")" -eq 0
test "$(awk '/pub struct HorizonTrace \{/,/^\}/' "$_HZ" | grep -cE '^[[:space:]]+pub ')" -eq 0
test "$(awk '/pub struct HorizonStep \{/,/^\}/' "$_HZ" | grep -cE '^[[:space:]]+pub ')" -eq 0
# data-curator is a dependency of cognitive-demo (the one-way demo -> curator arrow H1/H2/H5 need); the
# curator's own isolation is asserted separately above (its tree has no workspace edge), so this is safe.
grep -qF 'data-curator' crates/cognitive-demo/Cargo.toml
# The nine-line HORIZON-0 boundary is recorded verbatim (all nine lines, in the const + the module banner).
for _hz_bl in 'The horizon harness measures bounded interaction depth.' 'It does not train.' 'It does not execute external actions.' 'It does not create truth.' 'It does not create memory.' 'It does not promote hypotheses.' 'It does not grant new authority.' 'Longer horizons cannot bypass earlier gates.' 'Training eligibility remains closed.'; do
  if ! grep -qF "$_hz_bl" "$_HZ"; then exit 1; fi
done
# The harness makes NO false training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_HZ"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# HORIZON-1 — horizon operator guard. The operator manual (OPERATOR_MANUAL.md §16) documents the bounded
# horizon operator path: it documents H0..H5 with their max_turns and compositions, states the HorizonTrace is
# re-derived and byte-compared (never trusted from off-wire bytes) and is Serialize-not-Deserialize, and states
# that longer horizons cannot bypass curation / grounding / replay, that dream/hypothesis material cannot become
# evidence, and that training eligibility remains closed. The operator smoke (scripts/operator_smoke.sh) runs the
# REAL horizon harness over each level via its named cognitive-demo tests (H0..H5 + all-gates-held +
# training-never-opens), each with --exact so a dropped outcome is caught as vacuous ("1 passed"). A
# documentation + drift-guard sprint — NO code crate change (the HORIZON-0 harness above is unchanged; the
# cognitive-demo unit-count pin stays 190). The smoke is already RUN by the OPS-1 lock above (a horizon drift
# makes it fail closed and aborts the gate); the pins below stop the horizon coverage from being silently dropped
# from the smoke or the manual. Doctrine: The horizon operator path exercises bounded interaction depth. It does
# not train. It does not execute external actions. It does not create truth. It does not create memory. It does
# not promote hypotheses. It does not grant new authority. Longer horizons cannot bypass earlier gates. Training
# eligibility remains closed.
# ---------------------------------------------------------------------------------------------------
# The manual documents how to exercise the real harness (the cargo test command over the cognitive-demo horizon
# module) and names the bounded horizon harness.
grep -qF 'cargo test --offline --lib --manifest-path crates/cognitive-demo/Cargo.toml horizon::' OPERATOR_MANUAL.md
grep -qF 'bounded horizon harness' OPERATOR_MANUAL.md
# It documents all six levels, the per-level turn bound, the re-derive-not-trust + Serialize-not-Deserialize
# property, the three no-bypass invariants, the no-evidence invariant, and the closed training gate.
for _hzm in '**H0**' '**H1**' '**H2**' '**H3**' '**H4**' '**H5**' 'max_turns' 'never trusted from off-wire bytes' \
            'Deserialize' 'cannot skip curation' 'cannot skip grounding' 'cannot skip replay' \
            'never become evidence' 'training_justified=false'; do
  if ! grep -qF "$_hzm" OPERATOR_MANUAL.md; then exit 1; fi
done
# It records the HORIZON-1 nine-line horizon-operator-path boundary verbatim.
for _hzb in 'The horizon operator path exercises bounded interaction depth.' 'It does not train.' \
            'It does not execute external actions.' 'It does not create truth.' 'It does not create memory.' \
            'It does not promote hypotheses.' 'It does not grant new authority.' \
            'Longer horizons cannot bypass earlier gates.' 'Training eligibility remains closed.'; do
  if ! grep -qF "$_hzb" OPERATOR_MANUAL.md; then exit 1; fi
done
# The smoke runs the REAL harness over each required level via --exact named tests (cannot silently drop one),
# including the all-gates-held invariant proof and the training-never-opens proof.
grep -qF -- '--exact "horizon::tests::' scripts/operator_smoke.sh
for _hzt in horizon_h0_starts_from_verified_read horizon_h1_curates_document_before_reading \
            horizon_h2_curates_corpus_before_multidoc_read horizon_h3_dream_packet_requires_verified_corpus \
            horizon_h4_dream_export_stays_hypothesis_only horizon_h5_combines_curation_and_dream_export \
            horizon_all_gates_held_for_every_level horizon_training_never_opens_before_equals_after; do
  if ! grep -qF "$_hzt" scripts/operator_smoke.sh; then exit 1; fi
done
# The named-test runs are NON-VACUOUS: each asserts exactly one test ran, and a dropped outcome fails closed.
grep -qF 'horizon outcome did not run (vacuous)' scripts/operator_smoke.sh
# The smoke makes NO false training claim (re-asserted for the horizon additions).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' scripts/operator_smoke.sh; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# HORIZON-2 — the bounded-horizon failure matrix. crates/cognitive-demo/src/horizon.rs adds a FIXED, named set
# of 10 failure scenarios. Each constructs a BAD horizon input — an uncurated / ungrounded / replay-less
# candidate, a real horizon trace MUTATED to forge evidence / authority / training, an over-budget step count,
# an unknown level, or a tampered serialized trace — and runs the REAL machinery (the DATA-0 curate, the
# re-derive verify_horizon_json, the max_turns ceiling, from_slug) over it, RECORDING that the bad input was
# REFUSED. It only OBSERVES refusals: it exercises the real verifier (NOT a hard-coded table — a no-op mutation
# cannot pass because the cell requires `mutated != canonical`), never trusts a serialized HorizonTrace as
# authority, and keeps the P12 verdict closed in every cell. The cargo unit-count pin above already RUNS the 16
# HORIZON-2 tests; the source pins below stop a failure cell from being silently dropped or made vacuous.
# Doctrine: The horizon failure matrix mutates bounded traces. It observes refusals. It does not create truth. It
# does not create memory. It does not train. It does not execute external actions. It does not promote
# hypotheses. It does not grant new authority. Training eligibility remains closed.
# ---------------------------------------------------------------------------------------------------
_HZ2=crates/cognitive-demo/src/horizon.rs
grep -qF 'pub fn horizon_failure_matrix(' "$_HZ2"
grep -qF 'pub const FAILURE_SCENARIO_COUNT: usize = 10;' "$_HZ2"
# The matrix EXERCISES the real machinery — it is NOT a hard-coded table: the re-derive verifier, the real
# curator, a non-vacuous mutation guard, the unknown-level lookup, and the turn-bound check are all called.
grep -qF 'verify_horizon_json(' "$_HZ2"
grep -qF 'curate(' "$_HZ2"
grep -qF 'mutated != canonical' "$_HZ2"
grep -qF 'HorizonLevel::from_slug(' "$_HZ2"
grep -qF 'within_turn_bound(' "$_HZ2"
# The ten failure scenario names are all present (a dropped cell fails closed here).
for _hz2 in uncurated_candidate_refused missing_grounding_refused missing_replay_refused \
            dream_to_evidence_refused hypothesis_to_evidence_refused training_open_refused \
            authority_escalation_refused max_turns_overflow_refused unknown_horizon_level_refused \
            serialized_trace_replay_refused; do
  if ! grep -qF "$_hz2" "$_HZ2"; then exit 1; fi
done
# The refusal + training-closed cells and the five refusal mechanisms are recorded.
grep -qF 'pub refused: bool' "$_HZ2"
grep -qF 'pub training_still_closed: bool' "$_HZ2"
for _hz2m in 'RefusalMechanism::CurationRejected' 'RefusalMechanism::CurationQuarantined' \
             'RefusalMechanism::VerifyMismatch' 'RefusalMechanism::TurnBoundExceeded' \
             'RefusalMechanism::UnknownLevel'; do
  if ! grep -qF "$_hz2m" "$_HZ2"; then exit 1; fi
done
# The nine-line HORIZON-2 failure-matrix boundary is recorded verbatim.
for _hz2b in 'The horizon failure matrix mutates bounded traces.' 'It observes refusals.' \
             'It does not create truth.' 'It does not create memory.' 'It does not train.' \
             'It does not execute external actions.' 'It does not promote hypotheses.' \
             'It does not grant new authority.' 'Training eligibility remains closed.'; do
  if ! grep -qF "$_hz2b" "$_HZ2"; then exit 1; fi
done
# HORIZON-2 makes NO false training claim in the matrix source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_HZ2"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# HORIZON-3 — horizon track milestone freeze. The HORIZON-0 -> HORIZON-2 staged-interaction arc (the
# bounded-horizon harness, the operator guard, and the failure matrix) is frozen as horizon-track-v0.1. The
# milestone record (HORIZON_TRACK_MILESTONE.md) pins the HORIZON-0..HORIZON-2 commit lineage, the prior frozen
# substrate base (data-curation-v0.1) and the deeper frozen milestone tags + commits, the six H0..H5 bounded
# horizons, the ten HORIZON-2 failure scenarios, the cannot-bypass boundary (curation / grounding / replay /
# authority / training closure), the structurally-closed training eligibility, the P12 verdict, and the honest
# residuals, and is locked here so the freeze cannot silently drift. The pinned commit hashes are auditable
# against `git log`; this lock stays git-free and does NOT require the tag to exist (the tag is created only
# after a clean tree + green gate). Documentation freeze only — no code crate change, no model, no training; the
# milestone records training_not_justified. Doctrine: The horizon track stages bounded interaction depth. It
# does not create truth. It does not create memory. It does not train. It does not execute external actions. It
# does not promote hypotheses. It does not grant new authority. Longer horizons cannot bypass curation,
# grounding, replay, authority, or training closure.
# ---------------------------------------------------------------------------------------------------
test -f HORIZON_TRACK_MILESTONE.md
grep -q 'FROZEN' HORIZON_TRACK_MILESTONE.md
grep -qF 'horizon-track-v0.1' HORIZON_TRACK_MILESTONE.md
grep -qF 'HORIZON-0' HORIZON_TRACK_MILESTONE.md
grep -qF 'HORIZON-1' HORIZON_TRACK_MILESTONE.md
grep -qF 'HORIZON-2' HORIZON_TRACK_MILESTONE.md
grep -qF 'training_not_justified' HORIZON_TRACK_MILESTONE.md
grep -qF 'training_justified=false' HORIZON_TRACK_MILESTONE.md
# Full HORIZON-0..HORIZON-2 commit lineage is pinned (cross-checkable against git log).
grep -qF 'db8a776' HORIZON_TRACK_MILESTONE.md
grep -qF 'b20b2e4' HORIZON_TRACK_MILESTONE.md
grep -qF 'd86799e' HORIZON_TRACK_MILESTONE.md
# data-curation-v0.1 is named as the prior frozen substrate milestone (tag + commit), and the deeper frozen
# milestones are referenced (tag + commit).
grep -qF 'data-curation-v0.1' HORIZON_TRACK_MILESTONE.md
for _t in data-curation-v0.1 dream-export-v0.1 corpus-flow-v0.1 document-flow-v0.1 operator-controls-v0.1 multi-trace-validation-v0.1 integration-demo-v0.1 hypothesis-track-v0.1 reading-track-v0.1 cognitive-os-governance-v0.1; do
  if ! grep -qF "$_t" HORIZON_TRACK_MILESTONE.md; then exit 1; fi
done
for _sha in b47665b 5238fe8 b8577fe 0cc7399 34b4f47 460be0c 95b586d bb20acf f6fa55a bbd1113; do
  if ! grep -qF "$_sha" HORIZON_TRACK_MILESTONE.md; then exit 1; fi
done
# The three frozen horizon capabilities are referenced by name (the harness, the operator guard manual, the
# failure matrix).
grep -qF 'horizon.rs' HORIZON_TRACK_MILESTONE.md
grep -qF 'run_horizon' HORIZON_TRACK_MILESTONE.md
grep -qF 'OPERATOR_MANUAL.md' HORIZON_TRACK_MILESTONE.md
grep -qF 'horizon_failure_matrix' HORIZON_TRACK_MILESTONE.md
# All six H0..H5 bounded horizons are listed.
for _hl in H0 H1 H2 H3 H4 H5; do
  if ! grep -qF "$_hl" HORIZON_TRACK_MILESTONE.md; then exit 1; fi
done
# All ten HORIZON-2 failure scenario names are listed in the freeze record.
for _fs in uncurated_candidate_refused missing_grounding_refused missing_replay_refused \
           dream_to_evidence_refused hypothesis_to_evidence_refused training_open_refused \
           authority_escalation_refused max_turns_overflow_refused unknown_horizon_level_refused \
           serialized_trace_replay_refused; do
  if ! grep -qF "$_fs" HORIZON_TRACK_MILESTONE.md; then exit 1; fi
done
# The cannot-bypass boundary the rubric requires is recorded verbatim, and training eligibility stays closed.
grep -qF 'cannot bypass curation, grounding, replay, authority, or training closure' HORIZON_TRACK_MILESTONE.md
grep -qF 'Training eligibility remains closed.' HORIZON_TRACK_MILESTONE.md
# The HORIZON-0 code-true harness boundary is cross-referenced verbatim (the surface the freeze preserves).
grep -qF 'The horizon harness measures bounded interaction depth.' HORIZON_TRACK_MILESTONE.md
grep -qF 'Longer horizons cannot bypass earlier gates.' HORIZON_TRACK_MILESTONE.md
# The nine-line HORIZON-3 freeze boundary is recorded verbatim (all nine lines).
for _bl in 'The horizon track stages bounded interaction depth.' 'It composes verified reading, curation, replay, dream, and hypothesis flows.' 'It does not create truth.' 'It does not create memory.' 'It does not train.' 'It does not execute external actions.' 'It does not promote hypotheses.' 'It does not grant new authority.' 'Longer horizons cannot bypass curation, grounding, replay, authority, or training closure.'; do
  if ! grep -qF "$_bl" HORIZON_TRACK_MILESTONE.md; then exit 1; fi
done
# The milestone makes NO false training claim (it never asserts training opened).
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' HORIZON_TRACK_MILESTONE.md; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# CORPUS-HARVEST-0 — the first model-readiness corpus-harvest pipeline. crates/cognitive-demo/src/corpus_harvest.rs
# collects already-verified substrate artifacts into a deterministic CuratedCorpusReceipt, but owns NO admission
# logic: every candidate is routed through the REAL DATA-0 gate (data_curator::curate) BEFORE it can become
# harvest material — admitted items become HarvestItems, rejected items are preserved in RejectedItemsReport,
# quarantined items are preserved in QuarantineReport (quarantine HOLDS, never deletes). It reads only
# caller-supplied CandidateManifest values (no filesystem, no memory ingest), is Serialize but NOT Deserialize
# (re-derived and byte-compared via verify_harvest_json with a non-vacuous tamper guard), and adds no
# training-permitting state — eligibility is the curator's own Closed/CandidateOnly (is_eligible() == false), so
# no harvest item is training eligible and opens_training is false in every scenario. The cargo unit-count pin
# above already RUNS the 26 CORPUS-HARVEST-0 tests; the source pins below stop the pipeline from faking curation,
# dropping a report, opening training, or trusting a serialized harvest. A capability sprint that ADDS the harvest
# module + tests — no other crate changes; the DATA-0 gate is unchanged. Doctrine: The corpus harvest path
# collects curated candidate data. It does not create truth. It does not create memory. It does not create
# evidence. It does not train. It does not execute external actions. It does not promote hypotheses. It does not
# grant new authority. Training eligibility remains closed.
# ---------------------------------------------------------------------------------------------------
_CH=crates/cognitive-demo/src/corpus_harvest.rs
test -f "$_CH"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod corpus_harvest;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use corpus_harvest::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn harvest_corpus(' "$_CH"
grep -qF 'pub fn corpus_harvest_matrix(' "$_CH"
grep -qF 'pub const HARVEST_SCENARIO_COUNT: usize = 14;' "$_CH"
# It DELEGATES to the REAL curator — it does not decide admissibility itself (a faked pipeline drops this call).
grep -qF 'use data_curator::' "$_CH"
grep -qF 'let receipt = curate(&src.manifest);' "$_CH"
# The harvest preserves admitted / rejected / quarantined items (never silently dropped) in their own records.
grep -qF 'pub struct HarvestItem' "$_CH"
grep -qF 'pub struct RejectedItemsReport' "$_CH"
grep -qF 'pub struct QuarantineReport' "$_CH"
grep -qF 'pub struct SplitIntegrityReport' "$_CH"
grep -qF 'pub struct CorpusHarvestManifest' "$_CH"
grep -qF 'pub struct CuratedCorpusReceipt' "$_CH"
grep -qF 'pub struct CorpusHarvestMatrix' "$_CH"
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares with a
# non-vacuous tamper guard (a no-op mutation cannot pass).
grep -qF 'Serialize' "$_CH"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_CH")" -eq 0
grep -qF 'pub fn verify_harvest_json(' "$_CH"
grep -qF 'tampered != canonical' "$_CH"
# All fourteen harvest scenario names are present (a dropped cell fails closed here).
for _ch2 in clean_document_harvested clean_corpus_harvested missing_provenance_rejected \
            duplicate_id_rejected empty_content_rejected unsupported_artifact_rejected \
            prompt_injection_quarantined split_leakage_quarantined \
            durable_claim_without_grounding_rejected trace_without_replay_rejected \
            valid_split_recorded invalid_split_rejected candidate_only_not_training_eligible \
            serialized_harvest_replay_refused; do
  if ! grep -qF "\"$_ch2\"" "$_CH"; then exit 1; fi
done
# Training eligibility cannot open: the harvest reuses the curator's TrainingEligibility (it defines NO eligibility
# enum of its own — no Eligible/TrainingEligible variant is introduced), opens_training is is_eligible() (pinned
# false by the DATA-0 gate above), and the matrix-level training_never_opens invariant is computed.
test "$(grep -cE 'enum TrainingEligibility' "$_CH")" -eq 0
grep -qF 'let opens_training = training_eligibility.is_eligible();' "$_CH"
grep -qF 'training_never_opens' "$_CH"
# The harvest tests assert delegation, report preservation, no-training, and the serialized refusal (so the
# coverage cannot be silently removed from the test battery the unit-count pin runs above).
for _cht in 'fn harvest_delegates_to_curator_and_admits_clean_document' \
            'fn harvest_does_not_silently_drop_rejected_or_quarantined' \
            'fn no_harvest_item_is_training_eligible' 'fn matrix_serialized_replay_is_refused' \
            'fn matrix_opens_no_training_in_any_scenario'; do
  if ! grep -qF "$_cht" "$_CH"; then exit 1; fi
done
# The nine-line CORPUS-HARVEST-0 boundary is recorded verbatim.
for _chb in 'The corpus harvest path collects curated candidate data.' 'It does not create truth.' \
            'It does not create memory.' 'It does not create evidence.' 'It does not train.' \
            'It does not execute external actions.' 'It does not promote hypotheses.' \
            'It does not grant new authority.' 'Training eligibility remains closed.'; do
  if ! grep -qF "$_chb" "$_CH"; then exit 1; fi
done
# CORPUS-HARVEST-0 makes NO false training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_CH"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# SCORE-0 — the verifier-as-scorer. It turns the EXISTING verifier outcomes (DATA-0 curation, corpus-harvest
# replay, HORIZON gates, INT-0 trace verification) into deterministic ScoreReceipts — but a score is an
# OBSERVATION, never authority. Every score is read off a REAL verifier run (the scorer decides no verdict
# itself); no score can promote evidence, create memory, grant authority, or open training; a failure is
# recorded as a FailureObservation, NEVER a training example (training_example is the structural const
# FAILURES_ARE_TRAINING_EXAMPLES = false). The matrix + receipts are Serialize but NOT Deserialize (re-derived
# and byte-compared via verify_score_matrix_json / verify_score_receipt_json with a non-vacuous tamper guard).
# The cargo unit-count pin above already RUNS the 20 SCORE-0 tests; the source pins below stop the pipeline from
# hard-coding scores, dropping the real verifier calls, opening training, or trusting a serialized score. A
# capability sprint that ADDS the score module + tests — no other crate changes; the frozen gates are unchanged.
# Doctrine: The scoring path observes verifier outcomes. It does not create truth. It does not create memory. It
# does not create evidence. It does not train. It does not execute external actions. It does not promote
# hypotheses. It does not grant new authority. Scores cannot open training eligibility.
# ---------------------------------------------------------------------------------------------------
_SCORE=crates/cognitive-demo/src/score.rs
test -f "$_SCORE"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod score;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use score::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn verifier_score_matrix(' "$_SCORE"
grep -qF 'pub fn verify_score_matrix_json(' "$_SCORE"
grep -qF 'pub fn verify_score_receipt_json(' "$_SCORE"
# The seven core score objects exist.
grep -qF 'pub struct ScoreReceipt' "$_SCORE"
grep -qF 'pub struct ScoreCell' "$_SCORE"
grep -qF 'pub enum ScoreClass' "$_SCORE"
grep -qF 'pub enum ScoreReason' "$_SCORE"
grep -qF 'pub struct VerifierScoreMatrix' "$_SCORE"
grep -qF 'pub struct FailureObservation' "$_SCORE"
grep -qF 'pub struct ScoringBoundary' "$_SCORE"
# The score class count is exactly seven, and all seven class names are present.
grep -qF 'pub const SCORE_CLASS_COUNT: usize = 7;' "$_SCORE"
for _scls in grounding_score replay_score curation_score horizon_boundary_score \
             refusal_score answer_support_score trace_integrity_score; do
  if ! grep -qF "$_scls" "$_SCORE"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all sixteen scenario names are present
# (a dropped cell fails closed here). training_never_opens is the matrix-level conjunction.
grep -qF 'pub const SCORE_SCENARIO_COUNT: usize = 16;' "$_SCORE"
for _ssc in grounded_answer_scores_pass ungrounded_answer_scores_fail valid_replay_scores_pass \
            tampered_replay_scores_fail curated_candidate_scores_pass \
            quarantined_candidate_scores_observed rejected_candidate_scores_fail \
            horizon_valid_trace_scores_pass horizon_boundary_failure_scores_fail \
            refusal_correct_scores_pass refusal_missing_scores_fail answer_support_pass \
            answer_support_fail trace_integrity_pass trace_integrity_tamper_fail \
            score_receipt_tamper_refused; do
  if ! grep -qF "\"$_ssc\"" "$_SCORE"; then exit 1; fi
done
grep -qF 'training_never_opens' "$_SCORE"
# The scores are read off REAL verifier/curator/horizon/harvest runs — the scorer decides no verdict itself.
# Dropping any of these calls (faking a hard-coded score) fails closed here.
grep -qF 'curate(' "$_SCORE"
grep -qF 'verify_harvest_json(' "$_SCORE"
grep -qF 'verify_trace_json(' "$_SCORE"
grep -qF 'run_horizon(' "$_SCORE"
grep -qF 'doc_trace(' "$_SCORE"
grep -qF 'horizon_failure_matrix(' "$_SCORE"
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares with a
# non-vacuous tamper guard (a no-op mutation cannot pass).
grep -qF 'Serialize' "$_SCORE"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_SCORE")" -eq 0
grep -qF 'tampered != canonical' "$_SCORE"
# A failure is OBSERVED for audit, NEVER a training example: the training_example flag is the structural const
# FAILURES_ARE_TRAINING_EXAMPLES = false, and is_training_example reads that flag (no path constructs it true).
grep -qF 'const FAILURES_ARE_TRAINING_EXAMPLES: bool = false;' "$_SCORE"
grep -qF 'training_example: FAILURES_ARE_TRAINING_EXAMPLES' "$_SCORE"
grep -qF 'pub fn is_training_example' "$_SCORE"
# A score cannot open training or launder one authority class into a stronger one: opens_training is false, and
# NO boundary/eligibility invariant is ever set true (a converted_* / opened_training / created_* set true fails).
grep -qF 'opens_training: false' "$_SCORE"
grep -qF 'converted_candidate_to_training_eligible' "$_SCORE"
grep -qF 'converted_hypothesis_to_evidence' "$_SCORE"
grep -qF 'converted_dream_to_export_authority' "$_SCORE"
if grep -qE '(opened_training|created_truth|created_memory|created_evidence|granted_authority|promoted_hypothesis|converted_[a-z_]+|opens_training):[[:space:]]*true' "$_SCORE"; then exit 1; fi
# The score tests assert the seven classes, the observed states, the never-training failures, the false-positive
# / false-negative answer-support guards, and the serialized refusal (so the coverage cannot be silently removed
# from the battery the unit-count pin runs above).
for _sct in 'fn there_are_exactly_seven_score_classes_with_stable_names' \
            'fn matrix_records_the_observed_states' 'fn matrix_failures_are_never_training_examples' \
            'fn answer_support_pass_for_matching_hash_fail_for_different_hash' \
            'fn matrix_has_the_sixteen_named_scenarios' 'fn matrix_json_re_derives_and_refuses_tampering' \
            'fn matrix_opens_no_training_and_boundary_is_inert'; do
  if ! grep -qF "$_sct" "$_SCORE"; then exit 1; fi
done
# The nine-line SCORE-0 boundary is recorded verbatim.
for _scb in 'The scoring path observes verifier outcomes.' 'It does not create truth.' \
            'It does not create memory.' 'It does not create evidence.' 'It does not train.' \
            'It does not execute external actions.' 'It does not promote hypotheses.' \
            'It does not grant new authority.' 'Scores cannot open training eligibility.'; do
  if ! grep -qF "$_scb" "$_SCORE"; then exit 1; fi
done
# SCORE-0 makes NO false training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_SCORE"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# FAIL-0 — the recurring-clean-failure detector. It CONSUMES SCORE-0 FailureObservation values (it cannot
# fabricate one — SCORE-0's constructor is private, so every signal's evidence is pulled from
# verifier_score_matrix()) and answers ONLY "did the same clean failure recur enough to be a
# ModelNeedCandidate?" — never "should we train?". It separates clean MODEL failures from SUBSTRATE
# failures (a replay/trace-integrity failure is never a model need) and the eight EXCLUSIONS (missing
# context / bad retrieval / uncurated data / bad prompt-or-schema / invalid test / stale artifact /
# unverified replay / quarantined candidate). A ModelNeedCandidate is emitted ONLY at the explicit,
# deterministic RECURRENCE_THRESHOLD with a stable class+reason — a single failure never emits one — and
# it is structurally NOT training authorization (training_justified / opens_training / authorizes_training
# all sourced from the const MODEL_NEED_IS_TRAINING_AUTHORIZATION = false). Reports are Serialize but NOT
# Deserialize (re-derived + byte-compared with a non-vacuous tamper guard). The cargo unit-count pin above
# already RUNS the 19 FAIL-0 tests; the source pins below stop the pipeline from hard-coding outcomes,
# dropping the SCORE-0 consumption, counting a single/excluded/substrate failure as a model need, or
# opening training. A capability sprint that ADDS the detector module + tests — no other crate changes.
# Doctrine: The failure detector observes recurring clean failures. It does not create truth. It does not
# create memory. It does not create evidence. It does not train. It does not execute external actions. It
# does not promote hypotheses. It does not grant new authority. ModelNeedCandidate is not training authorization.
# ---------------------------------------------------------------------------------------------------
_FAIL=crates/cognitive-demo/src/failure_detector.rs
test -f "$_FAIL"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod failure_detector;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use failure_detector::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn detect_failures(' "$_FAIL"
grep -qF 'pub fn verify_failure_report_json(' "$_FAIL"
grep -qF 'pub fn failure_detector_matrix(' "$_FAIL"
grep -qF 'pub fn verify_failure_detector_matrix_json(' "$_FAIL"
# The nine FAIL-0 core objects exist.
grep -qF 'pub struct FailureDetectorReport' "$_FAIL"
grep -qF 'pub struct FailureCase' "$_FAIL"
grep -qF 'pub enum FailureClass' "$_FAIL"
grep -qF 'pub enum FailureCause' "$_FAIL"
grep -qF 'pub enum CleanFailureStatus' "$_FAIL"
grep -qF 'pub struct ModelNeedCandidate' "$_FAIL"
grep -qF 'pub struct FailureRecurrencePolicy' "$_FAIL"
grep -qF 'pub enum FailureExclusion' "$_FAIL"
grep -qF 'pub struct FailureDetectorMatrix' "$_FAIL"
# The failure class count is exactly ten, and all ten class names are present.
grep -qF 'pub const FAILURE_CLASS_COUNT: usize = 10;' "$_FAIL"
for _fcl in reading_misgrounding source_selection_failure multi_doc_synthesis_failure \
            horizon_plan_failure tool_use_schema_failure refusal_boundary_failure \
            memory_retrieval_failure instruction_following_failure coding_patch_failure \
            replay_inconsistency; do
  if ! grep -qF "$_fcl" "$_FAIL"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all sixteen scenario names are present.
grep -qF 'pub const FAILURE_SCENARIO_COUNT: usize = 16;' "$_FAIL"
for _fsc in single_failure_no_candidate recurring_clean_model_failure_candidate \
            recurring_substrate_failure_no_candidate missing_context_excluded bad_retrieval_excluded \
            uncurated_data_excluded bad_prompt_schema_excluded invalid_test_excluded \
            stale_artifact_excluded unverified_replay_excluded quarantined_candidate_excluded \
            unstable_failure_class_excluded stable_failure_class_candidate \
            refusal_boundary_recurrence_candidate trace_integrity_failure_not_model_need \
            serialized_failure_report_tamper_refused; do
  if ! grep -qF "\"$_fsc\"" "$_FAIL"; then exit 1; fi
done
# The recurrence threshold is explicit and deterministic.
grep -qF 'pub const RECURRENCE_THRESHOLD: usize = 2;' "$_FAIL"
# It CONSUMES real SCORE-0 FailureObservations (cannot fabricate one — pulled from the matrix).
grep -qF 'FailureObservation' "$_FAIL"
grep -qF 'verifier_score_matrix()' "$_FAIL"
# A ModelNeedCandidate is NOT training authorization: training_justified / opens_training /
# authorizes_training are all sourced from the structural const (false); no path sets any true.
grep -qF 'const MODEL_NEED_IS_TRAINING_AUTHORIZATION: bool = false;' "$_FAIL"
grep -qF 'training_justified: MODEL_NEED_IS_TRAINING_AUTHORIZATION' "$_FAIL"
grep -qF 'authorizes_training' "$_FAIL"
grep -qF 'opens_training' "$_FAIL"
if grep -qE '(opened_training|created_truth|created_memory|created_evidence|granted_authority|promoted_hypothesis|authorizes_training|opens_training|training_justified):[[:space:]]*true' "$_FAIL"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares
# with a non-vacuous tamper guard (a no-op mutation cannot pass).
grep -qF 'Serialize' "$_FAIL"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_FAIL")" -eq 0
grep -qF 'tampered != canonical' "$_FAIL"
# The detector tests assert recurrence, every exclusion, substrate-is-not-a-model-need, training closure,
# and the serialized re-derivation (so the coverage cannot be silently removed from the battery above).
for _ftt in 'fn single_clean_failure_emits_no_candidate' 'fn recurring_clean_model_failure_emits_candidate' \
            'fn each_exclusion_blocks_a_candidate' 'fn recurring_substrate_failure_is_not_a_model_need' \
            'fn detector_never_opens_training_even_with_candidates' \
            'fn matrix_has_the_sixteen_named_scenarios' \
            'fn report_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_ftt" "$_FAIL"; then exit 1; fi
done
# The nine-line FAIL-0 boundary is recorded verbatim.
for _fbl in 'The failure detector observes recurring clean failures.' 'It does not create truth.' \
            'It does not create memory.' 'It does not create evidence.' 'It does not train.' \
            'It does not execute external actions.' 'It does not promote hypotheses.' \
            'It does not grant new authority.' 'ModelNeedCandidate is not training authorization.'; do
  if ! grep -qF "$_fbl" "$_FAIL"; then exit 1; fi
done
# FAIL-0 makes NO false training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_FAIL"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# P11-MODEL-EVAL — the honest fork. It CONSUMES FAIL-0 ModelNeedCandidate records (built by the REAL
# detect_failures() over REAL SCORE-0 failures — the SCORE-0 -> FAIL-0 -> MODEL-EVAL chain) plus
# baseline/prompt/retrieval/horizon/substrate comparison observations, and emits a deterministic
# ModelNeedVerdict (no_training_needed / improve_substrate_first / collect_more_data /
# training_candidate_only) WITHOUT opening training, touching weights, or promoting a model. A failure
# REMOVED by a non-weight fix is no model need; a SUBSTRATE-levered failure -> improve_substrate_first; a
# RESIDUAL clean failure that survives ALL cleanup AND a trustworthy holdout is the only thing that can
# yield training_candidate_only — and even that needs >= MODEL_NEED_MIN_RESIDUALS (a single residual falls
# to collect_more_data), and a contaminated holdout or memorization leakage forces collect_more_data (never
# passes). training_candidate_only is NOT authorization: the report's + signal's training_justified /
# opens_training / authorizes_training are all sourced from the const TRAINING_CANDIDATE_IS_AUTHORIZATION =
# false. Reports are Serialize but NOT Deserialize (re-derived + byte-compared with a non-vacuous tamper
# guard). The cargo unit-count pin above already RUNS the 18 P11-MODEL-EVAL tests; the source pins below stop
# the pipeline from hard-coding verdicts, dropping the FAIL-0 consumption, passing contaminated/leaked
# evidence, or opening training. A capability sprint that ADDS the eval module + tests — no other crate
# changes. Doctrine: The model-need evaluation compares residual clean failures. It does not create truth. It
# does not create memory. It does not create evidence. It does not train. It does not execute external
# actions. It does not promote models. It does not grant new authority. TrainingCandidateOnly is not training
# authorization.
# ---------------------------------------------------------------------------------------------------
_MEVAL=crates/cognitive-demo/src/model_eval.rs
test -f "$_MEVAL"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod model_eval;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use model_eval::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn evaluate_model_need(' "$_MEVAL"
grep -qF 'pub fn verify_model_eval_report_json(' "$_MEVAL"
grep -qF 'pub fn model_eval_matrix(' "$_MEVAL"
grep -qF 'pub fn verify_model_eval_matrix_json(' "$_MEVAL"
# The ten MODEL-EVAL core objects exist.
grep -qF 'pub struct ModelNeedEvalReport' "$_MEVAL"
grep -qF 'pub enum ModelNeedVerdict' "$_MEVAL"
grep -qF 'pub struct ModelEvalBattery' "$_MEVAL"
grep -qF 'pub struct EvalRun' "$_MEVAL"
grep -qF 'pub enum EvalCondition' "$_MEVAL"
grep -qF 'pub struct EvalComparison' "$_MEVAL"
grep -qF 'pub struct ResidualFailure' "$_MEVAL"
grep -qF 'pub struct ModelNeedEvidence' "$_MEVAL"
grep -qF 'pub struct TrainingCandidateSignal' "$_MEVAL"
grep -qF 'pub struct ModelEvalMatrix' "$_MEVAL"
# The verdict count is exactly four, and all four verdict names are present.
grep -qF 'pub const VERDICT_COUNT: usize = 4;' "$_MEVAL"
for _vn in no_training_needed improve_substrate_first collect_more_data training_candidate_only; do
  if ! grep -qF "$_vn" "$_MEVAL"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all fifteen scenario names are present.
grep -qF 'pub const MODEL_EVAL_SCENARIO_COUNT: usize = 15;' "$_MEVAL"
for _es in no_candidates_no_training_needed substrate_failures_improve_substrate_first \
           insufficient_evidence_collect_more_data unstable_candidate_collect_more_data \
           residual_clean_failure_training_candidate_only prompt_fix_removes_model_need \
           retrieval_fix_removes_model_need horizon_fix_removes_model_need \
           substrate_fix_removes_model_need holdout_clean_recorded holdout_contamination_detected \
           memorization_leakage_detected single_candidate_not_enough \
           serialized_eval_report_tamper_refused training_candidate_only_not_authorization; do
  if ! grep -qF "\"$_es\"" "$_MEVAL"; then exit 1; fi
done
# It CONSUMES real FAIL-0 ModelNeedCandidates (built via the real detect_failures over SCORE-0 failures).
grep -qF 'ModelNeedCandidate' "$_MEVAL"
grep -qF 'detect_failures(' "$_MEVAL"
grep -qF 'verifier_score_matrix()' "$_MEVAL"
# The residual policy is explicit (a single candidate is not enough by itself).
grep -qF 'pub const MODEL_NEED_MIN_RESIDUALS: usize = 2;' "$_MEVAL"
# Holdout-contamination and memorization-leakage detection exist (and force collect_more_data).
grep -qF 'holdout_contaminated' "$_MEVAL"
grep -qF 'memorization_leaked' "$_MEVAL"
# training_candidate_only is NOT authorization: every training flag is sourced from the structural const
# (false); no path sets any true.
grep -qF 'const TRAINING_CANDIDATE_IS_AUTHORIZATION: bool = false;' "$_MEVAL"
grep -qF 'training_justified: TRAINING_CANDIDATE_IS_AUTHORIZATION' "$_MEVAL"
grep -qF 'authorizes_training' "$_MEVAL"
grep -qF 'opens_training' "$_MEVAL"
if grep -qE '(opened_training|created_truth|created_memory|created_evidence|granted_authority|promoted_model|executed_external|authorizes_training|opens_training|training_justified):[[:space:]]*true' "$_MEVAL"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares with
# a non-vacuous tamper guard (a no-op mutation cannot pass).
grep -qF 'Serialize' "$_MEVAL"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_MEVAL")" -eq 0
grep -qF 'tampered != canonical' "$_MEVAL"
# The eval tests assert the verdicts, the chain to real FAIL-0 candidates, the contamination/leakage refusal,
# the single-not-enough rule, training closure, and the serialized re-derivation.
for _et in 'fn candidates_come_from_the_real_fail0_detector' \
           'fn two_residual_clean_failures_yield_training_candidate_only_not_authorization' \
           'fn single_residual_is_not_enough' 'fn contaminated_holdout_never_passes' \
           'fn memorization_leakage_never_passes' \
           'fn evaluation_never_opens_training_even_for_training_candidate_only' \
           'fn matrix_has_the_fifteen_named_scenarios' \
           'fn report_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_et" "$_MEVAL"; then exit 1; fi
done
# The nine-line MODEL-EVAL boundary is recorded verbatim.
for _eb in 'The model-need evaluation compares residual clean failures.' 'It does not create truth.' \
           'It does not create memory.' 'It does not create evidence.' 'It does not train.' \
           'It does not execute external actions.' 'It does not promote models.' \
           'It does not grant new authority.' 'TrainingCandidateOnly is not training authorization.'; do
  if ! grep -qF "$_eb" "$_MEVAL"; then exit 1; fi
done
# P11-MODEL-EVAL makes NO false training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_MEVAL"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# TRAIN-GATE-0 — the explicit, CLOSED-BY-DEFAULT gate before any weight change. It answers exactly ONE
# question: are the prerequisites complete enough to allow a FUTURE training attempt? It CONSUMES the
# REAL P11-MODEL-EVAL verdict (running evaluate_model_need() itself over the supplied battery — the
# SCORE-0 -> FAIL-0 -> MODEL-EVAL -> TRAIN-GATE chain) and emits training_attempt_allowed ONLY when the
# verdict is EXACTLY training_candidate_only AND every requirement holds together: recurring-failure
# evidence (>= MIN_RECURRING_FAILURES), an explicit operator authorization receipt, curated dataset
# receipts, a present+uncontaminated holdout, a clean contamination/memorization report, a rollback
# plan, a production safety plan, and an affirmative authority-drift check. A training_candidate_only
# verdict ALONE is insufficient; operator authorization ALONE is insufficient; an absent contamination
# report is NOT proven clean; an unchecked drift state is NOT clean. training_attempt_allowed is ONLY
# permission to ATTEMPT a later run: the report's trains / modifies_weights / promotes_model /
# deploys_model / training_justified / opens_training are ALL sourced from the const
# ALLOWED_ATTEMPT_AUTHORIZES_TRAINING = false, so no path sets any true, and the deeper P12 gate stays
# training_justified = false regardless of the decision. Reports are Serialize but NOT Deserialize
# (re-derived + byte-compared with a non-vacuous tamper guard). The cargo unit-count pin above already
# RUNS the 20 TRAIN-GATE-0 tests; the source pins below stop the pipeline from hard-coding the verdict,
# dropping the P11 consumption, weakening a requirement, or opening training. A capability sprint that
# ADDS the gate module + tests — no other crate changes. Doctrine: The training gate evaluates whether a
# training attempt may be authorized. It does not train. It does not modify weights. It does not create
# truth. It does not create memory. It does not create evidence. It does not promote models. It does not
# deploy models. TrainingAttemptAllowed is not model promotion.
# ---------------------------------------------------------------------------------------------------
_TGATE=crates/cognitive-demo/src/training_gate.rs
test -f "$_TGATE"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod training_gate;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use training_gate::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn evaluate_training_gate(' "$_TGATE"
grep -qF 'pub fn evaluate_training_gate_json(' "$_TGATE"
grep -qF 'pub fn verify_training_gate_report_json(' "$_TGATE"
grep -qF 'pub fn training_gate_matrix(' "$_TGATE"
grep -qF 'pub fn verify_training_gate_matrix_json(' "$_TGATE"
# The core objects exist.
grep -qF 'pub struct TrainingGateReport' "$_TGATE"
grep -qF 'pub enum TrainingGateDecision' "$_TGATE"
grep -qF 'pub struct TrainingGateInput' "$_TGATE"
grep -qF 'pub enum TrainingGateRequirement' "$_TGATE"
grep -qF 'pub enum TrainingGateRefusal' "$_TGATE"
grep -qF 'pub struct TrainingGateMatrix' "$_TGATE"
grep -qF 'pub struct OperatorAuthorizationReceipt' "$_TGATE"
grep -qF 'pub struct RollbackPlanReceipt' "$_TGATE"
grep -qF 'pub struct DatasetReadinessReceipt' "$_TGATE"
grep -qF 'pub struct HoldoutReadinessReceipt' "$_TGATE"
grep -qF 'pub struct ContaminationReportReceipt' "$_TGATE"
grep -qF 'pub struct ProductionSafetyPlanReceipt' "$_TGATE"
# Both decision states exist (the allowed/denied variants) and the count is exactly two.
grep -qF 'TrainingAttemptAllowed' "$_TGATE"
grep -qF 'TrainingAttemptDenied' "$_TGATE"
grep -qF 'pub const TRAIN_GATE_DECISION_COUNT: usize = 2;' "$_TGATE"
for _dn in training_attempt_denied training_attempt_allowed; do
  if ! grep -qF "\"$_dn\"" "$_TGATE"; then exit 1; fi
done
# The refusal count is exactly twelve, and all twelve refusal-reason names are present.
grep -qF 'pub const TRAIN_GATE_REFUSAL_COUNT: usize = 12;' "$_TGATE"
for _rr in missing_model_need_verdict verdict_not_training_candidate missing_operator_authorization \
           missing_curated_dataset_receipts missing_clean_holdout holdout_contaminated \
           memorization_leakage_detected missing_recurring_failure_evidence missing_rollback_plan \
           missing_production_safety_plan authority_drift_detected \
           training_gate_serialized_tamper_refused; do
  if ! grep -qF "\"$_rr\"" "$_TGATE"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all nineteen scenario names are present.
grep -qF 'pub const TRAIN_GATE_SCENARIO_COUNT: usize = 19;' "$_TGATE"
for _gs in closed_by_default_denied missing_model_need_verdict_denied no_training_needed_denied \
           improve_substrate_first_denied collect_more_data_denied \
           training_candidate_without_operator_auth_denied training_candidate_without_dataset_denied \
           training_candidate_without_holdout_denied holdout_contaminated_denied \
           memorization_leakage_denied missing_recurring_failure_evidence_denied \
           missing_rollback_plan_denied missing_production_safety_plan_denied authority_drift_denied \
           all_requirements_met_training_attempt_allowed allowed_is_not_training_execution \
           allowed_is_not_model_promotion serialized_gate_report_tamper_refused \
           training_justified_remains_false; do
  if ! grep -qF "\"$_gs\"" "$_TGATE"; then exit 1; fi
done
# It CONSUMES the real P11-MODEL-EVAL verdict (runs evaluate_model_need over a real battery).
grep -qF 'evaluate_model_need(' "$_TGATE"
grep -qF 'ModelNeedVerdict' "$_TGATE"
grep -qF 'ModelEvalBattery' "$_TGATE"
# The verdict must be EXACTLY training_candidate_only, and a single candidate is not enough.
grep -qF 'ModelNeedVerdict::TrainingCandidateOnly' "$_TGATE"
grep -qF 'pub const MIN_RECURRING_FAILURES: usize = 2;' "$_TGATE"
# Every requirement is enforced (the receipts + the drift check are load-bearing).
grep -qF 'MissingOperatorAuthorization' "$_TGATE"
grep -qF 'MissingCuratedDatasetReceipts' "$_TGATE"
grep -qF 'MissingCleanHoldout' "$_TGATE"
grep -qF 'HoldoutContaminated' "$_TGATE"
grep -qF 'MemorizationLeakageDetected' "$_TGATE"
grep -qF 'MissingRollbackPlan' "$_TGATE"
grep -qF 'MissingProductionSafetyPlan' "$_TGATE"
grep -qF 'AuthorityDriftDetected' "$_TGATE"
grep -qF 'is_clean()' "$_TGATE"
# training_attempt_allowed is NOT authorization/execution/promotion/deployment: every forbidden flag is
# sourced from the structural const (false); no path sets any true.
grep -qF 'const ALLOWED_ATTEMPT_AUTHORIZES_TRAINING: bool = false;' "$_TGATE"
grep -qF 'trains: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING' "$_TGATE"
grep -qF 'training_justified: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING' "$_TGATE"
grep -qF 'promotes_model: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING' "$_TGATE"
grep -qF 'deploys_model: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING' "$_TGATE"
grep -qF 'opens_training: ALLOWED_ATTEMPT_AUTHORIZES_TRAINING' "$_TGATE"
if grep -qE '(opened_training|created_truth|created_memory|created_evidence|granted_authority|promoted_model|promotes_model|deploys_model|modifies_weights|executed_external|trains|opens_training|training_justified):[[:space:]]*true' "$_TGATE"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares
# with a non-vacuous tamper guard (a no-op mutation cannot pass).
grep -qF 'Serialize' "$_TGATE"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_TGATE")" -eq 0
grep -qF 'tampered != canonical' "$_TGATE"
# The gate tests assert the P11 consumption, closed-by-default, candidate-alone-insufficient, the
# all-requirements allow, allow-is-not-authorization, the contamination/leakage refusal, the nineteen
# scenarios, and the serialized re-derivation.
for _gt in 'fn gate_consumes_the_real_p11_verdict' 'fn closed_by_default_denies_with_no_inputs' \
           'fn training_candidate_alone_is_insufficient' \
           'fn all_requirements_met_allows_a_training_attempt' \
           'fn allowed_attempt_is_not_training_authorization' \
           'fn contaminated_holdout_and_leakage_are_denied' \
           'fn matrix_has_the_nineteen_named_scenarios' \
           'fn report_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_gt" "$_TGATE"; then exit 1; fi
done
# The nine-line TRAIN-GATE boundary is recorded verbatim.
for _gb in 'The training gate evaluates whether a training attempt may be authorized.' \
           'It does not train.' 'It does not modify weights.' 'It does not create truth.' \
           'It does not create memory.' 'It does not create evidence.' 'It does not promote models.' \
           'It does not deploy models.' 'TrainingAttemptAllowed is not model promotion.'; do
  if ! grep -qF "$_gb" "$_TGATE"; then exit 1; fi
done
# TRAIN-GATE-0 makes NO false training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_TGATE"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# TRAIN-0 — the first gated, deterministic local training-ATTEMPT harness. It CONSUMES the real
# TRAIN-GATE-0 report (running evaluate_training_gate itself, which re-runs P11) and enforces TWO keys
# before preparing anything: the gate must emit TrainingAttemptAllowed AND a SEPARATE explicit operator
# authorization for the attempt must be present — neither alone suffices. A dry_run_only invocation
# prepares a plan that touches no weights and yields no candidate; an authorized_local_attempt prepares
# a CandidateOnly, hash-pinned candidate descriptor ONLY when both keys turn and every reproducibility
# prerequisite holds. A candidate is never promoted, deployed, made evidence, written to memory, granted
# authority, or used to replace the baseline; it MUST be evaluated later by S8. The harness performs no
# real weight mutation and leaves P12 training_justified = false. Boundary: The training attempt path may
# create a candidate model artifact only after gate approval and explicit operator authorization. It does
# not promote models. It does not deploy models. It does not create truth. It does not create memory. It
# does not create evidence. It does not grant new authority. A candidate model is not an accepted model.
# A candidate model must pass later evaluation before promotion.
# ---------------------------------------------------------------------------------------------------
_TATT=crates/cognitive-demo/src/training_attempt.rs
test -f "$_TATT"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod training_attempt;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use training_attempt::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn run_training_attempt(' "$_TATT"
grep -qF 'pub fn run_training_attempt_json(' "$_TATT"
grep -qF 'pub fn verify_training_attempt_receipt_json(' "$_TATT"
grep -qF 'pub fn training_attempt_matrix(' "$_TATT"
grep -qF 'pub fn verify_training_attempt_matrix_json(' "$_TATT"
# The core objects exist.
grep -qF 'pub struct TrainingAttemptPlan' "$_TATT"
grep -qF 'pub struct TrainingAttemptReceipt' "$_TATT"
grep -qF 'pub enum TrainingAttemptMode' "$_TATT"
grep -qF 'pub struct TrainingRunConfig' "$_TATT"
grep -qF 'pub struct TrainingDatasetBundle' "$_TATT"
grep -qF 'pub struct TrainingCandidateArtifact' "$_TATT"
grep -qF 'pub struct TrainingBaselineArtifact' "$_TATT"
grep -qF 'pub struct TrainingRollbackArtifact' "$_TATT"
grep -qF 'pub struct TrainingHoldoutBundle' "$_TATT"
grep -qF 'pub enum TrainingAttemptRefusal' "$_TATT"
grep -qF 'pub struct TrainingAttemptMatrix' "$_TATT"
# Exactly two modes, and both mode names are present.
grep -qF 'pub const TRAIN_ATTEMPT_MODE_COUNT: usize = 2;' "$_TATT"
grep -qF 'DryRunOnly' "$_TATT"
grep -qF 'AuthorizedLocalAttempt' "$_TATT"
for _mn in dry_run_only authorized_local_attempt; do
  if ! grep -qF "\"$_mn\"" "$_TATT"; then exit 1; fi
done
# The refusal count is exactly twelve, and all twelve refusal-reason names are present.
grep -qF 'pub const TRAIN_ATTEMPT_REFUSAL_COUNT: usize = 12;' "$_TATT"
for _rr in missing_training_gate_allow missing_explicit_operator_authorization \
           missing_training_run_config missing_curated_dataset_bundle missing_baseline_artifact \
           missing_holdout_bundle missing_rollback_artifact contaminated_dataset_refused \
           holdout_leakage_refused authority_drift_refused non_reproducible_config_refused \
           training_attempt_serialized_tamper_refused; do
  if ! grep -qF "\"$_rr\"" "$_TATT"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all twenty scenario names are present.
grep -qF 'pub const TRAIN_ATTEMPT_SCENARIO_COUNT: usize = 20;' "$_TATT"
for _as in dry_run_plan_created missing_training_gate_allow_denied \
           missing_operator_authorization_denied allowed_without_operator_authorization_denied \
           operator_authorization_without_allowed_gate_denied missing_run_config_denied \
           missing_dataset_bundle_denied missing_baseline_artifact_denied missing_holdout_bundle_denied \
           missing_rollback_artifact_denied contaminated_dataset_denied holdout_leakage_denied \
           authority_drift_denied non_reproducible_config_denied authorized_attempt_candidate_only \
           candidate_not_promoted candidate_not_deployed candidate_not_evidence \
           candidate_requires_s8_evaluation serialized_training_attempt_tamper_refused; do
  if ! grep -qF "\"$_as\"" "$_TATT"; then exit 1; fi
done
# It CONSUMES the real TRAIN-GATE-0 report (runs evaluate_training_gate; requires TrainingAttemptAllowed).
grep -qF 'evaluate_training_gate(' "$_TATT"
grep -qF 'TrainingGateDecision::TrainingAttemptAllowed' "$_TATT"
grep -qF 'TrainingGateReport' "$_TATT"
# TWO KEYS: a SEPARATE explicit operator authorization is required; neither key alone is sufficient.
grep -qF 'pub struct AttemptAuthorizationReceipt' "$_TATT"
grep -qF 'MissingExplicitOperatorAuthorization' "$_TATT"
grep -qF 'MissingTrainingGateAllow' "$_TATT"
# The candidate is CandidateOnly, must pass S8, and is never promoted/deployed/evidence/baseline-replacing.
grep -qF 'pub enum CandidateAcceptance' "$_TATT"
grep -qF 'CandidateOnly' "$_TATT"
grep -qF 'requires_s8_evaluation: true' "$_TATT"
grep -qF 'candidate_hash' "$_TATT"
# candidate_not_promoted / candidate_not_deployed / candidate_requires_s8_evaluation guards exist.
grep -qF 'candidate_not_promoted' "$_TATT"
grep -qF 'candidate_not_deployed' "$_TATT"
grep -qF 'candidate_not_evidence' "$_TATT"
grep -qF 'candidate_requires_s8_evaluation' "$_TATT"
# A prepared candidate is NOT an accepted model: every forbidden-action flag is sourced from the
# structural const (false); no path sets any true.
grep -qF 'const ATTEMPT_CREATES_ACCEPTED_MODEL: bool = false;' "$_TATT"
grep -qF 'promotes_model: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'deploys_model: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'replaces_baseline: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'modifies_weights: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'training_justified: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'promoted: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'deployed: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
grep -qF 'is_evidence: ATTEMPT_CREATES_ACCEPTED_MODEL' "$_TATT"
if grep -qE '(promotes_model|deploys_model|replaces_baseline|creates_truth|creates_memory|creates_evidence|grants_authority|modifies_weights|training_justified|promoted|deployed|is_evidence):[[:space:]]*true' "$_TATT"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares
# with a non-vacuous tamper guard (a no-op mutation cannot pass).
grep -qF 'Serialize' "$_TATT"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_TATT")" -eq 0
grep -qF 'tampered != canonical' "$_TATT"
# The harness tests assert the gate consumption, the dry-run plan, the two-key requirement (allow-alone
# and auth-alone both refused), the all-prerequisites CandidateOnly preparation, the no-promote/deploy/
# evidence guards, the twenty scenarios, and the serialized re-derivation.
for _at in 'fn attempt_consumes_the_real_train_gate_report' \
           'fn dry_run_builds_a_plan_without_touching_weights' \
           'fn authorized_attempt_requires_both_gate_allow_and_operator_authorization' \
           'fn allowed_gate_without_operator_authorization_is_refused' \
           'fn operator_authorization_without_allowed_gate_is_refused' \
           'fn authorized_attempt_with_all_prerequisites_prepares_a_candidate_only_artifact' \
           'fn candidate_is_not_promoted_deployed_or_evidence' \
           'fn matrix_has_the_twenty_named_scenarios' \
           'fn receipt_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_at" "$_TATT"; then exit 1; fi
done
# The nine-line TRAIN-0 boundary is recorded verbatim.
for _ab in 'The training attempt path may create a candidate model artifact only after gate approval and explicit operator authorization.' \
           'It does not promote models.' 'It does not deploy models.' 'It does not create truth.' \
           'It does not create memory.' 'It does not create evidence.' 'It does not grant new authority.' \
           'A candidate model is not an accepted model.' \
           'A candidate model must pass later evaluation before promotion.'; do
  if ! grep -qF "$_ab" "$_TATT"; then exit 1; fi
done
# TRAIN-0 makes NO false training/promotion claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_TATT"; then exit 1; fi
if grep -qE 'modifies_weights[[:space:]]*[=:][[:space:]]*true' "$_TATT"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# MODEL-EVAL-1 — the deterministic candidate-model ACCEPTANCE BATTERY. It CONSUMES a TRAIN-0
# TrainingCandidateArtifact (produced by the real run_training_attempt harness, evaluated here — never
# created), re-verifies the candidate is genuinely CandidateOnly and still requires_s8_evaluation,
# compares it against a pinned baseline across seven regression-guarded dimensions plus the target
# recurring clean failures, and runs holdout/contamination/memorization/adversarial/long-horizon/
# dry-run-production-smoke checks. Three verdicts (candidate_rejected, candidate_needs_more_evidence,
# candidate_ready_for_promotion_review) — NONE named accepted. Any critical regression or failed check
# rejects; clean-but-unimproved needs more evidence; only a clean improvement is ready for REVIEW.
# candidate_ready_for_promotion_review accepts/promotes/deploys nothing, replaces no baseline, creates no
# evidence/memory/authority, opens no production; every forbidden flag is sourced from
# READY_FOR_REVIEW_AUTHORIZES_PROMOTION = false; P12 stays training_justified = false. Boundary: The
# candidate evaluation path measures whether a candidate model artifact is ready for promotion review. It
# does not accept models. It does not promote models. It does not deploy models. It does not replace the
# baseline. It does not create truth. It does not create memory. It does not create evidence. It does not
# grant new authority.
# ---------------------------------------------------------------------------------------------------
_CEVAL=crates/cognitive-demo/src/candidate_eval.rs
test -f "$_CEVAL"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod candidate_eval;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use candidate_eval::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn evaluate_candidate(' "$_CEVAL"
grep -qF 'pub fn evaluate_candidate_json(' "$_CEVAL"
grep -qF 'pub fn verify_candidate_eval_report_json(' "$_CEVAL"
grep -qF 'pub fn candidate_eval_matrix(' "$_CEVAL"
grep -qF 'pub fn verify_candidate_eval_matrix_json(' "$_CEVAL"
# The core objects exist.
grep -qF 'pub struct CandidateEvalReport' "$_CEVAL"
grep -qF 'pub struct CandidateEvalBattery' "$_CEVAL"
grep -qF 'pub struct CandidateEvalInput' "$_CEVAL"
grep -qF 'pub enum CandidateEvalVerdict' "$_CEVAL"
grep -qF 'pub struct CandidateEvalComparison' "$_CEVAL"
grep -qF 'pub struct RegressionReport' "$_CEVAL"
grep -qF 'pub struct HoldoutReport' "$_CEVAL"
grep -qF 'pub struct SafetyBoundaryReport' "$_CEVAL"
grep -qF 'pub struct PromotionRecommendation' "$_CEVAL"
grep -qF 'pub struct CandidateResidualReport' "$_CEVAL"
grep -qF 'pub struct CandidateEvalMatrix' "$_CEVAL"
# Exactly three verdicts, all three names present, and NO verdict contains "accepted".
grep -qF 'pub const CANDIDATE_EVAL_VERDICT_COUNT: usize = 3;' "$_CEVAL"
for _vn in candidate_rejected candidate_needs_more_evidence candidate_ready_for_promotion_review; do
  if ! grep -qF "\"$_vn\"" "$_CEVAL"; then exit 1; fi
done
if grep -A4 'pub const CANDIDATE_EVAL_VERDICT_NAMES' "$_CEVAL" | grep -q 'accepted'; then exit 1; fi
# The rejection count is exactly eighteen, and all eighteen rejection-reason names are present.
grep -qF 'pub const CANDIDATE_EVAL_REJECTION_COUNT: usize = 18;' "$_CEVAL"
for _rr in missing_candidate not_candidate_only missing_s8_requirement missing_baseline missing_holdout \
           reading_regression grounding_regression curation_regression replay_regression \
           horizon_boundary_regression refusal_regression hallucination_regression holdout_contamination \
           memorization_leakage adversarial_prompt_failure long_horizon_failure \
           dry_run_production_smoke_failure serialized_candidate_eval_tamper_refused; do
  if ! grep -qF "\"$_rr\"" "$_CEVAL"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all twenty-three scenario names are present.
grep -qF 'pub const CANDIDATE_EVAL_SCENARIO_COUNT: usize = 23;' "$_CEVAL"
for _cs in missing_candidate_rejected non_candidate_only_rejected candidate_missing_s8_requirement_rejected \
           missing_baseline_rejected missing_holdout_rejected target_failure_improves_ready_for_review \
           no_target_improvement_needs_more_evidence reading_regression_rejected grounding_regression_rejected \
           curation_regression_rejected replay_regression_rejected horizon_boundary_regression_rejected \
           refusal_regression_rejected hallucination_regression_rejected holdout_contamination_rejected \
           memorization_leakage_rejected adversarial_prompt_failure_rejected long_horizon_failure_rejected \
           dry_run_production_smoke_failure_rejected ready_for_review_not_promotion \
           ready_for_review_not_deployment ready_for_review_not_baseline_replacement \
           serialized_candidate_eval_tamper_refused; do
  if ! grep -qF "\"$_cs\"" "$_CEVAL"; then exit 1; fi
done
# It CONSUMES the TRAIN-0 candidate (the real run_training_attempt artifact), and re-verifies it.
grep -qF 'run_training_attempt(' "$_CEVAL"
grep -qF 'TrainingCandidateArtifact' "$_CEVAL"
grep -qF 'CandidateAcceptance::CandidateOnly' "$_CEVAL"
grep -qF 'requires_s8_evaluation' "$_CEVAL"
# Baseline comparison + holdout check are required (their missing-cases reject).
grep -qF 'MissingBaseline' "$_CEVAL"
grep -qF 'MissingHoldout' "$_CEVAL"
grep -qF 'NotCandidateOnly' "$_CEVAL"
grep -qF 'MissingS8Requirement' "$_CEVAL"
# A critical regression rejects, and the seven regression dimensions are all enforced.
for _reg in ReadingRegression GroundingRegression CurationRegression ReplayRegression \
            HorizonBoundaryRegression RefusalRegression HallucinationRegression; do
  if ! grep -qF "$_reg" "$_CEVAL"; then exit 1; fi
done
# ready_for_review is NOT promotion/deployment/acceptance/baseline-replacement: every forbidden flag is
# sourced from the structural const (false); no path sets any true.
grep -qF 'const READY_FOR_REVIEW_AUTHORIZES_PROMOTION: bool = false;' "$_CEVAL"
grep -qF 'accepts_model: READY_FOR_REVIEW_AUTHORIZES_PROMOTION' "$_CEVAL"
grep -qF 'promotes_model: READY_FOR_REVIEW_AUTHORIZES_PROMOTION' "$_CEVAL"
grep -qF 'deploys_model: READY_FOR_REVIEW_AUTHORIZES_PROMOTION' "$_CEVAL"
grep -qF 'replaces_baseline: READY_FOR_REVIEW_AUTHORIZES_PROMOTION' "$_CEVAL"
grep -qF 'opens_production: READY_FOR_REVIEW_AUTHORIZES_PROMOTION' "$_CEVAL"
grep -qF 'training_justified: READY_FOR_REVIEW_AUTHORIZES_PROMOTION' "$_CEVAL"
if grep -qE '(accepts_model|promotes_model|deploys_model|replaces_baseline|creates_truth|creates_memory|creates_evidence|grants_authority|training_justified|opens_production):[[:space:]]*true' "$_CEVAL"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares
# with a non-vacuous tamper guard.
grep -qF 'Serialize' "$_CEVAL"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_CEVAL")" -eq 0
grep -qF 'tampered != canonical' "$_CEVAL"
# The battery tests assert the candidate consumption, the missing-candidate / non-candidate-only rejects,
# the target-improvement ready-for-review, the no-improvement needs-more-evidence, the critical-regression
# reject, the no-accepted-verdict rule, the twenty-three scenarios, and the serialized re-derivation.
for _ct in 'fn eval_consumes_a_real_train0_candidate' 'fn missing_candidate_is_rejected' \
           'fn non_candidate_only_is_rejected' \
           'fn target_improvement_is_ready_for_promotion_review' \
           'fn no_target_improvement_needs_more_evidence' \
           'fn critical_regression_rejects_even_with_target_improvement' \
           'fn ready_for_review_is_not_promotion_or_deployment' \
           'fn no_verdict_is_named_accepted' \
           'fn matrix_has_the_twenty_three_named_scenarios' \
           'fn report_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_ct" "$_CEVAL"; then exit 1; fi
done
# The nine-line MODEL-EVAL-1 boundary is recorded verbatim.
for _cb in 'The candidate evaluation path measures whether a candidate model artifact is ready for promotion review.' \
           'It does not accept models.' 'It does not promote models.' 'It does not deploy models.' \
           'It does not replace the baseline.' 'It does not create truth.' 'It does not create memory.' \
           'It does not create evidence.' 'It does not grant new authority.'; do
  if ! grep -qF "$_cb" "$_CEVAL"; then exit 1; fi
done
# MODEL-EVAL-1 makes NO false acceptance/promotion/training claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_CEVAL"; then exit 1; fi
if grep -qE 'accepts_model[[:space:]]*[=:][[:space:]]*true' "$_CEVAL"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# MODEL-PROMOTE-0 — the explicit, closed-by-default model PROMOTION GATE. It CONSUMES the real
# MODEL-EVAL-1 evaluation (running evaluate_candidate itself over the supplied CandidateEvalInput) and
# emits ModelPromotionDecision::PromotionReady ONLY when the verdict is exactly
# candidate_ready_for_promotion_review AND every requirement holds: pinned + corroborated candidate/
# baseline/dataset/eval-report hashes, an explicit operator promotion approval, a rollback artifact, a
# runtime config (baseline replacement only DESCRIBED as pending), a production safety plan, a clean
# holdout with no contamination/leakage/critical-regression, and an affirmative authority-drift check.
# PromotionReady is eligibility for S10 packaging / S11 smoke only — it trains nothing, deploys nothing,
# starts no production runtime, replaces no baseline; every production flag on the report and the sealed
# PromotedModelReceipt is sourced from PROMOTION_READY_IS_PRODUCTION = false; P12 stays
# training_justified = false. Boundary: The model promotion gate evaluates whether a candidate model is
# ready for promotion. It does not train. It does not deploy models. It does not start production runtime.
# It does not create truth. It does not create memory. It does not create evidence. It does not bypass
# rollback. PromotionReady is not production deployment.
# ---------------------------------------------------------------------------------------------------
_MPROMO=crates/cognitive-demo/src/model_promote.rs
test -f "$_MPROMO"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod model_promote;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use model_promote::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn evaluate_model_promotion(' "$_MPROMO"
grep -qF 'pub fn evaluate_model_promotion_json(' "$_MPROMO"
grep -qF 'pub fn verify_model_promotion_report_json(' "$_MPROMO"
grep -qF 'pub fn model_promotion_matrix(' "$_MPROMO"
grep -qF 'pub fn verify_model_promotion_matrix_json(' "$_MPROMO"
# The core objects exist.
grep -qF 'pub struct ModelPromotionReport' "$_MPROMO"
grep -qF 'pub struct ModelPromotionInput' "$_MPROMO"
grep -qF 'pub enum ModelPromotionDecision' "$_MPROMO"
grep -qF 'pub enum ModelPromotionRefusal' "$_MPROMO"
grep -qF 'pub struct PromotionCandidateReceipt' "$_MPROMO"
grep -qF 'pub struct PromotedModelReceipt' "$_MPROMO"
grep -qF 'pub struct PromotionOperatorApprovalReceipt' "$_MPROMO"
grep -qF 'pub struct PromotionRollbackReceipt' "$_MPROMO"
grep -qF 'pub struct PromotionRuntimeConfigReceipt' "$_MPROMO"
grep -qF 'pub struct PromotionEvalReceipt' "$_MPROMO"
grep -qF 'pub struct ModelPromotionMatrix' "$_MPROMO"
# Exactly two decisions, both names present.
grep -qF 'pub const MODEL_PROMOTE_DECISION_COUNT: usize = 2;' "$_MPROMO"
grep -qF 'PromotionDenied' "$_MPROMO"
grep -qF 'PromotionReady' "$_MPROMO"
for _dn in promotion_denied promotion_ready; do
  if ! grep -qF "\"$_dn\"" "$_MPROMO"; then exit 1; fi
done
# The refusal count is exactly sixteen, and all sixteen refusal-reason names are present.
grep -qF 'pub const MODEL_PROMOTE_REFUSAL_COUNT: usize = 16;' "$_MPROMO"
for _rr in missing_candidate_eval_report candidate_not_ready_for_promotion_review \
           missing_candidate_artifact_hash missing_baseline_artifact_hash missing_dataset_hash \
           missing_eval_report_hash missing_runtime_config missing_rollback_artifact \
           missing_operator_approval missing_production_safety_plan holdout_not_clean \
           contamination_detected memorization_leakage_detected critical_regression_present \
           authority_drift_detected serialized_promotion_report_tamper_refused; do
  if ! grep -qF "\"$_rr\"" "$_MPROMO"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all twenty-two scenario names are present.
grep -qF 'pub const MODEL_PROMOTE_SCENARIO_COUNT: usize = 22;' "$_MPROMO"
for _ps in missing_candidate_eval_report_denied candidate_rejected_denied \
           candidate_needs_more_evidence_denied ready_without_candidate_hash_denied \
           ready_without_baseline_hash_denied ready_without_dataset_hash_denied \
           ready_without_eval_hash_denied ready_without_runtime_config_denied \
           ready_without_rollback_denied ready_without_operator_approval_denied \
           ready_without_production_safety_plan_denied holdout_not_clean_denied \
           contamination_detected_denied memorization_leakage_denied critical_regression_denied \
           authority_drift_denied all_requirements_met_promotion_ready promotion_ready_not_deployment \
           promotion_ready_not_training promotion_ready_not_baseline_replacement \
           promotion_ready_requires_s10_s11 serialized_promotion_report_tamper_refused; do
  if ! grep -qF "\"$_ps\"" "$_MPROMO"; then exit 1; fi
done
# It CONSUMES the MODEL-EVAL-1 report (runs evaluate_candidate; requires the ready verdict).
grep -qF 'evaluate_candidate(' "$_MPROMO"
grep -qF 'CandidateEvalReport' "$_MPROMO"
grep -qF 'CandidateEvalVerdict::CandidateReadyForPromotionReview' "$_MPROMO"
# Every promotion requirement is enforced (hashes + receipts + safety re-checks + drift).
grep -qF 'MissingCandidateArtifactHash' "$_MPROMO"
grep -qF 'MissingBaselineArtifactHash' "$_MPROMO"
grep -qF 'MissingDatasetHash' "$_MPROMO"
grep -qF 'MissingEvalReportHash' "$_MPROMO"
grep -qF 'MissingRuntimeConfig' "$_MPROMO"
grep -qF 'MissingRollbackArtifact' "$_MPROMO"
grep -qF 'MissingOperatorApproval' "$_MPROMO"
grep -qF 'MissingProductionSafetyPlan' "$_MPROMO"
grep -qF 'HoldoutNotClean' "$_MPROMO"
grep -qF 'ContaminationDetected' "$_MPROMO"
grep -qF 'MemorizationLeakageDetected' "$_MPROMO"
grep -qF 'CriticalRegressionPresent' "$_MPROMO"
grep -qF 'AuthorityDriftDetected' "$_MPROMO"
grep -qF 'is_clean()' "$_MPROMO"
# PromotionReady is NOT deployment / baseline-replacement, and still requires S10/S11: every forbidden
# flag is sourced from the structural const (false); no path sets any true.
grep -qF 'const PROMOTION_READY_IS_PRODUCTION: bool = false;' "$_MPROMO"
grep -qF 'deploys_model: PROMOTION_READY_IS_PRODUCTION' "$_MPROMO"
grep -qF 'starts_production: PROMOTION_READY_IS_PRODUCTION' "$_MPROMO"
grep -qF 'replaces_baseline: PROMOTION_READY_IS_PRODUCTION' "$_MPROMO"
grep -qF 'trains: PROMOTION_READY_IS_PRODUCTION' "$_MPROMO"
grep -qF 'modifies_weights: PROMOTION_READY_IS_PRODUCTION' "$_MPROMO"
grep -qF 'opens_p12: PROMOTION_READY_IS_PRODUCTION' "$_MPROMO"
grep -qF 'requires_s10_packaging: true' "$_MPROMO"
grep -qF 'requires_s11_smoke: true' "$_MPROMO"
grep -qF 'baseline_replacement_pending: true' "$_MPROMO"
if grep -qE '(deploys_model|starts_production|replaces_baseline|trains|modifies_weights|creates_truth|creates_memory|creates_evidence|grants_authority|opens_p12|training_justified|bypasses_rollback):[[:space:]]*true' "$_MPROMO"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares.
grep -qF 'Serialize' "$_MPROMO"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_MPROMO")" -eq 0
grep -qF 'tampered != canonical' "$_MPROMO"
# The gate tests assert the eval consumption, the missing-eval / needs-more-evidence denials, the
# missing-operator-approval denial, the all-met promotion-ready, the not-deployment / requires-s10s11
# safety, the critical-regression denial, the twenty-two scenarios, and the serialized re-derivation.
for _mt in 'fn gate_consumes_the_real_candidate_eval_report' \
           'fn missing_candidate_eval_report_is_denied' \
           'fn candidate_needs_more_evidence_is_denied' \
           'fn ready_without_each_receipt_is_denied' \
           'fn all_requirements_met_is_promotion_ready' \
           'fn promotion_ready_is_not_deployment_or_training' \
           'fn promotion_ready_requires_s10_s11' \
           'fn critical_regression_is_denied' \
           'fn matrix_has_the_twenty_two_named_scenarios' \
           'fn report_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_mt" "$_MPROMO"; then exit 1; fi
done
# The nine-line MODEL-PROMOTE-0 boundary is recorded verbatim.
for _mb in 'The model promotion gate evaluates whether a candidate model is ready for promotion.' \
           'It does not train.' 'It does not deploy models.' 'It does not start production runtime.' \
           'It does not create truth.' 'It does not create memory.' 'It does not create evidence.' \
           'It does not bypass rollback.' 'PromotionReady is not production deployment.'; do
  if ! grep -qF "$_mb" "$_MPROMO"; then exit 1; fi
done
# MODEL-PROMOTE-0 makes NO false training/deployment claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_MPROMO"; then exit 1; fi
if grep -qE 'starts_production[[:space:]]*[=:][[:space:]]*true' "$_MPROMO"; then exit 1; fi

# ---------------------------------------------------------------------------------------------------
# PROD-0 — the deterministic, local PRODUCTION RUNTIME PACKAGE. It CONSUMES the real MODEL-PROMOTE-0
# evaluation (running evaluate_model_promotion itself for a model mode) and packages a complete, pinned,
# reversible, no-training, offline runtime artifact that is SMOKE-READY — NOT live production. The
# local_promoted_ready_runtime mode requires ModelPromotionDecision::PromotionReady; local_no_model_runtime
# packages the substrate runtime; local_candidate_ready_runtime packages an evaluated candidate without
# requiring promotion-ready. No-training is the only representable training state; an enabled training mode
# or network is refused. A packaged runtime deploys nothing, starts no service, claims no production,
# serves no traffic, replaces no baseline; every forbidden flag on the package and the sealed
# ProductionRuntimeReceipt is sourced from PACKAGE_IS_PRODUCTION = false, it requires_s11_smoke, and P12
# stays training_justified = false. Boundary: The production runtime package prepares a local runtime
# artifact. It does not train. It does not mutate weights. It does not deploy models. It does not start
# production service. It does not replace the baseline. It does not create truth, memory, or evidence. It
# does not grant new authority. ProductionRuntimePackage is not production smoke.
# ---------------------------------------------------------------------------------------------------
_PROD=crates/cognitive-demo/src/production_runtime.rs
test -f "$_PROD"
# The module is wired into the crate and its public entrypoints exist.
grep -qF 'mod production_runtime;' crates/cognitive-demo/src/lib.rs
grep -qF 'pub use production_runtime::' crates/cognitive-demo/src/lib.rs
grep -qF 'pub fn package_production_runtime(' "$_PROD"
grep -qF 'pub fn package_production_runtime_json(' "$_PROD"
grep -qF 'pub fn verify_production_runtime_package_json(' "$_PROD"
grep -qF 'pub fn production_runtime_matrix(' "$_PROD"
grep -qF 'pub fn verify_production_runtime_matrix_json(' "$_PROD"
# The operator runbook exists (a required precondition; missing_operator_runbook refuses without it).
test -f docs/PRODUCTION_RUNTIME_RUNBOOK.md
# The core objects exist.
grep -qF 'pub struct ProductionRuntimePackage' "$_PROD"
grep -qF 'pub struct ProductionRuntimeConfig' "$_PROD"
grep -qF 'pub struct ProductionRuntimeReceipt' "$_PROD"
grep -qF 'pub struct ProductionRuntimeManifest' "$_PROD"
grep -qF 'pub enum ProductionRuntimeMode' "$_PROD"
grep -qF 'pub enum ProductionRuntimeRefusal' "$_PROD"
grep -qF 'pub struct ProductionRuntimeBoundary' "$_PROD"
grep -qF 'pub struct RuntimeVersionReceipt' "$_PROD"
grep -qF 'pub struct RuntimeRollbackReceipt' "$_PROD"
grep -qF 'pub struct RuntimeModelSlot' "$_PROD"
grep -qF 'pub enum RuntimeNoTrainingMode' "$_PROD"
grep -qF 'pub struct ProductionRuntimeMatrix' "$_PROD"
# Exactly three runtime modes, all three names present.
grep -qF 'pub const PROD_RUNTIME_MODE_COUNT: usize = 3;' "$_PROD"
for _mn in local_no_model_runtime local_candidate_ready_runtime local_promoted_ready_runtime; do
  if ! grep -qF "\"$_mn\"" "$_PROD"; then exit 1; fi
done
# The refusal count is exactly fourteen, and all fourteen refusal-reason names are present.
grep -qF 'pub const PROD_RUNTIME_REFUSAL_COUNT: usize = 14;' "$_PROD"
for _rr in missing_runtime_config missing_promotion_report promotion_not_ready \
           missing_model_artifact_hash missing_baseline_hash missing_rollback_artifact \
           missing_version_receipt missing_operator_runbook training_mode_enabled \
           unauthorized_network_enabled missing_receipt_output_path missing_replay_output_path \
           authority_drift_detected serialized_runtime_package_tamper_refused; do
  if ! grep -qF "\"$_rr\"" "$_PROD"; then exit 1; fi
done
# The scenario count comes from the observed matrix, and all twenty scenario names are present.
grep -qF 'pub const PROD_RUNTIME_SCENARIO_COUNT: usize = 20;' "$_PROD"
for _ps in local_no_model_runtime_packaged missing_runtime_config_refused \
           missing_promotion_report_refused promotion_not_ready_refused \
           missing_model_artifact_hash_refused missing_baseline_hash_refused \
           missing_rollback_artifact_refused missing_version_receipt_refused \
           missing_operator_runbook_refused training_mode_enabled_refused \
           unauthorized_network_enabled_refused missing_receipt_output_path_refused \
           missing_replay_output_path_refused authority_drift_refused promoted_ready_runtime_packaged \
           package_is_not_deployment package_is_not_service_start package_is_not_baseline_replacement \
           package_requires_s11_smoke serialized_runtime_package_tamper_refused; do
  if ! grep -qF "\"$_ps\"" "$_PROD"; then exit 1; fi
done
# It CONSUMES the MODEL-PROMOTE-0 report (runs evaluate_model_promotion; promoted-ready requires ready).
grep -qF 'evaluate_model_promotion(' "$_PROD"
grep -qF 'ModelPromotionDecision::PromotionReady' "$_PROD"
# No-training is the default + only state; training mode + network are refused; rollback + output paths
# + S11 smoke are required.
grep -qF 'RuntimeNoTrainingMode::NoTraining' "$_PROD"
grep -qF 'TrainingModeEnabled' "$_PROD"
grep -qF 'UnauthorizedNetworkEnabled' "$_PROD"
grep -qF 'MissingRollbackArtifact' "$_PROD"
grep -qF 'MissingReceiptOutputPath' "$_PROD"
grep -qF 'MissingReplayOutputPath' "$_PROD"
grep -qF 'requires_s11_smoke: true' "$_PROD"
# A packaged runtime is NOT production: every forbidden flag is sourced from the structural const (false);
# no path sets any true.
grep -qF 'const PACKAGE_IS_PRODUCTION: bool = false;' "$_PROD"
grep -qF 'deploys_model: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'starts_production_service: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'replaces_baseline: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'claims_production: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'serves_traffic: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'trains: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'mutates_weights: PACKAGE_IS_PRODUCTION' "$_PROD"
grep -qF 'opens_p12: PACKAGE_IS_PRODUCTION' "$_PROD"
if grep -qE '(deploys_model|starts_production_service|replaces_baseline|trains|mutates_weights|creates_truth|creates_memory|creates_evidence|grants_authority|opens_p12|training_justified|claims_production|serves_traffic):[[:space:]]*true' "$_PROD"; then exit 1; fi
# Re-derived, never trusted: Serialize but NEVER derived Deserialize; verify re-derives + byte-compares.
grep -qF 'Serialize' "$_PROD"
test "$(grep -cE 'derive\([^)]*Deserialize' "$_PROD")" -eq 0
grep -qF 'tampered != canonical' "$_PROD"
# The packager tests assert the promotion consumption, the no-model packaging, the promoted-ready
# requirement, the training/network refusals, the not-deployment / requires-s11 safety, the twenty
# scenarios, and the serialized re-derivation.
for _pt in 'fn package_consumes_the_real_promotion_report' \
           'fn local_no_model_runtime_is_packaged' \
           'fn promoted_ready_runtime_requires_promotion_ready' \
           'fn training_mode_and_network_are_refused' \
           'fn packaged_runtime_is_not_deployment_or_service' \
           'fn packaged_runtime_requires_s11_smoke' \
           'fn no_training_mode_is_the_default_and_only_state' \
           'fn matrix_has_the_twenty_named_scenarios' \
           'fn report_is_deterministic_and_re_derives_refusing_tampering'; do
  if ! grep -qF "$_pt" "$_PROD"; then exit 1; fi
done
# The nine-line PROD-0 boundary is recorded verbatim.
for _pb in 'The production runtime package prepares a local runtime artifact.' \
           'It does not train.' 'It does not mutate weights.' 'It does not deploy models.' \
           'It does not start production service.' 'It does not replace the baseline.' \
           'It does not create truth, memory, or evidence.' 'It does not grant new authority.' \
           'ProductionRuntimePackage is not production smoke.'; do
  if ! grep -qF "$_pb" "$_PROD"; then exit 1; fi
done
# PROD-0 makes NO false training/production claim in its source.
if grep -qE 'training_justified[[:space:]]*[=:][[:space:]]*true' "$_PROD"; then exit 1; fi
if grep -qE 'claims_production[[:space:]]*[=:][[:space:]]*true' "$_PROD"; then exit 1; fi
