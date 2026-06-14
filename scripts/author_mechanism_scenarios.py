#!/usr/bin/env python3
"""One-time authoring tool for Sprint 32 — mechanism-source content binding.

Authors the five mechanism-source scenarios and adds a governed ``mechanism_authority`` signer to
the registry. The post-image of each change_set is the REAL current ``verifier_engine.py`` with a
targeted edit (a preserving comment, or a weakening of the hazard-rejection branch), bound by
content hash. Signed scenarios are signed with a freshly generated governed Ed25519 key whose
PRIVATE half is discarded — only the PUBLIC key is committed.

Not run by release_check (it generates fresh key material). The committed scenarios + registry +
manifest are the source of truth. Run once after the mechanism code is final, then regenerate the
manifest with ``mechanism_provenance.py --build``.
"""

from __future__ import annotations

import json
from collections import OrderedDict
from pathlib import Path

from change_provenance import content_hash, diff_digest, literal_diff
from design_signing import sign_change_set
from mechanism_provenance import MECHANISM_SOURCE_ARTIFACTS
from replay_asymmetric_key import (
    decode_private_key,
    generate_ephemeral_private_key_pem,
    public_key_pem_from_private_pem,
)

ROOT = Path(__file__).resolve().parents[1]
WORLD = ROOT / "simulations" / "bridge_world"
REGISTRY_FILE = WORLD / "authorized_design_signers.json"
SCENARIOS_DIR = WORLD / "scenarios"

ROLE = "adjudicator"
CONTROL_POINT = "hazard_gate"
ARTIFACT = MECHANISM_SOURCE_ARTIFACTS[ROLE]  # scripts/verifier_engine.py
HAZARD_INVARIANT = "D_invariant_hazard_blocks_action"

# The protected line in adjudicate (hazard hard-contradiction at low pressure must be rejected).
PROTECTED_BRANCH = (
    'if conflict_type == "hard_contradiction" and pressure < 0.45:\n'
    '        return "reject_episode"'
)


def _mechanism_change_set(pre_image: str, post_image: str, *, stale: bool = False) -> OrderedDict:
    pre_hash = content_hash(pre_image)
    post_hash = content_hash(post_image)
    if stale:
        # A change authored against a pre-image that is NOT the current on-disk gate code.
        pre_hash = content_hash(pre_image + "\n# divergent\n")
    diff_text = literal_diff(pre_image, post_image, ARTIFACT)
    return OrderedDict([
        ("binding", "mechanism_source"),
        ("role", ROLE),
        ("target", CONTROL_POINT),
        ("changed_artifact", ARTIFACT),
        ("pre_image", pre_image),
        ("pre_image_hash", pre_hash),
        ("post_image", post_image),
        ("post_image_hash", post_hash),
        ("diff_digest", diff_digest(CONTROL_POINT, ARTIFACT, pre_hash, post_hash, diff_text)),
    ])


def _scenario(name, command, proposal_id, claim, change_set, *, mutation_id) -> OrderedDict:
    proposal = OrderedDict([
        ("proposal_id", proposal_id),
        ("claim", claim),
        ("targets_invariant", HAZARD_INVARIANT),
        ("effect", "extend"),
        ("evaluation_tick", 0),
        ("change_set", change_set),
        ("source", f"{name}.json"),
        ("requested_use", "design_consolidation"),
        ("mutation_id", mutation_id),
    ])
    return OrderedDict([("name", name), ("command", command), ("design_proposal", proposal)])


