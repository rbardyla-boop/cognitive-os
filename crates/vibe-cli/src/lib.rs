//! vibe-cli — the local operator CLI for Cognitive OS (ADR-002).
//!
//! Three operations over a deterministic run:
//!   - `run`    : ingest a scenario, record the run, emit a recorded-run file.
//!   - `replay` : re-derive the run from the recorded frames and report the hash.
//!   - `verify` : report whether the recorded run is authentic and reproducible.
//!
//! This is the IO/operator outer layer: it is the ONLY crate allowed external
//! dependencies (serde/serde_json), and serialization happens here via plain
//! data-transfer objects — the engine value types never derive serde, so the
//! deterministic engine stays dependency-free. A recorded run persists only its
//! seed, per-tick frame observations, and run hash; replay rebuilds the frames
//! via `ObservationFrame::new` (re-validating them) and re-runs the engine, so a
//! tampered file is detected by a run-hash mismatch.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use vibe_core::{FrameObservation, ObservationFrame, Scalar, Tick};
use vibe_ingress::{EventId, ObservationEnvelope, SourceSession};
use vibe_run::{RecordedRun, RunRecorder, RunScript, ScriptedObservation};
use vibe_scheduler::SchedulerConfig;

const SCHEMA_SCENARIO: &str = "vibe-scenario-v1";
const SCHEMA_RUN: &str = "vibe-recorded-run-v1";

// --- serde data-transfer objects (primitives only; never engine types) ---

#[derive(Serialize, Deserialize)]
struct SchedulerDto {
    horizon: u64,
    max_per_tick: usize,
}

#[derive(Serialize, Deserialize)]
struct ObservationDto {
    event_id: u64,
    source: String,
    session: u64,
    source_sequence: u64,
    target_tick: u64,
    signal_micros: i64,
}

#[derive(Serialize, Deserialize)]
struct ScenarioFile {
    schema: String,
    seed: u64,
    scheduler: SchedulerDto,
    now: u64,
    run_ticks: u64,
    observations: Vec<ObservationDto>,
}

#[derive(Serialize, Deserialize)]
struct ObsDto {
    id: u64,
    signal_micros: i64,
}

#[derive(Serialize, Deserialize)]
struct FrameDto {
    tick: u64,
    observations: Vec<ObsDto>,
}

#[derive(Serialize, Deserialize)]
struct RecordedRunFile {
    schema: String,
    seed: u64,
    frames: Vec<FrameDto>,
    run_hash: u64,
}

impl ScenarioFile {
    fn into_run_script(self) -> RunScript {
        let observations = self
            .observations
            .into_iter()
            .map(|o| ScriptedObservation {
                envelope: ObservationEnvelope {
                    event_id: EventId(o.event_id),
                    source: SourceSession {
                        source: o.source,
                        session: o.session,
                    },
                    source_sequence: o.source_sequence,
                    signal: Scalar::from_micros(o.signal_micros),
                },
                target_tick: Tick(o.target_tick),
            })
            .collect();
        RunScript {
            seed: self.seed,
            scheduler: SchedulerConfig {
                horizon: self.scheduler.horizon,
                max_per_tick: self.scheduler.max_per_tick,
            },
            now: Tick(self.now),
            observations,
            run_ticks: self.run_ticks,
        }
    }
}

impl RecordedRunFile {
    fn from_recorded(recorded: &RecordedRun) -> Self {
        let frames = recorded
            .ticks
            .iter()
            .map(|t| FrameDto {
                tick: t.frame.tick().0,
                observations: t
                    .frame
                    .observations()
                    .iter()
                    .map(|o| ObsDto {
                        id: o.id,
                        signal_micros: o.signal.micros(),
                    })
                    .collect(),
            })
            .collect();
        RecordedRunFile {
            schema: SCHEMA_RUN.to_string(),
            seed: recorded.seed,
            frames,
            run_hash: recorded.run_hash,
        }
    }

    // Rebuild the canonical frames via `ObservationFrame::new`, which recomputes
    // each frame's hash from the stored observations — a tampered observation set
    // therefore yields a different frame, hence a different run hash on replay.
    fn to_frames(&self) -> Vec<ObservationFrame> {
        self.frames
            .iter()
            .map(|f| {
                let observations = f
                    .observations
                    .iter()
                    .map(|o| FrameObservation {
                        id: o.id,
                        signal: Scalar::from_micros(o.signal_micros),
                    })
                    .collect();
                ObservationFrame::new(Tick(f.tick), observations)
            })
            .collect()
    }
}

/// The outcome of `run`.
pub struct RunOutcome {
    /// The serialized recorded-run file (JSON).
    pub recorded_run_json: String,
    pub ticks: usize,
    pub run_hash: u64,
    pub final_vibe_micros: i64,
}

/// The outcome of `replay`.
pub struct ReplayOutcome {
    pub matches: bool,
    pub recomputed_run_hash: u64,
    pub expected_run_hash: u64,
    pub ticks: usize,
}

