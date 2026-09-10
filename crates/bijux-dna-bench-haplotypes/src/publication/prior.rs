//! Admit prior ownership without claiming unrelated files.
use super::model::{ArtifactOwnership, ReportOwner, INVENTORY, SCHEMA};
use super::paths::{admit_ancestry, admit_relative, read_bounded};
use anyhow::{bail, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
pub(super) fn admit_prior(
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
            bail!("haplotype artifact inventory has a foreign owner");
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
        bail!("haplotype publication report has a foreign owner");
    }
    // Legacy reports predate file inventories. Their admitted stage/tool owns only
    // the exact paths produced by this invocation; all other legacy files survive.
    Ok(current.clone())
}
