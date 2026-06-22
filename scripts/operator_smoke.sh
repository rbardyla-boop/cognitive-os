#!/usr/bin/env sh
# operator_smoke.sh — OPS-1 + DOCFLOW-1 + CORPUS-1: Operator Smoke Script / Manual Drift Guard.
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
# CORPUS-1 extends the same guard to the multi-document corpus path: §11 below runs the documented corpus
# flow (corpus-trace --input-dir/--out, corpus-report, corpus-bundle, corpus-bundle-verify) against a LOCAL
# directory of .txt documents (under the gitignored target/ dir), proves the directory filter matches CORPUS-0
# (hidden / non-.txt / symlink-escape refused), proves the trace started from the corpus's OWN verified first
# span, and proves that mutating the grounding document OR a non-grounding SIDE document — and tampering the
# trace, report, or manifest — is refused. The corpus is hash-bound as a whole; the documents are READ, never
# TRUSTED. No code crate changes — this sprint documents and smoke-tests existing CORPUS-0 behavior only.
#
# NOVELTY-1 extends the same guard to the hypothesis-only novelty path: §12 below runs the documented novelty
# flow (corpus-trace --out FIRST, then novelty-packet --input-dir/--corpus-trace/--frame/--out, novelty-report,
# novelty-replay) against a LOCAL corpus + frame (under the gitignored target/ dir), proves the packet's
# authority is hypothesis_only with every probe request non-executing, proves the only grounded content is the
# VERIFIED corpus span (the operator frame is recorded, never a preserved fact), and proves every refusal
# end-to-end: an empty frame, an UNSUPPORTED preserved fact (the frame's own claim swapped in), a tampered
# packet, and a receipt-hash-stripped corpus trace are each refused. Novelty packets PROPOSE but do not PROVE;
# the frame and corpus are READ, never TRUSTED. No code crate changes — this sprint documents and smoke-tests
# existing NOVELTY-0 behavior only.
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
# CORPUS-1: the corpus-flow commands only read a directory INSIDE the working dir, so the operator-corpus
# sample lives under target/ (gitignored, inside cwd) and is referenced by a RELATIVE path.
corpuswork="$(mktemp -d "$PWD/target/.corpus_smoke.XXXXXX")"
corpusrel="target/$(basename "$corpuswork")"
# NOVELTY-1: the novelty-flow commands only read a corpus directory and a frame file INSIDE the working dir,
# so the operator-novelty sample lives under target/ (gitignored, inside cwd) and uses a RELATIVE path.
noveltywork="$(mktemp -d "$PWD/target/.novelty_smoke.XXXXXX")"
noveltyrel="target/$(basename "$noveltywork")"
# DREAM-EXPORT-1: the dream-export-flow commands only read a corpus directory and a frame file INSIDE the working
# dir, so the operator-dream sample lives under target/ (gitignored, inside cwd) and uses a RELATIVE path.
dreamwork="$(mktemp -d "$PWD/target/.dream_smoke.XXXXXX")"
dreamrel="target/$(basename "$dreamwork")"
# DATA-1: the curation smoke runs the real curator over in-memory candidate manifests via its tests; the temp
# dir holds only an operator-readable illustration of those manifests (the curator itself does no filesystem IO).
curatework="$(mktemp -d "$PWD/target/.curate_smoke.XXXXXX")"
trap 'rm -rf "$work" "$docwork" "$corpuswork" "$noveltywork" "$dreamwork" "$curatework"' EXIT

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
            'doc-trace --input' 'doc-report --input' 'doc-bundle --input' 'doc-bundle-verify --input' \
            'corpus-trace --input-dir' 'corpus-report --input-dir' 'corpus-bundle --input-dir' \
            'corpus-bundle-verify --input-dir'; do
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
# The manual states the corpus is read but not trusted and hash-bound as a whole (CORPUS-1 doctrine), and
# records the CORPUS-1 nine-line corpus-operator-path boundary verbatim.
grep -qF 'read but not trusted' "$MANUAL" || fail 'manual no longer states the corpus is read but not trusted'
grep -qF 'hash-bound as a whole' "$MANUAL" || fail 'manual no longer states the whole corpus is hash-bound'
for _cbl in 'The corpus operator path reads local documents.' 'It does not trust local documents.' \
            'Source selection is verified and replayable.' 'The whole corpus is hash-bound.' \
            'Verification comes before tracing.' 'Nothing executes.' 'Nothing becomes evidence.' \
            'Nothing promotes.' 'Nothing trains.'; do
  grep -qF "$_cbl" "$MANUAL" || fail "manual CORPUS boundary line drifted: $_cbl"
