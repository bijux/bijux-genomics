use std::path::{Path, PathBuf};

use super::config_aliases::{remap_ci_registry, remap_runtime_profile};
use super::segments::relative_path;

#[must_use]
pub fn configs_dir(root: &Path) -> PathBuf {
    root.join("configs")
}

#[must_use]
pub fn configs_file(root: &Path, relative: &str) -> PathBuf {
    let normalized = remap_runtime_profile(relative)
        .or_else(|| remap_ci_registry(relative).map(str::to_string))
        .unwrap_or_else(|| relative.to_string());
    configs_dir(root).join(relative_path(&normalized))
}
