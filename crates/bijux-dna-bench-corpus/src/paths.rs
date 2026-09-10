//! Owner: bijux-dna-bench-corpus
//! Corpus declarations preserve manifest-relative paths and repository-relative report display.
use std::path::{Path, PathBuf};
pub(crate) fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}
pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

/// Check the explicitly declared corpus source files without changing their path convention.
pub(crate) fn validate_source_paths(
    repo_root: &Path,
    paths: &[PathBuf],
    label: &str,
) -> anyhow::Result<Vec<String>> {
    paths
        .iter()
        .map(|path| {
            let absolute = if path.is_absolute() { path.clone() } else { repo_root.join(path) };
            if !absolute.is_file() {
                anyhow::bail!("{label} source path is missing: {}", absolute.display());
            }
            Ok(path_relative_to_repo(repo_root, &absolute))
        })
        .collect()
}
