//! Revision pressure scoring.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RevisionPressureFactors {
    pub surprisal: f32,
    pub trust_episode: f32,
    pub reproducibility: f32,
    pub context_fit: f32,
    pub corroboration: f32,
    pub trust_rule: f32,
    pub known_exception_fit: f32,
    pub adversarial_risk: f32,
}

pub fn revision_pressure(factors: RevisionPressureFactors) -> f32 {
    let numerator = factors.surprisal
        * factors.trust_episode
        * factors.reproducibility
        * factors.context_fit
        * factors.corroboration;
    let denominator = factors.trust_rule.max(0.05)
        * factors.known_exception_fit.max(0.05)
        * factors.adversarial_risk.max(0.05);
    (numerator / denominator).min(1.0)
}
