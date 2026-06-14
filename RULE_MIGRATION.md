# Rule Migration And Cascades

## Versioned Rules

Rules are immutable once published. A change creates a new version:

- `R_bridge_safety:v1`
- `R_bridge_safety:v2`
- `R_bridge_safety:v3`

No rule is silently mutated.

## Dependency Tracing

Memory nodes track:

- rules they depend on
- source episodes supporting them
- procedures using them
- plans using them

## Impact Score

```text
impact =
dependency_strength
* rule_change_distance
* usage_risk
* memory_confidence
* consequence_severity
```

## Lazy Evaluation

When a rule changes:

- high-risk nodes -> `eager_revalidation`
- medium-risk nodes -> `confidence_reduced`
- low-risk nodes -> `pending_rederivation`
- unused nodes -> `deferred`

The cascade engine returns impact effects without blocking the active run.

