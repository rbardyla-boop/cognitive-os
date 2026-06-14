# Recovery Replay

Sprint 16 adds a unified correction queue for local replay.

> Sprint 24 (the Caitlin leap) reuses the same deferred-correction shape at the meta
> level: a design proposal that weakens a locked invariant emits a `BackpressureCommand`
> of type `design_revalidation` that keeps the release blocked until an explicit human
> exception plus revalidation resolves it — the design-process analog of
> `post_action_revalidation`.

The queue uses `CorrectionJob` records with:

- `job_id`
- `job_type`
- `source_packet_id`
- `trace_id`
- `priority`
- `status`
- `created_at_tick`
- `updated_at_tick`
- `target_object_ids`
- `required_authority`
- `deferred_reason`
- `resolution`
- `mutation_ids`

Statuses are `open`, `processing`, `resolved`, `deferred`, `failed`, and `coalesced`.

Priority order is deterministic:

- `P0` safety interrupt recovery
- `P1` action correction / post-action revalidation
- `P2` contradiction repair
- `P3` planner review
- `P4` attention mode review
- `P5` semantic consolidation

Replay:

```sh
python3 scripts/recovery_replay.py --scenario recovery_queue_orders_mixed_jobs
python3 scripts/recovery_replay.py --scenario recovery_replay_resolves_jobs_through_gateway
python3 scripts/recovery_replay.py --scenario recovery_queue_bounds_deferred_work
```

This is not a background daemon. It is a deterministic local proof surface for cognitive debt.

## Sprint 17: Idempotent Recovery and Replay Safety

Replay is explanatory, not generative. Rerunning must not duplicate mutations, double-promote
objects, or change final state.

A **replay ledger** records resolved jobs and applied mutations, keyed on `(job_id, mutation_id)`.
On replay, a job already resolved in the ledger is **verified** (a `verify` mutation-log entry that
re-references the original `mutation_id`) and is never re-applied through the gateway. The ledger
can be threaded in-process (`replay_queue(queue, scenario, ledger=...)` / `scenario.replay_ledger`)
or persisted across CLI runs:

```sh
rm -f /tmp/recovery_ledger.json
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger /tmp/recovery_ledger.json  # applies
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger /tmp/recovery_ledger.json  # verifies, no re-apply
python3 scripts/recovery_replay.py --scenario duplicate_correction_job_is_rejected_or_coalesced
python3 scripts/recovery_replay.py --scenario failed_job_retry_preserves_audit_lineage
```

**Duplicate coalescing.** Jobs sharing a dedup key (`job_type | sorted(target_object_ids) |
source_packet_id`) are coalesced onto the canonical job — the first in deterministic sort order.
Later duplicates move to status `coalesced` with `coalesced_into` set, are counted in
`coalesced_duplicate_count`, and never emit a second `mutation_id`.

**Retry lineage.** A job that enters replay already `failed` is retried while its
`original_failure` record is preserved and each attempt is appended to an append-only
`retry_lineage`. `epistemic_snapshot.py` surfaces coalesced jobs and `jobs_with_retry_lineage`.

## Sprint 18: Configuration Boundary Validation

Config is input; input is adversarial; configuration must not be authority. `CorrectionJob.from_config`
parses, then validates, then constructs:

- `priority` and `required_authority` are derived from `job_type` — never trusted from config. A
  supplied value is accepted only if it already matches the canonical one, otherwise rejected
  (`priority_not_in_allowlist`, `priority_overrides_job_type`, `required_authority_override`).
- `job_type` and `status` are allowlisted (`unknown_job_type`, `invalid_status`).
- Only `ALLOWED_CONFIG_FIELDS` may appear; authority/mutation fields are rejected outright
  (`authority_field_injection`), any other stray key is rejected (`unknown_config_field`).
- Resolution provenance (`resolution`, `mutation_ids`, `idempotent`, `coalesced_into`) and the
  counters are replay *outputs*, not config entry fields, so config cannot forge applied-mutation
  provenance in the ledger. Structured fields are shape-validated (`nested_unknown_field`) and
  primitive types are checked (`invalid_field_type`); a malformed item is a reported rejection,
  never a crash that drops the rest of the batch.

`load_correction_jobs` collects rejected attempts as `{job_id, reason, fields, detail}` rather than
crashing the queue: an invalid job cannot enter `CorrectionQueue`, but the rejection stays visible in
`replay["queue"]["rejected_config_attempts"]` and in the snapshot's correction-queue section.

## Sprint 19: Ledger Integrity / Replay-Identity Authentication

