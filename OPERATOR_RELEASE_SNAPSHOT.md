# Operator Release Snapshot — Cognitive OS Prototype (local, post-OPS-1)

> A **local** release snapshot of the Cognitive OS prototype state after OPS-1. It records, in one place,
> exactly what is frozen, which commit and tags recover it, which commands verify it, what the prototype
> can and cannot do, and that training stays closed. It is a *docs-only* record. It does **not** publish
> or push anything to a remote, it adds no behavior, and it grants no authority. Every commit, tag, and
> command below was read from the actual repository state, not recalled.

## 0. Snapshot identity (the state this records)

```text
snapshot of      the post-OPS-1 prototype state
HEAD commit      c33dea7  (c33dea77642cbe26e89e2fbb32e023f6915a762f)  — OPS-1: operator smoke / manual drift guard
date             2026-06-20
scope            local repository only — no remote push, no publication, no release upload
release_check    green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
operator_smoke   OK — the documented operator path runs and the manual matches the binary
training gate    training_not_justified  (P12 training_justified=false) — weights forbidden
recover state    git checkout c33dea7      (or any frozen tag below)
```

> **What "HEAD commit `c33dea7`" means here.** This snapshot describes the prototype *capability* state
> frozen at OPS-1 (`c33dea7`). The commit that *adds* this snapshot is a docs-only child of `c33dea7` — it
> adds this file (and a short charter entry and a gate lock) and changes no crate, no behavior, and no
> capability. To confirm on any later HEAD that the frozen tags and boundaries are unmoved, re-run the
> verification commands in §2; they do not depend on which commit is checked out.

## 1. Frozen milestones and recovery tags

Each layer is frozen as an annotated point in history. Recover any milestone with `git checkout <tag>`;
each tag has a freeze-record document. The commits below are the actual tag targets in this repository.

| Tag | Commit | Freezes | Record |
| --- | --- | --- | --- |
| `cognitive-os-governance-v0.1` | `bbd1113` | the v0.1 governance / evidence-contract lineage | `GOVERNANCE_MILESTONE.md` |
| `reading-track-v0.1` | `f6fa55a` | the reading verifier + verified reading receipt | `READING_TRACK_MILESTONE.md` |
| `hypothesis-track-v0.1` | `bb20acf` | the propose → probe → review → intent → observation → promotion-refusal chain | `HYPOTHESIS_TRACK_MILESTONE.md` |
| `integration-demo-v0.1` | `95b586d` | the operator-visible demo (trace / report / questions / bundle) | `INTEGRATION_DEMO_MILESTONE.md` |
| `multi-trace-validation-v0.1` | `460be0c` | the multi-trace validation pack (scenarios / matrix / failure injection) | `MULTI_TRACE_VALIDATION_MILESTONE.md` |

Full tag targets (for exact recovery):

```text
cognitive-os-governance-v0.1   -> bbd1113dbd9ccfbe398594959f20d026ed64efdd
reading-track-v0.1             -> f6fa55a5980b92295b6a4e21512834c5ee0ba5af
hypothesis-track-v0.1          -> bb20acfb40071431b243b13bdee2508fbe50b33f
integration-demo-v0.1          -> 95b586d3fb7b138b261153393aa3691e0defcd02
multi-trace-validation-v0.1    -> 460be0c66076e4b1be866f997df5a62605f2987b
```

```sh
# Recover any milestone (read-only checkout of the frozen point):
git checkout reading-track-v0.1
# Or recover the exact post-OPS-1 state this snapshot records:
git checkout c33dea7
```

## 2. How to verify this snapshot

Two commands re-verify everything this snapshot claims. Both are deterministic, offline, and read-only;
neither creates authority, executes a probe, promotes anything, or trains.

```sh
# 1. The release gate — green + byte-silent means every milestone lock and boundary still holds:
./scripts/release_check.sh ; echo "exit=$?"
# -> exit=0   (and nothing printed)

# 2. The operator smoke — runs the whole documented operator path against the built binary and
#    fails closed if the manual has drifted from the binary:
./scripts/operator_smoke.sh ; echo "exit=$?"
# -> operator-smoke: OK — the documented operator path runs and the manual matches the binary
# -> exit=0
```

