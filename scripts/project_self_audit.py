#!/usr/bin/env python3
"""Project self-audit: the Caitlin leap (Sprint 24).

The development process is a first-class citizen inside the same machinery that
governs the bridge world. This module replays the project's own design decisions
and invariants, evaluates new design proposals against locked invariants using the
*runtime* adjudicator, and consolidates project-health through the *real* mutation
gateway. No separate meta-immune-system: same verifier, same licenses, same audit.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

from verifier_engine import adjudicate
from mutation_gateway import (
    apply_memory_mutation,
    verifier_allows_mutation,
    verifier_blocks_mutation,
)
from bootstrap_ingest import ingest_design_history
from effect_classifier import derive_effect, effect_family
from trace_diff import combine_effects, derive_effect_from_trace
from design_signing import load_authorized_signers, verify_change_signature
from mechanism_provenance import verify_mechanism_manifest

ROOT = Path(__file__).resolve().parents[1]
WORLD = ROOT / "simulations" / "bridge_world"
DESIGN_MEMORY = WORLD / "design_memory.json"
DESIGN_VERIFIER_RULES = WORLD / "design_verifier_rules.json"

REQUIRED_DECISION_FIELDS = ("trace_id", "verifier_assessment", "epistemic_license")

# In-repo design corpus. Concept docs in the parent dir are ingested best-effort so a
# stand-alone clone of cognitive-os still audits cleanly.
IN_REPO_CORPUS = ("a.md", "DESIGN_REVIEW_NOTES.md", "FAILURE_LEDGER.md", "SPRINT_24_PLAN.md")
PARENT_CORPUS = ("project_birth.md", "COGNITIVE_OS_SELF_CORRECTING_LEAP.md")


def load_design_memory(path: Path = DESIGN_MEMORY) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def load_design_verifier_rules(path: Path = DESIGN_VERIFIER_RULES) -> list[dict]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def _rule_matches(rule_when: dict, proposal: dict, invariant: dict | None) -> bool:
    if "effect" in rule_when and proposal.get("effect") != rule_when["effect"]:
        return False
    if "targets_status" in rule_when and (invariant or {}).get("status") != rule_when["targets_status"]:
        return False
    return True


def evaluate_design_proposal(
    proposal: dict,
    invariant: dict | None,
    rules: list[dict] | None = None,
    authorized_signers: dict | None = None,
    now_tick: int | None = None,
) -> dict:
    """Evaluate a design proposal against design verifier rules.

    Sprint 25 — the effect is DERIVED from a lexical semantic diff of the proposal claim
    vs. the invariant claim (``effect_classifier.derive_effect``). Sprint 26 demotes that
    lexical diff to an early-warning layer and makes a runtime BEHAVIORAL trace the
    authority: a probe-guarded invariant is run pre/post the proposal's behavioral delta
    (``trace_diff``), and a regression of the protected outcome overrides any
    lexical/declared "consistent" verdict. A self-declared ``effect`` remains an untrusted
    hint, used only to flag mislabeling; it is never read as authority. The runtime
    ``adjudicate`` is re-used to independently confirm the hard-contradiction outcome, so
    the meta level cannot grade itself more leniently than the bridge world does.
    """
    rules = rules if rules is not None else load_design_verifier_rules()

    invariant_claim = (invariant or {}).get("claim", "")
    derivation = derive_effect(proposal.get("claim", ""), invariant_claim)
    lexical_effect = derivation["effect"]  # Sprint 25 early-warning layer
    trace = derive_effect_from_trace(invariant, proposal)  # Sprint 26 behavioral evidence
    invariant_locked = (invariant or {}).get("status") == "regression_lock"
    combined = combine_effects(lexical_effect, trace, invariant_locked=invariant_locked)
    derived_effect = combined["effect"]  # authority: trace overrides lexical, lexical overrides declared
    effect_authority = combined["authority"]
    declared_effect = proposal.get("effect")  # untrusted hint only

    # Sprint 30 — signature gate. Authorship is always evaluated for transparency, but it can
    # only constrain a would-be ACCEPT: a content-bound change to a LOCKED invariant must carry
    # a valid signature from an authorized signer. Authorization NEVER overrides a trace/lexical
    # block — a validly-signed weakening still blocks by trace.
    # Sprint 31 — authority is governed and evaluated at the decision tick: a genuine signature
    # from a now-revoked / expired / out-of-scope signer is not authorization. The decision tick
    # is a LOGICAL tick (``evaluation_tick`` on the proposal), never wall-clock, so the gate is
    # reproducible. The change's scope is its targeted control point (the change_set target).
    signers = authorized_signers if authorized_signers is not None else load_authorized_signers()
    change_set = proposal.get("change_set")
    decision_tick = now_tick if now_tick is not None else proposal.get("evaluation_tick", 0)
    change_scope = change_set.get("target") if isinstance(change_set, dict) else None
    signature = verify_change_signature(change_set, signers, now_tick=decision_tick, change_scope=change_scope)
    signature_status = signature["reason"]
    if invariant_locked and effect_family(derived_effect) == "consistent" and not signature["ok"]:
        derived_effect = "needs_review"
        effect_authority = "change_signature_unverified"

    effect_mislabel = bool(declared_effect) and effect_family(declared_effect) != effect_family(derived_effect)
    # Authority is the derived effect, never the declared one.
    evaluated = dict(proposal)
    evaluated["effect"] = derived_effect

    effect_fields = {
        "declared_effect": declared_effect,
        "lexical_effect": lexical_effect,
        "trace_effect": trace["trace_effect"],
        "trace_tested": trace["tested"],
        "trace_regressed": trace["regressed"],
        "trace_status": trace["status"],
        "trace_pre": trace["pre"],
        "trace_post": trace["post"],
        "trace_control_point": trace["control_point"],
        "trace_protected_outcome": trace["protected_outcome"],
        "trace_provenance": trace.get("provenance"),
        "mechanism_source": trace.get("mechanism_source"),
        "mechanism_role": trace.get("mechanism_role"),
        "changed_artifact": trace.get("changed_artifact"),
        "pre_image_hash": trace.get("pre_image_hash"),
        "post_image_hash": trace.get("post_image_hash"),
        "diff_digest": trace.get("diff_digest"),
        "delta_matches_change_set": trace.get("delta_matches_change_set"),
        "signer": signature["signer"],
        "signature_status": signature_status,
        "signed_payload_digest": signature["payload_digest"],
        "signer_status": signature["signer_status"],
        "signer_scope": signature["signer_scope"],
        "signer_expires_at": signature["signer_expires_at"],
        "signer_revoked_at": signature["signer_revoked_at"],
        "signer_rotated_to": signature["rotated_to"],
        "evaluation_tick": decision_tick,
        "effect_authority": effect_authority,
        "derived_effect": derived_effect,
        "effect": derived_effect,
        "effect_family": effect_family(derived_effect),
        "effect_mislabel": effect_mislabel,
        "effect_basis": derivation["basis"],
        "effect_evidence": derivation["evidence"],
    }

    for rule in rules:
        if not _rule_matches(rule["when"], evaluated, invariant):
            continue
        conflict_type = rule["conflict_type"]
        runtime_pressure = 0.2 if conflict_type == "hard_contradiction" else 0.6
        runtime_adjudication = adjudicate(conflict_type, runtime_pressure, repeated_anomalies=1)
        detected = conflict_type != "no_conflict"
        return {
            "proposal_id": proposal.get("proposal_id"),
            "targets_invariant": proposal.get("targets_invariant"),
            "verifier_rule_id": rule["id"],
            "conflict_type": conflict_type,
            "adjudication": rule["adjudication"],
            "runtime_adjudication": runtime_adjudication,
            "contradiction_detected": detected,
            "contradiction_license": rule["contradiction_license"] if detected else None,
            "blocks_release": rule["blocks_release"],
            "revalidation_required": detected,
            **effect_fields,
        }
    return {
        "proposal_id": proposal.get("proposal_id"),
        "targets_invariant": proposal.get("targets_invariant"),
        "verifier_rule_id": "VR_design_default_no_conflict",
        "conflict_type": "no_conflict",
        "adjudication": "preserve_as_exception",
        "runtime_adjudication": adjudicate("no_conflict", 0.6, repeated_anomalies=1),
        "contradiction_detected": False,
        "contradiction_license": None,
        "blocks_release": False,
        "revalidation_required": False,
        **effect_fields,
    }


def audit_design_decisions(decisions: list[dict]) -> dict:
    """Every design decision must carry a full trace + verifier assessment + license,
    and have no unresolved contradiction. A decision missing any is a violation."""
    violations = []
    unresolved = []
    for decision in decisions:
        missing = [field for field in REQUIRED_DECISION_FIELDS if not decision.get(field)]
        if missing:
            violations.append({
                "decision_id": decision.get("decision_id", "<unknown>"),
                "reason": "missing_audit_fields",
                "fields": missing,
            })
        open_contradictions = [
            item for item in decision.get("contradictions", [])
            if not (isinstance(item, dict) and item.get("resolved"))
        ]
        if open_contradictions:
            unresolved.append({
                "decision_id": decision.get("decision_id", "<unknown>"),
                "contradictions": open_contradictions,
            })
            violations.append({
                "decision_id": decision.get("decision_id", "<unknown>"),
                "reason": "unresolved_contradiction",
                "fields": open_contradictions,
            })
    return {
        "audited": len(decisions),
        "violations": violations,
        "unresolved_contradictions": unresolved,
    }


def _ingest_corpus() -> dict:
    files = []
    candidate_count = 0
    non_authoritative = True
    sample = None
    for name in IN_REPO_CORPUS:
        path = ROOT / name
        if not path.exists():
            continue
        candidates = ingest_design_history(path.read_text(encoding="utf-8"), name)
        files.append(name)
        candidate_count += len(candidates)
        for candidate in candidates:
            if sample is None:
                sample = candidate["claim"]
            if (
                candidate["epistemic_license"] != "hypothesis_only"
                or "runtime_action" not in candidate["forbidden_use"]
                or "memory_consolidation" not in candidate["forbidden_use"]
            ):
                non_authoritative = False
    for name in PARENT_CORPUS:
        path = ROOT.parent / name
        if not path.exists():
            continue
        candidates = ingest_design_history(path.read_text(encoding="utf-8"), name)
        files.append(name)
        candidate_count += len(candidates)
        for candidate in candidates:
            if (
                candidate["epistemic_license"] != "hypothesis_only"
                or "runtime_action" not in candidate["forbidden_use"]
            ):
                non_authoritative = False
    return {
        "files": files,
        "candidate_count": candidate_count,
        "all_non_authoritative": non_authoritative,
        "sample_claim": sample,
    }


def audit_project() -> dict:
    memory = load_design_memory()
    invariants = memory.get("invariants", [])
    decisions = memory.get("design_decisions", [])
    decision_audit = audit_design_decisions(decisions)
    # Sprint 32: the decision is only trustworthy if the enforcement code that produces it is the
    # bound, manifest-verified mechanism source. A gate-code change underneath a clean policy fails
    # here (mechanism_source_binding violated) and blocks the strict audit.
    manifest = verify_mechanism_manifest()
    mechanism_binding = "verified" if manifest["ok"] else "violated"
    strict_pass = (not decision_audit["violations"]) and manifest["ok"]
    report = {
        "surface": "project_self_audit",
        "surface_role": "project_cognition",
        "doctrine": memory.get("doctrine", ""),
        "invariant_count": len(invariants),
        "locked_invariants": [
            inv["memory_id"] for inv in invariants if inv.get("status") == "regression_lock"
        ],
        "design_decisions_audited": decision_audit["audited"],
        "violations": decision_audit["violations"],
        "unresolved_contradictions": decision_audit["unresolved_contradictions"],
        "ingested_corpus": _ingest_corpus(),
        "mechanism_source_binding": mechanism_binding,
        "mechanism_source_mismatches": manifest["mismatches"],
        "strict_audit": "pass" if strict_pass else "fail",
        "project_cognitive_health": "green" if strict_pass else "blocked",
    }
    return report


def consolidate_project_health(report: dict) -> dict:
    """Update the project-health node through the real mutation gateway.

    A green strict audit consolidates under memory_consolidation license; a failing
    audit emits an AuditViolation and the gateway refuses the consolidation, exactly
    as a safety-critical missing dependency blocks a runtime memory mutation.
    """
    strict_pass = report["strict_audit"] == "pass"
    health_node = {
        "memory_id": "D_project_cognitive_health",
        "status": "unknown",
        "node_type": "project_health",
    }
    source_packet = {
        "header": {"packet_id": "P_release_gate", "source_engine": "release_gate"},
        "permissions": {
            "allowed_use": ["memory_consolidation", "human_explanation"],
            "forbidden_use": ["direct_action", "rule_revision", "safety_certification"],
        },
    }
    if strict_pass:
        decision = verifier_allows_mutation(
            "V_DEC_project_health",
            "memory_confidence_update",
            "memory_consolidation",
            "D_project_cognitive_health",
            "P_release_gate",
        )
    else:
        decision = verifier_blocks_mutation(
            "V_DEC_project_health",
            "memory_confidence_update",
            "memory_consolidation",
            "D_project_cognitive_health",
            "P_release_gate",
            "strict project audit failed; project health cannot be consolidated",
        )
    request = {
        "mutation_id": "MUT_project_health",
        "trace_id": "T_project_health",
        "source_packet_id": "P_release_gate",
        "verifier_decision_id": "V_DEC_project_health",
        "target_object_id": "D_project_cognitive_health",
        "requested_use": "memory_consolidation",
        "mutation_type": "memory_confidence_update",
        "patch": {
            "status": "green" if strict_pass else "blocked",
            "strict_audit": report["strict_audit"],
            "violation_count": len(report["violations"]),
        },
        "authority_snapshot": {"forbidden_use": []},
    }
    mutation_log = []
    result = apply_memory_mutation(request, health_node, source_packet, decision, mutation_log)
    return {
        "project_cognitive_health_consolidated": result["applied"],
        "project_cognitive_health": result["target"].get("status"),
        "audit_violation_packet": None if strict_pass else {
            "type": "AuditViolationPacket",
            "reason": "strict_project_audit_failed",
            "blocked_consolidation": "D_project_cognitive_health",
            "correction_job": "design_revalidation",
        },
        "mutation_log_entry": result["log"],
    }


def run_project_audit(strict: bool = False, emit_health: bool = False) -> tuple[dict, int]:
    report = audit_project()
    if emit_health:
        report["health_consolidation"] = consolidate_project_health(report)
    exit_code = 0
    if strict and report["strict_audit"] != "pass":
        exit_code = 1
    return report, exit_code


def main() -> int:
    args = sys.argv[1:]
    strict = "--strict" in args or "--strict-project" in args
    emit_health = "--emit-health" in args or "--project" in args
    report, exit_code = run_project_audit(strict=strict, emit_health=emit_health)
    print(json.dumps(report, indent=2, sort_keys=True))
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
