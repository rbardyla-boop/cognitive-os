//! READ-3 — build a corpus from a real folder of documents.
//!
//! Each `.txt` file in the folder becomes one document; the file's text is split
//! into ONE SENTENCE PER SPAN using the substrate's shared sentence splitter
//! (so the spans a reader cites match the units the verifier checks). File
//! content is untrusted data — it only ever becomes inert spans. Reads are
//! confined to the given directory (no traversal / symlink escape).

use reading_substrate::{split_sentences, Corpus};
use std::io;
use std::path::Path;

/// Build a corpus from `(title, content)` documents: one sentence per span,
/// documents in the given order (so span ids are deterministic). Pure — no I/O.
pub fn corpus_from_documents(documents: &[(String, String)]) -> Corpus {
    let mut corpus = Corpus::new();
    for (title, content) in documents {
        let spans = split_sentences(content);
        let refs: Vec<&str> = spans.iter().map(String::as_str).collect();
        corpus.add_document(title, &refs);
    }
    corpus
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
}
