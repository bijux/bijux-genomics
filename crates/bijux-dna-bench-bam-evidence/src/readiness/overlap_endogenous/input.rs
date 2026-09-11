use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(in crate::readiness) fn ensure_repo_relative_file(
    repo_root: &Path,
    relative: &str,
) -> Result<String> {
    let path = repo_root.join(relative);
    if !path.is_file() {
        return Err(anyhow!("governed local-smoke artifact is missing: {}", path.display()));
    }
    Ok(relative.to_string())
}
