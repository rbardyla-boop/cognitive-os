# Document Flow Milestone: DOCFLOW-0 → DOCFLOW-2 (FROZEN for document-flow-v0.1)

> Status: **FROZEN** as of `document-flow-v0.1`. This document freezes the DOCFLOW-0 through DOCFLOW-2
> local-document-flow arc as a named, auditable milestone before any further document behavior is added. It
> is the single milestone-freeze record for the document-flow layer; the per-sprint decisions live in
> `docs/PROJECT_CHARTER.md` (`DD-2026-06-20-H`, `DD-2026-06-20-I`, `DD-2026-06-20-J`). This file freezes the
> arc, the commit lineage, the frozen base, the demonstrated capability, the read-not-trust boundary, the
> verification discipline, the training-gate verdict, the honest residuals, and the frozen-status
> declaration. It does not restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            document-flow-v0.1
points at      the DOCFLOW-3 freeze commit (this document + its gate lock)
freezes        the DOCFLOW-0..DOCFLOW-2 document-flow arc (head 4a04759)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
recover        git checkout document-flow-v0.1
training gate  training_not_justified (P12 training_justified=false) — weights forbidden
frozen base    operator-controls-v0.1 @ 34b4f47 (frozen)
deeper base    multi-trace-validation-v0.1 @ 460be0c, integration-demo-v0.1 @ 95b586d,
               hypothesis-track-v0.1 @ bb20acf, reading-track-v0.1 @ f6fa55a,
               cognitive-os-governance-v0.1 @ bbd1113 (all frozen)
