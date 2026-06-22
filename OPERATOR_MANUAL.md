# Operator Manual — Cognitive OS Prototype

> A plain operator guide to the frozen Cognitive OS prototype: what it is, what it can run, how to
> reproduce each demo, how to verify each milestone, and which authority boundaries stay closed. This
> manual EXPLAINS the prototype; it adds no behavior and grants no authority. Every command below was
> run against the built binary; the commands and outputs are real, not illustrative.

## Running these commands

All commands use the `cognitive-demo` binary in the `crates/cognitive-demo` crate. Run everything from
the `cognitive-os/` directory.

```sh
# Build once (uses the vendored/cached crates; --offline is optional if you have them):
cargo build --offline -p cognitive-demo

# Then invoke the binary directly:
./target/debug/cognitive-demo <command> [flags]

# Or run through cargo (equivalent):
cargo run --offline -q -p cognitive-demo -- <command> [flags]
```

The full command surface (also printed when you run the binary with no arguments):

```text
trace [--out PATH]                          report --trace PATH [--out PATH]
replay --trace PATH                         ask --trace PATH --question SLUG [--out PATH]
questions                                   bundle --out DIR
bundle-verify --path DIR                    scenarios
scenario-pack --out DIR                     scenario-verify --path DIR
scenario-matrix --pack DIR [--out PATH]     scenario-matrix-report --matrix PATH [--out PATH]
scenario-matrix-verify --pack DIR --matrix PATH
failure-cases                               failure-pack --out DIR
failure-verify --path DIR

doc-trace --input PATH [--out PATH]         doc-report --input PATH --trace PATH [--out PATH]
doc-bundle --input PATH --out DIR           doc-bundle-verify --input PATH --path DIR
doc-scenarios                               doc-scenario-pack --out DIR
doc-scenario-verify --path DIR              doc-scenario-matrix --path DIR [--out PATH]

corpus-trace --input-dir DIR [--out PATH]   corpus-report --input-dir DIR --trace PATH [--out PATH]
corpus-bundle --input-dir DIR --out DIR     corpus-bundle-verify --input-dir DIR --path DIR
corpus-scenarios                            corpus-scenario-pack --out DIR
corpus-scenario-verify --path DIR           corpus-scenario-matrix --path DIR [--out PATH]

novelty-packet --input-dir DIR --corpus-trace PATH --frame PATH [--out PATH]
novelty-report --input-dir DIR --frame PATH --packet PATH [--out PATH]
novelty-replay --input-dir DIR --frame PATH --packet PATH

dream-export --input-dir DIR --frame PATH [--seed N] [--weirdness W] [--dream-packet PATH] [--out PATH]
dream-export-report --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH [--out PATH]
dream-export-replay --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH
```

The `doc-*` commands run the same pipeline from a **local operator-supplied document** instead of the
fixed corpus — see §11. The document is read but not trusted: it is verified before it is traced.

The `corpus-*` commands run the same pipeline from a **local directory of `.txt` documents** — see §12. The
corpus is enumerated, path-filtered, sorted, verified, grounded, and **hash-bound as a whole**; the documents
are read but not trusted.

The `novelty-*` commands run a **hypothesis-only proposer above a verified corpus trace** — see §13. Given a
verified corpus trace and an operator `--frame`, they emit a deterministic novelty packet: the frame's lines
become candidate *broken assumptions* (no truth claimed), the only grounded content is the **verified corpus
span**, and the packet's authority is `hypothesis_only`. Novelty packets **propose but do not prove**; the
operator frame is recorded but **never grounded as fact**.

The `dream-export-*` commands run the prototype's **dream provenance bridge** — see §14. They let a terminal dream
packet (the inert, seeded distortion artifact from the frozen `dream-engine`) cross into the **existing** hypothesis-only
proposal path while preserving its dream origin, and they do so **without creating a new authority type**: the exported
material is an ordinary `hypothesis_only` proposal whose dream origin stays auditable, and the dream engine's private
`dream_only` authority never crosses.

> **Reproducibility note (important).** Use `trace --out FILE` to write a trace you will later `report`,
> `replay`, or `ask` against. Writing with a shell redirect (`trace > FILE`) appends a trailing newline,
> and every verify surface re-derives the canonical trace and byte-compares — so a redirected file is
> correctly REFUSED as "not the canonical trace". `--out` writes the exact bytes; a redirect does not.

## 1. What this prototype is

A **deterministic Rust prototype and testbed** for a cognitive-OS *boundary*: it shows how a reading /
verification step, a hypothesis / probe pipeline, and an operator-facing demo can be wired together so
that **nothing crosses from "proposed" into "authoritative" without an explicit human gate** — and then
proves, mechanically, that the gate holds.

