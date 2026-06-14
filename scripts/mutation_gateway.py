"""Central mutation gateway and append-only audit log."""

from __future__ import annotations

import copy
from datetime import datetime, timezone

from bootstrap_ingest import promote_candidate


MUTATION_TYPES = {
    "semantic_status_update",
    "bootstrap_promotion",
    "bootstrap_rejection",
    "memory_confidence_update",
    "procedure_status_update",
    "rule_status_update",
    "authority_scope_update",
    "contradiction_status_update",
    "planner_policy_update",
    "attention_policy_update",
}

MUTATION_AUTHORITY_RULE = "V_RULE_MUTATION_REQUIRES_AUTHORITY"
HUMAN_PROMOTION_RULE = "V_RULE_HUMAN_PROMOTION_REQUIRED"


def apply_memory_mutation(
    request: dict,
    target_object: dict,
    source_packet: dict | None,
    verifier_decision: dict | None,
    mutation_log: list[dict],
    memory=None,
) -> dict:
    _require_request_fields(request)
    before_status = _object_status(target_object)
    reason = _rejection_reason(request, target_object, source_packet, verifier_decision)
    if reason:
        record = _log_record(request, "reject", reason, before_status, before_status)
        mutation_log.append(record)
        return {"applied": False, "target": copy.deepcopy(target_object), "log": record}

    updated = copy.deepcopy(target_object)
    if request["mutation_type"] == "semantic_status_update":
        if memory is None:
            raise ValueError("semantic_status_update requires governed memory")
        updated = memory.semantic_graph.update_status(
            request["target_object_id"],
            request["new_status"],
            "mutation_gateway",
            patch=request.get("patch"),
        )
    elif request["mutation_type"] in {"authority_scope_update", "contradiction_status_update"}:
        if memory is None:
            raise ValueError(f"{request['mutation_type']} requires governed memory")
        updated = memory.semantic_graph.update_status(
            request["target_object_id"],
            request["new_status"],
            "mutation_gateway",
            patch=request.get("patch"),
        )
    elif request["mutation_type"] == "procedure_status_update":
        if memory is None:
            raise ValueError("procedure_status_update requires governed memory")
        updated = memory.procedural_store.update_status(
            request["target_object_id"],
            request["new_status"],
        )
    elif request["mutation_type"] == "bootstrap_promotion":
        updated = promote_candidate(
            target_object,
            human_approved=True,
            promoted_by=source_packet["header"]["source_engine"],
        )
    elif request["mutation_type"] == "bootstrap_rejection":
        updated["status"] = "rejected"
        updated["authority_class"] = "bootstrap_rejected"
        updated["inspection_view"] = "bootstrap_candidates"
    elif request["mutation_type"] in {"planner_policy_update", "attention_policy_update"}:
        updated.update(request.get("patch", {}))
    else:
        updated.update(request.get("patch", {}))

    after_status = _object_status(updated)
    record = _log_record(request, "allow", "authorized mutation applied", before_status, after_status)
    mutation_log.append(record)
    return {"applied": True, "target": updated, "log": record}


def verifier_allows_mutation(
    decision_id: str,
    mutation_type: str,
    requested_use: str,
    target_object_id: str,
    source_packet_id: str,
    rule_id: str = MUTATION_AUTHORITY_RULE,
) -> dict:
    return {
        "verifier_decision_id": decision_id,
        "rule_id": rule_id,
        "decision": "allow",
        "mutation_type": mutation_type,
        "requested_use": requested_use,
        "target_object_id": target_object_id,
        "source_packet_id": source_packet_id,
    }


def verifier_blocks_mutation(
    decision_id: str,
    mutation_type: str,
    requested_use: str,
    target_object_id: str,
    source_packet_id: str | None,
    reason: str,
) -> dict:
    return {
        "verifier_decision_id": decision_id,
        "rule_id": MUTATION_AUTHORITY_RULE,
        "decision": "block",
        "mutation_type": mutation_type,
        "requested_use": requested_use,
        "target_object_id": target_object_id,
        "source_packet_id": source_packet_id,
        "reason": reason,
    }


