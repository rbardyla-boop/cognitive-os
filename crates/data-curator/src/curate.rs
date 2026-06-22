//! The curation pipeline: `curate(&CandidateManifest) -> CurationReceipt`.
//!
//! Pure and deterministic — BTree ordering, FNV-1a hashing, no clock, no
//! entropy, no float, no filesystem. The input manifest is borrowed and never
//! mutated; the receipt is freshly owned.
//!
//! Per-item disposition is evaluated in a fixed precedence (first match wins):
//! duplicate id → empty content → unsupported artifact → missing provenance →
//! missing grounding → missing replay receipt → invalid split → prompt-injection
//! quarantine → provisional admit. After all items are classified, train/holdout
//! leakage is detected over the provisional admits and any leaked item is moved
//! to quarantine (never deleted).

use std::collections::{BTreeMap, BTreeSet};

use crate::hash::{content_hash, Fnv1a};
use crate::inject::first_injection_marker;
use crate::types::*;

/// Classify a candidate manifest into a deterministic [`CurationReceipt`].
pub fn curate(manifest: &CandidateManifest) -> CurationReceipt {
    let source_manifest_hash = hash_manifest(manifest);

    let mut admitted: Vec<AdmittedItem> = Vec::new();
    let mut rejected: Vec<RejectedItem> = Vec::new();
    let mut quarantined: Vec<QuarantinedItem> = Vec::new();

    let mut seen_ids: BTreeSet<String> = BTreeSet::new();
    let mut duplicate_ids: Vec<String> = Vec::new();
    let mut ungrounded_rejected_ids: Vec<String> = Vec::new();
    let mut missing_replay_rejected_ids: Vec<String> = Vec::new();
    let mut injected_ids: Vec<String> = Vec::new();

    for item in &manifest.items {
        let ch = content_hash(&item.content);

        if seen_ids.contains(&item.id) {
            duplicate_ids.push(item.id.clone());
            rejected.push(reject(&item.id, &ch, RejectReason::DuplicateId));
            continue;
        }
        seen_ids.insert(item.id.clone());

        if item.content.trim().is_empty() {
            rejected.push(reject(&item.id, &ch, RejectReason::EmptyContent));
            continue;
        }

        let kind = ArtifactKind::from_raw(&item.artifact_type);
        if kind == ArtifactKind::Unsupported {
            rejected.push(reject(&item.id, &ch, RejectReason::UnsupportedArtifact));
            continue;
        }

        if item.provenance.trim().is_empty() {
            rejected.push(reject(&item.id, &ch, RejectReason::MissingProvenance));
            continue;
        }

        if kind.requires_grounding() && item.grounding_ref.trim().is_empty() {
            ungrounded_rejected_ids.push(item.id.clone());
            rejected.push(reject(&item.id, &ch, RejectReason::MissingGrounding));
            continue;
        }

        if kind.requires_replay_receipt() && item.replay_receipt_ref.trim().is_empty() {
            missing_replay_rejected_ids.push(item.id.clone());
            rejected.push(reject(&item.id, &ch, RejectReason::MissingReplayReceipt));
            continue;
        }

        let split = match Split::from_raw(&item.split) {
            Some(s) => s,
            None => {
                rejected.push(reject(&item.id, &ch, RejectReason::InvalidSplit));
                continue;
            }
        };

        if let Some(marker) = first_injection_marker(&item.content) {
            injected_ids.push(item.id.clone());
            quarantined.push(QuarantinedItem {
                id: item.id.clone(),
                content_hash: ch,
                reason: QuarantineReason::PromptInjection,
                detail: marker.to_string(),
            });
            continue;
        }

        admitted.push(AdmittedItem {
            id: item.id.clone(),
            content_hash: ch,
            kind,
            split,
        });
    }

    // Train/holdout leakage: a content hash present in BOTH splits among the
    // provisional admits. Leaked items are quarantined (retained), never deleted.
    let mut splits_by_hash: BTreeMap<String, BTreeSet<Split>> = BTreeMap::new();
    for a in &admitted {
        splits_by_hash
            .entry(a.content_hash.clone())
            .or_default()
            .insert(a.split);
    }
    let leaked: BTreeSet<String> = splits_by_hash
        .iter()
        .filter(|(_, splits)| splits.contains(&Split::Train) && splits.contains(&Split::Holdout))
        .map(|(h, _)| h.clone())
        .collect();

    if !leaked.is_empty() {
        let (keep, move_out): (Vec<AdmittedItem>, Vec<AdmittedItem>) = admitted
            .into_iter()
            .partition(|a| !leaked.contains(&a.content_hash));
        admitted = keep;
        for a in move_out {
            quarantined.push(QuarantinedItem {
                id: a.id,
                content_hash: a.content_hash,
                reason: QuarantineReason::SplitLeakage,
                detail: String::new(),
            });
        }
    }

    // Canonical ordering: the receipt is independent of input ordering.
    admitted.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| a.content_hash.cmp(&b.content_hash))
    });
    rejected.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| a.content_hash.cmp(&b.content_hash))
    });
    quarantined.sort_by(|a, b| {
        a.id.cmp(&b.id)
            .then_with(|| a.content_hash.cmp(&b.content_hash))
    });

    let split_plan = SplitPlan {
        train_ids: admitted
            .iter()
            .filter(|a| a.split == Split::Train)
            .map(|a| a.id.clone())
            .collect(),
        holdout_ids: admitted
            .iter()
            .filter(|a| a.split == Split::Holdout)
            .map(|a| a.id.clone())
            .collect(),
    };

    let contamination_checks = ContaminationChecks {
        leaked_content_hashes: leaked.into_iter().collect(), // BTreeSet → sorted
        duplicate_ids: sorted_unique(duplicate_ids),
    };
    let poisoning_checks = PoisoningChecks {
        injected_ids: sorted_unique(injected_ids),
    };
    let grounding_requirements = GroundingRequirements {
        ungrounded_rejected_ids: sorted_unique(ungrounded_rejected_ids),
    };
    let replay_requirements = ReplayRequirements {
        missing_replay_rejected_ids: sorted_unique(missing_replay_rejected_ids),
    };
    let authority_boundary_checks = BoundaryChecks::inert();

    // CandidateOnly only when fully clean with at least one admitted item; still
    // NOT eligible for training (no value of this enum permits training).
    let training_eligibility =
        if contamination_checks.is_clean() && poisoning_checks.is_clean() && !admitted.is_empty() {
            TrainingEligibility::CandidateOnly
        } else {
            TrainingEligibility::Closed
        };

    let dataset_hash = hash_admitted(&manifest.dataset_id, &admitted);

    CurationReceipt {
        dataset_id: manifest.dataset_id.clone(),
        dataset_hash,
        source_manifest_hash,
        admitted_items: admitted,
        rejected_items: rejected,
        quarantined_items: quarantined,
        split_plan,
        contamination_checks,
        poisoning_checks,
        grounding_requirements,
        replay_requirements,
        authority_boundary_checks,
        training_eligibility,
    }
}

