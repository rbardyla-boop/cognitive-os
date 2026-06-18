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
    /// READ-14 structural-integrity hash binding the schema + document/section/span
    /// STRUCTURE. Present (and required to match) on `read0-run-v3`; absent on the
    /// pre-v3 v1/v2 receipts. Non-evidentiary: it gates tamper, never grounding.
    #[serde(default)]
    pub structure_hash: Option<u64>,
    pub receipt: Receipt,
}

/// The receipt schema versions `read0` understands (READ-13/14). Versions are
/// explicit so verify/replay behaviour is deterministic and a receipt's tag must
/// agree with its content, and any other tag is refused outright. The tag never
/// grants evidence authority — it only governs how the STRUCTURE is rebuilt and
/// whether the structural-integrity hash (READ-14) is required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SchemaVersion {
    /// `read0-run-v1` — the pre-section receipt: documents carry spans only, with
    /// no section metadata and no structure hash. Migrated forward deterministically
    /// by treating each document as one default (empty-heading) section over all its
    /// spans. The flat rebuild reproduces the same span ids and hashes a v1 run
    /// produced, so old receipts still verify/replay (sections affect reading ORDER
    /// only).
    V1,
    /// `read0-run-v2` — documents carry heading-labelled sections that partition
    /// their spans (READ-12), but no structure hash. Sections must be present (they
    /// cannot be dropped); a structure hash must NOT be present (it predates v3).
    V2,
    /// `read0-run-v3` — like v2, plus an explicit structural-integrity hash
    /// (READ-14) binding the schema, document titles, span texts, and section
    /// structure. Sections AND a matching structure hash must both be present. This
    /// is what `read0` writes today.
    V3,
}

impl SchemaVersion {
    const V1_TAG: &'static str = "read0-run-v1";
    const V2_TAG: &'static str = "read0-run-v2";
    const V3_TAG: &'static str = "read0-run-v3";

    /// Recognize a receipt's schema tag, or refuse an unknown version cleanly.
    fn parse(tag: &str) -> Result<Self, CliError> {
        match tag {
            Self::V1_TAG => Ok(Self::V1),
            Self::V2_TAG => Ok(Self::V2),
            Self::V3_TAG => Ok(Self::V3),
            other => Err(CliError::UnsupportedSchema(other.to_string())),
        }
    }

    /// The on-disk tag for this version.
    fn tag(self) -> &'static str {
        match self {
            Self::V1 => Self::V1_TAG,
            Self::V2 => Self::V2_TAG,
            Self::V3 => Self::V3_TAG,
        }
    }
}

/// The structural-integrity LEVEL a verified receipt provides (READ-15). It is
/// DERIVED from the receipt's validated schema version — never persisted — so it
/// cannot be forged: a receipt cannot claim a higher level than its tag earns. The
/// level classifies how strongly the receipt's STRUCTURE is bound; it never affects
/// grounding (evidence authority is identical at every level).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrityLevel {
    /// `read0-run-v3`: the structural metadata is bound by a structure hash (READ-14)
    /// — the current, full-integrity receipt `read0` writes today.
    Current,
    /// `read0-run-v1`/`read0-run-v2`: a LEGACY receipt whose structural metadata is
    /// NOT bound by a structure hash. Its evidence is still fully verified, but a
    /// structural-metadata edit (a heading/title string, an uncited span, a section
    /// boundary) is UNDETECTABLE. Accepted for backward compatibility, but explicitly
    /// NOT equivalent to current integrity — so a v3→v2 downgrade cannot pass itself
    /// off as full integrity.
    LegacyUnboundStructure,
}

impl IntegrityLevel {
    fn from_version(version: SchemaVersion) -> Self {
        match version {
            SchemaVersion::V3 => Self::Current,
            SchemaVersion::V1 | SchemaVersion::V2 => Self::LegacyUnboundStructure,
        }
    }

