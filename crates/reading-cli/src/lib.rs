//! reading-cli — READ-3, the `read0` operator surface for the reading track.
//!
//! `read0` loads a real folder of documents, builds a corpus of one sentence per
//! span, and runs an untrusted reading PLAN through the hardened P9 codec
//! (`reading_codec::decode`) — which validates it, executes it through the
//! substrate, and finalizes an answer only if the READ-1/READ-2 verifier
//! approves. It emits a replayable run file with the answer, content hashes, and
//! a verifier receipt. `verify`/`replay` re-derive the result from the run file
//! and reject any tamper. The plan never reaches memory except through the codec;
//! `read0` calls no substrate executor directly.

#![forbid(unsafe_code)]

mod corpus_load;

pub use corpus_load::{
    corpus_from_documents, corpus_from_sections, corpus_from_spans, load_documents,
    SectionedDocument,
};

use reading_codec::{decode, CodecPolicy};
use reading_substrate::{split_sentences, verify};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One heading-labelled section of a run-file document (READ-12): its heading
/// (METADATA — never a span) and the NUMBER of consecutive body spans it owns. The
/// sections partition the document's flat `spans`, so the section structure is
/// persisted without duplicating span text and without granting a heading a span.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionDto {
    pub heading: String,
    pub span_count: usize,
}

/// One document in the run file: its title, its sentence spans (in order), and its
/// heading-labelled sections (a partition of `spans`). `spans` stays the canonical
/// span-id source (so the existing grounding/hash/tamper checks are unchanged);
/// `sections` is additive metadata persisted so section-aware autonomy can operate
/// over a real read0 output without rebuilding a different structure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentDto {
    pub title: String,
    pub spans: Vec<String>,
    #[serde(default)]
    pub sections: Vec<SectionDto>,
}

/// The verifier receipt: the three READ-0/1/2 checks and their conjunction.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    pub grounded: bool,
    pub answer_supported: bool,
    pub replay_matches: bool,
    pub passed: bool,
}

/// The serialized run: enough to rebuild the corpus and re-derive the answer.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunFile {
    pub schema: String,
    pub question: String,
    pub documents: Vec<DocumentDto>,
    /// The untrusted reading plan (verbatim) — re-decoded on verify/replay.
    pub plan: String,
    pub answer: String,
    pub memory_hash: u64,
    pub answer_hash: u64,
    pub receipt: Receipt,
}

/// The receipt schema versions `read0` understands (READ-13). Versions are
/// explicit so verify/replay behaviour is deterministic and a receipt's tag must
/// agree with its content: a v2 receipt MUST carry its section metadata, a v1
/// receipt MUST NOT, and any other tag is refused outright. The tag never grants
/// evidence authority — it only governs how the section STRUCTURE is rebuilt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SchemaVersion {
    /// `read0-run-v1` — the pre-section receipt: documents carry spans only, with
    /// no section metadata. Migrated forward deterministically by treating each
    /// document as one default (empty-heading) section over all its spans. The flat
    /// rebuild reproduces the same span ids and hashes a v1 run produced, so old
    /// receipts still verify/replay (sections affect reading ORDER only).
    V1,
    /// `read0-run-v2` — documents carry heading-labelled sections that partition
    /// their spans (READ-12). Sections must be present (they cannot be dropped).
    V2,
}

impl SchemaVersion {
    const V1_TAG: &'static str = "read0-run-v1";
    const V2_TAG: &'static str = "read0-run-v2";

    /// Recognize a receipt's schema tag, or refuse an unknown version cleanly.
    fn parse(tag: &str) -> Result<Self, CliError> {
        match tag {
            Self::V1_TAG => Ok(Self::V1),
            Self::V2_TAG => Ok(Self::V2),
            other => Err(CliError::UnsupportedSchema(other.to_string())),
        }
    }
}

