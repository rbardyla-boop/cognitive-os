# Known Limitations

- No autonomous real-world action.
- No live internet access.
- No production LLM codec.
- No real robotics, financial, medical, legal, or physical-world execution.
- No self-modifying model weights.
- The verifier is heuristic and only suitable for toy-world testing.
- The planner supports only bridge-world actions.
- The dashboard components are smoke-tested scaffolds, not a complete UI.
- SQLite storage is local prototype storage, not distributed infrastructure.
- Latent world encoding is intentionally deferred.

## Design governance (Sprints 24–32)

- Derived-effect classification is deterministic lexical + trace-grounded, not formal verification.
- Only the `adjudicator` role is behaviorally probed against a proposed mechanism-source change; the other nine manifest-bound mechanism roles are integrity-bound (content hash) only — a proposed change to them fails closed to `needs_review` rather than being probe-tested.
- The mechanism-source AST probe supports a restricted subset (if / boolean / comparison / return over parameters and literals); a behavior-preserving adjudicator change that uses a helper call or a loop fails closed to a regression rather than accepting — a precision cost in the safe direction.
- Governance is single-signer: no threshold / multi-signer / m-of-n. A single active, in-scope, in-window key suffices; there is no recovery path if the `design_authority` key is compromised.
- Signer lifecycle (revocation, expiry, rotation) is logical-tick based (deterministic, not wall-clock); the operator supplies the evaluation tick.
- `mechanism_provenance.py` binds itself by hash but cannot fully self-attest (the who-watches-the-watchmen limit of a single-repo self-check); it is bounded by the behavioral probes + regression suite.
- This governance layer is a deterministic proof-of-concept and testbed, not production-ready for cryptographically-critical systems until the above are resolved or explicitly accepted. See [GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md) §7.

