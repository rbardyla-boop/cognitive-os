# Verifier And Adjudication

## Trust Score

Trust scoring uses:

- source reliability
- timestamp integrity
- corroboration
- parse confidence
- sensor confidence
- adversarial risk
- recency
- dependency stability

## Conflict Types

- `no_conflict`
- `soft_conflict`
- `hard_contradiction`
- `scope_mismatch`
- `known_exception`
- `unknown_anomaly`

## Adjudication Outcomes

- `reject_episode`
- `preserve_as_exception`
- `candidate_rule_revision`
- `fork_model_context`

For v0.1, rule revision is only a candidate packet-level decision. It does not rewrite a rule.

## Revision Pressure

```text
revision_pressure =
surprisal
* trust_episode
* reproducibility
* context_fit
* corroboration
/
trust_rule
/
known_exception_fit
/
adversarial_risk
```

One strange event should not rewrite a rule. Repeated high-trust anomalies can create `candidate_rule_revision`.

