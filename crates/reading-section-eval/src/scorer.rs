//! READ-10 — score the section-aware, multi-term reader against the budgeted one.
//!
//! For every READ-4 fixture we rebuild the corpus exactly as `read0` does and run
//! BOTH `reading_autonomy` selective readers: the READ-8 `read_budgeted` (metadata
//! order) and the READ-10 `read_section_ranked` (section-relevance order). Each
//! section-ranked finalized answer is CROSS-VALIDATED with a fresh
//! `reading_substrate::verify` AND the independent
//! `reading_autonomous_eval::independently_grounded` check, so a false-grounded
//! answer is measured. The committed pack is flat (one headingless section per
//! document), so section ranking reduces to title ranking and only REORDERS — the
//! eval proves NO-REGRESSION (section answer == budgeted answer) and 0
//! false-grounded. The section + multi-term WIN is measured separately by
//! `section_priority_demo`. Deterministic; no model, no training.

use reading_autonomous_eval::independently_grounded;
use reading_autonomy::{read_budgeted, read_section_ranked, ReaderBounds, ReaderOutcome};
use reading_cli::corpus_from_documents;
use reading_corpus_eval::fixtures;
use reading_substrate::{verify, Corpus, ReadingRun};

/// What the section-ranked reader did with a fixture's corpus + question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SectionOutcome {
    /// A verifier-approved, source-grounded answer over the relevant spans.
    Answered {
        answer: String,
        trace_hash: u64,
        claims: usize,
    },
    /// No relevant span was claimed within budget — a coverage miss.
    CoverageMiss { reason: String },
}

/// One scored fixture: the section reader's outcome and how it compares to budgeted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionScore {
    pub name: String,
    pub outcome: SectionOutcome,
    /// The unsafe class — a finalized answer the independent checks reject. Zero.
    pub false_grounded: bool,
    /// The section answer equals the budgeted answer (on the flat committed pack
    /// section ranking only reorders — it drops/adds/fabricates nothing).
    pub matches_budgeted: bool,
}

/// The READ-10 report over the committed pack: the no-regression comparison,
/// classified coverage misses, and the explicit (must-be-empty) false-grounded and
/// regression lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionReport {
    pub total: usize,
    pub bounds: ReaderBounds,
    pub answered: usize,
    pub coverage_misses: Vec<SectionScore>,
    pub false_grounded: Vec<SectionScore>,
    /// Fixtures where the section answer differs from the budgeted answer. On the
    /// flat committed pack this MUST be empty: section ranking reorders, never
    /// regresses.
    pub regressions: Vec<SectionScore>,
    pub scores: Vec<SectionScore>,
}

/// The finalized run of an outcome, if any.
fn finalized(outcome: &ReaderOutcome) -> Option<&ReadingRun> {
    match &outcome.decision {
        Ok(decoded) => decoded.finalized.as_ref(),
        Err(_) => None,
    }
}

/// Score the committed READ-4 fixture pack with the section-aware reader,
/// cross-validating every finalized answer. Deterministic.
pub fn evaluate_section_pack(bounds: ReaderBounds) -> SectionReport {
    let mut scores = Vec::new();
    let mut answered = 0usize;
    let mut coverage_misses = Vec::new();
    let mut false_grounded = Vec::new();
    let mut regressions = Vec::new();

    for fixture in fixtures() {
        let docs: Vec<(String, String)> = fixture
            .documents
            .iter()
            .map(|(name, content)| (name.to_string(), content.to_string()))
            .collect();
        let corpus = corpus_from_documents(&docs);

        let budgeted = read_budgeted(&corpus, fixture.question, bounds);
        let sectioned = read_section_ranked(&corpus, fixture.question, bounds);
        let matches_budgeted = budgeted.answer() == sectioned.answer();

        let (outcome, is_false_grounded) = match finalized(&sectioned) {
            Some(run) => {
                let report = verify(&corpus, run);
                let fg = !report.passed || !independently_grounded(&corpus, run);
                (
                    SectionOutcome::Answered {
                        answer: run.proof.answer_text.clone(),
                        trace_hash: run.answer_hash,
                        claims: run.memory.claims.len(),
                    },
                    fg,
                )
            }
            None => (
                SectionOutcome::CoverageMiss {
                    reason: "no question-relevant span claimed within budget".to_string(),
                },
                false,
            ),
        };

        let is_answered = matches!(outcome, SectionOutcome::Answered { .. });
        if is_answered {
            answered += 1;
        }

        let score = SectionScore {
            name: fixture.name.to_string(),
            outcome,
            false_grounded: is_false_grounded,
            matches_budgeted,
        };
        if is_false_grounded {
            false_grounded.push(score.clone());
        }
        if !matches_budgeted {
            regressions.push(score.clone());
        }
        if matches!(score.outcome, SectionOutcome::CoverageMiss { .. }) {
            coverage_misses.push(score.clone());
        }
        scores.push(score);
    }

    SectionReport {
        total: scores.len(),
        bounds,
        answered,
        coverage_misses,
        false_grounded,
        regressions,
        scores,
    }
}

