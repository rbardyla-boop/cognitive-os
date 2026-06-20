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
```

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
(it must exit 0 and print nothing — see §11/§12). Each tag has a freeze record document.

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

## 11. Authority boundaries

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

## 12. Training status

Weight training is **closed**. The P12 training verdict is `training_not_justified` — the
`TrainingDecision.training_justified` bit is `training_justified=false` — and every layer reads that
verdict before and after building its artifacts and proves it unchanged. P13–P15 (LoRA candidate, shadow
mode, promotion gate) stay closed under every freeze. Training stays forbidden until the P11 eval proves a
stable, recurring model failure that survives fixes to task spec, schema, prompt, examples, tooling,
context, and verifier design. This manual makes no claim that training has opened.

## 13. Next possible work

This manual is a comprehension and reproducibility checkpoint, not a new capability. Possible future work
(none started, none authorized here):

- **Parameterize the demo corpus** beyond the single fixed bridge scenario, still under the
  re-derive-not-trust discipline.
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
