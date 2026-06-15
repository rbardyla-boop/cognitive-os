//! ADR-002 layer L1 — frame collection (the FrameCollector).
//!
//! Folds the observations scheduled for ONE logical tick into a single
//! canonical, hash-stable frame. P5 promoted the canonical `ObservationFrame`
//! into vibe-core (L0); this collector now PRODUCES that one type — it filters
//! the scheduled observations for the tick, maps them to the L0 observation
//! unit, and hands them to `vibe_core::ObservationFrame::new`, which owns the
//! canonical sort and hash. The collector never evaluates a tick and never
//! touches the engine's state — evaluation is the engine's job.
//! See `ADR-002-runtime-engine-replay-contract.md`.

use vibe_core::{FrameObservation, ObservationFrame, Tick};
use vibe_scheduler::ScheduledObservation;

/// Frame collection. Stateless: a pure fold from scheduled observations to a
/// canonical `vibe_core::ObservationFrame`.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCollector;

impl FrameCollector {
    pub const fn new() -> Self {
        FrameCollector
    }

    /// Fold the observations scheduled for `tick` into the canonical frame. Only
    /// observations whose `target_tick` equals `tick` are included (no other
    /// tick's observations leak in). `ObservationFrame::new` canonicalizes by
    /// sorting, so collection order cannot affect the frame or its hash. A tick
    /// with no matching observations yields an explicit empty frame.
    pub fn collect(&self, tick: Tick, scheduled: &[ScheduledObservation]) -> ObservationFrame {
        let observations: Vec<FrameObservation> = scheduled
            .iter()
            .filter(|s| s.target_tick == tick)
            .map(|s| FrameObservation {
                id: s.event_id.0,
                signal: s.signal,
            })
            .collect();
        ObservationFrame::new(tick, observations)
    }
}
