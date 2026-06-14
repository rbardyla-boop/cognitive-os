#!/usr/bin/env python3
"""Stdlib-only bridge-world prototype for the v0.1 cognitive loop."""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

from attention_manager import (
    MODE_THRESHOLDS,
    SYSTEM_MODES,
    choose_mode,
    coalesce_anomalies,
    memory_backpressure,
    score_packet,
)
from cip_bus import InProcessBroker, PRIORITY_LANES
from governed_memory import GovernedMemory
from language_codec import assert_no_internal_prose_handoff, parse_human_command, render_human_explanation
from mutation_gateway import (
    HUMAN_PROMOTION_RULE,
    MUTATION_AUTHORITY_RULE,
    apply_memory_mutation,
    verifier_allows_mutation,
    verifier_blocks_mutation,
)
from rule_cascade import evaluate_rule_change, next_rule_version
from retrieval_policy import retrieval_has_degraded_action_support
from toy_action_engine import execute_action, record_action_outcome
from toy_planner import build_plan
from verifier_engine import verify_retrieval
from world_encoder import encode_world_state


ROOT = Path(__file__).resolve().parents[1]
WORLD = ROOT / "simulations" / "bridge_world"
SCHEMA_VERSION = "0.1"
LICENSE_RULES = {
    "full_premise": {
        "retrieval",
        "planning",
        "planning_with_fallback",
        "human_explanation",
        "contradiction_detection",
        "sandbox_testing",
        "direct_action",
        "memory_consolidation",
        "rule_revision",
        "safety_certification",
        "planner_policy_update",
        "attention_policy_update",
    },
    "weak_premise": {
        "retrieval",
        "planning_with_fallback",
        "human_explanation",
        "contradiction_detection",
        "sandbox_testing",
        "memory_consolidation",
        "planner_policy_update",
        "attention_policy_update",
    },
    "hypothesis_only": {
        "retrieval",
        "planning_with_fallback",
        "human_explanation",
        "contradiction_detection",
        "sandbox_testing",
    },
    "hazard_only": {"human_explanation", "contradiction_detection"},
    "do_not_use_for_action": {"retrieval", "human_explanation"},
}


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


class IdFactory:
    def __init__(self) -> None:
        self.trace_counter = 0
        self.packet_counter = 0

    def trace_id(self) -> str:
        self.trace_counter += 1
        return f"T_{self.trace_counter:03d}"

    def packet_id(self) -> str:
        self.packet_counter += 1
        return f"P_{self.packet_counter:03d}"


def packet(
    packet_id: str,
    trace_id: str,
    packet_type: str,
    source_engine: str,
    target_engine: str,
    priority: str,
    time_budget_ms: int,
    epistemics: dict,
    permissions: dict,
    payload: dict,
) -> dict:
    return {
        "header": {
            "packet_id": packet_id,
            "packet_type": packet_type,
            "schema_version": SCHEMA_VERSION,
            "source_engine": source_engine,
            "target_engine": target_engine,
            "trace_id": trace_id,
            "created_at": now(),
            "priority": priority,
            "time_budget_ms": time_budget_ms,
        },
        "epistemics": epistemics,
        "permissions": permissions,
        "payload": payload,
    }


def load_json(path: Path):
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def default_epistemics(
    confidence: float,
    uncertainty_type: str,
    epistemic_license: str,
    provenance: list[dict] | None = None,
    contradictions: list[dict] | None = None,
) -> dict:
    return {
        "confidence": confidence,
        "uncertainty_type": uncertainty_type,
        "epistemic_license": epistemic_license,
        "provenance": provenance or [],
        "contradictions": contradictions or [],
    }


def permissions(allowed_use: list[str], forbidden_use: list[str]) -> dict:
    overlap = set(allowed_use).intersection(forbidden_use)
    if overlap:
        raise ValueError(f"Permission use cannot be both allowed and forbidden: {sorted(overlap)}")
    return {"allowed_use": allowed_use, "forbidden_use": forbidden_use}


def can_use_packet(packet_item: dict, requested_use: str) -> bool:
    permission = packet_item["permissions"]
    license_name = packet_item["epistemics"]["epistemic_license"]
    if requested_use in permission["forbidden_use"]:
        return False
    if requested_use not in permission["allowed_use"]:
        return False
    return requested_use in LICENSE_RULES[license_name]


def require_packet_use(packet_item: dict, requested_use: str) -> None:
    if not can_use_packet(packet_item, requested_use):
        packet_type = packet_item["header"]["packet_type"]
        packet_id = packet_item["header"]["packet_id"]
        raise PermissionError(f"{packet_type} {packet_id} is not licensed for {requested_use}")


def load_scenario(name: str, allow_test_trusted: bool = True) -> dict:
    scenario = load_json(WORLD / "scenarios" / f"{name}.json")
    if not allow_test_trusted and scenario.get("replay_ledger_trust") == "test_trusted":
        raise PermissionError("production scenario loader rejects test_trusted replay ledgers")
    return scenario


