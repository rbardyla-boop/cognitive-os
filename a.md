# Cognitive OS v1.0 Remaining Sprint Plan

This file tracks the remaining build and test sprints from the current Sprint 20 state to Cognitive OS v1.0.

## Direction update (2026-06-13): the Caitlin Leap supersedes the incremental grind

Sprints 22 and 23 are green. Before building the incremental Sprints 24–35 one at a
time, we applied the move `COGNITIVE_OS_SELF_CORRECTING_LEAP.md` argues for: instead of
adding more scenarios, packet types, and review documents (the "collider" approach),
fold the development process itself into the already-proven runtime machinery — the
Cognitive Bus, Verifier, Epistemic Licenses, ContradictionPackets, Mutation Gateway,
Trace Audit, and deferred correction loop.

Sprint 24 below delivers that unification. A design proposal that would weaken a locked
invariant is now blocked by a `hazard_only` ContradictionPacket, denied consolidation
through the same mutation gateway as runtime memory, and routed into a deferred
`design_revalidation` job — exactly as Bridge A is blocked under hazard evidence. The
project audits its own design-decision chain (`decision_audit.py --project --strict`)
and gates its own release as a verified cognitive action. `./scripts/release_check.sh`
is green and silent.

The incremental Sprints 24i–35 below are retained as the backlog they always were, but
they are now the *verification surface* of one governance loop rather than twelve
separate immune systems. Any future proposal to add them must enter as a design
proposal and pass through this gate (per `SPRINT_24_PLAN.md`).

## Direction update (2026-06-14): prototype-first engine track, additive to the backlog

Sprints 24–32 froze the **governance / evidence layer** (v0.1, `GOVERNANCE_MILESTONE.md`).
In the layer model of [ADR-002](ADR-002-runtime-engine-replay-contract.md) that lineage is
**L3** — the content-bound, signed, mechanism-bound replay-evidence contract. What it secures,
the deterministic runtime engine itself (**L0–L2**: the kernel, ingress/scheduling/frames, and
run/record/replay), is still realized as Python scripts rather than a self-contained engine with
a pinned replay contract.

The forward direction is therefore **prototype-first**: build the minimal deterministic engine
ADR-002 charters (the `project_birth.md` async-bus model made replayable), then add a
**replaceable LLM language codec** at the human-language boundary — never inside the kernel.
This is the **Prototype-First Track (P0–P15)** added below.

This track is **additive**: the incremental 24i–35 backlog above remains the deferred
Python-cognition backlog, still gated by the unified self-correction loop. P0–P15 is the active
build order; ADR-002 records the layer contract it builds against. No engine code lands in this
planning pass — the section below is the spec the P-sprints execute.

## Progress

- [x] Write remaining v1.0 sprint plan into `a.md`.
- [x] Sprint 20R — Signed Replay Identity Review Pass.
- [x] Sprint 21 — Asymmetric Replay Identity.
- [x] Sprint 22 — Raw Experience Ingestion Kernel.
- [x] Sprint 23 — Semantic Candidate Extraction.
- [x] Sprint 24 — Unified Self-Correction (the Caitlin Leap). _Delivered; supersedes incremental 24–35 as separate immune systems._
- [x] Sprint 25 — Derived Effect Classification. _Delivered; closes the Sprint 24 effect-mislabel residual._
- [x] Sprint 26 — Trace-Grounded Invariant Diff. _Delivered; closes the Sprint 25 lexical-laundering residual by deriving effect from behavior, not words._
- [x] Sprint 27 — Complete Locked-Invariant Probe Coverage. _Delivered; every locked invariant is probe-backed and an unprobed locked invariant cannot reach accept — closes the Sprint 26 unprobed-invariant residual._
- [x] Sprint 28 — Delta-to-Code Provenance. _Delivered; the tested delta is derived from a provenance-verified change_set bound to the real changed artifact — closes the Sprint 27 mis-stated-delta residual._
- [x] Sprint 29 — Artifact Content-Hash Binding. _Delivered; the tested delta binds to the literal before/after content of a real on-disk policy artifact — closes the Sprint 28 structured-patch-vs-content residual._
- [x] Sprint 30 — Signed Change Provenance. _Delivered; a content-bound change to a locked invariant requires an authorized Ed25519 signature over the content digest, and authorization never overrides a trace block — closes the Sprint 29 who-authorized-it residual._
- [x] Sprint 31 — Signer-Set Governance. _Delivered; the signer registry is a governed object (scope + lifecycle), authority is evaluated at the decision tick, and a revoked/expired/out-of-scope signer cannot authorize — closes the Sprint 30 flat-key-list residual. Mechanism-source binding remains the next boundary._
- [x] Sprint 32 — Mechanism-Source Content Binding. _Delivered; the enforcement code itself is content-hash bound in a verified manifest, a mechanism-source change needs signed provenance, and a gate-code weakening is caught by probe even with a clean signed policy — closes the Sprint 29/30 "policy bound, not the mechanism" residual._
- [ ] Sprint 24i — Trust and Provenance Assignment. _(backlog, now gated by the unified loop)_
- [ ] Sprint 25i — Multi-Index Memory Layer. _(backlog, now gated by the unified loop)_
- [ ] Sprint 26i — Retrieval Under Task Pressure. _(backlog, now gated by the unified loop)_
- [ ] Sprint 27i — Outcome Testing Harness. _(backlog, now gated by the unified loop)_
- [ ] Sprint 28i — Evidence-Only Revision Pipeline. _(backlog, now gated by the unified loop)_
- [ ] Sprint 29i — Stable Consolidation. _(backlog, now gated by the unified loop)_
- [ ] Sprint 30i — Staleness, Demotion, and Forgetting. _(backlog, now gated by the unified loop)_
- [ ] Sprint 31i — LLM Linguistic Operating Layer Boundary. _(backlog, now gated by the unified loop)_
- [ ] Sprint 32i — Full Lifecycle End-to-End Scenario. _(backlog, now gated by the unified loop)_
- [ ] Sprint 33 — Runtime Packaging and Deployable Local Service. _(backlog, now gated by the unified loop)_
- [ ] Sprint 34 — Security and Boundary Red-Team. _(backlog, now gated by the unified loop)_
- [ ] Sprint 35 — v1.0 Release Candidate. _(backlog, now gated by the unified loop)_

Prototype-First Track (ADR-002 deterministic engine, then replaceable LLM codec):

- [ ] P0 — Tag/snapshot the frozen v0.1 governance milestone (recoverable before engine work).
- [x] P1 — Rust workspace skeleton + deterministic kernel boundary (`crates/vibe-core`). _Delivered 2026-06-14; 8 cargo tests green, release_check gates the L0 kernel boundary._
- [x] P2 — ObservationEnvelope + IngressGate. _Delivered 2026-06-14; `crates/vibe-ingress`, 6 cargo tests green, admission-only (no tick eval, no EngineState), release_check gates the L1 boundary._
- [x] P3 — TickScheduler + ScheduledObservation. _Delivered 2026-06-14; `crates/vibe-scheduler`, 7 cargo tests green, scheduling-only (deterministic target ticks, bounded horizon, overload→receipt, idempotent), release_check gates the L1 boundary._
- [x] P4 — FrameCollector + ObservationFrame. _Delivered 2026-06-14; `crates/vibe-frame`, 8 cargo tests green, canonical hash-stable frame (frame-only), release_check gates the L1 boundary; passed a fresh-context adversarial panel (0 confirmed rubric defects; 2 surfaced coverage gaps closed)._
- [x] P5 — Minimal VibeEngine evaluation loop. _Delivered 2026-06-14; canonical `ObservationFrame` promoted into vibe-core (L0), P1 stub retired, `evaluate_tick` folds the frame + emits `EngineOutput` with explicit `StateTransition` + `output_hash`. One frame definition. 12 vibe-core + 9 vibe-frame tests green; passed a fresh-context adversarial panel (0 confirmed defects)._
- [x] P6 — RunScript + RunRecorder + deterministic replay. _Delivered 2026-06-15; `crates/vibe-run` (L2) records the full pipeline + replays from recorded frames alone, detecting tampering. 8 cargo tests green; passed a fresh-context adversarial panel (0 confirmed defects; closed a surfaced tick-label internal-consistency gap)._
- [x] P7 — Local CLI prototype (`vibe run` / `vibe replay` / `vibe verify`). _Delivered 2026-06-15; `crates/vibe-cli` (the `vibe` binary), serde confined to the CLI, 5 cargo tests + end-to-end binary smoke (run→replay MATCH→verify OK→tamper exit 1)._
- [x] P8 — Prototype release gate (Rust tests + replay determinism + governance checks + no-secrets). _Delivered 2026-06-15; `release_check.sh` consolidates P1–P7 + Python governance + an end-to-end `vibe` binary smoke (replay determinism through the recorded-run path) + a no-secrets scan; green+silent; 3 sabotage probes (broken verify / serde-in-core / secret fixture) each fail it. No engine behavior added._
- [x] P9 — Language-codec boundary (LLM proposes typed packets; cannot mutate state). _Delivered 2026-06-15; `crates/reading-codec` — an untrained, deterministic codec on top of READ-0: untrusted model text → typed reading actions; prose/malformed/unknown/under-specified output rejected (never repaired); accepted actions execute ONLY through `reading_substrate::execute`; an answer finalizes ONLY if `reading_substrate::verify` approves it. 11-fixture eval harness (runnable `eval_report` example) scores valid/invalid/injection outputs; 14 codec + 11 substrate cargo tests incl. 3 codec sabotage probes (disable unknown-action rejection / source-span requirement / verify-finalize gate → eval fails). No trained weights, no live model. Depends on reading-substrate, no vibe-* dep; serde stays out of engine/substrate cores. **Folds in READ-1 claim-fidelity hardening** (an ultracode panel found READ-0 grounding was structural-only): a claim is grounded only if its statement is a literal substring of its cited span TEXT (deterministic floor, no paraphrase/LLM) — so a fabricated claim citing a real read span no longer finalizes. release_check gates it; P8 still green; vibe engine + CLI 0-diff._
- [x] P10 — Baseline off-the-shelf local LLM adapter (zero training). _Delivered 2026-06-15; `crates/reading-adapter` — a REPLACEABLE `ModelBackend` boundary: a backend produces untrusted reading-action text routed ONLY through `reading_codec::decode` (validate → substrate execute → READ-1 verifier finalize); the adapter holds no authority, mutates no memory, cannot finalize without verification. Default `ScriptedBackend` is deterministic; optional `local-model` feature adds a real local model via subprocess (`std::process`), OFF by default, never run by the gate (only compiled+linted). Baseline failure-profile eval (`baseline_report` + runnable example): 1 finalized / 7, 5 rejected across all classes incl. a fabricated-but-cited claim → Unverified. 7 cargo tests. release_check gates it (test+fmt+clippy ×2 features + runnable eval + no-executor-call scan + purity + feature-gate + no-ML-dep + separation); live bypass sabotage (adapter calls substrate executor directly) → gate fails, restored. P8/P9/READ-0/READ-1 still green; vibe + codec + substrate 0-diff. Hard rule honored: a model backend, not a smarter authority._
- [x] P11 — LLM codec eval harness (30–100 cases; model cannot self-grade). _Delivered 2026-06-15; `crates/reading-eval` — 37 committed fixtures (raw untrusted proposal text + a COMMITTED expected outcome) across all 10 categories (valid action, correct finalization, malformed JSON, unknown action, missing fields, bad span, ungrounded claim, fabricated cited claim, premature synthesize, prompt injection), scored through the P10 adapter → codec → READ-1/READ-2 verifier. The scorer compares the codec's actual decision to the committed label (never the model's prose); false-accepts (should-reject-but-accepted — the unsafe class) are surfaced as an explicit list and the report carries score + false-accepts + classified false-rejects + failure-category histogram + a deterministic `next_change`. **Folds in READ-2 sentence-fidelity** (a panel found the prior literal-substring floor finalized false answers built from verbatim sub-fragments): a claim must now be a complete sentence-level unit of a cited span, killing the fragment/composition false-accept class. Battery: **37/37 correct, 0 false-accepts, 0 false-rejects** (incl. the new fragment/composition cases). 9 tests incl. controls proving the scorer uses committed labels. release_check gates it (test+fmt+clippy + runnable example enforcing ≥30 + 0-false-accepts + source-count + purity + separation); live sabotage (disable the sentence-boundary check) reintroduces 2 false-accepts → gate exit 101. No model, no training. P8/P9/P10/READ-0/READ-1 green; vibe engine + CLI 0-diff._
- [x] P12 — Training-justification gate. _Delivered 2026-06-15; `crates/reading-train-gate` — a deterministic, machine-checkable gate that BLOCKS weight training unless a clean, recurring model failure survives cleanup of every fixable cause (bad fixture, schema, prompt, tooling, missing context, verifier weakness). `decide(false_accept_ids, diagnoses) -> TrainingDecision{training_justified, safety_fix_required, cited_failures, blockers, reason}`; the load-bearing bit is the bool, not prose. Doctrine: no failed cases → no training; any false-accept → a verifier/safety fix (never training); any defect-caused failure → no training; only a `CleanModelFailure` that survived cleanup AND recurs (≥2) can justify weights, and even one remaining defect blocks. On the live P11 battery (0 false-accepts, 0 residual) the decision is **training_justified=false** ("no unresolved failures"). 12 tests (the 6 first-tests + recurrence/cleanup/mixed-defect/determinism). release_check gates it (test+fmt+clippy + runnable decision example + purity + no-ML-dep + separation); live sabotage (ignore blockers) → doctrine test + gate exit 101. No model, no training. All prior crates 0-diff._
- [ ] P13 — Local LoRA/adapter candidate (only if justified).
- [ ] P14 — Shadow-mode insertion.
- [ ] P15 — Promotion / rejection gate.

Reading Substrate Track (separate track; runnable after P7/P8 — needs run/replay, not trained weights — bridges to the P9–P15 LLM track):

