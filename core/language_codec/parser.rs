//! Deterministic language parser boundary.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedIntent {
    pub goal: String,
    pub target: Option<String>,
    pub raw_text: String,
}

pub fn parse_human_command(command: &str) -> ParsedIntent {
    let lower = command.to_lowercase();
    let target = if lower.contains("bridge a") {
        Some("bridge_A".to_string())
    } else if lower.contains("bridge b") {
        Some("bridge_B".to_string())
    } else {
        None
    };
    let goal = if lower.contains("cross") {
        "cross"
    } else if lower.contains("wait") {
        "wait"
    } else {
        "reach_destination"
    };
    ParsedIntent {
        goal: goal.to_string(),
        target,
        raw_text: command.to_string(),
    }
}
