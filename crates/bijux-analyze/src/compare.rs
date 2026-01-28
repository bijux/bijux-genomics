use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::selection::ObjectiveSpec;

#[derive(Debug, Serialize)]
pub struct RunComparison {
    pub run_a: String,
    pub run_b: String,
    pub objective: String,
    pub metrics_a: serde_json::Value,
    pub metrics_b: serde_json::Value,
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
