# Risk Register

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Hidden memory mutation | Invalid audit trail | Require `memory_mutation` packets for every update. |
| Packet schema drift | Runtime disagreement | Validate packets against schemas in CI. |
| Verifier overconfidence | Unsafe or misleading plans | Require explicit epistemic license with confidence and caveats. |
| Stale memory dominates current evidence | Bad decisions | Track memory status, provenance, and staleness in retrieval results. |
| Conflicting rules are silently ignored | Unreliable behavior | Emit contradiction packets and adjudication records. |
| Prototype scope creep | Unsafe expectations | Maintain `NON_GOALS.md` and release checklist gates. |
| Dashboard becomes decorative | Loss of observability | Treat UI as required audit surface, not a marketing layer. |

