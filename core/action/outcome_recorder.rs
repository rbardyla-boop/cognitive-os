//! Action outcome recording model.

#[derive(Clone, Debug, PartialEq)]
pub struct RecordedOutcome {
    pub action_outcome_packet_id: String,
    pub episode_packet_id: String,
    pub memory_mutation_packet_id: String,
    pub trace_id: String,
}
