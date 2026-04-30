use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::prelude::ArtifactId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::metrics::{authenticity_score, BamMetricsV1, SexConfidenceClass};
use crate::params::ReadGroupSpec;

pub const BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION: &str = "bijux.bam.artifact_inventory.v1";
pub const BAM_SAMPLE_IDENTITY_SCHEMA_VERSION: &str = "bijux.bam.sample_identity.v1";
pub const BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION: &str = "bijux.bam.reference_preflight.v1";
pub const BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION: &str = "bijux.bam.alignment_provenance.v1";
pub const BAM_VALIDATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.validate.v1";
pub const BAM_MAPPING_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.mapping_summary.v1";
pub const BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.mapq_filter.v1";
pub const BAM_COVERAGE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.coverage_summary.v1";
pub const BAM_DUPLICATE_POLICY_SCHEMA_VERSION: &str = "bijux.bam.duplicate_policy.v1";
pub const BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION: &str = "bijux.bam.advisory_boundary.v1";
pub const BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION: &str = "bijux.bam.workflow_template.v1";
pub const BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION: &str = "bijux.bam.alignment_strategy.v1";
pub const BAM_MERGE_COMPATIBILITY_SCHEMA_VERSION: &str = "bijux.bam.merge_compatibility.v1";
pub const BAM_POST_ALIGNMENT_CHAIN_SCHEMA_VERSION: &str = "bijux.bam.post_alignment_chain.v1";
pub const BAM_DUPLICATE_COMPARISON_SCHEMA_VERSION: &str = "bijux.bam.duplicate_comparison.v1";
pub const BAM_COVERAGE_REGIME_SCHEMA_VERSION: &str = "bijux.bam.coverage_regime.v1";
pub const BAM_ADNA_WORKFLOW_SCHEMA_VERSION: &str = "bijux.bam.adna_workflow.v1";
pub const BAM_CONTAMINATION_WORKFLOW_SCHEMA_VERSION: &str = "bijux.bam.contamination_workflow.v1";
pub const BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION: &str = "bijux.bam.scientific_report.v1";
pub const BAM_RESOURCE_PLAN_SCHEMA_VERSION: &str = "bijux.bam.resource_plan.v1";
pub const BAM_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION: &str = "bijux.bam.bench_corpus_manifest.v1";
pub const BAM_DAMAGE_EVIDENCE_SCHEMA_VERSION: &str = "bijux.bam.damage_evidence.v1";
pub const BAM_AUTHENTICITY_ADVISORY_SCHEMA_VERSION: &str = "bijux.bam.authenticity_advisory.v1";
pub const BAM_CONTAMINATION_EVIDENCE_SCHEMA_VERSION: &str = "bijux.bam.contamination_evidence.v1";
pub const BAM_ENDOGENOUS_CONTENT_SCHEMA_VERSION: &str = "bijux.bam.endogenous_content.v1";
pub const BAM_SEX_EVIDENCE_SCHEMA_VERSION: &str = "bijux.bam.sex_evidence.v1";
pub const BAM_HAPLOGROUP_READINESS_SCHEMA_VERSION: &str = "bijux.bam.haplogroup_readiness.v1";
pub const BAM_KINSHIP_PREREQUISITES_SCHEMA_VERSION: &str = "bijux.bam.kinship_prerequisites.v1";

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
    #[serde(default)]
    pub proper_pair_reads: Option<u64>,
    #[serde(default)]
    pub secondary_reads: Option<u64>,
    #[serde(default)]
    pub supplementary_reads: Option<u64>,
    #[serde(default)]
    pub mapq_histogram: Vec<(u8, u64)>,
    #[serde(default)]
    pub read_group_breakdown: Vec<BamReadGroupMappingCountV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamReadGroupMappingCountV1 {
    pub read_group_id: String,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub proper_pair_reads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMapqFilterSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub mapq_threshold: u8,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub flagstat_before: BamFlagstatCountsV1,
    pub flagstat_after: BamFlagstatCountsV1,
    #[serde(default)]
    pub mapped_reads_removed: Option<u64>,
    #[serde(default)]
    pub mapped_fraction_retained: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamAlignmentSuitabilityV1 {
    GeneralShortRead,
    WholeGenomeLike,
    TargetedCapture,
    AncientShortFragments,
    SensitiveLocalRescue,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamPostAlignmentChainV1 {
    pub schema_version: String,
    pub chain_id: String,
    pub sorting_mode: String,
    pub index_type: String,
    pub validate_before_downstream: bool,
    pub coordinate_sorted_required_for: Vec<String>,
    pub downstream_refusal_stages: Vec<String>,
    pub required_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAlignmentStrategyV1 {
    pub schema_version: String,
    pub strategy_id: String,
    pub tool_id: String,
    pub default_preset: String,
    pub mode: String,
    pub supported_inputs: Vec<String>,
    pub required_reference_assets: Vec<String>,
    pub suitability: Vec<BamAlignmentSuitabilityV1>,
    pub hidden_default_allowed: bool,
    pub post_alignment_chain: BamPostAlignmentChainV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamMergeInputIdentityV1 {
    pub sample_id: String,
    #[serde(default)]
    pub read_group_ids: Vec<String>,
    #[serde(default)]
    pub reference_digest: Option<String>,
    #[serde(default)]
    pub sequencing_platform: Option<String>,
    #[serde(default)]
    pub library_id: Option<String>,
    #[serde(default)]
    pub lane_id: Option<String>,
    #[serde(default)]
    pub platform_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamMergeCompatibilityV1 {
    pub schema_version: String,
    pub compatible: bool,
    #[serde(default)]
    pub merged_sample_id: Option<String>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicateMethodMetricsV1 {
    pub method: String,
    #[serde(default)]
    pub duplicate_reads: Option<u64>,
    #[serde(default)]
    pub duplicate_fraction: Option<f64>,
    #[serde(default)]
    pub optical_duplicates: Option<String>,
    #[serde(default)]
    pub duplicate_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicateComparisonV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method_a: BamDuplicateMethodMetricsV1,
    pub method_b: BamDuplicateMethodMetricsV1,
    pub comparable: bool,
    #[serde(default)]
    pub duplicate_reads_delta: Option<i64>,
    #[serde(default)]
    pub duplicate_fraction_delta: Option<f64>,
    pub policy_explicit: bool,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamCoverageRegimeClassV1 {
    Unusable,
    Sparse,
    LowPass,
    TargetLike,
    WholeGenomeLike,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamCoverageRegimeV1 {
    pub schema_version: String,
    pub regime_id: String,
    pub advisory_label: String,
    pub enforced_label: String,
    pub regime_class: BamCoverageRegimeClassV1,
    pub mean_depth: f64,
    pub breadth_1x: f64,
    pub usable_for: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
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
    pub regime: Option<BamCoverageRegimeV1>,
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
    pub strategy_id: Option<String>,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub sensitivity_profile: Option<String>,
    #[serde(default)]
    pub seed_length: Option<u32>,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub reference_digest: Option<String>,
    #[serde(default)]
    pub post_alignment_chain: Option<BamPostAlignmentChainV1>,
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
    #[serde(default)]
    pub comparison_ready_with: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamAdnaWorkflowV1 {
    pub schema_version: String,
    pub workflow_id: String,
    pub damage_tools: Vec<String>,
    pub authenticity_tools: Vec<String>,
    pub evidence_only: bool,
    pub authenticity_caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDamageEvidenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub terminal_c_to_t_5p: f64,
    pub terminal_g_to_a_3p: f64,
    pub short_fragment_fraction: f64,
    pub damage_signal: String,
    pub strict_profile_upgraded: bool,
    pub advisory_boundary: BamAdvisoryBoundaryV1,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamAuthenticityAdvisoryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub score: f64,
    pub confidence: f64,
    pub pmd_like_signal_present: bool,
    pub advisory_boundary: BamAdvisoryBoundaryV1,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamContaminationEvidenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub scope: String,
    pub tool: String,
    pub prerequisites_passed: bool,
    #[serde(default)]
    pub estimate: Option<f64>,
    #[serde(default)]
    pub ci_low: Option<f64>,
    #[serde(default)]
    pub ci_high: Option<f64>,
    pub advisory_boundary: BamAdvisoryBoundaryV1,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamEndogenousContentEstimateV1 {
    pub schema_version: String,
    pub stage_id: String,
    #[serde(default)]
    pub prealignment_fraction: Option<f64>,
    pub postalignment_fraction: f64,
    pub prealignment_meaning: String,
    pub postalignment_meaning: String,
    #[serde(default)]
    pub caveats: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamContaminationToolContractV1 {
    pub tool_id: String,
    pub supported_scope: String,
    pub required_inputs: Vec<String>,
    pub emits_confidence: bool,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamContaminationWorkflowV1 {
    pub schema_version: String,
    pub workflow_id: String,
    pub tools: Vec<BamContaminationToolContractV1>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamStageResourcePlanV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bytes: u64,
    pub cpu_threads: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
    pub scratch_gb: u32,
    pub walltime_minutes: u32,
    pub io_profile: String,
    pub input_origin: BamInputOriginV1,
    pub input_scale: BamInputScaleV1,
    pub requires_index: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamInputOriginV1 {
    Real,
    Synthetic,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BamInputScaleV1 {
    Tiny,
    Small,
    Medium,
    Large,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BamBenchDatasetScenarioV1 {
    TinyAligned,
    UnmappedHeavy,
    DuplicateHeavy,
    LowCoverage,
    DamageLike,
    ReferenceMismatch,
    MissingIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamBenchCorpusDatasetManifestEntryV1 {
    pub dataset_id: String,
    pub scientific_scope: String,
    pub requires_index: bool,
    pub scenarios: Vec<BamBenchDatasetScenarioV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamBenchCorpusManifestV1 {
    pub schema_version: String,
    pub corpus_id: String,
    pub scenarios_covered: Vec<BamBenchDatasetScenarioV1>,
    pub ci_subset: Vec<String>,
    pub datasets: Vec<BamBenchCorpusDatasetManifestEntryV1>,
}

#[derive(Debug, Clone)]
struct TinySamRecord {
    qname: String,
    flag: u16,
    rname: String,
    pos: u64,
    mapq: u8,
    cigar: String,
    seq: String,
    read_group_id: Option<String>,
}

impl TinySamRecord {
    fn is_mapped(&self) -> bool {
        self.rname != "*" && (self.flag & 0x4) == 0
    }

    fn is_duplicate(&self) -> bool {
        (self.flag & 0x400) != 0
    }
}

#[derive(Debug, Clone, Default)]
struct TinySamDocument {
    sort_order: Option<String>,
    references: Vec<String>,
    reference_lengths: HashMap<String, u64>,
    read_groups: Vec<String>,
    read_group_samples: Vec<String>,
    records: Vec<TinySamRecord>,
}

fn parse_tag_value(field: &str, key: &str) -> Option<String> {
    if !field.starts_with(key) {
        return None;
    }
    let mut iter = field.splitn(3, ':');
    let first = iter.next()?;
    let second = iter.next()?;
    let value = iter.next()?;
    if first == key && second == "Z" {
        Some(value.to_string())
    } else {
        None
    }
}

fn parse_tiny_sam(path: &Path) -> Result<TinySamDocument> {
    let mut document = TinySamDocument::default();
    let payload = std::fs::read_to_string(path)?;
    for (line_index, raw_line) in payload.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('@') {
            let fields = line.split('\t').collect::<Vec<_>>();
            match fields.first().copied() {
                Some("@HD") => {
                    for field in &fields[1..] {
                        if let Some(sort_order) = field.strip_prefix("SO:") {
                            document.sort_order = Some(sort_order.to_string());
                        }
                    }
                }
                Some("@SQ") => {
                    let mut reference_name = None::<String>;
                    let mut reference_length = None::<u64>;
                    for field in &fields[1..] {
                        if let Some(name) = field.strip_prefix("SN:") {
                            reference_name = Some(name.to_string());
                        } else if let Some(length) = field.strip_prefix("LN:") {
                            reference_length = length.parse::<u64>().ok();
                        }
                    }
                    if let Some(name) = reference_name {
                        document.references.push(name.clone());
                        if let Some(length) = reference_length {
                            document.reference_lengths.insert(name, length);
                        }
                    }
                }
                Some("@RG") => {
                    for field in &fields[1..] {
                        if let Some(read_group_id) = field.strip_prefix("ID:") {
                            document.read_groups.push(read_group_id.to_string());
                        } else if let Some(sample_id) = field.strip_prefix("SM:") {
                            document.read_group_samples.push(sample_id.to_string());
                        }
                    }
                }
                _ => {}
            }
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 11 {
            return Err(anyhow!(
                "malformed SAM record at line {}: expected at least 11 fields",
                line_index + 1
            ));
        }
        let flag = fields[1].parse::<u16>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid flag {} ({error})",
                line_index + 1,
                fields[1]
            )
        })?;
        let pos = fields[3].parse::<u64>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid position {} ({error})",
                line_index + 1,
                fields[3]
            )
        })?;
        let mapq = fields[4].parse::<u8>().map_err(|error| {
            anyhow!(
                "malformed SAM record at line {}: invalid MAPQ {} ({error})",
                line_index + 1,
                fields[4]
            )
        })?;
        let mut read_group_id = None;
        for tag in &fields[11..] {
            if let Some(value) = parse_tag_value(tag, "RG") {
                read_group_id = Some(value);
                break;
            }
        }
        document.records.push(TinySamRecord {
            qname: fields[0].to_string(),
            flag,
            rname: fields[2].to_string(),
            pos,
            mapq,
            cigar: fields[5].to_string(),
            seq: fields[9].to_string(),
            read_group_id,
        });
    }
    Ok(document)
}

fn parse_reference_contigs(reference_fasta: &Path) -> Result<BTreeSet<String>> {
    let payload = std::fs::read_to_string(reference_fasta)?;
    let mut contigs = BTreeSet::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            let contig = header.split_whitespace().next().unwrap_or_default().trim();
            if !contig.is_empty() {
                contigs.insert(contig.to_string());
            }
        }
    }
    Ok(contigs)
}

fn flagstat_from_records(records: &[TinySamRecord]) -> BamFlagstatCountsV1 {
    let total_reads = records.len() as u64;
    let mapped_reads = records.iter().filter(|record| record.is_mapped()).count() as u64;
    let duplicate_reads = records.iter().filter(|record| record.is_duplicate()).count() as u64;
    let mapped_fraction =
        if total_reads == 0 { None } else { Some(mapped_reads as f64 / total_reads as f64) };
    BamFlagstatCountsV1 {
        total_reads: Some(total_reads),
        mapped_reads: Some(mapped_reads),
        duplicate_reads: Some(duplicate_reads),
        mapped_fraction,
    }
}

fn push_refusal(refusals: &mut Vec<String>, code: &str) {
    if !refusals.iter().any(|entry| entry == code) {
        refusals.push(code.to_string());
    }
}

/// Execute strict validation for tiny SAM/BAM fixtures and return a governed validation summary.
///
/// Refusal codes are deterministic and cover malformed records, header coherence, read-group/sample
/// identity, reference alignment scope, empty alignments, and sort/index requirements.
///
/// # Errors
/// Returns an error only when reading fixture files fails unexpectedly.
pub fn execute_bam_validation(
    input_bam: &Path,
    bam_index: Option<&Path>,
    reference_fasta: Option<&Path>,
) -> Result<BamValidationSummaryV1> {
    let mut refusal_codes = Vec::new();
    let mut flagstat = BamFlagstatCountsV1 {
        total_reads: None,
        mapped_reads: None,
        duplicate_reads: None,
        mapped_fraction: None,
    };

    if !input_bam.exists() {
        push_refusal(&mut refusal_codes, "input_bam_missing");
    }

    let document = if refusal_codes.is_empty() {
        match parse_tiny_sam(input_bam) {
            Ok(parsed) => {
                flagstat = flagstat_from_records(&parsed.records);
                Some(parsed)
            }
            Err(_) => {
                push_refusal(&mut refusal_codes, "malformed_alignment_record");
                None
            }
        }
    } else {
        None
    };

    if let Some(parsed) = &document {
        if parsed.references.is_empty() {
            push_refusal(&mut refusal_codes, "header_reference_missing");
        }
        if parsed.read_groups.is_empty() {
            push_refusal(&mut refusal_codes, "header_read_group_missing");
        }
        if parsed.read_group_samples.is_empty() {
            push_refusal(&mut refusal_codes, "header_sample_missing");
        } else if parsed.read_group_samples.windows(2).any(|pair| pair[0] != pair[1]) {
            push_refusal(&mut refusal_codes, "read_group_sample_conflict");
        }
        if parsed.records.is_empty() {
            push_refusal(&mut refusal_codes, "empty_alignment_records");
        }
        let header_references = parsed.references.iter().cloned().collect::<BTreeSet<_>>();
        if parsed
            .records
            .iter()
            .filter(|record| record.is_mapped())
            .any(|record| !header_references.contains(&record.rname))
        {
            push_refusal(&mut refusal_codes, "record_reference_missing_from_header");
        }
        if parsed.sort_order.as_deref() != Some("coordinate") {
            push_refusal(&mut refusal_codes, "sort_order_not_coordinate");
        }
        if parsed.sort_order.as_deref() == Some("coordinate") {
            match bam_index {
                Some(index_path) if index_path.exists() => {
                    if std::fs::metadata(index_path)?.len() == 0 {
                        push_refusal(&mut refusal_codes, "bam_index_empty");
                    }
                }
                _ => {
                    push_refusal(&mut refusal_codes, "bam_index_missing");
                }
            }
        }
        if let Some(reference) = reference_fasta {
            let contigs = parse_reference_contigs(reference)?;
            if contigs.is_empty() {
                push_refusal(&mut refusal_codes, "reference_contigs_missing");
            } else if !parsed.references.iter().all(|name| contigs.contains(name)) {
                push_refusal(&mut refusal_codes, "reference_header_mismatch");
            }
        }
    }

    Ok(BamValidationSummaryV1 {
        schema_version: BAM_VALIDATION_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.validate".to_string(),
        input_bam: input_bam.to_path_buf(),
        bam_index: bam_index.map(Path::to_path_buf),
        reference_fasta: reference_fasta.map(Path::to_path_buf),
        flagstat,
        validation_report_present: refusal_codes.is_empty(),
        refusal_codes,
    })
}

#[derive(Debug, Clone)]
struct TinyFastqRead {
    id: String,
    sequence: String,
}

#[derive(Debug, Clone)]
struct TinyReferenceContig {
    name: String,
    sequence: String,
}

#[derive(Debug, Clone)]
struct TinyAlignmentHit {
    reference_name: String,
    position: u64,
    mapq: u8,
    cigar: String,
}

fn parse_tiny_fastq(path: &Path) -> Result<Vec<TinyFastqRead>> {
    let payload = std::fs::read_to_string(path)?;
    let lines = payload.lines().collect::<Vec<_>>();
    if lines.len() % 4 != 0 {
        return Err(anyhow!(
            "FASTQ {} has {} lines, expected a multiple of four",
            path.display(),
            lines.len()
        ));
    }
    let mut reads = Vec::with_capacity(lines.len() / 4);
    for chunk in lines.chunks(4) {
        let header = chunk[0];
        let sequence = chunk[1];
        let plus = chunk[2];
        let quality = chunk[3];
        if !header.starts_with('@') {
            return Err(anyhow!("FASTQ {} has invalid header line: {header}", path.display()));
        }
        if plus != "+" {
            return Err(anyhow!("FASTQ {} has invalid separator line: {plus}", path.display()));
        }
        if sequence.len() != quality.len() {
            return Err(anyhow!(
                "FASTQ {} has sequence/quality length mismatch for read {header}",
                path.display()
            ));
        }
        let id = header
            .trim_start_matches('@')
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_string();
        if id.is_empty() {
            return Err(anyhow!("FASTQ {} has empty read id", path.display()));
        }
        reads.push(TinyFastqRead { id, sequence: sequence.to_string() });
    }
    Ok(reads)
}

fn parse_tiny_reference_fasta(reference_fasta: &Path) -> Result<Vec<TinyReferenceContig>> {
    let payload = std::fs::read_to_string(reference_fasta)?;
    let mut references = Vec::<TinyReferenceContig>::new();
    let mut current_name = String::new();
    let mut current_sequence = String::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            if !current_name.is_empty() {
                references.push(TinyReferenceContig {
                    name: current_name.clone(),
                    sequence: current_sequence.clone(),
                });
            }
            current_name = header.split_whitespace().next().unwrap_or_default().to_string();
            current_sequence.clear();
            continue;
        }
        current_sequence.push_str(line.trim());
    }
    if !current_name.is_empty() {
        references.push(TinyReferenceContig { name: current_name, sequence: current_sequence });
    }
    if references.is_empty() {
        return Err(anyhow!("reference {} has no FASTA contigs", reference_fasta.display()));
    }
    Ok(references)
}

fn bwa_style_hit(
    sequence: &str,
    references: &[TinyReferenceContig],
    seed_length: Option<u32>,
) -> Option<TinyAlignmentHit> {
    let seed = seed_length.unwrap_or(19).max(1) as usize;
    let prefix_len = usize::min(seed, sequence.len());
    let prefix = &sequence[..prefix_len];
    references.iter().find_map(|reference| {
        reference
            .sequence
            .find(prefix)
            .and_then(|seed_position| reference.sequence.find(sequence).or(Some(seed_position)))
            .map(|position| TinyAlignmentHit {
                reference_name: reference.name.clone(),
                position: position as u64 + 1,
                mapq: 60,
                cigar: format!("{}M", sequence.len()),
            })
    })
}

fn bowtie2_style_hit(
    sequence: &str,
    references: &[TinyReferenceContig],
) -> Option<TinyAlignmentHit> {
    let min_anchor = usize::min(sequence.len(), 18);
    if min_anchor == 0 {
        return None;
    }
    references.iter().find_map(|reference| {
        for anchor_len in (min_anchor..=sequence.len()).rev() {
            for start in 0..=(sequence.len() - anchor_len) {
                let anchor = &sequence[start..start + anchor_len];
                if let Some(position) = reference.sequence.find(anchor) {
                    return Some(TinyAlignmentHit {
                        reference_name: reference.name.clone(),
                        position: position as u64 + 1,
                        mapq: if anchor_len == sequence.len() { 45 } else { 32 },
                        cigar: format!("{}M", anchor_len),
                    });
                }
            }
        }
        None
    })
}

#[derive(Debug, Clone, Copy)]
enum TinyAlignBackend {
    BwaStyle,
    Bowtie2Style,
}

fn alignment_hit_for_backend(
    backend: TinyAlignBackend,
    sequence: &str,
    references: &[TinyReferenceContig],
    seed_length: Option<u32>,
) -> Option<TinyAlignmentHit> {
    match backend {
        TinyAlignBackend::BwaStyle => bwa_style_hit(sequence, references, seed_length),
        TinyAlignBackend::Bowtie2Style => bowtie2_style_hit(sequence, references),
    }
}

fn alignment_record_with_flags(
    qname: &str,
    read_group_id: &str,
    hit: Option<&TinyAlignmentHit>,
    paired: bool,
    first_in_pair: bool,
    mate_hit: Option<&TinyAlignmentHit>,
) -> TinySamRecord {
    let mut flag = 0_u16;
    if paired {
        flag |= 0x1;
    }
    if first_in_pair {
        flag |= 0x40;
    } else if paired {
        flag |= 0x80;
    }
    let this_mapped = hit.is_some();
    let mate_mapped = mate_hit.is_some();
    if !this_mapped {
        flag |= 0x4;
    }
    if paired && !mate_mapped {
        flag |= 0x8;
    }
    if paired && this_mapped && mate_mapped {
        if let (Some(this_hit), Some(other_hit)) = (hit, mate_hit) {
            if this_hit.reference_name == other_hit.reference_name {
                flag |= 0x2;
            }
        }
    }
    TinySamRecord {
        qname: qname.to_string(),
        flag,
        rname: hit.map(|result| result.reference_name.clone()).unwrap_or_else(|| "*".to_string()),
        pos: hit.map(|result| result.position).unwrap_or(0),
        mapq: hit.map(|result| result.mapq).unwrap_or(0),
        cigar: hit.map(|result| result.cigar.clone()).unwrap_or_else(|| "*".to_string()),
        seq: "*".to_string(),
        read_group_id: Some(read_group_id.to_string()),
    }
}

fn write_tiny_sam_document(
    output_bam: &Path,
    references: &[TinyReferenceContig],
    read_group: &ReadGroupSpec,
    records: &[TinySamRecord],
) -> Result<()> {
    let mut payload = String::new();
    payload.push_str("@HD\tVN:1.6\tSO:unsorted\n");
    for reference in references {
        payload.push_str(&format!("@SQ\tSN:{}\tLN:{}\n", reference.name, reference.sequence.len()));
    }
    payload.push_str(&format!(
        "@RG\tID:{}\tSM:{}\tPL:{}\tLB:{}",
        read_group.id, read_group.sample, read_group.platform, read_group.library
    ));
    if let Some(unit) = &read_group.platform_unit {
        payload.push_str(&format!("\tPU:{unit}"));
    }
    payload.push('\n');
    for record in records {
        payload.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t*\tRG:Z:{}\n",
            record.qname,
            record.flag,
            record.rname,
            record.pos,
            record.mapq,
            record.cigar,
            record.seq,
            record.read_group_id.as_deref().unwrap_or(&read_group.id)
        ));
    }
    std::fs::write(output_bam, payload)?;
    Ok(())
}

fn write_tiny_sam_from_document(
    output_bam: &Path,
    document: &TinySamDocument,
    sort_order: &str,
) -> Result<()> {
    let mut payload = String::new();
    payload.push_str(&format!("@HD\tVN:1.6\tSO:{sort_order}\n"));
    for reference in &document.references {
        payload.push_str(&format!("@SQ\tSN:{reference}\tLN:1\n"));
    }
    let sample =
        document.read_group_samples.first().cloned().unwrap_or_else(|| "unknown".to_string());
    for read_group in &document.read_groups {
        payload.push_str(&format!("@RG\tID:{read_group}\tSM:{sample}\n"));
    }
    for record in &document.records {
        payload.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t*\t0\t0\t{}\t*\tRG:Z:{}\n",
            record.qname,
            record.flag,
            record.rname,
            record.pos,
            record.mapq,
            record.cigar,
            record.seq,
            record.read_group_id.as_deref().unwrap_or("unknown"),
        ));
    }
    std::fs::write(output_bam, payload)?;
    Ok(())
}

fn mapq_regime_from_records(records: &[TinySamRecord]) -> Option<BamMapqRegimeV1> {
    let mapped = records.iter().filter(|record| record.is_mapped()).collect::<Vec<_>>();
    if mapped.is_empty() {
        return None;
    }
    let mean = mapped.iter().map(|record| record.mapq as f64).sum::<f64>() / mapped.len() as f64;
    let status = if mean < 20.0 {
        "fail"
    } else if mean < 30.0 {
        "warn"
    } else {
        "pass"
    };
    Some(BamMapqRegimeV1 { mean, warn_below: 30.0, fail_below: 20.0, status: status.to_string() })
}

fn align_fastq_to_bam_with_backend(
    backend: TinyAlignBackend,
    backend_tool_id: &str,
    preset: Option<&str>,
    mode: &str,
    sensitivity_profile: Option<&str>,
    seed_length: Option<u32>,
    reference_fasta: &Path,
    reads_r1: &Path,
    reads_r2: Option<&Path>,
    output_root: &Path,
    sample_id: &str,
    read_group: &ReadGroupSpec,
) -> Result<(BamAlignmentProvenanceV1, BamMappingSummaryV1)> {
    std::fs::create_dir_all(output_root)?;
    let references = parse_tiny_reference_fasta(reference_fasta)?;
    let r1_reads = parse_tiny_fastq(reads_r1)?;
    let r2_reads =
        if let Some(r2_path) = reads_r2 { Some(parse_tiny_fastq(r2_path)?) } else { None };

    if let Some(right) = &r2_reads {
        if right.len() != r1_reads.len() {
            return Err(anyhow!(
                "paired FASTQ counts differ: R1 has {} reads and R2 has {} reads",
                r1_reads.len(),
                right.len()
            ));
        }
    }

    let paired = r2_reads.is_some();
    let mut records = Vec::<TinySamRecord>::new();
    for (index, left) in r1_reads.iter().enumerate() {
        let right = r2_reads.as_ref().and_then(|reads| reads.get(index));
        let left_hit = alignment_hit_for_backend(backend, &left.sequence, &references, seed_length);
        let right_hit = right
            .map(|read| {
                alignment_hit_for_backend(backend, &read.sequence, &references, seed_length)
            })
            .unwrap_or(None);
        records.push(alignment_record_with_flags(
            &left.id,
            &read_group.id,
            left_hit.as_ref(),
            paired,
            true,
            right_hit.as_ref(),
        ));
        if let Some(right_read) = right {
            let qname =
                if right_read.id.is_empty() { left.id.as_str() } else { right_read.id.as_str() };
            records.push(alignment_record_with_flags(
                qname,
                &read_group.id,
                right_hit.as_ref(),
                true,
                false,
                left_hit.as_ref(),
            ));
        }
    }

    let output_bam = output_root.join("align.bam");
    let output_bai = output_root.join("align.bam.bai");
    write_tiny_sam_document(&output_bam, &references, read_group, &records)?;
    std::fs::write(
        &output_bai,
        format!(
            "tiny-bai\tmapped:{}\ttotal:{}\n",
            records.iter().filter(|record| record.is_mapped()).count(),
            records.len()
        ),
    )?;

    let outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("align_bam"),
            output_bam.clone(),
            ArtifactRole::Bam,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("align_bai"),
            output_bai.clone(),
            ArtifactRole::Index,
        ),
    ];
    let output_inventory = bam_artifact_inventory_from_outputs("bam.align", output_root, &outputs);
    let sample_identity = bam_sample_identity(
        sample_id,
        read_group,
        Some("preserve"),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let strategy = bam_alignment_strategy_for_tool(backend_tool_id, preset);
    let post_alignment_chain = strategy
        .as_ref()
        .map(|entry| entry.post_alignment_chain.clone())
        .or_else(|| bam_post_alignment_chain("samtools_coordinate_validate"))
        .ok_or_else(|| anyhow!("post-alignment chain samtools_coordinate_validate is undefined"))?;
    let provenance = BamAlignmentProvenanceV1 {
        schema_version: BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION.to_string(),
        stage_id: "bam.align".to_string(),
        backend_tool_id: backend_tool_id.to_string(),
        strategy_id: strategy.as_ref().map(|entry| entry.strategy_id.clone()),
        preset: preset
            .map(ToOwned::to_owned)
            .or_else(|| strategy.as_ref().map(|entry| entry.default_preset.clone())),
        mode: Some(mode.to_string()),
        sensitivity_profile: sensitivity_profile.map(ToOwned::to_owned),
        seed_length,
        reference_fasta: reference_fasta.to_path_buf(),
        reference_digest: None,
        post_alignment_chain: Some(post_alignment_chain),
        sample_identity,
        read_group: read_group.clone(),
        outputs: output_inventory,
    };

    let mapping_summary = BamMappingSummaryV1 {
        schema_version: BAM_MAPPING_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.mapping_summary".to_string(),
        flagstat: flagstat_from_records(&records),
        stats_present: true,
        idxstats_present: true,
        mapq_regime: mapq_regime_from_records(&records),
        proper_pair_reads: Some(
            records.iter().filter(|record| (record.flag & 0x2) != 0).count() as u64
        ),
        secondary_reads: Some(
            records.iter().filter(|record| (record.flag & 0x100) != 0).count() as u64
        ),
        supplementary_reads: Some(
            records.iter().filter(|record| (record.flag & 0x800) != 0).count() as u64,
        ),
        mapq_histogram: Vec::new(),
        read_group_breakdown: Vec::new(),
    };

    Ok((provenance, mapping_summary))
}

/// Align tiny FASTQ fixtures to a tiny reference with BWA-style semantics.
///
/// # Errors
/// Returns an error when FASTQ/FASTA inputs are malformed or outputs cannot be written.
pub fn align_fastq_to_bam_bwa_style(
    reference_fasta: &Path,
    reads_r1: &Path,
    reads_r2: Option<&Path>,
    output_root: &Path,
    sample_id: &str,
    read_group: &ReadGroupSpec,
    preset: Option<&str>,
    seed_length: Option<u32>,
) -> Result<(BamAlignmentProvenanceV1, BamMappingSummaryV1)> {
    align_fastq_to_bam_with_backend(
        TinyAlignBackend::BwaStyle,
        "bwa",
        preset,
        "end_to_end",
        None,
        seed_length,
        reference_fasta,
        reads_r1,
        reads_r2,
        output_root,
        sample_id,
        read_group,
    )
}

/// Align tiny FASTQ fixtures to a tiny reference with Bowtie2-style local sensitivity semantics.
///
/// # Errors
/// Returns an error when FASTQ/FASTA inputs are malformed or outputs cannot be written.
pub fn align_fastq_to_bam_bowtie2_style(
    reference_fasta: &Path,
    reads_r1: &Path,
    reads_r2: Option<&Path>,
    output_root: &Path,
    sample_id: &str,
    read_group: &ReadGroupSpec,
    sensitivity_profile: Option<&str>,
) -> Result<(BamAlignmentProvenanceV1, BamMappingSummaryV1)> {
    align_fastq_to_bam_with_backend(
        TinyAlignBackend::Bowtie2Style,
        "bowtie2",
        Some("default"),
        "local",
        sensitivity_profile,
        None,
        reference_fasta,
        reads_r1,
        reads_r2,
        output_root,
        sample_id,
        read_group,
    )
}

/// Sort tiny SAM/BAM fixtures by coordinate and emit a deterministic index sidecar.
///
/// # Errors
/// Returns an error if input parsing or output materialization fails.
pub fn sort_and_index_tiny_bam(
    input_bam: &Path,
    output_bam: &Path,
    output_bai: &Path,
) -> Result<BamArtifactInventoryV1> {
    let mut document = parse_tiny_sam(input_bam)?;
    let mut reference_rank = HashMap::<String, usize>::new();
    for (index, reference) in document.references.iter().enumerate() {
        reference_rank.insert(reference.clone(), index);
    }
    let fallback_rank = reference_rank.len() + 1;
    document.records.sort_by(|left, right| {
        let left_rank = if left.is_mapped() {
            *reference_rank.get(&left.rname).unwrap_or(&fallback_rank)
        } else {
            fallback_rank
        };
        let right_rank = if right.is_mapped() {
            *reference_rank.get(&right.rname).unwrap_or(&fallback_rank)
        } else {
            fallback_rank
        };
        (left_rank, left.pos, left.qname.as_str()).cmp(&(
            right_rank,
            right.pos,
            right.qname.as_str(),
        ))
    });
    write_tiny_sam_from_document(output_bam, &document, "coordinate")?;

    let mut mapped_by_reference = HashMap::<String, u64>::new();
    for record in document.records.iter().filter(|entry| entry.is_mapped()) {
        *mapped_by_reference.entry(record.rname.clone()).or_insert(0) += 1;
    }
    let mut index_payload = String::new();
    for reference in &document.references {
        let count = mapped_by_reference.get(reference).copied().unwrap_or(0);
        index_payload.push_str(&format!("{reference}\t{count}\n"));
    }
    std::fs::write(output_bai, index_payload)?;

    let output_root = output_bam.parent().unwrap_or_else(|| Path::new("."));
    let outputs = vec![
        ArtifactRef::required(
            ArtifactId::from_static("sorted_bam"),
            output_bam.to_path_buf(),
            ArtifactRole::Bam,
        ),
        ArtifactRef::required(
            ArtifactId::from_static("sorted_bai"),
            output_bai.to_path_buf(),
            ArtifactRole::Index,
        ),
    ];
    Ok(bam_artifact_inventory_from_outputs("bam.sort_index", output_root, &outputs))
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
    let lane = lane_id.map(ToOwned::to_owned).or_else(|| read_group.lane_id.clone());
    let library = library_id.map(ToOwned::to_owned).or_else(|| read_group.library_id());
    let platform = Some(read_group.platform.clone());
    let platform_unit =
        platform_unit.map(ToOwned::to_owned).or_else(|| read_group.platform_unit.clone());
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

/// Propagate BAM sample identity through a stage while preserving declared metadata lineage.
#[must_use]
pub fn propagate_bam_sample_identity(
    prior: &BamSampleIdentityV1,
    read_group: &ReadGroupSpec,
    stage_id: &str,
) -> BamSampleIdentityV1 {
    let mut read_group_ids = prior.read_group_ids.clone();
    if !read_group_ids.iter().any(|id| id == &read_group.id) {
        read_group_ids.push(read_group.id.clone());
        read_group_ids.sort_unstable();
    }
    BamSampleIdentityV1 {
        schema_version: BAM_SAMPLE_IDENTITY_SCHEMA_VERSION.to_string(),
        sample_id: prior.sample_id.clone(),
        lane_id: prior.lane_id.clone().or_else(|| read_group.lane_id.clone()),
        library_id: prior.library_id.clone().or_else(|| read_group.library_id()),
        platform: prior.platform.clone().or_else(|| Some(read_group.platform.clone())),
        platform_unit: prior.platform_unit.clone().or_else(|| read_group.platform_unit.clone()),
        run_id: prior.run_id.clone().or_else(|| read_group.run_id.clone()),
        subject_id: prior.subject_id.clone(),
        cohort_id: prior.cohort_id.clone(),
        read_group_policy: Some(format!("propagate:{stage_id}")),
        read_group_ids,
    }
}

#[must_use]
pub fn bam_post_alignment_chain(chain_id: &str) -> Option<BamPostAlignmentChainV1> {
    match chain_id {
        "samtools_coordinate_validate" => Some(BamPostAlignmentChainV1 {
            schema_version: BAM_POST_ALIGNMENT_CHAIN_SCHEMA_VERSION.to_string(),
            chain_id: chain_id.to_string(),
            sorting_mode: "coordinate".to_string(),
            index_type: "bai".to_string(),
            validate_before_downstream: true,
            coordinate_sorted_required_for: vec![
                "bam.markdup".to_string(),
                "bam.duplication_metrics".to_string(),
                "bam.coverage".to_string(),
                "bam.damage".to_string(),
                "bam.contamination".to_string(),
            ],
            downstream_refusal_stages: vec![
                "bam.coverage".to_string(),
                "bam.damage".to_string(),
                "bam.contamination".to_string(),
                "bam.kinship".to_string(),
            ],
            required_tools: vec![
                "samtools sort".to_string(),
                "samtools index".to_string(),
                "samtools quickcheck".to_string(),
            ],
        }),
        _ => None,
    }
}

#[must_use]
pub fn bam_alignment_strategies() -> Vec<BamAlignmentStrategyV1> {
    let chain = bam_post_alignment_chain("samtools_coordinate_validate")
        .expect("samtools post-alignment chain");
    vec![
        BamAlignmentStrategyV1 {
            schema_version: BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION.to_string(),
            strategy_id: "bwa_mem_default".to_string(),
            tool_id: "bwa".to_string(),
            default_preset: "default".to_string(),
            mode: "end_to_end".to_string(),
            supported_inputs: vec!["single_end".to_string(), "paired_end".to_string()],
            required_reference_assets: vec![
                "reference_fasta".to_string(),
                "reference_fai".to_string(),
                "reference_dict".to_string(),
                "bwa_index".to_string(),
            ],
            suitability: vec![
                BamAlignmentSuitabilityV1::GeneralShortRead,
                BamAlignmentSuitabilityV1::WholeGenomeLike,
                BamAlignmentSuitabilityV1::TargetedCapture,
            ],
            hidden_default_allowed: false,
            post_alignment_chain: chain.clone(),
        },
        BamAlignmentStrategyV1 {
            schema_version: BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION.to_string(),
            strategy_id: "bwa_aln_adna_short".to_string(),
            tool_id: "bwa".to_string(),
            default_preset: "adna_short".to_string(),
            mode: "seeded_short_read".to_string(),
            supported_inputs: vec!["single_end".to_string(), "paired_end".to_string()],
            required_reference_assets: vec![
                "reference_fasta".to_string(),
                "reference_fai".to_string(),
                "reference_dict".to_string(),
                "bwa_index".to_string(),
            ],
            suitability: vec![
                BamAlignmentSuitabilityV1::AncientShortFragments,
                BamAlignmentSuitabilityV1::SensitiveLocalRescue,
            ],
            hidden_default_allowed: false,
            post_alignment_chain: chain.clone(),
        },
        BamAlignmentStrategyV1 {
            schema_version: BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION.to_string(),
            strategy_id: "bowtie2_very_sensitive_local".to_string(),
            tool_id: "bowtie2".to_string(),
            default_preset: "default".to_string(),
            mode: "local".to_string(),
            supported_inputs: vec!["single_end".to_string(), "paired_end".to_string()],
            required_reference_assets: vec![
                "reference_fasta".to_string(),
                "reference_fai".to_string(),
                "reference_dict".to_string(),
                "bowtie2_index".to_string(),
            ],
            suitability: vec![
                BamAlignmentSuitabilityV1::GeneralShortRead,
                BamAlignmentSuitabilityV1::SensitiveLocalRescue,
                BamAlignmentSuitabilityV1::TargetedCapture,
            ],
            hidden_default_allowed: false,
            post_alignment_chain: chain,
        },
    ]
}

#[must_use]
pub fn bam_alignment_strategy_for_tool(
    tool_id: &str,
    preset: Option<&str>,
) -> Option<BamAlignmentStrategyV1> {
    let preset = preset.unwrap_or("default");
    bam_alignment_strategies().into_iter().find(|strategy| {
        match (tool_id, strategy.tool_id.as_str()) {
            ("bwa", "bwa") if preset == "adna_short" => {
                strategy.strategy_id == "bwa_aln_adna_short"
            }
            ("bwa", "bwa") => strategy.strategy_id == "bwa_mem_default",
            ("bowtie2", "bowtie2") => strategy.strategy_id == "bowtie2_very_sensitive_local",
            _ => false,
        }
    })
}

#[must_use]
pub fn evaluate_bam_merge_compatibility(
    inputs: &[BamMergeInputIdentityV1],
) -> BamMergeCompatibilityV1 {
    let mut refusal_codes = Vec::new();
    let mut notes = Vec::new();
    if inputs.is_empty() {
        refusal_codes.push("merge_requires_at_least_one_input".to_string());
    }
    let sample_ids = inputs.iter().map(|input| input.sample_id.as_str()).collect::<Vec<_>>();
    if sample_ids.windows(2).any(|pair| pair[0] != pair[1]) {
        refusal_codes.push("merge_sample_id_conflict".to_string());
    }
    let reference_digests =
        inputs.iter().filter_map(|input| input.reference_digest.as_deref()).collect::<Vec<_>>();
    if reference_digests.windows(2).any(|pair| pair[0] != pair[1]) {
        refusal_codes.push("merge_reference_digest_conflict".to_string());
    }
    let platforms =
        inputs.iter().filter_map(|input| input.sequencing_platform.as_deref()).collect::<Vec<_>>();
    if platforms.windows(2).any(|pair| pair[0] != pair[1]) {
        refusal_codes.push("merge_platform_conflict".to_string());
    }
    let libraries =
        inputs.iter().filter_map(|input| input.library_id.as_deref()).collect::<Vec<_>>();
    if libraries.windows(2).any(|pair| pair[0] != pair[1]) {
        refusal_codes.push("merge_library_conflict".to_string());
    }
    let read_groups_present = inputs.iter().all(|input| !input.read_group_ids.is_empty());
    if !read_groups_present {
        refusal_codes.push("merge_read_group_missing".to_string());
    } else {
        notes.push("read groups preserved per compatible lane/library input".to_string());
    }
    BamMergeCompatibilityV1 {
        schema_version: BAM_MERGE_COMPATIBILITY_SCHEMA_VERSION.to_string(),
        compatible: refusal_codes.is_empty(),
        merged_sample_id: inputs.first().map(|input| input.sample_id.clone()),
        refusal_codes,
        notes,
    }
}

/// Merge tiny BAM/SAM fixtures only when compatibility proof passes.
///
/// # Errors
/// Returns an error when input files cannot be read or output cannot be written.
pub fn merge_tiny_bam_with_conflict_refusal(
    input_bams: &[PathBuf],
    input_identities: &[BamMergeInputIdentityV1],
    output_bam: &Path,
) -> Result<BamMergeCompatibilityV1> {
    if input_bams.len() != input_identities.len() {
        return Err(anyhow!(
            "merge requires one identity per input BAM ({} BAMs, {} identities)",
            input_bams.len(),
            input_identities.len()
        ));
    }
    let mut compatibility = evaluate_bam_merge_compatibility(input_identities);
    if !compatibility.compatible {
        compatibility.notes.push("merge output refused due to compatibility conflicts".to_string());
        return Ok(compatibility);
    }

    let mut merged = TinySamDocument {
        sort_order: Some("coordinate".to_string()),
        ..TinySamDocument::default()
    };
    for input in input_bams {
        let document = parse_tiny_sam(input)?;
        for reference in &document.references {
            if !merged.references.iter().any(|entry| entry == reference) {
                merged.references.push(reference.clone());
            }
        }
        for read_group in &document.read_groups {
            if !merged.read_groups.iter().any(|entry| entry == read_group) {
                merged.read_groups.push(read_group.clone());
            }
        }
        for sample in &document.read_group_samples {
            if !merged.read_group_samples.iter().any(|entry| entry == sample) {
                merged.read_group_samples.push(sample.clone());
            }
        }
        merged.records.extend(document.records);
    }

    let mut reference_rank = HashMap::<String, usize>::new();
    for (index, reference) in merged.references.iter().enumerate() {
        reference_rank.insert(reference.clone(), index);
    }
    let fallback_rank = reference_rank.len() + 1;
    merged.records.sort_by(|left, right| {
        let left_rank = if left.is_mapped() {
            *reference_rank.get(&left.rname).unwrap_or(&fallback_rank)
        } else {
            fallback_rank
        };
        let right_rank = if right.is_mapped() {
            *reference_rank.get(&right.rname).unwrap_or(&fallback_rank)
        } else {
            fallback_rank
        };
        (left_rank, left.pos, left.qname.as_str()).cmp(&(
            right_rank,
            right.pos,
            right.qname.as_str(),
        ))
    });
    write_tiny_sam_from_document(output_bam, &merged, "coordinate")?;

    compatibility.notes.push("merge output materialized".to_string());
    Ok(compatibility)
}

/// Apply duplicate policy in mark/remove modes and emit explicit policy/comparison reports.
///
/// # Errors
/// Returns an error if input parsing fails, output writing fails, or action is unsupported.
pub fn apply_duplicate_policy_tiny_bam(
    input_bam: &Path,
    output_bam: &Path,
    duplicate_action: &str,
    umi_policy: Option<&str>,
) -> Result<(BamDuplicatePolicyV1, BamDuplicateComparisonV1)> {
    let input = parse_tiny_sam(input_bam)?;
    let mut observed = HashMap::<String, usize>::new();
    let mut output_records = Vec::<TinySamRecord>::new();
    let mut additional_marked = 0_u64;

    for mut record in input.records.clone() {
        if !record.is_mapped() {
            output_records.push(record);
            continue;
        }
        let key = format!("{}:{}:{}:{}", record.rname, record.pos, record.cigar, record.seq);
        let seen = observed.get(&key).copied().unwrap_or(0);
        observed.insert(key, seen + 1);
        if seen == 0 {
            output_records.push(record);
            continue;
        }
        match duplicate_action {
            "mark" => {
                if (record.flag & 0x400) == 0 {
                    record.flag |= 0x400;
                    additional_marked += 1;
                }
                output_records.push(record);
            }
            "remove" => {}
            other => {
                return Err(anyhow!(
                    "unsupported duplicate action {other}; expected mark or remove"
                ));
            }
        }
    }

    let output = TinySamDocument {
        sort_order: input.sort_order.clone(),
        references: input.references.clone(),
        reference_lengths: input.reference_lengths.clone(),
        read_groups: input.read_groups.clone(),
        read_group_samples: input.read_group_samples.clone(),
        records: output_records.clone(),
    };
    write_tiny_sam_from_document(
        output_bam,
        &output,
        input.sort_order.as_deref().unwrap_or("unsorted"),
    )?;

    let before = flagstat_from_records(&input.records);
    let after = flagstat_from_records(&output_records);
    let method_a = BamDuplicateMethodMetricsV1 {
        method: "before_policy".to_string(),
        duplicate_reads: before.duplicate_reads,
        duplicate_fraction: match (before.duplicate_reads, before.total_reads) {
            (Some(duplicates), Some(total)) if total > 0 => Some(duplicates as f64 / total as f64),
            _ => None,
        },
        optical_duplicates: Some("not_classified".to_string()),
        duplicate_action: Some("observe".to_string()),
    };
    let method_b = BamDuplicateMethodMetricsV1 {
        method: format!("after_{duplicate_action}"),
        duplicate_reads: after.duplicate_reads,
        duplicate_fraction: match (after.duplicate_reads, after.total_reads) {
            (Some(duplicates), Some(total)) if total > 0 => Some(duplicates as f64 / total as f64),
            _ => None,
        },
        optical_duplicates: Some("not_classified".to_string()),
        duplicate_action: Some(duplicate_action.to_string()),
    };

    let mut comparison = compare_bam_duplicate_methods("bam.markdup", method_a, method_b);
    if additional_marked > 0 {
        comparison.notes.push(format!("marked {additional_marked} additional duplicate reads"));
    }
    if duplicate_action == "remove" {
        comparison.notes.push("duplicate reads were removed from output alignment".to_string());
    }

    let policy = BamDuplicatePolicyV1 {
        schema_version: BAM_DUPLICATE_POLICY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.markdup".to_string(),
        library_type: None,
        optical_duplicates: Some("mark_only".to_string()),
        umi_policy: umi_policy.map(ToOwned::to_owned),
        duplicate_action: Some(duplicate_action.to_string()),
        policy_scope: if duplicate_action == "remove" {
            "remove_duplicate_reads".to_string()
        } else {
            "mark_duplicates".to_string()
        },
        library_semantics: vec!["coordinate_sorted_recommended".to_string()],
        comparison_ready_with: vec![
            "samtools_markdup".to_string(),
            "picard_markduplicates".to_string(),
        ],
    };

    Ok((policy, comparison))
}

/// Filter tiny BAM/SAM fixtures by MAPQ and emit retained/removed evidence.
///
/// # Errors
/// Returns an error if input parsing fails or output writing fails.
pub fn filter_tiny_bam_by_mapq(
    input_bam: &Path,
    output_bam: &Path,
    mapq_threshold: u8,
) -> Result<BamMapqFilterSummaryV1> {
    let input = parse_tiny_sam(input_bam)?;
    let before = flagstat_from_records(&input.records);
    let filtered_records = input
        .records
        .iter()
        .filter(|record| !record.is_mapped() || record.mapq >= mapq_threshold)
        .cloned()
        .collect::<Vec<_>>();
    let output = TinySamDocument {
        sort_order: input.sort_order.clone(),
        references: input.references.clone(),
        reference_lengths: input.reference_lengths.clone(),
        read_groups: input.read_groups.clone(),
        read_group_samples: input.read_group_samples.clone(),
        records: filtered_records.clone(),
    };
    write_tiny_sam_from_document(
        output_bam,
        &output,
        input.sort_order.as_deref().unwrap_or("unsorted"),
    )?;
    let after = flagstat_from_records(&filtered_records);
    let mapped_reads_removed = match (before.mapped_reads, after.mapped_reads) {
        (Some(before_mapped), Some(after_mapped)) if before_mapped >= after_mapped => {
            Some(before_mapped - after_mapped)
        }
        _ => None,
    };
    let mapped_fraction_retained = match (before.mapped_reads, after.mapped_reads) {
        (Some(before_mapped), Some(after_mapped)) if before_mapped > 0 => {
            Some(after_mapped as f64 / before_mapped as f64)
        }
        _ => None,
    };
    Ok(BamMapqFilterSummaryV1 {
        schema_version: BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.mapq_filter".to_string(),
        mapq_threshold,
        input_bam: input_bam.to_path_buf(),
        output_bam: output_bam.to_path_buf(),
        flagstat_before: before,
        flagstat_after: after,
        mapped_reads_removed,
        mapped_fraction_retained,
    })
}

/// Summarize mapping status for a tiny BAM/SAM fixture with MAPQ and read-group breakdown.
///
/// # Errors
/// Returns an error if the input cannot be parsed.
pub fn summarize_tiny_bam_mapping(input_bam: &Path) -> Result<BamMappingSummaryV1> {
    let document = parse_tiny_sam(input_bam)?;
    let flagstat = flagstat_from_records(&document.records);
    let proper_pair_reads =
        document.records.iter().filter(|record| (record.flag & 0x2) != 0).count() as u64;
    let secondary_reads =
        document.records.iter().filter(|record| (record.flag & 0x100) != 0).count() as u64;
    let supplementary_reads =
        document.records.iter().filter(|record| (record.flag & 0x800) != 0).count() as u64;

    let mut mapq_histogram_counts = HashMap::<u8, u64>::new();
    for record in document.records.iter().filter(|record| record.is_mapped()) {
        *mapq_histogram_counts.entry(record.mapq).or_insert(0) += 1;
    }
    let mut mapq_histogram = mapq_histogram_counts.into_iter().collect::<Vec<_>>();
    mapq_histogram.sort_by_key(|(mapq, _)| *mapq);

    let mut read_group_counts = HashMap::<String, (u64, u64, u64)>::new();
    for record in &document.records {
        let read_group = record.read_group_id.clone().unwrap_or_else(|| "unknown".to_string());
        let entry = read_group_counts.entry(read_group).or_insert((0, 0, 0));
        entry.0 += 1;
        if record.is_mapped() {
            entry.1 += 1;
        }
        if (record.flag & 0x2) != 0 {
            entry.2 += 1;
        }
    }
    let mut read_group_breakdown = read_group_counts
        .into_iter()
        .map(|(read_group_id, (total_reads, mapped_reads, proper_pair_reads))| {
            BamReadGroupMappingCountV1 {
                read_group_id,
                total_reads,
                mapped_reads,
                proper_pair_reads,
            }
        })
        .collect::<Vec<_>>();
    read_group_breakdown.sort_by(|left, right| left.read_group_id.cmp(&right.read_group_id));

    Ok(BamMappingSummaryV1 {
        schema_version: BAM_MAPPING_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.mapping_summary".to_string(),
        flagstat,
        stats_present: true,
        idxstats_present: true,
        mapq_regime: mapq_regime_from_records(&document.records),
        proper_pair_reads: Some(proper_pair_reads),
        secondary_reads: Some(secondary_reads),
        supplementary_reads: Some(supplementary_reads),
        mapq_histogram,
        read_group_breakdown,
    })
}

/// Summarize tiny BAM/SAM coverage and classify coverage regime from depth/breadth signals.
///
/// # Errors
/// Returns an error if input parsing fails.
pub fn summarize_tiny_bam_coverage(
    input_bam: &Path,
    depth_thresholds: &[u32],
) -> Result<BamCoverageSummaryV1> {
    let document = parse_tiny_sam(input_bam)?;
    let mut inferred_lengths = HashMap::<String, u64>::new();
    for record in document.records.iter().filter(|record| record.is_mapped()) {
        let end = record.pos + u64::max(record.seq.len() as u64, 1) - 1;
        let current = inferred_lengths.entry(record.rname.clone()).or_insert(0);
        *current = (*current).max(end);
    }

    let mut coverage_vectors = HashMap::<String, Vec<u32>>::new();
    for reference in &document.references {
        let declared = document.reference_lengths.get(reference).copied().unwrap_or(0);
        let inferred = inferred_lengths.get(reference).copied().unwrap_or(0);
        let length = declared.max(inferred).max(1) as usize;
        coverage_vectors.insert(reference.clone(), vec![0; length]);
    }

    for record in document.records.iter().filter(|record| record.is_mapped()) {
        let Some(depths) = coverage_vectors.get_mut(&record.rname) else {
            continue;
        };
        let start = record.pos.saturating_sub(1) as usize;
        let span = usize::max(record.seq.len(), 1);
        let end = usize::min(start.saturating_add(span), depths.len());
        for depth in depths.iter_mut().take(end).skip(start) {
            *depth += 1;
        }
    }

    let mut total_positions = 0_u64;
    let mut covered_positions = 0_u64;
    let mut depth_sum = 0_u64;
    for depths in coverage_vectors.values() {
        total_positions += depths.len() as u64;
        for depth in depths {
            depth_sum += *depth as u64;
            if *depth >= 1 {
                covered_positions += 1;
            }
        }
    }

    let mean_depth =
        if total_positions > 0 { depth_sum as f64 / total_positions as f64 } else { 0.0 };
    let breadth_1x =
        if total_positions > 0 { covered_positions as f64 / total_positions as f64 } else { 0.0 };
    let regime = classify_bam_coverage_regime(mean_depth, breadth_1x);

    Ok(BamCoverageSummaryV1 {
        schema_version: BAM_COVERAGE_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.coverage".to_string(),
        has_mosdepth_summary: false,
        has_samtools_depth: true,
        mean_depth: Some(mean_depth),
        coverage_regime: Some(regime.regime_id.clone()),
        coverage_family: Some(regime.enforced_label.clone()),
        regime: Some(regime),
        depth_thresholds: depth_thresholds.to_vec(),
    })
}

#[must_use]
pub fn compare_bam_duplicate_methods(
    stage_id: &str,
    method_a: BamDuplicateMethodMetricsV1,
    method_b: BamDuplicateMethodMetricsV1,
) -> BamDuplicateComparisonV1 {
    let duplicate_reads_delta = match (method_a.duplicate_reads, method_b.duplicate_reads) {
        (Some(a), Some(b)) => Some(a as i64 - b as i64),
        _ => None,
    };
    let duplicate_fraction_delta = match (method_a.duplicate_fraction, method_b.duplicate_fraction)
    {
        (Some(a), Some(b)) => Some(a - b),
        _ => None,
    };
    let policy_explicit =
        method_a.duplicate_action.is_some() && method_b.duplicate_action.is_some();
    let comparable = duplicate_reads_delta.is_some() || duplicate_fraction_delta.is_some();
    let mut notes = Vec::new();
    if !policy_explicit {
        notes.push("duplicate action must stay explicit for cross-tool comparison".to_string());
    }
    BamDuplicateComparisonV1 {
        schema_version: BAM_DUPLICATE_COMPARISON_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        method_a,
        method_b,
        comparable,
        duplicate_reads_delta,
        duplicate_fraction_delta,
        policy_explicit,
        notes,
    }
}

#[must_use]
pub fn classify_bam_coverage_regime(mean_depth: f64, breadth_1x: f64) -> BamCoverageRegimeV1 {
    let (regime_id, advisory_label, enforced_label, regime_class, usable_for, caveats) =
        if mean_depth < 0.2 || breadth_1x < 0.1 {
            (
                "unusable",
                "insufficient_signal",
                "refuse_downstream",
                BamCoverageRegimeClassV1::Unusable,
                Vec::new(),
                vec!["coverage is too low for reliable downstream interpretation".to_string()],
            )
        } else if mean_depth < 1.0 {
            (
                "sparse",
                "sparse_signal",
                "advisory_only",
                BamCoverageRegimeClassV1::Sparse,
                vec!["damage_screening".to_string()],
                vec!["site-level interpretation is unstable".to_string()],
            )
        } else if mean_depth < 5.0 {
            (
                "low_pass",
                "low_pass_signal",
                "guardrail_required",
                BamCoverageRegimeClassV1::LowPass,
                vec![
                    "damage_screening".to_string(),
                    "contamination_screening".to_string(),
                    "sex_screening".to_string(),
                ],
                vec!["population-scale inference requires explicit caveats".to_string()],
            )
        } else if breadth_1x >= 0.8 && mean_depth >= 15.0 {
            (
                "whole_genome_like",
                "whole_genome_like",
                "fully_supported",
                BamCoverageRegimeClassV1::WholeGenomeLike,
                vec![
                    "coverage_reporting".to_string(),
                    "variant_readiness".to_string(),
                    "kinship".to_string(),
                ],
                Vec::new(),
            )
        } else {
            (
                "target_like",
                "target_like_signal",
                "panel_specific_support",
                BamCoverageRegimeClassV1::TargetLike,
                vec![
                    "coverage_reporting".to_string(),
                    "haplogroups".to_string(),
                    "targeted_qc".to_string(),
                ],
                vec!["whole-genome-style uniformity assumptions do not hold".to_string()],
            )
        };
    BamCoverageRegimeV1 {
        schema_version: BAM_COVERAGE_REGIME_SCHEMA_VERSION.to_string(),
        regime_id: regime_id.to_string(),
        advisory_label: advisory_label.to_string(),
        enforced_label: enforced_label.to_string(),
        regime_class,
        mean_depth,
        breadth_1x,
        usable_for,
        caveats,
    }
}

/// Produce ancient-DNA damage evidence with explicit advisory boundary semantics.
#[must_use]
pub fn execute_ancient_damage_evidence(
    metrics: &BamMetricsV1,
    strict_profile: bool,
) -> BamDamageEvidenceV1 {
    let terminal_damage = metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p);
    let damage_signal = if terminal_damage >= 0.20 {
        "high"
    } else if terminal_damage >= 0.10 {
        "moderate"
    } else {
        "low"
    };
    let strict_upgrade =
        strict_profile && terminal_damage >= 0.10 && metrics.fragment_length.short_fraction >= 0.20;
    let advisory_boundary = BamAdvisoryBoundaryV1 {
        schema_version: BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.damage".to_string(),
        advisory_only: !strict_upgrade,
        scientific_scope: if strict_upgrade {
            "strict_damage_profile_with_context"
        } else {
            "damage_evidence_only"
        }
        .to_string(),
        evidence_inputs: vec![
            "terminal_damage_rates".to_string(),
            "fragment_length_distribution".to_string(),
            "mapq_profile".to_string(),
        ],
        safe_for_claims: if strict_upgrade {
            vec!["damage_signal_present".to_string(), "damage_profile_consistent".to_string()]
        } else {
            vec!["damage_signal_present".to_string()]
        },
        unsafe_for_claims: vec![
            "authenticity_certification".to_string(),
            "contamination_absence".to_string(),
        ],
    };
    let mut notes = vec![
        "damage evidence is contextual and requires contamination/capture/library interpretation"
            .to_string(),
    ];
    if strict_upgrade {
        notes.push(
            "strict profile upgraded this stage from advisory-only to guarded evidence status"
                .to_string(),
        );
    }
    BamDamageEvidenceV1 {
        schema_version: BAM_DAMAGE_EVIDENCE_SCHEMA_VERSION.to_string(),
        stage_id: "bam.damage".to_string(),
        terminal_c_to_t_5p: metrics.damage.c_to_t_5p,
        terminal_g_to_a_3p: metrics.damage.g_to_a_3p,
        short_fragment_fraction: metrics.fragment_length.short_fraction,
        damage_signal: damage_signal.to_string(),
        strict_profile_upgraded: strict_upgrade,
        advisory_boundary,
        notes,
    }
}

/// Execute PMD/authenticity interpretation as advisory evidence and never as certification.
#[must_use]
pub fn execute_pmd_authenticity_advisory(metrics: &BamMetricsV1) -> BamAuthenticityAdvisoryV1 {
    let authenticity = authenticity_score(metrics);
    let advisory_boundary = BamAdvisoryBoundaryV1 {
        schema_version: BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.authenticity".to_string(),
        advisory_only: true,
        scientific_scope: "authenticity_evidence_advisory_only".to_string(),
        evidence_inputs: vec![
            "damage_metrics".to_string(),
            "fragment_length_distribution".to_string(),
            "mapq_distribution".to_string(),
        ],
        safe_for_claims: vec![
            "pmd_like_signal_present".to_string(),
            "authenticity_evidence_score".to_string(),
        ],
        unsafe_for_claims: vec![
            "authenticity_certification".to_string(),
            "forensic_identity_claim".to_string(),
        ],
    };
    let mut assumptions = vec![
        "library preparation and contamination context are required to interpret PMD".to_string(),
        "authenticity score depends on damage, fragment profile, and MAPQ behavior".to_string(),
    ];
    if let Some(inference) = authenticity.library_type_inference {
        assumptions.push(format!(
            "inferred library type {} with confidence {:.2}",
            format!("{:?}", inference.inferred).to_lowercase(),
            inference.confidence
        ));
    }
    BamAuthenticityAdvisoryV1 {
        schema_version: BAM_AUTHENTICITY_ADVISORY_SCHEMA_VERSION.to_string(),
        stage_id: "bam.authenticity".to_string(),
        score: authenticity.score,
        confidence: authenticity.confidence,
        pmd_like_signal_present: authenticity.evidence.damage_high,
        advisory_boundary,
        assumptions,
        caveats: vec![
            "this stage is advisory and cannot certify authenticity by itself".to_string(),
            "contamination and reference-context evidence must be reviewed jointly".to_string(),
        ],
    }
}

/// Execute mitochondrial contamination evidence with explicit prerequisite checks.
#[must_use]
pub fn execute_mitochondrial_contamination_workflow(
    metrics: &BamMetricsV1,
    has_mito_reference: bool,
    has_damage_context: bool,
    minimum_mean_coverage: f64,
) -> BamContaminationEvidenceV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !has_mito_reference {
        refusal_codes.push("mitochondrial_reference_required".to_string());
    }
    if !has_damage_context {
        refusal_codes.push("damage_context_required".to_string());
    }
    if metrics.coverage.mean < minimum_mean_coverage {
        refusal_codes.push("coverage_below_minimum_for_mito_contamination".to_string());
    }
    let prerequisites_passed = refusal_codes.is_empty();
    BamContaminationEvidenceV1 {
        schema_version: BAM_CONTAMINATION_EVIDENCE_SCHEMA_VERSION.to_string(),
        stage_id: "bam.contamination".to_string(),
        scope: "mitochondrial".to_string(),
        tool: "schmutzi".to_string(),
        prerequisites_passed,
        estimate: prerequisites_passed.then_some(metrics.contamination.estimate),
        ci_low: prerequisites_passed.then_some(metrics.contamination.ci_low),
        ci_high: prerequisites_passed.then_some(metrics.contamination.ci_high),
        advisory_boundary: BamAdvisoryBoundaryV1 {
            schema_version: BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION.to_string(),
            stage_id: "bam.contamination".to_string(),
            advisory_only: true,
            scientific_scope: "mitochondrial_contamination_estimation".to_string(),
            evidence_inputs: vec![
                "mitochondrial_reference".to_string(),
                "damage_context".to_string(),
                "aligned_bam".to_string(),
            ],
            safe_for_claims: vec![
                "contamination_estimate_reported".to_string(),
                "mitochondrial_scope_only".to_string(),
            ],
            unsafe_for_claims: vec![
                "nuclear_contamination_absence".to_string(),
                "sample_authenticity_certification".to_string(),
            ],
        },
        refusal_codes,
        caveats: vec![
            "mitochondrial contamination does not prove nuclear contamination state".to_string(),
            "estimates depend on reference and damage-model assumptions".to_string(),
        ],
    }
}

/// Execute nuclear contamination evidence with panel/build compatibility prerequisites.
#[must_use]
pub fn execute_nuclear_contamination_workflow(
    metrics: &BamMetricsV1,
    has_reference_panel: bool,
    panel_build_compatible: bool,
    sex_context_available: bool,
    minimum_mean_coverage: f64,
) -> BamContaminationEvidenceV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !has_reference_panel {
        refusal_codes.push("reference_panel_required".to_string());
    }
    if !panel_build_compatible {
        refusal_codes.push("panel_build_incompatible".to_string());
    }
    if !sex_context_available {
        refusal_codes.push("sex_context_required".to_string());
    }
    if metrics.coverage.mean < minimum_mean_coverage {
        refusal_codes.push("coverage_below_minimum_for_nuclear_contamination".to_string());
    }
    let prerequisites_passed = refusal_codes.is_empty();
    BamContaminationEvidenceV1 {
        schema_version: BAM_CONTAMINATION_EVIDENCE_SCHEMA_VERSION.to_string(),
        stage_id: "bam.contamination".to_string(),
        scope: "nuclear".to_string(),
        tool: "verifybamid2".to_string(),
        prerequisites_passed,
        estimate: prerequisites_passed.then_some(metrics.contamination.estimate),
        ci_low: prerequisites_passed.then_some(metrics.contamination.ci_low),
        ci_high: prerequisites_passed.then_some(metrics.contamination.ci_high),
        advisory_boundary: BamAdvisoryBoundaryV1 {
            schema_version: BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION.to_string(),
            stage_id: "bam.contamination".to_string(),
            advisory_only: true,
            scientific_scope: "nuclear_contamination_estimation".to_string(),
            evidence_inputs: vec![
                "reference_panel".to_string(),
                "build_compatibility".to_string(),
                "sex_or_chromosome_context".to_string(),
            ],
            safe_for_claims: vec![
                "contamination_estimate_reported".to_string(),
                "nuclear_scope_only".to_string(),
            ],
            unsafe_for_claims: vec![
                "mt_contamination_absence".to_string(),
                "sample_authenticity_certification".to_string(),
            ],
        },
        refusal_codes,
        caveats: vec![
            "nuclear estimates require panel/build congruence and are context-sensitive"
                .to_string(),
            "confidence intervals must be interpreted with coverage and marker density".to_string(),
        ],
    }
}

