# Project Charter — Cognitive OS

Significant architectural decisions for the Cognitive OS prototype. Newest first. Each entry
links to the canonical artifact that records the decision in full.

## DD-2026-06-19-I — Add the multi-trace scenario pack (MTRACE-0), variation without authority expansion

**Decision.** Extend `crates/cognitive-demo` (the `cognitive-demo` binary) with a small deterministic scenario
pack: `scenarios` lists a finite scenario set, `scenario-pack --out DIR` writes one bundle subdirectory per
scenario plus a `pack-manifest.json`, and `scenario-verify --path DIR` re-derives the whole pack and refuses any
tamper. The four scenarios (`happy-boundary`, `review-rejected`, `review-deferred`, `high-risk-blocked`) run the
SAME frozen hypothesis chain under different review/observation/promotion outcomes, each proving the SAME
authority boundary. It adds NO capability and NO model behavior, no new dependency, no new file, no Cargo.toml
change, and it edits no frozen crate.

**Why.** The integration-demo-v0.1 freeze proves ONE canonical path. The next useful step was to prove the same
boundaries hold across several deterministic paths — variation WITHOUT authority expansion — before adding any
new behavior. The doctrine is sharpened for this surface: *Scenarios vary the path. They do not vary the
authority. Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.*

**Boundary recorded.** A `Scenario` enum varies ONLY the probe's risk/reversibility and the governance decision
(`Scenario::risk`/`reversibility`/`review_decision` passed into the new `CognitiveTrace::build_scenario`, which
`build()` now delegates to with `HappyBoundary`); everything else — reading verification, receipt citation, chain
linkage, verdict computation — is identical and read from the frozen crates. Every scenario preserves the full
boundary: execution never `executed` (`nothing_executed`), observation never `recorded` and `observation_only`
(`observation_quarantined`), promotion `rejected` with `grants_promotion=false`
(`promotion_refused`/`nothing_becomes_evidence`), and `training_justified=false` with the verifier receipt
unmoved. A rejected/deferred review yields a `blocked` (never executable) intent (the frozen `from_review` maps
Rejected/Deferred → Blocked); a blocked probe has no approval path (the frozen layer refuses to approve it).
Verification is by RE-DERIVATION: `scenario_bundle`/`scenario_pack_manifest` are pure, and
`verify_scenario_bundle`/`verify_scenario_pack_manifest` re-derive and byte-compare via the shared `compare_bundle`
core — `CognitiveTrace`, `BundleManifest`, and `ScenarioPackManifest` all derive `Serialize` but NOT
`Deserialize`, so no file is parsed back into authority and a tampered/missing/foreign scenario is refused. The
load-bearing risk — that parameterizing `build()` (and making `canonical_bundle`/`run_questions_doc`/`verify_bundle`
delegate to shared cores) could drift the frozen canonical trace — did NOT occur: all 44 frozen tests pass and
the happy-boundary scenario is byte-identical to `CognitiveTrace::demo()`. One self-found gap (the happy==demo
test is self-referential, so a silent happy-boundary risk/reversibility drift with an unchanged path would slip
it and the status greps) was folded before sabotage by pinning the frozen canonical `hypothesis_id`
(`16880898425785712701`, a stable FNV id) literally in the gate. `release_check.sh` gates it (surface signals,
the re-derive pins, twelve MTRACE-0 test-name pins, the unit-count pin raised 44→56, the `hypothesis_id`
freeze-pin, and a binary smoke proving the four-subdir pack, determinism, distinguishable statuses, the
no-authority guard, and refusal of a tampered scenario trace/manifest/pack-manifest + a missing file + a foreign
scenario) and stays green + byte-silent. Verified by three live sabotage probes (a rejected review approving; a
verify that trusts files; a silent happy-boundary canonical drift that kept the suite green but failed the gate
via the freeze-pin — each restored byte-identical) and a read-only adversarial panel (four Explore lenses, 0 real
findings, fully dry, no debris, each driving the compiled binary). Purely additive: only
`crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate block; no frozen crate source touched, the
`reading-track-v0.1` (`f6fa55a`), `hypothesis-track-v0.1` (`bb20acf`), and `integration-demo-v0.1` (`95b586d`)
tags unmoved, P12 `training_justified=false`, and P13–P15 closed. Recorded in full in [a.md](../a.md) (the
MTRACE-0 checklist entry and the "Multi-Trace Scenario Pack / Variation Without Authority Expansion (MTRACE-0)"
detail section). Local only — no remote push.

## DD-2026-06-19-H — Freeze the integration-demo track (INT-0 → INT-3) as integration-demo-v0.1

**Decision.** Freeze the INT-0 → INT-3 integration-demo arc as a named, auditable milestone
`integration-demo-v0.1`, recorded in a new freeze doc `INTEGRATION_DEMO_MILESTONE.md` and locked by a milestone
block in `scripts/release_check.sh`. Documentation freeze only — it adds no behavior and edits no code crate.

**Why.** INT-0 (trace), INT-1 (report CLI), INT-2 (question harness), and INT-3 (repro bundle) now form a
complete, demonstrable integration arc over the two frozen tracks: the prototype can produce a verified
reading-derived trace, show the operator what happened, answer fixed audit questions, and package the whole
thing into a reproducible, re-derivable bundle. Per the build→prove cadence, the arc is frozen before more
behavior is added — the same discipline that produced `reading-track-v0.1` and `hypothesis-track-v0.1`.

**Boundary recorded.** The milestone doc pins the commit lineage (INT-0 `2330f7c`, INT-1 `92c0692`, INT-2
`b5bcf66`, INT-3 `f451c39`), references the frozen dependencies (`reading-track-v0.1` @ `f6fa55a`,
`hypothesis-track-v0.1` @ `bb20acf`), states the demonstrable capability, and records the output-not-authority
boundary verbatim: *The integration demo shows the prototype. The trace is output, not authority. The report is
output, not authority. Questions explain the trace. The bundle demonstrates the prototype. Nothing executes.
Nothing becomes evidence. Nothing promotes. Nothing trains.* The arc-wide discipline is RE-DERIVE, NEVER TRUST:
every operator surface that accepts a file (report, replay, ask, bundle-verify) verifies by re-deriving the
canonical artifact and byte-comparing — no record in the crate derives `Deserialize` — so off-wire tampering can
never be laundered into authority. The milestone makes no false claim: it records P12 `training_justified=false`
(`training_not_justified`), and the integration crate executes no probe, promotes nothing, mutates no memory, and
moves no training verdict; P13–P15 stay closed. The `release_check.sh` milestone lock pins the freeze doc's
existence, the four INT commit hashes (auditable against `git log`), the frozen-dependency references, the nine
boundary lines verbatim, and the `training_not_justified` verdict, and guards against a false `training_justified
= true` claim, so the freeze cannot silently drift. Verified by a green byte-silent `release_check.sh`, live
sabotage probes of the milestone lock (each restored byte-identical), and a read-only adversarial panel.
The tag `integration-demo-v0.1` is created only after a clean tree and a green gate, on the freeze commit. No
frozen crate source is touched, the `reading-track-v0.1` (`f6fa55a`) and `hypothesis-track-v0.1` (`bb20acf`) tags
are unmoved, P12 `training_justified=false`, and P13–P15 closed. Recorded in full in
[INTEGRATION_DEMO_MILESTONE.md](../INTEGRATION_DEMO_MILESTONE.md). Local only — no remote push.

## DD-2026-06-19-G — Add the prototype demo bundle / operator repro pack (INT-3)

**Decision.** Extend `crates/cognitive-demo` (the `cognitive-demo` binary) with one reproducible operator pack
over the canonical trace and a re-deriving verifier: `bundle --out DIR` writes four files (`trace.json`,
`report.txt`, `questions.txt`, `manifest.json`) PURELY derived from the canonical trace; `bundle-verify --path
DIR` re-derives the pack and refuses any tampered/missing/foreign file. It is a thin demonstration surface over
the EXISTING canonical trace — NO new authority and NO new cognition, no new dependency, no new file, no
Cargo.toml change, and it edits no frozen crate.

**Why.** INT-0/1/2 built the trace, made it inspectable (a report), and made it interrogable (the question
harness); the next useful step was to make it PORTABLE — one command that produces a reproducible pack showing
what the prototype can do, and a second that verifies the pack — without the files becoming evidence or
authority. The doctrine is sharpened for this surface: *The bundle demonstrates the prototype. It does not create
evidence. It does not create authority. It does not execute. It does not promote. It does not train.*

**Boundary recorded.** The load-bearing design is the re-derivation trust boundary, now applied to a multi-file
pack. `verify_bundle` does NOT trust the files: it re-derives the canonical bundle via `canonical_bundle()`
(which builds from `run_trace` / `CognitiveTrace::demo()`) and byte-compares each provided file; a missing file
is `TraceError::BundleMissingFile` and any tampered/stale/foreign file (INCLUDING the manifest) is
`TraceError::BundleMismatch`. It never parses/deserializes a provided file into trusted state and never checks
the manifest's own recorded hash against the file. `CognitiveTrace` and the new `BundleManifest`/`BundleFileEntry`/
`BundleReplayProof` derive `Serialize` but NOT `Deserialize`, so no bundle file is read back into authority — a
tampered bundle can never pass, and the manifest (itself re-derived and byte-compared) can never vouch for a
forged pack. The manifest is honest: `bundle_content_hash` is Rust's `DefaultHasher` (deterministic,
dependency-free), named `rust-default-hasher-u64-hex` (NOT a crypto digest); it hashes the three content files
with distinct content-dependent hashes and does not hash itself (no fixpoint); the load-bearing integrity check
is the full byte-for-byte re-derivation, of which the hash is a demonstrable part. Purity is structural: the
filesystem I/O (`write_bundle`/`read_bundle`) lives only in `src/main.rs`; the library that derives/verifies the
bundle is filesystem-free, so the bundle content can never depend on disk, and the pack is a pure function of
fixed inputs (two bundles are byte-identical). The bundle creates no authority and no evidence — no file shows an
affirmative `executed`/`promoted`/`granted`/`recorded` status or a true grant, the trace records
`training_justified=false`, and the verifier receipt is unmoved. `release_check.sh` gates it (surface signals,
the re-derive pin, twelve INT-3 test-name pins, the unit-count pin raised 32→44, and a binary smoke that proves
the four files, the manifest hashing + distinct hashes + six verbatim boundary lines, determinism, the
no-authority guard, and refusal of a tamper of EACH file + a missing file + a foreign bundle) and stays green +
byte-silent. One self-found gap (the hash test is self-referential, so a constant/fake hash would slip it and the
count check) was folded before sabotage by adding a distinct-hash gate check. Verified by three live sabotage
probes (verify trusts the files; a constant fake hash that kept the suite green but failed the gate via the
distinct-hash check; a coordinated boundary drift that kept the suite green but failed the gate via the verbatim
six-line manifest loop — each restored byte-identical) and a read-only adversarial panel (four Explore lenses, 0
real findings, fully dry, no debris, each driving the compiled binary). Purely additive: only
`crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate block; no frozen crate source touched, the
`reading-track-v0.1` (`f6fa55a`) and `hypothesis-track-v0.1` (`bb20acf`) tags unmoved, P12 `training_justified=false`,
and P13–P15 closed. Recorded in full in [a.md](../a.md) (the INT-3 checklist entry and the "Prototype Demo Bundle
/ Operator Repro Pack (INT-3)" detail section). Local only — no remote push.

## DD-2026-06-19-F — Add the trace question harness / operator interrogation surface (INT-2)

**Decision.** Extend `crates/cognitive-demo` (the `cognitive-demo` binary) with a deterministic, FINITE,
enum-backed audit-question surface over the INT-0/INT-1 canonical trace: `questions` lists the closed set and
`ask --trace PATH --question SLUG [--out PATH]` answers exactly one of eight enumerated questions (what-read,
what-was-proven, what-was-hypothesized, what-probe-was-requested, was-anything-executed,
did-anything-become-evidence, why-was-promotion-refused, did-training-open). It is a thin interrogation surface
over the EXISTING canonical trace — NO LLM, NO natural-language parser, NO new authority and NO new cognition,
no new dependency, no new file, no Cargo.toml change, and it edits no frozen crate.

**Why.** INT-1 made the trace inspectable as a report; the next useful step was to let an operator ask fixed,
machine-checkable questions about what happened, what did not, and why authority was refused — without reading
Rust structs and without a chatbot. The doctrine is unchanged and sharpened for this surface: *Trace questions
explain the trace. They do not create authority. They do not execute. They do not promote. They do not train.*

**Boundary recorded.** The surface is CLOSED by construction: a question is a `TraceQuestion` enum variant
(`ALL: [TraceQuestion; 8]`); `TraceQuestion::from_slug` does EXACT-match only (no fuzzy/prefix/case/trim),
returning `None` on any miss; `run_ask` fails closed TWICE and in order — an unknown slug is
`TraceError::UnknownQuestion`, refused WITHOUT consulting any trace (prose can never become a question), and only
then is the trace re-derived and verified before any answer. The trust boundary is INT-1's, applied to `ask`:
because `CognitiveTrace` is `Serialize` but NOT `Deserialize`, `run_ask` answers ONLY the trace returned by
`verify_trace_json`, which RE-DERIVES the canonical trace via the pure `CognitiveTrace::demo()` and byte-compares
the provided file, refusing any tampered/stale/foreign input (`TraceError::TraceMismatch`) BEFORE answering — so
a forged trace can never be laundered into an answer (a tampered trace is refused for every question). Answers
are not authority: the private `CognitiveTrace::answer` + eight `answer_*` renderers FORMAT only the trace's
already-recorded fields (no new verdict, no frozen API, no authority object), distinguish the stages (proof vs
hypothesis vs review vs intent vs observation vs promotion), include the relevant ids/hashes, never show an
affirmative `executed`/`promoted`/`granted`/`recorded` status, and end with the five-line INT-2 boundary; the
only filesystem access is the pre-existing `main.rs` I/O shell, so the surface stays pure. `release_check.sh`
gates it (surface signals, the fail-closed/re-derive pins, twelve INT-2 test-name pins, the unit-count pin raised
20→32, and an end-to-end binary smoke that proves the questions listing, the real receipt hash in `what-read`,
the verbatim five-line boundary, the no-authority guard, and refusal of BOTH an unknown question and a tampered
trace) and stays green + byte-silent. One self-found gap (the boundary smoke pinned only two of five lines, and
the test only lines [0]/[4]) was folded before sabotage by adding a verbatim five-line loop to the gate and
pinning all five literals. Verified by three live sabotage probes (fail-open tamper-refusal, fail-open unknown
question, and a coordinated boundary drift that kept the unit suite green but still failed the gate via the
five-line loop — each restored byte-identical) and a read-only adversarial panel (four Explore lenses, 0 real
findings, fully dry, no debris, each driving the compiled binary). Purely additive: only
`crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate block; no frozen crate source touched, the
`reading-track-v0.1` (`f6fa55a`) and `hypothesis-track-v0.1` (`bb20acf`) tags unmoved, P12 `training_justified=false`,
and P13–P15 closed. Recorded in full in [a.md](../a.md) (the INT-2 checklist entry and the "End-to-End Trace
Question Harness / Operator Interrogation Surface (INT-2)" detail section). Local only — no remote push.

## DD-2026-06-19-E — Add the end-to-end trace CLI / operator report (INT-1)

**Decision.** Extend `crates/cognitive-demo` (INT-0) with the `cognitive-demo` binary: `trace` writes the
canonical `CognitiveTrace` JSON, `report` renders a plain operator report, `replay` confirms a byte-identical
reproduction. It is a thin operator surface over the EXISTING canonical trace — it adds NO new authority and NO
new cognition, consumes no new dependency, and edits no frozen crate.

**Why.** INT-0 proved the chain internally; the next useful step was to make it usable and inspectable by a human
operator (one command → a readable report plus the machine JSON) without reading Rust structs or test output —
not more capability. The doctrine is unchanged: *Reading verifies. Hypothesis proposes. Probe queue classifies.
Governance reviews. Execution intent records. Observation quarantines. Promotion refuses. Nothing becomes
evidence. Nothing trains.*

**Boundary recorded.** The load-bearing design is the trust boundary: because `CognitiveTrace` is `Serialize`
but NOT `Deserialize`, `report`/`replay` never parse a provided file back into authority — the pure
`verify_trace_json` RE-DERIVES the canonical trace via `CognitiveTrace::demo()` and compares the provided file
BYTE-FOR-BYTE, refusing any difference (`TraceError::TraceMismatch`); the report is rendered from the re-derived
canonical trace. So a tampered/stale/foreign `trace.json` can never be laundered into a clean report or a passing
replay — both refuse it (verified live by the panel and the gate). `to_report()` is pure formatting (no new
verdict, no frozen API, no authority object), so report prose cannot become authority. `std::fs` is confined to
the new `src/main.rs` (a thin I/O shell); the trace core and the example stay filesystem-free, so the trace
result can never depend on disk, and the CLI spawns no process and opens no socket. The report shows all seven
stages with the ids/hashes needed to audit/replay, prints all nine boundary lines verbatim, and states
explicitly that nothing executed, nothing became evidence, and training stayed false. `release_check.sh` gates it
(CLI-core + report signals, the trust-boundary greps, eight INT-1 test-name pins, the unit-count pin raised
12→20, the fs-confined scan, and an end-to-end binary smoke that proves trace determinism, full report coverage,
replay acceptance, and tamper rejection by both replay and report) and stays green + byte-silent. Verified by
three live sabotage probes (each restored byte-identical) and a read-only adversarial panel (four Explore lenses,
0 real findings, fully dry, no debris). Purely additive: only `crates/cognitive-demo/{Cargo.toml,src/lib.rs}`,
the new `src/main.rs`, and the gate block; no frozen crate source touched, the `reading-track-v0.1` (`f6fa55a`)
and `hypothesis-track-v0.1` (`bb20acf`) tags unmoved, P12 `training_justified=false`, and P13–P15 closed.
Recorded in full in [a.md](../a.md) (the INT-1 checklist entry and the "End-to-End Trace CLI / Operator Report
(INT-1)" detail section). Local only — no remote push.

## DD-2026-06-19-D — Add the end-to-end prototype trace demo (INT-0) as the first integration layer

**Decision.** Add a NEW crate `crates/cognitive-demo` (INT-0) that produces ONE deterministic, replayable
`CognitiveTrace` connecting a VERIFIED reading receipt to the full frozen hypothesis chain (hypothesis → probe →
review → execution intent → observation → promotion-refusal), and records every component id/hash plus
machine-checkable verdicts in a single auditable artifact. It is the FIRST integration sprint: additive above
the two frozen tracks, consuming their PUBLIC APIs only — it edits NEITHER frozen crate.

**Why.** The frozen pieces each held a boundary in isolation; the next useful step was not more capability inside
one layer but a thin demo proving the whole prototype can run one bounded cognitive path end to end WITHOUT
crossing any authority boundary. This is the project's typed answer to the frontier reasoning-trace idea: the
trace is a PUBLIC execution record of typed objects (each with its own authority limits, content id, and
integrity hash), not a private chain-of-thought to be trusted as truth. Custody, replay, and refusal are made
machine-checkable. Doctrine: *Reading verifies. Hypothesis proposes. Probe queue classifies. Governance reviews.
Execution intent records. Observation quarantines. Promotion refuses. Nothing becomes evidence. Nothing trains.*

**Boundary recorded.** The canonical flow is the strongest honest case: governance APPROVES the probe, yet the
execution intent is `requires_operator` (no `executed` state), the observation is `requires_review` /
`observation_only` (never `recorded`), and the promotion-to-`evidence` REQUEST is `rejected` with
`grants_promotion=false` — approval is not execution, an observation is not evidence. The trace is inert
(`Serialize` but NOT `Deserialize`, private fields, minted only by `demo`/`build`, no accessor returning
claim/evidence authority), so it cannot be forged or mutated into a later claim. The P12 verdict is read before
and after the flow and proven unmoved (`training_justified=false`). INT-0 grants no new authority, executes no
probe, promotes nothing, mutates no memory, and leaves the verifier receipt byte-identical. `release_check.sh`
gates it (encapsulation pin + API-exercise greps + 12 name-pinned tests + a 12-passed/0-ignored reality pin +
purity + no-probe-execution scan + separation + a determinism double-run + a precise no-grant guard that catches
a real grant but never false-positives on the legitimate `promotion_target: evidence` REQUEST) and stays green +
byte-silent. Verified by three live sabotage probes (each restored byte-identical) and a read-only adversarial
panel (four Explore lenses, 0 real findings, fully dry, no debris). Purely additive: only `crates/cognitive-demo/`,
the workspace member add, and the gate block change; no frozen crate source is touched, the `reading-track-v0.1`
(`f6fa55a`) and `hypothesis-track-v0.1` (`bb20acf`) tags are unmoved, and P13–P15 stay closed. Recorded in full
in [a.md](../a.md) (the INT-0 checklist entry and the "End-to-End Prototype Trace Demo (INT-0)" detail section).
Local only — no remote push.

## DD-2026-06-19-C — Freeze the hypothesis track (HYP-0 → HYP-5) as hypothesis-track-v0.1

**Decision.** Freeze the post-reading hypothesis-track arc HYP-0 → HYP-5 as a named, auditable milestone,
recorded in `HYPOTHESIS_TRACK_MILESTONE.md` and tagged `hypothesis-track-v0.1`. Documentation freeze only — no
code crate changes, no runtime behavior change, no Cargo/lock change; the only gate edit is the milestone lock
that pins the freeze. P13–P15 stay closed; training stays blocked at P12.

**Why.** HYP-0 through HYP-5 now form a complete post-freeze arc — hypothesis → probe queue → review →
execution intent → observation quarantine → promotion refusal — sitting above the frozen reading track
(`reading-track-v0.1` @ `f6fa55a`) and governance (`cognitive-os-governance-v0.1`). Before adding more
capability, the arc is frozen the same way the reading track was at READ-16.

**What is frozen.** The commit lineage (HYP-0 `f19a998`, HYP-1 `4b47736`, HYP-2 `cb68a73`, HYP-3 `6cbb3a8`,
HYP-4 `7703e2e`, HYP-5 `cef91db`, plus the post-HYP-5 charter snapshot `d899a61` `DD-2026-06-19-B`); the
authority boundary (*Hypothesis proposes. Probe queue classifies. Governance reviews. Execution intent does not
execute. Observation is quarantined. Promotion request does not promote. Nothing becomes evidence.*); the
structural quarantine; the P12 verdict `training_not_justified`; the verification discipline; and the honest
residuals. The milestone makes no new capability claim: no probe execution exists, no observation is evidence,
no promotion exists, and training stays closed. `release_check.sh` locks the milestone doc (file presence +
FROZEN + tag + HYP-0/HYP-5 endpoints + `training_not_justified` + all seven pinned commit hashes) and stays
green + silent; the tag is created only after a clean tree + green gate. Recorded in full in
[HYPOTHESIS_TRACK_MILESTONE.md](../HYPOTHESIS_TRACK_MILESTONE.md). Local only — no remote push.

## DD-2026-06-19-B — Cognitive OS prototype status snapshot after HYP-5

**Decision.** Record the cumulative status of the Cognitive OS prototype after HYP-5 commits. Documentation
only — no runtime behavior changes, no Cargo/lock change, no training path opened, no probe execution, no
evidence promotion, no memory mutation, no verifier change. P13–P15 stay closed.

**Frozen anchors.** The governance milestone `cognitive-os-governance-v0.1` remains frozen. The reading
milestone `reading-track-v0.1` points at `f6fa55a`. P12 remains the controlling training verdict:
`training_justified=false`. No LLM training, no probe execution, no observation promotion, and no evidence
authority expansion have occurred.

**Post-freeze hypothesis chain (all in `crates/hypothesis-layer`, untagged, local only):**

- HYP-0 `f19a998` — hypothesis-only abductive layer.
- HYP-1 `4b47736` — probe queue / human-review boundary.
- HYP-2 `cb68a73` — governance review receipt boundary.
- HYP-3 `6cbb3a8` — approved-probe execution stub / non-execution boundary.
- HYP-4 `7703e2e` — observation receipt quarantine.
- HYP-5 `cef91db` — observation promotion gate / still-no-evidence boundary.

**Status table.**

| Track | Status |
| ----- | ------ |
| Governance v0.1 | complete / frozen / tagged (`cognitive-os-governance-v0.1`) |
| Deterministic engine P1–P8 | complete / tested |
| Reading substrate READ-0–READ-15 | complete / tested / frozen / tagged (`reading-track-v0.1` @ `f6fa55a`) |
| Codec / model / eval / train gate P9–P12 | complete / tested; training blocked (`training_justified=false`) |
| Hypothesis track HYP-0–HYP-5 | complete / tested through promotion refusal |
| Training track P13–P15 | closed until P12 flips |

**Active doctrine.** *Probability proposes. Replay tests. Governance authorizes. Memory records.*

**Authority boundary (current).**

- Hypotheses are not claims.
- Probe requests are not evidence.
- Review receipts are not execution.
- Execution intents do not execute.
- Observations are quarantined.
- Promotion requests do not promote.
- Nothing becomes evidence without a future verifier-backed promotion path.

**Status.** Plain assessment: this is a strong prototype concept — a deterministic cognition substrate with
reading, verification, replay, bounded autonomy, hypothesis generation, review, execution-intent stubs,
observation quarantine, and promotion refusal. It is NOT an AI model yet; the model/training track is still
correctly blocked at P12. `release_check` remains green and silent; no code crates changed for this snapshot.

## DD-2026-06-19-A — Add the observation promotion gate / still-no-evidence boundary (P17 / HYP-5) in-crate

**Decision.** Add `crates/hypothesis-layer/src/promotion.rs` — a `PromotionRequest` derived from a HYP-4
`ProbeObservationReceipt` that records a REQUEST to promote a quarantined observation toward a
claim/evidence/memory-note, while refusing to promote anything to evidence until a future verifier-backed path
exists. Doctrine: *Hypothesis proposes. Probe queue classifies. Governance reviews. HYP-3 records intent. HYP-4
quarantines observations. HYP-5 records promotion requests. Nothing becomes evidence.* Kept INSIDE the existing
crate (a new module, no new dependency), so the serde-only quarantine is unchanged.

**Why.** HYP-4 quarantines an observation but cannot record anything (`recorded` is future-reserved); the next
authority leak is "the observation exists, therefore it is evidence." HYP-5 defines what a future promotion
REQUEST looks like while still refusing to promote: a request is minted only by `from_observation`, which
DERIVES the outcome from the observation's disposition and the requested target — a `rejected`/`requires_review`
observation yields `rejected` (for any target), and the future-reserved `recorded` observation yields
`requires_verifier` (claim/evidence) or `unsupported` (memory-note). Because HYP-4 makes `recorded`
unreachable, every real request is `rejected`: at HYP-5 nothing can be promoted. No status grants a promotion.

**Boundary (enforced by the compiler, types, the gate, and a behavioral surface).** A `PromotionRequest` is
minted only by `from_observation`, has private fields, and derives `Serialize` but not `Deserialize`
(`PromotionStatus`/`PromotionReason` are output-only, so the request is structurally non-deserializable — a
`compile_fail` proof, pinned live by cargo's doctest report; `PromotionTarget` is the deserializable input).
The reason/status derivation is exhaustive with no wildcard (E0004 on a new `ObservationStatus` or reason), and
`grants_promotion` matches every status with no wildcard returning `false` (E0004 on a future promoting
variant), so "still no evidence" cannot silently regress. The gate also pins the SOLE minting path with a
construction-literal count (`PromotionRequest {` appears exactly 5 times): since the crate is
`#![forbid(unsafe_code)]`, the type has no `Deserialize`, and its fields are private, a struct literal is the
only way to construct one, so a backdoor minting path of any return-type shape raises the count and fails. The
request binds its fields with an `integrity_hash`, cites its provenance, and reuses the forbidden-uses
quarantine so it can never become evidence. Verified by three read-only adversarial panel rounds: round one's
five substantive lenses clean, the still-no-evidence lens raising a backdoor-constructor finding (reproduced
first-hand, judged insider-forgery-scope, but the previously-ungated correct-if 1 was folded into a
sole-minting-path pin); round two's five lenses clean, the gate-vacuity lens showing the first pin was evadable
by a composite return type (reproduced first-hand and replaced with the robust construction-literal pin); round
three fully dry. Three live sabotage probes (forge a grant, make the request deserializable, inject a process
spawn) each failed the gate, restored byte-identical; a read-only panel agent's stray `test_alias` binary was
removed. No LLM, no training, no probe execution, no actual promotion; P12 still owns weights, P13–P15 stay
closed. `release_check` green + silent. Recorded in full in [a.md](../a.md) under "Observation Promotion Gate /
Still-No-Evidence Boundary (P17 / HYP-5)". Additive: HYP-0 through HYP-4 and all prior crates/docs 0-diff. Local
only — no remote push.

## DD-2026-06-18-E — Add the observation receipt quarantine (P16 / HYP-4) in-crate

**Decision.** Add `crates/hypothesis-layer/src/observation.rs` — a `ProbeObservationReceipt` derived from a
HYP-3 `ProbeExecutionIntent` that records a CLAIMED future probe result (`observation_text`) while remaining
`observation_only`: it can never become evidence, a claim, verifier input, or a memory mutation, and it does
not imply the probe ran. Doctrine: *Hypothesis proposes. Probe queue classifies. Governance reviews. HYP-3
records intent. HYP-4 quarantines observations. Nothing becomes evidence.* Kept INSIDE the existing crate (a
new module, no new dependency), so the serde-only quarantine is unchanged.

**Why.** HYP-3 records an execution intent but executes nothing; the next risk is the FORMAT a future probe
result would take. HYP-4 defines that format as a quarantine: an observation is minted only by `from_intent`,
which DERIVES the disposition from the intent — a `not_executed`/`blocked` intent yields `rejected`, a
`requires_operator` intent yields `requires_review`, and NO intent yields `recorded`. `recorded` is the
future-reserved promotion target; at HYP-4 nothing can be recorded, so an observation cannot quietly become a
result until a verifier/governance promotion path exists. The observation holds `observation_only` authority
(a single-variant enum) and reuses the forbidden-uses quarantine.

**Boundary (enforced by the compiler, types, the gate, and a behavioral surface).** A `ProbeObservationReceipt`
is minted only by `from_intent`, has private fields, and derives `Serialize` but not `Deserialize`
(`ObservationStatus`/`ObservationAuthority` are output-only, so the receipt is structurally non-deserializable
— a `compile_fail` proof, pinned live by cargo's doctest report). The disposition derivation is exhaustive
with no wildcard (E0004 on a new `ExecutionStatus`) and no arm yields `recorded`; the single-variant authority
is matched with no wildcard (E0004 on a second variant). The recorded-quarantine is a tested invariant
(`no_intent_disposition_yields_recorded`) AND a behavioral gate check (the example output must contain no
`recorded` token and `recorded == 0`). The observation binds its fields with an `integrity_hash`, cites its
provenance, and reuses the forbidden-uses quarantine so it can never become evidence. No execution code exists
in the crate (crate-wide gate scan over `src/` + examples). Verified by three read-only adversarial panel
rounds (round one fully dry; round two's five substantive lenses clean, with the gate-vacuity lens re-raising
the multi-file-forgery residual — reproduced first-hand and refuted, since the example is an independent
cross-file behavioral surface that catches a real `->recorded` regression even with the unit tests gutted, and
only coordinated multi-file fabrication bypasses it, which is beyond regression scope; an in-gate residual note
was added; round three fully dry post-fold) plus four live sabotage probes. No LLM, no training, no probe
execution; P12 still owns weights, P13–P15 stay closed. `release_check` green + silent. Recorded in full in
[a.md](../a.md) under "Observation Receipt Quarantine (P16 / HYP-4)". Additive: HYP-0, HYP-1, HYP-2, HYP-3,
and all prior crates/docs 0-diff. Local only — no remote push.

## DD-2026-06-18-D — Add the approved-probe execution stub / non-execution boundary (P16 / HYP-3) in-crate

**Decision.** Add `crates/hypothesis-layer/src/execution.rs` — a `ProbeExecutionIntent` derived from a HYP-2
`ReviewReceipt` that records what may happen to the probe NEXT (`not_executed` / `blocked` /
`requires_operator`) WITHOUT executing the probe, writing a probe result, or mutating anything. Doctrine:
*Hypothesis proposes. Probe queue classifies. Governance reviews. HYP-3 records intent. Nothing executes.
Nothing becomes evidence.* Kept INSIDE the existing crate (a new module, no new dependency), so the
serde-only quarantine is unchanged.

**Why.** HYP-2 can approve a probe; the next risk is that approval is mistaken for execution. HYP-3 makes the
execution boundary an explicit inert stub: an intent is minted only by `from_review`, which DERIVES the
disposition from the review — only an approved review yields a cleared intent (a disposition a human/operator
may run later), and a rejected or deferred review yields a `blocked` one. A blocked probe can never be
approved (HYP-2 refuses it), so it can never reach the cleared path. There is no `executed` status; HYP-3
records and runs nothing.

**Boundary (enforced by the compiler, types, the gate, and a behavioral surface).** A `ProbeExecutionIntent`
is minted only by `from_review`, has private fields, and derives `Serialize` but not `Deserialize`
(`ExecutionStatus`/`ExecutionReason` are output-only, so the intent is structurally non-deserializable — a
`compile_fail` proof, pinned live by cargo's doctest report). The disposition derivation and the
status-from-reason map are exhaustive with no wildcard (E0004 on a new variant), so a rejected/deferred review
can never derive a cleared status. The intent binds its fields with an `integrity_hash`, cites its provenance,
and reuses the forbidden-uses quarantine so it can never become evidence. No execution code exists in the
crate (crate-wide gate scan over `src/` + examples for any process/filesystem/network/side-effecting I/O).
Verified by two read-only adversarial panel rounds (five substantive lenses clean both rounds; the
gate-vacuity lens drove one fold — reproduced and refuted as stated, then a real strengthening: the gate now
greps all four `ExecutionReason` tokens against the least-fabricable surface, the real serialized intents
array, so each disposition is bound to genuine `from_review` output; round two fully dry) plus four live
sabotage probes. No LLM, no training, no probe execution; P12 still owns weights, P13–P15 stay closed.
`release_check` green + silent. Recorded in full in [a.md](../a.md) under "Approved Probe Execution Stub /
Non-Execution Boundary (P16 / HYP-3)". Additive: HYP-0, HYP-1, HYP-2, and all prior crates/docs 0-diff. Local
only — no remote push.

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
