//! Persisted BAM population reports and schema identities.

use crate::metrics::SexConfidenceClass;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION: &str = "bijux.bam.scientific_report.v1";

pub const BAM_ENDOGENOUS_CONTENT_SCHEMA_VERSION: &str = "bijux.bam.endogenous_content.v1";

pub const BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.endogenous_truth.v1";

pub const BAM_SEX_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.sex_summary.v1";

pub const BAM_SEX_EVIDENCE_SCHEMA_VERSION: &str = "bijux.bam.sex_evidence.v1";

pub const BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.haplogroup_truth.v1";

pub const BAM_HAPLOGROUP_READINESS_SCHEMA_VERSION: &str = "bijux.bam.haplogroup_readiness.v1";

pub const BAM_KINSHIP_PREREQUISITES_SCHEMA_VERSION: &str = "bijux.bam.kinship_prerequisites.v1";

pub const BAM_KINSHIP_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.kinship_summary.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamEndogenousContentEstimateV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub mapped_reads: u64,
    pub endogenous_reads: u64,
    pub total_reads: u64,
    pub endogenous_fraction: f64,
    #[serde(default)]
    pub prealignment_fraction: Option<f64>,
    pub postalignment_fraction: f64,
    #[serde(default)]
    pub host_reference_scope: Option<String>,
    pub prealignment_meaning: String,
    pub postalignment_meaning: String,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamEndogenousTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    #[serde(default)]
    pub host_reference_scope: Option<String>,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub contaminant_reads: u64,
    pub retained_reads: u64,
    pub endogenous_reads: u64,
    pub contaminant_fraction: f64,
    pub retained_fraction: f64,
    pub endogenous_fraction: f64,
    #[serde(default)]
    pub prealignment_fraction: Option<f64>,
    pub postalignment_fraction: f64,
    pub count_provenance: String,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamSexSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub input_bam: PathBuf,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub chromosome_system: Option<String>,
    #[serde(default)]
    pub minimum_y_sites: Option<u32>,
    pub x_contig: String,
    pub y_contig: String,
    pub autosomal_contigs: Vec<String>,
    pub x_coverage: f64,
    pub y_coverage: f64,
    pub autosomal_coverage: f64,
    pub x_covered_sites: u64,
    pub y_covered_sites: u64,
    #[serde(default)]
    pub x_to_y_ratio: Option<f64>,
    pub call: SexConfidenceClass,
    pub confidence: f64,
    pub status: String,
    #[serde(default)]
    pub insufficiency_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamSexInferenceEvidenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub par_aware: bool,
    pub prerequisites_passed: bool,
    pub classification: SexConfidenceClass,
    pub confidence: f64,
    #[serde(default)]
    pub x_to_y_ratio: Option<f64>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamKinshipPairResultV1 {
    pub sample_a: String,
    pub sample_b: String,
    pub overlap_snps: u32,
    pub matching_sites: u32,
    pub mismatch_sites: u32,
    pub concordance: f64,
    pub kinship_coefficient: f64,
    pub relationship_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamKinshipSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub input_bam: PathBuf,
    pub reference_panel: String,
    pub reference_build: String,
    pub population_scope: String,
    pub min_overlap_snps: u32,
    pub requires_cohort_context: bool,
    pub sample_count: u32,
    pub observed_max_overlap_snps: u32,
    pub pair_count: u32,
    pub status: String,
    #[serde(default)]
    pub insufficiency_reason: Option<String>,
    #[serde(default)]
    pub pairwise_results: Vec<BamKinshipPairResultV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamHaplogroupReadinessV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub ready: bool,
    #[serde(default)]
    pub reference_build: Option<String>,
    pub minimum_coverage: f64,
    pub observed_mean_coverage: f64,
    #[serde(default)]
    pub contamination_estimate: Option<f64>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamHaplogroupTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub input_bam: PathBuf,
    pub reference_panel: PathBuf,
    pub reference_panel_id: String,
    pub reference_build: String,
    pub population_scope: String,
    pub minimum_coverage: f64,
    pub observed_mean_coverage: f64,
    pub ready: bool,
    #[serde(default)]
    pub haplogroup_call: Option<String>,
    pub confidence: f64,
    pub status: String,
    pub markers_total: u64,
    pub markers_supported: u64,
    #[serde(default)]
    pub supported_marker_ids: Vec<String>,
    #[serde(default)]
    pub lineage_scope: Option<String>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamKinshipPrerequisitesV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub ready: bool,
    pub minimum_mean_coverage: f64,
    pub observed_mean_coverage: f64,
    pub marker_overlap_snps: u32,
    pub sample_identity_consistent: bool,
    pub contamination_estimate: f64,
    pub max_contamination: f64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamScientificReportIdV1 {
    EndogenousContent,
    Sex,
    Haplogroups,
    Kinship,
}

impl BamScientificReportIdV1 {
    #[must_use]
    pub const fn stage_id(self) -> &'static str {
        match self {
            Self::EndogenousContent => "bam.endogenous_content",
            Self::Sex => "bam.sex",
            Self::Haplogroups => "bam.haplogroups",
            Self::Kinship => "bam.kinship",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamScientificReportContractV1 {
    pub schema_version: String,
    pub report_id: BamScientificReportIdV1,
    pub optional: bool,
    pub suitable_scopes: Vec<String>,
    pub required_population_or_reference_context: Vec<String>,
    pub refusal_when_missing: Vec<String>,
}
