# Attention Review

Sprint 15 adds a narrow review loop for attention modes.

Attention review emits:

- `AttentionModeReviewPacket`
- `attention_mode_review` pending work
- `attention_policy_update` mutation through the mutation gateway

The mutation gateway only allows `attention_policy_update` on attention policy objects. It may update attention thresholds, mode policy, coalescing policy, backpressure policy, scoped confidence, and notes. It may not update memory authority, procedure authority, planner authority, verifier rules, or generic authority fields.

Replay:

```sh
python3 scripts/attention_review_audit.py --scenario reflex_mode_correctly_triggered
python3 scripts/attention_review_audit.py --scenario reflex_mode_false_alarm
python3 scripts/attention_review_audit.py --scenario interrupt_storm_recovery_replay
```