It is built from layers that have each been frozen as a named milestone (see §3). The top layer,
`crates/cognitive-demo`, is a thin, fully deterministic surface over the two deeper frozen tracks: it can
produce one canonical end-to-end trace, report it, answer fixed audit questions about it, package it into
a reproducible bundle, run that trace under several scenarios, summarize the coverage, and forge-then-
reject a set of forbidden-authority attacks. There is **no model in the loop** — every output is a pure
function of fixed inputs.

## 2. What this prototype is not

- It is **not a trained model** and contains no learned weights. No training has run.
- It does **not execute probes.** A probe is classified and queued; the execution intent records a
  `requires_operator` state and never becomes `executed`.
- It does **not turn observations into evidence.** An observation is quarantined (`requires_review` /
  `observation_only`); it is never recorded as evidence.
- It does **not promote anything.** A promotion-to-evidence request is refused (`grants_promotion=false`).
- It does **not mutate reading memory** or alter a verifier receipt.
- It is **not production-ready.** It is a prototype that demonstrates a boundary; it is not a deployed
  reasoning system.

## 3. Frozen milestones and recovery tags

Each layer is frozen as an annotated point in history. Recover any milestone with
`git checkout <tag>`; verify the whole stack at any time with `./scripts/release_check.sh`
(it must exit 0 and print nothing — see §17/§18). Each tag has a freeze record document.

| Tag | Commit | Freezes | Record |
| --- | --- | --- | --- |
| `cognitive-os-governance-v0.1` | `bbd1113` | the v0.1 governance / evidence-contract lineage | `GOVERNANCE_MILESTONE.md` |
| `reading-track-v0.1` | `f6fa55a` | the reading verifier + verified reading receipt | `READING_TRACK_MILESTONE.md` |
| `hypothesis-track-v0.1` | `bb20acf` | the propose → probe → review → intent → observation → promotion-refusal chain | `HYPOTHESIS_TRACK_MILESTONE.md` |
| `integration-demo-v0.1` | `95b586d` | the operator-visible demo (trace / report / questions / bundle) | `INTEGRATION_DEMO_MILESTONE.md` |
| `multi-trace-validation-v0.1` | `460be0c` | the multi-trace validation pack (scenarios / matrix / failure injection) | `MULTI_TRACE_VALIDATION_MILESTONE.md` |

```sh
# Recover a milestone (read-only checkout of the frozen point):
git checkout reading-track-v0.1

# Verify every milestone lock at once (green + silent == all milestones intact):
./scripts/release_check.sh ; echo "exit=$?"
```

### Self-check: the operator smoke test

