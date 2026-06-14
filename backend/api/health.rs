//! Health endpoint model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthResponse {
    pub ok: bool,
    pub service: String,
    pub schema_migrations: Vec<String>,
}
