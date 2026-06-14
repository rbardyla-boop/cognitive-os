#!/usr/bin/env python3
"""Attention mode review audit replay for Sprint 15 scenarios."""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run


def audit_attention_review_trace(trace: list[dict]) -> dict:
    review = next(
        packet for packet in trace
        if packet["header"]["packet_type"] == "AttentionModeReviewPacket"
    )
    mutation = next(
        packet for packet in trace
        if packet["header"]["packet_type"] == "MemoryMutation"
        and packet["payload"].get("operation") == "attention_mode_policy_update"
    )
    pending = next(
        packet for packet in trace
        if packet["header"]["packet_type"] == "BackpressureCommand"
        and packet["payload"].get("type") == "attention_mode_review"
    )
    log = mutation["payload"]["mutation_log_entry"]
    payload = review["payload"]
    return {
        "review_id": payload["review_id"],
        "observed_mode": payload["observed_mode"],
        "classification": payload["classification"],
        "review_required": payload["review_required"],
        "review_status": pending["payload"]["status"],
        "review_deferred": pending["payload"]["deferred"],
        "policy_update_kind": mutation["payload"]["policy_update_kind"],
        "target_kind": mutation["payload"]["target_kind"],
        "mutation": log["mutation_type"],
        "decision": log["decision"],
        "before": log["before_status"],
        "after": log["after_status"],
        "memory_authority_changed": mutation["payload"]["memory_authority_changed"],
        "procedure_authority_changed": mutation["payload"]["procedure_authority_changed"],
        "planner_authority_changed": mutation["payload"]["planner_authority_changed"],
        "verifier_rule_changed": mutation["payload"]["verifier_rule_changed"],
        "raw_packets_preserved": payload["raw_packet_preservation"]["preserved"],
        "raw_packet_count": payload["raw_packet_preservation"]["raw_packet_count"],
        "coalesced": payload["coalescing"]["coalesced"],
        "coalesced_source_count": payload["coalescing"]["source_count"],
        "kept_alive": payload["priority_preservation"]["kept_alive"],
        "deferred_priorities": payload["priority_preservation"]["deferred_priorities"],
        "recovery_replay": payload["recovery_replay"],
        "verifier_rule_id": mutation["payload"]["verifier_rule_id"],
        "append_only_log_entries": len(mutation["payload"]["mutation_log"]),
        "trace_id": review["header"]["trace_id"],
    }


def main() -> int:
    if len(sys.argv) < 3 or sys.argv[1] != "--scenario":
        raise SystemExit("usage: attention_review_audit.py --scenario <scenario_name>")
    scenario = load_scenario(sys.argv[2])
    trace = run(scenario["command"], scenario)
    print(json.dumps(audit_attention_review_trace(trace), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
