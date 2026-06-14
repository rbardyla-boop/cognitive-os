from bridge_world_demo import load_scenario, run


def main() -> None:
    normal = run(load_scenario("normal_crossing")["command"], load_scenario("normal_crossing"))
    normal_plan = next(packet for packet in normal if packet["header"]["packet_type"] == "PlanProposal")
    assert normal_plan["payload"]["route"] in {"Bridge A", "Bridge B"}

    stale = run(load_scenario("stale_memory_crossing")["command"], load_scenario("stale_memory_crossing"))
    assert any(packet["header"]["packet_type"] == "ContradictionPacket" for packet in stale)

    conflict = run(load_scenario("bridge_conflict")["command"], load_scenario("bridge_conflict"))
    assert any(packet["payload"].get("conflict_type") == "hard_contradiction" for packet in conflict if packet["header"]["packet_type"] == "ContradictionPacket")

    safety_query = run(
        load_scenario("bridge_a_safe_time_pressure")["command"],
        load_scenario("bridge_a_safe_time_pressure"),
    )
    plan = next(packet for packet in safety_query if packet["header"]["packet_type"] == "PlanProposal")
    assert plan["payload"]["mode"] == "minimax"
    assert plan["payload"]["route"] == "Bridge B"
    outcome = next(packet for packet in safety_query if packet["header"]["packet_type"] == "ActionOutcome")
    assert outcome["payload"]["action"] == "cross_bridge_B"

    b_degraded = run(load_scenario("bridge_b_also_degraded")["command"], load_scenario("bridge_b_also_degraded"))
    plan = next(packet for packet in b_degraded if packet["header"]["packet_type"] == "PlanProposal")
    assert plan["payload"]["route"] == "wait"

    false_alarm = run(load_scenario("false_alarm_damage_report")["command"], load_scenario("false_alarm_damage_report"))
    assert any(
        packet["payload"].get("operation") == "semantic_status_update"
        and packet["payload"].get("new_status") == "superseded"
        for packet in false_alarm
    )

    success = run(
        load_scenario("degraded_action_success_does_not_overconfirm")["command"],
        load_scenario("degraded_action_success_does_not_overconfirm"),
    )
    assert any(
        packet["payload"].get("operation") == "post_action_correction"
        and packet["payload"]["mutation_log_entry"]["after_status"] == "retest_required"
        for packet in success
    )


if __name__ == "__main__":
    main()
