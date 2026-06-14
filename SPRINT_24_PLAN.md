# Sprint 24 — Unified Self-Correction (the Caitlin Leap)

Status: Complete.

## Why this replaces incremental Sprints 24–35

`a.md` planned twelve more incremental sprints (trust/provenance, multi-index,
retrieval-under-pressure, outcome testing, revision, consolidation, forgetting, LLM
boundary, lifecycle, runtime, red-team, RC). `COGNITIVE_OS_SELF_CORRECTING_LEAP.md`
argues that grinding more "letters" is the collider approach. The higher-leverage move
("met or better") is to apply the already-proven runtime machinery — Cognitive Bus,
Verifier, Epistemic Licenses, ContradictionPackets, Mutation Gateway, Trace Audit, and
the deferred correction loop — to the project's own design decisions.

This sprint delivers that unification with minimal new machinery: no new verifier
engine, no new mutation path, no separate meta-immune-system. Design memory is data;
the design verifier rule is data; the audit reuses the runtime `adjudicate`; the health
update goes through the real `apply_memory_mutation`.

## Goal

Prove that a design proposal which would weaken a locked invariant is detected,
blocked, denied consolidation, and routed into a deferred revalidation job by the same
machinery that blocks Bridge A under hazard-only evidence — and that the project can
audit its own design history and gate its own release as a verified cognitive action.

## Build

```text
simulations/bridge_world/design_memory.json          (locked invariants + audited design decisions)
simulations/bridge_world/design_verifier_rules.json  (weaken-locked-invariant -> hard_contradiction)
scripts/project_self_audit.py                         (--project / --strict; health via real gateway)
scripts/design_audit.py                               (design-governance trace replay)
bridge_world_demo._run_design_proposal_scenario       (new scenario type, reuses emit/gateway)
decision_audit.py --project                            (delegates to project_self_audit)
scenarios/design_contradiction_in_sprint_plan.json
scenarios/design_proposal_consistent_with_invariants.json
```

## Rubric — DONE means ALL of these are checkable PASS

1. A design proposal with `effect: weaken` against a `regression_lock` invariant
   produces a **ContradictionPacket with epistemic_license `hazard_only`**.
2. That proposal is **denied consolidation through the real mutation gateway**
   (`mutation_decision: reject`), so the invariant is preserved
   (`invariant_preserved: true`, `proposal_consolidated: false`).
3. The contradiction opens a **deferred `design_revalidation` job** and
   `blocks_release: true`.
4. A consistent proposal (`effect: extend`) is **accepted, consolidated, no
   contradiction, no revalidation**.
5. Design invariants are **retrieved with a license and provenance, never as naked
   facts** (`naked_fact: false`).
6. `decision_audit.py --project --strict` (and `project_self_audit.py`) **passes with
   zero violations**, and a design decision missing trace / verifier / license makes
   strict audit **fail** (exit nonzero).
7. The release gate runs project audit as a **verified cognitive action**: project
   health consolidates to `green` only through the gateway under `memory_consolidation`
   license; a failing strict audit blocks consolidation with an AuditViolation.
8. `scripts/release_check.sh` exits 0 and is **silent**, with the Sprint 24 gates wired
   into both `scripts/test.sh` and `scripts/release_check.sh`.

## Wrong if

- The weakening proposal silently passes, or consolidates, or mutates the invariant.
- A naked design fact is surfaced without license/provenance.
- Strict project audit green-lights an untraced/unverified/unlicensed design decision.
- New bespoke verifier or mutation logic duplicates the runtime machinery.
- `release_check.sh` exits nonzero or prints output.

## Checks (commands)

```sh
python3 scripts/design_audit.py --scenario design_contradiction_in_sprint_plan
python3 scripts/design_audit.py --scenario design_proposal_consistent_with_invariants
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Review result

```text
weaken-locked-invariant -> hazard_only ContradictionPacket        PASS
weakening proposal denied consolidation (invariant preserved)     PASS
design_revalidation job scheduled + release blocked               PASS
consistent extend proposal accepted + consolidated                PASS
design invariant retrieved with license, not naked                PASS
project strict audit passes with zero violations                  PASS
strict audit fails on missing trace/verifier/license decision     PASS
release gate consolidates project health only through gateway     PASS
release_check.sh exits 0 and silent                               PASS
```

## Residual / next boundary (explicitly deferred)

The leap makes the remaining lifecycle sprints (25–32) the *verification surface* of
the same governance loop rather than twelve separate immune systems. Any future
proposal to add them must enter as a design proposal and pass through this gate. The
honest open boundary the verifier surfaced next — the design verifier rule keyed on a
declared `effect` field, so a proposal could mislabel a weakening as an extension — is
**closed by Sprint 25 (Derived Effect Classification)**: `effect` is derived from a
semantic diff of the claims and the declared label is an untrusted hint only.
