//! Optional LLM adapter boundary.

pub fn human_to_candidate_packet(_text: &str) -> Result<(), &'static str> {
    Err("LLM adapter is disabled in v0.1")
}

pub fn packet_state_to_human_explanation(_packet_state: &str) -> Result<String, &'static str> {
    Err("LLM adapter is disabled in v0.1")
}
