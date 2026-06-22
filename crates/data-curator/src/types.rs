//! Curation data model.
//!
//! A [`CandidateManifest`] is the ONLY input the curator reads: the caller
//! supplies explicit, already-materialized items — the curator performs no
//! filesystem access and reads no implicit blobs. Every output is a
//! classification recorded in a [`CurationReceipt`]; the curator creates no
//! truth, no memory, and no authority, and trains nothing.
//!
//! Receipts derive `Serialize` (so a later operator gate can emit them) but
//! deliberately NOT `Deserialize`: a receipt is re-derived from the manifest via
//! `curate`, never trusted from bytes.

use serde::Serialize;

/// What kind of artifact a candidate item is, parsed from the manifest's raw
/// `artifact_type` string. An unknown string parses to [`ArtifactKind::Unsupported`]
/// and is rejected. Each known kind carries an admissibility requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArtifactKind {
    /// Single-document verified span — durable claim-like data; needs grounding.
    DocumentSpan,
    /// Multi-document corpus span — durable claim-like data; needs grounding.
    CorpusSpan,
    /// Trace-derived record — needs a replay receipt.
    Trace,
    /// Dream-origin packet — hypothesis-only provenance; needs a grounding receipt.
    DreamPacket,
    /// Anything the curator does not recognize. Always rejected.
    Unsupported,
}

impl ArtifactKind {
    pub fn from_raw(raw: &str) -> ArtifactKind {
        match raw {
            "document_span" => ArtifactKind::DocumentSpan,
            "corpus_span" => ArtifactKind::CorpusSpan,
            "trace" => ArtifactKind::Trace,
            "dream_packet" => ArtifactKind::DreamPacket,
            _ => ArtifactKind::Unsupported,
        }
    }

    /// Durable, claim-like data must be grounded in source spans.
    pub fn requires_grounding(self) -> bool {
        matches!(
            self,
            ArtifactKind::DocumentSpan | ArtifactKind::CorpusSpan | ArtifactKind::DreamPacket
        )
    }

    /// Trace-derived data must carry a replay receipt.
    pub fn requires_replay_receipt(self) -> bool {
        matches!(self, ArtifactKind::Trace)
    }

    /// Stable tag used when feeding the kind into a hash.
    pub fn tag(self) -> &'static str {
        match self {
            ArtifactKind::DocumentSpan => "document_span",
            ArtifactKind::CorpusSpan => "corpus_span",
            ArtifactKind::Trace => "trace",
            ArtifactKind::DreamPacket => "dream_packet",
            ArtifactKind::Unsupported => "unsupported",
        }
    }
}

/// Which split an item is assigned to. Train/holdout leakage = the same content
/// hash present in BOTH splits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Split {
    Train,
    Holdout,
}

impl Split {
    pub fn from_raw(raw: &str) -> Option<Split> {
        match raw {
            "train" => Some(Split::Train),
            "holdout" => Some(Split::Holdout),
            _ => None,
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::Holdout => "holdout",
        }
    }
}

/// A single candidate record supplied by the caller. The curator treats every
/// field as untrusted input and never mutates it. Empty string ⇒ absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateItem {
    pub id: String,
    pub artifact_type: String,
    pub content: String,
    pub provenance: String,
    pub grounding_ref: String,
    pub replay_receipt_ref: String,
    pub split: String,
}

impl CandidateItem {
    /// A bare candidate: known artifact type + content, defaulting to the `train`
    /// split with no provenance/grounding/replay yet. Chain the `with_*` setters
    /// to fill in admissibility fields.
    pub fn new(
        id: impl Into<String>,
        artifact_type: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            artifact_type: artifact_type.into(),
            content: content.into(),
            provenance: String::new(),
            grounding_ref: String::new(),
            replay_receipt_ref: String::new(),
            split: "train".to_string(),
        }
    }

    pub fn with_provenance(mut self, p: impl Into<String>) -> Self {
        self.provenance = p.into();
        self
    }

    pub fn with_grounding(mut self, g: impl Into<String>) -> Self {
        self.grounding_ref = g.into();
        self
    }

    pub fn with_replay_receipt(mut self, r: impl Into<String>) -> Self {
        self.replay_receipt_ref = r.into();
        self
    }

    pub fn with_split(mut self, s: impl Into<String>) -> Self {
        self.split = s.into();
        self
    }
}

/// The complete curation input: a dataset id and its candidate items. This is
/// the ONLY thing `curate` reads — no filesystem, no ambient state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateManifest {
    pub dataset_id: String,
    pub items: Vec<CandidateItem>,
}

impl CandidateManifest {
    pub fn new(dataset_id: impl Into<String>, items: Vec<CandidateItem>) -> Self {
        Self {
            dataset_id: dataset_id.into(),
            items,
        }
    }
}