`./scripts/operator_smoke.sh` runs the whole documented operator path end-to-end against the freshly
built binary — `trace --out`, `report`, `replay`, `questions`, `ask`, `bundle`/`bundle-verify`,
`scenario-pack`/`scenario-verify`, `scenario-matrix`/`scenario-matrix-verify`,
`failure-pack`/`failure-verify`, the document and corpus flows (`doc-*` / `corpus-*`), the
hypothesis-only novelty flow (`novelty-packet` / `novelty-report` / `novelty-replay`), the dream export
provenance bridge (`dream-export` / `dream-export-report` / `dream-export-replay`), the data curation
gate (the real `curate()` over candidate manifests via `crates/data-curator`'s tests — admit / reject /
quarantine), and the bounded horizon harness (the real `run_horizon()` over `H0..H5` via
`crates/cognitive-demo`'s `horizon::tests` — bounded turns / no gate bypass / training never opens) — inside a throwaway
temp dir (no repo debris), and **fails closed** if any
documented command, boundary line, or verify step has drifted from this manual. It re-derives every
generated artifact through the binary's own verify subcommands (it never trusts the bytes) and confirms a
tampered artifact is still refused. It runs as part of `./scripts/release_check.sh`, so manual drift breaks
the gate. The smoke only reproduces the operator path: it creates no authority, executes nothing, promotes
nothing, and trains nothing.

```sh
./scripts/operator_smoke.sh ; echo "exit=$?"
# -> operator-smoke: OK — the documented operator path runs and the manual matches the binary
```

## 4. How to run the canonical demo

The canonical demo is one deterministic `CognitiveTrace`: a verified reading receipt, a hypothesis that
cites it by hash, a queued probe, a governance review, an execution intent that stays `requires_operator`,
a quarantined observation, and a refused promotion.

```sh
# Write the canonical trace to a file (exact bytes — replayable):
./target/debug/cognitive-demo trace --out trace.json
# -> trace.json (1994 bytes)
```

## 5. How to inspect the trace

```sh
# Render a plain operator report of every stage with ids/hashes:
./target/debug/cognitive-demo report --trace trace.json

# Confirm the trace is the byte-identical canonical trace:
./target/debug/cognitive-demo replay --trace trace.json
# -> replay: OK — the trace is the byte-identical canonical trace
```

`report` and `replay` never trust the file you give them: they re-derive the canonical trace and
byte-compare. A tampered or foreign trace is refused (non-zero exit), never rendered or replayed.

## 6. How to ask fixed trace questions

The interrogation surface is a **finite, enumerated** set — there is no free-form or natural-language
path. List the menu, then ask exactly one question by slug:

```sh
./target/debug/cognitive-demo questions
./target/debug/cognitive-demo ask --trace trace.json --question was-anything-executed
# -> WAS ANYTHING EXECUTED?  No. ...
```

The eight question slugs:

```text
what-read                     what the reading stage read and verified
what-was-proven               what was actually proven (only the reading receipt)
what-was-hypothesized         what was hypothesized (a proposal, not a claim)
what-probe-was-requested      what probe was requested (a queued record)
was-anything-executed         whether anything executed (no)
did-anything-become-evidence  whether anything became evidence (no)
why-was-promotion-refused     why the promotion was refused
did-training-open             whether training opened (no)
```

An unknown slug fails closed (`unknown question ...`); `ask` re-derives and verifies the trace before
answering, and every answer is prose formatted from the trace's recorded fields — never a new verdict.

## 7. How to build and verify a bundle

The repro bundle is a four-file pack purely derived from the canonical trace.

```sh
./target/debug/cognitive-demo bundle --out pack
# -> wrote 4 files: trace.json, report.txt, questions.txt, manifest.json
./target/debug/cognitive-demo bundle-verify --path pack
# -> bundle-verify: OK — every bundle file re-derives byte-identically from the canonical trace
```

`bundle-verify` re-derives every file (including the manifest) and byte-compares, so a missing, tampered,
or foreign file is refused. The bundle is a demonstration; it is never trusted as authority.

## 8. How to run the scenario pack

The scenario pack runs the same pipeline under several deterministic paths, each preserving the same
boundary.

```sh
./target/debug/cognitive-demo scenarios
./target/debug/cognitive-demo scenario-pack --out scn
# -> wrote 4 scenarios (16 bundle files) + pack-manifest.json
./target/debug/cognitive-demo scenario-verify --path scn
# -> scenario-verify: OK — every scenario bundle and the pack manifest re-derive byte-identically
```

The four scenarios:

```text
happy-boundary       governance approves; intent requires_operator; observation requires_review; promotion rejected
review-rejected      governance rejects; intent blocked; observation rejected; promotion rejected
review-deferred      governance defers; intent blocked; observation rejected; promotion rejected
high-risk-blocked    probe classified blocked (high-risk AND irreversible); no approval path; no execution
```

`happy-boundary` is the canonical demo trace, byte-for-byte. Every scenario keeps execution un-executed,
the observation quarantined, promotion refused, and training closed.

## 9. How to view the scenario matrix

The matrix is a coverage view of the scenario pack — it summarizes, it does not act.

```sh
# Verify the pack, then emit the canonical coverage matrix:
./target/debug/cognitive-demo scenario-matrix --pack scn --out matrix.json
# Render a plain report of the matrix:
./target/debug/cognitive-demo scenario-matrix-report --matrix matrix.json
# Verify BOTH the pack and the matrix re-derive byte-identically:
./target/debug/cognitive-demo scenario-matrix-verify --pack scn --matrix matrix.json
```

Each matrix cell is the trace's real verdict (`no_execution`, `no_evidence`, `no_promotion`,
`no_training`); the summary proves all sixteen cells (4 scenarios × 4 boundaries) hold. The matrix is
re-derived from the scenario set, never trusted from the pack files.

## 10. How to run failure-injection checks

The failure pack proves the **bad** paths fail closed: each case forges a forbidden authority claim onto a
canonical artifact and shows the existing re-derive-and-byte-compare verifier refuses it.

```sh
./target/debug/cognitive-demo failure-cases
./target/debug/cognitive-demo failure-pack --out fp
# -> wrote 2 files (every forged authority claim REJECTED)
./target/debug/cognitive-demo failure-verify --path fp
# -> failure-verify: OK — the failure pack re-derives byte-identically; every forged claim stays rejected
```

The seven forged cases (each is **forged and rejected** — none of these states is ever real):

```text
forged-execution   forge an executed status onto the execution intent
forged-evidence    forge evidence authority onto the quarantined observation
forged-promotion   forge the promotion request to grant a promotion
forged-training    forge the P12 training gate toward justified
forged-review      forge a rejected governance review to read as approved
forged-report      forge the operator report to narrate execution and evidence
forged-matrix      forge the coverage matrix to hide a failed boundary cell
```

The forged bytes are never persisted as trusted state — only the (prose) rejection record is written.

## 11. How to trace a local operator document

