# Data Curation Milestone: DATA-0 → DATA-2 (FROZEN for data-curation-v0.1)

> Status: **FROZEN** as of `data-curation-v0.1`. This document freezes the DATA-0 through DATA-2 dataset-curation
> arc as a named, auditable milestone before HORIZON-0 (the staged interaction harness) opens. It is the single
> milestone-freeze record for the curation layer; the per-sprint decisions live in `docs/PROJECT_CHARTER.md`
> (`DD-2026-06-22-K`, `DD-2026-06-22-L`, `DD-2026-06-22-M`). This file freezes the arc, the commit lineage, the
> prior frozen substrate base, the demonstrated capability, the classification-not-evidence boundary, the
> structurally-closed training eligibility, the verification discipline, the training-gate verdict, the honest
> residuals, and the frozen-status declaration. It does not restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            data-curation-v0.1
points at      the DATA-3 freeze commit (this document + its gate lock)
freezes        the DATA-0..DATA-2 dataset-curation arc (head c84233a)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
recover        git checkout data-curation-v0.1
training gate  training_not_justified (P12 training_justified=false) — weights forbidden
prior base     dream-export-v0.1 @ 5238fe8 (the prior frozen substrate milestone: the dream-provenance arc)
frozen base    corpus-flow-v0.1 @ b8577fe (the frozen multi-document corpus base),
               document-flow-v0.1 @ 0cc7399 (the frozen local-document base)
deeper base    operator-controls-v0.1 @ 34b4f47, multi-trace-validation-v0.1 @ 460be0c,
               integration-demo-v0.1 @ 95b586d, hypothesis-track-v0.1 @ bb20acf,
               reading-track-v0.1 @ f6fa55a, cognitive-os-governance-v0.1 @ bbd1113 (all frozen)
