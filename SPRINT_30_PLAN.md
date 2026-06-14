# Sprint 30 — Signed Change Provenance

Status: Complete.

## Why

Sprint 29 bound the tested delta to the literal artifact content. Its residual: content
binding proves *what* changed, not *who* accepted responsibility. A verified content-bound
change to a locked invariant must also carry accountable authorization over the artifact
content digest — and that authorization must never override a behavioral failure.

## Hard rule / doctrine (the invariant this sprint locks)

```text
Content binding proves what changed.
Trace binding proves what behavior changed.
Signature binding proves who accepted responsibility.
Authorization never overrides invariant failure.
```

## Goal

A would-be-ACCEPT of a content-bound change to a locked invariant requires a valid Ed25519
signature from an AUTHORIZED signer over the change's content digest (target, changed_artifact,
pre/post image hashes, diff_digest, control_point, nonce). Unsigned, wrong-signer, wrong-key,
or replayed signatures block. A validly-signed WEAKENING still blocks by trace — authorization
is necessary for accept but never sufficient to override a behavioral regression.

## Build

```text
scripts/design_signing.py                         change_signing_payload; sign_change_set; load_authorized_signers; verify_change_signature (reuses replay_asymmetric_key Ed25519). reasons: unsigned/unauthorized_signer/wrong_key/signature_payload_mismatch/signature_invalid/signature_verified
simulations/bridge_world/authorized_design_signers.json   committed registry: design_authority -> committed PUBLIC key (no private key in the repo)
project_self_audit.evaluate_design_proposal       signature gate: a would-be-accept on a LOCKED invariant requires verify_change_signature ok; surfaces signer / signature_status / signed_payload_digest; the gate NEVER relaxes a trace/lexical block
scripts/bridge_world_demo.py / scripts/design_audit.py    surface signer / signature_status
scenarios: unsigned_content_bound_change_blocks, wrong_signer_rejected, signature_replay_against_different_artifact_rejected, signed_preserving_change_accepts, signed_weakening_change_still_blocks
signed: the 5 existing accept-scenarios gain a committed design_authority signature (key generated at authoring, signature + public key committed, private key discarded)
```

## How signing is real (and secret-free)

`design_signing` reuses the Sprint-21 Ed25519 machinery: a private key signs, a public key
verifies, and public verification can never mint signing authority. The signed payload is the
canonical hash of `(scheme, signer, target, changed_artifact, pre_image_hash, post_image_hash,
diff_digest, control_point, nonce)` — so a signature is bound to that exact content and cannot
be replayed onto a different artifact/diff (the recomputed payload digest mismatches). The
committed registry holds only the authorized signer's PUBLIC key; the private key is generated
at authoring time, used to sign the committed scenarios, and discarded — never committed (a
committed signing secret is exactly what Sprint 19/20/21 forbid). Runtime round-trip tests use
ephemeral keys.

## Rubric — DONE means ALL of these are checkable PASS

1. `unsigned_content_bound_change_blocks`: a content-verified preserving change to a locked
   invariant with NO signature blocks (`signature_status: unsigned`,
   `effect_authority: change_signature_unverified`).
2. `wrong_signer_rejected`: a change signed by a signer not in the authorized registry blocks
   (`signature_status: unauthorized_signer`).
3. `signature_replay_against_different_artifact_rejected`: a valid signature copied onto a
   change_set with different content blocks (`signature_status: signature_payload_mismatch`) —
   a copied signature cannot authorize a different artifact/diff.
4. `signed_preserving_change_accepts`: a content-verified preserving change validly signed by
   an authorized signer is ACCEPTED (`signature_status: signature_verified`, consolidated),
   and the audit reports signer + content digest + trace verdict.
5. `signed_weakening_change_still_blocks`: a validly-signed WEAKENING still blocks by trace
   (`signature_status: signature_verified`, `trace_regressed: true`, derived weakening,
   `governance_decision: block`) — authorization never overrides invariant failure.
6. Content binding remains required (an unverifiable content change_set still blocks before the
   signature gate); a runtime round-trip proves sign→verify, and wrong-key / tampered-content /
   replay are rejected.
7. The Sprint 26/27/28/29 attack scenarios still BLOCK; the accept-scenarios still ACCEPT once
   they carry a committed authorized signature.
8. `decision_audit.py --project --strict` passes with zero violations and reports signer +
   content digest + trace verdict; DD_sprint_30 recorded.
9. `scripts/release_check.sh` exits 0 and is silent, with Sprint 30 gates in both
   `scripts/test.sh` and `scripts/release_check.sh`; a gate-sabotage of signature verification
   (accept an unsigned/unauthorized change, or let the signature gate override a trace block)
   makes `release_check.sh` fail.

## Wrong if

- Content is verified but authorship is unauthenticated (an unsigned/unauthorized change to a
  locked invariant reaches accept).
- A copied signature authorizes a different artifact, target, or diff.
- A signer can bypass a trace regression by being authorized (authorization overrides a block).
- A genuine signed preserving change is blocked, or a Sprint 26/27/28/29 gate regresses, or
  `release_check.sh` exits nonzero or prints output, or any private key is committed.

## Checks (commands)

```sh
python3 scripts/design_signing.py --selftest
python3 scripts/design_audit.py --scenario unsigned_content_bound_change_blocks
python3 scripts/design_audit.py --scenario wrong_signer_rejected
python3 scripts/design_audit.py --scenario signature_replay_against_different_artifact_rejected
python3 scripts/design_audit.py --scenario signed_preserving_change_accepts
python3 scripts/design_audit.py --scenario signed_weakening_change_still_blocks
python3 scripts/decision_audit.py --project --strict
./scripts/release_check.sh
```

## Residual / next boundary (explicitly deferred)

A content-bound change to a locked invariant now carries accountable Ed25519 authorization over
the content digest, and authorization is strictly necessary-not-sufficient (a signed weakening
still blocks by trace). Honest remaining limits (no safe-default claim): (1) the authorized
signer registry is a flat public-key list with no rotation, revocation, threshold/multi-signer,
or expiry — a single authorized key is a single point of authority; (2) the bound unit is still
the policy artifact, not the mechanism source (the Sprint-29 residual — an ADR-002 L0 runtime
concern). The next boundaries are signer-set governance (rotation/revocation/threshold) and
mechanism-source content binding.
