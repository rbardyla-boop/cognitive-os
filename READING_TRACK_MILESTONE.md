# Reading Track Milestone: READ-0 → READ-15 (FROZEN for reading-track-v0.1)

> Status: **FROZEN** as of `reading-track-v0.1`. This document freezes the READ-0
> through READ-15 reading-track arc as a named, auditable milestone before any
> further capability work. It is the single milestone-freeze record for the reading
> track; the per-sprint engineering narrative lives in `a.md` (the reading-track
> checklist and detail sections). This file freezes the arc, the commit lineage, the
> boundaries, the verification discipline, the training-gate verdict, the honest
> residuals, and the frozen-status declaration. It does not restate the per-sprint
> detail — it pins it.

## 0. Snapshot (recoverable freeze point)

```text
tag            reading-track-v0.1
points at      the READ-16 freeze commit (this document + its gate lock)
release_check  green + silent (exit 0, 0 bytes stdout, 0 bytes stderr; PATH=/usr/bin)
recover        git checkout reading-track-v0.1
training gate  training_not_justified (P12 training_justified = false) — weights forbidden
```

The reading track is a fully deterministic system: no model is in the loop. The P10
local-LLM adapter exists but is held shut by the P12 training-justification gate; all
READ-N reading, ranking, and autonomy use deterministic lexical machinery and no
weights.

## 1. What is frozen — the commit lineage

