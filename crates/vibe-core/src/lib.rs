//! vibe-core — the ADR-002 layer-L0 deterministic replay kernel for Cognitive OS.
//!
//! This crate is pure replay math. It deliberately depends on nothing: no
//! wall-clock, no entropy source, no filesystem, no network, no signing, no
//! executor, no backend. Those concerns live in outer layers (L1 ingress,
//! L2 record/replay) and reach the kernel only through typed, validated
//! inputs. See `ADR-002-runtime-engine-replay-contract.md`.

#![forbid(unsafe_code)]

mod kernel;

pub use kernel::{EngineOutput, EngineState, ObservationFrame, Scalar, Tick, VibeEngine};

#[cfg(test)]
mod tests {
    use super::*;

    // The kernel source and manifest, scanned by the boundary tests below. The
    // forbidden tokens appear in THIS file only as test needles; the kernel
    // itself lives in `kernel.rs`, which must stay free of them.
    const KERNEL_SRC: &str = include_str!("kernel.rs");
    const MANIFEST: &str = include_str!("../Cargo.toml");

    fn frame(tick: u64, signal_units: i64) -> ObservationFrame {
        ObservationFrame::new(Tick(tick), Scalar::from_int(signal_units))
    }

    // --- determinism / purity of the replay math (runtime-checkable) ---

    #[test]
    fn same_state_same_frame_same_output() {
        let engine = VibeEngine::new();
        let state = EngineState::genesis(42);
        let f = frame(0, 7);
        assert_eq!(
            engine.evaluate_tick(&state, &f),
            engine.evaluate_tick(&state, &f),
            "identical (state, frame) must yield identical (output, next_state)"
        );
    }

    #[test]
    fn state_changes_only_through_evaluate_tick() {
        let engine = VibeEngine::new();
        let state = EngineState::genesis(1);
        let (_, next) = engine.evaluate_tick(&state, &frame(0, 3));
        // the borrowed input state is untouched ...
        assert_eq!(
            state,
            EngineState::genesis(1),
            "evaluate_tick must not mutate its input state"
        );
        // ... and the returned state advanced by exactly one tick + the signal.
        assert_eq!(next.tick, Tick(1));
        assert_eq!(next.vibe, Scalar::from_int(3));
    }

    #[test]
    fn multi_tick_scenario_is_reproducible() {
        let engine = VibeEngine::new();
        let signals = [2_i64, -5, 11, 0, 4];
        let run = |seed: u64| {
            let mut st = EngineState::genesis(seed);
            let mut outs = Vec::new();
            for (i, s) in signals.iter().enumerate() {
                let (o, n) = engine.evaluate_tick(&st, &frame(i as u64, *s));
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
            "vibe accumulates the signals (2 - 5 + 11 + 0 + 4)"
        );
    }

    #[test]
    fn no_randomness_without_seed() {
        let engine = VibeEngine::new();
        let f = frame(0, 0);
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
    fn kernel_has_no_backend_dependencies() {
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
