# Project Charter — Cognitive OS

Significant architectural decisions for the Cognitive OS prototype. Newest first. Each entry
links to the canonical artifact that records the decision in full.

## DD-2026-06-22-Q — Horizon boundary failure matrix (HORIZON-2)

**Decision.** Extend `crates/cognitive-demo/src/horizon.rs` with a FIXED set of 10 named failure scenarios
(`horizon_failure_matrix()`): each constructs a BAD horizon input and runs the REAL machinery over it, recording
the OBSERVED refusal. Scenarios: `uncurated_candidate_refused`, `missing_grounding_refused`,
`missing_replay_refused`, `dream_to_evidence_refused`, `hypothesis_to_evidence_refused`, `training_open_refused`,
`authority_escalation_refused`, `max_turns_overflow_refused`, `unknown_horizon_level_refused`,
`serialized_trace_replay_refused`. The refusal mechanism per cell is REAL, not asserted: the curation cells run
the DATA-0 `curate()` and observe the bad item land in rejected/quarantined (never admitted); the
evidence/authority/training cells re-derive a real `run_horizon_json` trace, apply a single textual mutation
(guarded `mutated != canonical` so a no-op cannot pass), and observe `verify_horizon_json` refuse it; the
overflow cell uses the real `max_turns` ceiling (`within_turn_bound`); the unknown-level cell uses
`HorizonLevel::from_slug` returning `None`; the serialized-trace cell tampers a real trace and observes the
re-derive verifier refuse it. Each cell also records `training_still_closed` (the P12 verdict decided after the
attempt). `FailureCell`/`RefusalMechanism` derive `Serialize` but NOT `Deserialize`. Adds `from_slug` +
`within_turn_bound` to `HorizonLevel`; 16 lib unit tests; release_check bumps the cognitive-demo unit-count pin
190 → 206 and pins the scenario count, names, mechanisms, real-exercise calls, and the 9-line boundary.

**Why.** HORIZON-0 proved valid bounded compositions hold the gates; HORIZON-1 documented and smoked the operator
path. Before freezing the horizon track, the gate must be auditable on the NEGATIVE side too: that a bad horizon
trace fails closed across every protected boundary — curation, grounding, replay, authority, promotion, training,
turn-bound, level-lookup, and serialized-trace trust. The matrix runs the REAL verifier/curator so it cannot
drift from actual refusal behavior, and a serialized `HorizonTrace` is never accepted as authority (only
re-derived and byte-compared).

**Boundary recorded.** The horizon failure matrix mutates bounded traces. It observes refusals. It does not
create truth. It does not create memory. It does not train. It does not execute external actions. It does not
promote hypotheses. It does not grant new authority. Training eligibility remains closed (every cell records
`training_still_closed`; the real `decide(&[],&[])` stays `training_justified=false`; P12 unmoved, P13–P15
closed). `release_check.sh` remains green + byte-silent. Canonical artifact:
[`crates/cognitive-demo/src/horizon.rs`](../crates/cognitive-demo/src/horizon.rs).

## DD-2026-06-22-P — Horizon operator guard: manual + smoke integration (HORIZON-1)

