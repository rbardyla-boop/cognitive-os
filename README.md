# Cognitive OS

Local prototype for a cognitive architecture testbed. A human command enters as language, becomes a typed internal packet, flows across a cognitive bus, retrieves memory, gets verified, receives an epistemic license, passes through attention and budget control, produces a plan, executes a toy action, records the outcome, and updates memory.

The first environment is `bridge_world`: two bridges, changing weather, stale memories, conflicting rules, and time pressure.

## Quick Start

Run the local demo:

```sh
./scripts/dev.sh
```

Run checks:

```sh
./scripts/test.sh
./scripts/release_check.sh
```

## Requirements / Interpreter

Python >= 3.10 (tested on 3.12.3). One third-party dependency: `cryptography==41.0.7` (Ed25519 signing) — see `requirements.txt`.

The default `python3` in some shells is an unrelated virtualenv that lacks `cryptography`, so the scripts fail with `ModuleNotFoundError`. The correct interpreter is the system `/usr/bin/python3`. Prefix python invocations so it wins on PATH:

```sh
PATH=/usr/bin:$PATH bash scripts/release_check.sh
```

See [ENVIRONMENT.md](ENVIRONMENT.md) for the full runtime lock (interpreter, determinism, and no-network guarantees).

## Current Scope

This repository is a v0.1 testbed, not a general autonomous agent. It has no real-world actuation, no unsupervised internet access, no self-modifying model weights, and no hidden memory mutation. Every meaningful decision should become an inspectable packet or trace entry.

See [CIP.md](CIP.md) for the packet envelope, packet type list, epistemic licenses, and permission gate.

See [MEMORY.md](MEMORY.md) for governed episodic, semantic, procedural, and contradiction memory.

See [VERIFIER.md](VERIFIER.md) for trust scoring, conflict detection, adjudication outcomes, and revision pressure.

See [RULE_MIGRATION.md](RULE_MIGRATION.md) for versioned rules, dependency tracing, impact scoring, and lazy cascade evaluation.

See [LANGUAGE_CODEC.md](LANGUAGE_CODEC.md) for deterministic parsing, renderer boundaries, and the no-prose internal handoff rule.

See [WORLD_ENCODER.md](WORLD_ENCODER.md) for structured toy-world state and deterministic prediction stubs.

See [BACKEND.md](BACKEND.md) for SQLite storage, migrations, and local API endpoints.

See [QA_PLAN.md](QA_PLAN.md) for unit, integration, adversarial, and regression gates.

See [RELEASE_NOTES.md](RELEASE_NOTES.md), [CHANGELOG.md](CHANGELOG.md), and [KNOWN_LIMITATIONS.md](KNOWN_LIMITATIONS.md) for release management.

See [TRACE_AUDIT.md](TRACE_AUDIT.md) for replaying packet traces into decision audits.

See [SPRINT_9_PLAN.md](SPRINT_9_PLAN.md), [FAILURE_LEDGER.md](FAILURE_LEDGER.md), and review notes for the self-correcting build process.

See [SPRINT_10_PLAN.md](SPRINT_10_PLAN.md) and [MUTATION_AUTHORITY.md](MUTATION_AUTHORITY.md) for mutation authority and direct-call enforcement.

See [SPRINT_11_PLAN.md](SPRINT_11_PLAN.md) and [CORRECTION_LOOPS.md](CORRECTION_LOOPS.md) for post-action revalidation and belief/procedure separation.

See [SPRINT_12_PLAN.md](SPRINT_12_PLAN.md) and [CONTRADICTION_REPAIR.md](CONTRADICTION_REPAIR.md) for governed contradiction repair.

See [SPRINT_13_PLAN.md](SPRINT_13_PLAN.md) and [EPISTEMIC_SNAPSHOT.md](EPISTEMIC_SNAPSHOT.md) for live epistemic operating-state snapshots.

See [SPRINT_14_PLAN.md](SPRINT_14_PLAN.md) and [PLANNER_REGRET.md](PLANNER_REGRET.md) for planner regret and policy review.

See [SPRINT_15_PLAN.md](SPRINT_15_PLAN.md) and [ATTENTION_REVIEW.md](ATTENTION_REVIEW.md) for attention mode review and recovery replay.

See [SPRINT_16_PLAN.md](SPRINT_16_PLAN.md) and [RECOVERY_REPLAY.md](RECOVERY_REPLAY.md) for the unified correction queue and deterministic recovery replay.

See [SPRINT_24_PLAN.md](SPRINT_24_PLAN.md) and [DESIGN_REVIEW_NOTES.md](DESIGN_REVIEW_NOTES.md) for unified self-correction (the Caitlin leap): the development process governed by the same machinery as the runtime.

See [SPRINT_25_PLAN.md](SPRINT_25_PLAN.md) through [SPRINT_32_PLAN.md](SPRINT_32_PLAN.md) and [GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md) for the S25–S32 governance lineage: derived effect, trace-grounded invariants, content binding, signed provenance, signer governance, and mechanism-source integrity.

## Canonical Loop

Human Input -> Language Codec -> IntentPacket -> CIP Bus -> Attention Manager -> Memory Retrieval -> Verifier -> Planner -> Action Engine -> ActionOutcome -> Episodic Memory -> Consolidation Candidate -> Human Explanation

The invariants protecting this loop are themselves governed through Sprints 25–32, so a weakening of the verifier, mutation gateway, or a probe is caught by a behavioral test before release. See [GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md).
