# QA Plan

## Unit Tests

Covered in `tests/unit/test_core.py`:

- packet envelope validation
- permission enforcement
- epistemic license transitions
- priority scoring
- memory status retrieval
- conflict detection

## Integration Tests

Covered in `tests/integration/test_scenarios.py` and `tests/simulation/test_bridge_world.py`:

- normal safe route
- stale memory retrieved
- contradictory evidence
- rule revision candidate
- degraded emergency action
- Bridge A safety query under high time pressure
- false alarm damage report repair
- Bridge B also degraded minimax no-safe-route handling
- urgency spam attack resistance
- interrupt storm
- adversarial user prompt

## Adversarial Tests

Covered in `tests/adversarial/test_attacks.py`:

- user tries to force direct action
- user asserts false memory
- packet missing provenance
- packet has forbidden action
- contradiction hidden in payload
- LLM codec emits malformed packet
- high-priority spam flood
- natural-language internal routing attempt

## Regression Gates

Covered in `tests/regression/test_release_gates.py` and called by `scripts/release_check.sh`:

- all packet schemas parse
- all actions are traced
- degraded actions schedule revalidation
- no forbidden-use packet reaches action engine
- interrupt storm does not crash the system
- memory mutation is logged
- trace audit reconstructs the Bridge B recommendation
