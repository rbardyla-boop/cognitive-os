//! vibe-core — the ADR-002 layer-L0 deterministic replay kernel for Cognitive OS.
//!
//! This crate is pure replay math. It deliberately depends on nothing: no
//! wall-clock, no entropy source, no filesystem, no network, no signing, no
//! executor, no backend. Those concerns live in outer layers (L1 ingress,
//! scheduling, framing) and reach the kernel only through typed, validated
//! inputs. The engine consumes the canonical [`ObservationFrame`] (the single
//! frame definition, promoted here in P5). See
//! `ADR-002-runtime-engine-replay-contract.md`.

#![forbid(unsafe_code)]

mod kernel;

pub use kernel::{
    EngineOutput, EngineState, FrameObservation, ObservationFrame, Scalar, StateTransition, Tick,
    VibeEngine,
};

#[cfg(test)]
mod tests {
    use super::*;

    // The kernel source and manifest, scanned by the boundary tests below. The
    // forbidden tokens appear in THIS file only as test needles; the kernel
    // itself is in `kernel.rs`, which must stay free of them.
    const KERNEL_SRC: &str = include_str!("kernel.rs");
    const MANIFEST: &str = include_str!("../Cargo.toml");

    /// A frame whose observations are `(index, signal)` — one observation per
    /// signal, distinct ids.
    fn frame(tick: u64, signals: &[i64]) -> ObservationFrame {
        let observations = signals
            .iter()
            .enumerate()
            .map(|(i, s)| FrameObservation {
                id: i as u64,
                signal: Scalar::from_int(*s),
            })
            .collect();
        ObservationFrame::new(Tick(tick), observations)
    }

    /// A frame from explicit `(id, signal)` pairs — for canonicalization tests.
    fn frame_obs(tick: u64, pairs: &[(u64, i64)]) -> ObservationFrame {
        let observations = pairs
            .iter()
            .map(|(id, s)| FrameObservation {
                id: *id,
                signal: Scalar::from_int(*s),
            })
            .collect();
        ObservationFrame::new(Tick(tick), observations)
    }

    // --- evaluation determinism / purity (runtime-checkable) ---

    #[test]
    fn engine_consumes_canonical_frame() {
        // The engine folds ALL of the canonical frame's observation signals — it
        // does not consume a single loose signal.
        let engine = VibeEngine::new();
        let state = EngineState::genesis(1);
        let f = frame(5, &[10, 20, 30]);
        let (out, next) = engine.evaluate_tick(&state, &f);
        assert_eq!(
            out.transition.applied_signal,
            Scalar::from_int(60),
            "all observations are folded"
        );
        assert_eq!(next.vibe, Scalar::from_int(60));
        assert_eq!(
            out.frame_hash,
            f.frame_hash(),
            "the output carries the producing frame's hash"
        );
        // The frame is consumed already-canonical, so supply order cannot matter:
        let canonical = frame_obs(5, &[(0, 10), (1, 20), (2, 30)]);
        let shuffled = frame_obs(5, &[(2, 30), (0, 10), (1, 20)]);
        assert_eq!(
            engine.evaluate_tick(&state, &canonical),
            engine.evaluate_tick(&state, &shuffled),
            "differently-supplied but equal frames evaluate identically"
        );
    }

    #[test]
    fn same_state_same_frame_same_output() {
        let engine = VibeEngine::new();
        let state = EngineState::genesis(42);
        let f = frame(0, &[7]);
        assert_eq!(
            engine.evaluate_tick(&state, &f),
            engine.evaluate_tick(&state, &f),
            "identical (state, frame) must yield identical (output, next_state)"
        );
    }

    #[test]
    fn state_transition_explicit() {
        let engine = VibeEngine::new();
        let (out, next) = engine.evaluate_tick(&EngineState::genesis(0), &frame(0, &[3]));
        assert_eq!(out.transition.from_tick, Tick(0));
        assert_eq!(out.transition.to_tick, Tick(1));
        assert_eq!(out.transition.applied_signal, Scalar::from_int(3));
        // the transition describes exactly the state change that happened.
        assert_eq!(next.tick, out.transition.to_tick);
        assert_eq!(next.vibe, Scalar::from_int(3));
    }

    #[test]
    fn input_state_not_mutated() {
        let engine = VibeEngine::new();
        let state = EngineState::genesis(1);
        let snapshot = state.clone();
        let _ = engine.evaluate_tick(&state, &frame(0, &[3]));
        assert_eq!(
            state, snapshot,
            "evaluate_tick must not mutate its input state in place"
        );
    }

