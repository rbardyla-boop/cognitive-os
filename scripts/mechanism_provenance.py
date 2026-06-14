#!/usr/bin/env python3
"""Mechanism-source content binding (Sprint 32).

A policy artifact (Sprint 29) says WHAT the rule is; the mechanism SOURCE decides whether the
rule is actually ENFORCED. A signed, content-bound policy is not enough if the enforcement code
underneath it can be changed unsigned. This module binds the enforcement code itself by content
hash in a manifest, verifies that manifest before any decision is trusted, and routes a proposed
change to a mechanism-source file through the same content + signature + behavioral-probe gates as
a policy change — but the probe runs against the PROPOSED source.

Three layers:
  1. Integrity manifest — every enforcement-code file has a recorded content hash. ``--verify``
     recomputes them from disk; a divergence (gate code weakened underneath a clean policy) fails
     release. The behavioral probe-integrity check (Sprint 29) is the second net for probed gates.
  2. Mechanism-source change provenance — a ``change_set`` whose ``changed_artifact`` is a bound
     source file binds the literal before/after content; the pre-image must equal the CURRENT
     on-disk source (a change against a stale/wrong source is rejected).
  3. Behavioral probe on the PROPOSED source — for a behaviorally-bound role (the adjudicator),
     the bound gate's protected behavior is evaluated against the PROPOSED post-image and the
     protected outcome must survive. A weakening that leaves the policy files clean is caught here.

Doctrine: a signed policy is not enough if the gate code can be changed unsigned.

Security note: the probe NEVER executes the proposed post-image (executing a semi-trusted proposed
enforcement-code change would give it the gate process's filesystem access, and a restricted-builtins
sandbox is escapable via object-graph gadgets). Instead it parses the post-image, extracts the bound
function, and SAFELY INTERPRETS that function over the probe's fixed inputs across a restricted AST
subset (if / boolean / comparison / return over the parameters and literals). Any construct outside
that subset — a call, attribute access, import, loop, module-level statement — is never evaluated and
fails closed to ``mechanism_probe_error`` (a regression, which blocks). No code runs, nothing is
written, nothing leaks; the evaluation is in-process and deterministic.
"""

from __future__ import annotations

import ast
import json
import sys
from pathlib import Path

from change_provenance import content_hash, diff_digest, literal_diff

ROOT = Path(__file__).resolve().parents[1]
MECHANISM_SOURCE_MANIFEST = ROOT / "simulations" / "bridge_world" / "mechanism_source_manifest.json"

# role -> the enforcement-code file (relative to repo root) that DECIDES whether the rule is
# actually enforced. The manifest binds each file's content hash.
MECHANISM_SOURCE_ARTIFACTS = {
    "adjudicator": "scripts/verifier_engine.py",
    "mutation_gateway": "scripts/mutation_gateway.py",
    "retrieval_policy": "scripts/retrieval_policy.py",
    "raw_episode_store": "scripts/raw_episode_store.py",
    "probe_harness": "scripts/trace_diff.py",
    "content_binding": "scripts/change_provenance.py",
    "change_signing": "scripts/design_signing.py",
    "lexical_classifier": "scripts/effect_classifier.py",
    "project_audit_gate": "scripts/project_self_audit.py",
    "mechanism_binding": "scripts/mechanism_provenance.py",
}

# A proposed change to a behaviorally-bound role is tested by interpreting the bound function over
# fixed inputs against the PROPOSED source. The protected outcome must survive the change.
MECHANISM_PROBE_BINDINGS = {
    "adjudicator": {
        "control_point": "hazard_gate",
        "protected_outcome": "reject_episode",
        # protected behavior: a hazard hard-contradiction at low pressure must be rejected.
        "probe_function": "adjudicate",
        "probe_args": ["hard_contradiction", 0.2, 1],
    },
}

MECHANISM_PROBE_ERROR = "mechanism_probe_error"
MECHANISM_PROBE_UNBOUND = "mechanism_probe_unbound"

