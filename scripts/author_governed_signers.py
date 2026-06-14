#!/usr/bin/env python3
"""One-time authoring tool for Sprint 31 — signer-set governance.

Promotes the authorized-design-signers registry to the governed v0.2 schema and signs the
Sprint-31 governance scenarios. The EXISTING ``design_authority`` public key is preserved verbatim
so every Sprint 26–30 committed signature stays valid; this tool never re-signs those scenarios.

Run it once to (re)generate ``authorized_design_signers.json`` (v0.2) and the six governance
scenarios. It is NOT part of ``release_check.sh`` (it generates fresh key material each run); the
committed registry + scenarios are the source of truth. Only PUBLIC keys are written to disk — the
generated private keys live in memory for the duration of this process and are then discarded.

Determinism note: signer lifecycle is expressed in LOGICAL ticks (``evaluation_tick`` /
``valid_from_tick`` / ``expires_at_tick`` / ``revoked_at_tick``), never wall-clock, so the release
gate that evaluates these scenarios is reproducible.
"""

from __future__ import annotations

import json
from collections import OrderedDict
from pathlib import Path

from change_provenance import build_content_change_set, load_baseline_policy
from design_signing import sign_change_set
from replay_asymmetric_key import (
    decode_private_key,
    generate_ephemeral_private_key_pem,
    public_key_pem_from_private_pem,
)

ROOT = Path(__file__).resolve().parents[1]
WORLD = ROOT / "simulations" / "bridge_world"
REGISTRY_FILE = WORLD / "authorized_design_signers.json"
SCENARIOS_DIR = WORLD / "scenarios"

HAZARD = "hazard_gate"
HAZARD_INVARIANT = "D_invariant_hazard_blocks_action"
PRESERVE_CLAIM = "Add an audit_log field that preserves hazard_only blocking and still blocks direct action."
WEAKEN_CLAIM = "Tune the hazard urgency policy while preserving the overall block."


def _existing_design_authority_public_key() -> str:
    """Preserve the committed design_authority public key (its S26–S30 signatures must stay valid)."""
    data = json.loads(REGISTRY_FILE.read_text(encoding="utf-8"))
    entry = data["signers"]["design_authority"]
    return entry["public_key"] if isinstance(entry, dict) else entry


def _governed(public_key, scope, status="active", valid_from_tick=0,
              expires_at_tick=None, revoked_at_tick=None, rotated_to=None) -> OrderedDict:
    return OrderedDict([
        ("public_key", public_key),
        ("scope", scope),
        ("status", status),
        ("valid_from_tick", valid_from_tick),
        ("expires_at_tick", expires_at_tick),
        ("revoked_at_tick", revoked_at_tick),
        ("rotated_to", rotated_to),
    ])


def _preserving_change_set() -> dict:
    baseline = load_baseline_policy(HAZARD)
    return build_content_change_set(HAZARD, baseline, dict(baseline, audit_log=True))


def _weakening_change_set() -> dict:
    baseline = load_baseline_policy(HAZARD)
    return build_content_change_set(HAZARD, baseline, dict(baseline, urgency_overrides_hazard=True))


def _scenario(name, command, proposal_id, claim, change_set, signer, private_key,
              nonce, evaluation_tick, mutation_id) -> OrderedDict:
    signed = OrderedDict(change_set)
    signed["signature"] = sign_change_set(change_set, private_key, signer, nonce)
    proposal = OrderedDict([
        ("proposal_id", proposal_id),
        ("claim", claim),
        ("targets_invariant", HAZARD_INVARIANT),
        ("effect", "extend"),
        ("evaluation_tick", evaluation_tick),
        ("change_set", signed),
        ("source", f"{name}.json"),
        ("requested_use", "design_consolidation"),
        ("mutation_id", mutation_id),
    ])
    return OrderedDict([
        ("name", name),
        ("command", command),
        ("design_proposal", proposal),
    ])


