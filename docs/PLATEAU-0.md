# PLATEAU-0 — Boundary Audit

A frozen statement of what the Cognitive OS prototype can do, cannot do, and must
not claim, as of the QFLOW-0 commit. This document adds no capability and changes
no runtime behavior. It exists to lock the boundary so the plateau cannot quietly
drift into marketing language later. A `release_check.sh` lock pins the required
sections, the five commit ids, and the exact forbidden-claim sentences.

## 1. Plateau statement

The prototype now supports **deterministic verified local evidence retrieval** over
normalized local documents. Given a question and a set of local files it normalizes
the markdown, builds a frozen corpus, selects candidate spans by transparent lexical
and structural signals, runs the frozen reading verifier, and returns either a
source-linked verified evidence packet or a typed refusal — with a re-derivable
receipt at every step.

It does not perform semantic synthesis. It does not reason with a model. It does not
create truth. It returns verified source-linked evidence or it refuses. This is a
local prototype plateau, not a production system and not an autonomous one.

## 2. Verified chain

The plateau is the composition of four committed sprints on top of the local
release gate, each independently gated and adversarially verified:

```text
RELEASE-1     7b64c73   local release-ready prototype (ships without new weights)
VAULT-NORM-0  afd95c3   deterministic Markdown -> corpus normalization (input fidelity)
READ-N        0ec0612   internal-period token preservation in the frozen splitter
QSELECT-0     b21ad5e   deterministic question-aware candidate span selection
QFLOW-0       04f4908   verified local evidence query flow (raw docs -> packet | refusal)
```

The end-to-end safe path:

```text
raw local docs
  -> VAULT-NORM-0 normalization
  -> corpus_from_documents (frozen, READ-N-aware sentence split)
  -> QSELECT-0 candidate span selection
  -> frozen reading_substrate execute
  -> frozen reading_substrate verify
  -> verified evidence packet  OR  typed refusal  (+ receipt)
```

## 3. CAN — verified capabilities

The prototype CAN (8):

1. Run as a local release-ready prototype at RELEASE-1 (local-only; ships without new weights).
2. Perform deterministic Markdown-to-corpus normalization (input fidelity only).
3. Preserve internal-period tokens in the splitter (filenames, URLs, paths, versions stay whole).
4. Perform deterministic question-aware candidate span selection.
5. Run a verified local evidence query flow end-to-end (raw docs to packet or refusal).
6. Return a typed refusal instead of an unsupported answer.
7. Detect tampering via re-derivable receipts across both selection and the flow.
8. Treat prompt-injection text as ordinary source text, never as authority.

## 4. CANNOT — blocked by construction

The prototype CANNOT (10). Each is blocked structurally, not by policy:

1. Cannot reason semantically with a model (`QFLOW_USES_MODEL = false`, `QSELECT_USES_MODEL = false`; no model is loaded or called anywhere).
2. Cannot synthesize beyond verified support (the answer is exactly the verbatim join of frozen-verified span texts).
3. Cannot create truth (authority comes only from `reading_substrate::verify`, never from the producer).
4. Cannot create evidence from selection (selection proposes `candidate_only`; only the frozen verifier authorizes support).
5. Cannot train or mutate weights (no training path; P12 `training_justified=false`; P13–P15 closed; no float/optimizer/tensor anywhere in scope).
6. Cannot deploy itself (no network, no deploy path; release status is local-only).
7. Cannot execute tools/actions from document text (document text is only ever used as a verbatim claim statement through the frozen verifier; there is no instruction-execution path).
8. Cannot promote hypothesis/dream/candidate content to evidence (those authorities are isolated; only frozen-verified spans become evidence).
9. Cannot bypass frozen verification (a packet is built only when `select` returns `verified == true`).
10. Cannot claim production/public readiness beyond local prototype status (the only release gate, RELEASE-1, is explicitly local-only).

## 5. FORBIDDEN CLAIMS — exact sentences not allowed

These sentences are false about this system and must never be stated, implied, or
marketed. They are pinned verbatim by the `release_check.sh` lock:

