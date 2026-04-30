use super::Result;
use anyhow::{anyhow, Context};
use std::path::Path;

/// Load run-level and stage-level environment identity evidence.
///
/// # Errors
/// Returns an error if required environment contracts are missing or invalid.
pub fn environment_identity(run_dir: &Path) -> Result<serde_json::Value> {
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    if !layout.environment_path.exists() {
        return Err(anyhow!(
            "missing environment contract at {}",
            layout.environment_path.display()
        ));
    }
    let run_environment: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&layout.environment_path)?)
            .context("parse run environment contract")?;
    let stage_environments = collect_stage_environments(run_dir)?;
    Ok(serde_json::json!({
        "schema_version": "bijux.run_environment_identity.v1",
        "run_environment": run_environment,
        "stage_environments": stage_environments,
    }))
}

fn collect_stage_environments(run_dir: &Path) -> Result<Vec<serde_json::Value>> {
    let mut environments = Vec::new();
    for root in [
        run_dir.to_path_buf(),
        run_dir.parent().map_or_else(|| run_dir.to_path_buf(), Path::to_path_buf),
    ] {
        if !root.exists() {
            continue;
        }
        collect_invocations_under_root(&root, &mut environments)?;
    }
    environments.sort_by(|left, right| {
        left.get("stage_dir")
            .and_then(serde_json::Value::as_str)
            .cmp(&right.get("stage_dir").and_then(serde_json::Value::as_str))
    });
    environments.dedup();
    Ok(environments)
}

fn collect_invocations_under_root(root: &Path, sink: &mut Vec<serde_json::Value>) -> Result<()> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        if !path.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path.file_name().and_then(|name| name.to_str()) != Some("tool_invocation.json") {
                continue;
            }
            let raw = std::fs::read_to_string(&entry_path)
                .with_context(|| format!("read {}", entry_path.display()))?;
            let value: serde_json::Value = serde_json::from_str(&raw)
                .with_context(|| format!("parse {}", entry_path.display()))?;
            let stage_dir = entry_path
                .parent()
                .and_then(Path::parent)
                .unwrap_or(root)
                .to_path_buf();
            sink.push(serde_json::json!({
                "stage_dir": stage_dir.display().to_string(),
                "stage_id": value.get("stage_id").cloned().unwrap_or(serde_json::Value::Null),
                "runner_kind": value.get("runner_kind").cloned().unwrap_or(serde_json::Value::Null),
                "environment": value.get("environment").cloned().unwrap_or(serde_json::json!({})),
            }));
        }
    }
    Ok(())
}