/// What can go wrong at the CLI boundary. Every failure is explicit.
#[derive(Debug)]
pub enum CliError {
    Io(String),
    Json(String),
    /// The receipt's schema tag is not a version `read0` understands (READ-13).
    UnsupportedSchema(String),
    /// The codec rejected the untrusted plan (malformed / fabricated / etc.).
    Rejected(String),
    /// The plan produced no finalized answer (a partial plan, not an answer).
    NotFinalized,
    /// The verifier did not pass on the (re)built run.
    VerifyFailed(Vec<String>),
    /// A stored run does not match the run re-derived from its own plan.
    Tamper(String),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Io(m) => write!(f, "io error: {m}"),
            CliError::Json(m) => write!(f, "json error: {m}"),
            CliError::UnsupportedSchema(s) => write!(f, "unsupported receipt schema: {s}"),
            CliError::Rejected(m) => write!(f, "plan rejected by codec: {m}"),
            CliError::NotFinalized => write!(f, "plan produced no finalized answer"),
            CliError::VerifyFailed(p) => write!(f, "verifier failed: {}", p.join("; ")),
            CliError::Tamper(m) => write!(f, "tamper detected: {m}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Json(e.to_string())
    }
}

/// Core (pure, no I/O): build a corpus from documents, run the untrusted plan
/// through the codec, require a verifier-approved answer, and return the run
/// file. The plan reaches memory only via `reading_codec::decode`.
pub fn produce_run(
    documents: &[(String, String)],
    question: &str,
    plan: &str,
) -> Result<RunFile, CliError> {
    let corpus = corpus_from_documents(documents);
    let decoded = decode(&corpus, question, plan, CodecPolicy::strict())
        .map_err(|e| CliError::Rejected(format!("{e:?}")))?;
    let run = decoded.finalized.ok_or(CliError::NotFinalized)?;
    let report = verify(&corpus, &run);
    if !report.passed {
        return Err(CliError::VerifyFailed(report.problems));
    }
    // Store the spans the corpus actually built — the body sentences in span-id
    // order, with ATX heading lines excluded (READ-11) — AND the heading-labelled
    // sections that partition them (READ-12). The headings are metadata only (a
    // span count, never a span), so a heading can never be re-derived as evidence.
    let documents = corpus
        .metadata()
        .iter()
        .map(|doc| DocumentDto {
            title: doc.title.clone(),
            spans: doc
                .span_ids
                .iter()
                .map(|id| {
                    corpus
                        .read_span(*id)
                        .expect("metadata span ids exist in the corpus")
                        .text()
                        .to_string()
                })
                .collect(),
            sections: doc
                .sections
                .iter()
                .map(|s| SectionDto {
                    heading: s.heading.clone(),
                    span_count: s.span_ids.len(),
                })
                .collect(),
        })
        .collect();
    Ok(RunFile {
        // read0 always writes the current schema: every document carries its
        // sections (v2). v1 is recognized for reading old receipts, never written.
        schema: SchemaVersion::V2_TAG.to_string(),
        question: question.to_string(),
        documents,
        plan: plan.to_string(),
        answer: run.proof.answer_text.clone(),
        memory_hash: run.memory_hash,
        answer_hash: run.answer_hash,
        receipt: Receipt {
            grounded: report.grounded,
            answer_supported: report.answer_supported,
            replay_matches: report.replay_matches,
            passed: report.passed,
        },
    })
}

/// Re-derive the run from its own plan and confirm it matches AND verifies.
/// Catches a tampered answer/hash (mismatch) and a tampered plan/span
/// (re-decode rejects or grounding fails). Pure.
pub fn verify_file(file: &RunFile) -> Result<Receipt, CliError> {
    let (corpus, run) = rederive(file)?;
    if run.memory_hash != file.memory_hash
        || run.answer_hash != file.answer_hash
        || run.proof.answer_text != file.answer
    {
        return Err(CliError::Tamper(
            "stored answer/hashes do not match the run re-derived from the plan".to_string(),
        ));
    }
    let report = verify(&corpus, &run);
    if !report.passed {
        return Err(CliError::VerifyFailed(report.problems));
    }
    Ok(Receipt {
        grounded: report.grounded,
        answer_supported: report.answer_supported,
        replay_matches: report.replay_matches,
        passed: report.passed,
    })
}

