# Vault Clipping Orchestrator

The clipping orchestrator is the first safe slice of a higher-intelligence reading
loop for local notes:

```text
observe clipping -> preserve raw evidence -> extract candidate knowledge
-> infer intent -> queue review -> require human promotion
```

It is deliberately not model training and not autonomous vault editing. New
information published after the model cutoff is treated as local evidence with
provenance, not as model memory or action authority.

## Command

```sh
python3 scripts/clipping_orchestrator.py \
  --vault "/path/to/Obsidian Vault" \
  --seed-report "/path/to/Nightly Vault Review.md" \
  --model-cutoff 2024-06-01 \
  --out-json /tmp/vault_clipping_orchestrator.json \
  --out-md /tmp/vault_clipping_orchestrator.md
```

Without `--seed-report` or `--clipping`, it scans `Clippings/**/*.md`.

## What It Emits

- `raw_episodes`: immutable clipping receipts with path, title, source, date,
  word count, and content digest.
- `candidate_knowledge`: hypothesis-only semantic candidates that cite a raw
  episode and integrity digest.
- `review_queue`: human tasks such as reviewing post-cutoff evidence, extracting
  a safety SOP, or turning a receipt/memory idea into a project delta.
- `learning_boundary`: explicit proof that the run does not mutate notes, update
  model weights, or grant authority.

## Safety Invariants

- Raw clipping before semantic candidate.
- Every candidate cites a source digest.
- Post-cutoff information remains `hypothesis_only`.
- No direct action from clipping evidence.
- No model-weight update.
- No silent vault mutation.
- Human promotion required before durable memory or rule changes.

## Why This Matters

This is the path from passive clippings to useful intelligence:

1. The system reads actual current material from the vault.
2. It preserves what it saw as evidence.
3. It extracts intent hypotheses from the information itself.
4. It produces reviewable memory candidates instead of guessing user intent.
5. It creates a queue for promotion, rejection, or project deltas.

That gives future agents a better substrate: not "predict what the user meant,"
but "consult the user's evidence trail and its reviewed intent history."
