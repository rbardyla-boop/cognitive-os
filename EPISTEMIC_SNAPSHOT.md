# Epistemic Snapshot

`scripts/epistemic_snapshot.py` reconstructs the current operating view from a scenario trace.

Run:

```sh
python3 scripts/epistemic_snapshot.py --scenario bridge_a_safe_time_pressure
python3 scripts/epistemic_snapshot.py --scenario contradiction_remains_unresolved --strict
```

Snapshot sections:

- `surface` / `surface_role`
- `task`
- `driving_objects`
- `contradictions`
- `decision_constraints`
- `pending_work`
- `current_recommendation`

Strict mode fails if the snapshot omits required state such as EvidenceRequirementLevel, attention mode, planner mode, authority licenses, contradiction state, blocked/allowed actions, pending work, or current recommendation.

This is not a dashboard. It is the CLI proof surface for live epistemic inspection.

It is intentionally distinct from audit tools: audits explain historical decisions or mutations, while snapshots expose current cognition and authority before a supervising human or engine acts on it.
