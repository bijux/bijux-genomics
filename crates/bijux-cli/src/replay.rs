use std::path::Path;

use anyhow::Result;

/// # Errors
/// Returns an error if the run cannot be found or replay fails.
pub fn replay_run(run_id: &str, search_root: &Path) -> Result<()> {
    bijux_engine::api::replay::replay_run(run_id, search_root)
}
