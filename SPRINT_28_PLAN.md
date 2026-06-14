# Sprint 28 — Delta-to-Code Provenance

Status: Complete.

## Why

Sprint 27 made every locked invariant probe-backed, but the probe ran the proposal's
*self-declared* `behavioral_delta`. Its residual: a no-op/mis-stated delta whose tested
behavior does not regress, while the real weakening is shipped in the code/config the prose
gestures at. A delta without provenance is just another label. This sprint binds the tested
delta to the actual proposed change set.

## Hard rule / doctrine (the invariant this sprint locks)

```text
A trace is only evidence if it tests the thing being changed.
A behavioral delta without provenance is another label.
For a locked invariant, the tested delta is DERIVED from a provenance-verified change_set —
the self-declared behavioral_delta is never trusted as authority.
```

## Goal

A design proposal carries a `change_set` — the structured patch it would apply to a real
runtime control point, naming the actual changed artifact and a digest binding the patch to
it. `trace_diff` derives the tested delta FROM the change_set (not from `behavioral_delta`),
runs the probe on that derived delta, and cites the changed artifact. Missing or unverifiable
provenance blocks (needs_review) for a locked invariant.

## Build

```text
scripts/change_provenance.py          CONTROL_POINT_ARTIFACTS registry; canonical digest; verify_change_set_provenance; derive_delta_from_change_set; CLI --digest
scripts/trace_diff.py                  derive_effect_from_trace derives the tested delta from a provenance-verified change_set; surfaces provenance + changed_artifact + delta_matches_change_set; combine_effects blocks a locked invariant with missing/unverified provenance (authority delta_provenance_unverified)
scripts/project_self_audit.py          evaluate_design_proposal surfaces trace_provenance / changed_artifact / delta_matches_change_set
scripts/bridge_world_demo.py           surfaces the provenance fields on the design packets
scripts/design_audit.py                reports trace_provenance / changed_artifact / delta_matches_change_set
scenarios: delta_provenance_required_for_locked_invariant, misstated_noop_delta_with_weakening_patch_blocked, derived_delta_matches_patch_accepts_preserving_change, missing_patch_for_behavioral_delta_needs_review
migrated scenarios (behavioral_delta -> change_set): the 9 Sprint 26/27 design scenarios that carried a delta
```

## How provenance is real (not a text assertion)

A `change_set` is `{target, changed_artifact, patch, adds?, patch_digest}`. Provenance is
verified by: (1) `target` is a known control point; (2) `changed_artifact` equals the real
source file that implements that control point (`CONTROL_POINT_ARTIFACTS`) and that file
**actually exists on disk**; (3) `patch_digest` equals the canonical SHA-256 of
`(target, changed_artifact, patch, adds)`. Only then is the delta derived from `patch` and the
probe run. The self-declared `behavioral_delta`, if present, is surfaced as
`delta_matches_change_set` (hint only) and is never authority.

## Rubric — DONE means ALL of these are checkable PASS

1. `misstated_noop_delta_with_weakening_patch_blocked`: a proposal whose self-declared
   `behavioral_delta` is a no-op but whose `change_set.patch` is a weakening is BLOCKED — the
   patch is what is tested (`trace_regressed: true`, `delta_matches_change_set: false`,
   derived weakening, block). Prose/declared-delta cannot describe one delta while the patch
   produces another.
2. `derived_delta_matches_patch_accepts_preserving_change`: a genuine preserving change_set
   (no-regression patch) is ACCEPTED, with `trace_provenance: verified`,
   `changed_artifact` citing the real file, `delta_matches_change_set: true`.
3. `missing_patch_for_behavioral_delta_needs_review`: a proposal with a `behavioral_delta`
   but NO `change_set` against a locked invariant lands in `needs_review`
   (`effect_authority: delta_provenance_unverified`) — a self-declared delta is not trusted.
4. `delta_provenance_required_for_locked_invariant`: a proposal targeting a locked invariant
   with no provenance (no change_set) blocks; and an unverifiable change_set (wrong artifact,
   digest mismatch, unknown target) also blocks.
5. `trace_diff` cites the actual changed artifact (`changed_artifact`) and derives the tested
   policy from `change_set.patch`, not from `behavioral_delta`.
6. The Sprint 26/27 attack scenarios still BLOCK and the accept-scenarios still ACCEPT once
   migrated to carry a provenance-verified change_set (no governance regression).
7. `decision_audit.py --project --strict` passes with zero violations and the design trace
   reports delta provenance; DD_sprint_28 recorded.
8. `scripts/release_check.sh` exits 0 and is silent, with Sprint 28 gates in both
   `scripts/test.sh` and `scripts/release_check.sh`; a gate-sabotage (trusting the declared
   delta instead of the change_set, or skipping provenance) makes `release_check.sh` fail.

## Wrong if

- A proposal passes by testing a harmless no-op `behavioral_delta` while shipping a weakening
  in `change_set.patch` (or vice versa).
- A self-declared `behavioral_delta` is read as authority for a locked invariant.
- Missing/unverifiable provenance reaches accept for a locked invariant.
- A genuine preserving change_set is blocked, or a Sprint 26/27 gate regresses, or
  `release_check.sh` exits nonzero or prints output.

## Checks (commands)

```sh
python3 scripts/change_provenance.py --selftest
python3 scripts/trace_diff.py
python3 scripts/design_audit.py --scenario misstated_noop_delta_with_weakening_patch_blocked
python3 scripts/design_audit.py --scenario derived_delta_matches_patch_accepts_preserving_change
python3 scripts/design_audit.py --scenario missing_patch_for_behavioral_delta_needs_review
python3 scripts/design_audit.py --scenario delta_provenance_required_for_locked_invariant
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Residual / next boundary (explicitly deferred)

The change_set binds the tested delta to a named, on-disk runtime artifact and a digest over
the structured patch, so a proposal can no longer test a no-op while the patch ships a
weakening — the patch IS what is tested. Honest remaining limit (no safe-default claim): the
`patch` is still a structured description of the change rather than the literal source diff,
and the digest binds the patch to the artifact's *path*, not to the artifact's *content hash*
at apply time. A proposal could in principle name the right file and a faithful structured
patch while the eventual real code edit diverges from the structured patch. The next boundary
is content-hash binding — pin the change_set to the artifact's actual content digest and
re-derive the structured patch from the literal diff at apply time — and is the L2→L3 step in
the backend charter (ADR-002): the replay-evidence contract must bind to the artifact content,
not just its name.
