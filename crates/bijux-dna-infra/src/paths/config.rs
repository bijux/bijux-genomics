use std::path::{Path, PathBuf};

#[must_use]
pub fn configs_dir(root: &Path) -> PathBuf {
    root.join("configs")
}

#[must_use]
pub fn configs_file(root: &Path, relative: &str) -> PathBuf {
    configs_dir(root).join(relative)
}
