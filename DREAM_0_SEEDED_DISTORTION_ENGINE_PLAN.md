# DREAM-0 — Seeded Deterministic Distortion Engine

**Status:** PLANNED (awaiting implementation). No code, no tag, no commit until the operator directs it.
**Track:** Dream. **Predecessor:** NOVELTY-0 (`crates/cognitive-demo`, DD-2026-06-21-D) — DREAM-0 is its successor; NOVELTY-0 is left in place and is NOT migrated in this sprint.
**Gate cleared:** NOVELTY-1 committed cleanly (`5fc29e1`), tree clean, green byte-silent `release_check.sh`.

---

## 1. Boundary doctrine (recorded verbatim, pinned in the crate as `DREAM_BOUNDARY_LINES`)

```
The dream engine distorts.
It does not certify.
Dream authority is private to dream-engine.
No dream output enters the hypothesis layer in DREAM-0.
Dream packets are terminal and inert.
Probe requests do not execute.
Nothing becomes evidence.
Nothing promotes.
Nothing trains.
```

## 2. Decision & approved tightenings

- New crate `crates/dream-engine` generates weird, assumption-breaking but **receipt-grounded** `DreamPacket`s via seeded deterministic distortion. **Alien internally; terminal and inert at the boundary.**
- **Tightening 1 — authority is private.** `DreamAuthority::DreamOnly` exists ONLY inside `crates/dream-engine`. It must not touch `hypothesis-layer::Authority`; `hypothesis-layer` stays byte-unchanged. The cargo-tree purity gate is mandatory.
- **Tightening 2 — no export, at all.** DREAM-0 does not export to the hypothesis layer, not even through a bridge. Terminal `DreamPacket` only. **No `hypothesis-layer` dependency.** The export/provenance bridge (`DreamExportReceipt`) becomes a later sprint (DREAM-1 / DREAM-EXPORT-0).

## 3. Scope

**In:** a standalone deterministic crate; ≥5 seeded distortion operators; weirdness dial `0..5`; anti-degeneracy gates; verbatim-grounded preserved facts; well-formed falsifier *slots*; non-executing probe requests; byte-identical replay by seed; fail-closed refusal; a `release_check.sh` DREAM-0 block + named regression scenarios.

**Out (explicit, so DREAM-0 does not overclaim):** no LLM; no `DreamExportReceipt` / `propose()` bridge; no validated falsifiers (slots only — real falsifier power is deferred to the later LLM stage because the reading track deliberately has no entailment oracle); no NOVELTY-0 migration; no training, execution, promotion, or evidence.

## 4. Crate layout & dependencies

```
crates/dream-engine/
  Cargo.toml          # edition 2021, publish = false, #![forbid(unsafe_code)]
  src/lib.rs          # all logic + #[cfg(test)] mod tests
```

**Production deps ⊆ `{serde, serde_json, reading-substrate}`.** Forbidden in the production tree (gate-asserted): `hypothesis-layer`, `vibe-*`, `cognitive-demo`, any codec/ML. Rationale: `cognitive-demo`'s `corpus_trace`/`corpus_source` are private AND `cognitive-demo` depends on `hypothesis-layer`, so reusing them would violate Tightening 2. Grounding is therefore rebuilt on `reading-substrate` only.

## 5. Implementation design

### 5.1 Grounding (lawful) — `reading-substrate` only
- `build_corpus(documents: &[(String,String)]) -> Corpus`: for each `(title, content)`, `reading_substrate::split_sentences(content)` → span texts → `Corpus::add_document(title, &spans)`. Pure, deterministic, no markdown-section dependence on `reading-cli`.
- `verified_spans(&Corpus) -> Vec<VerifiedSpan{ id: SpanId, text: String, document_id: u64 }>`: iterate `metadata()` → `span_ids` → `read_span` → `text()`. These are the ONLY facts a packet may preserve.
- **Verified-read receipt:** build a canonical `ReadingTrace` (inspect corpus, read every span, one `ExtractClaim` per span grounded in that span, `Synthesize` an answer citing all claims), `execute(&corpus, question, &trace)` → `ReadingRun`, `verify(&corpus, &run)` → `VerifyReport`; **require `passed == true`, else fail closed** (`DreamError::CorpusDoesNotVerify`). Bind the packet to `run.memory_hash` + `run.answer_hash`. The receipt's job is **corpus-binding + tamper-evidence**, not semantic truth — stated plainly.
- A preserved fact is valid iff it is **verbatim** a `verified_spans` text; anything else is refused (`DreamError::UnsupportedPreservedFact`).

