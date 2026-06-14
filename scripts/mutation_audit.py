#!/usr/bin/env python3
"""Mutation audit replay for Sprint 10 scenarios."""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run


def audit_mutation_trace(trace: list[dict]) -> dict:
    mutations = [
        packet for packet in trace
        if packet["header"]["packet_type"] == "MemoryMutation"
        and "mutation_log_entry" in packet["payload"]
    ]
    if not mutations:
        raise ValueError("trace has no mutation audit record")
    entries = [_audit_entry(packet) for packet in mutations]
    latest = dict(entries[-1])
    latest["mutations"] = entries
    latest["correction_order"] = [entry["target"] for entry in entries]
    return latest


def _audit_entry(mutation: dict) -> dict:
    payload = mutation["payload"]
    log = payload["mutation_log_entry"]
    source_type = payload.get("source_packet_type") or _source_type(payload)
    verifier = payload.get("verifier_decision") or {}
    return {
        "mutation": log["mutation_type"],
        "target": log["target_object_id"],
        "before": log["before_status"],
        "after": log["after_status"],
        "source": source_type,
        "source_packet_id": log["source_packet_id"],
        "verifier_decision": log["verifier_decision_id"],
        "verifier_rule": verifier.get("rule_id", log["blocking_rule"]),
        "decision": log["decision"],
        "reason": log["reason"],
        "target_unchanged": log["before_status"] == log["after_status"],
        "append_only_log_entries": len(payload["mutation_log"]),
        "belief_update_position": payload.get("belief_update_position"),
        "target_kind": payload.get("target_kind") or _target_kind(log["mutation_type"]),
        "scope_conditions": payload.get("scope_conditions", {}),
        "overconfirmation_blocked": payload.get("overconfirmation_blocked"),
        "trace_id": mutation["header"]["trace_id"],
    }


def _source_type(payload: dict) -> str:
    source = payload.get("source_packet_type")
    if source:
        return source
    request = payload.get("mutation_request", {})
    if request.get("source_packet_id"):
        return "ActionOutcome" if payload.get("operation") == "post_action_correction" else "source_packet"
    return "direct_call"


def _target_kind(mutation_type: str) -> str:
    if mutation_type == "procedure_status_update":
        return "procedure"
    if mutation_type == "semantic_status_update":
        return "belief"
    return "state"


def main() -> int:
    if len(sys.argv) < 3 or sys.argv[1] != "--scenario":
        raise SystemExit("usage: mutation_audit.py --scenario <scenario_name>")
    scenario = load_scenario(sys.argv[2])
    trace = run(scenario["command"], scenario)
    print(json.dumps(audit_mutation_trace(trace), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
