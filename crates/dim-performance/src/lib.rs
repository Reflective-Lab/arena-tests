// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Performance envelope dimension
//!
//! Measures whether key platform operations stay within their declared
//! latency / throughput / memory budgets across runs. Captures
//! microbenchmarks for hot paths and flags regressions vs. a stored
//! baseline.
//!
//! ## Anchor
//!
//! Pre-emptive. No specific incident yet. This dimension exists so a
//! 2× slowdown in `Engine::run` doesn't make it to production unnoticed
//! because all our other gates are correctness-only.
//!
//! ## Recurring property
//!
//! No existing `RP-*` covers this directly. **Proposed:**
//! `RP-PERFORMANCE-ENVELOPE` — declared microbenchmarks for hot paths
//! do not regress by more than 20% (latency) or 10% (memory) without
//! an explicit ADR justifying the change.
//!
//! ## Benchmark surface (initial)
//!
//! These are the operations whose performance directly affects perceived
//! product quality. Each gets a Criterion benchmark in this crate.
//!
//! 1. `Engine::run` against a 1 000-proposal context.
//! 2. `Suggestor` invocation overhead (sync trait dispatch, no I/O).
//! 3. `ProposedFact::new` allocation cost.
//! 4. `Provenance` construction via `impl Into<Provenance>` (added in
//!    converge 3.9.2 — confirm the new generic doesn't measurably
//!    regress vs. the old `Provenance`-only signature).
//! 5. `ContextState::add_input_with_provenance` end-to-end.
//! 6. Truth package parse + validate (axiom).
//!
//! ## Verdict model
//!
//! For each benchmark, compare median wall-clock to the stored baseline
//! in `arena-tests/baselines/performance.json`.
//!
//! - `Fail` — latency regression > 50% on any benchmark.
//! - `Warn` — latency regression 20–50%, or memory regression > 10%.
//! - `Pass` — all within budget.
//! - Score = clamp(0, 100, `100 - max_regression_pct * 2`).
//!
//! ## Implementation roadmap
//!
//! 1. Add `[[bench]]` targets in this crate for each operation above.
//!    Use `criterion = "0.5"` as the bench framework.
//! 2. The dimension's `run()` shells out to
//!    `cargo bench --message-format=json -p dim-performance`.
//! 3. Parse the JSON, extract per-benchmark median (criterion emits
//!    `mean`, `median`, `std_dev`, `slope`).
//! 4. Read `arena-tests/baselines/performance.json`; diff. Emit
//!    Findings per regression. On Pass, optionally update baseline
//!    behind `--update-baseline`.
//!
//! Implementation notes:
//!
//! - Criterion benches need `--bench` flag (not `cargo test`).
//! - Cold-cache vs. warm-cache results differ wildly. Use criterion's
//!   warm-up phase; do not run benches in parallel with the other
//!   dimensions.
//! - Set `criterion::Criterion::default().sample_size(50)` to keep
//!   runs short enough for CI.
//! - Memory measurement requires `dhat-rs` or `peakmem` — defer to
//!   phase 2.
//!
//! ## Why performance matters
//!
//! Correctness is the first gate; latency is the second. Many subtle
//! quality regressions land as 10-30% slowdowns that nobody notices
//! until customers complain. Tracking microbenchmarks as a dimension
//! puts a number on a thing that otherwise drifts silently.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks performance budget drift.
pub struct PerformanceDimension;

impl Dimension for PerformanceDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "performance",
            "Performance envelope",
            "RP-PERFORMANCE-ENVELOPE",
            "Stub. See this crate's module docs for the Criterion bench \
             driver + baseline diff. Pre-emptive (proposes new RP).",
        )
    }
}
