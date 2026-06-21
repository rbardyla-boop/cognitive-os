# Corpus Flow Milestone: CORPUS-0 → CORPUS-2 (FROZEN for corpus-flow-v0.1)

> Status: **FROZEN** as of `corpus-flow-v0.1`. This document freezes the CORPUS-0 through CORPUS-2
> multi-document local-corpus arc as a named, auditable milestone before any ranking, retrieval,
> summarization, or novelty/probe behavior is added. It is the single milestone-freeze record for the
> corpus-flow layer; the per-sprint decisions live in `docs/PROJECT_CHARTER.md` (`DD-2026-06-20-L`,
> `DD-2026-06-21-A`, `DD-2026-06-21-B`). This file freezes the arc, the commit lineage, the frozen base, the
> demonstrated capability, the read-not-trust boundary, the whole-corpus binding, the verification
> discipline, the training-gate verdict, the honest residuals, and the frozen-status declaration. It does not
> restate the per-sprint detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            corpus-flow-v0.1
points at      the CORPUS-3 freeze commit (this document + its gate lock)
freezes        the CORPUS-0..CORPUS-2 multi-document local-corpus arc (head e0791ed)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr)
recover        git checkout corpus-flow-v0.1
training gate  training_not_justified (P12 training_justified=false) — weights forbidden
frozen base    document-flow-v0.1 @ 0cc7399 (frozen)
deeper base    operator-controls-v0.1 @ 34b4f47, multi-trace-validation-v0.1 @ 460be0c,
               integration-demo-v0.1 @ 95b586d, hypothesis-track-v0.1 @ bb20acf,
               reading-track-v0.1 @ f6fa55a, cognitive-os-governance-v0.1 @ bbd1113 (all frozen)
