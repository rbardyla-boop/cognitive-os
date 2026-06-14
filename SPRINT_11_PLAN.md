# Sprint 11 Plan

## Goal

When the system acts under degraded evidence, the outcome must update the correct thing:

- action procedure first
- underlying belief second
- raw episode immutably
- no overconfirmation from one lucky success

## Doctrine

- Outcome is evidence, not proof.
- Survival is not validation.
- Correction is mutation, so correction requires authority.

## Proof Scenarios

- `degraded_action_success_does_not_overconfirm`
- `degraded_action_failure_quarantines_memory`
- `degraded_action_partial_success_scopes_memory`

## Acceptance

- Success under caution does not promote a belief to `full_premise`; it leaves the implicated belief at `retest_required`.
- Failure under degraded evidence quarantines the implicated procedure and memory.
- Partial success creates scoped authority, not global authority.
- Every correction passes through `mutation_gateway.py`.
- Every correction is replayable by `mutation_audit.py`.
