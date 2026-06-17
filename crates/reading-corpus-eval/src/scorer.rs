//! READ-4 — drive every fixture through the real read0 pipeline and score it.
//!
//! Each fixture is materialized to a real docs folder; the plan is driven through
//! the actual `read0 run` (reading_cli::run_reading), and a finalized run is then
//! `verify`-ed AND `replay`-ed (reading_cli::verify_run / replay_run) — so every
//! fixture exercises run + verify + replay, not a single hand demo. The result is
//! compared to the COMMITTED expected label. A false-grounded answer (expected
//! Rejected, but a verified answer finalized) is the unsafe class and is surfaced
//! explicitly. Deterministic: fixed content → fixed hashes; the workdir paths do
//! not enter the report.

use crate::pack::{fixtures, CorpusFixture, Expected};
use reading_cli::{replay_run, run_reading, verify_run};
use std::io;
use std::path::{Path, PathBuf};

/// What read0 actually did with a fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A verified, source-grounded answer was finalized (and re-verified + replayed).
    Verified { answer: String, trace_hash: u64 },
    /// No verified answer was produced, with the reason.
    Rejected { reason: String },
}

/// How the actual outcome compares to the committed expectation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Correct,
    /// Expected a rejection but a verified answer finalized — UNSAFE.
    FalseGrounded,
    /// Expected a verified answer but it was rejected.
    FalseReject,
}

/// One scored fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixtureResult {
    pub name: String,
    pub expected: Expected,
    pub outcome: Outcome,
    pub verdict: Verdict,
}

impl FixtureResult {
    /// The trace hash for a verified fixture (the answer content hash), else None.
    pub fn trace_hash(&self) -> Option<u64> {
        match &self.outcome {
            Outcome::Verified { trace_hash, .. } => Some(*trace_hash),
            Outcome::Rejected { .. } => None,
        }
    }
    /// The rejection reason for a rejected fixture, else None.
    pub fn rejection_reason(&self) -> Option<&str> {
        match &self.outcome {
            Outcome::Rejected { reason } => Some(reason),
            Outcome::Verified { .. } => None,
        }
    }
}

/// The pack report — the score plus the explicit false-grounded / false-reject lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackReport {
    pub total: usize,
    pub correct: usize,
    pub false_grounded: Vec<FixtureResult>,
    pub false_rejects: Vec<FixtureResult>,
    pub results: Vec<FixtureResult>,
}

/// A unique temp working directory that cleans itself up on drop.
pub struct Workdir {
    path: PathBuf,
}

impl Workdir {
    pub fn new() -> io::Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("read4_pack_{}_{}", std::process::id(), n));
        std::fs::create_dir_all(&path)?;
        Ok(Workdir { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Workdir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Score the committed fixture pack under `workdir`. Deterministic.
pub fn evaluate_pack(workdir: &Path) -> io::Result<PackReport> {
    evaluate(workdir, &fixtures())
}

/// Score a given fixture set under `workdir`.
pub fn evaluate(workdir: &Path, fixtures: &[CorpusFixture]) -> io::Result<PackReport> {
    let mut results = Vec::with_capacity(fixtures.len());
    let mut correct = 0usize;
    let mut false_grounded = Vec::new();
    let mut false_rejects = Vec::new();

    for fixture in fixtures {
        let outcome = run_one(workdir, fixture)?;
        let verdict = match (fixture.expected, &outcome) {
            (Expected::Verified, Outcome::Verified { .. }) => Verdict::Correct,
            (Expected::Rejected, Outcome::Rejected { .. }) => Verdict::Correct,
            (Expected::Rejected, Outcome::Verified { .. }) => Verdict::FalseGrounded,
            (Expected::Verified, Outcome::Rejected { .. }) => Verdict::FalseReject,
        };
        let result = FixtureResult {
            name: fixture.name.to_string(),
            expected: fixture.expected,
            outcome,
            verdict,
        };
        match verdict {
            Verdict::Correct => correct += 1,
            Verdict::FalseGrounded => false_grounded.push(result.clone()),
            Verdict::FalseReject => false_rejects.push(result.clone()),
        }
        results.push(result);
    }

    Ok(PackReport {
        total: fixtures.len(),
        correct,
        false_grounded,
        false_rejects,
        results,
    })
}

/// Materialize one fixture to a docs folder and drive it through read0
/// run → verify → replay. The plan reaches memory only via the codec (run0).
fn run_one(workdir: &Path, fixture: &CorpusFixture) -> io::Result<Outcome> {
    let fixture_dir = workdir.join(fixture.name);
    let docs_dir = fixture_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;
    for (filename, content) in fixture.documents {
        std::fs::write(docs_dir.join(filename), content)?;
    }
    let plan_path = fixture_dir.join("plan.json");
    std::fs::write(&plan_path, fixture.plan)?;
    let out_path = fixture_dir.join("out.json");

    match run_reading(&docs_dir, fixture.question, &plan_path, &out_path) {
        Ok(file) => {
            // A verified answer must also clear verify AND replay (the full path).
            match (verify_run(&out_path), replay_run(&out_path)) {
                (Ok(_), Ok(())) => Ok(Outcome::Verified {
                    answer: file.answer,
                    trace_hash: file.answer_hash,
                }),
                (verify, replay) => Ok(Outcome::Rejected {
                    reason: format!(
                        "run finalized but verify/replay failed: verify={verify:?} replay={replay:?}"
                    ),
                }),
            }
        }
        Err(error) => Ok(Outcome::Rejected {
            reason: error.to_string(),
        }),
    }
}
