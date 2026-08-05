//! Owner: bijux-dna-bench
//! Immutable storage for exact FASTQ expectation publications.

use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};

const MAX_EXPECTATION_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;

pub(super) fn ensure_directory_chain(study_root: &Path, destination: &Path) -> Result<()> {
    let relative = destination
        .strip_prefix(study_root)
        .with_context(|| format!("expectation root escapes STUDY: {}", destination.display()))?;
    let mut current = study_root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            bail!("expectation root is not normalized: {}", destination.display());
        };
        current.push(component);
        match std::fs::create_dir(&current) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                let metadata = std::fs::symlink_metadata(&current).with_context(|| {
                    format!("inspect expectation directory {}", current.display())
                })?;
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    bail!("expectation path is not a non-symlink directory: {}", current.display());
                }
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("create expectation directory {}", current.display())
                });
            }
        }
    }
    Ok(())
}

pub(super) fn ensure_immutable_file(path: &Path, bytes: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => verify_existing_file(path, bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            match bijux_dna_infra::atomic_create_bytes(path, bytes) {
                Ok(()) => Ok(()),
                Err(create_error) => match std::fs::symlink_metadata(path) {
                    Ok(_) => verify_existing_file(path, bytes),
                    Err(_) => Err(create_error).with_context(|| {
                        format!("publish expectation artifact {}", path.display())
                    }),
                },
            }
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspect expectation artifact {}", path.display()))
        }
    }
}

pub(super) fn verify_directory_closure(
    path: &Path,
    expected_names: &BTreeSet<String>,
) -> Result<()> {
    let mut observed = BTreeSet::new();
    for entry in std::fs::read_dir(path)
        .with_context(|| format!("read expectation directory {}", path.display()))?
    {
        let entry = entry.with_context(|| format!("read entry under {}", path.display()))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("expectation artifact name is not UTF-8"))?;
        if !observed.insert(name.clone()) {
            bail!("expectation directory duplicates artifact {name}");
        }
    }
    if &observed != expected_names {
        bail!("expectation directory closure differs at {}", path.display());
    }
    Ok(())
}

fn verify_existing_file(path: &Path, expected: &[u8]) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect expectation artifact {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("expectation artifact is not a regular non-symlink file: {}", path.display());
    }
    let observed = bijux_dna_infra::read_to_end_bounded(path, MAX_EXPECTATION_ARTIFACT_BYTES)
        .with_context(|| format!("read expectation artifact {}", path.display()))?;
    if observed != expected {
        bail!("expectation artifact conflicts with immutable evidence: {}", path.display());
    }
    Ok(())
}