/// Re-derive the run from its own plan and confirm the content hashes reproduce.
/// Pure.
pub fn replay_file(file: &RunFile) -> Result<(), CliError> {
    let (_corpus, run) = rederive(file)?;
    if run.memory_hash != file.memory_hash || run.answer_hash != file.answer_hash {
        return Err(CliError::Tamper(
            "replay hashes differ from the recorded run".to_string(),
        ));
    }
    Ok(())
}

/// One document's heading-labelled sections as `(heading, span_texts)` pairs.
type DocumentSections = Vec<(String, Vec<String>)>;

/// Rebuild the SECTIONED corpus a run file describes, rejecting tamper, so a
/// consumer (verify/replay, or section-aware autonomy over a real read0 output)
/// gets the SAME sections and span ids `run` built. Integrity checks:
///
/// 0. The receipt's schema tag is a version `read0` understands (READ-13) — an
///    unknown tag is refused as `UnsupportedSchema` before any rebuild.
/// 1. Each stored span is exactly ONE sentence — a multi-sentence span is a corpus
///    the run path could never produce (it would let a claim ground against an
///    inner sentence). (Pre-existing READ-3 check, unchanged.)
/// 2. No stored span is an ATX heading — a heading must never be re-derived as a
///    span, or it could be cited and grounded (HEADING-AS-SPAN tamper).
/// 3. The section structure must AGREE with the schema tag (READ-13 version
///    discipline): a v2 receipt MUST carry section metadata (it cannot silently
///    vanish), a v1 receipt MUST NOT (a v1 tag wearing v2 sections is ambiguous).
///    A v1 receipt migrates to one default empty-heading section over all spans; a
///    v2 receipt's section span counts must partition its body spans exactly
///    (SECTION/BODY-MISMATCH tamper).
///
/// Sections are metadata: they affect reading ORDER only, never grounding, so the
/// re-derived memory/answer hashes are unaffected and the existing tamper checks
/// keep their full strength. The schema tag governs structure, never evidence.
pub fn rebuild_corpus(file: &RunFile) -> Result<reading_substrate::Corpus, CliError> {
    let version = SchemaVersion::parse(&file.schema)?;
    let mut docs_sectioned: Vec<SectionedDocument> = Vec::new();
    for document in &file.documents {
        for span in &document.spans {
            if split_sentences(span).len() != 1 {
                return Err(CliError::Tamper(format!(
                    "stored span is not a single sentence: {span:?}"
                )));
            }
            if corpus_load::parse_atx_heading(span).is_some() {
                return Err(CliError::Tamper(format!(
                    "stored span is an ATX heading (a heading must never be a span): {span:?}"
                )));
            }
        }
        let sections = match version {
            SchemaVersion::V1 => {
                // A v1 receipt carries NO section metadata. Sections under a v1 tag
                // are ambiguous (v1 or v2?) and rejected. Otherwise migrate forward
                // deterministically: one default empty-heading section over all spans.
                if !document.sections.is_empty() {
                    return Err(CliError::Tamper(format!(
                        "a {} receipt must not carry section metadata",
                        SchemaVersion::V1_TAG
                    )));
                }
                vec![(String::new(), document.spans.clone())]
            }
            SchemaVersion::V2 => {
                // A v2 receipt MUST carry its sections; empty sections means the
                // section metadata was dropped — it cannot silently disappear.
                if document.sections.is_empty() {
                    return Err(CliError::Tamper(format!(
                        "a {} receipt must carry section metadata (sections were dropped)",
                        SchemaVersion::V2_TAG
                    )));
                }
                partition_sections(document)?
            }
        };
        docs_sectioned.push((document.title.clone(), sections));
    }
    Ok(corpus_from_sections(&docs_sectioned))
}

