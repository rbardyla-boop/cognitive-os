# Production Runtime Runbook (PROD-0)

This runbook governs the **local** production-runtime package produced by PROD-0
(`crates/cognitive-demo/src/production_runtime.rs`). It is the operator's checklist for preparing a
runtime artifact. It is **not** a deployment guide: PROD-0 packages a runtime; it does not run one.

## Boundary (verbatim)

```text
The production runtime package prepares a local runtime artifact.
It does not train.
It does not mutate weights.
It does not deploy models.
It does not start production service.
It does not replace the baseline.
It does not create truth, memory, or evidence.
It does not grant new authority.
ProductionRuntimePackage is not production smoke.
```

## What PROD-0 does

`package_production_runtime(&ProductionRuntimeInput)` consumes the real MODEL-PROMOTE-0 evaluation
(re-running `evaluate_model_promotion` for a model-bearing mode) and emits a
`ProductionRuntimePackage`: either **packaged** (a `ProductionRuntimeManifest` + a sealed
`ProductionRuntimeReceipt`) or **refused** (with the specific `ProductionRuntimeRefusal` reasons). The
manifest describes the full verified path the runtime would run —
`curate → read → corpus → score → fail_detect → model_eval → training_gate → training_attempt →
candidate_eval → promotion_gate → runtime_receipt` — without executing it.

## Runtime modes

- `local_no_model_runtime` — the substrate runtime, no model slot. Requires only the common receipts.
- `local_candidate_ready_runtime` — a model slot holding an evaluated candidate. Requires a consumed
  promotion report and corroborated model/baseline hashes, but **not** `PromotionReady`.
- `local_promoted_ready_runtime` — a model slot holding a promotion-ready model. Requires the consumed
  MODEL-PROMOTE-0 decision to be exactly `PromotionReady`. "Promoted-ready" means **packaged for a
  later smoke/deploy decision** — it is not deployed and serves no traffic.

## Operator preconditions (all required for a packaged runtime)

1. A deterministic, hash-pinned `ProductionRuntimeConfig` with training mode **disabled** and network
   **disabled** (offline). An enabled training mode or network is refused.
2. A `RuntimeVersionReceipt` (pinned runtime version + hash).
3. A `RuntimeRollbackReceipt` (pinned, verified rollback artifact).
4. This runbook in hand (`OperatorRunbookReceipt`).
5. A receipt output path and a replay output path.
6. For a model mode: a consumed MODEL-PROMOTE-0 report and a `RuntimeModelSlot` whose
   `model_artifact_hash` and `baseline_hash` are pinned and **corroborated** against that report.
7. An affirmative, clean authority-drift check.

## No-training guarantee

The runtime is no-training by construction: `RuntimeNoTrainingMode::NoTraining` is the only
representable training state, and a config that requests training mode is refused
(`training_mode_enabled`). P12 stays `training_justified = false`; P13–P15 remain closed.

## After packaging

A packaged runtime **requires S11 production smoke** before any production claim. PROD-0 does not run
the smoke. Do not treat a `ProductionRuntimeReceipt` as deployment, a running service, served traffic,
or a baseline replacement. There is **no external deployment** in this sprint — no Clovelearn, no
Cloudflare, no server, no public endpoint.

## Verifying a package

A `ProductionRuntimePackage` is `Serialize` but never `Deserialize`. To verify a serialized package,
re-derive it from the same input and byte-compare with `verify_production_runtime_package_json`; a
tampered or foreign package is refused. The 20-scenario `production_runtime_matrix()` records the
observed outcome of the real packager across the no-model / each-refusal / promoted-ready / not-X /
tamper cases, and `production_never_opens` holds across all cells.

## End-to-end production smoke (PROD-SMOKE-0 / S11)

PROD-SMOKE-0 (`crates/cognitive-demo/src/production_smoke.rs`) is the deterministic, **local**
end-to-end smoke for the packaged runtime. It answers exactly one question: *can the runtime PACKAGED
above actually EXECUTE and VERIFY its end-to-end path in a fresh local context?* — never "is production
running?". A local smoke **PASS is NOT external production and NOT final release**; S12 (RELEASE-1) is
the release gate.

`run_production_smoke(&ProductionSmokeRun)` CONSUMES the PROD-0 package — it re-runs
`package_production_runtime` itself over the supplied runtime input (the substrate / no-model runtime;
the model-bearing package is PROD-0's own concern) and verifies it by re-derivation + byte-compare. It
then EXECUTES the real end-to-end sub-flows — a curated read, a corpus flow, a horizon flow, a refusal
case (the runtime packager genuinely refusing a training-mode config), and a replay verification — and
writes + hash-verifies receipt and replay artifacts into a `ProductionSmokeArtifactManifest`.

### The sixteen required steps

`fresh_runtime_context`, `release_check_green`, `operator_smoke_green`, `runtime_package_verified`,
`curated_read_executed`, `corpus_flow_executed`, `horizon_flow_executed`, `refusal_case_executed`,
`replay_verification_executed`, `receipt_artifacts_written`, `replay_artifacts_written`,
`rollback_check_executed`, `model_version_hash_confirmed`, `no_training_mode_confirmed`,
`no_unauthorized_network_confirmed`, `documented_operator_workflow_confirmed`. The smoke refuses
nineteen ways (a missing/tampered package, a missing fresh context, a non-green `release_check` or
`operator_smoke` receipt, an omitted sub-flow, missing receipt/replay artifacts, a failed rollback
check, a missing version hash, a detected training mode / unauthorized network / baseline replacement /
production claim, or a tampered serialized report).

### Running the smoke

Run `scripts/operator_smoke.sh` (offline, deterministic, temp-dir only). It runs the whole documented
operator path AND the PROD-SMOKE-0 harness end-to-end; a green run records the
`operator-smoke: PROD-SMOKE-0 OK …` receipt. The harness records — it does NOT shell out from the pure
library; it consumes the green `release_check` and `operator_smoke` receipts as hash-pinned inputs.

### The smoke boundary (verbatim)

```text
The production smoke path verifies a local runtime package execution.
It does not train.
It does not mutate weights.
It does not deploy externally.
It does not serve production traffic.
It does not replace the baseline.
It does not create truth, memory, or evidence.
It does not grant new authority.
ProductionSmokePass is not final release.
```

A smoke PASS seals a `ProductionSmokeReceipt` that `requires_release_1` and is **never** final release.
Every forbidden-action flag is sourced from `SMOKE_IS_PRODUCTION = false`; P12 stays
`training_justified = false`; P13–P15 remain closed. There is **no external deployment** — no
Clovelearn, no Cloudflare, no server, no public endpoint, no long-running service. The report is
`Serialize` but never `Deserialize`: a serialized smoke report is re-derived and byte-compared, so
tampering is refused. The 21-scenario `production_smoke_matrix()` keeps `production_never_opens` and
`final_release_never_claimed` across all cells.

## Final local release (RELEASE-1 / S12)

After a green production smoke, the FINAL local release gate
(`crates/cognitive-demo/src/release_gate.rs`, RELEASE-1) may declare the local prototype
**release-ready**. It CONSUMES this PROD-SMOKE-0 smoke (requiring `Passed`) and the PROD-0 package
(requiring `Packaged`), verifies the committed chain head + lineage, and requires every release
receipt. `local_release_ready` is **local readiness only** — never external/cloud deployment, public
production, traffic serving, baseline replacement, or training. See `docs/RELEASE_RUNBOOK.md` for the
operator checklist, the required lineage, and the tag rule, and `docs/RELEASE_NOTES_v0.1.md` for the
v0.1 release notes.