```

The curation arc is the first arc of the **substrate-not-agent** track recorded in `DD-2026-06-22-K`: long-horizon
capability comes from interaction structure and trustworthy trajectories, not bigger prompts, so before any
ingestion / memory / horizon / training path may consume material, a deterministic gate must decide what is even
admissible. DATA-0 adds that gate as a NEW standalone `crates/data-curator` that classifies a caller-supplied
`CandidateManifest` into admitted / rejected / quarantined records and emits a `CurationReceipt`; DATA-1 documents
and smoke-tests the operator path over the REAL curator; and DATA-2 proves the gate across a fixed 12-scenario
matrix driven by the real `curate()`. The crucial property is not "the system can collect data" — it is that
**curation only classifies: it admits, rejects, or quarantines, and it can never open training**. The curator
mints no authority, creates no evidence, writes no memory, executes nothing, promotes nothing, and trains nothing;
its `TrainingEligibility` carries no value that permits training. The arc ADDS the standalone `data-curator` crate
ONLY; it edits no frozen crate source.

## 1. What is frozen — the commit lineage

Three commits form the arc. The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| DATA-0 | Dataset Curation Manifest / Ingestion Gate: the NEW standalone `crates/data-curator` (workspace member, deps = **serde only**, no workspace dependency — it cannot reach the hypothesis-layer `Authority`, the reading train-gate, or any engine/memory crate). `curate(&CandidateManifest) -> CurationReceipt` classifies caller-supplied items into admitted / rejected / quarantined; it is pure and deterministic (FNV-1a length-prefixed hashing, `BTree` ordering, no clock / entropy / float / filesystem / network / process / async — manifest-in, receipt-out). It **rejects** missing-provenance, duplicate-id, empty-content, unsupported-type, ungrounded-durable (document/corpus span, dream packet), trace-without-replay-receipt, and invalid-split items; it **quarantines (never deletes, never admits)** prompt-injection markers (a closed 10-marker case-insensitive tripwire) and train/holdout leakage (the same content hash in both splits). The receipt carries an order-independent `dataset_hash` (the canonical admitted set) and an order-sensitive `source_manifest_hash` (binding the exact input), is `Serialize` but NOT `Deserialize` (re-derived via `curate`, never trusted from bytes), and records a `BoundaryChecks::inert()` that asserts mints-no-authority / no-evidence / no-promotion / no-execution / no-memory-ingest. `TrainingEligibility` is `{ Closed (default), CandidateOnly }` with `is_eligible()` wired to `const TRAINING_PERMITTED: bool = false` → unconditionally false. `#![forbid(unsafe_code)]`. 19 unit tests. No LLM, no training, no execution, no evidence, no promotion | `2a3e6aa` |
| DATA-1 | Curation Operator Guard / Manual + Smoke Integration: `OPERATOR_MANUAL.md` (new §15) documents the curation operator path — the curator ADMITS / REJECTS / QUARANTINES candidate data, a prompt-injection marker is **quarantined, not deleted**, train/holdout leakage is quarantined, duplicate ids and missing provenance are rejected, and training eligibility is structurally closed (`no code path can return training-eligible=true`, `const TRAINING_PERMITTED: bool = false`); `scripts/operator_smoke.sh` drives the REAL `curate()` over candidate manifests via six `--exact` named tests (clean→admitted, missing-provenance→rejected, duplicate→rejected, prompt-injection→quarantined, train/holdout-leakage→quarantined, eligibility→never-eligible), each asserting `1 passed` so a dropped outcome is caught as vacuous. A documentation + drift-guard sprint — NO code-crate change (`data-curator` src byte-identical; the DATA-0 gate unchanged). The smoke is RUN by the gate, so a curation drift fails closed | `a0bfd04` |
| DATA-2 | Curation Scenario Matrix: `crates/data-curator/src/matrix.rs` adds a FIXED, named set of 12 candidate-data scenarios; `curation_matrix()` constructs a real `CandidateManifest` per scenario and runs the REAL `curate()`, recording the OBSERVED `CurationReceipt` disposition in each `ScenarioCell` (admitted / rejected / quarantined counts, the dominant `Outcome`, the first reject/quarantine reason label, the `training_eligibility`, the `opens_training` bit, and the per-scenario `dataset_hash` / `source_manifest_hash`). `opens_training = receipt.training_eligibility.is_eligible()` is false for every cell, and `training_never_opens` is the conjunction across all 12. The cells derive `Serialize` but NOT `Deserialize` and are `PartialEq` / `Eq`, so the matrix is re-derived and compared, never trusted from bytes. 4 matrix unit tests (count, observed cells, no-training invariant, determinism); 23 crate tests total. `types.rs` / `curate.rs` untouched | `c84233a` |

The curation head frozen here is `c84233a` (DATA-2).

The 12 DATA-2 scenarios, named, are: `clean_document_admitted`, `missing_provenance_rejected`,
`duplicate_id_rejected`, `empty_content_rejected`, `unsupported_artifact_rejected`,
`prompt_injection_quarantined`, `split_leakage_quarantined`, `ungrounded_durable_rejected`,
`trace_without_replay_rejected`, `valid_split_admitted`, `invalid_split_rejected`, and
`training_eligibility_never_opens`.

## 2. The frozen bases

The curation arc is built AFTER the `dream-export-v0.1` substrate milestone (the prior frozen dream-provenance
arc) and the frozen corpus / document / hypothesis / governance tracks beneath it. The honest, precise statement:
every prior tag still points where it did, and the curation crate is **deliberately isolated** — it depends on no
workspace crate, so it cannot reach the `Authority` type, the training gate, or any engine / memory crate. The
arc ADDS `crates/data-curator` and edits no frozen crate source.

