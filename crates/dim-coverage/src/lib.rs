// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Coverage dimension
//!
//! Measures line, branch, and function coverage across the production
//! crates that the contract suite is meant to exercise. Coverage is
//! kept as a **trend metric**, not a gate — a numeric absolute target
//! invites perverse incentives (tests that paint coverage lines but
//! don't assert behavior). We watch the *direction*.
//!
//! ## Anchor
//!
//! Pre-emptive. The release train this dimension was born from
//! exercised contract tests that proved end-to-end composition works,
//! but we had no visibility into which crate-internal branches were
//! covered. When a crate ships a breaking change, low coverage in
//! the changed file is a load-bearing risk signal.
//!
//! ## Recurring property
//!
//! No existing `RP-*` covers this directly. **Proposed:**
//! `RP-COVERAGE-TREND` — line coverage on the platform train (converge,
//! axiom, organism, helms, mosaic-* libs) does not regress across two
//! consecutive review cycles.
//!
//! ## Verdict model
//!
//! Per-crate line coverage from `cargo llvm-cov`. Compare to the prior
//! run stored in `arena-tests/baselines/coverage.json`.
//!
//! - `Fail` — any crate's line coverage dropped by > 5 percentage
//!   points since the prior baseline.
//! - `Warn` — any crate dropped 1–5 points, or a brand-new crate has
//!   line coverage < 30%.
//! - `Pass` — no crate dropped, no new crate below 30%.
//! - Score = clamp(0, 100, `100 - max_regression_pp * 5`).
//!
//! ## Implementation roadmap
//!
//! 1. `cargo install cargo-llvm-cov` (one-time, document in
//!    `bootstrap`).
//! 2. For each workspace in the train, run:
//!    ```text
//!    cargo llvm-cov --workspace --no-clean --json --output-path
//!        $SCRATCH/cov-<crate>.json
//!    ```
//! 3. Parse JSON, extract per-crate line/branch/function coverage.
//! 4. Read `arena-tests/baselines/coverage.json`; diff. If absent,
//!    treat current run as baseline and emit a Warn finding noting
//!    that no prior baseline existed.
//! 5. Emit a Finding per regressing crate with the delta in
//!    `evidence`. On Pass, optionally update the baseline (gated
//!    behind `--update-baseline`).
//!
//! Implementation notes:
//!
//! - Run coverage with `RUSTFLAGS="-C instrument-coverage"` requires a
//!   from-scratch build the first time. Subsequent runs are
//!   incremental.
//! - Coverage of `tests/` themselves is meaningless; exclude.
//! - Excluded paths should be configurable per workspace via
//!   `arena-tests/baselines/coverage-exclude.json`.
//!
//! ## Why coverage matters (as a trend, not a target)
//!
//! Coverage as a hard target is a known anti-pattern — engineers will
//! write trivial tests to hit lines without testing behavior. Coverage
//! as a **trend** answers a different question: "are the parts of the
//! codebase that change *also* the parts that are tested?" When
//! coverage drops sharply in a release-train crate that just shipped a
//! breaking change, that's a signal — not a verdict.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks test coverage and contract coverage signals.
pub struct CoverageDimension;

impl Dimension for CoverageDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "coverage",
            "Coverage trend",
            "RP-COVERAGE-TREND",
            "Stub. See this crate's module docs for the cargo-llvm-cov \
             driver + baseline diff. Pre-emptive (proposes new RP).",
        )
    }
}
