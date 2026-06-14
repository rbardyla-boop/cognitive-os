//! Adjudication outcomes.

use super::conflict_detector::ConflictType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdjudicationOutcome {
    RejectEpisode,
    PreserveAsException,
    CandidateRuleRevision,
    ForkModelContext,
}

pub fn adjudicate(
    conflict_type: ConflictType,
    revision_pressure: f32,
    repeated_anomalies: usize,
) -> AdjudicationOutcome {
    match conflict_type {
        ConflictType::NoConflict | ConflictType::KnownException => {
            AdjudicationOutcome::PreserveAsException
        }
        ConflictType::ScopeMismatch => AdjudicationOutcome::ForkModelContext,
        ConflictType::UnknownAnomaly if repeated_anomalies < 3 => {
            AdjudicationOutcome::PreserveAsException
        }
        ConflictType::HardContradiction if revision_pressure < 0.45 => {
            AdjudicationOutcome::RejectEpisode
        }
        _ if repeated_anomalies >= 3 && revision_pressure >= 0.45 => {
            AdjudicationOutcome::CandidateRuleRevision
        }
        _ => AdjudicationOutcome::PreserveAsException,
    }
}
