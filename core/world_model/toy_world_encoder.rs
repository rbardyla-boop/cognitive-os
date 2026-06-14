//! Structured bridge-world encoder model.

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeState {
    pub status: String,
    pub rain_exposure: f32,
    pub damage_report: bool,
    pub base_minutes: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToyWorldState {
    pub location: String,
    pub destination: String,
    pub weather: String,
    pub time_budget_minutes: u32,
    pub bridge_a: BridgeState,
    pub bridge_b: BridgeState,
}
