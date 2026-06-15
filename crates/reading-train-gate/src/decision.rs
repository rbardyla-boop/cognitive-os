//! P12 — the training-justification decision.
//!
//! A deterministic, machine-checkable gate. Weight training is justified ONLY
//! when, after the eval is clean of false-accepts, a residual model failure
//! survives cleanup of every fixable cause (bad fixture, schema, prompt, tooling,
//! missing context, verifier weakness) AND recurs. Anything else blocks — and the
//! decision names the exact fixture ids and the reason, so a reviewer never has
//! to trust prose.

use reading_eval::EvalReport;

/// The minimum independent recurrences for a clean failure to count as "stable".
pub const MIN_RECURRENCES: usize = 2;

/// The diagnosed cause of a residual eval failure. Every cause except
/// `CleanModelFailure` is a fixable defect that must be cleaned up first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureCause {
    BadFixture,
    SchemaDefect,
    PromptDefect,
    ToolingDefect,
    MissingContext,
    VerifierWeakness,
    /// Survives every fixable cause above — a genuine model capability gap.
    CleanModelFailure,
}

impl FailureCause {
    fn machine_tag(&self) -> &'static str {
        match self {
            FailureCause::BadFixture => "fixture_defect",
            FailureCause::SchemaDefect => "schema_defect",
            FailureCause::PromptDefect => "prompt_defect",
            FailureCause::ToolingDefect => "tooling_defect",
            FailureCause::MissingContext => "missing_context",
            FailureCause::VerifierWeakness => "verifier_defect",
            FailureCause::CleanModelFailure => "clean_model_failure",
        }
    }
}

/// A diagnosed residual failure: which fixture, its category, the diagnosed
/// cause, whether the fixable causes were ruled out, and how many independent
/// runs reproduced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureDiagnosis {
    pub fixture_id: String,
    pub category: String,
    pub cause: FailureCause,
    /// Fixture/schema/prompt/tooling/context/verifier causes have all been ruled out.
    pub survived_cleanup: bool,
    pub recurrences: usize,
}

impl FailureDiagnosis {
    pub fn new(
        fixture_id: impl Into<String>,
        category: impl Into<String>,
        cause: FailureCause,
        survived_cleanup: bool,
        recurrences: usize,
    ) -> Self {
        FailureDiagnosis {
            fixture_id: fixture_id.into(),
            category: category.into(),
            cause,
            survived_cleanup,
            recurrences,
        }
    }

    /// A clean, recurring failure that survived cleanup — the ONLY thing that can
    /// justify training.
    pub fn is_training_candidate(&self) -> bool {
        matches!(self.cause, FailureCause::CleanModelFailure)
            && self.survived_cleanup
            && self.recurrences >= MIN_RECURRENCES
    }

    fn block_reason(&self) -> Option<String> {
        if self.is_training_candidate() {
            return None;
        }
        let why = match self.cause {
            FailureCause::CleanModelFailure if !self.survived_cleanup => "not_survived_cleanup",
            FailureCause::CleanModelFailure => "insufficient_recurrence",
            other => other.machine_tag(),
        };
        Some(format!(
            "fixture {} [{}] blocked: {} — fix before training",
            self.fixture_id, self.category, why
        ))
    }
}

/// The machine-checkable training decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrainingDecision {
    /// The single load-bearing bit: may weight training proceed?
    pub training_justified: bool,
    /// `true` iff a false-accept demands a verifier/safety fix (never training).
    pub safety_fix_required: bool,
    /// Fixture ids of the clean, recurring failures that justify training (if any).
    pub cited_failures: Vec<String>,
    /// Machine-checkable reasons training is blocked (empty iff justified).
    pub blockers: Vec<String>,
    /// One-line human summary (derived from the structured fields, not a substitute).
    pub reason: String,
}

/// Decide whether training is justified from the eval's false-accepts and the
/// diagnoses of its residual failures. Pure and deterministic.
pub fn decide(false_accept_ids: &[String], diagnoses: &[FailureDiagnosis]) -> TrainingDecision {
    decide_inner(false_accept_ids, diagnoses, &[])
}

