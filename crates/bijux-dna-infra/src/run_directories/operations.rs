use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{atomic_write_json, ensure_dir, FileLock, IoError};

use super::{RunLayoutPaths, RUN_LAYOUT_CONTRACT};

#[must_use]
pub fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> PathBuf {
    if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    }
}

#[must_use]
pub fn pipeline_run_dir(
    base_dir: &Path,
    pipeline_id: &str,
    sample_id: &str,
    run_id: &str,
) -> PathBuf {
    base_dir.join(pipeline_id).join(sample_id).join(run_id)
}

#[must_use]
pub fn run_layout_paths(base_dir: &Path, run_id: &str) -> RunLayoutPaths {
    let run_dir = base_dir.join(RUN_LAYOUT_CONTRACT.runs_dir).join(run_id);
    RunLayoutPaths {
        artifacts_dir: run_dir.join(RUN_LAYOUT_CONTRACT.artifacts_dir),
        logs_dir: run_dir.join(RUN_LAYOUT_CONTRACT.logs_dir),
        tmp_dir: run_dir.join(RUN_LAYOUT_CONTRACT.tmp_dir),
        run_dir,
    }
}

#[must_use]
pub fn run_stage_dir(base_dir: &Path, run_id: &str, stage: &str, tool: &str) -> PathBuf {
    run_layout_paths(base_dir, run_id)
        .run_dir
        .join(stage)
        .join(tool)
}

/// Acquire the run-level lock for coordinated publish/write operations.
///
/// # Errors
/// Returns an IO error if the lock cannot be acquired within the timeout.
pub fn lock_run(layout: &RunLayoutPaths, timeout: Duration) -> Result<FileLock, IoError> {
    ensure_dir(&layout.run_dir)?;
    FileLock::acquire(&layout.run_dir.join(RUN_LAYOUT_CONTRACT.lock_file), timeout)
}

/// Publish a run by writing an atomic marker into the artifacts directory.
///
/// # Errors
/// Returns an IO error if the marker cannot be written.
pub fn publish_run(layout: &RunLayoutPaths, run_id: &str) -> Result<PathBuf, IoError> {
    ensure_dir(&layout.artifacts_dir)?;
    let marker = layout
        .artifacts_dir
        .join(RUN_LAYOUT_CONTRACT.publish_marker);
    let payload = serde_json::json!({
        "schema_version": "bijux.run_publish.v1",
        "run_id": run_id,
    });
    atomic_write_json(&marker, &payload)?;
    Ok(marker)
}
