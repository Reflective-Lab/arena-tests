// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Determinism dimension
//!
//! Measures whether the test suite is deterministic: the same source
//! and environment, executed N times, produces N identical pass/fail
//! patterns. Flakiness is the loudest signal that some hidden state
//! (wall clock, RNG without seed, ordering of `HashMap` iteration,
//! network jitter, concurrent test pollution) is leaking into the
//! suite.
//!
//! ## Anchor
//!
//! Pre-emptive. No specific incident yet — this dimension exists to
//! prevent the *class* of incident where a test goes red on Tuesday,
//! green on Wednesday, and quietly gets `#[ignore]`-d into the dark.
//! Without a flake-rate measurement, we don't know whether the suite is
//! deterministic; with one, we have a number to defend.
//!
//! ## Recurring property
//!
//! [`RP-DETERMINISM`](../../../QUALITY_BACKLOG.md#recurring-system-properties).
//!
//! ## Verdict model
//!
//! Run the contract suite (`crates/cross-extension-smoke`,
//! `crates/intent-cases`, `crates/counterparty-kyc-convergence`) **N**
//! times back-to-back (default `N = 5`). For each test:
//!
//! - **Stable** — same verdict across all N runs.
//! - **Flaky** — at least one run disagreed with the others.
//!
//! Outcomes:
//!
//! - `Fail` — any test flaked.
//! - `Warn` — none flaked but a run's wall-clock varies > 50% across
//!   the N runs (suggests hidden contention).
//! - `Pass` — all stable, wall-clock variance < 50%.
//! - Score = `100 * (stable_tests / total_tests)`.
//!
//! ## Implementation roadmap
//!
//! 1. Read `N` from `RunContext` (env override
//!    `ARENA_DETERMINISM_RUNS`, default 5).
//! 2. For each run, invoke `cargo nextest run -j 1 --no-fail-fast
//!    --message-format json` (single-thread to surface ordering issues;
//!    nextest gives us a per-test JSON record).
//! 3. Aggregate per-test verdicts across runs; flag any mismatch as
//!    flaky and record the divergent run number.
//! 4. Compute median and 95th-percentile wall-clock per test; flag
//!    high-variance tests.
//! 5. Emit Findings per flaky test with the run-by-run verdict
//!    sequence in `evidence`.
//!
//! Implementation notes:
//!
//! - Use `nextest` (cargo install cargo-nextest) for fast, structured
//!   output. Falls back to parsing `cargo test --format=json` if
//!   nextest isn't installed; emit a Warn finding suggesting install.
//! - Set `RUST_TEST_THREADS=1` for the variance measurement run so the
//!   timing is meaningful; relax for the verdict-stability runs.
//! - Seed `RUST_LOG=warn` and unset `RUST_BACKTRACE` to keep output
//!   minimal.
//!
//! ## Why determinism matters
//!
//! An AI-driven workflow that re-runs failing tests is a workflow that
//! eventually stops noticing real failures. Flake rate is the cheapest
//! early-warning signal for hidden coupling — between tests, with the
//! environment, with time, or with concurrency. Tracking it as a
//! dimension makes the cost of "just retry the test" legible.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks repeatability of test and scenario results.
pub struct DeterminismDimension;

impl Dimension for DeterminismDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "determinism",
            "Determinism",
            "RP-DETERMINISM",
            "Stub. See this crate's module docs for the N-rerun + variance \
             flake-rate harness. Pre-emptive (no specific incident anchor).",
        )
    }
}