done
# The manual still records training as closed; this smoke asserts that and never opens training.
grep -qF 'training_justified=false' "$MANUAL" || fail 'manual no longer records training_justified=false'
# The manual documents the three NOVELTY-0 operator commands (manual surface == binary surface) and states the
# novelty doctrine: packets propose but do not prove; the operator frame is recorded but never grounded as
# fact; preserved facts come only from verified corpus spans; a packet can never become evidence, a promotion,
# or training. It records the NOVELTY-1 eight-line novelty-operator-path boundary verbatim.
for _nc in 'novelty-packet --input-dir' 'novelty-report --input-dir' 'novelty-replay --input-dir'; do
  grep -qF "$_nc" "$MANUAL" || fail "manual no longer documents: $_nc"
done
for _ns in 'propose but do not prove' 'never grounded as fact' 'come only from verified corpus spans' \
           'can never become evidence, a promotion, or training'; do
  grep -qF "$_ns" "$MANUAL" || fail "manual novelty doctrine line drifted: $_ns"
done
for _nbl in 'The novelty operator path proposes.' 'It does not prove.' 'It cites verified receipts.' \
            'The operator frame is not a preserved fact.' 'Probe requests do not execute.' \
            'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  grep -qF "$_nbl" "$MANUAL" || fail "manual NOVELTY boundary line drifted: $_nbl"
done
# The manual documents the three DREAM-EXPORT-0 operator commands (manual surface == binary surface) and states
# the dream-export doctrine: the bridge preserves dream provenance and creates NO new authority, the exported
# material stays hypothesis_only, the dream engine's private dream_only/DreamOnly authority never crosses, probe
# requests do not execute, and a dream-exported hypothesis can never become evidence, a promotion, or training. It
# records the DREAM-EXPORT-1 nine-line dream-export-operator-path boundary verbatim.
for _dxc in 'dream-export --input-dir' 'dream-export-report --input-dir' 'dream-export-replay --input-dir'; do
  grep -qF "$_dxc" "$MANUAL" || fail "manual no longer documents: $_dxc"
done
for _dxd in 'preserves dream provenance' 'without creating a new authority type' 'never crosses' \
            'can never become evidence, a promotion, or training'; do
  grep -qF "$_dxd" "$MANUAL" || fail "manual dream-export doctrine line drifted: $_dxd"
done
for _dxb in 'The dream export operator path preserves provenance.' 'It does not create a new authority.' \
            'Exported dream material remains HypothesisOnly.' 'Dream origin remains auditable.' \
            'DreamOnly remains private to dream-engine.' 'Probe requests do not execute.' \
            'Nothing becomes evidence.' 'Nothing promotes.' 'Nothing trains.'; do
  grep -qF "$_dxb" "$MANUAL" || fail "manual DREAM-EXPORT boundary line drifted: $_dxb"
done

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

