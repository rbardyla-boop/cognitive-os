//! vibe-scheduler — ADR-002 layer-L1 deterministic tick scheduling for Cognitive OS.
//!
//! Orders admitted, staged observations (from `vibe-ingress`) onto future logical
//! ticks. It depends only on `vibe-core` value types and the `vibe-ingress` staged
//! type. It never evaluates a tick and never touches the engine's state; frame
//! collection is P4 and evaluation is P5. See
//! `ADR-002-runtime-engine-replay-contract.md`.

#![forbid(unsafe_code)]

mod scheduler;

pub use scheduler::{
    ScheduleOutcome, ScheduleReason, ScheduleReceipt, ScheduleRequest, ScheduledObservation,
    SchedulerConfig, TickScheduler,
};

#[cfg(test)]
mod tests {
    use super::*;
    use vibe_core::{EngineState, Scalar, Tick};
    use vibe_ingress::{EventId, SourceSession, StagedObservation};

    // The scheduler source — scanned by the boundary test. The engine tokens
    // appear in THIS file only as test needles; `scheduler.rs` stays free of
    // them so the scheduler structurally cannot evaluate or mutate the engine.
    const SCHED_SRC: &str = include_str!("scheduler.rs");

    fn req(event_id: u64, target: Option<u64>, signal: i64) -> ScheduleRequest {
        ScheduleRequest {
            observation: StagedObservation {
                event_id: EventId(event_id),
                source: SourceSession {
                    source: "s".to_string(),
                    session: 1,
                },
                source_sequence: 0,
                signal: Scalar::from_int(signal),
            },
            target_tick: target.map(Tick),
        }
    }

    fn cfg() -> SchedulerConfig {
        SchedulerConfig {
            horizon: 10,
            max_per_tick: 8,
        }
    }

    #[test]
    fn schedule_same_inputs_same_order() {
        let run = || {
            let mut s = TickScheduler::new(cfg());
            for (e, t) in [(1u64, 3u64), (2, 2), (3, 3)] {
                s.schedule(Tick(0), req(e, Some(t), 1));
            }
            s
        };
        assert_eq!(
            run(),
            run(),
            "same requests in the same order produce the same schedule"
        );
    }

    #[test]
    fn target_tick_required() {
        let mut s = TickScheduler::new(cfg());
        let out = s.schedule(Tick(0), req(1, None, 5));
        assert!(
            matches!(
                out,
                ScheduleOutcome::Rejected(ScheduleReceipt {
                    reason: ScheduleReason::MissingTargetTick,
                    ..
                })
            ),
            "a request with no target tick is rejected"
        );
        assert_eq!(s.scheduled_count(), 0);
    }

    #[test]
    fn future_horizon_enforced() {
        let mut s = TickScheduler::new(SchedulerConfig {
            horizon: 5,
            max_per_tick: 8,
        });
        // beyond now + horizon (10 + 5 = 15).
        let beyond = s.schedule(Tick(10), req(1, Some(100), 5));
        assert!(matches!(
            beyond,
            ScheduleOutcome::Rejected(ScheduleReceipt {
                reason: ScheduleReason::BeyondHorizon { .. },
                ..
            })
        ));
        // not strictly in the future (target == now).
        let past = s.schedule(Tick(10), req(2, Some(10), 5));
        assert!(matches!(
            past,
            ScheduleOutcome::Rejected(ScheduleReceipt {
                reason: ScheduleReason::TargetTickInPast { .. },
                ..
            })
        ));
        // a valid in-window target (now < target <= now + horizon) is accepted.
        let ok = s.schedule(Tick(10), req(3, Some(15), 5));
        assert!(matches!(ok, ScheduleOutcome::Scheduled(_)));
        assert_eq!(s.scheduled_count(), 1);
    }

    #[test]
    fn overload_rejected_with_receipt() {
        let mut s = TickScheduler::new(SchedulerConfig {
            horizon: 10,
            max_per_tick: 2,
        });
        assert!(matches!(
            s.schedule(Tick(0), req(1, Some(1), 1)),
            ScheduleOutcome::Scheduled(_)
        ));
        assert!(matches!(
            s.schedule(Tick(0), req(2, Some(1), 1)),
            ScheduleOutcome::Scheduled(_)
        ));
        let over = s.schedule(Tick(0), req(3, Some(1), 1));
        assert!(
            matches!(
                over,
                ScheduleOutcome::Rejected(ScheduleReceipt {
                    reason: ScheduleReason::Overload {
                        target: 1,
                        capacity: 2
                    },
                    ..
                })
            ),
            "overload returns a receipt, not silent loss"
        );
        assert_eq!(
            s.schedule_for(Tick(1)).len(),
            2,
            "the overflowing observation is not silently placed"
        );
    }

    #[test]
    fn duplicate_schedule_idempotent() {
        let mut s = TickScheduler::new(cfg());
        let a = s.schedule(Tick(0), req(7, Some(3), 5));
        // same event_id, different target -> idempotent no-op.
        let b = s.schedule(Tick(0), req(7, Some(9), 5));
        assert!(matches!(a, ScheduleOutcome::Scheduled(_)));
        assert!(
            matches!(b, ScheduleOutcome::Duplicate(_)),
            "a repeated EventId is a duplicate"
        );
        assert_eq!(
            s.scheduled_count(),
            1,
            "a duplicate EventId creates no duplicate work"
        );
        assert_eq!(s.schedule_for(Tick(3)).len(), 1);
        assert_eq!(
            s.schedule_for(Tick(9)).len(),
            0,
            "the duplicate produced no second placement"
        );
    }

    #[test]
    fn scheduler_does_not_call_evaluate_tick() {
        for needle in ["evaluate_tick", "EngineState", "VibeEngine"] {
            assert!(
                !SCHED_SRC.contains(needle),
                "scheduler (scheduler.rs) must not reference `{needle}` — ordering only"
            );
        }
    }

    #[test]
    fn scheduler_does_not_mutate_state() {
        let state = EngineState::genesis(1);
        let snapshot = state.clone();
        let mut s = TickScheduler::new(cfg());
        let _ = s.schedule(Tick(0), req(1, Some(2), 5));
        assert_eq!(
            state, snapshot,
            "scheduling must not touch the engine state the caller holds"
        );
    }
}
