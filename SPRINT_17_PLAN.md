# Sprint 17 Plan

## Goal

Make recovery replay safe to rerun. Replay must not duplicate mutations, double-promote
objects, or change final state on a second run.

## Minimal Scope

- Add a persisted replay ledger keyed on `(job_id, mutation_id)` to `scripts/recovery_replay.py`.
- A job already resolved in the ledger is **verified**, never re-applied.
- Deterministically coalesce duplicate jobs (same `job_type` + target + source) onto one canonical job.
- Preserve a failed job's original failure record and append append-only retry lineage on retry.
- Surface coalesced jobs and retry lineage in `scripts/epistemic_snapshot.py`.
- Add a `--ledger <path>` CLI flag so rerunning `recovery_replay.py` against the same ledger is idempotent.

## New / Changed Fields

`CorrectionJob` adds: `coalesced_into`, `retry_lineage`, `original_failure`, `idempotent`.
New status: `coalesced`. New counter: `coalesced_duplicate_count` (kept separate from the
Sprint-16 capacity-deferral counter `coalesced_or_deferred_count`).

## Dedup Key

`job_type | sorted(target_object_ids) | source_packet_id`. The canonical job is the first in
deterministic sort order (`priority`, `created_at_tick`, `job_id`); later duplicates are coalesced.

## Acceptance

- Rerunning `recovery_replay.py` does not create duplicate `mutation_id`s.
- Resolved jobs stay resolved and are skipped or verified, not re-applied.
- Duplicate jobs against the same target/source are rejected or coalesced deterministically.
- Failed-job retry preserves the original failure record and appends retry lineage.
- `epistemic_snapshot.py` shows retry lineage when present.

## Scenarios

- `replay_resolved_job_is_idempotent`
- `duplicate_correction_job_is_rejected_or_coalesced`
- `failed_job_retry_preserves_audit_lineage`

## Doctrine

Replay must be explanatory, not generative. A recovery system that changes state differently on a
second run is not recovery; it is a hidden actor.
