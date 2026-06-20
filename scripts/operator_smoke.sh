#!/usr/bin/env sh
# operator_smoke.sh — OPS-1: Operator Smoke Script / Manual Drift Guard.
#
# Runs the documented operator path end-to-end against the built cognitive-demo binary and proves the
# OPERATOR_MANUAL.md has NOT drifted from the binary: every documented command still runs, every generated
# artifact re-derives byte-identically through the binary's OWN verify subcommands (never trusted from its
# bytes), a tampered artifact is still refused, and the boundary lines the manual leads an operator to
# expect are still emitted verbatim by the binary AND recorded verbatim in the manual.
#
# Deterministic, offline, temp-dir only (no repo debris), fail-closed: `set -e` means any documented
# command exiting non-zero aborts this script non-zero — command failures are never swallowed. The only
# redirections are on the negative (expected-to-fail) tamper checks, which ASSERT a refusal happens.
#
# OPS-1 boundary (recorded verbatim):
#   The smoke test verifies the operator path.
#   It does not create authority.
#   It does not execute.
#   It does not promote.
#   It does not train.
#
# This script RUNS the operator path; it MINTS nothing. It writes only under a temp dir, removed on exit.
set -eu
cd "$(dirname "$0")/.."

BIN=./target/debug/cognitive-demo
MANUAL=OPERATOR_MANUAL.md

fail() { echo "operator-smoke: DRIFT — $1" >&2; exit 1; }

# Build the operator binary (offline, deterministic). Self-contained so an operator can run this directly.
cargo build --offline --quiet --manifest-path crates/cognitive-demo/Cargo.toml --bin cognitive-demo

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# ---- 1. canonical trace — ALWAYS --out (exact replayable bytes), NEVER a shell redirect ----
$BIN trace --out "$work/trace.json"
# the trace carries the real canonical, boundary-preserving markers (not a stub)
for _m in '"training_justified": false' '"grants_promotion": false' \
          '"execution_status": "requires_operator"' '"promotion_status": "rejected"'; do
  grep -qF "$_m" "$work/trace.json" || fail "trace marker missing: $_m"
done

# ---- 2. inspect: report (all 7 stages, 9 boundary lines) + replay (byte-identical re-derive) ----
$BIN report --trace "$work/trace.json" --out "$work/report.txt"
for _stage in '[1] READING' '[2] HYPOTHESIS' '[3] PROBE QUEUE' '[4] GOVERNANCE REVIEW' \
              '[5] EXECUTION INTENT' '[6] OBSERVATION' '[7] PROMOTION REQUEST'; do
  grep -qF "$_stage" "$work/report.txt" || fail "report stage missing: $_stage"
done
# BINARY boundary drift guard: the nine boundary lines the binary emits must be present verbatim.
for _bl in 'Reading verifies.' 'Hypothesis proposes.' 'Probe queue classifies.' 'Governance reviews.' \
           'Execution intent records.' 'Observation quarantines.' 'Promotion refuses.' \
           'Nothing becomes evidence.' 'Nothing trains.'; do
  grep -qF "$_bl" "$work/report.txt" || fail "binary report boundary line drifted: $_bl"
done
# The report still refuses affirmative authority (never claims an executed/promoted/granted status).
if grep -qE '(executed|promoted|granted)$' "$work/report.txt"; then fail 'report claims executed/promoted/granted'; fi
out="$($BIN replay --trace "$work/trace.json")"
case "$out" in *'replay: OK'*) : ;; *) fail 'replay did not confirm the canonical trace' ;; esac

# ---- 3. questions + ask (finite, enumerated interrogation surface) ----
out="$($BIN questions)"
case "$out" in *'was-anything-executed'*) : ;; *) fail 'questions did not list the audit slugs' ;; esac
out="$($BIN ask --trace "$work/trace.json" --question was-anything-executed)"
case "$out" in *'No.'*) : ;; *) fail 'ask was-anything-executed did not answer No' ;; esac

# ---- 4. bundle + bundle-verify (re-derive, never trust the files) ----
$BIN bundle --out "$work/pack"
out="$($BIN bundle-verify --path "$work/pack")"
case "$out" in *'bundle-verify: OK'*) : ;; *) fail 'bundle-verify did not pass' ;; esac

