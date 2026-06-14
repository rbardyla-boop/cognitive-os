//! System mode definitions.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemMode {
    Reflective,
    Operational,
    Strained,
    Emergency,
    Reflex,
    Recovery,
}

impl SystemMode {
    pub fn behavior(self) -> &'static str {
        match self {
            Self::Reflective => "deep_reasoning_allowed",
            Self::Operational => "normal_planning",
            Self::Strained => "defer_consolidation",
            Self::Emergency => "minimax_safety_only",
            Self::Reflex => "precompiled_policy_only",
            Self::Recovery => "replay_deferred_packets",
        }
    }
}