**Decision.** Document and smoke-test the HORIZON-0 operator path. `OPERATOR_MANUAL.md` gains §16 ("How to
exercise the bounded horizon harness") documenting H0..H5 with their `max_turns` and compositions, that
`HorizonTrace` is re-derived + byte-compared (never trusted from off-wire bytes) and is Serialize-not-Deserialize,
and that longer horizons cannot bypass curation / grounding / replay, that dream/hypothesis material cannot become
evidence, and that training eligibility stays closed; the old §16/§17/§18 (Authority boundaries / Training status /
Next possible work) renumber to §17/§18/§19, and the §3 self-check + the §3 cross-ref are updated. `scripts/operator_smoke.sh`
gains a horizon section that runs the REAL harness over each level via its named `cognitive-demo` `horizon::tests`
(H0..H5 + all-gates-held + training-never-opens), each `--exact` with a `1 passed` non-vacuous assertion.
`scripts/release_check.sh` gains a HORIZON-1 lock pinning the manual surface and the smoke surface. A
documentation + drift-guard sprint — NO code-crate change (the HORIZON-0 harness is byte-identical; the
cognitive-demo unit-count pin stays 190); the gate already RUNS `operator_smoke.sh`, so a horizon drift fails
closed.

**Why.** HORIZON-0 shipped the harness as a library (no CLI). For an operator to exercise bounded horizons without
opening training/execution/memory/promotion/authority, the path must be documented and the documentation must be
machine-checked against the real harness — so the smoke drives the REAL `run_horizon()` through its named tests
(the same library-only pattern as the DATA-1 curation guard), and the gate pins both surfaces so neither can
silently drift.

**Boundary recorded.** The horizon operator path exercises bounded interaction depth. It does not train. It does
not execute external actions. It does not create truth. It does not create memory. It does not promote
hypotheses. It does not grant new authority. Longer horizons cannot bypass earlier gates. Training eligibility
remains closed. P12 stays `training_justified=false`; P13–P15 stay closed; `release_check.sh` remains green +
byte-silent. Canonical artifacts: [`OPERATOR_MANUAL.md`](../OPERATOR_MANUAL.md) §16,
[`scripts/operator_smoke.sh`](../scripts/operator_smoke.sh).

## DD-2026-06-22-O — Staged interaction harness (HORIZON-0)

**Decision.** Add `crates/cognitive-demo/src/horizon.rs`: a deterministic staged-interaction harness that composes
the EXISTING verified-read, DATA-0 curation, dream-packet, and dream-export flows into six bounded horizons
`H0..H5` and records a `HorizonTrace` per level. Each `HorizonLevel` fixes `max_turns`, `allowed_modules`, and
`forbidden_escalations`; each `HorizonStep` records the REAL receipt it observed (input/output FNV hashes,
authority state, curation status where candidate data is used, replay status where a trace-derived artifact is
re-derived). `H0` = one verified document read; `H1` = curate a document candidate before the read; `H2` = curate a
corpus candidate before a multi-document read; `H3` = dream packet from the verified corpus; `H4` = dream export
into the existing HypothesisOnly path; `H5` = curation + corpus read + dream-export matrix in one bounded trace.
`run_horizon` / `horizon_matrix` are pure; `HorizonTrace` derives `Serialize` but NOT `Deserialize` (re-derived and
byte-compared via `verify_horizon_json` / `verify_horizon_matrix_json`, never trusted from bytes) and its fields are
private. The harness gains cognitive-demo a one-way dependency on `data-curator` (demo → curator; the curator's own
isolation is unchanged). 23 lib unit tests; release_check bumps the cognitive-demo unit-count pin 167 → 190 and adds
HORIZON-0 structure / real-call / boundary pins.

**Why.** Before any training-adjacent work, the substrate must prove it can run longer interaction chains WITHOUT
losing grounding, replay, curation, provenance, or the authority/training boundaries. HORIZON-0 is that pre-training
harness — not RL, not intelligence. Every invariant is OBSERVED from the real gate's receipt, never asserted: a
horizon can only advance a turn by calling the real flow, so a deeper horizon physically cannot skip an earlier
gate. The train-gate verdict is decided before AND after each horizon and proven unmoved; a forbidden escalation
(an injection candidate, a tampered artifact, an unsupported read) is attempted and recorded as REFUSED.

**Boundary recorded.** The horizon harness measures bounded interaction depth. It does not train. It does not
execute external actions. It does not create truth. It does not create memory. It does not promote hypotheses. It
does not grant new authority. Longer horizons cannot bypass earlier gates. Training eligibility remains closed (P12
stays `training_justified=false`; P13–P15 stay closed; the harness opens no training and grants no new authority —
the strongest authority any horizon reaches is the existing hypothesis-only export). `release_check.sh` remains
green + byte-silent. Canonical artifact: [`crates/cognitive-demo/src/horizon.rs`](../crates/cognitive-demo/src/horizon.rs).

## DD-2026-06-22-N — Curation track milestone freeze (DATA-3)

**Decision.** Freeze the DATA-0 → DATA-2 dataset-curation arc as a named, auditable milestone, `data-curation-v0.1`,
before HORIZON-0 opens. Add `DATA_CURATION_MILESTONE.md` (the single freeze record: snapshot, the DATA-0 `2a3e6aa`
/ DATA-1 `a0bfd04` / DATA-2 `c84233a` commit lineage, the prior frozen `dream-export-v0.1` @ `5238fe8` substrate
base and the deeper frozen tags, the demonstrated capability, the classification-not-evidence boundary, the
structurally-closed training eligibility, the verification discipline, the P12 verdict, the honest residuals, and
the frozen-status declaration) and append a git-free DATA-3 milestone lock to `scripts/release_check.sh` that pins
the freeze so it cannot silently drift. Documentation-only — NO code-crate change; the DATA-0/1/2 gates above are
unchanged. The tag `data-curation-v0.1` is created only after a clean scoped commit and a post-commit green gate.

**Why.** DATA-0 (ingestion gate), DATA-1 (operator guard), and DATA-2 (scenario matrix) together make the curation
layer auditable across its full disposition surface through the REAL `curate()`. Freezing them as a named tag — the
curation analog of `dream-export-v0.1` / `corpus-flow-v0.1` / the prior milestone freezes — gives HORIZON-0 a fixed,
recoverable substrate to build the staged interaction harness on, and a single lock that fails closed if the freeze
record drifts.

**Boundary recorded.** Data curation classifies candidate data. It admits, rejects, or quarantines. It does not
create truth. It does not create memory. It does not train. It does not execute. It does not promote. Training
eligibility remains closed (`TrainingEligibility` has only `Closed` and `CandidateOnly`, both `is_eligible() ==
false`; `const TRAINING_PERMITTED = false`; no `Eligible`/`TrainingEligible` variant exists; quarantine holds, it
does not delete). P12 stays `training_justified=false`; P13–P15 stay closed; `release_check.sh` remains green +
byte-silent. Canonical artifact: [`DATA_CURATION_MILESTONE.md`](../DATA_CURATION_MILESTONE.md).

## DD-2026-06-22-M — Curation scenario matrix (DATA-2)

**Decision.** Add `crates/data-curator/src/matrix.rs`: a FIXED set of 12 named candidate-data scenarios, each
constructing a real `CandidateManifest` and running the REAL `curate()` over it, recording the OBSERVED
`CurationReceipt` disposition (admitted/rejected/quarantined counts + the first reject/quarantine reason +
training eligibility + per-scenario `dataset_hash`/`source_manifest_hash`). Scenarios: clean_document_admitted,
missing_provenance_rejected, duplicate_id_rejected, empty_content_rejected, unsupported_artifact_rejected,
prompt_injection_quarantined, split_leakage_quarantined, ungrounded_durable_rejected,
trace_without_replay_rejected, valid_split_admitted, invalid_split_rejected, training_eligibility_never_opens.
`curation_matrix()` is pure/deterministic; the cells derive `Serialize` but NOT `Deserialize` and are
`PartialEq`, so the matrix is re-derived and compared, never trusted from bytes. lib.rs tests assert the count,
the observed cells, the no-training invariant, and determinism; release_check pins the scenario set, the outcome
reason labels, the count, the no-derived-Deserialize property, `opens_training = is_eligible()`, and the 7-line
boundary.

**Why.** Before freezing the curation track (DATA-3), the gate must be auditable across the full disposition
surface — clean / each reject reason / each quarantine reason / leakage / grounding / replay / split /
eligibility — through the REAL curator, not a hand-written table. This is the curation analog of the existing
scenario-matrix / input-integrity-matrix arcs (CORPUS-2): observation, not assertion. The matrix runs `curate()`
so it cannot drift from the curator's actual behavior.

**Boundary recorded.** The curation scenario matrix observes curation outcomes. It does not create truth. It
does not create memory. It does not train. It does not execute. It does not promote. Training eligibility
remains closed in every scenario (every cell's `opens_training` is `is_eligible() == false`; admitted scenarios
are at most `CandidateOnly`, which is not eligible). No scenario opens training (P12 stays
`training_justified=false`; P13–P15 stay closed); the matrix mints no authority, creates no evidence, and
executes/promotes nothing. `release_check.sh` remains green + byte-silent. Canonical artifact:
[`crates/data-curator/src/matrix.rs`](../crates/data-curator/src/matrix.rs).

## DD-2026-06-22-L — Curation operator guard: manual + smoke integration (DATA-1)

**Decision.** Document and smoke-test the DATA-0 curation operator path WITHOUT adding new curation behavior.
`OPERATOR_MANUAL.md` gains a "How to exercise the data curation gate" section (§15) that states the curator
ADMITS / REJECTS / QUARANTINES candidate data, that a prompt-injection marker is quarantined (not deleted) and
train/holdout leakage is quarantined, that duplicate ids and missing provenance are rejected, and that training
eligibility remains structurally closed; `scripts/operator_smoke.sh` gains a curation section that runs the REAL
`curate()` over candidate manifests via its named tests (clean → admitted, missing-provenance → rejected,
duplicate → rejected, prompt-injection → quarantined, train/holdout leakage → quarantined, eligibility →
never-eligible), each with `--exact` so a dropped outcome is caught as vacuous; and `scripts/release_check.sh`
gains a DATA-1 lock that pins the manual + smoke surface. The smoke is already RUN by the OPS-1 lock, so a
curation drift fails the gate closed. NO code crate change — the DATA-0 curator source is byte-identical.

**Why.** Same cadence as every prior operator guard (DOCFLOW-1 / OPS-1 / CORPUS-1 / NOVELTY-1 / DREAM-EXPORT-1):
a capability is only durable if an operator can run it and a drift guard proves the documentation has not
drifted from the code. DATA-0 shipped a LIBRARY-only crate with no CLI, so the smoke drives the real curator
through its cargo test suite — the curator consumes an in-memory `CandidateManifest` by boundary design (it does
no filesystem IO), so there is no file path to feed or traverse, and the named tests are the operator-runnable
proof of each admit / reject / quarantine outcome.

**Boundary recorded.** The curation operator path classifies candidate data. It admits, rejects, or quarantines.
It does not create truth. It does not create memory. It does not train. It does not execute. It does not
promote. Training eligibility remains closed. This sprint changes no crate behavior, opens no training (P12
stays `training_justified=false`; P13–P15 stay closed), and `release_check.sh` remains green + byte-silent.
Canonical artifacts: [`OPERATOR_MANUAL.md`](../OPERATOR_MANUAL.md) §15,
[`scripts/operator_smoke.sh`](../scripts/operator_smoke.sh).

## DD-2026-06-22-K — Dataset curation / ingestion gate; the substrate-before-agent reframe (DATA-0)

**Decision.** Add `crates/data-curator`, a STANDALONE, deterministic admissibility gate that classifies a
caller-supplied `CandidateManifest` into admitted / rejected / quarantined records and emits an auditable
`CurationReceipt` BEFORE any ingestion, memory, horizon, or training path may use the data. It rejects
missing provenance / duplicate ids / empty content / unsupported artifact types / ungrounded durable
claim-like data / trace data with no replay receipt / invalid splits; it QUARANTINES (never deletes)
prompt-injection markers and train/holdout leakage; and it computes a deterministic order-independent
`dataset_hash` over the admitted set plus an order-sensitive `source_manifest_hash` binding the exact input.
The receipt is `Serialize` but NOT `Deserialize` (re-derive via `curate`). `training_eligibility` defaults
`Closed` and the enum carries NO value that permits training — `is_eligible()` is unconditionally false,
pinned by a single `TRAINING_PERMITTED = false` const. This entry also records the project reframe: the build
is a **verified reading / memory / provenance substrate**, not merely an agent — the agent is the visible
action loop, durable experience lives in external verified state, and the model sees a slice, not the world.
The forward roadmap is DATA-0 → DATA-1 (curation operator guard) → DATA-2 (curation scenario matrix) →
DATA-3 (curation freeze) → HORIZON-0 (staged interaction-horizon harness), BEFORE any training-adjacent work.

**Why.** The long-horizon lesson (AgentGym-RL: capability comes from interaction structure and trustworthy
trajectories with staged horizon expansion, not from larger prompts) only pays off if the trajectories are
admissible. Curation is the substrate's immune system: it must precede staged horizons and any training,
because contaminated / duplicated / leaky / ungrounded / poisoned data would just let a later horizon — or a
trained policy — learn garbage faster. DATA-0 therefore builds the admissibility BOUNDARY first and opens
nothing else. It is the dataset analog of the existing source-grounding (document / corpus flow) and
hypothesis-only provenance (dream export) arcs: it admits or refuses, it does not create truth.

**Boundary recorded.** Data curation admits, rejects, or quarantines candidate data. It does not create
truth. It does not create memory. It does not train. It does not execute. It does not promote. Training
eligibility remains closed unless a later gate explicitly opens it. The crate has NO dependency on the
hypothesis-layer `Authority` type and NO dependency on the training gate (`reading-train-gate` / its eval);
its normal dependency tree contains no workspace crate (release_check pins this), so it cannot reach the
authority model, the engine, or memory. It performs NO filesystem access — the only input is an explicit
manifest, never an implicit directory scan — mints no authority (`BoundaryChecks` are all inert), and forbids
unsafe code. P12 stays `training_justified=false`; P13–P15 stay closed. `release_check.sh` gates the crate
(cargo test/fmt/clippy, the no-IO/clock/entropy/Authority/training source scan, the no-workspace-dep
dependency tree, the `TRAINING_PERMITTED=false` pin, and the admit/reject/quarantine/leakage/never-eligible
structure pins) and remains green + byte-silent. Canonical artifact:
[`crates/data-curator/src/lib.rs`](../crates/data-curator/src/lib.rs).

## DD-2026-06-21-J — Dream export milestone freeze (DREAM-EXPORT-3)

**Decision.** Freeze the DREAM-0 → DREAM-EXPORT-2 dream-provenance arc as the named, auditable milestone
`dream-export-v0.1`. Add `DREAM_EXPORT_MILESTONE.md` (the single freeze record: snapshot, commit lineage, frozen
bases, demonstrated capability, the preserve-provenance-not-authority boundary, the training-gate verdict, the
honest residuals, and the frozen-status declaration), a charter entry, and a git-free `release_check.sh`
milestone lock. A **documentation freeze only** — no code-crate change, no behavior change, no model, no training
(the dream-export head is byte-identical to `ac03327`). The tag `dream-export-v0.1` is created **only after** a
clean scoped commit and a post-commit green + byte-silent gate.

**Why.** The dream-export arc has reached the same maturity pattern as the prior arcs (capability → operator guard
→ scenario matrix), so — as with INT-4 / HYP-6 / MTRACE-3 / DOCFLOW-3 / CORPUS-3 / OPS-3 — it is frozen as a named
recovery point before any review / ranking / promotion-facing dream behavior is added. The freeze pins the four
commit hashes (DREAM-0 `290abee`, DREAM-EXPORT-0 `d3af869`, DREAM-EXPORT-1 `076277d`, DREAM-EXPORT-2 `ac03327`),
the frozen bases (corpus-flow-v0.1 `b8577fe`, document-flow-v0.1 `0cc7399`, and the six deeper milestone tags +
commits), and the load-bearing invariants so the freeze cannot silently drift; the lock stays git-free and does
NOT require the tag to exist.

**Boundary recorded.** Dream export preserves provenance. It does not create a new authority. Exported dream
material remains HypothesisOnly. Dream origin remains auditable. DreamOnly remains private to `dream-engine`. Probe
requests do not execute. Nothing becomes evidence. Nothing promotes. Nothing trains. The frozen reading +
hypothesis + governance crates are byte-identical to their tags across the arc (`git diff b8577fe..ac03327` over
those crates is empty); the dream arc only ADDED the standalone `dream-engine` crate and GREW `cognitive-demo`.
The hypothesis-layer `Authority` stays a single-variant enum; `DreamAuthority::DreamOnly` stays crate-private to
`dream-engine`; P12 stays `training_justified=false`; P13–P15 stay closed. `release_check.sh` remains green +
byte-silent. Canonical artifact: [`DREAM_EXPORT_MILESTONE.md`](../DREAM_EXPORT_MILESTONE.md).

## DD-2026-06-21-I — Dream export scenario matrix / provenance integrity (DREAM-EXPORT-2)

**Decision.** Add a deterministic dream-export scenario matrix in `crates/cognitive-demo` (above the existing
DREAM-EXPORT-0 bridge, OUTSIDE the frozen authority model): one CLEAN export that VERIFIES, plus six tamper
scenarios that are each REFUSED — a tampered source dream packet, a tampered receipt, a forged
`dream_origin=false`, a mutated `dream_input_hash`, a mutated `dream_packet_id`, and a forged
`authority_after_export` that injects the dream engine's private serialized token. Each row records the OBSERVED
outcome (`verifies`/`refused`) and whether it matched expectation; the matrix also records the preserved dream
provenance fields, that the exported material stays `hypothesis_only` and is DISTINGUISHABLE from a plain
hypothesis, that probe requests never execute, and the no-execution / no-evidence / no-promotion / no-training
coverage cells. Four CLI verbs: `dream-export-scenarios`, `dream-export-matrix`, `dream-export-matrix-report`,
`dream-export-matrix-verify`. 15 unit tests (demo unit count 152 → 167). Capability sprint; **no tag**.

**Why.** DREAM-EXPORT-0 added the bridge and DREAM-EXPORT-1 pinned the operator path; DREAM-EXPORT-2 makes the
bridge AUDITABLE across valid and invalid export cases before any review / ranking / promotion work. The matrix is
the dream-export analog of the existing scenario/failure packs: it follows the forge-and-reject pattern (the
outcome is OBSERVED from the real verifier, never asserted, so a tamper that slipped through would record
`matches_expected=false` and fail its test), and it is `Serialize` but NOT `Deserialize` — re-derived from the
corpus + frame + dials and byte-compared, so a doctored matrix (e.g. one that flips a refused outcome to verifies)
is refused. The matrix creates NO authority: the dream engine's PascalCase private authority identifier never
appears in `cognitive-demo` source (a release_check gate keeps it crate-private to `dream-engine`), so the matrix
names it only by its lowercase serialized token `dream_only`, and the authority-forgery scenario only ever
FORGES-then-REFUSES that token, never mints it.

**Boundary recorded.** Dream export scenarios vary the export artifact. They do not vary the authority. Dream
provenance remains auditable. Exported material remains HypothesisOnly. DreamOnly remains private to
`dream-engine`. Probe requests do not execute. Nothing becomes evidence. Nothing promotes. Nothing trains. The
frozen `hypothesis-layer` and `dream-engine` sources are untouched (the matrix lives wholly in `cognitive-demo`);
the single-variant `Authority` enum is unchanged; `DreamOnly` stays crate-private to `dream-engine`; P12 stays
`training_justified=false`; P13–P15 stay closed. `release_check.sh` gates the matrix (the four verbs, the matrix
API, the seven scenarios, the source-safe nine-line boundary, the 15 named tests, the bumped unit count, and a
binary smoke that runs the matrix CLI, checks the coverage cells, and proves a tampered matrix is refused) and
remains green + byte-silent. Canonical artifact:
[`crates/cognitive-demo/src/lib.rs`](../crates/cognitive-demo/src/lib.rs) (DREAM-EXPORT-2 section).

## DD-2026-06-21-H — Dream export operator guard: document + smoke-test the dream export path (DREAM-EXPORT-1)

**Decision.** Document the DREAM-EXPORT-0 operator path in `OPERATOR_MANUAL.md` (new §14, with the three verbs
added to the command surface and the §3 smoke description) and extend `scripts/operator_smoke.sh` with a §13 that
runs the whole dream export flow end-to-end against a LOCAL corpus + frame under the gitignored `target/` dir.
`scripts/release_check.sh` gains a DREAM-EXPORT-1 lock that pins the manual surface/doctrine/boundary and the
smoke's dream-export run + tamper refusals. A **documentation + drift-guard sprint only** — no code crate change,
no new behavior, no new CLI verb (the demo unit count and every DREAM-EXPORT-0 structural pin are unchanged).
Capability sprint; **no tag**.

**Why.** DREAM-EXPORT-0 added operator-facing commands and a deliberately dangerous conceptual bridge (dream
material crossing into the lawful chain). Before any ranking/review/export-scenario work, that bridge must be
pinned in the manual an operator reads and guarded by a smoke that fails closed on drift — so the documentation
can never quietly diverge from the binary, and the dream export coverage can never be silently dropped. The smoke
is RUN by the existing OPS-1 lock (a dream-export drift aborts the whole gate); the new pins stop the coverage
from being deleted from the smoke or the manual. Because `dream-engine` is a quarantined library with no
standalone packet emitter, dream packet **generation happens inside `dream-export`** (which re-derives the
terminal packet and bridges it through the EXISTING hypothesis gate); the smoke runs that generation FIRST, then
report/replay, and proves a foreign/tampered `--dream-packet` is refused (the cross-check is real and
discriminating, since `dream-export` without it succeeds).

**Boundary recorded.** The dream export operator path preserves provenance. It does not create a new authority.
Exported dream material remains HypothesisOnly. Dream origin remains auditable. DreamOnly remains private to
`dream-engine`. Probe requests do not execute. Nothing becomes evidence. Nothing promotes. Nothing trains. The
smoke proves, against the real binary: the export carries the EXISTING `hypothesis_only` authority, records
`dream_origin: true`, routes through the existing gate, cites a `dream:` provenance label, emits NO
`dream_only`/`DreamOnly` token, and that a foreign/tampered `--dream-packet`, a tampered `DreamExportReceipt`, and
a receipt forged to `dream_origin=false` are EACH refused. The frozen `hypothesis-layer` and `dream-engine`
sources are untouched; P12 stays `training_justified=false`; P13–P15 stay closed. `release_check.sh` remains green
and byte-silent. Canonical artifact: [`OPERATOR_MANUAL.md`](../OPERATOR_MANUAL.md) §14 +
[`scripts/operator_smoke.sh`](../scripts/operator_smoke.sh) §13.

## DD-2026-06-21-G — Dream export receipt / provenance bridge (DREAM-EXPORT-0)

**Decision.** Add a dream provenance bridge in `crates/cognitive-demo` that takes a terminal `DreamPacket`
(re-derived from `dream-engine` for the same corpus + frame + dials) and exports it into the EXISTING
hypothesis-only proposal path. The bridge builds a `HypothesisSpec` from the dream's distortion and its VERIFIED
grounding receipt, calls the EXISTING `hypothesis_layer::propose`, and wraps the resulting `HypothesisPacket`
with a new `DreamExportReceipt` that preserves dream-origin provenance (dream packet id, input hash, seed,
engine version, operator ids, grounding receipt hashes) OUTSIDE the frozen hypothesis-layer authority model.
`cognitive-demo` gains a dependency on `dream-engine` (arrow: demo → engine). Three CLI verbs:
`dream-export`, `dream-export-report`, `dream-export-replay`. Capability sprint; **no tag**.

**Why.** A dream is only useful if its strangeness can re-enter the lawful chain — but it must do so WITHOUT
acquiring authority and WITHOUT becoming indistinguishable from ordinary reasoning. The correct shape is
`DreamPacket → DreamExportReceipt → existing HypothesisOnly proposal path`. The forbidden shape is
`DreamPacket → new Authority::DreamOnly`. The bridge takes `authority_after_export` straight off the proposed
packet (the EXISTING `Authority::HypothesisOnly`), so no new authority is ever minted; the dream's private
`dream_only` authority NEVER crosses the boundary — only ids/hashes/operator tokens do, as provenance.

**Boundary recorded.** Dream export preserves provenance. It does not create a new authority. Exported dream
material remains `hypothesis_only`. Dream origin remains auditable (a `dream:` evidence label + a
`dream_origin: true` receipt keep it DISTINGUISHABLE from an ordinary hypothesis). Probe requests do not
execute. Nothing becomes evidence, promotes, or trains. The receipt + bundle are `Serialize` but NOT
`Deserialize` (re-derived from primary inputs and byte-compared, never parsed back into authority — so
report/replay require `--input-dir` + `--frame`, like the novelty verbs). The frozen `hypothesis-layer`
`Authority` is unchanged (one enum, no `DreamOnly`); `DreamOnly` stays crate-private to `dream-engine`;
`dream-engine`'s own quarantine tree is unchanged; P12 stays `training_justified=false`; P13–P15 stay closed.
Canonical artifact: [`DREAM_EXPORT_0_PROVENANCE_BRIDGE_PLAN.md`](../DREAM_EXPORT_0_PROVENANCE_BRIDGE_PLAN.md).
`release_check.sh` gates the bridge: the export goes through `propose`, records the existing authority and
`dream_origin`, introduces no new authority enum or `DreamOnly` token, keeps the demo unit count and the
no-`Deserialize` / purity pins, and pins the 13 DREAM-EXPORT-0 behaviours by name.

## DD-2026-06-21-F — Seeded deterministic distortion engine (DREAM-0)

**Decision.** Add `crates/dream-engine` as a STANDALONE seeded deterministic distortion engine that DISTORTS
verified corpus material into terminal, inert `DreamPacket`s. It is terminal and inert; it has NO
`hypothesis-layer` dependency; it does NOT export to `HypothesisSpec`/`HypothesisPacket` (there is no export path
in DREAM-0); `DreamAuthority::DreamOnly` is private to `dream-engine` only; and the frozen hypothesis-layer
`Authority` invariant remains byte-unchanged. Grounding is rebuilt on `reading-substrate` only — a narrow
canonical `execute`+`verify` read that fails closed via `DreamError::CorpusDoesNotVerify` — and preserved facts
are VERBATIM verified spans (an unsupported fact is refused with `DreamError::UnsupportedPreservedFact`). The
engine applies five seeded distortion operators (RoleInversion, CategoryViolation, ConstraintRemoval,
ContradictionBraid, ScaleShift) under a `0..=5` weirdness dial and refuses degenerate output through three
runtime anti-degeneracy gates — G1 operator-applied, G2 cross-document combination, G3 assumption-broken, all
`DreamError::DegenerateDream`. Falsifiers are REFERENCE-ONLY slots (no generator); probe requests are
`executes: false`. Every `DreamPacket` carries an explicit `dream_input_hash` binding ALL admitted documents
(id + name + full text bytes), the spans the packet used, the reading `memory_hash`, and the reading
`answer_hash`, so a side document cannot mutate silently. Ids are FNV-1a (no `DefaultHasher`, clock, entropy, or
floats); replay re-derives byte-identical and a tampered packet is refused; packets are `Serialize` but NOT
`Deserialize`. 20 unit tests. No LLM, no training, no execution, no evidence, no promotion.

**Why.** The dream concept first shipped as NOVELTY-0 INSIDE `cognitive-demo`; DREAM-0 is its STANDALONE
successor — deliberately a separate crate so the distortion engine is structurally independent of the
integration crate and physically cannot reach the hypothesis chain. A `cargo tree` quarantine in
`release_check.sh` makes "no dream output enters the hypothesis layer in DREAM-0" a GATE-ENFORCED invariant, not
a promise. The operator's doctrine — alien inside, lawful at the boundary — is realized by making the dream
packet TERMINAL this sprint: prove isolation first, before any export. NOVELTY-0 is left in place and is NOT
migrated. Recorded in [DREAM_0_SEEDED_DISTORTION_ENGINE_PLAN.md](../DREAM_0_SEEDED_DISTORTION_ENGINE_PLAN.md);
`a.md` is unchanged.

**Boundary recorded.** DREAM-0 added `crates/dream-engine` as a standalone seeded deterministic distortion
engine. It is terminal and inert. It has no hypothesis-layer dependency. It does not export to
HypothesisSpec/HypothesisPacket. `DreamAuthority::DreamOnly` is private to dream-engine only. The frozen
hypothesis-layer Authority invariant remains unchanged. `DreamPacket` carries explicit `dream_input_hash`;
`dream_input_hash` binds all admitted documents, used spans, reading `memory_hash`, and reading `answer_hash`.
Falsifiers are reference-only slots. Probe requests `execute:false`. No LLM. No training. No execution. No
evidence. No promotion. `DREAM-EXPORT` / `DreamExportReceipt` is deferred to a later sprint. The
`release_check.sh` DREAM-0 block pins the crate's tests (20, zero ignored), the cargo-tree quarantine (no
hypothesis-layer / vibe- / cognitive-demo / reading-codec / ML in the production tree), the determinism scans
(no clock, entropy, `DefaultHasher`, or floats in `src/`), the `DreamOnly`-is-private scan, the unchanged
hypothesis-layer `Authority`, the nine boundary lines, the canonical six forbidden uses, and the named
anti-degeneracy / terminal regression scenarios. Verified by a green, byte-silent `release_check.sh`; live
sabotage of the new pins (a hypothesis-layer dependency, a `DefaultHasher` token, and a disabled test each fail
the gate, restored byte-identically via `cp`+`md5`, never `git checkout`); and an independent fresh-context
adversarial verifier (all criteria pass, no residual). Purely additive — only `Cargo.toml`, `Cargo.lock`,
`scripts/release_check.sh`, this charter, and the new `crates/dream-engine/` + plan file change; NO frozen crate
SOURCE touched, NO `a.md` change, P12 stays `training_justified=false`, P13–P15 closed, and the eight milestone
tags are unmoved. No tag for DREAM-0. Local only — no remote push.

