#!/usr/bin/env python3
"""Asymmetric replay identity for recovery ledgers (Ed25519).

HMAC signatures are convenient for local development, but the same secret both
signs and verifies. This module separates those powers: private keys sign
recovery ledgers, public keys verify them, and public verification can never
mint a suppressing replay identity.
"""

from __future__ import annotations

import hashlib
import json

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import ed25519


ASYMMETRIC_SIGNATURE_SCHEME = "ed25519"


def generate_ephemeral_private_key_pem() -> str:
    """Test-only private key material. Never persist this to source fixtures."""
    private_key = ed25519.Ed25519PrivateKey.generate()
    return private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption(),
    ).decode("ascii")


def public_key_pem_from_private_pem(private_pem: str) -> str:
    private_key = decode_private_key(private_pem)
    return private_key.public_key().public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo,
    ).decode("ascii")


def load_private_key(private_key_file: str | None = None) -> ed25519.Ed25519PrivateKey | None:
    if not private_key_file:
        return None
    with open(private_key_file, "rb") as handle:
        return decode_private_key(handle.read())


def load_public_key(public_key_file: str | None = None) -> ed25519.Ed25519PublicKey | None:
    if not public_key_file:
        return None
    with open(public_key_file, "rb") as handle:
        return decode_public_key(handle.read())


def decode_private_key(private_key_pem: str | bytes) -> ed25519.Ed25519PrivateKey:
    data = private_key_pem.encode("ascii") if isinstance(private_key_pem, str) else private_key_pem
    key = serialization.load_pem_private_key(data, password=None)
    if not isinstance(key, ed25519.Ed25519PrivateKey):
        raise ValueError("replay private key must be an Ed25519 private key")
    return key


def decode_public_key(public_key_pem: str | bytes) -> ed25519.Ed25519PublicKey:
    data = public_key_pem.encode("ascii") if isinstance(public_key_pem, str) else public_key_pem
    key = serialization.load_pem_public_key(data)
    if not isinstance(key, ed25519.Ed25519PublicKey):
        raise ValueError("replay public key must be an Ed25519 public key")
    return key


def public_key_id(public_key: ed25519.Ed25519PublicKey) -> str:
    raw = public_key.public_bytes(
        encoding=serialization.Encoding.Raw,
        format=serialization.PublicFormat.Raw,
    )
    return "ed25519_" + hashlib.sha256(raw).hexdigest()[:16]


def _signing_payload(ledger: dict, key_id_value: str, signed_at_tick: int) -> bytes:
    provenance = ledger.get("provenance", {}) if isinstance(ledger, dict) else {}
    payload = {
        "scheme": ASYMMETRIC_SIGNATURE_SCHEME,
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


def sign_ledger_asymmetric(
    ledger: dict,
    private_key: ed25519.Ed25519PrivateKey,
    signed_at_tick: int = 0,
) -> dict:
    public_key = private_key.public_key()
    key_id_value = public_key_id(public_key)
    payload = _signing_payload(ledger, key_id_value, signed_at_tick)
    return {
        "scheme": ASYMMETRIC_SIGNATURE_SCHEME,
        "key_id": key_id_value,
        "signed_at_tick": signed_at_tick,
        "payload_digest": hashlib.sha256(payload).hexdigest(),
        "signature_hex": private_key.sign(payload).hex(),
    }


def verify_asymmetric_signature(ledger: dict, public_key: ed25519.Ed25519PublicKey | None) -> str:
    """Return an Ed25519 signature verdict for an at-rest ledger.

    asymmetric_signed_valid – public key authenticated these exact records.
    unsigned                – no signature block present.
    no_public_key           – signed, but no public key is available.
    wrong_public_key        – signed by a different key than the one loaded.
    signature_invalid       – scheme mismatch, bad hex, or tampered records.
    """
    signature = ledger.get("signature") if isinstance(ledger, dict) else None
    if not isinstance(signature, dict) or not signature.get("signature_hex"):
        return "unsigned"
    if signature.get("scheme") != ASYMMETRIC_SIGNATURE_SCHEME:
        return "signature_invalid"
    if public_key is None:
        return "no_public_key"
    if signature.get("key_id") != public_key_id(public_key):
        return "wrong_public_key"
    payload = _signing_payload(ledger, signature.get("key_id"), signature.get("signed_at_tick"))
    expected_digest = hashlib.sha256(payload).hexdigest()
    if expected_digest != str(signature.get("payload_digest", "")):
        return "signature_invalid"
    try:
        public_key.verify(bytes.fromhex(str(signature.get("signature_hex", ""))), payload)
    except (InvalidSignature, ValueError):
        return "signature_invalid"
    return "asymmetric_signed_valid"
