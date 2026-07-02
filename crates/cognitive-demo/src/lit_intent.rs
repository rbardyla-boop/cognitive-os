//! LIT-INTENT-0 — deterministic literature intent maps from verified QFLOW spans.
//!
//! This is the first visible "comprehension/teaching" slice on top of QFLOW-0.
//! It consumes a [`VerifiedQueryFlow`] packet, maps only what verified spans can
//! support, and emits a bounded [`LiteratureIntentMap`]. It does NOT infer hidden
//! author motives, does NOT claim full understanding, and does NOT teach unsupported
//! claims. Missing thesis/intent/definition/tension evidence is recorded as a
//! field-level refusal inside an otherwise built map when enough verified spans
//! exist; terminal refusals are reserved for missing inputs or a refused QFLOW path.
//!
//! LIT-INTENT-0 is deliberately lexical and deterministic: marker phrases such as
//! "central thesis", "purpose", "means", "assumes", and "however" decide which
//! verified span is eligible for each map slot. The marker is a ROUTING signal, not
//! evidence. The evidence is still the verified span text and its source id.

use serde::Serialize;

use crate::{
    run_query, VerifiedEvidenceItem, VerifiedQueryConfig, VerifiedQueryDecision, VerifiedQueryFlow,
    VerifiedQueryRefusal,
};

const SCHEMA: &str = "literature-intent-map-v0.1";
const LIT_INTENT_USES_MODEL: bool = false;
const LIT_INTENT_USES_TRAINING: bool = false;
const DEFAULT_INTENT_SPAN_BUDGET: usize = 8;
const AUTHORITY_INTENT_MAP_FROM_VERIFIED_SPAN: &str = "intent_map_from_verified_span";

