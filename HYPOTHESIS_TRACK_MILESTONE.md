# Hypothesis Track Milestone: HYP-0 → HYP-5 (FROZEN for hypothesis-track-v0.1)

> Status: **FROZEN** as of `hypothesis-track-v0.1`. This document freezes the HYP-0
> through HYP-5 post-reading hypothesis-track arc as a named, auditable milestone before
> any further capability work. It is the single milestone-freeze record for the
> hypothesis track; the per-sprint engineering narrative lives in `a.md` (the HYP
> checklist and detail sections) and the per-sprint decisions in
> `docs/PROJECT_CHARTER.md`. This file freezes the arc, the commit lineage, the
> boundaries, the verification discipline, the training-gate verdict, the honest
> residuals, and the frozen-status declaration. It does not restate the per-sprint
> detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            hypothesis-track-v0.1
points at      the HYP-6 freeze commit (this document + its gate lock)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr; PATH=/usr/bin)
recover        git checkout hypothesis-track-v0.1
training gate  training_not_justified (P12 training_justified = false) — weights forbidden
sits above     reading-track-v0.1 @ f6fa55a (frozen) and cognitive-os-governance-v0.1 (frozen)
```

The hypothesis track is a fully deterministic system: no model is in the loop. The
`hypothesis-layer` crate depends on serde only — nothing that could execute a probe,
mutate reading memory, alter a verifier receipt, or change the P12 training verdict. It
is a PROPOSER layer: it may create, score, trace, classify, review, and record
dispositions, and nothing else.

## 1. What is frozen — the commit lineage

Six feature commits form the arc, plus the post-HYP-5 charter status snapshot. Each
commit is additive in the EXISTING `crates/hypothesis-layer` crate (no new dependency,
so the serde-only quarantine is unchanged across the arc). The hashes are auditable
against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| HYP-0 | hypothesis-only abductive layer: `HypothesisPacket` (propose / score / trace), authority `hypothesis_only` (the only variant), baked `FORBIDDEN_USES` | `f19a998` |
| HYP-1 | probe queue / human-review boundary: `ProbeRequest` / `ProbeQueue`, deterministic clearance — a high-risk or irreversible probe is escalated or blocked, never auto-run | `4b47736` |
| HYP-2 | governance review receipt boundary: `ReviewReceipt` (approved / rejected / deferred) — records a decision, executes nothing; a blocked probe can never be approved | `cb68a73` |
| HYP-3 | approved-probe execution stub / non-execution boundary: `ProbeExecutionIntent` (`not_executed` / `blocked` / `requires_operator`) — there is no `executed` state | `6cbb3a8` |
| HYP-4 | observation receipt quarantine: `ProbeObservationReceipt` (`rejected` / `requires_review`; `recorded` is future-reserved and unreachable) — `observation_only` authority | `7703e2e` |
| HYP-5 | observation promotion gate / still-no-evidence boundary: `PromotionRequest` (`rejected`; `requires_verifier` / `unsupported` future-reserved) — no status grants a promotion | `cef91db` |
| record | Cognitive OS prototype status snapshot after HYP-5 (charter `DD-2026-06-19-B`) | `d899a61` |

## 2. The boundaries that hold across the arc

These are the load-bearing invariants the whole track preserves. None was weakened by a
later sprint; each later sprint is additive over them.

1. **Hypotheses are proposals, never claims (HYP-0).** A `HypothesisPacket` carries
   `Authority::HypothesisOnly` (a single-variant enum, so any other authority is
   unrepresentable) and bakes the canonical `FORBIDDEN_USES`, so it can never ground a
   claim, serve as evidence, mutate reading memory, alter a verifier receipt, change the
   P12 verdict, or bypass codec/governance. Scoring is deterministic integer math; the
   packet is non-deserializable and minted only by `propose`.
2. **Probe requests are classified, never evidence (HYP-1).** A `ProbeRequest` derives a
   deterministic clearance from risk and reversibility; a high-risk OR irreversible probe
   is escalated to human review, and a high-risk AND irreversible one is blocked. The
   queue orders and classifies; it authorizes nothing.
3. **Review is a decision, never execution (HYP-2).** `ReviewReceipt::decide` records
   approved / rejected / deferred. A `blocked` probe can never be approved by ANY
   authority; a review-required probe needs a human/governance authority. Approval is a
   record for a human to act on LATER — it executes nothing.
4. **Execution intents record, never execute (HYP-3).** A `ProbeExecutionIntent` is
   DERIVED from the review: only an approved review yields a cleared (`not_executed` /
   `requires_operator`) intent; a rejected/deferred review yields `blocked`. There is no
   `executed` state. The crate runs no probe — proven by a crate-wide
   no-process/filesystem/network scan over `src/` and the examples.
5. **Observations are quarantined, never recorded or evidence (HYP-4).** A
   `ProbeObservationReceipt` is `observation_only`. A `not_executed`/`blocked` intent
   yields `rejected`, a `requires_operator` intent yields `requires_review`, and NO intent
   yields `recorded` — `recorded` is the future-reserved promotion target, unreachable at
   HYP-4, so nothing can be recorded. An observation does not imply the probe ran.
6. **Promotion requests record, never promote (HYP-5).** A `PromotionRequest` is DERIVED
   from the observation and the requested target. A `rejected`/`requires_review`
   observation yields `rejected` (for any target); the future-reserved `recorded`
   observation yields `requires_verifier` (claim/evidence) or `unsupported` (memory-note).
   No status grants a promotion (`grants_promotion` is exhaustive and always `false`), so
   an observation does not become evidence just because it exists. "Still no evidence"
   holds until a future verifier-backed promotion path exists.
7. **Structural quarantine across the arc.** Every inert output type has private fields
   and read-only accessors, derives `Serialize` but NOT `Deserialize` (compiler-proven by
   `compile_fail` doctests, pinned live by cargo's doctest report), is minted only by its
   single derive-only constructor, and binds its fields with an FNV-1a `integrity_hash`.
   Scoring/derivation is deterministic integer math with no floats, wall-clock, or
   entropy, so trace replay reproduces every record. Each type reuses `FORBIDDEN_USES`,
   and dev-only tests prove no sprint changes a verifier receipt or the P12 verdict.

## 3. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is
`false`). The hypothesis track is orthogonal to P12 and does not move it: every HYP sprint
proves, in a dev-only test, that generating its disposition leaves the training decision
identical. Weight training stays forbidden until the P11 eval proves a stable, recurring
model failure that survives fixes to task spec, schema, prompt, examples, tooling,
context, and verifier design. P13–P15 (LoRA candidate, shadow mode, promotion gate) stay
closed under this freeze.

## 4. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0
bytes stdout, 0 bytes stderr). The hypothesis-track blocks gate, for the whole crate,
`cargo test` + `cargo fmt --check` + `cargo clippy -D warnings`, plus: the serde-only
quarantine `cargo tree` (no `vibe-*` engine crate, no reading crate, no ML crate); a
crate-wide cargo doctest-REALITY pin (exact live + `compile fail` counts, one per inert
type) and a unit-test-REALITY pin (exact passed count, zero ignored); per-type private-
fields and manual-`impl Deserialize` scans; the crate-wide no-process / no-float /
no-wall-clock / no-IO / no-`#[allow]` scans; HYP-5's sole-minting-path construction-
literal pin; and per-sprint behavioral example double-runs that grep the real serialized
dispositions. The acceptance discipline for every sprint in this arc was: rubric → green
byte-silent `release_check` → a live sabotage proving the gate catches a regression
(restored byte-identical by md5) → an independent read-only adversarial verifier panel
with a fresh context → any residual folded before close.