/// Estimate endogenous content and keep prealignment/postalignment semantics distinct.
#[must_use]
pub fn estimate_endogenous_content(
    metrics: &BamMetricsV1,
    prealignment_fraction: Option<f64>,
) -> BamEndogenousContentEstimateV1 {
    let postalignment_fraction = if metrics.alignment.total > 0 {
        metrics.alignment.mapped as f64 / metrics.alignment.total as f64
    } else {
        0.0
    };
    let mut caveats = vec![
        "postalignment endogenous fraction reflects reference-dependent mapping behavior"
            .to_string(),
        "prealignment endogenous fraction reflects read-space depletion/screening context"
            .to_string(),
    ];
    if let Some(prealign) = prealignment_fraction {
        if (prealign - postalignment_fraction).abs() > 0.25 {
            caveats.push(
                "prealignment and postalignment estimates diverge; review reference compatibility"
                    .to_string(),
            );
        }
    } else {
        caveats.push(
            "prealignment endogenous estimate missing; only postalignment interpretation available"
                .to_string(),
        );
    }
    BamEndogenousContentEstimateV1 {
        schema_version: BAM_ENDOGENOUS_CONTENT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.endogenous_content".to_string(),
        prealignment_fraction,
        postalignment_fraction,
        prealignment_meaning:
            "fraction estimated before alignment from depletion/screening signals".to_string(),
        postalignment_meaning: "fraction of aligned reads relative to total reads after alignment"
            .to_string(),
        caveats,
    }
}

