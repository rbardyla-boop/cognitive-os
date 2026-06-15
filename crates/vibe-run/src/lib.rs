//! vibe-run — ADR-002 layer-L2 deterministic record/replay for Cognitive OS.
//!
//! Records a full deterministic run (ingress -> schedule -> frames -> engine
//! outputs -> run hash) and replays it from recorded evidence alone, detecting
//! tampering. It drives the one engine; it is not a second engine. See
//! `ADR-002-runtime-engine-replay-contract.md`.

#![forbid(unsafe_code)]

mod runner;

pub use runner::{
    RecordedRun, RecordedTick, ReplayReport, ReplayRunner, RunRecorder, RunScript,
    ScriptedObservation,
};

#[cfg(test)]
mod tests {
    use super::*;
    use vibe_core::{Scalar, Tick};
    use vibe_frame::FrameCollector;
    use vibe_ingress::{EventId, ObservationEnvelope, SourceSession};
    use vibe_scheduler::SchedulerConfig;

    fn envelope(event_id: u64, seq: u64, signal: i64) -> ObservationEnvelope {
        ObservationEnvelope {
            event_id: EventId(event_id),
            source: SourceSession {
                source: "s".to_string(),
                session: 1,
            },
            source_sequence: seq,
            signal: Scalar::from_int(signal),
        }
    }

    /// A 3-tick script: obs to tick 1 (signal 10), and ticks 2 (signals 20, 30).
    fn script() -> RunScript {
        RunScript {
            seed: 7,
            scheduler: SchedulerConfig {
                horizon: 10,
                max_per_tick: 8,
            },
            now: Tick(0),
            observations: vec![
                ScriptedObservation {
                    envelope: envelope(1, 0, 10),
                    target_tick: Tick(1),
                },
                ScriptedObservation {
                    envelope: envelope(2, 1, 20),
                    target_tick: Tick(2),
                },
                ScriptedObservation {
                    envelope: envelope(3, 2, 30),
                    target_tick: Tick(2),
                },
            ],
            run_ticks: 3,
        }
    }

    #[test]
    fn run_script_drives_deterministic_run() {
        let a = RunRecorder::new().record(&script());
        let b = RunRecorder::new().record(&script());
        assert_eq!(a, b, "the same script records the same run");
        assert_eq!(
            a.run_hash, b.run_hash,
            "the same script yields the same run_hash"
        );
        assert_eq!(a.ticks.len(), 3, "all ticks are recorded");
        // the recorder folded the scheduled signals: tick1=10, tick2=50, tick3=0.
        assert_eq!(a.ticks[0].output.vibe, Scalar::from_int(10));
        assert_eq!(a.ticks[1].output.vibe, Scalar::from_int(60));
        assert_eq!(a.ticks[2].output.vibe, Scalar::from_int(60));
    }

    #[test]
    fn record_then_replay_same_hash() {
        let recorded = RunRecorder::new().record(&script());
        let report = ReplayRunner::new().replay(&recorded);
        assert!(
            report.verified,
            "an authentic recording replays as verified"
        );
        assert!(report.run_hash_matches);
        assert_eq!(
            report.run_hash, recorded.run_hash,
            "replay reproduces the run_hash"
        );
        assert!(report.output_mismatches.is_empty());
    }

    #[test]
    fn replay_reconstructs_frames() {
        let recorded = RunRecorder::new().record(&script());
        let report = ReplayRunner::new().replay(&recorded);
        let recorded_frames: Vec<_> = recorded.ticks.iter().map(|t| &t.frame).collect();
        let replayed_frames: Vec<_> = report.reconstructed.iter().map(|t| &t.frame).collect();
        assert_eq!(
            recorded_frames, replayed_frames,
            "replay reproduces the same frames"
        );
    }

    #[test]
    fn replay_reconstructs_outputs() {
        let recorded = RunRecorder::new().record(&script());
        let report = ReplayRunner::new().replay(&recorded);
        let recorded_outputs: Vec<_> = recorded.ticks.iter().map(|t| &t.output).collect();
        let replayed_outputs: Vec<_> = report.reconstructed.iter().map(|t| &t.output).collect();
        assert_eq!(
            recorded_outputs, replayed_outputs,
            "replay reproduces the same outputs"
        );
    }