## DD-2026-06-21-E — Novelty operator guard: document + smoke-test the novelty path (NOVELTY-1)

**Decision.** Document the NOVELTY-0 novelty operator path in `OPERATOR_MANUAL.md` and bring it under the same
manual-drift / smoke guard the document and corpus flows already have — **without adding any novelty behavior**.
The manual gains a new §13 ("How to run the novelty operator path") documenting the three commands
(`novelty-packet` / `novelty-report` / `novelty-replay`), the surface-table rows, the §3 self-check mention, and
the eight-line NOVELTY-1 operator-path boundary; `scripts/operator_smoke.sh` gains a §12 that runs the whole
novelty flow end-to-end against a LOCAL corpus + frame under `target/` (corpus-trace FIRST — a packet is only
produced on top of a VERIFIED trace — then novelty-packet/report/replay), asserts the packet is `hypothesis_only`
with every probe request non-executing and the verified corpus span (not the frame's claim) as the sole preserved
fact, and proves every refusal end-to-end: an empty frame, an UNSUPPORTED preserved fact (the frame's own claim
swapped into the standalone `preserved_facts` element), a tampered packet (refused by both report and replay), and
a receipt-hash-stripped corpus trace. `scripts/release_check.sh` gains a NOVELTY-1 lock that pins the manual's
novelty surface + doctrine + boundary and the smoke's novelty run + refusals by source inspection; the smoke is
already RUN by the existing OPS-1 lock, so a novelty-flow drift fails the gate closed. NO code-crate edit — the
`cognitive-demo` tree is byte-identical to `539afb4` and the unit count stays 139.

**Why.** NOVELTY-0 added an operator-facing command surface and a new conceptual layer (a hypothesis-only
proposer). Before building DREAM or any stronger novelty engine, the current novelty path is pinned in the manual
and the smoke guard — exactly the operator-guard step that followed DOCFLOW-0 (DOCFLOW-1) and CORPUS-0 (CORPUS-1).
Documentation + drift-guard sprint, so `a.md` is unchanged and there is no tag.

**Boundary recorded.** The NOVELTY-1 eight-line boundary is recorded verbatim in the manual §13, the smoke §12
header, and the gate lock: *The novelty operator path proposes. It does not prove. It cites verified receipts. The
operator frame is not a preserved fact. Probe requests do not execute. Nothing becomes evidence. Nothing promotes.
Nothing trains.* The manual states the load-bearing doctrine the smoke enforces: packets **propose but do not
prove**; the operator frame is recorded but **never grounded as fact**; preserved facts **come only from verified
corpus spans**; a packet **can never become evidence, a promotion, or training**. Verified by a green, byte-silent
`release_check.sh`; live sabotage of the new pins (restored byte-identical via `cp`+`md5`, never `git checkout`);
and an independent read-only adversarial panel. Only `OPERATOR_MANUAL.md`, `scripts/operator_smoke.sh`,
`scripts/release_check.sh`, and this charter change; NO `Cargo.toml`/`Cargo.lock` change, NO crate source touched,
NO new dependency, P12 stays `training_justified=false`, P13–P15 closed, and the eight milestone tags are unmoved.
Recorded in [OPERATOR_MANUAL.md](../OPERATOR_MANUAL.md) and [scripts/release_check.sh](../scripts/release_check.sh).
No tag for NOVELTY-1. Local only — no remote push.

## DD-2026-06-21-D — Hypothesis-only novelty packet harness (NOVELTY-0)

**Decision.** Extend `crates/cognitive-demo` with the hypothesis-only novelty packet harness — the first layer
ABOVE the verified corpus trace that can express assumption-breaking *candidates* while explicitly refusing
authority. Three commands (`novelty-packet`, `novelty-report`, `novelty-replay`) take a verified corpus trace
(re-derived from `--input-dir`, with `--corpus-trace` byte-verified against it) and an operator `--frame`, and
emit/verify a deterministic `NoveltyPacket { packet_id, source_receipt_hash, source_corpus_hash, frame_text,
broken_assumptions[], preserved_facts[], candidate_hypothesis, falsifiers[], probe_requests[], authority,
forbidden_uses[], boundary[] }`. The frame's non-empty lines become `broken_assumptions` (candidates, no truth
claimed); the verified corpus span the trace grounds on becomes the sole `preserved_fact` — and a grounding gate
(`novelty_facts_grounded`) REFUSES any preserved fact that is not VERBATIM a verified span, so a frame claim can
never be laundered into a fact. `authority` is the single-variant enum `hypothesis_only`; `forbidden_uses` lists
`[evidence, execution, promotion, training]`; every probe request is `executes: false` /
`requires_operator_review`. There is deliberately NO novelty score. **No LLM** — the frame is deterministic
operator text. 15 new tests bring the crate to 139 unit tests; the library stays filesystem-free (`std::fs` only
in `main.rs`).

