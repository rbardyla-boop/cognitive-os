# Sprint 10 Plan

## Goal

No cognitive state may change without authorized, traceable mutation authority.

## Scope

- Central mutation gateway.
- Small mutation type enum.
- Authority-compatible requested use checks.
- Source packet and verifier decision checks.
- Append-only audit record for accepted and rejected attempts.
- Focused scenarios for direct bypass, low-authority evidence, and valid human promotion.

## Doctrine

- Decision authority is not mutation authority.
- Retrieval is not mutation authority.
- Urgency is not mutation authority.
- Bootstrap ingestion is not mutation authority.
- A script is not mutation authority.
- Human promotion is mutation authority only when packetized, verified, and logged.

## Proof Scenarios

- `direct_mutation_without_verifier`
- `memory_mutation_with_low_authority_packet`
- `valid_human_promotion_allows_invariant`

## Pass Question

Can any code path alter cognitive state without mutation authority?

Current Sprint 10 answer: mutation authority scenarios pass through `apply_memory_mutation`, and accepted plus rejected attempts emit append-only audit entries.
