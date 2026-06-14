# Migration Notes

## Sprint 9

- No SQLite schema migration required.
- Added verifier decision table fixture `simulations/bridge_world/verifier_rules.json`.
- Added bootstrap ingestion script with no persistent storage changes.

## Sprint 21

- Newly written recovery ledgers stamp `recovery-ledger-v2` provenance.
- Existing `recovery-ledger-v1` ledgers remain readable so old replay logs do not need migration.
- No SQLite schema migration required.

## Sprints 22–32

- No SQLite schema migration required. All additions are runtime artifacts/fixtures, not database schema: raw-episode store (S22), semantic candidates (S23), `design_memory.json` + `design_verifier_rules.json` (S24+), `control_point_policies/` (S29), `authorized_design_signers.json` schema v0.2 (S31), and `mechanism_source_manifest.json` (S32). Existing data needs no migration.
- Internal contract tightenings within the design-governance change_set were migrated in-place each sprint (S28 added the `change_set`, S29 made it content-bound, S30/S31 added governed signatures, S32 added the `binding: "mechanism_source"` variant); these affect only committed governance scenarios, not stored runtime memory.
