# Multi-Trace Validation Milestone: MTRACE-0 → MTRACE-2 (FROZEN for multi-trace-validation-v0.1)

> Status: **FROZEN** as of `multi-trace-validation-v0.1`. This document freezes the MTRACE-0
> through MTRACE-2 multi-trace validation arc as a named, auditable milestone before any further
> behavior is added. It is the single milestone-freeze record for the multi-trace validation track;
> the per-sprint engineering narrative lives in `a.md` (the MTRACE checklist and detail sections) and
> the per-sprint decisions in `docs/PROJECT_CHARTER.md` (`DD-2026-06-19-I`, `DD-2026-06-20-A`,
> `DD-2026-06-20-B`). This file freezes the arc, the commit lineage, the frozen base, the demonstrated
> validation capability, the scenario/coverage/failure boundary, the verification discipline, the
> training-gate verdict, the honest residuals, and the frozen-status declaration. It does not restate
> the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            multi-trace-validation-v0.1
points at      the MTRACE-3 freeze commit (this document + its gate lock)
freezes        the MTRACE-0..MTRACE-2 arc (head be6909f)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr; PATH=/usr/bin)
recover        git checkout multi-trace-validation-v0.1
training gate  training_not_justified (P12 training_justified = false) — weights forbidden
frozen base    integration-demo-v0.1 @ 95b586d (frozen)
deeper base    reading-track-v0.1 @ f6fa55a (frozen) and hypothesis-track-v0.1 @ bb20acf (frozen)
```

The multi-trace validation arc is a fully deterministic surface: no model is in the loop. It extends
the `crates/cognitive-demo` crate — additively, over its own PUBLIC surface and the two deeper frozen
tracks' PUBLIC APIs — with three validation views over the one canonical trace: a scenario pack that
varies the path, a coverage matrix that summarizes it, and a failure-injection pack that proves the
bad paths fail closed. Nothing it adds can execute a probe, mutate reading memory, alter a verifier
receipt, promote an observation, or change the P12 training verdict. It is a VALIDATION layer: it may
build scenarios, summarize coverage, forge-and-reject, serialize, report, and verify, and nothing else.

## 1. What is frozen — the commit lineage

Three feature commits form the arc. Each is additive in the `crates/cognitive-demo` crate (no new
dependency, no `Cargo.toml` change, no new file). The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| MTRACE-0 | Multi-Trace Scenario Pack: one deterministic pipeline run under a finite, enum-backed `Scenario` set producing MULTIPLE `CognitiveTrace` bundles that vary ONLY probe risk/reversibility and the governance decision — each proving the SAME authority boundary under a different review/observation/promotion outcome. The `happy-boundary` scenario IS the frozen canonical `demo()` trace, byte-for-byte. `scenario_bundle`/`scenario_pack_manifest` are purely derived; the verifiers re-derive and byte-compare | `aee733f` |
| MTRACE-1 | Scenario Matrix / Boundary Coverage Report: a deterministic coverage MATRIX (one `MatrixRow` per scenario — path statuses plus the four boundary cells `no_execution`/`no_evidence`/`no_promotion`/`no_training`, all true; `cells_proven=16`, `all_boundaries_hold=true`). Each cell is the trace's REAL verdict (proves, not asserts); `ScenarioMatrix` is `Serialize` but NOT `Deserialize`; the matrix is re-derived from `Scenario::ALL`, never trusted from the pack | `91189f2` |
| MTRACE-2 | Scenario Failure Injection / Boundary Regression Pack: a finite, enum-backed set of seven NEGATIVE cases — each deterministically forges one forbidden authority claim onto a canonical artifact and is REFUSED by the EXISTING re-derive-and-byte-compare verifier (`verify_trace_json`/`verify_scenario_bundle`/`verify_bundle`/`verify_scenario_matrix`). Each case records `injects_forbidden` (its specific forbidden token was injected, so a benign change cannot masquerade) and the observed typed rejection; forged bytes are never persisted | `be6909f` |

## 2. The frozen base

The multi-trace validation arc is built ON TOP OF the integration-demo crate it extends, and it edits
NEITHER of the two deeper frozen tracks it consumes through their public APIs:

```text
integration-demo-v0.1   @ 95b586d   (the cognitive-demo crate: trace / report / question harness / repro bundle)
reading-track-v0.1      @ f6fa55a   (the read0 verifier + the verified reading receipt)
hypothesis-track-v0.1   @ bb20acf   (the propose → probe → review → intent → observation → promotion-refusal chain)
```

The MTRACE arc additively extended the `crates/cognitive-demo` crate (`lib.rs` + `main.rs` only — no
new dependency, no `Cargo.toml` change, no new file). The honest, precise statement of what stayed
frozen: the `integration-demo-v0.1` tag still points at `95b586d` (unmoved), and the FROZEN canonical
`demo()` trace and bundle are byte-for-byte identical after every MTRACE sprint — preserved through the
refactors that parameterized `build()` and shared the bundle/manifest cores, and enforced every sprint
by the `happy_boundary_scenario_equals_canonical_demo` pin and the frozen `hypothesis_id` freeze-pin in
the gate. The two deeper tracks (`reading-track-v0.1`, `hypothesis-track-v0.1`) and the deeper
governance freeze `cognitive-os-governance-v0.1` are touched by nothing in this arc; every MTRACE sprint
proves it changes no frozen crate source outside `crates/cognitive-demo` and leaves all three tags
unmoved.

## 3. What the prototype can now validate (the demonstrated capability)

The prototype could already produce, report, interrogate, and package one canonical trace
(integration-demo-v0.1). The multi-trace arc adds the ability to VALIDATE that trace's boundary across
many paths, deterministically and reproducibly:

1. **Vary the path without varying the authority (MTRACE-0).** Run the same deterministic pipeline
   under several finite, enum-backed scenarios — a low-risk approved probe, a rejected review, a
   deferred review, a high-risk blocked probe — and produce one `CognitiveTrace` bundle per scenario.
   Every scenario preserves the full boundary (no execution, no evidence, no promotion, no training); a
   rejected/deferred review yields a `blocked` intent and a blocked probe has no approval path. The
   `happy-boundary` scenario is the frozen canonical `demo()` trace, byte-for-byte.
2. **Summarize the coverage, re-derived not trusted (MTRACE-1).** Derive a coverage matrix that records,
   per scenario, the path statuses and the four boundary cells — each cell the trace's REAL verdict —
   and a summary proving all sixteen cells hold and the paths are genuinely distinct. The matrix is
   re-derived from `Scenario::ALL`; emitting it first verifies the pack; verifying it re-derives and
   byte-compares.
3. **Prove the bad paths fail closed (MTRACE-2).** Deterministically forge each of seven forbidden
   authority claims onto a canonical artifact and show the EXISTING re-derive-and-byte-compare verifier
   refuses every one, recording — per case — that the forgery genuinely injected its specific forbidden
   token and the exact typed rejection reason. No new verification logic; a curated regression suite of
   attacks against the verifiers MTRACE-0/1 rely on.

Every one of these is a VALIDATION VIEW over the canonical trace. None of them is authority.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint;
each later sprint is additive over them, and each is enforced by the release gate from the artifacts'
own bytes.

1. **Scenarios vary the path, not the authority (MTRACE-0).** The `Scenario` enum varies ONLY the probe
   risk/reversibility and the governance decision; the reading verification, the receipt citation, the
   chain linkage, and the verdict computation are identical and read from the frozen crates. Every
   scenario keeps execution never `executed`, observation never `recorded` (`observation_only`),
   promotion `rejected` (`grants_promotion=false`), and `training_justified=false`. A rejected/deferred
   review yields a `blocked` intent; a blocked probe has no approval path.
2. **The matrix summarizes coverage, it does not create authority (MTRACE-1).** Each boundary cell is
   the trace's REAL verdict (`no_execution=trace.nothing_executed()`, and so on); the report is pure
   formatting of the re-derived matrix and computes no new verdict. No matrix or report field shows an
   affirmative `executed`/`promoted`/`granted`/`recorded` status, a true grant, or a `training_justified`
   verdict.
3. **Failure cases attack the boundary, they do not weaken it (MTRACE-2).** Each forged case runs the
   real verifier on a forged COPY and is refused; the pack records `injects_forbidden` so a benign
   byte-change cannot masquerade as a forbidden-authority forgery, and `rejected` is observed from the
   verifier's `Result`, never hardcoded. The forged bytes are never persisted — only the prose rejection
   record is — so neither emitted file carries affirmative authority.
4. **Re-derive, never trust (the arc-wide discipline).** Every operator surface that accepts a file
   (`scenario-verify`, `scenario-matrix`/`-report`/`-verify`, `failure-verify`) verifies by re-deriving
   the canonical artifact and byte-comparing — never by parsing the provided bytes into trusted state.
   No record added in this arc derives `Deserialize` (`CognitiveTrace`, `BundleManifest`,
   `ScenarioPackManifest`, `ScenarioMatrix`, `MatrixRow`, `MatrixCoverage`, `FailurePack`,
   `FailureRejection`, `FailureSummary` are all `Serialize`-only). So off-the-wire tampering — of a
   scenario bundle, a pack manifest, a matrix, or a failure pack — can never be laundered into a clean
   verification.
5. **No new authority across the arc.** The arc executes no probe (no process/network anywhere in `src`
   or the examples), promotes nothing, mutates no memory, and moves no training verdict. The strongest
   honest case is preserved: governance APPROVES the happy-boundary probe, yet execution stays
   `requires_operator` (never `executed`), the observation stays `requires_review`/`observation_only`
   (never `recorded`), and the promotion-to-evidence request is `rejected`. Approval is not execution; an
   observation is not evidence.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is
`training_justified=false`). The multi-trace validation arc is orthogonal to P12 and does not move it:
every MTRACE sprint reads the training decision before and after building its artifacts and proves it
identical, and leaves the reading verifier receipt byte-identical. Weight training stays forbidden until
the P11 eval proves a stable, recurring model failure that survives fixes to task spec, schema, prompt,
examples, tooling, context, and verifier design. P13–P15 (LoRA candidate, shadow mode, promotion gate)
stay closed under this freeze.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0
bytes stderr). The MTRACE blocks gate, for the whole `cognitive-demo` crate, `cargo test` +
`cargo fmt --check` + `cargo clippy -D warnings`, plus, per sprint: surface-signal greps proving the
scenario/matrix/failure surfaces exist and call the frozen crates; the re-derive pins
(`compare_bundle(&scenario_bundle(scenario)?`, `provided == scenario_pack_manifest()?`,
`provided == scenario_matrix()?`, `compare_bundle(&failure_pack_files()?`) proving each verifier
byte-compares a re-derived canonical rather than trusting a file; the anti-vacuity pins (the frozen
`hypothesis_id` freeze-pin `16880898425785712701`, and `injects_forbidden: forged.contains(token)`); a
unit-test REALITY pin (exactly 80 passed, zero ignored) and per-test name pins; crate-wide purity and
no-probe-execution scans; an fs-confined scan keeping `std::fs` out of the library and examples; and
end-to-end binary smokes that drive every command against real temp files and grep the real serialized
output — the no-authority guards, the tamper/missing/foreign refusals, the verbatim boundary loops, and
the byte-identity of the frozen canonical trace. This milestone block additionally pins the freeze record
itself (this document's commit lineage, frozen-base references, the nine boundary lines, and the
`training_not_justified` verdict), and guards against any milestone that falsely claims training has
opened. The acceptance discipline for every sprint in this arc was: rubric → green byte-silent `release_check` → live
sabotage probes proving the gate catches a regression (restored byte-identical by md5, never
`git checkout`) → an independent read-only adversarial verifier panel with a fresh context → any residual
folded before close.

## 7. Independent verification

Every sprint MTRACE-0 through MTRACE-2 was closed against read-only adversarial panels (Explore agents,
refute-by-default, no-compile-to-disk, each driving the compiled binary under a temp dir), run until a
fully-dry round with zero real findings and no debris. Several sprints drove gate strengthenings that were
reproduced first-hand before folding: MTRACE-0 added the frozen `hypothesis_id` freeze-pin after noticing
a silent happy-boundary drift would slip both the equality test and the status greps; MTRACE-2 added the
`injects_forbidden` anti-vacuity check after noticing a benign byte-change would also be byte-rejected and
so the applied+rejected asserts were vacuous. Each strengthening was proven load-bearing by a sabotage
that kept the unit suite green yet still failed the gate. (One MTRACE-1 panel attempt was abandoned
mid-run by a session-usage limit — absence of verification, not a pass — and was re-run to a genuine dry
round before close.) Every claim in this document is checkable by running `scripts/release_check.sh` and
reading the named commits.

## 8. Honest residuals (NOT closed in multi-trace-validation-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the
deterministic multi-trace validation arc, not bugs.

1. **The scenario set is small and fixed.** The `Scenario` enum enumerates four deterministic paths and
   the failure set seven forged cases. They prove the boundary holds across the path variations that
   matter and fails closed under the forbidden-authority forgeries that matter; they are not an
   exhaustive corpus. A future sprint may enumerate more paths, still under the re-derive-not-trust
   discipline.
2. **Re-derivation assumes one binary build.** The operator surfaces verify by re-deriving the canonical
   artifact within the same deterministic build. Cross-version reproduction is not claimed; the bundle's
   `content_hash` is Rust's `DefaultHasher` (named honestly as `rust-default-hasher-u64-hex`), a
   demonstrable digest, not a cryptographic one — the load-bearing integrity check is the byte-for-byte
   re-derivation.
3. **Forgery coverage is the curated forbidden-authority set.** MTRACE-2 forges the specific forbidden
   claims (execution, evidence, promotion, training, approved review, narrated report, hidden matrix
   cell) and proves each is refused. It is a regression suite against the known authority boundaries, not
   a proof that no forgery of any kind exists; a forgery outside the enumerated set is out of its scope by
   construction.
4. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline defends against
   off-wire tampering (untrusted input) and accidental regression, both of which the gate provably
   catches. It does not defend against an insider with commit access who authors malicious code AND
   rewrites the gate in the same change — that is the domain of code review and the governance/signing
   layer.
5. **No model in the loop.** The validation arc is fully deterministic. Any future model may only PROPOSE
   through the frozen hypothesis layer; it can never ground a claim, mutate memory, execute a probe,
   promote evidence, or self-authorize. The P10 adapter stays gated shut by P12.
6. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production
   reasoning system.
7. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left
   stray debris in the working tree despite their read-only instruction; each was untracked, unreferenced,
   and removed before close. The MTRACE-0..MTRACE-2 panels left no debris, but it remains a known
   operational caveat of the panel method.

## 9. Frozen-status declaration

The MTRACE-0 → MTRACE-2 multi-trace validation arc is **FROZEN at `multi-trace-validation-v0.1`**. The
output-not-authority boundary is the frozen surface:

```text
Scenarios vary the path.
They do not vary the authority.
The matrix summarizes coverage.
Failure cases attack the boundary.
Forged authority is rejected.
Nothing executes.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

Any change that lets a scenario, the matrix, a report, or the failure pack become authority; that lets a
provided file be trusted instead of re-derived; that executes a probe, promotes an observation, creates
evidence, mutates memory, or reopens training — must pass through the same machinery: a rubric, a green
byte-silent `release_check.sh`, a live sabotage, and an independent adversarial panel, and must leave
`training_justified = false` unless a clean recurring model failure is proven. Relaxing any criterion
requires explicit operator sign-off; it must not be edited mid-stream to make a failing check pass.
P13–P15 do not start under this freeze.
