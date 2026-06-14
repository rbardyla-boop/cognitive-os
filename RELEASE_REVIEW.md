# Release Review

## Sprint 9

- No new packet type was added.
- Verifier rules are explicit and low-license verifier rules are rejected.
- Evidence requirement levels do not weaken existing permission gates.
- Bootstrap ingestion cannot create release invariants without human approval.
- Release gate includes regression coverage for the new hardening paths.

## Sprints 24–32 — Development-process governance

Each link locks an invariant, prevents a specific attack, and is guarded by a named regression scenario (see [FAILURE_LEDGER.md](FAILURE_LEDGER.md) FAIL-0009..FAIL-0016 and [GOVERNANCE_MILESTONE.md](GOVERNANCE_MILESTONE.md) for the frozen chain).

- **S24 — Unified self-correction.** Locks that a design change weakening a proven runtime invariant is blocked, not silently merged. Attack: a future sprint weakens `D_invariant_hazard_blocks_action`. Guard: `design_contradiction_in_sprint_plan` (FAIL-0008).
- **S25 — Derived effect.** Locks that the `effect` cannot be self-declared as safe. Attack: a weakening labels itself `extend`. Guard: `design_effect_mislabel_attack` (FAIL-0009).
- **S26 — Trace-grounded diff.** Locks that a protected invariant is tested, not trusted. Attack: a lexical-preserve claim whose behavior regresses the hazard gate. Guard: `preserve_marker_launders_weakening_blocked` (FAIL-0010).
- **S27 — Complete probe coverage.** Locks that a locked invariant without a probe is ineligible for accept. Attack: a leading-preserve-marker weakening of an unprobed invariant. Guard: `trace_diff_blocks_*_laundering` (FAIL-0011).
- **S28 — Delta provenance.** Locks that the tested delta binds to the actual change. Attack: a no-op delta over a weakening patch. Guard: `misstated_noop_delta_with_weakening_patch_blocked` (FAIL-0012).
- **S29 — Content-hash binding.** Locks that the change is the literal artifact content. Attack: a stale/divergent pre-image. Guard: `stale_pre_image_hash_rejected` (FAIL-0013).
- **S30 — Signed provenance.** Locks that a content-bound change to a locked invariant carries an authorized Ed25519 signature; a signed weakening still blocks by trace. Attack: an unsigned/forged/replayed change. Guard: `unsigned_content_bound_change_blocks` (FAIL-0014).
- **S31 — Signer governance.** Locks that a public key is not permanent authority; authority is evaluated at the decision tick. Attack: a revoked/expired/out-of-scope signer, or a revoked-key replay. Guard: `revoked_signer_rejected` and the decision-tick scenarios (FAIL-0015).
- **S32 — Mechanism-source binding.** Locks that the enforcement code itself is content-bound and probe-tested; a signed weakening of the gate code blocks by a no-execution AST probe even with clean policy. Attack: weaken `adjudicate` under a clean, signed policy. Guard: `policy_artifact_clean_but_gate_code_weakened_fails` (FAIL-0016).

Verification discipline: each sprint reached a green, byte-silent `release_check.sh`, then an independent fresh-context adversarial verifier; any residual became the next sprint. S25 and S32 each carried a verifier-found defect that was fixed before close and locked with a named scenario. The closure chain was independently re-confirmed (all eight links enforced, zero overstatements) during the v0.1 freeze.

