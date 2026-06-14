//! Toy prediction stub.

#[derive(Clone, Debug, PartialEq)]
pub struct Prediction {
    pub risk: f32,
    pub cost_minutes: u32,
    pub likely_outcome: String,
}

pub fn predict_bridge_crossing(
    rain_exposure: f32,
    damage_report: bool,
    status: &str,
    base_minutes: u32,
    weather: &str,
) -> Prediction {
    let weather_multiplier = match weather {
        "heavy_rain" => 0.7,
        "rain" => 0.45,
        _ => 0.1,
    };
    let mut risk = 0.1 + rain_exposure * weather_multiplier;
    if damage_report {
        risk += 0.25;
    }
    if matches!(status, "unknown" | "closed") {
        risk += 0.15;
    }
    risk = risk.min(1.0);
    Prediction {
        risk,
        cost_minutes: base_minutes,
        likely_outcome: if risk < 0.65 && status != "closed" {
            "arrived".to_string()
        } else {
            "deferred_at_bridge".to_string()
        },
    }
}