    #[test]
    fn multi_tick_scenario_reproducible() {
        let engine = VibeEngine::new();
        let signals = [2_i64, -5, 11, 0, 4];
        let run = |seed: u64| {
            let mut st = EngineState::genesis(seed);
            let mut outs = Vec::new();
            for (i, s) in signals.iter().enumerate() {
                let (o, n) = engine.evaluate_tick(&st, &frame(i as u64, &[*s]));
                outs.push(o);
                st = n;
            }
            (outs, st)
        };
        assert_eq!(
            run(99),
            run(99),
            "same seed + signals must reproduce the whole run"
        );
        let (_, final_state) = run(99);
        assert_eq!(
            final_state.tick,
            Tick(5),
            "state advances exactly one tick per evaluation"
        );
        assert_eq!(
            final_state.vibe,
            Scalar::from_int(12),
            "vibe accumulates the folded signals"
        );
    }

    #[test]
    fn output_hash_changes_when_frame_changes() {
        let engine = VibeEngine::new();
        let state = EngineState::genesis(7);
        let a = engine.evaluate_tick(&state, &frame(0, &[10])).0;
        let a_again = engine.evaluate_tick(&state, &frame(0, &[10])).0;
        let b = engine.evaluate_tick(&state, &frame(0, &[11])).0;
        assert_eq!(
            a.output_hash(),
            a_again.output_hash(),
            "same state+frame -> same output hash"
        );
        assert_ne!(
            a.output_hash(),
            b.output_hash(),
            "a different frame -> a different output hash"
        );
        // order independence carries through to the output hash.
        let canonical = engine
            .evaluate_tick(&state, &frame_obs(0, &[(0, 10), (1, 20)]))
            .0;
        let shuffled = engine
            .evaluate_tick(&state, &frame_obs(0, &[(1, 20), (0, 10)]))
            .0;
        assert_eq!(
            canonical.output_hash(),
            shuffled.output_hash(),
            "output hash is order-independent"
        );
    }

    #[test]
    fn no_randomness_without_seed() {
        let engine = VibeEngine::new();
        let f = frame(0, &[]);
        let (a1, _) = engine.evaluate_tick(&EngineState::genesis(7), &f);
        let (a2, _) = engine.evaluate_tick(&EngineState::genesis(7), &f);
        let (b, _) = engine.evaluate_tick(&EngineState::genesis(8), &f);
        assert_eq!(
            a1.noise, a2.noise,
            "same seed must reproduce the same noise"
        );
        assert_ne!(
            a1.noise, b.noise,
            "different seeds diverge — noise derives only from the seed"
        );
    }

    #[test]
    fn empty_frame_advances_tick_without_changing_vibe() {
        let engine = VibeEngine::new();
        let f = frame(0, &[]);
        assert!(f.is_empty(), "an empty tick is an explicit empty frame");
        let (out, next) = engine.evaluate_tick(&EngineState::genesis(1), &f);
        assert_eq!(out.transition.applied_signal, Scalar::ZERO);
        assert_eq!(next.tick, Tick(1));
        assert_eq!(
            next.vibe,
            Scalar::ZERO,
            "an empty frame folds to no vibe change"
        );
    }

    #[test]
    fn scalar_is_exact_fixed_point() {
        assert_eq!(
            Scalar::from_int(3).add(Scalar::from_int(4)),
            Scalar::from_int(7)
        );
        assert_eq!(Scalar::from_micros(1).micros(), 1);
        assert_eq!(Scalar::ZERO, Scalar::from_int(0));
    }

    // --- kernel-boundary source invariants (ADR-002 L0) ---

    #[test]
    fn no_wall_clock_in_core() {
        for needle in [
            "std::time",
            "SystemTime",
            "Instant",
            "thread::sleep",
            "monotonic",
        ] {
            assert!(
                !KERNEL_SRC.contains(needle),
                "L0 kernel must hold no wall-clock; found `{needle}` in kernel.rs"
            );
        }
    }

    #[test]
    fn no_external_randomness_in_core() {
        for needle in [
            "use rand",
            "rand::",
            "extern crate rand",
            "thread_rng",
            "getrandom",
        ] {
            assert!(
                !KERNEL_SRC.contains(needle),
                "L0 kernel must not read entropy; found `{needle}` in kernel.rs"
            );
        }
    }

    #[test]
    fn core_still_has_no_backend_dependencies() {
        for needle in [
            "tokio", ".await", "async fn", "reqwest", "sqlx", "rusqlite", "std::fs", "std::net",
            "ed25519", "openssl", "serde",
        ] {
            assert!(
                !KERNEL_SRC.contains(needle),
                "L0 kernel must hold no backend dependency; found `{needle}` in kernel.rs"
            );
        }
        // the manifest's [dependencies] table is empty.
        let after = MANIFEST.split("[dependencies]").nth(1).unwrap_or("");
        let body = after.split("\n[").next().unwrap_or("");
        for line in body.lines() {
            let t = line.trim();
            assert!(
                t.is_empty() || t.starts_with('#'),
                "vibe-core must declare zero dependencies; found `{t}` under [dependencies]"
            );
        }
    }
}
