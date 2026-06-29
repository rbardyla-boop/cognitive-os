//! VAULT-NORM-0 — deterministic Markdown normalization ADAPTER.
//!
//! Improves raw-Markdown → corpus INPUT FIDELITY before corpus construction. The
//! claim is strictly **"better Markdown-to-corpus input fidelity," never "better
//! reading."** This module lives ONLY in `cognitive-demo` (the growable crate). It
//! does NOT change `reading-substrate`, does NOT change the sentence splitter, does
//! NOT train, runs no model, and never releases or retags.
//!
//! `normalize_markdown` only DELETES markup, STRIPS leading markers, UNWRAPS links,
//! and APPENDS a terminal period — it INVENTS no text. Grounding stays sound by
//! construction: the corpus is built from the normalized text, so the FROZEN
//! `reading_substrate::verify` still grounds verbatim against it. Every measurement
//! below drives the REAL frozen `execute` + `verify` (already `cognitive-demo`
//! dependencies — no `reading-autonomy`/`reading-codec`, no Cargo change).
//!
//! Report types are `Serialize` but never `Deserialize`: a serialized matrix is
//! re-derived and byte-compared, so a tampered matrix is refused.

use serde::Serialize;

use reading_cli::{corpus_from_documents, corpus_from_spans};
use reading_substrate::{execute, verify, ReadingAction, ReadingTrace, SpanId};

/// Structural invariant: this adapter never edits the substrate. Every forbidden
/// flag is sourced from this single `false` so no path can flip one true.
const NORM_EDITS_SUBSTRATE: bool = false;

/// How many leading spans a measurement run reads (mirrors the READ-6 reader's
/// default bound, deterministically, without depending on `reading-autonomy`).
const READ_BOUND: usize = 8;

/// The deterministic normalization rules, enumerated and pinned.
pub const NORM_RULE_COUNT: usize = 16;
pub const NORM_RULE_NAMES: [&str; NORM_RULE_COUNT] = [
    "frontmatter_strip",
    "fenced_code_strip",
    "indented_code_strip",
    "table_row_drop",
    "horizontal_rule_drop",
    "heading_marker_strip",
    "blockquote_marker_strip",
    "unordered_list_marker_strip",
    "ordered_list_marker_strip",
    "wikilink_unwrap",
    "markdown_link_unwrap",
    "image_drop",
    "inline_code_strip",
    "emphasis_strip",
    "line_as_sentence_unit",
    "unbalanced_fence_guard",
];

/// The authority boundary, verbatim (8 lines).
pub const NORM_BOUNDARY_LINES: [&str; 8] = [
    "The Markdown normalizer prepares cleaner corpus input from raw Markdown.",
    "It does not change the reading substrate.",
    "It does not change the sentence splitter.",
    "It does not train or run a model.",
    "It does not release or retag.",
    "It invents no text; it only deletes markup and unwraps links.",
    "Answers remain grounded only to the normalized corpus.",
    "NormalizedInput is not better reading.",
];

// ---------------------------------------------------------------------------
// Core normalizer
// ---------------------------------------------------------------------------

/// Unwrap inline Markdown on one line. Deterministic, no semantics, pure string
/// scanning (no regex dependency). `![alt](u)`->dropped, `[[A|B]]`->B, `[[A]]`->A,
/// `[t](u)`->t, `` `code` ``->code, emphasis `** __` removed.
fn unwrap_inline(line: &str) -> String {
    let mut s = line.to_string();
    // images first: ![alt](url) -> ""
    while let Some(start) = s.find("![") {
        if let Some(close) = s[start..].find(')') {
            s.replace_range(start..start + close + 1, "");
        } else {
            break;
        }
    }
    // wikilinks [[Target|Alias]] / [[Target]] -> alias or target
    while let Some(open) = s.find("[[") {
        let Some(rel_close) = s[open..].find("]]") else {
            break;
        };
        let close = open + rel_close;
        let inner = &s[open + 2..close];
        let text = inner.rsplit('|').next().unwrap_or(inner).to_string();
        s.replace_range(open..close + 2, &text);
    }
    // markdown links [text](url) -> text
    while let Some(open) = s.find('[') {
        let Some(rel_mid) = s[open..].find("](") else {
            break;
        };
        let mid = open + rel_mid;
        let Some(rel_close) = s[mid..].find(')') else {
            break;
        };
        let close = mid + rel_close;
        let text = s[open + 1..mid].to_string();
        s.replace_range(open..close + 1, &text);
    }
    s = s.replace('`', "");
    s = s.replace("**", "").replace("__", "");
    s
}