The `doc-*` commands run the **same** end-to-end pipeline as the canonical demo, but starting from a
**local operator-supplied text document** instead of the fixed bridge corpus. The crucial property is not
"it reads a file" — it is that **the document is read but not trusted**. The text is read, asked of the
frozen reader for its own first span, turned into a **verified reading receipt**, and only then traced. If
the read does not verify, the doc flow fails closed and produces no trace. A verified document still cannot
become execution, evidence, promotion, memory mutation, or training — exactly like the canonical trace.

The commands only read a **local path inside the working directory.** An absolute path, a `..` traversal,
or a symlink that escapes the working directory is refused before anything is read.

```sh
# Trace a local document (exact bytes — replayable). --input is the local doc; --out is the trace file:
./target/debug/cognitive-demo doc-trace --input notes.txt --out trace.json
# -> trace.json — a CognitiveTrace whose reading stage read and verified notes.txt's first span

# Render a plain report of that trace (re-derived from the SAME document + trace, never trusted from bytes):
./target/debug/cognitive-demo doc-report --input notes.txt --trace trace.json
# -> COGNITIVE OS — END-TO-END TRACE REPORT ... (question: "What does the document state in its first span?")

# Build and verify a four-file repro bundle purely derived from the document:
./target/debug/cognitive-demo doc-bundle --input notes.txt --out pack
# -> doc-bundle: wrote 4 files to .../pack  (trace.json, report.txt, questions.txt, manifest.json)
./target/debug/cognitive-demo doc-bundle-verify --input notes.txt --path pack
# -> doc-bundle-verify: OK — every bundle file re-derives byte-identically from the operator document
```

`doc-report` and `doc-bundle-verify` re-derive every artifact from the **same** document and byte-compare,
so a tampered document, trace, report, questions file, or manifest is refused (non-zero exit) — never
rendered, replayed, or accepted. The reading stage's answer is the document's own first span, so the trace
demonstrably read the operator's text; it never grants that text any authority. The whole doc flow is
exercised end-to-end by `./scripts/operator_smoke.sh` (see §3), so this documentation cannot drift from the
binary.

```text
The document operator path explains and verifies local-document tracing.
It does not trust local input.
It does not create authority.
It does not execute.
It does not promote.
It does not train.
```

## 12. How to trace a local corpus

The `corpus-*` commands run the **same** end-to-end pipeline as the canonical demo and the `doc-*` flow, but
starting from a **local directory of `.txt` documents** instead of one document or the fixed bridge corpus. The
crucial property is not "it reads a folder" — it is that the corpus is **enumerated, path-filtered, sorted,
verified, grounded, and hash-bound as a whole**, and that **the documents are read but not trusted**. The
directory is enumerated deterministically: only non-hidden `.txt` files are admitted (hidden files, non-`.txt`
files, and any entry whose resolved path escapes the directory — e.g. a symlink — are refused, never read), and
the admitted documents are sorted by name so the trace is replayable. The corpus's own first span becomes a
**verified reading receipt**, and only then is it traced; an empty corpus (no admitted document grounds a span)
fails closed and produces no trace.

**Source selection is verified and replayable, never a semantic judgment.** Which document and span grounded the
answer is recorded in `corpus-source.json` (`document_index`, the real `document_title` filename, `span_id`,
`span_text`), re-derived from the frozen reader — it is the corpus's globally-first span, not a model's opinion
about relevance. **The whole corpus is hash-bound:** the reading receipt's structure hash binds every document's
title, spans, and sections, so mutating **any** document — the grounding document *or a non-grounding "side"
document* — re-derives a different trace and invalidates the bundle. A side document cannot silently change while
the bundle still verifies.

The commands only read a **local directory inside the working directory.** An absolute path, a `..` traversal,
or a symlinked directory that escapes the working directory is refused before anything is read.

```sh
# Trace a local corpus directory (exact bytes — replayable). --input-dir is the local folder of .txt docs:
./target/debug/cognitive-demo corpus-trace --input-dir corpus --out trace.json
# -> trace.json — a CognitiveTrace whose reading stage read and verified the corpus's first span

# Render a plain report (re-derived from the SAME corpus + trace) with a SOURCE SELECTION section naming the
# grounded document/span and listing every corpus document — never trusted from the trace bytes:
./target/debug/cognitive-demo corpus-report --input-dir corpus --trace trace.json
# -> COGNITIVE OS — END-TO-END TRACE REPORT ... + SOURCE SELECTION (grounded document, span, every document)

# Build and verify a five-file repro bundle purely derived from the corpus:
./target/debug/cognitive-demo corpus-bundle --input-dir corpus --out pack
# -> corpus-bundle: wrote 5 files to .../pack  (corpus-source.json, trace.json, report.txt, questions.txt, manifest.json)
./target/debug/cognitive-demo corpus-bundle-verify --input-dir corpus --path pack
# -> corpus-bundle-verify: OK — every bundle file re-derives byte-identically from the operator corpus
```