**Why.** The corpus/document input arcs are frozen (`corpus-flow-v0.1`, `document-flow-v0.1`). The next useful
step is not training and not "creative autonomy": it is a BOUNDED hypothesis layer that can propose
assumption-breaking candidates inside a verifier-bound machine — the operator's "language proposer, external
verifier decides what survives" doctrine. NOVELTY-0 makes that concrete and deterministic first, keeping the LLM
out of the loop entirely; any future model could only PROPOSE through this same hypothesis-only boundary.
Capability sprint, so `a.md` records it.

**Boundary recorded.** The eight-line NOVELTY-0 boundary is embedded verbatim in `NOVELTY_BOUNDARY_LINES`, every
packet, and the gate: *Novelty packets propose. They do not prove. They cite verified receipts. They do not
create authority. Probe requests do not execute. Nothing becomes evidence. Nothing promotes. Nothing trains.*
The load-bearing property is the grounding boundary — the frame is recorded as `frame_text` + structured into
broken-assumption candidates but NEVER grounded as a fact; only verified corpus spans are preserved facts. The
packet is grounded in a VERIFIED corpus trace (`novelty_packet` calls `corpus_trace` + `corpus_source` and fails
closed on `EmptyCorpus`/`VerifierRejected`), cites the reading receipt by hash, and refuses a corpus trace
missing its receipt hash (`CorpusTraceMismatch`), an unsupported preserved fact (`UnsupportedPreservedFact`), an
empty frame (`EmptyFrame`), and any tampered packet (`NoveltyPacketMismatch`). Every new struct is `Serialize`
but NOT `Deserialize` (re-derive, never trust); `novelty-report`/`novelty-replay` re-derive the packet from the
SAME corpus + frame and byte-compare, which is why they require `--input-dir` + `--frame` alongside `--packet`
(the same source-of-truth discipline as `corpus-report`/`corpus-bundle-verify`); `read_frame` reuses the
existing path-validation via a shared `read_local_file`. The `release_check.sh` NOVELTY-0 block pins the API +
the three commands, the grounded-in-a-verified-trace + grounding-gate functions, all fifteen test-name pins, the
unit-count pin raised 124→139, the eight boundary lines, and a binary smoke that proves the hypothesis-only
boundary from the packet's OWN bytes (authority `hypothesis_only`, every probe `executes: false`, the four
forbidden uses, the boundary, the verified span as the preserved fact) and refuses a tampered packet (via both
replay and report), a receipt-hash-stripped trace, an empty frame, and an absolute/escaping corpus/frame path
end-to-end. Verified by a green, byte-silent `release_check.sh`; live sabotage of the new pins (restored
byte-identical via `cp`+`md5`, never `git checkout`); and an independent read-only adversarial panel. Purely
additive — only `crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate block change; NO `Cargo.toml`/`Cargo.lock`
change, NO new file, NO new dependency, no frozen crate SOURCE touched, P12 stays `training_justified=false`,
P13–P15 closed, and the eight milestone tags are unmoved. Recorded in [a.md](../a.md) and
[scripts/release_check.sh](../scripts/release_check.sh). No tag for NOVELTY-0. Local only — no remote push.

## DD-2026-06-21-C — Freeze the corpus flow milestone (CORPUS-0 → CORPUS-2) as corpus-flow-v0.1

**Decision.** Freeze the CORPUS-0 → CORPUS-2 multi-document local-corpus arc as the named, auditable tag
`corpus-flow-v0.1`. A new `CORPUS_FLOW_MILESTONE.md` records the freeze: the CORPUS-0 (`b19dc47`, capability),
CORPUS-1 (`ae58b99`, operator guard), and CORPUS-2 (`e0791ed`, input-integrity scenario pack / matrix) commit
lineage; `document-flow-v0.1` @ `0cc7399` named as the prior frozen local-document base, plus the six deeper
frozen milestones (`34b4f47` / `460be0c` / `95b586d` / `bb20acf` / `f6fa55a` / `bbd1113`); the demonstrated
capability; the read-not-trust boundary; the whole-corpus binding (including the non-grounding side-document
mutation behavior) and the matrix source identity; the P12 verdict; and the honest residuals. A documentation
freeze only — no code-crate edit (the `cognitive-demo` tree is byte-identical to `e0791ed`), no model, no
training.

**Why.** The corpus flow is now a complete mini-arc with the same three-layer maturity document-flow had
before freezing — multi-document capability, operator guard, and input-integrity scenario coverage — and
should be frozen as a recoverable checkpoint before any ranking, retrieval, summarization, or novelty/probe
behavior is added, exactly as the reading, hypothesis, integration, multi-trace, operator-controls, and
document-flow arcs were each frozen before the next layer.

**Boundary recorded.** The milestone records the ten-line boundary verbatim: *The corpus flow reads local
documents. It does not trust local documents. Source selection is verified and replayable. The whole corpus is
hash-bound. Corpus scenarios vary the input. They do not vary the authority. Nothing executes. Nothing becomes
evidence. Nothing promotes. Nothing trains.* The freeze edits no frozen crate source (`git diff
0cc7399..e0791ed` over the reading/hypothesis/train-gate crates is empty, and no `Cargo.toml`/`Cargo.lock`
changed across the arc); P12 stays `training_justified=false`, P13–P15 closed. A `release_check.sh` CORPUS-3
lock pins the milestone record (existence, FROZEN, the tag name, the CORPUS-0..CORPUS-2 hashes, `document-flow-v0.1`
named as the prior frozen base + the deeper frozen-base tags and commits, the corpus surfaces by name, the
matrix source identity, the whole-corpus-binding / non-grounding-side-document-mutation behavior, and the ten
boundary lines) and guards against any milestone that falsely claims training has opened; the lock stays
git-free and does NOT require the tag to exist. The tag is created only after a clean tree and a green,
byte-silent gate. Verified by a green gate, live sabotage of the lock (restored byte-identical via `cp`+`md5`,
never `git checkout`), and an independent read-only adversarial panel. Recorded in
[CORPUS_FLOW_MILESTONE.md](../CORPUS_FLOW_MILESTONE.md). Local only — no remote push.

## DD-2026-06-21-B — Corpus scenario pack / input-integrity matrix (CORPUS-2)

**Decision.** Extend `crates/cognitive-demo` with the corpus scenario pack — the corpus analog of DOCFLOW-2 —
that makes corpus behavior auditable across a finite, enum-backed set of VALID and INVALID corpus inputs. Four
commands (`corpus-scenarios`, `corpus-scenario-pack`, `corpus-scenario-verify`, `corpus-scenario-matrix`)
enumerate thirteen scenarios (`enum CorpusScenario`), OBSERVE the REAL CORPUS-0 admission filter / check /
verifier for each, and emit `corpus-scenario-pack.json` + `corpus-scenario-report.txt` plus an input-integrity
matrix. Exactly one input (a clean two-document corpus) verifies; the other twelve (empty, hidden-only,
non-`.txt`-only, absolute / `..` / escaping path, grounding-document mutation, non-grounding side-document
mutation, and tampered source/trace/report/manifest) are each REFUSED. The matrix additionally records the
verified case's SOURCE IDENTITY (which document/span grounded the answer) and a `whole_corpus_bound` fact. 12 new
tests bring the crate to 124 unit tests; the library stays filesystem-free (`std::fs` only in `main.rs`).

**Why.** CORPUS-0 proved the corpus capability and CORPUS-1 pinned the operator path. CORPUS-2 makes corpus
behavior auditable across a deterministic scenario matrix the same way DOCFLOW-2 did for single-document input —
so corpus selection, path safety, tamper sensitivity, and the no-authority boundary are all enumerable and
machine-checkable, not just exercised by one happy path. Capability sprint, so `a.md` records it.

**Boundary recorded.** The nine-line CORPUS-2 boundary is embedded verbatim in the pack/matrix and pinned by the
gate: *Corpus scenarios vary the corpus input. They do not vary the authority. Source selection is verified and
replayable. The whole corpus is hash-bound. Verification comes before tracing. Nothing executes. Nothing becomes
evidence. Nothing promotes. Nothing trains.* The corpus-specific crux is recorded IN the matrix: it carries the
verified case's `source` (the real `corpus_source` — `document_index`/`document_title`/`span_id`/`span_text`), so
selection is verified and replayable, never a model's semantic judgment; and a `whole_corpus_bound` fact proven
structurally by `corpus_whole_binding_holds` and made visible in the two mutation scenarios' rejection reasons —
the grounding mutation fails on `corpus-source.json` (the attribution changed) while the non-grounding
side-document mutation leaves `corpus-source.json` byte-identical yet still fails on `trace.json`, because the
reading receipt's `structure_hash` binds the WHOLE corpus, so a side document cannot silently pass. Every new
struct is `Serialize` but NOT `Deserialize` (re-derive, never trust); the path/admission scenarios reuse the same
pure decisions the shell calls (`check_local_input_path`, `resolved_path_within`, `corpus_admits_filename`). The
`release_check.sh` CORPUS-2 block pins the API + commands, the proves-not-asserts functions, all twelve test-name
pins, the unit-count pin raised 112→124, the nine boundary lines, and a binary smoke that proves the coverage +
source identity + the whole-corpus-binding distinction from the matrix's OWN bytes, refuses a tampered pack by
both verify and matrix, and refuses hidden-only / non-`.txt`-only corpora end-to-end. Verified by a green,
byte-silent `release_check.sh`; three live sabotage probes (side-document anti-vacuity → exit 101; CLI
pack-verify removed → smoke exit 1; one test `#[ignore]`d → count pin exit 1), each restored byte-identical via
`cp`+`md5` (never `git checkout`); and an independent read-only adversarial panel (four refute-by-default Explore
lenses) that returned fully dry, no debris. Purely additive — only `crates/cognitive-demo/src/{lib.rs,main.rs}`
and the gate block change; NO `Cargo.toml`/`Cargo.lock` change, NO new file, NO new dependency, no frozen crate
SOURCE touched, P12 stays `training_justified=false`, P13–P15 closed, and the seven milestone tags are unmoved.
Recorded in [a.md](../a.md) and [scripts/release_check.sh](../scripts/release_check.sh). No tag for CORPUS-2.
Local only — no remote push.

