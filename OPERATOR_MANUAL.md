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

corpus-trace --input-dir DIR [--out PATH]   corpus-report --input-dir DIR --trace PATH [--out PATH]
corpus-bundle --input-dir DIR --out DIR     corpus-bundle-verify --input-dir DIR --path DIR

novelty-packet --input-dir DIR --corpus-trace PATH --frame PATH [--out PATH]
novelty-report --input-dir DIR --frame PATH --packet PATH [--out PATH]
novelty-replay --input-dir DIR --frame PATH --packet PATH
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
(it must exit 0 and print nothing — see §14/§15). Each tag has a freeze record document.

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
`failure-pack`/`failure-verify`, the document and corpus flows (`doc-*` / `corpus-*`), and the
hypothesis-only novelty flow (`novelty-packet` / `novelty-report` / `novelty-replay`) — inside a throwaway
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

## 14. Authority boundaries

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

## 15. Training status

Weight training is **closed**. The P12 training verdict is `training_not_justified` — the
`TrainingDecision.training_justified` bit is `training_justified=false` — and every layer reads that
verdict before and after building its artifacts and proves it unchanged. P13–P15 (LoRA candidate, shadow
mode, promotion gate) stay closed under every freeze. Training stays forbidden until the P11 eval proves a
stable, recurring model failure that survives fixes to task spec, schema, prompt, examples, tooling,
context, and verifier design. This manual makes no claim that training has opened.

## 16. Next possible work

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