### 5.2 Determinism (FNV-1a, no entropy)
- Copy the hypothesis-layer FNV-1a constants/helpers **verbatim**: `FNV_OFFSET = 0xcbf2_9ce4_8422_2325`, `FNV_PRIME = 0x0000_0100_0000_01b3`, byte loop = XOR-then-`wrapping_mul`, strings **length-prefixed**, numerics via `to_le_bytes`.
- **Seed expansion is counter-mode FNV:** `stream(seed, i) = fnv(fnv(seed), i)`. No PRNG, no `rand`/`getrandom`, no `SystemTime`/`Instant`, no `DefaultHasher`, no `f32`/`f64`. The weirdness dial is an `i64` folded into the seed.
- All packet content is built into `Vec`/`BTreeMap` in fixed order; never HashMap iteration. Replay = re-derive from the same `(documents, frame, seed, dial)` and `serde_json::to_string_pretty` byte-compare (`DreamError::DreamPacketMismatch`), plus a two-fresh-process byte-diff test.

### 5.3 Types
- `DreamAuthority` — single variant `DreamOnly`, `#[derive(..., Serialize)]` only, guarded by a wildcard-free exhaustive-match test (mirrors the house invariant; never references `hypothesis-layer::Authority`).
- `DreamPacket` (`#[derive(Clone, Debug, PartialEq, Eq, Serialize)]`, **no Deserialize**): `schema, packet_id, seed, weirdness, source_receipt_memory_hash, source_receipt_answer_hash, source_corpus_hash, frame_text, preserved_facts[], broken_assumptions[BrokenAssumption], impossible_links[ImpossibleLink], candidate_frames[String], falsifiers[FalsifierSlot], probe_requests[DreamProbeRequest], authority: DreamAuthority, forbidden_uses[6], boundary[9]`.
- `forbidden_uses` = the **canonical 6**: `ground_claim, serve_as_evidence, mutate_reading_memory, alter_verifier_receipt, change_training_gate, bypass_codec_or_governance` (not NOVELTY-0's weaker 4).
- `BrokenAssumption{ operator_id, text, derived_from_span_ids[] }` · `ImpossibleLink{ operator_id, text, span_ids[≥2 distinct document_id] }` · `FalsifierSlot{ preserved_fact_span_id: SpanId, preserved_fact_memory_hash: u64, broken_assumption_index: usize }` (reference-only, no generated prose) · `DreamProbeRequest{ schema, request_id, question, status: "requires_operator_review", executes: false }`.

### 5.4 Distortion operators (≥5, seeded, deterministic)
Each operator consumes verified spans + the untrusted frame + `(seed, dial)` and emits an artifact that records its `operator_id`, references the SpanIds it used, and is **byte-distinct from every preserved span**. Starter set:
1. `RoleInversion` — reorder a span's clauses around delimiters (output ≠ source span).
2. `CategoryViolation` — bind span A (doc i) and span B (doc j≠i) into one frame ("treat A's subject as B's process") → cross-document `ImpossibleLink`.
3. `ConstraintRemoval` — negate/transform a frame assumption line into a `BrokenAssumption` byte-distinct from the frame line.
4. `ContradictionBraid` — join two different-document spans sharing a token into one `ImpossibleLink`.
5. `ScaleShift` — reframe a local span fact as a system law (template transform; output ≠ span).

Weirdness `0..5` selects how many operators/links fire; higher dial yields **≥** as many links/assumptions as the level below (monotone, byte-checkable).

### 5.5 Anti-degeneracy gates (runtime refusals AND tests)
- **G1 operator-applied:** every produced link/assumption text is byte-distinct from every `preserved_fact`. (Kills the NOVELTY-0 reformat-the-source shape.)
- **G2 cross-source combination:** ≥2 preserved facts whose spans have **different `document_id`** feed the **same** `ImpossibleLink`/`candidate_frame` (combination, not co-citation; `document_id`, not section).
- **G3 assumption genuinely broken:** ≥1 `BrokenAssumption` is operator-output, byte-distinct from every frame line and every preserved span.
- A packet failing any gate is **refused** (`DreamError::DegenerateDream`), not emitted.

## 6. DONE rubric (numbered, checkable — the acceptance criteria)

A. **Identity/isolation** — `crates/dream-engine` exists; `#![forbid(unsafe_code)]`; prod deps ⊆ {serde, serde_json, reading-substrate}; no `hypothesis-layer`/`vibe-*`/`cognitive-demo`/ML; `hypothesis-layer::Authority` byte-unchanged; token `DreamOnly` only inside dream-engine.
B. **Grounding** — packet binds a verified read (`verify().passed`), fails closed otherwise; every preserved fact is verbatim a verified span; unsupported fact refused.
C. **Distortion** — ≥5 registered seeded operators; each artifact records `operator_id`; weirdness dial `0..5` as `i64`.
D. **Anti-degeneracy** — G1, G2, G3 enforced as refusals.
E. **Falsifier/probe** — falsifiers are reference-only slots (span id + memory hash + in-range assumption index, each pair unique); **no generator**; probes `executes:false`, operator-review-gated.
F. **Authority/terminal** — `DreamAuthority` single variant, exhaustive-match test; canonical 6 `forbidden_uses` + 9 boundary lines pinned and asserted; no function returns a hypothesis/export type; terminal.
G. **Determinism** — FNV-1a only (no `DefaultHasher`/clock/entropy/float, grep==0); fixed ordering; in-process re-derive byte-compare + two-process byte-diff; one literal `DreamPacket` id pinned in the gate.
H. **Refusal** — tampered packet, empty corpus, empty frame, zero-operator output all fail closed.

## 7. Named regression scenarios (unit tests)

`dream_refuses_degenerate_single_span_reformat` (NOVELTY-0 shape → REFUSED) · `dream_links_two_distinct_document_ids_into_one_frame` · `dream_broken_assumption_is_operator_output_not_frame_echo` · `dream_falsifier_slot_well_formed_by_reference` · `dream_preserved_fact_must_be_verified_span` · `dream_replay_byte_identical_two_processes` · `dream_packet_tamper_refused` · `dream_empty_frame_fails_closed` · `dream_corpus_must_verify_or_fail_closed` · `dream_authority_has_exactly_one_variant` · `dream_packet_is_terminal_no_export` · `dream_forbidden_uses_are_canonical_six` · `dream_boundary_lines_present` · `dream_weirdness_dial_is_monotone`.

## 8. `release_check.sh` DREAM-0 gate block (mirrors the NOVELTY-0 + hypothesis-layer gates)

1. `cargo fmt --check` / `cargo clippy -D warnings` / `cargo test --offline` green for `crates/dream-engine`.
2. **Cargo-tree purity:** `cargo tree --edges normal` for dream-engine shows `grep -cE 'hypothesis-layer|vibe-|cognitive-demo'` **== 0**; `grep -ciE 'torch|tensorflow|candle|onnx|tract|burn|llama|inference'` of Cargo.toml **== 0**. (This is what makes "no export" a gate-enforced invariant.)
3. **Determinism scans:** `grep -rlE 'SystemTime|Instant|std::time|thread_rng|getrandom|rand::|use rand|std::net|tokio|\.await|reqwest|DefaultHasher' crates/dream-engine/src | wc -l` **== 0**; `grep -rE '\bf32\b|\bf64\b' crates/dream-engine/src | wc -l` **== 0**.
4. **Authority-unchanged:** assert `hypothesis-layer/src/lib.rs` Authority block byte-identical; no `DreamOnly` token outside dream-engine.
5. **Pinned literal id:** one fixed `(documents, frame, seed, dial)` fixture → a pinned `DreamPacket` packet_id literal (FNV-stable).
6. **Unit-count pin** for dream-engine + the named-test pins; total workspace unit count bumped accordingly.
7. **Binary smoke** (if a CLI surface ships): prove `DreamOnly`, no score, every probe `executes:false`, the 6 forbidden uses, the 9 boundary lines, and the degenerate-packet refusal from the packet's own bytes.

## 9. Acceptance (operator cadence)

Green byte-silent `release_check.sh` → independent fresh-context adversarial verifier graded per criterion (A–H), who must personally attempt the degenerate-packet attack (§7.1) and confirm no path reaches `hypothesis-layer` → any residual becomes the next sprint, which is the export work (`DreamExportReceipt` / DREAM-EXPORT-0). On close, record `DD-2026-06-21-F` in `docs/PROJECT_CHARTER.md` and (operator's call) freeze/tag.
