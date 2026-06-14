#!/usr/bin/env python3
"""Derived effect classification (Sprint 25).

Hard rule: a design proposal's effect on an invariant is DERIVED from semantic-diff
evidence between the proposal claim and the invariant claim. A self-declared `effect`
field is an untrusted hint, used only to detect mislabeling — never authority.

This is a deterministic lexical diff in the same spirit as the bridge-world verifier
rules (keyword/structure based). It classifies into:
  weaken     - relaxes/removes a protection the invariant provides
  contradict - directly negates a "never/must not" clause of the invariant
  extend     - adds new behavior while preserving the protection
  preserve   - restates/strengthens the protection without new scope
  needs_review - cannot be proven safe from evidence (blocks, never auto-accepts)
"""

from __future__ import annotations

import re

# Verbs that relax or remove a protection (proposal side). Includes participle/gerund
# forms so a weakening cannot evade detection by tense ("permitted", "allowing").
PERMISSIVE_VERBS = {
    "allow", "allows", "allowed", "allowing",
    "permit", "permits", "permitted", "permitting",
    "enable", "enables", "enabled", "enabling",
    "override", "overrides", "overriding", "overridden",
    "bypass", "bypasses", "bypassed", "bypassing",
    "relax", "relaxes", "relaxed", "relaxing",
    "remove", "removes", "removed", "removing",
    "ignore", "ignores", "ignored", "ignoring",
    "disable", "disables", "disabled", "disabling",
    "skip", "skips", "skipped", "skipping",
    "waive", "waives", "waived", "waiving",
    "lift", "lifts", "lifted", "lifting",
    "loosen", "loosens", "loosened", "loosening",
    "weaken", "weakens", "weakened", "weakening",
    "suspend", "suspends", "suspended", "suspending",
    "downgrade", "downgrades", "downgraded", "downgrading",
}

# Non-verb phrases that signal a weakening of a protection ("becomes advisory",
# "no longer mandatory"). These catch weakenings phrased without a permissive verb.
WEAKENING_PHRASES = [
    "advisory", "optional", "non-binding", "nonbinding", "not mandatory",
    "no longer", "rather than mandatory", "instead of mandatory", "becomes a suggestion",
    "treated as a hint", "may proceed", "can proceed despite", "except under", "only a recommendation",
]

# Verbs that mark the invariant as protective (invariant side).
PROTECTIVE_VERBS = {
    "block", "blocks", "blocking", "forbid", "forbids", "forbidden",
    "prevent", "prevents", "deny", "denies", "cannot", "never",
}

# Objects an invariant typically protects. Overlap between invariant and proposal on one
# of these signals the proposal is touching protected ground.
PROTECTED_OBJECTS = [
    "direct action", "hazard_only", "hazard-only", "raw episode", "raw episodes",
    "full premise", "naked fact", "epistemic license", "mutate memory",
    "assign authority", "consolidat",
]

# Markers that the proposal preserves/strengthens the protection.
PRESERVE_MARKERS = [
    "preserve", "preserves", "preserving", "still block", "still blocks",
    "never lets", "never let", "without weakening", "while preserving",
    "in addition", "strengthen", "strengthens", "tighten", "tightens",
    "keeps blocking", "keep blocking", "retain", "retains",
    "continues to block", "must still", "still hold", "still holds",
]

# Markers that the proposal adds new behavior (=> extend rather than bare preserve).
ADD_MARKERS = ["add", "adds", "introduce", "introduces", "extend", "extends", "additional", "augment", "augments"]

# Markers of a direct negation of a "never/must not" clause (=> contradict).
NEGATION_MARKERS = ["override", "overrides", "not blocked", "without blocking", "ignore the", "is safe", "bypass"]


def _has_word(word: str, text: str) -> bool:
    return re.search(r"\b" + re.escape(word) + r"\b", text) is not None


