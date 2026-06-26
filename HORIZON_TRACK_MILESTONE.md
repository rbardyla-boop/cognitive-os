# Horizon Track Milestone: HORIZON-0 → HORIZON-2 (FROZEN for horizon-track-v0.1)

> Status: **FROZEN** as of `horizon-track-v0.1`. This document freezes the HORIZON-0 through HORIZON-2
> staged-interaction arc as a named, auditable milestone before model-readiness work (corpus harvest,
> verifier-as-scorer, the recurring-failure detector, and the P11 model-need eval) opens. It is the single
> milestone-freeze record for the horizon layer; the per-sprint decisions live in `docs/PROJECT_CHARTER.md`
> (`DD-2026-06-22-O`, `DD-2026-06-22-P`, `DD-2026-06-22-Q`). This file freezes the arc, the commit lineage,
> the prior frozen substrate base, the demonstrated capability, the cannot-bypass boundary, the
> structurally-closed training eligibility, the verification discipline, the training-gate verdict, the
> honest residuals, and the frozen-status declaration. It does not restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            horizon-track-v0.1
points at      the HORIZON-3 freeze commit (this document + its gate lock)
freezes        the HORIZON-0..HORIZON-2 staged-interaction arc (head d86799e)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
recover        git checkout horizon-track-v0.1
training gate  training_not_justified (P12 training_justified=false) — weights forbidden
prior base     data-curation-v0.1 @ b47665b (the prior frozen substrate milestone: the dataset-curation arc)
frozen base    dream-export-v0.1 @ 5238fe8 (the frozen dream-provenance arc),
               corpus-flow-v0.1 @ b8577fe (the frozen multi-document corpus base),
               document-flow-v0.1 @ 0cc7399 (the frozen local-document base)
deeper base    operator-controls-v0.1 @ 34b4f47, multi-trace-validation-v0.1 @ 460be0c,
               integration-demo-v0.1 @ 95b586d, hypothesis-track-v0.1 @ bb20acf,
               reading-track-v0.1 @ f6fa55a, cognitive-os-governance-v0.1 @ bbd1113 (all frozen)