/// Why a candidate item was rejected (removed from the admitted set; the source
/// content is never altered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RejectReason {
    MissingProvenance,
    DuplicateId,
    EmptyContent,
    UnsupportedArtifact,
    MissingGrounding,
    MissingReplayReceipt,
    InvalidSplit,
}

/// Why a candidate item was quarantined. Quarantine RETAINS the item (never
/// deletes it) and never admits it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuarantineReason {
    /// Content tripped a prompt-injection marker.
    PromptInjection,
    /// Content hash appeared in both the train and holdout split.
    SplitLeakage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdmittedItem {
    pub id: String,
    pub content_hash: String,
    pub kind: ArtifactKind,
    pub split: Split,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RejectedItem {
    pub id: String,
    pub content_hash: String,
    pub reason: RejectReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuarantinedItem {
    pub id: String,
    pub content_hash: String,
    pub reason: QuarantineReason,
    /// For `PromptInjection`, the matched marker; for `SplitLeakage`, empty.
    pub detail: String,
}

/// The split assignment of the ADMITTED set only (leaked/quarantined excluded).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SplitPlan {
    pub train_ids: Vec<String>,
    pub holdout_ids: Vec<String>,
}

/// Train/holdout contamination findings. Clean ⇒ no leakage and no duplicate ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContaminationChecks {
    /// Content hashes found in BOTH train and holdout (leakage).
    pub leaked_content_hashes: Vec<String>,
    /// Ids that appeared more than once in the manifest.
    pub duplicate_ids: Vec<String>,
}

impl ContaminationChecks {
    pub fn is_clean(&self) -> bool {
        self.leaked_content_hashes.is_empty() && self.duplicate_ids.is_empty()
    }
}

/// Prompt-injection findings. Clean ⇒ no markers tripped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PoisoningChecks {
    /// Ids quarantined for prompt-injection markers.
    pub injected_ids: Vec<String>,
}

impl PoisoningChecks {
    pub fn is_clean(&self) -> bool {
        self.injected_ids.is_empty()
    }
}

/// Which durable items failed the source-span grounding requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GroundingRequirements {
    pub ungrounded_rejected_ids: Vec<String>,
}

/// Which trace-derived items failed the replay-receipt requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplayRequirements {
    pub missing_replay_rejected_ids: Vec<String>,
}

/// Invariants asserting the curator stayed inside its boundary. Every field is
/// `false` by construction — the curator has no code that could set any true.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct BoundaryChecks {
    pub created_authority: bool,
    pub created_evidence: bool,
    pub promoted_anything: bool,
    pub executed_anything: bool,
    pub ingested_into_memory: bool,
}

impl BoundaryChecks {
    /// The only constructor: all invariants held (everything inert).
    pub fn inert() -> Self {
        Self {
            created_authority: false,
            created_evidence: false,
            promoted_anything: false,
            executed_anything: false,
            ingested_into_memory: false,
        }
    }

    pub fn all_inert(&self) -> bool {
        !self.created_authority
            && !self.created_evidence
            && !self.promoted_anything
            && !self.executed_anything
            && !self.ingested_into_memory
    }
}

/// Whether this dataset may be used to train weights.
///
/// DATA-0 has NO state that permits training: the only inhabitable values are
/// `Closed` and `CandidateOnly`, and BOTH report `is_eligible() == false`.
/// Opening training is the job of a LATER gate that does not exist yet; this
/// enum carries no training-permitting value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum TrainingEligibility {
    /// Default: training is not permitted from this dataset.
    #[default]
    Closed,
    /// Data is structurally admissible, but training remains gated by a later,
    /// not-yet-implemented gate. Still NOT permitted.
    CandidateOnly,
}

/// DATA-0 carries no training-permitting state. This const is the single source
/// of truth for [`TrainingEligibility::is_eligible`]; the release gate pins it to
/// `false`, so flipping it is caught both statically and by the
/// `training_eligibility_is_never_eligible` test.
const TRAINING_PERMITTED: bool = false;

impl TrainingEligibility {
    /// ALWAYS false in DATA-0. No code path returns true.
    pub fn is_eligible(self) -> bool {
        TRAINING_PERMITTED
    }
}

/// The deterministic, auditable output of a curation run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CurationReceipt {
    pub dataset_id: String,
    /// Order-independent digest of the admitted set (canonical, replay-stable).
    pub dataset_hash: String,
    /// Order-sensitive digest binding the exact input manifest bytes.
    pub source_manifest_hash: String,
    pub admitted_items: Vec<AdmittedItem>,
    pub rejected_items: Vec<RejectedItem>,
    pub quarantined_items: Vec<QuarantinedItem>,
    pub split_plan: SplitPlan,
    pub contamination_checks: ContaminationChecks,
    pub poisoning_checks: PoisoningChecks,
    pub grounding_requirements: GroundingRequirements,
    pub replay_requirements: ReplayRequirements,
    pub authority_boundary_checks: BoundaryChecks,
    pub training_eligibility: TrainingEligibility,
}
