use crate::domain::{SourceKind, SourceSpec};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub(super) fn validate_source_path(root: &Path, source: &SourceSpec) -> Result<PathBuf> {
    let path = root.join(&source.locator);
    match source.kind {
        SourceKind::RepoFile | SourceKind::Document => {
            if !path.is_file() {
                return Err(anyhow!("required source path is not a file: {}", path.display()));
            }
        }
        SourceKind::RepoDirectory => {
            if !path.is_dir() {
                return Err(anyhow!("required source path is not a directory: {}", path.display()));
            }
        }
        SourceKind::ExternalDocument | SourceKind::ExternalRepository | SourceKind::Paper => {
            return Err(anyhow!(
                "external source {} cannot be resolved as a repository path",
                source.source_id
            ));
        }
    }
    Ok(path)
}
