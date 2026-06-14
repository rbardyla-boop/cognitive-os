//! Fallback planning model.

pub fn fallback_for_license(epistemic_license: &str) -> &'static str {
    match epistemic_license {
        "hypothesis_only" | "hazard_only" => "request_more_evidence",
        "do_not_use_for_action" => "wait",
        _ => "take_safe_route",
    }
}
