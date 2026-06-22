# Dream Export Milestone: DREAM-0 → DREAM-EXPORT-2 (FROZEN for dream-export-v0.1)

> Status: **FROZEN** as of `dream-export-v0.1`. This document freezes the DREAM-0 through DREAM-EXPORT-2
> dream-provenance arc as a named, auditable milestone before any review, ranking, or promotion-facing dream
> behavior is added. It is the single milestone-freeze record for the dream-export layer; the per-sprint
> decisions live in `docs/PROJECT_CHARTER.md` (`DD-2026-06-21-F`, `DD-2026-06-21-G`, `DD-2026-06-21-H`,
> `DD-2026-06-21-I`). This file freezes the arc, the commit lineage, the frozen bases, the demonstrated
> capability, the preserve-provenance-not-authority boundary, the private `DreamOnly` confinement, the
> verification discipline, the training-gate verdict, the honest residuals, and the frozen-status declaration. It
> does not restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            dream-export-v0.1
points at      the DREAM-EXPORT-3 freeze commit (this document + its gate lock)
freezes        the DREAM-0..DREAM-EXPORT-2 dream-provenance arc (head ac03327)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
recover        git checkout dream-export-v0.1
training gate  training_not_justified (P12 training_justified=false) — weights forbidden
frozen base    corpus-flow-v0.1 @ b8577fe (the prior frozen multi-document corpus base),
               document-flow-v0.1 @ 0cc7399 (the frozen local-document base)
deeper base    operator-controls-v0.1 @ 34b4f47, multi-trace-validation-v0.1 @ 460be0c,
               integration-demo-v0.1 @ 95b586d, hypothesis-track-v0.1 @ bb20acf,
               reading-track-v0.1 @ f6fa55a, cognitive-os-governance-v0.1 @ bbd1113 (all frozen)
