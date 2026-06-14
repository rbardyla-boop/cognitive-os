# Architecture

## System Boundary

Cognitive OS v0.1 is a local cognitive architecture testbed. It accepts a human command for a toy world, converts it into typed packets, routes those packets through inspectable subsystems, and executes only sandboxed toy-world actions.

It does not perform autonomous real-world action. The only executable environment is the local bridge-world simulation.

## Canonical Cognitive Loop

```text
Human Input
  -> Language Codec
  -> IntentPacket
  -> CIP Bus
  -> Attention Manager
  -> Memory Retrieval
  -> Verifier
  -> Planner
  -> Action Engine
  -> ActionOutcome
  -> Episodic Memory
  -> Consolidation Candidate
  -> Human Explanation
```

## Packet Contract

All internal communication uses Cognitive Interchange Protocol packets. Each packet has:

- `header`
- `epistemics`
- `permissions`
- `payload`

Packets are append-only in traces. Memory mutations are explicit packets, never side effects hidden behind planner or verifier code.

## Runtime Modules

- `language_codec`: converts human text into intent and retrieval requests.
- `bus`: routes CIP packets with local in-process priority queues, subscriptions, backpressure, and trace recording.
- `attention`: scores packet admission, selects system mode, issues backpressure, coalesces repeated signals, and enforces time and budget limits.
- `memory`: manages append-only episodic logs, semantic memory nodes, procedural policies, provenance, contradiction indexes, and memory statuses.
- `verifier`: detects conflicts, scores trust, adjudicates claims, and issues epistemic licenses.
- `planner`: builds route plans under constraints and prepares fallback choices.
- `action`: executes only sandboxed toy-world commands and records outcomes.
- `world_model`: encodes structured bridge-world state and provides deterministic risk/cost/outcome prediction stubs.
- `backend`: exposes API and persistence boundaries.
- `ui`: audit dashboard surface.

## Dashboard Views

The dashboard is the audit surface. It should show:

- Live CIP packet stream
- Current system mode
- Priority queue
- Memory node status
- Verifier decisions
- Epistemic licenses
- Planner output
- Action outcome trace
- Deferred jobs
- Backpressure commands
