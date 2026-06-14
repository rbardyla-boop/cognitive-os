"""Rule migration and cascade impact helpers."""

from __future__ import annotations

import copy


def next_rule_version(rule: dict, new_claim: str, changed_by: str) -> dict:
    base_id = rule.get("base_id") or rule["id"].split(":")[0]
    version = int(rule.get("version", 1)) + 1
    migrated = copy.deepcopy(rule)
    migrated["id"] = f"{base_id}:v{version}"
    migrated["base_id"] = base_id
    migrated["version"] = version
    migrated["claim"] = new_claim
    migrated["supersedes"] = rule["id"]
    migrated["changed_by"] = changed_by
    return migrated


def trace_dependencies(rule_id: str, semantic_nodes: list[dict], procedures: list[dict], plans: list[dict]) -> list[dict]:
    impacts = []
    for node in semantic_nodes:
        if rule_id not in node.get("depends_on_rules", []):
            continue
        node_id = node["memory_id"]
        procedure_ids = [
            procedure["procedure_id"]
            for procedure in procedures
            if node_id in procedure.get("depends_on_memories", []) or rule_id in procedure.get("depends_on_rules", [])
        ]
        plan_ids = [
            plan["plan_id"]
            for plan in plans
            if node_id in plan.get("depends_on_memories", []) or rule_id in plan.get("depends_on_rules", [])
        ]
        impacts.append({
            "memory_id": node_id,
            "depends_on_rules": node.get("depends_on_rules", []),
            "source_episodes": node.get("source_episodes", []),
            "used_by_procedures": sorted(set(node.get("used_by_procedures", []) + procedure_ids)),
            "used_by_plans": sorted(set(node.get("used_by_plans", []) + plan_ids)),
            "memory_confidence": node["confidence"],
            "status": node["status"],
        })
    return impacts


def impact_score(
    dependency_strength: float,
    rule_change_distance: float,
    usage_risk: float,
    memory_confidence: float,
    consequence_severity: float,
) -> float:
    return round(
        dependency_strength
        * rule_change_distance
        * usage_risk
        * memory_confidence
        * consequence_severity,
        4,
    )


def lazy_evaluation_action(score: float, used: bool = True) -> str:
    if not used:
        return "deferred"
    if score >= 0.45:
        return "eager_revalidation"
    if score >= 0.22:
        return "confidence_reduced"
    return "pending_rederivation"


def evaluate_rule_change(
    old_rule: dict,
    new_rule: dict,
    semantic_nodes: list[dict],
    procedures: list[dict],
    plans: list[dict],
) -> dict:
    dependencies = trace_dependencies(old_rule["id"], semantic_nodes, procedures, plans)
    rule_change_distance = 0.8 if old_rule["claim"] != new_rule["claim"] else 0.1
    effects = []
    for dependency in dependencies:
        used = bool(dependency["used_by_procedures"] or dependency["used_by_plans"])
        usage_risk = 0.9 if dependency["used_by_plans"] else 0.55 if dependency["used_by_procedures"] else 0.2
        consequence_severity = 0.9 if "safety" in old_rule.get("base_id", "") else 0.62
        score = impact_score(
            dependency_strength=0.9,
            rule_change_distance=rule_change_distance,
            usage_risk=usage_risk,
            memory_confidence=dependency["memory_confidence"],
            consequence_severity=consequence_severity,
        )
        effects.append({
            **dependency,
            "impact_score": score,
            "lazy_action": lazy_evaluation_action(score, used=used),
            "old_rule_id": old_rule["id"],
            "new_rule_id": new_rule["id"],
        })

    return {
        "old_rule_id": old_rule["id"],
        "new_rule_id": new_rule["id"],
        "dependency_count": len(dependencies),
        "effects": effects,
        "frozen": False,
    }
