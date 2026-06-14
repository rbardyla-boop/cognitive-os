//! Epistemic gate decision model.

#[derive(Clone, Debug, PartialEq)]
pub struct VerifierDecision {
    pub confidence: f32,
    pub epistemic_license: String,
    pub caveats: Vec<String>,
    pub contradictions: Vec<String>,
}
