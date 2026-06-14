//! CIP envelope model.

use super::epistemic_license::EpistemicLicense;
use super::packet_types::PacketType;
use super::permissions::PacketPermissions;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketHeader {
    pub packet_id: String,
    pub packet_type: PacketType,
    pub schema_version: String,
    pub source_engine: String,
    pub target_engine: String,
    pub trace_id: String,
    pub created_at: String,
    pub priority: String,
    pub time_budget_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PacketEpistemics {
    pub confidence: f32,
    pub uncertainty_type: String,
    pub epistemic_license: EpistemicLicense,
    pub provenance: Vec<String>,
    pub contradictions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CipEnvelope<T> {
    pub header: PacketHeader,
    pub epistemics: PacketEpistemics,
    pub permissions: PacketPermissions,
    pub payload: T,
}

impl<T> CipEnvelope<T> {
    pub fn can_use_payload_for(&self, requested_use: &str) -> bool {
        self.permissions
            .permits(self.epistemics.epistemic_license, requested_use)
    }
}