# ---- 11. CORPUS operator path: run the corpus flow from a LOCAL directory of .txt documents ----
# CORPUS-1 boundary (recorded verbatim):
#   The corpus operator path reads local documents.
#   It does not trust local documents.
#   Source selection is verified and replayable.
#   The whole corpus is hash-bound.
#   Verification comes before tracing.
#   Nothing executes.
#   Nothing becomes evidence.
#   Nothing promotes.
#   Nothing trains.
# The corpus commands only read a directory INSIDE the working dir, so the sample corpus lives under target/
# (relative path). Two admitted .txt documents (a-east grounding, b-west side) PLUS a hidden file, a non-.txt
# file, and an escaping symlink that the directory filter must REFUSE — proving the filter matches CORPUS-0.
mkdir -p "$corpuswork/corpus"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$corpuswork/corpus/a-east.txt"
printf 'The west tunnel remains closed. Crews continue repairs.' > "$corpuswork/corpus/b-west.txt"
printf 'hidden secret.' > "$corpuswork/corpus/.hidden.txt"
printf 'ignored note.' > "$corpuswork/corpus/notes.md"
ln -s /etc/hostname "$corpuswork/corpus/escape.txt" 2>/dev/null || true
# corpus-trace --input-dir --out: enumerate, filter, sort, verify, ground, hash-bind. The trace carries the
# corpus's OWN verified first span and the boundary markers — proof the documents were read, NOT trusted.
$BIN corpus-trace --input-dir "$corpusrel/corpus" --out "$corpuswork/trace.json"
for _cm in '"starts_from_verified_receipt": true' '"reading_passed": true' '"nothing_executed": true' \
           '"promotion_refused": true' '"nothing_becomes_evidence": true' \
           '"execution_status": "requires_operator"' '"promotion_status": "rejected"' \
           '"training_justified": false'; do
  grep -qF "$_cm" "$corpuswork/trace.json" || fail "corpus-trace marker missing: $_cm"
done
# The trace really read the corpus's first span (the grounding document's first sentence), not the canonical corpus.
grep -qF '"reading_answer": "The east bridge reopened today."' "$corpuswork/trace.json" \
  || fail 'corpus-trace did not read the operator corpus first span'
# No affirmative-authority status leaked into the corpus trace.
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$corpuswork/trace.json"; then
  fail 'corpus trace claims an executed/recorded/promoted/granted status'
fi
# corpus-report re-derives from the SAME corpus + trace and renders the SOURCE SELECTION (the grounded document,
# unambiguous) listing EXACTLY the two admitted documents — proving the directory filter excluded the hidden
# file, the .md, and the escaping symlink (matches CORPUS-0). The 9-line corpus boundary is present.
$BIN corpus-report --input-dir "$corpusrel/corpus" --trace "$corpuswork/trace.json" --out "$corpuswork/report.txt"
for _rm in 'SOURCE SELECTION' 'grounded document:  [0] a-east.txt' 'corpus documents:   2' 'Nothing trains.'; do
  grep -qF "$_rm" "$corpuswork/report.txt" || fail "corpus-report missing: $_rm"
done
# The refused entries never became documents (their names/content do not appear in the report).
if grep -qE 'hidden|notes\.md|escape\.txt' "$corpuswork/report.txt"; then fail 'corpus-report leaked a refused entry'; fi
# corpus-bundle + corpus-bundle-verify (clean) re-derive byte-identically and print the corpus boundary; the
# source attribution names the grounding document unambiguously.
$BIN corpus-bundle --input-dir "$corpusrel/corpus" --out "$corpuswork/pack"
grep -qF '"document_title": "a-east.txt"' "$corpuswork/pack/corpus-source.json" \
  || fail 'corpus-source.json did not name the grounding document'
out="$($BIN corpus-bundle-verify --input-dir "$corpusrel/corpus" --path "$corpuswork/pack")"
case "$out" in *'corpus-bundle-verify: OK'*) : ;; *) fail 'corpus-bundle-verify did not pass on a clean bundle' ;; esac
case "$out" in *'The corpus flow reads local documents.'*) : ;; *) fail 'corpus-bundle-verify did not emit the corpus boundary' ;; esac
# RE-DERIVE IS LOAD-BEARING over the WHOLE corpus — every mutation must be refused (never trusted from bytes):
# (a) mutating the GROUNDING document re-derives a different trace -> refused, then restore.
printf 'The east bridge collapsed today. Traffic stopped by noon.' > "$corpuswork/corpus/a-east.txt"
if $BIN corpus-bundle-verify --input-dir "$corpusrel/corpus" --path "$corpuswork/pack" >/dev/null 2>&1; then
  fail 'corpus-bundle-verify accepted a mutated grounding document'