fn reject(id: &str, content_hash: &str, reason: RejectReason) -> RejectedItem {
    RejectedItem {
        id: id.to_string(),
        content_hash: content_hash.to_string(),
        reason,
    }
}

fn sorted_unique(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v.dedup();
    v
}

/// Order-SENSITIVE digest binding the exact input manifest bytes.
fn hash_manifest(m: &CandidateManifest) -> String {
    let mut h = Fnv1a::new();
    h.feed_str(&m.dataset_id);
    h.feed_u64(m.items.len() as u64);
    for it in &m.items {
        h.feed_str(&it.id);
        h.feed_str(&it.artifact_type);
        h.feed_str(&it.content);
        h.feed_str(&it.provenance);
        h.feed_str(&it.grounding_ref);
        h.feed_str(&it.replay_receipt_ref);
        h.feed_str(&it.split);
    }
    format!("{:016x}", h.finish())
}

/// Order-INDEPENDENT digest of the admitted set (input pre-sorted by id).
fn hash_admitted(dataset_id: &str, admitted: &[AdmittedItem]) -> String {
    let mut h = Fnv1a::new();
    h.feed_str(dataset_id);
    h.feed_u64(admitted.len() as u64);
    for a in admitted {
        h.feed_str(&a.id);
        h.feed_str(&a.content_hash);
        h.feed_str(a.kind.tag());
        h.feed_str(a.split.tag());
    }
    format!("{:016x}", h.finish())
}
