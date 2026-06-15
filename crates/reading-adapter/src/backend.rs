//! P10 — the replaceable model-backend boundary.
//!
//! A backend is asked to act on a reading task (the corpus as an environment,
//! plus the question) and PRODUCES untrusted candidate text — a reading-action
//! JSON proposal — and nothing else. It holds no authority, no memory mutator,
//! and no path to the substrate executor/verifier. Its output is always treated
//! as untrusted and only ever flows into `reading_codec::decode`. Swapping the
//! scripted baseline for a real model is a backend replacement, not an authority
//! change: "P10 inserts a model backend, not a smarter authority."

use reading_substrate::Corpus;

/// What a backend is asked to act on: the reading ENVIRONMENT (the corpus, seen
/// as metadata) and the question. A real backend renders this into a prompt; the
/// scripted baseline replays a recorded answer regardless.
pub struct ReadingTask<'a> {
    pub corpus: &'a Corpus,
    pub question: &'a str,
}

impl<'a> ReadingTask<'a> {
    pub fn new(corpus: &'a Corpus, question: &'a str) -> Self {
        ReadingTask { corpus, question }
    }
}

/// A replaceable language-codec backend. The contract is intentionally tiny:
/// given a task, return untrusted candidate text. That is the ONLY thing a
/// backend may do — it cannot reach memory, authority, or the verifier.
pub trait ModelBackend {
    /// Produce untrusted candidate reading-action text for `task`.
    fn propose(&self, task: &ReadingTask) -> String;
}

/// The deterministic baseline backend: a recorded model response replayed
/// verbatim. Same task always yields the same text (temperature-0-equivalent),
/// so it stands in for an off-the-shelf local model with no live dependency and
/// keeps the eval reproducible.
pub struct ScriptedBackend {
    output: String,
}

impl ScriptedBackend {
    pub fn new(output: impl Into<String>) -> Self {
        ScriptedBackend {
            output: output.into(),
        }
    }
}

impl ModelBackend for ScriptedBackend {
    fn propose(&self, _task: &ReadingTask) -> String {
        // Untrusted text out — never executed here, only returned for the adapter
        // to hand to the codec.
        self.output.clone()
    }
}
