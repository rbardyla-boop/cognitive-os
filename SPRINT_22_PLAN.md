# Sprint 22 Plan

## Goal

Every incoming experience becomes an immutable raw episode before interpretation.

Sprint 22 adds the first raw ingestion kernel. It does not build full semantic extraction yet; it
only proves the ordering and authority boundary:

```text
ExperienceEnvelope -> RawEpisode -> semantic candidate
```

## Build

- `scripts/raw_episode_store.py`
- `scripts/ingest_experience.py`
- `schemas/cip/raw_episode_packet.schema.json`
- raw ingestion snapshot section

## Required Raw Episode Fields

```text
episode_id
trace_id
source
timestamp
logical_tick
raw_payload
modality
capture_context
integrity_digest
ingestion_license
schema_version
parsed_claims
semantic_candidate_ids
```

`parsed_claims` starts empty. Interpretation may reference a raw episode, but it may not overwrite
the raw payload.

## Required Scenarios

```text
experience_ingest_preserves_raw_episode
semantic_candidate_requires_raw_episode
raw_episode_is_append_only
malformed_experience_rejected_without_partial_state
```

## Acceptance

- Raw episode exists before semantic extraction.
- Raw payload is preserved with an integrity digest.
- Raw episode cannot be replaced or mutated by later interpretation.
- Semantic candidate creation requires an existing raw episode.
- Malformed experience is rejected without partial raw or semantic state.
- `epistemic_snapshot.py --strict` shows raw episode provenance when present.

## Doctrine

Experience is evidence before it is meaning. Meaning can be revised; raw capture is preserved.
