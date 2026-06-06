// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Shared types for arena-tests' quality dimensions.
//!
//! Every quality dimension implements [`Dimension`]. The driver
//! ([`arena-driver`](../../arena-driver)) runs each, aggregates results
//! into a [`Report`], appends to `reports/history.jsonl`, and emits
//! `reports/latest.md`.
//!
//! ## Aggregation contract
//!
//! Aggregate verdict is the **minimum** across dimensions (a single
//! [`Verdict::Fail`] makes the whole run fail). Aggregate score is the
//! minimum numeric score across dimensions that produced one. Averaging
//! is explicitly avoided so that a hole in one dimension cannot be
//! washed out by strength in another.
//!
//! ## Tone
//!
//! Findings are evidence-first: every [`Finding`] cites a path, line,
//! command, or other artifact (`evidence` field). Speculation is not a
//! finding — it's a code smell.

use serde::{Deserialize, Serialize};

/// The verdict a dimension produces for a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    /// Dimension ran, all assertions held.
    Pass,
    /// Dimension ran, soft thresholds breached but no hard failure.
    Warn,
    /// Dimension ran, hard threshold breached. The aggregate run fails.
    Fail,
    /// Dimension was deliberately not implemented this run (stub).
    /// Does not contribute to the aggregate.
    Skip,
    /// Dimension errored before it could produce a verdict.
    /// Treated as Fail in aggregate but distinguished in reports.
    Error,
}

impl Verdict {
    /// Combine two verdicts, returning the worse one. Skip is ignored.
    #[must_use]
    pub fn worsen_with(self, other: Verdict) -> Verdict {
        use Verdict::{Error, Fail, Pass, Skip, Warn};
        match (self, other) {
            (Skip, v) | (v, Skip) => v,
            (Error, _) | (_, Error) => Error,
            (Fail, _) | (_, Fail) => Fail,
            (Warn, _) | (_, Warn) => Warn,
            (Pass, Pass) => Pass,
        }
    }
}

/// A single evidence-cited observation produced by a dimension run.
///
/// Findings can be drafted as [`crate::QualityFinding`] entries for
/// `QUALITY_BACKLOG.md` when severity warrants it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Short headline, imperative or declarative. e.g. "unit test issues TCP connect".
    pub title: String,
    /// Severity hint, used for sorting and for backlog drafting.
    pub severity: Severity,
    /// Concrete artifact citation: file path, line, command output, metric value.
    pub evidence: String,
    /// Optional pointer to the recurring property this finding relates to
    /// (e.g. "RP-HERMETIC-UNIT"). Used to align findings with the QUALITY_BACKLOG.md
    /// invariants layer.
    pub recurring_property: Option<String>,
}

/// Severity hint for a single finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Blocks a release; maps to QF bucket A.
    Critical,
    /// Should fix soon; maps to QF bucket B.
    High,
    /// Strategic; maps to QF bucket C.
    Moderate,
    /// Informational; for trend lines, not gates.
    Info,
}

/// Result of a single dimension's run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionResult {
    /// The dimension's stable identifier, e.g. "hermeticity".
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// The recurring system property this dimension measures.
    pub recurring_property: String,
    /// Verdict for this run.
    pub verdict: Verdict,
    /// Numeric score 0–100 where 100 is perfect, or `None` if the dimension
    /// is binary (pass/fail only).
    pub score: Option<u8>,
    /// Concrete observations from this run.
    pub findings: Vec<Finding>,
    /// Wall-clock duration of this dimension's run, in milliseconds.
    pub duration_ms: u64,
}

impl DimensionResult {
    /// Construct a `Skip` result — used by stub dimensions.
    #[must_use]
    pub fn skipped(id: &str, name: &str, recurring_property: &str, reason: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            recurring_property: recurring_property.to_string(),
            verdict: Verdict::Skip,
            score: None,
            findings: vec![Finding {
                title: "dimension not yet implemented".to_string(),
                severity: Severity::Info,
                evidence: reason.to_string(),
                recurring_property: Some(recurring_property.to_string()),
            }],
            duration_ms: 0,
        }
    }
}

/// The uniform contract every dimension implements.
///
/// Dimensions are deliberately synchronous and self-contained — they
/// can shell out to external tools (cargo, grep, criterion) but they
/// must not require shared state from the driver. This keeps each
/// dimension auditable in isolation.
pub trait Dimension {
    /// Run this dimension and return its result. Implementations should
    /// be deterministic given the workspace state.
    fn run(&self, ctx: &RunContext) -> DimensionResult;
}

/// Shared context every dimension receives at run time.
#[derive(Debug, Clone)]
pub struct RunContext {
    /// Absolute path to the reflective workspace root (the dir
    /// containing `bedrock-platform/`, `mosaic-extensions/`, etc.).
    pub workspace_root: std::path::PathBuf,
    /// Where this run should write any per-run scratch artifacts.
    pub scratch_dir: std::path::PathBuf,
}

/// The full report for one validator run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// ISO-8601 timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Aggregate verdict — minimum of per-dimension verdicts.
    pub aggregate_verdict: Verdict,
    /// Aggregate score — minimum numeric score across dimensions that
    /// produced one. `None` if no dimension produced a score.
    pub aggregate_score: Option<u8>,
    /// Per-dimension results in stable order.
    pub dimensions: Vec<DimensionResult>,
}

impl Report {
    /// Aggregate a slice of dimension results into a Report.
    #[must_use]
    pub fn from_dimensions(dimensions: Vec<DimensionResult>) -> Self {
        let aggregate_verdict = dimensions
            .iter()
            .map(|d| d.verdict)
            .fold(Verdict::Pass, Verdict::worsen_with);

        let aggregate_score = dimensions.iter().filter_map(|d| d.score).min();

        Self {
            timestamp: chrono::Utc::now(),
            aggregate_verdict,
            aggregate_score,
            dimensions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worsen_with_picks_worst() {
        assert_eq!(Verdict::Pass.worsen_with(Verdict::Pass), Verdict::Pass);
        assert_eq!(Verdict::Pass.worsen_with(Verdict::Warn), Verdict::Warn);
        assert_eq!(Verdict::Warn.worsen_with(Verdict::Pass), Verdict::Warn);
        assert_eq!(Verdict::Warn.worsen_with(Verdict::Fail), Verdict::Fail);
        assert_eq!(Verdict::Fail.worsen_with(Verdict::Error), Verdict::Error);
        assert_eq!(Verdict::Skip.worsen_with(Verdict::Pass), Verdict::Pass);
        assert_eq!(Verdict::Pass.worsen_with(Verdict::Skip), Verdict::Pass);
    }

    #[test]
    fn aggregate_score_is_min_not_average() {
        let dims = vec![
            DimensionResult {
                id: "a".into(),
                name: "A".into(),
                recurring_property: "RP-A".into(),
                verdict: Verdict::Pass,
                score: Some(95),
                findings: vec![],
                duration_ms: 0,
            },
            DimensionResult {
                id: "b".into(),
                name: "B".into(),
                recurring_property: "RP-B".into(),
                verdict: Verdict::Warn,
                score: Some(30),
                findings: vec![],
                duration_ms: 0,
            },
        ];
        let report = Report::from_dimensions(dims);
        // Minimum, not (95+30)/2 = 62. A hole stays visible.
        assert_eq!(report.aggregate_score, Some(30));
        assert_eq!(report.aggregate_verdict, Verdict::Warn);
    }
}
