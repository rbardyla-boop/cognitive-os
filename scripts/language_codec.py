"""Deterministic language codec for v0.1."""

from __future__ import annotations

import re


INTERNAL_PROSE_FIELDS = {
    "instruction",
    "instructions",
    "message_to_engine",
    "natural_language_instruction",
    "prompt",
}


def parse_human_command(command: str) -> dict:
    lower = command.lower().strip()
    target = None
    if re.search(r"\bbridge\s*a\b", lower):
        target = "bridge_A"
    elif re.search(r"\bbridge\s*b\b", lower):
        target = "bridge_B"

    if "cross" in lower:
        goal = "cross"
    elif "wait" in lower:
        goal = "wait"
    else:
        goal = "reach_destination"

    return {
        "goal": goal,
        "target": target,
        "destination": "far_side",
        "preferred_bridge": _target_to_bridge(target),
        "urgency": "high" if any(token in lower for token in ("quickly", "urgent", "fast", "now")) else "normal",
        "evidence_requirement": _evidence_requirement(lower),
        "requires_logging": "do not log" not in lower,
        "raw_text": command,
        "codec": "stub_deterministic_v0.1",
    }


def render_human_explanation(plan: dict, confidence: float, epistemic_license: str) -> str:
    return (
        f"Chose {plan['route']} using {plan['mode']} planning with "
        f"{epistemic_license} license at confidence {confidence}."
    )


def llm_human_to_candidate_packet(_text: str) -> dict:
    raise NotImplementedError("LLM adapter is intentionally disabled in v0.1.")


def llm_packet_state_to_human_explanation(_packet_state: dict) -> str:
    raise NotImplementedError("LLM adapter is intentionally disabled in v0.1.")


def assert_no_internal_prose_handoff(packet: dict) -> None:
    source = packet["header"]["source_engine"]
    target = packet["header"]["target_engine"]
    if source in {"language_codec", "renderer"} or target in {"language_codec", "renderer"}:
        return
    _check_value(packet["payload"], path="payload")


def _check_value(value, path: str) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in INTERNAL_PROSE_FIELDS:
                raise ValueError(f"Natural-language internal routing field is forbidden: {path}.{key}")
            _check_value(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _check_value(child, f"{path}[{index}]")


def _target_to_bridge(target: str | None) -> str | None:
    if target == "bridge_A":
        return "Bridge A"
    if target == "bridge_B":
        return "Bridge B"
    return None


def _evidence_requirement(lower_text: str) -> str:
    if any(token in lower_text for token in ("certify", "guarantee", "prove", "100%")):
        return "Strict"
    if any(token in lower_text for token in ("safe", "damage", "danger", "risk")):
        return "Cautious"
    return "HypothesisOK"
