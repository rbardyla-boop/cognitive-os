#!/usr/bin/env python3
"""Live epistemic operating-state snapshot for scenarios."""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run
from ingest_experience import run_ingestion_scenario
from recovery_replay import replay_queue, CorrectionQueue, load_correction_jobs
from replay_key import load_replay_key
from semantic_candidate_extractor import run_extraction_scenario


def build_snapshot(trace: list[dict], user_input: str, scenario: dict | None = None) -> dict:
    by_type: dict[str, list[dict]] = {}
    for packet in trace:
        by_type.setdefault(packet["header"]["packet_type"], []).append(packet)

    intent = _last(by_type.get("IntentPacket", []))
    plan = _last(by_type.get("PlanProposal", []))
    attention = _attention_packet(by_type.get("BackpressureCommand", []))
    retrieval = _last_retrieval(by_type.get("RetrievalResult", []))
    contradictions = by_type.get("ContradictionPacket", [])
    mutations = by_type.get("MemoryMutation", [])
    regrets = by_type.get("PlanRegretPacket", [])
    attention_reviews = by_type.get("AttentionModeReviewPacket", [])

    raw_ingestion = run_ingestion_scenario(scenario["name"]) if scenario and scenario.get("experience_envelopes") else None
    candidate_extraction = (
        run_extraction_scenario(scenario["name"])
        if scenario and scenario.get("experience_envelopes") and scenario.get("candidate_extraction") is not None
        else None
    )
    return {
        "surface": "epistemic_snapshot",
        "surface_role": "current_cognition",
        "task": _task_section(user_input, intent, attention, plan),
        "driving_objects": _driving_objects(
            retrieval, contradictions, mutations, regrets, attention_reviews, raw_ingestion, candidate_extraction
        ),
        "contradictions": _contradictions(retrieval, contradictions, mutations),
        "decision_constraints": _decision_constraints(plan, contradictions, mutations, by_type),
        "pending_work": _pending_work(by_type, retrieval, mutations, scenario, raw_ingestion, candidate_extraction),
        "current_recommendation": _current_recommendation(plan, contradictions, mutations),
    }


def strict_validate(snapshot: dict) -> None:
    required_sections = {
        "task",
        "driving_objects",
        "contradictions",
        "decision_constraints",
        "pending_work",
        "current_recommendation",
    }
    missing = sorted(section for section in required_sections if section not in snapshot)
    if missing:
        raise AssertionError(f"snapshot missing sections: {missing}")
    task = snapshot["task"]
    for field in ("evidence_requirement", "attention_mode", "planner_mode"):
        if not task.get(field):
            raise AssertionError(f"snapshot task missing {field}")
    authority_objects = snapshot["driving_objects"].get("authority_objects")
    if not authority_objects:
        raise AssertionError("snapshot missing driving memory/authority objects")
    missing_license = [
        item.get("id") or item.get("kind")
        for item in authority_objects
        if not item.get("authority_license")
    ]
    if missing_license:
        raise AssertionError(f"snapshot authority objects missing licenses: {missing_license}")
    if "blocked_actions" not in snapshot["decision_constraints"]:
        raise AssertionError("snapshot missing blocked actions")
    if "allowed_actions" not in snapshot["decision_constraints"]:
        raise AssertionError("snapshot missing allowed actions")
    if "selected" not in snapshot["current_recommendation"]:
        raise AssertionError("snapshot missing current recommendation")
    if not any(
        key in snapshot["pending_work"]
        for key in ("post_action_revalidation", "contradiction_repair", "retest_required_memories", "deferred_jobs")
    ):
        raise AssertionError("snapshot missing pending work")


def _task_section(user_input: str, intent: dict | None, attention: dict | None, plan: dict | None) -> dict:
    payload = intent["payload"] if intent else {}
    attention_payload = attention["payload"] if attention else {}
    plan_payload = plan["payload"] if plan else {}
    return {
        "user_input": user_input,
        "parsed_intent": payload,
        "preferred_target": payload.get("preferred_bridge") or payload.get("target"),
        "urgency": payload.get("urgency", "NotApplicable"),
        "evidence_requirement": payload.get("evidence_requirement", "NotApplicable"),
        "attention_mode": attention_payload.get("system_mode", "NotApplicable"),
        "planner_mode": plan_payload.get("mode", "NotApplicable"),
    }


