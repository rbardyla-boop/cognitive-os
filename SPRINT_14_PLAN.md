# Sprint 14 Plan

## Goal

After action, compare expected outcome against actual outcome and create governed planner-correction signals without directly rewriting belief or procedure authority.

## Doctrine

- An outcome does not only update memory.
- It also evaluates the policy that chose the action.
- Planner regret is policy feedback, not automatic belief or procedure promotion.

## Proof Scenarios

- `planner_correct_under_uncertainty`
- `planner_near_miss_requires_policy_review`
- `planner_overconservative_waits_unnecessarily`

## Acceptance

- Correct degraded decision strengthens planner policy confidence only within scope.
- Near miss emits `PlanRegretPacket` and opens review, without automatic global rule rewrite.
- Overconservative wait is opportunity-cost regret, not a safety failure.
- Planner-regret corrections go through `mutation_gateway.py`.
- `epistemic_snapshot.py` shows pending planner review when present.