```text
dream-export-v0.1            @ 5238fe8   (the prior frozen substrate milestone: the dream-provenance arc)
corpus-flow-v0.1             @ b8577fe   (the frozen multi-document corpus base: a verified local corpus read)
document-flow-v0.1           @ 0cc7399   (the frozen local-document base: one operator document, read not trusted)
operator-controls-v0.1        @ 34b4f47   (operator manual / smoke drift guard / release snapshot)
multi-trace-validation-v0.1   @ 460be0c   (scenario pack / coverage matrix / failure injection)
integration-demo-v0.1         @ 95b586d   (the cognitive-demo crate: trace / report / questions / bundle)
hypothesis-track-v0.1         @ bb20acf   (propose → probe → review → intent → observation → promotion-refusal)
reading-track-v0.1            @ f6fa55a   (the read0 verifier + the verified reading receipt)
cognitive-os-governance-v0.1  @ bbd1113   (the v0.1 governance / evidence-contract lineage)
```

`crates/data-curator` is a standalone workspace member whose normal dependency tree contains no `vibe-*`,
`reading-*`, `hypothesis-layer`, `cognitive-demo`, or `dream-engine` crate. It therefore cannot acquire authority,
mutate memory, or open training even by accident: it has no path to those types at all. The DATA-1 documentation
and DATA-2 matrix changed no crate behavior — `git diff 2a3e6aa..c84233a -- crates/data-curator/src/types.rs
crates/data-curator/src/curate.rs` is empty.

## 3. What the operator can now do (the demonstrated capability)

The prototype could already read, trace, report, interrogate, package, validate, propose hypothesis-only novelty,
and bridge a dream into the hypothesis-only path with preserved provenance (dream-export-v0.1). The curation arc
adds the ingestion gate that stands BEFORE any of that consumes new material:

1. **Classify candidate data into admitted / rejected / quarantined (DATA-0).** `curate()` takes a
   caller-supplied `CandidateManifest` and returns a `CurationReceipt`: structurally admissible items are
   admitted, malformed or unsupported or ungrounded items are rejected with a reason, and prompt-injection or
   train/holdout-leakage items are quarantined — held, never deleted and never admitted. The receipt binds the
   admitted set and the exact input, and records that the run minted no authority, created no evidence, promoted
   nothing, executed nothing, ingested nothing into memory, and left training closed.
2. **Exercise and audit the gate from the operator path (DATA-1).** The operator manual documents what is
   admitted, rejected, and quarantined and states the doctrine; the operator smoke drives the REAL `curate()` over
   candidate manifests and fails closed if the gate drifts from the manual.
3. **Prove the gate across the full disposition surface (DATA-2).** A fixed 12-scenario matrix runs the real
   `curate()` over a clean admit, each reject reason, each quarantine reason, leakage, grounding, replay, split,
   and an eligibility probe, recording each OBSERVED outcome — never asserting it — and proving that
   `opens_training` is false in every single scenario.

Every one of these classifies through the real curator or fails closed. None of them is authority, none creates
evidence, and none opens training.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint; each is
enforced by the release gate from the artifacts' own bytes.

1. **Curation is classification, not evidence (whole arc).** `curate()` decides admissibility; it asserts no
   truth and creates no evidence. An admitted item is "structurally allowed in", not "true" — the receipt's
   `BoundaryChecks::inert()` records that the run mints no authority and creates no evidence.
2. **Quarantine holds; it does not delete (DATA-0/1/2).** Prompt-injection markers and train/holdout leakage are
   QUARANTINED — recorded in `quarantined_items` with a reason, neither admitted nor deleted — so the offending
   material remains auditable. The manual and the matrix both state and prove quarantine-not-deletion.
3. **Training eligibility cannot open (whole arc).** `TrainingEligibility` has exactly two inhabitable values,
   `Closed` (the default) and `CandidateOnly`, and BOTH report `is_eligible() == false`. There is **no `Eligible`
   or `TrainingEligible` variant**, and `is_eligible()` returns the single `const TRAINING_PERMITTED: bool =
   false`. `CandidateOnly` means "structurally admissible, but training stays gated by a later gate that does not
   exist yet" — it is explicitly NOT training-eligible. Opening training is the job of a future gate; this crate
   carries no value that permits it.