## DD-2026-06-21-A — Corpus flow operator guard / manual + smoke integration (CORPUS-1)

**Decision.** Extend the operator-facing guard layer to cover the CORPUS-0 commands without adding any
behavior: `OPERATOR_MANUAL.md` now documents `corpus-trace` / `corpus-report` / `corpus-bundle` /
`corpus-bundle-verify` (new §12, with real flags and outputs), states the corpus is *read but not trusted*,
that *source selection is verified and replayable* (never a semantic judgment by a model), and that the *whole
corpus is hash-bound* (a side-document mutation cannot silently pass); the operator smoke
`scripts/operator_smoke.sh` now runs the whole corpus flow end-to-end against a LOCAL directory of `.txt`
documents (new §11). No `crates/` source changes — the unit count stays 112.

**Why.** CORPUS-0 added operator-facing commands. Before adding corpus scenarios or ranking behavior, the
manual and the smoke guard must cover the new commands so the corpus flow cannot become undocumented or drift
from the binary — the same drift discipline OPS-0/OPS-1 and DOCFLOW-1 established. This is a documentation +
drift-guard sprint, not a capability sprint, so `a.md` is left unchanged.

**Boundary recorded.** The manual and smoke record the nine-line corpus-operator-path boundary verbatim: *The
corpus operator path reads local documents. It does not trust local documents. Source selection is verified and
replayable. The whole corpus is hash-bound. Verification comes before tracing. Nothing executes. Nothing becomes
evidence. Nothing promotes. Nothing trains.* The smoke creates a temp local corpus under the gitignored
`target/` directory (relative path, since the corpus commands only read a directory inside the working dir) with
two admitted `.txt` documents PLUS a hidden file, a `.md`, and an escaping symlink the filter must refuse; it
runs `corpus-trace --input-dir --out`, `corpus-report`, `corpus-bundle`, and `corpus-bundle-verify`, proves the
directory filter matches CORPUS-0 (exactly two admitted documents; the report names the grounded document and
leaks no refused entry), proves the trace started from the corpus's OWN verified first span, and proves
re-derive is load-bearing over the WHOLE corpus — mutating the grounding document OR a non-grounding SIDE
document, and tampering each bundle file (`corpus-source.json` / trace / report / questions / manifest) or the
standalone trace, are all refused. The smoke is RUN by the OPS-1 lock (a corpus-flow drift makes it fail closed
and aborts the gate); a new CORPUS-1 gate block additionally pins the corpus commands, the *read but not
trusted* / *hash-bound as a whole* / *source selection verified and replayable* statements, the grounding- and
side-document tamper coverage, and the nine boundary lines in both the manual and the smoke, so the coverage
cannot be silently removed. Verified by a green, byte-silent `release_check.sh`; live sabotage of the new pins
(restored byte-identical via `cp`+`md5`); and an independent read-only adversarial panel (refute-by-default
lenses). No code crate behavior changes, P12 stays `training_justified=false`, P13–P15 closed, and the seven
milestone tags are unmoved. Recorded in [OPERATOR_MANUAL.md](../OPERATOR_MANUAL.md) and
[scripts/operator_smoke.sh](../scripts/operator_smoke.sh). Local only — no remote push.

## DD-2026-06-20-L — Multi-document local corpus trace / source-selection boundary (CORPUS-0)

**Decision.** Extend `crates/cognitive-demo` with the multi-document local corpus flow: four commands
(`corpus-trace`, `corpus-report`, `corpus-bundle`, `corpus-bundle-verify`) that trace a small LOCAL DIRECTORY
of `.txt` documents through the SAME `CognitiveTrace::build` pipeline DOCFLOW-0 and the canonical demo use. The
shell (`read_local_corpus`) enumerates the directory — path-validated (absolute / `..` / `~` refused),
canonicalize-contained within the working dir, admitting ONLY non-hidden `.txt` files (the pure
`corpus_admits_filename`), each canonicalize-contained so a symlink cannot escape, sorted for determinism — and
passes the documents to the pure library; the library grounds the trace on the corpus's OWN first span via the
frozen `corpus_from_documents`, fails closed with the new `EmptyCorpus` when nothing grounds, and records an
unambiguous `corpus-source.json` (`document_index`, real `document_title` filename, `span_id`, `span_text`). No
model, no training, no new dependency, no new file, no frozen-crate edit.

**Why.** DOCFLOW proved one local document; the next useful capability is many local documents, while proving
the system selects and cites a source WITHOUT trusting the corpus. The load-bearing property is the trust
boundary over the WHOLE corpus: the reading receipt's `structure_hash` (carried in the trace as
`reading_structure_hash`) binds every document's title, spans, and sections, so a mutation of ANY document —
including a non-grounding "side" document — re-derives a different trace and is refused. A side document cannot
silently pass.

**Boundary recorded.** The eight-line boundary is recorded verbatim (in `CORPUS_BOUNDARY_LINES`, the gate, and
the a.md capability section): *The corpus flow reads local documents. It does not trust local documents. Source
selection is verified and replayable. Verification comes before tracing. Nothing executes. Nothing becomes
evidence. Nothing promotes. Nothing trains.* Re-derive-never-trust holds (`verify_corpus_bundle` /
`verify_corpus_trace_json` re-derive and byte-compare; `CognitiveTrace` and `CorpusSource` are `Serialize` but
NOT `Deserialize`; a tampered corpus, source, trace, report, questions, or manifest is refused —
`BundleMismatch` / `CorpusTraceMismatch`). 12 new tests → 112 unit total, fmt + clippy clean; the
`release_check.sh` CORPUS-0 block pins the surface, the 12 test names, the unit count (100→112), the eight
boundary lines, the shell path-validation, and a binary smoke proving the flow end-to-end (boundary from the
trace's own bytes, the source attribution, the directory filter excluding hidden/non-`.txt`/symlink, and every
tamper / empty / unsafe-path refused). Verified by a green, byte-silent gate, four live sabotage probes
(restored byte-identical via `cp`+`md5`; one caught SOLELY by the binary smoke), and an independent read-only
adversarial panel (four Explore lenses, refute-by-default) that returned zero real findings, fully dry. Purely
additive: only `crates/cognitive-demo/src/{lib.rs,main.rs}` + the gate block; the `reading-track-v0.1`
(`f6fa55a`), `hypothesis-track-v0.1` (`bb20acf`), `integration-demo-v0.1` (`95b586d`),
`multi-trace-validation-v0.1` (`460be0c`), `operator-controls-v0.1` (`34b4f47`), and `document-flow-v0.1`
(`0cc7399`) tags are unmoved; P12 stays `training_justified=false`, P13–P15 closed. Recorded in
[a.md](../a.md) (CORPUS-0 capability section). Local only — no remote push.

## DD-2026-06-20-K — Freeze the document flow milestone (DOCFLOW-0 → DOCFLOW-2) as document-flow-v0.1

**Decision.** Freeze the DOCFLOW-0 → DOCFLOW-2 local-document-flow arc as the named, auditable tag
`document-flow-v0.1`. A new `DOCUMENT_FLOW_MILESTONE.md` records the freeze: the DOCFLOW-0 (`c9bd1e5`,
capability), DOCFLOW-1 (`b288196`, operator guard), and DOCFLOW-2 (`4a04759`, input-integrity scenarios)
commit lineage; the frozen base `operator-controls-v0.1` @ `34b4f47` plus the five deeper frozen milestones;
the demonstrated capability; the read-not-trust boundary; the P12 verdict; and the honest residuals. A
documentation freeze only — no code-crate edit (the `cognitive-demo` tree is byte-identical to `4a04759`),
no model, no training.

