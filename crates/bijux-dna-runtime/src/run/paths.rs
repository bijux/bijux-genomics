use std::path::{Path, PathBuf};

#[must_use]
pub fn resolve_run_base_dir(cwd: &Path, run_base: &Path) -> PathBuf {
    bijux_dna_infra::normalize_run_base_dir(cwd, run_base)
}
