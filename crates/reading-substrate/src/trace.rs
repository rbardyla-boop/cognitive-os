//! READ-0 — the reading action log and its deterministic executor.
//!
//! The reader's behaviour is a `ReadingTrace`: an ordered log of
//! inspect → read → extract → compare → synthesize actions. For READ-0 the
//! reader is SCRIPTED (deterministic), standing in for the eventual LLM
//! controller. Executing a trace against a corpus deterministically builds the
//! structured memory and the answer; re-executing the same trace reproduces them
//! exactly (replay).

use crate::corpus::{Corpus, SpanId};
use crate::memory::{Claim, Entity, Memory, ProofObject};

/// One reading action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadingAction {
    /// Look at corpus metadata (must happen before any span is read).
    InspectCorpus,
    /// Read one span's text, by id.
    ReadSpan(SpanId),
    /// Extract a claim grounded in the given (already-read) source spans.
    ExtractClaim {
        statement: String,
        source_spans: Vec<SpanId>,
    },
    /// Extract a named entity grounded in the given source spans.
    ExtractEntity {
        name: String,
        source_spans: Vec<SpanId>,
    },
    /// Compare two already-extracted claims.
    CompareClaims { left: u64, right: u64 },
    /// Synthesize the final answer from supporting claims.
    Synthesize {
        answer_text: String,
        supporting_claims: Vec<u64>,
    },
}

/// The ordered, saved record of what the reader did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReadingTrace {
    pub actions: Vec<ReadingAction>,
}

impl ReadingTrace {
    pub fn new() -> Self {
        ReadingTrace::default()
    }

    pub fn push(&mut self, action: ReadingAction) {
        self.actions.push(action);
    }
}

/// Why executing a trace failed — every failure is explicit, never silent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadingError {
    /// A span was read or cited before corpus metadata was inspected.
    MetadataNotInspectedFirst,
    /// A claim/entity was extracted with no source span (ungrounded).
    UngroundedExtraction,
    /// A cited span id is not in the corpus.
    UnknownSpan(SpanId),
    /// A claim cites a span that was never read.
    UnreadSpan(SpanId),
    /// A compare/synthesize action cites a claim that does not exist.
    UnknownClaim(u64),
    /// The trace produced no answer (no `Synthesize`).
    NoAnswer,
}

/// A complete reading run — the structured memory, the answer, the saved trace,
/// and content hashes that make replay checkable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadingRun {
    pub question: String,
    pub memory: Memory,
    pub proof: ProofObject,
    pub trace: ReadingTrace,
    pub read_spans: Vec<SpanId>,
    pub memory_hash: u64,
    pub answer_hash: u64,
}

/// Execute `trace` against `corpus` for `question`, deterministically building
/// structured memory and the answer. Pure: same inputs → same `ReadingRun`.
pub fn execute(
    corpus: &Corpus,
    question: &str,
    trace: &ReadingTrace,
) -> Result<ReadingRun, ReadingError> {
    let mut memory = Memory::new();
    let mut read_spans: Vec<SpanId> = Vec::new();
    let mut inspected = false;
    let mut next_claim_id = 0u64;
    let mut next_entity_id = 0u64;
    let mut proof: Option<ProofObject> = None;

    for action in &trace.actions {
        match action {
            ReadingAction::InspectCorpus => {
                let _ = corpus.metadata();
                inspected = true;
            }
            ReadingAction::ReadSpan(id) => {
                if !inspected {
                    return Err(ReadingError::MetadataNotInspectedFirst);
                }
                if !corpus.contains(*id) {
                    return Err(ReadingError::UnknownSpan(*id));
                }
                if !read_spans.contains(id) {
                    read_spans.push(*id);
                }
            }
            ReadingAction::ExtractClaim {
                statement,
                source_spans,
            } => {
                require_grounded(source_spans, corpus, &read_spans, inspected)?;
                memory.claims.push(Claim {
                    id: next_claim_id,
                    statement: statement.clone(),
                    source_spans: source_spans.clone(),
                });
                next_claim_id += 1;
            }
            ReadingAction::ExtractEntity { name, source_spans } => {
                require_grounded(source_spans, corpus, &read_spans, inspected)?;
                memory.entities.push(Entity {
                    id: next_entity_id,
                    name: name.clone(),
                    source_spans: source_spans.clone(),
                });
                next_entity_id += 1;
            }
            ReadingAction::CompareClaims { left, right } => {
                if memory.claim(*left).is_none() {
                    return Err(ReadingError::UnknownClaim(*left));
                }
                if memory.claim(*right).is_none() {
                    return Err(ReadingError::UnknownClaim(*right));
                }
            }
            ReadingAction::Synthesize {
                answer_text,
                supporting_claims,
            } => {
                for cid in supporting_claims {
                    if memory.claim(*cid).is_none() {
                        return Err(ReadingError::UnknownClaim(*cid));
                    }
                }
                proof = Some(ProofObject {
                    question: question.to_string(),
                    answer_text: answer_text.clone(),
                    supporting_claims: supporting_claims.clone(),
                });
            }
        }
    }

    let proof = proof.ok_or(ReadingError::NoAnswer)?;
    let memory_hash = hash_memory(&memory);
    let answer_hash = hash_proof(&proof);
    Ok(ReadingRun {
        question: question.to_string(),
        memory,
        proof,
        trace: trace.clone(),
        read_spans,
        memory_hash,
        answer_hash,
    })
}

/// A claim/entity must be inspected-first, non-empty, and cite only spans that
/// exist and were actually read.
fn require_grounded(
    source_spans: &[SpanId],
    corpus: &Corpus,
    read_spans: &[SpanId],
    inspected: bool,
) -> Result<(), ReadingError> {
    if !inspected {
        return Err(ReadingError::MetadataNotInspectedFirst);
    }
    if source_spans.is_empty() {
        return Err(ReadingError::UngroundedExtraction);
    }
    for s in source_spans {
        if !corpus.contains(*s) {
            return Err(ReadingError::UnknownSpan(*s));
        }
        if !read_spans.contains(s) {
            return Err(ReadingError::UnreadSpan(*s));
        }
    }
    Ok(())
}

/// FNV-1a 64-bit mixing of one byte stream — pure and deterministic.
fn mix(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn mix_u64(h: u64, value: u64) -> u64 {
    mix(h, &value.to_le_bytes())
}

/// Deterministic content hash of the structured memory.
fn hash_memory(memory: &Memory) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h = mix_u64(h, memory.claims.len() as u64);
    for c in &memory.claims {
        h = mix_u64(h, c.id);
        h = mix(h, c.statement.as_bytes());
        h = mix_u64(h, c.source_spans.len() as u64);
        for s in &c.source_spans {
            h = mix_u64(h, s.0);
        }
    }
    h = mix_u64(h, memory.entities.len() as u64);
    for e in &memory.entities {
        h = mix_u64(h, e.id);
        h = mix(h, e.name.as_bytes());
        for s in &e.source_spans {
            h = mix_u64(h, s.0);
        }
    }
    h
}

/// Deterministic content hash of the answer proof.
fn hash_proof(proof: &ProofObject) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    h = mix(h, proof.question.as_bytes());
    h = mix(h, proof.answer_text.as_bytes());
    h = mix_u64(h, proof.supporting_claims.len() as u64);
    for c in &proof.supporting_claims {
        h = mix_u64(h, *c);
    }
    h
}