def derive_effect(proposal_claim: str, invariant_claim: str) -> dict:
    proposal = (proposal_claim or "").lower()
    invariant = (invariant_claim or "").lower()

    permissive = sorted(verb for verb in PERMISSIVE_VERBS if _has_word(verb, proposal))
    weakening_phrases = sorted(phrase for phrase in WEAKENING_PHRASES if phrase in proposal)
    protective_invariant = sorted(verb for verb in PROTECTIVE_VERBS if _has_word(verb, invariant))
    protected = [obj for obj in PROTECTED_OBJECTS if obj in invariant]
    touches = [obj for obj in protected if obj in proposal]
    preserve_hits = [marker for marker in PRESERVE_MARKERS if marker in proposal]
    add_hits = [marker for marker in ADD_MARKERS if _has_word(marker, proposal)]
    negation_hits = [marker for marker in NEGATION_MARKERS if marker in proposal]

    evidence = {
        "permissive_verbs": permissive,
        "weakening_phrases": weakening_phrases,
        "protective_invariant_verbs": protective_invariant,
        "protected_objects": protected,
        "proposal_touches_protected": touches,
        "preserve_markers": preserve_hits,
        "add_markers": add_hits,
        "negation_markers": negation_hits,
    }
    invariant_is_protective = bool(protective_invariant)
    weakening_signal = permissive or weakening_phrases

    # 1. Explicit weakening evidence (permissive verb or weakening phrase) aimed at a
    #    protected object the invariant guards => weaken/contradict. Checked first so that
    #    a preservation marker cannot launder a weakening.
    if weakening_signal and invariant_is_protective and touches:
        negates_never = (
            ("never" in invariant) or ("must not" in invariant) or ("cannot" in invariant)
        ) and (bool(negation_hits) or bool(weakening_phrases) or (("urgency" in proposal) and ("urgency" in invariant)))
        effect = "contradict" if negates_never else "weaken"
        return {"effect": effect, "basis": "weakening_signal_on_protected_object", "evidence": evidence}

    # 2. Weakening evidence but no clear protected-object overlap => cannot prove safe.
    if weakening_signal:
        return {"effect": "needs_review", "basis": "weakening_signal_without_clear_target", "evidence": evidence}

    # 3. Explicit preservation evidence and no weakening => extend (adds) or preserve.
    if preserve_hits:
        effect = "extend" if add_hits else "preserve"
        basis = "preserves_protection_and_adds" if add_hits else "preserves_protection"
        return {"effect": effect, "basis": basis, "evidence": evidence}

    # 4. Touches a protected object but offers no preservation evidence => cannot be proven
    #    safe (it may attack the protection with novel phrasing) => needs_review (blocks).
    if touches and invariant_is_protective:
        return {"effect": "needs_review", "basis": "touches_protected_without_preservation_evidence", "evidence": evidence}

    # 5. Adds new behavior that does not touch protected ground => extend.
    if add_hits and not touches:
        return {"effect": "extend", "basis": "adds_unrelated_behavior", "evidence": evidence}

    # 6. Cannot classify from evidence => needs review (never auto-accept).
    return {"effect": "needs_review", "basis": "insufficient_evidence", "evidence": evidence}


def effect_family(effect: str) -> str:
    if effect in ("weaken", "contradict"):
        return "weakening"
    if effect in ("extend", "preserve"):
        return "consistent"
    return "unknown"


# Deterministic self-test: run `python3 scripts/effect_classifier.py`.
_SELF_TESTS = [
    (
        "Allow direct action under hazard_only contradiction when urgency is high so urgent crossings are not blocked.",
        "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
        "weakening",
    ),
    (
        "Add staleness demotion that preserves hazard_only blocking and never lets demoted memory support a full premise.",
        "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
        "consistent",
    ),
    (
        "The hazard_only block on direct action must still hold under all urgency levels.",
        "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
        "consistent",
    ),
    (
        # Lexicon-avoiding weakening (no permissive verb): caught by weakening phrases.
        "Under high urgency, hazard_only contradiction evidence becomes advisory rather than mandatory for direct action.",
        "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
        "weakening",
    ),
    (
        # Touches protected ground with no preservation evidence: cannot prove safe => unknown (blocks).
        "Revisit how hazard_only contradiction evidence interacts with direct action during urgent crossings.",
        "Hazard-only contradiction evidence blocks direct action, and urgency must never override that block.",
        "unknown",
    ),
]


def main() -> int:
    failures = 0
    for proposal, invariant, expected_family in _SELF_TESTS:
        result = derive_effect(proposal, invariant)
        family = effect_family(result["effect"])
        ok = family == expected_family
        failures += 0 if ok else 1
        print(f"{'PASS' if ok else 'FAIL'}  derived={result['effect']:11} family={family:10} expected={expected_family}  basis={result['basis']}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
