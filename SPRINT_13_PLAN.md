# Sprint 13 Plan

## Goal

Expose the active planner/verifier view before or during decision-making, not only after trace replay.

## Minimal Build

- `scripts/epistemic_snapshot.py`
- CLI-first JSON output
- Strict mode for release-gated snapshot completeness

## Doctrine

- Audit is history.
- Snapshot is current cognition.
- A system that cannot expose its current epistemic state is still partially opaque.

## Required Scenarios

- `bridge_a_safe_time_pressure`
- `contradiction_remains_unresolved`
- `contradiction_scoped_by_context`
- `valid_human_promotion_allows_invariant`

## Pass Question

Can the system expose its live epistemic operating state without waiting for post-hoc audit?

Current answer: yes, the CLI snapshot exposes task context, authority-bearing objects, contradictions, constraints, pending work, and the current recommendation.