# ---- 5. scenario-pack + scenario-verify ----
$BIN scenario-pack --out "$work/scn"
out="$($BIN scenario-verify --path "$work/scn")"
case "$out" in *'scenario-verify: OK'*) : ;; *) fail 'scenario-verify did not pass' ;; esac

# ---- 6. scenario-matrix + scenario-matrix-report + scenario-matrix-verify ----
$BIN scenario-matrix --pack "$work/scn" --out "$work/matrix.json"
# The rendered matrix report is CONTENT-validated (not trusted on exit code alone): the header, the full
# 16/16 coverage proof, the all-boundaries-hold verdict, and the matrix boundary must all be present.
out="$($BIN scenario-matrix-report --matrix "$work/matrix.json")"
case "$out" in *'SCENARIO BOUNDARY COVERAGE MATRIX'*) : ;; *) fail 'scenario-matrix-report missing the coverage header' ;; esac
case "$out" in *'16/16'*) : ;; *) fail 'scenario-matrix-report did not prove 16/16 cells' ;; esac
case "$out" in *'all_boundaries_hold: true'*) : ;; *) fail 'scenario-matrix-report did not confirm all boundaries hold' ;; esac
case "$out" in *'It does not execute.'*) : ;; *) fail 'scenario-matrix-report missing its boundary' ;; esac
out="$($BIN scenario-matrix-verify --pack "$work/scn" --matrix "$work/matrix.json")"
case "$out" in *'scenario-matrix-verify: OK'*) : ;; *) fail 'scenario-matrix-verify did not pass' ;; esac

# ---- 7. failure-pack + failure-verify (forged authority stays rejected) ----
$BIN failure-pack --out "$work/fp"
out="$($BIN failure-verify --path "$work/fp")"
case "$out" in *'failure-verify: OK'*) : ;; *) fail 'failure-verify did not pass' ;; esac

# ---- 8. re-derive is LOAD-BEARING: tamper must be refused (a failed boundary is not hideable) ----
# Tamper the trace's promotion bit; replay AND report must REFUSE it (non-zero). We tamper grants_promotion,
# never the training bit, so this script never writes a "training justified" literal.
sed 's/"grants_promotion": false/"grants_promotion": true/' "$work/trace.json" > "$work/tampered.json"
if $BIN replay --trace "$work/tampered.json" >/dev/null 2>&1; then fail 'replay accepted a tampered trace'; fi
if $BIN report --trace "$work/tampered.json" >/dev/null 2>&1; then fail 'report accepted a tampered trace'; fi
# Tamper a bundle file; bundle-verify must REFUSE the whole pack (re-derive catches it).
sed 's/"grants_promotion": false/"grants_promotion": true/' "$work/pack/trace.json" > "$work/pack_tmp"
mv "$work/pack_tmp" "$work/pack/trace.json"
if $BIN bundle-verify --path "$work/pack" >/dev/null 2>&1; then fail 'bundle-verify accepted a tampered bundle'; fi

# ---- 9. MANUAL drift guard: the manual the operator reads must match the binary it describes ----
# The manual still records its six-line boundary verbatim.
for _ml in 'The manual explains the prototype.' 'It does not expand the prototype.' \
           'It does not create authority.' 'It does not execute.' 'It does not promote.' 'It does not train.'; do
  grep -qF "$_ml" "$MANUAL" || fail "manual boundary line drifted: $_ml"
done
# The manual still documents every command this smoke exercised (manual surface == binary surface).
for _cmd in 'trace --out' 'report --trace' 'replay --trace' 'questions' 'ask --trace' 'bundle --out' \
            'bundle-verify --path' 'scenario-pack --out' 'scenario-verify --path' 'scenario-matrix --pack' \
            'scenario-matrix-report --matrix' 'scenario-matrix-verify --pack' 'failure-pack --out' \
            'failure-verify --path'; do
  grep -qF "$_cmd" "$MANUAL" || fail "manual no longer documents: $_cmd"
done
# The manual still records training as closed; this smoke asserts that and never opens training.
grep -qF 'training_justified=false' "$MANUAL" || fail 'manual no longer records training_justified=false'

echo 'operator-smoke: OK — the documented operator path runs and the manual matches the binary'
