#!/usr/bin/env python3
"""Contradiction repair audit replay for Sprint 12 scenarios."""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run


def audit_contradiction_trace(trace: list[dict]) -> dict:
    repairs = [
        packet for packet in trace
        if packet["header"]["packet_type"] == "MemoryMutation"
        and packet["payload"].get("operation") == "contradiction_repair"
    ]
    if not repairs:
        raise ValueError("trace has no contradiction repair mutation")
    entries = [_repair_entry(packet) for packet in repairs]
    latest = dict(entries[-1])
    latest["mutations"] = entries
    latest["mutation_order"] = [entry["target"] for entry in entries]
    latest["raw_episodes_preserved"] = all(entry["raw_episodes_preserved"] for entry in entries)
    latest["unresolved_visible"] = latest["repair_type"] == "unresolved" and all(
        entry["after"] == "contradicted" for entry in entries
    )
    return latest


def _repair_entry(packet: dict) -> dict:
    payload = packet["payload"]
    log = payload["mutation_log_entry"]
    result = payload["repair_result"]
    return {
        "repair_id": payload["repair_id"],
        "repair_type": payload["repair_type"],
        "source_contradiction": payload["source_contradiction"],
        "target": log["target_object_id"],
        "before": log["before_status"],
        "after": log["after_status"],
        "mutation": log["mutation_type"],
        "decision": payload["repair_decision"],
        "source_evidence": payload["source_evidence"],
        "verifier_rule_id": payload["verifier_rule_id"],
        "verifier_decision": log["verifier_decision_id"],
        "scope_conditions": payload.get("scope_conditions", {}),
        "strict_action_blocked": payload.get("strict_action_blocked", False),
        "raw_episodes_preserved": (
            payload["raw_episodes_preserved"]
            and payload["raw_episode_count_before"] == payload["raw_episode_count_after"]
        ),
        "repair_result_mutations": result["mutations"],
        "append_only_log_entries": len(payload["mutation_log"]),
        "trace_id": packet["header"]["trace_id"],
    }


def main() -> int:
    if len(sys.argv) < 3 or sys.argv[1] != "--scenario":
        raise SystemExit("usage: contradiction_audit.py --scenario <scenario_name>")
    scenario = load_scenario(sys.argv[2])
    trace = run(scenario["command"], scenario)
    print(json.dumps(audit_contradiction_trace(trace), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