    /// A stable, MACHINE-CHECKABLE token (not prose) so a consumer can gate on the
    /// integrity level deterministically. `legacy_unbound_structure` is the explicit
    /// warning a legacy/downgraded receipt carries.
    pub fn token(self) -> &'static str {
        match self {
            Self::Current => "structure_bound",
            Self::LegacyUnboundStructure => "legacy_unbound_structure",
        }
    }

    /// Whether this is the current, fully-bound integrity level. A legacy or
    /// downgraded receipt returns `false`, so it is never treated as fully current.
    pub fn is_current(self) -> bool {
        matches!(self, Self::Current)
    }
}

/// The outcome of verifying a receipt (READ-15): the verifier `Receipt` plus the
/// structural-integrity LEVEL the receipt provides. Bundling them means verification
/// can never report a passing receipt without also reporting how strongly its
/// structure is bound — a legacy/downgraded receipt is flagged, never silently
/// accepted as equivalent to current integrity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyOutcome {
    pub receipt: Receipt,
    pub integrity: IntegrityLevel,
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
    let documents: Vec<DocumentDto> = corpus
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
    // read0 always writes the current schema (v3): every document carries its
    // sections, and the receipt carries a structural-integrity hash binding the
    // schema + document/section/span structure. v1/v2 are recognized for reading old
    // receipts, never written. The structure hash is non-evidentiary — it binds
    // tamper-detection over the structure, not grounding.
    let schema = SchemaVersion::V3_TAG.to_string();
    let structure_hash = Some(structural_hash(&schema, &documents));
    Ok(RunFile {
        schema,
        question: question.to_string(),
        documents,
        plan: plan.to_string(),
        answer: run.proof.answer_text.clone(),
        memory_hash: run.memory_hash,
        answer_hash: run.answer_hash,
        structure_hash,
        receipt: Receipt {
            grounded: report.grounded,
            answer_supported: report.answer_supported,
            replay_matches: report.replay_matches,
            passed: report.passed,
        },
    })
}

