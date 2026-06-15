//! P9 — the codec policy: which boundary guards are active.
//!
//! Production code can ONLY build the strict policy (every guard on). The
//! weakened constructors are `#[cfg(test)]`, so no production path can disable a
//! guard — yet the sabotage probe tests can flip one flag to prove that guard is
//! load-bearing (disabling it makes the eval harness fail).

/// Which codec guards are enforced. All three are on under `strict()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodecPolicy {
    /// Reject actions whose name is not a known typed reading action. When off,
    /// unknown actions are silently dropped (a silent-repair sabotage).
    pub(crate) reject_unknown: bool,
    /// Reject `extract_claim` / `extract_entity` proposals with no source span.
    pub(crate) require_source_spans: bool,
    /// Finalize a synthesized answer ONLY if the verifier approves it.
    pub(crate) require_verified_finalize: bool,
}

impl CodecPolicy {
    /// The only production policy: every boundary guard enforced.
    pub fn strict() -> Self {
        CodecPolicy {
            reject_unknown: true,
            require_source_spans: true,
            require_verified_finalize: true,
        }
    }
}

impl Default for CodecPolicy {
    fn default() -> Self {
        CodecPolicy::strict()
    }
}

#[cfg(test)]
impl CodecPolicy {
    /// Sabotage 1: stop rejecting unknown actions (they get silently dropped).
    pub(crate) fn without_unknown_rejection(self) -> Self {
        CodecPolicy {
            reject_unknown: false,
            ..self
        }
    }

    /// Sabotage 2: stop requiring source spans on extracted claims/entities.
    pub(crate) fn without_source_span_requirement(self) -> Self {
        CodecPolicy {
            require_source_spans: false,
            ..self
        }
    }

    /// Sabotage 3: finalize answers without the verifier's approval.
    pub(crate) fn without_verified_finalize(self) -> Self {
        CodecPolicy {
            require_verified_finalize: false,
            ..self
        }
    }
}
