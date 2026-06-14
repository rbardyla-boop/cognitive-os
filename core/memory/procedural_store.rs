//! Procedural memory model.

#[derive(Clone, Debug, PartialEq)]
pub struct Procedure {
    pub procedure_id: String,
    pub allowed_context: Vec<(String, String)>,
    pub steps: Vec<String>,
    pub confidence: f32,
    pub status: String,
}
