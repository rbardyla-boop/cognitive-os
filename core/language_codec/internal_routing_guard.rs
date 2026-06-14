//! Guard against natural-language internal routing.

pub const FORBIDDEN_PROSE_FIELDS: &[&str] = &[
    "instruction",
    "instructions",
    "message_to_engine",
    "natural_language_instruction",
    "prompt",
];

pub fn is_forbidden_internal_field(field: &str) -> bool {
    FORBIDDEN_PROSE_FIELDS.contains(&field)
}
