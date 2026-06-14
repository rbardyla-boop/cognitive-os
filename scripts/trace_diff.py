#!/usr/bin/env python3
"""Trace-grounded invariant diff (Sprint 26).

Words are claims; traces are evidence. Sprint 25 derived a design proposal's effect from a
lexical diff of the claim vs. the invariant claim. This module derives the effect from
BEHAVIOR: each protected invariant is bound to a real runtime probe (the same
``adjudicate`` / ``apply_memory_mutation`` machinery that governs the bridge world). The
probe is run at baseline and with the proposal's stated behavioral delta applied; if the
protected outcome regresses, the change is a weakening — regardless of how it is worded.

Hard rule: a protected invariant's preservation must be TESTED, not trusted. A proposal
targeting a probe-guarded invariant is accepted only if a trace proves the protected
outcome did not regress. The declared effect and the lexical classifier cannot override
the trace.
"""

from __future__ import annotations

from typing import Callable

from verifier_engine import adjudicate
from mutation_gateway import (
    apply_memory_mutation,
    verifier_allows_mutation,
)
from retrieval_policy import emergency_use_protocol
from raw_episode_store import RawEpisode, RawEpisodeStore, semantic_candidate_from_raw
from change_provenance import (
    build_content_change_set,
    load_baseline_policy,
    verify_change_set_provenance,
)
from mechanism_provenance import (
    MECHANISM_PROVENANCE_FAILURES,
    probe_outcome_for_proposed_source,
    verify_mechanism_change_provenance,
)


class BehavioralProbe:
    """A real, deterministic runtime probe. ``run(policy)`` returns the observed outcome.

    The protected outcome is what ``run({})`` must return (the protection intact). Any other
    outcome under a proposal's policy is a regression of the protected behavior.
    ``regress_effect`` is how a regression is classified (``contradict`` when the invariant
    carries a never-clause, ``weaken`` otherwise).
    """

    def __init__(
        self,
        control_point: str,
        protected_outcome: str,
        run: Callable[[dict], str],
        regress_effect: str,
    ) -> None:
        self.control_point = control_point
        self.protected_outcome = protected_outcome
        self._run = run
        self.regress_effect = regress_effect

    def run(self, policy: dict) -> str:
        return self._run(policy or {})


def _hazard_gate_outcome(policy: dict) -> str:
    """Protected: a hazard contradiction blocks direct action (``reject_episode``); high
    urgency must NOT lift that block. The proposed ``urgency_overrides_hazard`` delta
    reclassifies the hazard contradiction as a tolerable known-exception, so the real
    adjudicator no longer rejects the action."""
    if policy.get("urgency_overrides_hazard"):
        return adjudicate("known_exception", 0.6, repeated_anomalies=1)
    return adjudicate("hard_contradiction", 0.2, repeated_anomalies=1)


def _consolidation_gate_outcome(policy: dict) -> str:
    """Protected: a consolidation from a source whose authority forbids ``memory_consolidation``
    is rejected by the real mutation gateway. The proposed ``authorize_unprivileged_consolidation``
    delta grants that unprivileged source consolidation rights, so the real gateway applies it."""
    grant = bool(policy.get("authorize_unprivileged_consolidation"))
    target = {"memory_id": "D_probe_consolidation_target", "status": "candidate"}
    source = {
        "header": {"packet_id": "P_probe_consolidation", "source_engine": "design_author"},
        "permissions": {
            "allowed_use": ["memory_consolidation", "human_review"] if grant else ["human_review"],
            "forbidden_use": [] if grant else ["memory_consolidation"],
        },
    }
    decision = verifier_allows_mutation(
        "V_DEC_probe_consolidation",
        "memory_confidence_update",
        "memory_consolidation",
        "D_probe_consolidation_target",
        "P_probe_consolidation",
    )
    request = {
        "mutation_id": "MUT_probe_consolidation",
        "trace_id": "T_probe_consolidation",
        "source_packet_id": "P_probe_consolidation",
        "verifier_decision_id": "V_DEC_probe_consolidation",
        "target_object_id": "D_probe_consolidation_target",
        "requested_use": "memory_consolidation",
        "mutation_type": "memory_confidence_update",
        "patch": {"status": "consolidated"},
        "authority_snapshot": {"forbidden_use": []},
    }
    result = apply_memory_mutation(request, target, source, decision, [])
    return "consolidated" if result["applied"] else "rejected"


