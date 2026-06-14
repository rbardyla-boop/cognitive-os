# Governed Memory

## Append-Only Episodic Log

Raw episodes are immutable. New observations append a fresh episode with:

- `episode_id`
- `timestamp`
- `source`
- `raw_payload`
- `parsed_claims`
- `confidence`
- `trace_id`
- `linked_actions`
- `linked_rules`

## Raw Experience Ingestion

Sprint 22 adds a lower raw-ingestion boundary before interpretation. An incoming
`ExperienceEnvelope` becomes a `RawEpisode` with:

- `episode_id`
- `trace_id`
- `source`
- `timestamp`
- `logical_tick`
- `raw_payload`
- `modality`
- `capture_context`
- `integrity_digest`
- `ingestion_license`
- `schema_version`
- `parsed_claims`
- `semantic_candidate_ids`

`parsed_claims` starts empty. Semantic candidates must cite an existing raw episode and inherit its
integrity provenance; they cannot overwrite the raw payload.

## Semantic Candidate Extraction

Sprint 23 introduces `CandidateMemoryNode` records. They are interpretations, not accepted facts:

- `status`: `semantic_candidate`
- `epistemic_license`: `hypothesis_only`
- `confidence`: `0.0`
- `authority_class`: `semantic_candidate`
- `forbidden_use`: includes `direct_action`, `memory_consolidation`, `rule_revision`, and `safety_certification`

Every candidate cites `source_raw_episode_id` and `source_integrity_digest`. LLM proposals may supply
candidate text, but any LLM-supplied authority fields are ignored.

## Design Memory (the Caitlin Leap)

Sprint 24 makes the development process a first-class citizen of the same machinery.
`simulations/bridge_world/design_memory.json` holds:

- `invariants`: locked design rules (`status: regression_lock`) with `epistemic_license`,
  `allowed_use` (`release_invariant`), `forbidden_use` (`runtime_action`), and the
  `verifier_rule_id` that governs proposals against them.
- `design_decisions`: the audited sprint-decision chain; each carries `trace_id`,
  `verifier_assessment`, `epistemic_license`, `contradictions`, and `revalidation_status`.

A design proposal is evaluated against `design_verifier_rules.json`. A proposal that would
`weaken` a `regression_lock` invariant is a `hard_contradiction`: it emits a `hazard_only`
`ContradictionPacket`, is denied consolidation through the same mutation gateway as runtime
memory (the invariant is preserved), and opens a deferred `design_revalidation` correction
job. `project_self_audit.py --strict` fails on any design decision missing trace, verifier,
or license; project health (`D_project_cognitive_health`) consolidates to `green` only
through the gateway under `memory_consolidation` license.

Sprint 25 makes the proposal's `effect` evidence-derived, not self-declared.
`effect_classifier.derive_effect` computes a semantic diff of the proposal claim vs. the
invariant claim into `weaken` / `contradict` / `extend` / `preserve` / `needs_review`. The
derived effect is the authority that drives the design verifier rules; any declared `effect`
is an untrusted hint, and a declared/derived family mismatch is surfaced as `effect_mislabel`.
A weakening mislabelled `extend` is reclassified and blocked; an unprovable proposal lands in
`needs_review`, which blocks rather than auto-accepting.

## Semantic Memory Graph

Semantic nodes use:

- `memory_id`
- `claim`
- `confidence`
- `status`
- `source_episodes`
- `depends_on_rules`
- `contradictions`
- `created_by`
- `updated_by`
- `schema_version`

Statuses:

- `active`
- `active_with_superseded_dependency`
- `confidence_reduced`
- `pending_rederivation`
- `contradicted`
- `exception_scoped`
- `quarantined`
- `retest_required`
- `superseded`
- `deprecated_but_preserved`

## Procedural Memory

Procedures are action policies, not facts. They are retrieved separately from semantic claims and carry allowed contexts, steps, confidence, and status.

## Contradiction Index

Retrieval returns contradictions alongside supporting evidence. A memory node such as `M_bridge_a_passable` can point to semantic memories, evidence, or rules that conflict with it.

## Retrieval Policy

Retrieval never returns naked content. Each retrieved item includes:

- `content`
- `confidence`
- `status`
- `epistemic_license`
- `source_episodes`
- `contradictions`
- `allowed_use`
- `forbidden_use`
- `revalidation_requirement`

Urgent use follows the emergency-use protocol:

- `full_premise`: normal use
- `weak_premise`: use with fallback
- `hypothesis_only`: branch alternatives
- `hazard_only`: warning only
- `do_not_use_for_action`: cannot support action

If an action uses degraded memory, the system schedules `post_action_revalidation`.
