// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! Driver utilities for arena quality dimensions.

use arena_metrics::{Dimension, Report, RunContext};

/// Return the quality dimensions in stable report order.
#[must_use]
pub fn default_dimensions() -> Vec<Box<dyn Dimension>> {
    vec![
        Box::new(dim_hermeticity::HermeticityDimension),
        Box::new(dim_semver::SemverDimension),
        Box::new(dim_layering::LayeringDimension),
        Box::new(dim_snapshot_portability::SnapshotPortabilityDimension),
        Box::new(dim_determinism::DeterminismDimension),
        Box::new(dim_coverage::CoverageDimension),
        Box::new(dim_crate_footprint::CrateFootprintDimension),
        Box::new(dim_performance::PerformanceDimension),
    ]
}

/// Run every default dimension and aggregate the report.
#[must_use]
pub fn run_default_dimensions(ctx: &RunContext) -> Report {
    let dimensions = default_dimensions()
        .into_iter()
        .map(|dimension| dimension.run(ctx))
        .collect();
    Report::from_dimensions(dimensions)
}
