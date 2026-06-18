# Project Charter — Cognitive OS

Significant architectural decisions for the Cognitive OS prototype. Newest first. Each entry
links to the canonical artifact that records the decision in full.

## DD-2026-06-18-C — Add the governance review receipt boundary (P16 / HYP-2) in-crate

**Decision.** Add `crates/hypothesis-layer/src/review.rs` — a `ReviewReceipt` recording the governance
decision (approved / rejected / deferred) on a HYP-1 `ProbeRequest`, WITHOUT executing the probe or
mutating anything. Doctrine: *Hypothesis proposes. Probe queue classifies. Governance reviews. Nothing
executes. Nothing becomes evidence.* Kept INSIDE the existing crate (a new module, no new dependency), so
the serde-only quarantine is unchanged.

**Why.** HYP-1 creates inert probe queue items; the next boundary is an explicit, machine-checkable
governance decision that keeps human/governance authorization explicit before any future execution layer
exists. A receipt is minted only by `decide`, which enforces the policy: a blocked probe can never be
approved by any authority; a human_review_required probe needs Human/Governance authority (never
Automated); a queued probe may be approved but approval is a record for a human to act on later, not an
execution. `ReviewerAuthority` is a checked enum, never a free string.

**Boundary (enforced by the compiler, types, the gate, and a behavioral backstop).** A
`ReviewReceipt`/`ReviewLog` is minted only by `decide`/`from_receipts`, has private fields, and derives
`Serialize` but not `Deserialize` (compile_fail proofs, pinned live by cargo's doctest report; `ReasonCode`
is output-only to keep the receipt non-deserializable) — so a forged decision cannot be deserialized off
the wire or built from a raw struct. The receipt binds its fields with an `integrity_hash`, cites its
provenance, and reuses the forbidden-uses quarantine so it can never become evidence. No execution code
exists in the crate (crate-wide gate scan). Verified by three read-only adversarial panel rounds (five
substantive lenses clean; one determinism finding reproduced and refuted; the gate-vacuity lens drove two
first-hand-reproduced folds — a cargo unit-test-reality pin closing an `#[ignore]` test-disable bypass, and
a behavioral example backstop that re-runs the real `decide()` on the forbidden paths so the policy holds
even if the unit tests were gutted; round three fully dry) plus four live sabotage probes. No LLM, no
training, no probe execution; P12 still owns weights, P13–P15 stay closed. `release_check` green + silent.
Recorded in full in [a.md](../a.md) under "Governance Review Receipt Boundary (P16 / HYP-2)". Additive:
HYP-0, HYP-1, and all prior crates/docs 0-diff. Local only — no remote push.

## DD-2026-06-18-B — Add the probe queue / human-review boundary (P16 / HYP-1) in-crate

**Decision.** Add `crates/hypothesis-layer/src/probe.rs` — a `ProbeRequest` queue derived from a
`HypothesisPacket`'s recommended probe, with an explicit machine-checkable review status
(`queued` / `human_review_required` / `blocked`) — WITHOUT executing the probe or mutating anything.
Doctrine: *Hypothesis proposes a probe. HYP-1 queues or blocks it. Human/governance decides execution.
Nothing executes automatically.* Kept INSIDE the existing crate (a new module, no new dependency), so the
serde-only quarantine is unchanged; the queue needed no separate crate for dependency hygiene.

**Why.** HYP-0 can propose a probe; the next risk is what happens to it afterwards. HYP-1 makes probe
handling explicit, replayable, bounded, and incapable of side effects. The status is DERIVED from the
packet's canonical `ProbeClearance` (HYP-1 respects the HYP-0 decision, never recomputing one), so a
high-risk or irreversible probe is escalated to review or blocked and only a `queued` probe is
execution-eligible. The queue is content-ordered (insertion-order independent) so replay reproduces it.

