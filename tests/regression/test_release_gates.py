import json
import tempfile
from pathlib import Path

from bridge_world_demo import load_scenario, run
from attention_review_audit import audit_attention_review_trace
from contradiction_audit import audit_contradiction_trace
from decision_audit import audit_trace
from epistemic_snapshot import build_snapshot, strict_validate
from ingest_experience import run_ingestion_scenario
from mutation_audit import audit_mutation_trace
from mutation_gateway import apply_memory_mutation, verifier_allows_mutation
from planner_regret_audit import audit_planner_regret_trace
from recovery_replay import CorrectionJob, ConfigValidationError, load_correction_jobs, replay_scenario
from replay_asymmetric_key import (
    decode_private_key,
    generate_ephemeral_private_key_pem,
    public_key_pem_from_private_pem,
)
from replay_key import generate_ephemeral_key_hex
from design_signing import sign_change_set, verify_change_signature
from semantic_candidate_extractor import run_extraction_scenario
from design_audit import audit_design_trace
from effect_classifier import derive_effect, effect_family
from trace_diff import PROBES, combine_effects, derive_effect_from_trace, _change_set, _change_set_stale
from change_provenance import (
    CONTROL_POINT_POLICY_ARTIFACTS,
    build_content_change_set,
    content_hash,
    load_baseline_policy,
    verify_change_set_provenance,
)
from mechanism_provenance import (
    load_mechanism_manifest,
    probe_outcome_for_proposed_source,
    verify_mechanism_change_provenance,
    verify_mechanism_manifest,
)
from project_self_audit import (
    audit_design_decisions,
    consolidate_project_health,
    evaluate_design_proposal,
    load_design_memory,
    run_project_audit,
)
from toy_planner import build_plan
from qa_checks import (
    assert_all_actions_traced,
    assert_degraded_actions_schedule_revalidation,
    assert_memory_mutation_logged,
    assert_mutation_authority_audited,
    assert_no_forbidden_use_reaches_action_engine,
    validate_trace_packets,
)


