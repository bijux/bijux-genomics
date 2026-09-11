use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
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
pub(in crate::readiness) fn required_row_value(
    row: &BTreeMap<String, String>,
    key: &str,
    path: &Path,
) -> Result<String> {
    let value = row
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("{} is missing column `{key}`", path.display()))?;
    if value.is_empty() {
        return Err(anyhow!("{} has empty value for `{key}`", path.display()));
    }
    Ok(value)
}