- [x] READ-0 — External Text Reading Trace (verified, replayable answer from source-linked structured memory; no training). _Delivered 2026-06-15; `crates/reading-substrate` (zero-dep, no vibe deps), scripted deterministic reader, 9 tests incl. 3 sabotage probes; release_check gates it; P8 still green._
- [x] READ-1 — Claim fidelity (literal cited-span support). _Folded into P9 (`e4ccb6e`); a claim is grounded only if its statement is found in its cited span text._
- [x] READ-2 — Sentence-level fidelity floor. _Folded into P11 (`4b4aef5`); a claim must be a complete sentence-level unit of a single cited span (kills fragment/composition false-accepts)._
- [x] READ-3 — Real Corpus Reading CLI. _Delivered 2026-06-15; `crates/reading-cli` binary `read0`: load a real folder of `.txt` documents → corpus of one sentence per span (shared splitter), run an untrusted reading plan ONLY through `reading_codec::decode` → replayable run + proof + verifier receipt; `verify`/`replay` re-derive and reject tamper. Reads confined to the folder; plan never reaches memory except via the codec; no training. 9 + 3 substrate-shared tests; release_check gates it (build + run/verify/replay smoke + fabricated-rejected + tamper-fails + codec-only + separation); all prior crates 0-diff (substrate gained only the shared `split_sentences`)._
- [x] READ-4 — Real Corpus Eval Pack. _Delivered 2026-06-15; `crates/reading-corpus-eval` — committed real-corpus fixtures (docs + question + plan + COMMITTED expected verifier result) across weather/medical/infrastructure/finance/safety, each driven through the REAL read0 run → verify → replay path. A false-grounded answer (an expected-rejected fixture that finalizes a verified answer) is the unsafe class, surfaced explicitly, 0 required. Report carries per-fixture pass/fail + rejection reason + trace hash. 15/15 correct, 0 false-grounded, 0 false-rejects. 7 tests incl. a control proving labels are committed (a valid plan labelled reject → flagged false-grounded). release_check gates it (test + fmt + clippy + runnable pack [≥10 + 0-false-grounded] + source-count + separation); live sabotage (hide false-grounded) → gate exit 101. No model, no training (anecdotes never justify weights — P12 decides that). All prior crates 0-diff at delivery._
- [x] READ-5 — Deterministic Sentence Splitter Hardening. _Delivered 2026-06-15; hardened the shared `split_sentences` in `reading-substrate` so abbreviations (Dr./Mr./U.S./e.g./i.e.), decimals (3.14), versions (v1.2.3) and single-letter acronyms no longer mis-split — using ONLY deterministic lexical signals (digit.digit, a small fixed abbreviation list, single-letter-then-letter, lowercase-continuation scoped to single-letter tails); NO semantics/entailment/model. The corpus builder and the READ-2 verifier use the SAME function (no drift by construction); one sentence per span holds; normal sentences still split; a single-letter sentence-end before a capital still splits. 9 splitter tests (24 substrate total); the READ-4 abbreviation fixture flipped to Verified + a fragment-of-it stays Rejected (false-grounded still 0); release_check gates it (substrate tests + `fn is_period_boundary` signal); live sabotage (naive period-splitting) → gate exit 101. A panel found rule (d) over-reached on lowercase-start sentences; folded a fix scoping it to single-letter acronym tails. vibe/codec/adapter/eval/train-gate/cli all 0-diff._
- [x] READ-6 — Reader Autonomy v0. _Delivered 2026-06-15; `crates/reading-autonomy` — a DETERMINISTIC, BOUNDED reader (no model, no training) that generates a reading plan from corpus METADATA (titles + span ids, not all text) and routes every proposed action ONLY through `reading_codec::decode`. v0 strategy: inspect metadata → read up to `max_spans` spans by id → claim each span's sentence verbatim (READ-2 grounded) → one bounded finalize. `ReaderBounds{max_steps,max_spans,max_finalize_attempts}` enforced; the reader holds no executor/verifier handle and cannot finalize on its own — a fabricated claim is rejected by the codec/verifier. 8 tests (metadata-first, bounds, sentence-grounded, fabricated-rejected, replay/determinism) + runnable `autonomous_read` example (must finalize a verifier-authorized answer). release_check gates it (test+fmt+clippy + runnable example + codec-only [0 `execute(`/`verify(`] + bounds-struct + no-ML + separation); live sabotage (reader fabricates) → codec rejects, example exit 1, gate exit 101. Hard boundary held: autonomy proposes, codec validates, substrate executes, verifier authorizes, replay records, weights untouched. All prior crates 0-diff; READ-4/READ-5 packs green._
- [x] READ-7 — Autonomous Corpus Eval Pack. _Delivered 2026-06-15; `crates/reading-autonomous-eval` — drives the deterministic READ-6 reader across the READ-4 corpus fixtures with NO hand-written plans (each corpus rebuilt via `corpus_from_documents`, the reader proposes its own plan), INDEPENDENTLY re-verifies every finalized answer with `reading_substrate::verify` (false-grounded is measured, not assumed), and compares the manual-plan score to the autonomous-reader score. Result: autonomous 15/15 verified, **0 false-grounded**, 0 false-reject; manual baseline 6 verified / 9 rejected. The 9 reject-fixtures become "safe divergences" (the non-adversarial reader honestly grounds where the adversarial hand-plan was rejected) — notably the negation fixture keeps "Do not" intact (verbatim whole-sentence claim). 9 tests (every-fixture, no-hand-plan, 0-false-grounded, independent re-verify, manual-vs-autonomous partition, negation-preserved, tight-bounds classified false-rejects, determinism) + runnable `autonomous_pack_report` example. release_check gates it (test+fmt+clippy + runnable example + no-`fixture.plan` + `verify(` re-check + no-ML + separation); live sabotage (use hand-plan in audit) → test + gate exit 101. Hard rule honored: autonomy underperformance is an engineering signal, NOT a training justification — P12 still owns weights. All prior crates 0-diff._
- [x] READ-8 — Budgeted Autonomous Span Selection. _Delivered 2026-06-15; `reading_autonomy::read_budgeted` (new `budgeted.rs`, additive — the blunt READ-6 `read` is byte-identical so READ-7 stays green) makes autonomy **less blunt**: still metadata-first and codec-only, it reads spans under the budget and CLAIMS only spans LEXICALLY relevant to the question — deterministic word-prefix overlap (the shorter term ≥3 chars is a prefix of the longer, so "wind" matches "winds") against a small fixed stopword list; NO model/semantics/entailment/paraphrase. `crates/reading-budgeted-eval` measures it vs the blunt reader over the READ-4 fixtures: **blunt 21 claims → budgeted 17, 3 fixtures more focused** (weather → just the wind sentence, medical → just the ECG order, multi-sentence → just "No injuries were reported."), **0 false-grounded** (cross-validated via verify + `independently_grounded`), negation preserved. A tight budget (`max_spans=1`) yields **classified coverage misses** (relevant span beyond budget) — never a false answer. 13 reading-autonomy tests (5 budgeted: selective, codec-finalize, budget-enforced, deterministic, negation-preserved) + 7 eval tests + runnable `budgeted_pack_report`. release_check gates it (test+fmt+clippy + runnable example + `read_budgeted`/`decode(`/`prefix_overlap`/`content_terms` signals + no-ML + separation); live sabotage (relevance always-true → blunt) → 4 tests + gate exit 101. Boundary held: deterministic selection only — no model judgment/entailment/paraphrase/training; coverage misses are an engineering signal, P12 still owns weights. read() 0-diff; all other prior crates 0-diff._
- [x] READ-9 — Title-Aware Deterministic Relevance Ranking. _Delivered 2026-06-17; `reading_autonomy::read_ranked` (new `ranked.rs`, additive) orders the READ-8 budgeted reader's span reads by DETERMINISTIC title relevance (document TITLE vs question, the same lexical word-prefix overlap as READ-8 — NO model/semantics/entailment/paraphrase, and never a span-text preview before a span is read by id), so under a tight budget a title-relevant document is reached first instead of missed. The shared selective-read core (`read_selecting`, factored out of `read_budgeted` behavior-identically and parameterised only by read order) keeps budget + relevance filter + codec routing identical for both readers, so the claim FILTER is unchanged — a span is claimed only if its OWN text is relevant AND grounds verbatim, so a title match alone can never fabricate support (matching title + irrelevant span text → coverage miss, not a grounded answer). `crates/reading-ranked-eval` proves **no-regression** vs `read_budgeted` on the committed pack (15 answered, **0 regressions**, every fixture `==budgeted`, **0 false-grounded** cross-validated via verify + `independently_grounded`) and measures the **title-priority win** on a constructed scenario: relevant doc filed second + `max_spans=1` → budgeted **misses**, ranked **answers** "Winds will reach forty miles per hour.", stable across file order. Sort key `(title_relevance DESC, title ASC, document_id ASC)` is insertion-order-independent for distinct titles. 18 reading-autonomy tests (5 new: title-priority recovery, file-order stability, anti-fabrication, deterministic, loose-budget no-regression) + 8 eval tests + runnable `ranked_pack_report`. release_check gates it (test+fmt+clippy + runnable example + `read_ranked`/`read_selecting`/`title_relevance`/`title_ranked_order` signals + **no-`read_span`/`.text()` in ranked.rs** metadata-only proof + no-ML + separation); live sabotage (title_relevance ignores the title → blunt) → 1 unit + 2 eval tests + gate exit 101 (no-regression/0-false-grounded stay green under sabotage — safety is independent of the win), restored byte-identical. Read-only adversarial panel (9 agents): **0 defects** across 5 attack lenses, all rubric sub-points (a)–(g) PASS. Boundary held: deterministic title ranking only — no model/entailment/paraphrase/training; coverage misses are an engineering signal, P12 still owns weights. blunt `read` 0-diff; `read_budgeted` behavior-identical; all other prior crates 0-diff._
- [x] READ-10 — Section-Aware / Multi-Term Deterministic Ranking. _Delivered 2026-06-17; the substrate gains heading-labelled SECTIONS as METADATA (`SectionMeta{heading,span_ids}` on `DocumentMeta`; new `add_document_with_sections`) — a heading is NEVER inserted as a span, so no `SpanId` exists for it and no claim can cite/ground a heading (`add_document` delegates byte-identically → every prior reading crate stays green, proven). `reading_autonomy::read_section_ranked` (new `section.rs`, additive) orders the budgeted reader's span reads by `combined_relevance(title, heading, query)` = the count of DISTINCT query terms covered by the document TITLE or the section HEADING (multi-term), so under a tight budget the most relevant SECTION is reached first. Metadata-only (no span-text preview before read; `section.rs` calls no `read_span`/`.text()`), no model/semantics/entailment/paraphrase, and the ranking SCORE only orders reads — it builds no claim/answer (gate greps `extract_claim`/`synthesize`/`answer_text` in `section.rs` to 0, so a score can never become evidence). Reuses the shared `read_selecting` core → claim filter unchanged. `crates/reading-section-eval` proves **no-regression** vs `read_budgeted` on the flat committed pack (15 answered, **0 regressions** `==budgeted`, **0 false-grounded** cross-validated via verify + `independently_grounded`) and measures the **section + multi-term win** on constructed corpora: heading-relevant section filed second + `max_spans=1` → budgeted **misses**, section reader **answers** "Winds will reach forty miles per hour."; and a 3-term-covering section beats a 1-term one sharing the same token → "A severe storm wind warning is in effect tonight."; both stable across section order. Sort key `(combined_relevance DESC, title ASC, heading ASC, document_id ASC, section_index ASC)` is insertion-order-independent for distinct (title,heading). 24 reading-autonomy tests (6 new READ-10) + 9 section-eval tests + 27 substrate tests (3 new section) + runnable `section_pack_report`. release_check gates it (test+fmt+clippy + example + `SectionMeta`/`add_document_with_sections`/`read_section_ranked`/`section_ranked_order`/`combined_relevance` signals + **no-`read_span`/`.text()`** + **no-`extract_claim`/`synthesize`/`answer_text`** in section.rs + no-ML + separation). Live sabotage (invert the section sort → least-relevant first) → 6 tests + example + gate exit 101, restored byte-identical. Read-only adversarial panel (9 agents): **0 defects** across 5 attack lenses (heading/score-as-evidence, full-text-preview boundary, substrate-regression, multi-term/stability, gate-vacuity), all rubric sub-points (a)–(f) PASS. Third consecutive clean panel. Boundary held: heading/title metadata may RANK reads, may NOT ground claims; section score may NOT become evidence; span text not previewed before read; codec/verifier owns finalization. No training (P12 still owns weights). blunt `read` + `read_ranked` + `read_budgeted` source 0-diff; substrate additive/behavior-preserving; vibe engine 0-diff._
- [x] READ-11 — Real Document Section Metadata Ingestion. _Delivered 2026-06-17 (TDD: 7 named tests written first → RED → GREEN). `read0`'s corpus loader (`reading-cli/corpus_load.rs`) now detects Markdown ATX headings (`# `/`## `/`### `… up to 6) DETERMINISTICALLY via `parse_atx_heading` (strict: 1–6 `#`, then whitespace, then non-empty text — `#nospace`, 7+ hashes, bare `#` are body) and `parse_sections` groups body sentences into sections through `add_document_with_sections`. A heading line is NEVER split into a span — it lives only in `SectionMeta.heading` (metadata), has no `SpanId`, so `verify` (grounds only cited-span text) can never cite/ground it. A headingless file → one default empty-heading section, byte-identical to the flat build (READ-3/4 unaffected). `produce_run` stores the corpus's ACTUAL built spans (body sentences, heading-free) in span-id order, so verify/replay (flat rebuild) reproduce the same span ids → same hashes. 7 named tests pass — corpus_load: `markdown_heading_becomes_section_metadata`, `heading_is_not_a_span`, `sentence_under_heading_gets_section_id`, `unheaded_file_gets_default_section`, `non_atx_hash_lines_are_body_not_headings`; lib: `claim_citing_heading_is_rejected`, `misleading_heading_without_body_support_cannot_finalize`, `headed_document_runs_verifies_and_replays`; section-eval: `section_ranked_read0_recovers_heading_relevant_answer` — totalling 18 reading-cli + 10 section-eval tests. release_check READ-11 gate (`parse_atx_heading`/`parse_sections`/`add_document_with_sections` signals + the heading-free span-storage token + an end-to-end **headed-document binary smoke**: read0 run→verify→replay on a `# Wind Forecast` file, asserting the heading text never appears in the run file). Live sabotage (detector looks for `~` not `#`) → 4 cli + 1 section-eval tests + gate exit 101, restored byte-identical. Read-only panel (9 agents): heading-becomes-evidence / replay-consistency / parser-determinism / semantic-creep lenses **0 defects**; 1 gate-vacuity "high" **REFUTED first-hand** (it claimed reverting `produce_run` to `split_sentences(content)` evades the gate — reproduced: the revert leaks `# Wind Forecast` into a stored span and is caught THREE ways — the headed-doc test fails, the grep token is deleted, and the new binary smoke fires → gate exit 101). Folded the panel's kernel-of-truth as the comment-immune binary smoke (gate hardening; no production change — the code was already correct). Boundary held: real document structure exposed as metadata, never turned into evidence. No semantic heading inference, no all-caps guessing, no model, no training (P12 still owns weights). substrate + reading-autonomy + vibe + all other eval crates 0-diff; READ-3/4/7/8/9/10 green._
- [x] READ-12 — Persist Section Metadata in Run Receipts. _Delivered 2026-06-17; schema/receipt hardening (no model work). The run file now persists each document's heading-labelled SECTIONS — `DocumentDto.sections: Vec<SectionDto{heading, span_count}>` (a heading string + a COUNT of consecutive body spans, **never a span**) — so section-aware autonomy can operate over a real read0 output without rebuilding a different structure. SCHEMA bumped to `read0-run-v2`. The flat `spans` stays the canonical span-id source, so the pre-existing grounding/hash/tamper checks are UNCHANGED (`span_text_tamper_still_caught_under_v2`). The shared `pub fn rebuild_corpus` (verify/replay + section consumers) rejects **heading-as-span** tamper (no stored span is an ATX heading) and **section/body-mismatch** tamper (the section counts must partition the body via CHECKED, bounded arithmetic), then rebuilds the SAME sections the run built via `corpus_from_sections`. Headings affect reading ORDER only — `verify` grounds only cited-span text — so sections are evidence-inert and the memory/answer hashes are section-independent. 25 reading-cli tests (incl. `run_receipt_includes_section_metadata`, `rebuild_corpus_reconstructs_the_run_sections`, `heading_as_span_tamper_is_rejected`, `section_body_mismatch_tamper_is_rejected`, `headingless_document_round_trips_under_v2`, `span_text_tamper_still_caught_under_v2`, `section_count_overflow_tamper_is_rejected_without_panic`) + 11 section-eval tests (`section_ranked_read0_uses_persisted_metadata` rebuilds the corpus FROM the receipt and drives `read_section_ranked`). release_check READ-12 gate (schema-v2 / `SectionDto` / `rebuild_corpus` / `corpus_from_sections` / `parse_atx_heading` signals + an end-to-end **receipt-tamper binary smoke**: a headed receipt carries the heading+count, and injecting an ATX heading as a span / corrupting the counts / a usize::MAX overflow count are each rejected — the overflow GRACEFULLY, no panic). Live sabotage (neuter the heading-as-span check) → unit test + gate exit 101, restored byte-identical. Read-only panel (9 agents): section-as-evidence / schema-weakening / replay-reconstruction / gate-vacuity lenses **0 defects**; 1 tamper-completeness "critical" **FOLDED** — a usize::MAX `span_count` could overflow a plain `sum()` and panic the partition slice on a crafted receipt. Reproduced first-hand (read0 verify panicked "attempt to add with overflow"); fixed with a CHECKED cumulative partition (overflow/over/under-coverage → graceful `Tamper`, never a panic) + a regression test + the overflow binary smoke. Not an authority bypass (the file was always rejected — via crash), now a clean rejection. Boundary held: heading text → metadata only; body sentence → span evidence; verifier → cited span text only; a heading cannot ground a claim. No training (P12 still owns weights). substrate + reading-autonomy + codec + all other eval crates + vibe 0-diff; no Cargo.toml/lock change; READ-3/4/7/8/9/10/11 green._
- [x] READ-13 — Receipt Schema Compatibility / Migration Gate. _Delivered 2026-06-17; schema/receipt hardening (no model work). `verify`/`replay` now handle the run-receipt SCHEMA VERSION explicitly: a new `enum SchemaVersion{V1='read0-run-v1', V2='read0-run-v2'}` is parsed FIRST in the shared `rebuild_corpus` chokepoint, and the tag must AGREE with the receipt's content. A v2 receipt MUST carry its `sections` — empty → `Tamper("sections were dropped")` — which CLOSES the READ-12 hole where an empty `sections` array silently fell back to the flat rebuild and still verified, so section metadata could DISAPPEAR unnoticed (sections affect only reading order, not hashes). A v1 receipt (the pre-section shape) MUST NOT carry sections — a v1 tag wearing v2 sections is ambiguous → `Tamper` — and otherwise MIGRATES forward to one default empty-heading section over all spans (the flat rebuild reproduces the same span ids + hashes, so old headingless receipts still verify/replay). An unknown tag → `CliError::UnsupportedSchema` BEFORE any rebuild (never accepted by default, no panic on untrusted input). `produce_run` always writes v2 (v1 is recognized for READING old receipts, never written); `read_run_file` drops its duplicate string-check and delegates to the single pure chokepoint. The schema tag governs STRUCTURE only — it never reaches the codec/grounding and is never folded into `memory_hash`/`answer_hash`, so evidence authority is unchanged (the flat `spans` stays the canonical span-id source; `span_text_tamper_still_caught_under_v2` and the heading-as-span / section-partition / overflow tamper checks keep full strength). 29 reading-cli tests (4 new: `v1_headingless_receipt_migrates_and_verifies`, `v1_receipt_carrying_sections_is_rejected`, `v2_receipt_with_dropped_sections_is_rejected`, `unknown_schema_is_rejected`) + 11 section-eval tests stay green. release_check READ-13 gate (`enum SchemaVersion`/`UnsupportedSchema`/`read0-run-v1`/`fn partition_sections` signals + an end-to-end **schema-version binary smoke**: a real v2 receipt verifies, its v1 migration verifies, and dropped-sections / v1+sections / unknown-version are each rejected — the unknown one with no `panic` in stderr). Live sabotage (revert the v2-must-carry-sections check to the READ-12 silent flat fallback) → `v2_receipt_with_dropped_sections_is_rejected` fails + gate exit 101, restored byte-identical (md5 `d85644fe…`). Read-only adversarial panel (5 agents, Explore): **0 defects** across all 5 lenses (evidence-authority, silent-drop, ambiguity-relabel, panic-robustness, gate-vacuity) — gate-vacuity confirmed every signal grep matches production code and every binary smoke exercises the exact path it claims; the cleanest panel of the arc. Boundary held: READ-13 adds VERSION discipline, not evidence authority. No model, no training (P12 still owns weights). reading-substrate + reading-autonomy + reading-codec + all other eval crates + vibe 0-diff; no Cargo.toml/lock change; READ-3/4/7/8/9/10/11/12 green._
- [x] READ-14 — Receipt Integrity Hashing for Structural Metadata. _Delivered 2026-06-18; schema/receipt hardening (no model work). `read0` now writes `read0-run-v3`, which adds an explicit `structure_hash: Option<u64>` — a deterministic FNV-1a 64-bit hash (the same construction the substrate uses for content hashes; kept local so the substrate stays a pure evidence-hash layer) binding the schema tag + per-document title, ordered span texts, and ordered sections (heading + span_count), with every variable-length field LENGTH-PREFIXED so the hash input is an injective encoding of the structure (no cross-field collision beyond FNV's inherent 2⁻⁶⁴). `enforce_structure_hash` (called at the top of the shared `rebuild_corpus` chokepoint): a v3 receipt MUST carry a structure hash that MATCHES the recomputed one, a v1/v2 (pre-v3) receipt MUST NOT carry one (relabel-keeping-a-stale-hash → Tamper), unknown → `UnsupportedSchema`. This catches NON-EVIDENTIARY structural edits the READ-12/13 consistency checks missed: a heading or title STRING, an UNCITED span's text, a section boundary that still partitions — `uncited_span_tamper_caught_under_v3_not_v2` proves the before/after (a legacy v2 receipt does not bind the uncited span; v3 does). NON-EVIDENTIARY by construction: the structure hash never reaches the codec/grounding, never folds into `memory_hash`/`answer_hash`, never makes a heading citable — verified by trace (it appears only in reading-cli, never in substrate/codec) and by `verify_file` running the evidence re-derivation (`memory_hash`/`answer_hash` match + grounding) INDEPENDENTLY after the structure check. NO MASKING: the tamper tests RESEAL the structure hash (model the strongest attacker, who recomputes it) to prove the deeper checks — heading-as-span, partition, overflow-no-panic, grounding — still fire; the structure hash is an added layer, never a replacement. Honest scope: a v3→v2 downgrade (relabel + strip hash) reverts to legacy-unbound metadata, but only exposes non-evidentiary metadata to undetected edits and can never forge a grounded answer (evidence stays re-derivation-protected) — the migration-safety tradeoff READ-13 kept, not a regression. 37 reading-cli tests (8 new READ-14) + all 5 reading eval crates green (produce→verify within v3). release_check READ-14 gate (`read0-run-v3`/`structure_hash`/`fn structural_hash`/`fn enforce_structure_hash` signals + a structural-hash binary smoke: a v3 receipt carries+verifies a structure hash; heading-string tamper / hash corrupt / hash dropped / v2-keeping-hash each rejected) + the READ-13 smoke updated to build faithful legacy receipts. Live sabotage (neuter the v3 hash comparison) → 4 structural-tamper tests fail + gate exit 101 (the missing-hash and v2+hash tests STAYED green — different branches, precise coverage), restored byte-identical (md5 `066912b4…`). Read-only adversarial panel (6 agents, Explore): **0 defects** across all 6 lenses (evidence-authority, check-masking, forgery-downgrade, determinism-collision, panic-robustness, gate-vacuity) — the forgery lens endorsed the downgrade-within-scope reasoning and gate-vacuity confirmed every signal/smoke load-bearing. Boundary held: READ-14 binds structural integrity, metadata stays non-evidentiary; evidence authority unchanged. No model, no training (P12 still owns weights). reading-substrate + reading-autonomy + reading-codec + all other eval crates + vibe 0-diff; no Cargo.toml/lock change; READ-3/4/7/8/9/10/11/12/13 green._
- [x] READ-15 — Receipt Downgrade Policy / Final Receipt Boundary. _Delivered 2026-06-18; integrity classification only (no model work). Makes the v3→v2 downgrade behaviour EXPLICIT, tested, and operator-visible so the system never silently treats weaker receipt integrity as equivalent to current. `verify_file`/`verify_run` now return `VerifyOutcome{receipt, integrity}`; the new `enum IntegrityLevel{Current, LegacyUnboundStructure}` is DERIVED from the validated schema version (`from_version`: v3→Current, v1/v2→LegacyUnboundStructure) and NEVER persisted — so it can't be forged (a receipt cannot claim a higher level than its tag earns; the level is recomputed from the validated tag every verify). Each level carries a MACHINE-CHECKABLE `token()` — `structure_bound` / `legacy_unbound_structure` (the explicit legacy warning) — plus `is_current()`. `read0 verify` prints `integrity=<token>` and, for a legacy/downgraded receipt, an explicit `warning: legacy_unbound_structure …` line. So a v3→v2 stripped-hash downgrade still VERIFIES (its evidence is bound) but is reported as **legacy, not current** — weaker integrity can never pass itself off as equivalent. The classification touches STRUCTURE only and NEVER changes grounding: `integrity_level_does_not_change_evidence_authority` proves a v3 receipt and its v2 downgrade produce the IDENTICAL verifier `Receipt` (same grounded/answer_supported/replay_matches/passed) — only the level differs; the level is derived AFTER all evidence checks (rederive + memory/answer-hash match + grounding) pass, so a failing receipt never gets a level, and an unknown schema still REJECTS (never classified). 43 reading-cli tests (6 new: v3=structure_bound, v1/v2=legacy_unbound_structure, downgrade-not-current, evidence-unchanged, derived-not-stored, token-stability) + corpus-eval (uses `Ok(_)`+Debug → unaffected by the return-type change) + section-eval green. release_check READ-15 gate (`enum IntegrityLevel`/`struct VerifyOutcome`/`legacy_unbound_structure`/`structure_bound` signals + a downgrade-policy binary smoke: a v3 receipt's `read0 verify` output carries `integrity=structure_bound`; a faithful v2 downgrade verifies but its output carries `integrity=legacy_unbound_structure` + the warning and NEVER `integrity=structure_bound`). Live sabotage (classify legacy receipts as `Current`) → 4 classification tests fail + gate exit 101 (the v3-current and token-stability tests STAYED green — different branches, precise coverage), restored byte-identical (md5 `8d3a6e20…`). Read-only adversarial panel (6 agents, Explore): **0 defects** across all 6 lenses (grounding-unchanged, forgery, silent-equivalence, downgrade-completeness, no-regression, gate-vacuity). Boundary held: READ-15 CLASSIFIES the receipt integrity level; grounding authority is unchanged. No model, no training (P12 still owns weights). reading-substrate + reading-autonomy + reading-codec + all other eval crates + vibe 0-diff; no Cargo.toml/lock change; READ-3/4/7/8/9/10/11/12/13/14 green._
- [x] READ-16 — Reading Track Milestone Freeze. _Delivered 2026-06-18; documentation freeze only (no model work). Freezes the READ-0 → READ-15 reading-track arc as the named, auditable milestone `reading-track-v0.1` before any further capability work. New `READING_TRACK_MILESTONE.md` (mirrors the repo's `GOVERNANCE_MILESTONE.md` freeze pattern) records: (§1) the full 18-commit lineage with hashes — READ-0 substrate `f5b3fa9` (+ READ-1/READ-2 grounding contracts realized in the substrate/codec), the P9–P12 codec/adapter/eval/train-gate layer (`e4ccb6e`/`d197291`/`4b4aef5`/`3902418`), and READ-3…READ-15 (`bffce24`…`11e9c5f`); (§2) the boundaries that hold across the arc (grounding = cited-span text only; codec quarantine of untrusted plans; autonomy orders reads but never grounds; document structure is metadata never evidence; the versioned/integrity-bound/honestly-classified receipt boundary with the flat `spans` as the canonical span-id source so evidence authority is unchanged at every version); (§3) the P12 verdict `training_not_justified` (`training_justified = false`; 0 false-accepts + 0 false-rejects → no clean recurring model failure → weights forbidden; P13–P15 stay closed); (§4) the release-gate / sabotage / panel verification discipline; (§5) the independent-panel record (READ-12 & READ-14 verifier-found defects folded; READ-12→15 closed on 0-defect panels); (§6) honest residuals (deterministic-lexical only, literal sentence-level grounding, the v3→v2 downgrade legacy tradeoff, no model in the loop, prototype-not-production); (§7) the frozen-status declaration. Every one of the 19 doc hashes was cross-checked against `git log` and matches its exact commit subject (0 bogus). release_check READ-16 gate locks the milestone (`test -f` + `FROZEN` + `reading-track-v0.1` + `READ-0`/`READ-15` coverage + `training_not_justified` + pinned lineage endpoints `f5b3fa9`/`3902418`/`11e9c5f`), mirroring the governance-milestone lock. Independent read-only verifier audited the doc against git ground truth (hash accuracy, status correctness, no overstatement). No model, no training (P12 still owns weights). Gate green + silent. The tag `reading-track-v0.1` is created only after a clean tree + green gate (the milestone commit), per the rubric. reading crates + vibe + all prior reading docs 0-diff except the new milestone doc + the gate lock; no Cargo.toml/lock change; READ-3/4/7/8/9/10/11/12/13/14/15 green._

Hypothesis Layer Track (P16 / HYP-0 — a NEW post-freeze track, additive to the frozen `reading-track-v0.1`):

- [x] HYP-0 — Hypothesis-Only Abductive Layer. _Delivered 2026-06-18; `crates/hypothesis-layer` — an abductive layer ABOVE the frozen reading substrate and BELOW human review that may CREATE, SCORE, and TRACE proposed explanations / next probes and NOTHING else. Doctrine: **Probability proposes. Replay tests. Governance authorizes. Memory records.** Core object `HypothesisPacket` is inert data: built ONLY by `propose()`, its fields are PRIVATE with read-only accessors, and it does NOT derive `Deserialize` (so it cannot be forged or mutated off the wire). It carries `Authority::HypothesisOnly` (an enum with exactly ONE variant, so a hypothesis with claim/evidence/governance authority is unrepresentable) and bakes the canonical six `FORBIDDEN_USES` (ground_claim, serve_as_evidence, mutate_reading_memory, alter_verifier_receipt, change_training_gate, bypass_codec_or_governance), so it can never become a claim or evidence. It cites the receipts it was derived from (`EvidenceRef` by content hash — answer + memory hash + label, no handle). Scoring is deterministic integer math (per-mille i64, no floats, no model, no semantic judge); the id is FNV-1a over length-prefixed inputs; a trace is the INPUTS (`HypothesisSpec`, the only deserializable surface) and replay deserializes the spec and RE-DERIVES the identical packet (`verify_consistency` re-derives and rejects any mismatch). A high-risk OR irreversible probe escalates to `human_review_required`; high-risk AND irreversible is `blocked`; neither is `allowed` (deterministic `ProbeClearance::classify`) — probability can schedule a safe test but never authorize a dangerous one. Structural quarantine: production deps are serde + serde_json ONLY (the reading crates are DEV-only, to PROVE non-interference) — `cargo tree --edges normal` shows 0 reading-/0 vibe- deps and no ML crate, so the layer holds no handle that could mutate memory, the verifier, governance, receipts, or engine state, and a hypothesis changes neither the verifier receipt nor the P12 training verdict. 12 unit tests + 4 doctests (2 positive companions + 2 `compile_fail` non-deserializability proofs for `HypothesisPacket` AND `RecommendedProbe`), fmt + clippy clean, runnable `hypothesis_report` example. release_check HYP-0 gate: test+fmt+clippy + COMPILER-backed proofs (compile_fail existence asserts for both inert types; an exhaustive-match `authority_has_exactly_one_variant` test → E0004 on a 2nd variant; an identity-pinned `forbidden_uses_are_exactly_the_canonical_six` test → distinctness + literal refusal, non-circular) + private-fields awk + whole-file manual-`impl Deserialize` scan + no-float/wall-clock/entropy + quarantine cargo-tree (0 reading-/0 vibe-) + no-ML + a determinism double-run diff of the example JSON. **Six** read-only adversarial panel rounds (Explore agents, refute-by-default): the 5 substantive lenses (authority-escape, claim-evidence-isolation, determinism-replay, probe-safety, non-interference) were CLEAN for 5 consecutive rounds, and the gate-vacuity lens drove 4 rounds of structural folds — each REPRODUCED FIRST-HAND (break → confirm caught → md5 byte-restore): R1 encapsulation (public + `Deserialize` fields were forgeable/mutable → private fields + accessors + dropped `Deserialize`, spec is the only wire surface); R2 a hand-written `impl Deserialize` would dodge the derive-line grep → `compile_fail` doctest (compiler-enforced) + whole-file manual-impl scan; R3 the single-variant and propose()-exercised checks were vacuous → exhaustive-match test (E0004) + behavioral example run; R4 the `grep -B1 Deserialize` derive check was dodgeable by an interposing comment and `RecommendedProbe` (all-deserializable fields) had no compiler backstop → a `RecommendedProbe` compile_fail doctest + the bypassable greps replaced by compiler-proof existence asserts; R5 every forbidden-uses check referenced `FORBIDDEN_USES` (circular), so substituting a canonical use for a DUPLICATE (length still 6) un-forbade it undetected → the non-circular identity+distinctness test. R6 was the first fully-dry round (all 6 lenses clean). Boundary held: a hypothesis is a guess to be tested, never a fact — probability proposes but never grounds, mutates, or authorizes. No LLM, no training, no semantic judge — deterministic scoring only; P12 still owns weights and P13–P15 stay closed. A NEW post-freeze track, additive to `reading-track-v0.1`: reading crates + vibe + all prior docs 0-diff; only a new crate, the workspace member, and the gate block are added. Gate green + silent._

## Sprint 20R — Signed Replay Identity Review Pass

Status: Complete.

Correct if the HMAC replay identity path is leak-free, no-key mode is audit-only, wrong-key/tamper cannot suppress mutation, and embedded `test_trusted` cannot reach production loading.

Wrong if there is committed key material, static signed fixtures containing secrets, unsigned ledgers suppressing mutation, or embedded test trust reachable from untrusted scenarios.

Checks:

```text
grep repo for long hex secrets/signature fixtures
no-key ledger -> audit_only
wrong-key ledger -> no suppression
tampered ledger -> no suppression
embedded replay_ledger without test_trusted -> untrusted
production scenario loader rejects test_trusted
release_check.sh silent
```

Pass condition: Sprint 21 starts only after this review is green.

Review result:

```text
secret scan for long hex/private-key material        PASS
no-key signed ledger -> audit_only + reapply         PASS
wrong-key ledger -> audit_only + reapply             PASS
signature-tampered ledger -> audit_only + reapply    PASS
unsigned embedded test_trusted -> audit_only         PASS
embedded replay_ledger without test_trusted          PASS
production scenario loader rejects test_trusted      PASS
release_check.sh silent                              PASS
```

Completed work:

```text
Added production loader guard:
load_scenario(name, allow_test_trusted=False) rejects test_trusted replay ledgers.

Added API boundary:
POST /simulate/scenario rejects test_trusted scenarios with 403.

Added regression/API checks:
production loader rejection and API scenario rejection are now release-gated.
```

## Sprint 21 — Asymmetric Replay Identity

Status: Complete.

Goal: public verification does not imply signing authority.

Build:

```text
replay_asymmetric_key.py
Ed25519 signing/verification via cryptography
signed ledger provenance v2
public-key verification path
private-key signing path
legacy HMAC path retained for local-dev
```

Tests:

```text
asymmetric_signed_ledger_verifies_without_secret
public_key_can_verify_but_not_sign
wrong_public_key_rejects_ledger
tampered_asymmetric_ledger_rejected
hmac_legacy_path_still_supported
```

Acceptance:

```text
external verifier can authenticate ledger
external verifier cannot forge ledger
no private keys committed
snapshot reports asymmetric_signature_status
release_check.sh silent
```

Review result:

```text
Ed25519 private-key signing path                      PASS
Ed25519 public-key verification without private key   PASS
public-key-only fresh replay cannot sign              PASS
wrong public key -> audit_only + reapply              PASS
tampered asymmetric signature -> audit_only + reapply PASS
legacy HMAC signed replay still supported             PASS
new ledgers stamp recovery-ledger-v2                  PASS
existing recovery-ledger-v1 fixtures remain readable  PASS
snapshot reports asymmetric_signature_status          PASS
private-key/long-secret scan                          PASS
release_check.sh silent                               PASS
```

Completed work:

```text
Added scripts/replay_asymmetric_key.py for Ed25519 key generation, signing, and verification.
Extended recovery_replay.py with --ledger-private-key-file and --ledger-public-key-file.
Preserved verified signatures on public-key-only replay when no state-changing replay occurred.
Kept HMAC-SHA256 replay identity as the local-dev legacy path.
Added Sprint 21 regression, CLI, release-gate, migration, and documentation coverage.
```

## Sprint 22 — Raw Experience Ingestion Kernel

Status: Complete.

Goal: every incoming experience becomes an immutable raw episode before interpretation.

Build:

```text
ExperienceEnvelope
RawEpisodePacket
raw_episode_store.py
ingest_experience.py
```

Required fields:

```text
episode_id
trace_id
source
timestamp/logical_tick
raw_payload
modality
capture_context
integrity_digest
ingestion_license
```

Tests:

```text
experience_ingest_preserves_raw_episode
semantic_candidate_requires_raw_episode
raw_episode_is_append_only
malformed_experience_rejected_without_partial_state
```

Acceptance:

```text
raw episode exists before semantic extraction
raw payload preserved
raw episode cannot be mutated by later repair/consolidation
snapshot shows raw episode provenance
```

Review result:

```text
experience ingest preserves raw payload             PASS
semantic candidate requires raw episode             PASS
raw episode overwrite blocked                       PASS
malformed envelope rejected without partial state   PASS
raw episode integrity digest emitted                PASS
parsed_claims starts empty                          PASS
snapshot shows raw episode provenance               PASS
release_check.sh silent                             PASS
```

Completed work:

```text
Added scripts/raw_episode_store.py with ExperienceEnvelope, RawEpisode, and append-only store.
Added scripts/ingest_experience.py for CLI raw-ingestion proof scenarios.
Added RawEpisodePacket schema and four Sprint 22 scenarios.
Extended epistemic_snapshot.py to surface raw ingestion provenance and pending raw-ingestion state.
Added regression, scripts/test.sh, release_check.sh, MEMORY, changelog, implementation, and QA docs.
```

## Sprint 23 — Semantic Candidate Extraction

Status: Complete.

Goal: raw episodes produce candidate interpretations, not accepted facts.

Build:

```text
semantic_candidate_extractor.py
SemanticCandidatePacket
CandidateMemoryNode
```

LLM boundary:

```text
LLM may parse language and propose typed candidates.
LLM may not assign authority.
LLM may not consolidate.
LLM may not mutate memory.
```

Tests:

```text
raw_episode_generates_semantic_candidates
candidate_defaults_to_hypothesis_only
candidate_cites_raw_episode
llm_output_cannot_create_authoritative_memory
candidate_extraction_failure_preserves_raw_episode
```

Acceptance:

```text
candidate nodes are inspectable
candidate nodes cite raw episodes
candidate status is non-authoritative by default
```

Review result:

```text
raw episode generates candidate nodes              PASS
candidate defaults to hypothesis_only              PASS
candidate cites raw episode and integrity digest   PASS
LLM authority injection normalized/blocked         PASS
extraction failure preserves raw episode           PASS
snapshot shows candidate memory nodes              PASS
release_check.sh silent                            PASS
```

Completed work:

```text
Added scripts/semantic_candidate_extractor.py with CandidateMemoryNode extraction.
Added SemanticCandidatePacket schema and five Sprint 23 scenarios.
Extended epistemic_snapshot.py to surface candidate nodes and semantic extraction state.
Added regression, CLI, release-gate, MEMORY, changelog, implementation, and QA documentation coverage.
```

## Sprint 24 — Unified Self-Correction (the Caitlin Leap)

Status: Complete. See `SPRINT_24_PLAN.md` for the full rubric.

Goal: prove the development process is governed by the same machinery as the runtime — a design proposal that would weaken a locked invariant is detected, blocked, denied consolidation, and routed into a deferred revalidation job exactly as Bridge A is blocked under hazard-only evidence.

Build:

```text
simulations/bridge_world/design_memory.json          (locked invariants + audited design decisions)
simulations/bridge_world/design_verifier_rules.json  (weaken-locked-invariant -> hard_contradiction)
scripts/project_self_audit.py                         (--project / --strict; health via real gateway)
scripts/design_audit.py                               (design-governance trace replay)
bridge_world_demo._run_design_proposal_scenario       (new scenario type; reuses emit + mutation gateway)
decision_audit.py --project                            (delegates to project_self_audit)
```

LLM/meta boundary:

```text
Design memory is data; the design verifier rule is data.
The audit reuses the runtime adjudicate(); health updates use the real apply_memory_mutation().
No new verifier engine, no new mutation path, no separate meta-immune-system.
```

Tests:

```text
design_contradiction_weaken_locked_invariant_blocked
design_contradiction_emits_hazard_only_packet
design_contradiction_denied_consolidation_invariant_preserved
design_contradiction_opens_design_revalidation_job
design_proposal_extend_consistent_accepted_and_consolidated
project_strict_audit_passes_with_zero_violations
project_strict_audit_fails_on_untraced_decision
project_health_consolidates_green_only_through_gateway
```

Acceptance:

```text
weaken locked invariant -> hazard_only ContradictionPacket
weakening proposal denied consolidation (invariant preserved, not consolidated)
design_revalidation job scheduled and release blocked
consistent extend proposal accepted and consolidated
design invariant retrieved with license and provenance, never naked
decision_audit.py --project --strict passes with zero violations and fails on incomplete decisions
release gate consolidates project_cognitive_health only through the gateway under license
release_check.sh silent
```

Review result:

```text
weaken-locked-invariant -> hazard_only ContradictionPacket        PASS
weakening proposal denied consolidation (invariant preserved)     PASS
design_revalidation job scheduled + release blocked               PASS
consistent extend proposal accepted + consolidated                PASS
design invariant retrieved with license, not naked                PASS
project strict audit passes with zero violations                  PASS
strict audit fails on missing trace/verifier/license decision     PASS
release gate consolidates project health only through gateway     PASS
release_check.sh silent                                           PASS
```

Completed work:

```text
Added design_memory.json (5 locked invariants + audited design-decision chain) and design_verifier_rules.json.
Added scripts/project_self_audit.py and scripts/design_audit.py; wired decision_audit.py --project.
Added _run_design_proposal_scenario to bridge_world_demo.py reusing emit, ContradictionPacket, and the mutation gateway.
Added two scenarios, Sprint 24 regression assertions, and CLI+grep gates in test.sh and release_check.sh.
```

Residual / next boundary: **closed by Sprint 25 — Derived Effect Classification**. The design verifier rule no longer keys on a self-declared `effect`; the effect is derived from a semantic diff of the proposal claim vs. the invariant claim, and a weakening mislabelled as an extension is reclassified and blocked.

## Sprint 25 — Derived Effect Classification

Status: Complete. See `SPRINT_25_PLAN.md` for the full rubric.

Goal: derive whether a design proposal `weakens`, `contradicts`, `extends`, or `preserves` an invariant from a semantic diff of the claims, not from a self-declared `effect` field. Closes the Sprint 24 residual where a weakening labelled `extend` could bypass the gate.

Hard rule:

```text
effect is evidence-derived metadata.
effect is NOT user / config / assertion authority.
A self-declared effect is an untrusted hint, used only to detect mislabeling.
```

Build:

```text
scripts/effect_classifier.py                          derive_effect (semantic-diff) + effect_family
simulations/bridge_world/design_verifier_rules.json   rules for contradict / preserve / needs_review
project_self_audit.evaluate_design_proposal           derives effect; declared effect is hint-only; effect_mislabel via family
bridge_world_demo._run_design_proposal_scenario        surfaces declared_effect / derived_effect / effect_mislabel
scripts/design_audit.py                                reports declared/derived/mislabel
```

Tests / scenarios:

```text
design_effect_mislabel_attack                 weakening declared extend -> reclassified + mislabel + blocked
design_effect_derived_without_declaration     weakening, no declared effect -> classified from evidence + blocked
design_effect_preserve_consistent             restates/strengthens -> preserve -> accepted
design_effect_lexicon_avoiding_weaken         weakening without a permissive verb (declared preserve) -> reclassified + blocked
design_effect_ambiguous_needs_review          touches protected, no preservation evidence -> needs_review -> blocks
design_contradiction_in_sprint_plan           backward-compat: honest weaken still blocks, no false mislabel
design_proposal_consistent_with_invariants    backward-compat: honest extend still accepts
```

Acceptance:

```text
weakening declared extend -> derived weakening + effect_mislabel true + blocked + not consolidated
weakening with no declared effect -> derived from evidence + blocked (config not authoritative)
consistent claim -> derived extend/preserve + accepted + no mislabel
runtime adjudicate confirms hard_contradiction -> reject_episode on derived effect
Sprint 24 scenarios unchanged; project strict audit zero violations
release_check.sh silent
```

Review result:

```text
weakening declared extend -> reclassified weakening + mislabel + blocked   PASS
weakening with no declared effect -> classified from evidence + blocked    PASS
consistent claim -> derived preserve/extend + accepted + no mislabel       PASS
Sprint 24 scenarios unchanged (honest weaken blocks, honest extend accepts) PASS
runtime adjudicate confirms hard_contradiction -> reject_episode           PASS
project strict audit passes with zero violations                          PASS
release_check.sh silent                                                   PASS
```

Completed work:

```text
Added scripts/effect_classifier.py (deterministic lexical semantic diff: permissive verbs, protective verbs, protected objects, preservation markers) with a self-test.
Rewired evaluate_design_proposal to derive the effect from claims; the declared effect is an untrusted hint surfaced as effect_mislabel only.
Extended design_verifier_rules.json with contradict / preserve / needs_review rules; needs_review blocks (never auto-accepts an unprovable proposal).
Added three scenarios, Sprint 25 regression assertions, CLI+grep gates in test.sh and release_check.sh, and DD_sprint_25 in design_memory.json.
```

Residual / next boundary: **closed by Sprint 26 — Trace-Grounded Invariant Diff**. The lexical classifier is demoted to an early-warning layer; a probe-guarded invariant's preservation is now derived from a behavioral trace, so a preservation marker can no longer launder a weakening past the gate.

## Sprint 26 — Trace-Grounded Invariant Diff

Status: delivered (2026-06-13).

Goal: detect a weakening by what a proposal would *break*, not by the words it uses. Sprint 25's lexical diff is demoted to an early-warning layer; each protected design invariant is bound to a real runtime behavioral probe and the effect is derived from a pre/post behavior trace. Closes the Sprint 25 residual where an explicit preservation marker paired with an out-of-lexicon weakening verb could reach accept.

Hard rule: words are claims, traces are evidence. A protected invariant's preservation must be TESTED, not trusted. A proposal targeting a probe-guarded invariant is accepted only if a behavioral trace proves the protected outcome did not regress; the declared effect and the lexical classifier cannot override the trace.

How the probes are real (not declared): `hazard_gate` runs the actual `adjudicate` (`hard_contradiction` → `reject_episode` protected; the `urgency_overrides_hazard` delta reclassifies it to a `known_exception` → `preserve_as_exception`, i.e. the action is no longer blocked). `consolidation_gate` runs the actual `apply_memory_mutation` (a source whose authority forbids `memory_consolidation` → `rejected`; the `authorize_unprivileged_consolidation` delta grants it → `consolidated`). The proposal's `behavioral_delta` is the change under test, not an authority claim — the trace runs it and derives the effect from the observed behavior delta.

```text
DONE rubric — all checkable PASS
preserve-marker laundering (lexical reads 'preserve') blocked from the trace            PASS
euphemistic 'extend' hazard softening blocked from the trace                            PASS
consolidation-gate softening blocked by the real mutation gateway behavior              PASS
genuine preserving extension (no regression) accepted — gate does not block everything  PASS
declared effect / lexical verdict cannot override a tested regression                   PASS
preserve/extend against a probe-guarded invariant with no delta -> needs_review (block) PASS
Sprint 24/25 scenarios unchanged (attacks block, accepts now trace-confirmed)           PASS
runtime adjudicate confirms hard_contradiction -> reject_episode on the combined effect PASS
project strict audit zero violations; DD_sprint_26 recorded                             PASS
release_check.sh silent; gate-sabotage of the classifier makes it fail (non-decorative) PASS
```

Completed work:

```text
Added scripts/trace_diff.py: a PROBES registry over the real adjudicate / mutation gateway, derive_effect_from_trace (pre/post the proposal's behavioral_delta), and combine_effects (trace overrides lexical; lexical can only raise severity; a probe-guarded invariant cannot be accepted without a passing trace), with self-tests.
Bound behavioral_probe to D_invariant_hazard_blocks_action (hazard_gate) and D_invariant_mutation_requires_authority (consolidation_gate); added DD_sprint_26.
Rewired evaluate_design_proposal: lexical effect is early warning, the trace is authority; surfaced lexical_effect / trace_effect / trace_tested / trace_regressed / trace_pre / trace_post / effect_authority.
Added four scenarios (preserve_marker_launders_weakening_blocked, trace_diff_detects_hazard_gate_softening, trace_diff_detects_consolidation_gate_softening, trace_diff_accepts_true_preserving_extension); updated the two accept-scenarios to carry a no-regression delta; added Sprint 26 regression assertions and CLI+grep gates in test.sh and release_check.sh.
```

Residual / next boundary: residual (1) is **closed by Sprint 27 — Complete Locked-Invariant Probe Coverage** (every locked invariant is now probe-backed and an unprobed locked invariant cannot reach accept). Residual (2), the mis-stated no-op delta on a probed invariant, remains and is the input to the post-Sprint-27 boundary (delta-to-code provenance).

## Sprint 27 — Complete Locked-Invariant Probe Coverage

Status: delivered (2026-06-13).

Goal: bind a real runtime behavioral probe to EVERY locked invariant, and make a locked invariant without a probe ineligible for preserve/extend acceptance. Closes the Sprint 26 residual where the three still-lexical-only locked invariants (`no_naked_facts`, `raw_episode_append_only`, `llm_no_authority`) could be laundered past the gate.

Hard rule: a locked invariant without a behavioral probe is NOT eligible for preserve/extend acceptance. No probe means no proof of preservation — an unprobed locked invariant defaults to `needs_review`. Doctrine: a protected invariant is only protected if the system can test what breaking it looks like.

How the three new probes are real: `naked_fact_gate` runs `retrieval_policy.emergency_use_protocol` (a naked fact is `do_not_use_for_action` → `cannot_support_action`; the `allow_naked_facts` delta grants it `full_premise` → `normal_use`). `raw_append_only_gate` runs the real `raw_episode_store.RawEpisodeStore` (append-only is enforced by an immutable store, so the `allow_raw_overwrite` delta's tested behavior must invoke the store's refused `replace`, diverging from the untouched baseline). `llm_authority_gate` runs `raw_episode_store.semantic_candidate_from_raw` + the real `apply_memory_mutation` (the candidate's real `forbidden_use` blocks consolidation → `rejected`; the `grant_llm_authority` delta grants authority → `consolidated`).

```text
DONE rubric — all checkable PASS
every locked invariant has a runtime-backed probe; each baseline = protected outcome         PASS
no_naked_facts laundering (lexical reads 'preserve') blocked from the trace                   PASS
raw_episode_append_only laundering blocked (proposal's behavior hits the real store refusal)  PASS
llm_no_authority laundering blocked (real gateway rejected -> consolidated regression)        PASS
structural rule: a LOCKED invariant with no probe -> needs_review (cannot reach accept)       PASS
a genuine preserving extension is accepted for EACH locked invariant (no gate blocks all)     PASS
a fake/no-op delta cannot reach accept without a real probe pass (untested -> needs_review)   PASS
project strict audit zero violations; DD_sprint_27 recorded                                   PASS
Sprint 24/25/26 gates unchanged                                                               PASS
release_check.sh silent; gate-sabotage of the structural rule OR a probe makes it fail        PASS
```

Completed work:

```text
Added three real probes to scripts/trace_diff.py (naked_fact_gate, raw_append_only_gate, llm_authority_gate) over retrieval_policy / raw_episode_store / mutation_gateway; bound them in design_memory.json; added DD_sprint_27.
Added the structural rule to combine_effects (invariant_locked + no probe -> needs_review "locked_invariant_without_probe"); evaluate_design_proposal passes invariant_locked = (status == regression_lock).
Added three laundering scenarios (trace_diff_blocks_{no_naked_facts,raw_episode_append_only,llm_authority}_laundering), Sprint 27 regression assertions (all-locked-have-probe, accept-per-invariant, structural rule, unlocked fallback), and CLI+grep gates in test.sh and release_check.sh.
```

Residual / next boundary: **closed by Sprint 28 — Delta-to-Code Provenance** (the tested delta is derived from a provenance-verified `change_set` bound to the real changed artifact, so a mis-stated no-op delta can no longer mask a weakening shipped in the patch). `raw_append_only_gate` remains the honest edge of the behavioral model — append-only is enforced by an immutable store, so the probe detects a weakening by the proposal's tested behavior having to invoke the store's refused `replace`.

## Sprint 28 — Delta-to-Code Provenance

Status: delivered (2026-06-14).

Goal: bind the delta tested by `trace_diff` to the actual proposed change set. Sprint 27's probe ran the proposal's *self-declared* `behavioral_delta`; a mis-stated no-op delta could pass while a weakening shipped in the code/config the prose gestured at. A delta without provenance is just another label. Closes the Sprint 27 residual.

Hard rule / doctrine: a trace is only evidence if it tests the thing being changed. For a locked invariant, the tested delta is DERIVED from a provenance-verified `change_set` — the self-declared `behavioral_delta` is never authority; missing or unverifiable provenance blocks.

How provenance is real: a `change_set` is `{target, changed_artifact, patch, adds?, patch_digest}`. Provenance holds only when (1) `target` is a known control point, (2) `changed_artifact` equals the real source file that implements it (`change_provenance.CONTROL_POINT_ARTIFACTS`) AND that file exists on disk, and (3) `patch_digest` equals the canonical SHA-256 of `(target, changed_artifact, patch, adds)`. Only then is the tested delta derived from `patch`. The self-declared `behavioral_delta`, if present, is surfaced as `delta_matches_change_set` (a hint) and is never authority.

```text
DONE rubric — all checkable PASS
mis-stated no-op behavioral_delta + weakening change_set patch -> the PATCH is tested -> block   PASS
delta_matches_change_set surfaces the declared-vs-patch disagreement                              PASS
genuine preserving change_set accepted, citing the real changed_artifact (verified provenance)    PASS
behavioral_delta with NO change_set against a locked invariant -> needs_review (provenance)        PASS
no provenance at all for a locked invariant -> block (delta_provenance_unverified)                 PASS
trace_diff derives the tested policy from change_set.patch, not from behavioral_delta              PASS
Sprint 26/27 scenarios migrated to a provenance-verified change_set keep their governance          PASS
project strict audit zero violations; DD_sprint_28 recorded                                        PASS
release_check.sh silent; sabotage (trust declared delta / skip provenance) makes it fail           PASS
```

Completed work:

```text
Added scripts/change_provenance.py: CONTROL_POINT_ARTIFACTS registry, canonical digest, verify_change_set_provenance (target/artifact/file-exists/digest checks), --selftest CLI.
Rewired trace_diff.derive_effect_from_trace to derive the tested delta from a provenance-verified change_set (never the self-declared behavioral_delta); surfaced provenance / changed_artifact / delta_matches_change_set; combine_effects blocks a locked invariant with missing/unverified provenance (authority delta_provenance_unverified).
Migrated the 9 Sprint 26/27 design scenarios from behavioral_delta to a provenance-verified change_set; added four scenarios (misstated_noop_delta_with_weakening_patch_blocked, derived_delta_matches_patch_accepts_preserving_change, missing_patch_for_behavioral_delta_needs_review, delta_provenance_required_for_locked_invariant), DD_sprint_28, Sprint 28 regression assertions, and CLI+grep gates in test.sh and release_check.sh.
```

Residual / next boundary: **closed by Sprint 29 — Artifact Content-Hash Binding** (the tested delta is now bound to the literal before/after content of a real on-disk policy artifact whose hash is recomputed from disk at evaluation time; a stale or wrong-content pre-image, or a structured patch that diverges from the literal post-image, is rejected).

## Sprint 29 — Artifact Content-Hash Binding

Status: delivered (2026-06-14).

Goal: bind the tested delta to the actual artifact content it claims to modify, not merely an artifact path and a structured patch description. Sprint 28 bound the patch to the artifact's *path*; a faithful-looking structured patch could still diverge from the eventual file edit, or target the right path but assume the wrong content. Closes the Sprint 28 residual.

Hard rule / doctrine: a change is not the file name, and not the prose patch — it is the before/after artifact content and the behavior it produces. The tested delta is derived from the literal diff of the artifact's real content.

How it is real: each control point has a real on-disk policy artifact (`simulations/bridge_world/control_point_policies/<cp>.json`) whose content defines its protected baseline policy and which the probe reads as its baseline. A `change_set` carries `pre_image` + `pre_image_hash` + `post_image` + `post_image_hash` + `diff_digest`. Provenance holds only when the `changed_artifact` is the registered policy file and exists, the supplied `pre_image` hashes to `pre_image_hash` AND that equals the SHA-256 of the artifact's ACTUAL on-disk content (stale/wrong content is rejected), the `post_image` hashes to `post_image_hash`, the `diff_digest` binds the literal unified diff, and the `post_image` parses to a policy dict. The tested policy is the parsed post-image — derived from content. An optional declared `patch` must equal the literal-derived policy (else `structured_patch_diverges`). If the artifact's on-disk baseline content no longer yields the protected outcome, the probe-integrity check blocks.

```text
DONE rubric — all checkable PASS
stale_pre_image (does not match the artifact's real content) -> block                            PASS
wrong_post_image (post content does not hash to its declared hash) -> block                       PASS
structured patch diverges from the literal post-image -> block                                    PASS
literal-diff weakening (post flips a protected key) regresses -> block, cites pre/post/diff        PASS
literal-diff preserving change (benign added key) -> accept, cites artifact + pre/post/diff hashes PASS
tested delta derived from the literal post-image; change_set cannot verify unless content matches  PASS
Sprint 26/27/28 scenarios migrated to content-bound change_sets keep their governance             PASS
project strict audit zero violations; DD_sprint_29 recorded                                        PASS
release_check.sh silent; sabotage of the stale or divergence check makes it fail                   PASS
```

Completed work:

```text
Added simulations/bridge_world/control_point_policies/*.json (real per-control-point baseline policy artifacts); the probe baseline is loaded from the on-disk artifact.
Extended scripts/change_provenance.py with content binding: canonical_policy_text, content_hash, literal_diff/diff_digest, load_baseline_policy, build_content_change_set; verify_change_set_provenance now binds pre/post image hashes to the artifact's real content and derives the policy from the literal post-image (reasons: stale_pre_image, wrong_post_image, diff_digest_mismatch, non_applicable_patch, structured_patch_diverges, ...).
Rewired trace_diff to load the baseline from the on-disk artifact and surface pre_image_hash/post_image_hash/diff_digest; combine_effects blocks on any content-binding failure.
Migrated all Sprint 28 change_sets to content-bound change_sets; added five scenarios (stale_pre_image_hash_rejected, wrong_post_image_hash_rejected, structured_patch_diverges_from_literal_diff_blocked, literal_diff_weakening_change_blocks, literal_diff_preserving_change_accepts), DD_sprint_29, Sprint 29 regression assertions, and CLI+grep gates in test.sh and release_check.sh.
```

Residual / next boundary: the "content proves *what* changed, not *who* authorized it" half is **closed by Sprint 30 — Signed Change Provenance**. The mechanism-source-vs-policy-artifact half remains deferred to ADR-002 L0 (the runtime engine's replay contract).

## Sprint 30 — Signed Change Provenance

Status: delivered (2026-06-14).

Goal: a verified content-bound change to a locked invariant must also carry accountable Ed25519 authorization over the content digest, and authorization must never override a behavioral failure. Closes the Sprint 29 "who authorized it" residual.

Hard rule / doctrine: content binding proves WHAT changed; trace binding proves what BEHAVIOR changed; signature binding proves WHO accepted responsibility; authorization never overrides invariant failure.

How signing is real and secret-free: `design_signing` reuses the Sprint-21 Ed25519 machinery (private key signs, public key verifies; public verification can never mint signing authority). The signed payload is the canonical hash of `(scheme, signer, target, changed_artifact, pre_image_hash, post_image_hash, diff_digest, control_point, nonce)` — so a signature is bound to that exact content and cannot be replayed onto a different artifact/diff. The committed registry (`authorized_design_signers.json`) holds only the authorized signer's PUBLIC key; the signing private key is generated at authoring, used to sign the committed scenarios, and discarded — never committed. The signature gate is necessary-not-sufficient: it constrains a would-be ACCEPT on a locked invariant (unsigned/unauthorized/wrong-key/replayed → block) but NEVER relaxes a trace/lexical block — a validly-signed weakening still blocks by trace.

```text
DONE rubric — all checkable PASS
unsigned content-bound change to a locked invariant -> block (change_signature_unverified)        PASS
wrong signer (not in authorized registry) -> block (unauthorized_signer)                          PASS
replayed signature (copied onto different content) -> block (signature_payload_mismatch)          PASS
validly-signed preserving change -> accept (signature_verified), reports signer + digest + trace  PASS
validly-signed WEAKENING -> still blocks by trace (signature_verified but trace_behavior_regression) PASS
content binding remains required; runtime round-trip proves sign/verify, wrong-key, tamper, replay PASS
Sprint 26/27/28/29 attacks still block; accept-scenarios still accept (now signed)                PASS
project strict audit zero violations; DD_sprint_30 recorded; no private key committed              PASS
release_check.sh silent; sabotage of signature verification (no-op gate / always-verify) fails it  PASS
```

Completed work:

```text
Added scripts/design_signing.py: change_signing_payload, sign_change_set, load_authorized_signers, verify_change_signature (reasons: signature_verified / unsigned / unauthorized_signer / wrong_key / signature_payload_mismatch / signature_invalid), reusing replay_asymmetric_key Ed25519.
Added simulations/bridge_world/authorized_design_signers.json (committed PUBLIC key for design_authority; no private key in the repo).
evaluate_design_proposal gained a signature gate (a would-be-accept on a locked invariant requires a verified signature; the gate never overrides a trace/lexical block) and an authorized_signers override param; surfaced signer / signature_status / signed_payload_digest.
Signed the 5 existing accept-scenarios with a committed design_authority signature (key generated at authoring, discarded); added five scenarios (signed_preserving_change_accepts, signed_weakening_change_still_blocks, unsigned_content_bound_change_blocks, wrong_signer_rejected, signature_replay_against_different_artifact_rejected), DD_sprint_30, Sprint 30 regression assertions (incl. a runtime round-trip with ephemeral keys + a no-private-key-committed gate), and CLI+grep gates in test.sh and release_check.sh.
```

Residual / next boundary: the flat-public-key-list half is **closed by Sprint 31 — Signer-Set Governance** (the registry is now a governed object with scope + lifecycle, evaluated at the decision tick). The mechanism-source-vs-policy-artifact half remains deferred (the Sprint-29 residual, an ADR-002 L0 runtime concern); threshold/multi-signer governance is also deferred.

## Sprint 31 — Signer-Set Governance

Status: delivered (2026-06-14).

Goal: a public key is not permanent authority. Promote the signer registry to a governed object — each signer carries a scope and a lifecycle (active / expired / revoked / rotated) — and evaluate authority at the decision tick, so a genuine signature from a now-revoked, expired, or out-of-scope signer is not authorization. Closes the Sprint 30 flat-public-key-list residual.

Hard rule / doctrine: a public key is not permanent authority; a signer is an authority-bearing object evaluated at decision time; a valid signature from a no-longer-authorized signer is not authorization; authorization still never overrides a trace failure.

How governance is real and deterministic: the registry (`authorized_design_signers.json`, schema v0.2) maps each `signer_id` to `{public_key, scope, status, valid_from_tick, expires_at_tick, revoked_at_tick, rotated_to}`. `design_signing.verify_change_signature(change_set, registry, now_tick, change_scope)` proves cryptographic authorship FIRST (unchanged from Sprint 30), then `signer_authority` decides whether the genuine signer is currently authorized for this change: revoked (`status==revoked` or `now_tick >= revoked_at_tick`) → `signer_revoked`; expired (`now_tick >= expires_at_tick`) → `signer_expired`; before its window (`now_tick < valid_from_tick`) → `signer_not_yet_valid`; out of scope (the change's target control point is not in `scope`, and `scope` is not `*`) → `signer_wrong_scope`. Lifecycle is expressed in LOGICAL ticks (`evaluation_tick` per proposal, never wall-clock), so the gate is reproducible; a release-gate asserts no wall-clock symbol appears in `design_signing`. `design_authority` is preserved as active + wildcard scope, so every Sprint 26–30 committed signature stays valid. The gate still only constrains a would-be ACCEPT and never overrides a trace block.

```text
DONE rubric — all checkable PASS
revoked signer (status revoked) -> block (signer_revoked)                                            PASS
expired signer (now_tick past expires_at_tick) -> block (signer_expired)                             PASS
out-of-scope signer (scoped elsewhere) -> block (signer_wrong_scope)                                 PASS
rotated successor (active, in window, in scope) -> accept (signature_verified)                       PASS
revoked key cannot replay a prior signature (genuine sig, decided after revoked_at_tick) -> block    PASS
  + decision-time proof: SAME signature verified at tick 5, signer_revoked at tick 20                PASS
governed-but-valid signer on a WEAKENING -> still blocks by trace (trace_behavior_regression)        PASS
content binding + crypto authorship remain prerequisites; every governance change_set still verifies PASS
Sprint 26–30 scenarios keep their governance; audit is signer-authority-visible; DD_sprint_31        PASS
release_check.sh silent; sabotage of signer governance fails it; no private key; logical-tick only    PASS
```

Completed work:

```text
design_signing.py: governed registry — normalize_signer_registry (accepts v0.1 flat map or v0.2 governed objects), signer_authority (tick-based revoke/expire/valid-window + scope), verify_change_signature gains (now_tick, change_scope) and reasons signer_revoked / signer_expired / signer_not_yet_valid / signer_wrong_scope (all crypto reasons unchanged); selftest extended (incl. the decision-time replay proof).
authorized_design_signers.json promoted to v0.2 governed registry (8 signers; design_authority preserved active+wildcard so S26–S30 signatures stay valid); only PUBLIC keys committed.
evaluate_design_proposal threads now_tick (from proposal.evaluation_tick) + change_scope (the change_set target); surfaces signer_status / signer_scope / signer_expires_at / signer_revoked_at / signer_rotated_to / evaluation_tick. bridge_world_demo + design_audit surface them for audit visibility.
Added scripts/author_governed_signers.py (one-time authoring tool; generates governed keys, signs scenarios, discards private keys — not run by release_check).
Added six scenarios (revoked_signer_rejected, expired_signer_rejected, wrong_scope_signer_rejected, rotated_successor_accepted, revoked_key_cannot_replay_prior_signature, signed_weakening_still_blocks_under_governance), DD_sprint_31, Sprint 31 regression assertions (incl. an in-process decision-time + rotation-lineage proof) and CLI+grep gates in test.sh and release_check.sh (incl. a no-wall-clock determinism gate and test -f SPRINT_31_PLAN.md).
```

Residual / next boundary: the single-signer half (threshold/multi-signer) remains deferred. The "bound unit is the policy artifact, not the mechanism source" half is **closed by Sprint 32 — Mechanism-Source Content Binding**.

## Sprint 32 — Mechanism-Source Content Binding

Status: delivered (2026-06-14).

Goal: bind the enforcement CODE itself, not only the policy artifacts it reads. A signed, content-bound policy is not enough if the gate code underneath it can be changed unsigned. Closes the Sprint-29/30 residual that the bound unit was the policy artifact, not the mechanism source.

Hard rule / doctrine: a policy artifact says what the rule is; the mechanism source decides whether the rule is actually enforced; a signed policy is not enough if the gate code can be changed unsigned; authorization never overrides a trace failure.

How it works (three layers): (1) **Integrity manifest** — `mechanism_provenance.py` + `mechanism_source_manifest.json` bind every enforcement-code file (`verifier_engine.py`, `mutation_gateway.py`, `retrieval_policy.py`, `raw_episode_store.py`, `trace_diff.py`, `change_provenance.py`, `design_signing.py`, `effect_classifier.py`, `project_self_audit.py`, and `mechanism_provenance.py` itself) by content hash, keyed by role. `--verify` recomputes from disk and fails release on any divergence; the project strict audit gates on `mechanism_source_binding == verified`. (2) **Mechanism-source change provenance** — a `change_set` with `binding: "mechanism_source"` binds the literal before/after content of a real source file; the pre-image must equal the CURRENT on-disk source, and it flows through the same Sprint-30/31 governed signature gate. (3) **Behavioral probe on the PROPOSED source** — `trace_diff` evaluates the bound gate's protected behavior against the proposed post-image WITHOUT executing it: the post-image is parsed, the bound function extracted, and its body safely interpreted over fixed inputs across a restricted AST subset (if/boolean/comparison/return over parameters and literals), fail-closed to a regression on anything outside that subset. A weakening of the adjudicator is caught here, by probe, even with clean policy files and a valid signature. (The probe never runs the proposed code — an adversarial-panel finding that an earlier subprocess-execution design gave a probe-passing post-image filesystem access was fixed before close.)

```text
DONE rubric — all checkable PASS
mechanism-source change bound to a stale/wrong pre-image -> block (stale_pre_image)                  PASS
unsigned mechanism-source change to a locked gate -> block (change_signature_unverified)             PASS
signed preserving mechanism-source change -> accept (probe confirms the gate survives)               PASS
signed WEAKENING of the adjudicator -> block BY PROBE (signature_verified but trace_behavior_regression) PASS
policy files clean but gate CODE weakened -> block by probe                                          PASS
manifest verifies against real on-disk code; a tampered recorded hash is detected (non-vacuous)      PASS
project strict audit reports + gates on mechanism_source_binding; decision_audit --project surfaces it PASS
release_check silent; sabotage (weaken real gate code / always-pass probe) each fails release        PASS
Sprint 26-31 scenarios keep their governance; DD_sprint_32 recorded; no private key committed         PASS
```

Completed work:

```text
Added scripts/mechanism_provenance.py: MECHANISM_SOURCE_ARTIFACTS (role->enforcement file), build/load/verify_mechanism_manifest, verify_mechanism_change_provenance (reasons stale_pre_image/wrong_post_image/diff_digest_mismatch/mechanism_role_unknown/mechanism_artifact_mismatch/mechanism_source_missing/malformed_images), probe_outcome_for_proposed_source (safe AST interpretation of the bound function over fixed inputs — never executes the proposed code; fail-closed), --verify/--build/--selftest.
trace_diff.derive_effect_from_trace dispatches a binding=="mechanism_source" change_set to _derive_mechanism_effect (provenance + proposed-source probe); combine_effects' locked-block list extended with the mechanism provenance failures; new trace fields mechanism_source / mechanism_role.
project_self_audit.audit_project gained mechanism_source_binding (verified/violated) and gates strict_pass on it; evaluate_design_proposal surfaces mechanism_source/mechanism_role; design_audit + bridge_world_demo surface them; decision_audit --project reports mechanism_source_binding.
Added scripts/author_mechanism_scenarios.py (one-time; signs with a governed mechanism_authority key generated then discarded) and 5 scenarios (mechanism_source_hash_mismatch_fails_release, unsigned_mechanism_source_change_blocks, signed_mechanism_preserving_change_accepts, signed_mechanism_weakening_change_blocks_by_probe, policy_artifact_clean_but_gate_code_weakened_fails); added mechanism_authority (public key only) to the registry; generated mechanism_source_manifest.json; DD_sprint_32; Sprint 32 regression assertions + CLI+grep gates in test.sh and release_check.sh + test -f SPRINT_32_PLAN.md.
```

Residual / next boundary (explicitly deferred): the enforcement code is now content-bound and a proposed change to it is signed + probe-tested. Honest remaining limits (no safe-default claim): (1) only the adjudicator role is BEHAVIORALLY probed against a proposed change — the other 9 bound roles are integrity-bound (a tampered file is caught by the manifest hash + the strict audit), but a proposed change to them that cannot be behaviorally proven preserving fails closed to needs_review rather than being probe-tested; (2) the probe interprets only a restricted AST subset, so a behavior-preserving change that uses a construct outside that subset (a helper call, a loop) on the adjudicator fails closed to a regression rather than being accepted — a precision cost in the safe direction; (3) governance is still single-signer (threshold/multi-signer deferred); (4) the manifest verifier can only attest the OTHER files honestly — a tamper of `mechanism_provenance.py` itself is bounded by the behavioral probes + regression suite, not by a watcher above it (the irreducible who-watches-the-watchmen fixed point in a single-repo self-check). Next boundaries: behavioral binding for more mechanism roles, then threshold/multi-signer governance if still needed.

## Sprint 24i — Trust and Provenance Assignment

Status: Backlog (deferred by the Caitlin Leap; now gated by the unified self-correction loop).

Goal: every candidate gets explicit trust/provenance metadata from governed rules, not from prose confidence.

Build:

```text
trust_provenance_engine.py
SourceAuthorityProfile
EvidenceProvenancePacket
trust_rules.json
```

Tests:

```text
candidate_receives_source_provenance
untrusted_source_caps_license
trusted_source_still_requires_evidence
llm_confidence_ignored_for_authority
conflicting_provenance_creates_contradiction_or_review
```

Acceptance:

```text
epistemic_license derived by rule
allowed_use / forbidden_use attached
source lineage visible in snapshot
```

## Sprint 25i — Multi-Index Memory Layer

Status: Backlog (deferred by the Caitlin Leap; now gated by the unified self-correction loop).

Goal: memory is indexed by meaning, time, entity, causality, and use.

Build:

```text
memory_index.py
meaning_index
time_index
entity_index
causal_index
use_index
index_audit.py
```

Tests:

```text
memory_indexed_by_meaning
memory_indexed_by_time
memory_indexed_by_entity
memory_indexed_by_causality
memory_indexed_by_use
same_memory_retrievable_through_multiple_indexes
index_update_requires_mutation_gateway
```

Acceptance:

```text
each memory node has index memberships
index audit can reconstruct why a node is retrievable
no index mutation outside gateway
```

## Sprint 26i — Retrieval Under Task Pressure

Status: Not started.

Goal: retrieval changes behavior based on task pressure, EvidenceRequirementLevel, urgency, risk, and authority.

Build:

```text
retrieval_policy.py
TaskPressurePacket
RetrievalPolicyDecision
```

Tests:

```text
strict_task_rejects_hypothesis_only_memory
urgent_low_risk_allows_weak_premise
urgent_high_risk_uses_hazard_only_as_blocker
retrieval_returns_blocked_and_allowed_uses
retrieval_cites_indexes_and_authority
```

Acceptance:

```text
retrieval result includes license
retrieval result includes allowed/forbidden use
retrieval result includes pressure mode
snapshot shows retrieval constraints before planning
```

## Sprint 27i — Outcome Testing Harness

Status: Not started.

Goal: actions and predictions are tested against actual outcomes and produce evidence, not automatic belief promotion.

Build:

```text
outcome_test_harness.py
PredictionPacket
ExpectedOutcomePacket
ObservedOutcomePacket
OutcomeComparisonPacket
```

Tests:

```text
expected_outcome_bound_to_plan
observed_outcome_bound_to_action
success_updates_policy_not_belief_by_default
failure_creates_correction_job
partial_match_creates_scoped_review
```

Acceptance:

```text
outcome evidence enters correction queue
belief revision not automatic
procedure/planner/attention lanes remain separate
```

## Sprint 28i — Evidence-Only Revision Pipeline

Status: Not started.

Goal: memory revision requires explicit evidence, verifier rule, mutation authority, and audit trail.

Build:

```text
memory_revision_engine.py
RevisionProposalPacket
RevisionDecisionPacket
```

Tests:

```text
revision_requires_new_evidence
revision_without_evidence_rejected
revision_preserves_raw_episode
revision_uses_mutation_gateway
revision_audit_reconstructs_before_after
```

Acceptance:

```text
semantic nodes can be revised
raw episodes cannot be rewritten
revision cites evidence and verifier_rule_id
```

## Sprint 29i — Stable Consolidation

Status: Not started.

Goal: repeated stable evidence can promote candidates into consolidated semantic/procedural memory through governed rules.

Build:

```text
consolidation_engine.py
ConsolidationCandidate
ConsolidationDecisionPacket
stability_rules.json
```

Tests:

```text
unstable_candidate_not_consolidated
repeated_confirmed_candidate_consolidates
contradicted_candidate_cannot_consolidate
consolidation_requires_stability_window
consolidation_mutates_through_gateway
```

Acceptance:

```text
candidate -> consolidated only after stability criteria
consolidation audit shows evidence history
snapshot shows consolidated vs candidate status
```

## Sprint 30i — Staleness, Demotion, and Forgetting

Status: Not started.

Goal: stale memory is demoted, quarantined, archived, or forgotten according to explicit policy.

Build:

```text
staleness_policy.py
ForgettingDecisionPacket
MemoryDemotionPacket
retention_rules.json
```

Boundary:

```text
Raw episodes may be archived or hidden from active retrieval.
Raw episodes should not be silently destroyed unless explicit destructive-retention policy exists.
Semantic authority can decay.
```

Tests:

```text
stale_memory_demoted
stale_hazard_remains_as_hazard_if_safety_relevant
unused_low_authority_memory_archived
forgetting_does_not_delete_raw_episode_without_policy
demoted_memory_cannot_support_full_premise
```

Acceptance:

```text
stale memory loses authority
forgotten/archived memory visible in audit
retrieval respects demotion
```

## Sprint 31i — LLM Linguistic Operating Layer Boundary

Status: Not started. _(Renumbered 31 → 31i: Sprint 31 is the delivered Signer-Set Governance above; this LLM-boundary sprint is the backlog `i`-track item, also developed as the P9–P15 codec track below.)_

Goal: the LLM translates between human language and internal representations but does not store or authorize world state.

Build:

```text
language_codec.py
IntentParser
ExplanationRenderer
InternalPacketTranslator
LLMUsePolicy
```

Tests:

```text
llm_parses_user_intent_to_typed_packet
llm_renders_explanation_from_audit_packets
llm_cannot_write_memory_directly
llm_cannot_assign_epistemic_license
llm_cannot_retrieve_hidden_world_state
llm_output_requires_verifier_before_action
```

Acceptance:

```text
LLM = language codec
memory graph = world store
verifier/mutation gateway = authority boundary
```

## Sprint 32i — Full Lifecycle End-to-End Scenario

Status: Blocked until Sprints 22–31i are green. _(Renumbered 32 → 32i: Sprint 32 is the delivered Mechanism-Source Content Binding above.)_

Scenario:

```text
experience_to_consolidation_to_staleness_lifecycle
```

Trace must show:

```text
experience enters
raw episode preserved
semantic candidates extracted
trust/provenance attached
indexed five ways
retrieved under task pressure
action/outcome tested
revision only with evidence
stable consolidation
later stale demotion/forgetting
```

Tests:

```text
lifecycle_trace_contains_all_required_phases
all mutations_go_through_gateway
all authority_changes_cite_evidence
snapshot_shows_current_state_each_phase
audits_replay_end_to_end
```

Acceptance:

```text
one command proves full lifecycle
release_check requires it
```

## Sprint 33 — Runtime Packaging and Deployable Local Service

Status: Not started.

Goal: Cognitive OS can run as a local deployable runtime, not only scripts.

Build:

```text
cognitive_os_runtime.py
local CLI entrypoint
scenario runner
recovery runner
snapshot runner
config loader
key loading boundary
```

Operator commands:

```sh
cognitive-os run-scenario <name>
cognitive-os snapshot <trace>
cognitive-os replay-recovery --ledger <path>
cognitive-os verify-release
```

Tests:

```text
runtime_runs_scenario
runtime_generates_snapshot
runtime_replays_recovery
runtime_rejects_untrusted_config
runtime_handles_missing_keys_as_audit_only
```

Acceptance:

```text
fresh clone can run local runtime
no cloud dependency
no hardcoded secrets
release_check invokes runtime path
```

## Sprint 34 — Security and Boundary Red-Team

Status: Blocked until Sprint 33 is green.

Build:

```text
red_team_scenarios/
security_audit.py
BOUNDARY_THREAT_MODEL.md
```

Required attacks:

```text
direct_memory_mutation
llm_authority_injection
config_authority_injection
ledger_signature_forgery
stale_memory_full_premise_attack
retrieval_similarity_overrides_authority
consolidation_without_stability
forgetting_deletes_raw_evidence
```

Acceptance:

```text
all attacks blocked or downgraded
all failed attacks visible in audit/snapshot
release gate includes red-team suite
```

## Sprint 35 — v1.0 Release Candidate

Status: Blocked until Sprint 34 is green.

Build:

```text
VERSION
RELEASE_NOTES.md
OPERATOR_RUNBOOK.md
V1_ACCEPTANCE_REPORT.md
KNOWN_LIMITATIONS.md
```

Release checks:

```text
full regression suite
full red-team suite
full lifecycle scenario
snapshot/audit/recovery replay
secret scan
fresh-clone smoke test
docs links valid
no skipped critical tests
```

v1.0 acceptance:

```text
Cognitive OS v1.0 can ingest experience, preserve raw evidence, extract candidates, attach trust, index memory, retrieve under pressure, act/recommend under authority, test against outcomes, revise with evidence, consolidate when stable, demote/forget when stale, and explain current state through snapshot/audit tools.
```

## Dependency Order

```text
20R  Sprint 20 review
21   asymmetric replay identity
22   raw experience ingestion
23   semantic candidate extraction
24   unified self-correction (the Caitlin Leap) — DELIVERED
24i  trust/provenance assignment (backlog, gated by the unified loop)
25   derived effect classification — DELIVERED
25i  multi-index memory (backlog, gated by the unified loop)
26   trace-grounded invariant diff — DELIVERED
26i  retrieval under task pressure (backlog, gated by the unified loop)
27   complete locked-invariant probe coverage — DELIVERED
27i  outcome testing harness (backlog, gated by the unified loop)
28   delta-to-code provenance — DELIVERED
28i  evidence-only revision (backlog, gated by the unified loop)
29   artifact content-hash binding — DELIVERED
29i  stable consolidation (backlog, gated by the unified loop)
30   signed change provenance — DELIVERED
30i  staleness/demotion/forgetting (backlog, gated by the unified loop)
31   signer-set governance — DELIVERED
31i  LLM linguistic boundary (backlog; also built as the P9–P15 codec track)
32   mechanism-source content binding — DELIVERED
32i  full lifecycle scenario (backlog)
33   deployable runtime
34   red-team boundary audit
35   v1.0 release candidate
P0–P15  prototype-first track — ADR-002 deterministic engine (P0–P8) + replaceable LLM codec (P9–P15); see "Prototype-First Track" below
```

This ladder is the original incremental sequencing. After the Caitlin Leap, Sprint 24 (unified self-correction) is delivered and the trust/provenance sprint is renumbered **24i**; Sprints 24i–35 are no longer a strict linear build but a backlog gated by the unified self-correction loop. Any of them must enter as a design proposal and pass the Sprint 24 gate before it is built.

## Plan Self-Verification

Lifecycle coverage:

```text
experience enters                              Sprint 22
raw episode preserved                          Sprint 22
semantic candidates extracted                  Sprint 23
trust/provenance attached                      Sprint 24i (backlog)
indexed by meaning/time/entity/causality/use   Sprint 25i (backlog)
effect derived from behavior, not words        Sprint 26 (DELIVERED)
every locked invariant probe-backed            Sprint 27 (DELIVERED)
tested delta bound to the real change set      Sprint 28 (DELIVERED)
tested delta bound to artifact content         Sprint 29 (DELIVERED)
change authorized by a signed identity         Sprint 30 (DELIVERED)
retrieved under task pressure                  Sprint 26i (backlog)
tested against outcome                         Sprint 27i (backlog)
revised only with evidence                     Sprint 28i (backlog)
consolidated only when stable                  Sprint 29i (backlog)
forgotten or demoted when stale                Sprint 30i (backlog)
end-to-end lifecycle proof                     Sprint 32i (backlog)
```

LLM boundary:

```text
LLM parses language only                       Sprint 31i / P9–P10
LLM renders explanations only                  Sprint 31i / P9
LLM cannot assign authority                    Sprints 23/24/31i + P9/P14/P15
LLM cannot write memory directly               Sprint 31i / P9
World stored in memory graph/indexes           Sprints 22-30
```

Deployability:

```text
runtime entrypoint                             Sprint 33
operator commands                              Sprint 33
config/key boundary                            Sprints 18-21 + 33
release gate                                   Sprint 35
red-team suite                                 Sprint 34
docs/runbook                                   Sprint 35
fresh-clone smoke                              Sprint 35
known limitations                              Sprint 35
```
## Prototype-First Track (P0–P15): the ADR-002 deterministic engine + replaceable LLM codec

Goal: turn the frozen v0.1 governance proof-of-concept into a working local prototype by building
the minimal deterministic engine [ADR-002](ADR-002-runtime-engine-replay-contract.md) charters
(L0–L2), preserving the replay/governance discipline already proven in Sprints 25–32 (L3).

DONE (engine half) means all of: (1) one local command feeds observations in and returns
deterministic evaluated output; (2) the full ADR-002 spine exists — `ObservationEnvelope →
IngressGate → TickScheduler → ScheduledObservation → FrameCollector → ObservationFrame →
evaluate_tick → VibeEngine → RunRecorder`; (3) replay reproduces the same frames, outputs, and
hashes; (4) the kernel stays pure replay math (no backend/network/signing/governance inside);
(5) a release gate covers lint, tests, replay determinism, scenario proof, docs, and no-secrets.

Layer map (ADR-002): P1 = L0 kernel boundary; P2–P4 = L1 ingress/scheduling/frames; P5 = L0
evaluation; P6 = L2 record/replay; P7–P8 = operator surface + release gate; P9–P15 = the LLM codec
boundary, outside every engine layer. **Not required before a working prototype:**
threshold/multi-signer governance, distributed backend, sagas, consistent hashing, gossip, vector
clocks, production API, billing/upload workflows, cluster replication — all deferred.

### P0 — Tag and snapshot the v0.1 governance milestone

Status: Not started. Correct if v0.1 is tagged/snapshotted, `GOVERNANCE_MILESTONE.md` stays frozen,
`release_check` is green + silent before the snapshot, and known caveats are preserved (not hidden).
Wrong if engine work starts before the frozen state is preserved, docs claim production readiness, or
caveats disappear. Work: create the v0.1 snapshot, record the exact commit/hash + `release_check`
result + known caveats. Acceptance: the v0.1 governance proof is recoverable before engine work
starts. _(Note: this repo is not currently a git repository — P0's snapshot mechanism is an explicit
sub-decision, e.g. `git init` + tag, or an archived tarball + recorded hash.)_

### P1 — Rust workspace skeleton and deterministic kernel boundary (L0)

Status: delivered (2026-06-14). `crates/vibe-core` builds with zero dependencies; 8 cargo tests prove determinism, purity (input state never mutated), seed-only noise, and the kernel-boundary source invariants. `release_check.sh` runs `cargo test` (silenced) plus a source-absence scan over `kernel.rs` and a zero-dependency-tree assertion; both sabotage probes (inject a wall-clock token / add a dependency) were confirmed to fail the gate, then reverted byte-identically. Correct if the Rust workspace builds, a kernel module exists with no
backend/network/storage/auth code, and scalar/state/tick primitives are deterministic with tests
proving repeated-run equality. Wrong if HTTP/API/storage/signing enters the kernel, randomness is
unseeded, wall-clock enters evaluation, or float nondeterminism appears where fixed-point is required.
Build: `crates/vibe-core` with `Scalar / Fixed / Tick / EngineState`, a `VibeEngine` skeleton, and
`evaluate_tick(state, frame) -> EngineOutput`. Tests: `same_state_same_frame_same_output`,
`tick_order_is_deterministic`, `no_wall_clock_in_core`, `no_randomness_without_seed`,
`kernel_has_no_backend_dependencies`. Acceptance: `cargo test` passes; the core is pure deterministic
math.

### P2 — ObservationEnvelope and IngressGate (L1)

Status: delivered (2026-06-14). `crates/vibe-ingress` (depends only on `vibe-core` value types) admits external input as a typed `ObservationEnvelope`, validating malformed → duplicate → sequence and returning an `Accepted`/`Duplicate`/`Rejected` receipt; only fully-valid, in-order, non-duplicate observations are staged. 6 cargo tests green (`valid_observation_accepted`, `malformed_observation_rejected`, `duplicate_event_id_idempotent`, `source_sequence_gap_detected`, `rejected_observation_does_not_mutate_state`, `ingress_does_not_call_evaluate_tick`). Admission-only: the ingress source references no engine type, never schedules ticks (P3), and never touches engine state — gated by a source-token scan and a `vibe-ingress → vibe-core` two-line dependency-tree assertion, both sabotage-probed. Correct if all external input enters as an `ObservationEnvelope`, the
`IngressGate` validates schema/source/sequence/admissibility, invalid observations are rejected
without mutating engine state, and accepted observations produce receipts. Wrong if raw external data
mutates state directly, invalid input partially enters the scheduler, or missing
source/session/idempotency fields are accepted silently. Build: `ObservationEnvelope`, `SourceSession`,
`EventId`, `source_sequence`, `IngressGate`, `AcceptedObservationReceipt`, `RejectedObservationReceipt`.
Tests: `valid_observation_accepted`, `malformed_observation_rejected`, `duplicate_event_id_idempotent`,
`source_sequence_gap_detected`, `rejected_observation_does_not_mutate_state`. Acceptance: input is
controlled before scheduling.

### P3 — TickScheduler and ScheduledObservation (L1)

Status: delivered (2026-06-14). `crates/vibe-scheduler` (depends only on `vibe-core` + `vibe-ingress`) orders staged observations onto future logical ticks: `TickScheduler::schedule(now, request)` validates duplicate → target-required → strictly-future → bounded-horizon → overload, placing only valid in-window non-duplicate requests and returning an `Scheduled`/`Duplicate`/`Rejected` receipt otherwise. Determinism via `BTreeMap` tick lanes; `now` is a supplied logical tick, never wall-clock. 7 cargo tests green (`schedule_same_inputs_same_order`, `target_tick_required`, `future_horizon_enforced`, `overload_rejected_with_receipt`, `duplicate_schedule_idempotent`, `scheduler_does_not_call_evaluate_tick`, plus `scheduler_does_not_mutate_state`). Gated by a source-token scan + a workspace-only dependency-tree assertion; both a token and a behavioral (overload off-by-one) sabotage were probed. Correct if accepted observations are scheduled to deterministic target ticks, the
future horizon is bounded, the same input order produces the same schedule, and overload is
rejected/quarantined (never silently dropped). Wrong if scheduling is unbounded or wall-clock-based,
queue order depends on runtime timing, or overload disappears without a receipt. Build: `TickScheduler`,
`ScheduledObservation`, bounded horizon, deterministic ordering, overload receipt. Tests:
`schedule_same_inputs_same_order`, `target_tick_required`, `future_horizon_enforced`,
`overload_rejected_with_receipt`, `duplicate_schedule_idempotent`. Acceptance: observations are
deterministic before they reach frames.

### P4 — FrameCollector and ObservationFrame (L1)

Status: delivered (2026-06-14). `crates/vibe-frame` (depends only on `vibe-core` + `vibe-ingress` + `vibe-scheduler`) folds the observations scheduled for one tick into a canonical, hash-stable `ObservationFrame`: `FrameCollector::collect(tick, &[ScheduledObservation])` filters by `target_tick == tick`, sorts by `(event_id, signal)`, and FNV-1a hashes `(tick, len, per-obs event_id + signal)`. Empty ticks yield an explicit empty frame. 8 cargo tests green (the 6 named + `different_content_different_frame_hash` and `repeated_identity_canonicalized_deterministically`, added to close gaps a fresh-context adversarial panel surfaced — the panel confirmed 0 rubric-breaking defects). Gated by a source-token scan + a workspace-only dependency-tree assertion; a token and a canonicalization (remove-sort) sabotage were both probed. NOTE: this canonical frame lives in L1 for P4; P5 promotes it into vibe-core (L0) and rewires the engine, retiring the P1 stub frame. Correct if all scheduled observations for a tick become one canonical
`ObservationFrame`, frame ordering is stable, the frame hash is reproducible, and empty ticks are
handled explicitly. Wrong if the engine consumes loose observations, the frame hash changes across
equivalent runs, or empty-tick behavior is implicit. Build: `FrameCollector`, `ObservationFrame`,
`frame_hash`, canonical ordering, explicit empty-frame representation. Tests:
`same_tick_same_frame_hash`, `different_order_same_canonical_frame`, `empty_tick_frame_is_explicit`,
`frame_contains_only_scheduled_observations`. Acceptance: `evaluate_tick` receives frames, not raw
input.

### P5 — Minimal VibeEngine evaluation loop (L0)

Status: delivered (2026-06-14). The canonical `ObservationFrame` (prototyped in L1 in P4) was promoted into `vibe-core` (L0) as the SINGLE frame definition — `FrameObservation { id: u64, signal }` keeps the kernel dependency-free, and `ObservationFrame::new` owns the canonical sort + FNV hash. The P1 stub frame was retired; `vibe-frame::FrameCollector` now produces the L0 type (re-exporting it, defining none). `VibeEngine::evaluate_tick(&state, &frame)` folds the frame's observation signals, returns a new `EngineState` + `EngineOutput { tick, vibe, noise, frame_hash, transition, output_hash }`, with an explicit deterministic `StateTransition` and an order-independent `output_hash`. 12 vibe-core + 9 vibe-frame cargo tests green (incl. `engine_consumes_canonical_frame`, `state_transition_explicit`, `input_state_not_mutated`, `output_hash_changes_when_frame_changes`, `core_still_has_no_backend_dependencies`, and end-to-end `collected_frame_is_consumable_by_engine`). release_check gates a single ObservationFrame definition; a competing-definition and a broken-fold sabotage were both probed; a fresh-context adversarial panel confirmed 0 rubric defects. Correct if `VibeEngine` consumes an `ObservationFrame` and emits a deterministic
`EngineOutput`, the state transition is explicit, the output hash is reproducible, and one scenario
proves state evolves across ticks. Wrong if there is hidden mutable global state, output depends on the
environment, or state updates happen outside `evaluate_tick`. Build: `VibeEngine`, `evaluate_tick()`,
`EngineOutput`, `StateTransition`, `output_hash`. Tests: `same_run_same_outputs`,
`state_transition_explicit`, `no_external_mutation`, `multi_tick_scenario_reproducible`. Acceptance:
the deterministic engine actually runs.

### P6 — RunScript, RunRecorder, and deterministic replay (L2)

Status: delivered (2026-06-15). `crates/vibe-run` (depends on all four lower crates) records and replays a deterministic run. `RunRecorder::record(script)` drives the full pipeline (ingress admits → scheduler orders → collector canonicalizes frames → engine evaluates), recording accepted/scheduled observations, per-tick frames + outputs (with transitions + hashes), and a `run_hash`. `ReplayRunner::replay(recorded)` re-runs ONLY the engine over the RECORDED frames (no re-admission, no re-scheduling, no live input) and reports `verified = run_hash_matches && no output_mismatches && no tick_mismatches`. Tampering is detected three ways (output, run_hash, frame) plus an internal-consistency check (a relabelled tick whose label disagrees with its frame/output). Both DRIVE the one engine — gated so the source defines no `evaluate_tick` and reimplements no `split_mix64`. 8 cargo tests green; sabotage (defeat tamper-detection; make replay re-collect from live scheduled) both caught; a fresh-context adversarial panel confirmed 0 rubric defects (authenticity-under-active-forger stays the L3/S30 signing concern). Correct if a `RunScript` drives the whole pipeline, `RunRecorder` records accepted
observations/frames/outputs/hashes, and replay reproduces the same result hash-for-hash. Wrong if replay
depends on live input, a recorded run cannot reconstruct frames, or replay output differs without
explanation. Build: `RunScript`, `RecordedRun`, `RunRecorder`, `ReplayRunner`, `run_hash`,
`replay_report`. Tests: `record_then_replay_same_hash`, `replay_reconstructs_frames`,
`replay_reconstructs_outputs`, `tampered_recorded_run_detected`. Acceptance: the prototype is replayable,
not just runnable.

### P7 — Local CLI prototype

Status: delivered (2026-06-15). `crates/vibe-cli` exposes the `vibe` binary: `vibe run <scenario.json> [out.json]` ingests a scenario, records the deterministic run, and writes a recorded-run file; `vibe replay <run.json>` re-derives the run from the recorded frames alone (via `vibe_run::RunRecorder::record_from_frames`, which rebuilds each frame through `ObservationFrame::new` and re-runs the engine) and reports MATCH/MISMATCH; `vibe verify <run.json>` exits non-zero on tamper. serde/serde_json live ONLY in the CLI (the IO layer) via primitive DTOs — the engine crates stay dependency-free, gated by a serde-confinement check. 5 cargo tests (run/writes/replay-matches/verify-detects-tamper/malformed-rejected) + an end-to-end binary smoke (run→replay MATCH→verify OK→tampered exit 1). Sabotage (defeat verify; leak serde into the engine) both caught. Correct if an operator can run one local command that ingests a scenario, evaluates
ticks, writes a recorded run, replays it, and prints a concise report. Wrong if manual script chaining is
required, or there is no operator-facing run/replay command, or output cannot be inspected. Build:
`vibe run <scenario>`, `vibe replay <recorded_run>`, `vibe verify <recorded_run>`. Tests:
`cli_run_scenario_succeeds`, `cli_writes_recorded_run`, `cli_replay_matches_original`,
`cli_verify_detects_tamper`. Acceptance: a working local prototype exists from the operator's point of
view.

### P8 — Prototype release gate

Status: delivered (2026-06-15). `scripts/release_check.sh` is the single prototype release gate: it runs lint + the Python suite + governance/doc gates (P1-era through S32) AND the P1–P7 Rust suite (cargo test + fmt + clippy + dependency boundaries + serde confinement), then P8 consolidates the proof surface with (a) an end-to-end `vibe` binary smoke — `run` → `replay` (MATCH) → `verify` (authentic) → a tampered run MUST be rejected, exercising replay determinism through the recorded-run path — and (b) a no-secrets scan (no committed `.env`/`*.pem`/`*.key`/`id_*` files; no `BEGIN PRIVATE KEY`/`AKIA…`/`aws_secret_access_key` in the Rust tree). Green + silent on the clean tree (exit 0, 0 stdout, 0 stderr). Three sabotage probes each fail the gate: a broken `verify` (always-match) → exit 101; serde added to `vibe-core` → exit 101; a planted secret `.env` → exit 1. No engine math, replay semantics, observation semantics, or CLI behavior changed — P8 only wired checks. Correct if one release command runs Rust tests, the Python governance checks,
replay determinism, a no-secrets scan, docs checks, and the scenario proof — green and silent — and
failure is non-decorative (sabotage probes fail it). Wrong if verification is manual, the gate ignores
the Rust engine or replay, or it passes after sabotaging deterministic replay. Build: `release_check.sh`
extended for the prototype (cargo test + scenario replay integrated; governance checks retained;
environment assumptions documented). Tests: `release_check_green_silent`,
`sabotage_engine_determinism_fails`, `sabotage_replay_hash_fails`, `sabotage_ingress_validation_fails`,
`sabotage_docs_required_file_fails`. Acceptance: the prototype has the same verification discipline as the
governance milestone.

### LLM codec sub-track (P9–P15) — insert the language interface, train only if forced

The LLM is a replaceable language codec at the human-language boundary (ADR-002), never world memory,
authority, the mutation gateway, the verifier, the replay ledger, the scheduler, or the state engine.
Order: insert the untrained codec **after P8**; decide on training only **after P11/P12**. Flow:
`human text → LLM codec → typed packets / ObservationEnvelope → deterministic engine + governance →
audit packets → LLM explanation renderer → human-readable explanation`. The strongest rule: never train
cognition into the LLM; train only the language interface; keep the world in inspectable memory and
replayable state. The constraint-engineering discipline for any training decision is the Appendix below.

- **P9 — Language-codec boundary.** Correct if the LLM proposes typed packets but has no direct state
  mutation, cannot assign authority, and cannot bypass `ObservationEnvelope`/`IngressGate`; explanations
  render only from audit/replay/snapshot evidence. Wrong if a natural-language answer changes
  memory/state directly, LLM confidence becomes evidence, or hidden context becomes world storage. Tests:
  `llm_intent_to_observation_envelope`, `invalid_llm_packet_rejected`, `llm_cannot_call_mutation_gateway`,
  `llm_cannot_assign_epistemic_license`, `llm_explanation_requires_audit_source`.

  Status: delivered (2026-06-15). Built on the Reading Substrate track (READ-0) as `crates/reading-codec`
  — the boundary/IO layer for the reading line (serde allowed here, never in the engine or substrate
  cores). A strict, deterministic codec maps **untrusted model text** → typed `ReadingAction` proposals:
  invalid JSON / free-form prose / unknown action / missing-or-mistyped fields are rejected with a
  precise, recorded reason and **never silently repaired**. Referenced span ids are checked against the
  corpus *before* execution; an `extract_claim`/`extract_entity` with no source span is rejected.
  Accepted actions execute **only** through `reading_substrate::execute` (the codec mutates no memory
  itself), and a synthesized answer **finalizes only if `reading_substrate::verify` approves it**
  (grounded + supported + replayable) — so a prompt-injection that asserts an ungrounded answer is
  refused at the finalize gate. A `CodecPolicy` carries the three guards (reject-unknown,
  require-source-spans, require-verified-finalize); production builds only the strict policy, and
  `#[cfg(test)]` weakened constructors drive the **3 sabotage probes** (disable any one guard → the eval
  battery fails). The **eval harness** (`evaluate` + the runnable `eval_report` example) scores the
  required 10-fixture battery — valid inspect/read, malformed, unknown action, missing field, nonexistent
  span, ungrounded claim, synthesize-before-verify, prompt-injection override, and the full valid
  sequence (which must reproduce the canonical READ-0 answer + trace) — checking the **reason** for each
  rejection, not just that it was rejected. **No trained weights, no RL, no live-model dependency**;
  model output is untrusted strings. 13 cargo tests; `release_check` gates it (test + fmt + clippy +
  substrate-is-only-executor + purity/no-network + no-ML-dependency + separation: depends on
  reading-substrate, no `vibe-*`). READ-0 and the P8 engine gate stay green; substrate + engine 0-diff.
  (The map onto the engine packet names — `ObservationEnvelope`/`IngressGate`/mutation gateway — is the
  P10+ adapter's job; READ-0/codec proves the **shape** of the boundary on the reading substrate first.)

  **READ-1 claim-fidelity hardening (folded into this milestone).** An ultracode adversarial panel (7
  agents) confirmed — and the build reproduced first-hand — that READ-0's grounding was purely
  *structural*: a claim was "grounded" merely by citing a span that existed and was read; the claim's
  `statement` was never compared to the span's actual text. So a fabricated claim citing a real read
  span (`extract_claim "Bridge A is fully safe…" cite [0]` against a span that says Bridge A was
  *damaged*) finalized a verifier-approved answer that contradicted the source — the exact "model
  confidence becomes evidence" failure the boundary exists to stop. Because P9 is the first milestone
  that accepts untrusted, model-shaped text, shipping a codec that faithfully routes such claims through
  a structurally weak verifier would be a known-unsafe milestone, so the fix was folded in here rather
  than deferred. Fix (deterministic floor, no semantic entailment / no LLM judge): `reading_substrate`'s
  verifier now reads the cited span **text** and requires each claim's statement to be a **literal
  substring of a single cited span** — spans are checked individually, never concatenated, so a
  statement cannot be "grounded" by text that straddles the join of two spans and exists in neither —
  with minimal normalization (collapse whitespace, lowercase); the canonical READ-0 claims were
  rewritten as verbatim span excerpts, and the codec's accepted fixture uses verbatim support.
  Exploit-regression probes pin it: a fabricated claim citing a real span (`grounded_injection_fabricated_claim`)
  and a cross-span-join straddle (`sabotage_cross_span_join_straddle_fails_fidelity`) both fail
  grounding; disabling the fidelity check fails `release_check`. Boundary rule: **P9 may accept untrusted
  language only because READ-1 verifies cited-text support.** Paraphrase / semantic entailment is
  explicitly a later sprint. (The per-span tightening was surfaced by the P10 adversarial panel, which
  found the original concatenation check admitted boundary-straddling statements.)
- **P10 — Baseline local LLM adapter (zero training).** A local model parses requests into candidate
  typed packets at `temperature = 0`, structured/schema output, no autonomous tool calls, no write
  authority; bad output is rejected cleanly. Acceptance: the baseline works as a *proposed* translator,
  the deterministic engine still decides everything, `release_check` stays green.

  Status: delivered (2026-06-15). `crates/reading-adapter` inserts the model backend as a REPLACEABLE
  boundary in front of the P9 codec — "a model backend, not a smarter authority." A `ModelBackend` trait
  has one job: `propose(task) -> String` (untrusted candidate reading-action text). The `Adapter` routes
  that text through one and only one path — `reading_codec::decode` (which validates it into typed
  actions, executes them via the substrate, and finalizes an answer only if the READ-1 verifier
  approves) — and does nothing else with it; it holds no executor/verifier/finalizer handle, assigns no
  authority, and mutates no memory (gate-enforced by a source scan: zero `execute(`/`verify(`/`finalize(`
  calls in the adapter, and it routes through `decode`). The default `ScriptedBackend` replays a recorded
  model response verbatim (temperature-0-equivalent → deterministic, offline, reproducible eval). The
  optional `local-model` feature provides a real off-the-shelf local model as a subprocess (an explicit
  argv — no shell, no injection — with the corpus *metadata* + question as the prompt and the model's
  stdout returned as untrusted text); it is OFF by default and is **never executed by `release_check`**
  (only compiled + linted under `--features local-model`), so the gate stays offline and deterministic.
  The **baseline failure-profile eval** (`baseline_report` + the runnable `baseline_report` example)
  scores a battery of recorded model outputs against the hardened codec/verifier and records the score +
  failure categories — baseline today: **1 finalized / 7**, 1 accepted-partial, **5 rejected** (one each
  of Malformed, UnknownAction, UnknownSpan, Ungrounded, and — critically — a fabricated-but-cited claim →
  **Unverified** via READ-1). **No training, no RL, no live-model dependency in the verified build.** 7
  cargo tests (the operator's first-test list); `release_check` gates everything and a live bypass
  sabotage (adapter reaching the substrate executor directly) fails it. P8/P9/READ-0/READ-1 stay green;
  the vibe engine, the codec, and the substrate are 0-diff. Boundary rule: **a model may only ever
  propose; the codec + READ-1 verifier decide.** Training is deferred to P12+ (only if P11's harness
  exposes clean reproducible failures).
- **P11 — LLM codec eval harness.** Build the 30–100 case harness before any training: cover valid
  observation creation, ambiguous request, unsafe authority request, memory-mutation attempt, unsupported
  claim, explanation-from-audit, explanation-without-evidence, bad JSON/schema, wrong target tick, correct
  refusal. The scorer compares to committed ground truth; the model cannot self-grade. Acceptance: a
  reproducible baseline score with false-accepts visible and failures classified.

  Status: delivered (2026-06-15). `crates/reading-eval` is the scorer for the model-codec boundary. Each
  of **34 committed fixtures** (`src/fixtures.rs`) is raw untrusted proposal text plus a COMMITTED
  expected `Disposition` (Finalized / AcceptedPartial / Rejected(kind)) — the ground-truth label lives in
  source, never inferred from the model's prose. The scorer (`src/scorer.rs`) runs each fixture through
  the P10 adapter (→ `reading_codec::decode` → substrate → READ-1 verifier) and classifies the codec's
  actual decision against the committed label into **Correct / FalseAccept / FalseReject**. A
  **false-accept** (a should-reject output that was accepted or finalized) is the unsafe class: it is
  surfaced as an explicit list, never folded into the aggregate score, and the acceptance target is **0
  false-accepts**. False-rejects are allowed but classified by cause (the actual rejection reason). The
  battery covers all ten categories — valid action, correct finalization, malformed JSON, unknown action,
  missing fields, bad span, ungrounded claim, fabricated cited claim, premature synthesize, prompt
  injection — and the report carries the score, the false-accept / false-reject lists, the per-category
  tallies, the failure-category histogram, and a single deterministic **`next_change`** recommendation
  (which never recommends training: it stays forbidden until a recurring real failure survives the
  schema/prompt/tooling/fixture/verifier classification). Current result: **34/34 correct, 0
  false-accepts, 0 false-rejects.** 9 cargo tests including controls that prove the scorer grades against
  the committed label, not the model text (a deliberately-mislabelled fixture is reported as a
  false-reject; a valid output labelled "must reject" is reported as a false-accept). `release_check`
  gates it (test + fmt + clippy + a runnable `eval_report` example that exits non-zero unless ≥ 30
  fixtures and 0 false-accepts + a source-level ≥ 30 floor + purity + no-ML-dep + separation); a live
  sabotage that hides false-accepts (mis-classifying them as correct) fails the gate (exit 101). No
  model, no training, deterministic. P8/P9/P10/READ-0/READ-1 stay green; every prior crate is 0-diff.
  Hard rule honored: **training stays forbidden until P11 + P12 prove clean recurring failures not caused
  by schema, prompt, tooling, fixtures, or verifier defects** — and on this battery there are none.

  **READ-2 sentence-fidelity (folded into this milestone).** A P11 adversarial panel found — and the
  build reproduced first-hand — that the prior literal-substring floor admitted a real false-accept
  class the original 34 fixtures did not probe: a finalized answer assembled from verbatim
  **sub-fragments**. Each claim was individually a literal substring of one cited span (so per-claim
  grounding passed), but nothing required the claim to be a complete unit, so juxtaposing fragments
  finalized false answers — e.g. claim `"Bridge A"` (span 0) + claim `"remained passable"` (span 1) →
  `"Bridge A remained passable"` (Bridge A is in fact *damaged*); a lone fragment `"using Bridge A"`
  also finalized. This was *not* the deferred paraphrase/entailment limitation (zero paraphrase — all
  tokens verbatim), so it was hardened before commit, exactly as in P9/P10. Fix (`reading_substrate`'s
  verifier, still deterministic — no LLM, no semantics): a claim is grounded only if its normalized
  statement equals a **contiguous run of one or more of a single cited span's complete sentence units**
  (sentence-boundary-aligned literal support), which rejects arbitrary fragments and cross-fragment
  compositions while accepting every legitimate full-sentence claim. Three fixtures were added —
  `fc_compound_fragments` (the two-fragment composition), `fc_single_fragment` (a lone fragment), and
  `cf_full_sentence_span2` (a valid full-sentence control) — and the substrate carries fragment +
  negation-adjacent-fragment probes. The eval now reports **37/37 correct, 0 false-accepts** *after*
  adding the previously-missed class; disabling the sentence-boundary check reintroduces the
  false-accepts and fails `release_check` (the P11 example exits 1; gate exit 101). Grounding contract:
  **READ-1** = claim text must be found in cited span text; **READ-2** = claim text must be a complete
  sentence-level support unit. Paraphrase / semantic entailment remains a later sprint.
- **P12 — Training-justification gate.** Training is allowed only if: the task was specified, examples were
  representative, context was present, prompt/schema/tooling were stable, the eval caught the failure, and
  the baseline still repeatedly failed the same pattern. Not justified if failures trace to bad
  schema/prompt/examples/eval-labels/task-definition/tooling/context. Acceptance: the decision cites exact
  failed cases; no training without clean failures.

  Status: delivered (2026-06-15). `crates/reading-train-gate` is the deterministic, machine-checkable
  gate. `decide(false_accept_ids, diagnoses)` returns a `TrainingDecision` whose load-bearing field is
  the bool `training_justified` (plus `safety_fix_required`, `cited_failures`, structured `blockers`, and
  a derived one-line `reason` — the bool decides, not the prose). A residual failure is a `CleanModelFailure`
  only if it survived cleanup of every fixable cause — `BadFixture`, `SchemaDefect`, `PromptDefect`,
  `ToolingDefect`, `MissingContext`, `VerifierWeakness` — AND recurs (≥ `MIN_RECURRENCES` = 2). The
  decision logic: any **false-accept** sets `safety_fix_required` and blocks (the cure is hardening the
  verifier, never training); each diagnosed residual failure is either a clean candidate or a named
  blocker citing its cause; **zero failures** yields a "no unresolved failures" block; an undiagnosed
  false-reject (via `decide_from_report`) is forced to a non-clean cause so it blocks until triaged.
  `training_justified` is true **only** with ≥ 1 clean candidate AND zero blockers — so a single
  remaining defect, a non-recurring clean failure, or a not-survived-cleanup failure all block, and the
  decision always names exact fixture ids. `decide_from_eval()` runs the live P11 battery: 0 false-accepts,
  0 residual ⇒ **training_justified = false** ("no unresolved failures — no clean residual to justify
  training"). 12 cargo tests (the six first-tests — no-failures / false-accept-needs-safety-fix /
  eval-design / schema / clean-recurring-candidate / cites-fixture-ids — plus verifier-weakness,
  single-occurrence, not-survived, mixed-defect, current-battery, determinism). `release_check` gates it
  (test + fmt + clippy + a runnable `decision_report` example that refuses an unjustified "yes" + purity +
  no-ML-dep + separation); a live sabotage that ignores blockers fails the gate (exit 101). **No model is
  trained.** Hard doctrine in code: **no failed cases → no training; verifier defect → no training; schema
  defect → no training; only a clean recurring codec failure can justify weights.** The current verdict is
  a firm "not justified", which is correct: there is no clean residual failure to train against.

  Phantom-diagnosis hardening (P12 adversarial panel, 2026-06-15): the panel found — and the build
  reproduced first-hand — that `decide_from_report` admitted any reviewer-supplied diagnosis without
  checking it corresponded to a real failure, so a clean eval (0 false-accepts, 0 residual) could be
  coerced to `training_justified=true` by injecting one "phantom" diagnosis citing a fixture the eval
  never failed. The production path (`decide_from_eval`, always `&[]`) was already safe, but the public
  helper's contract was broken, so it was hardened before commit: `decide_from_report` now admits a
  diagnosis only if its `fixture_id` matches an actual residual failure (false-reject) in the report;
  a phantom becomes a `phantom_diagnosis` blocker and can never justify training. Pinned by
  `phantom_diagnosis_cannot_justify_training_on_clean_eval` (plus valid-admission and
  undiagnosed-residual tests). 15 cargo tests.
- **P13 — Local LoRA/adapter candidate (only if justified).** Train a small local adapter for the
  language-codec task only, from accepted eval traces + corrected examples; it emits typed packets or
  explanations only — no world facts as authoritative memory. Correct if codec accuracy improves without
  raising false-accepts and authority-injection tests still fail closed. Wrong if it becomes a memory
  store, gains direct mutation authority, or passes more unsafe requests.
- **P14 — Shadow-mode insertion.** Baseline and trained model both produce candidate packets; only the
  existing verified path controls execution; differences are logged; the trained model cannot affect state.
  Acceptance: it improves target cases with no new false-accepts and no hidden state mutation.
- **P15 — Promotion / rejection gate.** Promote only if it beats baseline on held-out eval, has zero
  critical authority-bypass failures, stays replaceable, and changes engine behavior only through valid
  typed packets. On failure: reject, keep baseline, record failure cases, do not tune weights blindly.

## Reading Substrate Track (READ-0): the model reads external text as an environment

A separate prototype track, placed **after P7/P8** (it needs the deterministic run/replay + verify
discipline, but NOT trained weights) and serving as the **bridge between the deterministic substrate
and the P9–P15 LLM track**. It gives a small/medium model the ability to read, inspect, decompose,
remember, compare, and verify external text without swallowing it into context.

Load-bearing invariant: **answer authority comes from verified source-linked memory, not from model
confidence.**

The substrate boundary (mirrors ADR-002's layering doctrine):

```text
Model          = reasoning / controller (a planner over the reading environment, NOT the memory store)
External Text  = addressable environment (documents → sections → spans, exposed metadata-first by handle)
Memory         = structured evidence state (notes, claims, entities, evidence links, contradictions, proof objects)
Reader Loop    = inspect → chunk → recurse → extract → compare → synthesize
Verifier       = grounding / citation / completeness / contradiction / trace-replay / no-free-prior judge
Training       = DOWNSTREAM of the verifier: imitate verified successful traces first, RL on verified tasks only
```

Hard rules: no claim enters durable memory without ≥1 source span; the controller may not answer from
prior knowledge when the task requires reading; the verifier gates every final answer; **no weights are
trained until the harness exposes clean, reproducible failures** (the constraint-engineering discipline
in the Appendix below). The probabilistic hypothesis layer ([P16](#)) sits ABOVE this, never inside it.

### READ-0 — External Text Reading Trace

Status: delivered (2026-06-15). `crates/reading-substrate` — a SEPARATE track from the vibe engine (zero-dependency; depends on no vibe crate; gated). Modules `corpus` (documents → addressable spans, metadata-first, read-by-id), `memory` (claims/entities/proof — a claim cannot exist without ≥1 source span), `trace` (the inspect→read→extract→compare→synthesize action log + a deterministic executor that enforces metadata-first and grounded extraction, with content hashes), `verify` (grounding + answer-support + trace-replay, the authority boundary). A scripted deterministic reader (the `fixture`) answers a fixed question over a fixed corpus, producing a source-linked answer; `verify` passes only if every claim is grounded, the answer text is exactly its cited claims' statements, and re-executing the trace reproduces the same memory + answer hashes. **9 cargo tests** including the 3 required sabotage probes — remove a claim's source span → grounding fails; reorder the trace → replay fails; add an unsupported answer sentence → support fails. No model weights trained; the scripted reader stands in for the eventual LLM controller (P9–P15). release_check gates it (test + fmt + clippy + separation); the P8 engine gate still passes. Goal: given a folder of documents and one
question, produce a **verified, replayable answer from source-linked structured memory**.

DONE means all of: (1) documents loaded as external addressable spans; (2) the controller receives
metadata first, not full text; (3) it selects spans to inspect; (4) claims/entities/evidence-links/proof
object extracted with source spans; (5) claims are source-linked; (6) claims compared across spans;
(7) an answer synthesized from verified claims; (8) the verifier checks grounding + completeness;
(9) the full trace is saved; (10) replay of the trace reproduces the same memory and answer.

Non-goals: no fine-tuning, no RL, no autonomous permanent belief updates, no giant-context dump, no
"it seems smarter" evaluation. (Full module breakdown — corpus/controller/memory/verifier/traces/training
— is the operator's "Reading Substrate v0" spec; recorded in project memory.)

### READ-3 — Real Corpus Reading CLI

Status: delivered (2026-06-15). `crates/reading-cli` (binary `read0`, serde as the IO layer like vibe-cli)
runs the reading substrate on a **real folder of documents** through a local command. `read0 run
<docs_dir> <question> <plan.json> <out.json>` loads every `.txt` file in the folder (path-confined:
canonicalize + the resolved path must stay under the folder + regular files only — no traversal/symlink
escape), builds a corpus of **one sentence per span** using the substrate's **shared `split_sentences`**
(the single source of sentence boundaries now used by BOTH the corpus builder and the READ-2 verifier, so
spans and grounding can never drift), then routes the **untrusted reading plan ONLY through
`reading_codec::decode`** — which validates it, executes it through the substrate, and finalizes an answer
only if the READ-1/READ-2 verifier approves. It writes a replayable run: documents (title + sentence
spans), the plan, the answer, memory/answer hashes, and a **verifier receipt** (grounded / answer-supported
/ replay-matches / passed). `read0 verify` and `read0 replay` rebuild the corpus from the stored spans,
**re-decode the stored plan through the codec**, and reject any tamper (an edited answer/hash → mismatch;
an edited span or a fabricated plan → grounding fails on re-decode). The plan never reaches memory except
through the codec, and `read0` calls no substrate executor directly (gate-asserted: zero `execute(` in the
CLI source). **9 cargo tests** (real-folder→verified-answer, metadata-before-read, one-sentence-per-span,
fabricated-claim-rejected, fragment-claim-rejected [READ-2], replay-and-tamper-caught, span-tamper-fails)
plus the substrate's 15. `release_check` gates it with a deterministic end-to-end binary smoke (build a
temp corpus, run→verify→replay, a fragment plan MUST be rejected, a tampered run MUST fail verify) +
codec-only + no-ML + separation; a live sabotage that neuters the tamper check fails the gate. No model,
no training, no paraphrase/entailment, no autonomous memory. All prior crates are 0-diff except
`reading-substrate`, which gained only the shared `split_sentences` (behavior-identical; its 15 tests stay
green). Boundary held: `read0` may load files, split sentences, build metadata, call codec/substrate, and
emit a trace/proof/receipt — it may NOT bypass the codec, weaken READ-1/READ-2, invent claims, or finalize
without a verifier pass.

### READ-4 — Real Corpus Eval Pack

Status: delivered (2026-06-15). `crates/reading-corpus-eval` turns `read0` from a single hand-run demo
into a **measured reading benchmark** over many real corpora. **14 committed fixtures** (`src/pack.rs`,
≥ 10 required) each carry a real document set + a question + an untrusted reading plan + a COMMITTED
expected verifier result (`Verified` or `Rejected`) — the label lives in source, never inferred from any
model output. The scorer (`src/scorer.rs`) materializes each fixture to a real docs folder and drives it
through the **actual read0 pipeline — `run` → `verify` → `replay`** (`reading_cli::run_reading` /
`verify_run` / `replay_run`): a Verified outcome is produced only after the answer finalizes AND
re-verifies AND replays, so **every fixture is replayed**, not hand-demoed once. Each result is compared
to its committed label into **Correct / FalseGrounded / FalseReject**, where a **false-grounded** answer
(an expected-rejected fixture that finalized a verified answer) is the unsafe class — surfaced as an
explicit list, **0 required**. The report lists, per fixture, the pass/fail verdict and either the
rejection reason or the verified answer + **trace hash** (the answer content hash). The fixtures span
weather, medical, infrastructure, finance, and safety corpora and cover valid single-span /
multi-sentence-doc / multi-document / compare-then-synthesize grounding, plus every rejection class
(fabricated claim, sub-sentence fragment, malformed plan, metadata-before-read, unknown action, bad span,
negation-dropped fragment, ungrounded claim). Current result: **13/13 correct, 0 false-grounded, 0
false-rejects** (5 verified, 8 rejected). 7 cargo tests including a control proving the scorer grades
actual-vs-committed (a genuinely valid plan labelled "reject" is flagged false-grounded, never silently
passed). `release_check` gates it (test + fmt + clippy + a runnable `pack_report` example that exits
non-zero on < 10 fixtures or any false-grounded + a source-level ≥ 10 floor + separation); a live sabotage
that hides false-grounded answers fails the gate (exit 101). Deterministic (fixed content → fixed hashes;
workdir paths excluded), self-cleaning temp dirs, **no model, no training** — anecdotal failures here
never justify weights; the P12 training-justification gate (still "not justified") owns that decision.
All prior crates are 0-diff.

### READ-5 — Deterministic Sentence Splitter Hardening

Status: delivered (2026-06-15). The READ-4 panel surfaced (and the build verified) that the shared
`split_sentences` was a naive **period-splitter** — every `.` was a boundary, so `U.S.` split a document
into `["The U.", "S.", "economy is strong this year."]`. That was never a verifier bypass (grounding
stayed honest and the abbreviation sentence was correctly *rejected* as un-groundable-whole), but it was
a real quality/coverage gap for real corpora. READ-5 hardens the splitter using **only deterministic,
lexical signals — no semantics, no entailment, no model**. A `.` is a boundary unless: (a) it is inside a
decimal/version (digit before and after — `3.14`, `v1.2.3`); (b) the word before it is a known
abbreviation from a small fixed list (`Dr.`, `Mr.`, `Mrs.`, `Ms.`, `Prof.`, `St.`, `etc.`, `vs.`, `Inc.`,
… — deliberately excluding ambiguous words like "no"); or (c)/(d) it is a **single-letter token** (an
acronym letter / initial) immediately followed by a letter (`U.S`, `e.g`, `i.e`) or by a lowercase
continuation (`U.S. economy`). `!`/`?` are always boundaries. Rule (d) is **scoped to single-letter
tokens on purpose** (a READ-5 panel finding): a genuine multi-letter word always ends the sentence, so a
real boundary before a lowercase word — `Do not attempt. and try again.` — still splits into two, and a
period before a capitalized new sentence (`Cross Bridge A. Avoid Bridge B.`) still splits. Only known
abbreviations and single-letter acronym tails are held back. Because the splitter is a **single shared
function**, the corpus builder (one sentence per span)
and the READ-2 verifier (`sentence_units`) move together by construction — they can never drift. 7 new
substrate tests pin the behavior (22 total); in the READ-4 pack the abbreviation sentence now grounds as
a whole (`abbreviation_whole_sentence_valid`) while a fragment of it is still rejected
(`abbreviation_sentence_fragment_reject`), so **false-grounded stays 0**. `release_check` gates it
(substrate tests + a `fn is_period_boundary` source signal); a live sabotage reverting to naive
period-splitting fails it (exit 101). Boundary held: READ-5 improves deterministic text segmentation only
— no entailment, paraphrase, world truth, or model judgment. (A heavier abbreviation case — an
abbreviation NOT on the fixed list, or a decimal/initials pattern outside these rules — would still
mis-split; that is the residual of a finite lexical floor, fixable only by broadening the deterministic
rules, never by semantics.) vibe/codec/adapter/eval/train-gate/cli all 0-diff.

### READ-6 — Reader Autonomy v0

Status: delivered (2026-06-15). `crates/reading-autonomy` gives the system a first, **bounded** ability
to read on its own — **deterministically, with no model and no training**. The reader sees corpus
**metadata** (document titles + span ids) — never the full text — and proposes a reading plan as
**untrusted text** routed through one and only one path: `reading_codec::decode`. The codec validates the
plan into typed actions, executes them through the substrate, and finalizes an answer only if the
READ-1/READ-2 verifier approves. The reader holds no executor or verifier handle and **cannot finalize on
its own**; a fabricated or ungrounded claim it proposes is rejected by the same codec/verifier path
(pinned by `fabricated_autonomous_claim_is_rejected`). It is bounded by `ReaderBounds { max_steps,
max_spans, max_finalize_attempts }` — it can never read all text at once (it reads at most `max_spans`
spans, one `read_span` action at a time) nor run unbounded. The v0 strategy is intentionally simple:
inspect metadata, read up to `max_spans` spans by id, claim each span's sentence **verbatim** (one
sentence per span ⇒ READ-2 grounded), and make one bounded finalize attempt synthesizing the read
sentences — proving the **propose → codec → verifier → replay loop within bounds**, not intelligence
(a smarter reader is a later, gated step; weights stay untouched). 8 cargo tests cover metadata-first,
each bound (`max_spans=0` ⇒ no finalize; a tight `max_steps` ⇒ exactly one span; `max_finalize_attempts=0`
⇒ no answer), sentence-grounding, fabrication-rejection, and determinism/replay (same inputs → same plan
and same verified-run hashes). The runnable `autonomous_read` example must finalize a verifier-authorized
answer. `release_check` gates it (test + fmt + clippy + the runnable example + a codec-only source scan
[zero `execute(`/`verify(`, routes through `decode`] + the `ReaderBounds` struct + no-ML + separation); a
live sabotage that makes the reader fabricate is rejected by the codec, the example exits non-zero, and
the gate fails (exit 101). Hard boundary held: **autonomy proposes, the codec validates, the substrate
executes, the verifier authorizes, replay records — and weights remain untouched.** All prior crates are
0-diff; the READ-4/READ-5 packs stay green.

### READ-7 — Autonomous Corpus Eval Pack

Status: delivered (2026-06-15). `crates/reading-autonomous-eval` is the first **measurement of the
autonomous reader as a system**, not a single demo. It reuses the committed READ-4 corpus fixtures —
their documents, question, and **committed manual label** — but throws away their hand-written plans.
For each fixture it rebuilds the corpus exactly as `read0` does (`corpus_from_documents`, one sentence per
span) and runs the deterministic READ-6 reader against the question; the reader proposes its own plan and
routes it through the hardened codec. Every finalized answer is then **cross-validated**: a fresh
`reading_substrate::verify` pass AND a separate `independently_grounded` check (with *different* logic —
exact whole-cited-span equality, never calling `verify`/`sentence_aligned`) must BOTH agree it is
grounded, else it is flagged false-grounded. (A READ-7 panel correctly noted that re-running the *same*
`verify` couldn't catch a bug *in* `verify`; the independent cross-check closes that — a `verify` that
wrongly accepted a fragment would disagree with the exact-span check and be flagged.) So a false-grounded
answer is **measured, not assumed**. Each autonomous outcome is compared to the fixture's manual label and
classified: both-verified, both-rejected, **autonomous-verified-where-manual-rejected** (a safe
divergence), or **autonomous-rejected-where-manual-verified** (a classified false-reject).

The measured result (default bounds): the autonomous reader **verifies 15/15 with 0 false-grounded and 0
false-rejects**, against a manual baseline of 6 verified / 9 rejected. The 9 reject-fixtures all become
*safe divergences*: the reader is non-adversarial, so it never reproduces the malformed / fabricated /
fragment / negation-dropping hand-plans those fixtures were built to reject — it just reads the documents
honestly and grounds a verbatim answer. The sharpest case is the negation fixture: where the adversarial
hand-plan claimed `"cross the river during the flood."` (dropping the "Do not") and was correctly
rejected, the autonomous reader claims the **whole sentence verbatim** — `"Do not cross the river during
the flood."` — so the negation survives and the answer is honestly grounded, never false-grounded. A
tight-bounds run (`max_spans = 0`) finalizes nothing and turns every manual-verified fixture into a
**classified** false-reject (still 0 false-grounded), exercising the false-reject path. 9 tests pin all of
this (every-fixture coverage, no-hand-plan, 0 false-grounded, independent re-verification, the
manual-vs-autonomous partition, negation preservation, tight-bounds classified false-rejects,
determinism); a runnable `autonomous_pack_report` example prints the manual-vs-autonomous comparison and
exits non-zero on any false-grounded. `release_check` gates it (test + fmt + clippy + the runnable example
+ a `fixture.plan`-is-never-read source check + a `verify(` independent-recheck signal + no-ML +
separation); a live sabotage that records the hand-written plan instead of the reader's own fails the gate
(exit 101). The honest read of the numbers: the v0 reader is **safe but blunt** — it reads everything and
can't be selective without a model. That underperformance is an **engineering signal** (it motivates a
smarter, still-gated reader), explicitly **not** a training justification — the P12 gate still owns weights
and remains "not justified". All prior crates are 0-diff.

### READ-8 — Budgeted Autonomous Span Selection

Status: delivered (2026-06-15). READ-7 measured the v0 reader as *safe but blunt* — it reads everything.
READ-8 makes it **less blunt without a model**, via `reading_autonomy::read_budgeted` (a new `budgeted.rs`,
additive: the blunt READ-6 `read` is byte-identical, so READ-6's tests and the READ-7 pack stay green). The
budgeted reader still inspects **metadata first**, still reads spans only by id and only within the budget
(it never previews text), and still routes its plan **only through the codec** — the codec-only source scan
over `reading-autonomy/src` covers the new module. The single change is **selection**: among the spans it
reads, it CLAIMS only those **lexically relevant** to the question. Relevance is deterministic and lexical —
the question and each span are tokenised into lowercase content terms (length ≥ 3, minus a small fixed
stopword list), and a span is relevant if some content term **prefix-overlaps** a query term (the shorter,
≥ 3 chars, is a prefix of the longer, so "wind" matches "winds" but "art" does not match "start"). No
stemming, synonyms, embeddings, entailment, or model judgment — the boundary the rubric draws.

`crates/reading-budgeted-eval` measures the selective reader against the blunt one over the READ-4 corpora,
cross-validating every finalized answer (a fresh `verify` plus the independent `independently_grounded`
check). The result: **blunt 21 claims → budgeted 17**, with **3 fixtures more focused** — the weather fixture
answers just `"Winds will reach forty miles per hour."` (dropping the off-topic rain sentence), the medical
fixture just `"An ECG was ordered immediately."`, and the multi-sentence fixture just `"No injuries were
reported."` — and **0 false-grounded**. Because each claim is still a **verbatim whole cited sentence**,
focusing never paraphrases and never drops a negation from a *relevant* span (the negation fixture stays
`"Do not cross the river during the flood."`); a focused answer that omits an *off-topic* span is grounding
by design, not a false answer. Under a tight budget (`max_spans = 1`) a relevant span beyond the budget is
simply never reached — a **classified coverage miss**, surfaced explicitly, still with 0 false-grounded.
13 reading-autonomy tests (5 new: selective drop, codec-finalize, budget-enforced, deterministic/replayable,
negation-preserved) and 7 eval tests pin it; a runnable `budgeted_pack_report` prints the focus comparison
and exits non-zero on any false-grounded. `release_check` gates it (test + fmt + clippy + the runnable
example + `read_budgeted`/`decode(`/`prefix_overlap`/`content_terms` source signals + no-ML + separation); a
live sabotage that makes relevance always-true (reverting to blunt) fails four tests and the gate (exit
101). Boundary held: **deterministic selection only — no model, semantics, entailment, paraphrase, or
training.** A coverage miss is an engineering signal about the lexical floor (a future reader could select
better), never a reason to open weights — P12 still owns that and remains "not justified". The blunt `read`
is 0-diff and every other prior crate is 0-diff.

### READ-9 — Title-Aware Deterministic Relevance Ranking

Status: delivered (2026-06-17). READ-8 made the reader *less blunt* by claiming only question-relevant
spans, but it still visited spans in raw **metadata order** — so under a tight budget a relevant document
filed late could be missed while an irrelevant one consumed the budget. READ-9 fixes the ORDER without a
model, via `reading_autonomy::read_ranked` (a new `ranked.rs`, additive). Before reading, it ranks the
documents by **title relevance** to the question and reads higher-ranked documents' spans first, so a tight
budget reaches the relevant document instead of missing it.

The ranking is **metadata-only**. `DocumentMeta.title` is exposed before any span text is read, so
`title_relevance` scores the TITLE against the question using the **same** lexical machinery READ-8 uses for
spans — content terms (length ≥ 3, minus the fixed stopword list) and word-prefix overlap — and
`title_ranked_order` sorts documents by `(title_relevance DESC, title ASC, document_id ASC)`. The primary
and secondary keys are independent of insertion order, so for distinct titles the ranking — and therefore
the selection — is **stable across any file-order permutation**; `document_id` is only the final tiebreak
for two documents sharing both a title and a score, which keeps the result replayable. No model, semantics,
entailment, or paraphrase enters selection, and the ordering **never previews a span's text** — `ranked.rs`
calls neither `read_span` nor `.text()` (a gate grep pins this to zero).

Crucially the **claim filter is unchanged**. The two readers share one core, `read_selecting`, factored out
of READ-8's `read_budgeted` behavior-identically and parameterised only by the order in which span ids are
visited (`read_budgeted` = metadata order; `read_ranked` = title-ranked order). Budget, the text-relevance
filter, and the codec routing are therefore identical for both, so a span is claimed only if its OWN text is
lexically relevant AND grounds verbatim through the codec/verifier. A title match only changes READING ORDER
— never whether something becomes a claim — so a title match alone **cannot fabricate support** (a document
whose title matches but whose span text is irrelevant yields a coverage miss, not a grounded answer).

`crates/reading-ranked-eval` proves both halves. On the committed READ-4 pack the relevant documents are
already first, so ranking only **reorders**: the eval shows **no-regression** (every fixture's ranked answer
equals the budgeted answer — 15 answered, **0 regressions**, **0 false-grounded**, cross-validated with a
fresh `verify` plus the independent `independently_grounded` check). The title-priority **win** is measured
on a constructed scenario where the relevant document is filed *second* but its title matches: under
`max_spans = 1` the budgeted reader reads the first, irrelevant document and **misses**, while the
title-ranked reader reads the relevant document first and **answers** `"Winds will reach forty miles per
hour."` — with 0 false-grounded and an identical answer whether the documents are inserted forward or
reversed. 18 reading-autonomy tests (5 new: title-priority recovery, file-order stability, anti-fabrication,
deterministic/replayable, loose-budget no-regression) and 8 eval tests pin it; a runnable `ranked_pack_report`
prints the comparison and the demo and exits non-zero on any false-grounded, regression, or a demo that
fails to show the win.

`release_check` gates it (test + fmt + clippy + the runnable example + `read_ranked`/`read_selecting`/
`title_relevance`/`title_ranked_order` source signals + the **no-`read_span`/`.text()` in `ranked.rs`**
metadata-only proof + no-ML + separation). A live sabotage that makes `title_relevance` ignore the title
(reverting to blunt order) fails one reading-autonomy test and two eval tests and the gate (exit 101), and
notably the no-regression and 0-false-grounded checks **stay green under that sabotage** — neutralising the
ranking degrades to a coverage miss, never a false answer, so safety is independent of the ranking win;
restored byte-identical. A read-only adversarial panel (9 agents, Explore) found **0 defects** across five
attack lenses (title-fabrication, full-text-preview boundary, stability/determinism, regression/aggregate-
hiding, gate-vacuity) and returned PASS on every rubric sub-point (a)–(g) with code citations. Boundary
held: **deterministic title ranking only — no model, semantics, entailment, paraphrase, or training.** A
coverage miss is an engineering signal; P12 still owns weights and remains "not justified". The blunt `read`
is 0-diff, `read_budgeted` is behavior-identical (refactored to share the core), and every other prior crate
is 0-diff.

### READ-10 — Section-Aware / Multi-Term Deterministic Ranking

Status: delivered (2026-06-17). READ-9 ranked reads by document TITLE; READ-10 makes the ranking richer
along two still-purely-lexical axes — SECTION structure and MULTI-TERM coverage — without crossing the
no-preview / no-semantics boundary.

The substrate gains heading-labelled SECTIONS as metadata: `SectionMeta{heading, span_ids}` on
`DocumentMeta`, built by a new `add_document_with_sections`. A heading is a metadata STRING, never inserted
into the span map — there is no `SpanId` for a heading, so a claim physically cannot cite or ground one
(the strongest possible form of "a ranking signal may not become evidence"). The flat `add_document`
delegates to the sectioned constructor with a single empty-heading section, byte-identically: span ids and
byte ranges are unchanged, so every prior reading crate (READ-0/1/2/3/5 grounding, read0 run files,
READ-6/7/8/9 readers) stays green — proven by the full suite, not asserted.

`reading_autonomy::read_section_ranked` (a new `section.rs`, additive) orders the budgeted reader's span
reads by `combined_relevance(title, heading, query)` = the number of DISTINCT question content terms that
share a word-prefix overlap with a term of the document TITLE *or* the section HEADING. So a section whose
heading answers more of a multi-term question is read before one that merely shares a single token, and
under a tight budget the most relevant section is reached instead of missed. Sections are ordered by
`(combined_relevance DESC, title ASC, heading ASC, document_id ASC, section_index ASC)` — a total order
independent of insertion for distinct (title, heading), so the ranking is stable across any permutation of
documents or sections. The signals are metadata-only: titles and headings are exposed before any span text
is read, and `section.rs` calls neither `read_span` nor `.text()` (a gate grep pins this to zero). Reads
still route through the shared `read_selecting` core, so the claim filter — a span is claimed only if its
OWN text is lexically relevant AND grounds verbatim through the codec/verifier — is unchanged. The ranking
SCORE only orders reads: `section.rs` constructs no `extract_claim`/`synthesize`/`answer_text` (a second
gate grep pins this to zero), so a score can never enter a claim or answer.

`crates/reading-section-eval` proves both halves. The committed READ-4 pack is flat (one headingless
section per document), so section ranking reduces to title ranking and only REORDERS: the eval shows
no-regression (15 answered, 0 regressions — every section answer equals the budgeted answer — and 0
false-grounded, cross-validated with a fresh `verify` plus the independent `independently_grounded` check).
The section + multi-term WIN is measured on constructed sectioned corpora under a 1-span budget: a
heading-relevant section filed second is reached first (budgeted misses, section reader answers `"Winds
will reach forty miles per hour."`), and when two headings share the token "wind" the section covering
three distinct query terms ("storm wind warning") is read before the one covering one, answering `"A severe
storm wind warning is in effect tonight."` — a choice single-token overlap could not make. Both answers are
identical across section order and cross-validated grounded. 24 reading-autonomy tests (6 new), 9 eval
tests, and 3 new substrate tests pin it; a runnable `section_pack_report` prints the comparison and the
demo and exits non-zero on any false-grounded, regression, or a demo that fails to show the win.

`release_check` gates it (test + fmt + clippy + the runnable example + the `SectionMeta` /
`add_document_with_sections` / `read_section_ranked` / `section_ranked_order` / `combined_relevance` source
signals + the no-`read_span`/`.text()` and no-`extract_claim`/`synthesize`/`answer_text` proofs over
`section.rs` + no-ML + separation). A live sabotage that inverts the section sort (least-relevant section
first) fails three reading-autonomy tests, three eval tests, the example, and the gate (exit 101); restored
byte-identical. A read-only adversarial panel (9 agents, Explore) found 0 defects across five attack lenses
(heading-or-score-as-evidence, full-text-preview boundary, substrate-regression, multi-term/stability,
gate-vacuity) and returned PASS on every rubric sub-point (a)–(f) with code citations — the third
consecutive clean panel. Boundary held: heading/title metadata may RANK reads, never GROUND claims; the
section score may not become evidence; span text is not previewed before a span is read by id; the
codec/verifier still owns finalization. A coverage miss is an engineering signal; P12 still owns weights
and remains "not justified". The blunt `read`, READ-9 `read_ranked`, and READ-8 `read_budgeted` are all
source-0-diff; the substrate change is additive and behavior-preserving; the vibe engine is 0-diff.

### READ-11 — Real Document Section Metadata Ingestion

Status: delivered (2026-06-17). READ-10 gave the autonomy reader section-aware ranking, but section
metadata only mattered for hand-built corpora. READ-11 makes `read0` extract section metadata from REAL
text files, so a real document folder benefits from section-aware ranking — while preserving the rule that
headings may rank reads but may never ground claims. Built test-first against the operator's seven named
tests (RED → GREEN).

`read0`'s corpus loader (`reading-cli/corpus_load.rs`) now parses Markdown ATX headings deterministically.
`parse_atx_heading` accepts a line as a heading iff it begins with 1–6 `#`, then whitespace, then non-empty
text — strictly lexical, so `#nospace`, seven-or-more hashes, and a bare `#` are ordinary body. There is no
all-caps heuristic, no blank-line layout inference, no semantic detection, no model. `parse_sections` walks
the lines, opening a new section at each heading and accumulating body lines between headings; the body of
each section is run through the shared `split_sentences`, so spans and grounding never drift. Content
before the first heading is a default empty-heading section, emitted only if it has sentences; a file with
no headings is a single default section — byte-identical to the pre-READ-11 flat build, so the READ-4 pack
and the READ-3 smoke are unaffected.

The safety invariant is structural: a heading line is consumed as a heading BEFORE `split_sentences` ever
sees it, and is stored only in `SectionMeta.heading`. It is never inserted into the span map, so it has no
`SpanId`, and `verify` (which grounds only the text of cited span ids) can never cite or ground a heading.
`produce_run` was updated to store the corpus's ACTUAL built spans — the body sentences in span-id order,
read back from the corpus — rather than re-splitting the raw content; for headingless content this equals
the old behavior, and for headed content the headings are excluded, so the flat corpus that verify/replay
rebuild from the stored spans reproduces the same span ids and the same hashes. The tests pin every clause:
headings become section metadata, no span is a heading, body sentences get their section's ids, an
unheaded file gets the default section, a plan that tries to launder the heading text into a claim is
rejected, a misleading heading with no body support cannot finalize, a headed document still runs/verifies/
replays, and the section-aware autonomy reader recovers a heading-relevant answer the budgeted reader
misses on a document built by the real loader.

`release_check` gates it with positive parser signals (`parse_atx_heading`, `parse_sections`,
`add_document_with_sections`), the heading-free span-storage token, and — added in response to the panel
below — an end-to-end **headed-document binary smoke**: the real `read0` binary runs a `# Wind Forecast`
file through run → verify → replay and asserts the heading text never appears anywhere in the run file. A
live sabotage that makes the detector look for `~` instead of `#` (so it never finds a heading) fails four
reading-cli tests and the section-eval recovery test and the gate (exit 101); restored byte-identical. The
heading-rejection tests stay green under that sabotage — a heading still cannot be claimed even when
detection breaks — so the safety property is independent of the ingestion feature.

A read-only adversarial panel (9 agents, Explore) returned 0 defects on the heading-becomes-evidence,
replay-consistency, parser-determinism, and semantic-creep lenses, and one "high" on the gate-vacuity
lens: that `produce_run` could be reverted to `split_sentences(content)`, re-leaking headings into spans,
while passing the suite and the grep. Reproduced first-hand, the claim is false: the revert leaks
`# Wind Forecast` into a stored span and is caught three independent ways — the `headed_document_runs_
verifies_and_replays` test fails, the grep token (which lives inside the replaced `.expect` block) is
deleted, and (now) the binary smoke fires — gate exit 101. The production code was already correct, so
nothing was folded into it; instead the panel's kernel of truth (a comment-string grep is a weak signal)
was answered by adding the comment-immune binary smoke. Boundary held: READ-11 exposes real document
structure as metadata and never turns it into evidence. No model, no training — P12 still owns weights and
remains "not justified". The reading-substrate, reading-autonomy, and vibe crates and every other eval
crate are 0-diff; READ-3/4/7/8/9/10 stay green.

### READ-12 — Persist Section Metadata in Run Receipts

Status: delivered (2026-06-17). READ-11 parsed real document structure into the corpus but did not persist
it: a read0 run receipt carried only the flat body spans, so section-aware autonomy could not operate over
a real read0 output without rebuilding the structure from the original file. READ-12 closes that gap by
persisting the section metadata in the receipt — strictly as schema/receipt hardening, with the rule that a
heading may rank reads but may never ground a claim kept intact.

The receipt schema is bumped to `read0-run-v2`. Each `DocumentDto` now carries
`sections: Vec<SectionDto{heading: String, span_count: usize}>` — a heading-labelled PARTITION of the
document's flat `spans`, recorded as a heading string plus a count of consecutive body spans. A heading is
never a span: it has no `SpanId`, so it cannot be cited, and `verify` grounds only the text of cited span
ids. The flat `spans` field stays the canonical span-id source, so the pre-existing grounding, hash, and
tamper checks operate exactly as before — the schema change is additive and does not weaken them
(`span_text_tamper_still_caught_under_v2` and the original tamper tests confirm). `produce_run` records the
sections from the built corpus's metadata.

The verify/replay path is refactored around a shared `pub fn rebuild_corpus`, used both by `rederive` (which
then re-decodes the plan) and by section-aware consumers that want the persisted structure. It rejects two
new tamper classes and rebuilds the same sections the run built. **Heading-as-span tamper**: a stored span
that parses as an ATX heading (via the shared `parse_atx_heading`) is rejected — a heading can never be
re-derived as a span and so can never be cited or grounded. **Section/body-mismatch tamper**: the section
counts must partition the body spans exactly. This is computed with CHECKED, bounded arithmetic — each
section's cumulative end is `checked_add`ed and required to stay within the body, and after all sections the
cover must be exact — so a crafted receipt can neither overflow the count arithmetic nor slice out of
bounds; it returns a graceful `Tamper`. A document with no persisted sections (a headingless or pre-section
receipt) becomes one default empty-heading section, so old headingless files still verify and replay.
Because sections affect reading ORDER only and never grounding, the re-derived memory/answer hashes are
section-independent, and the existing tamper checks keep their full strength.

25 reading-cli tests and 11 reading-section-eval tests pin it; the section-eval test
`section_ranked_read0_uses_persisted_metadata` produces a headed receipt, rebuilds the corpus FROM the
receipt (not the original content), and shows `read_section_ranked` reaching the heading-relevant section
under a tight budget where the budgeted reader misses — section-aware autonomy operating over a persisted
read0 output. `release_check` gates it with schema/`SectionDto`/`rebuild_corpus`/`corpus_from_sections`/
`parse_atx_heading` signals and an end-to-end receipt-tamper binary smoke: a headed receipt carries the
heading and a span count, and injecting an ATX heading as a span, corrupting the counts, and a usize::MAX
overflow count are each rejected (the overflow gracefully, with no panic). A live sabotage that neuters the
heading-as-span check fails the unit test and the gate (exit 101); restored byte-identical.

A read-only adversarial panel (9 agents, Explore) returned 0 defects on the section-as-evidence,
schema-weakening, replay-reconstruction, and gate-vacuity lenses, and one "critical" on the
tamper-completeness lens: a `span_count` of usize::MAX could overflow a plain `sum()` check and panic the
partition slice on a crafted receipt. Reproduced first-hand — `read0 verify` on the panel's exploit panicked
with "attempt to add with overflow" — the finding was real and FOLDED: the partition was rewritten with the
checked, bounded arithmetic above, which now returns `tamper detected: section span count … overruns the …
body spans` (exit non-zero, no panic), guarded by a regression test and the overflow binary smoke. It was
never an authority bypass — the crafted file was always rejected, previously via a crash, now via a clean
`Tamper`. Boundary held: heading text is metadata only, a body sentence is the span evidence, the verifier
sees only cited span text, and a heading cannot ground a claim. No model, no training — P12 still owns
weights and remains "not justified". The reading-substrate, reading-autonomy, reading-codec, vibe, and every
other eval crate are 0-diff; there is no Cargo.toml/lock change; READ-3/4/7/8/9/10/11 stay green.

### READ-13 — Receipt Schema Compatibility / Migration Gate

Status: delivered (2026-06-17). READ-12 persisted section metadata in the run receipt and bumped the schema
to `read0-run-v2`, but the version was handled loosely: `read_run_file` did a bare string compare, and
`rebuild_corpus` treated an EMPTY `sections` array as a headingless receipt and silently fell back to the
flat rebuild. Because sections affect reading ORDER only — never the memory/answer hashes — that fallback
meant a v2 receipt could have its section metadata STRIPPED and still verify and replay: the sections could
disappear unnoticed. READ-13 makes the schema version explicit and load-bearing, as schema/receipt
hardening only (no model work). The boundary added is version discipline, not evidence authority.

The schema tag is now a recognized `enum SchemaVersion { V1 = "read0-run-v1", V2 = "read0-run-v2" }`, parsed
FIRST inside the shared `rebuild_corpus` chokepoint (used by `rederive` → both `verify_file` and
`replay_file`, and by section-aware consumers), so every consumption path enforces it once with no driftable
duplicate. The tag must AGREE with the receipt's content, per document:

- **v2 requires sections.** An empty `sections` on a v2 document is rejected as `Tamper("sections were
  dropped")`. This is the load-bearing change: it closes the READ-12 silent-fallback hole, so section
  metadata can no longer vanish from a v2 receipt undetected.
- **v1 forbids sections.** A `read0-run-v1` receipt is the pre-section shape and carries no section
  metadata; a v1 tag wearing v2 sections is ambiguous (neither cleanly v1 nor v2) and rejected as `Tamper`.
  A faithful v1 receipt MIGRATES forward deterministically to one default empty-heading section over all
  spans — the flat rebuild reproduces the same span ids and the same hashes a v1 run produced, so old
  headingless receipts still verify and replay.
- **Unknown tags refuse cleanly.** Any other tag returns `CliError::UnsupportedSchema` before any rebuild —
  never accepted by default, and with no panic on the untrusted input.

The schema tag governs STRUCTURE only. It never reaches the codec or the grounding path and is never folded
into `memory_hash`/`answer_hash`; the flat `spans` field stays the canonical span-id source. So evidence
authority is unchanged and the pre-existing tamper checks keep their full strength: the one-sentence-span
check, the heading-as-span rejection, the checked/overflow-safe section partition (extracted unchanged into
`partition_sections`), and the answer/hash match all remain. `produce_run` always writes v2 (v1 is
recognized for READING old receipts, never written), and `read_run_file` drops its duplicate string compare
and delegates schema validation to the pure chokepoint.

29 reading-cli tests and 11 reading-section-eval tests pin it; the four new READ-13 tests are
`v1_headingless_receipt_migrates_and_verifies` (a v1 tag with sections cleared migrates and verifies),
`v1_receipt_carrying_sections_is_rejected` (the ambiguity attack), `v2_receipt_with_dropped_sections_is_rejected`
(the silent-drop hole, now caught), and `unknown_schema_is_rejected` (clean `UnsupportedSchema` on verify and
replay). `release_check` gates it with `enum SchemaVersion`/`UnsupportedSchema`/`read0-run-v1`/
`fn partition_sections` signals and an end-to-end schema-version binary smoke: a real v2 receipt verifies, a
Python-built v1 migration of it verifies, and the dropped-sections, v1-with-sections, and unknown-version
variants are each rejected (the unknown one asserting no `panic` reaches stderr). A live sabotage that
reverts the v2-must-carry-sections check back to the READ-12 silent flat fallback fails
`v2_receipt_with_dropped_sections_is_rejected` and the gate (exit 101); restored byte-identical (md5
`d85644fe…`).

A read-only adversarial panel (5 Explore agents) returned 0 defects across all five lenses —
evidence-authority, silent-drop, ambiguity-relabel, panic-robustness, and gate-vacuity — the cleanest panel
of the reading arc. The gate-vacuity lens confirmed every signal grep matches production code and every
binary smoke exercises the exact path it claims (tracing `verify_run`/`replay_run` → `rebuild_corpus` →
`SchemaVersion::parse` and checking each smoke's branch), so the gate is load-bearing, not decorative.
Boundary held: READ-13 adds version discipline, and the schema tag can never change what grounds a claim. No
model, no training — P12 still owns weights and remains "not justified". The reading-substrate,
reading-autonomy, reading-codec, vibe, and every other eval crate are 0-diff; there is no Cargo.toml/lock
change; READ-3/4/7/8/9/10/11/12 stay green.

### READ-14 — Receipt Integrity Hashing for Structural Metadata

Status: delivered (2026-06-18). READ-12 persisted the section structure and READ-13 versioned it, but the
structural metadata itself was only checked for INTERNAL CONSISTENCY (sections partition spans, no
heading-as-span, version↔content agree). The persisted fields — a section heading or document title string,
an uncited span's text, a section boundary that still partitions — could be edited without detection: they
are non-evidentiary (they cannot ground a claim), so nothing bound them. READ-14 binds them with an explicit
structural-integrity hash, as schema/receipt hardening only (no model work). The boundary added is integrity
over the structure; the metadata stays non-evidentiary.

`read0` now writes `read0-run-v3`, which carries `structure_hash: Option<u64>` — a deterministic FNV-1a
64-bit hash over the schema tag and, per document, the title, the ordered span texts, and the ordered
sections (heading + span count). It is the same FNV-1a construction the substrate uses for its content
hashes (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`), kept LOCAL to reading-cli so the
substrate stays a pure evidence-hash layer and the receipt-integrity concern lives with the receipt. Every
variable-length field is length-prefixed and every collection count-prefixed, so the hash input is an
injective encoding of the structure — two distinct structures cannot collide by re-grouping bytes across
fields (beyond FNV's inherent 2⁻⁶⁴, the same strength as the existing memory/answer hashes).

The hash is version-gated through `enforce_structure_hash`, called at the top of the shared `rebuild_corpus`
chokepoint (so it applies to verify, replay, and section-aware consumers): a v3 receipt MUST carry a
structure hash that equals the recomputed one (absent or mismatched → `Tamper`); a v1/v2 (pre-v3) receipt
MUST NOT carry one — forbidding it blocks a relabel-to-legacy that keeps a stale binding. `produce_run`
writes v3; v1/v2 remain recognized for reading old receipts. This catches the structural edits the prior
consistency checks missed: `heading_string_tamper_is_rejected`, `title_tamper_is_rejected`, and
`uncited_span_tamper_caught_under_v3_not_v2` — the last shows the gap explicitly, with a legacy v2 receipt
NOT binding the uncited span while the v3 receipt catches the same edit.

The structure hash is an INTEGRITY checksum, not an evidence signal. It never reaches the codec or the
grounding path, never folds into `memory_hash`/`answer_hash`, and never makes a heading or title citable —
the adversarial panel traced that `structure_hash` appears only in reading-cli, never in the substrate or
codec, and `verify_file` runs the evidence re-derivation (memory/answer hash match plus grounding from cited
span text) INDEPENDENTLY after the structure check. The pre-existing tamper checks are not masked: the tamper
tests RESEAL the structure hash (modelling the strongest attacker, one who recomputes it after tampering) to
prove the deeper checks — heading-as-span, partition, the overflow no-panic, and grounding — still fire. The
structure hash is an added layer, never a substitute. Because the metadata is non-evidentiary, the
recomputable nature of the hash is acceptable: a full-file attacker who reseals it can at most misdirect
future section ranking, never forge a grounded answer. The honest limit is a v3→v2 downgrade (relabel plus
strip the hash): it reverts to legacy-unbound metadata, which exposes only non-evidentiary fields to
undetected edits and never touches evidence authority — the migration-safety tradeoff READ-13 deliberately
kept, confirmed in scope by the panel's forgery lens.

37 reading-cli tests (8 new READ-14) and all five reading eval crates pass (each produces and verifies within
v3). `release_check` gates it with `read0-run-v3`/`structure_hash`/`fn structural_hash`/
`fn enforce_structure_hash` signals and a structural-hash binary smoke: a v3 receipt carries and verifies a
structure hash, and tampering a heading string, corrupting the hash, dropping the hash, or relabel-keeping it
under v2 are each rejected; the READ-13 smoke was updated to build faithful legacy receipts (a pre-v3 tag
carries no hash). A live sabotage that neuters the v3 hash comparison fails four structural-tamper tests and
the gate (exit 101) — while the missing-hash and v2-carrying-hash tests stay green, since they exercise
different branches — and was restored byte-identical (md5 `066912b4…`). A read-only adversarial panel (6
Explore agents) returned 0 defects across all six lenses: evidence-authority, check-masking, forgery-
downgrade, determinism-collision, panic-robustness, and gate-vacuity. Boundary held: READ-14 binds
structural integrity while keeping the metadata non-evidentiary, and evidence authority is unchanged. No
model, no training — P12 still owns weights and remains "not justified". The reading-substrate,
reading-autonomy, reading-codec, vibe, and every other eval crate are 0-diff; there is no Cargo.toml/lock
change; READ-3/4/7/8/9/10/11/12/13 stay green.

### READ-15 — Receipt Downgrade Policy / Final Receipt Boundary

Status: delivered (2026-06-18). READ-14 bound the structural metadata of the current receipt with a structure
hash, but accepted older v1/v2 receipts as legacy without that binding — and a v3→v2 downgrade (relabel plus
strip the hash) could revert to that unbound state. READ-14 documented this as an honest limitation; READ-15
makes the integrity LEVEL explicit, tested, and operator-visible, so the system never silently accepts a
weaker receipt as equivalent to the current one. This is classification only (no model work). The boundary
added is the ability to CLASSIFY receipt integrity; grounding authority is unchanged.

Verification now returns a `VerifyOutcome { receipt, integrity }`. The `IntegrityLevel` is either `Current`
(`read0-run-v3`, structurally bound) or `LegacyUnboundStructure` (`read0-run-v1`/`read0-run-v2`, structural
metadata not bound). It is DERIVED from the validated schema version via `from_version`, and crucially it is
never persisted in the run file — it is recomputed from the validated tag on every verify, so a receipt
cannot store a claim that overrides it and cannot earn a higher level than its tag deserves. Each level
exposes a machine-checkable `token()` — `structure_bound` or `legacy_unbound_structure` — and `is_current()`.
`read0 verify` prints `integrity=<token>`, and for a legacy or downgraded receipt it adds an explicit
`warning: legacy_unbound_structure …` line. So a v3→v2 stripped-hash downgrade still verifies (its evidence
is fully bound) but is reported as legacy, never current — the downgrade can no longer pass itself off as
full integrity.

The classification touches structure only and never grounding. The level is derived AFTER `rederive`, the
answer/hash match, and the substrate grounding all pass, so a receipt that fails any evidence check never
receives a level; and an unknown future schema is still rejected by `SchemaVersion::parse` before any
classification. The `integrity_level_does_not_change_evidence_authority` test proves a v3 receipt and its v2
downgrade produce the IDENTICAL verifier `Receipt` (same grounded, answer_supported, replay_matches, passed)
— only the level differs. The level never reaches the codec, the substrate verifier, or the memory/answer
hashes; grounding still flows only from cited span text.

43 reading-cli tests (six new READ-15: `v3_receipt_reports_current_integrity`,
`legacy_v2_and_v1_report_legacy_unbound_structure`, `v3_to_v2_downgrade_is_not_reported_as_current`,
`integrity_level_does_not_change_evidence_authority`,
`integrity_level_is_derived_from_version_not_a_stored_claim`,
`integrity_tokens_are_stable_and_machine_checkable`) pin it; reading-corpus-eval is unaffected by the
return-type change because it only checks `Ok`/`Err` (and formats the value with `Debug`). `release_check`
gates it with `enum IntegrityLevel`/`struct VerifyOutcome`/`legacy_unbound_structure`/`structure_bound`
signals and a downgrade-policy binary smoke: the v3 receipt's `read0 verify` output carries
`integrity=structure_bound`, and a faithful v2 downgrade verifies but its output carries
`integrity=legacy_unbound_structure` plus the warning and never `integrity=structure_bound`. A live sabotage
that classifies legacy receipts as `Current` fails four classification tests and the gate (exit 101) — while
the v3-current and token-stability tests stay green, since they exercise different branches — and was
restored byte-identical (md5 `8d3a6e20…`). A read-only adversarial panel (6 Explore agents) returned 0
defects across all six lenses: grounding-unchanged, forgery, silent-equivalence, downgrade-completeness,
no-regression, and gate-vacuity. Boundary held: READ-15 classifies the receipt integrity level while leaving
grounding authority unchanged. No model, no training — P12 still owns weights and remains "not justified".
The reading-substrate, reading-autonomy, reading-codec, vibe, and every other eval crate are 0-diff; there is
no Cargo.toml/lock change; READ-3/4/7/8/9/10/11/12/13/14 stay green.

### READ-16 — Reading Track Milestone Freeze

Status: delivered (2026-06-18). The reading track has accumulated enough moving parts — a grounding
contract, a codec quarantine, deterministic autonomy and ranking, a metadata-not-evidence rule, and a
four-step receipt boundary — that it needs a frozen, auditable milestone before the next expansion. READ-16
freezes the READ-0 → READ-15 arc as `reading-track-v0.1`. It is a documentation freeze (no model work) and
adds no behavior.

The freeze record is `READING_TRACK_MILESTONE.md`, mirroring the repo's existing `GOVERNANCE_MILESTONE.md`
pattern (a `*_MILESTONE.md` doc plus a tag, both locked by `release_check.sh`). It records, in order: the
full eighteen-commit lineage with hashes (READ-0 substrate and the READ-1/READ-2 grounding contracts; the
P9–P12 codec, adapter, eval, and training-gate layer; READ-3 through READ-15); the boundaries that hold
across the whole arc; the P12 training verdict; the release-gate and verification discipline; the
independent-panel record; the honest residuals; and the frozen-status declaration. The load-bearing
through-line is that the flat `spans` list stayed the canonical span-id source through the entire receipt
arc, so evidence authority — grounding from cited span text and the re-derived memory/answer hashes — is
unchanged at every receipt version; everything READ-11 through READ-15 added is non-evidentiary structure
(headings, sections, the structure hash, the integrity level) that orders reads or classifies integrity but
never grounds a claim.

The training verdict is recorded faithfully as `training_not_justified`: the P12
`TrainingDecision.training_justified` bit is `false`, because on the current battery there are zero
false-accepts and zero false-rejects, so there is no clean recurring model failure to justify weights.
P13–P15 stay closed.

Verification of a documentation freeze is accuracy, not behavior. Every commit hash named in the milestone
doc was cross-checked against `git log`: all nineteen resolve to real commits with the exact expected
subjects, and a reverse scan found zero bogus hashes in the document. The READ-16 `release_check` block locks
the milestone the same way the governance milestone is locked — `test -f`, a `FROZEN` grep, the tag name, the
`READ-0`/`READ-15` coverage endpoints, the `training_not_justified` verdict, and the pinned lineage endpoints
`f5b3fa9`/`3902418`/`11e9c5f` — so the freeze cannot silently drift. An independent read-only verifier
audited the document against the git ground truth for hash accuracy, status correctness, and overstatement.
`release_check.sh` is exit 0 and byte-silent.

The git tag `reading-track-v0.1` is created only after a clean working tree and a green gate — i.e., after
the milestone commit — per the rubric. No model, no training; P12 still owns weights and remains "not
justified". The reading crates, the vibe engine, and every prior reading doc are 0-diff except the new
milestone document and its gate lock; there is no Cargo.toml/lock change; READ-3/4/7/8/9/10/11/12/13/14/15
stay green.

## Hypothesis Layer Track (P16 / HYP-0): probability proposes, replay tests, governance authorizes

Date: 2026-06-18. A NEW post-freeze track, additive to the frozen `reading-track-v0.1`. The reading
substrate answers only from cited-span evidence and forbids anything it cannot ground. HYP-0 adds the
one faculty that substrate deliberately lacks — the ability to PROPOSE an explanation or a next probe
that is not yet grounded — without letting a proposal leak the authority of a fact. It sits ABOVE the
frozen substrate and BELOW human review, and it is a PROPOSER, never an actor. Its doctrine is four
lines: **Probability proposes. Replay tests. Governance authorizes. Memory records.**

The core object is the `HypothesisPacket` — scored and traced, but inert. The design makes the
quarantine STRUCTURAL (enforced by the compiler and types), not a convention a caller must honor:

- **Born only by `propose`, never forged.** The packet's fields are private with read-only accessors,
  and it does not derive `Deserialize`. The only way to obtain one is `propose`, so it cannot be
  mutated after the fact or constructed off the wire. The deserializable trace surface is the INPUTS
  (`HypothesisSpec`); replay deserializes the spec and RE-DERIVES every governed field, so a hand-edited
  trace cannot smuggle a forged score, id, clearance, authority, or shrunken forbidden-uses.
- **No authority but its own.** Every packet carries `Authority::HypothesisOnly`, an enum with exactly
  one variant, so a hypothesis marked as carrying claim/evidence/governance authority is unrepresentable.
- **Never a claim or evidence.** Each packet bakes the canonical six `FORBIDDEN_USES` and a `permits`
  predicate refuses them; the list is pinned by identity, so it cannot be silently shrunk or substituted.
- **Cites its provenance.** A packet references the receipts it was derived from by content hash
  (`EvidenceRef` = answer hash + memory hash + label) and holds no handle into the cited object.
- **Deterministic and replayable.** Scoring is integer per-mille math (no floats, no model, no semantic
  judge) and the id is an FNV-1a hash over length-prefixed inputs, so the same spec reproduces the same
  packet; `verify_consistency` re-derives and rejects any mismatch.
- **Cannot authorize a dangerous test.** A probe that is high-risk OR hard-to-reverse escalates to
  human review; high-risk AND irreversible is blocked; only a safe probe is auto-allowed.
- **Quarantined by dependency.** Production dependencies are serde only; the reading crates appear
  solely as dev-dependencies to PROVE non-interference. The gate asserts the non-dev tree holds no
  substrate/engine crate and no ML crate, so deriving a hypothesis changes neither the verifier receipt
  nor the P12 training verdict.

Verification followed the same discipline as the reading arc — a green-and-silent `release_check`, live
sabotage probes, and read-only adversarial panels — but pushed harder on the gate itself. Across six
panel rounds the five substantive lenses (authority-escape, claim/evidence isolation, determinism and
replay, probe safety, non-interference) were clean for five consecutive rounds. The gate-vacuity lens —
auditing whether the gate would actually CATCH a future regression — drove four rounds of structural
hardening, each defect reproduced first-hand before folding and the fix proven by re-running the exact
sabotage (break, confirm the gate goes red, restore byte-identical). The arc of those folds is the
interesting record: public/deserializable fields that made a packet forgeable were sealed to private
fields with no `Deserialize`; a grep that only saw the derive line was replaced by a `compile_fail`
doctest the COMPILER enforces against derive and hand-written impl alike; a vacuous single-variant check
became an exhaustive `match` that fails to compile if a second authority variant is ever added; a
comment-dodgeable derive grep and an unguarded `RecommendedProbe` were closed by a second compile_fail
proof plus compiler-proof existence asserts; and a circular forbidden-uses check (which a duplicate
substitution could pass while un-forbidding a use) became an identity-and-distinctness test written from
literals. The pattern throughout was to move each security-critical property off a heuristic grep and
onto the compiler, the type system, or a behavioral run. Round six was the first fully-dry round — all
six lenses clean.

Boundary held: a hypothesis is a guess to be tested, never a fact; probability may propose and schedule a
test, but it can never ground an answer, mutate memory, alter a receipt, change the training verdict, or
bypass governance. There is no LLM, no training, and no semantic judge in the layer — deterministic
scoring only for v0. P12 still owns weights and remains "not justified"; P13–P15 stay closed. The track
is additive: the reading crates, the vibe engine, and every prior document are 0-diff; only the new
crate, its workspace membership, and the gate block are added. `release_check.sh` is exit 0 and
byte-silent.

## Appendix — LLM Training as Constraint Engineering (supporting methodology)

Date: 2026-06-13

This appendix is operator-supplied methodology, retained here for reference. It is not a
sprint; it is the harness-first, feedback-loop, stop-criteria discipline that the Caitlin
Leap above already embodies (define DONE, build the harness, loop against checkable
criteria, change weights/process only where evidence says they are the bottleneck).

This plan converts the attached synthesis into a practical working program. It treats "training an LLM from scratch" less as a single monolithic pretraining run and more as a staged system: harness first, feedback loops second, weight updates only where they have measurable leverage.

### Operating Thesis

The hard problem is not just model optimization. It is preventing human drift while building a system that can improve itself under measurable constraints.

The training system should therefore include:

1. A harness that defines context, tools, retrieval, prompts, retries, and evaluation.
2. Feedback loops that update both behavior and weights when evidence says weights are the bottleneck.
3. Human-facing rules that prevent abandonment, scope creep, and vague progress claims.
4. Externalized domain knowledge in reusable skills, references, examples, and tests.

### Phase 0: Define the Real Target

Before any model work, write a one-page target spec:

- Task domain
- User workflow the system must support
- Inputs and outputs
- Failure modes that matter
- Minimum acceptable quality bar
- Evaluation method
- Cost and latency limits
- What must not be solved in v1

Gate: no training, fine-tuning, or dataset building starts until the target spec has a measurable pass/fail definition.

### Phase 1: Build the Harness Before Touching Weights

Create the simplest agent/harness that can attempt the task with an existing foundation model.

Required pieces:

- System prompt
- Task prompt template
- Retrieval or context loading
- Tool interface, if needed
- Output schema
- Retry and repair logic
- Evaluation runner
- Logging for inputs, outputs, failures, and scores

Gate: the harness must run end-to-end on at least 30 representative examples before any fine-tuning decision.

### Phase 2: Establish a Baseline

Run the harness without fine-tuning.

Track:

- Accuracy or task score
- Human review score, if objective grading is unavailable
- Latency
- Cost per attempt
- Common failure categories
- Prompt or retrieval changes that improve results

Decision rule:

- If failures are caused by missing context, improve retrieval.
- If failures are caused by unclear instructions, improve prompts and schemas.
- If failures are caused by tool misuse, improve tool descriptions and validation.
- If failures persist despite clear context, stable instructions, and representative examples, consider weight updates.

### Phase 3: Externalize Domain Knowledge

Convert recurring domain knowledge into reusable units.

Create:

- `skills/` for procedural knowledge
- `references/` for canonical background material
- `examples/` for good and bad outputs
- `evals/` for scored test cases
- `logs/` for run history and regressions

Each skill should define:

- When to use it
- Required inputs
- Allowed tools
- Procedure
- Validation checks
- Known failure modes

Gate: any knowledge used more than twice must become a reusable artifact instead of living only in chat history or memory.

### Phase 4: Add Self-Improvement Loops

Introduce iterative improvement only after the baseline exists.

Loop structure:

1. Run the harness on evaluation cases.
2. Classify failures.
3. Propose one change to prompts, retrieval, tools, examples, or weights.
4. Apply the change.
5. Re-run the same evaluation set.
6. Keep the change only if it improves the target metric without unacceptable regression.

Keep changes small. Each iteration should answer one question.

Gate: no loop continues without a metric trend, a failure log, and a written reason for the next change.

### Phase 5: Decide Whether Fine-Tuning Is Justified

Fine-tuning is justified only when at least one of these is true:

- The model repeatedly fails a stable pattern despite having the needed context.
- The desired output style or structure is highly specific and frequent.
- The task requires compressed domain behavior that cannot fit reliably in context.
- Cost or latency requires replacing long prompts with learned behavior.
- The evaluation set shows clear headroom after harness improvements plateau.

Use the smallest effective training method first:

1. Prompt and retrieval changes
2. Few-shot examples
3. Preference data or reranking
4. LoRA or adapter fine-tuning
5. Full fine-tuning
6. Pretraining from random initialization only if no foundation model can plausibly cover the domain

### Phase 6: If True From-Scratch Training Is Required

Only train from random initialization if the domain lacks usable foundation models or has hard constraints that prevent using them.

Minimum infrastructure:

- Clean corpus pipeline
- Tokenizer or domain representation
- Model architecture choice
- Training loop
- Validation set
- Evaluation suite
- Checkpointing and rollback
- Experiment tracker
- Cost budget
- Stop criteria

Gate: do not start from-scratch training without a baseline from a foundation model, unless legal, privacy, modality, or scientific constraints make that impossible.

### Phase 7: Stop Criteria

Stopping is not abandonment. Stopping is allowed when evidence says the next iteration is not worth its cost.

Stop or pause when:

- The metric plateaus across three meaningful iterations.
- Regressions exceed gains.
- Evaluation quality is too weak to guide progress.
- The system meets the target spec.
- A true blocker has been verified rather than assumed.

Continue when:

- The blocker is untested.
- Failures are not categorized.
- No baseline exists.
- The project is drifting because of boredom, novelty seeking, or tool switching.

### Daily Operating Rule

Every work session should end with:

- What changed on disk
- What improved measurably
- What failed
- The next smallest test
- Any open loop that could be abandoned

Progress is what exists in files, logs, metrics, and repeatable procedures.

### Immediate Next Step

Build the smallest possible harness for one concrete domain and run it on 30 examples. Do not fine-tune first. The first goal is to discover whether the bottleneck is context, instructions, tools, examples, evaluation, or weights.
