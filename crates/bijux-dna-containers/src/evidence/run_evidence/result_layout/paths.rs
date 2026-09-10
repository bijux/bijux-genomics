//! Filesystem admission below explicitly supplied result/evidence roots.
use super::admission::safe_relative;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn regular_metadata(path: &Path) -> Result<fs::Metadata> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect result path {}", path.display()))?;
    if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
        bail!("result evidence requires regular files and directories: {}", path.display());
    }
    Ok(metadata)
}

pub(super) fn require_directory(path: &Path) -> Result<()> {
    if !regular_metadata(path)?.is_dir() {
        bail!("result directory required: {}", path.display());
    }
    Ok(())
}

/// Inspect existing components beneath the supplied root without following links.
/// Missing tails are allowed for no-clobber publication and rollback destinations.
/// This observation is not a descriptor-relative transaction against concurrent replacement.
pub(super) fn owned_path(root: &Path, relative: &str, label: &str) -> Result<PathBuf> {
    let relative = safe_relative(relative, label)?;
    let mut current = root.to_path_buf();
    let destination = root.join(&relative);
    for component in std::iter::once(None).chain(relative.components().map(Some)) {
        if let Some(component) = component {
            current.push(component.as_os_str());
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir())
                {
                    bail!(
                        "result path contains a link or unsupported entry: {}",
                        current.display()
                    );
                }
                if current == root && !metadata.is_dir() {
                    bail!("result root is not a directory: {}", root.display());
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("inspect result path {}", current.display()))
            }
        }
    }
    Ok(destination)
}

/// Admit entry kinds before a saved plan records an applying transition.
/// Content digests remain checked by the existing per-movement inventory verifier.
pub(super) fn validate_regular_tree(root: &Path) -> Result<()> {
    require_directory(root)?;
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if regular_metadata(&path)?.is_dir() {
            validate_regular_tree(&path)?;
        }
    }
    Ok(())
}