```

The horizon arc is the staged-interaction layer of the **substrate-not-agent** track: long-horizon capability
comes from interaction structure and trustworthy trajectories, not bigger prompts, so before any
intelligence / agency path may run, the substrate must demonstrably hold its gates across a chain of turns —
and fail closed when a longer chain attempts a bypass. HORIZON-0 adds a deterministic harness in
`crates/cognitive-demo/src/horizon.rs` that composes the EXISTING verified-read, DATA-0 curation, dream-packet,
and dream-export flows into six bounded horizons `H0..H5` and records a `HorizonTrace` per level; HORIZON-1
documents and smoke-tests the operator path over the REAL harness; and HORIZON-2 proves the gate on the
NEGATIVE side via a fixed 10-scenario failure matrix driven by the real verifier/curator. The crucial property
is not "the system can run longer chains" — it is that **depth never unlocks a bypass**: a deeper horizon still
passes through curation, grounding, replay, authority, and the closed training gate, and a forged or
over-budget horizon is REFUSED. The arc grows only `cognitive-demo` (the integration crate); it edits no
frozen crate source, and it opens no training.

## 1. What is frozen — the commit lineage

Three commits form the arc. The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| HORIZON-0 | Staged Interaction Harness: `crates/cognitive-demo/src/horizon.rs` composes the existing verified-read, DATA-0 curation, dream-packet, and dream-export flows into six bounded horizons `H0..H5`, recording a `HorizonTrace` per level. Each `HorizonLevel` fixes `max_turns`, `allowed_modules` (a per-turn whitelist), and `forbidden_escalations` (the bypasses the level must refuse). Every turn is a REAL call into a frozen flow; each `HorizonStep` records that flow's receipt — `input_hash` / `output_hash` (FNV u64), `authority_state` (∈ `none` / `dream_only` / `hypothesis_only`, never evidence/promoted), `curation_status`, and `replay_status`. The six gate invariants (`curation_never_skipped`, `grounding_never_skipped`, `replay_never_skipped`, `no_promotion_to_evidence`, `training_never_opens`, `forbidden_escalation_refused`) are COMPUTED from the observed receipts, not hard-coded; `reading_train_gate::decide(&[], &[])` is evaluated before AND after every horizon and proven unmoved. The `HorizonTrace` / `HorizonStep` derive `Serialize` but NOT `Deserialize`, with PRIVATE fields read through accessors — re-derived and byte-compared, never trusted from bytes. Adds the one-way `cognitive-demo -> data-curator` dependency H1/H2/H5 need (the curator's own tree stays workspace-isolated). 23 lib unit tests | `db8a776` |
| HORIZON-1 | Horizon Operator Guard / Manual + Smoke Integration: `OPERATOR_MANUAL.md` (§16) documents the bounded-horizon operator path — H0..H5 with their `max_turns` and compositions, that `HorizonTrace` is re-derived + byte-compared (never trusted from off-wire bytes) and is Serialize-not-Deserialize, that longer horizons cannot skip curation / grounding / replay, that dream/hypothesis material cannot become evidence, and that training eligibility remains closed. `scripts/operator_smoke.sh` runs the REAL harness over each level via its named `cognitive-demo` `horizon::tests` (H0..H5 + all-gates-held + training-never-opens), each `--exact` with a `1 passed` non-vacuous assertion. A documentation + drift-guard sprint — NO code-crate change (the HORIZON-0 harness is byte-identical; the cognitive-demo unit-count pin stays 190); the gate already RUNS `operator_smoke.sh`, so a horizon drift fails closed | `b20b2e4` |
| HORIZON-2 | Horizon Boundary Failure Matrix: `crates/cognitive-demo/src/horizon.rs` adds a FIXED, named set of 10 failure scenarios (`horizon_failure_matrix()`). Each constructs a BAD horizon input and runs the REAL machinery over it, recording the OBSERVED refusal: the curation cells run the DATA-0 `curate()` and observe the bad item land in rejected/quarantined (never admitted); the evidence/authority/training cells re-derive a real `run_horizon_json` trace, apply a single textual mutation (guarded `mutated != canonical` so a no-op cannot pass), and observe `verify_horizon_json` refuse it; the overflow cell uses the real `max_turns` ceiling via `within_turn_bound`; the unknown-level cell uses `HorizonLevel::from_slug` returning `None`; the serialized-trace cell tampers a real trace and observes the re-derive verifier refuse it. Each cell records `training_still_closed` (the P12 verdict decided after the attempt). `FailureCell` / `RefusalMechanism` derive `Serialize` but NOT `Deserialize`. Adds `from_slug` + `within_turn_bound` to `HorizonLevel`; 16 lib unit tests; release_check bumps the cognitive-demo unit-count pin 190 → 206 | `d86799e` |

The horizon head frozen here is `d86799e` (HORIZON-2).

The six bounded horizons, named, with their turn bound and module composition (every step's module must
appear in the level's `allowed_modules` whitelist), are:

| Level | max_turns | Composition (in turn order) |
| --- | --- | --- |
| `H0` | 1 | `VerifiedRead` — one verified read |
| `H1` | 2 | `CurateDocument` → `VerifiedRead` — curate a document, then read it |
| `H2` | 2 | `CurateCorpus` → `CorpusRead` — curate a corpus, then a multi-document read |
| `H3` | 2 | `CorpusRead` → `DreamPacket` — a corpus read, then a grounded dream packet |
| `H4` | 3 | `CorpusRead` → `DreamPacket` → `DreamExport` — … then a dream export that stays `hypothesis_only` |
| `H5` | 3 | `CurationMatrix` → `CorpusRead` → `DreamExport` — curation matrix, read, and dream-export matrix |

Every level refuses `skip_curation`, `skip_grounding`, `skip_replay`, `promote_to_evidence`, and
`open_training`; H3/H4/H5 additionally refuse `dream_only_authority_escape`.

The ten HORIZON-2 failure scenarios, named, each with the REAL refusal mechanism the matrix observes, are:
`uncurated_candidate_refused` (CurationQuarantined), `missing_grounding_refused` (CurationRejected),
`missing_replay_refused` (CurationRejected), `dream_to_evidence_refused` (VerifyMismatch),
`hypothesis_to_evidence_refused` (VerifyMismatch), `training_open_refused` (VerifyMismatch),
`authority_escalation_refused` (VerifyMismatch), `max_turns_overflow_refused` (TurnBoundExceeded),
`unknown_horizon_level_refused` (UnknownLevel), and `serialized_trace_replay_refused` (VerifyMismatch).

## 2. The frozen bases

The horizon arc is built AFTER the `data-curation-v0.1` substrate milestone (the prior frozen dataset-curation
arc) and the frozen dream / corpus / document / hypothesis / governance tracks beneath it. The honest, precise
statement: every prior tag still points where it did, and the horizon arc grows ONLY `cognitive-demo` (the
integration crate, already the home of the verified-read / corpus / dream-export flows) plus the one-way
`cognitive-demo -> data-curator` dependency edge; it edits no frozen crate source. `git diff` for every frozen
crate across `b47665b..d86799e` is empty except `crates/cognitive-demo`.

```text
data-curation-v0.1           @ b47665b   (the prior frozen substrate milestone: the dataset-curation arc)
dream-export-v0.1            @ 5238fe8   (the frozen dream-provenance arc: dream material stays HypothesisOnly)
corpus-flow-v0.1             @ b8577fe   (the frozen multi-document corpus base: a verified local corpus read)
document-flow-v0.1           @ 0cc7399   (the frozen local-document base: one operator document, read not trusted)
operator-controls-v0.1        @ 34b4f47   (operator manual / smoke drift guard / release snapshot)
multi-trace-validation-v0.1   @ 460be0c   (scenario pack / coverage matrix / failure injection)
integration-demo-v0.1         @ 95b586d   (the cognitive-demo crate: trace / report / questions / bundle)
hypothesis-track-v0.1         @ bb20acf   (propose → probe → review → intent → observation → promotion-refusal)
reading-track-v0.1            @ f6fa55a   (the read0 verifier + the verified reading receipt)
cognitive-os-governance-v0.1  @ bbd1113   (the v0.1 governance / evidence-contract lineage)
```

`crates/data-curator` remains workspace-isolated: the `cognitive-demo -> data-curator` arrow is one-way (the
curator's own dependency tree carries no `vibe-*`, `reading-*`, `hypothesis-layer`, `cognitive-demo`, or
`dream-engine` edge), so curation cannot acquire authority, mutate memory, or open training even through the
horizon harness. The HORIZON-1 documentation and HORIZON-2 failure matrix changed no prior behavior; the
HORIZON-0 harness source is byte-identical from `db8a776` through `d86799e`.

## 3. What the operator can now do (the demonstrated capability)

The prototype could already read, trace, report, interrogate, package, validate, propose hypothesis-only
novelty, bridge a dream into the hypothesis-only path with preserved provenance (dream-export-v0.1), and gate
candidate data through deterministic curation (data-curation-v0.1). The horizon arc adds the staged-interaction
harness that runs those flows as bounded chains and proves the chains cannot bypass the gates:

1. **Run six bounded horizons `H0..H5` over the REAL frozen flows (HORIZON-0).** Each level composes a longer
   chain (one verified read, up through curation + corpus read + dream-export matrix) under a fixed `max_turns`
   and a per-turn module whitelist, recording a `HorizonTrace` whose every step is a real receipt. The six gate
   invariants — curation / grounding / replay never skipped, no promotion to evidence, training never opens,
   and every forbidden escalation refused — are computed from those receipts, and the train gate is decided
   before and after each horizon and proven unmoved.
2. **Exercise and audit the horizon path from the operator path (HORIZON-1).** The operator manual documents
   H0..H5, their turn bounds and compositions, and the doctrine; the operator smoke drives the REAL
   `run_horizon()` through its named tests and fails closed if the harness drifts from the manual.
3. **Prove the gate on the negative side (HORIZON-2).** A fixed 10-scenario failure matrix runs the real
   curator/verifier over a bad uncurated/ungrounded/replay-less candidate, a real trace mutated to forge
   evidence/authority/training, an over-budget step count, an unknown level, and a tampered serialized trace —
   recording each OBSERVED refusal, and proving `training_still_closed` in every cell.

Every one of these runs through the real harness or fails closed. None is intelligence, none creates evidence,
and none opens training.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint; each is
enforced by the release gate from the artifacts' own bytes.

1. **Depth never unlocks a bypass (whole arc).** A deeper horizon composes a longer chain, but every level
   still passes through the same gates. The invariants `curation_never_skipped`, `grounding_never_skipped`, and
   `replay_never_skipped` are computed from the real receipts, and HORIZON-2 proves that an attempt to skip any
   of them is REFUSED.
2. **No promotion to evidence (whole arc).** Dream material stays `dream_only` and exported dream/hypothesis
   material stays `hypothesis_only`; `no_promotion_to_evidence` holds across every horizon, and the
   `dream_to_evidence_refused` / `hypothesis_to_evidence_refused` failure cells prove a forged promotion is
   refused by the re-derive verifier.
3. **No new authority (whole arc).** A step's `authority_state` is only ever `none`, `dream_only`, or
   `hypothesis_only`; H3/H4/H5 explicitly refuse `dream_only_authority_escape`, and the
   `authority_escalation_refused` cell proves an authority forgery is refused.
4. **Training eligibility cannot open (whole arc).** `reading_train_gate::decide(&[], &[])` is decided before
   and after every horizon and proven equal with `training_justified=false`; the `training_open_refused` cell
   proves a forged "training opened" trace is refused, and `training_still_closed` is recorded in every failure
   cell. Opening training is the job of a future gate that does not exist yet.
5. **Re-derive, never trust the artifacts (whole arc).** `HorizonTrace`, `HorizonStep`, `FailureCell`, and
   `RefusalMechanism` derive `Serialize` but NOT `Deserialize`; `verify_horizon_json` / `verify_horizon_matrix_json`
   re-derive from the primary inputs and byte-compare, never parsing a trace back in and trusting it. The
   `serialized_trace_replay_refused` cell proves a tampered serialized trace is refused.
6. **Bounded turns and known levels (whole arc).** Every level has a hard `max_turns` ceiling
   (`within_turn_bound`), and an unknown level slug resolves to `None` (`from_slug`); the
   `max_turns_overflow_refused` and `unknown_horizon_level_refused` cells prove an over-budget chain and an
   unknown horizon are refused.
7. **A harness, not intelligence; no model in the loop.** The harness composes deterministic frozen flows and
   records receipts. It is not a model, it does no training, and it executes no external action. Any future
   model could only PROPOSE within a bounded horizon; it can never self-promote, mint authority, open training,
   or bypass a gate.

The HORIZON-0 nine-line harness boundary is recorded verbatim (the code-true surface in the `horizon.rs`
const + module banner):

```text
The horizon harness measures bounded interaction depth.
It does not train.
It does not execute external actions.
It does not create truth.
It does not create memory.
It does not promote hypotheses.
It does not grant new authority.
Longer horizons cannot bypass earlier gates.
Training eligibility remains closed.
```

The HORIZON-2 nine-line failure-matrix boundary is recorded verbatim:

```text
The horizon failure matrix mutates bounded traces.
It observes refusals.
It does not create truth.
It does not create memory.
It does not train.
It does not execute external actions.
It does not promote hypotheses.
It does not grant new authority.
Training eligibility remains closed.
```

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is `training_justified=false`).
The horizon arc is orthogonal to P12 and does not move it: every horizon decides `decide(&[], &[])` before and
after and proves it unmoved, and every HORIZON-2 failure cell records `training_still_closed`. Weight training
stays forbidden until the P11 eval proves a stable, recurring model failure that survives fixes to task spec,
schema, prompt, examples, tooling, context, and verifier design — AND the operator separately authorizes it.
P13–P15 (LoRA candidate, shadow mode, promotion gate) stay closed under this freeze. This milestone makes no
claim that training has opened, that the harness is intelligence, that any dream/hypothesis material became
evidence, or that any authority was granted.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0 bytes
stderr). The horizon locks pin, per sprint: the harness module wiring and public entrypoints, the REAL frozen
flow call sites (so a horizon cannot be faked from a hand-written table), the `decide(&[], &[])` before/after
gate, the six computed invariants, all six `H0..H5` levels, the `Serialize`-not-`Deserialize` /
private-fields property of `HorizonTrace` and `HorizonStep`, and the nine-line boundary (HORIZON-0); the
manual's documentation of the operator path and an actual RUN of the operator smoke over the REAL harness
(HORIZON-1); and the failure-matrix entrypoint, the fixed scenario count of 10, the real-exercise calls
(`verify_horizon_json`, `curate`, `mutated != canonical`, `from_slug`, `within_turn_bound`), the ten scenario
names, the five refusal mechanisms, and the nine-line failure-matrix boundary (HORIZON-2). This milestone block
additionally pins the freeze record itself (this document's HORIZON-0..HORIZON-2 commit lineage `db8a776` /
`b20b2e4` / `d86799e`, the prior frozen `data-curation-v0.1` @ `b47665b` substrate base and the deeper frozen
tags + commits, the six H0..H5 levels, the ten failure scenarios, the cannot-bypass boundary, and the
`training_not_justified` verdict), and guards against any milestone that falsely claims training has opened. The
pinned commit hashes are auditable against `git log`; this lock stays git-free and does NOT require the tag to
exist — the tag is created only after a clean tree and a green gate. The acceptance discipline for every sprint
in this arc was: rubric → green byte-silent `release_check` → live sabotage proving the gate catches a
regression (restored byte-identical by `cp`+`md5`, never `git checkout`) → an independent read-only adversarial
verifier panel with a fresh context → any residual folded before close.

## 7. Independent verification

Every sprint HORIZON-0 through HORIZON-2 was closed against read-only adversarial panels (Explore agents,
refute-by-default, scratch confined to a temp dir, each driving the compiled tests or inspecting the artifacts),
run until a fully-dry round with zero real findings. This HORIZON-3 freeze adds no behavior; it is verified by a
green byte-silent gate, live sabotage of the milestone lock (restored byte-identical via `cp`+`md5`, never
`git checkout`), and an independent read-only adversarial panel. Each gate lock across the arc was proven
load-bearing by live sabotage that failed the gate and was restored byte-identical. Every claim in this document
is checkable by running `scripts/release_check.sh` and reading the named commits.

## 8. Honest residuals (NOT closed in horizon-track-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the horizon
layer, not bugs.

1. **The horizon harness is a harness, not intelligence.** It composes deterministic frozen flows under bounded
   turns and records receipts; it proposes nothing of its own and learns nothing. Any future proposer (model or
   otherwise) must run INSIDE a bounded horizon and is subject to the same gates.
2. **The levels and the failure matrix are fixed sets, not exhaustive.** Six horizons and ten failure scenarios
   are enumerated; they are honest about what they cover and are not a proof that no other composition or attack
   shape exists. Every recorded outcome is OBSERVED from the real harness/verifier, so the set cannot drift from
   actual behavior.
3. **Integrity is byte-for-byte re-derivation, not a digest.** The load-bearing tamper check is re-deriving the
   trace / matrix from the primary inputs within one deterministic build and comparing; cross-version
   reproduction and cryptographic digests are not claimed.
4. **No model-need is proven or refuted here.** The horizon arc demonstrates the substrate holds its gates over
   bounded chains; it does NOT run a model-need eval. Whether a learned model is ever justified is the job of
   the later P11 model-need eval and remains entirely open. Training stays closed under P12.
5. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline and the gate locks defend
   against off-wire tampering and accidental regression, both of which the gate provably catches. They do not
   defend against an insider with commit access who authors malicious code AND rewrites the gate in the same
   change — that is the domain of code review and the governance/signing layer.
6. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production runtime,
   and the horizon layer is described as such.
7. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left stray
   debris in the working tree despite their read-only instruction, and have occasionally inverted the
   finding-label semantics; each was caught and reconciled before close. It remains a known operational caveat
   of the panel method. Separately, an unrelated process left Python/CSV edits dirty in the working tree across
   this arc; every horizon commit used explicit pathspecs so that dirt never entered a horizon commit.

## 9. Frozen-status declaration

The HORIZON-0 → HORIZON-2 staged-interaction arc is **FROZEN at `horizon-track-v0.1`**. The cannot-bypass
boundary is the frozen surface:

```text
The horizon track stages bounded interaction depth.
It composes verified reading, curation, replay, dream, and hypothesis flows.
It does not create truth.
It does not create memory.
It does not train.
It does not execute external actions.
It does not promote hypotheses.
It does not grant new authority.
Longer horizons cannot bypass curation, grounding, replay, authority, or training closure.
```

Any change that lets a horizon skip curation, grounding, or replay; that lets dream or hypothesis material
become evidence; that grants a horizon new authority or lets `dream_only` escape; that removes a level's
`max_turns` ceiling or silently coerces an unknown level; that trusts a serialized `HorizonTrace` as authority;
or that reopens training — must pass through the same machinery: a rubric, a green byte-silent
`release_check.sh`, a live sabotage, and an independent adversarial panel, and must leave
`training_justified=false` unless a clean recurring model failure is proven AND the operator authorizes
training. Relaxing any criterion requires explicit operator sign-off; it must not be edited mid-stream to make a
failing check pass. Model-readiness work (corpus harvest, the verifier-as-scorer, the recurring-failure
detector, and the P11 model-need eval) may open only after this freeze lands and `horizon-track-v0.1` is
tagged; P13–P15 do not start under this freeze.