/// Evaluate sex-inference evidence with PAR-aware and coverage/ploidy guardrails.
#[must_use]
pub fn evaluate_sex_inference_par_aware(
    metrics: &BamMetricsV1,
    reference_has_par_masks: bool,
    declared_ploidy: Option<&str>,
    minimum_mean_coverage: f64,
) -> BamSexInferenceEvidenceV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !reference_has_par_masks {
        refusal_codes.push("par_mask_required".to_string());
    }
    if declared_ploidy.is_none() {
        refusal_codes.push("ploidy_context_required".to_string());
    }
    if metrics.coverage.mean < minimum_mean_coverage {
        refusal_codes.push("coverage_below_sex_inference_minimum".to_string());
    }
    if !metrics.sex.sufficient_data {
        refusal_codes.push("insufficient_sex_signal_data".to_string());
    }
    let prerequisites_passed = refusal_codes.is_empty();
    BamSexInferenceEvidenceV1 {
        schema_version: BAM_SEX_EVIDENCE_SCHEMA_VERSION.to_string(),
        stage_id: "bam.sex".to_string(),
        par_aware: reference_has_par_masks,
        prerequisites_passed,
        classification: if prerequisites_passed {
            metrics.sex.classification
        } else {
            SexConfidenceClass::Insufficient
        },
        confidence: if prerequisites_passed { metrics.sex.confidence } else { 0.0 },
        x_to_y_ratio: prerequisites_passed.then_some(metrics.sex.x_to_y_ratio),
        refusal_codes,
        caveats: vec![
            "sex evidence is valid only within declared ploidy/reference assumptions".to_string(),
            "PAR handling is required to avoid inflated chrX/chrY bias".to_string(),
        ],
    }
}

