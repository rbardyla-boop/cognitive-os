//! Attention mode scheduler boundary.

use super::modes::SystemMode;

pub fn choose_mode(max_score: f32, queue_depth: usize, time_budget_minutes: u64) -> SystemMode {
    let mut pressure = max_score;
    if queue_depth >= 100 {
        pressure = pressure.max(4.0);
    }
    if queue_depth >= 500 {
        pressure = pressure.max(8.0);
    }
    if time_budget_minutes <= 3 {
        pressure = pressure.max(12.0);
    }

    if pressure >= 12.0 {
        SystemMode::Reflex
    } else if pressure >= 8.0 {
        SystemMode::Emergency
    } else if pressure >= 4.0 {
        SystemMode::Strained
    } else if pressure >= 0.30 {
        SystemMode::Operational
    } else {
        SystemMode::Reflective
    }
}
