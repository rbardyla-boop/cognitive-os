# Sprint 21 Plan

## Goal

Public verification does not imply signing authority.

Sprint 20 proved that only authenticated replay ledgers may suppress recovery mutations, but HMAC
uses one secret for both signing and verification. Sprint 21 adds Ed25519 replay identity so an
external verifier can authenticate a ledger with a public key without gaining the ability to forge a
new trusted ledger.

## Build

- `scripts/replay_asymmetric_key.py`
- Ed25519 private-key signing path
- Ed25519 public-key verification path
- signed replay ledger provenance v2 (`recovery-ledger-v2`)
- legacy HMAC path retained for local development

`recovery_replay.py` now accepts:

```sh
--ledger-private-key-file <path>
--ledger-public-key-file <path>
--ledger-key-file <path>
```

Private-key signing takes precedence when supplied. Public-key-only runs can verify an existing
Ed25519 ledger but cannot sign a fresh one.

## Signature Block

```json
{
  "signature": {
    "scheme": "ed25519",
    "key_id": "ed25519_<sha256(public_key)[:16]>",
    "signed_at_tick": 0,
    "payload_digest": "...",
    "signature_hex": "..."
  }
}
```

The signed payload covers `scheme`, `key_id`, `signed_at_tick`, ledger provenance (`schema`,
`run_id`, `integrity`), and replay records (`resolved_jobs`, `applied_mutations`, `failures`).

## Trust Resolution

- Base integrity/consistency failure remains `rejected`.
- Ed25519 ledger + matching public key + valid signature → `trusted`.
- Ed25519 ledger + missing public key → `audit_only` / `no_public_key`.
- Ed25519 ledger + wrong public key → `audit_only` / `wrong_public_key`.
- Ed25519 ledger + tampered signature/payload → `audit_only` / `signature_invalid`.
- HMAC ledger remains supported with `signature_status: signed_valid`.

Snapshots report both `signature_status` and `asymmetric_signature_status`.

## Tests

- asymmetric signed ledger verifies without secret/private key
- public key can verify but not sign
- wrong public key rejects ledger
- tampered asymmetric ledger is audit-only and re-applied
- HMAC legacy path still verifies

## Acceptance

- External verifier can authenticate a ledger using only a public key.
- External verifier cannot forge a ledger.
- No private keys are committed.
- `epistemic_snapshot.py --strict` exposes `asymmetric_signature_status`.
- `release_check.sh` remains silent.

## Doctrine

Verification is not authorship. A public key can recognize authority; it cannot create it.