A ledger is evidence of prior replay, not proof of prior replay — it must authenticate before it can
suppress a mutation. Every ledger the system writes is stamped with a provenance block
`{schema: "recovery-ledger-v1", writer, run_id, integrity}`, where `run_id` and `integrity` are
deterministic digests over the records. `authenticate_ledger` returns `trusted` / `untrusted` /
`rejected`:

- **Provided** (`--ledger` file / in-process): provenance required; `run_id` + `integrity` must
  recompute over the records (tamper → `ledger_integrity_mismatch`); records must be consistent.
- **Embedded** (`scenario.replay_ledger`): untrusted unless the scenario sets
  `replay_ledger_trust: "test_trusted"`; even then schema + consistency are enforced.
- **Consistency**: each `mutation_id` must equal `MUT_<job_id>`, be backed by an `applied_mutations`
  record whose `job_id`/`trace_id`/`source_packet_id` agree (`ledger_job_mutation_mismatch`).

A non-trusted ledger never suppresses a mutation: the job is re-resolved through the gateway, and the
verdict is reported in `replay["replay"]["ledger_authentication"]` and the snapshot's correction queue.

```sh
python3 scripts/recovery_replay.py --scenario scenario_embedded_ledger_requires_trust_marker
python3 scripts/recovery_replay.py --scenario forged_ledger_verified_idempotent_rejected
python3 scripts/recovery_replay.py --scenario ledger_job_mutation_mismatch_rejected
```

## Sprint 20: Signed / Keyed Replay Identity

Integrity says the records are internally consistent; a signature says a permitted signer accepted
responsibility. **Only a valid signature may suppress a mutation.** `scripts/replay_key.py` provides
HMAC-SHA256 signing; the key is resolved by explicit source only and never committed:

1. `--ledger-key-file <path>`
2. `COGNITIVE_OS_REPLAY_HMAC_KEY_HEX` (hex, ≥16 bytes)
3. no key → unsigned ledger is audit-only and cannot suppress

When a key is present, the written ledger carries a `signature` block (`scheme`, `key_id`,
`signed_at_tick`, `payload_digest`, `signature_hex`) over the provenance + records. On read,
`_resolve_ledger_trust` combines the Sprint-19 base verdict with a `signature_status`
(`signed_valid` / `unsigned` / `no_key` / `wrong_key` / `signature_invalid`). A well-formed but
unsigned, wrong-key, or signature-tampered ledger is downgraded to `audit_only` and re-applied
through the gateway — it never suppresses. The snapshot reports `signature_status`.

```sh
# signed round-trip (ephemeral key; nothing committed):
key=$(python3 -c "import os,binascii;print(binascii.hexlify(os.urandom(32)).decode())")
printf '%s' "$key" > /tmp/replay.key
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-key-file /tmp/replay.key --ledger /tmp/led.json   # signs
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent --ledger-key-file /tmp/replay.key --ledger /tmp/led.json   # verifies, no re-apply
python3 scripts/recovery_replay.py --scenario unsigned_ledger_cannot_suppress_mutation
python3 scripts/recovery_replay.py --scenario embedded_test_trusted_ledger_still_test_only
```

```sh
python3 scripts/recovery_replay.py --scenario config_priority_outside_allowlist_rejected
python3 scripts/recovery_replay.py --scenario config_unknown_job_type_rejected
python3 scripts/recovery_replay.py --scenario config_attempts_authority_field_injection_rejected
python3 scripts/recovery_replay.py --scenario config_valid_job_loads_without_mutation
```

## Sprint 21: Asymmetric Replay Identity

Verification is not authorship. Sprint 21 adds Ed25519 replay identity so an external verifier can
authenticate a ledger with only a public key, while only the private key can produce a trusted
signature. Newly written ledgers use provenance schema `recovery-ledger-v2`; existing
`recovery-ledger-v1` ledgers remain readable for migration/testing.

CLI options:

```sh
python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent \
  --ledger-private-key-file /tmp/replay_ed25519_private.pem \
  --ledger /tmp/recovery_ledger.json

python3 scripts/recovery_replay.py --scenario replay_resolved_job_is_idempotent \
  --ledger-public-key-file /tmp/replay_ed25519_public.pem \
  --ledger /tmp/recovery_ledger.json
```

Ed25519 signature verdicts appear as `asymmetric_signature_status`:

- `asymmetric_signed_valid`
- `unsigned`
- `no_public_key`
- `wrong_public_key`
- `signature_invalid`
- `not_asymmetric`

The legacy HMAC path remains supported for local development through `--ledger-key-file` or
`COGNITIVE_OS_REPLAY_HMAC_KEY_HEX`. Private-key signing takes precedence when an Ed25519 private key
is provided; public-key-only runs can verify existing ledgers but cannot sign fresh ledgers.