/// Partition a v2 document's body spans by its section span counts, using CHECKED,
/// bounded arithmetic: a count that overflows or overruns the body is tamper (so a
/// crafted receipt returns Tamper, never panics on an out-of-bounds slice), and
/// after all sections the cover must be EXACT (no under-coverage). This is
/// overflow-safe where a plain `sum() == len` check could be wrapped past by a
/// `usize::MAX` count.
fn partition_sections(document: &DocumentDto) -> Result<DocumentSections, CliError> {
    let mut idx = 0usize;
    let mut secs = Vec::with_capacity(document.sections.len());
    for s in &document.sections {
        let end = idx
            .checked_add(s.span_count)
            .filter(|&e| e <= document.spans.len())
            .ok_or_else(|| {
                CliError::Tamper(format!(
                    "section span count {} overruns the {} body spans",
                    s.span_count,
                    document.spans.len()
                ))
            })?;
        secs.push((s.heading.clone(), document.spans[idx..end].to_vec()));
        idx = end;
    }
    if idx != document.spans.len() {
        return Err(CliError::Tamper(format!(
            "section span counts cover only {idx} of {} body spans",
            document.spans.len()
        )));
    }
    Ok(secs)
}

/// Rebuild the corpus from the stored receipt and re-run the stored plan through
/// the codec. Shared by verify/replay so both use the codec-only path.
fn rederive(
    file: &RunFile,
) -> Result<(reading_substrate::Corpus, reading_substrate::ReadingRun), CliError> {
    let corpus = rebuild_corpus(file)?;
    let decoded = decode(&corpus, &file.question, &file.plan, CodecPolicy::strict())
        .map_err(|e| CliError::Rejected(format!("{e:?}")))?;
    let run = decoded.finalized.ok_or(CliError::NotFinalized)?;
    Ok((corpus, run))
}

// --- I/O wrappers (the binary's surface) ---

/// `read0 run`: load the folder, read the plan, produce the run, write `out`.
pub fn run_reading(
    docs_dir: &Path,
    question: &str,
    plan_path: &Path,
    out_path: &Path,
) -> Result<RunFile, CliError> {
    let documents = load_documents(docs_dir)?;
    let plan = std::fs::read_to_string(plan_path)?;
    let file = produce_run(&documents, question, &plan)?;
    std::fs::write(out_path, serde_json::to_string_pretty(&file)?)?;
    Ok(file)
}

/// `read0 verify`: load the run file and re-verify it.
pub fn verify_run(out_path: &Path) -> Result<Receipt, CliError> {
    let file: RunFile = read_run_file(out_path)?;
    verify_file(&file)
}

/// `read0 replay`: load the run file and re-derive its hashes.
pub fn replay_run(out_path: &Path) -> Result<(), CliError> {
    let file: RunFile = read_run_file(out_path)?;
    replay_file(&file)
}