/// Does this span text look like Markdown markup rather than prose?
fn looks_like_markup(s: &str) -> bool {
    let t = s.trim_start();
    t.starts_with('#')
        || t.starts_with("- ")
        || t.starts_with("* ")
        || t.starts_with("+ ")
        || t.starts_with('|')
        || t.starts_with('>')
        || t.starts_with("```")
        || t.starts_with("---")
        || t.starts_with("![")
        || t.contains("](")
        || t.contains("[[")
        || t.starts_with("    ")
        || t.chars().filter(|c| *c == '|').count() >= 2
}

/// Deterministic Markdown -> prose-ish text. Strips YAML frontmatter, fenced and
/// indented code, table rows and rules; removes heading/quote/list markers and
/// unwraps inline links; treats each surviving content line as a sentence unit
/// (appends a period if it lacks terminal punctuation) so the FROZEN splitter
/// yields prose spans instead of one markup blob. Pure: same input -> same output.
pub fn normalize_markdown(src: &str) -> String {
    // Unbalanced-fence guard: if fenced regions would consume >50% of non-blank
    // lines (a stray/doc-spanning fence), treating that as "all code" eats the
    // whole doc, so disable fence-stripping for it (the ``` marker lines are
    // still dropped).
    let strip_code = {
        let mut fenced = 0usize;
        let mut nonblank = 0usize;
        let mut inside = false;
        for raw in src.lines() {
            let t = raw.trim();
            if t.starts_with("```") || t.starts_with("~~~") {
                inside = !inside;
                continue;
            }
            if t.is_empty() {
                continue;
            }
            nonblank += 1;
            if inside {
                fenced += 1;
            }
        }
        nonblank == 0 || (fenced * 2) <= nonblank
    };

    let mut out: Vec<String> = Vec::new();
    let mut in_frontmatter = false;
    let mut in_code = false;
    for (i, raw) in src.lines().enumerate() {
        let trimmed = raw.trim();
        if i == 0 && trimmed == "---" {
            in_frontmatter = true;
            continue;
        }
        if in_frontmatter {
            if trimmed == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code = !in_code;
            continue; // always drop the fence marker line itself
        }
        if in_code && strip_code {
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('|') {
            continue;
        }
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            continue;
        }
        if raw.starts_with("    ") || raw.starts_with('\t') {
            continue; // indented code
        }
        // strip leading markers: heading #, quote >, list - * +, numbered "N. "
        let mut s = trimmed.trim_start_matches('#').trim_start().to_string();
        s = s.trim_start_matches('>').trim_start().to_string();
        for m in ["- ", "* ", "+ "] {
            if let Some(rest) = s.strip_prefix(m) {
                s = rest.to_string();
                break;
            }
        }
        let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() {
            if let Some(rest) = s[digits.len()..].strip_prefix(". ") {
                s = rest.to_string();
            }
        }
        let s = unwrap_inline(&s);
        let s = s.trim();
        if s.is_empty() {
            continue;
        }
        let ends_terminal = s.ends_with('.') || s.ends_with('!') || s.ends_with('?');
        if ends_terminal {
            out.push(s.to_string());
        } else {
            out.push(format!("{s}."));
        }
    }
    out.join(" ")
}

// ---------------------------------------------------------------------------
// Measurement through the REAL frozen pipeline (execute + verify)
// ---------------------------------------------------------------------------

/// One measurement of a text through the real frozen grounding path.
struct Measured {
    read_spans: usize,
    markup_spans: usize,
    finalized: bool,
    verified: bool,
}

/// Build a corpus from `text` (one sentence per span, exactly as `read0` does),
/// read up to `READ_BOUND` leading spans, claim each verbatim, synthesize, then
/// `execute` + `verify` — the genuine frozen path. `false_grounded` is
/// `finalized && !verified`.
fn measure_text(text: &str) -> Measured {
    let corpus = corpus_from_documents(&[("note.txt".to_string(), text.to_string())]);
    measure_corpus_first_spans(&corpus)
}

