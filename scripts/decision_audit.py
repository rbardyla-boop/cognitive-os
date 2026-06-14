#!/usr/bin/env python3
"""Trace replay and decision audit for bridge-world runs."""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run


def audit_trace(trace: list[dict]) -> dict:
    by_type = {}
    for packet in trace:
        by_type.setdefault(packet["header"]["packet_type"], []).append(packet)

    intent = by_type["IntentPacket"][0]
    retrieval = by_type["RetrievalResult"][0]
    contradictions = by_type.get("ContradictionPacket", [])
    attention = next(
        packet for packet in by_type.get("BackpressureCommand", [])
        if "system_mode" in packet["payload"]
    )
    plan = by_type["PlanProposal"][0]
    action = by_type["ActionCommand"][0]
    outcome = by_type["ActionOutcome"][0]
    revalidation = [
        packet for packet in by_type.get("BackpressureCommand", [])
        if packet["payload"].get("type") == "post_action_revalidation"
    ]

    primary_factors = []
    if intent["payload"].get("preferred_bridge"):
        primary_factors.append(f"User preferred {intent['payload']['preferred_bridge']}.")
    if intent["payload"].get("urgency") == "high":
        primary_factors.append("Urgency parsed as high.")
    if "M_bridge_a_damage_reported" in json.dumps(retrieval["payload"]):
        primary_factors.append("Bridge A had active damage report after rain.")
    if any(packet["epistemics"]["epistemic_license"] == "hazard_only" for packet in contradictions):
        primary_factors.append("Damage/risk evidence produced hazard_only contradiction packets.")
    primary_factors.append(f"Attention Manager entered {attention['payload']['system_mode']}.")
    primary_factors.append(f"Planner switched to {plan['payload']['mode']}.")
    primary_factors.append(f"{plan['payload']['route']} had lower worst-case consequence or safer fallback.")
    primary_factors.append("ActionOutcome and post_action_revalidation were scheduled." if revalidation else "ActionOutcome was recorded.")

    blocked_alternatives = []
    if any(packet["epistemics"]["epistemic_license"] == "hazard_only" for packet in contradictions):
        blocked_alternatives.append("Bridge A direct recommendation blocked by hazard_only contradiction evidence.")
    blocked_alternatives.append("Bridge A safety certification blocked by forbidden_use metadata.")
    if plan["payload"]["route"] == "wait":
        blocked_alternatives.append("Bridge B was not blindly selected because its worst-case risk was also degraded.")

    return {
        "decision": f"recommend {plan['payload']['route']}",
        "action": action["payload"]["action"],
        "outcome": outcome["payload"]["observed_state"],
        "trace_id": trace[0]["header"]["trace_id"],
        "primary_factors": primary_factors,
        "blocked_alternatives": blocked_alternatives,
        "post_action_revalidation": bool(revalidation),
        "packet_chain": [
            packet["header"]["packet_id"]
            for packet in trace
            if packet["header"]["packet_type"] in {
                "IntentPacket",
                "RetrievalRequest",
                "RetrievalResult",
                "ContradictionPacket",
                "PlanProposal",
                "ActionCommand",
                "ActionOutcome",
                "MemoryMutation",
                "BackpressureCommand",
            }
        ],
    }


def main() -> int:
    if "--project" in sys.argv[1:]:
        # The development process is audited by the same surface as runtime decisions.
        from project_self_audit import run_project_audit

        strict = "--strict" in sys.argv[1:] or "--strict-project" in sys.argv[1:]
        report, exit_code = run_project_audit(strict=strict, emit_health=True)
        print(json.dumps(report, indent=2, sort_keys=True))
        return exit_code
    if len(sys.argv) >= 3 and sys.argv[1] == "--scenario":
        scenario = load_scenario(sys.argv[2])
        trace = run(scenario["command"], scenario)
    else:
        command = " ".join(sys.argv[1:]) or "I need to cross the river quickly. Is Bridge A safe?"
        trace = run(command)
    print(json.dumps(audit_trace(trace), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

