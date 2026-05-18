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