Eighteen commits form the arc. READ-1 and READ-2 are grounding **contracts** realized
inside the READ-0 substrate and the P9 codec (a claim is grounded only as a complete
sentence-level unit of a cited span's text); they have no separate feature commit.
P9–P12 are the codec / adapter / eval / training-gate layer beneath the READ-3+ CLI
and autonomy.

| Step | What it added (invariant) | Commit |
| --- | --- | --- |
| READ-0 | reading substrate: corpus / memory / trace / verify; READ-1 (claim ∈ cited span text) + READ-2 (sentence-aligned) grounding contracts | `f5b3fa9` (+ `ba7c4a3` docs) |
| P9 | codec boundary: an untrusted plan reaches memory only via `reading_codec::decode`, with a claim-fidelity verifier | `e4ccb6e` |
| P10 | baseline local-LLM adapter (held shut by P12; never trains) | `d197291` |
| P11 | codec eval harness with sentence-fidelity grounding | `4b4aef5` |
| P12 | training-justification gate (`training_justified`; weights forbidden until a clean recurring model failure) | `3902418` |
| READ-3 | real-corpus `read0` CLI (run / verify / replay; codec-only path) | `bffce24` |
| READ-4 | real-corpus eval pack (≥10 fixtures, committed labels, 0 false-grounded) | `9d1dc68` |
| READ-5 | deterministic sentence-splitter hardening | `1585c76` |
| READ-6 | reader autonomy v0 (`read`) | `a6b0ff5` |
| READ-7 | autonomous corpus eval pack | `d691e7c` |
| READ-8 | budgeted autonomous span selection (`read_budgeted`) | `29ef60e` |
| READ-9 | title-aware deterministic relevance ranking (`read_ranked`) | `28e0959` |
| READ-10 | section-aware / multi-term deterministic ranking (`read_section_ranked`) | `429d88d` |
| READ-11 | real document section-metadata ingestion (ATX headings → metadata, never spans) | `0b89324` |
| READ-12 | persist section metadata in run receipts (schema `read0-run-v2`) | `8bca35a` |
| READ-13 | receipt schema compatibility / migration gate (v1/v2 version discipline) | `a1f8bf2` |
| READ-14 | receipt integrity hashing for structural metadata (schema `read0-run-v3`, `structure_hash`) | `175f783` |
| READ-15 | receipt downgrade policy / integrity-level classification (`structure_bound` vs `legacy_unbound_structure`) | `11e9c5f` |

## 2. The boundaries that hold across the arc

These are the load-bearing invariants the whole track preserves. None was weakened by
a later sprint; each later sprint is additive over them.

1. **Grounding is evidence, not words (READ-1/READ-2).** `verify` grounds a claim only
   against `corpus.read_span(id).text()` — never metadata, headings, titles, ranking
   scores, or structure hashes. A claim must be a complete sentence-level unit of a
   cited span's text. Answer authority comes from verified, source-linked evidence,
   not from model confidence.
2. **Untrusted plans are quarantined (P9).** A reading plan reaches memory only through
   `reading_codec::decode`; `read0` calls no substrate executor directly. A fabricated
   or fragment claim is rejected at the codec, never finalized.
3. **Autonomy orders reads, never grounds them (READ-6 → READ-10).** Span selection
   uses deterministic lexical relevance (lowercase word-prefix overlap) plus
   title/section ranking. Ranking changes read ORDER only; the claim filter is
   unchanged, so a title/heading match alone can never fabricate support. A budget
   miss is classified as a coverage miss — an engineering signal, never a training
   justification. No semantics, entailment, paraphrase, or model.
4. **Document structure is metadata, never evidence (READ-11).** ATX headings are
   parsed into section metadata with no `SpanId`, so a heading can never be cited or
   grounded. A headingless document is byte-identical to the flat build.
5. **The receipt boundary is versioned, integrity-bound, and honestly classified
   (READ-12 → READ-15).** The flat `spans` list stays the canonical span-id source, so
   evidence authority (the re-derived `memory_hash` / `answer_hash` and grounding) is
   unchanged at every receipt version. On top of that: sections are persisted
   (READ-12); the schema tag is explicit and must agree with its content, with v1/v2
   migrating safely and unknown tags rejected (READ-13); a `read0-run-v3` receipt
   binds the non-evidentiary structural metadata with a structure hash so a
   heading/title/uncited-span edit is caught (READ-14); and `verify` classifies the
   structural-integrity LEVEL (`structure_bound` for v3, `legacy_unbound_structure`
   for v1/v2 and downgrades) so a v3→v2 downgrade still verifies but is never reported
   as current integrity (READ-15).

## 3. The training-gate verdict (P12)

**`training_not_justified`** (the P12 `TrainingDecision.training_justified` bit is
`false`). Weight training stays forbidden until the P11 eval proves a stable, recurring
model failure that survives fixes to task spec, schema, prompt, examples, tooling,
context, and verifier design. On the current battery there are **0 false-accepts and 0
false-rejects** — there is no clean recurring model failure to justify weights, so the
training track is correctly stalled at P12 by design. Doctrine: no failed cases → no
training; any false-accept → a verifier/safety fix, never training; any
fixture/schema/prompt/tooling/context/verifier defect → no training. A model may only
ever PROPOSE; the codec and the READ-1/READ-2 verifier decide. P13–P15 (LoRA candidate,
shadow mode, promotion gate) stay closed.

## 4. The release gate (verification discipline)

The gate is `scripts/release_check.sh`. DONE means it **exits 0 AND is byte-silent** (0
bytes stdout, 0 bytes stderr). The reading-track blocks gate, per crate, `cargo test`
+ `cargo fmt --check` + `cargo clippy -D warnings`, plus per-READ signal greps,
crate-separation checks (`cargo tree` proves no reading crate depends on a `vibe-*`
engine crate, and no ML dependency), and end-to-end `read0` binary smokes that drive
real run → verify → replay and assert each tamper/downgrade case. The acceptance
discipline for every sprint in this arc was: rubric → green byte-silent release_check →
a live sabotage proving the gate catches a regression (restored byte-identical by
md5) → an independent read-only adversarial verifier panel with a fresh context → any
residual folded before close. READ-12 and READ-14 each carried a verifier-found defect
(a `usize::MAX` partition overflow panic; a structural-hash gap) that was reproduced
first-hand and folded before close.

## 5. Independent verification

Every sprint READ-9 through READ-15 was closed against a read-only adversarial panel
(Explore agents, refute-by-default) covering evidence-authority, boundary-masking,
forgery/downgrade, determinism, robustness, and gate-vacuity lenses. The final four
sprints (READ-12 through READ-15) closed on **0-defect panels** (after the READ-12 and
READ-14 folds). Every claim in this document is checkable by running
`scripts/release_check.sh` and reading the named commits.

## 6. Honest residuals (NOT closed in reading-track-v0.1)

Accepted limitations of the frozen milestone, published as caveats. They are the known
edge of the deterministic reading model, not bugs.

1. **Deterministic-lexical only.** Relevance and ranking use lowercase word-prefix
   overlap, not semantics. A question phrased without lexical overlap to its supporting
   span is a coverage miss, not an answer.
2. **Literal sentence-level grounding (READ-2).** Grounding is a complete sentence-level
   unit of cited span text; a paraphrase that means the same thing is not grounded. No
   semantic entailment.
3. **Legacy receipt downgrade (READ-14/15).** A `read0-run-v3` → v2 downgrade reverts
   to legacy-unbound structural metadata. It is accepted as legacy and explicitly
   flagged `legacy_unbound_structure`, and it can never forge a grounded answer
   (evidence stays re-derivation-protected), but its non-evidentiary metadata is no
   longer tamper-bound. This is the migration-safety tradeoff, not a regression.
4. **No model in the loop.** The reading track is fully deterministic. The P10 adapter
   is gated shut by P12; the autonomy uses no weights. Any future model may only
   propose; it cannot ground, mutate, or self-authorize.
5. **Prototype, not production.** This is a deterministic Rust prototype and testbed,
   not a production reading system.

## 7. Frozen-status declaration

The READ-0 → READ-15 reading-track arc is **FROZEN at `reading-track-v0.1`**. The
grounding contracts, the codec quarantine, the deterministic autonomy and ranking, the
metadata-not-evidence rule, the versioned/integrity-bound/honestly-classified receipt
boundary, and the training-gate verdict are the frozen surface. Any change that weakens
a grounding or receipt boundary, lets metadata become evidence, reports a legacy
receipt as current, or reopens training must pass through the same machinery — a
rubric, a green byte-silent `release_check.sh`, a live sabotage, and an independent
adversarial panel — and must leave `training_justified = false` unless a clean recurring
model failure is proven. Relaxing any criterion requires explicit operator sign-off; it
must not be edited mid-stream to make a failing check pass. P13–P15 do not start under
this freeze.
