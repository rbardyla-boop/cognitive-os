//! Packet admission scoring.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdmissionFactors {
    pub safety: f32,
    pub urgency: f32,
    pub goal_relevance: f32,
    pub expected_confidence_delta: f32,
    pub time_sensitivity: f32,
    pub compute_cost: f32,
    pub latency_cost: f32,
}

pub fn admission_score(factors: AdmissionFactors) -> f32 {
    let denominator = (factors.compute_cost * factors.latency_cost).max(0.01);
    factors.safety
        * factors.urgency
        * factors.goal_relevance
        * factors.expected_confidence_delta
        * factors.time_sensitivity
        / denominator
}
