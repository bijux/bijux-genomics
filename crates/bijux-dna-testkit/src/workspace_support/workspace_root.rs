use std::path::{Path, PathBuf};

/// Resolve the workspace root from a crate manifest directory.
///
/// # Panics
/// Panics if `manifest_dir` does not have the expected `crates/<name>` layout.
#[must_use]
pub fn workspace_root_from_manifest(manifest_dir: &str) -> PathBuf {
    Path::new(manifest_dir)
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| panic!("resolve workspace root from {manifest_dir}"))
        .to_path_buf()
}
