# Roadmap

## v0.1: Local Testbed

- Create repository skeleton and design docs.
- Define CIP schemas for core packet types.
- Implement bridge-world demo loop.
- Seed scenarios for normal, stale memory, conflict, interrupt storm, and adversarial prompt cases.
- Emit a full trace for each demo run.

## v0.1 (COMPLETE): Development-Process Governance (Sprints 24–32)

v0.1 extends beyond the runtime loop to govern the development process itself: unified self-correction (S24), derived effect classification (S25), trace-grounded invariant validation (S26), complete probe coverage of all five locked invariants (S27), delta-to-code provenance (S28), artifact content-hash binding (S29), signed change provenance via Ed25519 (S30), signer-set governance evaluated at the decision tick (S31), and mechanism-source content binding with a no-execution AST probe (S32). A weakening of a locked invariant cannot launder past the release gate by words, labels, a self-declared effect, a forged signature, or an unsigned gate-code change. See [GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md) for the frozen chain and honest residuals.

Deferred from this lineage (future versions, only if needed): threshold / multi-signer (m-of-n) design-authority governance; behavioral probing for the mutation gateway, retrieval policy, and raw-episode store (currently integrity-bound only); formal verification of the effect classifier and design logic.

## v0.2: Typed Runtime

- Harden verifier rules into explicit auditable decision tables.
- Add evidence requirement levels to task packets.
- Add bootstrap ingestion with forced low license and human promotion.
- Add mutation authority gateway and direct-call rejection.
- Add post-action correction loops with belief/procedure separation.
- Add contradiction repair loop with evidence, scope, and unresolved outcomes.
- Add CLI epistemic snapshot for live operating-state inspection.
- Add planner regret loop for policy feedback after action outcomes.
- Add attention mode review loop for Reflex false alarms and interrupt-storm recovery replay.
- Add unified correction queue and deterministic recovery replay for deferred cognitive debt.
- Replace script prototype with Rust CIP packet structs and schema validation.
- Implement broker subscriptions and priority queue.
- Add persistent append-only log and SQLite indexes.
- Add unit and simulation tests for verifier and planner behavior.

## v0.3: Audit Dashboard

- Build packet stream, memory graph, verifier, attention, and trace views.
- Add replay from append-only trace logs.
- Add release checklist automation for schema drift and non-goal enforcement.

## v0.4: More Worlds

- Add additional toy environments.
- Add adversarial test harnesses.
- Add formal budget/backpressure tests.
