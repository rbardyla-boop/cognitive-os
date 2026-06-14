//! Toy action policy.

pub const ALLOWED_TOY_ACTIONS: &[&str] = &[
    "cross_bridge_A",
    "cross_bridge_B",
    "wait",
    "request_more_evidence",
    "take_safe_route",
    "quarantine_memory",
];

pub fn is_allowed_toy_action(action: &str) -> bool {
    ALLOWED_TOY_ACTIONS.contains(&action)
}
