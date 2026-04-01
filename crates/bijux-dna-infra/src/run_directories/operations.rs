use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::paths::run_layout_paths;
use crate::{atomic_write_json, ensure_dir, FileLock, IoError};

use super::{RunLayoutPaths, RUN_LAYOUT_CONTRACT};

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
