# Governance Milestone: S25-S32 Closure Chain (FROZEN for v0.1)

> Status: **FROZEN** as of v0.1.0 (cognitive-os-v0.1.0). This document freezes the
> Sprint 25 -> Sprint 32 governance lineage. It is the single milestone-freeze record;
> it reconciles with existing release docs rather than duplicating them (see Reconciliation
> at the end). The per-sprint engineering narrative and the honest per-sprint residuals
> already live in `DESIGN_REVIEW_NOTES.md`; the attack-closure record lives in
> `FAILURE_LEDGER.md` (FAIL-0009..FAIL-0016); the operator changelog lives in `CHANGELOG.md`.
> This file does not restate them — it freezes the chain, the invariants, the probes, the
> gates, the verification discipline, the honest residuals, and the frozen-status declaration.

## 1. What is frozen

The development process itself is governed by the same machinery as the runtime loop. Eight
links form a chain in which no single layer can be weakened in isolation to push a proposal
past the release gate. Each link closes the previous link's residual and a numbered
`FAILURE_LEDGER.md` entry.

| Sprint | Invariant (what cannot be done) | Mechanism file | FAIL closed |
| --- | --- | --- | --- |
| S25 | Declared effect cannot self-authorize; effect is evidence-derived from a semantic diff | `effect_classifier.py` | FAIL-0009 |
| S26 | Words are claims, traces are evidence; preservation must be TESTED, not trusted | `trace_diff.py` | FAIL-0010 |
| S27 | A locked invariant without a behavioral probe is ineligible for preserve/extend accept | `trace_diff.py` (all 5 probes) | FAIL-0011 |
| S28 | Tested delta is DERIVED from a provenance-verified `change_set`, not self-declared | `change_provenance.py` | FAIL-0012 |
| S29 | `change_set` binds to literal artifact CONTENT via pre/post-image SHA-256 | `change_provenance.py` | FAIL-0013 |
| S30 | A content-bound change to a locked invariant requires a valid Ed25519 signature | `design_signing.py` | FAIL-0014 |
| S31 | Signer authority is governed (scope + lifecycle), evaluated at the decision tick | `design_signing.py` + `authorized_design_signers.json` (v0.2) | FAIL-0015 |
| S32 | The enforcement-code SOURCE is content-bound and safely probe-tested (no execution) | `mechanism_provenance.py` + `mechanism_source_manifest.json` | FAIL-0016 |

The chain's load-bearing property: a weakening must defeat ALL of derived-effect classification,
the behavioral trace probe, content-hash binding, a valid governed signature, AND mechanism-source
binding simultaneously. The signature is necessary but never sufficient: a validly-signed
weakening still blocks by trace (`effect_authority: trace_behavior_regression`).

## 2. Locked invariants and their probes (S27 completeness)

All five locked design invariants have a real runtime probe bound in `trace_diff.py`; each probe
runs actual enforcement machinery, not a mock:

| Locked invariant | Probe | Runtime code exercised |
| --- | --- | --- |
| `hazard_gate` | `_hazard_gate_outcome` | `verifier_engine.adjudicate` |
| `consolidation_gate` | `_consolidation_gate_outcome` | `mutation_gateway.apply_memory_mutation` |
| `naked_fact_gate` | `_no_naked_facts_outcome` | `retrieval_policy.emergency_use_protocol` |
| `raw_append_only_gate` | `_raw_append_only_outcome` | `raw_episode_store.RawEpisodeStore` (immutable) |
| `llm_authority_gate` | `_llm_authority_outcome` | semantic candidate + `apply_memory_mutation` |

Structural rule (`combine_effects`): a LOCKED invariant whose probe status is `not_probed` is
ineligible for preserve/extend acceptance and defaults to `needs_review` (which blocks). No probe
means no proof of preservation — a future locked invariant added without a probe blocks rather
than laundering through.

## 3. Mechanism-source binding (S32)

