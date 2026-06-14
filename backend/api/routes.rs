//! Local API route registry.

pub const ROUTES: &[&str] = &[
    "GET /health",
    "POST /input",
    "GET /packets",
    "GET /traces/:id",
    "GET /memory/:id",
    "GET /system-state",
    "POST /simulate/scenario",
];
