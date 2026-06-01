//! BAM local-ready and local-smoke domain helpers for v1.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Materialize the governed local-ready `bam.align` dry-run plan.
///
/// The written artifact lives at `target/local-ready/bam.align/plan.json` under the active
/// repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed planner config is
/// invalid, or the plan artifact cannot be written.
pub fn write_local_align_plan() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plan = bijux_dna_planner_bam::stage_api::local_align_plan(&repo_root)?;
    let plan_dir = resolve_plan_dir(&repo_root, &plan.out_dir);
    bijux_dna_infra::ensure_dir(&plan_dir)?;
    let plan_path = plan_dir.join("plan.json");
    bijux_dna_infra::atomic_write_json(&plan_path, &plan)?;
    Ok(plan_path)
}

fn resolve_plan_dir(repo_root: &Path, out_dir: &Path) -> PathBuf {
    if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        repo_root.join(out_dir)
    }
}
