# Language Codec

## Stub Codec

v0.1 uses deterministic parsing, not an LLM.

Example:

```text
cross bridge A
```

becomes:

```json
{
  "goal": "cross",
  "target": "bridge_A"
}
```

The full intent payload also keeps the human `raw_text` for audit, but engines must use typed fields.

## LLM Adapter

The LLM adapter is intentionally disabled in v0.1. Later it may only perform:

- human language -> candidate packet
- packet state -> human explanation

The verifier must validate candidate packets before use.

## No Internal Prose Routing

Engine-to-engine handoffs may not send internal instructions as prose. QA rejects payload fields such as:

- `instruction`
- `instructions`
- `message_to_engine`
- `natural_language_instruction`
- `prompt`

Human-facing prose is allowed only at the language codec and renderer boundary.