fi
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$corpuswork/corpus/a-east.txt"
# (b) mutating a NON-GROUNDING SIDE document also re-derives a different trace (structure hash binds the whole
#     corpus) -> refused, then restore. This is the corpus-specific property a single-document guard cannot show.
printf 'The west tunnel reopened early. Crews left.' > "$corpuswork/corpus/b-west.txt"
if $BIN corpus-bundle-verify --input-dir "$corpusrel/corpus" --path "$corpuswork/pack" >/dev/null 2>&1; then
  fail 'corpus-bundle-verify accepted a mutated non-grounding side document'
fi
printf 'The west tunnel remains closed. Crews continue repairs.' > "$corpuswork/corpus/b-west.txt"
# (c) a tampered BUNDLE FILE (source / trace / report / questions / manifest) is refused — each file re-derives.
for _cf in corpus-source.json trace.json report.txt questions.txt manifest.json; do
  cp -r "$corpuswork/pack" "$corpuswork/pack_t"
  printf '\n{tampered}' >> "$corpuswork/pack_t/$_cf"
  if $BIN corpus-bundle-verify --input-dir "$corpusrel/corpus" --path "$corpuswork/pack_t" >/dev/null 2>&1; then
    fail "corpus-bundle-verify accepted a tampered $_cf"
  fi
  rm -rf "$corpuswork/pack_t"
done
# (d) a tampered standalone TRACE is refused by corpus-report.
printf '\n{tampered}' >> "$corpuswork/trace.json"
if $BIN corpus-report --input-dir "$corpusrel/corpus" --trace "$corpuswork/trace.json" >/dev/null 2>&1; then
  fail 'corpus-report accepted a tampered trace'
fi

# ---- 12. NOVELTY operator path: the hypothesis-only proposer ABOVE a verified corpus trace ----
# NOVELTY-1 boundary (recorded verbatim):
#   The novelty operator path proposes.
#   It does not prove.
#   It cites verified receipts.
#   The operator frame is not a preserved fact.
#   Probe requests do not execute.
#   Nothing becomes evidence.
#   Nothing promotes.
#   Nothing trains.
# The novelty commands only read a corpus directory + a frame file INSIDE the working dir, so the sample lives
# under target/ (relative paths). Two admitted .txt documents (a-east grounding, b-west side) and an operator
# frame whose lines are CANDIDATE broken assumptions — never trusted as fact.
mkdir -p "$noveltywork/corpus"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$noveltywork/corpus/a-east.txt"
printf 'The west tunnel remains closed. Crews continue repairs.' > "$noveltywork/corpus/b-west.txt"
printf 'The east bridge stays closed indefinitely.\nTraffic never recovers after a closure.\n' > "$noveltywork/frame.txt"
# A novelty packet is ONLY ever produced on top of a VERIFIED corpus trace, so corpus-trace runs FIRST and its
# trace is the source of truth the packet must cite. --out writes exact replayable bytes (never a redirect).
$BIN corpus-trace --input-dir "$noveltyrel/corpus" --out "$noveltywork/trace.json"
# novelty-packet: re-derive + byte-verify the corpus trace, then emit the hypothesis-only packet from the
# VERIFIED corpus + operator frame.
$BIN novelty-packet --input-dir "$noveltyrel/corpus" --corpus-trace "$noveltywork/trace.json" --frame "$noveltyrel/frame.txt" --out "$noveltywork/novelty.json"
# Authority is hypothesis_only; the packet carries no score and no affirmative-authority status.
grep -qF '"authority": "hypothesis_only"' "$noveltywork/novelty.json" || fail 'novelty-packet did not record hypothesis_only authority'
if grep -qF '"score"' "$noveltywork/novelty.json"; then fail 'novelty packet carries a score'; fi
# Every probe request is NON-executing (executes:false); none executes.
grep -qF '"executes": false' "$noveltywork/novelty.json" || fail 'novelty packet did not record a non-executing probe request'
if grep -qF '"executes": true' "$noveltywork/novelty.json"; then fail 'novelty packet records an executing probe request'; fi
if grep -qE '"(execution_status|observation_status|promotion_status)": "(executed|recorded|promoted|granted|evidence)"' "$noveltywork/novelty.json"; then
  fail 'novelty packet claims an executed/recorded/promoted/granted status'