/// Ingest a scenario (JSON) and record a deterministic run; return the recorded
/// run as JSON plus a small summary.
pub fn run_scenario(scenario_json: &str) -> Result<RunOutcome, String> {
    let scenario: ScenarioFile =
        serde_json::from_str(scenario_json).map_err(|e| format!("invalid scenario: {e}"))?;
    if scenario.schema != SCHEMA_SCENARIO {
        return Err(format!(
            "unexpected scenario schema {:?}, want {SCHEMA_SCENARIO:?}",
            scenario.schema
        ));
    }
    let script = scenario.into_run_script();
    let recorded = RunRecorder::new().record(&script);
    let file = RecordedRunFile::from_recorded(&recorded);
    let recorded_run_json =
        serde_json::to_string_pretty(&file).map_err(|e| format!("serialize recorded run: {e}"))?;
    let final_vibe_micros = recorded.ticks.last().map_or(0, |t| t.output.vibe.micros());
    Ok(RunOutcome {
        recorded_run_json,
        ticks: recorded.ticks.len(),
        run_hash: recorded.run_hash,
        final_vibe_micros,
    })
}

/// Re-derive a recorded run from its frames alone and compare the run hash to the
/// recording. No live input, no re-admission, no re-scheduling.
pub fn replay_run(recorded_json: &str) -> Result<ReplayOutcome, String> {
    let file: RecordedRunFile =
        serde_json::from_str(recorded_json).map_err(|e| format!("invalid recorded run: {e}"))?;
    if file.schema != SCHEMA_RUN {
        return Err(format!(
            "unexpected recorded-run schema {:?}, want {SCHEMA_RUN:?}",
            file.schema
        ));
    }
    let expected_run_hash = file.run_hash;
    let frames = file.to_frames();
    let rebuilt = RunRecorder::new().record_from_frames(file.seed, frames);
    Ok(ReplayOutcome {
        matches: rebuilt.run_hash == expected_run_hash,
        recomputed_run_hash: rebuilt.run_hash,
        expected_run_hash,
        ticks: rebuilt.ticks.len(),
    })
}

/// Verify a recorded run is authentic and reproducible (true) or tampered (false).
pub fn verify_run(recorded_json: &str) -> Result<bool, String> {
    Ok(replay_run(recorded_json)?.matches)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENARIO: &str = r#"{
        "schema": "vibe-scenario-v1",
        "seed": 7,
        "scheduler": { "horizon": 10, "max_per_tick": 8 },
        "now": 0,
        "run_ticks": 3,
        "observations": [
            { "event_id": 1, "source": "s", "session": 1, "source_sequence": 0, "target_tick": 1, "signal_micros": 10000000 },
            { "event_id": 2, "source": "s", "session": 1, "source_sequence": 1, "target_tick": 2, "signal_micros": 20000000 },
            { "event_id": 3, "source": "s", "session": 1, "source_sequence": 2, "target_tick": 2, "signal_micros": 30000000 }
        ]
    }"#;

    #[test]
    fn cli_run_scenario_succeeds() {
        let outcome = run_scenario(SCENARIO).expect("valid scenario records a run");
        assert_eq!(outcome.ticks, 3, "all ticks recorded");
        assert_eq!(
            outcome.final_vibe_micros, 60_000_000,
            "final vibe is the folded total (10+20+30)"
        );
        assert!(outcome.run_hash != 0);
    }

    #[test]
    fn cli_writes_recorded_run() {
        let outcome = run_scenario(SCENARIO).unwrap();
        let v: serde_json::Value = serde_json::from_str(&outcome.recorded_run_json).unwrap();
        assert_eq!(v["schema"], SCHEMA_RUN);
        assert_eq!(v["frames"].as_array().unwrap().len(), 3);
        assert_eq!(v["run_hash"].as_u64().unwrap(), outcome.run_hash);
        // tick 1 frame carries the single observation; tick 2 carries two.
        assert_eq!(v["frames"][0]["observations"].as_array().unwrap().len(), 1);
        assert_eq!(v["frames"][1]["observations"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn cli_replay_matches_original() {
        let outcome = run_scenario(SCENARIO).unwrap();
        let replay = replay_run(&outcome.recorded_run_json).unwrap();
        assert!(
            replay.matches,
            "an authentic recorded run replays as a match"
        );
        assert_eq!(replay.recomputed_run_hash, outcome.run_hash);
        assert!(verify_run(&outcome.recorded_run_json).unwrap());
    }

    #[test]
    fn cli_verify_detects_tamper() {
        let outcome = run_scenario(SCENARIO).unwrap();

        // (a) tamper the stored run hash.
        let mut a: serde_json::Value = serde_json::from_str(&outcome.recorded_run_json).unwrap();
        a["run_hash"] = serde_json::json!(0u64);
        assert!(
            !verify_run(&a.to_string()).unwrap(),
            "a tampered run hash fails verification"
        );

        // (b) tamper a recorded observation signal.
        let mut b: serde_json::Value = serde_json::from_str(&outcome.recorded_run_json).unwrap();
        b["frames"][0]["observations"][0]["signal_micros"] = serde_json::json!(999_999);
        assert!(
            !verify_run(&b.to_string()).unwrap(),
            "a tampered observation fails verification"
        );
    }

    #[test]
    fn malformed_or_wrong_schema_rejected() {
        assert!(run_scenario("not json").is_err());
        assert!(
            replay_run("{}").is_err(),
            "missing schema/fields is rejected"
        );
        let wrong = r#"{"schema":"nope","seed":0,"scheduler":{"horizon":1,"max_per_tick":1},"now":0,"run_ticks":0,"observations":[]}"#;
        assert!(
            run_scenario(wrong).is_err(),
            "an unexpected schema is rejected"
        );
    }
}