def main() -> int:
    # Generate fresh key material for the governed signers. Private keys stay in memory only.
    private_pems = {name: generate_ephemeral_private_key_pem() for name in (
        "governed_authority", "revoked_authority", "expired_authority", "scoped_authority",
        "replay_authority", "rotated_predecessor", "rotated_successor",
    )}
    keys = {name: decode_private_key(pem) for name, pem in private_pems.items()}
    pub = {name: public_key_pem_from_private_pem(pem) for name, pem in private_pems.items()}

    registry = OrderedDict([
        ("schema", "authorized-design-signers-v0.2"),
        ("note", "Public keys only. Each signer is a governed, authority-bearing object: a scope "
                 "plus a lifecycle (status / valid_from_tick / expires_at_tick / revoked_at_tick / "
                 "rotated_to). Authority is evaluated at the decision tick — a public key is not "
                 "permanent authority. Private signing keys are NEVER committed."),
        ("signers", OrderedDict([
            # Preserved verbatim from v0.1 so every Sprint 26–30 signature stays valid.
            ("design_authority", _governed(_existing_design_authority_public_key(), ["*"])),
            ("governed_authority", _governed(pub["governed_authority"], [HAZARD])),
            ("revoked_authority", _governed(pub["revoked_authority"], [HAZARD], status="revoked")),
            ("expired_authority", _governed(pub["expired_authority"], [HAZARD], expires_at_tick=50)),
            ("scoped_authority", _governed(pub["scoped_authority"], ["consolidation_gate"])),
            ("replay_authority", _governed(pub["replay_authority"], [HAZARD], revoked_at_tick=10)),
            ("rotated_predecessor", _governed(
                pub["rotated_predecessor"], [HAZARD], status="revoked",
                revoked_at_tick=100, rotated_to="rotated_successor")),
            ("rotated_successor", _governed(pub["rotated_successor"], [HAZARD], valid_from_tick=100)),
        ])),
    ])

    scenarios = [
        _scenario(
            "revoked_signer_rejected",
            "Submit a content-bound preserving change signed by a signer whose authority is revoked.",
            "DP_revoked_signer", PRESERVE_CLAIM, _preserving_change_set(),
            "revoked_authority", keys["revoked_authority"], "nonce_revoked",
            evaluation_tick=0, mutation_id="MUT_DP_revoked_signer"),
        _scenario(
            "expired_signer_rejected",
            "Submit a preserving change whose signer's authority has expired by the decision tick.",
            "DP_expired_signer", PRESERVE_CLAIM, _preserving_change_set(),
            "expired_authority", keys["expired_authority"], "nonce_expired",
            evaluation_tick=60, mutation_id="MUT_DP_expired_signer"),
        _scenario(
            "wrong_scope_signer_rejected",
            "Submit a hazard_gate change signed by a signer scoped only to consolidation_gate.",
            "DP_wrong_scope_signer", PRESERVE_CLAIM, _preserving_change_set(),
            "scoped_authority", keys["scoped_authority"], "nonce_scope",
            evaluation_tick=0, mutation_id="MUT_DP_wrong_scope_signer"),
        _scenario(
            "rotated_successor_accepted",
            "Submit a preserving change signed by the rotated successor key, in its validity window.",
            "DP_rotated_successor", PRESERVE_CLAIM, _preserving_change_set(),
            "rotated_successor", keys["rotated_successor"], "nonce_rotated",
            evaluation_tick=150, mutation_id="MUT_DP_rotated_successor"),
        _scenario(
            "revoked_key_cannot_replay_prior_signature",
            "Submit a change carrying a genuine signature whose signer was revoked before this tick.",
            "DP_revoked_replay", PRESERVE_CLAIM, _preserving_change_set(),
            "replay_authority", keys["replay_authority"], "nonce_replay",
            evaluation_tick=20, mutation_id="MUT_DP_revoked_replay"),
        _scenario(
            "signed_weakening_still_blocks_under_governance",
            "Submit a weakening change validly signed by an active, in-scope governed signer.",
            "DP_governed_weaken", WEAKEN_CLAIM, _weakening_change_set(),
            "governed_authority", keys["governed_authority"], "nonce_governed_weaken",
            evaluation_tick=0, mutation_id="MUT_DP_governed_weaken"),
    ]

    REGISTRY_FILE.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")
    for scenario in scenarios:
        path = SCENARIOS_DIR / f"{scenario['name']}.json"
        path.write_text(json.dumps(scenario, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {path.relative_to(ROOT)}")
    print(f"wrote {REGISTRY_FILE.relative_to(ROOT)} (schema v0.2, {len(registry['signers'])} signers)")
    # Private keys go out of scope here and are discarded; only public keys were written.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
