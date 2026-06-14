# Mutation Authority

Sprint 10 introduces `apply_memory_mutation(...)` as the central gateway for governed state mutation.

Required mutation request fields:

- `mutation_id`
- `trace_id`
- `source_packet_id`
- `verifier_decision_id`
- `target_object_id`
- `requested_use`
- `mutation_type`
- `authority_snapshot`

Supported mutation types:

- `semantic_status_update`
- `bootstrap_promotion`
- `bootstrap_rejection`
- `memory_confidence_update`
- `procedure_status_update`
- `rule_status_update`

Every accepted or rejected attempt records:

- before/after status
- source packet
- verifier decision
- blocking or authorizing rule
- reason

Rejected attempts are first-class audit records.
