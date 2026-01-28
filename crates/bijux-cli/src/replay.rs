use std::path::Path;

use anyhow::Result;

pub fn replay_run(run_id: &str, search_root: &Path) -> Result<()> {
    bijux_engine::api::replay::replay_run(run_id, search_root)
}
