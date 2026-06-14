//! SQLite storage boundary for v0.1.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredPacket {
    pub packet_id: String,
    pub trace_id: String,
    pub packet_type: String,
    pub schema_version: String,
    pub source_engine: String,
    pub target_engine: String,
    pub created_at: String,
    pub priority: String,
    pub raw_json: String,
}

pub const REQUIRED_TABLES: &[&str] = &[
    "packets",
    "episodes",
    "memory_nodes",
    "rules",
    "procedures",
    "contradictions",
    "traces",
    "deferred_jobs",
    "system_events",
];
