//! ADR-002 layer L1 — deterministic tick scheduling (the TickScheduler).
//!
//! Admitted, staged observations (from ingress) are ordered onto FUTURE logical
//! ticks. The scheduler validates that a target tick is present, strictly in the
//! future, and within a bounded horizon; it rejects overload with a receipt and
//! is idempotent per `EventId`. It never evaluates a tick and never touches the
//! engine's state — frame collection is P4 and evaluation is P5. Time is the
//! logical [`vibe_core::Tick`]; there is no wall-clock anywhere.
//! See `ADR-002-runtime-engine-replay-contract.md`.

use std::collections::BTreeMap;
use vibe_core::{Scalar, Tick};
use vibe_ingress::{EventId, StagedObservation};

/// Scheduling bounds. `horizon` caps how far ahead a target tick may be;
/// `max_per_tick` is the per-tick capacity beyond which scheduling overloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SchedulerConfig {
    pub horizon: u64,
    pub max_per_tick: usize,
}

/// A request to schedule one staged observation at a target tick. The target is
/// REQUIRED: a `None` target is rejected (`MissingTargetTick`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleRequest {
    pub observation: StagedObservation,
    pub target_tick: Option<Tick>,
}

/// A staged observation placed on a concrete future tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledObservation {
    pub target_tick: Tick,
    pub event_id: EventId,
    pub signal: Scalar,
}

/// Why a schedule request was refused. Every refusal is reported, never a silent
/// drop (overload included).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleReason {
    /// No target tick was supplied.
    MissingTargetTick,
    /// The target tick is not strictly in the future.
    TargetTickInPast { now: u64, target: u64 },
    /// The target tick is beyond `now + horizon`.
    BeyondHorizon { now: u64, horizon: u64, target: u64 },
    /// The target tick is already at capacity.
    Overload { target: u64, capacity: usize },
}

/// A receipt for a refused schedule request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleReceipt {
    pub event_id: EventId,
    pub reason: ScheduleReason,
}

/// The outcome of one schedule request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScheduleOutcome {
    /// Newly placed on its target tick.
    Scheduled(ScheduledObservation),
    /// The same `EventId` was already scheduled — idempotent no-op returning the
    /// original placement (no duplicate work, even if the target differs).
    Duplicate(ScheduledObservation),
    /// Refused; nothing was scheduled.
    Rejected(ScheduleReceipt),
}

/// Deterministic tick scheduler. Holds only scheduling bookkeeping — the
/// per-tick ordered lanes and an idempotency index — never the engine's state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickScheduler {
    config: SchedulerConfig,
    lanes: BTreeMap<Tick, Vec<ScheduledObservation>>,
    scheduled: BTreeMap<EventId, ScheduledObservation>,
}

impl TickScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        TickScheduler {
            config,
            lanes: BTreeMap::new(),
            scheduled: BTreeMap::new(),
        }
    }

    /// Observations scheduled onto `tick`, in scheduling order.
    pub fn schedule_for(&self, tick: Tick) -> &[ScheduledObservation] {
        self.lanes.get(&tick).map_or(&[], Vec::as_slice)
    }

    /// Total number of distinct observations scheduled.
    pub fn scheduled_count(&self) -> usize {
        self.scheduled.len()
    }

    /// Schedule one request relative to the logical tick `now` (supplied by the
    /// caller — never read from a clock). Order: duplicate -> target required ->
    /// future -> horizon -> overload. Only a valid, in-window, non-duplicate,
    /// non-overloading request is placed; every other path returns a receipt and
    /// changes nothing.
    pub fn schedule(&mut self, now: Tick, request: ScheduleRequest) -> ScheduleOutcome {
        let event_id = request.observation.event_id;

        // duplicate -> idempotent no-op.
        if let Some(prior) = self.scheduled.get(&event_id) {
            return ScheduleOutcome::Duplicate(prior.clone());
        }

        // target required.
        let target = match request.target_tick {
            Some(t) => t,
            None => {
                return ScheduleOutcome::Rejected(ScheduleReceipt {
                    event_id,
                    reason: ScheduleReason::MissingTargetTick,
                });
            }
        };

        // strictly in the future.
        if target.0 <= now.0 {
            return ScheduleOutcome::Rejected(ScheduleReceipt {
                event_id,
                reason: ScheduleReason::TargetTickInPast {
                    now: now.0,
                    target: target.0,
                },
            });
        }

        // within the bounded horizon.
        if target.0 > now.0.saturating_add(self.config.horizon) {
            return ScheduleOutcome::Rejected(ScheduleReceipt {
                event_id,
                reason: ScheduleReason::BeyondHorizon {
                    now: now.0,
                    horizon: self.config.horizon,
                    target: target.0,
                },
            });
        }

        // overload -> receipt, not silent loss.
        let lane = self.lanes.entry(target).or_default();
        if lane.len() >= self.config.max_per_tick {
            return ScheduleOutcome::Rejected(ScheduleReceipt {
                event_id,
                reason: ScheduleReason::Overload {
                    target: target.0,
                    capacity: self.config.max_per_tick,
                },
            });
        }

        // place it.
        let placed = ScheduledObservation {
            target_tick: target,
            event_id,
            signal: request.observation.signal,
        };
        lane.push(placed.clone());
        self.scheduled.insert(event_id, placed.clone());
        ScheduleOutcome::Scheduled(placed)
    }
}
