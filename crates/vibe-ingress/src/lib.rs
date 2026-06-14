//! vibe-ingress — ADR-002 layer-L1 admission control for Cognitive OS.
//!
//! Depends only on `vibe-core` value types. It admits, deduplicates,
//! sequence-checks, and stages or rejects external observations. It never
//! evaluates a tick and never touches `EngineState`; scheduling begins in P3.
//! See `ADR-002-runtime-engine-replay-contract.md`.

#![forbid(unsafe_code)]

mod gate;

pub use gate::{
    AcceptedObservationReceipt, Admission, EventId, IngressGate, ObservationEnvelope, RejectReason,
    RejectedObservationReceipt, SourceSession, StagedObservation,
};

#[cfg(test)]
mod tests {
    use super::*;
    use vibe_core::{EngineState, Scalar};

    // The ingress source — scanned by the boundary test below. The engine
    // tokens appear in THIS file only as test needles; `gate.rs` must stay free
    // of them so ingress structurally cannot evaluate or mutate engine state.
    const GATE_SRC: &str = include_str!("gate.rs");

    fn env(event_id: u64, source: &str, seq: u64, signal: i64) -> ObservationEnvelope {
        ObservationEnvelope {
            event_id: EventId(event_id),
            source: SourceSession {
                source: source.to_string(),
                session: 1,
            },
            source_sequence: seq,
            signal: Scalar::from_int(signal),
        }
    }

    #[test]
    fn valid_observation_accepted() {
        let mut gate = IngressGate::new();
        let outcome = gate.admit(env(1, "sensor-a", 0, 5));
        assert!(
            matches!(outcome, Admission::Accepted(_)),
            "a valid observation is accepted"
        );
        assert_eq!(
            gate.staged().len(),
            1,
            "an accepted observation is staged once"
        );
    }

    #[test]
    fn malformed_observation_rejected() {
        let mut gate = IngressGate::new();
        let outcome = gate.admit(env(1, "   ", 0, 5)); // blank source
        assert!(
            matches!(
                outcome,
                Admission::Rejected(RejectedObservationReceipt {
                    reason: RejectReason::EmptySource,
                    ..
                })
            ),
            "a malformed observation is rejected with a receipt"
        );
        assert_eq!(
            gate.staged().len(),
            0,
            "malformed input must not enter the staged set"
        );
    }

    #[test]
    fn duplicate_event_id_idempotent() {
        let mut gate = IngressGate::new();
        let first = gate.admit(env(7, "sensor-a", 0, 5));
        // same event_id, different sequence/payload — must be an idempotent no-op.
        let again = gate.admit(env(7, "sensor-a", 1, 9));
        assert!(matches!(first, Admission::Accepted(_)));
        assert!(
            matches!(again, Admission::Duplicate(_)),
            "a repeated EventId is a duplicate"
        );
        assert_eq!(
            gate.staged().len(),
            1,
            "a duplicate EventId must not create duplicate accepted work"
        );
    }

    #[test]
    fn source_sequence_gap_detected() {
        let mut gate = IngressGate::new();
        assert!(matches!(
            gate.admit(env(1, "sensor-a", 0, 1)),
            Admission::Accepted(_)
        ));
        // jump from 0 to 2 — sequence 1 is missing.
        let gap = gate.admit(env(2, "sensor-a", 2, 1));
        assert!(
            matches!(
                gap,
                Admission::Rejected(RejectedObservationReceipt {
                    reason: RejectReason::SequenceGap {
                        expected: 1,
                        got: 2
                    },
                    ..
                })
            ),
            "a sequence gap is detected and reported"
        );
        assert_eq!(
            gate.staged().len(),
            1,
            "a gapped observation must not be staged"
        );
    }

    #[test]
    fn rejected_observation_does_not_mutate_state() {
        // Ingress has no access to EngineState; prove a held engine state is
        // untouched by a rejected admit, and that nothing was staged.
        let state = EngineState::genesis(123);
        let snapshot = state.clone();
        let mut gate = IngressGate::new();
        let _ = gate.admit(env(1, "", 0, 5)); // malformed -> rejected
        assert_eq!(
            state, snapshot,
            "ingress must not touch the engine state the caller holds"
        );
        assert_eq!(
            gate.staged().len(),
            0,
            "a rejected observation stages nothing"
        );
    }

    #[test]
    fn ingress_does_not_call_evaluate_tick() {
        // Structural: the ingress source references neither the engine nor its
        // evaluation/mutation entry points, so it cannot advance or change state.
        for needle in ["evaluate_tick", "EngineState", "VibeEngine"] {
            assert!(
                !GATE_SRC.contains(needle),
                "ingress (gate.rs) must not reference `{needle}` — admission control only"
            );
        }
    }
}