def _no_naked_facts_outcome(policy: dict) -> str:
    """Protected: a fact returned without a license/provenance is treated as
    ``do_not_use_for_action`` and the real retrieval policy refuses to let it support action.
    The proposed ``allow_naked_facts`` delta returns the naked fact as a usable premise."""
    epistemic_license = "full_premise" if policy.get("allow_naked_facts") else "do_not_use_for_action"
    return emergency_use_protocol(epistemic_license, urgent=True)


def _raw_probe_episode(text: str) -> RawEpisode:
    return RawEpisode(
        episode_id="RE_probe_append_only",
        trace_id="T_probe_append_only",
        source="probe",
        timestamp="2026-06-13T00:00:00Z",
        logical_tick=0,
        raw_payload={"text": text},
        modality="report",
        capture_context={},
        integrity_digest="d_probe",
        ingestion_license="semantic_candidate_allowed",
    )


def _raw_append_only_outcome(policy: dict) -> str:
    """Append-only is enforced by an immutable store: ``RawEpisodeStore.replace`` always
    refuses. The protected baseline performs no rewrite. The proposed ``allow_raw_overwrite``
    delta rewrites an existing raw episode — exercised against the REAL store, which refuses,
    so the outcome diverges from the untouched baseline (the protection a weakening must defeat)."""
    store = RawEpisodeStore()
    store.append(_raw_probe_episode("original"))
    if policy.get("allow_raw_overwrite"):
        try:
            store.replace("RE_probe_append_only", {"raw_payload": {"text": "rewritten"}})
            return "raw_episode_rewritten"
        except PermissionError:
            return "raw_overwrite_blocked_by_store"
    return "append_only_preserved"


def _llm_authority_outcome(policy: dict) -> str:
    """Protected: an LLM-proposed candidate (built by the real ``semantic_candidate_from_raw``)
    defaults to ``hypothesis_only`` and its ``forbidden_use`` blocks ``memory_consolidation``,
    so routing it through the real mutation gateway as a consolidation is rejected. The proposed
    ``grant_llm_authority`` delta grants the candidate consolidation authority -> the gateway applies."""
    raw_episode = {
        "episode_id": "RE_llm_probe",
        "integrity_digest": "d_llm_probe",
        "ingestion_license": "semantic_candidate_allowed",
    }
    candidate = semantic_candidate_from_raw(raw_episode, "LLM-proposed claim", "CMN_llm_probe")
    grant = bool(policy.get("grant_llm_authority"))
    source = {
        "header": {"packet_id": "P_llm_probe", "source_engine": "llm_extractor"},
        "permissions": {
            "allowed_use": ["memory_consolidation", "human_review"] if grant else candidate["allowed_use"],
            "forbidden_use": [] if grant else candidate["forbidden_use"],
        },
    }
    decision = verifier_allows_mutation(
        "V_DEC_llm_probe",
        "memory_confidence_update",
        "memory_consolidation",
        "CMN_llm_probe",
        "P_llm_probe",
    )
    request = {
        "mutation_id": "MUT_llm_probe",
        "trace_id": "T_llm_probe",
        "source_packet_id": "P_llm_probe",
        "verifier_decision_id": "V_DEC_llm_probe",
        "target_object_id": "CMN_llm_probe",
        "requested_use": "memory_consolidation",
        "mutation_type": "memory_confidence_update",
        "patch": {"status": "consolidated"},
        "authority_snapshot": {"forbidden_use": []},
    }
    target = {"memory_id": "CMN_llm_probe", "status": "semantic_candidate"}
    result = apply_memory_mutation(request, target, source, decision, [])
    return "consolidated" if result["applied"] else "rejected"


