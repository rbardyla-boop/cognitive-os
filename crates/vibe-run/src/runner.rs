//! ADR-002 layer L2 — deterministic record and replay.
//!
//! [`RunRecorder`] drives the full pipeline — ingress admits, the scheduler
//! orders, the collector canonicalizes frames, and the engine evaluates them —
//! and records the evidence (accepted observations, scheduled observations, the
//! per-tick frames and outputs, and a `run_hash`). [`ReplayRunner`] reproduces
//! the run by re-running ONLY the engine over the RECORDED frames: it never
//! re-admits or re-schedules live input, and it detects a tampered recording.
//!
//! Both DRIVE the one engine (`vibe_core::VibeEngine`); neither reimplements it.
//! See `ADR-002-runtime-engine-replay-contract.md`.

use std::collections::BTreeMap;

use vibe_core::{EngineOutput, EngineState, ObservationFrame, Tick, VibeEngine};
use vibe_frame::FrameCollector;
use vibe_ingress::{AcceptedObservationReceipt, Admission, IngressGate, ObservationEnvelope};
use vibe_scheduler::{
    ScheduleOutcome, ScheduleRequest, ScheduledObservation, SchedulerConfig, TickScheduler,
};

/// One scripted input: an observation envelope and the tick it targets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptedObservation {
    pub envelope: ObservationEnvelope,
    pub target_tick: Tick,
}

/// A deterministic run specification. The same script always records the same
/// run (and the same `run_hash`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunScript {
    pub seed: u64,
    pub scheduler: SchedulerConfig,
    /// The logical tick at which observations are admitted and scheduled.
    pub now: Tick,
    pub observations: Vec<ScriptedObservation>,
    /// Evaluate ticks `1..=run_ticks`.
    pub run_ticks: u64,
}

/// Per-tick recorded evidence: the canonical frame (carrying its own hash) and
/// the engine output (carrying the state transition and output hash).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedTick {
    pub tick: Tick,
    pub frame: ObservationFrame,
    pub output: EngineOutput,
}

/// A complete recorded run — sufficient to replay with NO live input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordedRun {
    pub seed: u64,
    pub accepted: Vec<AcceptedObservationReceipt>,
    pub scheduled: Vec<ScheduledObservation>,
    pub ticks: Vec<RecordedTick>,
    pub run_hash: u64,
}