4. **Re-derive, never trust the artifacts (DATA-0/2).** The `CurationReceipt` and the scenario matrix derive
   `Serialize` but NOT `Deserialize`: integrity is re-deriving from the primary input via `curate()` and
   comparing, never parsing a receipt or matrix back in and trusting it.
5. **The curator is pure and isolated (DATA-0).** No filesystem, network, process, clock, entropy, float, or
   async appears in its source; it depends on no workspace crate. It cannot reach the `Authority` type, the
   training gate, memory, or any engine — it classifies a manifest and returns a receipt.
6. **The curator creates no memory, execution, or promotion (whole arc).** The receipt's boundary checks record
   no-memory-ingest, no-execution, and no-promotion; nothing the curator does mutates state, runs a probe, or
   promotes an item.
7. **No model in the loop.** Curation is fully deterministic — the caller supplies the manifest and the gate
   classifies it. Any future model could only PROPOSE candidate data into a manifest; it can never self-admit,
   mint authority, open training, mutate memory, execute, or promote.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is `training_justified=false`).
The curation arc is orthogonal to P12 and does not move it: `TrainingEligibility` carries no training-permitting
value, `is_eligible()` is pinned to `TRAINING_PERMITTED = false`, and the DATA-2 matrix records `opens_training`
false for every one of the 12 scenarios — including the clean-admit and valid-split scenarios, whose eligibility
is at most `CandidateOnly` (NOT eligible). Weight training stays forbidden until the P11 eval proves a stable,
recurring model failure that survives fixes to task spec, schema, prompt, examples, tooling, context, and verifier
design. P13–P15 (LoRA candidate, shadow mode, promotion gate) stay closed under this freeze. This milestone makes
no claim that training has opened, that `CandidateOnly` means training-eligible, that an admitted item is
evidence, or that anything was ingested into memory.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0 bytes
stderr). The curation locks pin, per sprint: the `data-curator` API and purity (the `curate` entrypoint, the
`CurationReceipt`, the default-closed `TrainingEligibility`, `BoundaryChecks::inert`, quarantine-not-delete for
BOTH quarantine reasons, `#![forbid(unsafe_code)]`, `const TRAINING_PERMITTED: bool = false`, the no-filesystem /
network / process / clock / entropy / async source scan, and the no-workspace-dependency `cargo tree` boundary),
plus the cargo test / fmt / clippy battery (DATA-0); the manual's documentation of the operator path and an actual
RUN of the operator smoke over the REAL `curate()` proving every disposition (DATA-1); and the matrix API, the 12
scenario names, the outcome reason labels, the count, the no-derived-`Deserialize` property, `opens_training =
is_eligible()`, `training_never_opens`, and the matrix tests (DATA-2). This milestone block additionally pins the
freeze record itself (this document's DATA-0..DATA-2 commit lineage `2a3e6aa` / `a0bfd04` / `c84233a`, the prior
frozen `dream-export-v0.1` @ `5238fe8` substrate base and the deeper frozen tags + commits, the eight boundary
lines, the `Closed` vs `CandidateOnly` distinction, the `TRAINING_PERMITTED`-pinned-false and no-`Eligible`-variant
facts, the classification-not-evidence and quarantine-not-deletion invariants, and the `training_not_justified`
verdict), and guards against any milestone that falsely claims training has opened. The pinned commit hashes are
auditable against `git log`; this lock stays git-free and does NOT require the tag to exist — the tag is created
only after a clean tree and a green gate. The acceptance discipline for every sprint in this arc was: rubric →
green byte-silent `release_check` → live sabotage proving the gate catches a regression (restored byte-identical
by `cp`+`md5`, never `git checkout`) → an independent read-only adversarial verifier panel with a fresh context →
any residual folded before close.

## 7. Independent verification

