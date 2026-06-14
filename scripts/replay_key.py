#!/usr/bin/env python3
"""Keyed replay identity for recovery ledgers (HMAC-SHA256).

Integrity proves a ledger is internally consistent. A signature proves an
authorized runtime accepted responsibility for it. Only a signature may suppress
a mutation. Keys never live in source, scenarios, or committed fixtures: they are
loaded explicitly from a key file or environment, and absence of a key downgrades
a ledger to audit-only.
"""

from __future__ import annotations

import binascii
import hashlib
import hmac
import json
import os


SIGNATURE_SCHEME = "hmac-sha256"
KEY_ENV_VAR = "COGNITIVE_OS_REPLAY_HMAC_KEY_HEX"


def load_replay_key(ledger_key_file: str | None = None) -> bytes | None:
    """Resolve the HMAC key by explicit source only: --ledger-key-file, then env, then none."""
    if ledger_key_file:
        with open(ledger_key_file, "r", encoding="utf-8") as handle:
            return _decode_key(handle.read().strip())
    env_value = os.environ.get(KEY_ENV_VAR)
    if env_value:
        return _decode_key(env_value.strip())
    return None


def _decode_key(raw: str) -> bytes:
    try:
        key = binascii.unhexlify(raw)
    except (binascii.Error, ValueError) as exc:
        raise ValueError("replay HMAC key must be hex-encoded") from exc
    if len(key) < 16:
        raise ValueError("replay HMAC key must be at least 16 bytes")
    return key


def key_id(key: bytes) -> str:
    """Non-secret, non-reversible fingerprint of the key for the signature block."""
    return "k_" + hashlib.sha256(key).hexdigest()[:12]


def generate_ephemeral_key_hex() -> str:
    """Test-only ephemeral key. Never persisted to source or committed fixtures."""
    return binascii.hexlify(os.urandom(32)).decode("ascii")


def _signing_payload(ledger: dict, key_id_value: str, signed_at_tick: int) -> bytes:
    provenance = ledger.get("provenance", {}) if isinstance(ledger, dict) else {}
    payload = {
        "scheme": SIGNATURE_SCHEME,
        "key_id": key_id_value,
        "signed_at_tick": signed_at_tick,
        "schema": provenance.get("schema"),
        "run_id": provenance.get("run_id"),
        "integrity": provenance.get("integrity"),
        "resolved_jobs": ledger.get("resolved_jobs", {}),
        "applied_mutations": ledger.get("applied_mutations", {}),
        "failures": ledger.get("failures", {}),
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sign_ledger(ledger: dict, key: bytes, signed_at_tick: int = 0) -> dict:
    key_id_value = key_id(key)
    payload = _signing_payload(ledger, key_id_value, signed_at_tick)
    return {
        "scheme": SIGNATURE_SCHEME,
        "key_id": key_id_value,
        "signed_at_tick": signed_at_tick,
        "payload_digest": hashlib.sha256(payload).hexdigest(),
        "signature_hex": hmac.new(key, payload, hashlib.sha256).hexdigest(),
    }


def verify_signature(ledger: dict, key: bytes | None) -> str:
    """Return a signature status verdict for an at-rest ledger.

    signed_valid     – a permitted signer authenticated these exact records.
    unsigned         – no signature block present.
    no_key           – signed, but no key is available to verify it.
    wrong_key        – signed by a different key than the one loaded.
    signature_invalid – scheme mismatch or records/provenance tampered after signing.
    """
    signature = ledger.get("signature") if isinstance(ledger, dict) else None
    if not isinstance(signature, dict) or not signature.get("signature_hex"):
        return "unsigned"
    if key is None:
        return "no_key"
    if signature.get("scheme") != SIGNATURE_SCHEME:
        return "signature_invalid"
    if signature.get("key_id") != key_id(key):
        return "wrong_key"
    payload = _signing_payload(ledger, signature.get("key_id"), signature.get("signed_at_tick"))
    expected_digest = hashlib.sha256(payload).hexdigest()
    if not hmac.compare_digest(expected_digest, str(signature.get("payload_digest", ""))):
        return "signature_invalid"
    expected_signature = hmac.new(key, payload, hashlib.sha256).hexdigest()
    if not hmac.compare_digest(expected_signature, str(signature.get("signature_hex", ""))):
        return "signature_invalid"
    return "signed_valid"