# Registry: invariant ``behavioral_probe`` id -> the real probe that guards it.
PROBES: dict[str, BehavioralProbe] = {
    "hazard_gate": BehavioralProbe(
        control_point="hazard_gate",
        protected_outcome="reject_episode",
        run=_hazard_gate_outcome,
        regress_effect="contradict",  # invariant carries "urgency must never override"
    ),
    "consolidation_gate": BehavioralProbe(
        control_point="consolidation_gate",
        protected_outcome="rejected",
        run=_consolidation_gate_outcome,
        regress_effect="weaken",
    ),
    "naked_fact_gate": BehavioralProbe(
        control_point="naked_fact_gate",
        protected_outcome="cannot_support_action",
        run=_no_naked_facts_outcome,
        regress_effect="weaken",
    ),
    "raw_append_only_gate": BehavioralProbe(
        control_point="raw_append_only_gate",
        protected_outcome="append_only_preserved",
        run=_raw_append_only_outcome,
        regress_effect="weaken",
    ),
    "llm_authority_gate": BehavioralProbe(
        control_point="llm_authority_gate",
        protected_outcome="rejected",
        run=_llm_authority_outcome,
        regress_effect="weaken",
    ),
}


def derive_effect_from_trace(invariant: dict | None, proposal: dict) -> dict:
    """Run the bound probe pre/post the delta DERIVED from a provenance-verified change_set.

    Sprint 28: the tested delta is derived from ``proposal["change_set"]`` (a structured patch
    bound to the real changed artifact), never from the self-declared ``behavioral_delta`` —
    a delta without provenance is just a label. Missing/unverifiable provenance -> untested.

    status:
      not_probed           - the targeted invariant has no bound probe (lexical layer only)
      untested             - probe-guarded, but no provenance-verified change_set to test
      probe_integrity_error- baseline did not reproduce the protected outcome (fail safe -> blocks)
      tested               - the probe ran on the change_set-derived delta; ``regressed`` is authoritative
    """
    probe_id = (invariant or {}).get("behavioral_probe")
    base = {
        "trace_effect": "not_applicable",
        "status": "not_probed",
        "tested": False,
        "regressed": False,
        "control_point": None,
        "protected_outcome": None,
        "pre": None,
        "post": None,
        "probe_id": probe_id,
        "provenance": "not_applicable",
        "changed_artifact": None,
        "pre_image_hash": None,
        "post_image_hash": None,
        "diff_digest": None,
        "delta_matches_change_set": None,
        "mechanism_source": False,
        "mechanism_role": None,
    }
    if not probe_id or probe_id not in PROBES:
        return base

    probe = PROBES[probe_id]
    base["control_point"] = probe.control_point
    base["protected_outcome"] = probe.protected_outcome

    # Sprint 29: the baseline is the behavior of the on-disk policy artifact's CURRENT content.
    # If that content no longer yields the protected outcome (a tampered policy file), block.
    try:
        baseline_policy = load_baseline_policy(probe_id)
    except (OSError, ValueError):
        base.update({"trace_effect": "needs_review", "status": "probe_integrity_error", "provenance": "policy_artifact_unreadable"})
        return base
    baseline = probe.run(baseline_policy)
    base["pre"] = baseline
    if baseline != probe.protected_outcome:
        # The protected baseline no longer holds: never silently accept.
        base.update({"trace_effect": "needs_review", "status": "probe_integrity_error", "provenance": "probe_integrity_error"})
        return base

    # Sprint 32: a change to the mechanism SOURCE (the enforcement code) is tested by running the
    # bound probe against the PROPOSED source, not against a policy artifact.
    change_set = proposal.get("change_set")
    if isinstance(change_set, dict) and change_set.get("binding") == "mechanism_source":
        return _derive_mechanism_effect(base, probe, change_set)

    # The tested delta must come from a content-verified change_set, not a self-declared delta.
    provenance = verify_change_set_provenance(proposal.get("change_set"))
    base["provenance"] = provenance["reason"]
    base["changed_artifact"] = provenance["changed_artifact"]
    base["pre_image_hash"] = provenance.get("pre_image_hash")
    base["post_image_hash"] = provenance.get("post_image_hash")
    base["diff_digest"] = provenance.get("diff_digest")
    if not provenance["ok"]:
        # No verifiable change_set: a self-declared behavioral_delta is not trusted -> untested.
        base.update({"trace_effect": "untested", "status": "untested"})
        return base

    derived_delta = provenance["derived_delta"]
    if derived_delta["control_point"] != probe.control_point:
        # The change_set patches a different control point than this invariant's probe.
        base.update({"trace_effect": "untested", "status": "untested", "provenance": "target_mismatch"})
        return base

    # Surface whether a self-declared behavioral_delta agrees with the provenance-derived one
    # (a hint only — the change_set is authority).
    declared = proposal.get("behavioral_delta")
    if isinstance(declared, dict):
        base["delta_matches_change_set"] = (
            declared.get("control_point") == derived_delta["control_point"]
            and declared.get("policy", {}) == derived_delta["policy"]
        )

    policy = derived_delta["policy"]
    post = probe.run(policy)
    regressed = post != probe.protected_outcome
    if regressed:
        trace_effect = probe.regress_effect
    else:
        trace_effect = "extend" if derived_delta.get("adds") else "preserve"
    base.update({
        "trace_effect": trace_effect,
        "status": "tested",
        "tested": True,
        "regressed": regressed,
        "post": post,
    })
    return base


