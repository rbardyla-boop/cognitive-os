# Integration Demo Milestone: INT-0 → INT-3 (FROZEN for integration-demo-v0.1)

> Status: **FROZEN** as of `integration-demo-v0.1`. This document freezes the INT-0
> through INT-3 integration-demo arc as a named, auditable milestone before any further
> behavior is added. It is the single milestone-freeze record for the integration track;
> the per-sprint engineering narrative lives in `a.md` (the INT checklist and detail
> sections) and the per-sprint decisions in `docs/PROJECT_CHARTER.md`
> (`DD-2026-06-19-D..G`). This file freezes the arc, the commit lineage, the frozen
> dependencies, the demonstrable capability, the output-not-authority boundary, the
> verification discipline, the training-gate verdict, the honest residuals, and the
> frozen-status declaration. It does not restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            integration-demo-v0.1
points at      the INT-4 freeze commit (this document + its gate lock)
freezes        the INT-0..INT-3 arc (head f451c39)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr; PATH=/usr/bin)
recover        git checkout integration-demo-v0.1
training gate  training_not_justified (P12 training_justified = false) — weights forbidden
sits above     reading-track-v0.1 @ f6fa55a (frozen) and hypothesis-track-v0.1 @ bb20acf (frozen)
```

The integration demo is a fully deterministic surface: no model is in the loop. The
`crates/cognitive-demo` crate consumes the two frozen tracks through their PUBLIC APIs
only and adds serde for serialization — nothing that could execute a probe, mutate
reading memory, alter a verifier receipt, promote an observation, or change the P12
training verdict. It is a DEMONSTRATION layer: it may build, serialize, report, answer
fixed audit questions, and package a reproducible bundle, and nothing else.

## 1. What is frozen — the commit lineage

Four feature commits form the arc. Each is additive in the `crates/cognitive-demo` crate
(INT-0 created it; INT-1..INT-3 extended it with no new dependency and no Cargo.toml
change after the INT-1 binary target). The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| INT-0 | End-to-End Prototype Trace Demo: one deterministic, replayable `CognitiveTrace` connecting a VERIFIED reading receipt to the full hypothesis→probe→review→intent→observation→promotion-refusal chain; `Serialize` but NOT `Deserialize`, minted only by `demo`/`build` | `2330f7c` |
| INT-1 | End-to-End Trace CLI / Operator Report: the `cognitive-demo` binary — `trace` / `report` / `replay`; `report`/`replay` RE-DERIVE the canonical trace and refuse any tampered file (`TraceMismatch`); `std::fs` confined to `main.rs` | `92c0692` |
| INT-2 | Trace Question Harness / Operator Interrogation Surface: `questions` + `ask`, a finite enum-backed question set; an unknown slug fails closed (`UnknownQuestion`), `ask` re-derives the canonical trace before answering; answers are prose, not authority | `b5bcf66` |
| INT-3 | Prototype Demo Bundle / Operator Repro Pack: `bundle` + `bundle-verify`; the pack (trace.json, report.txt, questions.txt, manifest.json) is purely derived; `bundle-verify` re-derives every file and byte-compares, refusing any tampered/missing/foreign file | `f451c39` |

## 2. Frozen dependencies

The integration arc sits ON TOP OF — and edits NEITHER of — the two frozen tracks it
consumes through their public APIs:

```text
reading-track-v0.1      @ f6fa55a   (the read0 verifier + the verified reading receipt)
hypothesis-track-v0.1   @ bb20acf   (the propose → probe → review → intent → observation → promotion-refusal chain)
```

Every integration sprint proves it touches no frozen crate source (`git diff` over
`crates/hypothesis-layer`, `crates/reading-*`, `crates/vibe-*`) and leaves both tags
unmoved. The deeper governance freeze `cognitive-os-governance-v0.1` is unchanged.

## 3. What the prototype can do (the demonstrable capability)

This is now a demonstrable prototype, not just an architecture. From fixed inputs it can,
deterministically and reproducibly:

1. **Produce a verified trace.** Run a reading plan through the read0 verifier to a
   passing receipt, then derive one end-to-end `CognitiveTrace` that cites that receipt by
   hash and walks the full hypothesis chain to a refused promotion (INT-0).
2. **Show the operator what happened.** Render a plain operator report of every stage with
   the ids/hashes needed to audit and replay (INT-1).
3. **Answer fixed audit questions.** Answer a finite, enumerated set of audit questions
   about the trace — what was read, what was proven, what was hypothesized, whether
   anything executed, whether anything became evidence, why promotion was refused, whether
   training opened — with no LLM and no natural-language parser (INT-2).
4. **Package a reproducible bundle.** Write a four-file repro pack and verify it by
   re-deriving every file and byte-comparing, refusing any tamper (INT-3).

Every one of these is output ABOUT the canonical trace. None of them is authority.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a
later sprint; each later sprint is additive over them, and each is enforced by the release
gate from the artifacts' own bytes.

1. **The trace is output, not authority (INT-0).** `CognitiveTrace` derives `Serialize`
   but NOT `Deserialize`, has private fields and read-only accessors, and is minted only
   by `demo`/`build` from the frozen crates' real outputs. It records ids, hashes, and
   machine-checkable verdicts; it cannot be forged or mutated into claiming an execution,
   an evidence promotion, or an opened training gate.
2. **The report is output, not authority (INT-1).** `report`/`replay` never parse a
   provided file back into authority: `verify_trace_json` RE-DERIVES the canonical trace
   via `CognitiveTrace::demo()` and byte-compares the provided file, refusing any
   tampered/stale/foreign input (`TraceMismatch`). `to_report` is pure formatting of the
   re-derived trace's fields — it computes no new verdict and grants no authority.
3. **Questions explain the trace (INT-2).** The question set is a finite `TraceQuestion`
   enum; `from_slug` is exact-match and an unknown slug fails closed (`UnknownQuestion`)
   before any trace is consulted. `ask` re-derives and verifies the trace before
   answering, and every answer is prose formatted from the trace's recorded fields — never
   a new verdict, never authority.
4. **The bundle demonstrates the prototype (INT-3).** The repro pack is purely derived from
   the canonical trace; `bundle-verify` re-derives every file (including the manifest) and
   byte-compares, so a missing/tampered/foreign file is refused. The bundle is a
   reproducible demonstration; it is never trusted as authority.
5. **Re-derive, never trust (the arc-wide discipline).** Every operator surface that
   accepts a file (report, replay, ask, bundle-verify) verifies by re-deriving the
   canonical artifact and byte-comparing — never by parsing the provided bytes into trusted
   state. No record in the crate derives `Deserialize`. So off-the-wire tampering can never
   be laundered into a clean report, a passing replay, an answer, or a verified bundle.
6. **No new authority across the arc.** The integration crate executes no probe (no
   process/network anywhere in `src` or the examples), promotes nothing
   (`grants_promotion = false`), mutates no memory, and moves no training verdict. The
   canonical flow is the strongest honest case: governance APPROVES the probe, yet
   execution stays `requires_operator` (never `executed`), the observation stays
   `requires_review` / `observation_only` (never `recorded`), and the promotion-to-evidence
   request is `rejected`. Approval is not execution; an observation is not evidence.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is
`training_justified=false`). The integration track is orthogonal to P12 and does not move
it: every INT sprint reads the training decision before and after building its artifacts
and proves it identical, and leaves the reading verifier receipt byte-identical. Weight
training stays forbidden until the P11 eval proves a stable, recurring model failure that
survives fixes to task spec, schema, prompt, examples, tooling, context, and verifier
design. P13–P15 (LoRA candidate, shadow mode, promotion gate) stay closed under this
freeze.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0
bytes stdout, 0 bytes stderr). The integration blocks gate, for the whole
`cognitive-demo` crate, `cargo test` + `cargo fmt --check` + `cargo clippy -D warnings`,
plus, per sprint: an encapsulation pin (no `derive(...Deserialize)`, no manual
`impl Deserialize`, zero `pub` struct fields on `CognitiveTrace`); API-exercise greps
proving the demo really calls the frozen crates (not a hardcoded artifact); a unit-test
REALITY pin (exactly 44 passed, zero ignored) and per-test name pins; crate-wide purity
and no-probe-execution scans; an fs-confined scan keeping `std::fs` out of the library and
examples; separation `cargo tree` (the two frozen tracks present, no `vibe-*` engine crate,
no ML crate); and end-to-end binary smokes that drive every command against real temp files
and grep the real serialized output — including a precise no-grant guard, the re-derive /
tamper-refusal checks for report/replay/ask/bundle-verify, verbatim boundary loops, and a
distinct-hash check on the bundle manifest. This milestone block additionally pins the
freeze record itself (this document's commit lineage, frozen-dependency references,
boundary lines, and the `training_not_justified` verdict). The acceptance discipline for
every sprint in this arc was: rubric → green byte-silent `release_check` → live sabotage
probes proving the gate catches a regression (restored byte-identical by md5, never `git
checkout`) → an independent read-only adversarial verifier panel with a fresh context →
any residual folded before close.

## 7. Independent verification

Every sprint INT-0 through INT-3 was closed against read-only adversarial panels (Explore
agents, refute-by-default, no-compile-to-disk, each driving the compiled binary under a
temp dir), run until a fully-dry round with zero real findings and no debris. Several
sprints drove gate strengthenings that were reproduced first-hand before folding: INT-0
replaced a prose-tripping `Deserialize` grep with precise `derive`/`impl` scans; INT-2
added a verbatim five-line boundary loop after noticing the boundary smoke pinned only two
lines; INT-3 added a distinct-hash check after noticing the manifest-hash test was
self-referential. Each strengthening was proven load-bearing by a sabotage that kept the
unit suite green yet still failed the gate. Every claim in this document is checkable by
running `scripts/release_check.sh` and reading the named commits.

## 8. Honest residuals (NOT closed in integration-demo-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known
edge of the deterministic integration demo, not bugs.

1. **The demo is a single fixed scenario.** `CognitiveTrace::demo()` runs one fixed bridge
   scenario end to end. It proves the whole path runs without crossing a boundary; it is
   not a broad corpus of traces. A future sprint may parameterize it, still under the
   re-derive-not-trust discipline.
2. **Re-derivation assumes one binary build.** The operator surfaces verify by re-deriving
   the canonical artifact within the same deterministic build. Cross-version reproduction is
   not claimed; the bundle's `content_hash` is Rust's `DefaultHasher` (named honestly as
   `rust-default-hasher-u64-hex`), a demonstrable digest, not a cryptographic one — the
   load-bearing integrity check is the byte-for-byte re-derivation.
3. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline
   defends against off-wire tampering (untrusted input) and accidental regression, both of
   which the gate provably catches. It does not defend against an insider with commit access
   who authors malicious code AND rewrites the gate in the same change — that is the domain
   of code review and the governance/signing layer.
4. **No model in the loop.** The integration demo is fully deterministic. Any future model
   may only PROPOSE through the frozen hypothesis layer; it can never ground a claim, mutate
   memory, execute a probe, promote evidence, or self-authorize. The P10 adapter stays gated
   shut by P12.
5. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a
   production reasoning system.
6. **Process caveat (verification method).** The read-only adversarial panels have on prior
   tracks left stray debris in the working tree despite their read-only instruction; each
   was untracked, unreferenced, and removed before close. The INT-0..INT-3 panels left no
   debris, but it remains a known operational caveat of the panel method.

## 9. Frozen-status declaration

The INT-0 → INT-3 integration-demo arc is **FROZEN at `integration-demo-v0.1`**. The
output-not-authority boundary is the frozen surface:

```text
The integration demo shows the prototype.
The trace is output, not authority.
The report is output, not authority.
Questions explain the trace.
The bundle demonstrates the prototype.
Nothing executes.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

Any change that lets the trace, report, an answer, or the bundle become authority; that
lets a provided file be trusted instead of re-derived; that executes a probe, promotes an
observation, creates evidence, mutates memory, or reopens training — must pass through the
same machinery: a rubric, a green byte-silent `release_check.sh`, a live sabotage, and an
independent adversarial panel, and must leave `training_justified = false` unless a clean
recurring model failure is proven. Relaxing any criterion requires explicit operator
sign-off; it must not be edited mid-stream to make a failing check pass. P13–P15 do not
start under this freeze.
