//! Human explanation renderer boundary.

pub fn render_human_explanation(route: &str, mode: &str, confidence: f32, license: &str) -> String {
    format!(
        "Chose {route} using {mode} planning with {license} license at confidence {confidence}."
    )
}
