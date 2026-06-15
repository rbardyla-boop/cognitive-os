//! READ-0 — structured evidence memory.
//!
//! Memory is structured claims/entities/proof, NOT vague summaries. The
//! load-bearing rule (enforced by the trace executor): no claim or entity enters
//! memory without ≥1 source span. The source spans ARE the evidence links.

use crate::corpus::SpanId;

/// A claim extracted from source. It MUST cite at least one source span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claim {
    pub id: u64,
    pub statement: String,
    /// Evidence links: the source spans that support this claim (never empty).
    pub source_spans: Vec<SpanId>,
}

/// A named entity with the spans that mention it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity {
    pub id: u64,
    pub name: String,
    pub source_spans: Vec<SpanId>,
}

/// The synthesized answer: the ordered supporting claim ids and the rendered
/// answer text (which must be exactly those claims' statements).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofObject {
    pub question: String,
    pub answer_text: String,
    pub supporting_claims: Vec<u64>,
}

/// Structured reading memory.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Memory {
    pub claims: Vec<Claim>,
    pub entities: Vec<Entity>,
}

impl Memory {
    pub fn new() -> Self {
        Memory::default()
    }

    /// Look up a claim by id.
    pub fn claim(&self, id: u64) -> Option<&Claim> {
        self.claims.iter().find(|c| c.id == id)
    }
}
