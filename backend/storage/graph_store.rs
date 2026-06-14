//! Dependency graph storage models.

#[derive(Clone, Debug, PartialEq)]
pub struct MemoryDependencyTrace {
    pub memory_id: String,
    pub depends_on_rules: Vec<String>,
    pub source_episodes: Vec<String>,
    pub used_by_procedures: Vec<String>,
    pub used_by_plans: Vec<String>,
    pub memory_confidence: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CascadeEffect {
    pub memory_id: String,
    pub old_rule_id: String,
    pub new_rule_id: String,
    pub impact_score: f32,
    pub lazy_action: String,
}

pub fn impact_score(
    dependency_strength: f32,
    rule_change_distance: f32,
    usage_risk: f32,
    memory_confidence: f32,
    consequence_severity: f32,
) -> f32 {
    dependency_strength
        * rule_change_distance
        * usage_risk
        * memory_confidence
        * consequence_severity
}

pub fn lazy_action(score: f32, used: bool) -> &'static str {
    if !used {
        "deferred"
    } else if score >= 0.45 {
        "eager_revalidation"
    } else if score >= 0.22 {
        "confidence_reduced"
    } else {
        "pending_rederivation"
    }
}