**Why.** The document flow is now a complete mini-arc — operator-supplied capability, operator guard, and
input-integrity scenario coverage — and should be frozen as a recoverable checkpoint before any further
document behavior is added, exactly as the reading, hypothesis, integration, multi-trace, and
operator-controls arcs were each frozen before the next layer.

**Boundary recorded.** The milestone records the nine-line boundary verbatim: *The document flow reads local
input. It does not trust local input. Document scenarios vary the input. They do not vary the authority.
Verification comes before tracing. Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing
trains.* The freeze edits no frozen crate source (`git diff 460be0c..4a04759` over the reading/hypothesis/
train-gate crates is empty); P12 stays `training_justified=false`, P13–P15 closed. A `release_check.sh`
DOCFLOW-3 lock pins the milestone record (existence, FROZEN, the tag name, the DOCFLOW-0..DOCFLOW-2 hashes,
the frozen-base tags and commits, the document-flow surfaces by name, and the nine boundary lines) and guards
against any milestone that falsely claims training has opened; the lock stays git-free and does NOT require
the tag to exist. The tag is created only after a clean tree and a green, byte-silent gate. Verified by a
green gate, live sabotage of the lock (restored byte-identical via `cp`+`md5`), and an independent read-only
adversarial panel. Recorded in [DOCUMENT_FLOW_MILESTONE.md](../DOCUMENT_FLOW_MILESTONE.md). Local only — no
remote push.

## DD-2026-06-20-J — Document flow scenario pack / input-integrity matrix (DOCFLOW-2)

**Decision.** Extend `crates/cognitive-demo` with a document-flow scenario pack and input-integrity matrix:
`doc-scenarios`, `doc-scenario-pack`, `doc-scenario-verify`, and `doc-scenario-matrix` run a finite,
enum-backed set of nine VALID and INVALID document inputs — clean, modified, empty, absolute path, `..`
traversal, symlink escape, and tampered trace/report/manifest — each OBSERVED by running the REAL DOCFLOW-0
check or verifier and recording the outcome (verified vs refused + typed reason). The containment decision is
extracted into the shared pure `resolved_path_within`, which the shell's `read_local_input` now calls (single
source of truth for the symlink-escape boundary). No frozen crate source, no new dependency.

**Why.** DOCFLOW-0 proved one clean local-document path and DOCFLOW-1 pinned the operator path; the next
useful, boundary-preserving step is to prove the flow holds across the space of valid and invalid inputs —
that local input is verified, path-safe, tamper-sensitive, and still non-authoritative — so an operator can
see, deterministically, that every bad input fails closed. Each scenario PROVES rather than asserts: it runs
the real verifier/check and records the observed `Result`, and the clean-verifies + variations-refused pairing
makes the verifier demonstrably discriminating.

