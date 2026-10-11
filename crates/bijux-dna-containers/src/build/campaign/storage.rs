//! Owner: progress decoding and immutable image campaign publication.
use super::admission::{execution_from_tools, validate_execution};
use super::model::{ImageCampaignExecution, ImageCampaignToolOutcome};
use anyhow::{bail, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::{fs, path::Path};
pub(super) fn load_progress(
    path: &Path,
    commit: &str,
    catalog_sha256: &str,
    architecture: &str,
) -> Result<BTreeMap<String, ImageCampaignToolOutcome>> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let execution: ImageCampaignExecution = serde_json::from_slice(&fs::read(path)?)?;
    validate_execution(&execution, commit, catalog_sha256, architecture)?;
    Ok(execution.tools)
}

pub(super) fn write_progress(
    path: &Path,
    commit: &str,
    catalog_sha256: &str,
    architecture: &str,
    tools: &BTreeMap<String, ImageCampaignToolOutcome>,
) -> Result<()> {
    let execution = execution_from_tools(
        commit.to_string(),
        catalog_sha256.to_string(),
        architecture.to_string(),
        tools.clone(),
    );
    Ok(bijux_dna_infra::atomic_write_json(path, &execution)?)
}

pub(super) fn publish_campaign_execution(
    evidence_root: &Path,
    execution: &ImageCampaignExecution,
) -> Result<()> {
    let attempts_root = evidence_root.join("executions");
    bijux_dna_infra::ensure_dir(&attempts_root)?;
    for attempt in 1..=999_u16 {
        let path = attempts_root.join(format!("execution-{attempt:03}.json"));
        match bijux_dna_infra::atomic_create_json(&path, execution) {
            Ok(()) => {
                if execution.failed == 0 {
                    ensure_immutable_json(&evidence_root.join("execution.json"), execution)?;
                }
                return Ok(());
            }
            Err(_) if path.is_file() => {}
            Err(error) => return Err(error.into()),
        }
    }
    bail!("image campaign exhausted 999 immutable execution identities")
}

pub(super) fn ensure_immutable_json<T: Serialize + serde::de::DeserializeOwned + PartialEq>(
    path: &Path,
    value: &T,
) -> Result<()> {
    match bijux_dna_infra::atomic_create_json(path, value) {
        Ok(()) => Ok(()),
        Err(error) if path.is_file() => {
            let existing: T = serde_json::from_slice(&fs::read(path)?)?;
            if &existing == value {
                Ok(())
            } else {
                bail!(
                    "immutable image campaign publication conflicts at {}: {error}",
                    path.display()
                )
            }
        }
        Err(error) => Err(error.into()),
    }
}