`corpus-report` and `corpus-bundle-verify` re-derive every artifact from the **same** corpus and byte-compare,
so a tampered document (grounding **or** side), source attribution, trace, report, questions file, or manifest is
refused (non-zero exit) — never rendered, replayed, or accepted. The reading stage's answer is the corpus's own
first span, so the trace demonstrably read the operator's documents; it never grants that text any authority, and
no document ever becomes evidence. The whole corpus flow is exercised end-to-end by
`./scripts/operator_smoke.sh` (see §3), so this documentation cannot drift from the binary.

```text
The corpus operator path reads local documents.
It does not trust local documents.
Source selection is verified and replayable.
The whole corpus is hash-bound.
Verification comes before tracing.
Nothing executes.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

## 13. How to run the novelty operator path

The `novelty-*` commands add the prototype's first **hypothesis-only proposer**: a deterministic layer that
sits *above* a verified corpus trace and produces a **novelty packet** — an assumption-breaking *proposal*
bound to verified receipts. There is still **no model in the loop**; the operator's `--frame` is the
proposer, and the harness only verifies, grounds, and structures. The crucial property is that **novelty
packets propose but do not prove**: the frame's lines become candidate *broken assumptions* with **no truth
claimed**, and the only grounded content in the packet is the **verified corpus span**. The operator frame is
recorded but **never grounded as fact** — a frame's claim can be a broken-assumption candidate, never a
preserved fact. **Preserved facts come only from verified corpus spans** (the harness refuses any preserved
fact that is not a verbatim verified span). The packet's authority is `hypothesis_only`; it carries no score
and grants no authority, **probe requests do not execute** (each is recorded with `executes: false`), and a
novelty packet **can never become evidence, a promotion, or training**.

A novelty packet is only ever produced **on top of a verified corpus trace**: `novelty-packet` re-derives the
corpus trace from `--input-dir`, byte-verifies the operator-supplied `--corpus-trace` against it, and refuses
to ground on a trace that is not the canonical one (e.g. a trace whose verifier receipt hash has been
stripped). An empty frame (no candidate assumption) fails closed and produces no packet. The corpus directory
and the frame are read but not trusted, and — like every other local-input command — an absolute path, a `..`
traversal, or a symlink that escapes the working directory is refused before anything is read.

```sh
# 1. Produce the verified corpus trace the packet will cite (exact bytes — re-derived, never trusted):
./target/debug/cognitive-demo corpus-trace --input-dir corpus --out trace.json

# 2. Emit the hypothesis-only novelty packet from that verified trace + an operator frame:
./target/debug/cognitive-demo novelty-packet --input-dir corpus --corpus-trace trace.json --frame frame.txt --out novelty.json
# -> novelty.json — authority "hypothesis_only"; broken assumptions = the frame's lines (no truth claimed);
#    preserved facts = the verified corpus span; every probe request executes:false; forbidden_uses records
#    evidence/execution/promotion/training as uses this packet may never become or do.

# 3. Render a plain operator report (re-derived from the SAME corpus + frame; a tampered packet is refused):
./target/debug/cognitive-demo novelty-report --input-dir corpus --frame frame.txt --packet novelty.json
# -> NOVELTY PACKET (PROPOSAL ONLY — hypothesis_only, not truth) ... PRESERVED FACTS (verified corpus spans ...)

# 4. Confirm the packet re-derives byte-identically (a determinism proof that also refuses any tamper):
./target/debug/cognitive-demo novelty-replay --input-dir corpus --frame frame.txt --packet novelty.json
# -> novelty-replay: OK — ... It proposes; it does not prove.
```

`novelty-report` and `novelty-replay` re-derive the whole packet from the **same** corpus + frame and
byte-compare, so a tampered packet — including one whose preserved facts were swapped for the frame's own
(unverified) claim — is refused (non-zero exit), never rendered or replayed. The packet demonstrably cites
the corpus's verified span; it never grants that span, or the operator's frame, any authority. The whole
novelty flow is exercised end-to-end by `./scripts/operator_smoke.sh` (see §3), so this documentation cannot
drift from the binary.

```text
The novelty operator path proposes.
It does not prove.
It cites verified receipts.
The operator frame is not a preserved fact.
Probe requests do not execute.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

## 14. How to run the dream export operator path

