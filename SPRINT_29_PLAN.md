# Sprint 29 — Artifact Content-Hash Binding

Status: Complete.

## Why

Sprint 28 bound the tested delta to a `change_set` (a structured patch + the changed
artifact's *path* + a digest). Its residual: the digest bound the patch to the artifact's
*path*, not its *content*. A faithful-looking structured patch could diverge from the
eventual file edit, or target the right path but assume the wrong file content. This sprint
binds the tested delta to the actual before/after artifact content and the literal diff.

## Hard rule / doctrine (the invariant this sprint locks)

```text
A change is not the file name.
A change is not the prose patch.
A change is the before/after artifact content and the behavior it produces.
The tested delta is derived from the literal diff of the artifact's real content.
```

## Goal

Each control point has a real on-disk policy artifact whose content defines its protected
policy. A `change_set` carries `pre_image` + `pre_image_hash` + `post_image` +
`post_image_hash` + `diff_digest`. `trace_diff` verifies the `pre_image_hash` against the
artifact's ACTUAL on-disk content (rejecting stale/wrong-content), verifies `post_image_hash`,
binds `diff_digest` to the literal diff, derives the tested policy from the literal post-image
content, and runs the probe on it. Accepted trace evidence cites artifact path + pre hash +
post hash + diff digest.

## Build

```text
simulations/bridge_world/control_point_policies/*.json   real per-control-point baseline policy artifacts (the protected content)
scripts/change_provenance.py   CONTROL_POINT_POLICY_ARTIFACTS; canonical_policy_text; content_hash; literal_diff/diff_digest; load_baseline_policy; verify_change_set_provenance binds pre/post image hashes to real file content and derives the policy from the literal post-image
scripts/trace_diff.py          baseline loaded from the on-disk policy artifact; tested delta derived from the literal post-image; combine_effects blocks on content-binding failure (delta_provenance_unverified)
scripts/project_self_audit.py  surfaces pre_image_hash / post_image_hash / diff_digest / changed_artifact
scripts/design_audit.py        reports the artifact content binding
scenarios: stale_pre_image_hash_rejected, wrong_post_image_hash_rejected, structured_patch_diverges_from_literal_diff_blocked, literal_diff_preserving_change_accepts, literal_diff_weakening_change_blocks
migrated scenarios: every Sprint 28 change_set re-expressed as a content-bound change_set
```

## How the binding is real (not a path + description)

A policy artifact (e.g. `control_point_policies/hazard_gate.json`) holds the protected
baseline content (`{"urgency_overrides_hazard": false}`) and is read by the probe as its
baseline. A `change_set` supplies the literal `pre_image` and `post_image` text. Provenance
holds only when: the `changed_artifact` is the registered policy file and exists; the supplied
`pre_image` hashes to `pre_image_hash` AND that hash equals the SHA-256 of the artifact's
ACTUAL current on-disk content (a stale or wrong-content pre-image is rejected); the
`post_image` hashes to `post_image_hash`; the `diff_digest` equals the canonical hash of
`(target, changed_artifact, pre_image_hash, post_image_hash, literal_unified_diff)`; and the
`post_image` parses to a policy dict (else non-applicable). The tested policy is the parsed
post-image — derived from content, never self-declared. If the artifact's on-disk baseline
content itself no longer yields the protected outcome, the probe integrity check blocks.

## Rubric — DONE means ALL of these are checkable PASS

1. `stale_pre_image_hash_rejected`: a `change_set` whose `pre_image_hash` does not match the
   artifact's actual on-disk content is rejected (`trace_provenance: stale_pre_image`,
   `effect_authority: delta_provenance_unverified`, block).
2. `wrong_post_image_hash_rejected`: a `change_set` whose `post_image_hash` does not match
   the SHA-256 of `post_image` is rejected (`wrong_post_image`, block).
3. `structured_patch_diverges_from_literal_diff_blocked`: a `change_set` whose declared
   structured `patch` disagrees with the policy derived from the literal `post_image` is
   blocked (`structured_patch_diverges`, block) — the literal diff is authority.
4. `literal_diff_weakening_change_blocks`: a `post_image` that flips a protected key to a
   weakening regresses the probe and is blocked (`trace_regressed: true`, derived weakening,
   `trace_provenance: verified`, block), citing pre/post/diff.
5. `literal_diff_preserving_change_accepts`: a `post_image` with a real non-regressing diff
   (a benign added key) is ACCEPTED (`trace_regressed: false`, `trace_provenance: verified`,
   citing `pre_image_hash` + `post_image_hash` + `diff_digest` + the artifact path),
   consolidated.
6. `trace_diff` derives the tested delta from the literal post-image content (not a declared
   patch/delta), and a `change_set` cannot verify unless the artifact's content hash matches.
7. The Sprint 26/27/28 attack scenarios still BLOCK and the accept-scenarios still ACCEPT
   once migrated to content-bound change_sets (no governance regression).
8. `decision_audit.py --project --strict` passes with zero violations, reporting the artifact
   content binding; DD_sprint_29 recorded.
9. `scripts/release_check.sh` exits 0 and is silent, with Sprint 29 gates in both
   `scripts/test.sh` and `scripts/release_check.sh`; a gate-sabotage of content-hash
   verification (accept a stale pre-image, or derive the delta from the declared patch
   instead of the post-image) makes `release_check.sh` fail.

## Wrong if

- A faithful-looking structured patch can diverge from the eventual file edit and be accepted.
- A patch targets the right path but the wrong file content and is accepted.
- A proposal passes trace_diff against a patch that was never applicable to the actual
  artifact (a stale pre-image).
- A genuine preserving literal-diff change is blocked, or a Sprint 26/27/28 gate regresses, or
  `release_check.sh` exits nonzero or prints output.

## Checks (commands)

```sh
python3 scripts/change_provenance.py --selftest
python3 scripts/trace_diff.py
python3 scripts/design_audit.py --scenario stale_pre_image_hash_rejected
python3 scripts/design_audit.py --scenario wrong_post_image_hash_rejected
python3 scripts/design_audit.py --scenario structured_patch_diverges_from_literal_diff_blocked
python3 scripts/design_audit.py --scenario literal_diff_weakening_change_blocks
python3 scripts/design_audit.py --scenario literal_diff_preserving_change_accepts
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Residual / next boundary (explicitly deferred)

The tested delta is now bound to the literal before/after content of a real on-disk policy
artifact, the hash of which is recomputed from disk at evaluation time — a stale or
wrong-content pre-image is rejected, and the structured patch cannot diverge from the literal
diff. Honest remaining limit (no safe-default claim): the policy artifact is the bound unit,
and the *behavioral mechanism* it configures (the runtime function the probe calls) is bound
only by the probe's own baseline-integrity check, not by a content hash of the mechanism's
source. A change that weakens the mechanism source while leaving the policy artifact intact is
out of this sprint's binding (it is the runtime engine's own replay-contract responsibility —
ADR-002 L0). The next boundary is signed change provenance — bind the change_set to an
authorized author over the content digest (the asymmetric-identity machinery from Sprint 21
applied to design changes) so content provenance also carries authorship.
