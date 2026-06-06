# Latest arena quality report

_Generated 2026-06-02T14:33:34.781210+00:00._

## Aggregate

- Verdict: **Pass**

## Dimensions

| Dimension | Verdict | Score | Property | Duration |
|---|---|---|---|---|
| `hermeticity` | Skip | - | `RP-HERMETIC-UNIT` | 0 ms |
| `semver` | Skip | - | `RP-SEMVER-GATED` | 0 ms |
| `layering` | Skip | - | `RP-LAYERING` | 0 ms |
| `snapshot-portability` | Skip | - | `RP-SNAPSHOT-PORTABLE` | 0 ms |
| `determinism` | Skip | - | `RP-DETERMINISM` | 0 ms |
| `coverage` | Skip | - | `RP-COVERAGE-TREND` | 0 ms |
| `crate-footprint` | Skip | - | `RP-CRATE-SIZE-BUDGET` | 0 ms |
| `performance` | Skip | - | `RP-PERFORMANCE-ENVELOPE` | 0 ms |

## Findings

- **[hermeticity] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the implementation roadmap (sandboxed cargo-test runner, socket/env/fs accounting). Anchor: QF-2026-06-02-05.
  - Property: RP-HERMETIC-UNIT
- **[semver] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the cargo-public-api diff classifier. Anchor: QF-2026-06-02-04.
  - Property: RP-SEMVER-GATED
- **[layering] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the dep-graph walker. Anchors: QF-2026-06-02-08, QF-2026-06-02-13.
  - Property: RP-LAYERING
- **[snapshot-portability] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the fixture scanner. Anchor: QF-2026-06-02-06.
  - Property: RP-SNAPSHOT-PORTABLE
- **[determinism] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the N-rerun + variance flake-rate harness. Pre-emptive (no specific incident anchor).
  - Property: RP-DETERMINISM
- **[coverage] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the cargo-llvm-cov driver + baseline diff. Pre-emptive (proposes new RP).
  - Property: RP-COVERAGE-TREND
- **[crate-footprint] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the cargo-package size accountant. Anchor: QF-2026-06-02-09.
  - Property: RP-CRATE-SIZE-BUDGET
- **[performance] dimension not yet implemented** (Info)
  - Evidence: Stub. See this crate's module docs for the Criterion bench driver + baseline diff. Pre-emptive (proposes new RP).
  - Property: RP-PERFORMANCE-ENVELOPE