`mechanism_source_manifest.json` (schema `mechanism-source-manifest-v0.1`) binds **10**
enforcement-code files by SHA-256 content hash, keyed by role:
`verifier_engine.py`, `mutation_gateway.py`, `retrieval_policy.py`, `raw_episode_store.py`,
`trace_diff.py`, `change_provenance.py`, `design_signing.py`, `effect_classifier.py`,
`project_self_audit.py`, and `mechanism_provenance.py` (itself).

Three nets:
1. **Integrity** — `mechanism_provenance.py --verify` recomputes each hash from disk and fails on
   any divergence; `decision_audit.py --project` reports `mechanism_source_binding: verified` and
   the strict audit gates on it. Two independent nets catch an un-regenerated manifest.
2. **Provenance** — a `change_set` with `binding == "mechanism_source"` binds the literal
   before/after gate-code content; the pre-image must equal the CURRENT on-disk source.
3. **Behavioral (no execution)** — `probe_outcome_for_proposed_source` NEVER executes the proposed
   post-image. It parses the post-image to an AST, extracts the bound function (`adjudicate`), and
   safely interprets it over fixed probe inputs across a restricted subset (if / boolean /
   comparison / return over parameters and literals). Any construct outside the subset — a call,
   attribute access, import, loop, or module-level statement — is never evaluated and fails closed
   to `mechanism_probe_error` (a regression, which blocks).

**Correctness note (binding for v0.1):** this is a no-execution AST interpreter, NOT a subprocess.
The original S32 design ran the post-image in a subprocess with the gate process's filesystem
access; an adversarial verifier reproduced a real HIGH (a probe-passing post-image could write
files via module-level code; the `().__class__.__subclasses__()` gadget escapes any in-process
restricted-builtins sandbox). The probe was reworked to the no-execution interpreter, and selftest
cases lock the exact attack (a module-level backdoor write creates no file). `DESIGN_REVIEW_NOTES.md`
(S32 entry) records this correctly; `FAILURE_LEDGER.md` FAIL-0016 and `SPRINT_32_PLAN.md` rubric
item 7 carried stale 'subprocess' wording from the original design and were corrected to the shipped
no-execution interpreter during this freeze.

## 4. Signer governance (S31)

`authorized_design_signers.json` (schema `authorized-design-signers-v0.2`) is a governed registry:
each signer is `{public_key, scope, status, valid_from_tick, expires_at_tick, revoked_at_tick,
rotated_to}`. `design_signing.signer_authority` proves cryptographic authorship FIRST, then
evaluates whether the genuine signer is currently authorized for this change at the proposal's
logical `evaluation_tick`: `signer_revoked` / `signer_expired` / `signer_not_yet_valid` /
`signer_wrong_scope`. Authority is evaluated at the **decision tick**, not the signing tick — the
same genuine signature is authorized at tick 5 and rejected at tick 20 for a signer revoked at
tick 10, so a revoked key cannot replay a prior signature. An empty scope `[]` fails closed; only
`["*"]` is a wildcard. Lifecycle is logical-tick based (no wall-clock); a release gate asserts
zero `datetime`/`time.time`/`time.monotonic` symbols in `design_signing.py`. Only PUBLIC keys are
committed; a release gate asserts no private key material exists in `simulations/` or `scripts/`.

## 5. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0 bytes
stdout, 0 bytes stderr). Re-verified during this freeze: exit 0, 0 bytes stdout, 0 bytes stderr.

The gate runs, for S25-S32: `--selftest` of `change_provenance`/`design_signing`/
`mechanism_provenance`, the S25-S32 adversarial and accept scenarios (39 `design_audit` scenario
runs in the gate) with their outcome-field assertions
(`trace_regressed`, `effect_authority`, `signature_status`, `governance_decision`, `mechanism_source`),
manifest `--verify`, and `decision_audit.py --project` strict gating on `mechanism_source_binding`.
It also greps that `declared_effect` is never used as authority (only `derived_effect` drives the
governance decision). The acceptance discipline for the lineage was: rubric -> green byte-silent
release_check -> independent adversarial verifier with a fresh context -> any residual becomes the
next sprint. Multiple sprints (S25, S32) carried a VERIFIER-FOUND DEFECT that was fixed before close
and locked with a named regression scenario.

