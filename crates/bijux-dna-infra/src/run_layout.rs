use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{ensure_dir, atomic_write_json, FileLock, IoError};

#[must_use]
pub fn normalize_run_base_dir(cwd: &Path, run_base: &Path) -> PathBuf {
    if run_base.is_absolute() {
        run_base.to_path_buf()
    } else {
        cwd.join(run_base)
    }
}

#[derive(Debug, Clone)]
pub struct RunLayoutPaths {
    pub run_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct RunLayoutContract {
    pub runs_dir: &'static str,
    pub artifacts_dir: &'static str,
    pub logs_dir: &'static str,
    pub tmp_dir: &'static str,
    pub lock_file: &'static str,
    pub publish_marker: &'static str,
}

pub const RUN_LAYOUT_CONTRACT: RunLayoutContract = RunLayoutContract {
    runs_dir: "runs",
    artifacts_dir: "artifacts",
    logs_dir: "logs",
    tmp_dir: "tmp",
    lock_file: ".run.lock",
    publish_marker: "published.json",
};

pub const PIPELINE_RUN_DIR_TEMPLATE: &str = "{pipeline_id}/{sample_id}/{run_id}";

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
