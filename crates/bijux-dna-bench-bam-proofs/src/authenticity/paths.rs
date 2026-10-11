use std::path::Path;
use std::path::PathBuf;
pub(super) fn normalize_path_string(repo_root: &Path, path: &str) -> String {
    let candidate = PathBuf::from(path);
    let absolute = if candidate.is_absolute() { candidate } else { repo_root.join(candidate) };
    path_relative_to_repo(repo_root, &absolute)
}

pub(super) fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

pub(super) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
