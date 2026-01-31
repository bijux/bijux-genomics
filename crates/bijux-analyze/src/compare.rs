use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use bijux_core::metrics_registry::metric_semantics;
use bijux_core::FactsRowV1;
use bijux_core::ObjectiveSpec;

use crate::aggregate::stats::{robust_summary, RobustSummary};

#[derive(Debug, Serialize)]
pub struct RunComparison {
    pub run_a: String,
    pub run_b: String,
    pub objective: String,
    pub metrics_a: serde_json::Value,
    pub metrics_b: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct CompareRobustStats {
    pub runtime_s: RobustSummary,
    pub memory_mb: RobustSummary,
    pub read_retention: RobustSummary,
    pub flags: Vec<String>,
}

fn load_json(path: &Path) -> Result<serde_json::Value> {
    let data = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&data)?;
    Ok(value)
}

/// Compare two runs using their summary metrics.
///
/// # Errors
/// Returns an error if metrics cannot be loaded.
pub fn compare_runs(
    run_a: &Path,
    run_b: &Path,
    objective: &ObjectiveSpec,
) -> Result<RunComparison> {
    let metrics_a = load_json(&run_a.join("summary").join("metrics_deltas.json"))?;
    let metrics_b = load_json(&run_b.join("summary").join("metrics_deltas.json"))?;
    Ok(RunComparison {
        run_a: run_a.display().to_string(),
        run_b: run_b.display().to_string(),
        objective: objective.name.clone(),
        metrics_a,
        metrics_b,
    })
}

/// Compute robust summary stats for runtime/memory/retention.
#[must_use]
pub fn compare_robust_stats(rows: &[FactsRowV1]) -> CompareRobustStats {
    assert_metric_semantics(&["runtime_s", "memory_mb", "read_retention"]);
    let runtime_values: Vec<f64> = rows.iter().map(|row| row.runtime_s).collect();
    let memory_values: Vec<f64> = rows.iter().map(|row| row.memory_mb).collect();
    let retention_values: Vec<f64> = rows
        .iter()
        .filter_map(|row| match (row.reads_in, row.reads_out) {
            #[allow(clippy::cast_precision_loss)]
            (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
            _ => None,
        })
        .collect();
    let runtime = robust_summary(&runtime_values);
    let memory = robust_summary(&memory_values);
    let retention = robust_summary(&retention_values);
    let mut flags = Vec::new();
    if runtime.n < 3 || memory.n < 3 || retention.n < 3 {
        flags.push("sample_size_too_small".to_string());
    }
    if runtime.high_variance || memory.high_variance || retention.high_variance {
        flags.push("high_variance".to_string());
    }
    if runtime.outlier_count > 0 || memory.outlier_count > 0 || retention.outlier_count > 0 {
        flags.push("outliers_detected".to_string());
    }
    CompareRobustStats {
        runtime_s: runtime,
        memory_mb: memory,
        read_retention: retention,
        flags,
    }
}

fn assert_metric_semantics(metric_ids: &[&str]) {
    for metric_id in metric_ids {
        assert!(
            metric_semantics(metric_id).is_some(),
            "missing metric semantics for {metric_id}"
        );
    }
}
