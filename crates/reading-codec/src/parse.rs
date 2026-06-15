//! P9 — parse untrusted model output into typed reading-action proposals.
//!
//! Input is UNTRUSTED text — the stand-in for a future model/controller's
//! output. It must be a JSON array of action objects; anything else (prose,
//! broken JSON, a bare object) is rejected, never coerced. We dispatch on the
//! string `action` field via `serde_json::Value` so each defect gets a precise
//! reason (malformed vs. missing field vs. unknown action) instead of one
//! catch-all error. No field is ever defaulted or repaired.

use crate::error::CodecError;
use crate::policy::CodecPolicy;
use reading_substrate::{ReadingAction, SpanId};
use serde_json::Value;

/// Parse untrusted text into an ordered list of typed reading actions.
///
/// `reject_unknown` (from the policy) decides whether an unknown `action` name
/// is a hard rejection (strict) or silently dropped (a sabotaged configuration).
pub(crate) fn parse(
    untrusted: &str,
    policy: &CodecPolicy,
) -> Result<Vec<ReadingAction>, CodecError> {
    let value: Value = serde_json::from_str(untrusted)
        .map_err(|e| CodecError::MalformedSyntax(format!("not valid JSON: {e}")))?;
    let elements = value.as_array().ok_or_else(|| {
        CodecError::MalformedSyntax("model output must be a JSON array of actions".to_string())
    })?;

    let mut actions = Vec::with_capacity(elements.len());
    for element in elements {
        let object = element.as_object().ok_or_else(|| {
            CodecError::MalformedSyntax("each action must be a JSON object".to_string())
        })?;
        let name = object
            .get("action")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                CodecError::MalformedSyntax(
                    "action object needs a string \"action\" field".to_string(),
                )
            })?;

        match parse_one(name, object) {
            Ok(action) => actions.push(action),
            Err(CodecError::UnknownAction(n)) if !policy.reject_unknown => {
                // Sabotaged config: silently drop the unknown action instead of
                // rejecting. Strict policy never reaches this arm.
                let _ = n;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(actions)
}

/// Parse a single action object whose `action` name is `name`.
fn parse_one(
    name: &str,
    object: &serde_json::Map<String, Value>,
) -> Result<ReadingAction, CodecError> {
    match name {
        "inspect_corpus" => Ok(ReadingAction::InspectCorpus),
        "read_span" => Ok(ReadingAction::ReadSpan(SpanId(u64_field(
            name, object, "span_id",
        )?))),
        "extract_claim" => Ok(ReadingAction::ExtractClaim {
            statement: string_field(name, object, "statement")?,
            source_spans: span_list_field(name, object, "source_span_ids")?,
        }),
        "extract_entity" => Ok(ReadingAction::ExtractEntity {
            name: string_field(name, object, "name")?,
            source_spans: span_list_field(name, object, "source_span_ids")?,
        }),
        "compare_claims" => Ok(ReadingAction::CompareClaims {
            left: u64_field(name, object, "left")?,
            right: u64_field(name, object, "right")?,
        }),
        "synthesize" => Ok(ReadingAction::Synthesize {
            answer_text: string_field(name, object, "answer_text")?,
            supporting_claims: u64_list_field(name, object, "supporting_claims")?,
        }),
        other => Err(CodecError::UnknownAction(other.to_string())),
    }
}

fn require<'a>(
    action: &str,
    object: &'a serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<&'a Value, CodecError> {
    object.get(field).ok_or(CodecError::MissingField {
        action: action.to_string(),
        field,
    })
}

fn u64_field(
    action: &str,
    object: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<u64, CodecError> {
    require(action, object, field)?
        .as_u64()
        .ok_or(CodecError::MalformedField {
            action: action.to_string(),
            field,
        })
}

fn string_field(
    action: &str,
    object: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<String, CodecError> {
    require(action, object, field)?
        .as_str()
        .map(str::to_string)
        .ok_or(CodecError::MalformedField {
            action: action.to_string(),
            field,
        })
}

fn u64_list_field(
    action: &str,
    object: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<Vec<u64>, CodecError> {
    let array = require(action, object, field)?
        .as_array()
        .ok_or(CodecError::MalformedField {
            action: action.to_string(),
            field,
        })?;
    let mut out = Vec::with_capacity(array.len());
    for element in array {
        out.push(element.as_u64().ok_or(CodecError::MalformedField {
            action: action.to_string(),
            field,
        })?);
    }
    Ok(out)
}

fn span_list_field(
    action: &str,
    object: &serde_json::Map<String, Value>,
    field: &'static str,
) -> Result<Vec<SpanId>, CodecError> {
    Ok(u64_list_field(action, object, field)?
        .into_iter()
        .map(SpanId)
        .collect())
}
