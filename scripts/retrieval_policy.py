"""Status-aware retrieval policy and emergency-use rules."""

from __future__ import annotations


DEGRADED_STATUSES = {
    "active_with_superseded_dependency",
    "confidence_reduced",
    "pending_rederivation",
    "contradicted",
    "exception_scoped",
    "quarantined",
    "retest_required",
    "superseded",
    "deprecated_but_preserved",
    "stale",
}


def license_for_status(status: str, confidence: float, contradiction_count: int) -> str:
    if status in {"quarantined", "superseded", "deprecated_but_preserved"}:
        return "do_not_use_for_action"
    if status in {"contradicted", "retest_required"}:
        return "hazard_only"
    if contradiction_count:
        return "hypothesis_only"
    if status in {"confidence_reduced", "pending_rederivation", "active_with_superseded_dependency", "stale"}:
        return "weak_premise" if confidence >= 0.5 else "hypothesis_only"
    return "full_premise" if confidence >= 0.85 else "weak_premise"


def emergency_use_protocol(epistemic_license: str, urgent: bool) -> str:
    if not urgent:
        return {
            "full_premise": "normal_use",
            "weak_premise": "normal_use_with_fallback_available",
            "hypothesis_only": "branch_alternatives",
            "hazard_only": "warning_only",
            "do_not_use_for_action": "cannot_support_action",
        }[epistemic_license]
    return {
        "full_premise": "normal_use",
        "weak_premise": "use_with_fallback",
        "hypothesis_only": "branch_alternatives",
        "hazard_only": "warning_only",
        "do_not_use_for_action": "cannot_support_action",
    }[epistemic_license]


def permissions_for_license(epistemic_license: str) -> dict:
    if epistemic_license == "full_premise":
        return {
            "allowed_use": ["retrieval", "planning", "planning_with_fallback", "human_explanation", "contradiction_detection"],
            "forbidden_use": ["direct_action", "rule_revision", "safety_certification"],
        }
    if epistemic_license == "weak_premise":
        return {
            "allowed_use": ["retrieval", "planning_with_fallback", "human_explanation", "contradiction_detection"],
            "forbidden_use": ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        }
    if epistemic_license == "hypothesis_only":
        return {
            "allowed_use": ["retrieval", "planning_with_fallback", "human_explanation", "contradiction_detection"],
            "forbidden_use": ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        }
    if epistemic_license == "hazard_only":
        return {
            "allowed_use": ["retrieval", "human_explanation", "contradiction_detection"],
            "forbidden_use": ["planning", "planning_with_fallback", "direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        }
    return {
        "allowed_use": ["retrieval", "human_explanation"],
        "forbidden_use": ["planning", "planning_with_fallback", "direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
    }


def revalidation_requirement(status: str, epistemic_license: str, urgent: bool) -> str:
    if status in DEGRADED_STATUSES or epistemic_license != "full_premise":
        return "post_action_revalidation" if urgent else "revalidate_before_consolidation"
    return "none"


def wrap_retrieval_item(
    content: dict,
    memory_id: str,
    confidence: float,
    status: str,
    source_episodes: list[str],
    contradictions: list[dict],
    urgent: bool,
) -> dict:
    epistemic_license = license_for_status(status, confidence, len(contradictions))
    return {
        "content": content,
        "confidence": confidence,
        "status": status,
        "epistemic_license": epistemic_license,
        "source_episodes": source_episodes,
        "contradictions": contradictions,
        "allowed_use": permissions_for_license(epistemic_license)["allowed_use"],
        "forbidden_use": permissions_for_license(epistemic_license)["forbidden_use"],
        "revalidation_requirement": revalidation_requirement(status, epistemic_license, urgent),
        "emergency_use": emergency_use_protocol(epistemic_license, urgent),
    }


def retrieval_has_degraded_action_support(retrieval: dict) -> bool:
    for group in ("episodes", "semantic_nodes", "procedures"):
        for item in retrieval[group]:
            if item["revalidation_requirement"] == "post_action_revalidation":
                if "planning_with_fallback" in item["allowed_use"] or item["emergency_use"] in {"use_with_fallback", "branch_alternatives"}:
                    return True
    return False

