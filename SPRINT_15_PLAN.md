# Sprint 15 Plan

## Goal

Review whether Reflex, Emergency, or Strained attention modes were justified after operation.

## Scope

- Emit `AttentionModeReviewPacket` after configured attention-review scenarios.
- Route `attention_policy_update` through `mutation_gateway.py`.
- Keep attention review limited to attention thresholds, mode policy, coalescing, and backpressure policy.
- Expose pending `attention_mode_review` in `epistemic_snapshot.py`.
- Replay review decisions with `scripts/attention_review_audit.py`.

## Scenarios

- `reflex_mode_correctly_triggered`
- `reflex_mode_false_alarm`
- `interrupt_storm_recovery_replay`

## Acceptance

- Correct Reflex activation creates no penalty or only scoped policy support.
- False Reflex activation opens attention review without memory/procedure/planner/verifier authority changes.
- Interrupt storm preserves raw packet count, coalesces low-value signals, keeps P0/P1 alive, defers lower priority work, and records Recovery replay.
- All attention corrections pass through mutation authority.

## Doctrine

Attention is policy. Policy can be wrong. Attention correction must not become authority correction.