/// The measured section + multi-term WIN on constructed sectioned corpora. Each
/// scenario uses a 1-span budget so the read ORDER decides the answer; the budgeted
/// reader (metadata order) reads the first, irrelevant section and misses, while
/// the section-aware reader reads the relevant section first and answers — with 0
/// false-grounded. The corpora are fixed test data, not a model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectionDemo {
    /// Heading priority: a heading-relevant section filed second is reached first.
    pub heading_budgeted_answered: bool,
    pub heading_ranked_answered: bool,
    pub heading_answer: Option<String>,
    /// Multi-term: when both headings share one token, the section covering MORE
    /// distinct query terms is read first (single-token overlap could not pick it).
    pub multiterm_budgeted_answered: bool,
    pub multiterm_ranked_answered: bool,
    pub multiterm_answer: Option<String>,
    /// Neither recovered answer is false-grounded (cross-validated).
    pub any_false_grounded: bool,
    /// The heading answer is identical with the sections inserted forward vs reversed.
    pub stable_across_section_order: bool,
}

/// Cross-validate a finalized outcome against its corpus: returns (answer, fg).
fn answer_and_fg(corpus: &Corpus, outcome: &ReaderOutcome) -> (Option<String>, bool) {
    match finalized(outcome) {
        Some(run) => {
            let report = verify(corpus, run);
            let fg = !report.passed || !independently_grounded(corpus, run);
            (Some(run.proof.answer_text.clone()), fg)
        }
        None => (None, false),
    }
}

/// Build the constructed sectioned scenarios and measure the section + multi-term
/// win. Deterministic.
pub fn section_priority_demo() -> SectionDemo {
    let tight = ReaderBounds {
        max_spans: 1,
        ..ReaderBounds::default()
    };

    // --- Heading priority: relevant section's HEADING matches, filed second. ---
    let mut heading = Corpus::new();
    heading.add_document_with_sections(
        "bulletin",
        &[
            ("general notes", &["The office opened at nine."]),
            (
                "storm wind forecast",
                &["Winds will reach forty miles per hour."],
            ),
        ],
    );
    let hq = "What is the storm wind forecast?";
    let heading_budgeted = read_budgeted(&heading, hq, tight);
    let heading_sectioned = read_section_ranked(&heading, hq, tight);
    let (heading_answer, heading_fg) = answer_and_fg(&heading, &heading_sectioned);

    // Reversed section order — distinct headings, so the answer must be identical.
    let mut heading_rev = Corpus::new();
    heading_rev.add_document_with_sections(
        "bulletin",
        &[
            (
                "storm wind forecast",
                &["Winds will reach forty miles per hour."],
            ),
            ("general notes", &["The office opened at nine."]),
        ],
    );
    let heading_rev_answer = read_section_ranked(&heading_rev, hq, tight)
        .answer()
        .map(String::from);

    // --- Multi-term: both headings share "wind"; one covers 3 terms, one covers 1. ---
    let mut multiterm = Corpus::new();
    multiterm.add_document_with_sections(
        "alerts",
        &[
            ("wind notes", &["Breezes stayed calm all afternoon."]),
            (
                "storm wind warning",
                &["A severe storm wind warning is in effect tonight."],
            ),
        ],
    );
    let mq = "Is there a storm wind warning?";
    let multiterm_budgeted = read_budgeted(&multiterm, mq, tight);
    let multiterm_sectioned = read_section_ranked(&multiterm, mq, tight);
    let (multiterm_answer, multiterm_fg) = answer_and_fg(&multiterm, &multiterm_sectioned);

    SectionDemo {
        heading_budgeted_answered: heading_budgeted.finalized(),
        heading_ranked_answered: heading_sectioned.finalized(),
        heading_answer: heading_answer.clone(),
        multiterm_budgeted_answered: multiterm_budgeted.finalized(),
        multiterm_ranked_answered: multiterm_sectioned.finalized(),
        multiterm_answer,
        any_false_grounded: heading_fg || multiterm_fg,
        stable_across_section_order: heading_answer.is_some()
            && heading_answer == heading_rev_answer,
    }
}