/// Re-derive the run from its own plan and confirm it matches AND verifies, and
/// classify the receipt's structural-integrity level (READ-15). Catches a tampered
/// answer/hash (mismatch) and a tampered plan/span (re-decode rejects or grounding
/// fails). The integrity level is derived from the validated schema version, so a
/// legacy/downgraded receipt is reported as such, never as current. Pure.
pub fn verify_file(file: &RunFile) -> Result<VerifyOutcome, CliError> {
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
    // The schema was already validated inside rederive -> rebuild_corpus, so this
    // re-parse cannot disagree; it derives the (unforgeable, never-persisted)
    // integrity level from the validated version.
    let integrity = IntegrityLevel::from_version(SchemaVersion::parse(&file.schema)?);
    Ok(VerifyOutcome {
        receipt: Receipt {
            grounded: report.grounded,
            answer_supported: report.answer_supported,
            replay_matches: report.replay_matches,
            passed: report.passed,
        },
        integrity,
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

// --- Structural-integrity hash (READ-14) ---
//
// A deterministic FNV-1a 64-bit hash over the receipt's STRUCTURAL metadata: the
// schema tag, and per document the title, the ordered span texts, and the ordered
// sections (heading + span count). This is the same FNV-1a 64-bit construction the
// substrate uses for its content hashes (offset basis 0xcbf29ce484222325, prime
// 0x100000001b3); it is kept local here so the substrate stays a pure evidence-hash
// layer and the receipt-integrity concern lives with the receipt.
//
// The structural hash is an INTEGRITY checksum, NOT an evidence signal: it never
// reaches the codec or the grounding path, and it never makes a heading or title
// citable. It binds the persisted, NON-EVIDENTIARY structure so a field edit that
// the consistency checks would miss — a heading or title string, an UNCITED span's
// text, a section boundary that still partitions — is caught as tamper. Evidence
// authority is protected independently and unchanged: memory_hash/answer_hash are
// re-derived from the plan through the codec, and grounding still flows only from
// cited span text. Because the structure is non-evidentiary, this checksum is a
// faithfulness/corruption guard over it, never a substitute for that re-derivation.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_bytes(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

fn fnv_u64(h: u64, value: u64) -> u64 {
    fnv_bytes(h, &value.to_le_bytes())
}

/// Deterministic structural hash over the schema tag and documents (titles, ordered
/// span texts, ordered sections). Every variable-length field is length-prefixed at
/// every level, so distinct structures cannot collide by re-grouping bytes across
/// fields. Pure and entropy-free, so identical structure always hashes identically.
fn structural_hash(schema: &str, documents: &[DocumentDto]) -> u64 {
    let mut h = FNV_OFFSET;
    h = fnv_u64(h, schema.len() as u64);
    h = fnv_bytes(h, schema.as_bytes());
    h = fnv_u64(h, documents.len() as u64);
    for doc in documents {
        h = fnv_u64(h, doc.title.len() as u64);
        h = fnv_bytes(h, doc.title.as_bytes());
        h = fnv_u64(h, doc.spans.len() as u64);
        for span in &doc.spans {
            h = fnv_u64(h, span.len() as u64);
            h = fnv_bytes(h, span.as_bytes());
        }
        h = fnv_u64(h, doc.sections.len() as u64);
        for sec in &doc.sections {
            h = fnv_u64(h, sec.heading.len() as u64);
            h = fnv_bytes(h, sec.heading.as_bytes());
            h = fnv_u64(h, sec.span_count as u64);
        }
    }
    h
}

/// Enforce the READ-14 structural-hash discipline by schema version. A v3 receipt
/// MUST carry a structure hash that matches the one recomputed from its structural
/// fields, so a structural field edit is caught as tamper. A pre-v3 (v1/v2) receipt
/// MUST NOT carry a structure hash — a stray hash under an older tag is ambiguous,
/// and forbidding it blocks a relabel-to-legacy that keeps the field. The hash binds
/// structure only; it is never consulted for grounding.
fn enforce_structure_hash(file: &RunFile, version: SchemaVersion) -> Result<(), CliError> {
    match version {
        SchemaVersion::V3 => {
            let stored = file.structure_hash.ok_or_else(|| {
                CliError::Tamper(format!(
                    "a {} receipt must carry a structure hash",
                    SchemaVersion::V3_TAG
                ))
            })?;
            let computed = structural_hash(&file.schema, &file.documents);
            if stored != computed {
                return Err(CliError::Tamper(format!(
                    "structure hash mismatch: receipt {stored:#018x} vs recomputed {computed:#018x}"
                )));
            }
            Ok(())
        }
        SchemaVersion::V1 | SchemaVersion::V2 => {
            if file.structure_hash.is_some() {
                return Err(CliError::Tamper(format!(
                    "a {} receipt must not carry a structure hash",
                    version.tag()
                )));
            }
            Ok(())
        }
    }
}

/// Rebuild the SECTIONED corpus a run file describes, rejecting tamper, so a
/// consumer (verify/replay, or section-aware autonomy over a real read0 output)
/// gets the SAME sections and span ids `run` built. Integrity checks:
///
/// 0. The receipt's schema tag is a version `read0` understands (READ-13) — an
///    unknown tag is refused as `UnsupportedSchema` before any rebuild — and the
///    structural-integrity hash agrees with the tag (READ-14): a v3 receipt must
///    carry a matching structure hash, a v1/v2 receipt must carry none.
/// 1. Each stored span is exactly ONE sentence — a multi-sentence span is a corpus
///    the run path could never produce (it would let a claim ground against an
///    inner sentence). (Pre-existing READ-3 check, unchanged.)
/// 2. No stored span is an ATX heading — a heading must never be re-derived as a
///    span, or it could be cited and grounded (HEADING-AS-SPAN tamper).
/// 3. The section structure must AGREE with the schema tag (READ-13 version
///    discipline): a v2/v3 receipt MUST carry section metadata (it cannot silently
///    vanish), a v1 receipt MUST NOT (a v1 tag wearing sections is ambiguous).
///    A v1 receipt migrates to one default empty-heading section over all spans; a
///    v2/v3 receipt's section span counts must partition its body spans exactly
///    (SECTION/BODY-MISMATCH tamper).
///
/// Sections are metadata: they affect reading ORDER only, never grounding, so the
/// re-derived memory/answer hashes are unaffected and the existing tamper checks
/// keep their full strength. The schema tag governs structure, never evidence.
pub fn rebuild_corpus(file: &RunFile) -> Result<reading_substrate::Corpus, CliError> {
    let version = SchemaVersion::parse(&file.schema)?;
    enforce_structure_hash(file, version)?;
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
                // are ambiguous (v1 or v2+?) and rejected. Otherwise migrate forward
                // deterministically: one default empty-heading section over all spans.
                if !document.sections.is_empty() {
                    return Err(CliError::Tamper(format!(
                        "a {} receipt must not carry section metadata",
                        SchemaVersion::V1_TAG
                    )));
                }
                vec![(String::new(), document.spans.clone())]
            }
            SchemaVersion::V2 | SchemaVersion::V3 => {
                // A v2/v3 receipt MUST carry its sections; empty sections means the
                // section metadata was dropped — it cannot silently disappear.
                if document.sections.is_empty() {
                    return Err(CliError::Tamper(format!(
                        "a {} receipt must carry section metadata (sections were dropped)",
                        version.tag()
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

/// `read0 verify`: load the run file and re-verify it, returning the receipt and
/// its structural-integrity level (READ-15).
pub fn verify_run(out_path: &Path) -> Result<VerifyOutcome, CliError> {
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
        assert!(verify_file(&file).unwrap().receipt.passed);
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
        assert!(verify_file(&file).unwrap().receipt.passed);
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

    // Downgrade a freshly produced (v3) receipt to a FAITHFUL legacy receipt: v2
    // drops the structure hash (keeps sections); v1 drops both the hash and the
    // sections. These are exactly the shapes an old read0 wrote, so they verify.
    fn as_v2(mut file: RunFile) -> RunFile {
        file.schema = SchemaVersion::V2_TAG.to_string();
        file.structure_hash = None;
        file
    }
    fn as_v1(mut file: RunFile) -> RunFile {
        file.schema = SchemaVersion::V1_TAG.to_string();
        file.structure_hash = None;
        for doc in &mut file.documents {
            doc.sections.clear();
        }
        file
    }

    // Re-seal a hand-edited v3 receipt's structure hash, modelling the STRONGEST
    // attacker — one who recomputes the structure hash after tampering. The deeper
    // checks (heading-as-span, partition, grounding) must still fire, proving the
    // structure hash is an added layer that never MASKS them.
    fn reseal(mut file: RunFile) -> RunFile {
        file.structure_hash = Some(structural_hash(&file.schema, &file.documents));
        file
    }

    #[test]
    fn run_receipt_includes_section_metadata() {
        // The run file is schema v3 and carries the heading-labelled sections that
        // partition the body spans (a span COUNT per heading — never a span).
        let file = headed_run();
        assert_eq!(file.schema, "read0-run-v3");
        assert!(file.structure_hash.is_some(), "v3 carries a structure hash");
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
        // span (so it can never be cited or grounded). Re-sealed so the heading-as-
        // span check fires even against an attacker who recomputes the structure hash.
        let mut file = headed_run();
        file.documents[0].spans[0] = "# Injected Heading".to_string();
        let file = reseal(file);
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn section_body_mismatch_tamper_is_rejected() {
        // A receipt whose section span counts no longer partition the body spans is
        // rejected — the sections must match the body they describe. Re-sealed so the
        // partition check fires even if the attacker recomputes the structure hash.
        let mut file = headed_run();
        file.documents[0].sections[0].span_count = 5; // sum 5+1 != 2 body spans
        let file = reseal(file);
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn headingless_document_round_trips_under_v3() {
        // A headingless document still runs/verifies/replays under the v3 receipt:
        // its sole section is the default empty-heading section over all spans, and a
        // structure hash is still bound over that (degenerate) structure.
        let file = produce_run(&docs(), QUESTION, VALID_PLAN).expect("headingless run finalizes");
        assert_eq!(file.schema, "read0-run-v3");
        assert!(file.structure_hash.is_some());
        assert_eq!(file.documents[0].sections.len(), 1);
        assert_eq!(file.documents[0].sections[0].heading, "");
        assert_eq!(
            file.documents[0].sections[0].span_count,
            file.documents[0].spans.len()
        );
        assert!(verify_file(&file).unwrap().receipt.passed);
        replay_file(&file).expect("headingless receipt replays");
    }

    #[test]
    fn section_count_overflow_tamper_is_rejected_without_panic() {
        // A crafted receipt whose section counts OVERFLOW (so a naive `sum == len`
        // check could wrap past) must return a graceful Tamper, never panic on the
        // out-of-bounds partition slice. Re-sealed so execution REACHES the partition
        // (otherwise the structure-hash check would reject first and the no-panic
        // property of partition_sections would never actually be exercised).
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
        let file = reseal(file);
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn span_text_tamper_still_caught_under_v3() {
        // The pre-existing GROUNDING re-derivation keeps its full strength: editing a
        // cited body span so it no longer supports the answer fails verify. Re-sealed
        // so the structure hash matches — proving grounding catches the tamper even
        // when the attacker recomputes the structure hash (the evidence binding is
        // independent of, and not masked by, the structural one).
        let mut file = headed_run();
        file.documents[0].spans[1] = "Winds will be calm and gentle tonight.".to_string();
        let file = reseal(file);
        assert!(verify_file(&file).is_err());
    }

    // --- READ-13: explicit receipt schema versioning / migration discipline ---

    #[test]
    fn v1_headingless_receipt_migrates_and_verifies() {
        // A faithful old `read0-run-v1` receipt: the schema tag is v1 and it carries
        // NO section metadata and NO structure hash (the pre-READ-12 shape). It
        // migrates forward — each document becomes one default empty-heading section
        // over all its spans — and still verifies and replays, because sections affect
        // reading ORDER only (the flat rebuild reproduces the same span ids and hashes).
        let file = as_v1(headed_run());
        assert!(
            verify_file(&file).unwrap().receipt.passed,
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
        // Ambiguity attack: a receipt tagged v1 but still wearing section metadata is
        // neither cleanly v1 nor v2+. It is rejected, so a v1 tag can never be used to
        // smuggle (or relabel away the integrity of) sections. The structure hash is
        // cleared so this isolates the v1+sections case (not the v1+hash case).
        let mut file = headed_run(); // v3, with sections
        file.schema = SchemaVersion::V1_TAG.to_string();
        file.structure_hash = None;
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
        let mut file = as_v2(headed_run());
        for doc in &mut file.documents {
            doc.sections.clear(); // faithful v2, sections then dropped
        }
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn unknown_schema_is_rejected() {
        // An unrecognized schema version is refused cleanly (never accepted by
        // default), on both verify and replay.
        let mut file = headed_run();
        file.schema = "read0-run-v9".to_string();
        assert!(matches!(
            verify_file(&file),
            Err(CliError::UnsupportedSchema(_))
        ));
        assert!(matches!(
            replay_file(&file),
            Err(CliError::UnsupportedSchema(_))
        ));
    }

    // --- READ-14: structural-integrity hash over the receipt's metadata ---

    #[test]
    fn v3_receipt_carries_and_verifies_structure_hash() {
        // read0 writes v3 with a structure hash bound over the schema + structure; it
        // verifies and replays, and the hash never makes the heading evidence (the
        // heading text does not appear in the grounded answer).
        let file = headed_run();
        assert_eq!(file.schema, "read0-run-v3");
        assert!(file.structure_hash.is_some());
        assert!(verify_file(&file).unwrap().receipt.passed);
        replay_file(&file).expect("a v3 receipt replays");
        assert!(
            !file.answer.contains("Wind Forecast"),
            "the heading is bound for integrity but is never grounded evidence"
        );
    }

    #[test]
    fn heading_string_tamper_is_rejected() {
        // The headline READ-14 capability: editing a section HEADING string — NOT to
        // a heading-as-span, just a different label that still partitions the body —
        // passes every READ-12/13 consistency check but breaks the structure hash, so
        // it is now caught. (Heading metadata is non-evidentiary, but it drives
        // section ranking, so it must be tamper-evident.)
        let mut file = headed_run();
        file.documents[0].sections[1].heading = "Calm Skies".to_string();
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn title_tamper_is_rejected() {
        // Editing a document title (non-evidentiary metadata) breaks the structure
        // hash and is rejected — previously it would have slipped through unnoticed.
        let mut file = headed_run();
        file.documents[0].title = "tampered.txt".to_string();
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn uncited_span_tamper_caught_under_v3_not_v2() {
        // An UNCITED span (span 0 — the plan only reads span 1) edited to a different
        // single non-heading sentence is invisible to grounding (the plan never reads
        // it), passes the one-sentence and heading-as-span checks, and keeps the
        // partition. Under a legacy v2 receipt it slips through; under v3 the
        // structure hash binds the full span list and catches it.
        let mut v2 = as_v2(headed_run());
        v2.documents[0].spans[0] = "The bridge is closed.".to_string();
        assert!(
            verify_file(&v2).unwrap().receipt.passed,
            "a legacy v2 receipt does not bind the uncited span (the gap v3 closes)"
        );
        let mut v3 = headed_run();
        v3.documents[0].spans[0] = "The bridge is closed.".to_string();
        assert!(
            matches!(verify_file(&v3), Err(CliError::Tamper(_))),
            "v3 binds the full span structure, so the uncited-span edit is caught"
        );
    }

    #[test]
    fn v3_receipt_with_missing_structure_hash_is_rejected() {
        // A v3 receipt MUST carry a structure hash; stripping it is rejected (the
        // binding cannot silently disappear — the same discipline as dropped sections).
        let mut file = headed_run();
        file.structure_hash = None;
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn v3_structure_hash_tamper_is_rejected() {
        // A corrupted structure hash (without a matching structure) is rejected.
        let mut file = headed_run();
        file.structure_hash = Some(file.structure_hash.unwrap() ^ 0xDEAD_BEEF);
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn v2_receipt_carrying_structure_hash_is_rejected() {
        // Relabel guard: take a v3 receipt, relabel to v2 but KEEP the structure hash.
        // A pre-v3 tag must not carry the field (it is ambiguous and would let a
        // downgrade keep a stale binding), so it is rejected.
        let mut file = headed_run();
        file.schema = SchemaVersion::V2_TAG.to_string();
        assert!(matches!(verify_file(&file), Err(CliError::Tamper(_))));
        assert!(matches!(replay_file(&file), Err(CliError::Tamper(_))));
    }

    #[test]
    fn structural_hash_is_deterministic_and_field_sensitive() {
        // The hash is pure (identical structure → identical hash) and sensitive to
        // every bound field, so any structural edit changes it.
        let file = headed_run();
        let base = structural_hash(&file.schema, &file.documents);
        assert_eq!(
            base,
            structural_hash(&file.schema, &file.documents),
            "identical structure hashes identically"
        );
        let mut heading_edit = file.clone();
        heading_edit.documents[0].sections[0].heading = "Changed".to_string();
        assert_ne!(
            base,
            structural_hash(&heading_edit.schema, &heading_edit.documents)
        );
        let mut title_edit = file.clone();
        title_edit.documents[0].title = "other.txt".to_string();
        assert_ne!(
            base,
            structural_hash(&title_edit.schema, &title_edit.documents)
        );
        let mut span_edit = file.clone();
        span_edit.documents[0].spans[0] = "Different sentence here.".to_string();
        assert_ne!(
            base,
            structural_hash(&span_edit.schema, &span_edit.documents)
        );
    }

    // --- READ-15: receipt integrity LEVEL classification / downgrade policy ---

    #[test]
    fn v3_receipt_reports_current_integrity() {
        // A v3 receipt verifies AND reports the current, fully-bound integrity level
        // with the machine-checkable `structure_bound` token.
        let outcome = verify_file(&headed_run()).expect("v3 verifies");
        assert!(outcome.receipt.passed);
        assert_eq!(outcome.integrity, IntegrityLevel::Current);
        assert!(outcome.integrity.is_current());
        assert_eq!(outcome.integrity.token(), "structure_bound");
    }

    #[test]
    fn legacy_v2_and_v1_report_legacy_unbound_structure() {
        // Legacy receipts still verify (their evidence is fully bound) but are
        // classified LegacyUnboundStructure with the explicit, machine-checkable
        // `legacy_unbound_structure` token — never the current level.
        for legacy in [as_v2(headed_run()), as_v1(headed_run())] {
            let outcome = verify_file(&legacy).expect("legacy receipt still verifies");
            assert!(outcome.receipt.passed);
            assert_eq!(outcome.integrity, IntegrityLevel::LegacyUnboundStructure);
            assert!(!outcome.integrity.is_current());
            assert_eq!(outcome.integrity.token(), "legacy_unbound_structure");
        }
    }

    #[test]
    fn v3_to_v2_downgrade_is_not_reported_as_current() {
        // The headline READ-15 policy: a v3 receipt downgraded to v2 (relabel + strip
        // the structure hash) still verifies, but is NOT reported as current
        // integrity — so weaker integrity can never pass itself off as equivalent.
        let v3 = verify_file(&headed_run()).expect("v3 verifies");
        assert!(v3.integrity.is_current());
        let downgraded = verify_file(&as_v2(headed_run())).expect("downgrade still verifies");
        assert!(
            !downgraded.integrity.is_current(),
            "a v3→v2 downgrade must not report current integrity"
        );
        assert_eq!(downgraded.integrity.token(), "legacy_unbound_structure");
    }

    #[test]
    fn integrity_level_does_not_change_evidence_authority() {
        // Classifying the integrity level must not touch grounding: the v3 receipt and
        // its v2 downgrade produce the IDENTICAL verifier Receipt (same grounded /
        // answer_supported / replay_matches / passed) — only the integrity LEVEL
        // differs. The level is about structural binding, never about evidence.
        let v3 = verify_file(&headed_run()).expect("v3 verifies");
        let v2 = verify_file(&as_v2(headed_run())).expect("v2 verifies");
        assert_eq!(
            v3.receipt, v2.receipt,
            "evidence receipt is level-independent"
        );
        assert_ne!(
            v3.integrity, v2.integrity,
            "but the integrity level differs"
        );
    }

    #[test]
    fn integrity_level_is_derived_from_version_not_a_stored_claim() {
        // The level follows the (validated) schema version, not a persisted field, so
        // it cannot be forged: the SAME run reports Current as v3 and Legacy as v2.
        let mut file = headed_run();
        assert_eq!(
            verify_file(&file).unwrap().integrity,
            IntegrityLevel::Current
        );
        file = as_v2(file);
        assert_eq!(
            verify_file(&file).unwrap().integrity,
            IntegrityLevel::LegacyUnboundStructure
        );
    }

    #[test]
    fn integrity_tokens_are_stable_and_machine_checkable() {
        // Pin the machine-checkable tokens the gate and downstream consumers rely on.
        assert_eq!(IntegrityLevel::Current.token(), "structure_bound");
        assert_eq!(
            IntegrityLevel::LegacyUnboundStructure.token(),
            "legacy_unbound_structure"
        );
    }
}
