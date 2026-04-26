use std::path::{Component, Path};

use anyhow::{anyhow, Context, Result};

use super::RunDirs;

/// # Errors
/// Returns an error if run directories cannot be created.
pub fn prepare_tool_run_dirs(tools_root: &Path, tool: &str, run_id: &str) -> Result<RunDirs> {
    validate_path_segment("tool", tool)?;
    validate_path_segment("run_id", run_id)?;
    let tool_dir = tools_root.join(tool);
    let run_dir = tool_dir.join("run").join(run_id);
    let artifacts_dir = run_dir.join("artifacts");
    let logs_dir = run_dir.join("logs");
    bijux_dna_infra::ensure_dir(&artifacts_dir).context("create artifacts dir")?;
    bijux_dna_infra::ensure_dir(&logs_dir).context("create logs dir")?;
    Ok(RunDirs {
        artifacts_dir,
        logs_dir: logs_dir.clone(),
        manifest_path: run_dir.join("manifest.json"),
        metrics_path: run_dir.join("metrics.json"),
        run_manifest_path: run_dir.join("run_manifest.json"),
    })
}

fn validate_path_segment(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} path segment must not be empty"));
    }
    let path = Path::new(value);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(anyhow!("{label} path segment must not contain separators or traversal"));
    }
    Ok(())
}