/// Evaluate whether BAM evidence is ready for downstream haplogroup inference workflows.
#[must_use]
pub fn evaluate_haplogroup_readiness(
    metrics: &BamMetricsV1,
    reference_build: Option<&str>,
    contamination_context_available: bool,
) -> BamHaplogroupReadinessV1 {
    let mut refusal_codes = Vec::<String>::new();
    let minimum_coverage = metrics.haplogroup_sufficiency.min_coverage;
    if reference_build.is_none() {
        refusal_codes.push("reference_build_required".to_string());
    }
    if !contamination_context_available {
        refusal_codes.push("contamination_context_required".to_string());
    }
    if !metrics.haplogroup_sufficiency.sufficient {
        refusal_codes.push("haplogroup_sufficiency_not_met".to_string());
    }
    if metrics.coverage.mean < minimum_coverage {
        refusal_codes.push("coverage_below_haplogroup_minimum".to_string());
    }
    let ready = refusal_codes.is_empty();
    BamHaplogroupReadinessV1 {
        schema_version: BAM_HAPLOGROUP_READINESS_SCHEMA_VERSION.to_string(),
        stage_id: "bam.haplogroups".to_string(),
        ready,
        reference_build: reference_build.map(ToOwned::to_owned),
        minimum_coverage,
        observed_mean_coverage: metrics.coverage.mean,
        contamination_estimate: contamination_context_available
            .then_some(metrics.contamination.estimate),
        refusal_codes,
        caveats: vec![
            "haplogroup interpretation depends on organellar/Y reference compatibility".to_string(),
            "contamination estimates must be interpreted alongside haplogroup confidence"
                .to_string(),
        ],
    }
}

