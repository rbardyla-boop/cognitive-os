"""v0.1 attention, budget, backpressure, and coalescing helpers."""

from __future__ import annotations

from collections import Counter


SYSTEM_MODES = {
    "Reflective": "deep_reasoning_allowed",
    "Operational": "normal_planning",
    "Strained": "defer_consolidation",
    "Emergency": "minimax_safety_only",
    "Reflex": "precompiled_policy_only",
    "Recovery": "replay_deferred_packets",
}


MODE_THRESHOLDS = {
    "Reflex": 12.0,
    "Emergency": 8.0,
    "Strained": 4.0,
    "Operational": 0.30,
    "Reflective": 0.0,
}


def admission_score(
    safety: float,
    urgency: float,
    goal_relevance: float,
    expected_confidence_delta: float,
    time_sensitivity: float,
    compute_cost: float,
    latency_cost: float,
) -> float:
    denominator = max(compute_cost * latency_cost, 0.01)
    score = (
        safety
        * urgency
        * goal_relevance
        * expected_confidence_delta
        * time_sensitivity
        / denominator
    )
    return round(score, 4)


def score_packet(packet: dict) -> float:
    header = packet["header"]
    packet_type = header["packet_type"]
    priority = header["priority"]
    confidence = packet["epistemics"]["confidence"]
    time_budget_ms = max(header["time_budget_ms"], 1)

    safety_by_priority = {
        "P0": 1.0,
        "P1": 0.88,
        "P2": 0.72,
        "P3": 0.81,
        "P4": 0.35,
        "P5": 0.2,
        "P6": 0.12,
    }
    urgency_by_priority = {
        "P0": 1.0,
        "P1": 0.92,
        "P2": 0.74,
        "P3": 0.68,
        "P4": 0.25,
        "P5": 0.14,
        "P6": 0.08,
    }
    relevance_by_type = {
        "IntentPacket": 0.95,
        "RetrievalRequest": 0.8,
        "RetrievalResult": 0.82,
        "ContradictionPacket": 0.86,
        "BackpressureCommand": 0.84,
        "PlanProposal": 0.9,
        "ActionCommand": 0.92,
        "ActionOutcome": 0.88,
        "MemoryMutation": 0.38,
        "SystemStatePacket": 0.76,
    }
    expected_delta = max(1.0 - confidence, 0.05)
    compute_cost = 0.45 if packet_type in {"PlanProposal", "RetrievalResult"} else 0.25
    latency_cost = max(time_budget_ms / 1000.0, 0.05)
    return admission_score(
        safety_by_priority[priority],
        urgency_by_priority[priority],
        relevance_by_type.get(packet_type, 0.5),
        expected_delta,
        1.0 if time_budget_ms <= 100 else 0.75,
        compute_cost,
        latency_cost,
    )


def choose_mode(max_score: float, queue_depth: int, time_budget_minutes: int) -> str:
    pressure = max_score
    if queue_depth >= 100:
        pressure = max(pressure, 4.0)
    if queue_depth >= 500:
        pressure = max(pressure, 8.0)
    if time_budget_minutes <= 3:
        pressure = max(pressure, 12.0)

    for mode, threshold in MODE_THRESHOLDS.items():
        if pressure >= threshold:
            return mode
    return "Operational"


def memory_backpressure(mode: str, max_results: int = 3) -> dict:
    return {
        "type": "BackpressureCommand",
        "target_engine": "memory",
        "mode": "reduce_output" if mode in {"Strained", "Emergency", "Reflex"} else "normal_output",
        "max_results": max_results,
        "preserve": ["high_confidence", "high_relevance"],
        "defer": ["background_consolidation"] if mode in {"Strained", "Emergency", "Reflex"} else [],
    }


def coalesce_anomalies(anomalies: list[dict]) -> dict | None:
    if len(anomalies) < 3:
        return None

    subjects = Counter(item.get("subject", "unknown") for item in anomalies)
    subject, _count = subjects.most_common(1)[0]
    confidence = min(0.99, 0.5 + (len(anomalies) / 1000.0 * 0.33))
    return {
        "trend": f"{subject} risk increasing",
        "confidence": round(confidence, 2),
        "source_count": len(anomalies),
        "coalesced_from": "low_level_anomaly_packets",
    }
