use std::path::Path;

use anyhow::{Context, Result};

use super::RunLayout;
use crate::run::new_run_id;

/// Create the canonical run layout under the base directory.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn create_run_layout(base_dir: &Path) -> Result<(String, RunLayout)> {
    let run_id = new_run_id().0;
    let run_dir = bijux_dna_infra::run_layout_paths(base_dir, &run_id).run_dir;
    let layout = RunLayout::from_run_dir(run_dir);
    for (label, dir) in [
        ("run dir", &layout.run_dir),
        ("run stages dir", &layout.stages_dir),
        ("run manifests dir", &layout.manifests_dir),
        ("run logs dir", &layout.logs_dir),
        ("run reports dir", &layout.reports_dir),
        ("run summary dir", &layout.summary_dir),
        ("run artifacts dir", &layout.run_artifacts_dir),
        ("run checkpoints dir", &layout.checkpoints_dir),
    ] {
        bijux_dna_infra::ensure_dir(dir).with_context(|| format!("create {label}"))?;
    }
    Ok((run_id, layout))
}
