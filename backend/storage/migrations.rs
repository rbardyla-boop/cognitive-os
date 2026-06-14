//! Migration registry boundary.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Migration {
    pub version: String,
    pub sql: String,
}

pub const INITIAL_MIGRATION: &str = "001_initial_backend";
pub const PACKET_READ_COMPAT_MIGRATION: &str = "002_packet_read_compat";
