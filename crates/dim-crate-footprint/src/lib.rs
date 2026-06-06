// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Crate footprint dimension
//!
//! Measures the on-the-wire size of each publishable crate's
//! `cargo package` artifact against the crates.io 10 MiB upload limit
//! and a project-set soft budget. Tracks total runtime-dependency
//! count as a complementary signal of incidental surface area.
//!
//! ## Anchor incident
//!
//! [QF-2026-06-02-09](../../../QUALITY_BACKLOG.md) — `cargo publish -p
//! runway-storage-contract` failed mid-release with `HTTP 413 Payload
//! Too Large; max upload size is 10485760`. The crate had grown past
//! 10 MiB silently; the release halted; we diagnosed by hand. We
//! didn't ship `runway-storage-contract` at all in v3.4.2.
//!
//! ## Recurring property
//!
//! [`RP-CRATE-SIZE-BUDGET`](../../../QUALITY_BACKLOG.md#recurring-system-properties).
//!
//! ## Verdict model
//!
//! For each publishable crate:
//!
//! - `Fail` — `cargo package`-produced `.crate` exceeds 10 MiB
//!   (crates.io hard limit).
//! - `Warn` — exceeds the soft budget (default 5 MiB; per-crate
//!   override in `arena-tests/baselines/crate-sizes.json`).
//! - `Warn` — grew by > 20% vs. the prior baseline.
//! - `Pass` — within soft budget and within 20% of prior.
//! - Score = `100` if all pass, `90 - 10 * fail_count - 2 * warn_count`
//!   otherwise, floored at 0.
//!
//! ## Implementation roadmap
//!
//! 1. Walk every crate with `publish = true` (or absent).
//! 2. For each, run `cargo package --list --allow-dirty -p <crate>`
//!    to enumerate included files; compute total bytes.
//! 3. For a precise measurement, run `cargo package --allow-dirty -p
//!    <crate>` and `stat` the resulting `.crate` in `target/package/`.
//!    This is slower but accurate (the `.crate` is gzipped).
//! 4. Read `arena-tests/baselines/crate-sizes.json`; diff. Emit a
//!    Finding per regression with `evidence` containing the size band
//!    and the largest files in the package.
//! 5. Specifically flag large test fixtures, binary blobs, and
//!    accidentally-included `target/` content via
//!    `package.include` / `.exclude`.
//!
//! Implementation notes:
//!
//! - Many crates accidentally include `kb/`, `examples/`, or
//!   generated artifacts because their `Cargo.toml` lacks
//!   `[package] include = [...]`. The dimension should suggest the
//!   `include = [...]` set when it sees clear bloat.
//! - The 10 MiB limit is crates.io's; private registries may differ.
//! - The grew-by-20% check needs a baseline file. First run records
//!   it.
//!
//! ## Why footprint matters
//!
//! Crate size is a leading indicator of poor `include` hygiene and of
//! accidental data shipping. A crate that grows from 200 KiB to 8 MiB
//! over a few releases is almost always carrying something it
//! shouldn't (vendored test data, generated docs, snapshots that
//! belong in `target/`). Catching that drift before the 10 MiB cliff
//! avoids the kind of mid-release wall we hit on June 2.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks dependency and binary footprint drift.
pub struct CrateFootprintDimension;

impl Dimension for CrateFootprintDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "crate-footprint",
            "Crate footprint",
            "RP-CRATE-SIZE-BUDGET",
            "Stub. See this crate's module docs for the cargo-package \
             size accountant. Anchor: QF-2026-06-02-09.",
        )
    }
}
