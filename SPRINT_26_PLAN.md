# Sprint 26 — Trace-Grounded Invariant Diff

Status: Complete.

## Why

Sprint 25 derived a proposal's effect from a lexical semantic diff of the claim vs.
the invariant claim. Its honest residual: a claim pairing an explicit preservation
marker with a weakening phrased entirely outside both lexicons (e.g. "while preserving
the abort path, soften how strictly hazard_only gates direct action") could reach the
preserve/accept path. Words are not evidence. This sprint detects a weakening by what
the proposal would *break*, not by the words it uses.

## Hard rule (the invariant this sprint locks)

```text
Words are claims. Traces are evidence.
A protected invariant's preservation must be TESTED, not trusted.
A proposal targeting a probe-guarded invariant is accepted only if a behavioral trace
proves the protected outcome did not regress. The declared effect and the lexical
classifier cannot override the trace result.
```

## Goal

Bind each protected design invariant to a real runtime behavioral probe (the same
`adjudicate` / `apply_memory_mutation` machinery that governs the bridge world). Run the
probe at baseline and with the proposal's stated behavioral delta applied; if the
protected outcome regresses, derive a weakening/contradiction from the behavior delta —
regardless of how the proposal is worded or what effect it declares.

## Build

```text
scripts/trace_diff.py                                 (PROBES registry over real adjudicate/gateway; derive_effect_from_trace; combine_effects; self-tests)
simulations/bridge_world/design_memory.json           (behavioral_probe binding on hazard + mutation-authority invariants; DD_sprint_26)
project_self_audit.evaluate_design_proposal           (lexical = early warning; trace = authority; combine; surface trace_* + lexical_effect + effect_authority)
bridge_world_demo._run_design_proposal_scenario        (surface trace_effect/trace_regressed/trace_tested/effect_authority on packets)
scripts/design_audit.py                                (report trace_effect/trace_regressed/trace_tested/lexical_effect/effect_authority)
scenarios: preserve_marker_launders_weakening_blocked, trace_diff_detects_hazard_gate_softening, trace_diff_detects_consolidation_gate_softening, trace_diff_accepts_true_preserving_extension
updated scenarios (now carry a behavioral_delta): design_effect_preserve_consistent, design_proposal_consistent_with_invariants
```

## How the probes are real (not declared)

- `hazard_gate`: `adjudicate("hard_contradiction", 0.2)` → `reject_episode` (protected). The
  proposed delta `urgency_overrides_hazard` reclassifies the hazard contradiction as a
  tolerable known-exception → `adjudicate("known_exception", 0.6)` → `preserve_as_exception`
  (action no longer blocked). The real adjudicator returns both outcomes.
- `consolidation_gate`: `apply_memory_mutation` from a source whose authority forbids
  `memory_consolidation` → `rejected` (protected). The proposed delta
  `authorize_unprivileged_consolidation` grants that source consolidation rights → the real
  gateway now `consolidated`. The real gateway returns both outcomes.

The proposal's `behavioral_delta` is the change *under test*, not an authority claim: the
trace runs it and derives the effect from the observed behavior delta.

## Rubric — DONE means ALL of these are checkable PASS

1. **preserve_marker_launders_weakening_blocked**: a claim with preservation markers and a
   weakening verb outside both Sprint-25 lexicons (so the lexical layer alone returns a
   consistent effect), declared `effect: preserve`, but whose behavioral delta flips the
   hazard gate, is reclassified from the **trace**: `trace_regressed: true`,
   `effect_authority: trace_behavior_regression`, derived effect is a weakening,
   `effect_mislabel: true`, `governance_decision: block`, `proposal_consolidated: false`.
   The Sprint-25 laundering hole is closed by behavior.
2. **trace_diff_detects_hazard_gate_softening**: a euphemistic claim whose delta softens the
   hazard gate is blocked from the trace (`trace_regressed: true`, derived weakening,
   `governance_decision: block`, hazard_only contradiction).
3. **trace_diff_detects_consolidation_gate_softening**: a proposal whose delta would let an
   unprivileged source consolidate is blocked from the trace against
   `D_invariant_mutation_requires_authority` (`trace_regressed: true`, derived weakening,
   `governance_decision: block`).
4. **trace_diff_accepts_true_preserving_extension**: a genuine extension whose delta does not
   regress the hazard gate is accepted with `trace_tested: true`, `trace_regressed: false`,
   `effect_authority: trace_confirmed_preservation`, `proposal_consolidated: true` (proving
   the gate is not blocking everything).
5. **Declared effect cannot override the trace**: nowhere in the decision path is the
   proposal's declared `effect` or `behavioral_delta`-asserted-effect read as authority; the
   trace outcome decides, and the lexical layer can only *raise* severity, never grant accept.
6. **Preservation must be tested**: a proposal claiming preserve/extend against a
   probe-guarded invariant with NO behavioral delta lands in `needs_review` (blocks) —
   `effect_authority: preservation_not_tested`. Words alone cannot earn accept.
7. Backward compatible: every Sprint 24/25 design scenario keeps its prior governance
   outcome (the two accept-scenarios targeting a probe-guarded invariant now carry a
   no-regression delta; the attack scenarios still block).
8. The runtime `adjudicate` still independently confirms the hard-contradiction →
   `reject_episode` outcome on the derived (combined) effect.
9. `decision_audit.py --project --strict` passes with zero violations; DD_sprint_26 is
   recorded with trace + verifier + license.
10. `scripts/release_check.sh` exits 0 and is silent, with Sprint 26 gates in both
    `scripts/test.sh` and `scripts/release_check.sh`; a gate-sabotage of the classifier
    makes `release_check.sh` fail (non-decorative).

## Wrong if

- A proposal with clever wording weakens hazard_only, consolidation, mutation authority, or
  verifier behavior without a trace-level block.
- The declared `effect` (or a self-asserted delta-effect) is read as authority anywhere.
- A preserve/extend claim against a probe-guarded invariant is accepted without a passing
  trace.
- A genuine preserving extension is blocked (the gate blocks everything).
- A Sprint 24/25 gate regresses, or `release_check.sh` exits nonzero or prints output.

## Checks (commands)

```sh
python3 scripts/trace_diff.py                                                          # probe self-tests
python3 scripts/design_audit.py --scenario preserve_marker_launders_weakening_blocked  # Sprint-25 hole closed by behavior
python3 scripts/design_audit.py --scenario trace_diff_detects_hazard_gate_softening
python3 scripts/design_audit.py --scenario trace_diff_detects_consolidation_gate_softening
python3 scripts/design_audit.py --scenario trace_diff_accepts_true_preserving_extension
python3 scripts/design_audit.py --scenario design_effect_preserve_consistent           # backward compat (now trace-confirmed)
python3 scripts/design_audit.py --scenario design_proposal_consistent_with_invariants  # backward compat (now trace-confirmed)
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Residual / next boundary (explicitly deferred)

The trace closes the words-vs-behavior gap for the invariants it guards: a proposal can no
longer earn accept against a probe-guarded invariant on wording alone — it must supply a
delta whose tested behavior preserves the protected outcome. Two honest residuals remain
(neither is a safe-default claim), ranked by danger:

1. **Most dangerous — probe coverage is partial.** Only `hazard_gate`
   (`D_invariant_hazard_blocks_action`) and `consolidation_gate`
   (`D_invariant_mutation_requires_authority`) have behavioral probes. The other locked
   invariants (`no_naked_facts`, `raw_episode_append_only`, `llm_no_authority`) are still on
   the Sprint-25 lexical early-warning layer alone, so the Sprint-25 laundering hole remains
   open *for them*: a real weakening (e.g. "let the LLM's high-confidence candidates be
   auto-consolidated", which weakens `llm_no_authority`) phrased with a leading preservation
   marker and no permissive verb can still reach accept against an unprobed invariant. The
   fix is to extend probe coverage to every locked invariant; until then this is the largest
   open gap.
2. **Lesser — a mis-stated delta on a probed invariant.** The delta is the proposal's own
   description of its change. An empty or under-stated delta paired with weakening prose
   lands in `needs_review`/block, but a *mis-stated* delta whose tested behavior is a
   harmless no-op while the prose claims more is accepted as the no-op it tests as. The fix
   is delta-to-code provenance — derive the behavioral delta from the actual change set
   rather than trusting the proposal's description.

The next boundary is therefore: probes for every locked invariant, then delta-to-code
provenance so the tested delta cannot diverge from the real change.
