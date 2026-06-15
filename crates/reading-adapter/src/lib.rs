//! reading-adapter — P10, the baseline local LLM adapter boundary.
//!
//! Inserts a REPLACEABLE model backend in front of the P9 reading-codec. A
//! backend produces untrusted candidate text; the adapter routes it ONLY through
//! `reading_codec::decode`, which validates it into typed actions, executes them
//! through the substrate, and finalizes an answer only if the READ-1 verifier
//! approves it. The backend holds no authority, mutates no memory, and cannot
//! finalize on its own.
//!
//! The default backend is a deterministic scripted (recorded-model) baseline, so
//! the build and eval stay offline and reproducible. The optional `local-model`
//! feature adds a real off-the-shelf local model as a subprocess — off by
//! default, never run by release_check. No training, ever.
//!
//! Hard rule: P10 inserts a model backend, not a smarter authority.

#![forbid(unsafe_code)]

mod adapter;
mod backend;
mod baseline;
#[cfg(feature = "local-model")]
mod local_backend;

pub use adapter::Adapter;
pub use backend::{ModelBackend, ReadingTask, ScriptedBackend};
pub use baseline::{
    baseline_outputs, baseline_report, BaselineEntry, BaselineReport, Outcome, RecordedOutput,
};
#[cfg(feature = "local-model")]
pub use local_backend::LocalProcessBackend;

#[cfg(test)]
mod tests {
    use super::*;
    use reading_codec::RejectKind;
    use reading_substrate::fixture;

    fn task_env() -> (reading_substrate::Corpus, String) {
        let (corpus, question, _) = fixture();
        (corpus, question)
    }

    fn output_named(name: &str) -> RecordedOutput {
        baseline_outputs()
            .into_iter()
            .find(|o| o.name == name)
            .expect("named recorded output exists")
    }

    #[test]
    fn baseline_adapter_emits_untrusted_text() {
        // A backend only PRODUCES text — it does not execute or finalize anything.
        let (corpus, question) = task_env();
        let backend = ScriptedBackend::new(r#"[{"action":"inspect_corpus"}]"#);
        let task = ReadingTask::new(&corpus, &question);
        let emitted = backend.propose(&task);
        assert_eq!(emitted, r#"[{"action":"inspect_corpus"}]"#);
    }

    #[test]
    fn adapter_output_decoded_only_by_codec() {
        // The adapter's sole processing of the model output is decode(): it hands
        // back the exact untrusted text plus the codec's decision, nothing else.
        let (corpus, question) = task_env();
        let text = r#"[{"action":"inspect_corpus"},{"action":"read_span","span_id":1}]"#;
        let adapter = Adapter::new(ScriptedBackend::new(text));
        let (untrusted, decision) = adapter.run(&ReadingTask::new(&corpus, &question));
        assert_eq!(
            untrusted, text,
            "adapter returns the raw untrusted text for audit"
        );
        let decoded = decision.expect("valid actions decode");
        assert!(
            decoded.finalized.is_none(),
            "no synthesize → no finalized answer"
        );
    }

    #[test]
    fn invalid_json_from_model_rejected() {
        let (corpus, question) = task_env();
        let adapter = Adapter::new(ScriptedBackend::new("just cross bridge B, it's fine"));
        let (_text, decision) = adapter.run(&ReadingTask::new(&corpus, &question));
        assert_eq!(decision.unwrap_err().kind(), RejectKind::Malformed);
    }

    #[test]
    fn fabricated_supported_claim_from_model_rejected() {
        // The exact READ-1 exploit, emitted by a model: a fabricated claim citing
        // a real, read span. The verifier (reached only through the codec) refuses.
        let (corpus, question) = task_env();
        let adapter = Adapter::new(ScriptedBackend::new(
            output_named("fabricated_supported_claim").text,
        ));
        let (_text, decision) = adapter.run(&ReadingTask::new(&corpus, &question));
        assert_eq!(decision.unwrap_err().kind(), RejectKind::Unverified);
    }

    #[test]
    fn verbatim_grounded_claim_from_model_accepted() {
        // A verbatim, source-grounded model output finalizes a verifier-approved
        // answer (the codec ran the substrate + READ-1 verifier internally; the
        // adapter itself never touches them).
        let (corpus, question) = task_env();
        let adapter = Adapter::new(ScriptedBackend::new(
            output_named("verbatim_grounded_full_sequence").text,
        ));
        let (_text, decision) = adapter.run(&ReadingTask::new(&corpus, &question));
        let run = decision
            .unwrap()
            .finalized
            .expect("verbatim grounded sequence finalizes");
        assert_eq!(
            run.proof.answer_text,
            "Bridge B remained passable during light rain on the same day. \
             Bridge A was reported structurally damaged after the June storm."
        );
    }

    #[test]
    fn adapter_cannot_call_substrate_execute_directly() {
        // Structural guarantee (the adapter holds no execute/verify/finalize path)
        // is gate-enforced by a source scan. Behaviorally we prove the accept path
        // runs THROUGH the verifier: a fabricated-but-cited claim is rejected (a
        // direct, codec-bypassing execute would have built memory and finalized).
        let (corpus, question) = task_env();
        let adapter = Adapter::new(ScriptedBackend::new(
            output_named("fabricated_supported_claim").text,
        ));
        let (_text, decision) = adapter.run(&ReadingTask::new(&corpus, &question));
        assert!(
            matches!(decision, Err(e) if e.kind() == RejectKind::Unverified),
            "the adapter must reach the substrate only through the verifying codec"
        );
    }

    #[test]
    fn baseline_eval_report_is_deterministic() {
        let (corpus, question) = task_env();
        let a = baseline_report(&corpus, &question, &baseline_outputs());
        let b = baseline_report(&corpus, &question, &baseline_outputs());
        assert_eq!(a, b, "scripted backend + pure codec ⇒ identical report");
        // And the baseline profile is the expected shape: exactly one finalized
        // (the verbatim sequence), one partial, the rest rejected.
        assert_eq!(a.total, 7);
        assert_eq!(a.finalized, 1);
        assert_eq!(a.accepted_partial, 1);
        assert_eq!(a.rejected, 5);
        // The fabricated-but-cited claim lands in the Unverified rejection class.
        assert_eq!(
            a.entries
                .iter()
                .find(|e| e.name == "fabricated_supported_claim")
                .unwrap()
                .outcome,
            Outcome::Rejected(RejectKind::Unverified)
        );
    }
}