fi
# forbidden_uses records exactly the four refused uses (a packet may never become or do these).
for _fu in evidence execution promotion training; do
  grep -qF "\"$_fu\"" "$noveltywork/novelty.json" || fail "novelty packet forbidden_uses missing: $_fu"
done
# THE LOAD-BEARING GROUNDING PROPERTY: the only grounded content is the VERIFIED corpus span; the operator
# frame's claim is a broken-assumption candidate, NEVER a preserved fact.
grep -qF '"The east bridge reopened today."' "$noveltywork/novelty.json" || fail 'novelty packet did not preserve the verified corpus span'
# The eight-line NOVELTY-0 boundary is present in the packet's own bytes.
for _nb in 'Novelty packets propose.' 'They do not prove.' 'Nothing becomes evidence.' 'Nothing trains.'; do
  grep -qF "$_nb" "$noveltywork/novelty.json" || fail "novelty packet boundary line drifted: $_nb"
done
# novelty-report re-derives from the SAME corpus + frame and renders the PROPOSAL-ONLY report.
$BIN novelty-report --input-dir "$noveltyrel/corpus" --frame "$noveltyrel/frame.txt" --packet "$noveltywork/novelty.json" --out "$noveltywork/report.txt"
for _rm in 'PROPOSAL ONLY' 'PRESERVED FACTS (verified corpus spans' 'PROBE REQUESTS (recorded, NOT executed)' \
           'never trusted as fact' 'Nothing trains.'; do
  grep -qF "$_rm" "$noveltywork/report.txt" || fail "novelty-report missing: $_rm"
done
# novelty-replay confirms the packet re-derives byte-identically (a determinism proof: proposes, never proves).
out="$($BIN novelty-replay --input-dir "$noveltyrel/corpus" --frame "$noveltyrel/frame.txt" --packet "$noveltywork/novelty.json")"
case "$out" in *'does not prove'*) : ;; *) fail 'novelty-replay did not confirm the deterministic packet' ;; esac
# RE-DERIVE IS LOAD-BEARING over the novelty packet — every refusal end-to-end (never trusted from bytes):
# (a) an empty frame (no candidate assumption) fails closed — no packet is produced.
printf '\n   \n' > "$noveltywork/empty_frame.txt"
if $BIN novelty-packet --input-dir "$noveltyrel/corpus" --corpus-trace "$noveltywork/trace.json" --frame "$noveltyrel/empty_frame.txt" >/dev/null 2>&1; then
  fail 'novelty-packet accepted an empty frame'
fi
# (b) an UNSUPPORTED preserved fact is refused: swap the preserved fact for the frame's OWN (unverified) claim.
#     Only the standalone preserved_facts element line is rewritten (the candidate/falsifier lines that quote the
#     span end differently and are untouched), so this proves the frame's claim cannot be laundered into a fact.
sed 's/^\( *\)"The east bridge reopened today\."$/\1"The east bridge stays closed indefinitely."/' \
  "$noveltywork/novelty.json" > "$noveltywork/unsupported.json"
if cmp -s "$noveltywork/novelty.json" "$noveltywork/unsupported.json"; then fail 'novelty unsupported-fact tamper was a no-op'; fi
if $BIN novelty-report --input-dir "$noveltyrel/corpus" --frame "$noveltyrel/frame.txt" --packet "$noveltywork/unsupported.json" >/dev/null 2>&1; then
  fail 'novelty-report accepted an unsupported preserved fact'
fi
if $BIN novelty-replay --input-dir "$noveltyrel/corpus" --frame "$noveltyrel/frame.txt" --packet "$noveltywork/unsupported.json" >/dev/null 2>&1; then
  fail 'novelty-replay accepted an unsupported preserved fact'
fi
# (c) a tampered packet is refused by BOTH report and replay.
cp "$noveltywork/novelty.json" "$noveltywork/tampered.json"
printf '\n{tampered}' >> "$noveltywork/tampered.json"
if $BIN novelty-report --input-dir "$noveltyrel/corpus" --frame "$noveltyrel/frame.txt" --packet "$noveltywork/tampered.json" >/dev/null 2>&1; then
  fail 'novelty-report accepted a tampered packet'
