# Planner Regret

Sprint 14 adds policy-level feedback after action outcomes.

Planner regret emits:

- `PlanRegretPacket`
- `planner_review` pending work
- `planner_policy_update` mutation through the mutation gateway

Planner regret does not mutate belief authority, procedure authority, or global rules.

Sprint 14 review hardening:

- `planner_policy_update` is rejected unless the target object is a planner policy.
- Planner policy updates cannot patch authority fields such as `epistemic_license`, `authority_class`, `allowed_use`, or `forbidden_use`.
- Regret classes separate `policy_success`, `safety_near_miss`, and `opportunity_cost`.
- Pending planner reviews carry an explicit `status` and remain visible in `epistemic_snapshot.py` while open.

Replay:

```sh
python3 scripts/planner_regret_audit.py --scenario planner_correct_under_uncertainty
python3 scripts/planner_regret_audit.py --scenario planner_near_miss_requires_policy_review
python3 scripts/planner_regret_audit.py --scenario planner_overconservative_waits_unnecessarily
```
