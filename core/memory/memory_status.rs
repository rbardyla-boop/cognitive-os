//! Governed memory statuses.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryStatus {
    Active,
    ActiveWithSupersededDependency,
    ConfidenceReduced,
    PendingRederivation,
    Contradicted,
    ExceptionScoped,
    Quarantined,
    RetestRequired,
    Superseded,
    DeprecatedButPreserved,
}
