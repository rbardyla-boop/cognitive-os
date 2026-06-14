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
- [ ] P4 — FrameCollector + ObservationFrame.
- [ ] P5 — Minimal VibeEngine evaluation loop.
- [ ] P6 — RunScript + RunRecorder + deterministic replay.
- [ ] P7 — Local CLI prototype (`vibe run` / `vibe replay` / `vibe verify`).
- [ ] P8 — Prototype release gate (Rust tests + replay determinism + governance checks + no-secrets).
- [ ] P9 — Language-codec boundary (LLM proposes typed packets; cannot mutate state).
- [ ] P10 — Baseline off-the-shelf local LLM adapter (zero training).
- [ ] P11 — LLM codec eval harness (30–100 cases; model cannot self-grade).
- [ ] P12 — Training-justification gate.
- [ ] P13 — Local LoRA/adapter candidate (only if justified).
- [ ] P14 — Shadow-mode insertion.
- [ ] P15 — Promotion / rejection gate.

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

Status: Not started. Correct if all scheduled observations for a tick become one canonical
`ObservationFrame`, frame ordering is stable, the frame hash is reproducible, and empty ticks are
handled explicitly. Wrong if the engine consumes loose observations, the frame hash changes across
equivalent runs, or empty-tick behavior is implicit. Build: `FrameCollector`, `ObservationFrame`,
`frame_hash`, canonical ordering, explicit empty-frame representation. Tests:
`same_tick_same_frame_hash`, `different_order_same_canonical_frame`, `empty_tick_frame_is_explicit`,
`frame_contains_only_scheduled_observations`. Acceptance: `evaluate_tick` receives frames, not raw
input.

### P5 — Minimal VibeEngine evaluation loop (L0)

Status: Not started. Correct if `VibeEngine` consumes an `ObservationFrame` and emits a deterministic
`EngineOutput`, the state transition is explicit, the output hash is reproducible, and one scenario
proves state evolves across ticks. Wrong if there is hidden mutable global state, output depends on the
environment, or state updates happen outside `evaluate_tick`. Build: `VibeEngine`, `evaluate_tick()`,
`EngineOutput`, `StateTransition`, `output_hash`. Tests: `same_run_same_outputs`,
`state_transition_explicit`, `no_external_mutation`, `multi_tick_scenario_reproducible`. Acceptance:
the deterministic engine actually runs.

### P6 — RunScript, RunRecorder, and deterministic replay (L2)

Status: Not started. Correct if a `RunScript` drives the whole pipeline, `RunRecorder` records accepted
observations/frames/outputs/hashes, and replay reproduces the same result hash-for-hash. Wrong if replay
depends on live input, a recorded run cannot reconstruct frames, or replay output differs without
explanation. Build: `RunScript`, `RecordedRun`, `RunRecorder`, `ReplayRunner`, `run_hash`,
`replay_report`. Tests: `record_then_replay_same_hash`, `replay_reconstructs_frames`,
`replay_reconstructs_outputs`, `tampered_recorded_run_detected`. Acceptance: the prototype is replayable,
not just runnable.

### P7 — Local CLI prototype

Status: Not started. Correct if an operator can run one local command that ingests a scenario, evaluates
ticks, writes a recorded run, replays it, and prints a concise report. Wrong if manual script chaining is
required, or there is no operator-facing run/replay command, or output cannot be inspected. Build:
`vibe run <scenario>`, `vibe replay <recorded_run>`, `vibe verify <recorded_run>`. Tests:
`cli_run_scenario_succeeds`, `cli_writes_recorded_run`, `cli_replay_matches_original`,
`cli_verify_detects_tamper`. Acceptance: a working local prototype exists from the operator's point of
view.

### P8 — Prototype release gate

Status: Not started. Correct if one release command runs Rust tests, the Python governance checks,
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
- **P10 — Baseline local LLM adapter (zero training).** A local model parses requests into candidate
  typed packets at `temperature = 0`, structured/schema output, no autonomous tool calls, no write
  authority; bad output is rejected cleanly. Acceptance: the baseline works as a *proposed* translator,
  the deterministic engine still decides everything, `release_check` stays green.
- **P11 — LLM codec eval harness.** Build the 30–100 case harness before any training: cover valid
  observation creation, ambiguous request, unsafe authority request, memory-mutation attempt, unsupported
  claim, explanation-from-audit, explanation-without-evidence, bad JSON/schema, wrong target tick, correct
  refusal. The scorer compares to committed ground truth; the model cannot self-grade. Acceptance: a
  reproducible baseline score with false-accepts visible and failures classified.
- **P12 — Training-justification gate.** Training is allowed only if: the task was specified, examples were
  representative, context was present, prompt/schema/tooling were stable, the eval caught the failure, and
  the baseline still repeatedly failed the same pattern. Not justified if failures trace to bad
  schema/prompt/examples/eval-labels/task-definition/tooling/context. Acceptance: the decision cites exact
  failed cases; no training without clean failures.
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
