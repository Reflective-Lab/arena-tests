# arena-tests

Cross-extension integration and arena tests for the Reflective stack. This repo
wires Bedrock and Mosaic crates together in one test-only workspace, so it can
catch regressions and composition gaps that single-repo tests miss.

## Boundary

> Owns: cross-extension integration + contract-shape pressure tests. **Test code only** (`publish = false` everywhere) — provides the dependency-direction firewall (Bedrock must not depend on Mosaic). Does NOT own: any production code; CI for Bedrock or Mosaic themselves.

— Canonical claim: [Arena Tests](https://github.com/Reflective-Lab/reflective/blob/main/KB/04-architecture/current-system-map.md#arena-tests) in the boundary registry. Update there first; this README quotes that source.

## Why this lives here, not in Bedrock or Mosaic

Bedrock crates must not depend on Mosaic extensions. Putting cross-extension
tests inside Converge, Organism, Axiom, or Helm would invert the dependency
direction. This repo sits at the workspace root and consumes Bedrock plus
Mosaic through local path dependencies, so the platform stays clean while still
getting integration coverage.

## Layout

- `crates/cross-extension-smoke/` — smoke and composition tests; most logic
  lives in `tests/`.
- `crates/intent-cases/` — shared business-intent fixtures used by Organism
  routing tests.
- `crates/counterparty-kyc-convergence/` — live-by-default arena binary for
  counterparty identity, sanctions, and procurement evidence.
- Add a new member crate per scenario as the suite grows.

Current claim-portfolio coverage:

- Expense non-finance high-value commit exemplar.
- Strict HITL rejection when approval would not change the Cedar decision.
- Vendor due-diligence gate.
- Flow phase commit gate.
- Data-classification PII block.

## Running

```sh
cd ~/dev/reflective/arena-tests
cargo test --workspace
```

This pulls Bedrock, Mosaic, and Atelier crates from the local checkout. The
first build is slow; incremental is fast.

## Adding a test

1. Create a new member under `crates/<scenario>/` and add it to the root `Cargo.toml` `members` list.
2. Pull whatever Bedrock, Mosaic, or Atelier crates your scenario needs from `[workspace.dependencies]`.
3. Put the actual test in `tests/<name>.rs` — keep `src/lib.rs` empty unless you need shared helpers.
4. Run `cargo test -p <scenario>`.

## Contract

- No production code. This is a test-only repo (`publish = false` everywhere).
- Don't pull from crates.io for any Bedrock or Mosaic crate — always use the local path. If a `[patch.crates-io]` entry is missing for a transitive dep, add it.
- Keep tests deterministic. Wall-clock comparisons must strip timestamps (see `engine_converges_deterministically` in `converge-core`'s test for the pattern).
