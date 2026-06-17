//! READ-3 — build a corpus from a real folder of documents.
//!
//! Each `.txt` file in the folder becomes one document; the file's text is split
//! into ONE SENTENCE PER SPAN using the substrate's shared sentence splitter
//! (so the spans a reader cites match the units the verifier checks). File
//! content is untrusted data — it only ever becomes inert spans. Reads are
//! confined to the given directory (no traversal / symlink escape).
//!
//! READ-11 — real document SECTION metadata. Markdown-style ATX heading lines
//! (`# Heading`, `## Heading`, … up to `######`) are detected DETERMINISTICALLY
//! and become `SectionMeta` headings (metadata only). The body sentences under a
//! heading are assigned to that section; content before the first heading is a
//! default (empty-heading) section; a file with no headings is one default
//! section, byte-identical to the pre-READ-11 flat build. A heading line is NEVER
//! split into a span, so it has no `SpanId` and can never be cited or grounded —
//! section structure may rank reads, never supply evidence. No semantic heading
//! detection, no all-caps guessing, no layout inference, no model.

use reading_substrate::{split_sentences, Corpus};
use std::io;
use std::path::Path;

/// Build a corpus from `(title, content)` documents: ATX headings become section
/// metadata and the body sentences become one-sentence spans, documents in the
/// given order (so span ids are deterministic). Pure — no I/O.
pub fn corpus_from_documents(documents: &[(String, String)]) -> Corpus {
    let mut corpus = Corpus::new();
    for (title, content) in documents {
        let sections = parse_sections(content);
        // Own the per-section `&str` span slices so they outlive the call.
        let section_spans: Vec<Vec<&str>> = sections
            .iter()
            .map(|(_, sentences)| sentences.iter().map(String::as_str).collect())
            .collect();
        let section_refs: Vec<(&str, &[&str])> = sections
            .iter()
            .zip(&section_spans)
            .map(|((heading, _), spans)| (heading.as_str(), spans.as_slice()))
            .collect();
        corpus.add_document_with_sections(title, &section_refs);
    }
    corpus
}

/// Parse `content` into `(heading, sentences)` sections using Markdown ATX
/// headings. Deterministic and purely lexical: a line is a heading iff it begins
/// with 1–6 `#` then whitespace then non-empty text; everything else is body. Body
/// sentences are produced by the shared `split_sentences`, so spans and grounding
/// never drift. A leading body block (before the first heading) is an empty-heading
/// section; it is emitted only if it has sentences, so a file that opens with a
/// heading does not gain a spurious empty section. A file with no headings yields a
/// single empty-heading section holding every sentence.
fn parse_sections(content: &str) -> Vec<(String, Vec<String>)> {
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    let mut heading = String::new();
    let mut body: Vec<&str> = Vec::new();

    for line in content.lines() {
        if let Some(h) = parse_atx_heading(line) {
            push_section(&mut sections, &heading, &body);
            heading = h;
            body.clear();
        } else {
            body.push(line);
        }
    }
    push_section(&mut sections, &heading, &body);

    if sections.is_empty() {
        // An empty file (or only blank/heading-less content with no sentences) is
        // still one default section, matching the flat build.
        sections.push((String::new(), Vec::new()));
    }
    sections
}

/// Emit a `(heading, sentences)` section from the accumulated body, unless it would
/// be a spurious empty default section (empty heading and no sentences). A heading
/// with no body is still emitted (its heading is real metadata; it just owns no
/// spans), so headings are never silently dropped.
fn push_section(sections: &mut Vec<(String, Vec<String>)>, heading: &str, body: &[&str]) {
    let sentences = split_sentences(&body.join("\n"));
    if heading.is_empty() && sentences.is_empty() {
        return;
    }
    sections.push((heading.to_string(), sentences));
}