Every sprint DATA-0 through DATA-2 was closed against read-only adversarial panels (Explore agents, refute-by-
default, scratch confined to a temp dir, each driving the compiled tests or inspecting the artifacts), run until a
fully-dry round with zero real findings: DATA-0 returned 29 PASS / 0 FAIL, DATA-1 returned 31 PASS / 0 FAIL, and
DATA-2 returned ALL PASS / 0 defects across 26 criteria, with the panel confirming the matrix is OBSERVED, not
hard-coded. This DATA-3 freeze adds no behavior; it is verified by a green byte-silent gate, live sabotage of the
milestone lock (restored byte-identical via `cp`+`md5`, never `git checkout`), and an independent read-only
adversarial panel. Each gate lock across the arc was proven load-bearing by live sabotage that failed the gate and
was restored byte-identical. Every claim in this document is checkable by running `scripts/release_check.sh` and
reading the named commits.

## 8. Honest residuals (NOT closed in data-curation-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the curation layer,
not bugs.

1. **Curation decides admissibility, not truth.** An admitted item is structurally allowed in; it is not endorsed
   as true, and the curator creates no evidence. Scoring, ranking, or trusting admitted material is future work,
   all of which must stay under the same classification-not-evidence discipline.
2. **The curator consumes a manifest, not a filesystem.** `curate()` classifies a caller-supplied
   `CandidateManifest`; it does no filesystem traversal or ingestion of its own (by boundary design — it has no
   IO). Building the manifest from real sources, and trusting that builder, is out of scope here.
3. **Integrity is byte-for-byte re-derivation, not a digest.** The load-bearing tamper check is re-deriving the
   receipt / matrix from the same manifest via `curate()` and comparing within one deterministic build;
   cross-version reproduction and cryptographic digests are not claimed.
4. **The injection tripwire and the scenario matrix are fixed sets, not exhaustive.** The prompt-injection check
   is a closed marker list and the matrix is 12 enumerated scenarios; they are honest about what they cover and
   are not a proof that no other injection string or candidate shape exists. Every matrix outcome is OBSERVED from
   the real curator, so the set cannot drift from actual behavior.
5. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline and the gate locks defend
   against off-wire tampering and accidental regression, both of which the gate provably catches. They do not
   defend against an insider with commit access who authors malicious code AND rewrites the gate in the same
   change — that is the domain of code review and the governance/signing layer.
6. **No model in the loop.** Curation is deterministic; the manifest is data, not a model. Any future model may
   only PROPOSE candidate data; it can never self-admit, open training, mint authority, mutate memory, execute, or
   promote. The training gate stays closed under P12.
7. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production data
   pipeline, and the curation layer is described as such.
8. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left stray
   debris in the working tree despite their read-only instruction, and have occasionally inverted the
   finding-label semantics; each was caught and reconciled before close. It remains a known operational caveat of
   the panel method. Separately, an unrelated process left Python/CSV edits dirty in the working tree across this
   arc; every curation commit used explicit pathspecs so that dirt never entered a curation commit.

## 9. Frozen-status declaration

The DATA-0 → DATA-2 dataset-curation arc is **FROZEN at `data-curation-v0.1`**. The classification-not-evidence
boundary is the frozen surface:

```text
Data curation classifies candidate data.
It admits, rejects, or quarantines.
It does not create truth.
It does not create memory.
It does not train.
It does not execute.
It does not promote.
Training eligibility remains closed.
```

Any change that lets curation create evidence; that lets quarantine delete source material instead of holding it;
that adds a training-permitting `TrainingEligibility` value or makes `CandidateOnly` report eligible; that lets
the curator mint authority, mutate memory, execute, or promote; or that reopens training — must pass through the
same machinery: a rubric, a green byte-silent `release_check.sh`, a live sabotage, and an independent adversarial
panel, and must leave `training_justified=false` unless a clean recurring model failure is proven. Relaxing any
criterion requires explicit operator sign-off; it must not be edited mid-stream to make a failing check pass.
HORIZON-0 may open only after this freeze lands and `data-curation-v0.1` is tagged; P13–P15 do not start under
this freeze.