def main() -> int:
    pre_image = (ROOT / ARTIFACT).read_text(encoding="utf-8")
    if PROTECTED_BRANCH not in pre_image:
        raise SystemExit("adjudicator protected branch not found; update PROTECTED_BRANCH")

    preserving = pre_image + "\n# mechanism review (Sprint 32): preserving audit note, hazard rejection intact.\n"
    weakening_a = pre_image.replace(PROTECTED_BRANCH, PROTECTED_BRANCH.replace('"reject_episode"', '"preserve_as_exception"'))
    weakening_b = pre_image.replace(PROTECTED_BRANCH, PROTECTED_BRANCH.replace('"reject_episode"', '"candidate_rule_revision"'))

    # A governed signer for mechanism-source changes; private key generated then discarded.
    priv_pem = generate_ephemeral_private_key_pem()
    private_key = decode_private_key(priv_pem)
    public_pem = public_key_pem_from_private_pem(priv_pem)

    def signed(change_set: OrderedDict, nonce: str) -> OrderedDict:
        out = OrderedDict(change_set)
        out["signature"] = sign_change_set(change_set, private_key, "mechanism_authority", nonce)
        return out

    scenarios = [
        # 1. The change is bound to a pre-image that is not the current gate code -> blocks at provenance.
        _scenario(
            "mechanism_source_hash_mismatch_fails_release",
            "Submit a mechanism-source change whose pre-image hash does not match the current gate code.",
            "DP_mechanism_hash_mismatch",
            "Refactor the adjudicator while preserving hazard rejection.",
            _mechanism_change_set(pre_image, preserving, stale=True),
            mutation_id="MUT_DP_mechanism_hash_mismatch"),
        # 2. A preserving mechanism-source change to a locked gate, UNSIGNED -> blocks on the signature gate.
        _scenario(
            "unsigned_mechanism_source_change_blocks",
            "Submit a preserving mechanism-source change to the adjudicator with no signature.",
            "DP_mechanism_unsigned",
            "Add an audit note to the adjudicator, preserving hazard rejection.",
            _mechanism_change_set(pre_image, preserving),
            mutation_id="MUT_DP_mechanism_unsigned"),
        # 3. A signed preserving mechanism-source change -> accepted (probe confirms the gate survives).
        _scenario(
            "signed_mechanism_preserving_change_accepts",
            "Submit a signed preserving mechanism-source change to the adjudicator.",
            "DP_mechanism_preserving",
            "Add an audit note to the adjudicator, preserving hazard rejection.",
            signed(_mechanism_change_set(pre_image, preserving), "nonce_mech_preserve"),
            mutation_id="MUT_DP_mechanism_preserving"),
        # 4. A signed WEAKENING of the adjudicator -> blocks BY PROBE (authorship never overrides trace).
        _scenario(
            "signed_mechanism_weakening_change_blocks_by_probe",
            "Submit a signed change that softens the adjudicator's hazard rejection.",
            "DP_mechanism_weakening",
            "Tune the adjudicator's hazard handling while preserving the overall block.",
            signed(_mechanism_change_set(pre_image, weakening_a), "nonce_mech_weaken"),
            mutation_id="MUT_DP_mechanism_weakening"),
        # 5. Policy artifacts untouched, but the gate CODE is weakened -> blocks by probe.
        _scenario(
            "policy_artifact_clean_but_gate_code_weakened_fails",
            "Submit a signed change that weakens the adjudicator while leaving all policy files clean.",
            "DP_mechanism_policy_clean_code_weak",
            "Keep the hazard policy intact and adjust only the adjudicator code.",
            signed(_mechanism_change_set(pre_image, weakening_b), "nonce_mech_policy_clean"),
            mutation_id="MUT_DP_mechanism_policy_clean_code_weak"),
    ]

    for scenario in scenarios:
        path = SCENARIOS_DIR / f"{scenario['name']}.json"
        path.write_text(json.dumps(scenario, indent=2) + "\n", encoding="utf-8")
        print(f"wrote {path.relative_to(ROOT)}")

    # Add the governed mechanism signer (PUBLIC key only) to the registry, preserving existing signers.
    registry = json.loads(REGISTRY_FILE.read_text(encoding="utf-8"))
    registry["signers"]["mechanism_authority"] = OrderedDict([
        ("public_key", public_pem),
        ("scope", [CONTROL_POINT]),
        ("status", "active"),
        ("valid_from_tick", 0),
        ("expires_at_tick", None),
        ("revoked_at_tick", None),
        ("rotated_to", None),
    ])
    REGISTRY_FILE.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")
    print(f"added mechanism_authority signer to {REGISTRY_FILE.relative_to(ROOT)} (public key only)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
