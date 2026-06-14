//! Epistemic licenses for CIP packets.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EpistemicLicense {
    FullPremise,
    WeakPremise,
    HypothesisOnly,
    HazardOnly,
    DoNotUseForAction,
}

impl EpistemicLicense {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FullPremise => "full_premise",
            Self::WeakPremise => "weak_premise",
            Self::HypothesisOnly => "hypothesis_only",
            Self::HazardOnly => "hazard_only",
            Self::DoNotUseForAction => "do_not_use_for_action",
        }
    }

    pub fn permits_use(self, requested_use: &str) -> bool {
        match self {
            Self::FullPremise => true,
            Self::WeakPremise => matches!(
                requested_use,
                "retrieval"
                    | "planning_with_fallback"
                    | "human_explanation"
                    | "contradiction_detection"
                    | "sandbox_testing"
                    | "memory_consolidation"
            ),
            Self::HypothesisOnly => matches!(
                requested_use,
                "retrieval"
                    | "planning_with_fallback"
                    | "human_explanation"
                    | "contradiction_detection"
                    | "sandbox_testing"
            ),
            Self::HazardOnly => matches!(requested_use, "human_explanation" | "contradiction_detection"),
            Self::DoNotUseForAction => matches!(requested_use, "retrieval" | "human_explanation"),
        }
    }
}
