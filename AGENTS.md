# Arena Tests — Cross-Extension Integration Harness

This repo validates how platform and extension crates compose together. It is
test-only: no production code, no release surface, and no downstream shortcuts
around the public Converge and Organism APIs.

## Scope

- Cross-extension smoke tests live under `crates/cross-extension-smoke/tests/`.
- Reusable test fixtures live in small publish-false crates under `crates/`.
- Keep tests deterministic. Avoid wall-clock assertions unless timestamps are
  fixed or stripped.
- Do not move platform contracts into this repo. Foundation code stays in
  `bedrock-platform/converge`; formation and intent machinery stays in
  `bedrock-platform/organism`; extension behavior stays in the mosaic repos.

## Dependency Rules

- Platform and extension crates must resolve from local paths in `Cargo.toml`.
- This repo may depend on platform and extension crates for tests, but those
  crates must not depend back on `arena-tests`.
- Prefer public crates: `converge-pack`, `converge-kernel`, `converge-model`,
  `organism-pack`, `organism-runtime`, and extension public crates.
- No `unsafe` code.

## Validation

Use:

```bash
cargo test --workspace
```

For focused work:

```bash
cargo test -p cross-extension-smoke
```

## Editing Guidance

- Add a new member crate when a fixture should be reused by another repo.
- Add a new `tests/*.rs` file when the case belongs only to the smoke suite.
- Keep fixtures small and named by business intent, not implementation detail.

## Cursor Cloud specific instructions

This workspace is a thin test/validator harness that depends on a whole sibling
checkout of the Reflective stack. The non-obvious parts of running it here:

- **Sibling repos resolve from the filesystem root.** `Cargo.toml`'s path deps
  and `[patch.crates-io]` use `../bedrock-platform`, `../mosaic-extensions`,
  `../atelier-showcase`. Because this repo is checked out at `/workspace`, those
  resolve to `/bedrock-platform/{converge,organism,helms}`,
  `/mosaic-extensions/{arbiter-policy,mnemos-knowledge,prism-analytics,manifold-adapters,embassy-ports}`,
  and `/atelier-showcase`. These are separate GitHub repos under
  `Reflective-Lab` cloned into those container dirs. They are provisioned in the
  VM snapshot, not in this repo — **any** `cargo` command fails to even resolve
  until they exist (eager `[patch.crates-io]`). If they are missing, re-clone the
  `Reflective-Lab` repos into those paths.
- **The `arena` CLI needs a workspace-root marker.** It walks up from the cwd
  looking for `MASTERPLAN.md` + `KB/` to find the "reflective workspace root".
  In this layout the root is `/`, where stand-in `MASTERPLAN.md` and `KB/` are
  provisioned (snapshot). Run `cargo run --bin arena -- report` (or
  `./target/debug/arena report`) from anywhere under `/workspace`. All quality
  dimensions are stubs today, so the scoreboard reports `SKIP`/aggregate `PASS`.
- **`intent_codec_applets` smoke test compiles in external KB data.** It uses
  `include_str!(".../KB/02-product/applets/*.intent.json")`, resolving to
  `/KB/02-product/applets/`. Those manifests live in the (separate, not always
  accessible) reflective KB monorepo; stand-in fixtures are provisioned at
  `/KB/...` in the snapshot. If the suite fails to compile with
  `couldn't read .../KB/...`, that data is missing.
- **Toolchain** is pinned to `1.96.0` (edition 2024) via `rust-toolchain.toml`.
- **`just` is not installed.** Run the underlying cargo commands directly (see
  `Justfile` / `README.md`): `cargo test --workspace`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`.
  The "patch ... was not used in the crate graph" warnings during build/clippy
  are expected (the patch table is a superset of what the test members pull).
- **`counterparty-kyc-convergence` is live-by-default** and exits non-zero
  without `--mock-ok`. Use `cargo run -p arena-counterparty-kyc-convergence --
  --mock-ok` for an offline, deterministic end-to-end run.
