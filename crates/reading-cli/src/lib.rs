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

pub use corpus_load::{corpus_from_documents, corpus_from_spans, load_documents};

use reading_codec::{decode, CodecPolicy};
use reading_substrate::{split_sentences, verify};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// One document in the run file: its title and its sentence spans (in order).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentDto {
    pub title: String,
    pub spans: Vec<String>,
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

const SCHEMA: &str = "read0-run-v1";

/// What can go wrong at the CLI boundary. Every failure is explicit.
#[derive(Debug)]
pub enum CliError {
    Io(String),
    Json(String),
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
    let documents = documents
        .iter()
        .map(|(title, content)| DocumentDto {
            title: title.clone(),
            spans: split_sentences(content),
        })
        .collect();
    Ok(RunFile {
        schema: SCHEMA.to_string(),
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

/// Rebuild the corpus from the stored spans and re-run the stored plan through
/// the codec. Shared by verify/replay so both use the codec-only path.
fn rederive(
    file: &RunFile,
) -> Result<(reading_substrate::Corpus, reading_substrate::ReadingRun), CliError> {
    // Integrity: a run read0 produced has exactly ONE sentence per span. Reject a
    // run file whose stored spans are not single sentences (e.g. a hand-edited
    // multi-sentence span that the run path could never produce, which would let
    // a claim ground against an inner sentence on the verify/replay path).
    for document in &file.documents {
        for span in &document.spans {
            if split_sentences(span).len() != 1 {
                return Err(CliError::Tamper(format!(
                    "stored span is not a single sentence: {span:?}"
                )));
            }
        }
    }
    let docs: Vec<(String, Vec<String>)> = file
        .documents
        .iter()
        .map(|d| (d.title.clone(), d.spans.clone()))
        .collect();
    let corpus = corpus_from_spans(&docs);
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
    let file: RunFile = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    if file.schema != SCHEMA {
        return Err(CliError::Json(format!(
            "unexpected schema: {}",
            file.schema
        )));
    }
    Ok(file)
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
}
