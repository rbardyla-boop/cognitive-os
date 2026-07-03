# PLATEAU-1 — Learning Loop Boundary Lock

A frozen statement of what the Cognitive OS prototype can do, cannot do, and must
not claim, as of the SESSION-LOOP-0 commit. This document adds no capability and
changes no runtime behavior. It exists to lock the learning-loop boundary — the
crossing from isolated verified organs into one composed deterministic learning
session — so the plateau cannot quietly drift into companion-app or
understanding-engine language later. A `release_check.sh` lock pins the required
sections, the eight commit ids, and the exact forbidden-claim sentences.

## 1. Plateau statement

The prototype now supports **one deterministic receipt-linked learning session**
over verified local evidence. Given a question, local documents, and the learner's
explicit observations, it retrieves verified evidence, maps bounded literature
intent, builds a supported lesson, records exact-match quiz results, projects a
bounded learner-state receipt, produces a receipt-linked memory candidate, and —
only on explicit scope-bound consent — appends a pointer entry to a
**consented append-only pointer journal**, with a re-derivable receipt at every
step and a typed refusal at every unsupported one.

The session composer adds no authority of its own: it does not score, rank, grade,
select, or verify content itself; every step carries the frozen organ's own receipt
and authority. It does not reason with a model. It does not know the user, does not
profile the user, does not remember anything on its own initiative, and does not
adapt across sessions. Wrong quiz answers are recorded outcomes, not judgments of
the learner. Memory is pointer receipts only, written solely through consented
appends. This is a local prototype plateau: a verified learning loop demo, not a
companion, not a tutor product, and not an autonomous system.

## 2. Verified chain since PLATEAU-0

The learning loop is the composition of seven committed sprints (plus one
hardening sprint) on top of the PLATEAU-0 evidence boundary, each independently
gated, sabotaged, and adversarially verified:

```text
PLATEAU-0                 e44bbf0   verified local evidence boundary lock (docs-only)
LIT-INTENT-0              6b434d5   bounded literature intent map from verified QFLOW spans
TEACH-0                   99106f2   supported lesson from span-backed intent findings
LEARNER-MODEL-0           fe56822   bounded learner-state receipt map (exact-match quiz law)
TAMPER-DEBRIS-CLEANUP-0   d55abbf   serialized tamper refusals made non-vacuous (hardening)
LEARNER-MEMORY-0          668d0ce   receipt-linked memory candidate (candidate only, no store)
LEARNER-MEMORY-1          fe58734   consented append-only pointer journal (chain-verified)
SESSION-LOOP-0            3b103a4   receipt-linked learning session composing all of the above
```

The end-to-end safe path:

```text
question + local docs + explicit learner observations
  -> verified evidence (QFLOW, inside LIT-INTENT)
  -> bounded literature intent map
  -> supported lesson
  -> exact-match quiz result
  -> learner-state receipt
  -> memory candidate (candidate_only)
  -> consented journal append (pointer entries, chain-verified before write)
  -> ONE receipt-linked session artifact  OR  a typed refusal  (+ receipt)
```

## 3. CAN — verified learning-loop capabilities

The prototype CAN (10):

