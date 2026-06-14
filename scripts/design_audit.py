#!/usr/bin/env python3
"""Design-governance audit replay for Sprint 24 (the Caitlin leap).

Reconstructs why a design proposal was accepted or blocked, using the same trace
the bridge world produces. A proposal that weakens a locked invariant must be
blocked by a hazard_only ContradictionPacket, denied consolidation through the
mutation gateway, and routed into a deferred design-revalidation job.
"""

from __future__ import annotations

import json
import sys

from bridge_world_demo import load_scenario, run


def audit_design_trace(trace: list[dict]) -> dict:
    by_type: dict[str, list[dict]] = {}
    for packet in trace:
        by_type.setdefault(packet["header"]["packet_type"], []).append(packet)

    retrieval = by_type["RetrievalResult"][0]
    plan = by_type["PlanProposal"][0]
    mutation = next(
        packet for packet in by_type.get("MemoryMutation", [])
        if "proposal_id" in packet["payload"]
    )
    contradictions = [
        packet for packet in by_type.get("ContradictionPacket", [])
        if "proposal_id" in packet["payload"]
    ]
    revalidations = [
        packet for packet in by_type.get("BackpressureCommand", [])
        if packet["payload"].get("type") == "design_revalidation"
    ]

    mutation_payload = mutation["payload"]
    contradiction = contradictions[0] if contradictions else None
    return {
        "design_decision": plan["payload"]["route"],
        "proposal_id": plan["payload"]["proposal_id"],
        "effect": mutation_payload["effect"],
        "declared_effect": mutation_payload.get("declared_effect"),
        "derived_effect": mutation_payload.get("derived_effect"),
        "lexical_effect": mutation_payload.get("lexical_effect"),
        "trace_effect": mutation_payload.get("trace_effect"),
        "trace_tested": mutation_payload.get("trace_tested"),
        "trace_regressed": mutation_payload.get("trace_regressed"),
        "trace_pre": mutation_payload.get("trace_pre"),
        "trace_post": mutation_payload.get("trace_post"),
        "trace_provenance": mutation_payload.get("trace_provenance"),
        "mechanism_source": mutation_payload.get("mechanism_source"),
        "mechanism_role": mutation_payload.get("mechanism_role"),
        "changed_artifact": mutation_payload.get("changed_artifact"),
        "pre_image_hash": mutation_payload.get("pre_image_hash"),
        "post_image_hash": mutation_payload.get("post_image_hash"),
        "diff_digest": mutation_payload.get("diff_digest"),
        "delta_matches_change_set": mutation_payload.get("delta_matches_change_set"),
        "signer": mutation_payload.get("signer"),
        "signature_status": mutation_payload.get("signature_status"),
        "signed_payload_digest": mutation_payload.get("signed_payload_digest"),
        "signer_status": mutation_payload.get("signer_status"),
        "signer_scope": mutation_payload.get("signer_scope"),
        "signer_expires_at": mutation_payload.get("signer_expires_at"),
        "signer_revoked_at": mutation_payload.get("signer_revoked_at"),
        "signer_rotated_to": mutation_payload.get("signer_rotated_to"),
        "evaluation_tick": mutation_payload.get("evaluation_tick"),
        "effect_authority": mutation_payload.get("effect_authority"),
        "effect_mislabel": mutation_payload.get("effect_mislabel"),
        "effect_basis": mutation_payload.get("effect_basis"),
        "targets_invariant": mutation_payload["targets_invariant"],
        "invariant_retrieved_with_license": retrieval["epistemics"]["epistemic_license"],
        "naked_fact": retrieval["payload"].get("naked_fact", True),
        "contradiction_detected": contradiction is not None,
        "contradiction_license": (
            contradiction["epistemics"]["epistemic_license"] if contradiction else None
        ),
        "conflict_type": contradiction["payload"]["conflict_type"] if contradiction else "no_conflict",
        "verifier_rule_id": plan["payload"]["verifier_rule_id"],
        "governance_decision": plan["payload"]["decision"],
        "blocks_release": plan["payload"]["blocks_release"],
        "proposal_consolidated": mutation_payload["proposal_consolidated"],
        "invariant_preserved": mutation_payload["invariant_preserved"],
        "mutation_decision": mutation_payload["mutation_log_entry"]["decision"],
        "revalidation_scheduled": bool(revalidations),
        "trace_id": trace[0]["header"]["trace_id"],
    }


def main() -> int:
    if len(sys.argv) >= 3 and sys.argv[1] == "--scenario":
        scenario = load_scenario(sys.argv[2])
        trace = run(scenario["command"], scenario)
    else:
        raise SystemExit("design_audit requires --scenario <name>")
    print(json.dumps(audit_design_trace(trace), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
