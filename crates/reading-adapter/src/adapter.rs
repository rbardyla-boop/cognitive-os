//! P10 — the adapter: model backend → untrusted text → reading-codec ONLY.
//!
//! The adapter is the single seam where a backend's untrusted output enters the
//! system. Its only substrate-facing call is `reading_codec::decode`, which
//! validates the text into typed actions, runs them through the substrate, and
//! finalizes an answer only if the READ-1 verifier approves it. The adapter
//! itself never reaches the substrate executor, the verifier, or the finalizer —
//! it cannot bypass the codec, cannot assign authority, and cannot mutate memory.

use crate::backend::{ModelBackend, ReadingTask};
use reading_codec::{decode, CodecError, CodecPolicy, Decoded};

/// Wraps a backend and routes its untrusted proposals through the P9 codec under
/// the strict policy. Replace the backend (scripted ↔ real model) without
/// changing where authority lives.
pub struct Adapter<B: ModelBackend> {
    backend: B,
    policy: CodecPolicy,
}

impl<B: ModelBackend> Adapter<B> {
    /// Build an adapter that always decodes under the strict codec policy.
    pub fn new(backend: B) -> Self {
        Adapter {
            backend,
            policy: CodecPolicy::strict(),
        }
    }

    /// Ask the backend to propose, then decode its UNTRUSTED output through the
    /// codec. Returns both the raw untrusted text (for audit — what the model
    /// actually said) and the codec's decision. The adapter performs no other
    /// processing of the model's output.
    pub fn run(&self, task: &ReadingTask) -> (String, Result<Decoded, CodecError>) {
        let untrusted = self.backend.propose(task);
        let decision = decode(task.corpus, task.question, &untrusted, self.policy);
        (untrusted, decision)
    }
}