## 5. Independent verification

Every sprint HYP-0 through HYP-5 was closed against read-only adversarial panels (Explore
agents, refute-by-default) covering forgery/policy, no-execution/no-mutation, provenance/
integrity, determinism/replay, cannot-be-evidence, and gate-vacuity lenses, run until a
fully-dry round. Two sprints drove real gate strengthenings that were reproduced
first-hand before folding: HYP-3 bound all four execution-reason tokens to the real
behavioral output (closing an asymmetry where two were guarded only by fabricable
booleans); HYP-5 added a sole-minting-path pin and, after a panel showed a return-type
count was evadable by a composite return type, replaced it with a construction-literal pin
that catches a backdoor of any return-type shape (sound because the crate is
`#![forbid(unsafe_code)]`, the type has no `Deserialize`, and its fields are private, so a
struct literal is the only construction path). Every claim in this document is checkable by
running `scripts/release_check.sh` and reading the named commits.

## 6. Honest residuals (NOT closed in hypothesis-track-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known
edge of the deterministic hypothesis model, not bugs.

1. **Multi-file insider forgery is out of scope.** The structural quarantine defends
   against off-wire forgery (untrusted input) and accidental regression, both of which the
   gate provably catches. It does NOT defend against an insider with commit access who
   authors new malicious code AND rewrites the gate in the same change — that is
   review-evident multi-file forgery, beyond regression scope, and is the domain of code
   review and the governance/signing layer, not a deterministic build gate.
2. **The promotion path is deliberately empty.** `recorded` observations,
   `requires_verifier`, and `unsupported` are future-reserved dispositions: no
   verifier-backed promotion path exists yet, so at this freeze nothing can be recorded or
   promoted. This is the still-no-evidence boundary by design, not an unfinished feature.
3. **No model in the loop.** The hypothesis track is fully deterministic. Any future model
   may only PROPOSE; it can never ground a claim, mutate memory, execute a probe, or
   self-authorize. The P10 adapter stays gated shut by P12.
4. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a
   production reasoning system.
5. **Process caveat (verification method).** The read-only adversarial panels twice left
   stray debris in the working tree (a test file in an earlier track; a compiled
   `test_alias` binary during HYP-5) despite their read-only instruction; each was
   untracked, unreferenced, and removed before close. It is a known operational caveat of
   the panel method, not a property of the frozen code.

## 7. Frozen-status declaration

The HYP-0 → HYP-5 hypothesis-track arc is **FROZEN at `hypothesis-track-v0.1`**. The
authority boundary is the frozen surface:

```text
Hypothesis proposes.
Probe queue classifies.
Governance reviews.
Execution intent does not execute.
Observation is quarantined.
Promotion request does not promote.
Nothing becomes evidence.
```

Any change that lets a hypothesis become a claim, a probe request become evidence, a
review become execution, an execution intent execute, an observation become recorded or
evidence, or a promotion request promote — or that reopens training — must pass through the
same machinery: a rubric, a green byte-silent `release_check.sh`, a live sabotage, and an
independent adversarial panel, and must leave `training_justified = false` unless a clean
recurring model failure is proven. Relaxing any criterion requires explicit operator
sign-off; it must not be edited mid-stream to make a failing check pass. P13–P15 do not
start under this freeze.
