//! ADR-002 layer L0 — the deterministic replay kernel.
//!
//! Pure replay math: `evaluate_tick(state, frame) -> (output, next_state)`.
//! The kernel holds no wall-clock, no entropy source, no filesystem, no
//! network, no signing, no executor, and no backend dependency. Its only
//! notion of time is a logical [`Tick`]; any seed-derived value is a pure
//! function of a seed carried in [`EngineState`], so a recorded run replays
//! bit-for-bit. See `ADR-002-runtime-engine-replay-contract.md`.
//!
//! P5 promoted the canonical [`ObservationFrame`] into this L0 kernel (it was
//! prototyped in the L1 `vibe-frame` crate in P4) and retired the P1 stub frame,
//! so there is exactly one frame definition and the engine consumes it.

/// Fixed-point scalar in micro-units (scale 1e6) backed by `i64`: deterministic
/// integer arithmetic with exact equality, never floating-point. (ADR-002 L0.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Scalar(i64);

impl Scalar {
    /// Micro-units per whole unit.
    pub const SCALE: i64 = 1_000_000;
    pub const ZERO: Scalar = Scalar(0);

    /// Construct from raw micro-units.
    pub const fn from_micros(micros: i64) -> Self {
        Scalar(micros)
    }

    /// Construct from whole units (saturating, so it is total).
    pub const fn from_int(units: i64) -> Self {
        Scalar(units.saturating_mul(Self::SCALE))
    }

    /// Raw micro-unit value.
    pub const fn micros(self) -> i64 {
        self.0
    }

    /// Saturating addition — total and deterministic (no overflow, no panic).
    pub const fn add(self, rhs: Scalar) -> Scalar {
        Scalar(self.0.saturating_add(rhs.0))
    }
}

/// A logical tick: the kernel's only notion of time. Never a wall-clock value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Tick(pub u64);

impl Tick {
    /// The next logical tick. Saturating, so it is total.
    pub const fn next(self) -> Tick {
        Tick(self.0.saturating_add(1))
    }
}

/// Engine state. Every piece of mutable cognition lives here and is only ever
/// advanced by RETURNING a new value from [`VibeEngine::evaluate_tick`]; the
/// kernel never mutates a state in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineState {
    pub tick: Tick,
    /// Accumulated deterministic vibe value.
    pub vibe: Scalar,
    /// Seed for any seed-derived value. This is the ONLY input that drives the
    /// kernel's pseudo-noise, so the same seed reproduces the same stream.
    pub seed: u64,
}

impl EngineState {
    /// The genesis state at tick 0 for a given seed.
    pub const fn genesis(seed: u64) -> Self {
        EngineState {
            tick: Tick(0),
            vibe: Scalar::ZERO,
            seed,
        }
    }
}

/// One observation within a frame: a plain `u64` identity (kept in L0 so the
/// kernel stays dependency-free) and its payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameObservation {
    pub id: u64,
    pub signal: Scalar,
}

/// The canonical, hash-stable input the engine evaluates — the SINGLE frame
/// definition in the system. Build it with [`ObservationFrame::new`], which
/// canonicalizes (sorts) and hashes, so the frame and its hash depend only on
/// the SET of observations for a tick, not on the order they were supplied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationFrame {
    tick: Tick,
    observations: Vec<FrameObservation>,
    frame_hash: u64,
}

impl ObservationFrame {
    /// Build a canonical frame for `tick` from its observations.
    pub fn new(tick: Tick, mut observations: Vec<FrameObservation>) -> Self {
        observations.sort_by_key(|o| (o.id, o.signal.micros()));
        let frame_hash = hash_frame(tick, &observations);
        ObservationFrame {
            tick,
            observations,
            frame_hash,
        }
    }

    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Observations in canonical order (sorted by `(id, signal)`).
    pub fn observations(&self) -> &[FrameObservation] {
        &self.observations
    }

    /// Deterministic content hash — equal iff two frames have the same tick and
    /// the same set of observations, regardless of supply order.
    pub fn frame_hash(&self) -> u64 {
        self.frame_hash
    }

    /// An empty tick is represented by an explicit empty frame, not a skip.
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
}

/// An explicit, inspectable description of how one tick advanced the state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateTransition {
    pub from_tick: Tick,
    pub to_tick: Tick,
    /// The folded signal applied to the vibe this tick.
    pub applied_signal: Scalar,
}

/// The deterministic output of evaluating one tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineOutput {
    pub tick: Tick,
    pub vibe: Scalar,
    /// Seed-derived pseudo-noise — a pure function of the state seed.
    pub noise: u64,
    /// The hash of the frame that produced this output.
    pub frame_hash: u64,
    /// The explicit state transition this tick performed.
    pub transition: StateTransition,
    output_hash: u64,
}

impl EngineOutput {
    /// Deterministic fingerprint of this output (tick, vibe, noise, frame hash).
    /// Two runs with the same state and frame produce the same `output_hash`;
    /// any change to the frame changes it.
    pub fn output_hash(&self) -> u64 {
        self.output_hash
    }
}

/// FNV-1a 64-bit mixing of one value. Pure and deterministic on every platform.
/// Shared by the frame and output content hashes.
fn mix(mut h: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Hash a frame's canonical content: tick, count, then each observation's
/// identity and payload in canonical order.
fn hash_frame(tick: Tick, observations: &[FrameObservation]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h = mix(h, tick.0);
    h = mix(h, observations.len() as u64);
    for obs in observations {
        h = mix(h, obs.id);
        h = mix(h, obs.signal.micros() as u64);
    }
    h
}

/// Hash an output's content: tick, vibe, noise, and the producing frame's hash.
fn hash_output(tick: Tick, vibe: Scalar, noise: u64, frame_hash: u64) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h = mix(h, tick.0);
    h = mix(h, vibe.micros() as u64);
    h = mix(h, noise);
    h = mix(h, frame_hash);
    h
}

/// splitmix64: a deterministic seed-mixing step. The kernel's only producer of
/// a pseudo-noise value, and it is a pure function — same seed, same result, on
/// every platform. No entropy is read.
const fn split_mix64(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The stateless L0 kernel. It owns no data: all state is passed in and a new
/// state is returned, so engine state can change ONLY through `evaluate_tick`.
#[derive(Clone, Copy, Debug, Default)]
pub struct VibeEngine;

impl VibeEngine {
    pub const fn new() -> Self {
        VibeEngine
    }

    /// The replay contract — `(state, frame) -> (output, next_state)`. Pure,
    /// total, and deterministic: identical inputs always yield identical
    /// outputs. The input `state` is borrowed and never mutated. The canonical
    /// frame's observation signals are folded into the vibe.
    pub fn evaluate_tick(
        &self,
        state: &EngineState,
        frame: &ObservationFrame,
    ) -> (EngineOutput, EngineState) {
        let applied_signal = frame
            .observations()
            .iter()
            .fold(Scalar::ZERO, |acc, obs| acc.add(obs.signal));
        let next_vibe = state.vibe.add(applied_signal);
        let noise = split_mix64(state.seed);
        let next_state = EngineState {
            tick: state.tick.next(),
            vibe: next_vibe,
            seed: noise,
        };
        let transition = StateTransition {
            from_tick: state.tick,
            to_tick: next_state.tick,
            applied_signal,
        };
        let frame_hash = frame.frame_hash();
        let output_hash = hash_output(next_state.tick, next_vibe, noise, frame_hash);
        let output = EngineOutput {
            tick: next_state.tick,
            vibe: next_vibe,
            noise,
            frame_hash,
            transition,
            output_hash,
        };
        (output, next_state)
    }
}