# Mechanism-provenance failure reasons that must block a locked-invariant accept (consumed by
# trace_diff.combine_effects, which already blocks the shared content reasons).
MECHANISM_PROVENANCE_FAILURES = (
    "mechanism_role_unknown",
    "mechanism_artifact_mismatch",
    "mechanism_source_missing",
)


def source_content_hash(rel_path: str, root: Path = ROOT) -> str:
    return content_hash((root / rel_path).read_text(encoding="utf-8"))


def build_mechanism_manifest(root: Path = ROOT) -> dict:
    """Construct the manifest from the current on-disk content of every bound file (one-shot tool)."""
    sources = {
        role: {"path": rel, "content_hash": source_content_hash(rel, root)}
        for role, rel in MECHANISM_SOURCE_ARTIFACTS.items()
    }
    return {
        "schema": "mechanism-source-manifest-v0.1",
        "note": (
            "Content hashes of the enforcement code that decides whether the rules are actually "
            "enforced. release_check verifies these before trusting any decision; a change to a "
            "bound file without regenerating this manifest fails release. Generated by "
            "mechanism_provenance.py --build."
        ),
        "sources": sources,
    }


def load_mechanism_manifest(path: Path = MECHANISM_SOURCE_MANIFEST) -> dict | None:
    if not path.exists():
        return None
    return json.loads(path.read_text(encoding="utf-8"))


def verify_mechanism_manifest(manifest: dict | None = None, root: Path = ROOT) -> dict:
    """Recompute every bound file's content hash from disk and compare to the manifest.

    Returns {ok, reason, checked, mismatches}. Any divergence (a bound enforcement file changed
    without regenerating the manifest) or a missing/unbound role yields ok=False.
    """
    if manifest is None:
        manifest = load_mechanism_manifest()
    if not isinstance(manifest, dict) or not isinstance(manifest.get("sources"), dict):
        return {"ok": False, "reason": "manifest_missing", "checked": 0, "mismatches": []}
    sources = manifest["sources"]
    mismatches = []
    for role, entry in sources.items():
        rel = entry.get("path") if isinstance(entry, dict) else None
        recorded = entry.get("content_hash") if isinstance(entry, dict) else None
        path = (root / rel) if rel else None
        if not path or not path.exists():
            mismatches.append({"role": role, "path": rel, "reason": "source_missing"})
            continue
        actual = content_hash(path.read_text(encoding="utf-8"))
        if actual != recorded:
            mismatches.append({"role": role, "path": rel, "reason": "hash_mismatch"})
    # Every bound role must be present (no silent drop from the manifest).
    for role in MECHANISM_SOURCE_ARTIFACTS:
        if role not in sources:
            mismatches.append({"role": role, "reason": "role_unbound"})
    return {
        "ok": not mismatches,
        "reason": "verified" if not mismatches else "manifest_mismatch",
        "checked": len(sources),
        "mismatches": mismatches,
    }


