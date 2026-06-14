# ADR-002 — Runtime Engine Replay Contract (the backend charter)

Status: Accepted (2026-06-14). Supersedes nothing; complements the v0.1 governance
milestone ([GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md)) and the async-bus /
epistemic-license architecture recorded in `project_birth.md`.

This ADR is referenced as the home of the "runtime engine's replay contract" by
`SPRINT_28_PLAN.md`, `SPRINT_29_PLAN.md`, `SPRINT_30_PLAN.md`, `DESIGN_REVIEW_NOTES.md`,
and `a.md`. It was previously cited ("ADR-002 L0", "the L2→L3 step in the backend
charter") before the charter itself was written; this document closes that gap and makes
the layer names those docs use authoritative.

## Context

The v0.1 work proved a **governance / evidence layer** first: a development-process change
that would weaken a locked runtime invariant is blocked by the same machinery that blocks an
unsafe runtime action (the Caitlin Leap, `COGNITIVE_OS_SELF_CORRECTING_LEAP.md`). Sprints
24–32 hardened *what evidence is trusted*: a tested delta must bind to provenance
(S28), to literal artifact content (S29), to an authorized signer evaluated at the decision
tick (S30–S31), and to the content of the enforcement code itself (S32).

That is the guardrail. It secures an evidence contract whose **subject** — the deterministic
runtime engine that produces the replayable decision traces — is still realized as Python
scripts and JSON scenarios, not as a self-contained engine with a pinned replay contract.
`project_birth.md` already chose the eventual substrate: an async event-driven bus with
correlation IDs, deadline-bound awaits, partial results, and safe degradation, with **Rust
as the better eventual substrate because packet schemas, permissions, and engine contracts
benefit from type discipline**. ADR-002 is the charter for building that engine without
letting it become a distributed-systems project, and without weakening the determinism the
evidence contract depends on.

## Decision

Build the runtime engine as a **layered replay contract**. Each layer has a single
responsibility and a hard boundary. The deterministic math kernel (L0) holds no backend,
network, storage, signing, or governance concern; those live in outer layers and may only
reach the kernel through typed, validated inputs.

### Layer L0 — Deterministic replay kernel

The pure replay math. Given a state and a frame, it produces an output and a new state, and
nothing else.

```
evaluate_tick(state, frame) -> (output, next_state)
```

Invariants (each is a checkable release gate in the eventual engine crate):

- **No wall-clock.** Time is a logical `Tick`. `evaluate_tick` never reads a system clock.
  `project_birth.md`'s `deadline_ms` budgets are reconciled as **tick budgets** in the
  kernel; wall-clock scheduling lives only in the live runtime shell (outside L0), so a
  recorded run replays identically regardless of how fast or slow the original ran.
- **No unseeded randomness.** Any stochastic behavior takes an explicit seed carried in the
  state; the same seed reproduces the same output.
- **Fixed-point / integer scalars where equality matters.** No floating nondeterminism on
  any value that feeds a hash, a comparison gate, or a replay assertion.
- **No backend dependency.** The kernel does not import HTTP, SQLite, signing, or governance
  modules. It is pure data-in / data-out and is unit-testable in isolation.

L0 is the "runtime engine's own replay-contract responsibility" that Sprints 29–32 deferred
to "ADR-002 L0": the mechanism that *enforces* a gate is itself deterministic and
content-bound, so a probe over a proposed change is a behavioral test, not an execution.

### Layer L1 — Ingress and tick scheduling

All external input becomes a typed, validated packet before it can influence the kernel.
This is the deterministic realization of `project_birth.md`'s async bus: admissibility,
correlation, and deadline-as-tick-budget are decided here, not inside the math.

```
ObservationEnvelope            external input as a typed, source-attributed packet
IngressGate                    schema / source / sequence / idempotency admission; emits receipts
TickScheduler + ScheduledObservation   assign each accepted observation to a deterministic target tick (bounded horizon)
FrameCollector + ObservationFrame       fold all observations for a tick into one canonical, hash-stable frame
```

Boundary: invalid input is rejected with a receipt and **never mutates engine state**;
overload is rejected or quarantined with a receipt, never silently dropped. The kernel only
ever sees an `ObservationFrame`, never raw input.

### Layer L2 — Run recording and deterministic replay

A run is a script of observations; recording it produces evidence that replays exactly.

```
RunScript / RecordedRun        the ordered inputs that drove a run
RunRecorder                    records accepted observations, frames, outputs, and per-tick hashes
ReplayRunner + run_hash        re-executes a recorded run and proves frame/output/hash equality
```

Boundary: replay depends only on the recorded run, never on live input; a tampered recorded
run is detected by hash mismatch. This is the engine-side counterpart of the trace-replay /
decision-audit tools that already exist for the Python prototype (`decision_audit.py`,
`TRACE_AUDIT.md`).

### Layer L3 — Content-bound, signed replay evidence (the governance layer, already built)

The replay-evidence contract **binds to artifact content, not to a name**, and to an
accountable author. This is the layer the v0.1 governance lineage already implements for the
development process:

- S28 delta-to-code provenance, S29 artifact content-hash binding — *"the L2→L3 step in the
  backend charter: the replay-evidence contract must bind to the artifact content, not just
  its name."*
- S30 signed change provenance, S31 signer-set governance (authority at the decision tick).
- S32 mechanism-source content binding — the L0 mechanism's own source is content-bound and
  probe-tested.

L3 is therefore **done first, deliberately**: the guardrail that decides which engine changes
are trustworthy exists before the L0–L2 engine it guards. The prototype-first track (P0–P15
in `a.md`) builds L0–L2 underneath the L3 contract that already governs it.

## The LLM boundary (where language sits relative to the layers)

The LLM is a **replaceable language codec at the human-language boundary**, never a layer of
the engine. It may propose typed packets (which then enter L1's `IngressGate` like any other
input) and render explanations from L2/L3 evidence. It must never become world memory, an
authority source, a mutation gateway, a verifier, the replay ledger, the scheduler, or the
state engine. Training is deferred until an eval harness proves prompting/tooling is
insufficient (the constraint-engineering discipline in the `a.md` appendix). This is `a.md`
Sprints P9–P15 and is consistent with the existing `LANGUAGE_CODEC.md` no-prose-internal-handoff
rule and Sprint 23's LLM boundary.

## Consequences

- **Positive.** The engine is replayable by construction; the determinism the evidence
  contract assumes is now enforced by kernel-level gates, not by convention. Wall-clock and
  backend concerns cannot leak into decisions. The async-bus degradation semantics of
  `project_birth.md` are preserved (partial evidence, safe degradation, deferred jobs) but
  made deterministic — *met or better*.
- **Cost.** Reconciling `deadline_ms` to tick budgets means the live runtime shell, not the
  kernel, owns wall-clock; the shell must translate real deadlines into tick budgets at
  ingress. This is intentional: it is the only place nondeterminism is allowed to exist.
- **Scope discipline.** No Kafka/NATS/Redis/RabbitMQ/microservices, no distributed backend,
  no threshold/multi-signer governance, and no production API are required to reach a working
  prototype (explicitly out of scope in the `a.md` prototype-first track). Those are post-
  prototype concerns.
- **Substrate.** Rust (`crates/vibe-core`) is the chosen L0 substrate per `project_birth.md`;
  the engine is not built in this turn — ADR-002 records the contract that the P0–P15 sprints
  build against.
