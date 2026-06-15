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

`arena-tests` is a thin harness over a full sibling checkout of the Reflective
stack. Run everything from `/workspace`:

| Task | Command |
|------|---------|
| Test (primary validation) | `cargo test --workspace` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Format check | `cargo fmt --check` |
| Run the `arena` validator | `cargo run --bin arena -- report` |
| Run the KYC arena (offline) | `cargo run -p arena-counterparty-kyc-convergence -- --mock-ok` |

Notes that save time:

- `just` is **not** installed — use the cargo commands above (the `Justfile`
  only wraps them). Toolchain is pinned to `1.96.0` / edition 2024 via
  `rust-toolchain.toml`.
- `patch ... was not used in the crate graph` warnings are expected (the
  `[patch.crates-io]` table is a superset of what the test members pull).
- The `arena` dimensions are all stubs today, so `arena report` prints
  `SKIP`/aggregate `PASS` — that is the healthy result, not a failure.

### Out-of-repo provisioning (one command if missing)

This repo does **not** track three things it needs at runtime, all of which
normally persist in the VM snapshot:

1. Sibling repos at the filesystem root (parent of `/workspace`), because
   `Cargo.toml` path deps + eager `[patch.crates-io]` resolve there — so **no
   cargo command resolves until they exist**:
   `/bedrock-platform/{converge,organism,helms}`,
   `/mosaic-extensions/{arbiter-policy,mnemos-knowledge,prism-analytics,manifold-adapters,embassy-ports}`,
   `/atelier-showcase`.
2. `/MASTERPLAN.md` + `/KB/` — the marker the `arena` CLI walks up to find ("could
   not locate reflective workspace root" means this is missing).
3. `/KB/02-product/applets/*.intent.json` — manifests the `intent_codec_applets`
   smoke test `include_str!`s (`couldn't read .../KB/...` means these are
   missing). The canonical reflective KB monorepo is not reachable here, so
   tracked stand-ins live in [`bootstrap/KB/`](bootstrap/KB) and mirror
   `crates/intent-cases/src/lib.rs::APPLET_CASES`.

If any of the above is missing on a fresh VM, restore it all idempotently with:

```bash
bash bootstrap/restore-env.sh
```