The operator manual (`OPERATOR_MANUAL.md`) documents every individual `cognitive-demo` command
(`trace`, `report`, `replay`, `questions`, `ask`, `bundle`/`bundle-verify`, `scenarios`,
`scenario-pack`/`scenario-verify`, `scenario-matrix`/`-report`/`-verify`, `failure-cases`/`failure-pack`/
`failure-verify`) with real flags and the eight audit-question slugs.

## 3. What the prototype can do (the artifacts that prove the operator path)

A deterministic Rust prototype and testbed. The top layer, `crates/cognitive-demo`, is a thin, fully
deterministic surface over two deeper frozen tracks. There is **no model in the loop** — every output is a
pure function of fixed inputs. It can:

1. **Produce one canonical end-to-end trace** — a verified reading receipt, a hypothesis that cites it by
   hash, a queued probe, a governance review, an execution intent that stays `requires_operator`, a
   quarantined observation, and a refused promotion (`trace --out`).
2. **Report and replay that trace** — render every stage with ids/hashes, and confirm a file is the
   byte-identical canonical trace; a tampered or foreign trace is refused, never rendered (`report`,
   `replay`).
3. **Answer a finite, enumerated set of audit questions** about the trace — eight fixed slugs, no
   free-form path; an unknown slug fails closed (`questions`, `ask`).
4. **Package a reproducible bundle** purely derived from the canonical trace, and re-derive-and-verify it
   byte-for-byte (`bundle`, `bundle-verify`).
5. **Run the trace under several deterministic scenarios** that vary the path but not the authority, and
   verify them (`scenarios`, `scenario-pack`, `scenario-verify`).
6. **Summarize coverage as a matrix** (4 scenarios × 4 boundary cells = 16/16) and verify it re-derives
   (`scenario-matrix`, `scenario-matrix-report`, `scenario-matrix-verify`).
7. **Forge-then-reject forbidden authority** — a curated set of forged-authority attacks, each refused by
   the existing re-derive-and-byte-compare verifier (`failure-cases`, `failure-pack`, `failure-verify`).

Every operator surface that accepts a file **re-derives the canonical artifact and byte-compares** — no
record derives `Deserialize`, so off-wire tampering can never be laundered into a clean result.

## 4. What the prototype cannot do (the boundaries that stay closed)

- It is **not a trained model** and contains no learned weights. No training has run.
- It does **not execute probes.** A probe is classified and queued; the execution intent records
  `requires_operator` and never becomes `executed`. Approval is a decision recorded for a human, not
  execution.
- It does **not turn observations into evidence.** An observation is quarantined
  (`requires_review` / `observation_only`); it is never recorded as evidence.
- It does **not promote anything.** A promotion-to-evidence request is refused
  (`grants_promotion=false`).
- It does **not mutate reading memory** or alter a verifier receipt.
- It is **not production-ready**, and this snapshot is **not a remote release**: nothing here is pushed,
  published, or uploaded anywhere.

## 5. Training status (P12)

Weight training is **closed**. The P12 training verdict is `training_not_justified` — the
`TrainingDecision.training_justified` bit is `training_justified=false` — and every layer reads that
verdict before and after building its artifacts and proves it unchanged. **P13–P15** (LoRA candidate,
shadow mode, promotion gate) stay **closed** under every freeze. Training stays forbidden until the P11
eval proves a stable, recurring model failure that survives fixes to task spec, schema, prompt, examples,
tooling, context, and verifier design. This snapshot makes no claim that training has opened.

## 6. What this snapshot is not

- **Not a remote release.** No push, no publication, no package upload, no deployment. It is a local
  record in the repository.
- **Not authority.** It records state; it grants nothing and changes no boundary.
- **Not a capability change.** It adds no code, no command, and no behavior. The crates are byte-identical
  to `c33dea7`.
- **Not a trained-model release.** There are no weights to release.

Any future capability must go through the same cadence that produced every layer above — a written rubric,
a green and byte-silent `./scripts/release_check.sh`, live sabotage of the new gate, and an independent
read-only adversarial panel — and must leave `training_justified=false` unless the P11 eval justifies
otherwise. P13–P15 do not open without that justification.

## Boundary

```text
The snapshot records the prototype state.
It does not release remotely.
It does not create authority.
It does not execute.
It does not promote.
It does not train.
```
