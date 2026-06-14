//! Contradiction index model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContradictionLink {
    pub memory_id: String,
    pub contradicts: Vec<String>,
}