fn measure_corpus_first_spans(corpus: &reading_substrate::Corpus) -> Measured {
    let n = corpus.span_count().min(READ_BOUND);
    if n == 0 {
        return Measured {
            read_spans: 0,
            markup_spans: 0,
            finalized: false,
            verified: false,
        };
    }
    let mut trace = ReadingTrace::new();
    trace.push(ReadingAction::InspectCorpus);
    let mut statements: Vec<String> = Vec::new();
    let mut supporting: Vec<u64> = Vec::new();
    let mut markup = 0usize;
    for i in 0..n {
        let id = SpanId(i as u64);
        trace.push(ReadingAction::ReadSpan(id));
        let text = corpus
            .read_span(id)
            .map(|s| s.text().to_string())
            .unwrap_or_default();
        if looks_like_markup(&text) {
            markup += 1;
        }
        trace.push(ReadingAction::ExtractClaim {
            statement: text.clone(),
            source_spans: vec![id],
        });
        supporting.push(i as u64);
        statements.push(text);
    }
    let answer = statements.join(" ");
    trace.push(ReadingAction::Synthesize {
        answer_text: answer,
        supporting_claims: supporting,
    });
    match execute(corpus, "what does this note say?", &trace) {
        Ok(run) => {
            let report = verify(corpus, &run);
            Measured {
                read_spans: n,
                markup_spans: markup,
                finalized: true,
                verified: report.passed,
            }
        }
        Err(_) => Measured {
            read_spans: n,
            markup_spans: markup,
            finalized: false,
            verified: false,
        },
    }
}

// ---------------------------------------------------------------------------
// Synthetic fixtures (NO personal vault content) — reproduce real structures
// ---------------------------------------------------------------------------

/// A synthetic Markdown fixture; its label is committed in source.
#[derive(Debug, Clone)]
pub struct NormFixture {
    pub name: &'static str,
    /// True if the fixture carries real prose (so normalized markup must be 0 and
    /// it should still finalize). Pure-markup / empty docs are false.
    pub prose_bearing: bool,
    pub markdown: &'static str,
}

pub const NORM_FIXTURE_COUNT: usize = 22;

fn fixtures() -> [NormFixture; NORM_FIXTURE_COUNT] {
    [
        NormFixture { name: "frontmatter", prose_bearing: true, markdown: "---\ntitle: Note\ntags: [a, b]\n---\nThe body starts here. It has two sentences." },
        NormFixture { name: "heading_prose", prose_bearing: true, markdown: "# Big Heading\nThis is the first real sentence. Here is the second." },
        NormFixture { name: "bullet_list", prose_bearing: true, markdown: "- first item\n- second item\n- third item" },
        NormFixture { name: "ordered_list", prose_bearing: true, markdown: "1. step one\n2. step two\n3. step three" },
        NormFixture { name: "fenced_code", prose_bearing: true, markdown: "Intro line.\n```python\nx = 1\nprint(x)\n```\nClosing line." },
        NormFixture { name: "indented_code", prose_bearing: true, markdown: "A paragraph here.\n\n    indented_code_block()\n    more_code()\n\nAnother paragraph." },
        NormFixture { name: "table", prose_bearing: true, markdown: "Before the table.\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\nAfter the table." },
        NormFixture { name: "horizontal_rule", prose_bearing: true, markdown: "Section one text.\n\n---\n\nSection two text." },
        NormFixture { name: "wikilinks", prose_bearing: true, markdown: "See [[Some Note]] and [[Other|alias]] for details. Then continue here." },
        NormFixture { name: "markdown_links", prose_bearing: true, markdown: "Read [the guide](https://example.com/guide) carefully. Then proceed." },
        NormFixture { name: "image", prose_bearing: true, markdown: "![diagram](img/diagram.png)\nThe diagram explains the flow. It is important." },
        NormFixture { name: "inline_code", prose_bearing: true, markdown: "Call `do_thing()` to start. Then call `stop()` to finish." },
        NormFixture { name: "emphasis", prose_bearing: true, markdown: "This is **bold** and this is __also bold__ in the text. Done." },
        NormFixture { name: "blockquote", prose_bearing: true, markdown: "> quoted insight here\n> second quoted line" },
        NormFixture { name: "unbalanced_fence", prose_bearing: true, markdown: "```\nThe whole note is wrapped in one stray fence.\nIt is really prose, not code.\nThird prose line of the note." },
        NormFixture { name: "markup_only", prose_bearing: false, markdown: "| a | b |\n| - | - |\n| 1 | 2 |" },
        NormFixture { name: "empty", prose_bearing: false, markdown: "" },
        NormFixture { name: "prose_only", prose_bearing: true, markdown: "A clean note with no markup. Two sentences total." },
        NormFixture { name: "filename_dot", prose_bearing: true, markdown: "See drive_scout.py for details. It scans the drive." },
        NormFixture { name: "url_sentence", prose_bearing: true, markdown: "Visit https://example.com/path.html for the spec. Then return." },
        NormFixture { name: "version_number", prose_bearing: true, markdown: "We upgraded to v1.2 today. It works well." },
        NormFixture { name: "mixed_note", prose_bearing: true, markdown: "---\nkind: log\n---\n# Daily Log\n- met the team\n- shipped [[Feature X]]\n\nThe build passed. All green." },
    ]
}

