use std::path::{Path, PathBuf};

use bijux_dna_core::contract::ArtifactRef;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::ReadGroupSpec;

pub const BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION: &str = "bijux.bam.artifact_inventory.v1";
pub const BAM_SAMPLE_IDENTITY_SCHEMA_VERSION: &str = "bijux.bam.sample_identity.v1";
pub const BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION: &str = "bijux.bam.reference_preflight.v1";
pub const BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION: &str = "bijux.bam.alignment_provenance.v1";
pub const BAM_VALIDATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.validate.v1";
pub const BAM_MAPPING_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.mapping_summary.v1";
pub const BAM_COVERAGE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.coverage_summary.v1";
pub const BAM_DUPLICATE_POLICY_SCHEMA_VERSION: &str = "bijux.bam.duplicate_policy.v1";
pub const BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION: &str = "bijux.bam.advisory_boundary.v1";
pub const BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION: &str = "bijux.bam.workflow_template.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamArtifactEntryV1 {
    pub name: String,
    pub role: String,
    pub path: PathBuf,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamArtifactInventoryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_family: String,
    pub output_root: PathBuf,
    pub outputs: Vec<BamArtifactEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamSampleIdentityV1 {
    pub schema_version: String,
    pub sample_id: String,
    #[serde(default)]
    pub lane_id: Option<String>,
    #[serde(default)]
    pub library_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub platform_unit: Option<String>,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub subject_id: Option<String>,
    #[serde(default)]
    pub cohort_id: Option<String>,
    #[serde(default)]
    pub read_group_policy: Option<String>,
    #[serde(default)]
    pub read_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamReferenceAssetIdentityV1 {
    pub asset_kind: String,
    pub path: PathBuf,
    #[serde(default)]
    pub sha256: Option<String>,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamReferencePreflightV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub reference_digest: Option<String>,
    pub contig_alias_policy: String,
    pub required_assets: Vec<BamReferenceAssetIdentityV1>,
    pub passes: bool,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamFlagstatCountsV1 {
    #[serde(default)]
    pub total_reads: Option<u64>,
    #[serde(default)]
    pub mapped_reads: Option<u64>,
    #[serde(default)]
    pub duplicate_reads: Option<u64>,
    #[serde(default)]
    pub mapped_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamValidationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    #[serde(default)]
    pub bam_index: Option<PathBuf>,
    #[serde(default)]
    pub reference_fasta: Option<PathBuf>,
    pub flagstat: BamFlagstatCountsV1,
    pub validation_report_present: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMapqRegimeV1 {
    pub mean: f64,
    pub warn_below: f64,
    pub fail_below: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMappingSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub flagstat: BamFlagstatCountsV1,
    pub stats_present: bool,
    pub idxstats_present: bool,
    #[serde(default)]
    pub mapq_regime: Option<BamMapqRegimeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamCoverageSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub has_mosdepth_summary: bool,
    pub has_samtools_depth: bool,
    #[serde(default)]
    pub mean_depth: Option<f64>,
    #[serde(default)]
    pub coverage_regime: Option<String>,
    #[serde(default)]
    pub coverage_family: Option<String>,
    #[serde(default)]
    pub depth_thresholds: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentProvenanceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub backend_tool_id: String,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub sensitivity_profile: Option<String>,
    #[serde(default)]
    pub seed_length: Option<u32>,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub reference_digest: Option<String>,
    pub sample_identity: BamSampleIdentityV1,
    pub read_group: ReadGroupSpec,
    pub outputs: BamArtifactInventoryV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicatePolicyV1 {
    pub schema_version: String,
    pub stage_id: String,
    #[serde(default)]
    pub library_type: Option<String>,
    #[serde(default)]
    pub optical_duplicates: Option<String>,
    #[serde(default)]
    pub umi_policy: Option<String>,
    #[serde(default)]
    pub duplicate_action: Option<String>,
    pub policy_scope: String,
    #[serde(default)]
    pub library_semantics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAdvisoryBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub advisory_only: bool,
    pub scientific_scope: String,
    #[serde(default)]
    pub evidence_inputs: Vec<String>,
    #[serde(default)]
    pub safe_for_claims: Vec<String>,
    #[serde(default)]
    pub unsafe_for_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamWorkflowModeV1 {
    Modern,
    AncientLike,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamWorkflowTemplateV1 {
    pub schema_version: String,
    pub template_id: String,
    pub mode: BamWorkflowModeV1,
    pub profile_id: String,
    pub summary: String,
    pub required_stages: Vec<String>,
    pub advisory_stages: Vec<String>,
}

#[must_use]
pub fn bam_artifact_inventory_from_outputs(
    stage_id: &str,
    output_root: &Path,
    outputs: &[ArtifactRef],
) -> BamArtifactInventoryV1 {
    BamArtifactInventoryV1 {
        schema_version: BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        stage_family: "bam".to_string(),
        output_root: output_root.to_path_buf(),
        outputs: outputs
            .iter()
            .map(|output| BamArtifactEntryV1 {
                name: output.name.to_string(),
                role: output.role.as_str().to_string(),
                path: output.path.clone(),
                optional: output.optional,
            })
            .collect(),
    }
}

#[must_use]
pub fn bam_sample_identity(
    sample_id: &str,
    read_group: &ReadGroupSpec,
    read_group_policy: Option<&str>,
    lane_id: Option<&str>,
    library_id: Option<&str>,
    platform_unit: Option<&str>,
    run_id: Option<&str>,
    subject_id: Option<&str>,
    cohort_id: Option<&str>,
) -> BamSampleIdentityV1 {
    let lane = lane_id
        .map(ToOwned::to_owned)
        .or_else(|| read_group.lane_id.clone());
    let library = library_id
        .map(ToOwned::to_owned)
        .or_else(|| read_group.library_id());
    let platform = Some(read_group.platform.clone());
    let platform_unit = platform_unit
        .map(ToOwned::to_owned)
        .or_else(|| read_group.platform_unit.clone());
    let run = run_id.map(ToOwned::to_owned).or_else(|| read_group.run_id.clone());
    BamSampleIdentityV1 {
        schema_version: BAM_SAMPLE_IDENTITY_SCHEMA_VERSION.to_string(),
        sample_id: sample_id.to_string(),
        lane_id: lane,
        library_id: library,
        platform,
        platform_unit,
        run_id: run,
        subject_id: subject_id.map(ToOwned::to_owned),
        cohort_id: cohort_id.map(ToOwned::to_owned),
        read_group_policy: read_group_policy.map(ToOwned::to_owned),
        read_group_ids: vec![read_group.id.clone()],
    }
}

#[must_use]
pub fn bam_workflow_templates() -> Vec<BamWorkflowTemplateV1> {
    vec![
        BamWorkflowTemplateV1 {
            schema_version: BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION.to_string(),
            template_id: "bam.essential_modern".to_string(),
            mode: BamWorkflowModeV1::Modern,
            profile_id: "bam-to-bam__default__v1".to_string(),
            summary: "Modern BAM alignment/QC template with enforced validate, mapping summary, and coverage.".to_string(),
            required_stages: vec![
                "bam.align".to_string(),
                "bam.validate".to_string(),
                "bam.mapping_summary".to_string(),
                "bam.mapq_filter".to_string(),
                "bam.coverage".to_string(),
            ],
            advisory_stages: vec![
                "bam.duplication_metrics".to_string(),
                "bam.markdup".to_string(),
            ],
        },
        BamWorkflowTemplateV1 {
            schema_version: BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION.to_string(),
            template_id: "bam.essential_ancient_like".to_string(),
            mode: BamWorkflowModeV1::AncientLike,
            profile_id: "bam-to-bam__adna_shotgun__v1".to_string(),
            summary: "Ancient-like BAM template that keeps validate/alignment enforced and damage/authenticity/contamination explicitly advisory.".to_string(),
            required_stages: vec![
                "bam.align".to_string(),
                "bam.validate".to_string(),
                "bam.mapping_summary".to_string(),
                "bam.mapq_filter".to_string(),
                "bam.coverage".to_string(),
            ],
            advisory_stages: vec![
                "bam.damage".to_string(),
                "bam.authenticity".to_string(),
                "bam.contamination".to_string(),
            ],
        },
    ]
}

#[must_use]
pub fn bam_workflow_template_by_id(template_id: &str) -> Option<BamWorkflowTemplateV1> {
    bam_workflow_templates().into_iter().find(|template| template.template_id == template_id)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use bijux_dna_core::contract::ArtifactRole;
    use bijux_dna_core::prelude::ArtifactId;

    #[test]
    fn bam_artifact_inventory_round_trips() {
        let inventory = bam_artifact_inventory_from_outputs(
            "bam.align",
            Path::new("out"),
            &[ArtifactRef::required(
                ArtifactId::from_static("align_bam"),
                PathBuf::from("out/align.bam"),
                ArtifactRole::Bam,
            )],
        );
        assert_eq!(inventory.stage_family, "bam");
        let json = serde_json::to_string_pretty(&inventory).expect("serialize artifact inventory");
        let reparsed: BamArtifactInventoryV1 =
            serde_json::from_str(&json).expect("deserialize artifact inventory");
        assert_eq!(reparsed.outputs[0].role, ArtifactRole::Bam.as_str());
    }

    #[test]
    fn bam_sample_identity_prefers_declared_and_read_group_defaults() {
        let read_group = ReadGroupSpec {
            id: "rg1".to_string(),
            sample: "sample-a".to_string(),
            platform: "ILLUMINA".to_string(),
            library: "lib-a".to_string(),
            platform_unit: Some("pu-01".to_string()),
            lane_id: Some("L001".to_string()),
            run_id: Some("run-a".to_string()),
        };
        let identity = bam_sample_identity(
            "sample-a",
            &read_group,
            Some("regenerate"),
            None,
            None,
            None,
            None,
            Some("subject-a"),
            Some("cohort-a"),
        );
        assert_eq!(identity.lane_id.as_deref(), Some("L001"));
        assert_eq!(identity.library_id.as_deref(), Some("lib-a"));
        assert_eq!(identity.platform_unit.as_deref(), Some("pu-01"));
        assert_eq!(identity.run_id.as_deref(), Some("run-a"));
        assert_eq!(identity.subject_id.as_deref(), Some("subject-a"));
    }

    #[test]
    fn bam_workflow_templates_are_distinct_and_lookupable() {
        let templates = bam_workflow_templates();
        assert_eq!(templates.len(), 2);
        assert!(templates[0].required_stages.contains(&"bam.align".to_string()));
        let ancient = bam_workflow_template_by_id("bam.essential_ancient_like")
            .expect("ancient-like template");
        assert!(ancient.advisory_stages.contains(&"bam.damage".to_string()));
    }
}
