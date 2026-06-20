#!/usr/bin/env sh
# operator_smoke.sh — OPS-1 + DOCFLOW-1: Operator Smoke Script / Manual Drift Guard.
#
# Runs the documented operator path end-to-end against the built cognitive-demo binary and proves the
# OPERATOR_MANUAL.md has NOT drifted from the binary: every documented command still runs, every generated
# artifact re-derives byte-identically through the binary's OWN verify subcommands (never trusted from its
# bytes), a tampered artifact is still refused, and the boundary lines the manual leads an operator to
# expect are still emitted verbatim by the binary AND recorded verbatim in the manual.
#
# DOCFLOW-1 extends the same guard to the operator-supplied-document path: §10 below runs the documented
# doc flow (doc-trace --input/--out, doc-report, doc-bundle, doc-bundle-verify) against a LOCAL sample
# document (under the gitignored target/ dir, inside the working dir — the doc commands only read local
# paths), proves the trace started from the document's OWN verified read, and proves a tampered document,
# trace, report, or manifest is still refused. The local document is READ, never TRUSTED.
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
# DOCFLOW-1: the doc-flow commands only read paths INSIDE the working dir, so the operator-document sample
# lives under target/ (gitignored, inside cwd) and is referenced by a RELATIVE path. Both temp dirs are
# removed on exit (no repo debris).
docwork="$(mktemp -d "$PWD/target/.docflow_smoke.XXXXXX")"
docrel="target/$(basename "$docwork")"
trap 'rm -rf "$work" "$docwork"' EXIT

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
# The manual still documents every command this smoke exercised (manual surface == binary surface),
# including the DOCFLOW-1 operator-document commands.
for _cmd in 'trace --out' 'report --trace' 'replay --trace' 'questions' 'ask --trace' 'bundle --out' \
            'bundle-verify --path' 'scenario-pack --out' 'scenario-verify --path' 'scenario-matrix --pack' \
            'scenario-matrix-report --matrix' 'scenario-matrix-verify --pack' 'failure-pack --out' \
            'failure-verify --path' \
            'doc-trace --input' 'doc-report --input' 'doc-bundle --input' 'doc-bundle-verify --input'; do
  grep -qF "$_cmd" "$MANUAL" || fail "manual no longer documents: $_cmd"
done
# The manual states the document flow reads local input but does NOT trust it (DOCFLOW-1 doctrine).
grep -qF 'read but not trusted' "$MANUAL" || fail 'manual no longer states local input is read but not trusted'
# The manual records the DOCFLOW-1 document-operator-path boundary verbatim (all six lines).
for _dbl in 'The document operator path explains and verifies local-document tracing.' \
            'It does not trust local input.' 'It does not create authority.' 'It does not execute.' \
            'It does not promote.' 'It does not train.'; do
  grep -qF "$_dbl" "$MANUAL" || fail "manual DOCFLOW boundary line drifted: $_dbl"
done
# The manual still records training as closed; this smoke asserts that and never opens training.
grep -qF 'training_justified=false' "$MANUAL" || fail 'manual no longer records training_justified=false'

# ---- 10. DOCFLOW operator path: run the doc flow from a LOCAL operator-supplied document ----
# DOCFLOW-1 boundary (recorded verbatim):
#   The document operator path explains and verifies local-document tracing.
#   It does not trust local input.
#   It does not create authority.
#   It does not execute.
#   It does not promote.
#   It does not train.
# The doc commands only read paths INSIDE the working dir, so the sample lives under target/ (relative path).
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$docwork/doc.txt"
# doc-trace --input --out: read the local doc, verify it, trace it. The trace carries the document's OWN
# verified read and the boundary markers (verified receipt, requires_operator, rejected, no evidence,
# training false) — proof the operator's text was read, NOT trusted as authority.
$BIN doc-trace --input "$docrel/doc.txt" --out "$docwork/trace.json"
for _dm in '"starts_from_verified_receipt": true' '"reading_passed": true' '"nothing_executed": true' \
           '"promotion_refused": true' '"nothing_becomes_evidence": true' \
           '"execution_status": "requires_operator"' '"promotion_status": "rejected"' \
           '"training_justified": false'; do
  grep -qF "$_dm" "$docwork/trace.json" || fail "doc-trace marker missing: $_dm"
done
# The trace really read the OPERATOR's text (answer == the document's own first span), not the canonical corpus.
grep -qF '"reading_answer": "The east bridge reopened today."' "$docwork/trace.json" \
  || fail 'doc-trace did not read the operator document'
# No affirmative-authority status leaked into the doc trace.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$docwork/trace.json"; then
  fail 'doc trace claims an executed/recorded/promoted/granted status'
fi
# doc-report re-derives from the SAME input + trace and renders the 9-line trace boundary.
$BIN doc-report --input "$docrel/doc.txt" --trace "$docwork/trace.json" --out "$docwork/report.txt"
grep -qF 'Nothing trains.' "$docwork/report.txt" || fail 'doc-report boundary line drifted'
# doc-bundle + doc-bundle-verify (clean) re-derive byte-identically and print the DOCFLOW boundary.
$BIN doc-bundle --input "$docrel/doc.txt" --out "$docwork/pack"
out="$($BIN doc-bundle-verify --input "$docrel/doc.txt" --path "$docwork/pack")"
case "$out" in *'doc-bundle-verify: OK'*) : ;; *) fail 'doc-bundle-verify did not pass on a clean bundle' ;; esac
case "$out" in *'The document flow reads local input.'*) : ;; *) fail 'doc-bundle-verify did not emit the DOCFLOW boundary' ;; esac
# RE-DERIVE IS LOAD-BEARING over operator input — every tamper must be refused (never trusted from bytes):
# (a) a tampered DOCUMENT (different text -> different trace) is refused.
printf 'The west bridge collapsed today. Traffic stopped by noon.' > "$docwork/doc2.txt"
if $BIN doc-bundle-verify --input "$docrel/doc2.txt" --path "$docwork/pack" >/dev/null 2>&1; then
  fail 'doc-bundle-verify accepted a tampered document'
fi
# (b) a tampered BUNDLE FILE (trace / report / questions / manifest) is refused — each file re-derives.
for _bf in trace.json report.txt questions.txt manifest.json; do
  cp -r "$docwork/pack" "$docwork/pack_t"
  printf '\n{tampered}' >> "$docwork/pack_t/$_bf"
  if $BIN doc-bundle-verify --input "$docrel/doc.txt" --path "$docwork/pack_t" >/dev/null 2>&1; then
    fail "doc-bundle-verify accepted a tampered $_bf"
  fi
  rm -rf "$docwork/pack_t"
done
# (c) a tampered standalone TRACE is refused by doc-report.
printf '\n{tampered}' >> "$docwork/trace.json"
if $BIN doc-report --input "$docrel/doc.txt" --trace "$docwork/trace.json" >/dev/null 2>&1; then
  fail 'doc-report accepted a tampered trace'
fi

echo 'operator-smoke: OK — the documented operator path runs and the manual matches the binary'
