# Sprint 27 — Complete Locked-Invariant Probe Coverage

Status: Complete.

## Why

Sprint 26 grounded the effect in behavior, but only for two invariants (`hazard_gate`,
`consolidation_gate`). Its largest residual: the other three locked invariants
(`no_naked_facts`, `raw_episode_append_only`, `llm_no_authority`) were still protected by the
Sprint-25 lexical layer alone, so a leading-preservation-marker weakening could still launder
past the gate against an unprobed invariant. This sprint binds a real runtime probe to every
locked invariant and makes "no probe" mean "not eligible for preserve/extend acceptance".

## Hard rule (the invariant this sprint locks)

```text
A locked invariant without a behavioral probe is NOT eligible for preserve/extend acceptance.
No probe means no proof of preservation: an unprobed locked invariant defaults to needs_review.
```

## Doctrine

```text
A protected invariant is only protected if the system can test what breaking it looks like.
```

## Build

```text
scripts/trace_diff.py
  + naked_fact_gate     -> retrieval_policy.emergency_use_protocol (a naked fact is do_not_use_for_action -> cannot_support_action; the weakening grants it full_premise -> normal_use)
  + raw_append_only_gate-> raw_episode_store.RawEpisodeStore (append-only is structurally immutable; the weakening's tested behavior must invoke store.replace, which the real store refuses -> outcome diverges from the untouched baseline)
  + llm_authority_gate  -> raw_episode_store.semantic_candidate_from_raw + mutation_gateway.apply_memory_mutation (an LLM candidate's real forbidden_use blocks consolidation -> rejected; the weakening grants authority -> consolidated)
  combine_effects(lexical, trace, invariant_locked): a LOCKED invariant with no probe and a consistent lexical verdict -> needs_review ("locked_invariant_without_probe")
simulations/bridge_world/design_memory.json  behavioral_probe on no_naked_facts / raw_episode_append_only / llm_no_authority; DD_sprint_27
project_self_audit.evaluate_design_proposal   passes invariant_locked (status == regression_lock) into combine_effects
scenarios: trace_diff_blocks_no_naked_facts_laundering, trace_diff_blocks_raw_episode_append_only_laundering, trace_diff_blocks_llm_authority_laundering
```

## Rubric — DONE means ALL of these are checkable PASS

1. Every locked (`regression_lock`) invariant in `design_memory.json` has a `behavioral_probe`
   bound to a probe in `trace_diff.PROBES`, and every probe's baseline reproduces its protected
   outcome by running real runtime code.
2. `trace_diff_blocks_no_naked_facts_laundering`: a claim the LEXICAL layer reads as
   `preserve`, whose delta flips the naked-fact gate, is blocked from the trace
   (`trace_regressed: true`, `effect_authority: trace_behavior_regression`, derived weakening,
   `governance_decision: block`).
3. `trace_diff_blocks_raw_episode_append_only_laundering`: same, against the append-only store
   (the proposal's tested behavior hits the real store's refusal → regression → block).
4. `trace_diff_blocks_llm_authority_laundering`: same, against the LLM-authority gate (the real
   candidate's `forbidden_use` blocks consolidation at baseline; the weakening grants authority
   → `consolidated` → regression → block).
5. **Structural rule:** a preserve/extend claim against a LOCKED invariant that has NO probe
   lands in `needs_review` and blocks (`effect_authority: locked_invariant_without_probe`) — an
   unprobed locked invariant cannot reach accept. Verified with a synthetic locked-no-probe
   invariant.
6. For EACH locked invariant, a genuine preserving extension (a no-regression delta) is
   ACCEPTED — every gate confirms preservation, none blocks everything.
7. A fake/no-op `behavioral_delta` (or a malformed one) for a protected invariant cannot reach
   accept without a real probe pass: with no matching delta it is `untested` → `needs_review`.
8. `decision_audit.py --project --strict` passes with zero violations; DD_sprint_27 recorded.
9. The Sprint 24/25/26 gates do not regress.
10. `scripts/release_check.sh` exits 0 and is silent, with Sprint 27 gates in both
    `scripts/test.sh` and `scripts/release_check.sh`; gate-sabotage of the structural rule or a
    probe makes `release_check.sh` fail (non-decorative).

## Wrong if

- Any locked invariant is still protected only by lexical phrases (no probe).
- A preserve-marker weakening is accepted because no probe exists for the targeted invariant.
- A proposal is classified `preserve`/`extend` against a locked invariant without behavioral
  evidence (a passing trace).
- A fake/no-op `behavioral_delta` is accepted for a protected invariant lacking a real probe.
- A genuine preserving extension is blocked, or a Sprint 24/25/26 gate regresses, or
  `release_check.sh` exits nonzero or prints output.

## Checks (commands)

```sh
python3 scripts/trace_diff.py
python3 scripts/design_audit.py --scenario trace_diff_blocks_no_naked_facts_laundering
python3 scripts/design_audit.py --scenario trace_diff_blocks_raw_episode_append_only_laundering
python3 scripts/design_audit.py --scenario trace_diff_blocks_llm_authority_laundering
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Residual / next boundary (explicitly deferred)

Every locked invariant is now probe-backed and an unprobed locked invariant cannot earn accept,
so the Sprint-26 unprobed-invariant laundering hole is closed. The remaining residual (no
safe-default claim): the probe runs the proposal's self-described `behavioral_delta`, so a
mis-stated delta whose tested behavior is a harmless no-op while the prose claims more is
accepted as the no-op it tests as. The next boundary is delta-to-code provenance — derive the
behavioral delta from the actual change set so the tested delta cannot diverge from what the
proposal would really do. (The `raw_append_only_gate` is the honest edge of the behavioral
model: append-only is enforced by an immutable store, so the probe detects a weakening by the
proposal's tested behavior having to invoke the store's refused `replace`, rather than by a
tunable outcome — documented so the mechanism is not mistaken for a runtime that can itself be
weakened from outside.)
