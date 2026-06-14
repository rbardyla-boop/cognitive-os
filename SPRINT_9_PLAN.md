# Sprint 9 Plan: Verifier + Bootstrap Hardening

## Goal

Protect the mechanism that protected Bridge A by making verifier logic auditable and bootstrap ingestion low-privilege.

## Non-Goals

- No broad new runtime correction loops.
- No production LLM ingestion.
- No promotion of design history into invariants without human approval.

## Files Expected To Change

- `scripts/verifier_engine.py`
- `scripts/language_codec.py`
- `scripts/toy_planner.py`
- `scripts/bootstrap_ingest.py`
- `simulations/bridge_world/verifier_rules.json`
- QA tests and release gate files

## New Scenarios

- Strict evidence request blocks degraded crossing.
- Low-license verifier rule is rejected.
- Bootstrap candidates require human promotion.

## New Packet Types

- None in this sprint.

## New Memory Statuses

- `pending_human_promotion` for bootstrap candidates.

## Risk List

- Verifier rule table becomes stale relative to code.
- Strict evidence mode blocks too much if over-triggered.
- Bootstrap ingestion can create noisy candidates.

## Definition Of Done

- Verifier decisions are driven by explicit decision table rules.
- Low-license verifier rules fail validation.
- `IntentPacket` carries `evidence_requirement`.
- Strict evidence requirement blocks degraded route action.
- Bootstrap ingestion forces `hypothesis_only`.
- Promotion requires explicit human approval.
- Release gate passes.