/// Evaluate BAM-side prerequisites required before kinship/relatedness workflows.
#[must_use]
pub fn evaluate_kinship_prerequisites(
    metrics: &BamMetricsV1,
    marker_overlap_snps: u32,
    sample_identity_consistent: bool,
    max_contamination: f64,
    minimum_mean_coverage: f64,
) -> BamKinshipPrerequisitesV1 {
    let mut refusal_codes = Vec::<String>::new();
    if metrics.coverage.mean < minimum_mean_coverage {
        refusal_codes.push("coverage_below_kinship_minimum".to_string());
    }
    if marker_overlap_snps < metrics.kinship_sufficiency.overlap_snps {
        refusal_codes.push("marker_overlap_below_required_minimum".to_string());
    }
    if !sample_identity_consistent {
        refusal_codes.push("sample_identity_inconsistent".to_string());
    }
    if metrics.contamination.estimate > max_contamination {
        refusal_codes.push("contamination_above_kinship_limit".to_string());
    }
    if !metrics.kinship_sufficiency.sufficient {
        refusal_codes.push("kinship_sufficiency_not_met".to_string());
    }
    let ready = refusal_codes.is_empty();
    BamKinshipPrerequisitesV1 {
        schema_version: BAM_KINSHIP_PREREQUISITES_SCHEMA_VERSION.to_string(),
        stage_id: "bam.kinship".to_string(),
        ready,
        minimum_mean_coverage,
        observed_mean_coverage: metrics.coverage.mean,
        marker_overlap_snps,
        sample_identity_consistent,
        contamination_estimate: metrics.contamination.estimate,
        max_contamination,
        refusal_codes,
        caveats: vec![
            "kinship results are sensitive to marker overlap and contamination assumptions"
                .to_string(),
            "sample identity and cohort context must remain consistent across merged inputs"
                .to_string(),
        ],
    }
}

