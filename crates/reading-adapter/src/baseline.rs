//! P10 — the baseline failure-profile eval.
//!
//! Runs a battery of RECORDED baseline-model outputs (the kind of text an
//! off-the-shelf model emits — some valid, many not) through the adapter, and
//! records the score and the failure categories. This establishes the baseline
//! model's actual failure profile against the hardened codec + READ-1 verifier
//! BEFORE any training is considered. Deterministic: scripted backend + pure
//! codec ⇒ same report every run.

use crate::adapter::Adapter;
use crate::backend::{ReadingTask, ScriptedBackend};
use reading_codec::RejectKind;
use reading_substrate::Corpus;
use std::collections::BTreeMap;

/// One recorded baseline-model output (a sampled response), named for the report.
pub struct RecordedOutput {
    pub name: &'static str,
    pub text: &'static str,
}

/// The category one model output lands in after the codec/verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A verifier-approved, source-grounded answer was finalized.
    Finalized,
    /// A legal partial proposal that did not finalize an answer.
    AcceptedPartial,
    /// Rejected by the codec/verifier, with the reason.
    Rejected(RejectKind),
}

impl Outcome {
    /// A stable string label for the failure-profile histogram.
    pub fn label(&self) -> String {
        match self {
            Outcome::Finalized => "finalized".to_string(),
            Outcome::AcceptedPartial => "accepted_partial".to_string(),
            Outcome::Rejected(kind) => format!("rejected:{kind:?}"),
        }
    }
}

/// One scored entry of the baseline battery.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineEntry {
    pub name: String,
    pub outcome: Outcome,
}

/// The baseline failure profile: the score and the category histogram.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineReport {
    pub total: usize,
    pub finalized: usize,
    pub accepted_partial: usize,
    pub rejected: usize,
    /// Histogram keyed by outcome label (deterministic order via BTreeMap).
    pub by_category: BTreeMap<String, usize>,
    pub entries: Vec<BaselineEntry>,
}

/// Score `outputs` against the hardened codec for `corpus`/`question`. Each
/// recorded output is replayed through a scripted backend + the adapter, so the
/// model's text only ever reaches the substrate via `reading_codec::decode`.
pub fn baseline_report(
    corpus: &Corpus,
    question: &str,
    outputs: &[RecordedOutput],
) -> BaselineReport {
    let mut entries = Vec::with_capacity(outputs.len());
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    let (mut finalized, mut accepted_partial, mut rejected) = (0usize, 0usize, 0usize);

    for output in outputs {
        let adapter = Adapter::new(ScriptedBackend::new(output.text));
        let task = ReadingTask::new(corpus, question);
        let (_untrusted, decision) = adapter.run(&task);
        let outcome = match decision {
            Ok(decoded) => {
                if decoded.finalized.is_some() {
                    finalized += 1;
                    Outcome::Finalized
                } else {
                    accepted_partial += 1;
                    Outcome::AcceptedPartial
                }
            }
            Err(error) => {
                rejected += 1;
                Outcome::Rejected(error.kind())
            }
        };
        *by_category.entry(outcome.label()).or_insert(0) += 1;
        entries.push(BaselineEntry {
            name: output.name.to_string(),
            outcome,
        });
    }

    BaselineReport {
        total: outputs.len(),
        finalized,
        accepted_partial,
        rejected,
        by_category,
        entries,
    }
}

/// The default recorded-output battery: a representative slice of what a baseline
/// model emits against the bridge-safety task — one fully valid (verbatim)
/// sequence, one legal partial, and several failures spanning the rejection
/// classes (malformed prose, unknown action, hallucinated span, ungrounded
/// extraction, and a fabricated-but-cited claim the READ-1 verifier refuses).
pub fn baseline_outputs() -> Vec<RecordedOutput> {
    vec![
        RecordedOutput {
            name: "verbatim_grounded_full_sequence",
            text: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":1},
                {"action":"read_span","span_id":0},
                {"action":"read_span","span_id":2},
                {"action":"extract_claim","statement":"Bridge B remained passable during light rain on the same day.","source_span_ids":[1]},
                {"action":"extract_claim","statement":"Bridge A was reported structurally damaged after the June storm.","source_span_ids":[0,2]},
                {"action":"extract_entity","name":"Bridge B","source_span_ids":[1]},
                {"action":"extract_entity","name":"Bridge A","source_span_ids":[0,2]},
                {"action":"compare_claims","left":0,"right":1},
                {"action":"synthesize","answer_text":"Bridge B remained passable during light rain on the same day. Bridge A was reported structurally damaged after the June storm.","supporting_claims":[0,1]}
            ]"#,
        },
        RecordedOutput {
            name: "inspect_only_partial",
            text: r#"[{"action":"inspect_corpus"}]"#,
        },
        RecordedOutput {
            name: "free_form_prose",
            text: "Bridge B looks fine to me, so go ahead and cross it.",
        },
        RecordedOutput {
            name: "unknown_action",
            text: r#"[{"action":"decide","verdict":"Bridge B is safe"}]"#,
        },
        RecordedOutput {
            name: "hallucinated_span",
            text: r#"[{"action":"read_span","span_id":42}]"#,
        },
        RecordedOutput {
            name: "ungrounded_extraction",
            text: r#"[{"action":"inspect_corpus"},{"action":"extract_claim","statement":"Bridge B is safe","source_span_ids":[]}]"#,
        },
        RecordedOutput {
            name: "fabricated_supported_claim",
            text: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"Bridge A is fully safe to cross after the storm.","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"Bridge A is fully safe to cross after the storm.","supporting_claims":[0]}
            ]"#,
        },
    ]
}
