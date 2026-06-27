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
