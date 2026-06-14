//! Sandboxed toy action executor model.

#[derive(Clone, Debug, PartialEq)]
pub struct ActionOutcome {
    pub action: String,
    pub success: bool,
    pub observed_state: String,
    pub message: String,
}
