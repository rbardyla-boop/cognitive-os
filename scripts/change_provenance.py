#!/usr/bin/env python3
"""Artifact content-hash binding (Sprint 29; extends Sprint 28 delta-to-code provenance).

A change is not the file name and not the prose patch — it is the before/after artifact
content and the behavior it produces. Each control point has a real on-disk policy artifact
whose content defines its protected policy. A ``change_set`` carries the literal ``pre_image``
and ``post_image`` of that artifact plus their hashes and a ``diff_digest`` over the literal
diff. The delta tested by ``trace_diff`` is DERIVED from the literal post-image content, and
provenance holds only when the ``pre_image_hash`` matches the artifact's ACTUAL on-disk content
(a stale or wrong-content pre-image is rejected).

A content-bound change_set is
``{target, changed_artifact, pre_image, pre_image_hash, post_image, post_image_hash,
diff_digest, patch?}``. The optional ``patch`` is a hint only; if present it must equal the
policy derived from the literal post-image (else ``structured_patch_diverges``).
"""

from __future__ import annotations

import difflib
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Each control point is implemented by a real source artifact (for human traceability) and is
# CONFIGURED by a real on-disk policy artifact whose content the probe reads as its baseline
# and which a change_set binds to by content hash.
CONTROL_POINT_ARTIFACTS = {
    "hazard_gate": "scripts/verifier_engine.py",
    "consolidation_gate": "scripts/mutation_gateway.py",
    "naked_fact_gate": "scripts/retrieval_policy.py",
    "raw_append_only_gate": "scripts/raw_episode_store.py",
    "llm_authority_gate": "scripts/raw_episode_store.py",
}

_POLICY_DIR = "simulations/bridge_world/control_point_policies"
CONTROL_POINT_POLICY_ARTIFACTS = {
    "hazard_gate": f"{_POLICY_DIR}/hazard_gate.json",
    "consolidation_gate": f"{_POLICY_DIR}/consolidation_gate.json",
    "naked_fact_gate": f"{_POLICY_DIR}/naked_fact_gate.json",
    "raw_append_only_gate": f"{_POLICY_DIR}/raw_append_only_gate.json",
    "llm_authority_gate": f"{_POLICY_DIR}/llm_authority_gate.json",
}


def canonical_policy_text(policy: dict) -> str:
    """The canonical on-disk serialization of a control-point policy artifact."""
    return json.dumps(policy, indent=2, sort_keys=True) + "\n"


