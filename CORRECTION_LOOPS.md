# Correction Loops

Sprint 11 adds a narrow post-action correction loop.

The loop is intentionally limited:

1. degraded action outcome is recorded as an immutable episode
2. post-action revalidation evaluates the outcome
3. procedure correction is attempted first
4. belief correction is attempted second
5. both corrections pass through mutation authority
6. mutation audit replay reconstructs order and before/after status

This avoids treating one successful degraded action as proof that the underlying belief is globally safe.

Sprint 11 review checks:

- audit entries label procedure and belief mutations separately
- degraded success does not raise belief authority above `retest_required`
- partial success carries explicit scope conditions for both policy and belief