def _derive_mechanism_effect(base: dict, probe: BehavioralProbe, change_set: dict) -> dict:
    """Sprint 32: test a proposed change to the mechanism SOURCE that enforces a gate.

    The change_set binds the literal before/after content of a real enforcement-code file (its
    pre-image must equal the CURRENT on-disk source). The bound probe is run against the PROPOSED
    post-image source: if the gate's protected outcome no longer survives, the change is a
    weakening of the enforcement code — caught here even when the policy artifacts stay clean.
    A failure to demonstrate the protected outcome (a probe error) is a regression (fail closed).
    """
    base["mechanism_source"] = True
    base["mechanism_role"] = change_set.get("role")
    provenance = verify_mechanism_change_provenance(change_set)
    base["provenance"] = provenance["reason"]
    base["changed_artifact"] = provenance["changed_artifact"]
    base["pre_image_hash"] = provenance.get("pre_image_hash")
    base["post_image_hash"] = provenance.get("post_image_hash")
    base["diff_digest"] = provenance.get("diff_digest")
    if not provenance["ok"]:
        # No verifiable mechanism-source change_set: cannot test the proposed enforcement code.
        base.update({"trace_effect": "untested", "status": "untested"})
        return base
    post = probe_outcome_for_proposed_source(provenance["role"], change_set.get("post_image"))
    base["post"] = post
    regressed = post != probe.protected_outcome
    if regressed:
        trace_effect = probe.regress_effect
    else:
        # A real source diff that preserves the protected outcome is an extension.
        trace_effect = "extend" if provenance.get("post_image_hash") != provenance.get("pre_image_hash") else "preserve"
    base.update({"trace_effect": trace_effect, "status": "tested", "tested": True, "regressed": regressed})
    return base


def combine_effects(lexical_effect: str, trace: dict, invariant_locked: bool = False) -> dict:
    """Combine the Sprint-25 lexical early-warning effect with the Sprint-26 trace.

    The trace is authority: a tested regression overrides any lexical or declared
    "consistent" verdict. The lexical layer can only RAISE severity (catch a weakening the
    trace did not model), never grant accept. A probe-guarded invariant cannot be accepted
    as preserve/extend without a trace that proved no regression.

    Sprint 27 structural rule: a LOCKED invariant with no probe is not eligible for
    preserve/extend acceptance — no probe means no proof of preservation, so it defaults to
    needs_review. (Unlocked invariants keep the Sprint-25 lexical fallback.)

    Sprint 28 provenance rule: a LOCKED invariant whose change_set provenance is missing or
    unverifiable cannot earn accept — a self-declared delta is just a label (authority
    ``delta_provenance_unverified``).
    """
    if trace["tested"] and trace["regressed"]:
        return {"effect": trace["trace_effect"], "authority": "trace_behavior_regression"}
    if lexical_effect in ("weaken", "contradict"):
        return {"effect": lexical_effect, "authority": "lexical_early_warning"}
    if lexical_effect == "needs_review":
        return {"effect": "needs_review", "authority": "lexical_needs_review"}
    # lexical_effect is consistent (extend/preserve) here.
    if trace["tested"] and not trace["regressed"]:
        return {"effect": lexical_effect, "authority": "trace_confirmed_preservation"}
    if trace["status"] in ("untested", "probe_integrity_error"):
        # Probe-guarded invariant, but preservation was never tested: words cannot earn accept.
        provenance = trace.get("provenance")
        if invariant_locked and provenance in (
            "missing", "malformed_images", "artifact_unknown", "artifact_mismatch",
            "artifact_missing", "stale_pre_image", "wrong_post_image", "diff_digest_mismatch",
            "non_applicable_patch", "structured_patch_diverges", "target_mismatch",
        ) + MECHANISM_PROVENANCE_FAILURES:
            # No content-verified change_set binds the tested delta to the actual artifact.
            return {"effect": "needs_review", "authority": "delta_provenance_unverified"}
        return {"effect": "needs_review", "authority": "preservation_not_tested"}
    # Not probe-guarded (trace status == not_probed).
    if invariant_locked:
        # A locked invariant with no probe cannot be tested -> no proof of preservation.
        return {"effect": "needs_review", "authority": "locked_invariant_without_probe"}
    # Unlocked invariant: fall back to the Sprint-25 lexical verdict.
    return {"effect": lexical_effect, "authority": "lexical_only_no_probe"}


