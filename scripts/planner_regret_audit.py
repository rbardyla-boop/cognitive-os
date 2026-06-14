#!/usr/bin/env python3
"""Planner regret audit replay for Sprint 14 scenarios."""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run


def audit_planner_regret_trace(trace: list[dict]) -> dict:
    regret = next(
        packet for packet in trace
        if packet["header"]["packet_type"] == "PlanRegretPacket"
    )
    mutation = next(
        packet for packet in trace
        if packet["header"]["packet_type"] == "MemoryMutation"
        and packet["payload"].get("operation") == "planner_regret_policy_update"
    )
    review = next(
        packet for packet in trace
        if packet["header"]["packet_type"] == "BackpressureCommand"
        and packet["payload"].get("type") == "planner_review"
    )
    log = mutation["payload"]["mutation_log_entry"]
    return {
        "regret_id": regret["payload"]["regret_id"],
        "regret_type": regret["payload"]["regret_type"],
        "regret_class": regret["payload"]["regret_class"],
        "expected": regret["payload"]["expected"],
        "actual": regret["payload"]["actual"],
        "selected_action": regret["payload"]["selected_action"],
        "selected_route": regret["payload"]["selected_route"],
        "target_policy_id": regret["payload"]["target_policy_id"],
        "review_required": regret["payload"]["review_required"],
        "review_status": review["payload"]["status"],
        "review_deferred": review["payload"]["deferred"],
        "review_reason": review["payload"]["reason"],
        "policy_update_kind": mutation["payload"]["policy_update_kind"],
        "scope_conditions": mutation["payload"]["scope_conditions"],
        "mutation": log["mutation_type"],
        "before": log["before_status"],
        "after": log["after_status"],
        "decision": log["decision"],
        "verifier_rule_id": mutation["payload"]["verifier_rule_id"],
        "global_rule_rewrite": mutation["payload"]["global_rule_rewrite"],
        "belief_or_procedure_authority_changed": mutation["payload"]["belief_or_procedure_authority_changed"],
        "append_only_log_entries": len(mutation["payload"]["mutation_log"]),
        "trace_id": regret["header"]["trace_id"],
    }


def main() -> int:
    if len(sys.argv) < 3 or sys.argv[1] != "--scenario":
        raise SystemExit("usage: planner_regret_audit.py --scenario <scenario_name>")
    scenario = load_scenario(sys.argv[2])
    trace = run(scenario["command"], scenario)
    print(json.dumps(audit_planner_regret_trace(trace), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
