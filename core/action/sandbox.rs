//! Action sandbox policy.

pub fn toy_action_only(action: &str) -> bool {
    matches!(
        action,
        "cross_bridge_A"
            | "cross_bridge_B"
            | "wait"
            | "request_more_evidence"
            | "take_safe_route"
            | "quarantine_memory"
    )
}
