//! ADR-002 layer L1 — admission control (the IngressGate).
//!
//! All external input enters as an [`ObservationEnvelope`] and is admitted,
//! deduplicated, sequence-checked, and either STAGED or REJECTED with a
//! receipt. Ingress never evaluates a tick and never touches the engine's
//! state; scheduling begins in P3. The only vibe-core type it uses is the
//! value type [`vibe_core::Scalar`]. See `ADR-002-runtime-engine-replay-contract.md`.

use std::collections::BTreeMap;
use vibe_core::Scalar;

/// Idempotency key for an observation — globally unique per logical event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(pub u64);

/// The source identity (which producer) and its session.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceSession {
    pub source: String,
    pub session: u64,
}

/// Raw external input. It may be malformed; the [`IngressGate`] decides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationEnvelope {
    pub event_id: EventId,
    pub source: SourceSession,
    /// Per-source monotonic sequence number, used for gap detection.
    pub source_sequence: u64,
    /// Observation payload (a vibe-core value type).
    pub signal: Scalar,
}

/// Proof that an observation was admitted and staged (or already staged).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedObservationReceipt {
    pub event_id: EventId,
    pub source: SourceSession,
    pub source_sequence: u64,
}

/// Why an observation was refused. Gaps and malformed input are DETECTED and
/// reported in a receipt, never silently dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RejectReason {
    /// The source identity was empty or blank.
    EmptySource,
    /// A later sequence number arrived before its predecessor.
    SequenceGap { expected: u64, got: u64 },
    /// An already-superseded sequence number arrived (replay/out-of-order).
    StaleSequence { expected: u64, got: u64 },
}

/// A receipt for a refused observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RejectedObservationReceipt {
    pub event_id: EventId,
    pub reason: RejectReason,
}

/// The outcome of submitting one observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Admission {
    /// Newly admitted and staged.
    Accepted(AcceptedObservationReceipt),
    /// The same `EventId` was already admitted — an idempotent no-op that
    /// returns the original receipt and creates no duplicate work.
    Duplicate(AcceptedObservationReceipt),
    /// Refused; nothing was staged.
    Rejected(RejectedObservationReceipt),
}

/// A staged observation awaiting the scheduler (P3). Staging is admission
/// bookkeeping only — it is NOT engine state and carries no tick assignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StagedObservation {
    pub event_id: EventId,
    pub source: SourceSession,
    pub source_sequence: u64,
    pub signal: Scalar,
}

/// Admission control. Holds only ingress bookkeeping — accepted receipts (for
/// idempotency), the per-source sequence cursor, and the staged queue. It never
/// holds or mutates engine state.
#[derive(Clone, Debug, Default)]
pub struct IngressGate {
    accepted: BTreeMap<EventId, AcceptedObservationReceipt>,
    last_sequence: BTreeMap<String, u64>,
    staged: Vec<StagedObservation>,
}

impl IngressGate {
    pub fn new() -> Self {
        IngressGate::default()
    }

    /// Observations staged so far, in admission order.
    pub fn staged(&self) -> &[StagedObservation] {
        &self.staged
    }

    /// Admit one observation. Validation order: malformed -> duplicate ->
    /// sequence. Only a fully-valid, in-order, non-duplicate observation is
    /// staged. Every other path returns a receipt and stages nothing, so
    /// malformed input never partially enters the staged set.
    pub fn admit(&mut self, env: ObservationEnvelope) -> Admission {
        // 1. malformed?
        if env.source.source.trim().is_empty() {
            return Admission::Rejected(RejectedObservationReceipt {
                event_id: env.event_id,
                reason: RejectReason::EmptySource,
            });
        }

        // 2. duplicate event id -> idempotent; return the original receipt.
        if let Some(prior) = self.accepted.get(&env.event_id) {
            return Admission::Duplicate(prior.clone());
        }

        // 3. per-source sequence ordering.
        let expected = self
            .last_sequence
            .get(&env.source.source)
            .map_or(0, |s| s + 1);
        if env.source_sequence > expected {
            return Admission::Rejected(RejectedObservationReceipt {
                event_id: env.event_id,
                reason: RejectReason::SequenceGap {
                    expected,
                    got: env.source_sequence,
                },
            });
        }
        if env.source_sequence < expected {
            return Admission::Rejected(RejectedObservationReceipt {
                event_id: env.event_id,
                reason: RejectReason::StaleSequence {
                    expected,
                    got: env.source_sequence,
                },
            });
        }

        // accept + stage.
        let receipt = AcceptedObservationReceipt {
            event_id: env.event_id,
            source: env.source.clone(),
            source_sequence: env.source_sequence,
        };
        self.accepted.insert(env.event_id, receipt.clone());
        self.last_sequence
            .insert(env.source.source.clone(), env.source_sequence);
        self.staged.push(StagedObservation {
            event_id: env.event_id,
            source: env.source,
            source_sequence: env.source_sequence,
            signal: env.signal,
        });
        Admission::Accepted(receipt)
    }
}
