# Sprint 25 — Derived Effect Classification

Status: Complete.

## Why

Sprint 24 gated design proposals on a self-declared `effect` field (`weaken`/`extend`).
That is a "label says safe" gap: a proposal could mislabel a weakening as an extension
and bypass the gate. This sprint closes it by deriving the effect from semantic-diff
evidence between the proposal claim and the targeted invariant claim.

## Hard rule (the invariant this sprint locks)

```text
effect is evidence-derived metadata.
effect is NOT user / config / assertion authority.
A self-declared effect is an untrusted hint, used only to detect mislabeling.
```

## Goal

Derive whether a change `weakens`, `contradicts`, `extends`, or `preserves` an invariant
from the claims themselves, not from a declared field. A weakening claim labeled
`extend` must be reclassified to a weakening from evidence, flagged as a mislabel, and
blocked exactly as an honestly-declared weakening is.

## Build

```text
scripts/effect_classifier.py                          (derive_effect: semantic-diff -> weaken/contradict/extend/preserve/needs_review; effect_family)
simulations/bridge_world/design_verifier_rules.json   (rules for contradict / preserve / needs_review)
project_self_audit.evaluate_design_proposal           (derives effect; declared effect is hint-only; effect_mislabel via family mismatch)
bridge_world_demo._run_design_proposal_scenario        (surfaces declared_effect, derived_effect, effect_mislabel)
scripts/design_audit.py                                (reports declared/derived/mislabel)
scenarios: design_effect_mislabel_attack, design_effect_derived_without_declaration, design_effect_preserve_consistent, design_effect_lexicon_avoiding_weaken, design_effect_ambiguous_needs_review
```

## Rubric — DONE means ALL of these are checkable PASS

1. A weakening claim **declared `effect: extend`** is reclassified: `derived_effect` is a
   weakening (`weaken` or `contradict`), `effect_mislabel: true`, `governance_decision:
   block`, `proposal_consolidated: false`, hazard_only contradiction, revalidation
   scheduled. The lie does not get it past the gate.
2. A weakening claim with **no declared `effect` at all** is still classified as a
   weakening from evidence and blocked — proving config/declaration is not required and
   not authoritative.
3. A genuinely consistent claim (preserves/strengthens the invariant) derives `extend`
   or `preserve`, `effect_mislabel: false`, and is accepted/consolidated.
4. The Sprint 24 scenarios still pass unchanged (backward compatible): the honest
   weakening still blocks, the honest extend still accepts; `effect_mislabel: false` for
   both (declared and derived agree on family).
5. The runtime `adjudicate()` still independently confirms the hard-contradiction →
   `reject_episode` outcome on the derived effect (no lenient self-grading).
6. `decision_audit.py --project --strict` still passes with zero violations; the
   Sprint 25 decision is recorded with trace + verifier + license.
7. `scripts/release_check.sh` exits 0 and is silent, with Sprint 25 gates in both
   `scripts/test.sh` and `scripts/release_check.sh`.

## Wrong if

- A weakening labeled `extend` is accepted or consolidated (the attack succeeds).
- `effect` is read from the proposal/config as authority anywhere in the decision path.
- An unknown/ambiguous proposal is auto-accepted (needs_review must block, not pass).
- The Sprint 24 gates regress.
- `release_check.sh` exits nonzero or prints output.

## Checks (commands)

```sh
python3 scripts/design_audit.py --scenario design_effect_mislabel_attack
python3 scripts/design_audit.py --scenario design_effect_derived_without_declaration
python3 scripts/design_audit.py --scenario design_effect_preserve_consistent
python3 scripts/design_audit.py --scenario design_effect_lexicon_avoiding_weaken     # bypass closed
python3 scripts/design_audit.py --scenario design_effect_ambiguous_needs_review      # needs_review blocks
python3 scripts/design_audit.py --scenario design_contradiction_in_sprint_plan       # backward compat
python3 scripts/design_audit.py --scenario design_proposal_consistent_with_invariants # backward compat
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Review result

```text
weakening declared extend -> reclassified weakening + mislabel + blocked   PASS
weakening with no declared effect -> classified from evidence + blocked    PASS
consistent claim -> derived extend/preserve + accepted + no mislabel       PASS
Sprint 24 scenarios unchanged (honest weaken blocks, honest extend accepts) PASS
runtime adjudicate confirms hard_contradiction -> reject_episode           PASS
project strict audit passes with zero violations                          PASS
release_check.sh silent                                                   PASS
```

## Residual / next boundary (explicitly deferred)

The classifier is a deterministic lexical semantic-diff (permissive verbs, weakening
phrases, protective verbs, protected objects, preservation markers) — consistent with the
bridge verifier's keyword rules. What it blocks (release-gated): a permissive verb or a
weakening phrase aimed at a protected object the invariant guards (→ weaken/contradict),
and touching a protected object without any preservation evidence (→ `needs_review`, which
blocks — `design_effect_ambiguous_needs_review`). A weakening phrased without a permissive
verb (e.g. "becomes advisory rather than mandatory") is caught by the weakening-phrase
lexicon (`design_effect_lexicon_avoiding_weaken`).

Honest remaining limit (NOT a safe-default claim): a claim that pairs an explicit
preservation marker with a weakening phrased entirely outside BOTH the permissive-verb and
weakening-phrase lexicons can still reach the preserve/accept path. The lexicon makes that
harder but cannot be exhaustive. The real fix is evidence richer than lexical — a
trace-grounded diff against the invariant's regression scenarios, so a weakening is
detected by what it would break, not by the words it uses. That is the deferred next
boundary.
