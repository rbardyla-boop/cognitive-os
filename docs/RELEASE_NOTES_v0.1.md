# Cognitive OS Prototype — Release Notes v0.1

**Status: local prototype release-ready (RELEASE-1).** This is a **local** readiness declaration, not a
cloud or public deployment. Nothing here trains, mutates weights, deploys externally, starts public
production, serves traffic, or replaces a baseline.

## What v0.1 is

A verified reading / memory / provenance **substrate**, not an agent. The ordering is deliberate:
reading → memory → intelligence → agency. Each gate genuinely CONSUMES the prior layer's real output by
re-running its function, so no layer can fabricate the one beneath it.

## The verified chain (local, reproducible)

The local prototype is sealed over this committed lineage (each verified as an ancestor of the release
head by `scripts/release_check.sh`):

| Layer | Commit | What it added |
|---|---|---|
| SCORE-0 | `e30176e` | deterministic verifier score matrix over the reading path |
| FAIL-0 | `f6fd0d8` | clean-failure detection (model-need candidates) |
| P11-MODEL-EVAL | `187466c` | the honest fork — does residual failure need new weights? |
| TRAIN-GATE-0 | `2e438c4` | closed-by-default training-authorization gate |
| TRAIN-0 | `72adfe4` | gated, deterministic, local training-attempt harness (no real weight mutation) |
| MODEL-EVAL-1 | `9597c49` | candidate acceptance battery (no verdict named `accepted`) |
| MODEL-PROMOTE-0 | `e33701b` | model promotion eligibility gate |
| PROD-0 | `fc57104` | local production runtime **package** (not a running runtime) |
| PROD-SMOKE-0 | `b653dd3` | local end-to-end production **smoke** (executes + verifies the package) |
| RELEASE-1 | _this release_ | the final local release gate |

## The honest fork stayed open

No model was accepted, promoted, deployed, or served. No weights were mutated. The system may still
ship **without** new weights — the substrate / retrieval / horizon fixes resolved the residuals, and
every training door is structurally closed (`data-curator` `TRAINING_PERMITTED = false`; P12
`training_justified = false`; P13–P15 closed).

## What release-ready means here

`local_release_ready` means the local prototype is **reproducible** (deterministic, re-derived and
byte-compared, never trusted from serialized bytes), **smoke-passed** (PROD-SMOKE-0 executed and
verified the packaged runtime end-to-end), **rollback-backed** (a verified rollback receipt), and
**boundary-safe** (every forbidden-action flag sourced from a structural `false` const, pinned by
`scripts/release_check.sh`).

## What v0.1 is NOT

- Not an external deployment (no Clovelearn, no Cloudflare, no server, no public endpoint).
- Not public production, not a long-running service daemon, not traffic serving.
- Not a baseline replacement, not model training, not weight mutation.
- Not a source of truth, memory, evidence, or new authority created from release status.

## Verification

`scripts/release_check.sh` is the single green/byte-silent gate (exit 0, 0B stdout, 0B stderr).
`scripts/operator_smoke.sh` runs the documented operator path, the PROD-SMOKE-0 harness, and the
RELEASE-1 gate end-to-end. See `docs/RELEASE_RUNBOOK.md` for the operator checklist and the tag rule.