def content_hash(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def literal_diff(pre_image: str, post_image: str, changed_artifact: str) -> str:
    return "\n".join(
        difflib.unified_diff(
            pre_image.splitlines(),
            post_image.splitlines(),
            fromfile=f"a/{changed_artifact}",
            tofile=f"b/{changed_artifact}",
            lineterm="",
        )
    )


def diff_digest(target: str, changed_artifact: str, pre_image_hash: str, post_image_hash: str, diff_text: str) -> str:
    payload = json.dumps(
        {
            "target": target,
            "changed_artifact": changed_artifact,
            "pre_image_hash": pre_image_hash,
            "post_image_hash": post_image_hash,
            "diff": diff_text,
        },
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def load_baseline_policy(target: str, root: Path = ROOT) -> dict:
    """Parse the control point's on-disk baseline policy artifact (the protected content)."""
    path = root / CONTROL_POINT_POLICY_ARTIFACTS[target]
    return json.loads(path.read_text(encoding="utf-8"))


def build_content_change_set(target: str, pre_policy: dict, post_policy: dict, patch: dict | None = None) -> dict:
    """Construct a content-bound change_set (test/scenario helper)."""
    artifact = CONTROL_POINT_POLICY_ARTIFACTS[target]
    pre_image = canonical_policy_text(pre_policy)
    post_image = canonical_policy_text(post_policy)
    pre_hash = content_hash(pre_image)
    post_hash = content_hash(post_image)
    diff_text = literal_diff(pre_image, post_image, artifact)
    change_set = {
        "target": target,
        "changed_artifact": artifact,
        "pre_image": pre_image,
        "pre_image_hash": pre_hash,
        "post_image": post_image,
        "post_image_hash": post_hash,
        "diff_digest": diff_digest(target, artifact, pre_hash, post_hash, diff_text),
    }
    if patch is not None:
        change_set["patch"] = patch
    return change_set


def verify_change_set_provenance(change_set, root: Path = ROOT) -> dict:
    """Verify a content-bound change_set. Returns a dict:
    ``{ok, reason, target, changed_artifact, pre_image_hash, post_image_hash, diff_digest,
    derived_delta}``.

    reason ∈ {verified, missing, artifact_unknown, artifact_mismatch, artifact_missing,
    malformed_images, stale_pre_image, wrong_post_image, diff_digest_mismatch,
    non_applicable_patch, structured_patch_diverges}.
    """
    result = {
        "ok": False,
        "reason": "missing",
        "target": None,
        "changed_artifact": None,
        "pre_image_hash": None,
        "post_image_hash": None,
        "diff_digest": None,
        "derived_delta": None,
    }
    if not isinstance(change_set, dict):
        return result
    target = change_set.get("target")
    changed_artifact = change_set.get("changed_artifact")
    pre_image = change_set.get("pre_image")
    post_image = change_set.get("post_image")
    pre_hash = change_set.get("pre_image_hash")
    post_hash = change_set.get("post_image_hash")
    supplied_diff_digest = change_set.get("diff_digest")
    result.update({"target": target, "changed_artifact": changed_artifact})

    if not target or target not in CONTROL_POINT_POLICY_ARTIFACTS:
        result["reason"] = "artifact_unknown"
        return result
    expected_artifact = CONTROL_POINT_POLICY_ARTIFACTS[target]
    if changed_artifact != expected_artifact:
        result["reason"] = "artifact_mismatch"
        return result
    artifact_path = root / changed_artifact
    if not artifact_path.exists():
        result["reason"] = "artifact_missing"
        return result
    if not isinstance(pre_image, str) or not isinstance(post_image, str):
        result["reason"] = "malformed_images"
        return result

    # The pre-image must match the artifact's ACTUAL on-disk content (reject stale/wrong content).
    actual_pre = artifact_path.read_text(encoding="utf-8")
    actual_pre_hash = content_hash(actual_pre)
    result["pre_image_hash"] = pre_hash
    if pre_hash != actual_pre_hash or content_hash(pre_image) != actual_pre_hash:
        result["reason"] = "stale_pre_image"
        return result
    # The post-image must hash to its declared hash.
    result["post_image_hash"] = post_hash
    if content_hash(post_image) != post_hash:
        result["reason"] = "wrong_post_image"
        return result
    # The diff_digest must bind the literal diff of the real before/after content.
    diff_text = literal_diff(pre_image, post_image, changed_artifact)
    computed_diff_digest = diff_digest(target, changed_artifact, pre_hash, post_hash, diff_text)
    result["diff_digest"] = supplied_diff_digest
    if supplied_diff_digest != computed_diff_digest:
        result["reason"] = "diff_digest_mismatch"
        return result
    # The tested policy is DERIVED from the literal post-image content.
    try:
        derived_policy = json.loads(post_image)
    except (json.JSONDecodeError, ValueError):
        derived_policy = None
    if not isinstance(derived_policy, dict):
        result["reason"] = "non_applicable_patch"
        return result
    # A declared structured patch is a hint only; if present it must equal the literal-derived policy.
    declared_patch = change_set.get("patch")
    if declared_patch is not None and declared_patch != derived_policy:
        result["reason"] = "structured_patch_diverges"
        return result

    adds = post_hash != pre_hash  # a real (non-empty) literal diff is an extension, not a bare preserve
    result["ok"] = True
    result["reason"] = "verified"
    result["derived_delta"] = {"control_point": target, "policy": derived_policy, "adds": adds}
    return result


# Deterministic self-test: run `python3 scripts/change_provenance.py --selftest`.
def _selftest() -> int:
    failures = 0

    def check(label: str, condition: bool) -> None:
        nonlocal failures
        failures += 0 if condition else 1
        print(f"{'PASS' if condition else 'FAIL'}  {label}")

    target = "hazard_gate"
    pre_policy = load_baseline_policy(target)
    weaken = dict(pre_policy, urgency_overrides_hazard=True)

    good = verify_change_set_provenance(build_content_change_set(target, pre_policy, weaken))
    check("verified content change_set -> ok + derived delta", good["ok"] and good["derived_delta"]["policy"] == weaken)
    check("derived policy comes from the literal post-image", good["derived_delta"]["policy"]["urgency_overrides_hazard"] is True)
    check("missing change_set -> missing", verify_change_set_provenance(None)["reason"] == "missing")

    cs = build_content_change_set(target, pre_policy, weaken)
    check(
        "wrong artifact -> artifact_mismatch",
        verify_change_set_provenance({**cs, "changed_artifact": "simulations/bridge_world/control_point_policies/wrong.json"})["reason"] == "artifact_mismatch",
    )
    check("unknown target -> artifact_unknown", verify_change_set_provenance({**cs, "target": "nope"})["reason"] == "artifact_unknown")
    # A stale pre-image (does not match the artifact's real on-disk content) is rejected.
    stale = build_content_change_set(target, dict(pre_policy, urgency_overrides_hazard=True), weaken)
    check("stale pre-image -> stale_pre_image", verify_change_set_provenance(stale)["reason"] == "stale_pre_image")
    check("tampered pre_image_hash -> stale_pre_image", verify_change_set_provenance({**cs, "pre_image_hash": "0" * 64})["reason"] == "stale_pre_image")
    check("wrong post_image_hash -> wrong_post_image", verify_change_set_provenance({**cs, "post_image_hash": "0" * 64})["reason"] == "wrong_post_image")
    check("tampered diff_digest -> diff_digest_mismatch", verify_change_set_provenance({**cs, "diff_digest": "0" * 64})["reason"] == "diff_digest_mismatch")
    check("non-JSON post_image -> non_applicable_patch", verify_change_set_provenance(_repost(cs, "not json")) ["reason"] == "non_applicable_patch")
    # A declared structured patch that diverges from the literal post-image is rejected.
    check(
        "structured patch diverges -> structured_patch_diverges",
        verify_change_set_provenance({**build_content_change_set(target, pre_policy, weaken, patch={})})["reason"] == "structured_patch_diverges",
    )
    check(
        "structured patch matching the literal diff -> verified",
        verify_change_set_provenance(build_content_change_set(target, pre_policy, weaken, patch=weaken))["ok"] is True,
    )
    return 1 if failures else 0


def _repost(change_set: dict, post_image: str) -> dict:
    """Replace the post_image (and re-bind its hash + diff_digest) for testing the parse path."""
    cs = dict(change_set)
    cs["post_image"] = post_image
    cs["post_image_hash"] = content_hash(post_image)
    diff_text = literal_diff(cs["pre_image"], post_image, cs["changed_artifact"])
    cs["diff_digest"] = diff_digest(cs["target"], cs["changed_artifact"], cs["pre_image_hash"], cs["post_image_hash"], diff_text)
    return cs


def main(argv: list[str]) -> int:
    if "--selftest" in argv:
        return _selftest()
    print("usage: change_provenance.py [--selftest]")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
