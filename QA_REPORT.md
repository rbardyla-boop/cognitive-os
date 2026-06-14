# QA Report

## Sprint 9

- Unit coverage added for verifier rule validation, evidence requirement parsing, strict evidence planner behavior, and bootstrap promotion.
- Existing release gate continues to run integration, simulation, adversarial, regression, backend, and dashboard smoke checks.
- Failure ledger entries are locked by regression tests.

## Sprint 10

- Regression coverage added for direct mutation rejection, low-authority mutation rejection, valid human promotion, and mutation audit replay.

## Sprint 11

- Regression coverage added for degraded success, degraded failure, and partial success correction behavior.

## Sprint 12

- Regression coverage added for contradiction repair by new evidence, scoped context, and unresolved preservation.

## Sprint 13

- Regression coverage added for strict epistemic snapshots across safety planning, unresolved contradiction, scoped contradiction, and human promotion scenarios.

## Sprint 14

- Regression coverage added for correct-under-uncertainty, near-miss review, and overconservative-wait planner regret scenarios.

## Sprint 15

- Regression coverage added for correct Reflex activation, false Reflex over-trigger review, interrupt-storm recovery replay, attention policy mutation boundaries, and snapshot-visible pending attention review.

## Sprint 16

- Regression coverage added for mixed correction job ordering, gateway-governed recovery resolution, bounded low-priority deferral, and snapshot-visible correction queue state.

## Sprint 21

- Regression coverage added for Ed25519 signed ledger verification with public key only.
- Regression coverage confirms public-key-only replay cannot sign fresh ledgers.
- Wrong public keys and tampered asymmetric signatures downgrade to audit-only and re-apply through the gateway.
- Legacy HMAC signed replay identity remains covered.

## Sprint 22

- Regression coverage added for raw episode preservation before semantic candidates.
- Semantic candidate creation without a raw episode is blocked.
- Raw episode overwrite attempts are rejected by the append-only store.
- Malformed experience envelopes are rejected without partial raw or semantic state.
- Strict snapshots expose raw episode provenance when raw ingestion is present.

## Sprint 23

- Regression coverage added for raw episode to semantic candidate extraction.
- Candidate defaults are gated as non-authoritative (`hypothesis_only`, `semantic_candidate`, no direct action/consolidation).
- Candidate provenance gates require raw episode IDs and integrity digests.
- LLM authority injection attempts are normalized back to non-authoritative candidate state.
- Extraction failures preserve raw episodes and report rejected candidate attempts.

## Sprint 24 — Unified self-correction (the Caitlin leap)

- A design proposal that would weaken a `regression_lock` invariant is blocked by a `hazard_only` ContradictionPacket, denied consolidation through the real mutation gateway, and opens a deferred `design_revalidation` job; `decision_audit.py --project --strict` gates the release.

## Sprint 25 — Derived effect classification

- A weakening mislabeled `extend` is reclassified from a semantic diff and blocked; a weakening with no declared effect is still classified from evidence and blocked. The declared `effect` is an untrusted hint (`effect_mislabel`); only `derived_effect` drives the decision.

## Sprints 26–27 — Trace-grounded invariants + complete probe coverage

- A preserve/extend claim against a probe-guarded invariant is accepted only if a runtime trace proves no behavioral regression; a lexical-preserve claim whose behavior regresses the gate is reclassified and blocked (`preserve_marker_launders_weakening_blocked`). All five locked invariants are probe-backed, and a locked invariant without a probe is ineligible for accept (`locked_invariant_without_probe`).

## Sprints 28–29 — Delta provenance + artifact content-hash binding

- The tested delta is derived from a provenance-verified `change_set`, not a self-declared delta; a mis-stated no-op delta over a weakening patch is blocked. The `change_set` binds to the literal before/after artifact content; a stale pre-image, wrong post-image hash, or a structured patch that diverges from the literal diff is blocked.

## Sprints 30–31 — Signed change provenance + signer-set governance

- A content-bound change to a locked invariant requires a valid Ed25519 signature from an authorized signer (unsigned / wrong-signer / replayed signatures block); a validly-signed weakening still blocks by trace. Signer authority is governed (scope + lifecycle) and evaluated at the decision tick: revoked / expired / out-of-scope signers are rejected, a rotated successor is accepted, and a revoked key cannot replay a prior signature. Only public keys are committed.

## Sprint 32 — Mechanism-source content binding

- The enforcement code is content-hash bound in a verified manifest (`mechanism_provenance.py --verify`; strict audit gates on `mechanism_source_binding`); an unsigned mechanism-source change blocks; a signed weakening of the adjudicator blocks by a no-execution AST probe; a clean-policy-but-weakened-gate-code change fails release. Sabotage of the manifest check or the probe fails the release gate.