fi
if $BIN novelty-replay --input-dir "$noveltyrel/corpus" --frame "$noveltyrel/frame.txt" --packet "$noveltywork/tampered.json" >/dev/null 2>&1; then
  fail 'novelty-replay accepted a tampered packet'
fi
# (d) a corpus trace with its verifier RECEIPT HASH stripped is NOT the verified trace -> novelty-packet refuses
#     to ground on it (the packet is only ever produced on top of a verified corpus trace).
grep -v structure_hash "$noveltywork/trace.json" > "$noveltywork/trace_nohash.json"
if $BIN novelty-packet --input-dir "$noveltyrel/corpus" --corpus-trace "$noveltywork/trace_nohash.json" --frame "$noveltyrel/frame.txt" >/dev/null 2>&1; then
  fail 'novelty-packet accepted a receipt-hash-stripped corpus trace'
fi

# ---- 13. DREAM EXPORT operator path: the dream provenance bridge into the hypothesis-only path ----
# DREAM-EXPORT-1 boundary (recorded verbatim):
#   The dream export operator path preserves provenance.
#   It does not create a new authority.
#   Exported dream material remains HypothesisOnly.
#   Dream origin remains auditable.
#   DreamOnly remains private to dream-engine.
#   Probe requests do not execute.
#   Nothing becomes evidence.
#   Nothing promotes.
#   Nothing trains.
# The dream-export commands only read a corpus directory + a frame file INSIDE the working dir, so the sample lives
# under target/ (relative paths). dream-export re-derives (GENERATES) the terminal dream packet from the corpus +
# frame + dials and bridges it through the EXISTING hypothesis gate — no standalone packet emitter exists
# (dream-engine is a quarantined library), so dream packet generation happens FIRST, here, inside dream-export.
mkdir -p "$dreamwork/corpus"
printf 'The east bridge reopened today. Traffic resumed by noon.' > "$dreamwork/corpus/a-east.txt"
printf 'The west tunnel remains closed. Crews continue repairs.' > "$dreamwork/corpus/b-west.txt"
printf 'The east bridge stays closed indefinitely.\nTraffic never recovers after a closure.\n' > "$dreamwork/frame.txt"
# dream packet generation FIRST: dream-export re-derives the terminal dream packet and bridges it; --out writes
# exact replayable bytes (never a redirect).
$BIN dream-export --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --out "$dreamwork/export.json"
# The export carries the EXISTING hypothesis_only authority (NOT a new dream authority), records the dream origin
# (auditable), and routes through the existing gate.
grep -qF '"authority_after_export": "hypothesis_only"' "$dreamwork/export.json" || fail 'dream-export did not record hypothesis_only authority_after_export'
grep -qF '"dream_origin": true' "$dreamwork/export.json" || fail 'dream-export did not record dream_origin true'
grep -qF '"exported_via_existing_hypothesis_gate": true' "$dreamwork/export.json" || fail 'dream-export did not route through the existing hypothesis gate'
# Dream origin is AUDITABLE: the exported hypothesis cites a dream: provenance label.
grep -qF '"source_label": "dream:' "$dreamwork/export.json" || fail 'dream-export did not record the dream provenance label'
# THE LOAD-BEARING CROSSING PROPERTY: the dream engine's private dream_only / DreamOnly authority NEVER crosses —
# it appears NOWHERE in the emitted export JSON (only the EXISTING hypothesis_only authority does).
if grep -qiF 'dream_only' "$dreamwork/export.json"; then fail 'dream-export leaked a dream_only authority'; fi
if grep -qiF 'dreamonly' "$dreamwork/export.json"; then fail 'dream-export leaked a DreamOnly authority'; fi
# The source dream's probe requests do NOT execute (no executing probe leaks into the export).
if grep -qF '"executes": true' "$dreamwork/export.json"; then fail 'dream-export recorded an executing probe request'; fi
# The export records its forbidden uses (a dream-exported hypothesis may never become or do these).
for _du in ground_claim serve_as_evidence change_training_gate; do
  grep -qF "\"$_du\"" "$dreamwork/export.json" || fail "dream-export forbidden_uses missing: $_du"
