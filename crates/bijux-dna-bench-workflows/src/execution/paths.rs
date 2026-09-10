//! Owner: bijux-dna-bench-workflows
//! Preserve repository-relative report paths and explicit external output paths.
use std::path::Path;
pub(crate) fn path_relative_to_repo(repository: &Path, path: &Path) -> String {
    path.strip_prefix(repository).unwrap_or(path).display().to_string()
}
