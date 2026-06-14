//! Trace API response model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceResponse {
    pub trace_id: String,
    pub packets_json: Vec<String>,
}
