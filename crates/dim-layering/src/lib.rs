// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Layering dimension
//!
//! Measures whether the cross-repo dependency graph respects the
//! architectural train. Two invariants:
//!
//! 1. **Publishability monotone.** A publishable crate (`publish = true`
//!    or absent in Cargo.toml) must not depend on a `publish = false`
//!    crate. Otherwise the publishable crate cannot ship without
//!    leaking an internal dependency.
//! 2. **Train direction.** Dependencies flow downstream-to-upstream along
//!    the declared release train order:
//!    `converge → axiom → organism → helms → mosaic-* → atelier →
//!    arena → runway → commerce-rails`. A crate in an earlier train
//!    position must not depend on a crate in a later one.
//!
//! ## Anchor incidents
//!
//! - [QF-2026-06-02-08](../../../QUALITY_BACKLOG.md) — `runway-accounts`
//!   and `runway-app-host` carry path-deps on `commerce-rails-stripe`,
//!   which is `publish = false` and UNLICENSED. Both runway crates
//!   therefore cannot be published. The wall hit at release time;
//!   nothing in the workspace warned.
//! - [QF-2026-06-02-13](../../../QUALITY_BACKLOG.md) — `arena-tests` and
//!   `atelier-showcase` had `../stack/bedrock-platform/...` path-deps
//!   that broke silently after the stack/ flatten. A direction-aware
//!   audit would have flagged it.
//!
//! ## Recurring property
//!
//! [`RP-LAYERING`](../../../QUALITY_BACKLOG.md#recurring-system-properties).
//!
//! ## Verdict model
//!
//! - `Fail` — at least one publishable crate depends on an
//!   un-publishable crate.
//! - `Fail` — at least one crate has an upstream-pointing dependency
//!   (later train position depends on earlier, in violation of
//!   downstream-to-upstream order).
//! - `Warn` — workspace has path-deps without `version =` fields (these
//!   are publish-time blockers waiting to happen).
//! - `Pass` — neither.
//! - Score = `100 - (violations * 10)`, floored at 0.
//!
//! ## Implementation roadmap
//!
//! 1. Walk every `Cargo.toml` under the reflective root.
//! 2. For each, read:
//!    - `package.publish` (default `true`)
//!    - `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`
//!    - `[workspace.dependencies]` (separately)
//! 3. Resolve each `{ path = "..." }` dependency to the target crate's
//!    `Cargo.toml` and read its `publish` flag.
//! 4. Build a graph; assign each crate to its train position by repo.
//! 5. Walk the edges, emitting Findings for any violation. Cite the
//!    specific `Cargo.toml` line.
//!
//! Implementation notes:
//!
//! - Use `cargo_metadata` or hand-parse with `toml_edit` for fidelity.
//! - Dev-dependencies that point at sibling workspace crates are OK
//!   without `version =` because cargo strips dev-deps when publishing
//!   (this is documented behavior). Distinguish dev from runtime deps.
//! - Be lenient with sub-workspace internal deps (a crate depending on
//!   a sibling in the same repo's workspace is the normal case and
//!   gets a `version = "X.Y.Z", path = "..."` pattern, not a layering
//!   violation).
//!
//! ## Why layering matters
//!
//! Architecture is the dependency graph. Once that graph carries cycles
//! or upstream pointers, every release becomes a coordination problem.
//! Catching layering drift at PR time is much cheaper than catching it
//! during a release rehearsal.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks that crates respect platform and extension dependency direction.
pub struct LayeringDimension;

impl Dimension for LayeringDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "layering",
            "Layering",
            "RP-LAYERING",
            "Stub. See this crate's module docs for the dep-graph walker. \
             Anchors: QF-2026-06-02-08, QF-2026-06-02-13.",
        )
    }
}
