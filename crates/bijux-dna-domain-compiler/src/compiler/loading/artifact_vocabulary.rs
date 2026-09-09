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
#[serde(deny_unknown_fields)]
struct ArtifactDoc {
    schema_version: String,
    domain: String,
    #[serde(default, deserialize_with = "present_list")]
    artifact_ids: Option<Vec<String>>,
    #[serde(default, deserialize_with = "present_list")]
    artifacts: Option<Vec<ArtifactDocEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactDocEntry {
    id: String,
    #[serde(rename = "type")]
    artifact_type: String,
}

// A missing declaration is distinct from an explicitly null declaration.
fn present_list<'de, D, T>(deserializer: D) -> std::result::Result<Option<Vec<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Vec::<T>::deserialize(deserializer).map(Some)
}

pub(crate) fn load_artifacts(
    domain_dir: &Path,
    domain_id: &str,
) -> Result<(String, Vec<ArtifactRoleSnapshot>)> {
    let path = domain_dir.join(domain_id).join("artifacts.yaml");
    let doc: ArtifactDoc = read_yaml(&path)?;
    if !matches!(doc.schema_version.as_str(), "bijux.domain.artifacts.v1" | "bijux.artifacts.v1") {
        return Err(anyhow!(
            "{} uses unsupported artifact schema {}",
            path.display(),
            doc.schema_version
        ));
    }
    if doc.domain != domain_id {
        return Err(anyhow!(
            "{} has domain {} but expected {}",
            path.display(),
            doc.domain,
            domain_id
        ));
    }
    let mut artifacts = match (doc.artifact_ids, doc.artifacts) {
        (Some(ids), None) => ids.into_iter().map(|artifact_id| ArtifactRoleSnapshot {
            artifact_id, artifact_type: None,
        }).collect::<Vec<_>>(),
        (None, Some(entries)) => entries.into_iter().map(|entry| ArtifactRoleSnapshot {
            artifact_id: entry.id, artifact_type: Some(entry.artifact_type),
        }).collect::<Vec<_>>(),
        (Some(_), Some(_)) => return Err(anyhow!(
            "{} mixes artifact_ids and artifacts; use one complete declaration so no IDs are discarded", path.display()
        )),
        (None, None) => return Err(anyhow!("{} requires artifact_ids or artifacts", path.display())),
    };
    if artifacts.is_empty() {
        return Err(anyhow!("{} requires a non-empty artifact vocabulary", path.display()));
    }
    artifacts.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    for artifact in &artifacts {
        if artifact
            .artifact_type
            .as_deref()
            .is_some_and(|kind| kind.is_empty() || kind.trim() != kind)
        {
            return Err(anyhow!(
                "{} contains a blank or padded artifact type for {}",
                path.display(),
                artifact.artifact_id
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::load_artifacts;
    use anyhow::Context as _;

    #[test]
    fn artifact_source_schemas_reject_unknown_or_incomplete_fields() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let domain = root.path().join("fastq");
        std::fs::create_dir(&domain)?;
        let path = domain.join("artifacts.yaml");
        for (body, diagnostic) in [
            ("artifacts: [{id: reads, type: fastq, required: true}]", "unknown field"),
            ("artifacts: [{id: reads, format: fastq}]", "unknown field"),
            ("artifacts: [{id: reads}]", "missing field"),
            ("artifacts: [{id: reads, type: ''}]", "blank or padded artifact type"),
            ("artifacts: [{id: reads, type: ' fastq '}]", "blank or padded artifact type"),
            ("artifact_ids: [reads]\nextra: true", "unknown field"),
            (
                "artifact_ids: []\nartifacts: [{id: reads, type: fastq}]",
                "mixes artifact_ids and artifacts",
            ),
            ("artifact_ids: null", "invalid type"),
            ("artifacts: null", "invalid type"),
            ("artifact_ids: null\nartifacts: [{id: reads, type: fastq}]", "invalid type"),
            ("artifact_ids: []", "non-empty artifact vocabulary"),
            ("artifacts: []", "non-empty artifact vocabulary"),
            ("", "requires artifact_ids or artifacts"),
        ] {
            std::fs::write(
                &path,
                format!("schema_version: bijux.artifacts.v1\ndomain: fastq\n{body}\n"),
            )?;
            let error = load_artifacts(root.path(), "fastq")
                .err()
                .context("malformed artifact document must fail")?;
            assert!(format!("{error:#}").contains(diagnostic), "{body}: {error:#}");
        }
        Ok(())
    }

    #[test]
    fn artifact_schema_versions_preserve_supported_forms_and_reject_unknown_versions(
    ) -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let domain = root.path().join("fastq");
        std::fs::create_dir(&domain)?;
        let path = domain.join("artifacts.yaml");
        for version in ["bijux.domain.artifacts.v1", "bijux.artifacts.v1"] {
            for body in ["artifact_ids: [reads]", "artifacts: [{id: reads, type: fastq}]"] {
                std::fs::write(
                    &path,
                    format!("schema_version: {version}\ndomain: fastq\n{body}\n"),
                )?;
                let (actual_version, artifacts) = load_artifacts(root.path(), "fastq")?;
                assert_eq!(actual_version, version);
                assert_eq!(artifacts[0].artifact_id, "reads");
            }
        }
        for version in ["bijux.artifacts.v9", "''"] {
            std::fs::write(
                &path,
                format!("schema_version: {version}\ndomain: fastq\nartifact_ids: [reads]\n"),
            )?;
            assert!(load_artifacts(root.path(), "fastq").is_err());
        }
        std::fs::write(&path, "domain: fastq\nartifact_ids: [reads]\n")?;
        assert!(load_artifacts(root.path(), "fastq").is_err());
        Ok(())
    }
}