The `dream-export-*` commands add the prototype's **dream provenance bridge**: they let a terminal dream packet
(the inert, seeded distortion artifact produced by the frozen `dream-engine`) cross into the **existing
hypothesis-only proposal path** while preserving its dream origin — and they do this **without creating a new
authority type**. The crucial property is that the bridge **preserves dream provenance but grants no new
authority**: the exported material is an ordinary `hypothesis_only` proposal (the same authority every
hypothesis carries), its dream origin is recorded so it stays **auditable** (the exported hypothesis cites a
`dream:` evidence label, and the receipt records the dream packet id, input hash, seed, engine version, and
operators), and the dream engine's own private `dream_only` authority — `DreamOnly` — **never crosses**: it
remains private to `dream-engine` and appears nowhere in the emitted export. The source dream's **probe requests
do not execute** (each is recorded with `executes: false`), and a dream-exported hypothesis **can never become
evidence, a promotion, or training**.

`dream-export` re-derives (generates) the terminal dream packet from the **same** local corpus + operator frame +
`--seed`/`--weirdness` dials, then bridges it through the existing hypothesis gate, emitting a `DreamExportReceipt`
plus the proposed `HypothesisPacket`. If an operator supplies a `--dream-packet`, it is **refused unless it is
byte-for-byte the re-derived packet** — a tampered, stale, or foreign packet cannot be laundered into an export.
The corpus directory and the frame are read but not trusted, and — like every other local-input command — an
absolute path, a `..` traversal, or a symlink that escapes the working directory is refused before anything is read.

```sh
# 1. Generate the dream packet and bridge it into the hypothesis-only path (exact bytes — replayable):
./target/debug/cognitive-demo dream-export --input-dir corpus --frame frame.txt --out dream-export.json
# -> dream-export.json — a DreamExportReceipt (dream provenance: packet id, input hash, seed, engine version,
#    operators) + the proposed HypothesisPacket; authority_after_export "hypothesis_only"; dream_origin true;
#    no "dream_only" authority anywhere.

# 2. Render a plain operator report (re-derived from the SAME corpus + frame; a tampered export is refused):
./target/debug/cognitive-demo dream-export-report --input-dir corpus --frame frame.txt --export dream-export.json
# -> DREAM EXPORT (PROVENANCE BRIDGE — hypothesis_only, dream_origin) ... the source dream's probe requests are
#    recorded with executes: false — NEVER executed.

# 3. Confirm the export re-derives byte-identically (a determinism proof that also refuses any tamper):
./target/debug/cognitive-demo dream-export-replay --input-dir corpus --frame frame.txt --export dream-export.json
# -> dream-export-replay: OK — ... Dream origin is preserved; the exported material is hypothesis_only.
```

`dream-export-report` and `dream-export-replay` re-derive the whole export bundle from the **same** corpus + frame
+ dials and byte-compare, so a tampered receipt — including one whose `dream_origin` was flipped to `false` — is
refused (non-zero exit), never rendered or replayed. The receipt is **never parsed back into authority**: it is only
compared against the re-derived bundle, so off-wire tampering can never be laundered into a clean report or replay.
The bundle records the dream packet's origin (so the dream is auditable) but carries only the **existing**
`hypothesis_only` authority; the dream engine's private `dream_only` / `DreamOnly` authority never appears. The whole
dream export flow is exercised end-to-end by `./scripts/operator_smoke.sh` (see §3), so this documentation cannot
drift from the binary.

```text
The dream export operator path preserves provenance.
It does not create a new authority.
Exported dream material remains HypothesisOnly.
Dream origin remains auditable.
DreamOnly remains private to dream-engine.
Probe requests do not execute.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

## 15. How to exercise the data curation gate

The `data-curator` crate (frozen as the **DATA-0** ingestion gate) is the substrate's *immune system*: a
deterministic admissibility gate that classifies caller-supplied **candidate data** before anything may ingest
it. It **admits, rejects, or quarantines** each candidate item and emits an auditable `CurationReceipt` — and it
does this **without creating truth, memory, evidence, execution, promotion, or training eligibility**. The
load-bearing property is what it refuses to do. The curator reads only an explicit in-memory `CandidateManifest`
— **never the filesystem** — so there is no file to feed it and no path to traverse; the operator exercises the
real `curate()` through the crate's test suite, which constructs candidate manifests and runs the gate over each.

What the gate does, per item:

- It **rejects** an item with **missing provenance**, a **duplicate id**, empty content, an unsupported artifact
  type, durable claim-like data without a source-span grounding (a `document_span` / `corpus_span` /
  `dream_packet`), trace-derived data without a replay receipt, or an invalid split.
- It **quarantines** — *quarantined, not deleted*, and never admitted — an item carrying a **prompt-injection
  marker**, and any item caught in **train/holdout leakage** (the same content in both the train and holdout
  splits). A quarantined item is retained in the receipt for audit; it is removed from the admitted set, not
  altered or deleted.
- It **admits** only clean, grounded, single-split items — and even an admitted item is `candidate_only`, never
  training-eligible.

Training eligibility is **structurally closed**: `TrainingEligibility` defaults `Closed`, carries no
training-permitting value, and `is_eligible()` is pinned to a single `const TRAINING_PERMITTED: bool = false`, so
**no code path can return training-eligible=true**. Opening training is the job of a later gate that does not
exist yet.

```sh
# Exercise the real curator over candidate manifests (it consumes an in-memory CandidateManifest — no file IO):
cargo test --offline --manifest-path crates/data-curator/Cargo.toml
# -> all curation tests pass — the admit / reject / quarantine / leakage / determinism / never-eligible / inert
#    battery, each constructing a candidate manifest and running the real curate() over it.