/// Core decision. `extra_blockers` carries caller-detected invalid inputs (e.g.
/// phantom diagnoses that cite a fixture the eval never failed) so they block
/// training and appear in the machine-checkable blocker list.
fn decide_inner(
    false_accept_ids: &[String],
    diagnoses: &[FailureDiagnosis],
    extra_blockers: &[String],
) -> TrainingDecision {
    let mut blockers: Vec<String> = extra_blockers.to_vec();

    // 1. Safety first: ANY false-accept is an unsafe verifier, fixed by hardening
    //    the verifier — never by training.
    let safety_fix_required = !false_accept_ids.is_empty();
    if safety_fix_required {
        blockers.push(format!(
            "false_accepts_present: {} — harden the verifier (safety fix), never train: [{}]",
            false_accept_ids.len(),
            false_accept_ids.join(", ")
        ));
    }

    // 2. Each residual failure either survives cleanup (a candidate) or blocks.
    let mut cited_failures = Vec::new();
    for diagnosis in diagnoses {
        match diagnosis.block_reason() {
            None => cited_failures.push(diagnosis.fixture_id.clone()),
            Some(reason) => blockers.push(reason),
        }
    }

    // 3. Nothing blocked and nothing to train on → explicit "no unresolved failures".
    if blockers.is_empty() && cited_failures.is_empty() {
        blockers.push(
            "no_unresolved_failures: 0 residual failures — nothing to train against".to_string(),
        );
    }

    // Training is justified ONLY with ≥1 clean candidate and ZERO blockers.
    let training_justified = blockers.is_empty() && !cited_failures.is_empty();

    let reason = if training_justified {
        format!(
            "justified: {} clean recurring model failure(s) survive cleanup: [{}]",
            cited_failures.len(),
            cited_failures.join(", ")
        )
    } else if safety_fix_required {
        "blocked: false-accepts require a verifier/safety fix, not training".to_string()
    } else if !extra_blockers.is_empty() {
        "blocked: phantom diagnoses do not correspond to real eval failures".to_string()
    } else if diagnoses.is_empty() {
        "blocked: no unresolved failures — no clean residual to justify training".to_string()
    } else {
        "blocked: residual failures trace to fixable defects (fixture/schema/prompt/tooling/context/verifier) — fix first".to_string()
    };

    TrainingDecision {
        training_justified,
        safety_fix_required,
        cited_failures,
        blockers,
        reason,
    }
}

/// Decide directly from the live P11 eval. False-accepts come from the report;
/// residual false-rejects are surfaced as UNDIAGNOSED failures (which block until
/// a reviewer diagnoses them) — so an eval with 0 residual failures yields a
/// blocked "no unresolved failures" decision, never a training recommendation.
pub fn decide_from_eval() -> TrainingDecision {
    let report = reading_eval::run();
    decide_from_report(&report, &[])
}

/// Decide from a given report plus any reviewer-supplied diagnoses. Any
/// false-reject in the report that lacks a diagnosis is treated as an
/// undiagnosed residual failure and blocks training (it cannot be a clean
/// candidate until its cause is ruled out).
pub fn decide_from_report(report: &EvalReport, diagnoses: &[FailureDiagnosis]) -> TrainingDecision {
    let false_accept_ids: Vec<String> = report
        .false_accepts
        .iter()
        .map(|c| c.name.clone())
        .collect();

    // Only diagnoses that correspond to an ACTUAL residual failure in the report
    // are admitted. A "phantom" diagnosis citing a fixture the eval never failed
    // cannot justify training — it becomes a blocker (so a clean eval can never be
    // coerced into a training recommendation by injecting a fabricated failure).
    let residual_ids: std::collections::BTreeSet<&str> = report
        .false_rejects
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    let mut admitted = Vec::new();
    let mut phantom_blockers = Vec::new();
    for d in diagnoses {
        if residual_ids.contains(d.fixture_id.as_str()) {
            admitted.push(d.clone());
        } else {
            phantom_blockers.push(format!(
                "phantom_diagnosis: {} is not a residual failure in the eval — cannot justify training",
                d.fixture_id
            ));
        }
    }
    // Every residual failure without an admitted diagnosis blocks (forces triage).
    let diagnosed: std::collections::BTreeSet<String> =
        admitted.iter().map(|d| d.fixture_id.clone()).collect();
    for fr in &report.false_rejects {
        if !diagnosed.contains(&fr.name) {
            admitted.push(FailureDiagnosis::new(
                fr.name.clone(),
                fr.category.clone(),
                FailureCause::MissingContext, // placeholder non-clean cause: "undiagnosed"
                false,
                0,
            ));
        }
    }
    decide_inner(&false_accept_ids, &admitted, &phantom_blockers)
}
