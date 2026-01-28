use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::types::ExecutionManifest;

pub fn compare_runs(run_a: &str, run_b: &str, search_root: &Path) -> Result<serde_json::Value> {
    if crate::types::trace_enabled() {
        println!("[engine][composer] compare {run_a} vs {run_b}");
    }
    let manifest_a = load_manifest(run_a, search_root)?;
    let manifest_b = load_manifest(run_b, search_root)?;
    let metrics_a = load_metrics(&manifest_a)?;
    let metrics_b = load_metrics(&manifest_b)?;

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

fn load_manifest(run_id: &str, search_root: &Path) -> Result<ExecutionManifest> {
    let manifest_path = find_manifest(search_root, run_id)?
        .ok_or_else(|| anyhow!("run_id {run_id} not found under {}", search_root.display()))?;
    let bytes = std::fs::read(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    let manifest: ExecutionManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse manifest {}", manifest_path.display()))?;
    Ok(manifest)
}

fn load_metrics(manifest: &ExecutionManifest) -> Result<serde_json::Value> {
    let metrics_path = Path::new(&manifest.output_dir).join("metrics.json");
    if !metrics_path.exists() {
        return Ok(serde_json::json!({}));
    }
    let bytes = std::fs::read(&metrics_path)
        .with_context(|| format!("read metrics {}", metrics_path.display()))?;
    Ok(serde_json::from_slice(&bytes)?)
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

fn find_manifest(root: &Path, run_id: &str) -> Result<Option<PathBuf>> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|s| s.to_str()) == Some("manifest.json") {
                let bytes = std::fs::read(&path)
                    .with_context(|| format!("read manifest {}", path.display()))?;
                if let Ok(manifest) = serde_json::from_slice::<ExecutionManifest>(&bytes) {
                    if manifest.run_id == run_id {
                        return Ok(Some(path));
                    }
                }
            }
        }
    }
    Ok(None)
}
