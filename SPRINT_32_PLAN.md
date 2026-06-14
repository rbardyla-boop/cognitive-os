# Sprint 32 — Mechanism-Source Content Binding

Status: **Complete (2026-06-14).** `release_check.sh` is green+silent (exit 0, 0 stdout, 0 stderr)
and an independent adversarial panel returned VERIFIED on all 8 rubric items; its three sabotages
(weaken the real gate code under a clean policy / neutralize the behavioral probe / neutralize the
manifest verifier) each turned `release_check` red and were restored byte-identical.

Verifier-found defect (fixed before Complete): the panel reproduced a real HIGH — the original
behavioral probe **executed** the proposed post-image in a subprocess with the gate process's
filesystem access, so a malicious-but-probe-passing proposal could write files during evaluation
(restricted builtins would not have fixed it — the object-graph gadget escape defeats any in-process
sandbox). The probe was reworked to NEVER execute the proposed code: it parses the post-image,
extracts the bound function, and SAFELY INTERPRETS that function over the probe's fixed inputs across
a restricted AST subset (if / boolean / comparison / return over parameters and literals). Any
construct outside that subset — a call, attribute access, import, loop, module-level statement — is
never evaluated and fails closed to `mechanism_probe_error` (a regression, which blocks). This stays
behavioral (it computes the real outcome of the proposed `adjudicate`) while removing all execution,
filesystem access, side effects, and sandbox-escape surface. Locked by selftest cases proving the
exact attack (a module-level backdoor write) creates no file and the proposed code is never run.

## Why

Sprint 29 bound the change to a locked invariant to the literal content of its POLICY artifact, and
Sprints 30–31 bound it to a governed signed identity. But the policy artifact only says WHAT the
rule is — the enforcement CODE (the adjudicator, the mutation gateway, the probe harness, the
signature gate itself) decides whether the rule is actually ENFORCED. A signed, content-bound policy
is not enough if the gate code underneath it can be changed unsigned: weaken `verifier_engine.adjudicate`
while leaving `hazard_gate.json` clean and signed, and the protection is gone.

## Hard rule / doctrine

```text
A policy artifact says what the rule is.
The mechanism SOURCE decides whether the rule is actually enforced.
A signed policy is not enough if the gate code can be changed unsigned.
(Unchanged: authorization never overrides a trace failure.)
```

## What was built

1. **Integrity manifest** — `scripts/mechanism_provenance.py` + `simulations/bridge_world/mechanism_source_manifest.json`
   bind every enforcement-code file (`verifier_engine.py`, `mutation_gateway.py`, `retrieval_policy.py`,
   `raw_episode_store.py`, `trace_diff.py`, `change_provenance.py`, `design_signing.py`,
   `effect_classifier.py`, `project_self_audit.py`, and `mechanism_provenance.py` itself) by content
   hash, keyed by `role`. `--verify` recomputes from disk and fails release on any divergence; the
   project strict audit also gates on `mechanism_source_binding == verified`. A gate-code change
   without regenerating the manifest fails release.
2. **Mechanism-source change provenance** — a `change_set` with `binding: "mechanism_source"` binds
   the literal before/after content of a real source file; the pre-image must equal the CURRENT
   on-disk source (a change against a stale/wrong gate version is rejected). It flows through the
   same Sprint-30/31 governed signature gate (unsigned change to a locked gate blocks).
3. **Behavioral probe on the PROPOSED source** — `trace_diff.derive_effect_from_trace` dispatches a
   mechanism-source change_set to a probe that evaluates the bound gate's protected behavior against
   the PROPOSED post-image WITHOUT executing it: the post-image is parsed, the bound function
   extracted, and its body safely interpreted over fixed inputs across a restricted AST subset
   (fail-closed to a regression on anything outside that subset). A weakening of the adjudicator is
   caught here, by probe, even with clean policy files and a valid signature. `decision_audit
   --project` reports `mechanism_source_binding`.

## Rubric — DONE means ALL of these are checkable PASS

1. `mechanism_source_hash_mismatch_fails_release`: a mechanism-source change bound to a pre-image
   that is not the current gate code blocks (`stale_pre_image` → `delta_provenance_unverified`).
2. `unsigned_mechanism_source_change_blocks`: a preserving mechanism-source change to a locked gate
   with no signature blocks (`change_signature_unverified`).
3. `signed_mechanism_preserving_change_accepts`: a signed preserving change accepts (the probe
   confirms the protected outcome survives the proposed source).
4. `signed_mechanism_weakening_change_blocks_by_probe`: a signed change that softens the adjudicator
   blocks BY PROBE (`signature_verified` but `trace_behavior_regression`).
5. `policy_artifact_clean_but_gate_code_weakened_fails`: a signed change that weakens the gate code
   while every policy file stays clean blocks by probe.
6. The manifest verifies against the real on-disk code; a tampered recorded hash is detected
   (non-vacuous). The project strict audit reports + gates on `mechanism_source_binding`.
7. `release_check.sh` exits 0 and is byte-silent; the behavioral probe is a no-execution AST
   interpreter (the proposed post-image is parsed, never run, and never leaks output); lifecycle is
   deterministic. Sabotage of the manifest check or the mechanism probe fails release_check. No
   private key is committed.
8. Sprint 26–31 scenarios keep their governance; DD_sprint_32 recorded.

## Wrong if

- The policy files stay signed and clean while the enforcement code is weakened underneath them and
  the change is accepted.
- A mechanism-source weakening reaches accept; an unsigned mechanism-source change to a locked gate
  is accepted; a governed signer's authorship overrides the probe regression.
- The probe executes the proposed code (filesystem/side-effect access), leaks output (non-silent
  release_check), evaluates the live tree instead of the proposed post-image, or fails OPEN instead
  of closed; or a private key is committed.

## Explicitly NOT in this sprint

- Threshold / multi-signer governance (deferred unless this lands cleanly).
- Behavioral binding for every mechanism role — only the adjudicator role is behaviorally probed for
  now; the other roles are integrity-bound (manifest) and a proposed change to them that cannot be
  behaviorally proven preserving fails closed (`needs_review`).

## Doctrine

```text
A policy artifact says what the rule is.
Mechanism source decides whether the rule is actually enforced.
A signed policy is not enough if the gate code can be changed unsigned.
```
