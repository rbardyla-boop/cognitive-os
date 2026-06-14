//! ADR-002 layer L0 — the deterministic replay kernel.
//!
//! Pure replay math: `evaluate_tick(state, frame) -> (output, next_state)`.
//! The kernel holds no wall-clock, no entropy source, no filesystem, no
//! network, no signing, no executor, and no backend dependency. Its only
//! notion of time is a logical [`Tick`]; any seed-derived value is a pure
//! function of a seed carried in [`EngineState`], so a recorded run replays
//! bit-for-bit. See `ADR-002-runtime-engine-replay-contract.md`.

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

/// The canonical input for one tick. In P1 this is a stub carrying a single
/// signal; ingress, scheduling, and frame collection arrive in P2–P4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationFrame {
    pub tick: Tick,
    pub signal: Scalar,
}

impl ObservationFrame {
    pub const fn new(tick: Tick, signal: Scalar) -> Self {
        ObservationFrame { tick, signal }
    }
}

/// The deterministic output of evaluating one tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EngineOutput {
    pub tick: Tick,
    pub vibe: Scalar,
    /// Seed-derived pseudo-noise — a pure function of the state seed.
    pub noise: u64,
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
    /// outputs. The input `state` is borrowed and never mutated.
    pub fn evaluate_tick(
        &self,
        state: &EngineState,
        frame: &ObservationFrame,
    ) -> (EngineOutput, EngineState) {
        let next_vibe = state.vibe.add(frame.signal);
        let noise = split_mix64(state.seed);
        let next_state = EngineState {
            tick: state.tick.next(),
            vibe: next_vibe,
            seed: noise,
        };
        let output = EngineOutput {
            tick: next_state.tick,
            vibe: next_vibe,
            noise,
        };
        (output, next_state)
    }
}
