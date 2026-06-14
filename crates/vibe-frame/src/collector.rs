//! ADR-002 layer L1 — frame collection (the FrameCollector).
//!
//! Folds the observations scheduled for ONE logical tick into a single
//! canonical, hash-stable [`ObservationFrame`]. Canonical means the frame
//! content and its hash depend only on the SET of observations for the tick, not
//! on the order they were collected in. The collector never evaluates a tick and
//! never touches the engine's state — evaluation is P5. Empty ticks produce an
//! explicit empty frame, never a skipped one.
//! See `ADR-002-runtime-engine-replay-contract.md`.
//!
//! NOTE: the canonical `ObservationFrame` lives here (L1) in P4; P5 promotes it
//! into vibe-core (L0) and rewires the engine to consume it, retiring the P1
//! stub frame in vibe-core. P4 does not touch the engine.

use vibe_core::{Scalar, Tick};
use vibe_ingress::EventId;
use vibe_scheduler::ScheduledObservation;

/// One observation within a frame — the canonical unit (identity + payload).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameObservation {
    pub event_id: EventId,
    pub signal: Scalar,
}

/// The canonical, hash-stable fold of every observation scheduled for one tick.
/// Built only via [`FrameCollector::collect`], so its invariants (canonical
/// order, matching hash) always hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationFrame {
    tick: Tick,
    observations: Vec<FrameObservation>,
    frame_hash: u64,
}

impl ObservationFrame {
    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Observations in canonical order (sorted by `(event_id, signal)`).
    pub fn observations(&self) -> &[FrameObservation] {
        &self.observations
    }

    /// Deterministic content hash — equal iff two frames have the same tick and
    /// the same set of observations, regardless of collection order.
    pub fn frame_hash(&self) -> u64 {
        self.frame_hash
    }

    /// An empty tick yields an explicit empty frame (not a skipped one).
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
}

/// FNV-1a 64-bit mixing of one value. Pure and deterministic on every platform.
fn mix(mut h: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Hash the canonical frame content: tick, count, then each observation's
/// identity and payload in canonical order.
fn hash_frame(tick: Tick, observations: &[FrameObservation]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h = mix(h, tick.0);
    h = mix(h, observations.len() as u64);
    for obs in observations {
        h = mix(h, obs.event_id.0);
        h = mix(h, obs.signal.micros() as u64);
    }
    h
}

/// Frame collection. Stateless: a pure fold from scheduled observations to a
/// canonical frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCollector;

impl FrameCollector {
    pub const fn new() -> Self {
        FrameCollector
    }

    /// Fold the observations scheduled for `tick` into a canonical frame. Only
    /// observations whose `target_tick` equals `tick` are included (no other
    /// tick's observations leak in). The result is canonicalized by sorting on
    /// `(event_id, signal)`, so collection order cannot affect the frame or its
    /// hash. A tick with no matching observations yields an explicit empty frame.
    pub fn collect(&self, tick: Tick, scheduled: &[ScheduledObservation]) -> ObservationFrame {
        let mut observations: Vec<FrameObservation> = scheduled
            .iter()
            .filter(|s| s.target_tick == tick)
            .map(|s| FrameObservation {
                event_id: s.event_id,
                signal: s.signal,
            })
            .collect();
        // Total order on (event_id, signal) -> canonical regardless of input
        // order, even for pathological inputs with repeated identities.
        observations.sort_by_key(|o| (o.event_id.0, o.signal.micros()));
        let frame_hash = hash_frame(tick, &observations);
        ObservationFrame {
            tick,
            observations,
            frame_hash,
        }
    }
}
