# MVP Scope

## Objective

Build a working local prototype where a human route command flows through the full cognitive loop in a toy bridge world.

## In Scope

- Local language command parsing with a stub codec.
- Typed internal packet dictionaries following CIP schemas.
- In-process cognitive bus and trace list.
- Memory retrieval from seeded JSON episodes and rules.
- Conflict and staleness detection.
- Epistemic license generation.
- Attention and budget scoring.
- Simple route planning across Bridge A and Bridge B.
- Sandboxed toy action execution.
- Outcome recording and memory update proposal.

## First Scenario

The agent must choose a route under time pressure:

- Bridge A is shorter but weather-sensitive.
- Bridge B is longer but more stable.
- Weather can change.
- Some memories are stale.
- Some rules conflict.

## Success Criteria

- `./scripts/dev.sh` runs a full demo locally.
- The demo emits packets for each major loop stage.
- The chosen route includes a verifier decision and epistemic license.
- The action outcome is recorded.
- A memory mutation packet is produced rather than silently applied.

