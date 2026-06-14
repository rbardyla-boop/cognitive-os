//! Memory API response model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryResponse {
    pub memory_id: String,
    pub raw_json: String,
}
