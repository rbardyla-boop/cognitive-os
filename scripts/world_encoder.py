"""Structured toy-world encoder and prediction stub."""

from __future__ import annotations


BRIDGE_LABELS = {
    "Bridge A": "A",
    "Bridge B": "B",
    "bridge_A": "A",
    "bridge_B": "B",
    "A": "A",
    "B": "B",
}


def encode_world_state(raw_state: dict) -> dict:
    bridges = raw_state["bridges"]
    return {
        "location": raw_state["location"],
        "destination": raw_state["destination"],
        "weather": raw_state["weather"],
        "time_budget_minutes": raw_state["time_budget_minutes"],
        "bridges": {
            "A": _bridge(bridges["A"]),
            "B": _bridge(bridges["B"]),
        },
    }


def predict_action(action: str, world_state: dict) -> dict:
    bridge_key = _action_bridge(action)
    if bridge_key is None:
        return {
            "risk": 0.05,
            "cost_minutes": 1 if action in {"wait", "request_more_evidence"} else 2,
            "likely_outcome": _non_bridge_outcome(action),
        }

    bridge = world_state["bridges"][bridge_key]
    risk = 0.1 + bridge["rain_exposure"] * _weather_multiplier(world_state["weather"])
    if bridge["damage_report"]:
        risk += 0.25
    if bridge["status"] in {"unknown", "closed"}:
        risk += 0.15
    risk = round(min(risk, 1.0), 2)
    return {
        "risk": risk,
        "cost_minutes": bridge["base_minutes"],
        "likely_outcome": "arrived" if risk < 0.65 and bridge["status"] != "closed" else "deferred_at_bridge",
    }


def bridge_key(label: str | None) -> str | None:
    if label is None:
        return None
    return BRIDGE_LABELS.get(label, label)


def bridge_display(key: str) -> str:
    return "Bridge A" if key == "A" else "Bridge B"


def _bridge(value: dict) -> dict:
    return {
        "status": value["status"],
        "rain_exposure": float(value["rain_exposure"]),
        "damage_report": bool(value["damage_report"]),
        "base_minutes": int(value["base_minutes"]),
    }


def _weather_multiplier(weather: str) -> float:
    if weather == "heavy_rain":
        return 0.7
    if weather == "rain":
        return 0.45
    return 0.1


def _action_bridge(action: str) -> str | None:
    if action == "cross_bridge_A":
        return "A"
    if action == "cross_bridge_B":
        return "B"
    if action == "take_safe_route":
        return "B"
    return None


def _non_bridge_outcome(action: str) -> str:
    return {
        "wait": "waiting_for_conditions",
        "request_more_evidence": "evidence_requested",
        "quarantine_memory": "memory_quarantined",
    }.get(action, "noop")