def run(command: str, scenario: dict | None = None) -> list[dict]:
    ids = IdFactory()
    trace_id = ids.trace_id()
    broker = InProcessBroker()
    world = encode_world_state(load_json(WORLD / "world_state.json"))
    memory = GovernedMemory(WORLD)
    mutation_log = []
    rules = load_json(WORLD / "rules.json")
    scenario = scenario or {}
    if "weather" in scenario:
        world["weather"] = scenario["weather"]
    if "time_budget_minutes" in scenario:
        world["time_budget_minutes"] = scenario["time_budget_minutes"]
    for bridge_key, bridge_patch in scenario.get("bridge_overrides", {}).items():
        world["bridges"][bridge_key].update(bridge_patch)
    plans = load_json(WORLD / "plans.json")
    intent = parse_human_command(command)
    if intent.get("urgency") == "high":
        world["time_budget_minutes"] = min(world["time_budget_minutes"], 3)

    subscriptions = {
        "language_codec": ["SystemStatePacket"],
        "bus": ["IntentPacket", "BackpressureCommand"],
        "memory": ["RetrievalRequest", "ActionOutcome", "MemoryMutation"],
        "verifier": ["RetrievalResult", "EvidencePacket"],
        "planner": ["ContradictionPacket"],
        "action": ["PlanProposal", "ActionCommand"],
        "episodic_log": ["EpisodePacket", "MemoryMutation"],
        "audit": ["MemoryMutation"],
        "mutation_gateway": ["ClaimPacket", "HumanPromotionPacket", "PlanRegretPacket", "AttentionModeReviewPacket"],
    }
    for engine, packet_types in subscriptions.items():
        for packet_type in packet_types:
            broker.subscribe(engine, packet_type)

    def emit(
        packet_type: str,
        source_engine: str,
        target_engine: str,
        priority: str,
        time_budget_ms: int,
        epistemics: dict,
        permission: dict,
        payload: dict,
    ) -> dict:
        item = packet(
            ids.packet_id(),
            trace_id,
            packet_type,
            source_engine,
            target_engine,
            priority,
            time_budget_ms,
            epistemics,
            permission,
            payload,
        )
        assert_no_internal_prose_handoff(item)
        return broker.publish(item)

    def deliver(engine: str, expected_packet_id: str, disposition: str = "ack", reason: str = "") -> dict:
        item = broker.poll(engine)
        if item is None:
            raise RuntimeError(f"No packet available for {engine}")
        actual_packet_id = item["header"]["packet_id"]
        if actual_packet_id != expected_packet_id:
            raise RuntimeError(f"{engine} received {actual_packet_id}, expected {expected_packet_id}")
        if disposition == "ack":
            broker.ack(actual_packet_id)
        elif disposition == "defer":
            broker.defer(actual_packet_id, reason)
        elif disposition == "dead_letter":
            broker.dead_letter(actual_packet_id, reason)
        else:
            raise ValueError(f"Unknown disposition: {disposition}")
        return item

    if scenario.get("mutation_scenario"):
        return _run_mutation_scenario(
            scenario,
            trace_id,
            broker,
            memory,
            mutation_log,
            emit,
            deliver,
        )
    if scenario.get("contradiction_repair"):
        return _run_contradiction_repair_scenario(
            scenario,
            trace_id,
            broker,
            memory,
            mutation_log,
            emit,
            deliver,
        )
    if scenario.get("design_proposal"):
        return _run_design_proposal_scenario(
            scenario,
            trace_id,
            broker,
            mutation_log,
            emit,
            deliver,
        )

    system_packet = emit(
        "SystemStatePacket",
        "world_model",
        "language_codec",
        "P2",
        25,
        default_epistemics(0.86, "observed", "weak_premise", [{"source": "world_state.json"}]),
        permissions(
            ["retrieval", "planning_with_fallback", "human_explanation", "contradiction_detection"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        world,
    )
    deliver("language_codec", system_packet["header"]["packet_id"])
    intent_packet = emit(
        "IntentPacket",
        "language_codec",
        "bus",
        "P1",
        100,
        default_epistemics(0.74, "user_assertion", "hypothesis_only"),
        permissions(
            ["retrieval", "planning_with_fallback", "human_explanation", "contradiction_detection"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        intent,
    )
    deliver("bus", intent_packet["header"]["packet_id"])
    require_packet_use(intent_packet, "retrieval")
    retrieval_request = emit(
        "RetrievalRequest",
        "memory",
        "memory",
        "P2",
        75,
        default_epistemics(0.74, "derived", "hypothesis_only", [{"packet_id": intent_packet["header"]["packet_id"]}]),
        permissions(
            ["retrieval", "human_explanation"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        {"goal": intent["goal"], "preferred_bridge": intent["preferred_bridge"]},
    )
    deliver("memory", retrieval_request["header"]["packet_id"])
    require_packet_use(retrieval_request, "retrieval")
    retrieval = memory.retrieve(intent, world, rules)
    retrieval_packet = emit(
        "RetrievalResult",
        "memory",
        "verifier",
        "P2",
        150,
        default_epistemics(
            0.68,
            "memory_retrieval",
            "hypothesis_only",
            [
                {"source": "append_only_episodic_log"},
                {"source": "semantic_memory_graph"},
                {"source": "procedural_memory_store"},
                {"source": "contradiction_index"},
            ],
        ),
        permissions(
            ["planning_with_fallback", "human_explanation", "contradiction_detection"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        retrieval,
    )
    deliver("verifier", retrieval_packet["header"]["packet_id"])
    require_packet_use(retrieval_packet, "contradiction_detection")
    verification = verify_retrieval(retrieval, world)
    verifier_epistemics = default_epistemics(
        verification["confidence"],
        "rule_based",
        verification["epistemic_license"],
        [{"packet_id": retrieval_packet["header"]["packet_id"]}, {"packet_id": system_packet["header"]["packet_id"]}],
        verification["contradictions"],
    )
    for contradiction in verification["contradictions"]:
        contradiction_packet = emit(
            "ContradictionPacket",
            "verifier",
            "planner",
            "P3",
            50,
            default_epistemics(0.82, "rule_based", "hazard_only", [{"packet_id": retrieval_packet["header"]["packet_id"]}], [contradiction]),
            permissions(
                ["human_explanation", "contradiction_detection"],
                ["direct_action", "memory_consolidation", "rule_revision", "safety_certification", "planning"],
            ),
            contradiction,
        )
        deliver("planner", contradiction_packet["header"]["packet_id"])
    anomaly_count = int(scenario.get("interrupts", 0))
    low_level_anomalies = [
        {"subject": "Bridge A", "signal": "risk_increase", "sequence": index + 1}
        for index in range(anomaly_count)
    ]
    trend = coalesce_anomalies(low_level_anomalies)
    trend_packet = None
    if trend:
        trend_packet = emit(
            "SystemStatePacket",
            "attention",
            "verifier",
            "P3",
            40,
            default_epistemics(
                trend["confidence"],
                "derived",
                "hazard_only",
                [{"source_count": trend["source_count"]}],
            ),
            permissions(
                ["human_explanation", "contradiction_detection"],
                ["direct_action", "memory_consolidation", "rule_revision", "safety_certification", "planning"],
            ),
            trend,
        )

    queue_depth = anomaly_count + len(broker.published)
    max_admission_score = max(score_packet(item) for item in broker.published)
    system_mode = choose_mode(max_admission_score, queue_depth, world["time_budget_minutes"])
    attention = {
        "system_mode": system_mode,
        "mode_behavior": SYSTEM_MODES[system_mode],
        "budget_minutes": world["time_budget_minutes"],
        "queue_depth": queue_depth,
        "max_admission_score": max_admission_score,
        "deferred_jobs": ["semantic_consolidation"] if system_mode in {"Strained", "Emergency", "Reflex"} else [],
        "backpressure": memory_backpressure(system_mode, max_results=3),
    }
    backpressure_packet = emit(
        "BackpressureCommand",
        "attention",
        "bus",
        "P1",
        40,
        default_epistemics(0.91, "derived", "full_premise"),
        permissions(
            ["sandbox_testing", "human_explanation"],
            ["memory_consolidation", "rule_revision", "safety_certification"],
        ),
        attention,
    )
    deliver("bus", backpressure_packet["header"]["packet_id"])
    if scenario.get("attention_review"):
        _emit_attention_review(
            scenario["attention_review"],
            trace_id,
            attention,
            backpressure_packet,
            trend_packet,
            anomaly_count,
            mutation_log,
            emit,
            deliver,
        )

    if scenario.get("rule_change"):
        old_rule = next(rule for rule in rules if rule["id"] == scenario["rule_change"]["rule_id"])
        new_rule = next_rule_version(old_rule, scenario["rule_change"]["new_claim"], "scenario")
        cascade = evaluate_rule_change(
            old_rule,
            new_rule,
            memory.semantic_graph.all(),
            memory.procedural_store.all(),
            plans,
        )
        cascade_packet = emit(
            "MemoryMutation",
            "rule_cascade",
            "memory",
            "P4",
            110,
            default_epistemics(0.9, "derived", "full_premise", [{"old_rule_id": old_rule["id"]}]),
            permissions(
                ["human_explanation", "memory_consolidation"],
                ["direct_action", "safety_certification"],
            ),
            {
                "operation": "rule_version_cascade",
                "old_rule": old_rule,
                "new_rule": new_rule,
                "cascade": cascade,
            },
        )
        deliver("memory", cascade_packet["header"]["packet_id"])
        deliver("audit", cascade_packet["header"]["packet_id"])
        deliver(
            "episodic_log",
            cascade_packet["header"]["packet_id"],
            disposition="defer",
            reason="rule cascade queued for lazy evaluation",
        )
    require_packet_use(retrieval_packet, "planning_with_fallback")
    require_packet_use(system_packet, "planning_with_fallback")
    require_packet_use(intent_packet, "planning_with_fallback")
    risk_budget = float(scenario.get("risk_budget", 0.25 if intent.get("urgency") == "high" else 0.4 if world["weather"] == "heavy_rain" else 0.7))
    plan = build_plan(
        goal=intent,
        retrieved_memories=retrieval,
        epistemic_license=verification["epistemic_license"],
        world_state=world,
        time_budget_minutes=world["time_budget_minutes"],
        risk_budget=risk_budget,
        system_mode=system_mode,
    )
    plan_packet = emit(
        "PlanProposal",
        "planner",
        "action",
        "P1",
        120,
        {
            **verifier_epistemics,
            "provenance": [
                {"packet_id": intent_packet["header"]["packet_id"]},
                {"packet_id": retrieval_packet["header"]["packet_id"]},
                {"packet_id": system_packet["header"]["packet_id"]},
            ],
        },
        permissions(
            ["planning_with_fallback", "sandbox_testing", "human_explanation"],
            ["memory_consolidation", "rule_revision", "safety_certification"],
        ),
        plan,
    )
    deliver("action", plan_packet["header"]["packet_id"])
    require_packet_use(plan_packet, "sandbox_testing")
    command_payload = {
        "action": plan["action"],
        "route": plan["route"],
        "fallback_action": plan["fallback_plan"]["action"],
        "required_assumptions": plan["required_assumptions"],
    }
    command_packet = emit(
        "ActionCommand",
        "action",
        "action",
        "P1",
        80,
        {
            **verifier_epistemics,
            "provenance": [{"packet_id": plan_packet["header"]["packet_id"]}],
        },
        permissions(
            ["sandbox_testing", "human_explanation"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        command_payload,
    )
    deliver("action", command_packet["header"]["packet_id"])
    require_packet_use(command_packet, "sandbox_testing")
    outcome = execute_action(command_payload, world)
    if scenario.get("post_action_correction", {}).get("outcome_override"):
        outcome.update(scenario["post_action_correction"]["outcome_override"])
    if scenario.get("planner_regret", {}).get("outcome_override"):
        outcome.update(scenario["planner_regret"]["outcome_override"])
    outcome_packet = emit(
        "ActionOutcome",
        "action",
        "memory",
        "P2",
        60,
        default_epistemics(0.95, "simulation_result", "full_premise", [{"packet_id": command_packet["header"]["packet_id"]}]),
        permissions(
            ["memory_consolidation", "human_explanation", "sandbox_testing"],
            ["direct_action", "rule_revision", "safety_certification"],
        ),
        outcome,
    )
    deliver("memory", outcome_packet["header"]["packet_id"])
    require_packet_use(outcome_packet, "memory_consolidation")
    recorded = record_action_outcome(
        outcome,
        trace_id,
        command_packet["header"]["packet_id"],
        outcome_packet["header"]["packet_id"],
        now(),
    )
    episode_packet = emit(
        "EpisodePacket",
        "action",
        "episodic_log",
        "P4",
        90,
        default_epistemics(
            recorded["episode_packet"]["confidence"],
            "simulation_result",
            "full_premise",
            [{"packet_id": outcome_packet["header"]["packet_id"]}],
        ),
        permissions(
            ["memory_consolidation", "human_explanation"],
            ["direct_action", "rule_revision", "safety_certification"],
        ),
        {
            **recorded["episode_packet"],
            "trace_link": recorded["trace_link"],
        },
    )
    deliver("episodic_log", episode_packet["header"]["packet_id"])
    mutation = {
        **recorded["memory_update_candidate"],
        "episode": memory.episodic_log.append(**recorded["episode_packet"]),
        "trace_link": recorded["trace_link"],
    }
    mutation_packet = emit(
        "MemoryMutation",
        "memory",
        "episodic_log",
        "P4",
        100,
        default_epistemics(0.88, "derived", "full_premise", [{"packet_id": outcome_packet["header"]["packet_id"]}]),
        permissions(
            ["human_explanation"],
            ["direct_action", "rule_revision", "safety_certification"],
        ),
        mutation,
    )
    deliver("memory", mutation_packet["header"]["packet_id"])
    deliver("episodic_log", mutation_packet["header"]["packet_id"])
    deliver("audit", mutation_packet["header"]["packet_id"])
    if retrieval_has_degraded_action_support(retrieval):
        revalidation_packet = emit(
            "BackpressureCommand",
            "memory",
            "bus",
            "P4",
            90,
            default_epistemics(0.9, "derived", "full_premise", [{"packet_id": outcome_packet["header"]["packet_id"]}]),
            permissions(
                ["human_explanation", "memory_consolidation"],
                ["direct_action", "rule_revision", "safety_certification"],
            ),
            {
                "type": "post_action_revalidation",
                "reason": "action_used_degraded_memory",
                "after_action": outcome_packet["header"]["packet_id"],
                "trace_id": trace_id,
                "degraded_sources": [
                    item["content"].get("memory_id", item["content"].get("episode_id", item["content"].get("procedure_id")))
                    for group in ("episodes", "semantic_nodes", "procedures")
                    for item in retrieval[group]
                    if item["revalidation_requirement"] == "post_action_revalidation"
                ],
            },
        )
        deliver("bus", revalidation_packet["header"]["packet_id"])
        if scenario.get("false_alarm_revalidation"):
            repair = scenario["false_alarm_revalidation"]
            target = memory.semantic_graph.get(repair["memory_id"])
            verifier_decision = verifier_allows_mutation(
                "V_DEC_false_alarm_revalidation",
                "semantic_status_update",
                "memory_consolidation",
                repair["memory_id"],
                revalidation_packet["header"]["packet_id"],
            )
            mutation_request = {
                "mutation_id": "MUT_false_alarm_revalidation",
                "trace_id": trace_id,
                "source_packet_id": revalidation_packet["header"]["packet_id"],
                "verifier_decision_id": verifier_decision["verifier_decision_id"],
                "target_object_id": repair["memory_id"],
                "requested_use": "memory_consolidation",
                "mutation_type": "semantic_status_update",
                "new_status": repair["new_status"],
                "authority_snapshot": {"forbidden_use": []},
            }
            mutation_result = apply_memory_mutation(
                mutation_request,
                target,
                revalidation_packet,
                verifier_decision,
                mutation_log,
                memory=memory,
            )
            repair_packet = emit(
                "MemoryMutation",
                "memory",
                "semantic_graph",
                "P4",
                100,
                default_epistemics(0.91, "derived", "full_premise", [{"packet_id": revalidation_packet["header"]["packet_id"]}]),
                permissions(
                    ["human_explanation", "memory_consolidation"],
                    ["direct_action", "rule_revision", "safety_certification"],
                ),
                {
                    "operation": "semantic_status_update",
                    "mutation_id": mutation_request["mutation_id"],
                    "memory_id": repair["memory_id"],
                    "old_status": repair["old_status"],
                    "new_status": repair["new_status"],
                    "reason": repair["reason"],
                    "effect": "Bridge A damage report no longer permanently hazard-only.",
                    "trace_id": trace_id,
                    "verifier_decision": verifier_decision,
                    "mutation_log_entry": mutation_result["log"],
                    "applied": mutation_result["applied"],
                },
            )
            deliver("memory", repair_packet["header"]["packet_id"])
            deliver("audit", repair_packet["header"]["packet_id"])
        if scenario.get("post_action_correction"):
            _emit_post_action_corrections(
                scenario["post_action_correction"],
                trace_id,
                outcome_packet,
                memory,
                mutation_log,
                emit,
                deliver,
            )
    if scenario.get("planner_regret"):
        _emit_planner_regret(
            scenario["planner_regret"],
            trace_id,
            plan,
            plan_packet,
            outcome_packet,
            plans,
            mutation_log,
            emit,
            deliver,
        )
    return broker.trace_for(trace_id)


def _attention_policy_object() -> dict:
    return {
        "attention_policy_id": "ATTN_bridge_world_v0_1",
        "status": "active_attention_policy",
        "confidence": 0.64,
        "mode_thresholds": MODE_THRESHOLDS,
        "mode_policy": {
            "Reflex": "precompiled_policy_only",
            "Emergency": "minimax_safety_only",
            "Strained": "defer_consolidation",
            "Recovery": "replay_deferred_packets",
        },
        "coalescing_policy": {"low_value_signal_threshold": 3},
        "backpressure_policy": {"preserve": ["P0", "P1"], "defer_below": "P4"},
    }


def _emit_attention_review(
    review: dict,
    trace_id: str,
    attention: dict,
    attention_packet: dict,
    trend_packet: dict | None,
    raw_interrupt_count: int,
    mutation_log: list[dict],
    emit,
    deliver,
) -> None:
    policy = _attention_policy_object()
    review_packet = emit(
        "AttentionModeReviewPacket",
        "attention_review",
        "mutation_gateway",
        "P4",
        100,
        default_epistemics(
            review["confidence"],
            "derived",
            "weak_premise",
            [{"packet_id": attention_packet["header"]["packet_id"]}],
        ),
        permissions(
            ["human_explanation", "attention_policy_update"],
            ["direct_action", "memory_consolidation", "planner_policy_update", "rule_revision", "safety_certification"],
        ),
        {
            "type": "AttentionModeReviewPacket",
            "review_id": review["review_id"],
            "observed_mode": attention["system_mode"],
            "classification": review["classification"],
            "target_policy_id": policy["attention_policy_id"],
            "review_required": review["review_required"],
            "raw_packet_preservation": {
                "raw_packet_count": raw_interrupt_count,
                "storage": "attention_raw_interrupt_buffer",
                "preserved": True,
            },
            "coalescing": {
                "coalesced": trend_packet is not None,
                "source_count": trend_packet["payload"]["source_count"] if trend_packet else 0,
            },
            "priority_preservation": {
                "kept_alive": review.get("kept_alive", ["P0", "P1"]),
                "deferred_priorities": review.get("deferred_priorities", ["P4", "P5", "P6"]),
            },
            "recovery_replay": review.get("recovery_replay", {"mode": "Recovery", "processed_deferred_jobs": []}),
            "scope_conditions": review.get("scope_conditions", {}),
            "policy_update": review["policy_update"],
            "doctrine": "Attention is policy. Policy can be wrong. Attention correction must not become authority correction.",
        },
    )
    deliver("mutation_gateway", review_packet["header"]["packet_id"])
    review_work_packet = emit(
        "BackpressureCommand",
        "attention_review",
        "bus",
        "P4",
        80,
        default_epistemics(0.86, "derived", "weak_premise", [{"packet_id": review_packet["header"]["packet_id"]}]),
        permissions(
            ["human_explanation", "attention_policy_update"],
            ["direct_action", "memory_consolidation", "planner_policy_update", "rule_revision", "safety_certification"],
        ),
        {
            "type": "attention_mode_review",
            "review_id": review["review_id"],
            "classification": review["classification"],
            "target_policy_id": policy["attention_policy_id"],
            "review_required": review["review_required"],
            "status": "open" if review["review_required"] else "not_required",
            "deferred": False,
            "reason": review["review_reason"],
        },
    )
    deliver("bus", review_work_packet["header"]["packet_id"])
    verifier_decision = verifier_allows_mutation(
        f"V_DEC_{review['review_id']}",
        "attention_policy_update",
        "attention_policy_update",
        policy["attention_policy_id"],
        review_packet["header"]["packet_id"],
        review["verifier_rule_id"],
    )
    request = {
        "mutation_id": f"MUT_{review['review_id']}",
        "trace_id": trace_id,
        "source_packet_id": review_packet["header"]["packet_id"],
        "verifier_decision_id": verifier_decision["verifier_decision_id"],
        "target_object_id": policy["attention_policy_id"],
        "requested_use": "attention_policy_update",
        "mutation_type": "attention_policy_update",
        "patch": review["policy_update"],
        "authority_snapshot": {"forbidden_use": []},
    }
    result = apply_memory_mutation(request, policy, review_packet, verifier_decision, mutation_log)
    mutation_packet = emit(
        "MemoryMutation",
        "attention_review",
        "audit",
        "P4",
        100,
        default_epistemics(0.84, "derived", "weak_premise", [{"packet_id": review_packet["header"]["packet_id"]}]),
        permissions(
            ["human_explanation", "attention_policy_update"],
            ["direct_action", "memory_consolidation", "planner_policy_update", "rule_revision", "safety_certification"],
        ),
        {
            "operation": "attention_mode_policy_update",
            "review_id": review["review_id"],
            "classification": review["classification"],
            "review_required": review["review_required"],
            "review_reason": review["review_reason"],
            "target_kind": "attention_policy",
            "policy_update_kind": review["policy_update_kind"],
            "memory_authority_changed": False,
            "procedure_authority_changed": False,
            "planner_authority_changed": False,
            "verifier_rule_changed": False,
            "scope_conditions": review.get("scope_conditions", {}),
            "raw_packet_preservation": review_packet["payload"]["raw_packet_preservation"],
            "coalescing": review_packet["payload"]["coalescing"],
            "priority_preservation": review_packet["payload"]["priority_preservation"],
            "recovery_replay": review_packet["payload"]["recovery_replay"],
            "verifier_decision": verifier_decision,
            "verifier_rule_id": review["verifier_rule_id"],
            "mutation_request": request,
            "target_before": policy,
            "target_after": result["target"],
            "mutation_log_entry": result["log"],
            "mutation_log": mutation_log,
        },
    )
    deliver("memory", mutation_packet["header"]["packet_id"])
    deliver(
        "episodic_log",
        mutation_packet["header"]["packet_id"],
        disposition="defer",
        reason="attention policy review queued outside episodic memory mutation",
    )
    deliver("audit", mutation_packet["header"]["packet_id"])


def _emit_planner_regret(
    regret: dict,
    trace_id: str,
    plan: dict,
    plan_packet: dict,
    outcome_packet: dict,
    plans: list[dict],
    mutation_log: list[dict],
    emit,
    deliver,
) -> None:
    target_policy = dict(next(item for item in plans if item["plan_id"] == regret["target_policy_id"]))
    target_policy.setdefault("status", "active_planner_policy")
    target_policy.setdefault("confidence", 0.62)
    regret_packet = emit(
        "PlanRegretPacket",
        "planner_regret",
        "mutation_gateway",
        "P4",
        100,
        default_epistemics(
            regret["confidence"],
            "simulation_result",
            "weak_premise",
            [{"packet_id": plan_packet["header"]["packet_id"]}, {"packet_id": outcome_packet["header"]["packet_id"]}],
        ),
        permissions(
            ["human_explanation", "planner_policy_update"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        {
            "type": "PlanRegretPacket",
            "regret_id": regret["regret_id"],
            "regret_type": regret["regret_type"],
            "regret_class": regret["regret_class"],
            "expected": regret["expected"],
            "actual": regret["actual"],
            "selected_action": plan["action"],
            "selected_route": plan["route"],
            "target_policy_id": regret["target_policy_id"],
            "review_required": regret["review_required"],
            "scope_conditions": regret.get("scope_conditions", {}),
            "policy_update": regret["policy_update"],
            "doctrine": "An outcome does not only update memory. It also evaluates the policy that chose the action.",
        },
    )
    deliver("mutation_gateway", regret_packet["header"]["packet_id"])
    review_packet = emit(
        "BackpressureCommand",
        "planner_regret",
        "bus",
        "P4",
        80,
        default_epistemics(0.86, "derived", "weak_premise", [{"packet_id": regret_packet["header"]["packet_id"]}]),
        permissions(
            ["human_explanation", "planner_policy_update"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        {
            "type": "planner_review",
            "regret_id": regret["regret_id"],
            "regret_type": regret["regret_type"],
            "regret_class": regret["regret_class"],
            "target_policy_id": regret["target_policy_id"],
            "review_required": regret["review_required"],
            "status": "open" if regret["review_required"] else "not_required",
            "deferred": False,
            "reason": regret["review_reason"],
        },
    )
    deliver("bus", review_packet["header"]["packet_id"])
    verifier_decision = verifier_allows_mutation(
        f"V_DEC_{regret['regret_id']}",
        "planner_policy_update",
        "planner_policy_update",
        regret["target_policy_id"],
        regret_packet["header"]["packet_id"],
        regret["verifier_rule_id"],
    )
    request = {
        "mutation_id": f"MUT_{regret['regret_id']}",
        "trace_id": trace_id,
        "source_packet_id": regret_packet["header"]["packet_id"],
        "verifier_decision_id": verifier_decision["verifier_decision_id"],
        "target_object_id": regret["target_policy_id"],
        "requested_use": "planner_policy_update",
        "mutation_type": "planner_policy_update",
        "patch": regret["policy_update"],
        "authority_snapshot": {"forbidden_use": []},
    }
    result = apply_memory_mutation(request, target_policy, regret_packet, verifier_decision, mutation_log)
    mutation_packet = emit(
        "MemoryMutation",
        "planner_regret",
        "audit",
        "P4",
        100,
        default_epistemics(0.84, "derived", "weak_premise", [{"packet_id": regret_packet["header"]["packet_id"]}]),
        permissions(
            ["human_explanation", "planner_policy_update"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        {
            "operation": "planner_regret_policy_update",
            "regret_id": regret["regret_id"],
            "regret_type": regret["regret_type"],
            "regret_class": regret["regret_class"],
            "review_required": regret["review_required"],
            "review_reason": regret["review_reason"],
            "scope_conditions": regret.get("scope_conditions", {}),
            "target_kind": "planner_policy",
            "policy_update_kind": regret["policy_update_kind"],
            "global_rule_rewrite": False,
            "belief_or_procedure_authority_changed": False,
            "verifier_decision": verifier_decision,
            "verifier_rule_id": regret["verifier_rule_id"],
            "mutation_request": request,
            "target_before": target_policy,
            "target_after": result["target"],
            "mutation_log_entry": result["log"],
            "mutation_log": mutation_log,
        },
    )
    deliver("audit", mutation_packet["header"]["packet_id"])


def _emit_post_action_corrections(
    correction: dict,
    trace_id: str,
    outcome_packet: dict,
    memory: GovernedMemory,
    mutation_log: list[dict],
    emit,
    deliver,
) -> None:
    for index, mutation in enumerate(correction["mutations"], start=1):
        target_id = mutation["target_object_id"]
        if mutation["mutation_type"] == "procedure_status_update":
            target = memory.procedural_store.get(target_id)
        else:
            target = memory.semantic_graph.get(target_id)
        verifier_decision = verifier_allows_mutation(
            f"V_DEC_{correction['id']}_{index}",
            mutation["mutation_type"],
            "memory_consolidation",
            target_id,
            outcome_packet["header"]["packet_id"],
            correction["verifier_rule_id"],
        )
        request = {
            "mutation_id": f"MUT_{correction['id']}_{index}",
            "trace_id": trace_id,
            "source_packet_id": outcome_packet["header"]["packet_id"],
            "verifier_decision_id": verifier_decision["verifier_decision_id"],
            "target_object_id": target_id,
            "requested_use": "memory_consolidation",
            "mutation_type": mutation["mutation_type"],
            "new_status": mutation["new_status"],
            "authority_snapshot": mutation.get("authority_snapshot", {"forbidden_use": []}),
        }
        result = apply_memory_mutation(request, target, outcome_packet, verifier_decision, mutation_log, memory=memory)
        packet_item = emit(
            "MemoryMutation",
            "post_action_revalidation",
            "audit",
            "P4",
            100,
            default_epistemics(
                0.86,
                "derived",
                "weak_premise",
                [{"packet_id": outcome_packet["header"]["packet_id"]}],
            ),
            permissions(
                ["human_explanation", "memory_consolidation"],
                ["direct_action", "rule_revision", "safety_certification"],
            ),
            {
                "operation": "post_action_correction",
                "correction_id": correction["id"],
                "doctrine": "Outcome is evidence, not proof.",
                "belief_update_position": mutation["belief_update_position"],
                "target_kind": "procedure" if mutation["mutation_type"] == "procedure_status_update" else "belief",
                "scope_conditions": mutation.get("scope_conditions", {}),
                "overconfirmation_blocked": correction.get("overconfirmation_blocked", True),
                "mutation_request": request,
                "verifier_decision": verifier_decision,
                "target_before": target,
                "target_after": result["target"],
                "mutation_log_entry": result["log"],
                "mutation_log": mutation_log,
            },
        )
        deliver("audit", packet_item["header"]["packet_id"])


def _run_mutation_scenario(
    scenario: dict,
    trace_id: str,
    broker: InProcessBroker,
    memory: GovernedMemory,
    mutation_log: list[dict],
    emit,
    deliver,
) -> list[dict]:
    config = scenario["mutation_scenario"]
    target_id = config["target_object_id"]
    if config["mutation_type"] == "bootstrap_promotion":
        from bootstrap_ingest import ingest_design_history

        target = ingest_design_history(config["bootstrap_claim"], "sprint_10_scenario.md")[0]
        target["memory_id"] = target_id
    else:
        target = memory.semantic_graph.get(target_id)

    source_packet = None
    if config.get("source_packet"):
        source = config["source_packet"]
        source_packet = emit(
            source.get("packet_type", "ClaimPacket"),
            source["source_engine"],
            "mutation_gateway",
            "P2",
            80,
            default_epistemics(
                source["confidence"],
                "derived",
                source["epistemic_license"],
                [{"scenario": scenario["name"]}],
            ),
            permissions(source["allowed_use"], source["forbidden_use"]),
            source["payload"],
        )
        deliver("mutation_gateway", source_packet["header"]["packet_id"])

    verifier_decision = None
    if config.get("verifier_decision") == "allow":
        verifier_decision = verifier_allows_mutation(
            config["verifier_decision_id"],
            config["mutation_type"],
            config["requested_use"],
            target_id,
            source_packet["header"]["packet_id"],
            config.get("verifier_rule_id", MUTATION_AUTHORITY_RULE),
        )
    elif config.get("verifier_decision") == "block":
        verifier_decision = verifier_blocks_mutation(
            config["verifier_decision_id"],
            config["mutation_type"],
            config["requested_use"],
            target_id,
            source_packet["header"]["packet_id"] if source_packet else None,
            config.get("block_reason", "mutation authority denied"),
        )

    mutation_request = {
        "mutation_id": config["mutation_id"],
        "trace_id": trace_id,
        "source_packet_id": source_packet["header"]["packet_id"] if source_packet else config.get("source_packet_id"),
        "verifier_decision_id": config.get("verifier_decision_id"),
        "target_object_id": target_id,
        "requested_use": config["requested_use"],
        "mutation_type": config["mutation_type"],
        "new_status": config.get("new_status"),
        "authority_snapshot": config.get("authority_snapshot", {"forbidden_use": []}),
    }
    before_target = target
    mutation_result = apply_memory_mutation(
        mutation_request,
        before_target,
        source_packet,
        verifier_decision,
        mutation_log,
        memory=memory,
    )
    mutation_packet = emit(
        "MemoryMutation",
        "mutation_gateway",
        "audit",
        "P3" if not mutation_result["applied"] else "P2",
        100,
        default_epistemics(
            0.95 if mutation_result["applied"] else 0.9,
            "rule_based",
            "full_premise",
            [{"packet_id": source_packet["header"]["packet_id"]}] if source_packet else [{"scenario": scenario["name"]}],
        ),
        permissions(
            ["human_explanation", "memory_consolidation"],
            ["direct_action", "rule_revision", "safety_certification"],
        ),
        {
            "operation": "mutation_applied" if mutation_result["applied"] else "mutation_rejected",
            "mutation_request": mutation_request,
            "source_packet_type": config.get("source_packet", {}).get("packet_type"),
            "verifier_decision": verifier_decision,
            "target_before": before_target,
            "target_after": mutation_result["target"],
            "mutation_log_entry": mutation_result["log"],
            "mutation_log": mutation_log,
        },
    )
    deliver("audit", mutation_packet["header"]["packet_id"])
    return broker.trace_for(trace_id)


def _run_contradiction_repair_scenario(
    scenario: dict,
    trace_id: str,
    broker: InProcessBroker,
    memory: GovernedMemory,
    mutation_log: list[dict],
    emit,
    deliver,
) -> list[dict]:
    repair = scenario["contradiction_repair"]
    world = encode_world_state(load_json(WORLD / "world_state.json"))
    rules = load_json(WORLD / "rules.json")
    intent = parse_human_command(scenario["command"])
    raw_episode_count_before = len(memory.episodic_log.all())
    source_evidence_packets = []
    for evidence in repair.get("source_evidence", []):
        if not isinstance(evidence, dict):
            continue
        evidence_packet = emit(
            "EvidencePacket",
            evidence["source"],
            "verifier",
            "P2",
            80,
            default_epistemics(
                0.93,
                "observed",
                evidence["authority"],
                [{"episode_id": evidence["episode_id"]}],
            ),
            permissions(
                ["human_explanation", "contradiction_detection", "memory_consolidation"],
                ["direct_action", "rule_revision", "safety_certification"],
            ),
            evidence,
        )
        deliver("verifier", evidence_packet["header"]["packet_id"])
        source_evidence_packets.append(evidence_packet)

    repair_job_packet = emit(
        "BackpressureCommand",
        "verifier",
        "bus",
        "P3",
        100,
        default_epistemics(
            0.9,
            "rule_based",
            "full_premise",
            [{"packet_id": packet_item["header"]["packet_id"]} for packet_item in source_evidence_packets],
        ),
        permissions(
            ["human_explanation", "contradiction_detection", "memory_consolidation"],
            ["direct_action", "rule_revision", "safety_certification"],
        ),
        {
            "type": "contradiction_repair",
            "repair_id": repair["repair_id"],
            "repair_type": repair["repair_type"],
            "source_contradiction": repair["source_contradiction"],
            "target_nodes": repair["target_nodes"],
            "decision": repair["decision"],
            "verifier_rule_id": repair["verifier_rule_id"],
            "source_evidence": repair.get("source_evidence", []),
        },
    )
    deliver("bus", repair_job_packet["header"]["packet_id"])

    mutation_ids = []
    for index, mutation in enumerate(repair["mutations"], start=1):
        target = memory.semantic_graph.get(mutation["target_object_id"])
        verifier_decision = verifier_allows_mutation(
            f"V_DEC_{repair['repair_id']}_{index}",
            mutation["mutation_type"],
            "memory_consolidation",
            mutation["target_object_id"],
            repair_job_packet["header"]["packet_id"],
            repair["verifier_rule_id"],
        )
        request = {
            "mutation_id": f"MUT_{repair['repair_id']}_{index}",
            "trace_id": trace_id,
            "source_packet_id": repair_job_packet["header"]["packet_id"],
            "verifier_decision_id": verifier_decision["verifier_decision_id"],
            "target_object_id": mutation["target_object_id"],
            "requested_use": "memory_consolidation",
            "mutation_type": mutation["mutation_type"],
            "new_status": mutation["new_status"],
            "patch": mutation.get("patch", {}),
            "authority_snapshot": mutation.get("authority_snapshot", {"forbidden_use": []}),
        }
        result = apply_memory_mutation(request, target, repair_job_packet, verifier_decision, mutation_log, memory=memory)
        mutation_ids.append(request["mutation_id"])
        repair_packet = emit(
            "MemoryMutation",
            "contradiction_repair",
            "audit",
            "P3",
            100,
            default_epistemics(
                0.88,
                "rule_based",
                "weak_premise",
                [{"packet_id": repair_job_packet["header"]["packet_id"]}],
            ),
            permissions(
                ["human_explanation", "memory_consolidation"],
                ["direct_action", "rule_revision", "safety_certification"],
            ),
            {
                "operation": "contradiction_repair",
                "repair_id": repair["repair_id"],
                "repair_type": repair["repair_type"],
                "source_contradiction": repair["source_contradiction"],
                "target_nodes": repair["target_nodes"],
                "repair_decision": repair["decision"],
                "source_evidence": repair.get("source_evidence", []),
                "verifier_decision": verifier_decision,
                "verifier_rule_id": repair["verifier_rule_id"],
                "scope_conditions": mutation.get("patch", {}).get("scope_conditions", repair.get("scope_conditions", {})),
                "strict_action_blocked": repair.get("strict_action_blocked", False),
                "raw_episodes_preserved": repair["raw_episodes_preserved"],
                "raw_episode_count_before": raw_episode_count_before,
                "raw_episode_count_after": len(memory.episodic_log.all()),
                "repair_result": {
                    "repair_id": repair["repair_id"],
                    "repair_type": repair["repair_type"],
                    "source_contradiction": repair["source_contradiction"],
                    "target_nodes": repair["target_nodes"],
                    "decision": repair["decision"],
                    "source_evidence": repair.get("source_evidence", []),
                    "verifier_rule_id": repair["verifier_rule_id"],
                    "mutations": mutation_ids,
                },
                "mutation_request": request,
                "target_before": target,
                "target_after": result["target"],
                "mutation_log_entry": result["log"],
                "mutation_log": mutation_log,
            },
        )
        deliver("audit", repair_packet["header"]["packet_id"])
    post_repair_retrieval = memory.retrieve(intent, world, rules)
    retrieval_packet = emit(
        "RetrievalResult",
        "memory",
        "verifier",
        "P3",
        120,
        default_epistemics(
            0.72,
            "memory_retrieval",
            "hypothesis_only" if repair["repair_type"] == "unresolved" else "weak_premise",
            [{"repair_id": repair["repair_id"]}],
        ),
        permissions(
            ["human_explanation", "contradiction_detection", "planning_with_fallback"],
            ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
        ),
        {
            "retrieval_after_repair": True,
            "repair_id": repair["repair_id"],
            "repair_type": repair["repair_type"],
            **post_repair_retrieval,
        },
    )
    deliver("verifier", retrieval_packet["header"]["packet_id"])
    return broker.trace_for(trace_id)


def _run_design_proposal_scenario(
    scenario: dict,
    trace_id: str,
    broker: InProcessBroker,
    mutation_log: list[dict],
    emit,
    deliver,
) -> list[dict]:
    """The Caitlin leap: a design proposal is governed by the same bus, verifier,
    licenses, contradiction packets, mutation gateway, and correction loop as a
    bridge decision. A proposal that would weaken a locked invariant is blocked
    exactly as Bridge A is blocked under hazard_only evidence."""
    from project_self_audit import evaluate_design_proposal, load_design_memory

    config = scenario["design_proposal"]
    design_memory = load_design_memory()
    invariant = next(
        (inv for inv in design_memory["invariants"] if inv["memory_id"] == config["targets_invariant"]),
        None,
    )

    # 1. Retrieve the targeted design invariant. Design memory is queried like runtime memory.
    request_packet = emit(
        "RetrievalRequest",
        "planner",
        "memory",
        "P2",
        80,
        default_epistemics(0.9, "derived", "full_premise", [{"scenario": scenario["name"]}]),
        permissions(["design_discussion", "human_review"], ["direct_action", "memory_consolidation"]),
        {
            "query": config["targets_invariant"],
            "kind": "design_invariant_lookup",
            "proposal_id": config["proposal_id"],
        },
    )
    deliver("memory", request_packet["header"]["packet_id"])

    # 2. RetrievalResult surfaces the invariant with license + provenance: never a naked fact.
    invariant_license = invariant["epistemic_license"] if invariant else "hypothesis_only"
    retrieval_packet = emit(
        "RetrievalResult",
        "memory",
        "verifier",
        "P2",
        100,
        default_epistemics(0.9, "memory_retrieval", invariant_license, [{"design_memory": "design_memory.json"}]),
        permissions(
            ["design_discussion", "human_review", "contradiction_detection"],
            ["direct_action", "runtime_action"],
        ),
        {"design_invariant": invariant, "proposal_id": config["proposal_id"], "naked_fact": False},
    )
    deliver("verifier", retrieval_packet["header"]["packet_id"])

    # 3. Evaluate the proposal against the locked invariant (re-uses the runtime adjudicator).
    decision = evaluate_design_proposal(config, invariant)

    contradiction_packet = None
    if decision["contradiction_detected"]:
        contradiction_packet = emit(
            "ContradictionPacket",
            "verifier",
            "planner",
            "P1",
            80,
            default_epistemics(
                0.9,
                "rule_based",
                decision["contradiction_license"],
                [{"packet_id": retrieval_packet["header"]["packet_id"]}],
            ),
            permissions(
                ["human_explanation", "contradiction_detection"],
                ["direct_action", "rule_revision", "release_invariant", "memory_consolidation"],
            ),
            {
                "subject": config["targets_invariant"],
                "proposal_id": config["proposal_id"],
                "conflict_type": decision["conflict_type"],
                "verifier_rule_id": decision["verifier_rule_id"],
                "adjudication": decision["adjudication"],
                "effect": decision["derived_effect"],
                "declared_effect": decision["declared_effect"],
                "derived_effect": decision["derived_effect"],
                "lexical_effect": decision["lexical_effect"],
                "trace_effect": decision["trace_effect"],
                "trace_tested": decision["trace_tested"],
                "trace_regressed": decision["trace_regressed"],
                "trace_pre": decision["trace_pre"],
                "trace_post": decision["trace_post"],
                "trace_provenance": decision["trace_provenance"],
                "changed_artifact": decision["changed_artifact"],
                "delta_matches_change_set": decision["delta_matches_change_set"],
                "effect_authority": decision["effect_authority"],
                "effect_mislabel": decision["effect_mislabel"],
                "effect_basis": decision["effect_basis"],
                "reason": (
                    f"Design proposal {config['proposal_id']} would {decision['derived_effect']} "
                    f"locked invariant {config['targets_invariant']} "
                    f"(authority={decision['effect_authority']}, lexical={decision['lexical_effect']}, "
                    f"trace={decision['trace_effect']}, regressed={decision['trace_regressed']}, "
                    f"declared={decision['declared_effect']}, mislabel={decision['effect_mislabel']})."
                ),
            },
        )
        deliver("planner", contradiction_packet["header"]["packet_id"])

    # 4. PlanProposal carries the governance decision: reject/require-exception vs accept.
    governance_route = "reject_or_require_exception" if decision["contradiction_detected"] else "accept"
    upstream = contradiction_packet or retrieval_packet
    plan_packet = emit(
        "PlanProposal",
        "planner",
        "action",
        "P2",
        120,
        default_epistemics(0.85, "rule_based", "weak_premise", [{"packet_id": upstream["header"]["packet_id"]}]),
        permissions(["human_explanation", "design_discussion"], ["direct_action", "safety_certification"]),
        {
            "mode": "design_governance",
            "route": governance_route,
            "proposal_id": config["proposal_id"],
            "decision": "block" if decision["contradiction_detected"] else "accept",
            "blocks_release": decision["blocks_release"],
            "verifier_rule_id": decision["verifier_rule_id"],
            "derived_effect": decision["derived_effect"],
            "declared_effect": decision["declared_effect"],
            "lexical_effect": decision["lexical_effect"],
            "trace_effect": decision["trace_effect"],
            "trace_tested": decision["trace_tested"],
            "trace_regressed": decision["trace_regressed"],
            "trace_provenance": decision["trace_provenance"],
            "changed_artifact": decision["changed_artifact"],
            "delta_matches_change_set": decision["delta_matches_change_set"],
            "effect_authority": decision["effect_authority"],
            "effect_mislabel": decision["effect_mislabel"],
        },
    )
    deliver("action", plan_packet["header"]["packet_id"])

    # 5. Route the consolidation attempt through the REAL mutation gateway.
    #    block decision -> proposal cannot become authoritative; the invariant is preserved.
    proposal_candidate = {
        "memory_id": config["proposal_id"],
        "claim": config["claim"],
        "source": config.get("source", "design_proposal"),
        "authority_class": "bootstrap_candidate",
        "inspection_view": "bootstrap_candidates",
        "confidence": 0.25,
        "status": "pending_human_promotion",
        "epistemic_license": "hypothesis_only",
        "allowed_use": ["human_review", "design_discussion"],
        "forbidden_use": ["runtime_action", "release_invariant", "memory_consolidation"],
    }
    source_packet = emit(
        "ClaimPacket",
        "design_author",
        "mutation_gateway",
        "P2",
        80,
        default_epistemics(0.6, "derived", "hypothesis_only", [{"proposal_id": config["proposal_id"]}]),
        permissions(["design_consolidation", "human_review"], ["direct_action", "safety_certification"]),
        {"type": "DesignProposalPacket", "proposal_id": config["proposal_id"], "claim": config["claim"]},
    )
    deliver("mutation_gateway", source_packet["header"]["packet_id"])

    requested_use = config.get("requested_use", "design_consolidation")
    if decision["contradiction_detected"]:
        verifier_decision = verifier_blocks_mutation(
            f"V_DEC_{config['proposal_id']}",
            "bootstrap_promotion",
            requested_use,
            config["proposal_id"],
            source_packet["header"]["packet_id"],
            f"design proposal would {decision['derived_effect']} locked invariant {config['targets_invariant']}",
        )
    else:
        verifier_decision = verifier_allows_mutation(
            f"V_DEC_{config['proposal_id']}",
            "bootstrap_promotion",
            requested_use,
            config["proposal_id"],
            source_packet["header"]["packet_id"],
        )
    mutation_request = {
        "mutation_id": config.get("mutation_id", f"MUT_{config['proposal_id']}"),
        "trace_id": trace_id,
        "source_packet_id": source_packet["header"]["packet_id"],
        "verifier_decision_id": verifier_decision["verifier_decision_id"],
        "target_object_id": config["proposal_id"],
        "requested_use": requested_use,
        "mutation_type": "bootstrap_promotion",
        "authority_snapshot": {"forbidden_use": []},
    }
    mutation_result = apply_memory_mutation(
        mutation_request,
        proposal_candidate,
        source_packet,
        verifier_decision,
        mutation_log,
    )
    mutation_packet = emit(
        "MemoryMutation",
        "mutation_gateway",
        "audit",
        "P3",
        100,
        default_epistemics(
            0.9,
            "rule_based",
            "full_premise",
            [{"packet_id": source_packet["header"]["packet_id"]}],
        ),
        permissions(
            ["human_explanation", "memory_consolidation"],
            ["direct_action", "rule_revision", "safety_certification"],
        ),
        {
            "operation": "mutation_applied" if mutation_result["applied"] else "mutation_rejected",
            "proposal_id": config["proposal_id"],
            "targets_invariant": config["targets_invariant"],
            "effect": decision["derived_effect"],
            "declared_effect": decision["declared_effect"],
            "derived_effect": decision["derived_effect"],
            "lexical_effect": decision["lexical_effect"],
            "trace_effect": decision["trace_effect"],
            "trace_tested": decision["trace_tested"],
            "trace_regressed": decision["trace_regressed"],
            "trace_pre": decision["trace_pre"],
            "trace_post": decision["trace_post"],
            "trace_provenance": decision["trace_provenance"],
            "mechanism_source": decision["mechanism_source"],
            "mechanism_role": decision["mechanism_role"],
            "changed_artifact": decision["changed_artifact"],
            "pre_image_hash": decision["pre_image_hash"],
            "post_image_hash": decision["post_image_hash"],
            "diff_digest": decision["diff_digest"],
            "delta_matches_change_set": decision["delta_matches_change_set"],
            "signer": decision["signer"],
            "signature_status": decision["signature_status"],
            "signed_payload_digest": decision["signed_payload_digest"],
            "signer_status": decision["signer_status"],
            "signer_scope": decision["signer_scope"],
            "signer_expires_at": decision["signer_expires_at"],
            "signer_revoked_at": decision["signer_revoked_at"],
            "signer_rotated_to": decision["signer_rotated_to"],
            "evaluation_tick": decision["evaluation_tick"],
            "effect_authority": decision["effect_authority"],
            "effect_mislabel": decision["effect_mislabel"],
            "effect_basis": decision["effect_basis"],
            "invariant_preserved": not (decision["contradiction_detected"] and mutation_result["applied"]),
            "proposal_consolidated": mutation_result["applied"],
            "mutation_request": mutation_request,
            "verifier_decision": verifier_decision,
            "target_before": proposal_candidate,
            "target_after": mutation_result["target"],
            "mutation_log_entry": mutation_result["log"],
            "mutation_log": mutation_log,
        },
    )
    deliver("audit", mutation_packet["header"]["packet_id"])

    # 6. A contradiction opens a deferred design-revalidation job: the same correction loop
    #    that schedules post-action revalidation. Release stays blocked until resolved.
    if decision["contradiction_detected"]:
        revalidation_packet = emit(
            "BackpressureCommand",
            "verifier",
            "bus",
            "P3",
            100,
            default_epistemics(
                0.9,
                "rule_based",
                "full_premise",
                [{"packet_id": contradiction_packet["header"]["packet_id"]}],
            ),
            permissions(
                ["human_explanation", "contradiction_detection"],
                ["direct_action", "rule_revision", "memory_consolidation"],
            ),
            {
                "type": "design_revalidation",
                "proposal_id": config["proposal_id"],
                "targets_invariant": config["targets_invariant"],
                "verifier_rule_id": decision["verifier_rule_id"],
                "resolution_required": "explicit_human_exception_plus_revalidation",
                "release_blocked": True,
            },
        )
        deliver("bus", revalidation_packet["header"]["packet_id"])
    return broker.trace_for(trace_id)


def main() -> int:
    args = sys.argv[1:]
    scenario = None
    if args[:1] == ["--scenario"]:
        if len(args) < 2:
            raise SystemExit("--scenario requires a scenario name")
        scenario = load_scenario(args[1])
        command = scenario["command"]
    else:
        command = " ".join(args) or "Get to the far side safely under time pressure."
    trace = run(command, scenario)
    for item in trace:
        header = item["header"]
        epistemics = item["epistemics"]
        provenance = ",".join(source["packet_id"] for source in epistemics["provenance"] if "packet_id" in source)
        provenance_text = f" provenance={provenance}" if provenance else ""
        print(
            f"{header['trace_id']} {header['packet_id']} {header['packet_type']:22} "
            f"priority={header['priority']} lane={PRIORITY_LANES[header['priority']]} "
            f"source={header['source_engine']} target={header['target_engine']} "
            f"license={epistemics['epistemic_license']} "
            f"attention={score_packet(item)}{provenance_text}"
        )
        print(json.dumps(item["payload"], indent=2, sort_keys=True))
    plan = next((item for item in trace if item["header"]["packet_type"] == "PlanProposal"), None)
    if plan:
        print(
            "HumanExplanation       "
            + render_human_explanation(
                plan["payload"],
                plan["epistemics"]["confidence"],
                plan["epistemics"]["epistemic_license"],
            )
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
