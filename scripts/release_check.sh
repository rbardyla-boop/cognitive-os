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
