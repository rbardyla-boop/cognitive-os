//! P9 — codec rejection reasons.
//!
//! The codec NEVER silently repairs untrusted model output: every defect becomes
//! an explicit, typed rejection (no auto-fix, no fallback that fabricates an
//! action). Each variant maps to one `RejectKind` so the eval harness can assert
//! that a fixture is rejected for the *right* reason, not just rejected.

/// Why the codec rejected an untrusted model output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    /// The text is not a JSON array of action objects (free-form prose, broken
    /// JSON, or a non-object element). Prose is never treated as an action.
    MalformedSyntax(String),
    /// A known action is missing a required field (e.g. `read_span` with no
    /// `span_id`).
    MissingField { action: String, field: &'static str },
    /// A required field has the wrong JSON type (e.g. `span_id` is a string).
    MalformedField { action: String, field: &'static str },
    /// The `action` name is not one of the typed reading actions.
    UnknownAction(String),
    /// A referenced span id does not exist in the corpus — rejected BEFORE the
    /// substrate executes anything.
    UnknownSpan(u64),
    /// An `extract_claim` / `extract_entity` proposal cites no source span.
    UngroundedProposal,
    /// The substrate rejected the assembled trace (read-before-cite, unknown
    /// claim, etc.). The substrate — not the codec — is the executor of record.
    SubstrateRejected(String),
    /// The proposal sequence synthesized an answer, but the verifier did not
    /// approve it. The codec finalizes ONLY verifier-approved, source-grounded
    /// answers; an unverified answer (including injected text) is rejected.
    UnverifiedAnswer(Vec<String>),
}

/// A coarse classification of a rejection, used by the eval oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectKind {
    Malformed,
    MissingField,
    UnknownAction,
    UnknownSpan,
    Ungrounded,
    SubstrateRejected,
    Unverified,
}

impl CodecError {
    /// The rejection class — lets a test assert the *reason*, not just failure.
    pub fn kind(&self) -> RejectKind {
        match self {
            CodecError::MalformedSyntax(_) => RejectKind::Malformed,
            CodecError::MissingField { .. } => RejectKind::MissingField,
            CodecError::MalformedField { .. } => RejectKind::Malformed,
            CodecError::UnknownAction(_) => RejectKind::UnknownAction,
            CodecError::UnknownSpan(_) => RejectKind::UnknownSpan,
            CodecError::UngroundedProposal => RejectKind::Ungrounded,
            CodecError::SubstrateRejected(_) => RejectKind::SubstrateRejected,
            CodecError::UnverifiedAnswer(_) => RejectKind::Unverified,
        }
    }
}
