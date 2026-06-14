# Release Notes: cognitive-os-v0.1.0

`cognitive-os-v0.1.0` is the first local prototype release of the cognitive architecture testbed.

Version set:

- `cognitive-os-v0.1.0`
- `cip-schema-v0.1`
- `memory-schema-v0.1`
- `simulation-bridge-world-v0.1`

This release proves the local loop:

Human command -> deterministic language codec -> CIP packets -> in-process bus -> attention manager -> governed retrieval -> verifier/adjudication -> planner -> toy action -> outcome recording -> memory mutation -> post-action revalidation.

Sprint 15 extends the proof surface with attention mode review: Reflex false alarms and interrupt-storm recovery can update only attention policy through mutation authority.

Sprint 16 adds unified recovery replay so deferred correction work can be ordered, inspected, replayed, bounded, and resolved only through mutation authority.

Sprint 21 adds asymmetric replay identity: Ed25519 private keys sign recovery ledgers, public keys verify them, and public-key-only replay cannot forge mutation-suppressing authority. HMAC remains available for local development.

This release also includes the complete Sprint 24–32 development-process governance lineage: unified self-correction (S24), derived effect classification (S25), trace-grounded invariant validation (S26), complete probe coverage of all five locked invariants (S27), delta-to-code provenance (S28), artifact content-hash binding (S29), signed change provenance via Ed25519 (S30), signer-set governance evaluated at the decision tick (S31), and mechanism-source content binding with a no-execution AST probe (S32). Every locked design invariant is probe-backed, and a weakening is caught by behavioral test, content hash, governed signature, and mechanism-source binding — not by words alone. The frozen chain and its honest residuals are recorded in [GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md); per-attack closure in [FAILURE_LEDGER.md](FAILURE_LEDGER.md) (FAIL-0009..FAIL-0016). Runtime and dependencies are pinned in [ENVIRONMENT.md](ENVIRONMENT.md) and `requirements.txt`.

Release gates:

- format/lint clean
- unit tests pass
- integration tests pass
- simulation tests pass
- adversarial tests pass
- schema migration test pass
- dashboard smoke test pass
- release notes written
- known limitations documented
- design-governance gates pass (S24–S32: derived effect, trace probes, content + signed + mechanism-source binding)
- environment lock present (`requirements.txt` + `ENVIRONMENT.md`)
- governance milestone frozen (`GOVERNANCE_MILESTONE.md`)
- release gate is byte-silent (exit 0, 0 bytes stdout/stderr)
