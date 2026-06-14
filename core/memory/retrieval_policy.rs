//! Status-aware retrieval policy model.

#[derive(Clone, Debug, PartialEq)]
pub struct RetrievedMemory<T> {
    pub content: T,
    pub confidence: f32,
    pub status: String,
    pub epistemic_license: String,
    pub source_episodes: Vec<String>,
    pub contradictions: Vec<String>,
    pub allowed_use: Vec<String>,
    pub forbidden_use: Vec<String>,
    pub revalidation_requirement: String,
}

pub fn emergency_use_protocol(epistemic_license: &str, urgent: bool) -> &'static str {
    match (epistemic_license, urgent) {
        ("full_premise", _) => "normal_use",
        ("weak_premise", true) => "use_with_fallback",
        ("weak_premise", false) => "normal_use_with_fallback_available",
        ("hypothesis_only", _) => "branch_alternatives",
        ("hazard_only", _) => "warning_only",
        ("do_not_use_for_action", _) => "cannot_support_action",
        _ => "cannot_support_action",
    }
}

pub fn needs_post_action_revalidation(requirement: &str) -> bool {
    requirement == "post_action_revalidation"
}
