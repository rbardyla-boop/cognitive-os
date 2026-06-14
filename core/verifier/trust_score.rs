//! Trust scoring.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrustFactors {
    pub source_reliability: f32,
    pub timestamp_integrity: f32,
    pub corroboration: f32,
    pub parse_confidence: f32,
    pub sensor_confidence: f32,
    pub adversarial_risk: f32,
    pub recency: f32,
    pub dependency_stability: f32,
}

pub fn trust_score(factors: TrustFactors) -> f32 {
    let positive = factors.source_reliability
        * factors.timestamp_integrity
        * factors.corroboration
        * factors.parse_confidence
        * factors.sensor_confidence
        * factors.recency
        * factors.dependency_stability;
    (positive / factors.adversarial_risk.max(0.05)).min(1.0)
}
