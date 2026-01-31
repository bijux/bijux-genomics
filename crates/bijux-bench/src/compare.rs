use std::path::Path;

use anyhow::{anyhow, Result};

use crate::repo::{load_manifest, load_metrics, RunIndexRepository, RunRepository};

pub fn compare_runs(
    run_a: &str,
    run_b: &str,
    index_path: &Path,
    artifacts_root: &Path,
) -> Result<serde_json::Value> {
    ensure_repo_exists(index_path)?;
    let repo = RunIndexRepository::new(index_path.to_path_buf(), artifacts_root.to_path_buf());
    compare_runs_with_repo(run_a, run_b, &repo)
}

pub fn compare_runs_with_repo(
    run_a: &str,
    run_b: &str,
    repo: &dyn RunRepository,
) -> Result<serde_json::Value> {
    let meta_a = repo.run_metadata(run_a)?;
    let meta_b = repo.run_metadata(run_b)?;
    let manifest_a = load_manifest(&meta_a.manifest_path)?;
    let manifest_b = load_manifest(&meta_b.manifest_path)?;
    let metrics_a = load_metrics(&meta_a.metrics_path)?;
    let metrics_b = load_metrics(&meta_b.metrics_path)?;

    let tool_changed = manifest_a.tool != manifest_b.tool;
    let command_changed = manifest_a.command != manifest_b.command;

    let delta = numeric_delta(&metrics_a, &metrics_b);
    Ok(serde_json::json!({
        "run_a": manifest_a.run_id,
        "run_b": manifest_b.run_id,
        "tool_changed": tool_changed,
        "command_changed": command_changed,
        "metrics_delta": delta,
    }))
}

fn numeric_delta(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value {
    let mut delta = serde_json::Map::new();
    let map_a = a.as_object().cloned().unwrap_or_default();
    let map_b = b.as_object().cloned().unwrap_or_default();
    for (key, val_a) in &map_a {
        if let Some(val_b) = map_b.get(key) {
            if let (Some(a_num), Some(b_num)) = (val_a.as_f64(), val_b.as_f64()) {
                delta.insert(key.clone(), serde_json::json!(b_num - a_num));
            }
        }
    }
    serde_json::Value::Object(delta)
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