# Run a single curation outcome — e.g. prove a prompt-injection marker is QUARANTINED, never admitted or deleted:
cargo test --offline --manifest-path crates/data-curator/Cargo.toml -- --exact tests::prompt_injection_is_quarantined_not_deleted_or_admitted
# -> 1 passed.
```

The curator is pure and deterministic (FNV-1a hashing, BTree ordering, no clock, no entropy, no float); its
`CurationReceipt` is `Serialize` but **not** `Deserialize`, so a receipt is re-derived by re-running `curate()`,
never trusted from off-wire bytes. The whole curation path is exercised end-to-end by
`./scripts/operator_smoke.sh` (see §3), so this documentation cannot drift from the crate.

```text
The curation operator path classifies candidate data.
It admits, rejects, or quarantines.
It does not create truth.
It does not create memory.
It does not train.
It does not execute.
It does not promote.
Training eligibility remains closed.
```

## 16. How to exercise the bounded horizon harness

The `horizon` harness (the **HORIZON-0** staged interaction harness, in `cognitive-demo`) runs bounded,
multi-step substrate interactions — **horizons `H0` through `H5`** — and proves that a longer horizon
**cannot bypass a gate a shorter one already passed**. It is a *harness, not intelligence*: every turn is one
REAL call into an already-frozen flow, and each step RECORDS what that flow returned. The operator exercises the
real harness through the crate's test suite — the harness is library-only (`run_horizon` / `horizon_matrix`,
with no CLI), exactly like the curation gate above.

The six bounded horizons, each with a hard turn ceiling (`max_turns`) and a fixed composition:

- **H0** (`max_turns` 1) — one **verified document read**.
- **H1** (`max_turns` 2) — **curate a document candidate**, then read it.
- **H2** (`max_turns` 2) — **curate a corpus candidate**, then a multi-document read.
- **H3** (`max_turns` 2) — a verified **corpus read**, then a **dream packet** grounded on it.
- **H4** (`max_turns` 3) — corpus read → dream packet → **dream export** into the hypothesis-only path.
- **H5** (`max_turns` 3) — **curation + corpus read + dream-export matrix** in one bounded trace.

Each horizon produces a `HorizonTrace` whose every step records the REAL receipt it observed: the input and
output hashes, the authority state, the curation status (where candidate data is used), and the replay status
(where a trace-derived artifact is re-derived). The harness OBSERVES; it never asserts. Because a horizon can
advance a turn ONLY by calling the real gate, **longer horizons cannot skip curation, cannot skip grounding, and
cannot skip replay** — the only way to reach turn N is to have passed the gate at turn N-1, and that gate's real
receipt is what the step records.

The boundaries hold at every depth:

- **Curation is never bypassed** — H1/H2/H5 run the real DATA-0 `curate()` over a candidate manifest BEFORE the
  read or export, and the step records the admitted/rejected/quarantined disposition; uncurated candidate data
  is never ingested.
- **Grounding is never bypassed** — every read step starts from a verifier-passed receipt; the dream packet
  grounds on a verified corpus read internally and fails closed if it does not verify.
- **Replay is never bypassed** — every step is re-derived and byte-compared; a tampered trace is refused.
- **Dream and hypothesis material never become evidence** — the strongest authority any horizon reaches is the
  EXISTING `hypothesis_only` of a dream export; the dream packet's own authority stays private to the engine;
  no step promotes anything or creates evidence.
- **Training eligibility remains closed** — the P12 verdict is read before AND after every horizon and proven
  unmoved (`training_justified=false`); no depth opens training.

`HorizonTrace` is `Serialize` but **not** `Deserialize`: a horizon record is re-derived by re-running the
harness and byte-compared (`verify_horizon_json` / `verify_horizon_matrix_json`), never trusted from off-wire
bytes.

```sh
# Exercise the whole bounded-horizon harness (it composes the real frozen flows — no file IO):
cargo test --offline --lib --manifest-path crates/cognitive-demo/Cargo.toml horizon::
# -> all horizon tests pass — H0..H5, the bounded turn counts, the allowed-module whitelist, and the six gate
#    invariants (curation/grounding/replay never skipped, no promotion to evidence, training never opens,
#    forbidden escalation refused), each running the real run_horizon() over the fixed fixtures.

