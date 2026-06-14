//! Append-only packet log boundary.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendOnlyRecord {
    pub record_id: String,
    pub schema_version: String,
    pub raw_json: String,
}
