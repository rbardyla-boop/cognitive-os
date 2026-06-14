//! Append-only packet trace boundary.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent {
    pub trace_id: String,
    pub packet_id: String,
    pub packet_type: String,
    pub source_engine: String,
    pub target_engine: String,
    pub priority: String,
    pub created_at: String,
}
