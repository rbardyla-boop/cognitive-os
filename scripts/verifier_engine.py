"""Verifier and adjudication helpers for v0.1."""

from __future__ import annotations

import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERIFIER_RULES = ROOT / "simulations" / "bridge_world" / "verifier_rules.json"

CONFLICT_TYPES = {
    "no_conflict",
    "soft_conflict",
    "hard_contradiction",
    "scope_mismatch",
    "known_exception",
    "unknown_anomaly",
}

ADJUDICATION_OUTCOMES = {
    "reject_episode",
    "preserve_as_exception",
    "candidate_rule_revision",
    "fork_model_context",
}

DERIVED_POLICIES = {
    "contradiction_index": "VR_contradiction_index_policy:v0.1",
    "heavy_rain_route_block": "VR_heavy_rain_route_block:v0.1",
}


def trust_score(
    source_reliability: float,
    timestamp_integrity: float,
    corroboration: float,
    parse_confidence: float,
    sensor_confidence: float,
    adversarial_risk: float,
    recency: float,
    dependency_stability: float,
) -> float:
    positive = (
        source_reliability
        * timestamp_integrity
        * corroboration
        * parse_confidence
        * sensor_confidence
        * recency
        * dependency_stability
    )
    score = positive / max(adversarial_risk, 0.05)
    return round(min(score, 1.0), 4)


def episode_trust(episode: dict) -> float:
    episode = episode["content"]
    raw = episode.get("raw_payload", {})
    stale = raw.get("staleness") == "stale"
    return trust_score(
        source_reliability=episode.get("confidence", 0.5),
        timestamp_integrity=0.95 if episode.get("timestamp") else 0.3,
        corroboration=0.55 if stale else 0.82,
        parse_confidence=0.9,
        sensor_confidence=0.72 if raw.get("status") == "risky" else 0.8,
        adversarial_risk=0.22 if stale else 0.12,
        recency=0.42 if stale else 0.9,
        dependency_stability=0.58 if episode.get("linked_rules") else 0.82,
    )


def load_verifier_rules(path: Path = VERIFIER_RULES) -> list[dict]:
    with path.open("r", encoding="utf-8") as handle:
        rules = json.load(handle)
    for verifier_rule in rules:
        validate_verifier_rule(verifier_rule)
    return rules


def validate_verifier_rule(verifier_rule: dict) -> None:
    if verifier_rule["epistemic_license"] not in {"full_premise", "weak_premise"}:
        raise ValueError(f"Verifier rule is too weak to govern decisions: {verifier_rule['id']}")
    if verifier_rule["conflict_type"] not in CONFLICT_TYPES:
        raise ValueError(f"Unknown conflict type in verifier rule: {verifier_rule['id']}")


def detect_conflict(node: dict, rule: dict, world: dict, verifier_rules: list[dict] | None = None) -> str:
    return detect_conflict_decision(node, rule, world, verifier_rules)["conflict_type"]


def detect_conflict_decision(
    node: dict,
    rule: dict,
    world: dict,
    verifier_rules: list[dict] | None = None,
) -> dict:
    verifier_rules = verifier_rules or load_verifier_rules()
    for verifier_rule in verifier_rules:
        if _matches_verifier_rule(verifier_rule, node, rule, world):
            return {
                "conflict_type": verifier_rule["conflict_type"],
                "verifier_rule_id": verifier_rule["id"],
                "verifier_rule_license": verifier_rule["epistemic_license"],
            }
    return {
        "conflict_type": "no_conflict",
        "verifier_rule_id": "VR_default_no_conflict:v0.1",
        "verifier_rule_license": "full_premise",
    }


def _matches_verifier_rule(verifier_rule: dict, node: dict, rule: dict, world: dict) -> bool:
    when = verifier_rule["when"]
    claim = node["claim"].lower()
    rule_claim = rule["claim"].lower()
    applies_to = set(rule.get("applies_to", []))

    if "claim_contains" in when and not all(token in claim for token in when["claim_contains"]):
        return False
    if "rule_claim_contains" in when and not all(token in rule_claim for token in when["rule_claim_contains"]):
        return False
    if "status" in when and node.get("status") != when["status"]:
        return False
    if "weather" in when and world.get("weather") != when["weather"]:
        return False
    if "applies_to_excludes" in when and applies_to.intersection(when["applies_to_excludes"]):
        return False
    return True