done
# The eight-line DREAM-EXPORT-0 binary boundary is present in the export's own bytes (lowercase hypothesis_only).
for _db in 'Dream export preserves provenance.' 'It does not create a new authority.' \
           'Exported dream material remains hypothesis_only.' 'Probe requests do not execute.' 'Nothing trains.'; do
  grep -qF "$_db" "$dreamwork/export.json" || fail "dream-export boundary line drifted: $_db"
done
# dream-export-report re-derives from the SAME corpus + frame and renders the PROVENANCE BRIDGE report: the
# hypothesis_only/dream_origin banner, the auditable provenance, and the executes:false probe-provenance note.
$BIN dream-export-report --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --export "$dreamwork/export.json" --out "$dreamwork/report.txt"
grep -qF 'DREAM EXPORT (PROVENANCE BRIDGE — hypothesis_only, dream_origin)' "$dreamwork/report.txt" || fail 'dream-export-report missing the provenance banner'
grep -qE 'authority_after_export: +hypothesis_only' "$dreamwork/report.txt" || fail 'dream-export-report did not state hypothesis_only'
grep -qE 'dream_origin: +true' "$dreamwork/report.txt" || fail 'dream-export-report did not state dream_origin true'
grep -qF 'executes: false — NEVER executed' "$dreamwork/report.txt" || fail 'dream-export-report did not state probe requests do not execute'
grep -qF 'Nothing trains.' "$dreamwork/report.txt" || fail 'dream-export-report boundary line drifted'
# dream-export-replay confirms the export re-derives byte-identically (a determinism proof; provenance preserved).
out="$($BIN dream-export-replay --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --export "$dreamwork/export.json")"
case "$out" in *'dream-export-replay: OK'*) : ;; *) fail 'dream-export-replay did not confirm the deterministic export' ;; esac
case "$out" in *'hypothesis_only'*) : ;; *) fail 'dream-export-replay did not preserve hypothesis_only' ;; esac
# RE-DERIVE IS LOAD-BEARING over the dream export — every tamper must be refused (never trusted from bytes):
# (a) a FOREIGN / tampered --dream-packet (here the export bundle itself — NOT the byte-exact re-derived dream
#     packet) cannot be laundered into an export. dream-export WITHOUT --dream-packet succeeded above, so this
#     proves the --dream-packet cross-check is real and discriminating, not a no-op.
if $BIN dream-export --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --dream-packet "$dreamwork/export.json" >/dev/null 2>&1; then
  fail 'dream-export accepted a foreign/tampered dream packet'
fi
# (b) a tampered DreamExportReceipt (flip the via-existing-gate bit) is refused by replay.
sed 's/"exported_via_existing_hypothesis_gate": true/"exported_via_existing_hypothesis_gate": false/' "$dreamwork/export.json" > "$dreamwork/receipt_tamper.json"
if cmp -s "$dreamwork/export.json" "$dreamwork/receipt_tamper.json"; then fail 'dream-export receipt tamper was a no-op'; fi
if $BIN dream-export-replay --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --export "$dreamwork/receipt_tamper.json" >/dev/null 2>&1; then
  fail 'dream-export-replay accepted a tampered receipt'
fi
# (c) a receipt whose dream_origin was forged to FALSE is refused by BOTH report and replay — a forged non-dream
#     origin cannot be laundered into a clean report or replay (dream origin is load-bearing and auditable).
sed 's/"dream_origin": true/"dream_origin": false/' "$dreamwork/export.json" > "$dreamwork/origin_false.json"
if cmp -s "$dreamwork/export.json" "$dreamwork/origin_false.json"; then fail 'dream-export dream_origin tamper was a no-op'; fi
if $BIN dream-export-report --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --export "$dreamwork/origin_false.json" >/dev/null 2>&1; then
  fail 'dream-export-report accepted dream_origin=false'
