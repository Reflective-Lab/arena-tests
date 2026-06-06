// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Snapshot portability dimension
//!
//! Measures whether the workspace's golden-output fixtures
//! (`*.stderr`, `*.snap`, `*.golden`, `*.expected`) are portable across
//! machines and resilient to incidental drift. Brittle snapshots are
//! one of the easiest ways an AI-driven workflow can encode a single
//! developer's machine into the test suite without anyone noticing.
//!
//! ## Anchor incident
//!
//! [QF-2026-06-02-06](../../../QUALITY_BACKLOG.md) — `bedrock-platform/
//! organism/crates/pack/tests/compile_fail/fact_no_new.stderr` was
//! blessed via `TRYBUILD=overwrite` and captured the absolute path
//! `/Users/kpernyer/dev/reflective/bedrock-platform/converge/crates/
//! pack/src/fact.rs:1123:5` plus current line numbers. Test passed on
//! one machine; nowhere else. Repaired in commit `3e1a7c8` by restoring
//! `$CARGO/converge-pack-$VERSION/` placeholders and adding a runtime
//! skip guard for `[patch.crates-io]`.
//!
//! ## Recurring property
//!
//! [`RP-SNAPSHOT-PORTABLE`](../../../QUALITY_BACKLOG.md#recurring-system-properties).
//!
//! ## Verdict model
//!
//! For every fixture file in the workspace:
//!
//! - `Fail` — fixture contains an absolute filesystem path under
//!   `/Users/`, `/home/`, `/private/`, `/var/`, or `C:\`.
//! - `Fail` — fixture contains a 4-or-more-digit line gutter (e.g.
//!   `1123 |`) referring to a file outside the fixture's own crate.
//!   These are line numbers from foreign sources that will drift on
//!   every upstream edit.
//! - `Warn` — fixture contains a username embedded as a path component
//!   (regex `/[A-Za-z][A-Za-z0-9_-]+/dev/`).
//! - `Warn` — fixture references a specific `$HOME`-derived path or
//!   contains `$CARGO_HOME` literally (these are typically a placeholder
//!   mistake).
//! - `Pass` — none of the above.
//! - Score = `100 * (clean_fixtures / total_fixtures)`.
//!
//! ## Implementation roadmap
//!
//! 1. Discover fixtures: `find . -type f \( -name '*.stderr' -o
//!    -name '*.snap' -o -name '*.golden' -o -name '*.expected' \)`,
//!    skipping `target/`, `node_modules/`, `.git/`.
//! 2. For each, regex-scan for the patterns above. Use `grep -nE` or
//!    a Rust `regex::Regex` set.
//! 3. Emit a `Finding` per offending file with severity:
//!    - `Critical` for absolute paths (Fail).
//!    - `High` for foreign-file 4+digit gutters (Fail).
//!    - `Moderate` for username paths or `$CARGO_HOME` literals (Warn).
//! 4. Include the file path, line number, and the offending substring
//!    in `evidence`.
//!
//! Implementation notes:
//!
//! - `trybuild` has built-in placeholders (`$CARGO`, `$VERSION`,
//!   `$WORKSPACE`, `$DIR`). When a fixture contains these literally,
//!   that's correct — don't flag.
//! - `insta` snapshots may carry a header with redactions; respect
//!   those.
//! - Some fixtures legitimately reference paths (e.g. an `ls` output
//!   golden). Provide an opt-out marker: a fixture starting with the
//!   line `// arena-snapshot: allow-absolute-paths` skips the
//!   absolute-path check. Use sparingly.
//!
//! ## Why portability matters
//!
//! A test that passes on one machine only is worse than a test that
//! doesn't exist — it gives false confidence and creates a tax every
//! time someone tries to bless or move it. The cheapest defense is a
//! linter run that anyone can audit in one screen.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks whether snapshots replay across machines and storage backends.
pub struct SnapshotPortabilityDimension;

impl Dimension for SnapshotPortabilityDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "snapshot-portability",
            "Snapshot portability",
            "RP-SNAPSHOT-PORTABLE",
            "Stub. See this crate's module docs for the fixture scanner. \
             Anchor: QF-2026-06-02-06.",
        )
    }
}
