//! Owner: bijux-dna-bench-workflows
//! Admit prior workflow ownership before retiring declared generated files.
use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

/// Preserve unrelated files; only a matching prior report can identify retired outputs.
/// Callers must admit their scientific inputs before invoking this effect boundary.
pub(crate) fn prepare_output_root(
    repository: &Path,
    output_root: &Path,
    report_path: &Path,
    pipeline_id: &str,
    schema_version: &str,
) -> Result<()> {
    let repository = repository.canonicalize().context("resolve workflow repository")?;
    if report_path.parent() != Some(output_root) {
        bail!("workflow report must belong directly to its declared output root");
    }
    reject_parent_components(output_root)?;
    if output_root.is_symlink() || report_path.is_symlink() {
        bail!("workflow output ownership cannot follow symbolic links");
    }
    if !output_root.exists() {
        bijux_dna_infra::ensure_dir(output_root)?;
        return Ok(());
    }
    let root = output_root.canonicalize().context("resolve workflow output root")?;
    if !root.is_dir() || repository.starts_with(&root) {
        bail!("workflow output root must not contain the repository itself");
    }
    if !report_path.exists() {
        let artifacts = root.join("artifacts");
        if artifacts.is_symlink()
            || (artifacts.exists() && std::fs::read_dir(&artifacts)?.next().is_some())
        {
            bail!("existing workflow artifacts have no matching ownership report; preserve or relocate them before rerunning");
        }
        return Ok(());
    }
    let retired =
        admit_previous_outputs(&repository, &root, report_path, pipeline_id, schema_version)?;
    // Every path is checked before the first removal, including absent paths and symlink ancestry.
    for path in retired {
        bijux_dna_infra::remove_file_if_exists(&path)?;
    }
    bijux_dna_infra::remove_file_if_exists(report_path)?;
    Ok(())
}

fn admit_previous_outputs(
    repository: &Path,
    root: &Path,
    report_path: &Path,
    pipeline_id: &str,
    schema_version: &str,
) -> Result<BTreeSet<PathBuf>> {
    let metadata = std::fs::symlink_metadata(report_path)?;
    if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
        bail!("workflow ownership report must be a bounded regular file");
    }
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
    if report["pipeline_id"].as_str() != Some(pipeline_id)
        || report["schema_version"].as_str() != Some(schema_version)
    {
        bail!("existing report belongs to a different workflow contract");
    }
    let expected =
        report_path.strip_prefix(repository).unwrap_or(report_path).display().to_string();
    if report["output_path"].as_str() != Some(expected.as_str()) {
        bail!("existing workflow report does not identify its own publication path");
    }
    let rows = report["rows"].as_array().context("workflow report has no stage rows")?;
    let mut retired = BTreeSet::new();
    for row in rows {
        let outputs =
            row["outputs"].as_object().context("workflow stage has no declared outputs")?;
        for value in outputs.values().chain(std::iter::once(&row["evidence_path"])) {
            if value.is_null() {
                continue;
            }
            let raw = value.as_str().context("workflow output declaration must be a path")?;
            let candidate = Path::new(raw);
            reject_parent_components(candidate)?;
            let path = if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                repository.join(candidate)
            };
            // Source/reference artifacts outside this output root are never owned by this run.
            if !path.starts_with(root) {
                continue;
            }
            for ancestor in path.ancestors().take_while(|part| *part != root) {
                if ancestor.is_symlink() {
                    bail!("workflow output declaration traverses a symbolic link");
                }
            }
            if path.exists() {
                if !path.canonicalize()?.starts_with(root) {
                    bail!("workflow output declaration escapes its owner");
                }
                if path.is_file() {
                    retired.insert(path);
                }
            }
        }
    }
    Ok(retired)
}

fn reject_parent_components(path: &Path) -> Result<()> {
    if path.components().any(|part| matches!(part, Component::ParentDir)) {
        bail!("workflow output declarations must not contain parent traversal");
    }
    Ok(())
}
