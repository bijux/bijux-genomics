//! Owner: bijux-bench
//! Typed diffs and effect sizes between runs.
//! Must not perform IO.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::compare::stratify::CompareStratum;
use crate::repo::{load_manifest, load_metrics_map, RunIndexRepository, RunRepository};

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricDiff {
    pub metric_id: String,
    pub absolute: f64,
    pub relative: Option<f64>,
    pub practical: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareReport {
    pub run_a: String,
    pub run_b: String,
    pub tool_changed: bool,
    pub command_changed: bool,
    pub diffs: Vec<MetricDiff>,
    pub strata: Vec<CompareStratum>,
}

pub fn compare_runs(
    run_a: &str,
    run_b: &str,
    index_path: &Path,
    artifacts_root: &Path,
) -> Result<CompareReport> {
    ensure_repo_exists(index_path)?;
    let repo = RunIndexRepository::new(index_path.to_path_buf(), artifacts_root.to_path_buf());
    compare_runs_with_repo(run_a, run_b, &repo)
}

pub fn compare_runs_with_repo(
    run_a: &str,
    run_b: &str,
    repo: &dyn RunRepository,
) -> Result<CompareReport> {
    let meta_a = repo.run_metadata(run_a)?;
    let meta_b = repo.run_metadata(run_b)?;
    let manifest_a = load_manifest(&meta_a.manifest_path)?;
    let manifest_b = load_manifest(&meta_b.manifest_path)?;
    let metrics_a = load_metrics_map(&meta_a.metrics_path)?;
    let metrics_b = load_metrics_map(&meta_b.metrics_path)?;

    let tool_changed = manifest_a.tool != manifest_b.tool;
    let command_changed = manifest_a.command != manifest_b.command;

    let diffs = numeric_diff(&metrics_a, &metrics_b, 0.05);
    Ok(CompareReport {
        run_a: manifest_a.run_id,
        run_b: manifest_b.run_id,
        tool_changed,
        command_changed,
        diffs,
        strata: Vec::new(),
    })
}

fn numeric_diff(
    a: &BTreeMap<String, f64>,
    b: &BTreeMap<String, f64>,
    practical_threshold: f64,
) -> Vec<MetricDiff> {
    let mut diffs = Vec::new();
    for (metric_id, a_val) in a {
        if let Some(b_val) = b.get(metric_id) {
            let absolute = b_val - a_val;
            let relative = if a_val.abs() > f64::EPSILON {
                Some(absolute / a_val)
            } else {
                None
            };
            let practical = absolute.abs() >= practical_threshold;
            diffs.push(MetricDiff {
                metric_id: metric_id.clone(),
                absolute,
                relative,
                practical,
            });
        }
    }
    diffs.sort_by(|a, b| a.metric_id.cmp(&b.metric_id));
    diffs
}

fn ensure_repo_exists(index_path: &Path) -> Result<()> {
    if !index_path.exists() {
        return Err(anyhow!(
            "run_index.jsonl not found at {}",
            index_path.display()
        ));
    }
    Ok(())
}