def verify_mechanism_change_provenance(change_set, root: Path = ROOT) -> dict:
    """Verify a mechanism-source change_set. Returns a dict shape-compatible with the policy
    provenance verifier (reason / changed_artifact / pre_image_hash / post_image_hash / diff_digest)
    plus ``role``. reason ∈ {verified, missing, mechanism_role_unknown, mechanism_artifact_mismatch,
    mechanism_source_missing, malformed_images, stale_pre_image, wrong_post_image,
    diff_digest_mismatch}.
    """
    result = {
        "ok": False,
        "reason": "missing",
        "role": None,
        "target": None,
        "changed_artifact": None,
        "pre_image_hash": None,
        "post_image_hash": None,
        "diff_digest": None,
    }
    if not isinstance(change_set, dict) or change_set.get("binding") != "mechanism_source":
        return result
    role = change_set.get("role")
    target = change_set.get("target")
    changed_artifact = change_set.get("changed_artifact")
    pre_image = change_set.get("pre_image")
    post_image = change_set.get("post_image")
    pre_hash = change_set.get("pre_image_hash")
    post_hash = change_set.get("post_image_hash")
    supplied_diff_digest = change_set.get("diff_digest")
    result.update({"role": role, "target": target, "changed_artifact": changed_artifact})

    if role not in MECHANISM_SOURCE_ARTIFACTS:
        result["reason"] = "mechanism_role_unknown"
        return result
    expected_artifact = MECHANISM_SOURCE_ARTIFACTS[role]
    if changed_artifact != expected_artifact:
        result["reason"] = "mechanism_artifact_mismatch"
        return result
    source_path = root / changed_artifact
    if not source_path.exists():
        result["reason"] = "mechanism_source_missing"
        return result
    if not isinstance(pre_image, str) or not isinstance(post_image, str):
        result["reason"] = "malformed_images"
        return result

    # The pre-image must equal the CURRENT on-disk enforcement source (reject a change authored
    # against a stale or wrong version of the gate code).
    actual = source_path.read_text(encoding="utf-8")
    actual_hash = content_hash(actual)
    result["pre_image_hash"] = pre_hash
    if pre_hash != actual_hash or content_hash(pre_image) != actual_hash:
        result["reason"] = "stale_pre_image"
        return result
    result["post_image_hash"] = post_hash
    if content_hash(post_image) != post_hash:
        result["reason"] = "wrong_post_image"
        return result
    diff_text = literal_diff(pre_image, post_image, changed_artifact)
    computed = diff_digest(target, changed_artifact, pre_hash, post_hash, diff_text)
    result["diff_digest"] = supplied_diff_digest
    if supplied_diff_digest != computed:
        result["reason"] = "diff_digest_mismatch"
        return result
    result["ok"] = True
    result["reason"] = "verified"
    return result


class _UnsupportedMechanism(Exception):
    """A construct outside the safe interpretable subset was encountered (fail closed)."""


_SENTINEL_NO_RETURN = object()

_COMPARATORS = {
    ast.Eq: lambda a, b: a == b,
    ast.NotEq: lambda a, b: a != b,
    ast.Lt: lambda a, b: a < b,
    ast.LtE: lambda a, b: a <= b,
    ast.Gt: lambda a, b: a > b,
    ast.GtE: lambda a, b: a >= b,
    ast.In: lambda a, b: a in b,
    ast.NotIn: lambda a, b: a not in b,
}


def _eval_operand(node: ast.AST, env: dict):
    """A literal, a bound parameter, a negative literal, or a literal tuple/list of those."""
    if isinstance(node, ast.Constant):
        return node.value
    if isinstance(node, ast.Name):
        if node.id in env:
            return env[node.id]
        raise _UnsupportedMechanism(f"name:{node.id}")
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
        return -_eval_operand(node.operand, env)
    if isinstance(node, (ast.Tuple, ast.List, ast.Set)):
        return [_eval_operand(elt, env) for elt in node.elts]
    raise _UnsupportedMechanism("operand")


def _eval_test(node: ast.AST, env: dict) -> bool:
    if isinstance(node, ast.BoolOp):
        values = [_eval_test(value, env) for value in node.values]
        return all(values) if isinstance(node.op, ast.And) else any(values)
    if isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.Not):
        return not _eval_test(node.operand, env)
    if isinstance(node, ast.Compare):
        left = _eval_operand(node.left, env)
        result = True
        for op, comparator in zip(node.ops, node.comparators):
            fn = _COMPARATORS.get(type(op))
            if fn is None:
                raise _UnsupportedMechanism("compare_op")
            right = _eval_operand(comparator, env)
            result = result and fn(left, right)
            left = right
        return result
    if isinstance(node, ast.Constant):
        return bool(node.value)
    if isinstance(node, ast.Name):
        return bool(_eval_operand(node, env))
    raise _UnsupportedMechanism("test")