fn read_run_file(path: &Path) -> Result<RunFile, CliError> {
    // The schema tag is validated in the pure path (`rebuild_corpus`, the single
    // chokepoint shared by verify/replay and the section consumers), so an unknown
    // version is refused as `UnsupportedSchema` and a tag/content mismatch as
    // `Tamper` there — no duplicated, driftable check here.
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs() -> Vec<(String, String)> {
        vec![(
            "report.txt".to_string(),
            "Bridge A was damaged. Bridge B stayed open.".to_string(),
        )]
        // spans: 0 = "Bridge A was damaged.", 1 = "Bridge B stayed open."
    }
    const QUESTION: &str = "Which bridge is open?";

    // A valid, sentence-grounded plan that finalizes "Bridge B stayed open."
    const VALID_PLAN: &str = r#"[
        {"action":"inspect_corpus"},
        {"action":"read_span","span_id":1},
        {"action":"extract_claim","statement":"Bridge B stayed open.","source_span_ids":[1]},
        {"action":"synthesize","answer_text":"Bridge B stayed open.","supporting_claims":[0]}
    ]"#;

    #[test]
    fn real_folder_to_verified_answer() {
        let file = produce_run(&docs(), QUESTION, VALID_PLAN).expect("valid plan finalizes");
        assert_eq!(file.answer, "Bridge B stayed open.");
        assert!(file.receipt.passed);
        assert!(verify_file(&file).unwrap().passed);
        replay_file(&file).expect("replay reproduces the run");
    }

    #[test]
    fn metadata_before_read_span() {
        // read_span before inspect_corpus → the substrate rejects via the codec.
        let plan = r#"[{"action":"read_span","span_id":1}]"#;
        let err = produce_run(&docs(), QUESTION, plan).unwrap_err();
        assert!(matches!(
            err,
            CliError::NotFinalized | CliError::Rejected(_)
        ));
    }

    #[test]
    fn fabricated_claim_rejected() {
        // A claim the cited span does not support.
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":1},
            {"action":"extract_claim","statement":"Bridge B is closed forever.","source_span_ids":[1]},
            {"action":"synthesize","answer_text":"Bridge B is closed forever.","supporting_claims":[0]}
        ]"#;
        let err = produce_run(&docs(), QUESTION, plan).unwrap_err();
        assert!(matches!(err, CliError::Rejected(_)));
    }

    #[test]
    fn fragment_claim_rejected() {
        // READ-2: a sub-sentence fragment ("Bridge A") is not a full sentence.
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":0},
            {"action":"extract_claim","statement":"Bridge A","source_span_ids":[0]},
            {"action":"synthesize","answer_text":"Bridge A","supporting_claims":[0]}
        ]"#;
        let err = produce_run(&docs(), QUESTION, plan).unwrap_err();
        assert!(matches!(err, CliError::Rejected(_)));
    }

    #[test]
    fn trace_replay_same_answer_and_tamper_is_caught() {
        let file = produce_run(&docs(), QUESTION, VALID_PLAN).unwrap();
        replay_file(&file).expect("clean replay matches");
        // Tamper the recorded answer hash → replay must detect the mismatch.
        let mut tampered = file.clone();
        tampered.answer_hash ^= 0xDEAD_BEEF;
        assert!(matches!(replay_file(&tampered), Err(CliError::Tamper(_))));
        // Tamper the recorded answer text → verify must detect the mismatch.
        let mut tampered2 = file;
        tampered2.answer = "Bridge A was damaged.".to_string();
        assert!(matches!(verify_file(&tampered2), Err(CliError::Tamper(_))));
    }

    #[test]
    fn tampering_a_span_to_fabricate_fails_verify() {
        // Edit the stored span so the cited text no longer supports the answer.
        let mut file = produce_run(&docs(), QUESTION, VALID_PLAN).unwrap();
        file.documents[0].spans[1] = "Bridge B collapsed entirely.".to_string();
        // Re-deriving against the tampered span: the claim is no longer grounded,
        // so the codec rejects on re-decode (no finalized run).
        assert!(verify_file(&file).is_err());
    }

    #[test]
    fn multi_sentence_stored_span_is_rejected_on_verify_and_replay() {
        // A hand-edited run file whose stored span holds TWO sentences is not a
        // corpus read0 run could produce (run guarantees one sentence per span).
        // verify/replay must reject it, so a claim cannot ground against an inner
        // sentence of a multi-sentence stored span.
        let mut file = produce_run(&docs(), QUESTION, VALID_PLAN).unwrap();
        file.documents[0].spans = vec!["Bridge A was damaged. Bridge B stayed open.".to_string()];
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    // --- READ-11: headed documents through the read0 run/verify/replay path ---

    // A Markdown document: the heading "Wind Forecast" is metadata; only the body
    // sentences (span ids 0,1) are addressable spans.
    fn headed_docs() -> Vec<(String, String)> {
        vec![(
            "forecast.txt".to_string(),
            "# Overview\nThe bridge is open.\n## Wind Forecast\nWinds will reach forty miles per hour."
                .to_string(),
        )]
        // spans: 0 = "The bridge is open.", 1 = "Winds will reach forty miles per hour."
    }

    #[test]
    fn headed_document_runs_verifies_and_replays() {
        // A body sentence under a heading is a normal grounded span: the run
        // finalizes, verifies, and replays. The stored spans are the body
        // sentences only (the heading is never stored as a span).
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":1},
            {"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},
            {"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}
        ]"#;
        let file = produce_run(&headed_docs(), "What is the wind forecast?", plan)
            .expect("a body sentence finalizes");
        assert_eq!(file.answer, "Winds will reach forty miles per hour.");
        assert!(file.receipt.passed);
        // The heading is not stored as a span.
        for span in &file.documents[0].spans {
            assert!(
                !span.starts_with('#'),
                "no stored span is a heading: {span:?}"
            );
            assert_ne!(span, "Overview");
            assert_ne!(span, "Wind Forecast");
        }
        assert!(verify_file(&file).unwrap().passed);
        replay_file(&file).expect("a headed document replays");
    }

    #[test]
    fn claim_citing_heading_is_rejected() {
        // A plan that tries to launder the HEADING text ("Wind Forecast") into a
        // grounded claim by citing a body span is rejected: the heading is not the
        // span's text, so it does not ground.
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":1},
            {"action":"extract_claim","statement":"Wind Forecast","source_span_ids":[1]},
            {"action":"synthesize","answer_text":"Wind Forecast","supporting_claims":[0]}
        ]"#;
        let err = produce_run(&headed_docs(), "What is the wind forecast?", plan).unwrap_err();
        assert!(matches!(err, CliError::Rejected(_)));
    }

    #[test]
    fn misleading_heading_without_body_support_cannot_finalize() {
        // The heading claims the bridge is safe; the body says it is damaged. A plan
        // asserting the heading's claim, citing the body span, cannot finalize — a
        // heading promise is never grounded unless a body sentence supports it.
        let docs = vec![(
            "bridge.txt".to_string(),
            "# Bridge A Is Safe\nBridge A was damaged in the storm.".to_string(),
        )];
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":0},
            {"action":"extract_claim","statement":"Bridge A Is Safe.","source_span_ids":[0]},
            {"action":"synthesize","answer_text":"Bridge A Is Safe.","supporting_claims":[0]}
        ]"#;
        let err = produce_run(&docs, "Is Bridge A safe?", plan).unwrap_err();
        assert!(matches!(err, CliError::Rejected(_)));
    }

    // --- READ-12: persisted section metadata in run receipts ---

    // A valid headed-document run, finalizing the body sentence under "Wind Forecast".
    fn headed_run() -> RunFile {
        let plan = r#"[
            {"action":"inspect_corpus"},
            {"action":"read_span","span_id":1},
            {"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},
            {"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}
        ]"#;
        produce_run(&headed_docs(), "What is the wind forecast?", plan).expect("valid headed run")
    }

    #[test]
    fn run_receipt_includes_section_metadata() {
        // The run file is schema v2 and carries the heading-labelled sections that
        // partition the body spans (a span COUNT per heading — never a span).
        let file = headed_run();
        assert_eq!(file.schema, "read0-run-v2");
        let doc = &file.documents[0];
        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Overview", "Wind Forecast"]);
        let total: usize = doc.sections.iter().map(|s| s.span_count).sum();
        assert_eq!(total, doc.spans.len(), "sections partition the body spans");
    }

    #[test]
    fn rebuild_corpus_reconstructs_the_run_sections() {
        // verify/replay rebuild the SAME sections (and span ids) the run built — the
        // heading is metadata, the span texts are the body sentences.
        let file = headed_run();
        let corpus = rebuild_corpus(&file).expect("a valid receipt rebuilds");
        let doc = &corpus.metadata()[0];
        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Overview", "Wind Forecast"]);
        let text_of = |sid: &reading_substrate::SpanId| corpus.read_span(*sid).unwrap().text();
        assert_eq!(
            doc.sections[1]
                .span_ids
                .iter()
                .map(text_of)
                .collect::<Vec<_>>(),
            vec!["Winds will reach forty miles per hour."]
        );
    }

    #[test]
    fn heading_as_span_tamper_is_rejected() {
        // A hand-edited receipt that injects an ATX heading as a body span is
        // rejected before any grounding — a heading can never be re-derived as a
        // span (so it can never be cited or grounded).
        let mut file = headed_run();
        file.documents[0].spans[0] = "# Injected Heading".to_string();
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn section_body_mismatch_tamper_is_rejected() {
        // A receipt whose section span counts no longer partition the body spans is
        // rejected — the sections must match the body they describe.
        let mut file = headed_run();
        file.documents[0].sections[0].span_count = 5; // sum 5+1 != 2 body spans
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn headingless_document_round_trips_under_v2() {
        // A headingless document still runs/verifies/replays under the v2 receipt:
        // its sole section is the default empty-heading section over all spans.
        let file = produce_run(&docs(), QUESTION, VALID_PLAN).expect("headingless run finalizes");
        assert_eq!(file.schema, "read0-run-v2");
        assert_eq!(file.documents[0].sections.len(), 1);
        assert_eq!(file.documents[0].sections[0].heading, "");
        assert_eq!(
            file.documents[0].sections[0].span_count,
            file.documents[0].spans.len()
        );
        assert!(verify_file(&file).unwrap().passed);
        replay_file(&file).expect("headingless receipt replays");
    }

    #[test]
    fn section_count_overflow_tamper_is_rejected_without_panic() {
        // A crafted receipt whose section counts OVERFLOW (so a naive `sum == len`
        // check could wrap past) must return a graceful Tamper, never panic on the
        // out-of-bounds partition slice.
        let mut file = headed_run();
        file.documents[0].sections = vec![
            SectionDto {
                heading: "H1".to_string(),
                span_count: usize::MAX,
            },
            SectionDto {
                heading: "H2".to_string(),
                span_count: 6,
            },
        ];
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn span_text_tamper_still_caught_under_v2() {
        // The pre-existing tamper check keeps its full strength: editing a body span
        // so it no longer supports the answer fails verify (no weakening from v2).
        let mut file = headed_run();
        file.documents[0].spans[1] = "Winds will be calm and gentle tonight.".to_string();
        assert!(verify_file(&file).is_err());
    }

    // --- READ-13: explicit receipt schema versioning / migration discipline ---

    #[test]
    fn v1_headingless_receipt_migrates_and_verifies() {
        // A faithful old `read0-run-v1` receipt: the schema tag is v1 and it carries
        // NO section metadata (the pre-READ-12 shape). It migrates forward — each
        // document becomes one default empty-heading section over all its spans —
        // and still verifies and replays, because sections affect reading ORDER only
        // (the flat rebuild reproduces the same span ids and hashes).
        let mut file = headed_run();
        file.schema = SchemaVersion::V1_TAG.to_string();
        for doc in &mut file.documents {
            doc.sections.clear();
        }
        assert!(
            verify_file(&file).unwrap().passed,
            "a v1 headingless receipt migrates and verifies"
        );
        replay_file(&file).expect("a v1 receipt migrates and replays");
        // The migration rebuilds one default section over the body spans.
        let corpus = rebuild_corpus(&file).expect("v1 migrates");
        assert_eq!(corpus.metadata()[0].sections.len(), 1);
        assert_eq!(corpus.metadata()[0].sections[0].heading, "");
    }

    #[test]
    fn v1_receipt_carrying_sections_is_rejected() {
        // Ambiguity attack: a receipt tagged v1 but still wearing v2 section
        // metadata is neither cleanly v1 nor v2. It is rejected, so a v1 tag can
        // never be used to smuggle (or relabel away the integrity of) sections.
        let mut file = headed_run(); // v2, with sections
        file.schema = SchemaVersion::V1_TAG.to_string();
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn v2_receipt_with_dropped_sections_is_rejected() {
        // The load-bearing READ-13 hardening: a v2 receipt whose section metadata has
        // been stripped is rejected. Under READ-12 the empty-sections path silently
        // fell back to a flat rebuild and still verified (sections affect only order,
        // not hashes) — so sections could DISAPPEAR unnoticed. Now a v2 tag REQUIRES
        // its sections, so the drop is caught.
        let mut file = headed_run();
        for doc in &mut file.documents {
            doc.sections.clear(); // still tagged v2
        }
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn unknown_schema_is_rejected() {
        // An unrecognized schema version is refused cleanly (never accepted by
        // default), on both verify and replay.
        let mut file = headed_run();
        file.schema = "read0-run-v3".to_string();
        assert!(matches!(
            verify_file(&file),
            Err(CliError::UnsupportedSchema(_))
        ));
        assert!(matches!(
            replay_file(&file),
            Err(CliError::UnsupportedSchema(_))
        ));
    }
}