def _change_set(probe_id: str, patch: dict, adds: bool = False) -> dict:
    """Build a content-bound change_set whose post-image is the baseline policy with ``patch``
    applied (test helper). The literal post-image is what the probe tests."""
    pre_policy = load_baseline_policy(probe_id)
    post_policy = dict(pre_policy, **patch)
    return build_content_change_set(probe_id, pre_policy, post_policy)


def _change_set_stale(probe_id: str, patch: dict) -> dict:
    """Build a change_set whose pre-image does NOT match the on-disk artifact (test helper)."""
    real_pre = load_baseline_policy(probe_id)
    stale_pre = dict(real_pre, _stale_marker=True)  # not the real on-disk content
    return build_content_change_set(probe_id, stale_pre, dict(stale_pre, **patch))


# Deterministic self-test: run `python3 scripts/trace_diff.py`.
# (probe_id, patch, adds, expect_regressed, expect_post) — tested via a provenance-verified change_set.
_SELF_TESTS = [
    ("hazard_gate", {"urgency_overrides_hazard": True}, False, True, "preserve_as_exception"),
    ("hazard_gate", {}, True, False, "reject_episode"),
    ("consolidation_gate", {"authorize_unprivileged_consolidation": True}, False, True, "consolidated"),
    ("consolidation_gate", {}, False, False, "rejected"),
    ("naked_fact_gate", {"allow_naked_facts": True}, False, True, "normal_use"),
    ("naked_fact_gate", {}, False, False, "cannot_support_action"),
    ("raw_append_only_gate", {"allow_raw_overwrite": True}, False, True, "raw_overwrite_blocked_by_store"),
    ("raw_append_only_gate", {}, False, False, "append_only_preserved"),
    ("llm_authority_gate", {"grant_llm_authority": True}, False, True, "consolidated"),
    ("llm_authority_gate", {}, False, False, "rejected"),
]