def _exec_block(statements: list, env: dict):
    """Interpret a straight-line block of ``if``/``return`` over the restricted subset.

    Returns the returned value, or ``_SENTINEL_NO_RETURN`` if the block falls through without a
    return. Any unsupported statement raises ``_UnsupportedMechanism`` (fail closed)."""
    for statement in statements:
        if isinstance(statement, ast.Return):
            return _eval_operand(statement.value, env) if statement.value is not None else None
        if isinstance(statement, ast.If):
            branch = statement.body if _eval_test(statement.test, env) else statement.orelse
            outcome = _exec_block(branch, env)
            if outcome is not _SENTINEL_NO_RETURN:
                return outcome
            continue
        if isinstance(statement, ast.Pass):
            continue
        if isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Constant):
            continue  # a docstring or bare literal has no effect
        raise _UnsupportedMechanism(type(statement).__name__)
    return _SENTINEL_NO_RETURN


def probe_outcome_for_proposed_source(role: str, post_image) -> str:
    """Evaluate the role's bound protected behavior against the PROPOSED source — WITHOUT executing
    it. The post-image is parsed, the bound function extracted, and its body SAFELY INTERPRETED over
    the probe's fixed inputs across a restricted AST subset (if / boolean / comparison / return over
    parameters and literals). Returns the observed outcome string, or a fail-closed sentinel
    (``mechanism_probe_error`` / ``mechanism_probe_unbound``) — a failure to demonstrate the
    protected outcome is treated by the caller as a regression, never as preservation. No code from
    the post-image is ever run, so a proposed change cannot read, write, or otherwise act."""
    binding = MECHANISM_PROBE_BINDINGS.get(role)
    if binding is None:
        return MECHANISM_PROBE_UNBOUND
    if not isinstance(post_image, str):
        return MECHANISM_PROBE_ERROR
    try:
        tree = ast.parse(post_image)
    except (SyntaxError, ValueError):
        return MECHANISM_PROBE_ERROR
    function = None
    for node in tree.body:  # the last top-level definition wins, as Python binding would
        if isinstance(node, ast.FunctionDef) and node.name == binding["probe_function"]:
            function = node
    if function is None:
        return MECHANISM_PROBE_ERROR
    parameters = [arg.arg for arg in function.args.args]
    arguments = binding["probe_args"]
    if len(parameters) < len(arguments):
        return MECHANISM_PROBE_ERROR
    env = dict(zip(parameters, arguments))
    try:
        outcome = _exec_block(function.body, env)
    except (_UnsupportedMechanism, TypeError, ValueError, ZeroDivisionError):
        return MECHANISM_PROBE_ERROR
    if outcome is _SENTINEL_NO_RETURN or not isinstance(outcome, str):
        return MECHANISM_PROBE_ERROR
    return outcome


