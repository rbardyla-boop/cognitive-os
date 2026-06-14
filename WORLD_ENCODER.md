# World Encoder

## Toy World State

v0.1 uses structured bridge-world state:

```json
{
  "bridges": {
    "A": {
      "status": "unknown",
      "rain_exposure": 0.7,
      "damage_report": true
    },
    "B": {
      "status": "passable",
      "rain_exposure": 0.2,
      "damage_report": false
    }
  }
}
```

The full local state also includes location, destination, weather, time budget, and bridge traversal cost.

## Prediction Stub

Given an action and world state, the stub predicts:

- risk
- cost
- likely outcome

The planner uses these predictions for normal and minimax route selection. The action executor records the prediction in the action outcome.

## Latent Encoder

Do not add JEPA-like or latent world encoding in v0.1. First prove packet flow, memory governance, verification, attention, and action feedback.

