# integration-tests

Cross-extension integration tests for the Converge stack. This repo is the only place where the platform and extension crates are wired together in one workspace, so it can catch regressions that single-crate tests miss.

## Why this lives here, not in `platform/`

Platform crates must not depend on extensions (the dependency graph is one-way: extensions → platform). Putting cross-extension tests inside any platform crate would invert that direction. This repo sits at the same level as `platform/` and `extensions/` and consumes both via `dev-dependencies` and a `[patch.crates-io]` block, so the platform stays clean while still getting integration coverage.

## Layout

- `crates/cross-extension-smoke/` — first member. Lib is empty; tests live in `tests/`.
- Add a new member crate per scenario as the suite grows (e.g., `crates/mnemos-arbiter-flow`, `crates/manifold-ferrox-budget`).

## Running

```sh
cd ~/dev/reflective/stack/integration-tests
cargo test --workspace
```

This pulls every platform and extension crate from the local checkout. The first build is slow; incremental is fast.

## Adding a test

1. Create a new member under `crates/<scenario>/` and add it to the root `Cargo.toml` `members` list.
2. Pull whatever platform + extension crates your scenario needs from `[workspace.dependencies]`.
3. Put the actual test in `tests/<name>.rs` — keep `src/lib.rs` empty unless you need shared helpers.
4. Run `cargo test -p <scenario>`.

## Contract

- No production code. This is a test-only repo (`publish = false` everywhere).
- Don't pull from crates.io for any platform/extension crate — always use the local path. If a `[patch.crates-io]` entry is missing for a transitive dep, add it.
- Keep tests deterministic. Wall-clock comparisons must strip timestamps (see `engine_converges_deterministically` in `converge-core`'s test for the pattern).
