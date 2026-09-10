//! Bounded ownership observations and symlink-safe artifact paths.
use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
pub(super) fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    admit_ancestry(path)?;
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
        bail!("haplotype publication ownership must be a bounded regular file");
    }
    fs::read(path).with_context(|| format!("read {}", path.display()))
}

pub(super) fn collect_files(root: &Path) -> Result<BTreeSet<PathBuf>> {
    let mut files = BTreeSet::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                files.insert(entry.path().strip_prefix(root)?.to_path_buf());
            } else {
                bail!("haplotype staging contains a nonregular artifact");
            }
        }
    }
    Ok(files)
}

pub(super) fn admit_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("haplotype artifact inventory contains an escaping path");
    }
    Ok(())
}

pub(super) fn admit_ancestry(path: &Path) -> Result<()> {
    if path.components().any(|part| matches!(part, Component::ParentDir)) {
        bail!("haplotype publication path contains parent traversal");
    }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!("haplotype publication cannot follow symlinks")
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