def revision_pressure(
    surprisal: float,
    trust_episode: float,
    reproducibility: float,
    context_fit: float,
    corroboration: float,
    trust_rule: float,
    known_exception_fit: float,
    adversarial_risk: float,
) -> float:
    numerator = surprisal * trust_episode * reproducibility * context_fit * corroboration
    denominator = max(trust_rule, 0.05) * max(known_exception_fit, 0.05) * max(adversarial_risk, 0.05)
    return round(min(numerator / denominator, 1.0), 4)


def adjudicate(conflict_type: str, pressure: float, repeated_anomalies: int) -> str:
    if conflict_type == "no_conflict":
        return "preserve_as_exception"
    if conflict_type == "scope_mismatch":
        return "fork_model_context"
    if conflict_type == "known_exception":
        return "preserve_as_exception"
    if conflict_type == "unknown_anomaly" and repeated_anomalies < 3:
        return "preserve_as_exception"
    if conflict_type == "hard_contradiction" and pressure < 0.45:
        return "reject_episode"
    if repeated_anomalies >= 3 and pressure >= 0.45:
        return "candidate_rule_revision"
    return "preserve_as_exception"


def verify_retrieval(retrieval: dict, world: dict) -> dict:
    stale = [
        episode for episode in retrieval["episodes"]
        if episode["content"]["raw_payload"].get("staleness") == "stale"
    ]
    episode_scores = [
        {"episode_id": episode["content"]["episode_id"], "trust": episode_trust(episode)}
        for episode in retrieval["episodes"]
    ]

    decisions = []
    contradictions = []
    for wrapped_node in retrieval["semantic_nodes"]:
        node = wrapped_node["content"]
        for rule in retrieval["rules"]:
            conflict_decision = detect_conflict_decision(node, rule, world)
            conflict_type = conflict_decision["conflict_type"]
            if conflict_type == "no_conflict":
                continue
            pressure = revision_pressure(
                surprisal=0.85 if conflict_type == "hard_contradiction" else 0.45,
                trust_episode=max([score["trust"] for score in episode_scores] or [0.5]),
                reproducibility=0.35,
                context_fit=0.82 if world["weather"] == "heavy_rain" else 0.55,
                corroboration=0.72 if retrieval["contradictions"] else 0.35,
                trust_rule=0.82,
                known_exception_fit=0.9,
                adversarial_risk=0.22,
            )
            outcome = adjudicate(conflict_type, pressure, repeated_anomalies=1)
            decision = {
                "subject": node["memory_id"],
                "rule_id": rule["id"],
                "verifier_rule_id": conflict_decision["verifier_rule_id"],
                "verifier_rule_license": conflict_decision["verifier_rule_license"],
                "conflict_type": conflict_type,
                "adjudication": outcome,
                "revision_pressure": pressure,
                "reason": node["claim"],
                "confidence": node["confidence"],
            }
            decisions.append(decision)
            if conflict_type in {"hard_contradiction", "unknown_anomaly", "soft_conflict"}:
                contradictions.append(decision)

    for wrapped_item in retrieval["contradictions"]:
        item = wrapped_item["content"]
        contradictions.append({
            "subject": item["memory_id"],
            "rule_id": "contradiction_index",
            "verifier_rule_id": DERIVED_POLICIES["contradiction_index"],
            "verifier_rule_license": "full_premise",
            "conflict_type": "hard_contradiction",
            "adjudication": "preserve_as_exception",
            "revision_pressure": 0.31,
            "reason": item["claim"],
            "confidence": item["confidence"],
        })

    if world["weather"] == "heavy_rain":
        contradictions.append({
            "subject": "Bridge A",
            "rule_id": "R_bridge_safety:v1",
            "verifier_rule_id": DERIVED_POLICIES["heavy_rain_route_block"],
            "verifier_rule_license": "full_premise",
            "conflict_type": "hard_contradiction",
            "adjudication": "reject_episode",
            "revision_pressure": 0.28,
            "reason": "Shortest-route rule conflicts with heavy-rain avoidance rule.",
            "confidence": 0.82,
        })

    confidence = 0.74 if contradictions else 0.9
    if stale:
        confidence -= 0.12
    if confidence >= 0.85:
        license_level = "full_premise"
    elif confidence >= 0.6:
        license_level = "weak_premise"
    elif contradictions:
        license_level = "hypothesis_only"
    else:
        license_level = "do_not_use_for_action"

    return {
        "confidence": round(confidence, 2),
        "epistemic_license": license_level,
        "trust_scores": episode_scores,
        "decisions": decisions,
        "caveats": [
            "Bridge A memory is stale." if stale else "No stale bridge memory retrieved.",
            "Heavy rain increases Bridge A risk." if world["weather"] == "heavy_rain" else "Weather does not raise bridge-specific risk.",
        ],
        "contradictions": contradictions,
    }
