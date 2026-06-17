//! READ-4 — the committed real-corpus eval fixtures.
//!
//! Each fixture is a real (small) document set + a question + an untrusted
//! reading plan + a COMMITTED expected verifier result (Verified or Rejected).
//! The expected label is authored here in source, never inferred from any model
//! output. 15 fixtures (≥ 10 required) across varied corpora: weather, medical,
//! infrastructure, finance, safety — exercising valid single/multi-span/multi-doc
//! grounding and every rejection class (fabricated, fragment, malformed,
//! metadata-first, unknown action, bad span, negation fragment, ungrounded), plus
//! the READ-5 abbreviation pair: a "U.S."-bearing sentence now grounds as a whole
//! (deterministic splitter hardening), while a fragment of it is still rejected.

/// The committed expected verifier result for a fixture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expected {
    /// The plan must finalize a verifier-approved, source-grounded answer.
    Verified,
    /// The plan must be rejected (no verified answer is produced).
    Rejected,
}

/// One real-corpus fixture: documents (filename + content), a question, an
/// untrusted plan, and the committed expected result.
pub struct CorpusFixture {
    pub name: &'static str,
    pub documents: &'static [(&'static str, &'static str)],
    pub question: &'static str,
    pub plan: &'static str,
    pub expected: Expected,
}

/// The committed fixture pack (13 fixtures).
pub fn fixtures() -> Vec<CorpusFixture> {
    vec![
        // --- valid: a verified, source-grounded answer must be produced ---
        CorpusFixture {
            name: "weather_wind_valid",
            documents: &[(
                "forecast.txt",
                "Heavy rain is expected tonight. Winds will reach forty miles per hour.",
            )],
            question: "What is the wind forecast?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":1},
                {"action":"extract_claim","statement":"Winds will reach forty miles per hour.","source_span_ids":[1]},
                {"action":"synthesize","answer_text":"Winds will reach forty miles per hour.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Verified,
        },
        CorpusFixture {
            name: "medical_test_valid",
            documents: &[(
                "note.txt",
                "The patient reported chest pain. An ECG was ordered immediately.",
            )],
            question: "What test was ordered?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":1},
                {"action":"extract_claim","statement":"An ECG was ordered immediately.","source_span_ids":[1]},
                {"action":"synthesize","answer_text":"An ECG was ordered immediately.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Verified,
        },
        CorpusFixture {
            name: "two_document_synthesis_valid",
            documents: &[
                ("a_bridge.txt", "The bridge inspection found cracks."),
                ("b_road.txt", "The road was closed for repairs."),
            ],
            question: "What happened to the bridge and road?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"read_span","span_id":1},
                {"action":"extract_claim","statement":"The bridge inspection found cracks.","source_span_ids":[0]},
                {"action":"extract_claim","statement":"The road was closed for repairs.","source_span_ids":[1]},
                {"action":"synthesize","answer_text":"The bridge inspection found cracks. The road was closed for repairs.","supporting_claims":[0,1]}
            ]"#,
            expected: Expected::Verified,
        },
        CorpusFixture {
            name: "multi_sentence_doc_valid",
            documents: &[(
                "incident.txt",
                "The alarm sounded at noon. Staff evacuated the building. No injuries were reported.",
            )],
            question: "Were there injuries?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":2},
                {"action":"extract_claim","statement":"No injuries were reported.","source_span_ids":[2]},
                {"action":"synthesize","answer_text":"No injuries were reported.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Verified,
        },
        CorpusFixture {
            name: "compare_then_synthesize_valid",
            documents: &[("sales.txt", "Sales rose in March. Sales fell in April.")],
            question: "How did sales change?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"read_span","span_id":1},
                {"action":"extract_claim","statement":"Sales rose in March.","source_span_ids":[0]},
                {"action":"extract_claim","statement":"Sales fell in April.","source_span_ids":[1]},
                {"action":"compare_claims","left":0,"right":1},
                {"action":"synthesize","answer_text":"Sales rose in March. Sales fell in April.","supporting_claims":[0,1]}
            ]"#,
            expected: Expected::Verified,
        },
        // --- must be rejected (no verified answer) ---
        CorpusFixture {
            name: "fabricated_answer_reject",
            documents: &[("status.txt", "The reactor is operating normally.")],
            question: "Is the reactor safe?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"The reactor is melting down.","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"The reactor is melting down.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "fragment_claim_reject",
            documents: &[("status.txt", "The reactor is operating normally.")],
            question: "Is the reactor safe?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"The reactor","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"The reactor","supporting_claims":[0]}
            ]"#,
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "malformed_plan_reject",
            documents: &[("status.txt", "The reactor is operating normally.")],
            question: "Is the reactor safe?",
            plan: "the reactor seems fine to me",
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "metadata_before_read_reject",
            documents: &[("status.txt", "The reactor is operating normally.")],
            question: "Is the reactor safe?",
            plan: r#"[
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"The reactor is operating normally.","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"The reactor is operating normally.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "unknown_action_reject",
            documents: &[("status.txt", "The reactor is operating normally.")],
            question: "Is the reactor safe?",
            plan: r#"[{"action":"decide","verdict":"safe"}]"#,
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "nonexistent_span_reject",
            documents: &[("status.txt", "The reactor is operating normally.")],
            question: "Is the reactor safe?",
            plan: r#"[{"action":"inspect_corpus"},{"action":"read_span","span_id":99}]"#,
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "negation_dropped_fragment_reject",
            documents: &[("advice.txt", "Do not cross the river during the flood.")],
            question: "Can I cross the river?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"cross the river during the flood.","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"cross the river during the flood.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Rejected,
        },
        CorpusFixture {
            name: "ungrounded_claim_reject",
            documents: &[("misc.txt", "Something happened here.")],
            question: "What happened?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"extract_claim","statement":"An invented fact.","source_span_ids":[]}
            ]"#,
            expected: Expected::Rejected,
        },
        // READ-5: the deterministic splitter now keeps abbreviations together, so
        // "The U.S. economy is strong this year." is ONE span and the whole
        // sentence grounds correctly.
        CorpusFixture {
            name: "abbreviation_whole_sentence_valid",
            documents: &[("econ.txt", "The U.S. economy is strong this year.")],
            question: "How is the economy?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"The U.S. economy is strong this year.","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"The U.S. economy is strong this year.","supporting_claims":[0]}
            ]"#,
            expected: Expected::Verified,
        },
        // READ-5: keeping the abbreviation sentence whole does NOT open a
        // false-grounded hole — a fragment of that single span is still not a
        // complete sentence-level unit and is rejected.
        CorpusFixture {
            name: "abbreviation_sentence_fragment_reject",
            documents: &[("econ.txt", "The U.S. economy is strong this year.")],
            question: "How is the economy?",
            plan: r#"[
                {"action":"inspect_corpus"},
                {"action":"read_span","span_id":0},
                {"action":"extract_claim","statement":"The U.S. economy","source_span_ids":[0]},
                {"action":"synthesize","answer_text":"The U.S. economy","supporting_claims":[0]}
            ]"#,
            expected: Expected::Rejected,
        },
    ]
}
