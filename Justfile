set shell := ["zsh", "-cu"]

# Run every quality dimension and print the scoreboard.
default: report

# Print the scoreboard for every default dimension.
report:
    cargo run --quiet --bin arena -- report

# Same, plus persist a JSON line to reports/history.jsonl and
# refresh reports/latest.md.
report-write:
    cargo run --quiet --bin arena -- report --write-history --write-latest

# Emit machine-readable JSON to stdout.
report-json:
    cargo run --quiet --bin arena -- report --json

# Run only the existing contract suite (no dimension reporting).
contracts:
    cargo test --workspace --all-targets

# Build everything.
build:
    cargo build --workspace --all-targets

# Run the arena-metrics unit tests (aggregation/Verdict invariants).
test-metrics:
    cargo test -p arena-metrics --all-targets

# Clippy clean across the validator.
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format check.
fmt-check:
    cargo fmt --check

# rustfmt in place.
fmt:
    cargo fmt
