// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! # Hermeticity dimension
//!
//! Measures whether unit tests run hermetically: **zero outbound network**
//! requests, **zero reads** of API-key-shaped env vars (`*_API_KEY`,
//! `*_TOKEN`, `*_SECRET`), and **zero filesystem writes** outside
//! `TempDir` / `tempfile`-scoped paths.
//!
//! ## Anchor incident
//!
//! [QF-2026-06-02-05](../../../QUALITY_BACKLOG.md) — `axiom-truth@0.15.1`
//! shipped a unit test that read `OPENAI_API_KEY` from the developer's
//! `.envrc` and issued real billable LLM API calls during `cargo test`.
//! Fixed in `axiom-truth@0.15.2` via a dependency-injected backend
//! selector. The dimension exists so we never have to fix the same class
//! of leak twice.
//!
//! ## Recurring property
//!
//! [`RP-HERMETIC-UNIT`](../../../QUALITY_BACKLOG.md#recurring-system-properties).
//!
//! ## Verdict model
//!
//! - `Fail` — any unit test issued a successful TCP/UDP connect during
//!   its run.
//! - `Warn` — any unit test read an env var matching the credential
//!   regex (above) but did not open a socket.
//! - `Pass` — neither.
//! - Score = `100 * (hermetic_tests / total_tests)`.
//!
//! ## Implementation roadmap
//!
//! 1. Discover every workspace under the reflective root that has a
//!    `Cargo.toml` with a `[lib]` or `[[bin]]` plus tests.
//! 2. For each, run the unit test target under a sandbox:
//!    - **Network blocked** at the kernel level (`unshare --net` on
//!      Linux; on macOS, link a stub `libc::connect` via
//!      `DYLD_INTERPOSE`, or set `https_proxy=http://127.0.0.1:1` and
//!      assert no connect attempts succeed).
//!    - Well-known credential env vars (`OPENAI_API_KEY`,
//!      `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `AWS_*`, `GITHUB_TOKEN`,
//!      …) unset.
//!    - `TMPDIR` redirected to a per-test scratch dir.
//! 3. Record per-test:
//!    - Count of attempted socket opens. Must be 0.
//!    - Count of env reads matching the credential regex. Must be 0.
//!    - Count of filesystem writes outside `TMPDIR`. Must be 0.
//! 4. Each violation emits a `Finding` with severity `High` by default,
//!    `Critical` if a real network connection succeeded.
//!
//! ## Why hermeticity matters
//!
//! Hermetic tests are the cheapest correctness signal we have. The
//! moment a unit test depends on dev-machine env, it stops testing the
//! code and starts testing the machine. We lose the ability to refactor
//! safely, we lose deterministic CI, and — as
//! [QF-2026-06-02-05](../../../QUALITY_BACKLOG.md) showed — we start
//! paying real money for `cargo test`.

use arena_metrics::{Dimension, DimensionResult, RunContext};

/// Checks whether tests avoid undeclared network, filesystem, and service dependencies.
pub struct HermeticityDimension;

impl Dimension for HermeticityDimension {
    fn run(&self, _ctx: &RunContext) -> DimensionResult {
        DimensionResult::skipped(
            "hermeticity",
            "Hermeticity",
            "RP-HERMETIC-UNIT",
            "Stub. See this crate's module docs for the implementation \
             roadmap (sandboxed cargo-test runner, socket/env/fs \
             accounting). Anchor: QF-2026-06-02-05.",
        )
    }
}