# Run a single horizon — e.g. prove H4's dream export stays hypothesis_only and promotes nothing:
cargo test --offline --lib --manifest-path crates/cognitive-demo/Cargo.toml -- --exact horizon::tests::horizon_h4_dream_export_stays_hypothesis_only
# -> 1 passed.
```

The whole horizon path is exercised end-to-end by `./scripts/operator_smoke.sh` (see §3), so this documentation
cannot drift from the harness.

```text
The horizon operator path exercises bounded interaction depth.
It does not train.
It does not execute external actions.
It does not create truth.
It does not create memory.
It does not promote hypotheses.
It does not grant new authority.
Longer horizons cannot bypass earlier gates.
Training eligibility remains closed.
```

## 17. Authority boundaries

These are the load-bearing invariants the whole prototype preserves. Each is enforced mechanically by
`./scripts/release_check.sh` from the artifacts' own bytes.

1. **Reading verifies.** Only the reading receipt is proven; everything downstream cites it by hash.
2. **Hypothesis proposes.** A hypothesis is a proposal, not a claim, and never becomes authority on its own.
3. **A probe is queued, never executed.** Even when governance approves, the execution intent stays
   `requires_operator` (`nothing_executed=true`). **Approval is a decision recorded for a human, not
   execution.**
4. **An observation is quarantined, never evidence.** It stays `requires_review` / `observation_only`; it
   is never recorded as evidence.
5. **Promotion is refused.** A promotion-to-evidence request is rejected (`grants_promotion=false`). An
   observation is not evidence.
6. **Re-derive, never trust.** Every surface that accepts a file (`report`, `replay`, `ask`,
   `bundle-verify`, `scenario-verify`, `scenario-matrix*`, `failure-verify`) verifies by re-deriving the
   canonical artifact and byte-comparing. No record derives `Deserialize`, so off-wire tampering can never
   be laundered into a clean report, replay, answer, bundle, matrix, or pack.
7. **No model in the loop.** The prototype is fully deterministic; any future model could only PROPOSE
   through the frozen hypothesis layer — it can never ground a claim, mutate memory, execute a probe,
   promote evidence, or self-authorize.

## 18. Training status

Weight training is **closed**. The P12 training verdict is `training_not_justified` — the
`TrainingDecision.training_justified` bit is `training_justified=false` — and every layer reads that
verdict before and after building its artifacts and proves it unchanged. P13–P15 (LoRA candidate, shadow
mode, promotion gate) stay closed under every freeze. Training stays forbidden until the P11 eval proves a
stable, recurring model failure that survives fixes to task spec, schema, prompt, examples, tooling,
context, and verifier design. This manual makes no claim that training has opened.

## 19. Next possible work

This manual is a comprehension and reproducibility checkpoint, not a new capability. Possible future work
(none started, none authorized here):

- **Parameterizing the demo corpus** beyond the single fixed bridge scenario has begun: the document flow
  (§11) traces a local operator-supplied document, and the corpus flow (§12) traces a local directory of `.txt`
  documents — both under the same re-derive-not-trust discipline, with the corpus hash-bound as a whole. Further
  document- or corpus-flow work (scenario packs, ranking) would extend those surfaces, not open a new authority.
- **A hypothesis-only novelty layer** (§13) now sits above the corpus flow: it proposes assumption-breaking
  candidates bound to verified receipts, with authority `hypothesis_only`. It is deliberately deterministic
  and model-free. Any stronger novelty engine (e.g. a model-backed proposer) would be a new sprint under the
  same cadence and would still only *propose* — never prove, execute, promote, or train.
- **A dream provenance bridge** (§14) now lets a terminal dream packet from the frozen `dream-engine` cross into
  that same hypothesis-only path while preserving its dream origin — *without* creating a new authority type. The
  exported material stays `hypothesis_only`, the dream engine's private `dream_only` authority never crosses, and
  the bridge proves but does not promote: nothing it exports becomes evidence, a promotion, or training. Any
  richer dream export work (ranking, review, scenario packs) would extend that surface, not open a new authority.
- **RDT-0 (recurrent-depth)** material is noted as future inspiration only; it is not started.

Any future capability must go through the same cadence that produced every layer above — a written rubric,
a green and byte-silent `./scripts/release_check.sh`, live sabotage of the new gate, and an independent
read-only adversarial panel — and must leave `training_justified=false` unless the P11 eval justifies
otherwise. P13–P15 do not open without that justification.

## Boundary

```text
The manual explains the prototype.
It does not expand the prototype.
It does not create authority.
It does not execute.
It does not promote.
It does not train.
```