// ---------------------------------------------------------------------------
// Over-split experiment + literal-token survival
// ---------------------------------------------------------------------------

/// One over-split probe sentence per token class.
const OVER_SPLIT_PROBES: [(&str, &str); 3] = [
    ("filename", "See drive_scout.py for details."),
    ("url", "Visit https://example.com/path.html now."),
    ("version", "We upgraded to v1.2 today."),
];

/// Measure whether the adapter can resolve a token's over-split WITHOUT semantic
/// leakage, by supplying the protected sentence as ONE verbatim span via
/// `corpus_from_spans` and running it through the real frozen `execute`+`verify`.
/// Resolved == the whole-span claim verifies (the frozen splitter accepts it as a
/// single sentence unit). Per the operator: MEASURE, do not assume.
fn over_split_resolved(class: &str) -> bool {
    let sentence = OVER_SPLIT_PROBES
        .iter()
        .find(|(c, _)| *c == class)
        .map(|(_, s)| *s)
        .unwrap_or("");
    let corpus = corpus_from_spans(&[("note.txt".to_string(), vec![sentence.to_string()])]);
    let id = SpanId(0);
    if !corpus.contains(id) {
        return false;
    }
    let mut trace = ReadingTrace::new();
    trace.push(ReadingAction::InspectCorpus);
    trace.push(ReadingAction::ReadSpan(id));
    let text = corpus
        .read_span(id)
        .map(|s| s.text().to_string())
        .unwrap_or_default();
    trace.push(ReadingAction::ExtractClaim {
        statement: text.clone(),
        source_spans: vec![id],
    });
    trace.push(ReadingAction::Synthesize {
        answer_text: text,
        supporting_claims: vec![0],
    });
    match execute(&corpus, "q", &trace) {
        Ok(run) => verify(&corpus, &run).passed,
        Err(_) => false,
    }
}

/// The no-semantic-leakage guard: these tokens must survive VERBATIM in normalized
/// output (the normalizer must never rewrite token text, e.g. `.py` -> ` dot py`).
const SURVIVAL_TOKENS: [&str; 4] = [
    "drive_scout.py",
    "https://example.com/path.html",
    "v1.2",
    "U.S.",
];

/// True iff every survival token appears verbatim in the normalized output of a
/// sentence that contains it.
pub fn literal_tokens_survive() -> bool {
    SURVIVAL_TOKENS.iter().all(|tok| {
        let src = format!("A line with {tok} inside it.");
        normalize_markdown(&src).contains(tok)
    })
}

/// Overall: did the adapter resolve over-split for ALL probed classes (measured).
pub fn over_split_resolved_by_adapter() -> bool {
    OVER_SPLIT_PROBES
        .iter()
        .all(|(class, _)| over_split_resolved(class))
}

// ---------------------------------------------------------------------------
// Report types — Serialize but NEVER Deserialize
// ---------------------------------------------------------------------------

const SCHEMA: &str = "markdown-normalization-v0.1";

#[derive(Debug, Clone, Serialize)]
pub struct NormCell {
    pub name: String,
    pub prose_bearing: bool,
    pub raw_read_spans: usize,
    pub raw_markup_spans: usize,
    pub raw_finalized: bool,
    pub raw_false_grounded: bool,
    pub norm_read_spans: usize,
    pub norm_markup_spans: usize,
    pub norm_finalized: bool,
    pub norm_false_grounded: bool,
}

/// Inert forbidden-action flags, every one sourced from `NORM_EDITS_SUBSTRATE`.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct NormalizationBoundary {
    pub edits_reading_substrate: bool,
    pub edits_frozen_crate: bool,
    pub changes_split_sentences: bool,
    pub trains: bool,
    pub is_model: bool,
    pub claims_better_reading: bool,
    pub retags_release: bool,
    pub is_release: bool,
}

