use std::path::{Path, PathBuf};

#[must_use]
pub fn workspace_root_from_manifest(manifest_dir: &str) -> PathBuf {
    Path::new(manifest_dir)
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| panic!("resolve workspace root from {manifest_dir}"))
        .to_path_buf()
}
