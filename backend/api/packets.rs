//! Packet API response model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacketsResponse {
    pub packets_json: Vec<String>,
}
