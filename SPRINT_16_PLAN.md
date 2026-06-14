# Sprint 16 Plan

## Goal

Make deferred correction work visible, ordered, replayable, bounded, and mutation-governed.

## Minimal Scope

- Add `CorrectionJob`.
- Add `CorrectionQueue`.
- Add `scripts/recovery_replay.py`.
- Support CLI/scenario-level replay only.

## Job Types

- `post_action_revalidation`
- `contradiction_repair`
- `planner_review`
- `attention_mode_review`
- `semantic_consolidation`

## Ordering

Jobs sort deterministically by:

1. priority
2. `created_at_tick`
3. `job_id`

## Acceptance

- Mixed jobs sort deterministically.
- State-changing resolution attaches `mutation_ids` and goes through `mutation_gateway.py`.
- Low-priority excess work is deferred and counted.
- `epistemic_snapshot.py --strict` shows open, deferred, failed, blocked, authority-requiring, and highest-priority correction jobs when a scenario defines a correction queue.

## Doctrine

Deferred work is cognitive debt. Cognitive debt must be visible, prioritized, replayable, and closed or explicitly carried.