def _require_request_fields(request: dict) -> None:
    required = {
        "mutation_id",
        "trace_id",
        "target_object_id",
        "requested_use",
        "mutation_type",
        "authority_snapshot",
    }
    missing = [field for field in required if not request.get(field)]
    if missing:
        raise ValueError(f"mutation request missing required fields: {', '.join(sorted(missing))}")


def _rejection_reason(
    request: dict,
    target_object: dict,
    source_packet: dict | None,
    verifier_decision: dict | None,
) -> str | None:
    if request["mutation_type"] not in MUTATION_TYPES:
        return f"unknown mutation_type: {request['mutation_type']}"
    if not request.get("source_packet_id"):
        return "missing source_packet_id"
    if not request.get("verifier_decision_id"):
        return "missing verifier_decision_id"
    if source_packet is None:
        return "missing source_packet"
    if verifier_decision is None:
        return "missing verifier_decision_id"
    if source_packet["header"]["packet_id"] != request["source_packet_id"]:
        return "source_packet_id does not match source packet"
    if verifier_decision["verifier_decision_id"] != request["verifier_decision_id"]:
        return "verifier_decision_id does not match verifier decision"
    if request["target_object_id"] != _object_id(target_object):
        return "target_object_id does not match target object"
    if request["requested_use"] in request["authority_snapshot"].get("forbidden_use", []):
        return f"target authority forbids requested_use: {request['requested_use']}"
    if request["requested_use"] in source_packet["permissions"].get("forbidden_use", []):
        return f"source packet authority forbids requested_use: {request['requested_use']}"
    if request["requested_use"] not in source_packet["permissions"].get("allowed_use", []):
        return f"source packet authority does not allow requested_use: {request['requested_use']}"
    if verifier_decision.get("decision") != "allow":
        return verifier_decision.get("reason", "verifier decision blocked mutation")
    for field in ("mutation_type", "requested_use", "target_object_id", "source_packet_id"):
        if verifier_decision.get(field) != request[field]:
            return f"verifier decision does not authorize {field}: {request[field]}"
    if request["mutation_type"] == "planner_policy_update":
        if not target_object.get("plan_id"):
            return "planner_policy_update target must be planner policy"
        authority_fields = {"epistemic_license", "authority_class", "allowed_use", "forbidden_use"}
        requested_authority_changes = sorted(authority_fields.intersection(request.get("patch", {})))
        if requested_authority_changes:
            return f"planner_policy_update cannot change authority fields: {requested_authority_changes}"
    if request["mutation_type"] == "attention_policy_update":
        if not target_object.get("attention_policy_id"):
            return "attention_policy_update target must be attention policy"
        allowed_fields = {
            "status",
            "confidence",
            "mode_thresholds",
            "mode_policy",
            "coalescing_policy",
            "backpressure_policy",
            "scope_conditions",
            "attention_review_note",
        }
        forbidden_fields = sorted(set(request.get("patch", {})).difference(allowed_fields))
        if forbidden_fields:
            return f"attention_policy_update cannot change non-attention fields: {forbidden_fields}"
    return None


def _log_record(
    request: dict,
    decision: str,
    reason: str,
    before_status: str,
    after_status: str,
) -> dict:
    return {
        "mutation_id": request["mutation_id"],
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "trace_id": request["trace_id"],
        "target_object_id": request["target_object_id"],
        "mutation_type": request["mutation_type"],
        "requested_use": request["requested_use"],
        "source_packet_id": request.get("source_packet_id"),
        "verifier_decision_id": request.get("verifier_decision_id"),
        "blocking_rule": MUTATION_AUTHORITY_RULE,
        "decision": decision,
        "reason": reason,
        "before_status": before_status,
        "after_status": after_status,
    }


def _object_id(target_object: dict) -> str:
    return (
        target_object.get("memory_id")
        or target_object.get("procedure_id")
        or target_object.get("plan_id")
        or target_object.get("attention_policy_id")
        or target_object.get("rule_id")
        or target_object.get("id")
    )


def _object_status(target_object: dict) -> str:
    return target_object.get("authority_class") or target_object.get("status", "unknown")