/// If `line` is a Markdown ATX heading (1–6 leading `#`, then whitespace, then
/// non-empty text), return its trimmed heading text; otherwise `None`. Strict and
/// deterministic: `#nospace`, `#######` (7+), and a bare `#` are NOT headings.
fn parse_atx_heading(line: &str) -> Option<String> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &line[hashes..];
    if !rest.starts_with([' ', '\t']) {
        return None;
    }
    let text = rest.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Rebuild a corpus from already-split spans (used on replay/verify so the
/// reconstructed corpus is byte-identical to the one `run` built). Pure.
pub fn corpus_from_spans(documents: &[(String, Vec<String>)]) -> Corpus {
    let mut corpus = Corpus::new();
    for (title, spans) in documents {
        let refs: Vec<&str> = spans.iter().map(String::as_str).collect();
        corpus.add_document(title, &refs);
    }
    corpus
}

/// Load `(title, content)` for every `.txt` file directly in `dir`, sorted by
/// file name (deterministic order → deterministic span ids). Reads are confined
/// to `dir`: each entry's canonical path must stay within the canonical `dir`,
/// and only regular files are read (no recursion, no symlink escape).
pub fn load_documents(dir: &Path) -> io::Result<Vec<(String, String)>> {
    let root = dir.canonicalize()?;
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "documents path is not a directory",
        ));
    }
    let mut entries: Vec<(String, String)> = Vec::new();
    for entry in std::fs::read_dir(&root)? {
        let path = entry?.path();
        // Only plain `.txt` files, and only those that canonicalize to a regular
        // file inside `root` (rejects symlinks pointing outside the folder).
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(&root) || !canonical.is_file() {
            continue;
        }
        let title = canonical
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document")
            .to_string();
        let content = std::fs::read_to_string(&canonical)?;
        entries.push((title, content));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_sentence_per_span() {
        let docs = vec![(
            "report.txt".to_string(),
            "Bridge A was damaged. Bridge B stayed open. Inspectors closed Bridge A.".to_string(),
        )];
        let corpus = corpus_from_documents(&docs);
        // Three sentences → three spans, each a single sentence.
        assert_eq!(corpus.span_count(), 3);
        assert_eq!(
            corpus
                .read_span(reading_substrate::SpanId(0))
                .unwrap()
                .text(),
            "Bridge A was damaged."
        );
        assert_eq!(
            corpus
                .read_span(reading_substrate::SpanId(1))
                .unwrap()
                .text(),
            "Bridge B stayed open."
        );
    }

    #[test]
    fn documents_are_addressable_metadata_first() {
        let docs = vec![
            ("a.txt".to_string(), "First. Second.".to_string()),
            ("b.txt".to_string(), "Third.".to_string()),
        ];
        let corpus = corpus_from_documents(&docs);
        let meta = corpus.metadata();
        assert_eq!(meta.len(), 2);
        assert_eq!(meta[0].title, "a.txt");
        assert_eq!(meta[0].span_ids.len(), 2);
        assert_eq!(meta[1].title, "b.txt");
        assert_eq!(meta[1].span_ids.len(), 1);
    }

    #[test]
    fn rebuild_from_spans_matches_original_ids() {
        let docs = vec![("a.txt".to_string(), "One. Two.".to_string())];
        let original = corpus_from_documents(&docs);
        let spans: Vec<(String, Vec<String>)> = vec![(
            "a.txt".to_string(),
            vec!["One.".to_string(), "Two.".to_string()],
        )];
        let rebuilt = corpus_from_spans(&spans);
        assert_eq!(original.span_count(), rebuilt.span_count());
        for id in 0..original.span_count() as u64 {
            let sid = reading_substrate::SpanId(id);
            assert_eq!(
                original.read_span(sid).map(|s| s.text()),
                rebuilt.read_span(sid).map(|s| s.text())
            );
        }
    }

    // --- READ-11: real document section metadata ingestion (Markdown ATX) ---

    const HEADED: &str = "# Overview\nThe bridge is open.\n## Wind Forecast\nWinds will reach forty miles per hour.\nGusts may be higher.";

    #[test]
    fn markdown_heading_becomes_section_metadata() {
        // ATX heading lines (# / ##) become SectionMeta headings, deterministically.
        let corpus = corpus_from_documents(&[("report.txt".to_string(), HEADED.to_string())]);
        let doc = &corpus.metadata()[0];
        let headings: Vec<&str> = doc.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(headings, vec!["Overview", "Wind Forecast"]);
    }

    #[test]
    fn heading_is_not_a_span() {
        // No span is a heading line: spans are exactly the body sentences, and no
        // span text starts with '#' or equals a heading string.
        let corpus = corpus_from_documents(&[("report.txt".to_string(), HEADED.to_string())]);
        let doc = &corpus.metadata()[0];
        let texts: Vec<&str> = doc
            .span_ids
            .iter()
            .map(|id| corpus.read_span(*id).unwrap().text())
            .collect();
        assert_eq!(
            texts,
            vec![
                "The bridge is open.",
                "Winds will reach forty miles per hour.",
                "Gusts may be higher.",
            ]
        );
        for t in &texts {
            assert!(!t.starts_with('#'), "no span is a heading line: {t:?}");
            assert_ne!(*t, "Overview");
            assert_ne!(*t, "Wind Forecast");
        }
    }

    #[test]
    fn sentence_under_heading_gets_section_id() {
        // Each body sentence is assigned to the section of the heading above it.
        let corpus = corpus_from_documents(&[("report.txt".to_string(), HEADED.to_string())]);
        let doc = &corpus.metadata()[0];
        let text_of = |sid: &reading_substrate::SpanId| corpus.read_span(*sid).unwrap().text();
        let overview: Vec<&str> = doc.sections[0].span_ids.iter().map(text_of).collect();
        let wind: Vec<&str> = doc.sections[1].span_ids.iter().map(text_of).collect();
        assert_eq!(overview, vec!["The bridge is open."]);
        assert_eq!(
            wind,
            vec![
                "Winds will reach forty miles per hour.",
                "Gusts may be higher.",
            ]
        );
    }

    #[test]
    fn unheaded_file_gets_default_section() {
        // A file with no headings is one default (empty-heading) section holding
        // every sentence — byte-identical to the pre-READ-11 flat build.
        let content = "First sentence. Second sentence.".to_string();
        let corpus = corpus_from_documents(&[("a.txt".to_string(), content.clone())]);
        let doc = &corpus.metadata()[0];
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "");
        assert_eq!(doc.sections[0].span_ids, doc.span_ids);
        // identical to the substrate's own flat builder on the same sentences.
        let mut flat = reading_substrate::Corpus::new();
        flat.add_document("a.txt", &["First sentence.", "Second sentence."]);
        assert_eq!(doc.span_ids.len(), flat.metadata()[0].span_ids.len());
        for id in 0..corpus.span_count() as u64 {
            let sid = reading_substrate::SpanId(id);
            assert_eq!(
                corpus.read_span(sid).map(|s| s.text()),
                flat.read_span(sid).map(|s| s.text())
            );
        }
    }

    #[test]
    fn non_atx_hash_lines_are_body_not_headings() {
        // A '#' not followed by a space (no ATX form) is ordinary body text, and 7+
        // hashes is not a heading either — purely lexical, no layout guessing.
        let content = "#nospace is text. ####### too many hashes.".to_string();
        let corpus = corpus_from_documents(&[("a.txt".to_string(), content)]);
        let doc = &corpus.metadata()[0];
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].heading, "");
        assert!(!doc.span_ids.is_empty());
    }
}
