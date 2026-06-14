# Contradiction Repair

Sprint 12 adds a narrow contradiction repair loop for semantic memory nodes.

Repair outcomes:

- `resolved_by_new_evidence`: stronger evidence supersedes or downgrades one semantic interpretation while preserving raw episodes.
- `resolved_by_scope`: both nodes remain valid under explicit scope conditions.
- `unresolved`: both nodes remain visible as contradicted and cannot support full-premise action.

Every repair emits a `contradiction_repair` job and one or more `MemoryMutation` audit records through the mutation gateway.

Sprint 12 review checks:

- raw episode preservation is release-gated through contradiction audit replay
- unresolved contradiction remains visible in post-repair retrieval
- scoped contradiction affects planner behavior through explicit `scope_conditions`

Replay:

```sh
python3 scripts/contradiction_audit.py --scenario contradiction_resolved_by_new_evidence
python3 scripts/contradiction_audit.py --scenario contradiction_scoped_by_context
python3 scripts/contradiction_audit.py --scenario contradiction_remains_unresolved
```