fi
if $BIN dream-export-replay --input-dir "$dreamrel/corpus" --frame "$dreamrel/frame.txt" --export "$dreamwork/origin_false.json" >/dev/null 2>&1; then
  fail 'dream-export-replay accepted dream_origin=false'
fi

# ---- DATA-1: data curation operator path — run the REAL curator over candidate manifests ----
# data-curator (DATA-0) is a LIBRARY crate with NO file ingestion: the curator consumes an in-memory
# CandidateManifest by boundary design (it does no filesystem IO), so the operator exercises it through its
# cargo test suite, which constructs candidate manifests and runs the real curate() over each. We write an
# operator-readable illustration of those manifests to a temp dir, then run each required outcome as a named
# test (--exact, so EXACTLY one test runs — a dropped/typo'd outcome shows "0 passed" and fails here, never
# "1 passed"). No code crate change — this documents and smoke-tests existing DATA-0 behavior only.
#
# DATA-1 boundary (recorded verbatim):
#   The curation operator path classifies candidate data.
#   It admits, rejects, or quarantines.
#   It does not create truth.
#   It does not create memory.
#   It does not train.
#   It does not execute.
#   It does not promote.
#   Training eligibility remains closed.
cat > "$curatework/candidate_manifest.txt" <<'CANDIDATES'
# candidate manifest (illustration; the real curator consumes the in-memory CandidateManifest the tests build)
# id              kind            provenance   grounding    split     -> expected disposition
# clean           document_span   src://doc    span:0..10   train     -> ADMITTED (candidate_only)
# no_provenance   document_span   (none)       span:0..10   train     -> REJECTED (missing provenance)
# dup, dup        document_span   src://doc    span:0..10   train     -> REJECTED (duplicate id)
# injection       document_span   src://doc    span:0..10   train     -> QUARANTINED (prompt-injection marker)
# leak (train)    document_span   src://doc    span:0..10   train     -> QUARANTINED (train/holdout leakage)
# leak (holdout)  document_span   src://doc    span:0..10   holdout   -> QUARANTINED (train/holdout leakage)
# training_eligibility is ALWAYS Closed; is_eligible() can never return true.
CANDIDATES
_DCM=crates/data-curator/Cargo.toml
# Run the REAL curator over each required outcome. Each named test builds a candidate manifest and runs curate();
# --exact + the full tests:: path means EXACTLY one test runs, so a dropped outcome shows "0 passed" and fails here.
for _ct in clean_document_span_is_admitted_but_only_candidate \
           missing_provenance_is_rejected \
           duplicate_id_is_rejected_and_recorded_as_contamination \
           prompt_injection_is_quarantined_not_deleted_or_admitted \
           train_holdout_leakage_is_detected_and_quarantined \
           training_eligibility_is_never_eligible; do
  if _ct_out="$(cargo test --offline --manifest-path "$_DCM" -- --exact "tests::$_ct" 2>&1)"; then
    printf '%s\n' "$_ct_out" | grep -qF '1 passed' || fail "curation outcome did not run (vacuous): $_ct"
  else
    fail "curation outcome test failed: $_ct"
  fi
done
# Manual drift guard: the manual documents the data-curation operator path and records its doctrine + boundary.
grep -qF 'admits, rejects, or quarantines' "$MANUAL" || fail 'manual no longer states curation admits/rejects/quarantines'
grep -qF 'quarantined, not deleted' "$MANUAL" || fail 'manual no longer states prompt-injection is quarantined, not deleted'
grep -qF 'no code path can return training-eligible' "$MANUAL" || fail 'manual no longer states eligibility can never be true'
for _cul in 'The curation operator path classifies candidate data.' 'It admits, rejects, or quarantines.' \
            'It does not create truth.' 'It does not create memory.' 'It does not train.' \
            'It does not execute.' 'It does not promote.' 'Training eligibility remains closed.'; do
  grep -qF "$_cul" "$MANUAL" || fail "manual curation boundary line drifted: $_cul"
done

echo 'operator-smoke: OK — the documented operator path runs and the manual matches the binary'
