//! Semantic memory graph model.

use super::memory_status::MemoryStatus;

#[derive(Clone, Debug, PartialEq)]
pub struct SemanticMemoryNode {
    pub memory_id: String,
    pub claim: String,
    pub confidence: f32,
    pub status: MemoryStatus,
    pub source_episodes: Vec<String>,
    pub depends_on_rules: Vec<String>,
    pub used_by_procedures: Vec<String>,
    pub used_by_plans: Vec<String>,
    pub contradictions: Vec<String>,
    pub created_by: String,
    pub updated_by: String,
    pub schema_version: String,
}
