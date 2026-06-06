// Copyright 2026 Reflective Labs
// SPDX-License-Identifier: MIT

//! `arena` CLI — runs every quality dimension against the reflective
//! workspace and emits a structured report.
//!
//! ## Usage
//!
//! ```text
//! arena report                       # run every default dimension, print scoreboard
//! arena report --json                # emit machine-readable JSON to stdout
//! arena report --write-history       # also append to reports/history.jsonl
//! arena report --write-latest        # also write reports/latest.md (human-readable)
//! ```
//!
//! With no arguments, `arena` runs all dimensions in stable order and
//! prints the scoreboard.
//!
//! Exit codes:
//! - `0` — aggregate verdict is `Pass`, `Warn`, or `Skip`.
//! - `2` — aggregate verdict is `Fail`.
//! - `3` — aggregate verdict is `Error` (a dimension errored out).

use std::path::PathBuf;

use anyhow::{Context, Result};
use arena_driver::run_default_dimensions;
use arena_metrics::{Report, RunContext, Verdict};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let want_json = args.iter().any(|a| a == "--json");
    let write_history = args.iter().any(|a| a == "--write-history");
    let write_latest = args.iter().any(|a| a == "--write-latest");

    let workspace_root = locate_workspace_root()?;
    let scratch_dir = std::env::temp_dir().join("arena-tests");
    std::fs::create_dir_all(&scratch_dir)
        .with_context(|| format!("create scratch dir {}", scratch_dir.display()))?;
    let ctx = RunContext {
        workspace_root: workspace_root.clone(),
        scratch_dir,
    };

    let report = run_default_dimensions(&ctx);

    if want_json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_scoreboard(&report);
    }

    if write_history {
        let path = workspace_root.join("arena-tests/reports/history.jsonl");
        append_history(&path, &report).with_context(|| format!("append {}", path.display()))?;
    }
    if write_latest {
        let path = workspace_root.join("arena-tests/reports/latest.md");
        write_latest_markdown(&path, &report)
            .with_context(|| format!("write {}", path.display()))?;
    }

    std::process::exit(match report.aggregate_verdict {
        Verdict::Fail => 2,
        Verdict::Error => 3,
        Verdict::Pass | Verdict::Warn | Verdict::Skip => 0,
    });
}

/// Walk up from the current dir looking for the marker file `MASTERPLAN.md`,
/// which lives at the reflective workspace root.
fn locate_workspace_root() -> Result<PathBuf> {
    let mut dir = std::env::current_dir().context("current dir")?;
    loop {
        if dir.join("MASTERPLAN.md").exists() && dir.join("KB").exists() {
            return Ok(dir);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => anyhow::bail!(
                "could not locate reflective workspace root \
                 (looked for MASTERPLAN.md + KB/ walking up from cwd)"
            ),
        }
    }
}

fn print_scoreboard(report: &Report) {
    let verdict_glyph = |v: Verdict| match v {
        Verdict::Pass => "PASS ",
        Verdict::Warn => "WARN ",
        Verdict::Fail => "FAIL ",
        Verdict::Skip => "SKIP ",
        Verdict::Error => "ERROR",
    };

    println!();
    println!("arena quality report — {}", report.timestamp.to_rfc3339());
    println!(
        "aggregate: {}  score: {}",
        verdict_glyph(report.aggregate_verdict),
        report
            .aggregate_score
            .map_or_else(|| "-".to_string(), |s| format!("{s}/100"))
    );
    println!();
    println!(
        "{:<24} {:<6} {:<10} {:<22} duration",
        "dimension", "verd.", "score", "property"
    );
    println!("{}", "-".repeat(80));
    for d in &report.dimensions {
        let score = d
            .score
            .map_or_else(|| "-".to_string(), |s| format!("{s}/100"));
        println!(
            "{:<24} {:<6} {:<10} {:<22} {} ms",
            d.id,
            verdict_glyph(d.verdict),
            score,
            d.recurring_property,
            d.duration_ms,
        );
    }
    println!();
    let total_findings: usize = report.dimensions.iter().map(|d| d.findings.len()).sum();
    if total_findings > 0 {
        println!("{total_findings} finding(s) emitted. Run with --json for detail.");
    }
}

fn append_history(path: &std::path::Path, report: &Report) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string(report)?)?;
    Ok(())
}

fn write_latest_markdown(path: &std::path::Path, report: &Report) -> Result<()> {
    use std::fmt::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut s = String::new();
    writeln!(s, "# Latest arena quality report").unwrap();
    writeln!(s).unwrap();
    writeln!(s, "_Generated {}._", report.timestamp.to_rfc3339()).unwrap();
    writeln!(s).unwrap();
    writeln!(s, "## Aggregate").unwrap();
    writeln!(s).unwrap();
    writeln!(s, "- Verdict: **{:?}**", report.aggregate_verdict).unwrap();
    if let Some(score) = report.aggregate_score {
        writeln!(
            s,
            "- Score: **{score}/100** (minimum across dimensions, not average)"
        )
        .unwrap();
    }
    writeln!(s).unwrap();
    writeln!(s, "## Dimensions").unwrap();
    writeln!(s).unwrap();
    writeln!(s, "| Dimension | Verdict | Score | Property | Duration |").unwrap();
    writeln!(s, "|---|---|---|---|---|").unwrap();
    for d in &report.dimensions {
        let score = d
            .score
            .map_or_else(|| "-".to_string(), |v| format!("{v}/100"));
        writeln!(
            s,
            "| `{}` | {:?} | {} | `{}` | {} ms |",
            d.id, d.verdict, score, d.recurring_property, d.duration_ms,
        )
        .unwrap();
    }
    writeln!(s).unwrap();
    writeln!(s, "## Findings").unwrap();
    writeln!(s).unwrap();
    let mut any = false;
    for d in &report.dimensions {
        for f in &d.findings {
            any = true;
            writeln!(s, "- **[{}] {}** ({:?})", d.id, f.title, f.severity).unwrap();
            writeln!(s, "  - Evidence: {}", f.evidence).unwrap();
            if let Some(rp) = &f.recurring_property {
                writeln!(s, "  - Property: {rp}").unwrap();
            }
        }
    }
    if !any {
        writeln!(s, "_No findings emitted this run._").unwrap();
    }
    std::fs::write(path, s)?;
    Ok(())
}
