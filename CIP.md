# Cognitive Interchange Protocol

Version `0.1`.

## Envelope

Every packet has four top-level sections:

- `header`: identity, routing, schema version, trace, priority, and time budget.
- `epistemics`: confidence, uncertainty type, epistemic license, provenance, and contradictions.
- `permissions`: allowed and forbidden uses.
- `payload`: packet-specific content.

```json
{
  "header": {
    "packet_id": "P_0001",
    "packet_type": "ClaimPacket",
    "schema_version": "0.1",
    "source_engine": "language_codec",
    "target_engine": "verifier",
    "trace_id": "T_0001",
    "created_at": "2026-06-12T12:00:00Z",
    "priority": "P2",
    "time_budget_ms": 100
  },
  "epistemics": {
    "confidence": 0.74,
    "uncertainty_type": "user_assertion",
    "epistemic_license": "hypothesis_only",
    "provenance": [],
    "contradictions": []
  },
  "permissions": {
    "allowed_use": ["planning_with_fallback"],
    "forbidden_use": ["direct_action", "memory_consolidation"]
  },
  "payload": {}
}
```

## Essential Packet Types

- `IntentPacket`
- `ClaimPacket`
- `EvidencePacket`
- `EpisodePacket`
- `RulePacket`
- `RetrievalRequest`
- `RetrievalResult`
- `ContradictionPacket`
- `PlanProposal`
- `ActionCommand`
- `ActionOutcome`
- `MemoryMutation`
- `SystemStatePacket`
- `BackpressureCommand`
- `HumanPromotionPacket` (Sprint 10 mutation authority only)
- `PlanRegretPacket` (Sprint 14 planner policy feedback only)
- `AttentionModeReviewPacket` (Sprint 15 attention policy feedback only)

## Epistemic Licenses

- `full_premise`: can support direct planning.
- `weak_premise`: can support planning with fallback.
- `hypothesis_only`: must branch alternatives.
- `hazard_only`: may warn, cannot support action.
- `do_not_use_for_action`: retrieval/display only.

## Permission Uses

Common allowed uses:

- `retrieval`
- `planning_with_fallback`
- `human_explanation`
- `contradiction_detection`
- `sandbox_testing`

Common forbidden uses:

- `direct_action`
- `memory_consolidation`
- `rule_revision`
- `safety_certification`

## Gate Rule

No engine may act on `payload` until it checks `permissions.allowed_use`, `permissions.forbidden_use`, and the packet's epistemic license.

## Priority Lanes

- `P0`: safety interrupt.
- `P1`: active action correction.
- `P2`: active goal relevance.
- `P3`: contradiction/anomaly.
- `P4`: memory maintenance.
- `P5`: curiosity/background learning.
- `P6`: archival/compression.

## Local Broker

The v0.1 bus is an in-process broker. It deliberately does not use a distributed queue, Kafka, RabbitMQ, or cloud infrastructure.

Core operations:

- `publish(packet)`
- `subscribe(engine, packet_type)`
- `poll(engine)`
- `ack(packet_id)`
- `defer(packet_id, reason)`
- `dead_letter(packet_id, reason)`

Given any action packet, QA should be able to follow the shared `trace_id` and packet provenance back through the plan, retrieval result, retrieval request, and original intent.

## Attention And Budget

System modes:

- `Reflective`: deep reasoning allowed.
- `Operational`: normal planning.
- `Strained`: defer consolidation.
- `Emergency`: minimax/safety only.
- `Reflex`: precompiled policy only.
- `Recovery`: replay deferred packets.

Packet admission score:

```text
A(p) =
safety * urgency * goal_relevance * expected_confidence_delta * time_sensitivity
/
compute_cost * latency_cost
```

For v0.1 the weights are hardcoded in the local attention manager.

Backpressure commands target memory with `reduce_output`, `max_results`, preserved result classes, and deferred background jobs when the system is strained or worse.

Repeated low-level anomaly packets are coalesced into a `SystemStatePacket` trend. The interrupt storm scenario compresses 1000 Bridge A anomaly signals into one trend packet.
