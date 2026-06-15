//! P10 — OPTIONAL real local-model backend (feature `local-model`, OFF by
//! default; never run by release_check, only compiled/linted).
//!
//! It invokes an operator-chosen local model command (e.g. a llama.cpp or ollama
//! CLI) with an EXPLICIT argv — no shell, so there is no command injection — and
//! writes the rendered task (corpus METADATA + question, never the full text) to
//! the model's stdin. The model's stdout is returned as UNTRUSTED text and flows
//! through the exact same `Adapter` → `reading_codec::decode` path as the scripted
//! baseline. The backend holds no authority and no memory mutator; it cannot
//! finalize an answer without the READ-1 verifier. Set temperature 0 in the argv
//! for deterministic proposals.

use crate::backend::{ModelBackend, ReadingTask};
use std::io::Write;
use std::process::{Command, Stdio};

/// A real local model invoked as a subprocess. `argv` is the explicit command,
/// e.g. `vec!["ollama".into(), "run".into(), "llama3".into()]`.
pub struct LocalProcessBackend {
    pub argv: Vec<String>,
}

impl LocalProcessBackend {
    pub fn new(argv: Vec<String>) -> Self {
        LocalProcessBackend { argv }
    }

    /// Render the task as a prompt: show the model document METADATA (titles +
    /// span ids) and the question, and require a reading-action JSON array. The
    /// full span text is never dumped — the model must read spans by id.
    fn render_prompt(task: &ReadingTask) -> String {
        let mut prompt = String::new();
        prompt.push_str(
            "Read external documents by emitting ONLY a JSON array of reading actions.\n",
        );
        prompt.push_str("Actions: inspect_corpus, read_span{span_id}, extract_claim{statement,source_span_ids}, ");
        prompt.push_str("extract_entity{name,source_span_ids}, compare_claims{left,right}, synthesize{answer_text,supporting_claims}.\n");
        prompt.push_str(&format!("Question: {}\n", task.question));
        prompt.push_str("Documents (metadata only):\n");
        for doc in task.corpus.metadata() {
            let ids: Vec<u64> = doc.span_ids.iter().map(|s| s.0).collect();
            prompt.push_str(&format!("- {} (span ids: {:?})\n", doc.title, ids));
        }
        prompt.push_str("Claims must be verbatim excerpts of their cited spans.\n");
        prompt
    }
}

impl ModelBackend for LocalProcessBackend {
    fn propose(&self, task: &ReadingTask) -> String {
        let prompt = Self::render_prompt(task);
        let (program, args) = match self.argv.split_first() {
            Some(parts) => parts,
            None => return String::new(),
        };
        let spawned = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(_) => return String::new(), // model unavailable → empty (codec rejects)
        };
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(prompt.as_bytes());
        }
        match child.wait_with_output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).into_owned(),
            Err(_) => String::new(),
        }
    }
}
