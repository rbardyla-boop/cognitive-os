# Sprint 31 — Signer-Set Governance

Status: **Complete (2026-06-14).** `release_check.sh` is green+silent (exit 0, 0 stdout, 0 stderr)
and an independent adversarial panel returned VERIFIED on all 9 rubric items (every item reproduced
from source + live execution, not attacker assertion). The panel's 3 governance sabotages
(ignore revocation / expiry / scope) each made `release_check.sh` fail and were restored
byte-identical; decision-time authority is proven by the SAME genuine signature verifying at tick 5
and `signer_revoked` at tick 20.

Verifier-found defect (fixed before Complete): the panel reproduced a real fail-open — an EXPLICIT
empty scope `[]` collapsed to wildcard via `... or [SIGNER_SCOPE_WILDCARD]` (because `[]` is falsy),
so a signer scoped to nothing would authorize everything. It was outside the strict rubric (item 3
covers a signer scoped to a *different* control point, which was correctly enforced) and unreachable
via committed data (all 8 registry signers carry non-empty scopes), but a deny-all scope silently
meaning allow-all is exactly the fail-open class this lineage exists to close, so it was fixed: an
`is None` guard now distinguishes an omitted scope (legacy unscoped → wildcard) from an explicit
empty `[]` (authorizes nothing). Locked by a `design_signing --selftest` case and a regression
assertion.

Accepted residual (fail-safe, not fail-open; not fixed): a v0.2 entry that omits `status`/`scope`
defaults to active/wildcard (committed data never relies on this), and a non-integer lifecycle tick
raises a `TypeError` that blocks rather than authorizes. Both fail closed. Threshold/multi-signer
governance and mechanism-source content binding remain deferred (Sprint 32+).

## Why

Sprint 30 made a content-bound change to a locked invariant require a valid Ed25519 signature from
a signer in `authorized_design_signers.json`. That registry is now itself an authority surface, and
its only governance is "the public key is in the file." A public key is not permanent authority. A
signer is an authority-bearing object with scope and lifecycle.

## Hard rule / doctrine (the invariant this sprint locks)

```text
A public key is not permanent authority.
A signer is an authority-bearing object: it has a scope and a lifecycle (active / expired / revoked / rotated).
Authority is evaluated at decision time, not at signing time.
A valid signature from a no-longer-authorized signer is not authorization.
(Unchanged from S30: authorization never overrides a trace failure.)
```

## Goal

Promote the signer registry from a flat `{signer_id: public_key_pem}` map to a governed object. Each
signer carries `signer_id`, `public_key`, `scope`, `status`, `expires_at`, `revoked_at`, and an
optional `rotated_to` successor. A change is authorized only if its signer's verified signature comes
from a signer that is, **at the evaluation tick**, active, unexpired, unrevoked, and scoped to the
change's target. A rotated successor key is accepted; the predecessor (revoked) is not — and a
signature made by a now-revoked key cannot be replayed to authorize a later change.

## Determinism note (binding)

Expiry/revocation MUST be expressed in **logical ticks** (as Sprint 21's `signed_at_tick`), never
wall-clock — `release_check.sh` must stay reproducible. The evaluator takes an explicit `now_tick`
(committed per scenario); no `datetime.now()` in the decision path.

## Build (intended)

```text
simulations/bridge_world/authorized_design_signers.json   governed registry v0.2: signers[signer_id] = {public_key, scope, status, valid_from_tick, expires_at_tick, revoked_at_tick, rotated_to}
scripts/design_signing.py        verify_change_signature(change_set, registry, now_tick, change_scope) gains signer governance: status/expiry/revocation/scope checks layered AFTER cryptographic verify. New reasons: signer_revoked, signer_expired, signer_wrong_scope, signer_unknown (+ existing). Helper signer_authority(registry, signer_id, now_tick) -> governed verdict.
project_self_audit.evaluate_design_proposal   passes now_tick + change_scope (derived from the targeted invariant/control point); surfaces signer_scope / signer_status / signer_expires_at / signer_revoked_at; the gate still only constrains a would-be ACCEPT and never overrides a trace block.
scripts/design_audit.py / bridge_world_demo.py   surface the governed signer fields.
re-sign committed scenarios under the governed registry (active design_authority with a scope + validity window); private key generated at authoring and DISCARDED (never committed).
scenarios: revoked_signer_rejected, expired_signer_rejected, wrong_scope_signer_rejected, rotated_successor_accepted, revoked_key_cannot_replay_prior_signature, signed_weakening_still_blocks_under_governance
docs + DD_sprint_31 + test.sh/release_check.sh gates + test -f SPRINT_31_PLAN.md.
```

## Rubric — DONE means ALL of these are checkable PASS

1. `revoked_signer_rejected`: a change whose signer is `revoked` (or `revoked_at_tick <= now_tick`)
   blocks (`signature_status: signer_revoked`), even though the Ed25519 signature is cryptographically valid.
2. `expired_signer_rejected`: a change whose signer is past `expires_at_tick` at the evaluation tick
   blocks (`signer_expired`).
3. `wrong_scope_signer_rejected`: a signer scoped to one control point/invariant set cannot authorize
   a change to a target outside its scope (`signer_wrong_scope`).
4. `rotated_successor_accepted`: a change signed by the successor key named in the predecessor's
   `rotated_to` (successor `active`, in-scope, unexpired) is accepted; and the predecessor key, now
   revoked, is rejected for new changes.
5. `revoked_key_cannot_replay_prior_signature`: a signature that verified before revocation does NOT
   authorize a change evaluated after the signer's `revoked_at_tick` — authority is evaluated at
   decision time, not signing time.
6. A valid, in-scope, active signer still **cannot override a trace failure** — a governed-but-valid
   signer on a weakening still blocks by trace (S30 doctrine preserved under governance).
7. Content binding + signature crypto remain required (governance layers on top, never replaces them);
   an unsigned or content-unverifiable change still blocks first.
8. The Sprint 26–30 scenarios keep their governance; the audit is signer-authority-visible
   (`decision_audit --project --strict` reports signer scope/status + content digest + trace verdict);
   DD_sprint_31 recorded.
9. `release_check.sh` exits 0 and is silent, with Sprint 31 gates in `test.sh` and `release_check.sh`;
   a gate-sabotage of signer governance (ignore revocation / ignore expiry / ignore scope) fails it;
   no private key is committed; expiry/revocation are logical-tick based (deterministic).

## Wrong if

- A revoked, expired, or wrong-scope signer authorizes a change.
- A signature from a now-revoked key replays to authorize a later change.
- A governed signer overrides a trace failure.
- Expiry/revocation depends on wall-clock (non-reproducible release_check), or a private key is committed,
  or a Sprint 26–30 gate regresses, or `release_check.sh` exits nonzero / prints output.

## Explicitly NOT in this sprint

- Threshold / multi-signer governance (deferred to a later sprint, only if still needed).
- Mechanism-source content binding (Sprint 32 — the larger Sprint-29/30 residual).

## Doctrine

```text
A public key is not permanent authority.
A signer is an authority-bearing object, evaluated at decision time.
Authority never overrides invariant failure.
```