def _driving_objects(
    retrieval: dict | None,
    contradictions: list[dict],
    mutations: list[dict],
    regrets: list[dict],
    attention_reviews: list[dict],
    raw_ingestion: dict | None = None,
    candidate_extraction: dict | None = None,
) -> dict:
    objects = []
    if raw_ingestion:
        for raw_episode in raw_ingestion["raw_episode_store"]["episodes"]:
            objects.append({
                "kind": "raw_episode",
                "id": raw_episode["episode_id"],
                "claim": [],
                "status": "captured_raw",
                "authority_license": "hypothesis_only",
                "allowed_use": ["retrieval", "human_explanation"],
                "forbidden_use": ["direct_action", "memory_consolidation", "rule_revision", "safety_certification"],
                "scope_conditions": raw_episode.get("capture_context", {}),
                "source_episodes": [raw_episode["episode_id"]],
                "integrity_digest": raw_episode["integrity_digest"],
                "ingestion_license": raw_episode["ingestion_license"],
            })
    if candidate_extraction:
        for candidate in candidate_extraction["candidate_memory_nodes"]:
            objects.append({
                "kind": "candidate_memory_node",
                "id": candidate["memory_id"],
                "claim": candidate["claim"],
                "status": candidate["status"],
                "authority_license": candidate["epistemic_license"],
                "allowed_use": candidate["allowed_use"],
                "forbidden_use": candidate["forbidden_use"],
                "scope_conditions": {},
                "source_episodes": candidate["source_episodes"],
                "source_raw_episode_id": candidate["source_raw_episode_id"],
                "integrity_digest": candidate["source_integrity_digest"],
                "authority_class": candidate["authority_class"],
            })
    if retrieval:
        payload = retrieval["payload"]
        for group in ("semantic_nodes", "procedures", "episodes", "contradictions"):
            for item in payload.get(group, []):
                content = item.get("content", {})
                objects.append({
                    "kind": group[:-1] if group.endswith("s") else group,
                    "id": content.get("memory_id") or content.get("procedure_id") or content.get("episode_id"),
                    "claim": content.get("claim") or content.get("parsed_claims"),
                    "status": item.get("status"),
                    "authority_license": item.get("epistemic_license"),
                    "allowed_use": item.get("allowed_use", []),
                    "forbidden_use": item.get("forbidden_use", []),
                    "scope_conditions": content.get("scope_conditions", {}),
                    "source_episodes": item.get("source_episodes", []),
                })
    for mutation in mutations:
        payload = mutation["payload"]
        for key in ("target_before", "target_after"):
            target = payload.get(key)
            if not isinstance(target, dict):
                continue
            if target.get("authority_class") in {"bootstrap_candidate", "promoted_invariant"}:
                objects.append({
                    "kind": "bootstrap_design_claim",
                    "id": target.get("memory_id"),
                    "claim": target.get("claim"),
                    "status": target.get("status"),
                    "authority_class": target.get("authority_class"),
                    "authority_license": target.get("epistemic_license"),
                    "allowed_use": target.get("allowed_use", []),
                    "forbidden_use": target.get("forbidden_use", []),
                    "scope_conditions": target.get("scope_conditions", {}),
                })
    for packet in contradictions:
        payload = packet["payload"]
        objects.append({
            "kind": "verifier_contradiction",
            "id": payload.get("subject"),
            "claim": payload.get("reason"),
            "status": payload.get("conflict_type"),
            "authority_license": packet["epistemics"].get("epistemic_license"),
            "allowed_use": packet["permissions"].get("allowed_use", []),
            "forbidden_use": packet["permissions"].get("forbidden_use", []),
            "scope_conditions": payload.get("scope_conditions", {}),
            "verifier_rule_id": payload.get("verifier_rule_id"),
        })
    for packet in regrets:
        payload = packet["payload"]
        objects.append({
            "kind": "planner_regret",
            "id": payload.get("regret_id"),
            "claim": payload.get("review_reason"),
            "status": payload.get("regret_type"),
            "regret_class": payload.get("regret_class"),
            "authority_license": packet["epistemics"].get("epistemic_license"),
            "allowed_use": packet["permissions"].get("allowed_use", []),
            "forbidden_use": packet["permissions"].get("forbidden_use", []),
            "scope_conditions": payload.get("scope_conditions", {}),
            "target_policy_id": payload.get("target_policy_id"),
        })
    for packet in attention_reviews:
        payload = packet["payload"]
        objects.append({
            "kind": "attention_mode_review",
            "id": payload.get("review_id"),
            "claim": payload.get("classification"),
            "status": payload.get("observed_mode"),
            "authority_license": packet["epistemics"].get("epistemic_license"),
            "allowed_use": packet["permissions"].get("allowed_use", []),
            "forbidden_use": packet["permissions"].get("forbidden_use", []),
            "scope_conditions": payload.get("scope_conditions", {}),
            "target_policy_id": payload.get("target_policy_id"),
        })
    return {
        "authority_objects": objects,
        "verifier_rules_used": sorted({
            item
            for item in (
                [packet["payload"].get("verifier_rule_id") for packet in contradictions]
                + [
                    mutation["payload"].get("verifier_rule_id")
                    or mutation["payload"].get("verifier_decision", {}).get("rule_id")
                    for mutation in mutations
                ]
            )
            if item
        }),
    }