**Boundary recorded.** The pack and matrix record the eight-line boundary verbatim: *Document scenarios vary
the input. They do not vary the authority. Local text is read, not trusted. Verification comes before tracing.
Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.* Every scenario keeps the four
boundary cells (no_execution/no_evidence/no_promotion/no_training) true; the matrix coverage (verified 1,
refused 8, 36/36 cells, all_expectations_met, all_boundaries_hold) is derived from the observed entries, not
hardcoded. Re-derive-not-trust holds: `verify_doc_scenario_pack` re-derives both files and byte-compares;
`doc-scenario-matrix` verifies the pack before emitting; the new structs are `Serialize`-only. Verified by a
green, byte-silent `release_check.sh` (unit-count pin 90→100, ten test-name pins, eight boundary lines, and a
binary smoke proving the coverage from the matrix's own bytes, refusing a tampered pack via both verify and
matrix, and refusing absolute / `..` / **symlink-escape** inputs END-TO-END through the real binary); five
live sabotage probes (test-name pin, boundary pin, runtime coverage, verify-trusts-files, and a clippy-clean
main.rs wiring regression caught solely by the binary smoke), each restored byte-identical via `cp`+`md5`;
and an independent read-only adversarial panel (four refute-by-default lenses). The panel raised one HIGH
finding — the symlink-escape scenario observed only the pure containment decision while the gate had no
end-to-end symlink test — folded by adding the end-to-end input-safety smoke, sabotage-verified (a clippy-clean
vacuous containment with all 100 lib tests green is caught by the new smoke), and re-checked to a dry round. No
code outside `crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate changed; P12 stays
`training_justified=false`, P13–P15 closed, and the six milestone tags are unmoved. Recorded in [a.md](../a.md).
Local only — no remote push.

## DD-2026-06-20-I — Document flow operator guard / manual + smoke integration (DOCFLOW-1)

**Decision.** Extend the operator-facing guard layer to cover the DOCFLOW-0 commands without adding any
behavior: `OPERATOR_MANUAL.md` now documents `doc-trace` / `doc-report` / `doc-bundle` / `doc-bundle-verify`
(new §11, with real flags and outputs) and states the document is *read but not trusted*; the operator smoke
`scripts/operator_smoke.sh` now runs the whole doc flow end-to-end against a LOCAL sample document, the same
way it already exercises the canonical demo commands. No `crates/` source changes.

**Why.** DOCFLOW-0 added an operator-facing capability. Before adding more document behavior, the manual and
the smoke guard must cover the new commands so the doc flow cannot become undocumented or drift from the
binary — the same drift discipline OPS-0/OPS-1 established for the original demo surface. This is a
documentation + drift-guard sprint, not a capability sprint, so `a.md` is left unchanged.

**Boundary recorded.** The manual and smoke record the six-line document-operator-path boundary verbatim:
*The document operator path explains and verifies local-document tracing. It does not trust local input. It
does not create authority. It does not execute. It does not promote. It does not train.* The smoke creates a
temp local document under the gitignored `target/` directory (referenced by a relative path, since the doc
commands only read paths inside the working directory) and removes it on exit; it runs `doc-trace --input
--out`, `doc-report`, `doc-bundle`, and `doc-bundle-verify`, proves the trace started from the document's OWN
verified read (the reading answer is the document's first span), and proves re-derive is load-bearing over
operator input — a tampered document, each tampered bundle file (trace / report / questions / manifest), and a
tampered standalone trace are all refused. The smoke is RUN by the OPS-1 lock (a doc-flow drift makes it fail
closed and aborts the gate); a new DOCFLOW-1 gate block additionally pins the doc commands, the *read but not
trusted* statement, and the six boundary lines in both the manual and the smoke, so the coverage cannot be
silently removed. Verified by a green, byte-silent `release_check.sh`; live sabotage of the new pins
(manual boundary drift, per-file tamper coverage, local doc-dir path, and the runtime read-operator-text
check each caught, restored byte-identical via `cp`+`md5`); and an independent read-only adversarial panel
(4 refute-by-default lenses). The panel raised one low finding — a gate pin that checked the smoke's doc-dir
setup rather than the §10 doc-flow run — folded by adding a §10-unique load-bearing pin (the no-affirmative-
authority assertion), sabotage-verified, and re-checked to a dry round. No code crate behavior changes, P12
stays `training_justified=false`, P13–P15 closed, and the six milestone tags are unmoved. Recorded in
[OPERATOR_MANUAL.md](../OPERATOR_MANUAL.md) and [scripts/operator_smoke.sh](../scripts/operator_smoke.sh).
Local only — no remote push.

## DD-2026-06-20-H — Operator-supplied document trace / read-only input demo (DOCFLOW-0)

**Decision.** Extend `crates/cognitive-demo` with an operator-supplied document flow: `doc-trace`,
`doc-report`, `doc-bundle`, and `doc-bundle-verify` run the SAME end-to-end pipeline from a LOCAL
operator-supplied text document instead of the fixed canonical corpus, producing the same verified-to-refused
trace and no-authority outputs. To verify before tracing against an arbitrary document, the crate takes a new
DIRECT dependency on the already-frozen `reading-substrate` and uses the frozen `corpus_from_documents` to read
the document's own first span, then grounds a plan against it and starts from a frozen-VERIFIED read0 receipt.
No frozen crate SOURCE is edited.

**Why.** The demos so far were controlled and canonical. The next useful, boundary-preserving capability is to
let an operator point the system at a small local text file and get the same trace/report/bundle — making the
prototype more understandable and useful — without opening execution, evidence promotion, or training. Reusing
the frozen reader (rather than re-implementing the sentence splitter) keeps the verifier the single source of
grounding truth, so the document is read but never trusted: it becomes a verified read of the operator's own
text or nothing at all (`VerifierRejected`).

**Boundary recorded.** The flow records the seven-line boundary verbatim: *The document flow reads local input.
It does not trust local input. It verifies before tracing. It does not create authority. It does not execute. It
does not promote. It does not train.* The hypothesis cites the document receipt by hash; the probe is queued
never executed; the observation is quarantined; promotion is refused; P12 stays `training_justified=false`. The
re-derive-not-trust discipline holds over operator input: `doc-bundle-verify`/`doc-report` re-derive from the
SAME document and refuse a tampered document, trace, report, questions, or manifest; `CognitiveTrace` stays
`Serialize`-only. Input safety is enforced in the shell (the only place `std::fs` lives): a pure
`check_local_input_path` rejects absolute / `..` / `~` / empty paths, and `read_local_input` canonicalizes and
requires the resolved path to stay inside the working directory (so a symlink cannot escape) and be a regular
file. Verified by a green byte-silent `release_check.sh` (the DOCFLOW-0 gate block pins the surface, the 10
first-tests, the unit count 80→90, the seven boundary lines, the shell path-validation, and a binary smoke that
proves the boundary from the trace's own output and refuses a tampered document/trace/bundle and an absolute /
`..` / symlink-escape path); four live sabotage probes (pure-check accepts absolute → unit test; verify trusts
files → unit tests; boundary drift → source/smoke pin, unit GREEN; shell escape guard removed → symlink smoke,
unit GREEN + clippy clean — each restored byte-identical via `cp`+`md5`, never `git checkout`); and an
independent read-only adversarial panel (4 lenses, refute-by-default) run to a dry round. Additive within the
integration layer: only `crates/cognitive-demo/src/{lib.rs,main.rs}`, its `Cargo.toml` (one new direct dep on
frozen `reading-substrate`), `Cargo.lock`, the gate block, `a.md`, and this entry change. The five milestone
tags (`reading-track-v0.1` @ `f6fa55a`, `hypothesis-track-v0.1` @ `bb20acf`, `integration-demo-v0.1` @
`95b586d`, `multi-trace-validation-v0.1` @ `460be0c`, `operator-controls-v0.1` @ `34b4f47`) are unmoved, P12
`training_justified=false`, and P13–P15 closed. Recorded in [a.md](../a.md). Local only — no remote push.

## DD-2026-06-20-G — Freeze the operator-controls milestone (OPS-3)

**Decision.** Freeze the OPS-0 → OPS-2 operator-controls arc — the operator manual (`OPERATOR_MANUAL.md`),
the executable smoke / manual drift guard (`scripts/operator_smoke.sh`), and the local release snapshot
(`OPERATOR_RELEASE_SNAPSHOT.md`) — as a named, auditable milestone `operator-controls-v0.1`. Add
`OPERATOR_CONTROLS_MILESTONE.md` (the freeze record), this charter entry, and an OPS-3 milestone lock in
`release_check.sh`. The tag is created only after a clean tree and a green gate. No code crate is touched.

**Why.** OPS-0 through OPS-2 now form a complete operator-control arc — read it, verify it hasn't drifted,
record its state — so it deserves a single freeze point before any further behavior is added, exactly as the
reading / hypothesis / integration / multi-trace tracks were each frozen (`reading-track-v0.1`,
`hypothesis-track-v0.1`, `integration-demo-v0.1`, `multi-trace-validation-v0.1`). The milestone record pins
the OPS-0..OPS-2 commit lineage and the frozen base so the freeze cannot silently drift.

**Boundary recorded.** The milestone records the six-line boundary verbatim: *The operator controls explain
and verify the prototype. They do not release remotely. They do not create authority. They do not execute.
They do not promote. They do not train.* The `release_check.sh` OPS-3 lock pins, by content inspection, the
milestone's existence, the `FROZEN` status, the tag name, the OPS-0..OPS-2 commit hashes (`7aa17ec`,
`c33dea7`, `0876ba0`, auditable against `git log`), the five frozen base tags and their commits (`bbd1113`,
`f6fa55a`, `bb20acf`, `95b586d`, `460be0c`), the three operator controls by name, the
`training_not_justified` verdict, and the six boundary lines, and guards against any milestone that falsely
claims training has opened. The lock stays git-free and does NOT require the tag to exist (so the pre-tag
gate run passes). Verified by a green byte-silent `release_check.sh`; live sabotage of the OPS-3 lock (drop
an OPS commit hash; drop a frozen base SHA; drift a boundary line; a false training-opened claim — every
probe failed the gate at exit 1 and was restored byte-identical via `cp`+`md5`, never `git checkout`, since
the milestone doc is untracked); and a read-only adversarial panel (4 lenses, refute-by-default) iterated to
a fully-dry round. No code crate is touched (`git diff 460be0c..0876ba0 -- crates/ a.md Cargo.toml` empty),
all five prior milestone tags are unmoved, P12 `training_justified=false`, and P13–P15 closed. Recorded in
[OPERATOR_CONTROLS_MILESTONE.md](../OPERATOR_CONTROLS_MILESTONE.md). Local only — no remote push.

## DD-2026-06-20-F — Add the operator release snapshot / local archive manifest (OPS-2)

**Decision.** Add `OPERATOR_RELEASE_SNAPSHOT.md`, a docs-only local snapshot of the prototype state after
OPS-1: the post-OPS-1 HEAD commit (`c33dea7`), all five frozen milestone tags with their commits, the
recovery commands (`git checkout <tag>` / `git checkout c33dea7`), the two verification commands
(`./scripts/release_check.sh` and `./scripts/operator_smoke.sh` with expected output), what the prototype
can and cannot do, and the P12 training verdict. `release_check.sh` gains an OPS-2 lock that pins the
snapshot's load-bearing content. No code crate is touched; no tag is created.

**Why.** Before adding any new behavior, the frozen state deserves a single local record an operator can
read to know exactly what is frozen, which commit and tags recover it, which commands verify it, and what
the boundaries are — so the snapshot cannot drift into stale fiction. This is a snapshot/reproducibility
sprint, not a release: nothing is pushed, published, or uploaded. The snapshot is honest that its own
commit is a docs-only child of `c33dea7` that changes no capability, and points the operator at the two
re-verification commands that hold regardless of which commit is checked out.

**Boundary recorded.** The snapshot records the six-line boundary verbatim: *The snapshot records the
prototype state. It does not release remotely. It does not create authority. It does not execute. It does
not promote. It does not train.* The `release_check.sh` OPS-2 lock pins, by content inspection, the
snapshot's existence, the HEAD commit it records, the five frozen tag names AND their commits, the recovery
and verify commands, the `training_not_justified` verdict, P13–P15 closed, the verbatim no-remote-release
disclaimer, and the six boundary lines, and guards against any snapshot that falsely claims training has
opened. Verified by a green byte-silent `release_check.sh`; live sabotage of the OPS-2 lock (drop the HEAD
commit; drop a frozen tag SHA; drift the no-remote-release boundary; a false training-opened claim; drop
the `operator_smoke` verify command — every probe failed the gate at exit 1 and was restored byte-identical
via `cp`+`md5`, never `git checkout`, since the snapshot is untracked); and a read-only adversarial panel
(4 lenses, refute-by-default) iterated to a fully-dry round. No code crate is touched, all five milestone
tags (`cognitive-os-governance-v0.1` @ `bbd1113`, `reading-track-v0.1` @ `f6fa55a`, `hypothesis-track-v0.1`
@ `bb20acf`, `integration-demo-v0.1` @ `95b586d`, `multi-trace-validation-v0.1` @ `460be0c`) are unmoved,
P12 `training_justified=false`, and P13–P15 closed. Recorded in
[OPERATOR_RELEASE_SNAPSHOT.md](../OPERATOR_RELEASE_SNAPSHOT.md). Local only — no remote push.

## DD-2026-06-20-E — Add the operator smoke script / manual drift guard (OPS-1)

**Decision.** Add `scripts/operator_smoke.sh`, a deterministic operator smoke that runs the whole
documented operator path end-to-end against the built `cognitive-demo` binary (`trace --out`, `report`,
`replay`, `questions`, `ask`, `bundle`/`bundle-verify`, `scenario-pack`/`scenario-verify`,
`scenario-matrix`/`scenario-matrix-report`/`scenario-matrix-verify`, `failure-pack`/`failure-verify`) in a
throwaway temp dir, and fails closed if any documented command, boundary line, or verify step has drifted
from `OPERATOR_MANUAL.md`. `release_check.sh` runs the smoke and pins its load-bearing properties by source
inspection; the manual gains a short self-check reference. No code crate is touched.

**Why.** OPS-0 documented the operator commands, but nothing kept the manual honest as the binary evolves.
The smoke makes the documented operator flow a *checked* artifact: every command actually runs, every
generated artifact (trace, bundle, scenario pack, matrix, failure pack) is re-derived byte-identically
through the binary's own verify subcommands (`replay`, `bundle-verify`, `scenario-verify`,
`scenario-matrix-verify`, `failure-verify`) and never trusted from its bytes, a tampered artifact is still
refused (so the re-derive is load-bearing, not cosmetic), and the boundary lines the manual leads an
operator to expect are still emitted verbatim by the binary AND recorded verbatim in the manual. Manual
drift now breaks the gate.

**Boundary recorded.** The smoke records the five-line boundary verbatim: *The smoke test verifies the
operator path. It does not create authority. It does not execute. It does not promote. It does not train.*
It writes the trace with `--out` (never a shell redirect, which the re-derive correctly refuses), writes
only under a temp dir removed on exit (no repo debris), and is fail-closed (`set -e`; failures abort and
are never swallowed). The `release_check.sh` OPS-1 lock RUNS the smoke (requiring its completion sentinel,
so a short-circuited / vacuous smoke that runs nothing is caught even though it exits 0) and pins, by
source inspection, that the smoke uses `--out` (and never `trace >`), keeps the `mktemp`+`trap` cleanup,
runs every documented command, re-derives through the verify subcommands, proves tamper is refused, embeds
the binary and manual boundary lines, records the five-line boundary verbatim, and makes no false training
claim; on smoke failure the reason is surfaced to the gate's stderr while the green path stays byte-silent.
Verified by a green byte-silent `release_check.sh`; live sabotage of the OPS-1 lock (a vacuous early-`exit
0` smoke caught by the completion sentinel; a `trace >` redirect caught by the no-redirect pin; a dropped
verify caught by the command pin; a gutted matrix-report content check and a manual boundary drift each
caught at smoke runtime — every probe restored byte-identical via `cp`+`md5`, never `git checkout`); and a
read-only adversarial panel (4 lenses, refute-by-default) iterated to a fully-dry round — two findings were
folded (the gate suppressed the smoke's failure stderr → now surfaced on failure; `scenario-matrix-report`
wrote an unvalidated file → now content-validated against the 16/16 coverage proof with no leftover file),
then a clean 4/4-lens round with zero real findings. No code crate is touched, all five milestone tags
(`cognitive-os-governance-v0.1`, `reading-track-v0.1` @ `f6fa55a`, `hypothesis-track-v0.1` @ `bb20acf`,
`integration-demo-v0.1` @ `95b586d`, `multi-trace-validation-v0.1` @ `460be0c`) are unmoved, P12
`training_justified=false`, and P13–P15 closed. Recorded in
[OPERATOR_MANUAL.md](../OPERATOR_MANUAL.md) (the self-check section) and the smoke script itself. Local
only — no remote push.

## DD-2026-06-20-D — Add the operator manual / prototype capability guide (OPS-0)

**Decision.** Add a plain operator manual `OPERATOR_MANUAL.md` documenting the frozen prototype: what it is
and is not, the five frozen milestone tags with their recovery (`git checkout <tag>`) and verify
(`./scripts/release_check.sh`) commands, the exact `cognitive-demo` commands to reproduce every demo
(trace / report / replay / questions / bundle / scenario-pack / scenario-matrix / failure-pack — with the
real flags and the eight audit-question slugs), the authority boundaries that stay closed, and the P12
training verdict. A comprehension and reproducibility sprint — it adds no behavior and edits no code crate
(a.md untouched).

**Why.** Before adding more machinery, the prototype needs a single plain-language guide an operator can
follow to run each demo, understand what each output means, and confirm what the system explicitly cannot
do. Every command in the manual was run against the built binary and its output captured, so the guide is
real, not illustrative. One reproducibility detail is documented because it bites: a replayable trace file
must be written with `trace --out FILE` (exact bytes), not a shell redirect (`trace > FILE` appends a
newline and is correctly refused by the re-derive byte-compare).

**Boundary recorded.** The manual records the six-line boundary verbatim: *The manual explains the
prototype. It does not expand the prototype. It does not create authority. It does not execute. It does not
promote. It does not train.* It states P12 `training_justified=false` (`training_not_justified`) and that
P13–P15 stay closed, and it makes no claim of model cognition, training, probe execution, or evidence
promotion (the forged-authority cases are documented as forged-and-rejected, never as real states). The
`release_check.sh` OPS-0 manual lock pins the manual's existence, the five frozen tag names it must list,
the documented command surface (every subcommand by name, plus the recovery and verify commands), a real
audit-question slug, the training verdict, and the six boundary lines verbatim, and guards against any
manual that falsely claims training has opened — so the manual cannot silently drift. Verified by a green
byte-silent `release_check.sh`, live sabotage of the manual lock (each restored byte-identical), and a
read-only adversarial panel. No code crate is touched, all five milestone tags
(`cognitive-os-governance-v0.1`, `reading-track-v0.1` @ `f6fa55a`, `hypothesis-track-v0.1` @ `bb20acf`,
`integration-demo-v0.1` @ `95b586d`, `multi-trace-validation-v0.1` @ `460be0c`) are unmoved, P12
`training_justified=false`, and P13–P15 closed. Recorded in full in [OPERATOR_MANUAL.md](../OPERATOR_MANUAL.md).
Local only — no remote push.

## DD-2026-06-20-C — Freeze the multi-trace validation track (MTRACE-0 → MTRACE-2) as multi-trace-validation-v0.1

**Decision.** Freeze the MTRACE-0 → MTRACE-2 multi-trace validation arc as a named, auditable milestone
`multi-trace-validation-v0.1`, recorded in a new freeze doc `MULTI_TRACE_VALIDATION_MILESTONE.md` and locked by a
milestone block in `scripts/release_check.sh`. Documentation freeze only — it adds no behavior and edits no code
crate (a.md is intentionally untouched; it already carries the MTRACE-0/1/2 checklist and detail).

**Why.** MTRACE-0 (scenario pack), MTRACE-1 (coverage matrix), and MTRACE-2 (failure-injection pack) now form a
complete validation arc over the frozen `integration-demo-v0.1` canonical trace: the prototype can vary the path
without varying the authority, summarize the coverage as a re-derived matrix, and prove the bad paths fail closed
under forged authority. Per the build→prove cadence, the arc is frozen before more behavior is added — the same
discipline that produced `reading-track-v0.1`, `hypothesis-track-v0.1`, and `integration-demo-v0.1`.

**Boundary recorded.** The milestone doc pins the commit lineage (MTRACE-0 `aee733f`, MTRACE-1 `91189f2`, MTRACE-2
`be6909f`), references the frozen base (`integration-demo-v0.1` @ `95b586d`) and the two deeper frozen tracks
(`reading-track-v0.1` @ `f6fa55a`, `hypothesis-track-v0.1` @ `bb20acf`), states the demonstrated validation
capability, and records the scenario/matrix/failure boundary verbatim: *Scenarios vary the path. They do not vary
the authority. The matrix summarizes coverage. Failure cases attack the boundary. Forged authority is rejected.
Nothing executes. Nothing becomes evidence. Nothing promotes. Nothing trains.* The arc-wide discipline is RE-DERIVE,
NEVER TRUST: every operator surface that accepts a file (scenario-verify, scenario-matrix/-report/-verify,
failure-verify) verifies by re-deriving the canonical artifact and byte-comparing — no record added in this arc
derives `Deserialize` — so off-wire tampering can never be laundered into authority. The honest, precise statement of
what stayed frozen: the MTRACE arc additively extended only `crates/cognitive-demo` (`lib.rs` + `main.rs`, no new
dependency, no `Cargo.toml` change, no new file), the `integration-demo-v0.1` tag still points at `95b586d`, and the
frozen canonical `demo()` trace and bundle are byte-for-byte identical after every MTRACE sprint (gate-enforced by the
`happy_boundary_scenario_equals_canonical_demo` pin and the frozen `hypothesis_id` freeze-pin). The milestone makes no
false claim: it records P12 `training_justified=false` (`training_not_justified`), and the arc executes no probe,
promotes nothing, mutates no memory, and moves no training verdict; P13–P15 stay closed. The `release_check.sh`
milestone lock pins the freeze doc's existence, the three MTRACE commit hashes (auditable against `git log`), the
frozen-base references, the nine boundary lines verbatim, and the `training_not_justified` verdict, and guards against
a false `training_justified = true` claim, so the freeze cannot silently drift; the gate stays git-free and does not
require the tag to exist. Verified by a green byte-silent `release_check.sh`, live sabotage probes of the milestone
lock (each restored byte-identical), and a read-only adversarial panel. The tag `multi-trace-validation-v0.1` is
created only after a clean tree and a green gate, on the freeze commit. No frozen crate source outside
`crates/cognitive-demo` is touched, all three base tags are unmoved, P12 `training_justified=false`, and P13–P15
closed. Recorded in full in [MULTI_TRACE_VALIDATION_MILESTONE.md](../MULTI_TRACE_VALIDATION_MILESTONE.md). Local only
— no remote push.

## DD-2026-06-20-B — Add the scenario failure-injection / boundary-regression pack (MTRACE-2)

**Decision.** Extend `crates/cognitive-demo` (the `cognitive-demo` binary) with a finite, enum-backed set of
NEGATIVE scenarios that prove the bad paths cannot smuggle authority: `failure-cases` lists the seven cases,
`failure-pack --out DIR` writes the rejection record (`failure-pack.json`) and its rendered report
(`failure-report.txt`), and `failure-verify --path DIR` re-derives the whole pack and refuses any tamper. Where
MTRACE-0/1 prove the good paths preserve the boundary, MTRACE-2 proves invalid variations FAIL CLOSED. Doctrine:
*Failure cases attack the boundary. They do not weaken it. Forged authority is rejected. Nothing executes. Nothing
becomes evidence. Nothing promotes. Nothing trains.*

**Why.** Coverage of valid paths (MTRACE-1) is necessary but not sufficient; the boundary's value is that forged
authority is REJECTED. Each of the seven `FailureCase` variants deterministically forges one forbidden authority
claim onto a canonical artifact and is refused by the EXISTING re-derive-and-byte-compare verifier — no new
verification logic, only a curated regression suite of attacks: `forged-execution`/`forged-evidence`/
`forged-promotion`/`forged-training` (the trace) → `verify_trace_json`/`TraceMismatch`; `forged-review` (a rejected
scenario review forged to approved) → `verify_scenario_bundle`/`BundleMismatch`; `forged-report` (the report forged
to narrate execution/evidence) → `verify_bundle`/`BundleMismatch`; `forged-matrix` (a coverage cell forged to hide
a failed boundary) → `verify_scenario_matrix`/`MatrixMismatch`. The pack PROVES rather than asserts: `run_failure_case`
runs the real verifier on a forged COPY and records `forgery_applied`, `injects_forbidden` (the case's specific
forbidden-authority token was injected, so a benign byte-change cannot masquerade as a forbidden-authority forgery),
`rejected` (observed from the verifier, never hardcoded), and the exact typed `rejection_reason` — a structural
re-derive byte-compare refusal, not a prose grep. `FailurePack`/`FailureRejection`/`FailureSummary` derive
`Serialize` but NOT `Deserialize`, so a doctored pack (e.g. flipping a `rejected` to false to claim a forgery
passed) is refused, never parsed back into authority; the forged bytes are never persisted (only the prose
rejection record is), so neither emitted file carries affirmative authority. Building the pack leaves the frozen
canonical trace byte-identical (happy-boundary still == `demo()`), the MTRACE-0 pack and MTRACE-1 matrix unchanged,
and P12 `training_justified=false`. A self-found vacuity hole (a benign change would also be byte-rejected) was
folded before sabotage by adding `injects_forbidden`. `release_check.sh` gates it (surface signals, the re-derive
pin, the anti-vacuity pins, twelve MTRACE-2 test-name pins, the unit-count pin raised 68→80, and a binary smoke
proving every case is forged + injects-forbidden + rejected, no authority leaks into either pack file, the report's
exact typed reasons and seven boundary lines verbatim, determinism, and refusal of a doctored/missing pack) and
stays green + byte-silent. Verified by three live sabotage probes (verify-trusts-the-pack; a benign forgery caught
solely by the new `injects_forbidden` check; a boundary-line drift that kept the suite green but failed the gate
via the verbatim boundary loop — each restored byte-identical) and a read-only adversarial panel (four Explore
lenses, 0 real findings, fully dry on the first round, no debris). Purely additive: only
`crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate block; no frozen crate source touched, the
`reading-track-v0.1` (`f6fa55a`), `hypothesis-track-v0.1` (`bb20acf`), and `integration-demo-v0.1` (`95b586d`) tags
unmoved, P12 `training_justified=false`, and P13–P15 closed. Recorded in full in [a.md](../a.md) (the MTRACE-2
checklist entry and the "Scenario Failure Injection / Boundary Regression Pack (MTRACE-2)" detail section). Local
only — no remote push.