```

The dream-export arc is the first arc to add a DREAM layer to the prototype — a deliberately ALIEN generator
made lawful at the boundary. DREAM-0 adds a standalone, terminal seeded distortion engine that turns verified
corpus material into inert dream packets; DREAM-EXPORT-0 lets such a packet cross into the EXISTING
hypothesis-only proposal path while preserving its dream origin OUTSIDE the frozen authority model; DREAM-EXPORT-1
documents and smoke-tests that operator path; and DREAM-EXPORT-2 proves the bridge auditable across one valid and
six invalid export cases. The crucial property is not "the system can dream" — it is that **a dream preserves its
provenance without ever acquiring authority**: the dream's own private `DreamAuthority::DreamOnly` never leaves
`dream-engine`, the exported material is an ordinary `hypothesis_only` proposal, the dream origin stays auditable
and distinguishable, and nothing a dream touches becomes execution, evidence, promotion, memory mutation, or
training. The arc ADDS the new standalone `dream-engine` crate and GROWS the `cognitive-demo` integration layer
ONLY; it edits no frozen crate source.

## 1. What is frozen — the commit lineage

Four commits form the arc. The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| DREAM-0 | Seeded Deterministic Distortion Engine: the NEW standalone `crates/dream-engine` DISTORTS verified corpus material into terminal, inert `DreamPacket`s. It is terminal and inert — it has NO `hypothesis-layer` dependency and NO export path; `DreamAuthority::DreamOnly` is private to `dream-engine` only; grounding is rebuilt on `reading-substrate` (a canonical `execute`+`verify` read that fails closed via `DreamError::CorpusDoesNotVerify`), and preserved facts are VERBATIM verified spans (an unsupported fact is refused). It applies five seeded distortion operators (RoleInversion, CategoryViolation, ConstraintRemoval, ContradictionBraid, ScaleShift) under a `0..=5` weirdness dial, refuses degenerate output through three runtime anti-degeneracy gates, leaves falsifiers as reference-only slots, and records probe requests with `executes: false`. Each packet carries a `dream_input_hash` binding ALL admitted documents; ids are FNV-1a (no clock/entropy/floats); packets are `Serialize` but NOT `Deserialize`, replay byte-identically, and a tampered packet is refused. 20 unit tests. No LLM, no training, no execution, no evidence, no promotion | `290abee` |
| DREAM-EXPORT-0 | Dream Export Receipt / Provenance Bridge: a bridge in `cognitive-demo` re-derives the terminal `DreamPacket` (for the SAME corpus + frame + dials), builds a `HypothesisSpec` from the dream's distortion + its VERIFIED grounding receipt, and calls the EXISTING `hypothesis_layer::propose`. The result is a real `HypothesisPacket` carrying the EXISTING `Authority::HypothesisOnly`, wrapped with a `DreamExportReceipt` that preserves dream-origin provenance (packet id, input hash, seed, engine version, operator ids, grounding hashes) OUTSIDE the frozen authority model. `cognitive-demo` gains a dependency on `dream-engine` (arrow demo → engine); three verbs `dream-export` / `dream-export-report` / `dream-export-replay`. The receipt + bundle are `Serialize` but NOT `Deserialize` (re-derived and byte-compared, never parsed back into authority); the dream's private `dream_only` authority NEVER crosses; a tampered `--dream-packet` or `--export` is refused. 13 unit tests | `d3af869` |
| DREAM-EXPORT-1 | Dream Export Operator Guard / Manual + Smoke Integration: `OPERATOR_MANUAL.md` (new §14) documents the three `dream-export` commands and states the doctrine (preserves provenance, creates no new authority, exported material stays `hypothesis_only`, `DreamOnly` stays private to `dream-engine`, probe requests do not execute, nothing becomes evidence / promotion / training); `scripts/operator_smoke.sh` (new §13) runs the whole dream export flow end-to-end against a LOCAL corpus + frame — dream-export generation FIRST, then report / replay — proving the export carries the existing `hypothesis_only` authority, records `dream_origin`, emits no `dream_only`/`DreamOnly` token, and that a foreign/tampered `--dream-packet`, a tampered `DreamExportReceipt`, and a forged `dream_origin=false` are each refused. A documentation + drift-guard sprint — no code-crate behavior change | `076277d` |
| DREAM-EXPORT-2 | Dream Export Scenario Matrix / Provenance Integrity: `dream-export-scenarios` / `dream-export-matrix` / `dream-export-matrix-report` / `dream-export-matrix-verify` run a deterministic matrix of one CLEAN export that VERIFIES plus six tamper scenarios that are each REFUSED (a tampered source dream packet, a tampered receipt, a forged `dream_origin=false`, a mutated `dream_input_hash`, a mutated `dream_packet_id`, and a forged `authority_after_export` that injects the dream's private serialized token). Each row records the OBSERVED outcome (never asserted), and the matrix records the preserved dream provenance fields, that the exported material stays `hypothesis_only` and is DISTINGUISHABLE from a plain hypothesis, that probe requests never execute, and the no-execution / no-evidence / no-promotion / no-training coverage cells. `Serialize` but NOT `Deserialize` — a doctored matrix is refused. 15 unit tests | `ac03327` |

The dream-export head frozen here is `ac03327` (DREAM-EXPORT-2).

## 2. The frozen bases

The dream-export arc is built ON TOP OF the `corpus-flow-v0.1` milestone (the prior frozen multi-document corpus
base, which grounds the dream's distortion in a verified corpus read) and the `document-flow-v0.1` local-document
base beneath it, and the frozen tracks beneath those. It edits NONE of their source. The honest, precise
statement: every prior tag still points where it did, and the FROZEN crate source — the reading substrate / CLI /
codec, the hypothesis layer, and the reading train-gate — is byte-for-byte identical to its tag.

```text
corpus-flow-v0.1             @ b8577fe   (the prior frozen multi-document corpus base: a verified local corpus read)
document-flow-v0.1           @ 0cc7399   (the frozen local-document base: one operator document, read not trusted)
operator-controls-v0.1        @ 34b4f47   (operator manual / smoke drift guard / release snapshot)
multi-trace-validation-v0.1   @ 460be0c   (scenario pack / coverage matrix / failure injection)
integration-demo-v0.1         @ 95b586d   (the cognitive-demo crate: trace / report / questions / bundle)
hypothesis-track-v0.1         @ bb20acf   (propose → probe → review → intent → observation → promotion-refusal)
reading-track-v0.1            @ f6fa55a   (the read0 verifier + the verified reading receipt)
cognitive-os-governance-v0.1  @ bbd1113   (the v0.1 governance / evidence-contract lineage)
```

The dream-export arc ADDS the new standalone `dream-engine` crate (DREAM-0) and GROWS the `cognitive-demo`
integration layer (DREAM-EXPORT-0/1/2) — that is where the new capability lives — and touches no frozen crate:
`git diff b8577fe..ac03327 -- crates/reading-substrate crates/reading-cli crates/reading-codec
crates/hypothesis-layer crates/reading-train-gate` is empty. The reading verifier, the hypothesis chain, and the
governance/evidence contract are unchanged; `dream-engine` consumes only the reading substrate, and the bridge
consumes the hypothesis layer through its public `propose` API and adds no authority of its own. The
**hypothesis-layer `Authority` remains a single-variant enum** (`HypothesisOnly`); the bridge reads that existing
authority off the proposed packet and never mints a new one.

## 3. What the operator can now do (the demonstrated capability)

The prototype could already trace, report, interrogate, package, validate, forge-then-reject, and propose
hypothesis-only novelty over a verified local corpus (corpus-flow-v0.1). The dream-export arc adds a deliberately
alien generator made lawful at the boundary:

1. **Distort verified material into a terminal dream (DREAM-0).** `dream-engine` turns a verified corpus read
   into an inert `DreamPacket` — five seeded distortion operators under a bounded weirdness dial, grounded on
   VERBATIM verified spans, with non-executing probe requests and a whole-corpus input hash. The packet is
   terminal: it has no export path of its own and its `DreamOnly` authority is private to the engine.
2. **Bridge a dream into the hypothesis-only path, preserving provenance (DREAM-EXPORT-0).** `dream-export`
   re-derives the terminal packet and bridges it through the EXISTING `hypothesis_layer::propose`, emitting a
   `DreamExportReceipt` + the proposed `HypothesisPacket`. The exported material carries the EXISTING
   `hypothesis_only` authority; the receipt records the dream origin so it stays auditable and distinguishable; a
   tampered packet or export is refused.
3. **Keep the dream-export path documented and honest (DREAM-EXPORT-1).** The operator manual documents the three
   `dream-export` commands and the preserve-provenance / no-new-authority / `DreamOnly`-stays-private boundary;
   the operator smoke runs the whole flow end-to-end and fails closed if it drifts from the manual.
4. **Prove the bridge holds across valid and invalid exports (DREAM-EXPORT-2).** A seven-scenario matrix proves a
   clean export verifies while a tampered packet, a tampered receipt, a stripped `dream_origin`, a mutated input
   hash or packet id, and a forged authority are each refused — every outcome OBSERVED by running the real
   verifier, never asserted — and records the preserved provenance, the distinguishability, and the
   no-evidence/promotion/training cells.

Every one of these distorts or bridges through the frozen reader/hypothesis gate or fails closed. None of them is
authority, and none lets a dream act.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint; each is
enforced by the release gate from the artifacts' own bytes.

1. **Dream export preserves provenance, not authority (DREAM-EXPORT-0/1/2).** The bridge carries the dream's
   origin (packet id, input hash, seed, engine version, operators, grounding hashes) so it stays auditable, but
   `authority_after_export` is the EXISTING `Authority::HypothesisOnly` read off the proposed packet — never a new
   or fabricated variant.
2. **`DreamOnly` stays private to `dream-engine` (DREAM-0/EXPORT-0/1/2).** The dream's own
   `DreamAuthority::DreamOnly` is crate-private to `dream-engine` and appears in no other crate's source; a
   release gate asserts the PascalCase identifier occurs ONLY under `crates/dream-engine`. The forged-authority
   scenario only ever FORGES-then-REFUSES the lowercase serialized token; it never mints it.
3. **Exported material remains `hypothesis_only` and DISTINGUISHABLE (DREAM-EXPORT-0/2).** The exported hypothesis
   cites a `dream:` provenance label and the receipt records `dream_origin`, so a dream-exported hypothesis is
   distinguishable from a plain one while carrying only the existing hypothesis-only authority.
4. **Verify before bridging (DREAM-0/EXPORT-0).** The dream is grounded on a frozen-VERIFIED corpus read and
   refuses a degenerate dream; the export re-derives that terminal packet and fails closed if there is nothing
   valid to export.
5. **Re-derive, never trust the artifacts (DREAM-0/EXPORT-0/1/2).** Every surface that accepts a file
   (`dream-export-report`/`-replay`, `dream-export-matrix-report`/`-verify`, and the dream-engine packet verify)
   re-derives from the SAME corpus + frame + dials and byte-compares; a tampered packet, receipt, bundle, or
   matrix is refused. No dream record derives `Deserialize`.
6. **Probe requests do not execute (DREAM-0/EXPORT-0/2).** The source dream's probe requests are recorded with
   `executes: false`; the matrix records `probe_requests_execute: false` and `no_execution: true`.
7. **The dream layer creates no evidence, promotion, or training (whole arc).** A dream-exported hypothesis
   carries the canonical hypothesis-layer quarantine (`forbidden_uses`): it can never become evidence, promote, or
   change the training gate. The strongest honest case is preserved from the frozen layers below: governance may
   APPROVE a probe, yet execution stays `requires_operator`, the observation stays `observation_only`, and the
   promotion request is `rejected`.
8. **No model in the loop.** The dream engine is fully deterministic — the operator's `--frame` is the proposer
   and the harness only verifies, grounds, distorts, and structures. Any future model could only PROPOSE through
   the frozen hypothesis layer; it can never ground a claim, mint authority, mutate memory, execute a probe,
   promote evidence, or self-authorize.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is `training_justified=false`).
The dream-export arc is orthogonal to P12 and does not move it: the export and the scenario matrix read the
training decision as unchanged, and the matrix records the `no_training` boundary cell true for every scenario.
Weight training stays forbidden until the P11 eval proves a stable, recurring model failure that survives fixes to
task spec, schema, prompt, examples, tooling, context, and verifier design. P13–P15 (LoRA candidate, shadow mode,
promotion gate) stay closed under this freeze. This milestone makes no claim that training has opened, that a
dream becomes evidence, or that any probe executes or promotes.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0 bytes
stderr). The dream-export locks pin, per sprint: the `dream-engine` API and the terminal-packet design (no
`hypothesis-layer` dep, no export path, `DreamOnly` private, FNV ids, Serialize-not-Deserialize, the anti-
degeneracy gates), the 20 DREAM-0 first-tests and the dream-engine quarantine via `cargo tree` (DREAM-0); the
demo → engine dependency arrow, the bridge through the EXISTING `propose`, `authority_after_export:
hypothesis.authority()`, `dream_origin: true`, the absence of any new authority enum or `DreamOnly` token in
`cognitive-demo`, the single-variant hypothesis-layer `Authority`, and the 13 DREAM-EXPORT-0 first-tests
(DREAM-EXPORT-0); the manual's documentation of the three commands and the doctrine, and an actual RUN of the
operator smoke over the dream-export flow proving every tamper refusal (DREAM-EXPORT-1); and the matrix API and
four commands, the seven scenarios, the source-safe nine-line matrix boundary, the 15 DREAM-EXPORT-2 first-tests
and the unit count held at 167, and a binary smoke that proves the coverage cells and refuses a tampered matrix
(DREAM-EXPORT-2). This milestone block additionally pins the freeze record itself (this document's
DREAM-0..DREAM-EXPORT-2 commit lineage, the frozen-base tag and commit references, the nine boundary lines, the
private-`DreamOnly` confinement, the exported-material-stays-`hypothesis_only` and single-variant-`Authority`
facts, the auditable `dream_origin`, and the `training_not_justified` verdict), and guards against any milestone
that falsely claims training has opened. The pinned commit hashes are auditable against `git log`; this lock stays
git-free and does NOT require the tag to exist — the tag is created only after a clean tree and a green gate. The
acceptance discipline for every sprint in this arc was: rubric → green byte-silent `release_check` → live sabotage
proving the gate catches a regression (restored byte-identical by `cp`+`md5`, never `git checkout`) → an
independent read-only adversarial verifier panel with a fresh context → any residual folded before close.

## 7. Independent verification

Every sprint DREAM-0 through DREAM-EXPORT-2 was closed against read-only adversarial panels (Explore agents,
refute-by-default, scratch confined to a temp dir, each driving the compiled binary or inspecting the artifacts),
run until a fully-dry round with zero real findings. DREAM-EXPORT-1 and DREAM-EXPORT-2 each returned VERDICT
ALL PASS with file-and-line evidence for every rubric criterion, and DREAM-EXPORT-2's panel additionally confirmed
the matrix is OBSERVED, not hard-coded. This DREAM-EXPORT-3 freeze adds no behavior; it is verified by a green
byte-silent gate, live sabotage of the milestone lock (restored byte-identical via `cp`+`md5`, never `git
checkout`), and an independent read-only adversarial panel. Each gate lock across the arc was proven load-bearing
by live sabotage that failed the gate and was restored byte-identical. Every claim in this document is checkable by
running `scripts/release_check.sh` and reading the named commits.

## 8. Honest residuals (NOT closed in dream-export-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the dream-export
layer, not bugs.

1. **A dream is a candidate to probe, not a claim.** The whole arc PROPOSES; it does not prove. A dream-exported
   hypothesis is `hypothesis_only`, highly speculative (low prior, high uncertainty, reversible low-risk probe),
   and grounded only on the verified reading receipt it cites. Review, ranking, scoring, or promotion of dream
   exports is future work, all of which must stay under the same preserve-provenance-not-authority discipline.
2. **The dream engine has no standalone packet emitter.** `dream-engine` is a quarantined library with no binary;
   a `DreamPacket` is re-derived (generated) INSIDE `dream-export` from the corpus + frame + dials. The optional
   `--dream-packet` cross-check is verified byte-for-byte against the re-derived packet, never trusted.
3. **Integrity is byte-for-byte re-derivation, not a digest.** The load-bearing tamper check is re-deriving the
   packet / bundle / matrix from the same corpus + frame and byte-comparing within one deterministic build;
   cross-version reproduction and cryptographic digests are not claimed.
4. **The matrix is a fixed scenario set, not exhaustive fuzzing.** DREAM-EXPORT-2 proves one clean export and six
   specific tampers; it is a finite, enumerated integrity matrix, not a proof that NO other mutation could pass.
   Every tamper outcome is OBSERVED from the real verifier, so the set is honest about what it covers.
5. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline and the gate locks defend
   against off-wire tampering and accidental regression, both of which the gate provably catches. They do not
   defend against an insider with commit access who authors malicious code AND rewrites the gate in the same
   change — that is the domain of code review and the governance/signing layer.
6. **No model in the loop.** The dream engine is deterministic; the operator frame is data, not a model. Any
   future model may only PROPOSE through the frozen hypothesis layer; it can never ground a claim, mint authority,
   mutate memory, execute a probe, promote evidence, or self-authorize. The P10 adapter stays gated shut by P12.
7. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production reasoning
   system, and the dream-export layer is described as such.
8. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left stray
   debris in the working tree despite their read-only instruction, and have occasionally inverted the
   finding-label semantics; each was caught and reconciled before close. It remains a known operational caveat of
   the panel method. Separately, an unrelated process left Python/CSV edits dirty in the working tree across this
   arc; every dream-export commit used explicit pathspecs so that dirt never entered a dream commit.

## 9. Frozen-status declaration

The DREAM-0 → DREAM-EXPORT-2 dream-provenance arc is **FROZEN at `dream-export-v0.1`**. The
preserve-provenance-not-authority boundary is the frozen surface:

```text
Dream export preserves provenance.
It does not create a new authority.
Exported dream material remains HypothesisOnly.
Dream origin remains auditable.
DreamOnly remains private to dream-engine.
Probe requests do not execute.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

Any change that lets a dream become trusted authority; that lets `DreamOnly` leave `dream-engine` or enter the
hypothesis-layer / cognitive-demo authority space; that strips dream-origin provenance without refusal; that makes
a dream-exported hypothesis indistinguishable from a plain one; that lets a dream become evidence; that executes a
probe, promotes an observation, mutates memory, or reopens training — must pass through the same machinery: a
rubric, a green byte-silent `release_check.sh`, a live sabotage, and an independent adversarial panel, and must
leave `training_justified=false` unless a clean recurring model failure is proven. Relaxing any criterion requires
explicit operator sign-off; it must not be edited mid-stream to make a failing check pass. P13–P15 do not start
under this freeze.