/// FNV-1a mixing of one value (run-level hashing; not the engine's).
fn mix(mut h: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Deterministic hash over the whole run: seed, then each tick's frame hash and
/// output hash in order.
fn run_hash(seed: u64, ticks: &[RecordedTick]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h = mix(h, seed);
    h = mix(h, ticks.len() as u64);
    for rt in ticks {
        h = mix(h, rt.tick.0);
        h = mix(h, rt.frame.frame_hash());
        h = mix(h, rt.output.output_hash());
    }
    h
}

/// Records a deterministic run by driving the full L0/L1 pipeline.
#[derive(Clone, Copy, Debug, Default)]
pub struct RunRecorder;

impl RunRecorder {
    pub const fn new() -> Self {
        RunRecorder
    }

    /// Drive the pipeline for `script` and record the evidence. Engine state is
    /// advanced ONLY by the value `evaluate_tick` returns.
    pub fn record(&self, script: &RunScript) -> RecordedRun {
        let engine = VibeEngine::new();

        // 1. ingress: admit each envelope.
        let mut gate = IngressGate::new();
        let mut accepted = Vec::new();
        for scripted in &script.observations {
            if let Admission::Accepted(receipt) = gate.admit(scripted.envelope.clone()) {
                accepted.push(receipt);
            }
        }

        // 2. schedule each staged observation at its scripted target tick.
        let targets: BTreeMap<u64, Tick> = script
            .observations
            .iter()
            .map(|s| (s.envelope.event_id.0, s.target_tick))
            .collect();
        let mut scheduler = TickScheduler::new(script.scheduler);
        let mut scheduled = Vec::new();
        for staged in gate.staged() {
            let request = ScheduleRequest {
                observation: staged.clone(),
                target_tick: targets.get(&staged.event_id.0).copied(),
            };
            if let ScheduleOutcome::Scheduled(placed) = scheduler.schedule(script.now, request) {
                scheduled.push(placed);
            }
        }

        // 3. for each tick: collect the canonical frame and evaluate it.
        let collector = FrameCollector::new();
        let mut state = EngineState::genesis(script.seed);
        let mut ticks = Vec::new();
        for t in 1..=script.run_ticks {
            let tick = Tick(t);
            let frame = collector.collect(tick, &scheduled);
            let (output, next_state) = engine.evaluate_tick(&state, &frame);
            ticks.push(RecordedTick {
                tick,
                frame,
                output,
            });
            state = next_state;
        }

        let hash = run_hash(script.seed, &ticks);
        RecordedRun {
            seed: script.seed,
            accepted,
            scheduled,
            ticks,
            run_hash: hash,
        }
    }
}

/// The result of replaying a recorded run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayReport {
    /// The run hash recomputed during replay.
    pub run_hash: u64,
    /// Whether the recomputed run hash matches the recording.
    pub run_hash_matches: bool,
    /// The frames and outputs reconstructed by re-running the engine.
    pub reconstructed: Vec<RecordedTick>,
    /// Indices of ticks whose replayed output differs from the recording.
    pub output_mismatches: Vec<usize>,
    /// Indices of ticks whose recorded label disagrees with their own frame or
    /// recomputed output tick — an internally-inconsistent recording.
    pub tick_mismatches: Vec<usize>,
    /// True iff the run hash matches, every output matches, and every tick is
    /// internally consistent — the recording is reproducible and well-formed.
    pub verified: bool,
}

/// Replays a recorded run by re-running ONLY the engine over the recorded
/// frames. It verifies evidence; it does not become a second engine.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReplayRunner;

impl ReplayRunner {
    pub const fn new() -> Self {
        ReplayRunner
    }

    /// Reproduce `recorded` from its frames and seed alone — no live input, no
    /// re-admission, no re-scheduling. The recomputed outputs and run hash are
    /// compared against the recording; any divergence is reported.
    pub fn replay(&self, recorded: &RecordedRun) -> ReplayReport {
        let engine = VibeEngine::new();
        let mut state = EngineState::genesis(recorded.seed);
        let mut reconstructed = Vec::new();
        let mut output_mismatches = Vec::new();
        let mut tick_mismatches = Vec::new();

        for (index, rt) in recorded.ticks.iter().enumerate() {
            // Replay the RECORDED frame through the engine; do not rebuild it
            // from live input.
            let (output, next_state) = engine.evaluate_tick(&state, &rt.frame);
            if output != rt.output {
                output_mismatches.push(index);
            }
            // Internal consistency: the recorded tick label must agree with the
            // frame it carries and the recomputed output tick. This rejects a
            // relabelled or reordered recording even if its run hash was
            // recomputed to match (authenticity vs an active forger remains the
            // L3 signing concern — ADR-002).
            if rt.tick != rt.frame.tick() || rt.tick != output.tick {
                tick_mismatches.push(index);
            }
            reconstructed.push(RecordedTick {
                tick: rt.tick,
                frame: rt.frame.clone(),
                output,
            });
            state = next_state;
        }

        let replay_hash = run_hash(recorded.seed, &reconstructed);
        let run_hash_matches = replay_hash == recorded.run_hash;
        let verified =
            run_hash_matches && output_mismatches.is_empty() && tick_mismatches.is_empty();
        ReplayReport {
            run_hash: replay_hash,
            run_hash_matches,
            reconstructed,
            output_mismatches,
            tick_mismatches,
            verified,
        }
    }
}
