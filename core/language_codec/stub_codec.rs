//! Deterministic stub codec.

pub const CODEC_ID: &str = "stub_deterministic_v0.1";

pub fn target_to_bridge(target: Option<&str>) -> Option<&'static str> {
    match target {
        Some("bridge_A") => Some("Bridge A"),
        Some("bridge_B") => Some("Bridge B"),
        _ => None,
    }
}
