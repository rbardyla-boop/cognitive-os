# Sprint 18 Plan

## Goal

No external configuration may smuggle unauthorized priority, authority, mutation type, job type,
status, or policy fields into the recovery system. This is a boundary-hardening sprint, not a
feature sprint. It closes `INV4b` (config-supplied priority was trusted).

## Minimal Scope

- Strict validation in `CorrectionJob.from_config`.
- Priority normalized through `JOB_PRIORITIES`; out-of-allowlist or wrong-lane priority rejected.
- `job_type` allowlist (keys of `JOB_PRIORITIES`).
- `status` allowlist.
- Authority/mutation field rejection (config is not authority).
- A strict config audit surface that reports rejected attempts instead of crashing.

## Boundary Rules

- `priority` and `required_authority` are derived from `job_type`; a config value is accepted only
  when it already matches the canonical one, and rejected otherwise.
- A config item may set only `ALLOWED_CONFIG_FIELDS` (identity, routing, and prior-failure state).
  Resolution provenance (`resolution`, `mutation_ids`, `idempotent`, `coalesced_into`) and the
  dedup/defer counters are replay *outputs* — config may not assert them, or it would forge
  provenance in the ledger/audit trail. Any other key is rejected.
- Structured fields (`original_failure`, `retry_lineage`) are shape-validated so forbidden keys
  cannot be smuggled inside them. Primitive types are validated, and adversarial input becomes a
  reported rejection (`load_correction_jobs` never crashes the batch), never an uncaught error.
- `AUTHORITY_INJECTION_FIELDS` (`epistemic_license`, `authority_class`, `allowed_use`,
  `forbidden_use`, `verifier_rule_id`, `verifier_decision_id`, `mutation_type`, `requested_use`,
  `authority_snapshot`, `new_status`, `patch`) are rejected with a dedicated reason code.
- Rejections carry a stable `reason` code and the offending `fields`; valid jobs still load.

## Required Scenarios

- `config_priority_outside_allowlist_rejected`
- `config_unknown_job_type_rejected`
- `config_attempts_authority_field_injection_rejected`
- `config_valid_job_loads_without_mutation`

## Acceptance

- `from_config` rejects or normalizes priority only through `JOB_PRIORITIES`.
- Unknown `job_type` cannot enter `CorrectionQueue`.
- Config cannot inject `epistemic_license`, `authority_class`, `allowed_use`, `forbidden_use`,
  `verifier_rule_id`, or mutation authority fields.
- Valid config remains loadable.
- `epistemic_snapshot.py` reports rejected config attempts when relevant.
- `release_check.sh` remains quiet.

## Hard Rule

Config is input. Input is adversarial. Configuration must not be authority.

## Doctrine

A trusted config parser is an authority bypass waiting to happen. Parse first. Validate second.
Only then construct cognitive objects.
