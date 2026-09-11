use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;
pub(super) fn ensure_repo_relative_file(repo_root: &Path, relative: &str) -> Result<()> {
    let path = repo_root.join(relative);
    if !path.is_file() {
        return Err(anyhow!("governed kinship artifact is missing: {}", path.display()));
    }
    Ok(())
}

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
