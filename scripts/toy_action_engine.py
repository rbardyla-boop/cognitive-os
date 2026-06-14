"""Toy-world action execution and outcome recording."""

from __future__ import annotations

from world_encoder import predict_action


ALLOWED_ACTIONS = {
    "cross_bridge_A",
    "cross_bridge_B",
    "wait",
    "request_more_evidence",
    "take_safe_route",
    "quarantine_memory",
}


def execute_action(command: dict, world_state: dict) -> dict:
    action = command["action"]
    if action not in ALLOWED_ACTIONS:
        raise ValueError(f"Unsupported toy action: {action}")

    if action == "cross_bridge_A":
        prediction = predict_action(action, world_state)
        success = prediction["likely_outcome"] == "arrived"
        return _outcome(action, success, prediction["likely_outcome"], prediction)
    if action == "cross_bridge_B":
        prediction = predict_action(action, world_state)
        return _outcome(action, prediction["likely_outcome"] == "arrived", prediction["likely_outcome"], prediction)
    if action == "take_safe_route":
        prediction = predict_action(action, world_state)
        return _outcome(action, True, "arrived_via_safe_route", prediction)
    if action == "wait":
        prediction = predict_action(action, world_state)
        return _outcome(action, True, prediction["likely_outcome"], prediction)
    if action == "request_more_evidence":
        prediction = predict_action(action, world_state)
        return _outcome(action, True, prediction["likely_outcome"], prediction)
    prediction = predict_action(action, world_state)
    return _outcome(action, True, prediction["likely_outcome"], prediction)


def record_action_outcome(outcome: dict, trace_id: str, command_packet_id: str, outcome_packet_id: str, timestamp: str) -> dict:
    episode = {
        "episode_id": f"E_{outcome_packet_id.lower()}",
        "timestamp": timestamp,
        "source": "action",
        "raw_payload": outcome,
        "parsed_claims": [f"{outcome['action']} -> {outcome['observed_state']}"],
        "confidence": 0.9 if outcome["success"] else 0.62,
        "trace_id": trace_id,
        "linked_actions": [command_packet_id, outcome_packet_id],
        "linked_rules": ["R_bridge_safety:v1"],
    }
    memory_update_candidate = {
        "operation": "append_episode",
        "hidden": False,
        "candidate": True,
        "episode": episode,
    }
    return {
        "action_outcome": outcome,
        "episode_packet": episode,
        "memory_update_candidate": memory_update_candidate,
        "trace_link": {
            "trace_id": trace_id,
            "action_command": command_packet_id,
            "action_outcome": outcome_packet_id,
        },
    }


def _outcome(action: str, success: bool, observed_state: str, prediction: dict) -> dict:
    return {
        "action": action,
        "success": success,
        "observed_state": observed_state,
        "prediction": prediction,
        "message": "Toy action executed in sandbox.",
    }
