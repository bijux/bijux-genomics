//! Owner: pure campaign scope paths and portable study-relative path admission.
use anyhow::{bail, Context, Result};
use std::path::{Component, Path, PathBuf};
pub(super) fn campaign_root(base: &Path, commit: &str, scope: Option<&str>) -> PathBuf {
    let root = base.join(commit);
    scope.map_or(root.clone(), |scope| root.join(scope))
}

pub(super) fn require_campaign_path(
    path: &Path,
    root: &Path,
    label: &str,
    tool: &str,
) -> Result<()> {
    if path == root || !path.starts_with(root) {
        bail!("resumed {label} path is outside the campaign scope for {tool}");
    }
    Ok(())
}

pub(super) fn portable_study_path(path: &Path, study_root: &Path) -> Result<String> {
    let relative = path.strip_prefix(study_root).context("campaign path is outside study root")?;
    Ok(format!("study://{}", relative.to_string_lossy().replace('\\', "/")))
}

pub(super) fn resolve_study_path(study_root: &Path, value: &str) -> Result<PathBuf> {
    let relative = value.strip_prefix("study://").context("campaign path is not study-relative")?;
    let relative = Path::new(relative);
    if relative.components().any(|part| !matches!(part, Component::Normal(_))) {
        bail!("campaign path contains a non-normal component");
    }
    Ok(study_root.join(relative))
}
