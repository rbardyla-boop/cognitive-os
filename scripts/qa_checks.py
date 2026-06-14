"""QA gate helpers for local Cognitive OS tests."""

from __future__ import annotations


REQUIRED_PACKET_TOP_LEVEL = {"header", "epistemics", "permissions", "payload"}
REQUIRED_HEADER = {
    "packet_id",
    "packet_type",
    "schema_version",
    "source_engine",
    "target_engine",
    "trace_id",
    "created_at",
    "priority",
    "time_budget_ms",
}
REQUIRED_EPISTEMICS = {
    "confidence",
    "uncertainty_type",
    "epistemic_license",
    "provenance",
    "contradictions",
}
REQUIRED_PERMISSIONS = {"allowed_use", "forbidden_use"}


def validate_packet_envelope(packet: dict) -> None:
    missing = REQUIRED_PACKET_TOP_LEVEL.difference(packet)
    if missing:
        raise AssertionError(f"packet missing top-level keys: {sorted(missing)}")
    if REQUIRED_HEADER.difference(packet["header"]):
        raise AssertionError("packet header is incomplete")
    if REQUIRED_EPISTEMICS.difference(packet["epistemics"]):
        raise AssertionError("packet epistemics are incomplete")
    if REQUIRED_PERMISSIONS.difference(packet["permissions"]):
        raise AssertionError("packet permissions are incomplete")
    if packet["permissions"]["allowed_use"] and set(packet["permissions"]["allowed_use"]).intersection(packet["permissions"]["forbidden_use"]):
        raise AssertionError("packet has overlapping allowed/forbidden uses")


def validate_trace_packets(trace: list[dict]) -> None:
    for packet in trace:
        validate_packet_envelope(packet)


def assert_all_actions_traced(trace: list[dict]) -> None:
    packets = {packet["header"]["packet_id"]: packet for packet in trace}
    commands = [packet for packet in trace if packet["header"]["packet_type"] == "ActionCommand"]
    if not commands:
        raise AssertionError("trace has no action command")
    for command in commands:
        command_id = command["header"]["packet_id"]
        outcomes = [
            packet for packet in trace
            if packet["header"]["packet_type"] == "ActionOutcome"
            and packet["epistemics"]["provenance"]
            and packet["epistemics"]["provenance"][0].get("packet_id") == command_id
        ]
        if not outcomes:
            raise AssertionError(f"action {command_id} has no outcome")
        outcome_id = outcomes[0]["header"]["packet_id"]
        episode = _first_with_provenance(trace, "EpisodePacket", outcome_id)
        mutation = _first_with_provenance(trace, "MemoryMutation", outcome_id)
        if episode is None or mutation is None:
            raise AssertionError(f"action outcome {outcome_id} is not fully logged")
        if command_id not in packets:
            raise AssertionError(f"action command missing from packet map: {command_id}")


def assert_degraded_actions_schedule_revalidation(trace: list[dict]) -> None:
    used_degraded_memory = any(
        packet["header"]["packet_type"] == "RetrievalResult"
        and _payload_has_value(packet["payload"], "post_action_revalidation")
        for packet in trace
    )
    if used_degraded_memory:
        has_job = any(
            packet["header"]["packet_type"] == "BackpressureCommand"
            and packet["payload"].get("type") == "post_action_revalidation"
            for packet in trace
        )
        if not has_job:
            raise AssertionError("degraded action did not schedule post_action_revalidation")


def assert_no_forbidden_use_reaches_action_engine(trace: list[dict]) -> None:
    for packet in trace:
        if packet["header"]["target_engine"] != "action":
            continue
        if "sandbox_testing" not in packet["permissions"]["allowed_use"]:
            raise AssertionError(f"packet {packet['header']['packet_id']} reached action without sandbox permission")
        if "direct_action" in packet["permissions"]["allowed_use"]:
            raise AssertionError(f"packet {packet['header']['packet_id']} allowed direct_action at action engine")


def assert_memory_mutation_logged(trace: list[dict]) -> None:
    if not any(packet["header"]["packet_type"] == "MemoryMutation" for packet in trace):
        raise AssertionError("trace has no memory mutation")


def assert_mutation_authority_audited(trace: list[dict]) -> None:
    audited = [
        packet for packet in trace
        if packet["header"]["packet_type"] == "MemoryMutation"
        and "mutation_log_entry" in packet["payload"]
    ]
    if not audited:
        raise AssertionError("trace has no mutation authority audit entry")
    for packet in audited:
        log = packet["payload"]["mutation_log_entry"]
        for field in {
            "mutation_id",
            "timestamp",
            "trace_id",
            "target_object_id",
            "mutation_type",
            "requested_use",
            "source_packet_id",
            "verifier_decision_id",
            "decision",
            "reason",
            "before_status",
            "after_status",
        }:
            if field not in log:
                raise AssertionError(f"mutation audit log missing {field}")
        if not packet["payload"].get("mutation_log"):
            raise AssertionError("mutation packet did not expose append-only mutation log")


def assert_packet_has_provenance(packet: dict) -> None:
    if not packet["epistemics"]["provenance"]:
        raise AssertionError(f"packet {packet['header']['packet_id']} has no provenance")


def assert_no_hidden_contradiction_payload(packet: dict) -> None:
    if packet["header"]["packet_type"] == "ContradictionPacket":
        return
    if _payload_has_key(packet["payload"], "hidden_contradiction"):
        raise AssertionError("contradiction hidden in non-contradiction payload")


def _first_with_provenance(trace: list[dict], packet_type: str, provenance_packet_id: str) -> dict | None:
    for packet in trace:
        if packet["header"]["packet_type"] != packet_type:
            continue
        if any(ref.get("packet_id") == provenance_packet_id for ref in packet["epistemics"]["provenance"]):
            return packet
    return None


def _payload_has_value(value, needle: str) -> bool:
    if isinstance(value, dict):
        return any(_payload_has_value(child, needle) for child in value.values())
    if isinstance(value, list):
        return any(_payload_has_value(child, needle) for child in value)
    return value == needle


def _payload_has_key(value, needle: str) -> bool:
    if isinstance(value, dict):
        return needle in value or any(_payload_has_key(child, needle) for child in value.values())
    if isinstance(value, list):
        return any(_payload_has_key(child, needle) for child in value)
    return False
