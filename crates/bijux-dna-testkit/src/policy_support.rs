use std::path::{Path, PathBuf};

#[must_use]
pub fn workspace_root_from_manifest(manifest_dir: &str) -> PathBuf {
    Path::new(manifest_dir)
        .parent()
        .and_then(|path| path.parent())
        .unwrap_or_else(|| panic!("resolve workspace root from {manifest_dir}"))
        .to_path_buf()
}

#[must_use]
pub fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}