#[must_use]
pub fn bam_adna_workflow_contract() -> BamAdnaWorkflowV1 {
    BamAdnaWorkflowV1 {
        schema_version: BAM_ADNA_WORKFLOW_SCHEMA_VERSION.to_string(),
        workflow_id: "ancient_dna_damage_and_authenticity".to_string(),
        damage_tools: vec![
            "mapdamage2".to_string(),
            "damageprofiler".to_string(),
            "pmdtools".to_string(),
            "pydamage".to_string(),
        ],
        authenticity_tools: vec!["authenticct".to_string(), "pmdtools".to_string()],
        evidence_only: true,
        authenticity_caveats: vec![
            "damage signatures are evidence and must not be reported as authenticity certification"
                .to_string(),
            "tool outputs require context from contamination, fragment length, and library prep"
                .to_string(),
        ],
    }
}

#[must_use]
pub fn bam_contamination_workflow_contract() -> BamContaminationWorkflowV1 {
    BamContaminationWorkflowV1 {
        schema_version: BAM_CONTAMINATION_WORKFLOW_SCHEMA_VERSION.to_string(),
        workflow_id: "bam_contamination_assessment".to_string(),
        tools: vec![
            BamContaminationToolContractV1 {
                tool_id: "schmutzi".to_string(),
                supported_scope: "mitochondrial_or_both".to_string(),
                required_inputs: vec![
                    "mitochondrial_reference".to_string(),
                    "aligned_bam".to_string(),
                    "damage_context".to_string(),
                ],
                emits_confidence: true,
                caveats: vec![
                    "nuclear contamination is not estimated by schmutzi alone".to_string()
                ],
            },
            BamContaminationToolContractV1 {
                tool_id: "verifybamid2".to_string(),
                supported_scope: "nuclear_or_both".to_string(),
                required_inputs: vec![
                    "reference_panel".to_string(),
                    "aligned_bam".to_string(),
                    "sex_or_chromosome_context".to_string(),
                    "minimum_coverage_context".to_string(),
                ],
                emits_confidence: true,
                caveats: vec!["sex chromosome assumptions must be declared explicitly".to_string()],
            },
            BamContaminationToolContractV1 {
                tool_id: "contammix".to_string(),
                supported_scope: "nuclear_or_both".to_string(),
                required_inputs: vec![
                    "reference_panel".to_string(),
                    "aligned_bam".to_string(),
                    "minimum_coverage_context".to_string(),
                ],
                emits_confidence: true,
                caveats: vec![
                    "results are panel-dependent and must carry coverage caveats".to_string()
                ],
            },
        ],
    }
}

#[must_use]
pub fn bam_scientific_report_contracts() -> Vec<BamScientificReportContractV1> {
    vec![
        BamScientificReportContractV1 {
            schema_version: BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION.to_string(),
            report_id: BamScientificReportIdV1::EndogenousContent,
            optional: true,
            suitable_scopes: vec!["host_alignment_qc".to_string()],
            required_population_or_reference_context: vec!["host_reference_scope".to_string()],
            refusal_when_missing: vec!["host_reference_scope_required".to_string()],
        },
        BamScientificReportContractV1 {
            schema_version: BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION.to_string(),
            report_id: BamScientificReportIdV1::Sex,
            optional: true,
            suitable_scopes: vec!["human_chrxy".to_string(), "chrxy_like_reference".to_string()],
            required_population_or_reference_context: vec![
                "chromosome_system".to_string(),
                "minimum_y_sites".to_string(),
            ],
            refusal_when_missing: vec!["sex_context_required".to_string()],
        },
        BamScientificReportContractV1 {
            schema_version: BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION.to_string(),
            report_id: BamScientificReportIdV1::Haplogroups,
            optional: true,
            suitable_scopes: vec!["human_mitochondrial".to_string()],
            required_population_or_reference_context: vec![
                "reference_panel".to_string(),
                "reference_build".to_string(),
                "population_scope".to_string(),
            ],
            refusal_when_missing: vec!["haplogroup_context_required".to_string()],
        },
        BamScientificReportContractV1 {
            schema_version: BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION.to_string(),
            report_id: BamScientificReportIdV1::Kinship,
            optional: true,
            suitable_scopes: vec!["human_cohort".to_string()],
            required_population_or_reference_context: vec![
                "reference_panel".to_string(),
                "reference_build".to_string(),
                "population_scope".to_string(),
                "cohort_context".to_string(),
            ],
            refusal_when_missing: vec!["kinship_context_required".to_string()],
        },
    ]
}

#[must_use]
pub fn bam_scientific_report_contract_for_stage(
    stage_id: &str,
) -> Option<BamScientificReportContractV1> {
    bam_scientific_report_contracts()
        .into_iter()
        .find(|contract| contract.report_id.stage_id() == stage_id)
}

#[must_use]
pub fn estimate_bam_stage_resources(stage_id: &str, input_bytes: u64) -> BamStageResourcePlanV1 {
    estimate_bam_stage_resources_with_origin(stage_id, input_bytes, BamInputOriginV1::Real)
}

#[must_use]
pub fn estimate_bam_stage_resources_with_origin(
    stage_id: &str,
    input_bytes: u64,
    input_origin: BamInputOriginV1,
) -> BamStageResourcePlanV1 {
    const MIB: u64 = 1024_u64 * 1024_u64;
    let gib = 1024_u64 * 1024_u64 * 1024_u64;
    let size_gb = input_bytes.div_ceil(gib).max(1) as u32;
    let input_scale = if input_bytes <= 256_u64 * MIB {
        BamInputScaleV1::Tiny
    } else if input_bytes <= 4_u64 * gib {
        BamInputScaleV1::Small
    } else if input_bytes <= 40_u64 * gib {
        BamInputScaleV1::Medium
    } else {
        BamInputScaleV1::Large
    };
    let (
        mut cpu_threads,
        mut memory_gb,
        mut disk_gb,
        mut scratch_gb,
        mut walltime_minutes,
        io_profile,
        requires_index,
        note,
    ) = match stage_id {
        "bam.sort_index" | "bam.sort" => (
            4_u32,
            size_gb.saturating_mul(2).max(4),
            size_gb.saturating_mul(3).max(8),
            size_gb.saturating_mul(2).max(6),
            size_gb.saturating_mul(14).max(20),
            "io_heavy_sequential_plus_temp_shuffle",
            true,
            "sorting/indexing scales with temporary shards and deterministic merge passes",
        ),
        "bam.markdup" => (
            4_u32,
            size_gb.saturating_mul(2).max(4),
            size_gb.saturating_mul(3).max(8),
            size_gb.saturating_mul(2).max(6),
            size_gb.saturating_mul(12).max(18),
            "io_heavy_coordinate_streaming",
            true,
            "duplicate marking scales with coordinate-sorted temporary shards",
        ),
        "bam.coverage" => (
            2_u32,
            size_gb.max(2),
            size_gb.max(2),
            size_gb.max(1),
            size_gb.saturating_mul(8).max(12),
            "mixed_random_and_streaming_reads",
            true,
            "coverage requires indexed random access for deterministic summaries",
        ),
        "bam.damage" => (
            2_u32,
            size_gb.max(2),
            size_gb.max(2),
            size_gb.max(2),
            size_gb.saturating_mul(10).max(15),
            "streaming_reads_with_sidecar_metrics",
            true,
            "damage tools stream BAM plus sidecar summaries",
        ),
        "bam.endogenous_content" => (
            2_u32,
            size_gb.max(2),
            size_gb.max(2),
            size_gb.max(1),
            size_gb.saturating_mul(7).max(10),
            "indexed_depth_pass",
            true,
            "endogenous estimation reuses indexed depth calculations",
        ),
        _ => (
            4_u32,
            size_gb.saturating_mul(2).max(4),
            size_gb.saturating_mul(2).max(4),
            size_gb.saturating_mul(2).max(4),
            size_gb.saturating_mul(9).max(14),
            "balanced_compute_and_io",
            true,
            "sorting/indexing/validation chain dominates temporary storage",
        ),
    };
    if matches!(input_origin, BamInputOriginV1::Synthetic) {
        cpu_threads = cpu_threads.saturating_sub(1).max(1);
        memory_gb = memory_gb.max(2).div_ceil(2);
        disk_gb = disk_gb.max(2).div_ceil(2);
        scratch_gb = scratch_gb.max(2).div_ceil(2);
        walltime_minutes = walltime_minutes.max(4).div_ceil(2);
    };
    BamStageResourcePlanV1 {
        schema_version: BAM_RESOURCE_PLAN_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        input_bytes,
        cpu_threads,
        memory_gb,
        disk_gb,
        scratch_gb,
        walltime_minutes,
        io_profile: io_profile.to_string(),
        input_origin,
        input_scale,
        requires_index,
        notes: vec![note.to_string()],
    }
}

#[must_use]
pub fn required_bam_bench_corpus_scenarios() -> Vec<BamBenchDatasetScenarioV1> {
    vec![
        BamBenchDatasetScenarioV1::TinyAligned,
        BamBenchDatasetScenarioV1::UnmappedHeavy,
        BamBenchDatasetScenarioV1::DuplicateHeavy,
        BamBenchDatasetScenarioV1::LowCoverage,
        BamBenchDatasetScenarioV1::DamageLike,
        BamBenchDatasetScenarioV1::ReferenceMismatch,
        BamBenchDatasetScenarioV1::MissingIndex,
    ]
}

