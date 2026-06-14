"""Toy bridge-world planner with normal and minimax modes."""

from __future__ import annotations

from world_encoder import bridge_display, bridge_key, predict_action


def build_plan(
    goal: dict,
    retrieved_memories: dict,
    epistemic_license: str,
    world_state: dict,
    time_budget_minutes: int,
    risk_budget: float,
    system_mode: str,
) -> dict:
    if goal.get("evidence_requirement") == "Strict" and epistemic_license != "full_premise":
        return {
            "goal": goal["goal"],
            "mode": "evidence_strict_refusal",
            "route": "wait",
            "action": "request_more_evidence",
            "eta_minutes": 1,
            "prediction": {"risk": 0.05, "cost_minutes": 1, "likely_outcome": "evidence_requested"},
            "fallback_plan": {
                "action": "wait",
                "reason": "Strict evidence requirement blocks fallback-aware crossing.",
            },
            "risk_note": "Strict evidence required; degraded verifier license cannot support crossing.",
            "required_assumptions": ["Fresh verification is required before any crossing recommendation."],
            "risk_budget": risk_budget,
            "rationale": "Evidence requirement level overrides urgency and route optimization.",
            "candidate_actions": [
                "wait",
                "request_more_evidence",
                "quarantine_memory",
            ],
        }

    bridge_scores = {
        "Bridge A": _bridge_score("Bridge A", world_state, retrieved_memories, epistemic_license, risk_budget),
        "Bridge B": _bridge_score("Bridge B", world_state, retrieved_memories, epistemic_license, risk_budget),
    }
    mode = "minimax" if system_mode in {"Emergency", "Reflex"} or risk_budget <= 0.35 else "normal"
    route = _minimax_route(bridge_scores) if mode == "minimax" else _normal_route(goal, bridge_scores, time_budget_minutes)
    if route == "wait":
        action = "wait"
        selected = {
            "eta": 1,
            "prediction": {"risk": 0.05, "cost_minutes": 1, "likely_outcome": "waiting_for_conditions"},
            "risk_note": "No crossing route is inside the risk budget; waiting is least catastrophic.",
        }
    else:
        action = "cross_bridge_A" if route == "Bridge A" else "cross_bridge_B"
        selected = bridge_scores[route]
    fallback = "request_more_evidence" if epistemic_license in {"hypothesis_only", "hazard_only"} else "take_safe_route"
    if route == "Bridge B":
        fallback = "wait"

    return {
        "goal": goal["goal"],
        "mode": mode,
        "route": route,
        "action": action,
        "eta_minutes": selected["eta"],
        "prediction": selected["prediction"],
        "fallback_plan": {
            "action": fallback,
            "reason": "Fallback preserves safety if route assumptions fail.",
        },
        "risk_note": selected["risk_note"],
        "required_assumptions": _required_assumptions(route, retrieved_memories, epistemic_license),
        "risk_budget": risk_budget,
        "rationale": "Selected route from world state, retrieval licenses, time budget, and risk budget.",
        "candidate_actions": [
            "cross_bridge_A",
            "cross_bridge_B",
            "wait",
            "request_more_evidence",
            "take_safe_route",
            "quarantine_memory",
        ],
    }


def _normal_route(goal: dict, bridge_scores: dict, time_budget_minutes: int) -> str:
    preferred = goal.get("preferred_bridge")
    if preferred and bridge_scores[preferred]["allowed"] and bridge_scores[preferred]["eta"] <= time_budget_minutes:
        return preferred
    allowed = [name for name, score in bridge_scores.items() if score["allowed"] and score["eta"] <= time_budget_minutes]
    if allowed:
        return min(allowed, key=lambda name: (bridge_scores[name]["risk"], bridge_scores[name]["eta"]))
    return _minimax_route(bridge_scores)


def _minimax_route(bridge_scores: dict) -> str:
    route = min(bridge_scores, key=lambda name: bridge_scores[name]["worst_case_loss"])
    if bridge_scores[route]["worst_case_loss"] >= 75:
        return "wait"
    return route


def _bridge_score(route: str, world_state: dict, retrieved_memories: dict, epistemic_license: str, risk_budget: float) -> dict:
    key = bridge_key(route)
    action = "cross_bridge_A" if key == "A" else "cross_bridge_B"
    prediction = predict_action(action, world_state)
    risk = prediction["risk"]
    scope_note = _scoped_memory_note(route, world_state, retrieved_memories)
    if scope_note["mismatches"]:
        risk += 0.25
    if epistemic_license in {"hypothesis_only", "hazard_only"}:
        risk += 0.2
    if epistemic_license == "do_not_use_for_action":
        risk += 0.5
    risk = min(risk, 1.0)
    return {
        "eta": prediction["cost_minutes"],
        "risk": risk,
        "allowed": risk <= risk_budget or key == "B",
        "worst_case_loss": risk * 100 + prediction["cost_minutes"],
        "risk_note": (
            f"{bridge_display(key)} risk={risk:.2f}; worst_case_loss={risk * 100 + prediction['cost_minutes']:.1f}"
            + scope_note["text"]
        ),
        "prediction": prediction,
    }


def _required_assumptions(route: str, retrieved_memories: dict, epistemic_license: str) -> list[str]:
    assumptions = [
        f"{route} remains physically passable during execution.",
        f"Verifier license remains {epistemic_license} or better for fallback-aware planning.",
    ]
    if any(item["revalidation_requirement"] == "post_action_revalidation" for item in retrieved_memories["semantic_nodes"]):
        assumptions.append("Degraded memory must be revalidated after action.")
    return assumptions


def _scoped_memory_note(route: str, world_state: dict, retrieved_memories: dict) -> dict:
    matches = []
    mismatches = []
    route_token = route.lower()
    for item in retrieved_memories.get("semantic_nodes", []):
        content = item["content"]
        if item.get("status") != "exception_scoped":
            continue
        if "bridge a" not in content.get("claim", "").lower() and "bridge b" not in content.get("claim", "").lower():
            continue
        if route_token not in content.get("claim", "").lower():
            continue
        scope = content.get("scope_conditions", {})
        if _scope_matches(scope, world_state):
            matches.append(content["memory_id"])
        else:
            mismatches.append(content["memory_id"])
    text = ""
    if matches:
        text += f"; scoped_matches={','.join(matches)}"
    if mismatches:
        text += f"; scoped_mismatches={','.join(mismatches)}"
    return {"matches": matches, "mismatches": mismatches, "text": text}


def _scope_matches(scope: dict, world_state: dict) -> bool:
    if not scope:
        return True
    rain = scope.get("rain_level")
    if rain == "heavy" and world_state.get("weather") != "heavy_rain":
        return False
    if rain == "clear_or_light" and world_state.get("weather") == "heavy_rain":
        return False
    inspection = scope.get("inspection_status")
    if inspection == "recent" and not world_state.get("recent_inspection", False):
        return False
    if inspection == "not_recent" and world_state.get("recent_inspection", False):
        return False
    return True