impl NormalizationBoundary {
    fn inert() -> Self {
        NormalizationBoundary {
            edits_reading_substrate: NORM_EDITS_SUBSTRATE,
            edits_frozen_crate: NORM_EDITS_SUBSTRATE,
            changes_split_sentences: NORM_EDITS_SUBSTRATE,
            trains: NORM_EDITS_SUBSTRATE,
            is_model: NORM_EDITS_SUBSTRATE,
            claims_better_reading: NORM_EDITS_SUBSTRATE,
            retags_release: NORM_EDITS_SUBSTRATE,
            is_release: NORM_EDITS_SUBSTRATE,
        }
    }

    fn all_inert(&self) -> bool {
        !self.edits_reading_substrate
            && !self.edits_frozen_crate
            && !self.changes_split_sentences
            && !self.trains
            && !self.is_model
            && !self.claims_better_reading
            && !self.retags_release
            && !self.is_release
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MarkdownNormalizationMatrix {
    pub schema: String,
    pub rule_count: usize,
    pub fixture_count: usize,
    pub cells: Vec<NormCell>,
    pub raw_read_total: usize,
    pub raw_markup_total: usize,
    pub norm_read_total: usize,
    pub norm_markup_total: usize,
    /// Integer permille (x1000) to avoid floating point per the purity floor.
    pub raw_markup_permille: usize,
    pub norm_markup_permille: usize,
    pub raw_false_grounded_total: usize,
    pub norm_false_grounded_total: usize,
    pub over_split_filename_resolved: bool,
    pub over_split_url_resolved: bool,
    pub over_split_version_resolved: bool,
    pub over_split_resolved_by_adapter: bool,
    pub literal_tokens_survive: bool,
    pub boundary: NormalizationBoundary,
    /// True iff every forbidden flag on `boundary` is inert (sourced from
    /// `NORM_EDITS_SUBSTRATE = false`). Pinned by the gate.
    pub boundary_all_inert: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationError {
    ReplayMismatch,
}

fn permille(part: usize, whole: usize) -> usize {
    if whole == 0 {
        0
    } else {
        part * 1000 / whole
    }
}

/// Build the before/after matrix by running every fixture through the real frozen
/// pipeline, raw and normalized. Deterministic.
pub fn normalization_matrix() -> MarkdownNormalizationMatrix {
    let mut cells = Vec::new();
    let (mut raw_read_total, mut raw_markup_total) = (0usize, 0usize);
    let (mut norm_read_total, mut norm_markup_total) = (0usize, 0usize);
    let (mut raw_fg, mut norm_fg) = (0usize, 0usize);

    for f in fixtures() {
        let raw = measure_text(f.markdown);
        let norm = measure_text(&normalize_markdown(f.markdown));
        let raw_false = raw.finalized && !raw.verified;
        let norm_false = norm.finalized && !norm.verified;
        raw_read_total += raw.read_spans;
        raw_markup_total += raw.markup_spans;
        norm_read_total += norm.read_spans;
        norm_markup_total += norm.markup_spans;
        raw_fg += raw_false as usize;
        norm_fg += norm_false as usize;
        cells.push(NormCell {
            name: f.name.to_string(),
            prose_bearing: f.prose_bearing,
            raw_read_spans: raw.read_spans,
            raw_markup_spans: raw.markup_spans,
            raw_finalized: raw.finalized,
            raw_false_grounded: raw_false,
            norm_read_spans: norm.read_spans,
            norm_markup_spans: norm.markup_spans,
            norm_finalized: norm.finalized,
            norm_false_grounded: norm_false,
        });
    }

    let fname = over_split_resolved("filename");
    let url = over_split_resolved("url");
    let version = over_split_resolved("version");

    MarkdownNormalizationMatrix {
        schema: SCHEMA.to_string(),
        rule_count: NORM_RULE_COUNT,
        fixture_count: NORM_FIXTURE_COUNT,
        cells,
        raw_read_total,
        raw_markup_total,
        norm_read_total,
        norm_markup_total,
        raw_markup_permille: permille(raw_markup_total, raw_read_total),
        norm_markup_permille: permille(norm_markup_total, norm_read_total),
        raw_false_grounded_total: raw_fg,
        norm_false_grounded_total: norm_fg,
        over_split_filename_resolved: fname,
        over_split_url_resolved: url,
        over_split_version_resolved: version,
        over_split_resolved_by_adapter: fname && url && version,
        literal_tokens_survive: literal_tokens_survive(),
        boundary: NormalizationBoundary::inert(),
        boundary_all_inert: NormalizationBoundary::inert().all_inert(),
    }
}

pub fn normalization_matrix_json() -> String {
    serde_json::to_string(&normalization_matrix()).expect("normalization matrix serializes")
}

/// Re-derive the canonical matrix and byte-compare; a tampered/foreign matrix is
/// refused (never trusted off-wire — `Serialize` only, no `Deserialize`).
pub fn verify_markdown_normalization_matrix_json(
    candidate: &str,
) -> Result<(), NormalizationError> {
    if candidate == normalization_matrix_json() {
        Ok(())
    } else {
        Err(NormalizationError::ReplayMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_set_has_sixteen_named_rules() {
        assert_eq!(NORM_RULE_COUNT, NORM_RULE_NAMES.iter().count());
        assert_eq!(NORM_RULE_NAMES.iter().count(), 16);
        assert!(NORM_RULE_NAMES.iter().all(|n| !n.is_empty()));
    }

    #[test]
    fn boundary_is_eight_lines_and_claims_input_fidelity_only() {
        assert_eq!(NORM_BOUNDARY_LINES.iter().count(), 8);
        assert!(NORM_BOUNDARY_LINES
            .iter()
            .any(|l| l.contains("not better reading")));
        assert!(NORM_BOUNDARY_LINES
            .iter()
            .any(|l| l.contains("does not change the reading substrate")));
    }

    #[test]
    fn fixture_pack_is_at_least_twenty() {
        assert_eq!(NORM_FIXTURE_COUNT, fixtures().len());
        assert!(fixtures().len() >= 20);
    }

    #[test]
    fn frontmatter_is_stripped() {
        let out = normalize_markdown("---\ntitle: X\n---\nReal body sentence.");
        assert!(!out.contains("title:"));
        assert!(out.contains("Real body sentence."));
    }

    #[test]
    fn fenced_code_is_stripped() {
        let out = normalize_markdown("Intro.\n```\nsecret_code()\n```\nOutro.");
        assert!(!out.contains("secret_code"));
        assert!(out.contains("Intro."));
        assert!(out.contains("Outro."));
    }

    #[test]
    fn table_rows_and_rules_are_dropped() {
        let out = normalize_markdown("Before.\n\n| a | b |\n| - | - |\n\n---\n\nAfter.");
        assert!(!out.contains('|'));
        assert!(out.contains("Before."));
        assert!(out.contains("After."));
    }

    #[test]
    fn heading_and_list_markers_are_stripped() {
        let out = normalize_markdown("# Title\n- one\n- two\n1. three");
        assert!(!out.contains('#'));
        assert!(!out.contains("- one"));
        assert!(out.contains("Title."));
        assert!(out.contains("one."));
        assert!(out.contains("three."));
    }

    #[test]
    fn wikilinks_and_links_are_unwrapped() {
        let out = normalize_markdown("See [[Note A]] and [[B|alias]] and [text](http://u).");
        assert!(!out.contains("[["));
        assert!(!out.contains("]("));
        assert!(out.contains("Note A"));
        assert!(out.contains("alias"));
        assert!(out.contains("text"));
    }

    #[test]
    fn unbalanced_fence_does_not_eat_the_document() {
        // A stray opening fence whose only close is end-of-doc must NOT strip the
        // whole note (the >50% guard disables fence-stripping).
        let out = normalize_markdown("```\nThis is really prose.\nSecond prose line.\nThird line.");
        assert!(out.contains("This is really prose."));
        assert!(out.contains("Third line."));
    }

    #[test]
    fn prose_only_passes_through_essentially_unchanged() {
        let out = normalize_markdown("A clean note. Two sentences total.");
        assert!(out.contains("A clean note."));
        assert!(out.contains("Two sentences total."));
    }

    #[test]
    fn literal_tokens_survive_verbatim_no_semantic_leakage() {
        // The red line: the normalizer must not rewrite token text.
        assert!(literal_tokens_survive());
        for tok in SURVIVAL_TOKENS {
            assert!(normalize_markdown(&format!("Here is {tok} ok.")).contains(tok));
        }
    }

    #[test]
    fn normalize_is_deterministic() {
        let f = fixtures();
        for fx in &f {
            assert_eq!(
                normalize_markdown(fx.markdown),
                normalize_markdown(fx.markdown)
            );
        }
    }

    #[test]
    fn raw_markdown_grounds_but_pollutes_with_markup() {
        // The measured pain: on raw markdown, the reader grounds on markup spans.
        let m = normalization_matrix();
        assert!(
            m.raw_markup_total > 0,
            "raw markdown must show markup pollution"
        );
        assert!(m.raw_markup_permille > 0);
    }

    #[test]
    fn normalization_eliminates_markup_pollution() {
        let m = normalization_matrix();
        assert_eq!(
            m.norm_markup_total, 0,
            "normalized corpus must have zero markup spans"
        );
        assert_eq!(m.norm_markup_permille, 0);
    }

    #[test]
    fn grounding_safety_preserved_zero_false_grounded_both_ways() {
        // The load-bearing safety claim: verify never accepts an ungrounded answer,
        // raw OR normalized.
        let m = normalization_matrix();
        assert_eq!(m.raw_false_grounded_total, 0);
        assert_eq!(m.norm_false_grounded_total, 0);
    }

    #[test]
    fn prose_bearing_fixtures_finalize_after_normalization() {
        let m = normalization_matrix();
        for cell in &m.cells {
            if cell.prose_bearing {
                assert!(
                    cell.norm_finalized,
                    "{} should finalize normalized",
                    cell.name
                );
                assert_eq!(cell.norm_markup_spans, 0, "{} normalized markup", cell.name);
            }
        }
    }

    #[test]
    fn markup_never_increases_under_normalization() {
        let m = normalization_matrix();
        for cell in &m.cells {
            assert!(
                cell.norm_markup_spans <= cell.raw_markup_spans,
                "{}",
                cell.name
            );
        }
    }

    #[test]
    fn over_split_is_measured_not_assumed() {
        // Measure each class. Version (digit.digit) was always protected by the
        // splitter. Filename/URL over-split was UNRESOLVABLE by the adapter alone
        // (the adapter cannot edit token text without semantic leakage) and was the
        // recorded evidence for READ-N. After READ-N the substrate splitter keeps
        // `drive_scout.py` / `example.com/path.html` whole, so the one-span probe
        // now verifies. These expectations therefore record the post-READ-N measured
        // reality: filename/URL over-split resolves at the SUBSTRATE layer (not via
        // normalize_markdown — the adapter still never rewrites token text). The
        // historical field name `over_split_resolved_by_adapter` is kept stable.
        let m = normalization_matrix();
        assert!(
            m.over_split_version_resolved,
            "v1.2 should resolve (digit.digit protected)"
        );
        assert!(
            m.over_split_filename_resolved,
            "filename over-split resolved at the substrate layer (READ-N)"
        );
        assert!(
            m.over_split_url_resolved,
            "URL over-split resolved at the substrate layer (READ-N)"
        );
        assert_eq!(
            m.over_split_resolved_by_adapter,
            m.over_split_filename_resolved
                && m.over_split_url_resolved
                && m.over_split_version_resolved
        );
        assert!(
            m.over_split_resolved_by_adapter,
            "overall resolved after READ-N (the substrate splitter keeps filename/URL whole)"
        );
    }

    #[test]
    fn boundary_flags_are_all_inert_sourced_from_one_const() {
        // Every boundary flag is sourced from NORM_EDITS_SUBSTRATE; checking the
        // runtime matrix proves no path flipped one true.
        let m = normalization_matrix();
        assert!(m.boundary.all_inert());
    }

    #[test]
    fn matrix_reports_locked_counts() {
        let m = normalization_matrix();
        assert_eq!(m.rule_count, 16);
        assert_eq!(m.fixture_count, 22);
        assert_eq!(m.cells.len(), 22);
        assert_eq!(m.schema, "markdown-normalization-v0.1");
    }

    #[test]
    fn matrix_json_re_derives_and_refuses_tampering() {
        let json = normalization_matrix_json();
        assert!(verify_markdown_normalization_matrix_json(&json).is_ok());
        let tampered = json.replacen("\"norm_markup_total\":0", "\"norm_markup_total\":9", 1);
        assert_ne!(tampered, json);
        assert_eq!(
            verify_markdown_normalization_matrix_json(&tampered),
            Err(NormalizationError::ReplayMismatch)
        );
    }
}
