# Sprint 20 Plan

## Goal

A ledger is trusted only because a permitted signer authenticated it — not because it is well-formed.
This adds a keyed root of trust (HMAC-SHA256) on top of Sprint 19's integrity/consistency. Only a
valid signature may suppress a mutation.

## Decision

HMAC-SHA256 first, not asymmetric signatures. Local-first, single-runtime recovery infrastructure:
HMAC gives signed replay identity with no keypair lifecycle and no premature PKI. Asymmetric signing
can come later if ledgers must be verified by external runtimes without sharing the signing secret.

## Secret Management

- No hardcoded secrets. No secret in scenario files. No secret in committed fixtures.
- No automatic trust without an explicit key source.

Key resolution priority (`scripts/replay_key.py`):

1. `--ledger-key-file <path>`
2. `COGNITIVE_OS_REPLAY_HMAC_KEY_HEX` (hex-encoded, ≥16 bytes)
3. no key → unsigned ledger is audit-only and cannot suppress mutation

## Signature Block

```json
{
  "signature": {
    "scheme": "hmac-sha256",
    "key_id": "k_<sha256(key)[:12]>",
    "signed_at_tick": 0,
    "payload_digest": "...",
    "signature_hex": "..."
  }
}
```

The signed payload covers `scheme`, `key_id`, `signed_at_tick`, the provenance (`schema`, `run_id`,
`integrity`), and the records (`resolved_jobs`, `applied_mutations`, `failures`). `key_id` is a
non-secret, non-reversible fingerprint of the key.

## Trust Resolution

`authenticate_ledger` (Sprint 19: schema/integrity/consistency, embedded trust marker) produces a
base verdict; `_resolve_ledger_trust` then applies the signature gate:

- base `rejected` / `untrusted` → unchanged (signature not evaluated).
- base `trusted`, empty records → `trusted` (nothing to suppress).
- base `trusted`, records, `signature_status == signed_valid` → `trusted` (may suppress).
- base `trusted`, records, signature `unsigned` / `no_key` / `wrong_key` / `signature_invalid`
  → downgraded to `audit_only` (re-applied through the gateway, never suppressed).

`signature_status` verdicts: `signed_valid`, `unsigned`, `no_key`, `wrong_key`, `signature_invalid`.

## Required Scenarios

- `unsigned_ledger_cannot_suppress_mutation`
- `signed_ledger_verifies_without_reapply` (keyed round-trip; ephemeral key, nothing committed)
- `tampered_signed_ledger_rejected` (record tamper → integrity; signature tamper → signature_invalid)
- `wrong_key_rejects_ledger`
- `embedded_test_trusted_ledger_still_test_only`

Keyed positive/tamper/wrong-key behaviors are exercised via ephemeral-key round-trips in
`tests/regression/test_release_gates.py`, `scripts/test.sh`, and `scripts/release_check.sh` (no
signed fixture is committed, because that would require committing a key).

## Acceptance

- Unsigned ledger never suppresses mutation in strict mode.
- Signed ledger run 2 verifies without gateway re-apply.
- Tampering with records, provenance, run_id, or mutation_ids invalidates the signature.
- Wrong key rejects the signature.
- No secrets appear in docs, scenarios, logs, or committed files.
- `epistemic_snapshot.py` reports `signature_status`.
- `release_check.sh` remains silent.

## Hard Boundary

A ledger is not trusted because it is well-formed. A ledger is trusted only because a permitted
signer authenticated it.

## Doctrine

Integrity says the file is internally consistent. Signature says an authorized runtime accepted
responsibility for it. Only signature may suppress mutation.

## Out of Scope

Key rotation, multi-signer trust stores, asymmetric verification, remote attestation. Seal the local
keyed-auth path first.
