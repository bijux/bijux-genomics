use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VcfScientificDriftChangeKind {
    DefaultsChange,
    BackendChange,
    FilterPolicyChange,
    NormalizationPolicyChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftSnapshotV1 {
    pub label: String,
    pub stage_id: String,
    pub tool_id: String,
    pub backend_version: Option<String>,
    pub defaults_fingerprint: Option<String>,
    pub normalization_policy_id: Option<String>,
    pub filter_policy_id: Option<String>,
    pub metrics: BTreeMap<String, f64>,
    pub artifacts: BTreeMap<String, String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftMetricDeltaV1 {
    pub metric_id: String,
    pub baseline_value: f64,
    pub candidate_value: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftArtifactDeltaV1 {
    pub artifact_id: String,
    pub baseline_hash: Option<String>,
    pub candidate_hash: Option<String>,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub baseline_label: String,
    pub candidate_label: String,
    pub baseline_tool_id: String,
    pub candidate_tool_id: String,
    pub change_kinds: Vec<VcfScientificDriftChangeKind>,
    pub metric_deltas: Vec<VcfScientificDriftMetricDeltaV1>,
    pub artifact_deltas: Vec<VcfScientificDriftArtifactDeltaV1>,
    pub downstream_risks: Vec<String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfValidationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_vcf: PathBuf,
    pub record_count: u64,
    pub sample_count: u32,
    pub header_valid: bool,
    pub sorted_records: bool,
    pub has_index: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfStatsWorkflowSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub variant_count: u64,
    pub sample_count: u32,
    pub snv_count: u64,
    pub indel_count: u64,
    #[serde(default)]
    pub ti_tv_ratio: Option<f64>,
    pub missing_genotype_calls: u64,
    #[serde(default)]
    pub filter_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub per_sample_missingness: BTreeMap<String, f64>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfFilterConsequenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub filter_expression: String,
    pub variants_in: u64,
    pub variants_retained: u64,
    pub variants_removed: u64,
    #[serde(default)]
    pub reason_counts: BTreeMap<String, u64>,
    pub output_subset_identity: String,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfNormalizationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub records_in: u64,
    pub records_out: u64,
    pub records_changed: u64,
    pub split_multiallelic_records: u64,
    pub duplicate_records_after_normalization: u64,
    pub raw_view_preserved: bool,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct VcfReferenceContextResolutionV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub reference_build: String,
    pub panel_build: String,
    #[serde(default)]
    pub genetic_map_build: Option<String>,
    pub contigs_observed: u32,
    pub alias_mappings_used: u32,
    pub fasta_present: bool,
    pub fai_present: bool,
    pub panel_compatible: bool,
    pub genetic_map_compatible: bool,
    pub passes: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfDamageFilterSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub action: String,
    pub prerequisites_passed: bool,
    pub variants_in: u64,
    pub damage_risk_sites: u64,
    pub removed_sites: u64,
    pub masked_sites: u64,
    pub annotated_sites: u64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfCallingBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub mode: String,
    pub prerequisites_passed: bool,
    pub confidence: f64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfLikelihoodWorkflowBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub prerequisites_passed: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfGenotypeTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub input_vcf: PathBuf,
    pub sample_count: u32,
    pub variant_count: u64,
    #[serde(default)]
    pub observed_ploidy_widths: Vec<u32>,
    pub called_calls: u64,
    pub missing_calls: u64,
    pub reference_only_calls: u64,
    pub mixed_allele_calls: u64,
    pub alternate_only_calls: u64,
    pub phased_calls: u64,
    pub unphased_calls: u64,
    #[serde(default)]
    pub per_sample_missingness: BTreeMap<String, f64>,
    #[serde(default)]
    pub likelihood_fields_present: Vec<String>,
    #[serde(default)]
    pub sites_with_likelihood_values: BTreeMap<String, u64>,
    #[serde(default)]
    pub sites_missing_likelihood_values: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfFilterOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub input_vcf: PathBuf,
    pub sample_count: u32,
    pub variant_count: u64,
    pub pass_variant_count: u64,
    pub failed_variant_count: u64,
    #[serde(default)]
    pub observed_filter_ids: Vec<String>,
    #[serde(default)]
    pub per_filter_variant_count: BTreeMap<String, u64>,
    #[serde(default)]
    pub per_filter_sites: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub pass_sites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfPhasingOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub input_vcf: PathBuf,
    pub sample_count: u32,
    #[serde(default)]
    pub sample_ids: Vec<String>,
    pub variant_count: u64,
    pub called_genotype_count: u64,
    pub phased_genotype_count: u64,
    pub unphased_genotype_count: u64,
    pub phase_set_count: u64,
    #[serde(default)]
    pub phase_sets_by_sample: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfImputationOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub input_vcf: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truth_vcf: Option<PathBuf>,
    pub sample_count: u32,
    #[serde(default)]
    pub sample_ids: Vec<String>,
    pub variant_count: u64,
    pub sites_with_info_score: u64,
    pub sites_with_r2_score: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean_info_score: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mean_r2_score: Option<f64>,
    pub masked_truth_site_count: u64,
    pub masked_truth_match_count: u64,
    pub masked_truth_mismatch_count: u64,
    pub unresolved_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genotype_concordance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfRohSegmentTruthRowV1 {
    pub sample_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
    pub variant_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfRohSampleTruthRowV1 {
    pub sample_id: String,
    pub segment_count: u64,
    pub total_length: u64,
    pub mean_length: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfRohOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub status: String,
    pub sample_count: u32,
    pub segment_count: u64,
    pub total_length: u64,
    #[serde(default)]
    pub sample_ids: Vec<String>,
    #[serde(default)]
    pub segments: Vec<VcfRohSegmentTruthRowV1>,
    #[serde(default)]
    pub per_sample_summary: Vec<VcfRohSampleTruthRowV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfIbdPairTruthRowV1 {
    pub sample_a: String,
    pub sample_b: String,
    pub segment_count: u64,
    pub total_length: f64,
    pub overlap_marker_count: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfIbdInsufficientOverlapTruthV1 {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insufficient_reason: Option<String>,
    pub filtered_segment_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfIbdOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insufficient_reason: Option<String>,
    pub pair_count: u64,
    pub retained_segment_count: u64,
    pub total_length: f64,
    pub overlap_marker_total: u64,
    #[serde(default)]
    pub sample_ids: Vec<String>,
    #[serde(default)]
    pub rows: Vec<VcfIbdPairTruthRowV1>,
    pub insufficient_overlap_probe: VcfIbdInsufficientOverlapTruthV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfDemographyEstimateTruthRowV1 {
    pub generation: u64,
    pub ne: f64,
    pub ci_low: f64,
    pub ci_high: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfDemographyInsufficientDataTruthV1 {
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insufficient_reason: Option<String>,
    #[serde(default)]
    pub time_bins: Vec<u64>,
    #[serde(default)]
    pub ne_estimates: Vec<VcfDemographyEstimateTruthRowV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfDemographyOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub method: String,
    pub inference_status: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insufficient_reason: Option<String>,
    pub estimate_count: u64,
    #[serde(default)]
    pub time_bins: Vec<u64>,
    #[serde(default)]
    pub ne_estimates: Vec<VcfDemographyEstimateTruthRowV1>,
    pub insufficient_data_probe: VcfDemographyInsufficientDataTruthV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfPopulationCoordinateTruthRowV1 {
    pub sample_id: String,
    pub population_id: String,
    pub sex: String,
    pub role: String,
    pub pc1: f64,
    pub pc2: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfAdmixtureOutputTruthRowV1 {
    pub sample_id: String,
    pub population_id: String,
    pub sex: String,
    pub role: String,
    pub status: String,
    pub dominant_cluster: String,
    pub dominant_fraction: f64,
    #[serde(default)]
    pub cluster_fractions: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfPopulationStructureOutputTruthRowV1 {
    pub sample_id: String,
    pub population_id: String,
    pub sex: String,
    pub role: String,
    pub dominant_cluster: String,
    pub dominant_fraction: f64,
    pub pc1: f64,
    pub pc2: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfPopulationDistanceTruthRowV1 {
    pub left_sample_id: String,
    pub right_sample_id: String,
    pub left_population_id: String,
    pub right_population_id: String,
    pub distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfPcaOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub sample_metadata_path: PathBuf,
    pub sample_count: u32,
    pub joined_sample_count: u32,
    pub pair_count: u64,
    #[serde(default)]
    pub population_ids: Vec<String>,
    #[serde(default)]
    pub rows: Vec<VcfPopulationCoordinateTruthRowV1>,
    #[serde(default)]
    pub pairwise_distances: Vec<VcfPopulationDistanceTruthRowV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfAdmixtureOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub sample_metadata_path: PathBuf,
    pub selected_k: u64,
    pub sample_count: u32,
    pub joined_sample_count: u32,
    pub population_count: u64,
    #[serde(default)]
    pub cluster_headers: Vec<String>,
    #[serde(default)]
    pub population_ids: Vec<String>,
    #[serde(default)]
    pub rows: Vec<VcfAdmixtureOutputTruthRowV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfPopulationStructureOutputTruthSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub sample_metadata_path: PathBuf,
    pub sample_count: u32,
    pub joined_sample_count: u32,
    pub pair_count: u64,
    pub within_population_pair_count: u64,
    pub cross_population_pair_count: u64,
    pub min_pc_distance: f64,
    pub max_pc_distance: f64,
    pub mean_pc_distance: f64,
    #[serde(default)]
    pub population_ids: Vec<String>,
    #[serde(default)]
    pub sample_groups: Vec<VcfPopulationStructureOutputTruthRowV1>,
    #[serde(default)]
    pub pairwise_distances: Vec<VcfPopulationDistanceTruthRowV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfPhasingWorkflowBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub prerequisites_passed: bool,
    pub panel_compatible: bool,
    pub genetic_map_compatible: bool,
    pub confidence: f64,
    pub sample_count: u32,
    pub minimum_samples: u32,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}
