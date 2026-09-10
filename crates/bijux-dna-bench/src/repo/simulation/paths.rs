//! Owner: observed file-kind and link admission for simulation destinations.
use anyhow::{ensure, Context, Result};
use std::{
    fs,
    io::ErrorKind,
    path::{Component, Path},
};
pub(super) fn admit_entry(repo: &Path, path: &Path, directory: bool) -> Result<()> {
    let relative = path.strip_prefix(repo).context("simulation path escapes repository")?;
    ensure!(relative.file_name().is_some(), "simulation path requires a destination");
    let mut current = repo.to_path_buf();
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        ensure!(matches!(component, Component::Normal(_)), "simulation path must be normalized");
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink(),
                    "simulation path contains a symlink: {}",
                    current.display()
                );
                let wants_directory = components.peek().is_some() || directory;
                ensure!(
                    if wants_directory { metadata.is_dir() } else { metadata.is_file() },
                    "simulation path has wrong file kind: {}",
                    current.display()
                );
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", current.display()))
            }
        }
    }
    Ok(())
}