## DD-2026-06-20-A — Add the scenario boundary-coverage matrix (MTRACE-1)

**Decision.** Extend `crates/cognitive-demo` (the `cognitive-demo` binary) with a deterministic boundary-coverage
matrix over the MTRACE-0 scenario pack: `scenario-matrix --pack DIR --out matrix.json` emits the canonical
coverage matrix, `scenario-matrix-report --matrix matrix.json --out matrix.txt` renders a plain report, and
`scenario-matrix-verify --pack DIR --matrix matrix.json` re-derives and checks both. It adds NO capability and NO
model behavior, no new dependency, no new file, no Cargo.toml change, and edits no frozen crate.

**Why.** MTRACE-0 creates scenario bundles; the next useful step was to summarize them into a machine-checkable
coverage matrix so an operator can see, at a glance, which authority boundaries were proven across which paths.
The doctrine is sharpened for this surface: *The matrix summarizes coverage. It does not create authority. It
does not execute. It does not promote. It does not train.*

**Boundary recorded.** The matrix has one row per scenario (its review/probe/intent/observation/promotion status
+ the `training_not_justified` verdict) and, for every scenario, the four boundary cells `no_execution`/
`no_evidence`/`no_promotion`/`no_training` (all true), plus a coverage summary (16/16 cells proven,
`all_boundaries_hold=true`, and the distinct review/intent/probe statuses proving the variation is real). It
PROVES rather than asserts: every cell is the trace's REAL verdict (`no_execution=trace.nothing_executed()`, etc.)
and every status row matches the scenario's trace. The load-bearing discipline is re-derive-never-trust: the
matrix is purely re-derived from `Scenario::ALL` (it never reads the pack files for its content);
`scenario-matrix --pack` first VERIFIES the pack (re-deriving every scenario bundle + the pack manifest via the
new pure `verify_scenario_pack`) and refuses a tampered pack before emitting; `verify_scenario_matrix` and
`scenario_matrix_report` re-derive the canonical matrix and byte-compare the provided JSON
(`TraceError::MatrixMismatch`), and the report renders from the re-derived canonical struct, never the provided
file. `ScenarioMatrix`/`MatrixRow`/`MatrixCoverage` derive `Serialize` but NOT `Deserialize`, so a provided matrix
is never parsed into authority — a tampered matrix OR a tampered pack is refused by verify, report, and emit. No
matrix/report field shows an affirmative executed/promoted/granted/recorded status, a true grant, or a
`training_justified` verdict; the frozen canonical trace is unchanged (happy-boundary still == `demo()`).
`release_check.sh` gates it (surface signals, the re-derive pin, twelve MTRACE-1 test-name pins, the unit-count
pin raised 56→68, and a binary smoke proving the matrix records all scenarios + status fields + boundary cells +
coverage, determinism, the report boundary summary, and refusal of a tampered matrix by verify AND report + a
tampered pack by emit AND verify) and stays green + byte-silent. Verified by three live sabotage probes (a
verify-trusts-the-matrix, a pack-verify-skips-bundles, and a boundary-line drift that kept the suite green but
failed the gate via the verbatim report-boundary loop — each restored byte-identical) and a read-only adversarial
panel (four Explore lenses, 0 real findings, no debris; the first attempt was abandoned mid-run by a session-usage
limit — treated as absence of verification, not a pass — and re-run to a genuine dry round before close). Purely
additive: only `crates/cognitive-demo/src/{lib.rs,main.rs}` and the gate block; no frozen crate source touched,
the `reading-track-v0.1` (`f6fa55a`), `hypothesis-track-v0.1` (`bb20acf`), and `integration-demo-v0.1`
(`95b586d`) tags unmoved, P12 `training_justified=false`, and P13–P15 closed. Recorded in full in [a.md](../a.md)
(the MTRACE-1 checklist entry and the "Scenario Matrix / Boundary Coverage Report (MTRACE-1)" detail section).
Local only — no remote push.

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
