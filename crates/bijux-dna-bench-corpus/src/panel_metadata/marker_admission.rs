//! Preflight all marker destinations, rejecting foreign bytes without side effects.
use super::markers::Marker;
use anyhow::{bail, ensure, Result};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Component, Path};

pub(super) fn admit_component(value: &str) -> Result<()> {
    let path = Path::new(value);
    admit_relative(path)?;
    ensure!(path.components().count() == 1, "panel identity must be one path component");
    Ok(())
}

pub(super) fn admit_relative(path: &Path) -> Result<()> {
    ensure!(
        !path.as_os_str().is_empty()
            && path.components().all(|component| matches!(component, Component::Normal(_))),
        "panel metadata contains an unsafe relative path"
    );
    Ok(())
}

pub(super) fn admit_markers(markers: &[Marker]) -> Result<Vec<&Marker>> {
    let mut destinations = BTreeSet::new();
    let mut absent = Vec::new();
    for marker in markers {
        ensure!(destinations.insert(&marker.path), "duplicate panel marker destination");
        admit_ancestry(&marker.path)?;
        match fs::symlink_metadata(&marker.path) {
            Ok(metadata) => {
                ensure!(metadata.is_file(), "panel marker destination is not a regular file");
                let expected = marker.payload.as_bytes();
                ensure!(
                    metadata.len() == expected.len() as u64,
                    "refuse to replace existing panel asset"
                );
                let mut observed = Vec::new();
                fs::File::open(&marker.path)?
                    .take(expected.len() as u64 + 1)
                    .read_to_end(&mut observed)?;
                ensure!(
                    observed == expected,
                    "refuse to replace unknown panel marker or reference bytes"
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => absent.push(marker),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(absent)
}

fn admit_ancestry(path: &Path) -> Result<()> {
    ensure!(
        !path.components().any(|part| matches!(part, Component::ParentDir)),
        "panel destination contains parent traversal"
    );
    for (index, ancestor) in path.ancestors().enumerate() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                ensure!(
                    !metadata.file_type().is_symlink(),
                    "panel destination cannot follow symlinks"
                );
                if index > 0 && !metadata.is_dir() {
                    bail!("panel destination parent is not a directory");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