1. Map bounded literature intent from verified evidence spans (`6b434d5`; lexical and structural signals only, every finding span-backed).
2. Build a supported lesson from span-backed findings (`99106f2`; every lesson item carries intent-authority support).
3. Record exact-match quiz results (`fe56822`; correct, incorrect, or unanswered — an incorrect answer completes and is recorded, never judged further).
4. Project a bounded learner-state receipt (`fe56822`; seen items, taught concept, quiz summary, explicit misconception flags, self-reported confidence, non-adaptive review pointer).
5. Produce a receipt-linked memory candidate (`668d0ce`; every item points to explicit learner-state fields and all four source receipt hashes, or the whole candidate refuses).
6. Append to a consented append-only pointer journal (`fe58734`; scope-bound explicit consent per append, pinned canonical operator, pointer entries only).
7. Run the full receipt-linked learning session end-to-end (`3b103a4`; one artifact composing all six organs with no new authority).
8. Verify the journal chain before any write (`fe58734`; per-entry recompute, seq monotonicity and continuity, prev-pointer linkage, duplicate detection, root check — each violation a distinct refusal; the on-disk journal is byte-verified against a re-derived canonical state, never parsed).
9. Return a typed refusal at every unsupported step (`3b103a4` and all composed organs; 16 session refusal variants plus each organ's own, all constructed in production paths).
10. Detect serialized tamper on session artifacts (`3b103a4`, hardened by `d55abbf`; byte-flip on any demo or matrix artifact is detected by re-derive-and-byte-compare replay).

## 4. CANNOT — blocked by construction

The prototype CANNOT (12). Each is blocked structurally, not by policy:

1. Cannot claim semantic understanding (selection and intent mapping are lexical/structural; the quiz law is exact string match; no model is loaded or called anywhere in scope).
2. Cannot infer hidden author motives (intent findings are verbatim span-backed structure; the demo lesson's own misconception check marks hidden-motive inference as the error).
3. Cannot teach unsupported content (every lesson item requires span-backed findings carrying `intent_map_from_verified_span` authority; unsupported lesson support is a refusal).
4. Cannot grade free-form answers (`QuizOutcome` is `CorrectExactMatch`, `IncorrectExactMismatch`, or `Unanswered` — there is no scoring, rubric, or partial-credit path).
5. Cannot personalize generation (`PERSONALIZES = false` structural consts in the teach, learner-model, memory, journal, and session organs; the intent layer runs no model at all, `LIT_INTENT_USES_MODEL = false`; a personalization signal is a refusal before any work happens).
6. Cannot autonomously recall memories (`AUTONOMOUSLY_RECALLS = false`; no recall path exists — the journal is a pointer log read only through operator verify verbs).
7. Cannot adapt behavior across sessions yet (the session takes its starting journal as an input value and mutates nothing; the review pointer is non-adaptive; cross-session behavior is a future, separately gated sprint).
8. Cannot write memory without explicit journal consent (append requires a scope-bound consent naming the exact candidate receipt; a missing consent is the session's own distinct refusal before the journal fold ever runs).
9. Cannot store rich personal memory content (journal entries are pointer receipts — hashes, counts, consent fields; a test pins that no memory item content serializes into the journal).
10. Cannot infer traits, health profile, psychology, identity, or diagnosis (`Infers* = false` consts; each such signal is a distinct refusal; confidence is self-reported with `inferred = false`).
11. Cannot train or mutate weights (no training path; P12 `training_justified=false`; P13–P15 closed; no float/optimizer/tensor anywhere in scope).
12. Cannot deploy itself (no network, no deploy path; release status is local-only at RELEASE-1).

## 5. FORBIDDEN CLAIMS — exact sentences not allowed

These sentences are false about this system and must never be stated, implied, or
marketed. They are pinned verbatim by the `release_check.sh` lock:

1. This system understands literature.
2. This system knows the user.
3. This system is a personalized AI companion.
4. This system remembers the user autonomously.
5. This system adapts itself across sessions.
6. This system can grade free-form answers.
7. This system can infer the user's psychology.
8. This system diagnoses the user.
9. This system stores rich personal memory.
10. This system writes memory without consent.
11. This system trains on the user's learning history.
12. This system creates truth.

## 6. Authority and consent boundary

Authority in this system still has exactly one source: `reading_substrate::verify`.
Everything above it is a bounded projection that carries — never creates —
authority:

```text
The FROZEN verifier AUTHORIZES evidence spans.
Intent maps PROJECT verified spans        (intent_map_from_verified_span).
Lessons PROJECT span-backed findings      (teach_from_span_backed_intent_map).
Learner state PROJECTS the taught lesson  (learner_state_from_supported_teach_map).
Memory candidates PROJECT learner state   (memory_candidate_from_learner_state).
The SESSION COMPOSES; it authorizes nothing.
CONSENT — explicit and scope-bound — is the only gate to durable memory.
```

Consent is an explicit affirmation naming the exact memory-candidate receipt it
authorizes (`learner_memory_receipt:{hash}`). One consent cannot be replayed
against a different candidate. The canonical journal pins the consent operator so
every derivation stays deterministic; the live append verb must re-affirm it
exactly, treats the on-disk journal as untrusted bytes, and writes nothing on any
refusal. There is no other path to persistence.

The verifier and the entire reading core remain FROZEN behind the
`cognitive-os-prototype-v0.1` tag (`7b64c73`). All eight plateau commits live in
`cognitive-demo`, above these frozen public APIs, and every one was gated.

## 7. Evidence table

| Capability | Commit | Gate/test evidence | Boundary preserved | Forbidden overclaim |
|---|---|---|---|---|
| Evidence boundary lock (PLATEAU-0) | `e44bbf0` | docs-only lock in `release_check.sh`; 3 overclaim sabotage RED; 2 read-only lenses PASS | No runtime change; claims pinned | "This system creates truth." |
| Bounded literature intent map | `6b434d5` | LIT-INTENT-0 (14 tests); span-backed findings; QFLOW refusal propagates | Lexical structure only; no model | "This system understands literature." |
| Supported lesson | `99106f2` | TEACH-0 (14 tests); every item span-backed; unsupported support refused | Teaches only verified content | "This system understands literature." |
| Learner-state receipt (exact-match quiz) | `fe56822` | LEARNER-MODEL-0 (18 tests); `LEARNER_MODEL_USES_MODEL=false`; non-adaptive review pointer | Projection, not a profile; grading is exact match | "This system can grade free-form answers." |
| Non-vacuous tamper refusals | `d55abbf` | TAMPER-DEBRIS-CLEANUP-0 (+2 tests); byte-flip constructs each `Serialized*Tamper` | Replay trust made testable | "This system creates truth." |
| Receipt-linked memory candidate | `668d0ce` | LEARNER-MEMORY-0 (18 tests); pointer-law guard; 4-hash source spine | Candidate only; no store, no profile | "This system knows the user." |
| Consented append-only pointer journal | `fe58734` | LEARNER-MEMORY-1 (25 tests); chain guard before append; content-free entries test-pinned | Pointer receipts only; consent per append | "This system stores rich personal memory." |
| Receipt-linked learning session | `3b103a4` | SESSION-LOOP-0 (24 tests); 16 refusals constructed; cross-organ receipt-anchor proof | Composes only; adds no authority | "This system is a personalized AI companion." |

## 8. Residual risks / next gates

- **Lexical teaching is not semantic teaching.** Intent maps and lessons rank and
  structure by token, phrase, and position. Paraphrase and synonymy are missed by
  design; closing that gap would require model work, which is out of scope at this
  plateau and gated by P12.
- **Exact-match quizzes are brittle by design.** A correct idea worded differently
  is recorded as a mismatch. That honesty is load-bearing: relaxing it means
  free-form grading, which is a forbidden capability at this plateau.
- **The canonical loop runs on fixture content.** The demo journal is finite (two
  canonical candidates) and the session demo composes the canonical fixture chain.
  Real content variety requires future corpus-flow gates, not a quiet widening.
- **Single-session boundary.** Session N+1 does not yet start from session N's
  journal. Cross-session growth is the named next gate (MULTI-SESSION-0, MAP
  only) and must arrive with its own consent and boundary analysis.
- **Memory stays pointer-only.** Any move toward storing content, preferences, or
  history is a new boundary crossing requiring its own gate, consent law, and
  forbidden-claims review — not an extension of LEARNER-MEMORY-1.
- **No model readiness.** P12 `training_justified=false`; P13–P15 remain closed.

The disciplined next move after this plateau is NOT another capability sprint by
default; it is to hold the boundary and require explicit authorization (with scope
and a rubric) for whatever comes next.

## 9. Public-safe summary

Cognitive OS (prototype) now includes a local, deterministic learning-loop demo.
Given a question, local documents, and the learner's explicit answers, it retrieves
verified evidence, builds a source-linked lesson, records exact-match quiz results,
and — only with explicit consent — appends a pointer receipt of the session to an
append-only journal. Every step is replay-verifiable and every unsupported step is
a typed refusal. It runs no model, grades nothing beyond exact match, stores no
personal content, remembers nothing on its own, adapts nothing across sessions, and
makes no truth claims. It is a local prototype of a verified learning loop, not a
companion, tutor product, or autonomous system.
