//! Owner: bijux-dna-domain-compiler
//! Shared artifact vocabulary admission for validation and registry snapshots.

use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use super::super::support::read_yaml;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactRoleSnapshot {
    pub artifact_id: String,
    pub artifact_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactDoc {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    artifact_ids: Vec<String>,
    #[serde(default)]
    artifacts: Vec<ArtifactDocEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactDocEntry {
    id: String,
    #[serde(default, rename = "type")]
    artifact_type: Option<String>,
}

pub(crate) fn load_artifacts(
    domain_dir: &Path,
    domain_id: &str,
) -> Result<(String, Vec<ArtifactRoleSnapshot>)> {
    let path = domain_dir.join(domain_id).join("artifacts.yaml");
    let doc: ArtifactDoc = read_yaml(&path)?;
    if doc.domain != domain_id {
        return Err(anyhow!(
            "{} has domain {} but expected {}",
            path.display(),
            doc.domain,
            domain_id
        ));
    }
    if !doc.artifact_ids.is_empty() && !doc.artifacts.is_empty() {
        return Err(anyhow!(
            "{} mixes artifact_ids and artifacts; use one complete declaration so no IDs are discarded",
            path.display()
        ));
    }
    let mut artifacts = if doc.artifacts.is_empty() {
        doc.artifact_ids
            .into_iter()
            .map(|artifact_id| ArtifactRoleSnapshot { artifact_id, artifact_type: None })
            .collect::<Vec<_>>()
    } else {
        doc.artifacts
            .into_iter()
            .map(|entry| ArtifactRoleSnapshot {
                artifact_id: entry.id,
                artifact_type: entry.artifact_type,
            })
            .collect::<Vec<_>>()
    };
    artifacts.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    for artifact in &artifacts {
        if artifact.artifact_id.trim().is_empty()
            || artifact.artifact_id.trim() != artifact.artifact_id
        {
            return Err(anyhow!("{} contains a blank or padded artifact ID", path.display()));
        }
    }
    if let Some(pair) = artifacts.windows(2).find(|pair| pair[0].artifact_id == pair[1].artifact_id)
    {
        return Err(anyhow!("{} repeats artifact ID {}", path.display(), pair[0].artifact_id));
    }
    Ok((doc.schema_version, artifacts))
}
