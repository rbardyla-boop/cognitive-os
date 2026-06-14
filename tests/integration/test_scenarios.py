from bridge_world_demo import load_scenario, run
from contradiction_audit import audit_contradiction_trace
from qa_checks import (
    assert_all_actions_traced,
    assert_degraded_actions_schedule_revalidation,
    assert_memory_mutation_logged,
    assert_mutation_authority_audited,
    assert_no_forbidden_use_reaches_action_engine,
    validate_trace_packets,
)


def packet_types(trace):
    return [packet["header"]["packet_type"] for packet in trace]


def main() -> None:
    scenarios = [
        "normal_crossing",
        "stale_memory_crossing",
        "bridge_conflict",
        "rule_change_cascade",
        "interrupt_storm",
        "adversarial_prompt",
        "bridge_a_safe_time_pressure",
        "false_alarm_damage_report",
        "bridge_b_also_degraded",
        "urgency_spam_attack",
    ]
    for scenario_name in scenarios:
        scenario = load_scenario(scenario_name)
        trace = run(scenario["command"], scenario)
        validate_trace_packets(trace)
        assert "IntentPacket" in packet_types(trace)
        assert "PlanProposal" in packet_types(trace)
        assert "ActionOutcome" in packet_types(trace)
        assert_all_actions_traced(trace)
        assert_degraded_actions_schedule_revalidation(trace)
        assert_no_forbidden_use_reaches_action_engine(trace)
        assert_memory_mutation_logged(trace)

    interrupt = run(load_scenario("interrupt_storm")["command"], load_scenario("interrupt_storm"))
    assert any(packet["payload"].get("source_count") == 1000 for packet in interrupt if packet["header"]["packet_type"] == "SystemStatePacket")

    revision = run(load_scenario("rule_change_cascade")["command"], load_scenario("rule_change_cascade"))
    assert any(
        packet["payload"].get("operation") == "rule_version_cascade"
        and packet["payload"]["cascade"]["frozen"] is False
        for packet in revision
    )

    bridge_a_safety = run(
        load_scenario("bridge_a_safe_time_pressure")["command"],
        load_scenario("bridge_a_safe_time_pressure"),
    )
    assert any(
        packet["header"]["packet_type"] == "RetrievalResult"
        and "M_bridge_a_damage_reported" in str(packet["payload"])
        for packet in bridge_a_safety
    )
    assert any(
        packet["header"]["packet_type"] == "ContradictionPacket"
        and packet["epistemics"]["epistemic_license"] == "hazard_only"
        for packet in bridge_a_safety
    )
    attention = next(packet for packet in bridge_a_safety if packet["header"]["packet_type"] == "BackpressureCommand")
    assert attention["payload"]["system_mode"] in {"Emergency", "Reflex"}
    plan = next(packet for packet in bridge_a_safety if packet["header"]["packet_type"] == "PlanProposal")
    assert plan["payload"]["mode"] == "minimax"
    assert plan["payload"]["route"] == "Bridge B"
    assert any(packet["payload"].get("type") == "post_action_revalidation" for packet in bridge_a_safety)

    false_alarm = run(load_scenario("false_alarm_damage_report")["command"], load_scenario("false_alarm_damage_report"))
    repair = [
        packet for packet in false_alarm
        if packet["header"]["packet_type"] == "MemoryMutation"
        and packet["payload"].get("operation") == "semantic_status_update"
    ]
    assert repair
    assert repair[0]["payload"]["memory_id"] == "M_bridge_a_damage_reported"
    assert repair[0]["payload"]["new_status"] == "superseded"

    b_degraded = run(load_scenario("bridge_b_also_degraded")["command"], load_scenario("bridge_b_also_degraded"))
    b_plan = next(packet for packet in b_degraded if packet["header"]["packet_type"] == "PlanProposal")
    assert b_plan["payload"]["mode"] == "minimax"
    assert b_plan["payload"]["route"] == "wait"
    assert "No crossing route" in b_plan["payload"]["risk_note"]

    spam = run(load_scenario("urgency_spam_attack")["command"], load_scenario("urgency_spam_attack"))
    spam_intent = next(packet for packet in spam if packet["header"]["packet_type"] == "IntentPacket")
    assert spam_intent["payload"]["urgency"] == "high"
    assert any(packet["epistemics"]["epistemic_license"] == "hazard_only" for packet in spam if packet["header"]["packet_type"] == "ContradictionPacket")
    spam_command = next(packet for packet in spam if packet["header"]["packet_type"] == "ActionCommand")
    assert "direct_action" in spam_command["permissions"]["forbidden_use"]
    assert spam_command["payload"]["route"] != "Bridge A"

    direct = run(
        load_scenario("direct_mutation_without_verifier")["command"],
        load_scenario("direct_mutation_without_verifier"),
    )
    assert_mutation_authority_audited(direct)
    direct_mutation = next(packet for packet in direct if packet["header"]["packet_type"] == "MemoryMutation")
    assert direct_mutation["payload"]["operation"] == "mutation_rejected"
    assert direct_mutation["payload"]["mutation_log_entry"]["reason"] == "missing verifier_decision_id"
    assert direct_mutation["payload"]["mutation_log_entry"]["before_status"] == direct_mutation["payload"]["mutation_log_entry"]["after_status"]

    low_authority = run(
        load_scenario("memory_mutation_with_low_authority_packet")["command"],
        load_scenario("memory_mutation_with_low_authority_packet"),
    )
    assert_mutation_authority_audited(low_authority)
    low_mutation = next(packet for packet in low_authority if packet["header"]["packet_type"] == "MemoryMutation")
    assert low_mutation["payload"]["operation"] == "mutation_rejected"
    assert "source packet authority forbids" in low_mutation["payload"]["mutation_log_entry"]["reason"]

    promotion = run(
        load_scenario("valid_human_promotion_allows_invariant")["command"],
        load_scenario("valid_human_promotion_allows_invariant"),
    )
    assert_mutation_authority_audited(promotion)
    promoted = next(packet for packet in promotion if packet["header"]["packet_type"] == "MemoryMutation")
    assert promoted["payload"]["operation"] == "mutation_applied"
    assert promoted["payload"]["mutation_log_entry"]["before_status"] == "bootstrap_candidate"
    assert promoted["payload"]["mutation_log_entry"]["after_status"] == "promoted_invariant"

    success = run(
        load_scenario("degraded_action_success_does_not_overconfirm")["command"],
        load_scenario("degraded_action_success_does_not_overconfirm"),
    )
    success_corrections = _post_action_corrections(success)
    assert [packet["payload"]["mutation_log_entry"]["target_object_id"] for packet in success_corrections] == [
        "PROC_use_stable_bridge_under_rain",
        "M_bridge_a_damage_reported",
    ]
    assert success_corrections[-1]["payload"]["mutation_log_entry"]["after_status"] == "retest_required"
    assert success_corrections[-1]["payload"]["overconfirmation_blocked"] is True
    assert success_corrections[0]["payload"]["target_kind"] == "procedure"
    assert success_corrections[1]["payload"]["target_kind"] == "belief"

    failure = run(
        load_scenario("degraded_action_failure_quarantines_memory")["command"],
        load_scenario("degraded_action_failure_quarantines_memory"),
    )
    failure_corrections = _post_action_corrections(failure)
    assert failure_corrections[0]["payload"]["mutation_log_entry"]["after_status"] == "quarantined"
    assert failure_corrections[1]["payload"]["mutation_log_entry"]["after_status"] == "quarantined"

    partial = run(
        load_scenario("degraded_action_partial_success_scopes_memory")["command"],
        load_scenario("degraded_action_partial_success_scopes_memory"),
    )
    partial_corrections = _post_action_corrections(partial)
    assert partial_corrections[0]["payload"]["mutation_log_entry"]["after_status"] == "exception_scoped_policy"
    assert partial_corrections[1]["payload"]["mutation_log_entry"]["after_status"] == "exception_scoped"
    assert partial_corrections[0]["payload"]["scope_conditions"]["constraint"] == "abort_path_preserved"
    assert partial_corrections[1]["payload"]["scope_conditions"]["constraint"] == "damage_report_not_globally_resolved"

    resolved = audit_contradiction_trace(run(
        load_scenario("contradiction_resolved_by_new_evidence")["command"],
        load_scenario("contradiction_resolved_by_new_evidence"),
    ))
    assert resolved["mutation_order"] == ["M_bridge_a_damage_reported", "M_bridge_a_passable"]
    assert resolved["raw_episodes_preserved"] is True

    scoped = audit_contradiction_trace(run(
        load_scenario("contradiction_scoped_by_context")["command"],
        load_scenario("contradiction_scoped_by_context"),
    ))
    assert scoped["mutations"][0]["after"] == "exception_scoped"
    assert scoped["mutations"][1]["after"] == "exception_scoped"

    unresolved = audit_contradiction_trace(run(
        load_scenario("contradiction_remains_unresolved")["command"],
        load_scenario("contradiction_remains_unresolved"),
    ))
    assert unresolved["unresolved_visible"] is True
    assert unresolved["strict_action_blocked"] is True
    unresolved_trace = run(
        load_scenario("contradiction_remains_unresolved")["command"],
        load_scenario("contradiction_remains_unresolved"),
    )
    retrieval = next(
        packet for packet in unresolved_trace
        if packet["header"]["packet_type"] == "RetrievalResult"
        and packet["payload"].get("retrieval_after_repair")
    )
    assert any(
        item["content"]["memory_id"] == "M_bridge_a_passable"
        and item["status"] == "contradicted"
        and item["contradictions"]
        for item in retrieval["payload"]["semantic_nodes"]
    )


def _post_action_corrections(trace):
    return [
        packet for packet in trace
        if packet["header"]["packet_type"] == "MemoryMutation"
        and packet["payload"].get("operation") == "post_action_correction"
    ]


if __name__ == "__main__":
    main()
