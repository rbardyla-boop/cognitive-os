//! READ-8 — score the budgeted selective reader against the blunt one.
//!
//! For every READ-4 fixture we rebuild the corpus exactly as `read0` does and run
//! BOTH `reading_autonomy` readers: the blunt READ-6 `read` (claims every span)
//! and the budgeted READ-8 `read_budgeted` (claims only spans lexically relevant
//! to the question). Each budgeted finalized answer is CROSS-VALIDATED with a
//! fresh `reading_substrate::verify` AND the independent
//! `reading_autonomous_eval::independently_grounded` check, so a false-grounded
//! answer is measured. A budgeted run that finalizes nothing (no relevant span
//! within budget) is a CLASSIFIED coverage miss — an engineering signal, never a
//! training justification. Deterministic; no model, no training.

use reading_autonomous_eval::independently_grounded;
use reading_autonomy::{read, read_budgeted, ReaderBounds};
use reading_cli::corpus_from_documents;
use reading_corpus_eval::fixtures;
use reading_substrate::{verify, ReadingRun};

/// What the budgeted reader did with a fixture's corpus + question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BudgetedOutcome {
    /// A verifier-approved, source-grounded answer over the relevant spans.
    Answered {
        answer: String,
        trace_hash: u64,
        claims: usize,
    },
    /// No relevant span was claimed within budget — a coverage miss.
    CoverageMiss { reason: String },
}

/// One scored fixture: the blunt reader's breadth vs the budgeted reader's focus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetedScore {
    pub name: String,
    /// Claims the blunt READ-6 reader made (it claims every span it reads).
    pub blunt_claims: usize,
    /// Claims the budgeted READ-8 reader made (the relevant subset).
    pub budgeted_claims: usize,
    pub outcome: BudgetedOutcome,
    /// The unsafe class — a finalized answer the independent checks reject. Zero.
    pub false_grounded: bool,
    /// The budgeted reader claimed strictly fewer spans than the blunt reader.
    pub more_focused: bool,
}

/// The READ-8 report: focus comparison, classified coverage misses, and the
/// explicit (must-be-empty) false-grounded list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetedReport {
    pub total: usize,
    pub bounds: ReaderBounds,
    pub answered: usize,
    pub more_focused: usize,
    pub total_blunt_claims: usize,
    pub total_budgeted_claims: usize,
    pub coverage_misses: Vec<BudgetedScore>,
    pub false_grounded: Vec<BudgetedScore>,
    pub scores: Vec<BudgetedScore>,
}

/// The number of grounded claims a reader's finalized run carries (0 if it did
/// not finalize).
fn claims_of(outcome: &reading_autonomy::ReaderOutcome) -> usize {
    finalized(outcome)
        .map(|run| run.memory.claims.len())
        .unwrap_or(0)
}

/// The finalized run of an outcome, if any.
fn finalized(outcome: &reading_autonomy::ReaderOutcome) -> Option<&ReadingRun> {
    match &outcome.decision {
        Ok(decoded) => decoded.finalized.as_ref(),
        Err(_) => None,
    }
}

/// Score the committed READ-4 fixture pack with the budgeted reader. Deterministic.
pub fn evaluate_budgeted_pack(bounds: ReaderBounds) -> BudgetedReport {
    let mut scores = Vec::new();
    let mut answered = 0usize;
    let mut more_focused = 0usize;
    let mut total_blunt_claims = 0usize;
    let mut total_budgeted_claims = 0usize;
    let mut coverage_misses = Vec::new();
    let mut false_grounded = Vec::new();

    for fixture in fixtures() {
        let docs: Vec<(String, String)> = fixture
            .documents
            .iter()
            .map(|(name, content)| (name.to_string(), content.to_string()))
            .collect();
        let corpus = corpus_from_documents(&docs);

        let blunt = read(&corpus, fixture.question, bounds);
        let budgeted = read_budgeted(&corpus, fixture.question, bounds);
        let blunt_claims = claims_of(&blunt);
        let budgeted_claims = claims_of(&budgeted);

        let (outcome, is_false_grounded) = match finalized(&budgeted) {
            Some(run) => {
                // Cross-validated: both the fresh verify pass and the independent
                // grounding check must agree, else it is false-grounded.
                let report = verify(&corpus, run);
                let fg = !report.passed || !independently_grounded(&corpus, run);
                (
                    BudgetedOutcome::Answered {
                        answer: run.proof.answer_text.clone(),
                        trace_hash: run.answer_hash,
                        claims: budgeted_claims,
                    },
                    fg,
                )
            }
            None => (
                BudgetedOutcome::CoverageMiss {
                    reason: "no question-relevant span claimed within budget".to_string(),
                },
                false,
            ),
        };

        let is_answered = matches!(outcome, BudgetedOutcome::Answered { .. });
        let focused = budgeted_claims < blunt_claims;
        total_blunt_claims += blunt_claims;
        total_budgeted_claims += budgeted_claims;
        if is_answered {
            answered += 1;
        }
        if focused {
            more_focused += 1;
        }

        let score = BudgetedScore {
            name: fixture.name.to_string(),
            blunt_claims,
            budgeted_claims,
            outcome,
            false_grounded: is_false_grounded,
            more_focused: focused,
        };
        if is_false_grounded {
            false_grounded.push(score.clone());
        }
        if matches!(score.outcome, BudgetedOutcome::CoverageMiss { .. }) {
            coverage_misses.push(score.clone());
        }
        scores.push(score);
    }

    BudgetedReport {
        total: scores.len(),
        bounds,
        answered,
        more_focused,
        total_blunt_claims,
        total_budgeted_claims,
        coverage_misses,
        false_grounded,
        scores,
    }
}
