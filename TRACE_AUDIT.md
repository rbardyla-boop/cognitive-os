# Trace Replay And Decision Audit

The decision audit reconstructs why the system selected a plan from emitted packets.

Run:

```sh
python3 scripts/decision_audit.py --scenario bridge_a_safe_time_pressure
```

Example output shape:

```json
{
  "decision": "recommend Bridge B",
  "primary_factors": [
    "User preferred Bridge A.",
    "Urgency parsed as high.",
    "Bridge A had active damage report after rain.",
    "Damage/risk evidence produced hazard_only contradiction packets.",
    "Attention Manager entered Reflex.",
    "Planner switched to minimax.",
    "Bridge B had lower worst-case consequence or safer fallback.",
    "ActionOutcome and post_action_revalidation were scheduled."
  ],
  "blocked_alternatives": [
    "Bridge A direct recommendation blocked by hazard_only contradiction evidence.",
    "Bridge A safety certification blocked by forbidden_use metadata."
  ]
}
```

This is the release-grade proof surface for the first safety-degraded planning path.