```

The document-flow arc is the first arc to add OPERATOR-SUPPLIED INPUT to the prototype. Before it, every
trace ran from a single fixed canonical corpus; the document flow lets an operator point the system at a
local text file and get the same verified-to-refused trace. The crucial property is not "it reads a file" —
it is that **the file is read but never trusted**: it is read, verified through the frozen reader into a
verified reading receipt, traced, and still cannot become execution, evidence, promotion, memory mutation,
or training. The arc adds capability to the `cognitive-demo` integration layer ONLY; it edits no frozen
crate source.

## 1. What is frozen — the commit lineage

Three commits form the arc. The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| DOCFLOW-0 | Operator-Supplied Document Trace / Read-Only Input Demo: `doc-trace` / `doc-report` / `doc-bundle` / `doc-bundle-verify` run the SAME end-to-end pipeline from a LOCAL operator-supplied text document. The flow asks the frozen reader (`corpus_from_documents`) for the document's own first span, grounds a plan against it, and starts from a frozen-VERIFIED read0 receipt — failing closed if the read does not verify. The shell validates the input path (absolute / `..` / symlink-escape refused); the library stays filesystem-free. Adds a direct dependency on the already-frozen `reading-substrate`; no frozen crate SOURCE edited | `c9bd1e5` |
| DOCFLOW-1 | Document Flow Operator Guard / Manual + Smoke Integration: `OPERATOR_MANUAL.md` documents the four `doc-*` commands and states the document is read but not trusted; `scripts/operator_smoke.sh` runs the whole doc flow end-to-end against a local sample document and proves a tampered document, trace, report, or manifest is refused. A documentation + drift-guard sprint — no code-crate behavior change | `b288196` |
| DOCFLOW-2 | Document Flow Scenario Pack / Input-Integrity Matrix: `doc-scenarios` / `doc-scenario-pack` / `doc-scenario-verify` / `doc-scenario-matrix` run a finite, enum-backed set of nine VALID and INVALID inputs — clean, modified, empty, absolute path, `..` traversal, symlink escape, and tampered trace/report/manifest — each OBSERVED by running the REAL DOCFLOW-0 check and recording the outcome. The matrix records all outcomes (verified 1, refused 8, 36/36 boundary cells). No frozen crate source, no new dependency | `4a04759` |

The document-flow head frozen here is `4a04759` (DOCFLOW-2).

## 2. The frozen base

The document-flow arc is built ON TOP OF the operator-controls milestone and the frozen tracks it documents,
and it edits NONE of their source. The honest, precise statement: every prior tag still points where it did,
and the FROZEN crate source — the reading substrate / CLI / codec, the hypothesis layer, and the reading
train-gate — is byte-for-byte identical to its tag.

```text
operator-controls-v0.1        @ 34b4f47   (operator manual / smoke drift guard / release snapshot)
multi-trace-validation-v0.1   @ 460be0c   (scenario pack / coverage matrix / failure injection)
integration-demo-v0.1         @ 95b586d   (the cognitive-demo crate: trace / report / questions / bundle)
hypothesis-track-v0.1         @ bb20acf   (propose → probe → review → intent → observation → promotion-refusal)
reading-track-v0.1            @ f6fa55a   (the read0 verifier + the verified reading receipt)
cognitive-os-governance-v0.1  @ bbd1113   (the v0.1 governance / evidence-contract lineage)
```

Unlike the operator-controls arc (which added no `cognitive-demo` behavior), the document-flow arc GROWS the
`cognitive-demo` integration layer — that is where the new capability lives. It does not touch any frozen
crate: `git diff 460be0c..4a04759 -- crates/reading-substrate crates/reading-cli crates/reading-codec
crates/hypothesis-layer crates/reading-train-gate` is empty. The reading verifier, the hypothesis chain, and
the governance/evidence contract are unchanged; the document flow consumes them through their public APIs and
adds no authority of its own.

## 3. What the operator can now do (the demonstrated capability)

The prototype could already produce, report, interrogate, package, validate, and forge-then-reject one
canonical trace, and an operator could read, verify, and snapshot that frozen system. The document-flow arc
adds operator-supplied input over that frozen system:

1. **Trace a local document (DOCFLOW-0).** Point `doc-trace` at a local text file and get the same
   end-to-end trace, starting from a verified reading receipt over the document's own first span. The
   document is read, verified, and traced — and fails closed if it does not verify.
2. **Keep the document path documented and honest (DOCFLOW-1).** The operator manual documents the four
   `doc-*` commands and states the read-not-trust boundary; the operator smoke runs the whole doc flow
   end-to-end and fails closed if it drifts from the manual.
3. **Prove the flow holds across valid and invalid inputs (DOCFLOW-2).** A nine-scenario input-integrity
   pack and matrix prove a clean document verifies while a modified, empty, unsafe-path, escaping, or
   tampered input is refused — each outcome OBSERVED by running the real check, never asserted.

Every one of these reads local input and either verifies it through the frozen reader or fails closed. None
of them is authority, and none lets the input act.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint; each is
enforced by the release gate from the artifacts' own bytes.

1. **Read, never trust (DOCFLOW-0/1/2).** Operator-supplied text is read and verified through the frozen
   reader before it is traced; it is never accepted as authority. A document becomes a verified read of its
   own text or nothing at all.
2. **Verify before tracing (DOCFLOW-0).** The trace starts from a frozen-VERIFIED read0 receipt over the
   document; if the read does not verify, the flow fails closed and produces no trace.
3. **Re-derive, never trust the artifacts (DOCFLOW-0/1/2).** Every surface that accepts a file
   (`doc-report`, `doc-bundle-verify`, `doc-scenario-verify`, `doc-scenario-matrix`) re-derives from the
   SAME document/scenario set and byte-compares; a tampered document, trace, report, questions, manifest,
   pack, or matrix is refused. No record derives `Deserialize`.
4. **Path safety (DOCFLOW-0/2).** The input path is validated: an absolute path, a `..` traversal, and a
   symlink that escapes the working directory are each refused — proven end-to-end through the binary, not
   only via the pure containment decision.
5. **Vary the input, not the authority (DOCFLOW-2).** Across all nine scenarios — valid and invalid — the
   four boundary cells (no_execution / no_evidence / no_promotion / no_training) hold. The strongest honest
   case is preserved from the frozen layers below: governance may APPROVE a probe, yet execution stays
   `requires_operator` (never `executed`), the observation stays `requires_review` / `observation_only`
   (never `recorded`), and the promotion request is `rejected`. Approval is not execution; an observation is
   not evidence.
6. **No model in the loop.** The document flow is fully deterministic; the operator's text is data read
   through the frozen reader, not a model. Any future model could only PROPOSE through the frozen hypothesis
   layer; it can never ground a claim, mutate memory, execute a probe, promote evidence, or self-authorize.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is
`training_justified=false`). The document-flow arc is orthogonal to P12 and does not move it: every DOCFLOW
sprint reads the training decision before and after building its artifacts and proves it identical, and the
scenario matrix records the no_training boundary cell true for every scenario. Weight training stays
forbidden until the P11 eval proves a stable, recurring model failure that survives fixes to task spec,
schema, prompt, examples, tooling, context, and verifier design. P13–P15 (LoRA candidate, shadow mode,
promotion gate) stay closed under this freeze. This milestone makes no claim that training has opened.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0 bytes
stderr). The document-flow locks pin, per sprint: the doc-flow API and commands, the verify-before-tracing
path through `CognitiveTrace::build` and `corpus_from_documents`, the path-validation checks, the ten
DOCFLOW-0 first-tests and the unit count, the seven-line DOCFLOW-0 boundary, and a binary smoke that runs the
whole flow against a real local document and refuses a tampered document / bundle file / trace and an
absolute / `..` / symlink-escape path (DOCFLOW-0); the manual's documentation of the four commands and the
read-not-trust statement, and an actual RUN of the operator smoke over the doc flow proving every tamper is
refused (DOCFLOW-1); and the scenario API, the proves-not-asserts observation, the ten DOCFLOW-2 first-tests
and the unit count raised to 100, the eight-line DOCFLOW-2 boundary, and a binary smoke that proves the
coverage from the matrix's own bytes, refuses a tampered pack via both verify and matrix, and refuses the
absolute / `..` / symlink-escape inputs END-TO-END through the binary (DOCFLOW-2). This milestone block
additionally pins the freeze record itself (this document's DOCFLOW-0..DOCFLOW-2 commit lineage, the
frozen-base tag and commit references, the nine boundary lines, and the `training_not_justified` verdict),
and guards against any milestone that falsely claims training has opened. The pinned commit hashes are
auditable against `git log`; this lock stays git-free and does NOT require the tag to exist — the tag is
created only after a clean tree and a green gate. The acceptance discipline for every sprint in this arc
was: rubric → green byte-silent `release_check` → live sabotage proving the gate catches a regression
(restored byte-identical by `cp`+`md5`, never `git checkout`) → an independent read-only adversarial verifier
panel with a fresh context → any residual folded before close.

## 7. Independent verification

Every sprint DOCFLOW-0 through DOCFLOW-2 was closed against read-only adversarial panels (Explore agents,
refute-by-default, scratch confined to a temp dir, each driving the compiled binary or inspecting the
artifacts), run until a fully-dry round with zero real findings. DOCFLOW-0 reached a dry first round.
DOCFLOW-1's panel raised one low finding — a gate pin that checked the smoke's setup rather than its doc-flow
run — folded by adding a section-unique load-bearing pin, then re-verified dry. DOCFLOW-2's panel raised one
high finding — the symlink-escape scenario observed only the pure containment decision while the gate had no
end-to-end symlink test — folded by adding an end-to-end input-safety smoke (absolute / `..` / a real
filesystem symlink each refused through the binary) and re-verified dry. Each gate lock was proven
load-bearing by live sabotage that failed the gate and was restored byte-identical; several sabotage attempts
tripped clippy's lints before the intended check and were re-run clippy-clean, which is recorded honestly.
Every claim in this document is checkable by running `scripts/release_check.sh` and reading the named commits.

## 8. Honest residuals (NOT closed in document-flow-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the
document-flow layer, not bugs.

1. **The flow reads one document, one question.** `doc-trace` reads the document's first span under a fixed
   title and a fixed question, producing one deterministic trace. Multi-span synthesis, operator-chosen
   questions, and multi-document corpora are future work, all of which must stay under the same
   read-not-trust and verify-before-tracing discipline.
2. **Integrity is byte-for-byte re-derivation, not a digest.** The load-bearing tamper check is re-deriving
   the artifact from the same document and byte-comparing within one deterministic build; cross-version
   reproduction and cryptographic digests are not claimed.
3. **Path safety is local-containment, not a sandbox.** The flow refuses absolute / `..` / working-directory-
   escaping paths and reads only a regular local file inside the working directory; it is not a general OS
   sandbox, and the gate proves the refusals end-to-end against real paths and a real symlink.
4. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline and the gate locks
   defend against off-wire tampering and accidental regression, both of which the gate provably catches. They
   do not defend against an insider with commit access who authors malicious code AND rewrites the gate in
   the same change — that is the domain of code review and the governance/signing layer.
5. **No model in the loop.** The document flow is deterministic; operator text is data, not a model. Any
   future model may only PROPOSE through the frozen hypothesis layer; it can never ground a claim, mutate
   memory, execute a probe, promote evidence, or self-authorize. The P10 adapter stays gated shut by P12.
6. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production
   reasoning system, and the document flow is described as such.
7. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left stray
   debris in the working tree despite their read-only instruction, and have occasionally inverted the
   finding-label semantics; each was caught and reconciled before close. It remains a known operational
   caveat of the panel method.

## 9. Frozen-status declaration

The DOCFLOW-0 → DOCFLOW-2 document-flow arc is **FROZEN at `document-flow-v0.1`**. The read-not-trust
boundary is the frozen surface:

```text
The document flow reads local input.
It does not trust local input.
Document scenarios vary the input.
They do not vary the authority.
Verification comes before tracing.
Nothing executes.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

Any change that lets local input become trusted authority; that lets a document become evidence; that
executes a probe, promotes an observation, mutates memory, or reopens training — must pass through the same
machinery: a rubric, a green byte-silent `release_check.sh`, a live sabotage, and an independent adversarial
panel, and must leave `training_justified=false` unless a clean recurring model failure is proven. Relaxing
any criterion requires explicit operator sign-off; it must not be edited mid-stream to make a failing check
pass. P13–P15 do not start under this freeze.