## 6. Independent verification result

The closure-chain verifier (fresh context, given only the rubric and the code) confirmed all 8
links enforced by actual code execution, with **zero overstatements** flagged. Every governance
claim in this document is checkable by running `scripts/release_check.sh` and the named scenarios.

## 7. Honest residuals (NOT closed in v0.1)

These are accepted limitations of the frozen milestone, published as caveats. They are not bugs;
they are the known edge of the behavioral model.

1. **Single-role behavioral probing.** Only the `adjudicator` role is behaviorally probed against a
   proposed mechanism-source change. The other 9 manifest-bound roles are integrity-bound (content
   hash) only; a proposed change to them fails closed to `needs_review` rather than being
   probe-tested.
2. **AST-subset precision cost.** The no-execution interpreter supports only if / boolean /
   comparison / return over parameters and literals. A behavior-PRESERVING adjudicator change that
   uses a helper call or a loop fails closed to a regression rather than accepting — a precision
   cost in the safe direction.
3. **Single-signer governance.** No threshold / multi-signer / m-of-n. A single active, in-scope,
   in-window key suffices; there is no recovery path if the `design_authority` key is compromised.
4. **Self-attestation fixed point.** `mechanism_provenance.py` binds itself by hash but cannot
   fully attest itself (the who-watches-the-watchmen limit of a single-repo self-check). It is
   bounded by the behavioral probes + the regression suite, not by a watcher above it.
5. **Runtime, not build-time, staleness detection.** Stale pre-image detection happens when a
   change_set is evaluated, not at build time; correct operation requires operational discipline
   (atomic change application, lifecycle tick hygiene).
6. **v0.2 registry defaults are safe but require care.** `design_authority` is active + wildcard
   scope so all prior committed signatures stay valid; evolving the registry must preserve this
   intentionally.
7. **Prototype, not production.** This is a deterministic Python prototype. It is a governance
   proof-of-concept and testbed, not production-ready for cryptographically-critical systems until
   the above are resolved or explicitly accepted for a target use case. A Rust rewrite, threshold
   governance, and behavioral probing for more roles are deferred (see `ROADMAP.md`).

## 8. Frozen-status declaration

The S25-S32 governance lineage is **FROZEN at v0.1.0**. The eight invariants, their five probes,
the mechanism-source manifest (10 files), the v0.2 signer registry, and the release gate are the
release surface. Any change that weakens a locked invariant, removes a probe, drops a manifest
entry, or relaxes a gate must itself pass through this same machinery — derived effect, trace probe,
content-hash binding, governed signature, and mechanism-source binding — and must leave
`release_check.sh` exit 0 and byte-silent. Relaxing any rubric criterion requires explicit operator
sign-off; it must not be edited mid-stream to make a failing check pass.

## 9. Reconciliation with existing release docs

- **Keep authoritative, do not duplicate:** `CHANGELOG.md` (operator changelog, current through S32),
  `FAILURE_LEDGER.md` (FAIL-0009..FAIL-0016, all locked; FAIL-0016 wording corrected to the
  no-execution interpreter during this freeze),
  `DESIGN_REVIEW_NOTES.md` (per-sprint narrative + honest residuals, current through S32).
- **Reconcile forward (stale), cross-referencing this doc and CHANGELOG instead of restating:**
  `RELEASE_NOTES.md` (frozen at S21), `RELEASE_REVIEW.md` (frozen at S9), `QA_REPORT.md` (frozen at
  S23), `ROADMAP.md` (advertises S25 as future), `README.md` (omits the lineage and the interpreter
  requirement).
- **New artifacts this milestone adds:** this file, `requirements.txt`, and `ENVIRONMENT.md` (the
  runtime lock). The new artifacts are locked into `release_check.sh` (see recommended gates).
