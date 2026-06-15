//! vibe-frame — ADR-002 layer-L1 frame collection for Cognitive OS.
//!
//! Folds the observations scheduled for one logical tick into the canonical,
//! hash-stable `vibe_core::ObservationFrame` (the single frame definition,
//! promoted into L0 in P5). It never evaluates a tick and never touches the
//! engine's state; evaluation is the engine's job. See
//! `ADR-002-runtime-engine-replay-contract.md`.

#![forbid(unsafe_code)]

mod collector;

pub use collector::FrameCollector;
// Re-exported for convenience — these are the single L0 definitions, not copies.
pub use vibe_core::{FrameObservation, ObservationFrame};

#[cfg(test)]
mod tests {
    use super::*;
    use vibe_core::{EngineState, Scalar, Tick, VibeEngine};
    use vibe_ingress::EventId;
    use vibe_scheduler::ScheduledObservation;

    // The collector source — scanned by the boundary test. The engine tokens
    // appear in THIS file only as test needles; `collector.rs` stays free of
    // them so frame collection structurally cannot evaluate or mutate the engine.
    const COLLECTOR_SRC: &str = include_str!("collector.rs");

    fn sched(event_id: u64, target: u64, signal: i64) -> ScheduledObservation {
        ScheduledObservation {
            target_tick: Tick(target),
            event_id: EventId(event_id),
            signal: Scalar::from_int(signal),
        }
    }

    #[test]
    fn same_tick_same_frame_hash() {
        let c = FrameCollector::new();
        let obs = [sched(1, 5, 10), sched(2, 5, 20), sched(3, 5, 30)];
        let f1 = c.collect(Tick(5), &obs);
        let f2 = c.collect(Tick(5), &obs);
        assert_eq!(f1.frame_hash(), f2.frame_hash());
        assert_eq!(
            f1, f2,
            "same observations for a tick produce the same frame"
        );
    }

    #[test]
    fn different_order_same_canonical_frame() {
        let c = FrameCollector::new();
        let ascending = [sched(1, 5, 10), sched(2, 5, 20), sched(3, 5, 30)];
        let shuffled = [sched(3, 5, 30), sched(1, 5, 10), sched(2, 5, 20)];
        let fa = c.collect(Tick(5), &ascending);
        let fb = c.collect(Tick(5), &shuffled);
        assert_eq!(
            fa, fb,
            "insertion order must not change the canonical frame"
        );
        assert_eq!(
            fa.frame_hash(),
            fb.frame_hash(),
            "frame hash must not depend on order"
        );
        let ids: Vec<u64> = fa.observations().iter().map(|o| o.id).collect();
        assert_eq!(
            ids,
            vec![1, 2, 3],
            "observations are in canonical (sorted) order"
        );
    }

    #[test]
    fn different_content_different_frame_hash() {
        let c = FrameCollector::new();
        let base = c.collect(Tick(5), &[sched(1, 5, 10), sched(2, 5, 20)]);
        let diff_signal = c.collect(Tick(5), &[sched(1, 5, 10), sched(2, 5, 21)]);
        let diff_event = c.collect(Tick(5), &[sched(1, 5, 10), sched(3, 5, 20)]);
        assert_ne!(
            base.frame_hash(),
            diff_signal.frame_hash(),
            "a differing signal changes the hash"
        );
        assert_ne!(
            base.frame_hash(),
            diff_event.frame_hash(),
            "a differing event_id changes the hash"
        );
    }

    #[test]
    fn repeated_identity_canonicalized_deterministically() {
        let c = FrameCollector::new();
        let forward = c.collect(Tick(5), &[sched(1, 5, 10), sched(1, 5, 20)]);
        let reverse = c.collect(Tick(5), &[sched(1, 5, 20), sched(1, 5, 10)]);
        assert_eq!(
            forward, reverse,
            "repeated identities canonicalize regardless of input order"
        );
        let signals: Vec<i64> = forward
            .observations()
            .iter()
            .map(|o| o.signal.micros())
            .collect();
        assert_eq!(
            signals,
            vec![10_000_000, 20_000_000],
            "the tiebreaker orders by signal"
        );
    }

    #[test]
    fn empty_tick_frame_is_explicit() {
        let c = FrameCollector::new();
        let empty = c.collect(Tick(9), &[]);
        assert!(
            empty.is_empty(),
            "an empty tick yields an explicit empty frame"
        );
        assert_eq!(empty.tick(), Tick(9));
        assert_eq!(empty.observations().len(), 0);
        assert_eq!(empty.frame_hash(), c.collect(Tick(9), &[]).frame_hash());
        assert_ne!(
            empty.frame_hash(),
            c.collect(Tick(9), &[sched(1, 9, 1)]).frame_hash()
        );
    }

    #[test]
    fn frame_contains_only_scheduled_observations() {
        let c = FrameCollector::new();
        let mixed = [
            sched(1, 3, 10),
            sched(2, 5, 20),
            sched(3, 3, 30),
            sched(4, 7, 40),
        ];
        let frame = c.collect(Tick(3), &mixed);
        let ids: Vec<u64> = frame.observations().iter().map(|o| o.id).collect();
        assert_eq!(
            ids,
            vec![1, 3],
            "no observations from other ticks leak into the frame"
        );
    }

    #[test]
    fn collected_frame_is_consumable_by_engine() {
        // End-to-end through the real layers: collect a canonical frame and feed
        // it to the L0 engine. The engine folds the frame's observation signals.
        let c = FrameCollector::new();
        let frame = c.collect(
            Tick(5),
            &[sched(1, 5, 10), sched(2, 5, 20), sched(3, 5, 30)],
        );
        let engine = VibeEngine::new();
        let (out, next) = engine.evaluate_tick(&EngineState::genesis(1), &frame);
        assert_eq!(
            out.frame_hash,
            frame.frame_hash(),
            "the output carries the collected frame's hash"
        );
        assert_eq!(
            next.vibe,
            Scalar::from_int(60),
            "the engine folds the collected observations"
        );
        // and it is reproducible.
        assert_eq!(
            engine.evaluate_tick(&EngineState::genesis(1), &frame),
            engine.evaluate_tick(&EngineState::genesis(1), &frame)
        );
    }

    #[test]
    fn collector_does_not_call_evaluate_tick() {
        for needle in ["evaluate_tick", "EngineState", "VibeEngine"] {
            assert!(
                !COLLECTOR_SRC.contains(needle),
                "collector (collector.rs) must not reference `{needle}` — framing only"
            );
        }
    }

    #[test]
    fn collector_does_not_mutate_engine_state() {
        let state = EngineState::genesis(1);
        let snapshot = state.clone();
        let c = FrameCollector::new();
        let _ = c.collect(Tick(5), &[sched(1, 5, 10)]);
        assert_eq!(
            state, snapshot,
            "frame collection must not touch the engine state the caller holds"
        );
    }
}
