use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use serde::{Deserialize, Serialize};

pub const GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION: &str = "bijux.fastq.report_qc.inputs.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedQcManifestContributorV1 {
    pub contributor_id: String,
    pub stage_id: String,
    #[serde(default)]
    pub tool_id: String,
    pub artifact_id: String,
    pub artifact_role: ArtifactRole,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedQcInputsManifestV1 {
    pub schema_version: String,
    pub qc_inputs: Vec<ArtifactRef>,
    #[serde(default)]
    pub contributors: Vec<GovernedQcManifestContributorV1>,
    #[serde(default)]
    pub raw_fastqc_dir: Option<PathBuf>,
    #[serde(default)]
    pub lineage_hash: Option<String>,
}

#[must_use]
pub fn governed_qc_contributors_from_inputs(
    qc_inputs: &[ArtifactRef],
) -> Vec<GovernedQcManifestContributorV1> {
    let mut contributors =
        qc_inputs.iter().filter_map(governed_qc_contributor_from_artifact).collect::<Vec<_>>();
    contributors.sort_by(|left, right| {
        left.contributor_id
            .cmp(&right.contributor_id)
            .then_with(|| left.artifact_id.cmp(&right.artifact_id))
            .then_with(|| left.artifact_role.as_str().cmp(right.artifact_role.as_str()))
            .then_with(|| left.path.cmp(&right.path))
    });
    contributors.dedup_by(|left, right| {
        left.contributor_id == right.contributor_id
            && left.artifact_id == right.artifact_id
            && left.artifact_role == right.artifact_role
            && left.path == right.path
    });
    contributors
}

#[must_use]
pub fn governed_qc_inputs_manifest_from_inputs(
    qc_inputs: &[ArtifactRef],
) -> GovernedQcInputsManifestV1 {
    let contributors = governed_qc_contributors_from_inputs(qc_inputs);
    GovernedQcInputsManifestV1 {
        schema_version: GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION.to_string(),
        qc_inputs: qc_inputs.to_vec(),
        contributors: contributors.clone(),
        raw_fastqc_dir: None,
        lineage_hash: derived_governed_qc_lineage_hash(&contributors),
    }
}

#[must_use]
pub fn derived_governed_qc_lineage_hash(
    contributors: &[GovernedQcManifestContributorV1],
) -> Option<String> {
    if contributors.is_empty() {
        return None;
    }
    Some(
        contributors
            .iter()
            .map(|contributor| {
                format!(
                    "{}:{}:{}={}",
                    contributor.contributor_id,
                    contributor.artifact_id,
                    contributor.artifact_role.as_str(),
                    contributor.path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("|"),
    )
}

fn governed_qc_contributor_from_artifact(
    artifact: &ArtifactRef,
) -> Option<GovernedQcManifestContributorV1> {
    let artifact_name = artifact.name.as_str();
    let (contributor_id, artifact_id) = artifact_name.rsplit_once('.')?;
    let contributor_parts = contributor_id.split('.').collect::<Vec<_>>();
    if contributor_parts.len() < 3 {
        return None;
    }
    let tool_id = if contributor_parts.get(2) == Some(&"tool") {
        contributor_parts.get(3..)?.join(".")
    } else {
        contributor_parts[2..].join(".")
    };
    Some(GovernedQcManifestContributorV1 {
        contributor_id: contributor_id.to_string(),
        stage_id: format!("{}.{}", contributor_parts[0], contributor_parts[1]),
        tool_id,
        artifact_id: artifact_id.to_string(),
        artifact_role: artifact.role,
        path: artifact.path.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        derived_governed_qc_lineage_hash, governed_qc_contributors_from_inputs,
        governed_qc_inputs_manifest_from_inputs, GovernedQcInputsManifestV1,
        GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
    };
    use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
    use bijux_dna_core::ids::ArtifactId;
    use std::path::Path;

    #[test]
    fn governed_qc_inputs_manifest_round_trips() {
        let manifest = governed_qc_inputs_manifest_from_inputs(&[
            ArtifactRef::required(
                ArtifactId::from_static("fastq.profile_reads.tool.seqkit_stats.qc_json"),
                Path::new("profile_reads/qc.json").to_path_buf(),
                ArtifactRole::MetricsJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("fastq.detect_adapters.tool.fastqc.report_json"),
                Path::new("detect_adapters/report.json").to_path_buf(),
                ArtifactRole::ReportJson,
            ),
        ]);

        let encoded =
            serde_json::to_string(&manifest).unwrap_or_else(|error| panic!("serialize: {error}"));
        let decoded: GovernedQcInputsManifestV1 =
            serde_json::from_str(&encoded).unwrap_or_else(|error| panic!("deserialize: {error}"));
        assert_eq!(decoded.schema_version, GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION);
        assert_eq!(decoded.qc_inputs.len(), 2);
        assert_eq!(decoded.contributors.len(), 2);
        assert!(decoded.lineage_hash.is_some());
    }

    #[test]
    fn governed_qc_contributors_are_sorted_and_deduplicated() {
        let contributors = governed_qc_contributors_from_inputs(&[
            ArtifactRef::required(
                ArtifactId::from_static("fastq.detect_adapters.tool.fastqc.report_json"),
                Path::new("detect_adapters/report.json").to_path_buf(),
                ArtifactRole::ReportJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("fastq.detect_adapters.tool.fastqc.report_json"),
                Path::new("detect_adapters/report.json").to_path_buf(),
                ArtifactRole::ReportJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("fastq.profile_reads.tool.seqkit_stats.qc_json"),
                Path::new("profile_reads/qc.json").to_path_buf(),
                ArtifactRole::MetricsJson,
            ),
        ]);
        assert_eq!(contributors.len(), 2);
        assert_eq!(contributors[0].stage_id, "fastq.detect_adapters");
        assert_eq!(contributors[1].stage_id, "fastq.profile_reads");
        assert!(derived_governed_qc_lineage_hash(&contributors).is_some());
    }
}
