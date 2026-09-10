//! Owner: bijux-dna-bench-population
//! Publish an admitted artifact inventory while retaining unrelated local files.
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

const INVENTORY: &str = "population-artifact-ownership.json";
const SCHEMA: &str = "bijux.bench.population_artifact_ownership.v1";

/// Identity of the report that owns this fixed publication location.
pub(crate) struct ReportOwner<'a> {
    pub(crate) report_name: &'a str,
    pub(crate) schema: &'a str,
    pub(crate) stage: &'a str,
    pub(crate) tool: &'a str,
}

#[derive(Serialize, Deserialize)]
struct ArtifactOwnership {
    schema_version: String,
    report_schema: String,
    stage_id: String,
    tool_id: String,
    files: BTreeSet<PathBuf>,
}

/// Preflight every destination before effects. File replacement is atomic; publishing
/// the complete set is not transactional if the filesystem fails during replacement.
pub(crate) fn publish(staged: &Path, destination: &Path, owner: &ReportOwner<'_>) -> Result<()> {
    admit_ancestry(destination)?;
    admit_ancestry(staged)?;
    let files = collect_files(staged)?;
    if !files.contains(Path::new(owner.report_name)) || files.contains(Path::new(INVENTORY)) {
        bail!("population staging must contain its report and no ownership receipt");
    }
    let prior = admit_prior(destination, owner, &files)?;
    for file in &files {
        let target = destination.join(file);
        admit_ancestry(&target)?;
        if target.exists() && (!target.is_file() || !prior.contains(file)) {
            bail!("refuse to overwrite unowned population artifact {}", target.display());
        }
    }
    for file in &prior {
        admit_relative(file)?;
        let target = destination.join(file);
        admit_ancestry(&target)?;
        if target.exists() && !target.is_file() {
            bail!("prior population artifact is not a regular file: {}", target.display());
        }
    }
    bijux_dna_infra::ensure_dir(destination)?;
    // Publish the report last so readers do not observe a new report before its files.
    for file in files.iter().filter(|file| file.as_path() != Path::new(owner.report_name)) {
        publish_file(staged, destination, file)?;
    }
    publish_file(staged, destination, Path::new(owner.report_name))?;
    for retired in prior.difference(&files) {
        let path = destination.join(retired);
        if path.exists() {
            bijux_dna_infra::remove_file(&path)?;
        }
    }
    bijux_dna_infra::atomic_write_json(
        &destination.join(INVENTORY),
        &ArtifactOwnership {
            schema_version: SCHEMA.to_owned(),
            report_schema: owner.schema.to_owned(),
            stage_id: owner.stage.to_owned(),
            tool_id: owner.tool.to_owned(),
            files,
        },
    )?;
    Ok(())
}

fn admit_prior(
    destination: &Path,
    owner: &ReportOwner<'_>,
    current: &BTreeSet<PathBuf>,
) -> Result<BTreeSet<PathBuf>> {
    let inventory = destination.join(INVENTORY);
    admit_ancestry(&inventory)?;
    if inventory.exists() {
        let previous: ArtifactOwnership = serde_json::from_slice(&read_bounded(&inventory)?)?;
        if previous.schema_version != SCHEMA
            || previous.report_schema != owner.schema
            || previous.stage_id != owner.stage
            || previous.tool_id != owner.tool
            || previous.files.contains(Path::new(INVENTORY))
        {
            bail!("population artifact inventory has a foreign owner");
        }
        for file in &previous.files {
            admit_relative(file)?;
        }
        return Ok(previous.files);
    }
    let report = destination.join(owner.report_name);
    if !report.exists() {
        return Ok(BTreeSet::new());
    }
    let previous: serde_json::Value = serde_json::from_slice(&read_bounded(&report)?)?;
    if previous.get("schema_version").and_then(serde_json::Value::as_str) != Some(owner.schema)
        || previous.get("stage_id").and_then(serde_json::Value::as_str) != Some(owner.stage)
        || previous.get("tool_id").and_then(serde_json::Value::as_str) != Some(owner.tool)
    {
        bail!("population publication report has a foreign owner");
    }
    // Legacy reports predate file inventories. Their admitted stage/tool owns only
    // the exact paths produced by this invocation; all other legacy files survive.
    Ok(current.clone())
}

fn read_bounded(path: &Path) -> Result<Vec<u8>> {
    admit_ancestry(path)?;
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() || metadata.len() > 8 * 1024 * 1024 {
        bail!("population publication ownership must be a bounded regular file");
    }
    fs::read(path).with_context(|| format!("read {}", path.display()))
}

fn collect_files(root: &Path) -> Result<BTreeSet<PathBuf>> {
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
                bail!("population staging contains a nonregular artifact");
            }
        }
    }
    Ok(files)
}

fn admit_relative(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.components().any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("population artifact inventory contains an escaping path");
    }
    Ok(())
}

fn admit_ancestry(path: &Path) -> Result<()> {
    if path.components().any(|part| matches!(part, Component::ParentDir)) {
        bail!("population publication path contains parent traversal");
    }
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                bail!("population publication cannot follow symlinks")
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn publish_file(staged: &Path, destination: &Path, relative: &Path) -> Result<()> {
    let target = destination.join(relative);
    bijux_dna_infra::rename(&staged.join(relative), &target)
        .with_context(|| format!("publish {}", target.display()))
}