#[must_use]
pub fn bam_bench_corpus_manifest() -> BamBenchCorpusManifestV1 {
    BamBenchCorpusManifestV1 {
        schema_version: BAM_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION.to_string(),
        corpus_id: "bam_production_regression".to_string(),
        scenarios_covered: required_bam_bench_corpus_scenarios(),
        ci_subset: vec![
            "SYNTHETIC_TINY_ALIGNED".to_string(),
            "SYNTHETIC_DUPLICATE_HEAVY".to_string(),
            "SYNTHETIC_MISSING_INDEX".to_string(),
        ],
        datasets: vec![
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_TINY_ALIGNED".to_string(),
                scientific_scope: "alignment_baseline".to_string(),
                requires_index: true,
                scenarios: vec![BamBenchDatasetScenarioV1::TinyAligned],
            },
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_UNMAPPED_HEAVY".to_string(),
                scientific_scope: "mapping_failure_regression".to_string(),
                requires_index: true,
                scenarios: vec![BamBenchDatasetScenarioV1::UnmappedHeavy],
            },
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_DUPLICATE_HEAVY".to_string(),
                scientific_scope: "duplicate_policy_regression".to_string(),
                requires_index: true,
                scenarios: vec![BamBenchDatasetScenarioV1::DuplicateHeavy],
            },
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_LOW_COVERAGE".to_string(),
                scientific_scope: "coverage_regime_regression".to_string(),
                requires_index: true,
                scenarios: vec![BamBenchDatasetScenarioV1::LowCoverage],
            },
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_DAMAGE_LIKE".to_string(),
                scientific_scope: "ancient_dna_regression".to_string(),
                requires_index: true,
                scenarios: vec![BamBenchDatasetScenarioV1::DamageLike],
            },
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_REFERENCE_MISMATCH".to_string(),
                scientific_scope: "reference_guardrail_regression".to_string(),
                requires_index: true,
                scenarios: vec![BamBenchDatasetScenarioV1::ReferenceMismatch],
            },
            BamBenchCorpusDatasetManifestEntryV1 {
                dataset_id: "SYNTHETIC_MISSING_INDEX".to_string(),
                scientific_scope: "index_guardrail_regression".to_string(),
                requires_index: false,
                scenarios: vec![BamBenchDatasetScenarioV1::MissingIndex],
            },
        ],
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
    fn bam_mapq_filter_summary_round_trips() {
        let payload = BamMapqFilterSummaryV1 {
            schema_version: BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION.to_string(),
            stage_id: "bam.mapq_filter".to_string(),
            mapq_threshold: 30,
            input_bam: PathBuf::from("input.bam"),
            output_bam: PathBuf::from("filtered.bam"),
            flagstat_before: BamFlagstatCountsV1 {
                total_reads: Some(100),
                mapped_reads: Some(90),
                duplicate_reads: Some(10),
                mapped_fraction: Some(0.9),
            },
            flagstat_after: BamFlagstatCountsV1 {
                total_reads: Some(80),
                mapped_reads: Some(70),
                duplicate_reads: Some(8),
                mapped_fraction: Some(0.875),
            },
            mapped_reads_removed: Some(20),
            mapped_fraction_retained: Some(70.0 / 90.0),
        };

        let json = serde_json::to_value(&payload).expect("serialize mapq filter summary");
        let roundtrip: BamMapqFilterSummaryV1 =
            serde_json::from_value(json).expect("roundtrip mapq filter summary");
        assert_eq!(roundtrip, payload);
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

    #[test]
    fn execute_ancient_damage_evidence_respects_advisory_boundary() {
        let mut metrics = BamMetricsV1::empty();
        metrics.damage.c_to_t_5p = 0.18;
        metrics.damage.g_to_a_3p = 0.12;
        metrics.fragment_length.short_fraction = 0.35;

        let advisory = execute_ancient_damage_evidence(&metrics, false);
        assert_eq!(advisory.damage_signal, "moderate");
        assert!(advisory.advisory_boundary.advisory_only);
        assert!(!advisory.strict_profile_upgraded);

        let strict = execute_ancient_damage_evidence(&metrics, true);
        assert_eq!(strict.damage_signal, "moderate");
        assert!(!strict.advisory_boundary.advisory_only);
        assert!(strict.strict_profile_upgraded);
    }

    #[test]
    fn execute_pmd_authenticity_advisory_never_certifies_authenticity() {
        let mut metrics = BamMetricsV1::empty();
        metrics.damage.c_to_t_5p = 0.22;
        metrics.damage.g_to_a_3p = 0.19;
        metrics.fragment_length.short_fraction = 0.30;
        metrics.mapq.mean = 22.0;

        let advisory = execute_pmd_authenticity_advisory(&metrics);
        assert!(advisory.pmd_like_signal_present);
        assert!(advisory.score > 0.0);
        assert!(advisory.confidence > 0.0);
        assert!(advisory.advisory_boundary.advisory_only);
        assert!(advisory
            .advisory_boundary
            .unsafe_for_claims
            .contains(&"authenticity_certification".to_string()));
    }

    #[test]
    fn execute_mitochondrial_contamination_workflow_requires_context() {
        let mut metrics = BamMetricsV1::empty();
        metrics.coverage.mean = 2.5;
        metrics.contamination.estimate = 0.03;
        metrics.contamination.ci_low = 0.01;
        metrics.contamination.ci_high = 0.05;

        let ready = execute_mitochondrial_contamination_workflow(&metrics, true, true, 1.0);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.scope, "mitochondrial");
        assert_eq!(ready.tool, "schmutzi");
        assert_eq!(ready.estimate, Some(0.03));

        let refused = execute_mitochondrial_contamination_workflow(&metrics, false, false, 3.0);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"mitochondrial_reference_required".to_string()));
        assert!(refused.refusal_codes.contains(&"damage_context_required".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"coverage_below_minimum_for_mito_contamination".to_string()));
    }

    #[test]
    fn execute_nuclear_contamination_workflow_checks_panel_build_and_coverage() {
        let mut metrics = BamMetricsV1::empty();
        metrics.coverage.mean = 6.0;
        metrics.contamination.estimate = 0.08;
        metrics.contamination.ci_low = 0.05;
        metrics.contamination.ci_high = 0.11;

        let ready = execute_nuclear_contamination_workflow(&metrics, true, true, true, 4.0);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.scope, "nuclear");
        assert_eq!(ready.tool, "verifybamid2");
        assert_eq!(ready.estimate, Some(0.08));

        let refused = execute_nuclear_contamination_workflow(&metrics, false, false, false, 8.0);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"reference_panel_required".to_string()));
        assert!(refused.refusal_codes.contains(&"panel_build_incompatible".to_string()));
        assert!(refused.refusal_codes.contains(&"sex_context_required".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"coverage_below_minimum_for_nuclear_contamination".to_string()));
    }

    #[test]
    fn estimate_endogenous_content_separates_prealign_and_postalign_meanings() {
        let mut metrics = BamMetricsV1::empty();
        metrics.alignment.total = 100;
        metrics.alignment.mapped = 62;

        let estimate = estimate_endogenous_content(&metrics, Some(0.20));
        assert_eq!(estimate.prealignment_fraction, Some(0.20));
        assert!((estimate.postalignment_fraction - 0.62).abs() < 1e-6);
        assert!(estimate
            .caveats
            .iter()
            .any(|item| item.contains("prealignment and postalignment estimates diverge")));
        assert!(estimate.prealignment_meaning.contains("before alignment"));
        assert!(estimate.postalignment_meaning.contains("after alignment"));
    }

    #[test]
    fn evaluate_sex_inference_par_aware_enforces_prerequisites() {
        let mut metrics = BamMetricsV1::empty();
        metrics.coverage.mean = 6.0;
        metrics.sex.sufficient_data = true;
        metrics.sex.confidence = 0.81;
        metrics.sex.x_to_y_ratio = 0.52;
        metrics.sex.classification = SexConfidenceClass::Male;

        let ready = evaluate_sex_inference_par_aware(&metrics, true, Some("diploid"), 2.0);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.classification, SexConfidenceClass::Male);
        assert_eq!(ready.x_to_y_ratio, Some(0.52));

        let refused = evaluate_sex_inference_par_aware(&metrics, false, None, 8.0);
        assert!(!refused.prerequisites_passed);
        assert_eq!(refused.classification, SexConfidenceClass::Insufficient);
        assert!(refused.refusal_codes.contains(&"par_mask_required".to_string()));
        assert!(refused.refusal_codes.contains(&"ploidy_context_required".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"coverage_below_sex_inference_minimum".to_string()));
    }

    #[test]
    fn evaluate_haplogroup_readiness_requires_reference_and_contamination_context() {
        let mut metrics = BamMetricsV1::empty();
        metrics.coverage.mean = 11.0;
        metrics.haplogroup_sufficiency.sufficient = true;
        metrics.haplogroup_sufficiency.min_coverage = 8.0;
        metrics.contamination.estimate = 0.02;

        let ready = evaluate_haplogroup_readiness(&metrics, Some("GRCh38"), true);
        assert!(ready.ready);
        assert_eq!(ready.reference_build.as_deref(), Some("GRCh38"));
        assert_eq!(ready.contamination_estimate, Some(0.02));

        let refused = evaluate_haplogroup_readiness(&metrics, None, false);
        assert!(!refused.ready);
        assert!(refused.refusal_codes.contains(&"reference_build_required".to_string()));
        assert!(refused.refusal_codes.contains(&"contamination_context_required".to_string()));
    }

    #[test]
    fn evaluate_kinship_prerequisites_checks_coverage_markers_identity_and_contamination() {
        let mut metrics = BamMetricsV1::empty();
        metrics.coverage.mean = 7.5;
        metrics.contamination.estimate = 0.02;
        metrics.kinship_sufficiency.sufficient = true;
        metrics.kinship_sufficiency.overlap_snps = 20_000;

        let ready = evaluate_kinship_prerequisites(&metrics, 22_000, true, 0.05, 4.0);
        assert!(ready.ready);
        assert_eq!(ready.marker_overlap_snps, 22_000);
        assert!(ready.refusal_codes.is_empty());

        let refused = evaluate_kinship_prerequisites(&metrics, 5_000, false, 0.01, 9.0);
        assert!(!refused.ready);
        assert!(refused.refusal_codes.contains(&"coverage_below_kinship_minimum".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"marker_overlap_below_required_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"sample_identity_inconsistent".to_string()));
        assert!(refused.refusal_codes.contains(&"contamination_above_kinship_limit".to_string()));
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bijux-{label}-{stamp}"));
        std::fs::create_dir_all(&path).expect("create temporary directory");
        path
    }

    #[test]
    fn execute_bam_validation_reports_fixture_integrity() {
        let temp = unique_temp_dir("bam-validate");
        let sam = temp.join("input.sam");
        let bai = temp.join("input.sam.bai");
        let reference = temp.join("reference.fa");
        std::fs::write(
            &sam,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sample1\n\
r001\t99\tchr1\t1\t60\t6M\t=\t7\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r001\t147\tchr1\t7\t60\t6M\t=\t1\t0\tTTAACT\tFFFFFF\tRG:Z:rg1\n",
        )
        .expect("write SAM fixture");
        std::fs::write(&bai, "tiny-index\n").expect("write BAI fixture");
        std::fs::write(&reference, ">chr1\nACGTACGTACGTACGTACGT\n").expect("write reference");

        let summary =
            execute_bam_validation(&sam, Some(&bai), Some(&reference)).expect("validate fixture");
        assert!(summary.validation_report_present);
        assert!(summary.refusal_codes.is_empty());
        assert_eq!(summary.flagstat.total_reads, Some(2));
        assert_eq!(summary.flagstat.mapped_reads, Some(2));
    }

    #[test]
    fn execute_bam_validation_refuses_malformed_or_missing_assets() {
        let temp = unique_temp_dir("bam-validate-negative");
        let sam = temp.join("broken.sam");
        std::fs::write(
            &sam,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
r001\t99\tchr1\n",
        )
        .expect("write malformed SAM fixture");

        let summary = execute_bam_validation(&sam, None, None).expect("validate malformed");
        assert!(!summary.validation_report_present);
        assert!(summary.refusal_codes.contains(&"malformed_alignment_record".to_string()));
    }

    fn write_fastq(path: &Path, records: &[(&str, &str)]) {
        let mut payload = String::new();
        for (id, sequence) in records {
            payload.push_str(&format!("@{id}\n{sequence}\n+\n{}\n", "I".repeat(sequence.len())));
        }
        std::fs::write(path, payload).expect("write FASTQ fixture");
    }

    #[test]
    fn align_fastq_to_bam_bwa_style_emits_inventory_and_mapping_summary() {
        let temp = unique_temp_dir("bam-align-bwa");
        let reference = temp.join("reference.fa");
        let reads_r1 = temp.join("reads_R1.fastq");
        let reads_r2 = temp.join("reads_R2.fastq");
        std::fs::write(&reference, ">chr1\nACGTACGTACGTACGTACGT\n").expect("write reference");
        write_fastq(&reads_r1, &[("read1/1", "ACGTAC"), ("read2/1", "TTTTTT")]);
        write_fastq(&reads_r2, &[("read1/2", "GTACGT"), ("read2/2", "AAAAAA")]);

        let read_group = ReadGroupSpec::with_defaults("sample1");
        let out = temp.join("align-bwa");
        let (provenance, summary) = align_fastq_to_bam_bwa_style(
            &reference,
            &reads_r1,
            Some(&reads_r2),
            &out,
            "sample1",
            &read_group,
            Some("default"),
            Some(12),
        )
        .expect("align with bwa-style backend");

        assert_eq!(provenance.backend_tool_id, "bwa");
        assert_eq!(provenance.mode.as_deref(), Some("end_to_end"));
        assert!(provenance.outputs.outputs.iter().any(|entry| entry.name == "align_bam"));
        assert_eq!(summary.flagstat.total_reads, Some(4));
        assert!(summary.mapq_regime.is_some());
        assert!(out.join("align.bam").exists());
        assert!(out.join("align.bam.bai").exists());
    }

    #[test]
    fn align_fastq_to_bam_bowtie2_style_keeps_local_sensitivity_context() {
        let temp = unique_temp_dir("bam-align-bowtie2");
        let reference = temp.join("reference.fa");
        let reads_r1 = temp.join("reads_R1.fastq");
        std::fs::write(&reference, ">chr1\nAACCGGTTAACCGGTT\n").expect("write reference");
        write_fastq(&reads_r1, &[("read-local", "GGTTAA"), ("read-unmapped", "TTTTTTTTTT")]);

        let read_group = ReadGroupSpec::with_defaults("sample2");
        let out = temp.join("align-bowtie2");
        let (provenance, summary) = align_fastq_to_bam_bowtie2_style(
            &reference,
            &reads_r1,
            None,
            &out,
            "sample2",
            &read_group,
            Some("very_sensitive_local"),
        )
        .expect("align with bowtie2-style backend");

        assert_eq!(provenance.backend_tool_id, "bowtie2");
        assert_eq!(provenance.mode.as_deref(), Some("local"));
        assert_eq!(provenance.sensitivity_profile.as_deref(), Some("very_sensitive_local"));
        assert_eq!(summary.flagstat.total_reads, Some(2));
        assert_eq!(summary.flagstat.mapped_reads, Some(1));
    }

    #[test]
    fn sort_and_index_tiny_bam_produces_coordinate_order_and_index() {
        let temp = unique_temp_dir("bam-sort-index");
        let input = temp.join("unsorted.sam");
        let output = temp.join("sorted.bam");
        let index = temp.join("sorted.bam.bai");
        std::fs::write(
            &input,
            "@HD\tVN:1.6\tSO:unsorted\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sample1\n\
r10\t0\tchr1\t10\t40\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r01\t0\tchr1\t1\t40\t6M\t*\t0\t0\tGTACGT\tFFFFFF\tRG:Z:rg1\n",
        )
        .expect("write unsorted SAM fixture");

        let inventory = sort_and_index_tiny_bam(&input, &output, &index).expect("sort and index");
        assert_eq!(inventory.stage_id, "bam.sort_index");
        assert!(output.exists());
        assert!(index.exists());

        let sorted_payload = std::fs::read_to_string(&output).expect("read sorted output");
        let record_lines =
            sorted_payload.lines().filter(|line| !line.starts_with('@')).collect::<Vec<_>>();
        assert!(record_lines[0].starts_with("r01\t"));
        assert!(record_lines[1].starts_with("r10\t"));

        let validation = execute_bam_validation(&output, Some(&index), None).expect("validate");
        assert!(validation.validation_report_present);
    }

    #[test]
    fn propagate_bam_sample_identity_preserves_lineage_fields() {
        let read_group = ReadGroupSpec {
            id: "rg-lane2".to_string(),
            sample: "sample-z".to_string(),
            platform: "ILLUMINA".to_string(),
            library: "lib-z".to_string(),
            platform_unit: Some("pu-z".to_string()),
            lane_id: Some("L002".to_string()),
            run_id: Some("run-z".to_string()),
        };
        let prior = BamSampleIdentityV1 {
            schema_version: BAM_SAMPLE_IDENTITY_SCHEMA_VERSION.to_string(),
            sample_id: "sample-z".to_string(),
            lane_id: Some("L001".to_string()),
            library_id: Some("lib-z".to_string()),
            platform: Some("ILLUMINA".to_string()),
            platform_unit: Some("pu-z".to_string()),
            run_id: Some("run-z".to_string()),
            subject_id: Some("subject-z".to_string()),
            cohort_id: Some("cohort-z".to_string()),
            read_group_policy: Some("preserve".to_string()),
            read_group_ids: vec!["rg-lane1".to_string()],
        };

        let propagated = propagate_bam_sample_identity(&prior, &read_group, "bam.align");
        assert_eq!(propagated.sample_id, "sample-z");
        assert_eq!(propagated.lane_id.as_deref(), Some("L001"));
        assert_eq!(propagated.library_id.as_deref(), Some("lib-z"));
        assert_eq!(propagated.subject_id.as_deref(), Some("subject-z"));
        assert_eq!(propagated.cohort_id.as_deref(), Some("cohort-z"));
        assert_eq!(propagated.read_group_ids, vec!["rg-lane1".to_string(), "rg-lane2".to_string()]);
        assert_eq!(propagated.read_group_policy.as_deref(), Some("propagate:bam.align"));
    }

    #[test]
    fn merge_tiny_bam_with_conflict_refusal_materializes_only_compatible_inputs() {
        let temp = unique_temp_dir("bam-merge");
        let lane1 = temp.join("lane1.sam");
        let lane2 = temp.join("lane2.sam");
        let merged = temp.join("merged.sam");
        std::fs::write(
            &lane1,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sampleA\n\
r01\t0\tchr1\t1\t40\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n",
        )
        .expect("write lane1");
        std::fs::write(
            &lane2,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg2\tSM:sampleA\n\
r02\t0\tchr1\t5\t40\t6M\t*\t0\t0\tGTACGT\tFFFFFF\tRG:Z:rg2\n",
        )
        .expect("write lane2");

        let identities = vec![
            BamMergeInputIdentityV1 {
                sample_id: "sampleA".to_string(),
                read_group_ids: vec!["rg1".to_string()],
                reference_digest: Some("sha256:ref".to_string()),
                sequencing_platform: Some("ILLUMINA".to_string()),
                library_id: Some("lib1".to_string()),
                lane_id: Some("L001".to_string()),
                platform_unit: Some("pu1".to_string()),
            },
            BamMergeInputIdentityV1 {
                sample_id: "sampleA".to_string(),
                read_group_ids: vec!["rg2".to_string()],
                reference_digest: Some("sha256:ref".to_string()),
                sequencing_platform: Some("ILLUMINA".to_string()),
                library_id: Some("lib1".to_string()),
                lane_id: Some("L002".to_string()),
                platform_unit: Some("pu2".to_string()),
            },
        ];
        let compatibility = merge_tiny_bam_with_conflict_refusal(
            &[lane1.clone(), lane2.clone()],
            &identities,
            &merged,
        )
        .expect("merge compatible inputs");
        assert!(compatibility.compatible);
        assert!(merged.exists());

        let conflict = merge_tiny_bam_with_conflict_refusal(
            &[lane1, lane2],
            &[
                identities[0].clone(),
                BamMergeInputIdentityV1 {
                    sample_id: "sampleB".to_string(),
                    ..identities[1].clone()
                },
            ],
            &temp.join("should-not-exist.sam"),
        )
        .expect("evaluate conflict");
        assert!(!conflict.compatible);
        assert!(conflict.refusal_codes.contains(&"merge_sample_id_conflict".to_string()));
    }

    #[test]
    fn apply_duplicate_policy_tiny_bam_supports_mark_and_remove_modes() {
        let temp = unique_temp_dir("bam-duplicate-policy");
        let input = temp.join("input.sam");
        std::fs::write(
            &input,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sampleA\n\
r01\t0\tchr1\t1\t40\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r02\t0\tchr1\t1\t40\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r03\t0\tchr1\t7\t40\t6M\t*\t0\t0\tTTTTTT\tFFFFFF\tRG:Z:rg1\n",
        )
        .expect("write duplicate fixture");

        let marked_output = temp.join("marked.sam");
        let (mark_policy, mark_comparison) =
            apply_duplicate_policy_tiny_bam(&input, &marked_output, "mark", Some("umi_optional"))
                .expect("mark duplicates");
        assert_eq!(mark_policy.duplicate_action.as_deref(), Some("mark"));
        assert!(mark_comparison.comparable);
        let marked = parse_tiny_sam(&marked_output).expect("parse marked output");
        assert_eq!(marked.records.iter().filter(|record| (record.flag & 0x400) != 0).count(), 1);

        let removed_output = temp.join("removed.sam");
        let (remove_policy, remove_comparison) = apply_duplicate_policy_tiny_bam(
            &input,
            &removed_output,
            "remove",
            Some("umi_optional"),
        )
        .expect("remove duplicates");
        assert_eq!(remove_policy.duplicate_action.as_deref(), Some("remove"));
        assert!(remove_comparison
            .notes
            .iter()
            .any(|note| note.contains("removed from output alignment")));
        let removed = parse_tiny_sam(&removed_output).expect("parse removed output");
        assert_eq!(removed.records.len(), 2);
    }

    #[test]
    fn filter_tiny_bam_by_mapq_tracks_retained_and_removed_reads() {
        let temp = unique_temp_dir("bam-mapq-filter");
        let input = temp.join("input.sam");
        let output = temp.join("filtered.sam");
        std::fs::write(
            &input,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sampleA\n\
r01\t0\tchr1\t1\t45\t6M\t*\t0\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r02\t0\tchr1\t7\t10\t6M\t*\t0\t0\tTTTTTT\tFFFFFF\tRG:Z:rg1\n\
r03\t4\t*\t0\t0\t*\t*\t0\t0\tNNNNNN\tFFFFFF\tRG:Z:rg1\n",
        )
        .expect("write mapq fixture");

        let summary = filter_tiny_bam_by_mapq(&input, &output, 30).expect("filter MAPQ");
        assert_eq!(summary.mapq_threshold, 30);
        assert_eq!(summary.flagstat_before.mapped_reads, Some(2));
        assert_eq!(summary.flagstat_after.mapped_reads, Some(1));
        assert_eq!(summary.mapped_reads_removed, Some(1));
        assert_eq!(summary.mapped_fraction_retained, Some(0.5));

        let filtered = parse_tiny_sam(&output).expect("parse filtered output");
        assert_eq!(filtered.records.len(), 2);
    }

    #[test]
    fn summarize_tiny_bam_mapping_reports_mapq_histogram_and_read_group_breakdown() {
        let temp = unique_temp_dir("bam-mapping-summary");
        let input = temp.join("input.sam");
        std::fs::write(
            &input,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:50\n\
@RG\tID:rg1\tSM:sampleA\n\
@RG\tID:rg2\tSM:sampleA\n\
r01\t99\tchr1\t1\t45\t6M\t=\t9\t0\tACGTAC\tFFFFFF\tRG:Z:rg1\n\
r01\t147\tchr1\t9\t45\t6M\t=\t1\t0\tGTACGT\tFFFFFF\tRG:Z:rg1\n\
r02\t0\tchr1\t20\t10\t6M\t*\t0\t0\tTTTTTT\tFFFFFF\tRG:Z:rg2\n\
r03\t4\t*\t0\t0\t*\t*\t0\t0\tNNNNNN\tFFFFFF\tRG:Z:rg2\n",
        )
        .expect("write mapping summary fixture");

        let summary = summarize_tiny_bam_mapping(&input).expect("summarize mapping");
        assert_eq!(summary.flagstat.total_reads, Some(4));
        assert_eq!(summary.flagstat.mapped_reads, Some(3));
        assert_eq!(summary.proper_pair_reads, Some(2));
        assert_eq!(summary.secondary_reads, Some(0));
        assert_eq!(summary.supplementary_reads, Some(0));
        assert_eq!(summary.mapq_histogram, vec![(10, 1), (45, 2)]);
        assert_eq!(summary.read_group_breakdown.len(), 2);
        assert_eq!(summary.read_group_breakdown[0].read_group_id, "rg1");
        assert_eq!(summary.read_group_breakdown[0].mapped_reads, 2);
        assert_eq!(summary.read_group_breakdown[1].read_group_id, "rg2");
        assert_eq!(summary.read_group_breakdown[1].mapped_reads, 1);
    }

    #[test]
    fn summarize_tiny_bam_coverage_classifies_regime_from_depth_and_breadth() {
        let temp = unique_temp_dir("bam-coverage-summary");
        let input = temp.join("input.sam");
        std::fs::write(
            &input,
            "@HD\tVN:1.6\tSO:coordinate\n\
@SQ\tSN:chr1\tLN:20\n\
@RG\tID:rg1\tSM:sampleA\n\
r01\t0\tchr1\t1\t45\t10M\t*\t0\t0\tACGTACGTAC\tFFFFFFFFFF\tRG:Z:rg1\n\
r02\t0\tchr1\t11\t45\t10M\t*\t0\t0\tTTTTGGGGCC\tFFFFFFFFFF\tRG:Z:rg1\n",
        )
        .expect("write coverage fixture");

        let summary = summarize_tiny_bam_coverage(&input, &[1, 5, 10]).expect("summarize coverage");
        assert_eq!(summary.stage_id, "bam.coverage");
        assert_eq!(summary.depth_thresholds, vec![1, 5, 10]);
        assert_eq!(summary.mean_depth, Some(1.0));
        assert_eq!(summary.coverage_regime.as_deref(), Some("low_pass"));
        let regime = summary.regime.expect("coverage regime");
        assert_eq!(regime.regime_class, BamCoverageRegimeClassV1::LowPass);
        assert!((regime.breadth_1x - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn estimate_bam_stage_resources_distinguishes_real_and_synthetic_inputs() {
        let real = estimate_bam_stage_resources_with_origin(
            "bam.markdup",
            48_u64 * 1024 * 1024 * 1024,
            BamInputOriginV1::Real,
        );
        let synthetic = estimate_bam_stage_resources_with_origin(
            "bam.markdup",
            48_u64 * 1024 * 1024 * 1024,
            BamInputOriginV1::Synthetic,
        );

        assert_eq!(real.input_scale, BamInputScaleV1::Large);
        assert_eq!(real.input_origin, BamInputOriginV1::Real);
        assert_eq!(synthetic.input_origin, BamInputOriginV1::Synthetic);
        assert!(real.memory_gb > synthetic.memory_gb);
        assert!(real.disk_gb > synthetic.disk_gb);
        assert!(real.scratch_gb > synthetic.scratch_gb);
        assert!(real.walltime_minutes > synthetic.walltime_minutes);
    }

    #[test]
    fn estimate_bam_stage_resources_captures_stage_specific_io_and_walltime() {
        let sort = estimate_bam_stage_resources("bam.sort_index", 8_u64 * 1024 * 1024 * 1024);
        let coverage = estimate_bam_stage_resources("bam.coverage", 8_u64 * 1024 * 1024 * 1024);
        let damage = estimate_bam_stage_resources("bam.damage", 8_u64 * 1024 * 1024 * 1024);

        assert!(sort.walltime_minutes > coverage.walltime_minutes);
        assert_eq!(sort.io_profile, "io_heavy_sequential_plus_temp_shuffle");
        assert_eq!(coverage.io_profile, "mixed_random_and_streaming_reads");
        assert_eq!(damage.io_profile, "streaming_reads_with_sidecar_metrics");
    }
}
