# Release Runbook (RELEASE-1) — Cognitive OS prototype v0.1

This runbook governs the **final local release gate** (`crates/cognitive-demo/src/release_gate.rs`,
RELEASE-1). It is the operator's checklist for declaring the local prototype **release-ready**. It is
**not** a deployment guide: RELEASE-1 declares local readiness; it does not deploy, serve traffic,
start production, train, or claim cloud/public release.

## Boundary (verbatim)

```text
The release gate declares local prototype release readiness only.
It does not train.
It does not mutate weights.
It does not deploy externally.
It does not start public production.
It does not serve production traffic.
It does not replace the baseline.
It does not create truth, memory, or evidence.
It does not grant new authority.
LocalReleaseReady is not cloud or public deployment.
```

## What RELEASE-1 does

`evaluate_release_gate(&ReleaseGateInput)` CONSUMES the real prior layers — it re-runs PROD-SMOKE-0's
`run_production_smoke` (requiring a `Passed` outcome) and PROD-0's `package_production_runtime`
(requiring a `Packaged` outcome), and corroborates the operator-supplied smoke/package hashes against
those re-derivations. It verifies the committed chain head and the full required lineage, requires
every release receipt, refuses any training / deployment / production-traffic / baseline-replacement
intent and any dirty scope, and emits a `ReleaseGateReport`: either **`local_release_ready`** (a sealed
`ReleaseGate` readiness receipt) or **`release_denied`** (with the specific `ReleaseRefusal` reasons).

## Required committed lineage (verified by `scripts/release_check.sh` with git)

The pure gate pins these hashes as constants; `release_check.sh` independently confirms each is an
ancestor of `HEAD` with `git merge-base --is-ancestor`:

| Layer | Commit |
|---|---|
| SCORE-0 | `e30176e` |
| FAIL-0 | `f6fd0d8` |
| P11-MODEL-EVAL | `187466c` |
| TRAIN-GATE-0 | `2e438c4` |
| TRAIN-0 | `72adfe4` |
| MODEL-EVAL-1 | `9597c49` |
| MODEL-PROMOTE-0 | `e33701b` |
| PROD-0 | `fc57104` |
| PROD-SMOKE-0 (chain head) | `b653dd3` |

A wrong chain head → `chain_head_mismatch`; a missing/wrong lineage commit → `missing_required_commit`.

## Operator preconditions (all required for `local_release_ready`)

1. A release request (`release_request_id`).
2. A PROD-SMOKE-0 receipt whose smoke **passed**, corroborated by its report hash.
3. The PROD-0 runtime package hash, corroborated against the re-derived package.
4. The committed-chain receipt (head + full lineage).
5. A release artifact manifest, release notes (`docs/RELEASE_NOTES_v0.1.md`), this release runbook, and
   the operator runbook (`docs/PRODUCTION_RUNTIME_RUNBOOK.md`).
6. A verified rollback receipt and a boundary-lock receipt.
7. A green `release_check` receipt and a green `operator_smoke` receipt.
8. The observed unit-test count equal to the pinned release count (439).
9. A clean release scope (no unrelated dirt staged) and a clean authority-drift check.

## Running the release gate

Run `scripts/operator_smoke.sh` (offline, deterministic, temp-dir only). It runs the whole documented
operator path, the PROD-SMOKE-0 harness, AND the RELEASE-1 gate end-to-end; a green run records the
`operator-smoke: RELEASE-1 OK …` receipt. The gate **records** the green `release_check`/`operator_smoke`
receipts as hash-pinned inputs; it does not shell out from the pure library.

## After `local_release_ready`

`local_release_ready` declares the **local prototype** is reproducible, smoke-passed, rollback-backed,
and boundary-safe. It is **not** external deployment, public production, traffic serving, baseline
replacement, or model training. There is **no external deployment** — no Clovelearn, no Cloudflare, no
server, no public endpoint, no long-running service. P12 stays `training_justified = false`; P13–P15
remain closed.

## Verifying a report

A `ReleaseGateReport` is `Serialize` but never `Deserialize`. To verify a serialized report, re-derive
it from the same input and byte-compare with `verify_release_gate_report_json`; a tampered or foreign
report is refused. The 29-scenario `release_matrix()` records the observed decision of the real gate
across the ready / each-denial / tamper / not-public cases, and `release_never_goes_public` plus
`public_release_never_claimed` hold across all cells.

## Tag

A tag (`cognitive-os-prototype-v0.1`) may be created **only after**: RELEASE-1 is committed;
`release_check` is `0 / 0B / 0B` post-commit; `operator_smoke` is green post-commit; an independent
verifier returns ALL PASS / 0 blocking defects; the final commit scope is clean; and the operator
issues a separate, explicit tag command.
