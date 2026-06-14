//! Memory provenance model.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceRef {
    pub source_kind: String,
    pub source_id: String,
}