def _contradictions(retrieval: dict | None, contradiction_packets: list[dict], mutations: list[dict]) -> dict:
    active = [
        {
            "subject": packet["payload"].get("subject"),
            "conflict_type": packet["payload"].get("conflict_type"),
            "license": packet["epistemics"].get("epistemic_license"),
            "verifier_rule_id": packet["payload"].get("verifier_rule_id"),
            "reason": packet["payload"].get("reason"),
        }
        for packet in contradiction_packets
    ]
    unresolved = []
    scoped = []
    resolved = []
    if retrieval:
        for item in retrieval["payload"].get("semantic_nodes", []):
            content = item["content"]
            record = {
                "id": content.get("memory_id"),
                "status": item.get("status"),
                "license": item.get("epistemic_license"),
                "contradictions": [entry.get("memory_id") for entry in item.get("contradictions", [])],
                "scope_conditions": content.get("scope_conditions", {}),
            }
            if item.get("status") == "contradicted":
                unresolved.append(record)
            if item.get("status") == "exception_scoped":
                scoped.append(record)
            if item.get("status") in {"superseded", "deprecated_but_preserved", "retest_required"}:
                resolved.append(record)
    for mutation in mutations:
        payload = mutation["payload"]
        if payload.get("operation") != "contradiction_repair":
            continue
        record = {
            "repair_id": payload["repair_id"],
            "repair_type": payload["repair_type"],
            "target": payload["mutation_log_entry"]["target_object_id"],
            "after": payload["mutation_log_entry"]["after_status"],
            "verifier_rule_id": payload["verifier_rule_id"],
            "scope_conditions": payload.get("scope_conditions", {}),
        }
        if payload["repair_type"] == "unresolved":
            unresolved.append(record)
        elif payload["repair_type"] == "resolved_by_scope":
            scoped.append(record)
        else:
            resolved.append(record)
    return {
        "active": active,
        "unresolved": unresolved,
        "scoped": scoped,
        "resolved_or_superseded": resolved,
    }


def _decision_constraints(plan: dict | None, contradictions: list[dict], mutations: list[dict], by_type: dict[str, list[dict]]) -> dict:
    plan_payload = plan["payload"] if plan else {}
    blocked = []
    if any(packet["epistemics"].get("epistemic_license") == "hazard_only" for packet in contradictions):
        blocked.append("Bridge A direct action blocked by hazard_only contradiction evidence.")
    if any(packet["payload"].get("strict_action_blocked") for packet in mutations):
        blocked.append("Strict/full-premise action blocked by unresolved contradiction.")
    if any(packet["payload"].get("operation") == "mutation_rejected" for packet in mutations):
        blocked.append("Mutation blocked by mutation authority.")
    if plan_payload.get("mode") == "evidence_strict_refusal":
        blocked.append("Action blocked by strict evidence requirement.")
    allowed = plan_payload.get("candidate_actions", [])
    if not allowed and by_type.get("HumanPromotionPacket"):
        allowed = ["human_approved_promotion"]
    return {
        "blocked_actions": blocked,
        "allowed_actions": allowed,
        "required_fallback": plan_payload.get("fallback_plan", {}).get("action"),
        "required_revalidation": any(
            packet["payload"].get("type") == "post_action_revalidation"
            for packet in by_type.get("BackpressureCommand", [])
        ),
    }