def main() -> None:
    for schema in Path("schemas/cip").glob("*.json"):
        with schema.open("r", encoding="utf-8") as handle:
            json.load(handle)

    trace = run("cross bridge A")
    validate_trace_packets(trace)
    assert_all_actions_traced(trace)
    assert_degraded_actions_schedule_revalidation(trace)
    assert_no_forbidden_use_reaches_action_engine(trace)
    assert_memory_mutation_logged(trace)

    storm = run(load_scenario("interrupt_storm")["command"], load_scenario("interrupt_storm"))
    validate_trace_packets(storm)
    assert any(packet["payload"].get("source_count") == 1000 for packet in storm if packet["header"]["packet_type"] == "SystemStatePacket")

    safety = run(load_scenario("bridge_a_safe_time_pressure")["command"], load_scenario("bridge_a_safe_time_pressure"))
    audit = audit_trace(safety)
    assert audit["decision"] == "recommend Bridge B"
    assert audit["post_action_revalidation"] is True
    assert "Urgency parsed as high." in audit["primary_factors"]
    assert any("hazard_only" in item for item in audit["primary_factors"])
    assert any("forbidden_use" in item for item in audit["blocked_alternatives"])

    for scenario_name, expected_decision in (
        ("direct_mutation_without_verifier", "reject"),
        ("memory_mutation_with_low_authority_packet", "reject"),
        ("valid_human_promotion_allows_invariant", "allow"),
    ):
        scenario = load_scenario(scenario_name)
        mutation_trace = run(scenario["command"], scenario)
        assert_mutation_authority_audited(mutation_trace)
        mutation_audit = audit_mutation_trace(mutation_trace)
        assert mutation_audit["decision"] == expected_decision
        assert mutation_audit["append_only_log_entries"] >= 1
    promotion = audit_mutation_trace(run(
        load_scenario("valid_human_promotion_allows_invariant")["command"],
        load_scenario("valid_human_promotion_allows_invariant"),
    ))
    assert promotion["source"] == "HumanPromotionPacket"
    assert promotion["before"] == "bootstrap_candidate"
    assert promotion["after"] == "promoted_invariant"

    success = audit_mutation_trace(run(
        load_scenario("degraded_action_success_does_not_overconfirm")["command"],
        load_scenario("degraded_action_success_does_not_overconfirm"),
    ))
    assert success["correction_order"] == ["PROC_use_stable_bridge_under_rain", "M_bridge_a_damage_reported"]
    assert success["after"] == "retest_required"
    assert success["overconfirmation_blocked"] is True
    assert success["mutations"][0]["target_kind"] == "procedure"
    assert success["mutations"][1]["target_kind"] == "belief"

    failure = audit_mutation_trace(run(
        load_scenario("degraded_action_failure_quarantines_memory")["command"],
        load_scenario("degraded_action_failure_quarantines_memory"),
    ))
    assert all(entry["after"] == "quarantined" for entry in failure["mutations"])

    partial = audit_mutation_trace(run(
        load_scenario("degraded_action_partial_success_scopes_memory")["command"],
        load_scenario("degraded_action_partial_success_scopes_memory"),
    ))
    assert partial["mutations"][0]["after"] == "exception_scoped_policy"
    assert partial["mutations"][1]["after"] == "exception_scoped"
    assert partial["mutations"][0]["scope_conditions"]["constraint"] == "abort_path_preserved"
    assert partial["mutations"][1]["scope_conditions"]["constraint"] == "damage_report_not_globally_resolved"

    resolved = audit_contradiction_trace(run(
        load_scenario("contradiction_resolved_by_new_evidence")["command"],
        load_scenario("contradiction_resolved_by_new_evidence"),
    ))
    assert resolved["repair_type"] == "resolved_by_new_evidence"
    assert resolved["mutations"][0]["after"] == "superseded"
    assert resolved["mutations"][1]["after"] == "retest_required"
    assert resolved["raw_episodes_preserved"] is True
    assert resolved["verifier_rule_id"] == "V_RULE_CONTRADICTION_REPAIR_REQUIRES_STRONGER_EVIDENCE"

    scoped = audit_contradiction_trace(run(
        load_scenario("contradiction_scoped_by_context")["command"],
        load_scenario("contradiction_scoped_by_context"),
    ))
    assert scoped["repair_type"] == "resolved_by_scope"
    assert all(entry["after"] == "exception_scoped" for entry in scoped["mutations"])
    assert scoped["mutations"][0]["scope_conditions"]["rain_level"] == "clear_or_light"
    assert scoped["mutations"][1]["scope_conditions"]["rain_level"] == "heavy"

    unresolved = audit_contradiction_trace(run(
        load_scenario("contradiction_remains_unresolved")["command"],
        load_scenario("contradiction_remains_unresolved"),
    ))
    assert unresolved["repair_type"] == "unresolved"
    assert unresolved["unresolved_visible"] is True
    assert unresolved["strict_action_blocked"] is True
    assert all(entry["after"] == "contradicted" for entry in unresolved["mutations"])

    unresolved_trace = run(
        load_scenario("contradiction_remains_unresolved")["command"],
        load_scenario("contradiction_remains_unresolved"),
    )
    post_repair_retrieval = next(
        packet for packet in unresolved_trace
        if packet["header"]["packet_type"] == "RetrievalResult"
        and packet["payload"].get("retrieval_after_repair")
    )
    statuses = {
        item["content"]["memory_id"]: item["status"]
        for item in post_repair_retrieval["payload"]["semantic_nodes"]
    }
    assert statuses["M_bridge_a_passable"] == "contradicted"
    assert statuses["M_bridge_a_damage_reported"] == "contradicted"
    assert any(item["contradictions"] for item in post_repair_retrieval["payload"]["semantic_nodes"])

    scoped_retrieval = {
        "semantic_nodes": [
            {
                "content": {
                    "memory_id": "M_bridge_a_passable",
                    "claim": "Bridge A passable under scoped inspection.",
                    "scope_conditions": {
                        "rain_level": "clear_or_light",
                        "inspection_status": "recent",
                    },
                },
                "status": "exception_scoped",
                "revalidation_requirement": "none",
            }
        ]
    }
    clear_plan = build_plan(
        goal={"goal": "cross", "preferred_bridge": "Bridge A"},
        retrieved_memories=scoped_retrieval,
        epistemic_license="full_premise",
        world_state=_planner_world("clear", recent_inspection=True),
        time_budget_minutes=12,
        risk_budget=0.2,
        system_mode="Operational",
    )
    heavy_plan = build_plan(
        goal={"goal": "cross", "preferred_bridge": "Bridge A"},
        retrieved_memories=scoped_retrieval,
        epistemic_license="full_premise",
        world_state=_planner_world("heavy_rain", recent_inspection=False),
        time_budget_minutes=12,
        risk_budget=0.2,
        system_mode="Operational",
    )
    assert clear_plan["route"] == "Bridge A"
    assert "scoped_matches=M_bridge_a_passable" in clear_plan["risk_note"]
    assert heavy_plan["route"] == "Bridge B"
    assert "scoped_mismatches=M_bridge_a_passable" in heavy_plan["risk_note"] or heavy_plan["route"] != "Bridge A"

    snapshot_trace = run(
        load_scenario("bridge_a_safe_time_pressure")["command"],
        load_scenario("bridge_a_safe_time_pressure"),
    )
    snapshot = build_snapshot(snapshot_trace, load_scenario("bridge_a_safe_time_pressure")["command"])
    strict_validate(snapshot)
    assert snapshot["surface"] == "epistemic_snapshot"
    assert snapshot["surface_role"] == "current_cognition"
    assert snapshot["task"]["attention_mode"] == "Reflex"
    assert snapshot["task"]["planner_mode"] == "minimax"
    assert snapshot["current_recommendation"]["selected"] == "Bridge B"
    assert any("Bridge A direct action blocked" in item for item in snapshot["decision_constraints"]["blocked_actions"])

    unresolved_snapshot = build_snapshot(
        run(load_scenario("contradiction_remains_unresolved")["command"], load_scenario("contradiction_remains_unresolved")),
        load_scenario("contradiction_remains_unresolved")["command"],
    )
    strict_validate(unresolved_snapshot)
    assert unresolved_snapshot["contradictions"]["unresolved"]
    assert any("Strict/full-premise action blocked" in item for item in unresolved_snapshot["decision_constraints"]["blocked_actions"])

    scoped_snapshot = build_snapshot(
        run(load_scenario("contradiction_scoped_by_context")["command"], load_scenario("contradiction_scoped_by_context")),
        load_scenario("contradiction_scoped_by_context")["command"],
    )
    strict_validate(scoped_snapshot)
    assert scoped_snapshot["contradictions"]["scoped"]
    assert any(entry["scope_conditions"].get("rain_level") == "heavy" for entry in scoped_snapshot["contradictions"]["scoped"])

    promotion_snapshot = build_snapshot(
        run(load_scenario("valid_human_promotion_allows_invariant")["command"], load_scenario("valid_human_promotion_allows_invariant")),
        load_scenario("valid_human_promotion_allows_invariant")["command"],
    )
    strict_validate(promotion_snapshot)
    assert any(
        item.get("authority_class") == "promoted_invariant"
        for item in promotion_snapshot["driving_objects"]["authority_objects"]
    )

    correct_regret = audit_planner_regret_trace(run(
        load_scenario("planner_correct_under_uncertainty")["command"],
        load_scenario("planner_correct_under_uncertainty"),
    ))
    assert correct_regret["regret_type"] == "correct_under_uncertainty"
    assert correct_regret["regret_class"] == "policy_success"
    assert correct_regret["after"] == "planner_policy_scoped_strengthened"
    assert correct_regret["belief_or_procedure_authority_changed"] is False

    near_miss = audit_planner_regret_trace(run(
        load_scenario("planner_near_miss_requires_policy_review")["command"],
        load_scenario("planner_near_miss_requires_policy_review"),
    ))
    assert near_miss["regret_type"] == "near_miss_policy_review"
    assert near_miss["regret_class"] == "safety_near_miss"
    assert near_miss["review_required"] is True
    assert near_miss["review_status"] == "open"
    assert near_miss["review_deferred"] is False
    assert near_miss["global_rule_rewrite"] is False

    overconservative = audit_planner_regret_trace(run(
        load_scenario("planner_overconservative_waits_unnecessarily")["command"],
        load_scenario("planner_overconservative_waits_unnecessarily"),
    ))
    assert overconservative["regret_class"] == "opportunity_cost"
    assert overconservative["policy_update_kind"] == "opportunity_cost_review"
    assert "not a safety failure" in overconservative["review_reason"]

    planner_snapshot = build_snapshot(
        run(load_scenario("planner_near_miss_requires_policy_review")["command"], load_scenario("planner_near_miss_requires_policy_review")),
        load_scenario("planner_near_miss_requires_policy_review")["command"],
    )
    strict_validate(planner_snapshot)
    assert planner_snapshot["pending_work"]["planner_review"]
    assert planner_snapshot["pending_work"]["planner_review"][0]["status"] == "open"
    assert planner_snapshot["pending_work"]["planner_review"][0]["deferred"] is False
    assert any(
        item["kind"] == "planner_regret"
        for item in planner_snapshot["driving_objects"]["authority_objects"]
    )
    assert any(
        item["kind"] == "planner_regret" and item["regret_class"] == "safety_near_miss"
        for item in planner_snapshot["driving_objects"]["authority_objects"]
    )

    bad_policy_source = {
        "header": {"packet_id": "P_REGRET_BAD", "source_engine": "planner_regret"},
        "permissions": {
            "allowed_use": ["planner_policy_update"],
            "forbidden_use": ["direct_action", "memory_consolidation"],
        },
    }
    bad_policy_request = {
        "mutation_id": "MUT_BAD_POLICY_AUTHORITY",
        "trace_id": "T_REVIEW",
        "source_packet_id": "P_REGRET_BAD",
        "verifier_decision_id": "V_DEC_BAD_POLICY_AUTHORITY",
        "target_object_id": "M_bridge_a_damage_reported",
        "requested_use": "planner_policy_update",
        "mutation_type": "planner_policy_update",
        "patch": {"epistemic_license": "full_premise"},
        "authority_snapshot": {"forbidden_use": []},
    }
    bad_policy_decision = verifier_allows_mutation(
        "V_DEC_BAD_POLICY_AUTHORITY",
        "planner_policy_update",
        "planner_policy_update",
        "M_bridge_a_damage_reported",
        "P_REGRET_BAD",
    )
    bad_policy_log = []
    bad_policy_result = apply_memory_mutation(
        bad_policy_request,
        {"memory_id": "M_bridge_a_damage_reported", "status": "hazard_only"},
        bad_policy_source,
        bad_policy_decision,
        bad_policy_log,
    )
    assert bad_policy_result["applied"] is False
    assert bad_policy_result["log"]["reason"] == "planner_policy_update target must be planner policy"

    reflex_review = audit_attention_review_trace(run(
        load_scenario("reflex_mode_correctly_triggered")["command"],
        load_scenario("reflex_mode_correctly_triggered"),
    ))
    assert reflex_review["classification"] == "justified"
    assert reflex_review["review_required"] is False
    assert reflex_review["memory_authority_changed"] is False
    assert reflex_review["procedure_authority_changed"] is False
    assert reflex_review["planner_authority_changed"] is False
    assert reflex_review["verifier_rule_changed"] is False

    false_reflex = audit_attention_review_trace(run(
        load_scenario("reflex_mode_false_alarm")["command"],
        load_scenario("reflex_mode_false_alarm"),
    ))
    assert false_reflex["classification"] == "over_triggered"
    assert false_reflex["review_status"] == "open"
    assert false_reflex["target_kind"] == "attention_policy"
    assert false_reflex["mutation"] == "attention_policy_update"
    assert false_reflex["memory_authority_changed"] is False
    assert false_reflex["procedure_authority_changed"] is False
    assert false_reflex["planner_authority_changed"] is False
    assert false_reflex["verifier_rule_changed"] is False

    storm_review = audit_attention_review_trace(run(
        load_scenario("interrupt_storm_recovery_replay")["command"],
        load_scenario("interrupt_storm_recovery_replay"),
    ))
    assert storm_review["classification"] == "recovery_replay_required"
    assert storm_review["raw_packets_preserved"] is True
    assert storm_review["raw_packet_count"] == 1000
    assert storm_review["coalesced"] is True
    assert storm_review["coalesced_source_count"] == 1000
    assert storm_review["kept_alive"] == ["P0", "P1"]
    assert "semantic_consolidation" in storm_review["recovery_replay"]["processed_deferred_jobs"]

    attention_snapshot = build_snapshot(
        run(load_scenario("reflex_mode_false_alarm")["command"], load_scenario("reflex_mode_false_alarm")),
        load_scenario("reflex_mode_false_alarm")["command"],
    )
    strict_validate(attention_snapshot)
    assert attention_snapshot["pending_work"]["attention_mode_review"]
    assert attention_snapshot["pending_work"]["attention_mode_review"][0]["status"] == "open"
    assert any(
        item["kind"] == "attention_mode_review" and item["claim"] == "over_triggered"
        for item in attention_snapshot["driving_objects"]["authority_objects"]
    )

    bad_attention_source = {
        "header": {"packet_id": "P_ATTENTION_BAD", "source_engine": "attention_review"},
        "permissions": {
            "allowed_use": ["attention_policy_update"],
            "forbidden_use": ["direct_action", "memory_consolidation"],
        },
    }
    bad_attention_request = {
        "mutation_id": "MUT_BAD_ATTENTION_AUTHORITY",
        "trace_id": "T_REVIEW",
        "source_packet_id": "P_ATTENTION_BAD",
        "verifier_decision_id": "V_DEC_BAD_ATTENTION_AUTHORITY",
        "target_object_id": "M_bridge_a_damage_reported",
        "requested_use": "attention_policy_update",
        "mutation_type": "attention_policy_update",
        "patch": {"epistemic_license": "full_premise"},
        "authority_snapshot": {"forbidden_use": []},
    }
    bad_attention_decision = verifier_allows_mutation(
        "V_DEC_BAD_ATTENTION_AUTHORITY",
        "attention_policy_update",
        "attention_policy_update",
        "M_bridge_a_damage_reported",
        "P_ATTENTION_BAD",
    )
    bad_attention_log = []
    bad_attention_result = apply_memory_mutation(
        bad_attention_request,
        {"memory_id": "M_bridge_a_damage_reported", "status": "hazard_only"},
        bad_attention_source,
        bad_attention_decision,
        bad_attention_log,
    )
    assert bad_attention_result["applied"] is False
    assert bad_attention_result["log"]["reason"] == "attention_policy_update target must be attention policy"

    recovery_order = replay_scenario("recovery_queue_orders_mixed_jobs")
    assert recovery_order["replay"]["deterministic_order"] == [
        "CJ_action_002",
        "CJ_contradiction_001",
        "CJ_planner_003",
        "CJ_attention_004",
        "CJ_semantic_005",
    ]
    assert recovery_order["replay"]["ordering_key"] == ["priority", "created_at_tick", "job_id"]

    recovery_resolved = replay_scenario("recovery_replay_resolves_jobs_through_gateway")
    assert len(recovery_resolved["queue"]["resolved"]) == 4
    assert recovery_resolved["replay"]["audit_replayable"] is True
    assert all(job["mutation_ids"] for job in recovery_resolved["queue"]["resolved"])
    assert all(entry["decision"] == "allow" for entry in recovery_resolved["replay"]["mutation_log"])

    recovery_bounds = replay_scenario("recovery_queue_bounds_deferred_work")
    assert recovery_bounds["queue"]["highest_priority_pending_job"]["priority"] == "P0"
    assert len(recovery_bounds["queue"]["deferred"]) == 2
    assert recovery_bounds["queue"]["coalesced_or_deferred_count"] == 2
    assert any(job["priority"] == "P1" for job in recovery_bounds["queue"]["open"])

    recovery_snapshot = build_snapshot(
        run(load_scenario("recovery_queue_bounds_deferred_work")["command"], load_scenario("recovery_queue_bounds_deferred_work")),
        load_scenario("recovery_queue_bounds_deferred_work")["command"],
        load_scenario("recovery_queue_bounds_deferred_work"),
    )
    strict_validate(recovery_snapshot)
    correction_queue = recovery_snapshot["pending_work"]["correction_queue"]
    assert correction_queue["highest_priority_pending_job"]["priority"] == "P0"
    assert correction_queue["deferred_correction_jobs"]
    assert correction_queue["jobs_requiring_mutation_authority"]

    # Sprint 17: idempotent recovery and replay safety.
    # Sprint 20 requires a signature to suppress a mutation, so these idempotent round-trips
    # sign with an ephemeral, never-committed key.
    replay_key_dir = tempfile.mkdtemp()
    replay_key_path = str(Path(replay_key_dir) / "replay.key")
    Path(replay_key_path).write_text(generate_ephemeral_key_hex(), encoding="utf-8")

    idem_first = replay_scenario("replay_resolved_job_is_idempotent", ledger_key_file=replay_key_path)
    assert idem_first["queue"]["resolved"]
    assert all(job["mutation_ids"] for job in idem_first["queue"]["resolved"])
    first_allow = [entry for entry in idem_first["replay"]["mutation_log"] if entry["decision"] == "allow"]
    assert first_allow, "first replay must apply mutations through the gateway"
    idem_second = replay_scenario(
        "replay_resolved_job_is_idempotent", ledger=idem_first["replay"]["ledger"], ledger_key_file=replay_key_path
    )
    second_allow = [entry for entry in idem_second["replay"]["mutation_log"] if entry["decision"] == "allow"]
    assert second_allow == [], "rerunning recovery replay must not re-apply mutations"
    assert idem_second["replay"]["idempotent_replay"] is True
    assert all(job["resolution"] == "verified_idempotent_replay" for job in idem_second["queue"]["resolved"])
    assert all(job["idempotent"] for job in idem_second["queue"]["resolved"])
    first_ids = sorted(mid for job in idem_first["queue"]["resolved"] for mid in job["mutation_ids"])
    second_ids = sorted(mid for job in idem_second["queue"]["resolved"] for mid in job["mutation_ids"])
    assert first_ids == second_ids
    assert len(second_ids) == len(set(second_ids)), "replay must not create duplicate mutation_ids"

    dedup = replay_scenario("duplicate_correction_job_is_rejected_or_coalesced")
    assert len(dedup["queue"]["coalesced"]) == 1
    assert dedup["queue"]["coalesced"][0]["coalesced_into"] == "CJ_dup_a"
    assert dedup["queue"]["coalesced_duplicate_count"] == 1
    assert [job["job_id"] for job in dedup["queue"]["resolved"]] == ["CJ_dup_a"]
    dedup_allow = [entry["mutation_id"] for entry in dedup["replay"]["mutation_log"] if entry["decision"] == "allow"]
    assert len(dedup_allow) == len(set(dedup_allow)) == 1

    retry = replay_scenario("failed_job_retry_preserves_audit_lineage")
    retry_job = next(job for job in retry["queue"]["jobs"] if job["job_id"] == "CJ_retry_001")
    assert retry_job["status"] == "resolved"
    assert retry_job["original_failure"]["reason"] == "missing_target_or_source_packet"
    assert len(retry_job["retry_lineage"]) >= 2
    assert retry_job["retry_lineage"][0]["outcome"] == "failed"
    assert retry_job["retry_lineage"][-1]["outcome"] == "resolved"
    assert retry_job["mutation_ids"] == ["MUT_CJ_retry_001"]

    retry_scenario = load_scenario("failed_job_retry_preserves_audit_lineage")
    retry_snapshot = build_snapshot(
        run(retry_scenario["command"], retry_scenario), retry_scenario["command"], retry_scenario
    )
    strict_validate(retry_snapshot)
    retry_queue = retry_snapshot["pending_work"]["correction_queue"]
    assert retry_queue["jobs_with_retry_lineage"]
    assert retry_queue["jobs_with_retry_lineage"][0]["original_failure"]["reason"] == "missing_target_or_source_packet"

    # Sprint 18: configuration boundary validation. Config is input; input is adversarial.
    rejected_configs = [
        ("priority_not_in_allowlist", {"job_id": "X", "job_type": "semantic_consolidation", "source_packet_id": "P", "created_at_tick": 1, "priority": "P9"}),
        ("priority_overrides_job_type", {"job_id": "X", "job_type": "safety_interrupt_recovery", "source_packet_id": "P", "created_at_tick": 1, "priority": "P5"}),
        ("unknown_job_type", {"job_id": "X", "job_type": "delete_all_memory", "source_packet_id": "P", "created_at_tick": 1}),
        ("authority_field_injection", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "epistemic_license": "full_premise"}),
        ("authority_field_injection", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "mutation_type": "bootstrap_promotion"}),
        ("invalid_status", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "status": "obliterated"}),
        ("invalid_status", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "status": "resolved"}),
        ("unknown_config_field", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "sudo": True}),
        ("required_authority_override", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "required_authority": "none"}),
        ("missing_required_field", {"job_id": "X", "job_type": "contradiction_repair"}),
        # Config may not assert resolution provenance — those are replay outputs, not entry fields.
        ("unknown_config_field", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "mutation_ids": ["MUT_FORGED"]}),
        ("unknown_config_field", {"job_id": "X", "job_type": "safety_interrupt_recovery", "source_packet_id": "P", "created_at_tick": 1, "resolution": "config_says_done"}),
        ("unknown_config_field", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": 1, "idempotent": True}),
        # Forbidden keys nested inside structured fields cannot smuggle past the top-level check.
        ("nested_unknown_field", {"job_id": "X", "job_type": "post_action_revalidation", "source_packet_id": "P", "created_at_tick": 1, "status": "failed", "original_failure": {"epistemic_license": "full_premise"}}),
        ("nested_unknown_field", {"job_id": "X", "job_type": "post_action_revalidation", "source_packet_id": "P", "created_at_tick": 1, "status": "failed", "retry_lineage": [{"mutation_type": "bootstrap_promotion"}]}),
        # Type confusion is a reported rejection, not a crash.
        ("invalid_field_type", {"job_id": "X", "job_type": "semantic_consolidation", "source_packet_id": "P", "created_at_tick": 1, "priority": ["P5"]}),
        ("invalid_field_type", {"job_id": "X", "job_type": "contradiction_repair", "source_packet_id": "P", "created_at_tick": "NaN"}),
    ]
    for expected_code, bad in rejected_configs:
        try:
            CorrectionJob.from_config(bad)
            raise AssertionError(f"config must be rejected: {bad}")
        except ConfigValidationError as exc:
            assert exc.code == expected_code, f"{bad} -> {exc.code}, expected {expected_code}"

    # Adversarial input must never crash the batch: a malformed item is reported, valid jobs survive.
    valid_item = {"job_id": "OK", "job_type": "semantic_consolidation", "source_packet_id": "P", "created_at_tick": 1}
    crash_item = {"job_id": "BAD", "job_type": "semantic_consolidation", "source_packet_id": "P", "created_at_tick": "NaN"}
    loaded_jobs, loaded_rejections = load_correction_jobs([valid_item, crash_item])
    assert [job.job_id for job in loaded_jobs] == ["OK"]
    assert [rejection["job_id"] for rejection in loaded_rejections] == ["BAD"]

    # Forged provenance cannot reach the ledger: a valid no-mutation job applies no mutations.
    forge = replay_scenario("config_valid_job_loads_without_mutation")
    assert forge["replay"]["ledger"]["applied_mutations"] == {}

    # A valid job loads and is normalized: priority/authority come from job_type, never config.
    normalized = CorrectionJob.from_config(
        {"job_id": "Y", "job_type": "safety_interrupt_recovery", "source_packet_id": "P", "created_at_tick": 1}
    )
    assert normalized.priority == "P0"
    assert normalized.required_authority == "none"

    config_priority = replay_scenario("config_priority_outside_allowlist_rejected")
    assert config_priority["queue"]["jobs"] == []
    assert config_priority["queue"]["rejected_config_attempts"][0]["reason"] == "priority_not_in_allowlist"

    config_unknown = replay_scenario("config_unknown_job_type_rejected")
    assert config_unknown["queue"]["jobs"] == []
    assert config_unknown["queue"]["rejected_config_attempts"][0]["reason"] == "unknown_job_type"

    config_inject = replay_scenario("config_attempts_authority_field_injection_rejected")
    assert config_inject["queue"]["jobs"] == []
    inject_rejection = config_inject["queue"]["rejected_config_attempts"][0]
    assert inject_rejection["reason"] == "authority_field_injection"
    assert "epistemic_license" in inject_rejection["fields"]
    assert "mutation_type" in inject_rejection["fields"]

    config_valid = replay_scenario("config_valid_job_loads_without_mutation")
    assert config_valid["queue"]["rejected_config_attempts"] == []
    assert len(config_valid["queue"]["resolved"]) == 1
    assert config_valid["queue"]["resolved"][0]["resolution"] == "no_state_mutation_required"
    assert config_valid["queue"]["resolved"][0]["mutation_ids"] == []
    assert config_valid["replay"]["mutation_log"] == []

    inject_scenario = load_scenario("config_attempts_authority_field_injection_rejected")
    inject_snapshot = build_snapshot(
        run(inject_scenario["command"], inject_scenario), inject_scenario["command"], inject_scenario
    )
    strict_validate(inject_snapshot)
    inject_queue = inject_snapshot["pending_work"]["correction_queue"]
    assert inject_queue["rejected_config_attempts"]
    assert inject_queue["rejected_config_attempts"][0]["reason"] == "authority_field_injection"
    assert inject_queue["open_correction_jobs"] == []

    # Sprint 19: ledger integrity / replay-identity authentication.
    # A ledger is evidence of prior replay, not proof; structurally bad ledgers never suppress.
    no_marker = replay_scenario("scenario_embedded_ledger_requires_trust_marker")
    assert no_marker["replay"]["ledger_authentication"]["status"] == "untrusted"
    assert no_marker["replay"]["ledger_authentication"]["reason"] == "embedded_ledger_requires_trust_marker"
    assert any(entry["decision"] == "allow" for entry in no_marker["replay"]["mutation_log"])
    assert no_marker["queue"]["resolved"][0]["resolution"] == "resolved_through_mutation_gateway"
    assert no_marker["queue"]["resolved"][0]["idempotent"] is False

    forged_ledger = replay_scenario("forged_ledger_verified_idempotent_rejected")
    assert forged_ledger["replay"]["ledger_authentication"]["status"] == "rejected"
    assert forged_ledger["replay"]["ledger_authentication"]["reason"] == "ledger_job_mutation_mismatch"
    assert any(entry["decision"] == "allow" for entry in forged_ledger["replay"]["mutation_log"])
    assert not any(job["idempotent"] for job in forged_ledger["queue"]["resolved"])

    ledger_mismatch = replay_scenario("ledger_job_mutation_mismatch_rejected")
    assert ledger_mismatch["replay"]["ledger_authentication"]["status"] == "rejected"
    assert ledger_mismatch["replay"]["ledger_authentication"]["reason"] == "ledger_job_mutation_mismatch"
    assert any(entry["decision"] == "allow" for entry in ledger_mismatch["replay"]["mutation_log"])

    # Tampering with the records is rejected by the integrity check before the signature is even evaluated.
    genuine_ledger = idem_first["replay"]["ledger"]
    assert genuine_ledger["provenance"]["schema"] == "recovery-ledger-v2"
    assert genuine_ledger["provenance"]["run_id"]
    assert genuine_ledger["provenance"]["integrity"]
    tampered = json.loads(json.dumps(genuine_ledger))
    tampered["resolved_jobs"]["CJ_INJECTED"] = {"resolution": "fake", "mutation_ids": [], "trace_id": "T", "source_packet_id": "P"}
    tampered_run = replay_scenario("replay_resolved_job_is_idempotent", ledger=tampered, ledger_key_file=replay_key_path)
    assert tampered_run["replay"]["ledger_authentication"]["status"] == "rejected"
    assert tampered_run["replay"]["ledger_authentication"]["reason"] == "ledger_integrity_mismatch"
    assert [entry for entry in tampered_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    # Sprint 20: signed / keyed replay identity. Only a valid signature may suppress a mutation.
    # genuine_ledger above is signed; idem_second already proved signed run-2 verify-not-reapply.
    assert genuine_ledger.get("signature", {}).get("scheme") == "hmac-sha256"
    assert idem_second["replay"]["ledger_authentication"]["status"] == "trusted"
    assert idem_second["replay"]["ledger_authentication"]["signature_status"] == "signed_valid"

    # Unsigned but otherwise well-formed ledger: trusted integrity, but cannot suppress.
    unsigned_ledger = json.loads(json.dumps(genuine_ledger))
    unsigned_ledger.pop("signature", None)
    unsigned_run = replay_scenario("replay_resolved_job_is_idempotent", ledger=unsigned_ledger, ledger_key_file=replay_key_path)
    assert unsigned_run["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert unsigned_run["replay"]["ledger_authentication"]["signature_status"] == "unsigned"
    assert [entry for entry in unsigned_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    # Strict mode: no key at all → an otherwise-signed ledger cannot be verified → no suppression.
    no_key_run = replay_scenario("replay_resolved_job_is_idempotent", ledger=genuine_ledger)
    assert no_key_run["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert no_key_run["replay"]["ledger_authentication"]["signature_status"] == "no_key"
    assert [entry for entry in no_key_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    # Tampering the signature block (records intact) → integrity passes, signature invalid → audit-only.
    sig_tampered = json.loads(json.dumps(genuine_ledger))
    sig_tampered["signature"]["signature_hex"] = "0" * 64
    sig_tampered_run = replay_scenario("replay_resolved_job_is_idempotent", ledger=sig_tampered, ledger_key_file=replay_key_path)
    assert sig_tampered_run["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert sig_tampered_run["replay"]["ledger_authentication"]["signature_status"] == "signature_invalid"
    assert [entry for entry in sig_tampered_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    # Wrong key → signature does not verify → audit-only, re-applied.
    other_key_path = str(Path(replay_key_dir) / "other.key")
    Path(other_key_path).write_text(generate_ephemeral_key_hex(), encoding="utf-8")
    wrong_key_run = replay_scenario("replay_resolved_job_is_idempotent", ledger=genuine_ledger, ledger_key_file=other_key_path)
    assert wrong_key_run["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert wrong_key_run["replay"]["ledger_authentication"]["signature_status"] == "wrong_key"
    assert [entry for entry in wrong_key_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    # Sprint 21: asymmetric replay identity. Public verification must not imply signing authority.
    private_key_path = str(Path(replay_key_dir) / "replay_ed25519_private.pem")
    public_key_path = str(Path(replay_key_dir) / "replay_ed25519_public.pem")
    private_pem = generate_ephemeral_private_key_pem()
    Path(private_key_path).write_text(private_pem, encoding="utf-8")
    Path(public_key_path).write_text(public_key_pem_from_private_pem(private_pem), encoding="utf-8")

    asym_first = replay_scenario(
        "replay_resolved_job_is_idempotent", ledger_private_key_file=private_key_path
    )
    assert asym_first["replay"]["ledger"]["signature"]["scheme"] == "ed25519"
    asym_second = replay_scenario(
        "replay_resolved_job_is_idempotent",
        ledger=asym_first["replay"]["ledger"],
        ledger_public_key_file=public_key_path,
    )
    assert asym_second["replay"]["ledger_authentication"]["status"] == "trusted"
    assert asym_second["replay"]["ledger_authentication"]["asymmetric_signature_status"] == "asymmetric_signed_valid"
    assert asym_second["replay"]["idempotent_replay"] is True
    assert not [entry for entry in asym_second["replay"]["mutation_log"] if entry["decision"] == "allow"]
    assert asym_second["replay"]["ledger"]["signature"]["scheme"] == "ed25519"

    public_only_fresh = replay_scenario(
        "replay_resolved_job_is_idempotent", ledger_public_key_file=public_key_path
    )
    assert "signature" not in public_only_fresh["replay"]["ledger"]
    assert [entry for entry in public_only_fresh["replay"]["mutation_log"] if entry["decision"] == "allow"]

    wrong_private_pem = generate_ephemeral_private_key_pem()
    wrong_public_key_path = str(Path(replay_key_dir) / "wrong_ed25519_public.pem")
    Path(wrong_public_key_path).write_text(public_key_pem_from_private_pem(wrong_private_pem), encoding="utf-8")
    wrong_public_run = replay_scenario(
        "replay_resolved_job_is_idempotent",
        ledger=asym_first["replay"]["ledger"],
        ledger_public_key_file=wrong_public_key_path,
    )
    assert wrong_public_run["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert wrong_public_run["replay"]["ledger_authentication"]["asymmetric_signature_status"] == "wrong_public_key"
    assert [entry for entry in wrong_public_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    asym_tampered = json.loads(json.dumps(asym_first["replay"]["ledger"]))
    asym_tampered["signature"]["signature_hex"] = "0" * 128
    asym_tampered_run = replay_scenario(
        "replay_resolved_job_is_idempotent",
        ledger=asym_tampered,
        ledger_public_key_file=public_key_path,
    )
    assert asym_tampered_run["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert asym_tampered_run["replay"]["ledger_authentication"]["asymmetric_signature_status"] == "signature_invalid"
    assert [entry for entry in asym_tampered_run["replay"]["mutation_log"] if entry["decision"] == "allow"]

    assert idem_second["replay"]["ledger_authentication"]["signature_status"] == "signed_valid"
    assert idem_second["replay"]["ledger_authentication"]["asymmetric_signature_status"] == "not_asymmetric"

    # The embedded test-trusted marker is still test-only: it cannot suppress without a signature.
    unsigned_scenario = replay_scenario("unsigned_ledger_cannot_suppress_mutation")
    assert unsigned_scenario["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert unsigned_scenario["replay"]["ledger_authentication"]["signature_status"] == "unsigned"
    assert any(entry["decision"] == "allow" for entry in unsigned_scenario["replay"]["mutation_log"])

    marker_only = replay_scenario("embedded_test_trusted_ledger_still_test_only")
    assert marker_only["replay"]["ledger_authentication"]["integrity_status"] == "trusted"
    assert marker_only["replay"]["ledger_authentication"]["status"] == "audit_only"
    assert any(entry["decision"] == "allow" for entry in marker_only["replay"]["mutation_log"])

    try:
        load_scenario("embedded_test_trusted_ledger_still_test_only", allow_test_trusted=False)
        raise AssertionError("production scenario loader must reject test_trusted replay ledgers")
    except PermissionError as exc:
        assert "rejects test_trusted" in str(exc)

    forged_scenario = load_scenario("forged_ledger_verified_idempotent_rejected")
    forged_snapshot = build_snapshot(
        run(forged_scenario["command"], forged_scenario), forged_scenario["command"], forged_scenario
    )
    strict_validate(forged_snapshot)
    forged_cq = forged_snapshot["pending_work"]["correction_queue"]
    assert forged_cq["ledger_authentication"]["status"] == "rejected"
    assert "signature_status" in forged_cq["ledger_authentication"]
    assert "asymmetric_signature_status" in forged_cq["ledger_authentication"]

    # Sprint 22: raw experience ingestion kernel. Raw evidence exists before interpretation.
    raw_ingest = run_ingestion_scenario("experience_ingest_preserves_raw_episode")
    assert raw_ingest["raw_episode_store"]["episode_count"] == 1
    raw_episode = raw_ingest["raw_episode_store"]["episodes"][0]
    assert raw_episode["episode_id"] == "RE_bridge_a_inspection_raw"
    assert raw_episode["raw_payload"]["text"] == "Bridge A has fresh scrape marks but no visible structural crack."
    assert raw_episode["parsed_claims"] == []
    assert raw_episode["integrity_digest"]
    assert raw_ingest["semantic_candidates"][0]["source_raw_episode_id"] == raw_episode["episode_id"]
    assert raw_ingest["raw_before_semantic"] is True

    candidate_gate = run_ingestion_scenario("semantic_candidate_requires_raw_episode")
    assert candidate_gate["candidate_without_raw_blocked"] is True
    assert candidate_gate["semantic_candidates"] == []

    append_only = run_ingestion_scenario("raw_episode_is_append_only")
    assert append_only["raw_episode_store"]["append_only_replace_blocked"] is True
    assert append_only["raw_episode_store"]["episode_count"] == 1

    malformed = run_ingestion_scenario("malformed_experience_rejected_without_partial_state")
    assert malformed["raw_episode_store"]["episode_count"] == 0
    assert malformed["semantic_candidates"] == []
    assert malformed["rejected_envelopes"][0]["reason"] == "ValueError"

    raw_snapshot_scenario = load_scenario("experience_ingest_preserves_raw_episode")
    raw_snapshot = build_snapshot(
        run(raw_snapshot_scenario["command"], raw_snapshot_scenario),
        raw_snapshot_scenario["command"],
        raw_snapshot_scenario,
    )
    strict_validate(raw_snapshot)
    assert raw_snapshot["pending_work"]["raw_ingestion"]["raw_before_semantic"] is True
    assert any(
        item["kind"] == "raw_episode" and item["integrity_digest"]
        for item in raw_snapshot["driving_objects"]["authority_objects"]
    )

    # Sprint 23: semantic candidate extraction. Candidates are interpretations, not accepted facts.
    extracted = run_extraction_scenario("raw_episode_generates_semantic_candidates")
    assert extracted["candidate_count"] == 1
    assert extracted["candidate_memory_nodes"][0]["memory_id"] == "CMN_bridge_a_standing_water"
    assert extracted["candidate_memory_nodes"][0]["claim"] == "Bridge A has standing water near the east approach."
    assert extracted["raw_episode_preserved"] is True

    default_candidate = run_extraction_scenario("candidate_defaults_to_hypothesis_only")
    candidate = default_candidate["candidate_memory_nodes"][0]
    assert candidate["epistemic_license"] == "hypothesis_only"
    assert candidate["status"] == "semantic_candidate"
    assert candidate["confidence"] == 0.0
    assert "direct_action" in candidate["forbidden_use"]
    assert "memory_consolidation" in candidate["forbidden_use"]
    assert default_candidate["non_authoritative_by_default"] is True

    cited = run_extraction_scenario("candidate_cites_raw_episode")
    cited_candidate = cited["candidate_memory_nodes"][0]
    assert cited_candidate["source_raw_episode_id"] == "RE_bridge_a_audio_raw"
    assert cited_candidate["source_raw_episode_id"] in cited_candidate["source_episodes"]
    assert cited_candidate["source_integrity_digest"] == cited["raw_episodes"][0]["integrity_digest"]
    assert cited["all_candidates_cite_raw_episode"] is True

    llm_authority = run_extraction_scenario("llm_output_cannot_create_authoritative_memory")
    assert llm_authority["candidate_count"] >= 1
    assert llm_authority["non_authoritative_by_default"] is True
    assert all(item["authority_class"] == "semantic_candidate" for item in llm_authority["candidate_memory_nodes"])
    assert all(item["epistemic_license"] == "hypothesis_only" for item in llm_authority["candidate_memory_nodes"])
    assert all("direct_action" in item["forbidden_use"] for item in llm_authority["candidate_memory_nodes"])

    extraction_failure = run_extraction_scenario("candidate_extraction_failure_preserves_raw_episode")
    assert extraction_failure["candidate_count"] == 0
    assert extraction_failure["raw_episode_count"] == 1
    assert extraction_failure["raw_episode_preserved"] is True
    assert extraction_failure["rejected_candidates"][0]["reason"] == "ValueError"

    candidate_snapshot_scenario = load_scenario("raw_episode_generates_semantic_candidates")
    candidate_snapshot = build_snapshot(
        run(candidate_snapshot_scenario["command"], candidate_snapshot_scenario),
        candidate_snapshot_scenario["command"],
        candidate_snapshot_scenario,
    )
    strict_validate(candidate_snapshot)
    assert candidate_snapshot["pending_work"]["semantic_candidate_extraction"]["candidate_count"] == 1
    assert candidate_snapshot["pending_work"]["semantic_candidate_extraction"]["non_authoritative_by_default"] is True
    assert any(
        item["kind"] == "candidate_memory_node"
        and item["authority_license"] == "hypothesis_only"
        and item["source_raw_episode_id"] == "RE_bridge_a_report_raw"
        for item in candidate_snapshot["driving_objects"]["authority_objects"]
    )

    broken_snapshot = json.loads(json.dumps(snapshot))
    broken_snapshot["driving_objects"]["authority_objects"][0].pop("authority_license")
    try:
        strict_validate(broken_snapshot)
        raise AssertionError("strict snapshot must fail when an authority license is missing")
    except AssertionError as exc:
        assert "missing licenses" in str(exc)

    # Sprint 24: unified self-correction (the Caitlin leap). The development process is
    # governed by the same bus, verifier, licenses, gateway, and correction loop as the
    # bridge world. A design proposal that weakens a locked invariant is blocked exactly
    # as Bridge A is blocked under hazard_only evidence.
    contradiction_design = audit_design_trace(run(
        load_scenario("design_contradiction_in_sprint_plan")["command"],
        load_scenario("design_contradiction_in_sprint_plan"),
    ))
    assert contradiction_design["contradiction_detected"] is True
    assert contradiction_design["contradiction_license"] == "hazard_only"
    assert contradiction_design["conflict_type"] == "hard_contradiction"
    assert contradiction_design["governance_decision"] == "block"
    assert contradiction_design["mutation_decision"] == "reject"
    assert contradiction_design["proposal_consolidated"] is False
    assert contradiction_design["invariant_preserved"] is True
    assert contradiction_design["revalidation_scheduled"] is True
    assert contradiction_design["blocks_release"] is True
    assert contradiction_design["naked_fact"] is False

    consistent_design = audit_design_trace(run(
        load_scenario("design_proposal_consistent_with_invariants")["command"],
        load_scenario("design_proposal_consistent_with_invariants"),
    ))
    assert consistent_design["contradiction_detected"] is False
    assert consistent_design["governance_decision"] == "accept"
    assert consistent_design["proposal_consolidated"] is True
    assert consistent_design["revalidation_scheduled"] is False
    assert consistent_design["blocks_release"] is False

    # Project self-audit passes clean and consolidates project health green through the
    # real mutation gateway under memory_consolidation license.
    project_report, project_exit = run_project_audit(strict=True, emit_health=True)
    assert project_exit == 0
    assert project_report["strict_audit"] == "pass"
    assert project_report["violations"] == []
    assert project_report["invariant_count"] == 5
    health = project_report["health_consolidation"]
    assert health["project_cognitive_health_consolidated"] is True
    assert health["project_cognitive_health"] == "green"
    assert health["mutation_log_entry"]["decision"] == "allow"
    assert project_report["ingested_corpus"]["all_non_authoritative"] is True

    # Strict audit must FAIL when a design decision lacks trace / verifier / license.
    incomplete = audit_design_decisions([
        {"decision_id": "DD_untraced", "epistemic_license": "weak_premise", "contradictions": []},
    ])
    assert incomplete["violations"]
    assert incomplete["violations"][0]["reason"] == "missing_audit_fields"
    assert "trace_id" in incomplete["violations"][0]["fields"]
    assert "verifier_assessment" in incomplete["violations"][0]["fields"]

    # A failing strict audit blocks project-health consolidation: the gateway refuses and
    # an AuditViolation is raised, exactly like a safety-critical missing dependency.
    blocked_health = consolidate_project_health({"strict_audit": "fail", "violations": [{"x": 1}]})
    assert blocked_health["project_cognitive_health_consolidated"] is False
    assert blocked_health["project_cognitive_health"] != "green"
    assert blocked_health["audit_violation_packet"]["type"] == "AuditViolationPacket"
    assert blocked_health["mutation_log_entry"]["decision"] == "reject"

    # The runtime adjudicator independently rejects the weakening proposal.
    weaken_decision = evaluate_design_proposal(
        {
            "proposal_id": "DP_probe",
            "claim": "Allow direct action under hazard_only contradiction when urgency is high.",
            "effect": "weaken",
            "targets_invariant": "D_invariant_hazard_blocks_action",
        },
        {
            "memory_id": "D_invariant_hazard_blocks_action",
            "status": "regression_lock",
            "claim": "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
        },
    )
    assert weaken_decision["runtime_adjudication"] == "reject_episode"
    assert weaken_decision["blocks_release"] is True
    assert weaken_decision["contradiction_license"] == "hazard_only"
    assert effect_family(weaken_decision["derived_effect"]) == "weakening"

    # Sprint 25: derived effect classification. The effect is derived from a semantic diff
    # of the claims; a self-declared effect is an untrusted hint, never authority.
    mislabel = audit_design_trace(run(
        load_scenario("design_effect_mislabel_attack")["command"],
        load_scenario("design_effect_mislabel_attack"),
    ))
    assert mislabel["declared_effect"] == "extend"  # the lie
    assert effect_family(mislabel["derived_effect"]) == "weakening"
    assert mislabel["effect_mislabel"] is True
    assert mislabel["conflict_type"] == "hard_contradiction"
    assert mislabel["governance_decision"] == "block"
    assert mislabel["mutation_decision"] == "reject"
    assert mislabel["proposal_consolidated"] is False
    assert mislabel["invariant_preserved"] is True
    assert mislabel["blocks_release"] is True
    assert mislabel["contradiction_license"] == "hazard_only"

    no_declaration = audit_design_trace(run(
        load_scenario("design_effect_derived_without_declaration")["command"],
        load_scenario("design_effect_derived_without_declaration"),
    ))
    assert no_declaration["declared_effect"] is None  # nothing declared; classified from evidence
    assert effect_family(no_declaration["derived_effect"]) == "weakening"
    assert no_declaration["effect_mislabel"] is False
    assert no_declaration["governance_decision"] == "block"
    assert no_declaration["proposal_consolidated"] is False
    assert no_declaration["invariant_preserved"] is True

    preserve = audit_design_trace(run(
        load_scenario("design_effect_preserve_consistent")["command"],
        load_scenario("design_effect_preserve_consistent"),
    ))
    assert preserve["derived_effect"] in ("preserve", "extend")
    assert preserve["effect_mislabel"] is False
    assert preserve["governance_decision"] == "accept"
    assert preserve["proposal_consolidated"] is True

    # A weakening phrased WITHOUT a permissive verb (e.g. "becomes advisory rather than
    # mandatory") and declared 'preserve' is still reclassified to a weakening and blocked.
    lexicon_evasion = audit_design_trace(run(
        load_scenario("design_effect_lexicon_avoiding_weaken")["command"],
        load_scenario("design_effect_lexicon_avoiding_weaken"),
    ))
    assert lexicon_evasion["declared_effect"] == "preserve"
    assert effect_family(lexicon_evasion["derived_effect"]) == "weakening"
    assert lexicon_evasion["effect_mislabel"] is True
    assert lexicon_evasion["governance_decision"] == "block"
    assert lexicon_evasion["proposal_consolidated"] is False
    assert lexicon_evasion["contradiction_license"] == "hazard_only"

    # Touching a protected invariant without preservation evidence cannot be proven safe:
    # it lands in needs_review, which BLOCKS (it must never auto-accept).
    ambiguous = audit_design_trace(run(
        load_scenario("design_effect_ambiguous_needs_review")["command"],
        load_scenario("design_effect_ambiguous_needs_review"),
    ))
    assert ambiguous["derived_effect"] == "needs_review"
    assert ambiguous["governance_decision"] == "block"
    assert ambiguous["proposal_consolidated"] is False

    # The classifier derives the family from evidence regardless of any declared label.
    inv_claim = "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block."
    assert effect_family(derive_effect("Allow direct action under hazard_only when urgency is high.", inv_claim)["effect"]) == "weakening"
    assert effect_family(derive_effect("Add demotion that preserves hazard_only blocking and never lets it lapse.", inv_claim)["effect"]) == "consistent"

    # Backward compat: Sprint 24 honest weaken still blocks and honest extend still accepts,
    # with no false mislabel flag (declared and derived agree on family).
    s24_block = audit_design_trace(run(
        load_scenario("design_contradiction_in_sprint_plan")["command"],
        load_scenario("design_contradiction_in_sprint_plan"),
    ))
    assert s24_block["effect_mislabel"] is False
    assert s24_block["governance_decision"] == "block"
    s24_accept = audit_design_trace(run(
        load_scenario("design_proposal_consistent_with_invariants")["command"],
        load_scenario("design_proposal_consistent_with_invariants"),
    ))
    assert s24_accept["effect_mislabel"] is False
    assert s24_accept["governance_decision"] == "accept"

    # Sprint 26: trace-grounded invariant diff. Words are claims; traces are evidence.
    # The real runtime probes return the protected outcome at baseline.
    assert PROBES["hazard_gate"].run({}) == "reject_episode"
    assert PROBES["consolidation_gate"].run({}) == "rejected"
    # The hazard probe regresses to an allow when urgency is granted override of the gate.
    assert PROBES["hazard_gate"].run({"urgency_overrides_hazard": True}) == "preserve_as_exception"
    # The consolidation probe regresses to applied when an unprivileged source is authorized.
    assert PROBES["consolidation_gate"].run({"authorize_unprivileged_consolidation": True}) == "consolidated"

    # The Sprint-25 laundering hole is closed by behavior: a claim the lexical layer reads as
    # 'preserve', whose behavioral delta flips the hazard gate, is reclassified from the trace.
    launder = audit_design_trace(run(
        load_scenario("preserve_marker_launders_weakening_blocked")["command"],
        load_scenario("preserve_marker_launders_weakening_blocked"),
    ))
    assert launder["lexical_effect"] == "preserve"  # the lexical layer alone is fooled
    assert launder["declared_effect"] == "preserve"  # and the declared effect lies
    assert launder["trace_tested"] is True
    assert launder["trace_regressed"] is True
    assert launder["trace_pre"] == "reject_episode"
    assert launder["trace_post"] == "preserve_as_exception"
    assert launder["effect_authority"] == "trace_behavior_regression"
    assert effect_family(launder["derived_effect"]) == "weakening"
    assert launder["effect_mislabel"] is True
    assert launder["governance_decision"] == "block"
    assert launder["proposal_consolidated"] is False
    assert launder["invariant_preserved"] is True
    assert launder["contradiction_license"] == "hazard_only"

    # A euphemistic claim the lexical layer reads as 'extend' is still blocked from the trace.
    trace_hazard = audit_design_trace(run(
        load_scenario("trace_diff_detects_hazard_gate_softening")["command"],
        load_scenario("trace_diff_detects_hazard_gate_softening"),
    ))
    assert trace_hazard["lexical_effect"] == "extend"
    assert trace_hazard["trace_regressed"] is True
    assert effect_family(trace_hazard["derived_effect"]) == "weakening"
    assert trace_hazard["governance_decision"] == "block"

    # A consolidation-gate softening is caught by the real mutation gateway behavior.
    trace_consolidation = audit_design_trace(run(
        load_scenario("trace_diff_detects_consolidation_gate_softening")["command"],
        load_scenario("trace_diff_detects_consolidation_gate_softening"),
    ))
    assert trace_consolidation["trace_pre"] == "rejected"
    assert trace_consolidation["trace_post"] == "consolidated"
    assert trace_consolidation["trace_regressed"] is True
    assert trace_consolidation["derived_effect"] == "weaken"
    assert trace_consolidation["governance_decision"] == "block"
    assert trace_consolidation["proposal_consolidated"] is False

    # A genuine preserving extension whose delta does not regress the gate is accepted —
    # the gate does not block everything.
    trace_accept = audit_design_trace(run(
        load_scenario("trace_diff_accepts_true_preserving_extension")["command"],
        load_scenario("trace_diff_accepts_true_preserving_extension"),
    ))
    assert trace_accept["trace_tested"] is True
    assert trace_accept["trace_regressed"] is False
    assert trace_accept["effect_authority"] == "trace_confirmed_preservation"
    assert trace_accept["governance_decision"] == "accept"
    assert trace_accept["proposal_consolidated"] is True

    # Preservation must be tested: a preserve/extend claim against a probe-guarded invariant
    # with NO provenance-verified change_set cannot earn accept — it lands in needs_review.
    probed_invariant = {"behavioral_probe": "hazard_gate"}
    untested_trace = derive_effect_from_trace(probed_invariant, {"claim": "preserve the gate"})
    assert untested_trace["status"] == "untested"
    assert combine_effects("preserve", untested_trace)["effect"] == "needs_review"
    assert combine_effects("preserve", untested_trace)["authority"] == "preservation_not_tested"
    # A malformed change_set is untrusted input: it must fail closed to the designed
    # untested block (needs_review), never crash and never launder a weakening to accept.
    for bad_cs in ("not-a-dict", {"target": "hazard_gate", "changed_artifact": "scripts/verifier_engine.py", "patch": ["bad"]}, None):
        malformed = derive_effect_from_trace(probed_invariant, {"change_set": bad_cs})
        assert malformed["status"] == "untested"
        assert malformed["tested"] is False
        assert combine_effects("preserve", malformed)["effect"] == "needs_review"
    # A tested regression (derived from a verified change_set) overrides a lexical 'preserve'.
    regressed_trace = derive_effect_from_trace(
        probed_invariant,
        {"change_set": _change_set("hazard_gate", {"urgency_overrides_hazard": True})},
    )
    assert regressed_trace["regressed"] is True
    assert regressed_trace["provenance"] == "verified"
    assert combine_effects("preserve", regressed_trace)["authority"] == "trace_behavior_regression"
    # Backward compat: the two accept-scenarios now pass on trace-confirmed preservation.
    preserve_compat = audit_design_trace(run(
        load_scenario("design_effect_preserve_consistent")["command"],
        load_scenario("design_effect_preserve_consistent"),
    ))
    assert preserve_compat["trace_tested"] is True
    assert preserve_compat["trace_regressed"] is False
    assert preserve_compat["governance_decision"] == "accept"

    # Sprint 27: complete locked-invariant probe coverage. Every locked invariant is probe-backed,
    # and a locked invariant without a probe is not eligible for preserve/extend acceptance.
    # The three previously lexical-only invariants now have real runtime probes that regress.
    assert PROBES["naked_fact_gate"].run({}) == "cannot_support_action"
    assert PROBES["naked_fact_gate"].run({"allow_naked_facts": True}) == "normal_use"
    assert PROBES["raw_append_only_gate"].run({}) == "append_only_preserved"
    assert PROBES["raw_append_only_gate"].run({"allow_raw_overwrite": True}) != "append_only_preserved"
    assert PROBES["llm_authority_gate"].run({}) == "rejected"
    assert PROBES["llm_authority_gate"].run({"grant_llm_authority": True}) == "consolidated"

    # The three laundering attacks (lexical reads 'preserve') are blocked from the trace.
    for scenario_name, regressed_to in (
        ("trace_diff_blocks_no_naked_facts_laundering", "normal_use"),
        ("trace_diff_blocks_raw_episode_append_only_laundering", "raw_overwrite_blocked_by_store"),
        ("trace_diff_blocks_llm_authority_laundering", "consolidated"),
    ):
        laundered = audit_design_trace(run(
            load_scenario(scenario_name)["command"], load_scenario(scenario_name),
        ))
        assert laundered["lexical_effect"] == "preserve"  # the lexical layer alone is fooled
        assert laundered["trace_regressed"] is True
        assert laundered["trace_post"] == regressed_to
        assert laundered["effect_authority"] == "trace_behavior_regression"
        assert effect_family(laundered["derived_effect"]) == "weakening"
        assert laundered["governance_decision"] == "block"
        assert laundered["proposal_consolidated"] is False
        assert laundered["invariant_preserved"] is True

    # Every locked invariant is probe-backed (no lexical-only locked invariant remains).
    locked_invariants = [inv for inv in load_design_memory()["invariants"] if inv.get("status") == "regression_lock"]
    assert locked_invariants
    for inv in locked_invariants:
        assert inv.get("behavioral_probe") in PROBES, f"locked invariant {inv['memory_id']} has no runtime probe"

    # An ephemeral authorized signer for in-process signed-change tests (private key never persisted).
    _signer_priv_pem = generate_ephemeral_private_key_pem()
    _signer_priv = decode_private_key(_signer_priv_pem)
    authorized = {"design_authority": public_key_pem_from_private_pem(_signer_priv_pem)}

    def _signed(change_set, signer="design_authority", nonce="t"):
        return dict(change_set, signature=sign_change_set(change_set, _signer_priv, signer, nonce))

    # For EACH locked invariant, a genuine preserving extension (a content-verified no-regression
    # change_set) that is validly signed by an authorized signer is accepted.
    for inv in locked_invariants:
        probe_id = inv["behavioral_probe"]
        accepted = evaluate_design_proposal(
            {
                "proposal_id": f"DP_preserve_{probe_id}",
                "claim": "Add audit logging that preserves this invariant and still blocks the protected action.",
                "targets_invariant": inv["memory_id"],
                "change_set": _signed(_change_set(probe_id, {}, adds=True)),
            },
            inv,
            authorized_signers=authorized,
        )
        assert accepted["contradiction_detected"] is False, f"{inv['memory_id']} preserving extension wrongly blocked"
        assert effect_family(accepted["derived_effect"]) == "consistent"
        assert accepted["effect_authority"] == "trace_confirmed_preservation"
        assert accepted["trace_provenance"] == "verified"
        assert accepted["signature_status"] == "signature_verified"
        # The SAME preserving change UNSIGNED blocks on the signature gate (authorship required).
        unsigned = evaluate_design_proposal(
            {
                "proposal_id": f"DP_preserve_unsigned_{probe_id}",
                "claim": "Add audit logging that preserves this invariant and still blocks the protected action.",
                "targets_invariant": inv["memory_id"],
                "change_set": _change_set(probe_id, {}, adds=True),
            },
            inv,
            authorized_signers=authorized,
        )
        assert unsigned["effect_authority"] == "change_signature_unverified"
        assert unsigned["contradiction_detected"] is True

    # Structural rule: a LOCKED invariant with NO probe cannot reach accept — no probe, no proof.
    synthetic_locked_no_probe = {
        "memory_id": "D_locked_no_probe_probe",
        "status": "regression_lock",
        "claim": "Some locked protection that must hold.",
    }
    no_probe = evaluate_design_proposal(
        {
            "proposal_id": "DP_no_probe_preserve",
            "claim": "Add logging that preserves this protection and still holds it.",
            "targets_invariant": "D_locked_no_probe_probe",
        },
        synthetic_locked_no_probe,
    )
    assert no_probe["effect_authority"] == "locked_invariant_without_probe"
    assert no_probe["derived_effect"] == "needs_review"
    assert no_probe["contradiction_detected"] is True  # needs_review blocks
    assert no_probe["blocks_release"] is True
    # An UNLOCKED invariant with no probe keeps the Sprint-25 lexical fallback (does not over-block).
    unlocked_no_probe = evaluate_design_proposal(
        {
            "proposal_id": "DP_unlocked_preserve",
            "claim": "Add logging that preserves this design note and still holds it.",
            "targets_invariant": "D_unlocked_note",
        },
        {"memory_id": "D_unlocked_note", "status": "consolidated", "claim": "An unlocked design note."},
    )
    assert unlocked_no_probe["effect_authority"] == "lexical_only_no_probe"

    # Sprint 28/29: delta-to-code + artifact content-hash provenance. The tested delta is
    # derived from a content-verified change_set; a self-declared behavioral_delta is never
    # trusted. The verifier rejects a tampered diff digest, wrong artifact, and unknown target.
    good_cs = _change_set("hazard_gate", {"urgency_overrides_hazard": True})
    assert verify_change_set_provenance(good_cs)["ok"] is True
    assert verify_change_set_provenance({**good_cs, "diff_digest": "0" * 64})["reason"] == "diff_digest_mismatch"
    assert verify_change_set_provenance({**good_cs, "changed_artifact": "scripts/nope.py"})["reason"] == "artifact_mismatch"
    assert verify_change_set_provenance(None)["reason"] == "missing"

    # A mis-stated no-op behavioral_delta with a weakening change_set patch is blocked: the
    # literal post-image is what is tested, and the declared delta is flagged as not matching.
    misstated = audit_design_trace(run(
        load_scenario("misstated_noop_delta_with_weakening_patch_blocked")["command"],
        load_scenario("misstated_noop_delta_with_weakening_patch_blocked"),
    ))
    assert misstated["trace_provenance"] == "verified"
    assert misstated["delta_matches_change_set"] is False  # declared no-op != literal post-image
    assert misstated["trace_regressed"] is True             # the literal post-image is what is tested
    assert effect_family(misstated["derived_effect"]) == "weakening"
    assert misstated["effect_authority"] == "trace_behavior_regression"
    assert misstated["governance_decision"] == "block"
    assert misstated["proposal_consolidated"] is False

    # A genuine preserving change is accepted, citing the real changed policy artifact.
    prov_accept = audit_design_trace(run(
        load_scenario("derived_delta_matches_patch_accepts_preserving_change")["command"],
        load_scenario("derived_delta_matches_patch_accepts_preserving_change"),
    ))
    assert prov_accept["trace_provenance"] == "verified"
    assert prov_accept["changed_artifact"] == CONTROL_POINT_POLICY_ARTIFACTS["hazard_gate"]
    assert prov_accept["trace_regressed"] is False
    assert prov_accept["governance_decision"] == "accept"
    assert prov_accept["proposal_consolidated"] is True

    # Missing/unverifiable provenance blocks a locked invariant — a self-declared delta is a label.
    for scenario_name in (
        "missing_patch_for_behavioral_delta_needs_review",
        "delta_provenance_required_for_locked_invariant",
    ):
        unprovenanced = audit_design_trace(run(
            load_scenario(scenario_name)["command"], load_scenario(scenario_name),
        ))
        assert unprovenanced["trace_provenance"] == "missing"
        assert unprovenanced["effect_authority"] == "delta_provenance_unverified"
        assert unprovenanced["derived_effect"] == "needs_review"
        assert unprovenanced["governance_decision"] == "block"
        assert unprovenanced["proposal_consolidated"] is False

    # The migrated Sprint 26/27 scenarios now carry a provenance-verified change_set whose
    # changed_artifact is a real file; every committed design change_set must verify.
    for scenario_name in (
        "preserve_marker_launders_weakening_blocked",
        "trace_diff_accepts_true_preserving_extension",
        "trace_diff_blocks_no_naked_facts_laundering",
        "trace_diff_blocks_raw_episode_append_only_laundering",
        "trace_diff_blocks_llm_authority_laundering",
        "derived_delta_matches_patch_accepts_preserving_change",
    ):
        change_set = load_scenario(scenario_name)["design_proposal"].get("change_set")
        assert change_set is not None, f"{scenario_name} lost its change_set"
        assert verify_change_set_provenance(change_set)["ok"] is True, f"{scenario_name} change_set provenance broken"

    # Sprint 29: artifact content-hash binding. The tested delta binds to the literal before/after
    # content of a real on-disk policy artifact; the pre-image hash is recomputed from disk.
    # Every locked invariant's policy artifact's current content yields the protected outcome.
    for inv in locked_invariants:
        probe_id = inv["behavioral_probe"]
        assert PROBES[probe_id].run(load_baseline_policy(probe_id)) == PROBES[probe_id].protected_outcome

    # A stale pre-image (not matching the artifact's real content) is rejected by recomputing from disk.
    stale = audit_design_trace(run(
        load_scenario("stale_pre_image_hash_rejected")["command"],
        load_scenario("stale_pre_image_hash_rejected"),
    ))
    assert stale["trace_provenance"] == "stale_pre_image"
    assert stale["effect_authority"] == "delta_provenance_unverified"
    assert stale["governance_decision"] == "block"
    assert stale["proposal_consolidated"] is False

    # A wrong post-image hash (post content does not hash to its declared hash) is rejected.
    wrong_post = audit_design_trace(run(
        load_scenario("wrong_post_image_hash_rejected")["command"],
        load_scenario("wrong_post_image_hash_rejected"),
    ))
    assert wrong_post["trace_provenance"] == "wrong_post_image"
    assert wrong_post["governance_decision"] == "block"

    # A declared structured patch that diverges from the literal post-image is rejected.
    diverges = audit_design_trace(run(
        load_scenario("structured_patch_diverges_from_literal_diff_blocked")["command"],
        load_scenario("structured_patch_diverges_from_literal_diff_blocked"),
    ))
    assert diverges["trace_provenance"] == "structured_patch_diverges"
    assert diverges["governance_decision"] == "block"

    # A literal-diff weakening (post-image flips a protected key) regresses and blocks, citing pre/post/diff.
    lit_weaken = audit_design_trace(run(
        load_scenario("literal_diff_weakening_change_blocks")["command"],
        load_scenario("literal_diff_weakening_change_blocks"),
    ))
    assert lit_weaken["trace_provenance"] == "verified"
    assert lit_weaken["trace_regressed"] is True
    assert effect_family(lit_weaken["derived_effect"]) == "weakening"
    assert lit_weaken["governance_decision"] == "block"
    assert lit_weaken["pre_image_hash"] and lit_weaken["post_image_hash"] and lit_weaken["diff_digest"]

    # A literal-diff preserving change (benign added key) accepts, citing artifact + pre/post/diff hashes.
    lit_preserve = audit_design_trace(run(
        load_scenario("literal_diff_preserving_change_accepts")["command"],
        load_scenario("literal_diff_preserving_change_accepts"),
    ))
    assert lit_preserve["trace_provenance"] == "verified"
    assert lit_preserve["trace_regressed"] is False
    assert lit_preserve["governance_decision"] == "accept"
    assert lit_preserve["proposal_consolidated"] is True
    assert lit_preserve["changed_artifact"] == CONTROL_POINT_POLICY_ARTIFACTS["hazard_gate"]
    assert lit_preserve["pre_image_hash"] and lit_preserve["post_image_hash"] and lit_preserve["diff_digest"]

    # Content-binding verifier reason codes are real (stale / wrong-post / diff-digest).
    pre_h = load_baseline_policy("hazard_gate")
    valid = build_content_change_set("hazard_gate", pre_h, dict(pre_h, urgency_overrides_hazard=True))
    assert verify_change_set_provenance(valid)["ok"] is True
    assert verify_change_set_provenance({**valid, "pre_image_hash": "0" * 64})["reason"] == "stale_pre_image"
    assert verify_change_set_provenance({**valid, "post_image_hash": "0" * 64})["reason"] == "wrong_post_image"
    assert verify_change_set_provenance(_change_set_stale("hazard_gate", {"urgency_overrides_hazard": True}))["reason"] == "stale_pre_image"
    # Every committed content-bound design change_set verifies against the real on-disk artifact,
    # except the scenarios whose whole point is an unverifiable change_set.
    import glob as _glob
    intentionally_invalid = {
        "stale_pre_image_hash_rejected",
        "wrong_post_image_hash_rejected",
        "structured_patch_diverges_from_literal_diff_blocked",
    }
    for path in _glob.glob("simulations/bridge_world/scenarios/*.json"):
        data = json.loads(Path(path).read_text())
        cs = (data.get("design_proposal") or {}).get("change_set")
        if not (isinstance(cs, dict) and "pre_image" in cs):
            continue
        if cs.get("binding") == "mechanism_source":
            continue  # Sprint 32: verified by the mechanism-source verifier, not the policy one.
        verdict = verify_change_set_provenance(cs)["ok"]
        if data["name"] in intentionally_invalid:
            assert verdict is False, f"{data['name']} should be an unverifiable change_set"
        else:
            assert verdict is True, f"{path} change_set should verify against the real artifact"

    # Sprint 30: signed change provenance. Authorship over the content digest; authorization is
    # necessary-not-sufficient and never overrides a trace regression.
    # Runtime round-trip: a fresh ephemeral signer round-trips, and forgeries are rejected.
    rt_pre = load_baseline_policy("hazard_gate")
    rt_cs = build_content_change_set("hazard_gate", rt_pre, dict(rt_pre, audit_log=True))
    rt_signed = _signed(rt_cs, nonce="rt")
    assert verify_change_signature(rt_signed, authorized)["reason"] == "signature_verified"
    assert verify_change_signature(rt_cs, authorized)["reason"] == "unsigned"
    assert verify_change_signature(rt_signed, {})["reason"] == "unauthorized_signer"
    # Replay: a valid signature copied onto a change_set with different content -> payload mismatch.
    rt_other = build_content_change_set("hazard_gate", rt_pre, dict(rt_pre, audit_log=True, more=True))
    assert verify_change_signature(dict(rt_other, signature=rt_signed["signature"]), authorized)["reason"] == "signature_payload_mismatch"
    # A different authorized key for the same signer name -> wrong_key.
    other_authorized = {"design_authority": public_key_pem_from_private_pem(generate_ephemeral_private_key_pem())}
    assert verify_change_signature(rt_signed, other_authorized)["reason"] == "wrong_key"

    # In-process precedence: a VALIDLY-SIGNED weakening still blocks by trace (authorization
    # never overrides invariant failure).
    signed_weaken = evaluate_design_proposal(
        {
            "proposal_id": "DP_signed_weaken_probe",
            "claim": "Tune the hazard urgency policy while preserving the overall block.",
            "targets_invariant": "D_invariant_hazard_blocks_action",
            "change_set": _signed(_change_set("hazard_gate", {"urgency_overrides_hazard": True})),
        },
        {"memory_id": "D_invariant_hazard_blocks_action", "status": "regression_lock",
         "claim": "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
         "behavioral_probe": "hazard_gate"},
        authorized_signers=authorized,
    )
    assert signed_weaken["signature_status"] == "signature_verified"  # authorship is valid
    assert signed_weaken["trace_regressed"] is True
    assert signed_weaken["effect_authority"] == "trace_behavior_regression"  # ...but trace blocks anyway
    assert signed_weaken["contradiction_detected"] is True

    # Committed static scenarios behave per rubric.
    signed_accept = audit_design_trace(run(
        load_scenario("signed_preserving_change_accepts")["command"],
        load_scenario("signed_preserving_change_accepts"),
    ))
    assert signed_accept["signature_status"] == "signature_verified"
    assert signed_accept["signer"] == "design_authority"
    assert signed_accept["governance_decision"] == "accept"
    assert signed_accept["proposal_consolidated"] is True

    signed_weaken_scn = audit_design_trace(run(
        load_scenario("signed_weakening_change_still_blocks")["command"],
        load_scenario("signed_weakening_change_still_blocks"),
    ))
    assert signed_weaken_scn["signature_status"] == "signature_verified"
    assert signed_weaken_scn["trace_regressed"] is True
    assert signed_weaken_scn["effect_authority"] == "trace_behavior_regression"
    assert signed_weaken_scn["governance_decision"] == "block"
    assert signed_weaken_scn["proposal_consolidated"] is False

    for scenario_name, expected_sig in (
        ("unsigned_content_bound_change_blocks", "unsigned"),
        ("wrong_signer_rejected", "unauthorized_signer"),
        ("signature_replay_against_different_artifact_rejected", "signature_payload_mismatch"),
    ):
        blocked = audit_design_trace(run(load_scenario(scenario_name)["command"], load_scenario(scenario_name)))
        assert blocked["signature_status"] == expected_sig, f"{scenario_name}: {blocked['signature_status']}"
        assert blocked["governance_decision"] == "block"
        assert blocked["proposal_consolidated"] is False
        if expected_sig != "signature_payload_mismatch":
            assert blocked["effect_authority"] == "change_signature_unverified"

    # The committed authorized-signer registry holds only a PUBLIC key (no private key material).
    registry_text = Path("simulations/bridge_world/authorized_design_signers.json").read_text()
    assert "BEGIN PUBLIC KEY" in registry_text
    assert "PRIVATE KEY" not in registry_text

    # Sprint 31: signer-set governance. A public key is not permanent authority — a genuine
    # signature is still subject to scope + lifecycle evaluated at the decision tick, and a
    # governed-but-valid signer can never override a trace failure.
    registry = json.loads(registry_text)
    assert registry["schema"] == "authorized-design-signers-v0.2"
    for signer_id, entry in registry["signers"].items():
        assert isinstance(entry, dict) and entry.get("public_key"), f"{signer_id} not a governed object"
        for field in ("scope", "status", "valid_from_tick", "expires_at_tick", "revoked_at_tick", "rotated_to"):
            assert field in entry, f"{signer_id} missing governance field {field}"
        assert "PRIVATE KEY" not in entry["public_key"]
    # design_authority is preserved as active+wildcard so every Sprint 26–30 signature stays valid.
    assert registry["signers"]["design_authority"]["status"] == "active"
    assert registry["signers"]["design_authority"]["scope"] == ["*"]

    # Decision-time authority (the crux): the SAME genuine signature is authorized before, and
    # rejected after, the signer's revocation tick — authority is evaluated at decision time, not
    # signing time. Built in-process with an ephemeral key (private key never persisted).
    gov_priv_pem = generate_ephemeral_private_key_pem()
    gov_priv = decode_private_key(gov_priv_pem)
    gov_pub = public_key_pem_from_private_pem(gov_priv_pem)
    hz_pre = load_baseline_policy("hazard_gate")
    gov_cs = build_content_change_set("hazard_gate", hz_pre, dict(hz_pre, audit_log=True))
    gov_signed = dict(gov_cs, signature=sign_change_set(gov_cs, gov_priv, "ticking_signer", "g"))
    governed_registry = {
        "ticking_signer": {"public_key": gov_pub, "scope": ["hazard_gate"], "status": "active",
                           "valid_from_tick": 0, "expires_at_tick": None, "revoked_at_tick": 10, "rotated_to": None},
        "predecessor": {"public_key": gov_pub, "scope": ["hazard_gate"], "status": "revoked",
                        "valid_from_tick": 0, "expires_at_tick": None, "revoked_at_tick": 100, "rotated_to": "successor"},
        "successor": {"public_key": gov_pub, "scope": ["hazard_gate"], "status": "active",
                      "valid_from_tick": 100, "expires_at_tick": None, "revoked_at_tick": None, "rotated_to": None},
    }
    assert verify_change_signature(gov_signed, governed_registry, now_tick=5)["reason"] == "signature_verified"
    assert verify_change_signature(gov_signed, governed_registry, now_tick=20)["reason"] == "signer_revoked"
    # Rotation lineage: the same genuine signature is the successor's at tick 150 (accepted) and the
    # revoked predecessor's (rejected) — a revoked predecessor cannot authorize a new change.
    succ_signed = dict(gov_cs, signature=sign_change_set(gov_cs, gov_priv, "successor", "g"))
    pred_signed = dict(gov_cs, signature=sign_change_set(gov_cs, gov_priv, "predecessor", "g"))
    assert verify_change_signature(succ_signed, governed_registry, now_tick=150)["reason"] == "signature_verified"
    assert verify_change_signature(pred_signed, governed_registry, now_tick=150)["reason"] == "signer_revoked"
    # Scope is enforced: a hazard_gate-scoped signature does not authorize a consolidation_gate change.
    cons_pre = load_baseline_policy("consolidation_gate")
    cons_cs = build_content_change_set("consolidation_gate", cons_pre, dict(cons_pre, audit_log=True))
    cons_signed = dict(cons_cs, signature=sign_change_set(cons_cs, gov_priv, "ticking_signer", "g"))
    assert verify_change_signature(cons_signed, governed_registry, now_tick=5)["reason"] == "signer_wrong_scope"
    # Fail-closed scope: an EXPLICIT empty scope [] authorizes NOTHING — it must not collapse to a
    # wildcard (a signer scoped to nothing silently authorizing everything is a fail-open).
    empty_scope_registry = {"ticking_signer": dict(governed_registry["ticking_signer"], scope=[], revoked_at_tick=None)}
    assert verify_change_signature(gov_signed, empty_scope_registry, now_tick=0, change_scope="hazard_gate")["reason"] == "signer_wrong_scope"

    # Committed governance scenarios behave per the Sprint-31 rubric, end-to-end through the demo.
    for scenario_name, sig_status in (
        ("revoked_signer_rejected", "signer_revoked"),
        ("expired_signer_rejected", "signer_expired"),
        ("wrong_scope_signer_rejected", "signer_wrong_scope"),
        ("revoked_key_cannot_replay_prior_signature", "signer_revoked"),
    ):
        blocked = audit_design_trace(run(load_scenario(scenario_name)["command"], load_scenario(scenario_name)))
        assert blocked["signature_status"] == sig_status, f"{scenario_name}: {blocked['signature_status']}"
        assert blocked["effect_authority"] == "change_signature_unverified", scenario_name
        assert blocked["governance_decision"] == "block", scenario_name
        assert blocked["proposal_consolidated"] is False, scenario_name
    # The replay scenario is evaluated AFTER its signer's revocation tick (10), proving the genuine
    # signature does not authorize at the later decision tick.
    replay = audit_design_trace(run(
        load_scenario("revoked_key_cannot_replay_prior_signature")["command"],
        load_scenario("revoked_key_cannot_replay_prior_signature"),
    ))
    assert replay["signer_revoked_at"] == 10 and replay["evaluation_tick"] == 20

    # The rotated successor, in its validity window, is accepted end-to-end.
    rotated = audit_design_trace(run(
        load_scenario("rotated_successor_accepted")["command"],
        load_scenario("rotated_successor_accepted"),
    ))
    assert rotated["signature_status"] == "signature_verified"
    assert rotated["governance_decision"] == "accept"
    assert rotated["proposal_consolidated"] is True
    assert rotated["evaluation_tick"] == 150

    # A weakening validly signed by an active, in-scope governed signer STILL blocks by trace —
    # governance never overrides invariant failure.
    gov_weaken = audit_design_trace(run(
        load_scenario("signed_weakening_still_blocks_under_governance")["command"],
        load_scenario("signed_weakening_still_blocks_under_governance"),
    ))
    assert gov_weaken["signature_status"] == "signature_verified"  # authorship + governance both OK
    assert gov_weaken["trace_regressed"] is True
    assert gov_weaken["effect_authority"] == "trace_behavior_regression"  # ...but trace blocks anyway
    assert gov_weaken["governance_decision"] == "block"
    assert gov_weaken["proposal_consolidated"] is False

    # Content binding + crypto authorship remain prerequisites (governance layers on top, never
    # replaces them): every committed governance scenario's change_set still verifies by content.
    for scenario_name in (
        "revoked_signer_rejected", "expired_signer_rejected", "wrong_scope_signer_rejected",
        "rotated_successor_accepted", "revoked_key_cannot_replay_prior_signature",
        "signed_weakening_still_blocks_under_governance",
    ):
        cs = load_scenario(scenario_name)["design_proposal"]["change_set"]
        assert verify_change_set_provenance(cs)["ok"] is True, f"{scenario_name} change_set provenance broken"

    # Sprint 32: mechanism-source content binding. The mechanism SOURCE (the enforcement code)
    # is bound by content hash; a gate-code weakening is caught by probe even with a clean policy
    # and a valid signature; authorship never overrides the trace.
    # The committed manifest verifies against the real on-disk enforcement code (non-vacuous: a
    # tampered recorded hash is detected).
    manifest_ok = verify_mechanism_manifest()
    assert manifest_ok["ok"] is True, f"mechanism manifest does not verify: {manifest_ok['mismatches']}"
    import copy as _copy
    tampered_manifest = _copy.deepcopy(load_mechanism_manifest())
    tampered_manifest["sources"]["adjudicator"]["content_hash"] = "0" * 64
    tampered = verify_mechanism_manifest(tampered_manifest)
    assert tampered["ok"] is False and any(m["role"] == "adjudicator" for m in tampered["mismatches"])

    # The probe runs the REAL adjudicator source and reproduces the protected outcome; a weakened
    # post-image does NOT, and a broken post-image fails closed (never the protected outcome).
    real_adjudicator = (Path("scripts/verifier_engine.py")).read_text()
    assert probe_outcome_for_proposed_source("adjudicator", real_adjudicator) == "reject_episode"
    weakened_adjudicator = real_adjudicator.replace(
        'if conflict_type == "hard_contradiction" and pressure < 0.45:\n        return "reject_episode"',
        'if conflict_type == "hard_contradiction" and pressure < 0.45:\n        return "preserve_as_exception"',
    )
    assert weakened_adjudicator != real_adjudicator
    assert probe_outcome_for_proposed_source("adjudicator", weakened_adjudicator) != "reject_episode"
    assert probe_outcome_for_proposed_source("adjudicator", "def adjudicate(:\n broken") == "mechanism_probe_error"

    # Committed mechanism-source scenarios behave per the Sprint-32 rubric, end-to-end through the demo.
    hash_mismatch = audit_design_trace(run(
        load_scenario("mechanism_source_hash_mismatch_fails_release")["command"],
        load_scenario("mechanism_source_hash_mismatch_fails_release"),
    ))
    assert hash_mismatch["mechanism_source"] is True
    assert hash_mismatch["trace_provenance"] == "stale_pre_image"
    assert hash_mismatch["effect_authority"] == "delta_provenance_unverified"
    assert hash_mismatch["governance_decision"] == "block"
    assert hash_mismatch["proposal_consolidated"] is False

    unsigned_mech = audit_design_trace(run(
        load_scenario("unsigned_mechanism_source_change_blocks")["command"],
        load_scenario("unsigned_mechanism_source_change_blocks"),
    ))
    assert unsigned_mech["mechanism_source"] is True
    assert unsigned_mech["signature_status"] == "unsigned"
    assert unsigned_mech["effect_authority"] == "change_signature_unverified"
    assert unsigned_mech["governance_decision"] == "block"

    signed_preserve = audit_design_trace(run(
        load_scenario("signed_mechanism_preserving_change_accepts")["command"],
        load_scenario("signed_mechanism_preserving_change_accepts"),
    ))
    assert signed_preserve["mechanism_source"] is True
    assert signed_preserve["mechanism_role"] == "adjudicator"
    assert signed_preserve["signature_status"] == "signature_verified"
    assert signed_preserve["trace_regressed"] is False
    assert signed_preserve["governance_decision"] == "accept"
    assert signed_preserve["proposal_consolidated"] is True

    # A signed WEAKENING of the gate code blocks BY PROBE — authorship never overrides the trace.
    for scenario_name in (
        "signed_mechanism_weakening_change_blocks_by_probe",
        "policy_artifact_clean_but_gate_code_weakened_fails",
    ):
        weaken = audit_design_trace(run(load_scenario(scenario_name)["command"], load_scenario(scenario_name)))
        assert weaken["mechanism_source"] is True, scenario_name
        assert weaken["signature_status"] == "signature_verified", scenario_name  # authorship is valid
        assert weaken["trace_regressed"] is True, scenario_name
        assert weaken["effect_authority"] == "trace_behavior_regression", scenario_name  # ...but probe blocks
        assert weaken["governance_decision"] == "block", scenario_name
        assert weaken["proposal_consolidated"] is False, scenario_name

    # The "policy clean but code weakened" scenario leaves every policy artifact untouched: the
    # weakening lives only in the mechanism source, and is caught by the probe.
    clean_scenario = load_scenario("policy_artifact_clean_but_gate_code_weakened_fails")["design_proposal"]["change_set"]
    assert clean_scenario["changed_artifact"] == "scripts/verifier_engine.py"
    for policy_rel in CONTROL_POINT_POLICY_ARTIFACTS.values():
        assert PROBES[Path(policy_rel).stem].run(load_baseline_policy(Path(policy_rel).stem)) == PROBES[Path(policy_rel).stem].protected_outcome

    # Every committed mechanism-source change_set verifies against the real on-disk gate code,
    # except the one whose whole point is a stale pre-image hash.
    for scenario_name in (
        "unsigned_mechanism_source_change_blocks",
        "signed_mechanism_preserving_change_accepts",
        "signed_mechanism_weakening_change_blocks_by_probe",
        "policy_artifact_clean_but_gate_code_weakened_fails",
    ):
        mech_cs = load_scenario(scenario_name)["design_proposal"]["change_set"]
        assert verify_mechanism_change_provenance(mech_cs)["ok"] is True, scenario_name
    assert verify_mechanism_change_provenance(
        load_scenario("mechanism_source_hash_mismatch_fails_release")["design_proposal"]["change_set"]
    )["reason"] == "stale_pre_image"

    # The project audit reports mechanism_source_binding and gates the strict audit on it.
    project_report, _ = run_project_audit(strict=True)
    assert project_report["mechanism_source_binding"] == "verified"
    assert project_report["strict_audit"] == "pass"


def _planner_world(weather: str, recent_inspection: bool) -> dict:
    return {
        "weather": weather,
        "recent_inspection": recent_inspection,
        "bridges": {
            "A": {"base_minutes": 6, "status": "passable", "rain_exposure": 0.0, "damage_report": False},
            "B": {"base_minutes": 10, "status": "passable", "rain_exposure": 0.0, "damage_report": False},
        },
    }


if __name__ == "__main__":
    main()
