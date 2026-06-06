// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # SemVer integrity dimension
//!
//! Measures whether public-API changes drive the version bump segment
//! the way SemVer demands: breaking change → major bump; additive
//! change → minor bump; everything else → patch.
//!
//! ## Anchor incident
//!
//! [QF-2026-06-02-04](../../../QUALITY_BACKLOG.md) — `converge` shipped
//! `pack: type Suggestor::provenance() as Provenance, not &'static str`
//! (a breaking trait-signature change) between `v3.9.1` and `v3.9.2`
//! and released the result as a **patch** bump. Sixty-plus downstream
//! crates required code edits — not just version bumps — to compile
//! against converge 3.9.2. Any external consumer who took
//! `converge = "^3.9"` would have broken on next `cargo update`.
//!
//! ## Recurring property
//!
//! [`RP-SEMVER-GATED`](../../../QUALITY_BACKLOG.md#recurring-system-properties).
//!
//! ## Verdict model
//!
//! For each publishable crate at the current `HEAD` vs. its last
//! released tag:
//!
//! - `Fail` — diff classified as `breaking`, declared bump is `patch`
//!   or `minor`.
//! - `Fail` — diff classified as `additive`, declared bump is `patch`.
//! - `Warn` — diff classified as `patch`, declared bump is `minor` or
//!   `major` (over-bump; benign but worth flagging).
//! - `Pass` — diff classification matches declared bump.
//! - Score = `100 * (matching_crates / total_publishable_crates)`.
//!
//! ## Implementation roadmap
//!
//! 1. For each crate with `publish = true` (or absent), find the most
//!    recent `vX.Y.Z` tag on the crate's repo.
//! 2. Run `cargo public-api diff <last-tag>..HEAD --simplified` and
//!    classify the output as `breaking` | `additive` | `patch`.
//! 3. Parse the crate's current `version = "..."` in `Cargo.toml`
//!    against the last tag and derive the declared bump segment.
//! 4. Emit a `Finding` for any mismatch, with the offending API diff
//!    in `evidence`.
//!
//! Tooling notes:
//!
//! - `cargo public-api` is the canonical tool. Install:
//!   `cargo install cargo-public-api`.
//! - For non-Rust crates this dimension reports `Skip` with reason.
//! - For workspaces where `[workspace.package] version` propagates to
//!   all members, every member gets evaluated separately — sub-crates
//!   can introduce breakage even if the workspace version bumped only
//!   by patch.
//!
//! ## Why SemVer integrity matters
//!
//! The release train this dimension was born out of had to yank-and-
//! republish two crates and emergency-patch eleven downstreams because
//! a single trait-signature change rode patch-version coattails into
//! production. Catching that kind of breakage **before** publish is the
//! difference between a release rehearsal and a release incident.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks versioned public-contract compatibility.
pub struct SemverDimension;

impl Dimension for SemverDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "semver",
            "SemVer integrity",
            "RP-SEMVER-GATED",
            "Stub. See this crate's module docs for the cargo-public-api \
             diff classifier. Anchor: QF-2026-06-02-04.",
        )
    }
}