    #[test]
    fn replay_reconstructs_state_transitions() {
        let recorded = RunRecorder::new().record(&script());
        let report = ReplayRunner::new().replay(&recorded);
        for (rec, rep) in recorded.ticks.iter().zip(&report.reconstructed) {
            assert_eq!(
                rec.output.transition, rep.output.transition,
                "replay reproduces the explicit state transition for each tick"
            );
        }
    }

    #[test]
    fn tampered_recorded_run_detected() {
        let recorder = RunRecorder::new();
        let replayer = ReplayRunner::new();

        // (a) a tampered OUTPUT (frame intact) is caught by the output check.
        let mut t_out = recorder.record(&script());
        t_out.ticks[0].output.vibe = Scalar::from_int(999);
        let r_out = replayer.replay(&t_out);
        assert!(!r_out.verified, "a tampered output is detected");
        assert!(r_out.output_mismatches.contains(&0));

        // (b) a tampered RUN HASH is caught by the run-hash check.
        let mut t_hash = recorder.record(&script());
        t_hash.run_hash ^= 0xdead_beef;
        let r_hash = replayer.replay(&t_hash);
        assert!(!r_hash.verified, "a tampered run_hash is detected");
        assert!(!r_hash.run_hash_matches);

        // (c) a tampered FRAME changes the recomputed output and run hash.
        let mut t_frame = recorder.record(&script());
        t_frame.ticks[0].frame = FrameCollector::new().collect(Tick(1), &[]);
        let r_frame = replayer.replay(&t_frame);
        assert!(!r_frame.verified, "a tampered frame is detected");

        // (d) a RELABELLED tick (its label disagrees with its own frame/output)
        // is caught by the internal-consistency check.
        let mut t_label = recorder.record(&script());
        t_label.ticks[0].tick = Tick(99);
        let r_label = replayer.replay(&t_label);
        assert!(
            !r_label.verified,
            "an internally-inconsistent tick label is detected"
        );
        assert!(r_label.tick_mismatches.contains(&0));
    }

    #[test]
    fn replay_does_not_depend_on_live_input() {
        let recorded = RunRecorder::new().record(&script());
        // Replay takes ONLY the recorded run — no script, no gate, no live input.
        // Clearing the ingress/scheduler evidence must not change the replay: it
        // reproduces from the recorded frames and seed alone.
        let mut frames_only = recorded.clone();
        frames_only.accepted.clear();
        frames_only.scheduled.clear();
        let report = ReplayRunner::new().replay(&frames_only);
        assert!(
            report.verified,
            "replay reproduces from recorded frames + seed alone"
        );
        assert_eq!(report.run_hash, recorded.run_hash);
    }

    #[test]
    fn record_from_frames_reproduces_run() {
        // The load-side entry point: re-deriving a run from its frames alone
        // reproduces the same run_hash and outputs (this is what the CLI replays).
        let recorded = RunRecorder::new().record(&script());
        let frames: Vec<_> = recorded.ticks.iter().map(|t| t.frame.clone()).collect();
        let rebuilt = RunRecorder::new().record_from_frames(recorded.seed, frames);
        assert_eq!(
            rebuilt.run_hash, recorded.run_hash,
            "re-deriving from frames reproduces the run_hash"
        );
        let recorded_outputs: Vec<_> = recorded.ticks.iter().map(|t| &t.output).collect();
        let rebuilt_outputs: Vec<_> = rebuilt.ticks.iter().map(|t| &t.output).collect();
        assert_eq!(
            recorded_outputs, rebuilt_outputs,
            "re-derived outputs match the recording"
        );
    }

    // The recorder/replayer drive the one engine; they do not reimplement it.
    #[test]
    fn run_layer_does_not_reimplement_the_engine() {
        const RUNNER_SRC: &str = include_str!("runner.rs");
        assert!(
            !RUNNER_SRC.contains("fn evaluate_tick"),
            "must not define its own evaluate_tick"
        );
        assert!(
            !RUNNER_SRC.contains("split_mix64"),
            "must not reimplement the engine's noise"
        );
        assert!(
            RUNNER_SRC.contains("evaluate_tick"),
            "must DRIVE the engine via evaluate_tick"
        );
    }
}