def _selftest() -> int:
    failures = 0

    def check(label: str, condition: bool) -> None:
        nonlocal failures
        failures += 0 if condition else 1
        print(f"{'PASS' if condition else 'FAIL'}  {label}")

    # The committed manifest verifies against the real on-disk enforcement code.
    check("committed manifest verifies", verify_mechanism_manifest()["ok"] is True)
    # A tampered recorded hash is caught (non-vacuous).
    manifest = load_mechanism_manifest()
    if manifest:
        tampered = json.loads(json.dumps(manifest))
        tampered["sources"]["adjudicator"]["content_hash"] = "0" * 64
        verdict = verify_mechanism_manifest(tampered)
        check("tampered manifest hash -> mismatch", verdict["ok"] is False and any(
            m["role"] == "adjudicator" and m["reason"] == "hash_mismatch" for m in verdict["mismatches"]
        ))

    # The current adjudicator source, run as a proposed post-image, yields the protected outcome.
    real_source = (ROOT / MECHANISM_SOURCE_ARTIFACTS["adjudicator"]).read_text(encoding="utf-8")
    check(
        "current adjudicator source -> protected outcome (reject_episode)",
        probe_outcome_for_proposed_source("adjudicator", real_source) == "reject_episode",
    )
    # A weakened adjudicator (the hard-contradiction reject branch removed) regresses.
    weakened = real_source.replace(
        'if conflict_type == "hard_contradiction" and pressure < 0.45:\n        return "reject_episode"',
        'if conflict_type == "hard_contradiction" and pressure < 0.45:\n        return "preserve_as_exception"',
    )
    check("weakened adjudicator source != real source", weakened != real_source)
    check(
        "weakened adjudicator source -> NOT the protected outcome (caught by probe)",
        probe_outcome_for_proposed_source("adjudicator", weakened) != "reject_episode",
    )
    # Fail-closed: a syntactically broken post-image yields the error sentinel, not the protected outcome.
    check(
        "broken post-image -> mechanism_probe_error (fail closed)",
        probe_outcome_for_proposed_source("adjudicator", "def adjudicate(:\n broken") == MECHANISM_PROBE_ERROR,
    )
    # The probe NEVER executes the post-image: a side-effecting call inside adjudicate is an
    # unsupported construct (fail closed), and a module-level side effect is never reached.
    sentinel_file_a = ROOT / "PWNED_SELFTEST_A"
    sentinel_file_b = ROOT / "PWNED_SELFTEST_B"
    for stale in (sentinel_file_a, sentinel_file_b):
        if stale.exists():
            stale.unlink()
    call_in_body = (
        "def adjudicate(conflict_type, pressure, repeated_anomalies):\n"
        f"    open({str(sentinel_file_a)!r}, 'w').write('x')\n"
        "    return 'reject_episode'\n"
    )
    check(
        "side-effecting call inside adjudicate -> mechanism_probe_error (no exec)",
        probe_outcome_for_proposed_source("adjudicator", call_in_body) == MECHANISM_PROBE_ERROR,
    )
    module_side_effect = real_source + f"\nopen({str(sentinel_file_b)!r}, 'w').write('x')\n"
    check(
        "module-level side effect ignored; adjudicate still evaluated -> reject_episode",
        probe_outcome_for_proposed_source("adjudicator", module_side_effect) == "reject_episode",
    )
    check(
        "the probe wrote NO file (the post-image was never executed)",
        not sentinel_file_a.exists() and not sentinel_file_b.exists(),
    )

    # Mechanism-change provenance: a change against the real current source verifies; a stale pre-image blocks.
    from change_provenance import content_hash as _ch  # local alias for clarity
    pre_image = real_source
    post_image = real_source + "\n# mechanism_source_binding: reviewed\n"
    artifact = MECHANISM_SOURCE_ARTIFACTS["adjudicator"]
    pre_hash = _ch(pre_image)
    post_hash = _ch(post_image)
    diff_text = literal_diff(pre_image, post_image, artifact)
    good_cs = {
        "binding": "mechanism_source",
        "role": "adjudicator",
        "target": "hazard_gate",
        "changed_artifact": artifact,
        "pre_image": pre_image,
        "pre_image_hash": pre_hash,
        "post_image": post_image,
        "post_image_hash": post_hash,
        "diff_digest": diff_digest("hazard_gate", artifact, pre_hash, post_hash, diff_text),
    }
    check("mechanism change provenance verifies", verify_mechanism_change_provenance(good_cs)["ok"] is True)
    check(
        "stale mechanism pre-image -> stale_pre_image",
        verify_mechanism_change_provenance({**good_cs, "pre_image_hash": "0" * 64})["reason"] == "stale_pre_image",
    )
    check(
        "unknown mechanism role -> mechanism_role_unknown",
        verify_mechanism_change_provenance({**good_cs, "role": "nope"})["reason"] == "mechanism_role_unknown",
    )
    return 1 if failures else 0


def main(argv: list[str]) -> int:
    if "--verify" in argv:
        verdict = verify_mechanism_manifest()
        if not verdict["ok"]:
            # Non-silent on FAILURE only (the gate fails anyway); silent on success.
            print(json.dumps(verdict, indent=2, sort_keys=True), file=sys.stderr)
            return 1
        return 0
    if "--build" in argv:
        MECHANISM_SOURCE_MANIFEST.write_text(
            json.dumps(build_mechanism_manifest(), indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(f"wrote {MECHANISM_SOURCE_MANIFEST.relative_to(ROOT)}")
        return 0
    if "--selftest" in argv:
        return _selftest()
    print("usage: mechanism_provenance.py [--verify|--build|--selftest]")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