**Boundary (enforced by the compiler, types, and the gate).** A `ProbeRequest`/`ProbeQueue` is minted only
by `from_hypothesis(es)`, has private fields, and derives `Serialize` but not `Deserialize` (compile_fail
proofs, pinned live via cargo's own doctest report) — so a forged status cannot be hand-set or deserialized
off the wire. `is_execution_eligible` is an exhaustive no-wildcard match (E0004 on a new status variant). A
crate-wide gate scan forbids any process spawn / filesystem / network / side-effecting I/O in the crate, so
the layer provably executes nothing. No LLM, no training, no probe execution; P12 still owns weights,
P13–P15 stay closed. Verified by four read-only adversarial panel rounds (five substantive lenses clean for
four rounds; the gate-vacuity lens drove three first-hand-reproduced folds — no-execution scan added, made
crate-wide, then a cargo doctest-reality pin; round four fully dry) plus three live sabotage probes.
`release_check` green + silent. Recorded in full in [a.md](../a.md) under "Probe Queue / Human Review
Boundary (P16 / HYP-1)". Additive: HYP-0 and all prior crates/docs 0-diff. Local only — no remote push.

## DD-2026-06-18-A — Open the hypothesis-only abductive layer (P16 / HYP-0) as a post-freeze track

**Decision.** Add `crates/hypothesis-layer` — an abductive layer ABOVE the frozen reading substrate
and BELOW human review that may CREATE, SCORE, and TRACE proposed explanations / next probes and
nothing else. Doctrine: *Probability proposes. Replay tests. Governance authorizes. Memory records.*
The core `HypothesisPacket` is inert: minted only by `propose`, private read-only fields, no
`Deserialize`, fixed `Authority::HypothesisOnly` (single-variant enum), a baked canonical
`FORBIDDEN_USES` set, receipt citations by content hash, deterministic integer scoring, and a
replay that re-derives the packet from its `HypothesisSpec`. This is a **new post-freeze track,
additive** to `reading-track-v0.1`, not part of the P0–P15 prototype track.

**Why.** The reading substrate grounds answers only from cited-span evidence and forbids whatever it
cannot ground; it deliberately cannot propose. HYP-0 adds the missing faculty — proposing an
explanation or next probe that is not yet grounded — while structurally preventing a proposal from
acquiring the authority of a fact. Probability can schedule a test but can never ground an answer,
mutate memory, alter a receipt, change the training verdict, or bypass governance.

**Boundary (enforced by the compiler and types, not convention).** No LLM, no training, no semantic
judge — deterministic scoring only for v0. The quarantine is structural: production deps are serde
only, the reading crates are dev-only to prove non-interference, and the gate asserts the non-dev
tree holds no substrate/engine/ML crate. P12 still owns weights and remains "not justified"; P13–P15
stay closed. Verified by six read-only adversarial panel rounds (five substantive lenses clean for
five consecutive rounds; the gate-vacuity lens drove four rounds of compiler-backed gate hardening,
each reproduced first-hand; round six fully dry). `release_check` green + silent. Recorded in full in
[a.md](../a.md) under "Hypothesis Layer Track (P16 / HYP-0)". Local only — no remote push.

## DD-2026-06-14-C — P0: snapshot v0.1 governance as a git freeze point

**Decision.** The repo was initialized as a git repository (Option A) and the frozen v0.1
governance state tagged before any engine work begins. `release_check` was green + silent
(`PATH=/usr/bin`) at snapshot time.

```text
tag     cognitive-os-governance-v0.1
commit  bbd1113dbd9ccfbe398594959f20d026ed64efdd
recover git checkout cognitive-os-governance-v0.1
```

Local only — no remote was added and nothing was pushed (a remote push needs separate
authorization per the project security rule). Recorded in
[GOVERNANCE_MILESTONE.md](../GOVERNANCE_MILESTONE.md) §0. P1 (Rust `crates/vibe-core`) may begin
from this freeze point.

## DD-2026-06-14-B — Adopt the prototype-first engine track (ADR-002 L0–L2), additive

**Decision.** The forward direction is prototype-first: build the minimal deterministic runtime
engine chartered by [ADR-002](../ADR-002-runtime-engine-replay-contract.md) — the L0 kernel, L1
ingress/scheduling/frames, and L2 run/record/replay — then add a replaceable LLM language codec at
the human-language boundary (never inside the kernel). This is the **Prototype-First Track
(P0–P15)** in [a.md](../a.md).

**Additive, not replacing.** The incremental 24i–35 Python-cognition backlog remains the deferred
backlog, still gated by the unified self-correction loop. P0–P15 is the active build order.

**Why.** The v0.1 governance lineage (S24–32) proved the *evidence contract* (ADR-002 L3) that
secures engine changes. The engine those traces describe (L0–L2) is still realized as Python
scripts; the prototype track builds it underneath the L3 guardrail that already governs it.

**Rationale for ADR-002.** It was cited as the "runtime engine replay contract" by
`SPRINT_28/29/30_PLAN.md`, `DESIGN_REVIEW_NOTES.md`, and `a.md` before the charter existed; writing
it resolved that dangling reference and made the L0–L3 layer names authoritative.

## DD-2026-06-14-A — Freeze v0.1 governance milestone as the ADR-002 L3 evidence contract

**Decision.** Sprints 24–32 (derived effect → trace-grounded invariants → content binding → signed
provenance → signer governance → mechanism-source binding) are frozen as the v0.1 governance
milestone. In ADR-002's layer model this lineage **is** L3: the content-bound, signed,
mechanism-bound replay-evidence contract. Recorded in
[GOVERNANCE_MILESTONE.md](../GOVERNANCE_MILESTONE.md) (FROZEN) and
[RELEASE_REVIEW.md](../RELEASE_REVIEW.md).

**Caveats preserved (not hidden).** Single-signer authority, adjudicator-only behavioral probe,
restricted-AST-subset precision, and the who-watches-the-watchmen fixed point remain published
residuals. This is a v0.1 governance proof-of-concept, not production-ready for crypto-critical use
until those are accepted or resolved.