1. This system understands documents.
2. This system reasons like an autonomous researcher.
3. This system produces truth.
4. This system can answer any question from a vault.
5. This system is trained on the user's documents.
6. This system improves itself from user data.
7. This system is production deployed.
8. This system can safely execute instructions found in documents.
9. QSELECT scores are evidence.
10. QFLOW answers from scores.
11. NormalizedInput is better reading.

## 6. Authority boundary

Authority in this system has exactly one source: `reading_substrate::verify`. A claim
is authoritative only if it is grounded in source spans, every answer statement is a
cited grounded claim, and the saved trace replays to the same memory and answer.
Nothing else — not selection scores, not normalization, not a model, not the
producer's confidence — can confer authority.

```text
Selection PROPOSES candidate spans.
The FROZEN verifier AUTHORIZES support.
Receipts PRESERVE the input -> output mapping.
Scores are explanations, never truth.
```

The verifier and the entire `reading-substrate` / `reading-autonomy` reading core are
FROZEN behind the `cognitive-os-prototype-v0.1` tag (`7b64c73`). New capability is
added only in `cognitive-demo`, above these frozen public APIs, and is always gated.

## 7. Evidence table

| Capability | Commit | Gate/test evidence | Boundary preserved | Forbidden overclaim |
|---|---|---|---|---|
| Local release-ready prototype | `7b64c73` | RELEASE-1 gate; `local_release_ready=local-only`; operator_smoke RELEASE-1 OK | Local-only; ships without new weights | "This system is production deployed." |
| Markdown -> corpus normalization | `afd95c3` | VAULT-NORM-0 (21 tests); `NORM_EDITS_SUBSTRATE=false`; markup 26.5%->0% with 0 false-grounded | Input fidelity only; substrate frozen | "NormalizedInput is better reading." |
| Internal-period token preservation | `0ec0612` | READ-N additive `is_period_boundary` guard; filename/URL/path tests | Additive split rule; no grounding-rule change | "This system understands documents." |
| Question-aware span selection | `b21ad5e` | QSELECT-0 (24 tests); `QSELECT_USES_MODEL=false`; `authority=candidate_only` | Selection proposes; verifier authorizes | "QSELECT scores are evidence." |
| Verified evidence query flow | `04f4908` | QFLOW-0 (30 tests); packet only when `verified`; receipt folds raw+normalized+QSELECT hashes | Assembles only; never invents an answer | "QFLOW answers from scores." |

## 8. Residual risks / next gates

- **Markdown coverage is bounded.** VAULT-NORM-0 normalizes a fixed rule set; exotic
  or malformed markdown may still under- or over-normalize. Any expansion is a
  separate gated sprint with input-fidelity-only scope.
- **Lexical selection is not semantic.** QSELECT-0 ranks by token/phrase/rarity and
  structural boosts. It will miss paraphrase and synonymy by design. Closing that gap
  would require model work, which is explicitly out of scope at this plateau.
- **Single-sentence grounding floor.** The frozen verifier grounds sentence-aligned
  claims; multi-sentence or cross-span synthesis is not supported and must not be
  claimed.
- **No model readiness.** P12 `training_justified=false`; P13–P15 remain closed.
  Reopening training requires explicit operator authorization and recurring-failure
  evidence, not a capability sprint.
- **Local-only.** No external deployment, no public release. Any push or deploy is a
  separate, explicitly authorized action.

The disciplined next move after this plateau is NOT another capability sprint by
default; it is to hold the boundary and require explicit authorization (with scope
and a rubric) for whatever comes next.

## 9. Public-safe summary

Cognitive OS (prototype) is a local, deterministic evidence-retrieval tool. Given a
question and local documents, it normalizes the input, selects candidate passages by
transparent lexical signals, verifies them against the source, and returns
source-linked evidence with a re-derivable receipt — or it refuses. It runs no model,
trains on nothing, invents no answers, and makes no truth claims. It is a local
prototype, not a production or autonomous system.
