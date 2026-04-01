use std::path::{Path, PathBuf};

use crate::run_directories::{RunLayoutPaths, RUN_LAYOUT_CONTRACT};

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
