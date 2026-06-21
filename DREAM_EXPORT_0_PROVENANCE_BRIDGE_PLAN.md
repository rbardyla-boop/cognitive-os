# DREAM-EXPORT-0 — Dream Export Receipt / Provenance Bridge (DONE rubric)

Capability sprint. **No tag.** Keep all edits OUTSIDE the frozen hypothesis-layer authority code.

## Goal

Allow a terminal `DreamPacket` to be exported into the **existing** `HypothesisOnly` proposal path
(`hypothesis_layer::propose`) while preserving dream-origin provenance OUTSIDE the frozen
`hypothesis-layer::Authority` model.

## Correct shape

```
DreamPacket  →  DreamExportReceipt  →  existing HypothesisOnly proposal path
```

## Forbidden shape

```
DreamPacket  →  new Authority::DreamOnly
```

## Implementation boundary

- do not edit `hypothesis-layer` `Authority`
- do not add `DreamOnly` outside `dream-engine`
- do not change frozen hypothesis-layer invariants
- do not let `DreamPacket` export without a `DreamExportReceipt`
- do not let exported dream material lose dream provenance
- do not execute probe requests
- do not create evidence / promote / train

## Architecture (decided against the real code + gates)

- Lives in **`crates/cognitive-demo`** (the integration crate that already consumes the frozen
  hypothesis track). It gains a dependency on `dream-engine` (consumes its PUBLIC terminal packet).
  The arrow is `cognitive-demo → dream-engine`, so dream-engine's own quarantine tree is unchanged.
- The bridge re-derives the canonical `DreamPacket` from primary inputs (`--input-dir` corpus +
  `--frame` + seed + weirdness) via `dream_engine::dream_packet`, builds a `HypothesisSpec`, and calls
  the EXISTING `hypothesis_layer::propose` — so the exported material is a real `HypothesisPacket`
  carrying the EXISTING `Authority::HypothesisOnly` (taken straight off the proposed packet).
- `DreamExportReceipt` and `DreamExportBundle` are `Serialize` but **NOT** `Deserialize` (gate
  `release_check.sh:1298` forbids `Deserialize` in cognitive-demo). The artifact is re-derived from
  primary inputs and byte-compared — never parsed back into authority — so `dream-export-report` /
  `dream-export-replay` require `--input-dir` + `--frame` (+ seed/weirdness), exactly like
  `novelty-report` / `novelty-replay`. The `dream_only` authority NEVER crosses; only ids/hashes/
  operator tokens cross, as provenance.

## Core object

```
DreamExportReceipt {
  schema, export_id,
  dream_packet_id, dream_input_hash, dream_seed, dream_weirdness, dream_engine_version,
  dream_operator_ids[], source_receipt_memory_hash, source_receipt_answer_hash,
  exported_hypothesis_hash,
  exported_via_existing_hypothesis_gate: true,
  authority_after_export: HypothesisOnly,   // the EXISTING enum, read off the proposed packet
  dream_origin: true,
  forbidden_uses[],                         // the proposed hypothesis's own forbidden-uses list
  export_trace_hash, boundary[]
}
```

## Correct if

1. export requires a valid re-derived `DreamPacket` (fails closed if the corpus does not verify / dream is degenerate)
2. a tampered `DreamPacket` (via `--dream-packet`) is refused
3. export creates a `DreamExportReceipt`
4. the receipt preserves `dream_packet_id` and `dream_input_hash`
5. the receipt records `dream_origin = true`
6. the receipt records `authority_after_export = HypothesisOnly`
7. the exported hypothesis uses the existing hypothesis-only path (`propose`, real forbidden-uses)
8. dream provenance is visible in report/replay output
9. plain hypothesis and dream-exported hypothesis remain distinguishable
10. probe requests remain `executes: false`
11. receipt replay is deterministic (byte-identical re-derivation)
12. a tampered export bundle is refused
13. hypothesis-layer `Authority` remains single-variant
14. `DreamOnly` appears only inside `dream-engine`
15. P12 remains `training_justified = false`; P13–P15 remain closed
16. `release_check` stays green + byte-silent

## Wrong if

- `Authority::DreamOnly` is added to hypothesis-layer
- `DreamPacket` can export without provenance
- a `DreamExportReceipt` can be forged without re-deriving the `DreamPacket`
- dream-origin material becomes indistinguishable from ordinary hypothesis material
- export grants evidence status / promotes / executes probes / opens training / edits frozen authority

## CLI

```
cognitive-demo dream-export        --input-dir DIR --frame PATH [--seed N] [--weirdness W] [--dream-packet PATH] [--out PATH]
cognitive-demo dream-export-report --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH [--out PATH]
cognitive-demo dream-export-replay --input-dir DIR --frame PATH [--seed N] [--weirdness W] --export PATH
```

(The suggested `--export`-only report/replay form is not possible without deserializing the artifact,
which the never-parse-authority-back invariant + gate 1298 forbid; primary inputs are required, as in
the novelty verbs.)

## Boundary to record

```
Dream export preserves provenance.
It does not create a new authority.
Exported dream material remains hypothesis_only.
Dream origin remains auditable.
Probe requests do not execute.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```
