//! Toy minimax planner.

#[derive(Clone, Debug, PartialEq)]
pub struct RouteRisk {
    pub route: String,
    pub worst_case_loss: f32,
}

pub fn least_catastrophic(routes: &[RouteRisk]) -> Option<&RouteRisk> {
    routes
        .iter()
        .min_by(|left, right| left.worst_case_loss.total_cmp(&right.worst_case_loss))
}