def _pending_work(
    by_type: dict[str, list[dict]],
    retrieval: dict | None,
    mutations: list[dict],
    scenario: dict | None = None,
    raw_ingestion: dict | None = None,
    candidate_extraction: dict | None = None,
) -> dict:
    backpressure = by_type.get("BackpressureCommand", [])
    retest = []
    if retrieval:
        for item in retrieval["payload"].get("semantic_nodes", []):
            if item.get("status") == "retest_required" or item.get("revalidation_requirement") == "post_action_revalidation":
                retest.append(item["content"].get("memory_id"))
    pending = {
        "post_action_revalidation": [
            packet["payload"] for packet in backpressure
            if packet["payload"].get("type") == "post_action_revalidation"
        ],
        "contradiction_repair": [
            packet["payload"] for packet in backpressure
            if packet["payload"].get("type") == "contradiction_repair"
        ],
        "retest_required_memories": sorted(set(retest)),
        "deferred_jobs": [
            job for packet in backpressure
            for job in packet["payload"].get("deferred_jobs", [])
        ],
        "mutation_rejections": [
            packet["payload"]["mutation_log_entry"] for packet in mutations
            if packet["payload"].get("operation") == "mutation_rejected"
        ],
        "planner_review": [
            packet["payload"] for packet in backpressure
            if packet["payload"].get("type") == "planner_review"
        ],
        "attention_mode_review": [
            packet["payload"] for packet in backpressure
            if packet["payload"].get("type") == "attention_mode_review"
        ],
    }
    if scenario and scenario.get("correction_jobs"):
        queue = CorrectionQueue(scenario.get("max_low_priority_open"))
        valid_jobs, rejected_config = load_correction_jobs(scenario["correction_jobs"])
        for job in valid_jobs:
            queue.add(job)
        replay = replay_queue(
            queue, {**scenario, "resolve_jobs": False}, rejected_config=rejected_config, key=load_replay_key()
        )
        pending["correction_queue"] = {
            "open_correction_jobs": replay["queue"]["open"],
            "deferred_correction_jobs": replay["queue"]["deferred"],
            "failed_correction_jobs": replay["queue"]["failed"],
            "coalesced_correction_jobs": replay["queue"]["coalesced"],
            "highest_priority_pending_job": replay["queue"]["highest_priority_pending_job"],
            "blocked_correction_jobs": replay["queue"]["blocked"],
            "jobs_requiring_mutation_authority": replay["queue"]["requires_mutation_authority"],
            "jobs_with_retry_lineage": replay["queue"]["with_retry_lineage"],
            "rejected_config_attempts": replay["queue"]["rejected_config_attempts"],
            "ledger_authentication": replay["replay"]["ledger_authentication"],
            "coalesced_or_deferred_count": replay["queue"]["coalesced_or_deferred_count"],
            "coalesced_duplicate_count": replay["queue"]["coalesced_duplicate_count"],
            "ordering_key": replay["replay"]["ordering_key"],
        }
    if raw_ingestion:
        pending["raw_ingestion"] = {
            "raw_episode_count": raw_ingestion["raw_episode_store"]["episode_count"],
            "raw_before_semantic": raw_ingestion["raw_before_semantic"],
            "semantic_candidates": raw_ingestion["semantic_candidates"],
            "rejected_envelopes": raw_ingestion["rejected_envelopes"],
            "candidate_without_raw_blocked": raw_ingestion["candidate_without_raw_blocked"],
        }
    if candidate_extraction:
        pending["semantic_candidate_extraction"] = {
            "candidate_count": candidate_extraction["candidate_count"],
            "non_authoritative_by_default": candidate_extraction["non_authoritative_by_default"],
            "all_candidates_cite_raw_episode": candidate_extraction["all_candidates_cite_raw_episode"],
            "rejected_candidates": candidate_extraction["rejected_candidates"],
        }
    return pending


def _current_recommendation(plan: dict | None, contradictions: list[dict], mutations: list[dict]) -> dict:
    if plan:
        payload = plan["payload"]
        rejected = []
        if payload.get("route") != "Bridge A" and any("Bridge A" in json.dumps(packet["payload"]) for packet in contradictions):
            rejected.append({
                "alternative": "Bridge A",
                "reason": "downgraded or blocked by contradiction evidence",
            })
        return {
            "selected": payload.get("route"),
            "action": payload.get("action"),
            "planner_mode": payload.get("mode"),
            "risk_note": payload.get("risk_note"),
            "rejected_alternatives": rejected,
        }
    if mutations:
        latest = mutations[-1]["payload"]
        return {
            "selected": latest.get("operation", "state_change"),
            "action": latest.get("mutation_log_entry", {}).get("mutation_type"),
            "planner_mode": "NotApplicable",
            "risk_note": latest.get("mutation_log_entry", {}).get("reason"),
            "rejected_alternatives": [],
        }
    return {
        "selected": "none",
        "action": None,
        "planner_mode": "NotApplicable",
        "risk_note": "No plan or mutation recommendation in trace.",
        "rejected_alternatives": [],
    }


def _attention_packet(packets: list[dict]) -> dict | None:
    return next((packet for packet in packets if "system_mode" in packet["payload"]), None)


def _last_retrieval(packets: list[dict]) -> dict | None:
    if not packets:
        return None
    return next((packet for packet in reversed(packets) if packet["payload"].get("retrieval_after_repair")), packets[-1])


def _last(packets: list[dict]) -> dict | None:
    return packets[-1] if packets else None


def main() -> int:
    args = sys.argv[1:]
    strict = False
    if "--strict" in args:
        strict = True
        args.remove("--strict")
    if len(args) >= 2 and args[0] == "--scenario":
        scenario = load_scenario(args[1])
        trace = run(scenario["command"], scenario)
        user_input = scenario["command"]
    else:
        user_input = " ".join(args) or "I need to cross the river quickly. Is Bridge A safe?"
        trace = run(user_input)
        scenario = None
    snapshot = build_snapshot(trace, user_input, scenario)
    if strict:
        strict_validate(snapshot)
    print(json.dumps(snapshot, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
