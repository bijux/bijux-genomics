//! Bounded removal of exclusively owned run artifacts, independent of retained products.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const CLEANUP_WORKERS: usize = 4;

/// Remove disjoint owned trees and collect every failure before returning.
/// The caller owns these roots exclusively for the duration of removal.
pub(crate) fn remove_owned_trees(paths: &[PathBuf]) -> Result<()> {
    for (index, path) in paths.iter().enumerate() {
        for other in &paths[..index] {
            anyhow::ensure!(
                !path.starts_with(other) && !other.starts_with(path),
                "owned artifact cleanup roots overlap: {} and {}",
                path.display(),
                other.display()
            );
        }
    }
    let next = AtomicUsize::new(0);
    let mut failures = std::thread::scope(|scope| {
        let workers = (0..CLEANUP_WORKERS.min(paths.len()))
            .map(|_| {
                scope.spawn(|| {
                    let mut failures = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(path) = paths.get(index) else { break };
                        if let Err(error) = remove_owned_tree(path) {
                            failures.push((index, format!("{error:#}")));
                        }
                    }
                    failures
                })
            })
            .collect::<Vec<_>>();
        let mut failures = Vec::new();
        for worker in workers {
            match worker.join() {
                Ok(errors) => failures.extend(errors),
                Err(_) => failures
                    .push((usize::MAX, "owned artifact cleanup worker panicked".to_string())),
            }
        }
        failures
    });
    failures.sort_by_key(|(index, _)| *index);
    anyhow::ensure!(
        failures.is_empty(),
        "owned artifact cleanup failed: {}",
        failures.into_iter().map(|(_, error)| error).collect::<Vec<_>>().join("; ")
    );
    Ok(())
}

/// Enumerate one owned directory without following a root symlink.
pub(crate) fn owned_children(path: &Path) -> Result<Vec<PathBuf>> {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect owned directory {}", path.display()))
        }
    };
    anyhow::ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "owned directory is not an ordinary directory: {}",
        path.display()
    );
    let mut children = std::fs::read_dir(path)
        .with_context(|| format!("list owned directory {}", path.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    children.sort();
    Ok(children)
}

/// Remove only the empty parent after all its independently owned children finished.
pub(crate) fn remove_empty_owned_directory(path: &Path) -> Result<()> {
    match std::fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("remove empty owned directory {}", path.display()))
        }
    }
}

pub(crate) fn remove_owned_tree(path: &Path) -> Result<()> {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect full shard ephemeral path {}", path.display()))
        }
    };
    if metadata.file_type().is_symlink() {
        anyhow::bail!("full shard ephemeral path is a symlink: {}", path.display());
    }
    make_owned_tree_removable(path)?;
    if metadata.is_dir() {
        std::fs::remove_dir_all(path)
            .with_context(|| format!("remove full shard ephemeral directory {}", path.display()))
    } else {
        std::fs::remove_file(path)
            .with_context(|| format!("remove full shard ephemeral file {}", path.display()))
    }
}

fn make_owned_tree_removable(path: &Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(path).follow_links(false) {
        let entry = entry.with_context(|| {
            format!("inspect full shard ephemeral path below {}", path.display())
        })?;
        let metadata = entry.path().symlink_metadata().with_context(|| {
            format!("read full shard ephemeral metadata {}", entry.path().display())
        })?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        let mut permissions = metadata.permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let owner_access = if metadata.is_dir() { 0o700 } else { 0o600 };
            if permissions.mode() & owner_access == owner_access {
                continue;
            }
            permissions.set_mode(permissions.mode() | owner_access);
        }
        #[cfg(not(unix))]
        {
            if !permissions.readonly() {
                continue;
            }
            permissions.set_readonly(false);
        }
        std::fs::set_permissions(entry.path(), permissions).with_context(|| {
            format!("restore owner access to full shard ephemera {}", entry.path().display())
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_owned_roots_reject_before_removing_anything() -> Result<()> {
        let root = tempfile::tempdir()?;
        let parent = root.path().join("owner");
        let child = parent.join("child");
        std::fs::create_dir_all(&child)?;
        std::fs::write(child.join("evidence"), b"preserved")?;
        assert!(remove_owned_trees(&[parent, child.clone()]).is_err());
        assert_eq!(std::fs::read(child.join("evidence"))?, b"preserved");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn owned_cleanup_collects_failures_and_finishes_independent_trees() -> Result<()> {
        let root = tempfile::tempdir()?;
        let external = root.path().join("retained");
        std::fs::create_dir_all(&external)?;
        std::fs::write(external.join("evidence"), b"preserved")?;
        let first = root.path().join("first-link");
        let last = root.path().join("last-link");
        std::os::unix::fs::symlink(&external, &first)?;
        std::os::unix::fs::symlink(&external, &last)?;
        let good = root.path().join("owned");
        std::fs::create_dir_all(&good)?;
        std::fs::write(good.join("output"), b"disposable")?;
        let error = remove_owned_trees(&[first, good.clone(), last])
            .err()
            .context("root links must reject")?
            .to_string();
        assert!(error.contains("first-link") && error.contains("last-link"));
        assert!(!good.exists());
        assert_eq!(std::fs::read(external.join("evidence"))?, b"preserved");
        Ok(())
    }
}
