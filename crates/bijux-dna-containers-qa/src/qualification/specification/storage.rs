//! Bounded JSON/YAML acquisition and atomic schema publication.
use super::model::{
    qualification_source_selection_schema, qualification_specification_source_schema,
    QualificationSourceSelectionV1, QualificationSpecificationSourceV1,
};
use anyhow::{bail, Context, Result};
use std::{fs, path::Path};

/// Write the generated declarative compiler-input schema.
///
/// # Errors
/// Returns an error when the schema cannot be serialized or written.
pub fn write_qualification_specification_source_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &qualification_specification_source_schema())
        .context("write qualification specification source schema")
}

/// Write the generated qualification data-selection schema.
///
/// # Errors
/// Returns an error when the schema cannot be serialized or written.
pub fn write_qualification_source_selection_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &qualification_source_selection_schema())
        .context("write qualification source selection schema")
}

/// Load one declarative qualification data selection from JSON or YAML.
///
/// # Errors
/// Returns an error when the source is not a bounded regular file or cannot be decoded.
pub fn load_qualification_source_selection(path: &Path) -> Result<QualificationSourceSelectionV1> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect qualification source selection {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("qualification source selection is not a regular file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
        .with_context(|| format!("read qualification source selection {}", path.display()))?;
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => {
            serde_json::from_slice(&bytes).context("parse qualification source selection JSON")
        }
        Some("yaml" | "yml") => {
            serde_yaml::from_slice(&bytes).context("parse qualification source selection YAML")
        }
        _ => bail!("qualification source selection must use a JSON or YAML extension"),
    }
}

/// Load one declarative qualification source from JSON or YAML.
///
/// # Errors
/// Returns an error when the source cannot be read or decoded.
pub fn load_qualification_specification_source(
    path: &Path,
) -> Result<QualificationSpecificationSourceV1> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect qualification source {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("qualification source is not a regular file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
        .with_context(|| format!("read qualification source {}", path.display()))?;
    match path.extension().and_then(|value| value.to_str()) {
        Some("json") => serde_json::from_slice(&bytes).context("parse qualification source JSON"),
        Some("yaml" | "yml") => {
            serde_yaml::from_slice(&bytes).context("parse qualification source YAML")
        }
        _ => bail!("qualification source must use a JSON or YAML extension"),
    }
}
