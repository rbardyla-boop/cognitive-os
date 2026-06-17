//! READ-9 — score the title-aware ranked reader against the budgeted one.
//!
//! For every READ-4 fixture we rebuild the corpus exactly as `read0` does and run
//! BOTH `reading_autonomy` selective readers: the READ-8 `read_budgeted` (metadata
//! order) and the READ-9 `read_ranked` (title-ranked order). Each ranked finalized
//! answer is CROSS-VALIDATED with a fresh `reading_substrate::verify` AND the
//! independent `reading_autonomous_eval::independently_grounded` check, so a
//! false-grounded answer is measured. On the committed pack the relevant documents
//! are already first, so ranking only REORDERS — the eval proves NO-REGRESSION
//! (ranked answer == budgeted answer) and 0 false-grounded. The title-ranking WIN
//! (reaching a relevant document filed second under a tight budget) is measured
//! separately by `title_priority_demo`. Deterministic; no model, no training.

use reading_autonomous_eval::independently_grounded;
use reading_autonomy::{read_budgeted, read_ranked, ReaderBounds, ReaderOutcome};
use reading_cli::corpus_from_documents;
use reading_corpus_eval::fixtures;
use reading_substrate::{verify, Corpus, ReadingRun};

/// What the title-ranked reader did with a fixture's corpus + question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RankedOutcome {
    /// A verifier-approved, source-grounded answer over the relevant spans.
    Answered {
        answer: String,
        trace_hash: u64,
        claims: usize,
    },
    /// No relevant span was claimed within budget — a coverage miss.
    CoverageMiss { reason: String },
}

/// One scored fixture: the ranked reader's outcome and how it compares to budgeted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankedScore {
    pub name: String,
    pub outcome: RankedOutcome,
    /// The unsafe class — a finalized answer the independent checks reject. Zero.
    pub false_grounded: bool,
    /// The ranked answer is identical to the budgeted answer (ranking only
    /// reorders reads on the committed pack — it drops/adds/fabricates nothing).
    pub matches_budgeted: bool,
}

/// The READ-9 report: the no-regression comparison, classified coverage misses,
/// and the explicit (must-be-empty) false-grounded and regression lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankedReport {
    pub total: usize,
    pub bounds: ReaderBounds,
    pub answered: usize,
    pub coverage_misses: Vec<RankedScore>,
    pub false_grounded: Vec<RankedScore>,
    /// Fixtures where the ranked answer differs from the budgeted answer. Under the
    /// default budget this MUST be empty: ranking reorders, it never regresses.
    pub regressions: Vec<RankedScore>,
    pub scores: Vec<RankedScore>,
}

/// The finalized run of an outcome, if any.
fn finalized(outcome: &ReaderOutcome) -> Option<&ReadingRun> {
    match &outcome.decision {
        Ok(decoded) => decoded.finalized.as_ref(),
        Err(_) => None,
    }
}

/// Score the committed READ-4 fixture pack with the title-ranked reader,
/// cross-validating every finalized answer against the budgeted reader and the
/// independent grounding check. Deterministic.
pub fn evaluate_ranked_pack(bounds: ReaderBounds) -> RankedReport {
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
        let ranked = read_ranked(&corpus, fixture.question, bounds);

        // No-regression: ranking must not change the answerable outcome. Comparing
        // the answer text covers both "both answered the same" and "both missed".
        let matches_budgeted = budgeted.answer() == ranked.answer();

        let (outcome, is_false_grounded) = match finalized(&ranked) {
            Some(run) => {
                // Cross-validated: both the fresh verify pass and the independent
                // grounding check must agree, else it is false-grounded.
                let report = verify(&corpus, run);
                let fg = !report.passed || !independently_grounded(&corpus, run);
                (
                    RankedOutcome::Answered {
                        answer: run.proof.answer_text.clone(),
                        trace_hash: run.answer_hash,
                        claims: run.memory.claims.len(),
                    },
                    fg,
                )
            }
            None => (
                RankedOutcome::CoverageMiss {
                    reason: "no question-relevant span claimed within budget".to_string(),
                },
                false,
            ),
        };

        let is_answered = matches!(outcome, RankedOutcome::Answered { .. });
        if is_answered {
            answered += 1;
        }

        let score = RankedScore {
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
        if matches!(score.outcome, RankedOutcome::CoverageMiss { .. }) {
            coverage_misses.push(score.clone());
        }
        scores.push(score);
    }

    RankedReport {
        total: scores.len(),
        bounds,
        answered,
        coverage_misses,
        false_grounded,
        regressions,
        scores,
    }
}

/// The measured title-ranking WIN on a constructed corpus: the question-relevant
/// document is filed SECOND but its TITLE matches the question. Under a 1-span
/// budget the budgeted reader (metadata order) reads the first, irrelevant document
/// and misses; the title-ranked reader reads the relevant document first and
/// answers — with 0 false-grounded — and the ranked answer is identical whether the
/// documents are inserted forward or reversed. The corpus is fixed test data, not a
/// model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankingDemo {
    /// Blunt metadata order under the tight budget reads the irrelevant doc → miss.
    pub budgeted_answered: bool,
    /// Title rank reads the relevant doc first under the same budget → answer.
    pub ranked_answered: bool,
    pub ranked_answer: Option<String>,
    /// The ranked answer survives both verify and the independent grounding check.
    pub ranked_false_grounded: bool,
    /// The ranked answer is identical with the documents inserted forward vs reversed.
    pub stable_across_file_order: bool,
}

/// Build the constructed two-document scenario and measure the title-ranking win.
/// Deterministic.
pub fn title_priority_demo() -> RankingDemo {
    let tight = ReaderBounds {
        max_spans: 1,
        ..ReaderBounds::default()
    };
    let question = "What is the wind forecast?";
    let log = (
        "daily_log.txt".to_string(),
        "The office opened at nine.".to_string(),
    );
    let wind = (
        "wind_forecast.txt".to_string(),
        "Winds will reach forty miles per hour.".to_string(),
    );
    // Forward: irrelevant doc first, the title-relevant doc second.
    let forward: Corpus = corpus_from_documents(&[log.clone(), wind.clone()]);
    // Reversed insertion order — distinct titles, so the ranked result must match.
    let reverse: Corpus = corpus_from_documents(&[wind, log]);

    let budgeted = read_budgeted(&forward, question, tight);
    let ranked = read_ranked(&forward, question, tight);
    let ranked_reverse = read_ranked(&reverse, question, tight);

    let (ranked_answer, ranked_false_grounded) = match finalized(&ranked) {
        Some(run) => {
            let report = verify(&forward, run);
            let fg = !report.passed || !independently_grounded(&forward, run);
            (Some(run.proof.answer_text.clone()), fg)
        }
        None => (None, false),
    };

    RankingDemo {
        budgeted_answered: budgeted.finalized(),
        ranked_answered: ranked.finalized(),
        ranked_answer,
        ranked_false_grounded,
        stable_across_file_order: ranked.answer().is_some()
            && ranked.answer() == ranked_reverse.answer(),
    }
}