def main() -> int:
    failures = 0

    def record(label: str, ok: bool) -> None:
        nonlocal failures
        failures += 0 if ok else 1
        print(f"{'PASS' if ok else 'FAIL'}  {label}")

    # Each probe's baseline (the on-disk policy artifact's content) must reproduce its protected outcome.
    for probe_id, probe in PROBES.items():
        baseline = probe.run(load_baseline_policy(probe_id))
        record(f"probe={probe_id:18} baseline={baseline:22} protected={probe.protected_outcome}", baseline == probe.protected_outcome)
    # The tested delta is derived from a provenance-verified change_set.
    for probe_id, patch, adds, expect_regressed, expect_post in _SELF_TESTS:
        trace = derive_effect_from_trace({"behavioral_probe": probe_id}, {"change_set": _change_set(probe_id, patch, adds)})
        ok = trace["status"] == "tested" and trace["regressed"] == expect_regressed and trace["post"] == expect_post and trace["provenance"] == "verified"
        record(
            f"probe={probe_id:18} regressed={str(trace['regressed']):5} post={str(trace['post']):22} "
            f"effect={trace['trace_effect']:12} provenance={trace['provenance']}",
            ok,
        )
    # Combine: a tested regression overrides a lexical 'preserve' (the laundering case).
    laundered = combine_effects("preserve", {"tested": True, "regressed": True, "trace_effect": "contradict", "status": "tested"})
    record(
        f"combine: lexical=preserve + trace regression -> {laundered['effect']} ({laundered['authority']})",
        laundered["effect"] == "contradict" and laundered["authority"] == "trace_behavior_regression",
    )
    # Sprint 28: a self-declared behavioral_delta with NO change_set is not trusted -> untested/block.
    no_prov = derive_effect_from_trace(
        {"behavioral_probe": "hazard_gate"},
        {"behavioral_delta": {"control_point": "hazard_gate", "policy": {"urgency_overrides_hazard": True}}},
    )
    record(
        f"self-declared delta, no change_set -> status={no_prov['status']} provenance={no_prov['provenance']}",
        no_prov["status"] == "untested" and no_prov["provenance"] == "missing",
    )
    prov_block = combine_effects("preserve", no_prov, invariant_locked=True)
    record(
        f"combine: locked + missing provenance -> {prov_block['effect']} ({prov_block['authority']})",
        prov_block["effect"] == "needs_review" and prov_block["authority"] == "delta_provenance_unverified",
    )
    # Sprint 28: a mis-stated no-op behavioral_delta with a weakening change_set patch -> the
    # PATCH is tested (regresses), and the declared delta is flagged as not matching.
    misstated = derive_effect_from_trace(
        {"behavioral_probe": "hazard_gate"},
        {
            "behavioral_delta": {"control_point": "hazard_gate", "policy": {}},
            "change_set": _change_set("hazard_gate", {"urgency_overrides_hazard": True}),
        },
    )
    record(
        f"mis-stated no-op delta + weakening patch -> regressed={misstated['regressed']} matches={misstated['delta_matches_change_set']}",
        misstated["regressed"] is True and misstated["delta_matches_change_set"] is False,
    )
    # Sprint 29: a tampered diff_digest -> untested + diff_digest_mismatch -> block.
    tampered = derive_effect_from_trace(
        {"behavioral_probe": "hazard_gate"},
        {"change_set": {**_change_set("hazard_gate", {"urgency_overrides_hazard": True}), "diff_digest": "0" * 64}},
    )
    record(
        f"tampered diff_digest -> status={tampered['status']} provenance={tampered['provenance']}",
        tampered["status"] == "untested" and tampered["provenance"] == "diff_digest_mismatch",
    )
    # Sprint 29: a stale pre-image (not matching the on-disk artifact content) -> block.
    stale = derive_effect_from_trace(
        {"behavioral_probe": "hazard_gate"},
        {"change_set": _change_set_stale("hazard_gate", {"urgency_overrides_hazard": True})},
    )
    record(
        f"stale pre-image -> status={stale['status']} provenance={stale['provenance']}",
        stale["status"] == "untested" and stale["provenance"] == "stale_pre_image",
    )
    # A malformed change_set must fail closed to the designed untested block, not crash.
    for bad_cs in ("not-a-dict", {"target": "hazard_gate", "changed_artifact": "x", "pre_image": ["bad"]}, 12):
        malformed = derive_effect_from_trace({"behavioral_probe": "hazard_gate"}, {"change_set": bad_cs})
        record(f"malformed change_set -> status={malformed['status']} (fail-closed, no crash)", malformed["status"] == "untested" and not malformed["tested"])
    # Sprint 27 structural rule: a LOCKED invariant with no probe cannot earn preserve/extend accept.
    not_probed = derive_effect_from_trace({"memory_id": "D_locked_no_probe"}, {})
    locked = combine_effects("preserve", not_probed, invariant_locked=True)
    record(
        f"combine: locked invariant + no probe -> {locked['effect']} ({locked['authority']})",
        locked["effect"] == "needs_review" and locked["authority"] == "locked_invariant_without_probe",
    )
    unlocked = combine_effects("preserve", not_probed, invariant_locked=False)
    record(f"combine: unlocked invariant + no probe -> {unlocked['effect']} ({unlocked['authority']})", unlocked["authority"] == "lexical_only_no_probe")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
