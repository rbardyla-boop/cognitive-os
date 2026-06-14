# Sprint 23 Plan

## Goal

Raw episodes produce candidate interpretations, not accepted facts.

Sprint 23 adds a narrow semantic extraction layer. It consumes immutable raw episodes and emits
inspectable `CandidateMemoryNode` records. It does not assign authority, consolidate memory, or
mutate state.

## Build

- `scripts/semantic_candidate_extractor.py`
- `schemas/cip/semantic_candidate_packet.schema.json`
- `CandidateMemoryNode`
- snapshot-visible semantic candidate extraction state

## Candidate Defaults

```text
status = semantic_candidate
epistemic_license = hypothesis_only
confidence = 0.0
authority_class = semantic_candidate
allowed_use = retrieval, human_explanation, contradiction_detection
forbidden_use = direct_action, memory_consolidation, rule_revision, safety_certification
```

Every candidate cites:

```text
source_raw_episode_id
source_integrity_digest
source_episodes
extraction_method
modality
```

## LLM Boundary

```text
LLM may parse language and propose typed candidates.
LLM may not assign authority.
LLM may not consolidate.
LLM may not mutate memory.
```

If an LLM proposal includes authority fields, extraction ignores them and emits a non-authoritative
candidate.

## Required Scenarios

```text
raw_episode_generates_semantic_candidates
candidate_defaults_to_hypothesis_only
candidate_cites_raw_episode
llm_output_cannot_create_authoritative_memory
candidate_extraction_failure_preserves_raw_episode
```

## Acceptance

- Candidate nodes are inspectable.
- Candidate nodes cite raw episodes and integrity digests.
- Candidate status is non-authoritative by default.
- LLM output cannot create active/promoted/full-premise memory.
- Extraction failure preserves raw evidence.
- `epistemic_snapshot.py --strict` shows candidate nodes when present.

## Doctrine

Interpretation is a hypothesis until governed evidence assigns authority.
