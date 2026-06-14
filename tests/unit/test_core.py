from pathlib import Path

from attention_manager import score_packet
from bridge_world_demo import packet, permissions, require_packet_use
from governed_memory import GovernedMemory
from qa_checks import validate_packet_envelope
from retrieval_policy import emergency_use_protocol
from verifier_engine import detect_conflict


def main() -> None:
    sample = packet(
        "P_TEST",
        "T_TEST",
        "IntentPacket",
        "language_codec",
        "bus",
        "P2",
        100,
        {
            "confidence": 0.7,
            "uncertainty_type": "user_assertion",
            "epistemic_license": "hypothesis_only",
            "provenance": [],
            "contradictions": [],
        },
        permissions(["retrieval", "planning_with_fallback"], ["direct_action"]),
        {"goal": "cross", "target": "bridge_A"},
    )
    validate_packet_envelope(sample)
    require_packet_use(sample, "retrieval")
    try:
        require_packet_use(sample, "direct_action")
        raise AssertionError("direct action should be denied")
    except PermissionError:
        pass

    assert emergency_use_protocol("full_premise", urgent=True) == "normal_use"
    assert emergency_use_protocol("weak_premise", urgent=True) == "use_with_fallback"
    assert emergency_use_protocol("hypothesis_only", urgent=True) == "branch_alternatives"
    assert emergency_use_protocol("hazard_only", urgent=True) == "warning_only"
    assert emergency_use_protocol("do_not_use_for_action", urgent=True) == "cannot_support_action"

    assert score_packet(sample) > 0

    memory = GovernedMemory(Path("simulations/bridge_world"))
    retrieval = memory.retrieve({"preferred_bridge": "Bridge A"}, {"weather": "heavy_rain", "time_budget_minutes": 12}, [])
    statuses = {item["status"] for item in retrieval["semantic_nodes"]}
    assert "confidence_reduced" in statuses
    assert retrieval["contradictions"]
    assert any(
        item["content"].get("memory_id") == "M_bridge_a_damage_reported"
        for item in retrieval["semantic_nodes"] + retrieval["contradictions"]
    )

    conflict = detect_conflict(
        {"claim": "Bridge A is passable in clear or light rain.", "status": "confidence_reduced"},
        {"claim": "Avoid Bridge A during heavy rain unless verified open.", "applies_to": ["Bridge A"]},
        {"weather": "heavy_rain"},
    )
    assert conflict == "hard_contradiction"


if __name__ == "__main__":
    main()
