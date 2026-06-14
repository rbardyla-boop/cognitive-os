//! Essential CIP packet type registry.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PacketType {
    IntentPacket,
    ClaimPacket,
    EvidencePacket,
    EpisodePacket,
    RulePacket,
    RetrievalRequest,
    RetrievalResult,
    ContradictionPacket,
    PlanProposal,
    ActionCommand,
    ActionOutcome,
    MemoryMutation,
    SystemStatePacket,
    BackpressureCommand,
}

impl PacketType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IntentPacket => "IntentPacket",
            Self::ClaimPacket => "ClaimPacket",
            Self::EvidencePacket => "EvidencePacket",
            Self::EpisodePacket => "EpisodePacket",
            Self::RulePacket => "RulePacket",
            Self::RetrievalRequest => "RetrievalRequest",
            Self::RetrievalResult => "RetrievalResult",
            Self::ContradictionPacket => "ContradictionPacket",
            Self::PlanProposal => "PlanProposal",
            Self::ActionCommand => "ActionCommand",
            Self::ActionOutcome => "ActionOutcome",
            Self::MemoryMutation => "MemoryMutation",
            Self::SystemStatePacket => "SystemStatePacket",
            Self::BackpressureCommand => "BackpressureCommand",
        }
    }
}
