//! Toy plan proposal model.

#[derive(Clone, Debug, PartialEq)]
pub struct PlanProposal {
    pub goal: String,
    pub mode: String,
    pub route: String,
    pub action: String,
    pub fallback_plan: String,
    pub risk_note: String,
    pub required_assumptions: Vec<String>,
    pub risk_budget: f32,
}
