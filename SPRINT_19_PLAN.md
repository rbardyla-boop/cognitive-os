# Sprint 19 Plan

## Goal

A replay ledger may explain prior recovery state, but it may not forge verified-idempotent state
without a trusted replay identity. This seals the at-rest replay authority surface left open after
Sprint 17 (idempotent replay) and Sprint 18 (config boundary).

## Minimal Scope

- Ledger schema validation (`recovery-ledger-v1`).
- Ledger provenance block: `{schema, writer, run_id, integrity}`.
- `run_id` binding via a deterministic integrity digest over the records.
- `job_id` / `mutation_id` / `trace_id` / `source_packet_id` consistency checks.
- Ledger tamper rejection.
- `scenario.replay_ledger` trust boundary (untrusted unless explicitly test-trusted).

## Trust Model

- **Empty / absent ledger** → trusted (suppresses nothing).
- **Provided ledger** (`--ledger` file or in-process round-trip) → must carry a provenance block
  whose `run_id` and `integrity` recompute correctly over its records, and whose records are
  internally consistent. Otherwise rejected (`ledger_schema_invalid`, `ledger_integrity_mismatch`,
  `ledger_job_mutation_mismatch`).
- **Embedded ledger** (`scenario.replay_ledger`) → untrusted unless the scenario sets
  `replay_ledger_trust: "test_trusted"`; even then it must pass schema + consistency.
- A non-trusted ledger never suppresses a mutation: the job is re-resolved through the gateway, and
  the verdict (`trusted` / `untrusted` / `rejected`) is reported in replay output and the snapshot.

## Consistency Rules

For every resolved job, each `mutation_id` must equal `MUT_<job_id>`, must be backed by an
`applied_mutations` record whose `job_id` matches, and whose `trace_id` / `source_packet_id` agree.
A trusted ledger record may only suppress the job it actually identifies (live trace/source binding).

## Required Scenarios

- `forged_ledger_verified_idempotent_rejected`
- `ledger_job_mutation_mismatch_rejected`
- `valid_ledger_verifies_without_reapply`
- `scenario_embedded_ledger_requires_trust_marker`

## Acceptance

- A forged ledger cannot mark a job verified-idempotent unless it matches a trusted replay identity.
- Ledger `mutation_id`s must correspond to prior accepted mutation records.
- `job_id`, `mutation_id`, `trace_id`, `source_packet_id` bind consistently.
- An embedded scenario `replay_ledger` is untrusted unless explicitly marked test-trusted.
- A valid ledger still allows run-2 verify-not-reapply behavior.
- `epistemic_snapshot.py` reports ledger rejection or untrusted ledger status.
- `release_check.sh` stays silent.

## Hard Boundary

Config cannot be authority. Ledger cannot be authority unless its identity is verified. Replay
identity is an authority-bearing object.

## Doctrine

A ledger is evidence of prior replay, not proof of prior replay. At-rest recovery state must
authenticate itself before it can suppress mutation.
