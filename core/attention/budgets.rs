//! Budget and backpressure models.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackpressureCommand {
    pub target_engine: String,
    pub mode: String,
    pub max_results: usize,
    pub preserve: Vec<String>,
    pub defer: Vec<String>,
}

impl BackpressureCommand {
    pub fn reduce_memory_output(max_results: usize) -> Self {
        Self {
            target_engine: "memory".to_string(),
            mode: "reduce_output".to_string(),
            max_results,
            preserve: vec!["high_confidence".to_string(), "high_relevance".to_string()],
            defer: vec!["background_consolidation".to_string()],
        }
    }
}
