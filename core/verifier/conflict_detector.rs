//! Conflict detection.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictType {
    NoConflict,
    SoftConflict,
    HardContradiction,
    ScopeMismatch,
    KnownException,
    UnknownAnomaly,
}