/// The authority boundary, verbatim. LIT-INTENT maps; it never promotes.
pub const LIT_INTENT_BOUNDARY_LINES: [&str; 9] = [
    "LIT-INTENT-0 maps verified spans only.",
    "It does not infer hidden author motives.",
    "It does not claim full comprehension.",
    "It does not teach unsupported claims.",
    "It does not create evidence.",
    "It does not answer from scores.",
    "It does not change grounding rules.",
    "It does not train or run a model.",
    "It does not retag v0.1.",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LiteratureIntentDecision {
    IntentMapBuilt,
    IntentMapRefused,
}

impl LiteratureIntentDecision {
    pub const ALL: [LiteratureIntentDecision; 2] = [
        LiteratureIntentDecision::IntentMapBuilt,
        LiteratureIntentDecision::IntentMapRefused,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            LiteratureIntentDecision::IntentMapBuilt => "intent_map_built",
            LiteratureIntentDecision::IntentMapRefused => "intent_map_refused",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LiteratureIntentRefusal {
    EmptyFocusQuestion,
    EmptyDocumentSet,
    QueryFlowRefused,
    NoVerifiedPacket,
    NoVerifiedSpans,
    ModelSignalDetected,
    TrainingSignalDetected,
    SerializedIntentMapTamper,
}

impl LiteratureIntentRefusal {
    pub const ALL: [LiteratureIntentRefusal; 8] = [
        LiteratureIntentRefusal::EmptyFocusQuestion,
        LiteratureIntentRefusal::EmptyDocumentSet,
        LiteratureIntentRefusal::QueryFlowRefused,
        LiteratureIntentRefusal::NoVerifiedPacket,
        LiteratureIntentRefusal::NoVerifiedSpans,
        LiteratureIntentRefusal::ModelSignalDetected,
        LiteratureIntentRefusal::TrainingSignalDetected,
        LiteratureIntentRefusal::SerializedIntentMapTamper,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            LiteratureIntentRefusal::EmptyFocusQuestion => "empty_focus_question_refused",
            LiteratureIntentRefusal::EmptyDocumentSet => "empty_document_set_refused",
            LiteratureIntentRefusal::QueryFlowRefused => "query_flow_refused",
            LiteratureIntentRefusal::NoVerifiedPacket => "no_verified_packet_refused",
            LiteratureIntentRefusal::NoVerifiedSpans => "no_verified_spans_refused",
            LiteratureIntentRefusal::ModelSignalDetected => "model_signal_detected_refused",
            LiteratureIntentRefusal::TrainingSignalDetected => "training_signal_detected_refused",
            LiteratureIntentRefusal::SerializedIntentMapTamper => {
                "serialized_intent_map_tamper_refused"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteratureIntentError {
    ReplayMismatch,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LiteratureIntentConfig {
    pub max_candidates: usize,
    pub min_term_len: usize,
    pub phrase_bonus: usize,
    pub title_boost_per_term: usize,
    pub heading_boost_per_term: usize,
    pub uses_model: bool,
    pub uses_training: bool,
}

impl LiteratureIntentConfig {
    pub fn default_config() -> Self {
        let qflow = VerifiedQueryConfig::default_config();
        LiteratureIntentConfig {
            max_candidates: DEFAULT_INTENT_SPAN_BUDGET,
            min_term_len: qflow.min_term_len,
            phrase_bonus: qflow.phrase_bonus,
            title_boost_per_term: qflow.title_boost_per_term,
            heading_boost_per_term: qflow.heading_boost_per_term,
            uses_model: LIT_INTENT_USES_MODEL,
            uses_training: LIT_INTENT_USES_TRAINING,
        }
    }

    fn to_qflow(self) -> VerifiedQueryConfig {
        VerifiedQueryConfig {
            max_candidates: self.max_candidates,
            min_term_len: self.min_term_len,
            phrase_bonus: self.phrase_bonus,
            title_boost_per_term: self.title_boost_per_term,
            heading_boost_per_term: self.heading_boost_per_term,
            uses_model: self.uses_model,
            uses_training: self.uses_training,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LiteratureIntentBoundary {
    pub infers_hidden_author_motives: bool,
    pub claims_full_comprehension: bool,
    pub teaches_unsupported_claims: bool,
    pub creates_evidence: bool,
    pub answers_from_scores: bool,
    pub changes_grounding_rules: bool,
    pub trains: bool,
    pub is_model: bool,
    pub retags_release: bool,
}

impl LiteratureIntentBoundary {
    fn inert() -> Self {
        LiteratureIntentBoundary {
            infers_hidden_author_motives: LIT_INTENT_USES_MODEL,
            claims_full_comprehension: LIT_INTENT_USES_MODEL,
            teaches_unsupported_claims: LIT_INTENT_USES_MODEL,
            creates_evidence: LIT_INTENT_USES_MODEL,
            answers_from_scores: LIT_INTENT_USES_MODEL,
            changes_grounding_rules: LIT_INTENT_USES_MODEL,
            trains: LIT_INTENT_USES_TRAINING,
            is_model: LIT_INTENT_USES_MODEL,
            retags_release: LIT_INTENT_USES_MODEL,
        }
    }

    fn all_inert(&self) -> bool {
        !self.infers_hidden_author_motives
            && !self.claims_full_comprehension
            && !self.teaches_unsupported_claims
            && !self.creates_evidence
            && !self.answers_from_scores
            && !self.changes_grounding_rules
            && !self.trains
            && !self.is_model
            && !self.retags_release
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteratureIntentRequest {
    pub focus_question: String,
    pub documents: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntentSpanRef {
    pub document_id: u64,
    pub document_name: String,
    pub span_id: u64,
    pub text: String,
    pub authority: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SpanBackedFinding {
    pub statement: String,
    pub support: Vec<IntentSpanRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct KeyTermFinding {
    pub term: String,
    pub usage_or_definition: String,
    pub support: Vec<IntentSpanRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TeachingStep {
    pub order: usize,
    pub action: String,
    pub support: Vec<IntentSpanRef>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct IntentFieldRefusal {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LiteratureIntentMap {
    pub document: String,
    pub central_thesis: Option<SpanBackedFinding>,
    pub author_intent: Option<SpanBackedFinding>,
    pub core_claims: Vec<SpanBackedFinding>,
    pub key_terms: Vec<KeyTermFinding>,
    pub assumptions: Vec<SpanBackedFinding>,
    pub tensions_or_contradictions: Vec<SpanBackedFinding>,
    pub teaching_path: Vec<TeachingStep>,
    pub user_facing_lesson: String,
    pub refusals: Vec<IntentFieldRefusal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteratureIntentReceipt {
    pub schema: String,
    pub focus_question: String,
    pub config: LiteratureIntentConfig,
    pub qflow_receipt_hash: u64,
    pub qflow_decision: String,
    pub qflow_refusal: Option<String>,
    pub evidence_span_count: usize,
    pub decision: LiteratureIntentDecision,
    pub refusal: Option<LiteratureIntentRefusal>,
    pub receipt_hash: u64,
    pub boundary: LiteratureIntentBoundary,
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteratureIntentRun {
    pub request: LiteratureIntentRequest,
    pub receipt: LiteratureIntentReceipt,
    pub map: Option<LiteratureIntentMap>,
    pub decision: LiteratureIntentDecision,
    pub refusal: Option<LiteratureIntentRefusal>,
}

fn fnv_mix(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn fnv_u64(h: u64, v: u64) -> u64 {
    fnv_mix(h, &v.to_le_bytes())
}

#[allow(clippy::too_many_arguments)]
fn receipt_hash(
    focus_question: &str,
    config: &LiteratureIntentConfig,
    qflow_receipt_hash: u64,
    qflow_decision: &str,
    qflow_refusal: Option<&str>,
    evidence: &[IntentSpanRef],
    decision: LiteratureIntentDecision,
    refusal: Option<LiteratureIntentRefusal>,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325;
    h = fnv_mix(h, SCHEMA.as_bytes());
    h = fnv_mix(h, focus_question.as_bytes());
    h = fnv_u64(h, config.max_candidates as u64);
    h = fnv_u64(h, config.min_term_len as u64);
    h = fnv_u64(h, config.phrase_bonus as u64);
    h = fnv_u64(h, config.title_boost_per_term as u64);
    h = fnv_u64(h, config.heading_boost_per_term as u64);
    h = fnv_u64(h, config.uses_model as u64);
    h = fnv_u64(h, config.uses_training as u64);
    h = fnv_u64(h, qflow_receipt_hash);
    h = fnv_mix(h, qflow_decision.as_bytes());
    h = fnv_mix(h, qflow_refusal.unwrap_or("none").as_bytes());
    h = fnv_u64(h, evidence.len() as u64);
    for span in evidence {
        h = fnv_u64(h, span.document_id);
        h = fnv_u64(h, span.span_id);
        h = fnv_mix(h, span.document_name.as_bytes());
        h = fnv_mix(h, span.text.as_bytes());
        h = fnv_mix(h, span.authority.as_bytes());
    }
    h = fnv_mix(h, decision.slug().as_bytes());
    h = fnv_mix(h, refusal.map(|r| r.slug()).unwrap_or("none").as_bytes());
    h
}

pub fn run_literature_intent_map_default(
    documents: &[(String, String)],
    focus_question: &str,
) -> LiteratureIntentRun {
    run_literature_intent_map(
        documents,
        focus_question,
        LiteratureIntentConfig::default_config(),
    )
}

pub fn literature_intent_demo() -> LiteratureIntentRun {
    run_literature_intent_map_default(&fixture_docs(), fixture_focus())
}

pub fn literature_intent_demo_json() -> String {
    serde_json::to_string_pretty(&literature_intent_demo())
        .expect("literature intent demo serializes")
}

pub fn verify_literature_intent_demo_json(candidate: &str) -> Result<(), LiteratureIntentError> {
    if candidate == literature_intent_demo_json() {
        Ok(())
    } else {
        Err(LiteratureIntentError::ReplayMismatch)
    }
}

fn flip_last_byte(input: &str) -> String {
    let mut bytes = input.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last = last.wrapping_add(1);
    }
    String::from_utf8(bytes).expect("json stays utf8 after single-byte flip")
}

pub fn run_literature_intent_map(
    documents: &[(String, String)],
    focus_question: &str,
    config: LiteratureIntentConfig,
) -> LiteratureIntentRun {
    let request = LiteratureIntentRequest {
        focus_question: focus_question.to_string(),
        documents: documents.to_vec(),
    };

    if config.uses_model {
        return assemble(
            request,
            config,
            0,
            "none".to_string(),
            None,
            vec![],
            LiteratureIntentDecision::IntentMapRefused,
            Some(LiteratureIntentRefusal::ModelSignalDetected),
            None,
        );
    }
    if config.uses_training {
        return assemble(
            request,
            config,
            0,
            "none".to_string(),
            None,
            vec![],
            LiteratureIntentDecision::IntentMapRefused,
            Some(LiteratureIntentRefusal::TrainingSignalDetected),
            None,
        );
    }
    if !focus_question.chars().any(|c| c.is_alphanumeric()) {
        return assemble(
            request,
            config,
            0,
            "none".to_string(),
            None,
            vec![],
            LiteratureIntentDecision::IntentMapRefused,
            Some(LiteratureIntentRefusal::EmptyFocusQuestion),
            None,
        );
    }
    if documents.is_empty() {
        return assemble(
            request,
            config,
            0,
            "none".to_string(),
            None,
            vec![],
            LiteratureIntentDecision::IntentMapRefused,
            Some(LiteratureIntentRefusal::EmptyDocumentSet),
            None,
        );
    }

    let flow = run_query(documents, focus_question, config.to_qflow());
    let qflow_receipt_hash = flow.receipt.receipt_hash;
    let qflow_decision = flow.receipt.decision.slug().to_string();
    let qflow_refusal = flow.receipt.refusal.map(|r| r.slug().to_string());

    if flow.decision == VerifiedQueryDecision::QueryRefused {
        return assemble(
            request,
            config,
            qflow_receipt_hash,
            qflow_decision,
            qflow_refusal,
            vec![],
            LiteratureIntentDecision::IntentMapRefused,
            Some(map_qflow_refusal(flow.refusal)),
            None,
        );
    }

    let packet = match flow.packet.as_ref() {
        Some(packet) => packet,
        None => {
            return assemble(
                request,
                config,
                qflow_receipt_hash,
                qflow_decision,
                qflow_refusal,
                vec![],
                LiteratureIntentDecision::IntentMapRefused,
                Some(LiteratureIntentRefusal::NoVerifiedPacket),
                None,
            );
        }
    };

    if packet.items.is_empty() {
        return assemble(
            request,
            config,
            qflow_receipt_hash,
            qflow_decision,
            qflow_refusal,
            vec![],
            LiteratureIntentDecision::IntentMapRefused,
            Some(LiteratureIntentRefusal::NoVerifiedSpans),
            None,
        );
    }

    let evidence = packet.items.iter().map(intent_span_ref).collect::<Vec<_>>();
    let map = build_map(&flow, &evidence);
    assemble(
        request,
        config,
        qflow_receipt_hash,
        qflow_decision,
        qflow_refusal,
        evidence,
        LiteratureIntentDecision::IntentMapBuilt,
        None,
        Some(map),
    )
}

fn map_qflow_refusal(refusal: Option<VerifiedQueryRefusal>) -> LiteratureIntentRefusal {
    match refusal {
        Some(VerifiedQueryRefusal::ModelSignalDetected) => {
            LiteratureIntentRefusal::ModelSignalDetected
        }
        Some(VerifiedQueryRefusal::TrainingSignalDetected) => {
            LiteratureIntentRefusal::TrainingSignalDetected
        }
        Some(VerifiedQueryRefusal::EmptyDocumentSet) => LiteratureIntentRefusal::EmptyDocumentSet,
        Some(VerifiedQueryRefusal::EmptyQuestion) => LiteratureIntentRefusal::EmptyFocusQuestion,
        _ => LiteratureIntentRefusal::QueryFlowRefused,
    }
}

#[allow(clippy::too_many_arguments)]
fn assemble(
    request: LiteratureIntentRequest,
    config: LiteratureIntentConfig,
    qflow_receipt_hash: u64,
    qflow_decision: String,
    qflow_refusal: Option<String>,
    evidence: Vec<IntentSpanRef>,
    decision: LiteratureIntentDecision,
    refusal: Option<LiteratureIntentRefusal>,
    map: Option<LiteratureIntentMap>,
) -> LiteratureIntentRun {
    let receipt_hash = receipt_hash(
        &request.focus_question,
        &config,
        qflow_receipt_hash,
        &qflow_decision,
        qflow_refusal.as_deref(),
        &evidence,
        decision,
        refusal,
    );
    let boundary = LiteratureIntentBoundary::inert();
    let receipt = LiteratureIntentReceipt {
        schema: SCHEMA.to_string(),
        focus_question: request.focus_question.clone(),
        config,
        qflow_receipt_hash,
        qflow_decision,
        qflow_refusal,
        evidence_span_count: evidence.len(),
        decision,
        refusal,
        receipt_hash,
        boundary,
        boundary_all_inert: boundary.all_inert(),
    };
    LiteratureIntentRun {
        request,
        receipt,
        map,
        decision,
        refusal,
    }
}

fn intent_span_ref(item: &VerifiedEvidenceItem) -> IntentSpanRef {
    IntentSpanRef {
        document_id: item.document_id,
        document_name: item.document_name.clone(),
        span_id: item.span_id,
        text: item.verified_text.clone(),
        authority: AUTHORITY_INTENT_MAP_FROM_VERIFIED_SPAN.to_string(),
    }
}

fn build_map(flow: &VerifiedQueryFlow, evidence: &[IntentSpanRef]) -> LiteratureIntentMap {
    let central = first_matching(evidence, is_central_thesis_marker);
    let author_intent =
        first_matching(evidence, is_author_intent_marker).map(|finding| SpanBackedFinding {
            statement: format!(
                "Bounded intent from explicit wording: {}",
                finding.statement
            ),
            support: finding.support,
        });
    let core_claims = evidence
        .iter()
        .map(|span| SpanBackedFinding {
            statement: span.text.clone(),
            support: vec![span.clone()],
        })
        .collect::<Vec<_>>();
    let key_terms = evidence
        .iter()
        .filter_map(extract_key_term)
        .collect::<Vec<_>>();
    let assumptions = evidence
        .iter()
        .filter(|span| is_assumption_marker(&span.text))
        .map(|span| SpanBackedFinding {
            statement: span.text.clone(),
            support: vec![span.clone()],
        })
        .collect::<Vec<_>>();
    let tensions = evidence
        .iter()
        .filter(|span| is_tension_marker(&span.text))
        .map(|span| SpanBackedFinding {
            statement: span.text.clone(),
            support: vec![span.clone()],
        })
        .collect::<Vec<_>>();

    let mut refusals = Vec::new();
    if central.is_none() {
        refusals.push(field_refusal(
            "central_thesis",
            "no verified span used an explicit thesis/main-argument marker",
        ));
    }
    if author_intent.is_none() {
        refusals.push(field_refusal(
            "author_intent",
            "no explicit purpose/aim/intent span was verified; hidden motive not inferred",
        ));
    }
    if key_terms.is_empty() {
        refusals.push(field_refusal(
            "key_terms",
            "no verified span used a deterministic definition marker such as means/refers to/defined as",
        ));
    }
    if assumptions.is_empty() {
        refusals.push(field_refusal(
            "assumptions",
            "no explicit assumes/depends-on/requires marker was verified",
        ));
    }
    if tensions.is_empty() {
        refusals.push(field_refusal(
            "tensions_or_contradictions",
            "no explicit however/but/although/tension/contradiction marker was verified",
        ));
    }
    refusals.push(field_refusal(
        "hidden_author_motives",
        "not inferred by LIT-INTENT-0",
    ));
    refusals.push(field_refusal(
        "full_comprehension",
        "not claimed by LIT-INTENT-0",
    ));

    let teaching_path = teaching_path(&central, &key_terms, &core_claims, &tensions);
    let user_facing_lesson = lesson(&central, &author_intent, &key_terms, &core_claims);
    LiteratureIntentMap {
        document: document_label(flow, evidence),
        central_thesis: central,
        author_intent,
        core_claims,
        key_terms,
        assumptions,
        tensions_or_contradictions: tensions,
        teaching_path,
        user_facing_lesson,
        refusals,
    }
}

fn first_matching(
    evidence: &[IntentSpanRef],
    predicate: fn(&str) -> bool,
) -> Option<SpanBackedFinding> {
    evidence
        .iter()
        .find(|span| predicate(&span.text))
        .map(|span| SpanBackedFinding {
            statement: span.text.clone(),
            support: vec![span.clone()],
        })
}

fn document_label(flow: &VerifiedQueryFlow, evidence: &[IntentSpanRef]) -> String {
    let mut names = evidence
        .iter()
        .map(|span| span.document_name.clone())
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    if names.len() == 1 {
        names[0].clone()
    } else if names.is_empty() {
        flow.request
            .documents
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        format!("pack: {}", names.join(", "))
    }
}

fn field_refusal(field: &str, reason: &str) -> IntentFieldRefusal {
    IntentFieldRefusal {
        field: field.to_string(),
        reason: reason.to_string(),
    }
}

fn contains_any(text: &str, markers: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    markers.iter().any(|marker| lower.contains(marker))
}

fn is_central_thesis_marker(text: &str) -> bool {
    contains_any(
        text,
        &[
            "central thesis",
            "main argument",
            "core argument",
            "the thesis",
            "this essay argues",
            "this text argues",
            "the article argues",
            "the author argues",
            "we argue",
            "i argue",
        ],
    )
}

fn is_author_intent_marker(text: &str) -> bool {
    contains_any(
        text,
        &[
            "purpose",
            "aim ",
            "aims ",
            "goal",
            "intends",
            "intention",
            "tries to show",
            "seeks to",
            "designed to",
            "teaches",
        ],
    )
}

fn is_assumption_marker(text: &str) -> bool {
    contains_any(
        text,
        &[
            "assumes",
            "assumption",
            "depends on",
            "requires",
            "relies on",
        ],
    )
}

fn is_tension_marker(text: &str) -> bool {
    contains_any(
        text,
        &[
            "however",
            " but ",
            "although",
            " yet ",
            "tension",
            "contradiction",
            "conflict",
        ],
    )
}

fn extract_key_term(span: &IntentSpanRef) -> Option<KeyTermFinding> {
    let lower = span.text.to_ascii_lowercase();
    let markers = [" is defined as ", " means ", " refers to "];
    for marker in markers {
        if let Some(idx) = lower.find(marker) {
            let raw_term = span.text[..idx]
                .trim()
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
            let term = raw_term
                .strip_prefix("The ")
                .or_else(|| raw_term.strip_prefix("the "))
                .unwrap_or(raw_term)
                .trim();
            if term.len() >= 3 && term.split_whitespace().count() <= 4 {
                return Some(KeyTermFinding {
                    term: term.to_string(),
                    usage_or_definition: span.text.clone(),
                    support: vec![span.clone()],
                });
            }
        }
    }
    None
}

fn teaching_path(
    central: &Option<SpanBackedFinding>,
    key_terms: &[KeyTermFinding],
    core_claims: &[SpanBackedFinding],
    tensions: &[SpanBackedFinding],
) -> Vec<TeachingStep> {
    let mut steps = Vec::new();
    if let Some(thesis) = central {
        steps.push(TeachingStep {
            order: steps.len() + 1,
            action: "Anchor the lesson in the verified central thesis.".to_string(),
            support: thesis.support.clone(),
        });
    } else if let Some(first) = core_claims.first() {
        steps.push(TeachingStep {
            order: steps.len() + 1,
            action: "Start with the first verified claim while marking thesis as unsupported."
                .to_string(),
            support: first.support.clone(),
        });
    }
    if let Some(term) = key_terms.first() {
        steps.push(TeachingStep {
            order: steps.len() + 1,
            action: format!(
                "Define '{}' from its verified usage before expanding.",
                term.term
            ),
            support: term.support.clone(),
        });
    }
    if core_claims.len() > 1 {
        steps.push(TeachingStep {
            order: steps.len() + 1,
            action: "Compare the verified claims in source-backed order.".to_string(),
            support: core_claims
                .iter()
                .take(2)
                .flat_map(|claim| claim.support.clone())
                .collect(),
        });
    }
    if let Some(tension) = tensions.first() {
        steps.push(TeachingStep {
            order: steps.len() + 1,
            action: "Inspect the verified tension before drawing a conclusion.".to_string(),
            support: tension.support.clone(),
        });
    }
    steps.push(TeachingStep {
        order: steps.len() + 1,
        action: "End by asking which claims still lack verified span support.".to_string(),
        support: vec![],
    });
    steps
}

fn lesson(
    central: &Option<SpanBackedFinding>,
    author_intent: &Option<SpanBackedFinding>,
    key_terms: &[KeyTermFinding],
    core_claims: &[SpanBackedFinding],
) -> String {
    let mut parts = Vec::new();
    if let Some(thesis) = central {
        parts.push(format!(
            "Start with the verified thesis: {}",
            thesis.statement
        ));
    } else if let Some(first) = core_claims.first() {
        parts.push(format!(
            "The verified material supports this claim: {}",
            first.statement
        ));
    }
    if let Some(intent) = author_intent {
        parts.push(intent.statement.clone());
    }
    if let Some(term) = key_terms.first() {
        parts.push(format!(
            "Use '{}' according to the verified wording: {}",
            term.term, term.usage_or_definition
        ));
    }
    parts.push("Do not extend the lesson beyond the verified spans.".to_string());
    parts.join(" ")
}

pub const LIT_INTENT_SCENARIO_COUNT: usize = 11;
pub const LIT_INTENT_SCENARIO_NAMES: [&str; LIT_INTENT_SCENARIO_COUNT] = [
    "verified_document_builds_intent_map",
    "central_thesis_is_span_backed",
    "author_intent_requires_explicit_marker",
    "key_term_definition_is_span_backed",
    "teaching_path_uses_verified_support",
    "query_flow_refusal_propagates",
    "empty_focus_question_refused",
    "empty_document_set_refused",
    "no_model_signal_detected",
    "no_training_signal_detected",
    "serialized_intent_map_tamper_refused",
];

#[derive(Debug, Clone, Serialize)]
pub struct LitIntentCell {
    pub scenario: String,
    pub outcome: String,
    pub refusal: Option<String>,
    pub evidence_spans: usize,
    pub field_refusals: usize,
    pub map_built: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiteratureIntentMatrix {
    pub schema: String,
    pub scenario_count: usize,
    pub cells: Vec<LitIntentCell>,
    pub built_count: usize,
    pub refused_count: usize,
    pub boundary: LiteratureIntentBoundary,
    pub boundary_all_inert: bool,
}

fn doc(name: &str, body: &str) -> (String, String) {
    (name.to_string(), body.to_string())
}

fn fixture_docs() -> Vec<(String, String)> {
    vec![doc(
        "companion.md",
        "The central thesis is that grounded AI companionship requires verified evidence before teaching. \
The purpose is to teach the user from supported spans rather than hidden motives. \
Symbiosis means the AI and user grow through repeated grounded lessons. \
The argument assumes the system can preserve source spans for every lesson. \
However, the text warns that companionship without grounding can flatter and overclaim.",
    )]
}

fn fixture_focus() -> &'static str {
    "central thesis purpose symbiosis assumes however grounding teaching hidden motives overclaim"
}

fn cell_for(scenario: &str) -> LitIntentCell {
    match scenario {
        "verified_document_builds_intent_map"
        | "central_thesis_is_span_backed"
        | "key_term_definition_is_span_backed"
        | "teaching_path_uses_verified_support" => {
            let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
            built_cell(scenario, &run)
        }
        "author_intent_requires_explicit_marker" => {
            let docs = vec![doc(
                "plain.md",
                "The central thesis is that grounded lessons need receipts. \
Symbiosis means a repeated learning relationship with source memory.",
            )];
            let run = run_literature_intent_map_default(
                &docs,
                "central thesis symbiosis grounded lessons receipts",
            );
            built_cell(scenario, &run)
        }
        "query_flow_refusal_propagates" => {
            let docs = vec![doc("plain.md", "The bridge is open.")];
            refused_cell(
                scenario,
                &run_literature_intent_map_default(&docs, "reactor turbine"),
            )
        }
        "empty_focus_question_refused" => refused_cell(
            scenario,
            &run_literature_intent_map_default(&fixture_docs(), "   "),
        ),
        "empty_document_set_refused" => {
            let docs: Vec<(String, String)> = vec![];
            refused_cell(
                scenario,
                &run_literature_intent_map_default(&docs, "central thesis"),
            )
        }
        "no_model_signal_detected" => {
            let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
            LitIntentCell {
                scenario: scenario.to_string(),
                outcome: run.decision.slug().to_string(),
                refusal: run.refusal.map(|r| r.slug().to_string()),
                evidence_spans: run.receipt.evidence_span_count,
                field_refusals: run.map.as_ref().map(|m| m.refusals.len()).unwrap_or(0),
                map_built: run.decision == LiteratureIntentDecision::IntentMapBuilt
                    && !run.receipt.config.uses_model
                    && run.receipt.boundary_all_inert,
            }
        }
        "no_training_signal_detected" => {
            let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
            LitIntentCell {
                scenario: scenario.to_string(),
                outcome: run.decision.slug().to_string(),
                refusal: run.refusal.map(|r| r.slug().to_string()),
                evidence_spans: run.receipt.evidence_span_count,
                field_refusals: run.map.as_ref().map(|m| m.refusals.len()).unwrap_or(0),
                map_built: run.decision == LiteratureIntentDecision::IntentMapBuilt
                    && !run.receipt.config.uses_training
                    && run.receipt.boundary_all_inert,
            }
        }
        "serialized_intent_map_tamper_refused" => {
            let json = literature_intent_demo_json();
            let refused = verify_literature_intent_demo_json(&flip_last_byte(&json)).is_err();
            LitIntentCell {
                scenario: scenario.to_string(),
                outcome: if refused {
                    "tamper_refused".to_string()
                } else {
                    "tamper_not_refused".to_string()
                },
                refusal: refused
                    .then_some(LiteratureIntentRefusal::SerializedIntentMapTamper)
                    .map(|r| r.slug().to_string()),
                evidence_spans: 0,
                field_refusals: 0,
                map_built: false,
            }
        }
        other => LitIntentCell {
            scenario: other.to_string(),
            outcome: "unknown".to_string(),
            refusal: None,
            evidence_spans: 0,
            field_refusals: 0,
            map_built: false,
        },
    }
}

fn built_cell(scenario: &str, run: &LiteratureIntentRun) -> LitIntentCell {
    LitIntentCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        evidence_spans: run.receipt.evidence_span_count,
        field_refusals: run.map.as_ref().map(|m| m.refusals.len()).unwrap_or(0),
        map_built: run.decision == LiteratureIntentDecision::IntentMapBuilt,
    }
}

fn refused_cell(scenario: &str, run: &LiteratureIntentRun) -> LitIntentCell {
    LitIntentCell {
        scenario: scenario.to_string(),
        outcome: run.decision.slug().to_string(),
        refusal: run.refusal.map(|r| r.slug().to_string()),
        evidence_spans: run.receipt.evidence_span_count,
        field_refusals: 0,
        map_built: false,
    }
}

pub fn literature_intent_matrix() -> LiteratureIntentMatrix {
    let cells = LIT_INTENT_SCENARIO_NAMES
        .iter()
        .map(|scenario| cell_for(scenario))
        .collect::<Vec<_>>();
    let built_count = cells.iter().filter(|cell| cell.map_built).count();
    let refused_count = cells.iter().filter(|cell| !cell.map_built).count();
    LiteratureIntentMatrix {
        schema: SCHEMA.to_string(),
        scenario_count: LIT_INTENT_SCENARIO_COUNT,
        cells,
        built_count,
        refused_count,
        boundary: LiteratureIntentBoundary::inert(),
        boundary_all_inert: LiteratureIntentBoundary::inert().all_inert(),
    }
}

pub fn literature_intent_matrix_json() -> String {
    serde_json::to_string(&literature_intent_matrix()).expect("literature intent matrix serializes")
}

pub fn verify_literature_intent_matrix_json(candidate: &str) -> Result<(), LiteratureIntentError> {
    if candidate == literature_intent_matrix_json() {
        Ok(())
    } else {
        Err(LiteratureIntentError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_field_refusal(map: &LiteratureIntentMap, field: &str) -> bool {
        map.refusals.iter().any(|r| r.field == field)
    }

    #[test]
    fn verified_document_builds_intent_map() {
        let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
        assert_eq!(run.decision, LiteratureIntentDecision::IntentMapBuilt);
        assert!(run.refusal.is_none());
        let map = run.map.expect("map is built");
        assert_eq!(map.document, "companion.md");
        assert!(map.central_thesis.is_some());
        assert!(map.author_intent.is_some());
        assert!(!map.core_claims.is_empty());
        assert!(!map.key_terms.is_empty());
        assert!(!map.assumptions.is_empty());
        assert!(!map.tensions_or_contradictions.is_empty());
    }

    #[test]
    fn central_thesis_is_span_backed() {
        let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
        let thesis = run
            .map
            .as_ref()
            .and_then(|m| m.central_thesis.as_ref())
            .expect("central thesis");
        assert_eq!(thesis.support.len(), 1);
        assert!(thesis.statement.contains("central thesis"));
        assert_eq!(
            thesis.support[0].authority,
            AUTHORITY_INTENT_MAP_FROM_VERIFIED_SPAN
        );
    }

    #[test]
    fn author_intent_requires_explicit_marker() {
        let docs = vec![doc(
            "plain.md",
            "The central thesis is that grounded lessons need receipts. \
Symbiosis means a repeated learning relationship with source memory.",
        )];
        let run = run_literature_intent_map_default(
            &docs,
            "central thesis symbiosis grounded lessons receipts",
        );
        let map = run.map.expect("map still builds from verified spans");
        assert!(map.author_intent.is_none());
        assert!(has_field_refusal(&map, "author_intent"));
        assert!(map
            .refusals
            .iter()
            .any(|r| r.reason.contains("hidden motive not inferred")));
    }

    #[test]
    fn key_term_definition_is_span_backed() {
        let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
        let map = run.map.expect("map");
        let term = map
            .key_terms
            .iter()
            .find(|t| t.term == "Symbiosis")
            .expect("symbiosis definition");
        assert!(term.usage_or_definition.contains("means"));
        assert_eq!(term.support.len(), 1);
    }

    #[test]
    fn teaching_path_uses_verified_support() {
        let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
        let map = run.map.expect("map");
        assert!(map
            .teaching_path
            .iter()
            .any(|s| s.action.contains("verified central thesis")));
        assert!(map
            .teaching_path
            .iter()
            .filter(|s| s.action != "End by asking which claims still lack verified span support.")
            .all(|s| s
                .support
                .iter()
                .all(|r| r.authority == AUTHORITY_INTENT_MAP_FROM_VERIFIED_SPAN)));
        assert!(map
            .user_facing_lesson
            .contains("Do not extend the lesson beyond the verified spans."));
    }

    #[test]
    fn non_terminal_teaching_steps_require_verified_support() {
        let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
        let map = run.map.expect("map");
        for step in map
            .teaching_path
            .iter()
            .filter(|s| s.action != "End by asking which claims still lack verified span support.")
        {
            assert!(
                !step.support.is_empty(),
                "non-terminal teaching step must cite verified support: {}",
                step.action
            );
            assert!(step
                .support
                .iter()
                .all(|r| r.authority == AUTHORITY_INTENT_MAP_FROM_VERIFIED_SPAN));
        }
    }

    #[test]
    fn query_flow_refusal_propagates() {
        let docs = vec![doc("plain.md", "The bridge is open.")];
        let run = run_literature_intent_map_default(&docs, "reactor turbine");
        assert_eq!(run.decision, LiteratureIntentDecision::IntentMapRefused);
        assert_eq!(run.refusal, Some(LiteratureIntentRefusal::QueryFlowRefused));
        assert!(run.map.is_none());
        assert_eq!(run.receipt.qflow_decision, "query_refused");
    }

    #[test]
    fn empty_focus_and_docs_refuse() {
        let run = run_literature_intent_map_default(&fixture_docs(), "   ");
        assert_eq!(
            run.refusal,
            Some(LiteratureIntentRefusal::EmptyFocusQuestion)
        );
        let docs: Vec<(String, String)> = vec![];
        let run = run_literature_intent_map_default(&docs, "central thesis");
        assert_eq!(run.refusal, Some(LiteratureIntentRefusal::EmptyDocumentSet));
    }

    #[test]
    fn model_and_training_signals_are_refused() {
        let mut cfg = LiteratureIntentConfig::default_config();
        cfg.uses_model = true;
        let run = run_literature_intent_map(&fixture_docs(), fixture_focus(), cfg);
        assert_eq!(
            run.refusal,
            Some(LiteratureIntentRefusal::ModelSignalDetected)
        );

        let mut cfg = LiteratureIntentConfig::default_config();
        cfg.uses_training = true;
        let run = run_literature_intent_map(&fixture_docs(), fixture_focus(), cfg);
        assert_eq!(
            run.refusal,
            Some(LiteratureIntentRefusal::TrainingSignalDetected)
        );
    }

    #[test]
    fn boundary_is_inert_and_recorded() {
        let run = run_literature_intent_map_default(&fixture_docs(), fixture_focus());
        assert!(run.receipt.boundary_all_inert);
        assert_eq!(LIT_INTENT_BOUNDARY_LINES.len(), 9);
        assert_eq!(
            LIT_INTENT_BOUNDARY_LINES[0],
            "LIT-INTENT-0 maps verified spans only."
        );
        assert_eq!(
            LIT_INTENT_BOUNDARY_LINES[1],
            "It does not infer hidden author motives."
        );
    }

    #[test]
    fn matrix_has_named_scenarios_and_replays() {
        let matrix = literature_intent_matrix();
        assert_eq!(matrix.scenario_count, LIT_INTENT_SCENARIO_COUNT);
        assert_eq!(matrix.cells.len(), LIT_INTENT_SCENARIO_COUNT);
        assert_eq!(
            matrix.built_count + matrix.refused_count,
            LIT_INTENT_SCENARIO_COUNT
        );
        for (cell, name) in matrix.cells.iter().zip(LIT_INTENT_SCENARIO_NAMES.iter()) {
            assert_eq!(&cell.scenario, name);
            assert_ne!(cell.outcome, "unknown");
        }
        let json = literature_intent_matrix_json();
        assert!(verify_literature_intent_matrix_json(&json).is_ok());
        assert_eq!(
            verify_literature_intent_matrix_json(&format!("{json} ")),
            Err(LiteratureIntentError::ReplayMismatch)
        );
    }

    #[test]
    fn demo_json_re_derives_and_refuses_tampering() {
        let json = literature_intent_demo_json();
        assert!(json.contains("\"central_thesis\""));
        assert!(verify_literature_intent_demo_json(&json).is_ok());
        assert_eq!(
            verify_literature_intent_demo_json(&format!("{json} ")),
            Err(LiteratureIntentError::ReplayMismatch)
        );
    }

    #[test]
    fn serialized_tamper_scenario_constructs_refusal() {
        let matrix = literature_intent_matrix();
        let cell = matrix
            .cells
            .iter()
            .find(|c| c.scenario == "serialized_intent_map_tamper_refused")
            .expect("tamper scenario");
        assert_eq!(cell.outcome, "tamper_refused");
        assert_eq!(
            cell.refusal.as_deref(),
            Some("serialized_intent_map_tamper_refused")
        );

        let json = literature_intent_demo_json();
        assert!(verify_literature_intent_demo_json(&flip_last_byte(&json)).is_err());
    }

    #[test]
    fn decisions_and_refusals_are_complete_and_slugged() {
        assert_eq!(LiteratureIntentDecision::ALL.len(), 2);
        assert_eq!(LiteratureIntentRefusal::ALL.len(), 8);
        let mut slugs: Vec<&str> = LiteratureIntentRefusal::ALL
            .iter()
            .map(|r| r.slug())
            .collect();
        slugs.sort_unstable();
        let n = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), n);
    }
}
