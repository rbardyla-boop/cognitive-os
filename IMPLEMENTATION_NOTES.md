# Implementation Notes

## Sprint 9

- Verifier conflict detection now evaluates `simulations/bridge_world/verifier_rules.json`.
- Language codec emits `evidence_requirement` as `Strict`, `Cautious`, or `HypothesisOK`.
- Planner blocks degraded crossing when `evidence_requirement` is `Strict`.
- Bootstrap ingestion is intentionally low-confidence and requires human promotion.

## Sprint 10

- Mutation authority now routes governed semantic/bootstrap changes through `scripts/mutation_gateway.py`.
- `scripts/mutation_audit.py` reconstructs accepted and rejected mutation attempts.

## Sprint 11

- Post-action correction scenarios route procedure and belief updates through mutation authority.
- `mutation_audit.py` now reports ordered correction mutations.

## Sprint 12

- Contradiction repair scenarios route semantic memory repairs through mutation authority.
- `scripts/contradiction_audit.py` replays resolved, scoped, and unresolved repair outcomes.

## Sprint 13

- `scripts/epistemic_snapshot.py` exposes task, authority objects, contradiction state, decision constraints, pending work, and current recommendation.

## Sprint 14

- `PlanRegretPacket` records expected-vs-actual planner feedback.
- `scripts/planner_regret_audit.py` replays scoped planner policy updates and pending reviews.

## Sprint 15

- `AttentionModeReviewPacket` records attention mode classification after Reflex/Strained/Emergency operation.
- `scripts/attention_review_audit.py` replays attention policy updates, false Reflex reviews, and interrupt-storm recovery replay.

## Sprint 16

- `scripts/recovery_replay.py` provides `CorrectionJob`, `CorrectionQueue`, deterministic ordering, bounded low-priority deferral, and mutation-gateway replay for resolved correction jobs.

## Sprint 21

- `scripts/replay_asymmetric_key.py` adds Ed25519 private-key signing and public-key verification for replay ledgers.
- `scripts/recovery_replay.py` accepts `--ledger-private-key-file` and `--ledger-public-key-file`.
- Newly written replay ledgers stamp `recovery-ledger-v2`; existing v1 ledgers remain readable.
- Public-key-only replay can verify an existing ledger but cannot sign a fresh ledger.

## Sprint 22

- `scripts/raw_episode_store.py` adds `ExperienceEnvelope`, `RawEpisode`, and an append-only in-memory raw episode store.
- `scripts/ingest_experience.py` provides a CLI proof surface for raw-before-semantic ingestion scenarios.
- `epistemic_snapshot.py` surfaces raw episode provenance and raw-ingestion pending work when a scenario includes experience envelopes.

## Sprint 23

- `scripts/semantic_candidate_extractor.py` extracts `CandidateMemoryNode` interpretations from raw episodes.
- Candidate nodes default to `hypothesis_only`, `semantic_candidate`, confidence `0.0`, and forbid direct action/consolidation.
- `epistemic_snapshot.py` surfaces candidate nodes and candidate-extraction status when present.
