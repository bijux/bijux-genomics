use std::path::Path;
use std::path::PathBuf;
pub(super) fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    repo_relative_path(repo_root, path).display().to_string()
}
