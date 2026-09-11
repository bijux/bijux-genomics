//! Reject path-bearing identities before campaign-owned mutations.
use anyhow::{anyhow, Context, Result};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

pub(crate) fn path_component(value: &str, label: &str) -> Result<()> {
    let mut components = Path::new(value).components();
    if value.is_empty()
        || value.contains(['/', '\\', '\0'])
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(anyhow!("{label} must be one ordinary path component"));
    }
    Ok(())
}

pub(crate) fn resolve_output_path(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute() || path.components().any(|part| matches!(part, Component::ParentDir)) {
        return Err(anyhow!("campaign output root must be absolute without parent traversal"));
    }
    let mut existing = path;
    let mut suffix = Vec::new();
    loop {
        match fs::symlink_metadata(existing) {
            Ok(metadata) => {
                if existing == path && metadata.file_type().is_symlink() {
                    return Err(anyhow!("campaign output root must not be a symlink"));
                }
                let mut resolved = existing
                    .canonicalize()
                    .with_context(|| format!("resolve {}", existing.display()))?;
                for name in suffix.into_iter().rev() {
                    resolved.push(name);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                suffix.push(
                    existing
                        .file_name()
                        .ok_or_else(|| anyhow!("output root has no existing ancestor"))?
                        .to_os_string(),
                );
                existing = existing
                    .parent()
                    .ok_or_else(|| anyhow!("output root has no existing ancestor"))?;
            }
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", existing.display()))
            }
        }
    }
}

pub(crate) fn owned_sample_root(output_root: &Path, sample_root: &Path) -> Result<()> {
    let relative =
        sample_root.strip_prefix(output_root).context("sample output escaped campaign root")?;
    let mut path = output_root.to_path_buf();
    for part in relative.components() {
        if !matches!(part, Component::Normal(_)) {
            return Err(anyhow!("sample output contains traversal"));
        }
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(anyhow!(
                    "campaign sample payload must not follow symlinks: {}",
                    path.display()
                ))
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).with_context(|| format!("inspect {}", path.display())),
        }
    }
    Ok(())
}
