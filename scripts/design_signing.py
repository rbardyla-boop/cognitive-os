#!/usr/bin/env python3
"""Signed change provenance (Sprint 30) + signer-set governance (Sprint 31).

Content binding proves WHAT changed; trace binding proves what BEHAVIOR changed; signature
binding proves WHO accepted responsibility. A would-be-accept of a content-bound change to a
locked invariant requires a valid Ed25519 signature from an authorized signer over the change's
content digest. Authorization is necessary-not-sufficient: a validly-signed weakening still
blocks by trace.

Reuses the Sprint-21 Ed25519 machinery (private key signs, public key verifies; public
verification never mints signing authority). The signed payload binds the change content, so a
signature cannot be replayed onto a different artifact/diff. Only PUBLIC keys are committed; the
signing private key is generated at authoring time and discarded.

Sprint 31 — a public key is not permanent authority. The signer registry is a governed object:
each signer carries a scope and a lifecycle (active / expired / revoked / rotated). Cryptographic
authorship is proven first; THEN authority is evaluated *at the decision tick*. A valid signature
from a now-revoked, expired, or out-of-scope signer is NOT authorization. Lifecycle is expressed
in LOGICAL ticks (never wall-clock) so the decision is reproducible. As in Sprint 30, governance
only constrains a would-be ACCEPT — it never overrides a trace failure.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from cryptography.exceptions import InvalidSignature

from replay_asymmetric_key import (
    ASYMMETRIC_SIGNATURE_SCHEME,
    decode_public_key,
    public_key_id,
)

ROOT = Path(__file__).resolve().parents[1]
AUTHORIZED_SIGNERS_FILE = ROOT / "simulations" / "bridge_world" / "authorized_design_signers.json"

CHANGE_SIGNATURE_SCHEME = ASYMMETRIC_SIGNATURE_SCHEME  # "ed25519"

# A signer scoped to this token may authorize a change to any control point.
SIGNER_SCOPE_WILDCARD = "*"


def change_signing_payload(change_set: dict, signer: str, nonce: str) -> bytes:
    """Canonical bytes a change signature covers: the change's content digest + signer + nonce.

    Binding all of these means a signature is valid only for this exact content, target, and
    diff — a copied signature cannot authorize a different artifact, target, or diff."""
    payload = {
        "scheme": CHANGE_SIGNATURE_SCHEME,
        "signer": signer,
        "target": change_set.get("target"),
        "control_point": change_set.get("target"),
        "changed_artifact": change_set.get("changed_artifact"),
        "pre_image_hash": change_set.get("pre_image_hash"),
        "post_image_hash": change_set.get("post_image_hash"),
        "diff_digest": change_set.get("diff_digest"),
        "nonce": nonce,
    }
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sign_change_set(change_set: dict, private_key, signer: str, nonce: str) -> dict:
    """Produce a signature block for a change_set (authoring/test helper)."""
    payload = change_signing_payload(change_set, signer, nonce)
    return {
        "scheme": CHANGE_SIGNATURE_SCHEME,
        "signer": signer,
        "key_id": public_key_id(private_key.public_key()),
        "nonce": nonce,
        "payload_digest": hashlib.sha256(payload).hexdigest(),
        "signature_hex": private_key.sign(payload).hex(),
    }


def load_authorized_signers(path: Path = AUTHORIZED_SIGNERS_FILE) -> dict:
    """Return the raw signer registry: {signer_id: governed_signer | public_key_pem}.

    Only PUBLIC keys are stored — never a private key. The value is a governed object
    (Sprint 31, schema v0.2) or, for backward compatibility, a bare public-key PEM string
    (treated as an active, unscoped, non-expiring signer). Use ``normalize_signer_registry``
    to resolve either shape to governed objects.
    """
    if not path.exists():
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    return data.get("signers", {})


def _normalize_signer(entry) -> dict | None:
    """Resolve a registry value to a governed signer object.

    Accepts a bare PEM string (legacy v0.1 — active, wildcard-scoped, non-expiring) or a
    governed v0.2 object. Returns None if the entry carries no public key.
    """
    if isinstance(entry, str) and entry:
        return {
            "public_key": entry,
            "scope": [SIGNER_SCOPE_WILDCARD],
            "status": "active",
            "valid_from_tick": 0,
            "expires_at_tick": None,
            "revoked_at_tick": None,
            "rotated_to": None,
        }
    if isinstance(entry, dict) and entry.get("public_key"):
        return {
            "public_key": entry["public_key"],
            "scope": entry.get("scope", [SIGNER_SCOPE_WILDCARD]),
            "status": entry.get("status", "active"),
            "valid_from_tick": entry.get("valid_from_tick", 0),
            "expires_at_tick": entry.get("expires_at_tick"),
            "revoked_at_tick": entry.get("revoked_at_tick"),
            "rotated_to": entry.get("rotated_to"),
        }
    return None


def normalize_signer_registry(registry) -> dict:
    """Return {signer_id: governed_signer} for either a v0.1 flat map or a v0.2 governed map."""
    out: dict = {}
    if not isinstance(registry, dict):
        return out
    for signer_id, entry in registry.items():
        normalized = _normalize_signer(entry)
        if normalized is not None:
            out[signer_id] = normalized
    return out


def signer_authority(signer_entry: dict, now_tick: int, change_scope) -> tuple[bool, str]:
    """Governance verdict for an ALREADY crypto-verified signer, evaluated at ``now_tick``.

    A public key is not permanent authority: authorship proves the signature is genuine, this
    decides whether the genuine signer is *currently* authorized for *this* change. Returns
    ``(ok, reason)`` with reason ∈ {authorized, signer_revoked, signer_expired,
    signer_not_yet_valid, signer_wrong_scope}. Revocation/expiry are checked against the logical
    decision tick, so a signature that verified before revocation does not authorize a change
    evaluated after it.
    """
    status = signer_entry.get("status", "active")
    revoked_at = signer_entry.get("revoked_at_tick")
    expires_at = signer_entry.get("expires_at_tick")
    valid_from = signer_entry.get("valid_from_tick", 0)
    # An OMITTED scope (None) inherits the legacy unscoped (wildcard) default; an EXPLICIT empty
    # scope ([]) authorizes NOTHING (fail closed) — `or` would wrongly collapse [] to wildcard.
    scope = signer_entry.get("scope")
    if scope is None:
        scope = [SIGNER_SCOPE_WILDCARD]
    if status == "revoked" or (revoked_at is not None and now_tick >= revoked_at):
        return False, "signer_revoked"
    if status == "expired" or (expires_at is not None and now_tick >= expires_at):
        return False, "signer_expired"
    if valid_from is not None and now_tick < valid_from:
        return False, "signer_not_yet_valid"
    if change_scope is not None and SIGNER_SCOPE_WILDCARD not in scope and change_scope not in scope:
        return False, "signer_wrong_scope"
    return True, "authorized"


def verify_change_signature(
    change_set,
    authorized_signers: dict,
    now_tick: int = 0,
    change_scope=None,
) -> dict:
    """Verify a change_set's signature against the governed signer registry.

    Two layers, in order: (1) cryptographic authorship — the signature is genuine and binds this
    exact content; (2) Sprint-31 governance — the genuine signer is currently authorized for this
    change at ``now_tick`` (unrevoked, unexpired, in valid window, in scope). ``change_scope``
    defaults to the change_set's target control point.

    Returns {ok, reason, signer, payload_digest, signer_status, signer_scope, signer_expires_at,
    signer_revoked_at, rotated_to, now_tick, change_scope}. reason ∈ {signature_verified, unsigned,
    unauthorized_signer, wrong_key, signature_payload_mismatch, signature_invalid, signer_revoked,
    signer_expired, signer_not_yet_valid, signer_wrong_scope}.
    """
    registry = normalize_signer_registry(authorized_signers)
    if change_scope is None and isinstance(change_set, dict):
        change_scope = change_set.get("target")
    result = {
        "ok": False,
        "reason": "unsigned",
        "signer": None,
        "payload_digest": None,
        "signer_status": None,
        "signer_scope": None,
        "signer_expires_at": None,
        "signer_revoked_at": None,
        "rotated_to": None,
        "now_tick": now_tick,
        "change_scope": change_scope,
    }
    signature = change_set.get("signature") if isinstance(change_set, dict) else None
    if not isinstance(signature, dict) or not signature.get("signature_hex"):
        return result
    signer = signature.get("signer")
    result["signer"] = signer
    result["payload_digest"] = signature.get("payload_digest")
    if signature.get("scheme") != CHANGE_SIGNATURE_SCHEME:
        result["reason"] = "signature_invalid"
        return result
    signer_entry = registry.get(signer)
    if not signer_entry:
        result["reason"] = "unauthorized_signer"
        return result
    # Surface governance metadata for transparency regardless of the eventual verdict.
    result["signer_status"] = signer_entry.get("status")
    result["signer_scope"] = signer_entry.get("scope")
    result["signer_expires_at"] = signer_entry.get("expires_at_tick")
    result["signer_revoked_at"] = signer_entry.get("revoked_at_tick")
    result["rotated_to"] = signer_entry.get("rotated_to")
    try:
        public_key = decode_public_key(signer_entry.get("public_key"))
    except (ValueError, TypeError):
        result["reason"] = "signature_invalid"
        return result
    if signature.get("key_id") != public_key_id(public_key):
        result["reason"] = "wrong_key"
        return result
    payload = change_signing_payload(change_set, signer, signature.get("nonce"))
    if hashlib.sha256(payload).hexdigest() != str(signature.get("payload_digest", "")):
        # The signature was made over different content (a replay onto a different artifact/diff).
        result["reason"] = "signature_payload_mismatch"
        return result
    try:
        public_key.verify(bytes.fromhex(str(signature.get("signature_hex", ""))), payload)
    except (InvalidSignature, ValueError):
        result["reason"] = "signature_invalid"
        return result
    # Sprint 31 — authorship is genuine; now governance decides authority AT THIS TICK.
    governed_ok, governance_reason = signer_authority(signer_entry, now_tick, change_scope)
    if not governed_ok:
        result["reason"] = governance_reason
        return result
    result["ok"] = True
    result["reason"] = "signature_verified"
    return result


# Deterministic self-test: run `python3 scripts/design_signing.py --selftest`.
def _selftest() -> int:
    from replay_asymmetric_key import (
        decode_private_key,
        generate_ephemeral_private_key_pem,
        public_key_pem_from_private_pem,
    )

    failures = 0

    def check(label: str, condition: bool) -> None:
        nonlocal failures
        failures += 0 if condition else 1
        print(f"{'PASS' if condition else 'FAIL'}  {label}")

    priv_pem = generate_ephemeral_private_key_pem()
    private_key = decode_private_key(priv_pem)
    pub_pem = public_key_pem_from_private_pem(priv_pem)
    signers = {"design_authority": pub_pem}

    change_set = {
        "target": "hazard_gate",
        "changed_artifact": "simulations/bridge_world/control_point_policies/hazard_gate.json",
        "pre_image_hash": "a" * 64,
        "post_image_hash": "b" * 64,
        "diff_digest": "c" * 64,
    }
    signed = dict(change_set, signature=sign_change_set(change_set, private_key, "design_authority", "nonce-1"))
    check("valid signature -> verified", verify_change_signature(signed, signers)["reason"] == "signature_verified")
    check("unsigned -> unsigned", verify_change_signature(change_set, signers)["reason"] == "unsigned")
    check(
        "unauthorized signer -> unauthorized_signer",
        verify_change_signature(dict(change_set, signature=sign_change_set(change_set, private_key, "rogue", "n")), signers)["reason"] == "unauthorized_signer",
    )
    # A different authorized key for the same signer name -> key_id mismatch.
    other_priv = decode_private_key(generate_ephemeral_private_key_pem())
    other_signed = dict(change_set, signature=sign_change_set(change_set, other_priv, "design_authority", "n"))
    check("wrong key -> wrong_key", verify_change_signature(other_signed, signers)["reason"] == "wrong_key")
    # Replay: a valid signature copied onto a change_set with different content.
    replayed = dict(change_set, post_image_hash="d" * 64, signature=signed["signature"])
    check("replay onto different content -> signature_payload_mismatch", verify_change_signature(replayed, signers)["reason"] == "signature_payload_mismatch")
    # Tampered signature bytes.
    bad_sig = dict(signed["signature"], signature_hex="00" * 64)
    check("tampered signature -> signature_invalid", verify_change_signature(dict(change_set, signature=bad_sig), signers)["reason"] == "signature_invalid")

    # Sprint 31 — signer-set governance. A genuine signature is still subject to lifecycle + scope
    # evaluated at the decision tick. The SAME signed change is authorized before, and rejected
    # after, the signer's revocation tick (authority evaluated at decision time, not signing time).
    governed = {
        "design_authority": {  # active, wildcard scope, non-expiring
            "public_key": pub_pem,
            "scope": ["*"],
            "status": "active",
            "valid_from_tick": 0,
            "expires_at_tick": None,
            "revoked_at_tick": None,
            "rotated_to": None,
        },
        "revoked_now": {  # genuine key, but lifecycle status is revoked
            "public_key": pub_pem, "scope": ["hazard_gate"], "status": "revoked",
            "valid_from_tick": 0, "expires_at_tick": None, "revoked_at_tick": None, "rotated_to": None,
        },
        "ticking": {  # crypto-valid, but revoked at tick 10
            "public_key": pub_pem, "scope": ["hazard_gate"], "status": "active",
            "valid_from_tick": 0, "expires_at_tick": None, "revoked_at_tick": 10, "rotated_to": None,
        },
        "expiring": {
            "public_key": pub_pem, "scope": ["hazard_gate"], "status": "active",
            "valid_from_tick": 0, "expires_at_tick": 50, "revoked_at_tick": None, "rotated_to": None,
        },
        "scoped": {
            "public_key": pub_pem, "scope": ["consolidation_gate"], "status": "active",
            "valid_from_tick": 0, "expires_at_tick": None, "revoked_at_tick": None, "rotated_to": None,
        },
        "successor": {
            "public_key": pub_pem, "scope": ["hazard_gate"], "status": "active",
            "valid_from_tick": 100, "expires_at_tick": None, "revoked_at_tick": None, "rotated_to": None,
        },
    }
    hz = dict(change_set, target="hazard_gate",
              changed_artifact="simulations/bridge_world/control_point_policies/hazard_gate.json")
    hz_signed = dict(hz, signature=sign_change_set(hz, private_key, "design_authority", "g"))
    check("active wildcard signer -> verified", verify_change_signature(hz_signed, governed, now_tick=0)["reason"] == "signature_verified")
    # Status-revoked signer: genuine signature, but lifecycle status is revoked.
    hz_for_revoked = dict(hz, signature=sign_change_set(hz, private_key, "revoked_now", "g"))
    check("status-revoked signer -> signer_revoked", verify_change_signature(hz_for_revoked, governed, now_tick=0)["reason"] == "signer_revoked")
    hz_for_ticking = dict(hz, signature=sign_change_set(hz, private_key, "ticking", "g"))
    check("crypto-valid but revoked@10, decided at tick 5 -> verified", verify_change_signature(hz_for_ticking, governed, now_tick=5)["reason"] == "signature_verified")
    check("SAME signature decided at tick 20 -> signer_revoked", verify_change_signature(hz_for_ticking, governed, now_tick=20)["reason"] == "signer_revoked")
    hz_for_expiring = dict(hz, signature=sign_change_set(hz, private_key, "expiring", "g"))
    check("expired signer (tick 60 > 50) -> signer_expired", verify_change_signature(hz_for_expiring, governed, now_tick=60)["reason"] == "signer_expired")
    hz_for_scoped = dict(hz, signature=sign_change_set(hz, private_key, "scoped", "g"))
    check("out-of-scope signer (consolidation_gate signing hazard_gate) -> signer_wrong_scope", verify_change_signature(hz_for_scoped, governed, now_tick=0)["reason"] == "signer_wrong_scope")
    hz_for_succ = dict(hz, signature=sign_change_set(hz, private_key, "successor", "g"))
    check("rotated successor before valid_from (tick 0 < 100) -> signer_not_yet_valid", verify_change_signature(hz_for_succ, governed, now_tick=0)["reason"] == "signer_not_yet_valid")
    check("rotated successor in window (tick 150) -> verified", verify_change_signature(hz_for_succ, governed, now_tick=150)["reason"] == "signature_verified")
    # Fail-closed scope: an EXPLICIT empty scope [] authorizes NOTHING (must not collapse to wildcard).
    empty_scope = {"empty": {"public_key": pub_pem, "scope": [], "status": "active",
                             "valid_from_tick": 0, "expires_at_tick": None, "revoked_at_tick": None, "rotated_to": None}}
    hz_empty = dict(hz, signature=sign_change_set(hz, private_key, "empty", "g"))
    check("explicit empty scope [] -> signer_wrong_scope (fail closed, not wildcard)", verify_change_signature(hz_empty, empty_scope, now_tick=0)["reason"] == "signer_wrong_scope")
    return 1 if failures else 0


def main(argv: list[str]) -> int:
    if "--selftest" in argv:
        return _selftest()
    print("usage: design_signing.py [--selftest]")
    return 1


if __name__ == "__main__":
    import sys

    raise SystemExit(main(sys.argv[1:]))