```

The corpus-flow arc is the first arc to add a MULTI-DOCUMENT LOCAL CORPUS to the prototype. The document-flow
arc (frozen as `document-flow-v0.1`) let an operator point the system at ONE local text file and get a
verified-to-refused trace; the corpus flow points it at a small LOCAL DIRECTORY of `.txt` documents and runs
the SAME verified-to-refused trace, selecting and citing a source across the corpus. The crucial property is
not "it reads many files" — it is that **the corpus is read but never trusted**: the directory is enumerated
under a path/admission filter, grounded through the frozen reader into a verified reading receipt whose
structure hash binds the WHOLE corpus, traced, and still cannot become execution, evidence, promotion, memory
mutation, or training. Source selection is verified and replayable, never a model's semantic judgment. The arc
adds capability to the `cognitive-demo` integration layer ONLY; it edits no frozen crate source.

## 1. What is frozen — the commit lineage

Three commits form the arc. The hashes are auditable against `git log`.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| CORPUS-0 | Multi-Document Local Corpus Trace / Source-Selection Boundary: `corpus-trace` / `corpus-report` / `corpus-bundle` / `corpus-bundle-verify` trace a small LOCAL DIRECTORY of `.txt` documents through the SAME `CognitiveTrace::build` pipeline DOCFLOW-0 and the canonical demo use. The shell (`read_local_corpus`) enumerates the directory — path-validated (absolute / `..` / `~` refused), canonicalize-contained within the working dir, admitting ONLY non-hidden `.txt` files (the pure `corpus_admits_filename`), each canonicalize-contained so a symlink cannot escape, sorted for determinism. The library grounds the trace on the corpus's OWN first span via the frozen `corpus_from_documents`, fails closed with `EmptyCorpus` when nothing grounds, and records an unambiguous `corpus-source.json` (`document_index`, real `document_title` filename, `span_id`, `span_text`). The reading receipt's `structure_hash` binds EVERY document, so a tamper of any document — even a non-grounding one — re-derives a different trace and is refused. No model, no training, no new dependency, no new file, no frozen-crate edit | `b19dc47` |
| CORPUS-1 | Corpus Flow Operator Guard / Manual + Smoke Integration: `OPERATOR_MANUAL.md` (new §12) documents the four `corpus-*` commands, states the corpus is read but not trusted, that source selection is verified and replayable (never a semantic judgment by a model), and that the whole corpus is hash-bound (a side-document mutation cannot silently pass); `scripts/operator_smoke.sh` (new §11) runs the whole corpus flow end-to-end against a LOCAL directory of `.txt` documents, proving the directory filter matches CORPUS-0 (hidden / non-`.txt` / escaping symlink excluded) and that mutating the grounding OR a non-grounding side document, and tampering each bundle file or the standalone trace, are all refused. A documentation + drift-guard sprint — no code-crate behavior change | `ae58b99` |
| CORPUS-2 | Corpus Scenario Pack / Input-Integrity Matrix: `corpus-scenarios` / `corpus-scenario-pack` / `corpus-scenario-verify` / `corpus-scenario-matrix` run a finite, enum-backed set of thirteen VALID and INVALID corpus inputs (`enum CorpusScenario`) — a clean two-document corpus, empty, hidden-only, non-`.txt`-only, absolute / `..` / symlink-escape path, grounding-document mutation, non-grounding side-document mutation, and tampered source / trace / report / manifest — each OBSERVED by running the REAL CORPUS-0 admission filter / check / verifier and recording the outcome. Exactly one input verifies; the other twelve are each REFUSED. The matrix records all outcomes (verified 1, refused 12, 52/52 cells), the verified case's SOURCE IDENTITY (which document/span grounded the answer), and a `whole_corpus_bound` fact. No frozen crate source, no new dependency | `e0791ed` |

The corpus-flow head frozen here is `e0791ed` (CORPUS-2).

## 2. The frozen base

The corpus-flow arc is built ON TOP OF the `document-flow-v0.1` milestone — the prior frozen local-document
base — and the frozen tracks beneath it, and it edits NONE of their source. The honest, precise statement:
every prior tag still points where it did, and the FROZEN crate source — the reading substrate / CLI / codec,
the hypothesis layer, and the reading train-gate — is byte-for-byte identical to its tag.

```text
document-flow-v0.1            @ 0cc7399   (the prior frozen local-document base: one operator document, read not trusted)
operator-controls-v0.1        @ 34b4f47   (operator manual / smoke drift guard / release snapshot)
multi-trace-validation-v0.1   @ 460be0c   (scenario pack / coverage matrix / failure injection)
integration-demo-v0.1         @ 95b586d   (the cognitive-demo crate: trace / report / questions / bundle)
hypothesis-track-v0.1         @ bb20acf   (propose → probe → review → intent → observation → promotion-refusal)
reading-track-v0.1            @ f6fa55a   (the read0 verifier + the verified reading receipt)
cognitive-os-governance-v0.1  @ bbd1113   (the v0.1 governance / evidence-contract lineage)
```

Like the document-flow arc, the corpus-flow arc GROWS the `cognitive-demo` integration layer — that is where
the new capability lives — and touches no frozen crate: `git diff 0cc7399..e0791ed -- crates/reading-substrate
crates/reading-cli crates/reading-codec crates/hypothesis-layer crates/reading-train-gate` is empty, and no
`Cargo.toml` / `Cargo.lock` changed across the arc. The reading verifier, the hypothesis chain, and the
governance/evidence contract are unchanged; the corpus flow consumes them through their public APIs and adds
no authority of its own.

## 3. What the operator can now do (the demonstrated capability)

The prototype could already trace, report, interrogate, package, validate, and forge-then-reject ONE local
operator document (document-flow-v0.1). The corpus-flow arc adds a multi-document local corpus over that
frozen system:

1. **Trace a local corpus directory (CORPUS-0).** Point `corpus-trace` at a local directory of `.txt`
   documents and get the same end-to-end trace, grounded on the corpus's OWN first span and citing an
   unambiguous `corpus-source.json` (document index, real filename title, span id, span text). The corpus is
   enumerated under a path/admission filter, verified, and traced — and fails closed with `EmptyCorpus` if
   nothing grounds.
2. **Keep the corpus path documented and honest (CORPUS-1).** The operator manual documents the four
   `corpus-*` commands and the read-not-trust / verified-and-replayable / hash-bound-as-a-whole boundary; the
   operator smoke runs the whole corpus flow end-to-end against a local directory and fails closed if it
   drifts from the manual.
3. **Prove the flow holds across valid and invalid corpora (CORPUS-2).** A thirteen-scenario input-integrity
   pack and matrix prove a clean two-document corpus verifies while an empty, hidden-only, non-`.txt`-only,
   unsafe-path, escaping, mutated, or tampered corpus is refused — each outcome OBSERVED by running the real
   check, never asserted. The matrix additionally records the verified case's source identity and a
   `whole_corpus_bound` fact.

Every one of these reads local input and either verifies it through the frozen reader or fails closed. None of
them is authority, and none lets the corpus act.

## 4. The boundary that holds across the arc

These are the load-bearing invariants the whole arc preserves. None was weakened by a later sprint; each is
enforced by the release gate from the artifacts' own bytes.

1. **Read, never trust (CORPUS-0/1/2).** Operator-supplied corpus text is read and verified through the frozen
   reader before it is traced; it is never accepted as authority. A corpus becomes a verified read of its own
   documents or nothing at all.
2. **Source selection is verified and replayable, not model judgment (CORPUS-0/1/2).** The grounded source is
   the corpus's OWN first span, selected by the frozen `corpus_from_documents` and recorded in
   `corpus-source.json` (document index, real filename title, span id, span text). Selection is a verified,
   re-derivable fact — never a model's semantic judgment — and CORPUS-2's matrix carries the verified case's
   `source` so the selection is replayable.
3. **The whole corpus is hash-bound (CORPUS-0/1/2).** The reading receipt's `structure_hash` (carried in the
   trace as `reading_structure_hash`) binds EVERY document's title, spans, and sections. A mutation of ANY
   document re-derives a different trace and is refused — including a non-grounding "side" document. This is
   made visible, not merely asserted: in CORPUS-2 a grounding-document mutation fails on `corpus-source.json`
   (the attribution itself changed), while a **non-grounding side-document mutation leaves `corpus-source.json`
   byte-identical yet still fails on `trace.json`**, because the whole corpus is bound. A side document cannot
   silently pass; `corpus_whole_binding_holds` proves it structurally.
4. **Verify before tracing (CORPUS-0).** The trace starts from a frozen-VERIFIED read0 receipt over the
   corpus; if the corpus grounds nothing, the flow fails closed with `EmptyCorpus` and produces no trace.
5. **Re-derive, never trust the artifacts (CORPUS-0/1/2).** Every surface that accepts a file
   (`corpus-report`, `corpus-bundle-verify`, `corpus-scenario-verify`, `corpus-scenario-matrix`) re-derives
   from the SAME corpus/scenario set and byte-compares; a tampered corpus, source, trace, report, questions,
   manifest, pack, or matrix is refused. No record derives `Deserialize`.
6. **Path safety (CORPUS-0/2).** The corpus directory and its entries are validated: an absolute path, a `..`
   traversal, and a symlink that escapes the working directory are each refused, and only non-hidden `.txt`
   files are admitted — proven end-to-end through the binary, not only via the pure containment/admission
   decisions.
7. **Vary the input, not the authority (CORPUS-2).** Across all thirteen scenarios — valid and invalid — the
   no-authority boundary cells hold (no_execution / no_evidence / no_promotion / no_training). The strongest
   honest case is preserved from the frozen layers below: governance may APPROVE a probe, yet execution stays
   `requires_operator` (never `executed`), the observation stays `requires_review` / `observation_only` (never
   `recorded`), and the promotion request is `rejected`. Approval is not execution; an observation is not
   evidence.
8. **No model in the loop.** The corpus flow is fully deterministic; the operator's documents are data read
   through the frozen reader, not a model. Any future model could only PROPOSE through the frozen hypothesis
   layer; it can never ground a claim, select a source, mutate memory, execute a probe, promote evidence, or
   self-authorize.

## 5. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is `training_justified=false`).
The corpus-flow arc is orthogonal to P12 and does not move it: every CORPUS sprint reads the training decision
before and after building its artifacts and proves it identical, and the scenario matrix records the
no_training boundary cell true for every scenario. Weight training stays forbidden until the P11 eval proves a
stable, recurring model failure that survives fixes to task spec, schema, prompt, examples, tooling, context,
and verifier design. P13–P15 (LoRA candidate, shadow mode, promotion gate) stay closed under this freeze. This
milestone makes no claim that training has opened, that a corpus document becomes evidence, or that any probe
executes or promotes.

## 6. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes stdout, 0 bytes
stderr). The corpus-flow locks pin, per sprint: the corpus API and the four `corpus-*` commands, the
verify-before-tracing path through `CognitiveTrace::build` and `corpus_from_documents`, the path/admission
checks (`check_local_input_path`, `resolved_path_within`, `corpus_admits_filename`), the twelve CORPUS-0
first-tests and the unit count, the eight-line CORPUS-0 boundary, and a binary smoke that runs the whole flow
against a real local corpus and refuses a tampered corpus / bundle file / trace, an empty corpus, and an
absolute / `..` / symlink-escape path (CORPUS-0); the manual's documentation of the four commands and the
read-not-trust / verified-and-replayable / hash-bound statements, and an actual RUN of the operator smoke over
the corpus flow proving the filter matches and every grounding- and side-document tamper is refused (CORPUS-1);
and the scenario API, the proves-not-asserts observation, the twelve CORPUS-2 first-tests and the unit count
held at 124, the nine-line CORPUS-2 boundary, and a binary smoke that proves the coverage + source identity +
the whole-corpus-binding distinction (both rejection reasons) from the matrix's own bytes, refuses a tampered
pack via both verify and matrix, and refuses hidden-only / non-`.txt`-only corpora end-to-end through the
binary (CORPUS-2). This milestone block additionally pins the freeze record itself (this document's
CORPUS-0..CORPUS-2 commit lineage, the frozen-base tag and commit references, the ten boundary lines, the
matrix source identity, and the whole-corpus-binding / non-grounding-side-document-mutation behavior, and the
`training_not_justified` verdict), and guards against any milestone that falsely claims training has opened. The
pinned commit hashes are auditable against `git log`; this lock stays git-free and does NOT require the tag to
exist — the tag is created only after a clean tree and a green gate. The acceptance discipline for every sprint
in this arc was: rubric → green byte-silent `release_check` → live sabotage proving the gate catches a
regression (restored byte-identical by `cp`+`md5`, never `git checkout`) → an independent read-only adversarial
verifier panel with a fresh context → any residual folded before close.

## 7. Independent verification

Every sprint CORPUS-0 through CORPUS-2 was closed against read-only adversarial panels (Explore agents,
refute-by-default, scratch confined to a temp dir, each driving the compiled binary or inspecting the
artifacts), run until a fully-dry round with zero real findings. CORPUS-0's panel returned zero real findings
fully dry (one sabotage probe was caught SOLELY by the binary smoke, proving the smoke independently
load-bearing). CORPUS-1's and CORPUS-2's panels returned fully dry with no working-tree debris. This CORPUS-3
freeze adds no behavior; it is verified by a green byte-silent gate, live sabotage of the milestone lock
(restored byte-identical via `cp`+`md5`, never `git checkout`), and an independent read-only adversarial panel.
Each gate lock across the arc was proven load-bearing by live sabotage that failed the gate and was restored
byte-identical. Every claim in this document is checkable by running `scripts/release_check.sh` and reading the
named commits.

## 8. Honest residuals (NOT closed in corpus-flow-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known edge of the corpus-flow
layer, not bugs.

1. **The flow grounds on the corpus's first span, one question.** `corpus-trace` grounds on the corpus's OWN
   first span (the first admitted document in deterministic sort order) under a fixed question, producing one
   deterministic trace and citing one source. Multi-span synthesis, ranking or retrieval across documents,
   operator-chosen questions, and summarization are future work, all of which must stay under the same
   read-not-trust, verified-and-replayable source selection, and whole-corpus-binding discipline.
2. **Source selection is first-span deterministic, not relevance ranking.** The grounded source is selected by
   the frozen reader's deterministic ordering, recorded and replayable; it is NOT a relevance score or a
   semantic match, and the milestone claims none. Any future ranking must be a verified, re-derivable function,
   never a model's ungoverned judgment.
3. **Integrity is byte-for-byte re-derivation, not a digest.** The load-bearing tamper check is re-deriving the
   artifact from the same corpus and byte-comparing within one deterministic build; cross-version reproduction
   and cryptographic digests are not claimed.
4. **Path safety is local-containment, not a sandbox.** The flow refuses absolute / `..` / working-directory-
   escaping paths and reads only regular non-hidden `.txt` files inside the working directory; it is not a
   general OS sandbox, and the gate proves the refusals end-to-end against real paths and a real symlink.
5. **Multi-file insider forgery is out of scope.** The re-derive-not-trust discipline and the gate locks defend
   against off-wire tampering and accidental regression, both of which the gate provably catches. They do not
   defend against an insider with commit access who authors malicious code AND rewrites the gate in the same
   change — that is the domain of code review and the governance/signing layer.
6. **No model in the loop.** The corpus flow is deterministic; operator documents are data, not a model. Any
   future model may only PROPOSE through the frozen hypothesis layer; it can never ground a claim, select a
   source, mutate memory, execute a probe, promote evidence, or self-authorize. The P10 adapter stays gated
   shut by P12.
7. **Prototype, not production.** This is a deterministic Rust prototype and testbed, not a production
   reasoning system, and the corpus flow is described as such.
8. **Process caveat (verification method).** The read-only adversarial panels have on prior tracks left stray
   debris in the working tree despite their read-only instruction, and have occasionally inverted the
   finding-label semantics; each was caught and reconciled before close. It remains a known operational caveat
   of the panel method.

## 9. Frozen-status declaration

The CORPUS-0 → CORPUS-2 multi-document local-corpus arc is **FROZEN at `corpus-flow-v0.1`**. The read-not-trust
boundary is the frozen surface:

```text
The corpus flow reads local documents.
It does not trust local documents.
Source selection is verified and replayable.
The whole corpus is hash-bound.
Corpus scenarios vary the input.
They do not vary the authority.
Nothing executes.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

Any change that lets corpus text become trusted authority; that makes source selection a model's semantic
judgment; that lets a corpus document become evidence; that executes a probe, promotes an observation, mutates
memory, or reopens training — must pass through the same machinery: a rubric, a green byte-silent
`release_check.sh`, a live sabotage, and an independent adversarial panel, and must leave
`training_justified=false` unless a clean recurring model failure is proven. Relaxing any criterion requires
explicit operator sign-off; it must not be edited mid-stream to make a failing check pass. P13–P15 do not start
under this freeze.
