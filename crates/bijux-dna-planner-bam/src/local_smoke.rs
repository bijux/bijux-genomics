use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_bam::{
    params::{
        ComplexityEffectiveParams, CoverageEffectiveParams, DuplicateAction, FilterEffectiveParams,
        MarkDupEffectiveParams, OpticalDuplicatePolicy, UmiPolicy,
    },
    types::BedRegions,
    BamStage,
};
use serde::Deserialize;

use crate::selection::{allowed_tools_for_stage, load_bam_domain_tool_planning_spec};

const LOCAL_VALIDATE_CONFIG_PATH: &str = "configs/bench/local/bam-validate.toml";
const DEFAULT_LOCAL_VALIDATE_OUTPUT_DIR: &str = "target/local-smoke/bam.validate";
const LOCAL_QC_PRE_CONFIG_PATH: &str = "configs/bench/local/bam-qc-pre.toml";
const DEFAULT_LOCAL_QC_PRE_OUTPUT_DIR: &str = "target/local-smoke/bam.qc_pre";
const LOCAL_MAPPING_SUMMARY_CONFIG_PATH: &str = "configs/bench/local/bam-mapping-summary.toml";
const DEFAULT_LOCAL_MAPPING_SUMMARY_OUTPUT_DIR: &str = "target/local-smoke/bam.mapping_summary";
const LOCAL_FILTER_CONFIG_PATH: &str = "configs/bench/local/bam-filter.toml";
const DEFAULT_LOCAL_FILTER_OUTPUT_DIR: &str = "target/local-smoke/bam.filter";
const LOCAL_MAPQ_FILTER_CONFIG_PATH: &str = "configs/bench/local/bam-mapq-filter.toml";
const DEFAULT_LOCAL_MAPQ_FILTER_OUTPUT_DIR: &str = "target/local-smoke/bam.mapq_filter";
const LOCAL_LENGTH_FILTER_CONFIG_PATH: &str = "configs/bench/local/bam-length-filter.toml";
const DEFAULT_LOCAL_LENGTH_FILTER_OUTPUT_DIR: &str = "target/local-smoke/bam.length_filter";
const LOCAL_MARKDUP_CONFIG_PATH: &str = "configs/bench/local/bam-markdup.toml";
const DEFAULT_LOCAL_MARKDUP_OUTPUT_DIR: &str = "target/local-smoke/bam.markdup";
const LOCAL_DUPLICATION_METRICS_CONFIG_PATH: &str =
    "configs/bench/local/bam-duplication-metrics.toml";
const DEFAULT_LOCAL_DUPLICATION_METRICS_OUTPUT_DIR: &str =
    "target/local-smoke/bam.duplication_metrics";
const LOCAL_COMPLEXITY_CONFIG_PATH: &str = "configs/bench/local/bam-complexity.toml";
const DEFAULT_LOCAL_COMPLEXITY_OUTPUT_DIR: &str = "target/local-smoke/bam.complexity";
const LOCAL_COVERAGE_CONFIG_PATH: &str = "configs/bench/local/bam-coverage.toml";
const DEFAULT_LOCAL_COVERAGE_OUTPUT_DIR: &str = "target/local-smoke/bam.coverage";
const LOCAL_INSERT_SIZE_CONFIG_PATH: &str = "configs/bench/local/bam-insert-size.toml";
const DEFAULT_LOCAL_INSERT_SIZE_OUTPUT_DIR: &str = "target/local-smoke/bam.insert_size";
const LOCAL_GC_BIAS_CONFIG_PATH: &str = "configs/bench/local/bam-gc-bias.toml";
const DEFAULT_LOCAL_GC_BIAS_OUTPUT_DIR: &str = "target/local-smoke/bam.gc_bias";
const LOCAL_ENDOGENOUS_CONTENT_CONFIG_PATH: &str =
    "configs/bench/local/bam-endogenous-content.toml";
const DEFAULT_LOCAL_ENDOGENOUS_CONTENT_OUTPUT_DIR: &str =
    "target/local-smoke/bam.endogenous_content";
const LOCAL_OVERLAP_CORRECTION_CONFIG_PATH: &str =
    "configs/bench/local/bam-overlap-correction.toml";
const DEFAULT_LOCAL_OVERLAP_CORRECTION_OUTPUT_DIR: &str =
    "target/local-smoke/bam.overlap_correction";

#[derive(Debug, Clone)]
pub struct LocalValidateSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub bam_index: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub expect_pass: bool,
    pub required_refusal_codes: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalQcPreSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_unmapped_reads: u64,
    pub expected_duplicate_flagged_reads: u64,
    pub expected_contigs: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalMappingSummarySmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_mapping_fraction: f64,
    pub expected_reference_name: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalFilterSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_input_reads: u64,
    pub expected_kept_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_active_filters: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalMapqFilterSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub mapq_threshold: u8,
    pub expected_input_reads: u64,
    pub expected_kept_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_mapped_reads_removed: u64,
    pub expected_mapped_fraction_retained: f64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalLengthFilterSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub min_length: u32,
    pub expected_input_reads: u64,
    pub expected_kept_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_observed_min_length: u32,
    pub expected_observed_max_length: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalMarkdupSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_input_reads: u64,
    pub expected_output_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_duplicate_reads_before: u64,
    pub expected_duplicate_reads_after: u64,
    pub expected_newly_marked_reads: u64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalDuplicationMetricsSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_examined_reads: u64,
    pub expected_duplicate_reads: u64,
    pub expected_duplicate_fraction: f64,
    pub expected_estimated_library_size: Option<u64>,
    pub expected_insufficient_library_size_reason: Option<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalComplexitySmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub min_reads: u64,
    pub projection_points: Vec<u64>,
    pub expected_observed_total_reads: u64,
    pub expected_observed_unique_reads: u64,
    pub expected_estimated_unique_reads: Option<u64>,
    pub expected_insufficient_data_reason: Option<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalCoverageSmokeExpectedRow {
    pub region_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
    pub mean_depth: f64,
    pub breadth_1x: f64,
    pub covered_bases: u64,
}

#[derive(Debug, Clone)]
pub struct LocalCoverageSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub regions: PathBuf,
    pub depth_thresholds: Vec<u32>,
    pub expected_coverage_regime: String,
    pub expected_rows: Vec<LocalCoverageSmokeExpectedRow>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalInsertSizeSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_read_pairs: u64,
    pub expected_median_insert_size: f64,
    pub expected_mean_insert_size: f64,
    pub expected_min_insert_size: u64,
    pub expected_max_insert_size: u64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalGcBiasSmokeExpectedRow {
    pub gc_bin: u8,
    pub normalized_coverage: f64,
    pub windows: u64,
    pub read_starts: u64,
}

#[derive(Debug, Clone)]
pub struct LocalGcBiasSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub window_size: u32,
    pub expected_rows: Vec<LocalGcBiasSmokeExpectedRow>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalEndogenousContentSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub host_reference_scope: String,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_endogenous_fraction: f64,
    pub expected_method: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalOverlapCorrectionSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_pair_count: u64,
    pub expected_corrected_pairs: u64,
    pub expected_corrected_overlap_bases: u64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalValidateSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalValidateSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalQcPreSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalQcPreSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMappingSummarySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMappingSummarySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalFilterSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMapqFilterSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMapqFilterSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalLengthFilterSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalLengthFilterSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMarkdupSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMarkdupSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDuplicationMetricsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDuplicationMetricsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalComplexitySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalComplexitySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalCoverageSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalCoverageSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalInsertSizeSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalInsertSizeSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalGcBiasSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalGcBiasSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalEndogenousContentSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalEndogenousContentSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalOverlapCorrectionSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalOverlapCorrectionSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalValidateSmokeCase {
    sample_id: String,
    bam: PathBuf,
    #[serde(default)]
    bam_index: Option<PathBuf>,
    #[serde(default)]
    reference_fasta: Option<PathBuf>,
    #[serde(default = "default_expect_pass")]
    expect_pass: bool,
    #[serde(default)]
    required_refusal_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LocalQcPreSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_unmapped_reads: u64,
    expected_duplicate_flagged_reads: u64,
    expected_contigs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LocalMappingSummarySmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_mapping_fraction: f64,
    expected_reference_name: String,
}

#[derive(Debug, Deserialize)]
struct LocalFilterSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_input_reads: u64,
    expected_kept_reads: u64,
    expected_removed_reads: u64,
    expected_active_filters: Vec<String>,
    mapq_threshold: u8,
    #[serde(default)]
    include_flags: Vec<u16>,
    #[serde(default)]
    exclude_flags: Vec<u16>,
    min_length: u32,
    remove_duplicates: bool,
    base_quality_threshold: u8,
}

#[derive(Debug, Deserialize)]
struct LocalMapqFilterSmokeCase {
    sample_id: String,
    bam: PathBuf,
    mapq_threshold: u8,
    expected_input_reads: u64,
    expected_kept_reads: u64,
    expected_removed_reads: u64,
    expected_mapped_reads_removed: u64,
    expected_mapped_fraction_retained: f64,
}

#[derive(Debug, Deserialize)]
struct LocalLengthFilterSmokeCase {
    sample_id: String,
    bam: PathBuf,
    min_length: u32,
    expected_input_reads: u64,
    expected_kept_reads: u64,
    expected_removed_reads: u64,
    expected_observed_min_length: u32,
    expected_observed_max_length: u32,
}

#[derive(Debug, Deserialize)]
struct LocalMarkdupSmokeCase {
    sample_id: String,
    bam: PathBuf,
    duplicate_action: DuplicateAction,
    optical_duplicates: OpticalDuplicatePolicy,
    umi_policy: UmiPolicy,
    expected_input_reads: u64,
    expected_output_reads: u64,
    expected_removed_reads: u64,
    expected_duplicate_reads_before: u64,
    expected_duplicate_reads_after: u64,
    expected_newly_marked_reads: u64,
}

#[derive(Debug, Deserialize)]
struct LocalDuplicationMetricsSmokeCase {
    sample_id: String,
    bam: PathBuf,
    optical_duplicates: OpticalDuplicatePolicy,
    umi_policy: UmiPolicy,
    duplicate_action: DuplicateAction,
    expected_examined_reads: u64,
    expected_duplicate_reads: u64,
    expected_duplicate_fraction: f64,
    #[serde(default)]
    expected_estimated_library_size: Option<u64>,
    #[serde(default)]
    expected_insufficient_library_size_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocalComplexitySmokeCase {
    sample_id: String,
    bam: PathBuf,
    min_reads: u64,
    projection_points: Vec<u64>,
    expected_observed_total_reads: u64,
    expected_observed_unique_reads: u64,
    #[serde(default)]
    expected_estimated_unique_reads: Option<u64>,
    #[serde(default)]
    expected_insufficient_data_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocalCoverageSmokeCase {
    sample_id: String,
    bam: PathBuf,
    regions: PathBuf,
    depth_thresholds: Vec<u32>,
    expected_coverage_regime: String,
    expected_rows: Vec<LocalCoverageSmokeExpectedRow>,
}

#[derive(Debug, Deserialize)]
struct LocalInsertSizeSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_read_pairs: u64,
    expected_median_insert_size: f64,
    expected_mean_insert_size: f64,
    expected_min_insert_size: u64,
    expected_max_insert_size: u64,
}

#[derive(Debug, Deserialize)]
struct LocalGcBiasSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference: PathBuf,
    window_size: u32,
    expected_rows: Vec<LocalGcBiasSmokeExpectedRow>,
}

#[derive(Debug, Deserialize)]
struct LocalEndogenousContentSmokeCase {
    sample_id: String,
    bam: PathBuf,
    host_reference_scope: String,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_endogenous_fraction: f64,
    expected_method: String,
}

#[derive(Debug, Deserialize)]
struct LocalOverlapCorrectionSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_pair_count: u64,
    expected_corrected_pairs: u64,
    expected_corrected_overlap_bases: u64,
}

const fn default_expect_pass() -> bool {
    true
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.validate` plans cannot be built.
pub fn local_validate_smoke_plans(repo_root: &Path) -> Result<Vec<LocalValidateSmokeCasePlan>> {
    let config = load_local_validate_smoke_config(repo_root)?;
    ensure_unique_sample_ids(&config.cases)?;

    let stage = BamStage::Validate;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.validate tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_VALIDATE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_validate_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.qc_pre` plans cannot be built.
pub fn local_qc_pre_smoke_plans(repo_root: &Path) -> Result<Vec<LocalQcPreSmokeCasePlan>> {
    let config = load_local_qc_pre_smoke_config(repo_root)?;
    ensure_unique_qc_pre_sample_ids(&config.cases)?;

    let stage = BamStage::QcPre;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.qc_pre tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_QC_PRE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_qc_pre_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.mapping_summary` plans cannot be built.
pub fn local_mapping_summary_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMappingSummarySmokeCasePlan>> {
    let config = load_local_mapping_summary_smoke_config(repo_root)?;
    ensure_unique_mapping_summary_sample_ids(&config.cases)?;

    let stage = BamStage::MappingSummary;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MAPPING_SUMMARY_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_mapping_summary_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.filter` plans cannot be built.
pub fn local_filter_smoke_plans(repo_root: &Path) -> Result<Vec<LocalFilterSmokeCasePlan>> {
    let config = load_local_filter_smoke_config(repo_root)?;
    ensure_unique_filter_sample_ids(&config.cases)?;

    let stage = BamStage::Filter;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.filter tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_FILTER_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_filter_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.mapq_filter` plans cannot be built.
pub fn local_mapq_filter_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMapqFilterSmokeCasePlan>> {
    let config = load_local_mapq_filter_smoke_config(repo_root)?;
    ensure_unique_mapq_filter_sample_ids(&config.cases)?;

    let stage = BamStage::MapqFilter;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MAPQ_FILTER_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_mapq_filter_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.length_filter` plans cannot be built.
pub fn local_length_filter_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalLengthFilterSmokeCasePlan>> {
    let config = load_local_length_filter_smoke_config(repo_root)?;
    ensure_unique_length_filter_sample_ids(&config.cases)?;

    let stage = BamStage::LengthFilter;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.length_filter tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_LENGTH_FILTER_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_length_filter_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.markdup` plans cannot be built.
pub fn local_markdup_smoke_plans(repo_root: &Path) -> Result<Vec<LocalMarkdupSmokeCasePlan>> {
    let config = load_local_markdup_smoke_config(repo_root)?;
    ensure_unique_markdup_sample_ids(&config.cases)?;

    let stage = BamStage::Markdup;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.markdup tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MARKDUP_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_markdup_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.duplication_metrics` plans cannot be built.
pub fn local_duplication_metrics_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDuplicationMetricsSmokeCasePlan>> {
    let config = load_local_duplication_metrics_smoke_config(repo_root)?;
    ensure_unique_duplication_metrics_sample_ids(&config.cases)?;

    let stage = BamStage::DuplicationMetrics;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DUPLICATION_METRICS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_duplication_metrics_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.complexity` plans cannot be built.
pub fn local_complexity_smoke_plans(repo_root: &Path) -> Result<Vec<LocalComplexitySmokeCasePlan>> {
    let config = load_local_complexity_smoke_config(repo_root)?;
    ensure_unique_complexity_sample_ids(&config.cases)?;

    let stage = BamStage::Complexity;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.complexity tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_COMPLEXITY_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_complexity_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.coverage` plans cannot be built.
pub fn local_coverage_smoke_plans(repo_root: &Path) -> Result<Vec<LocalCoverageSmokeCasePlan>> {
    let config = load_local_coverage_smoke_config(repo_root)?;
    ensure_unique_coverage_sample_ids(&config.cases)?;

    let stage = BamStage::Coverage;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.coverage tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_COVERAGE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_coverage_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.insert_size` plans cannot be built.
pub fn local_insert_size_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalInsertSizeSmokeCasePlan>> {
    let config = load_local_insert_size_smoke_config(repo_root)?;
    ensure_unique_insert_size_sample_ids(&config.cases)?;

    let stage = BamStage::InsertSize;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.insert_size tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_INSERT_SIZE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_insert_size_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.gc_bias` plans cannot be built.
pub fn local_gc_bias_smoke_plans(repo_root: &Path) -> Result<Vec<LocalGcBiasSmokeCasePlan>> {
    let config = load_local_gc_bias_smoke_config(repo_root)?;
    ensure_unique_gc_bias_sample_ids(&config.cases)?;

    let stage = BamStage::GcBias;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.gc_bias tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_GC_BIAS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_gc_bias_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.endogenous_content` plans cannot be built.
pub fn local_endogenous_content_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalEndogenousContentSmokeCasePlan>> {
    let config = load_local_endogenous_content_smoke_config(repo_root)?;
    ensure_unique_endogenous_content_sample_ids(&config.cases)?;

    let stage = BamStage::EndogenousContent;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_ENDOGENOUS_CONTENT_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_endogenous_content_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.overlap_correction` plans cannot be built.
pub fn local_overlap_correction_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalOverlapCorrectionSmokeCasePlan>> {
    let config = load_local_overlap_correction_smoke_config(repo_root)?;
    ensure_unique_overlap_correction_sample_ids(&config.cases)?;

    let stage = BamStage::OverlapCorrection;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_OVERLAP_CORRECTION_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_overlap_correction_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_validate_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalValidateSmokeCase,
) -> Result<LocalValidateSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.validate BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }

    if let Some(bam_index) = case.bam_index.as_ref() {
        let bam_index_abs = repo_root.join(bam_index);
        if !bam_index_abs.is_file() {
            return Err(anyhow!(
                "local-smoke bam.validate BAM index fixture is missing: {}",
                bam_index_abs.display()
            ));
        }
    }

    if let Some(reference_fasta) = case.reference_fasta.as_ref() {
        let reference_abs = repo_root.join(reference_fasta);
        if !reference_abs.is_file() {
            return Err(anyhow!(
                "local-smoke bam.validate reference fixture is missing: {}",
                reference_abs.display()
            ));
        }
    }

    if case.expect_pass && !case.required_refusal_codes.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.validate passing case `{}` must not declare refusal expectations",
            case.sample_id
        ));
    }
    if !case.expect_pass && case.required_refusal_codes.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.validate refusal case `{}` must declare at least one expected refusal code",
            case.sample_id
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::validate::plan(
        tool_spec,
        &case.bam,
        case.bam_index.as_deref(),
        case.reference_fasta.as_deref(),
        &out_dir,
    )?;

    Ok(LocalValidateSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        bam_index: case.bam_index,
        reference_fasta: case.reference_fasta,
        expect_pass: case.expect_pass,
        required_refusal_codes: case.required_refusal_codes,
        plan,
    })
}

fn build_local_qc_pre_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalQcPreSmokeCase,
) -> Result<LocalQcPreSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.qc_pre BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_contigs.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.qc_pre case `{}` must declare at least one expected contig",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads + case.expected_unmapped_reads != case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.qc_pre case `{}` must satisfy mapped + unmapped == total",
            case.sample_id
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::qc_pre::plan(tool_spec, &case.bam, &out_dir)?;

    Ok(LocalQcPreSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_unmapped_reads: case.expected_unmapped_reads,
        expected_duplicate_flagged_reads: case.expected_duplicate_flagged_reads,
        expected_contigs: case.expected_contigs,
        plan,
    })
}

fn build_local_mapping_summary_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMappingSummarySmokeCase,
) -> Result<LocalMappingSummarySmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_reference_name.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must declare a non-empty expected reference name",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads > case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` cannot declare mapped reads greater than total reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_mapping_fraction) {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must declare mapping fraction within [0, 1]",
            case.sample_id
        ));
    }
    let derived_fraction = if case.expected_total_reads == 0 {
        0.0
    } else {
        case.expected_mapped_reads as f64 / case.expected_total_reads as f64
    };
    if (derived_fraction - case.expected_mapping_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must keep expected mapping fraction aligned with mapped and total reads",
            case.sample_id
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::mapping_summary::plan(tool_spec, &case.bam, &out_dir)?;

    Ok(LocalMappingSummarySmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_mapping_fraction: case.expected_mapping_fraction,
        expected_reference_name: case.expected_reference_name,
        plan,
    })
}

fn build_local_filter_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalFilterSmokeCase,
) -> Result<LocalFilterSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.filter BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_kept_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.filter case `{}` cannot declare kept reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_kept_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.filter case `{}` must keep expected removed reads aligned with input and kept reads",
            case.sample_id
        ));
    }
    if case.expected_active_filters.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.filter case `{}` must declare at least one active filter",
            case.sample_id
        ));
    }
    let mut seen_filters = BTreeSet::new();
    for filter in &case.expected_active_filters {
        if filter.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.filter case `{}` must not declare empty active filter names",
                case.sample_id
            ));
        }
        if !seen_filters.insert(filter.clone()) {
            return Err(anyhow!(
                "local-smoke bam.filter case `{}` declared duplicate active filter `{}`",
                case.sample_id,
                filter
            ));
        }
    }

    let params = FilterEffectiveParams {
        mapq_threshold: case.mapq_threshold,
        include_flags: case.include_flags,
        exclude_flags: case.exclude_flags,
        min_length: case.min_length,
        remove_duplicates: case.remove_duplicates,
        base_quality_threshold: case.base_quality_threshold,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::filter::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalFilterSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_input_reads: case.expected_input_reads,
        expected_kept_reads: case.expected_kept_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_active_filters: case.expected_active_filters,
        plan,
    })
}

fn build_local_mapq_filter_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMapqFilterSmokeCase,
) -> Result<LocalMapqFilterSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.mapq_threshold == 0 {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` must declare a non-zero mapq_threshold",
            case.sample_id
        ));
    }
    if case.expected_kept_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` cannot declare kept reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_kept_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` must keep expected removed reads aligned with input and kept reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_mapped_fraction_retained) {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` must declare mapped_fraction_retained within [0, 1]",
            case.sample_id
        ));
    }

    let params = FilterEffectiveParams {
        mapq_threshold: case.mapq_threshold,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 0,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        crate::tool_adapters::bam::mapq_filter::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalMapqFilterSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        mapq_threshold: case.mapq_threshold,
        expected_input_reads: case.expected_input_reads,
        expected_kept_reads: case.expected_kept_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_mapped_reads_removed: case.expected_mapped_reads_removed,
        expected_mapped_fraction_retained: case.expected_mapped_fraction_retained,
        plan,
    })
}

fn build_local_length_filter_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalLengthFilterSmokeCase,
) -> Result<LocalLengthFilterSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.length_filter BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.min_length == 0 {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must declare a non-zero min_length",
            case.sample_id
        ));
    }
    if case.expected_kept_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` cannot declare kept reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_kept_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must keep expected removed reads aligned with input and kept reads",
            case.sample_id
        ));
    }
    if case.expected_observed_min_length > case.expected_observed_max_length {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must declare observed min length less than or equal to observed max length",
            case.sample_id
        ));
    }
    if case.expected_observed_min_length < case.min_length {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must keep observed min length at or above the filter threshold",
            case.sample_id
        ));
    }

    let params = FilterEffectiveParams {
        mapq_threshold: 0,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: case.min_length,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        crate::tool_adapters::bam::length_filter::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalLengthFilterSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        min_length: case.min_length,
        expected_input_reads: case.expected_input_reads,
        expected_kept_reads: case.expected_kept_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_observed_min_length: case.expected_observed_min_length,
        expected_observed_max_length: case.expected_observed_max_length,
        plan,
    })
}

fn build_local_markdup_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMarkdupSmokeCase,
) -> Result<LocalMarkdupSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.markdup BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_output_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare output reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_output_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` must keep expected removed reads aligned with input and output reads",
            case.sample_id
        ));
    }
    if case.expected_duplicate_reads_before > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare duplicate reads before greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_duplicate_reads_after > case.expected_output_reads {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare duplicate reads after greater than output reads",
            case.sample_id
        ));
    }
    if case.expected_newly_marked_reads > case.expected_duplicate_reads_after {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare newly marked reads greater than duplicate reads after processing",
            case.sample_id
        ));
    }
    match case.duplicate_action {
        DuplicateAction::Mark => {
            if case.expected_removed_reads != 0 {
                return Err(anyhow!(
                    "local-smoke bam.markdup case `{}` must not remove reads when duplicate_action is mark",
                    case.sample_id
                ));
            }
        }
        DuplicateAction::Remove => {
            if case.expected_newly_marked_reads != 0 {
                return Err(anyhow!(
                    "local-smoke bam.markdup case `{}` must not declare newly marked reads when duplicate_action is remove",
                    case.sample_id
                ));
            }
        }
    }

    let params = MarkDupEffectiveParams {
        optical_duplicates: case.optical_duplicates,
        umi_policy: case.umi_policy,
        duplicate_action: case.duplicate_action,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::markdup::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalMarkdupSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_input_reads: case.expected_input_reads,
        expected_output_reads: case.expected_output_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_duplicate_reads_before: case.expected_duplicate_reads_before,
        expected_duplicate_reads_after: case.expected_duplicate_reads_after,
        expected_newly_marked_reads: case.expected_newly_marked_reads,
        plan,
    })
}

fn build_local_duplication_metrics_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDuplicationMetricsSmokeCase,
) -> Result<LocalDuplicationMetricsSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_duplicate_reads > case.expected_examined_reads {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` cannot declare duplicate reads greater than examined reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_duplicate_fraction) {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must declare duplicate fraction within [0, 1]",
            case.sample_id
        ));
    }
    let derived_fraction = if case.expected_examined_reads == 0 {
        0.0
    } else {
        case.expected_duplicate_reads as f64 / case.expected_examined_reads as f64
    };
    if (derived_fraction - case.expected_duplicate_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must keep duplicate fraction aligned with examined and duplicate reads",
            case.sample_id
        ));
    }
    if case.expected_estimated_library_size.is_some()
        == case.expected_insufficient_library_size_reason.is_some()
    {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must declare exactly one of expected_estimated_library_size or expected_insufficient_library_size_reason",
            case.sample_id
        ));
    }
    if case.expected_insufficient_library_size_reason.as_deref().is_some_and(str::is_empty) {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must not declare an empty insufficiency reason",
            case.sample_id
        ));
    }

    let params = MarkDupEffectiveParams {
        optical_duplicates: case.optical_duplicates,
        umi_policy: case.umi_policy,
        duplicate_action: case.duplicate_action,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::duplication_metrics::plan(
        tool_spec, &case.bam, &out_dir, &params,
    )?;

    Ok(LocalDuplicationMetricsSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_examined_reads: case.expected_examined_reads,
        expected_duplicate_reads: case.expected_duplicate_reads,
        expected_duplicate_fraction: case.expected_duplicate_fraction,
        expected_estimated_library_size: case.expected_estimated_library_size,
        expected_insufficient_library_size_reason: case.expected_insufficient_library_size_reason,
        plan,
    })
}

fn build_local_complexity_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalComplexitySmokeCase,
) -> Result<LocalComplexitySmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.complexity BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.min_reads == 0 {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must declare min_reads greater than zero",
            case.sample_id
        ));
    }
    if case.projection_points.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must declare at least one projection point",
            case.sample_id
        ));
    }
    if case.projection_points.iter().any(|point| *point == 0) {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must keep projection points greater than zero",
            case.sample_id
        ));
    }
    if case.projection_points.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must keep projection points strictly increasing",
            case.sample_id
        ));
    }
    if case.expected_observed_unique_reads > case.expected_observed_total_reads {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` cannot declare unique reads greater than observed total reads",
            case.sample_id
        ));
    }
    if case.expected_estimated_unique_reads.is_some()
        == case.expected_insufficient_data_reason.is_some()
    {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must declare exactly one of expected_estimated_unique_reads or expected_insufficient_data_reason",
            case.sample_id
        ));
    }
    if case
        .expected_estimated_unique_reads
        .is_some_and(|value| value < case.expected_observed_unique_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must keep estimated unique reads greater than or equal to observed unique reads",
            case.sample_id
        ));
    }
    if case.expected_insufficient_data_reason.as_deref().is_some_and(str::is_empty) {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must not declare an empty insufficiency reason",
            case.sample_id
        ));
    }

    let params = ComplexityEffectiveParams {
        min_reads: case.min_reads,
        projection_points: case.projection_points.clone(),
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        crate::tool_adapters::bam::complexity::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalComplexitySmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        min_reads: case.min_reads,
        projection_points: case.projection_points,
        expected_observed_total_reads: case.expected_observed_total_reads,
        expected_observed_unique_reads: case.expected_observed_unique_reads,
        expected_estimated_unique_reads: case.expected_estimated_unique_reads,
        expected_insufficient_data_reason: case.expected_insufficient_data_reason,
        plan,
    })
}

fn build_local_coverage_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalCoverageSmokeCase,
) -> Result<LocalCoverageSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.coverage BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    let regions_abs = repo_root.join(&case.regions);
    if !regions_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.coverage regions fixture is missing: {}",
            regions_abs.display()
        ));
    }
    if case.depth_thresholds.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must declare at least one depth threshold",
            case.sample_id
        ));
    }
    if case.depth_thresholds.iter().any(|threshold| *threshold == 0) {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must keep depth thresholds greater than zero",
            case.sample_id
        ));
    }
    if case.depth_thresholds.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must keep depth thresholds strictly increasing",
            case.sample_id
        ));
    }
    if case.expected_coverage_regime.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must declare a non-empty expected coverage regime",
            case.sample_id
        ));
    }
    if case.expected_rows.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must declare at least one expected region row",
            case.sample_id
        ));
    }

    let mut seen_region_ids = BTreeSet::new();
    for row in &case.expected_rows {
        if row.region_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must not declare empty region identifiers",
                case.sample_id
            ));
        }
        if !seen_region_ids.insert(row.region_id.clone()) {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` declared duplicate region `{}`",
                case.sample_id,
                row.region_id
            ));
        }
        if row.contig.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must not declare empty contig names",
                case.sample_id
            ));
        }
        if row.start == 0 || row.end < row.start {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must keep region coordinates 1-based and ordered",
                case.sample_id
            ));
        }
        if row.length != row.end.saturating_sub(row.start).saturating_add(1) {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must keep expected row length aligned with region coordinates",
                case.sample_id
            ));
        }
        if !(0.0..=1.0).contains(&row.breadth_1x) {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must keep breadth_1x within [0, 1]",
                case.sample_id
            ));
        }
        if row.covered_bases > row.length {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` cannot declare covered bases greater than region length",
                case.sample_id
            ));
        }
    }

    let params = CoverageEffectiveParams {
        regions: Some(BedRegions(case.regions.clone())),
        depth_thresholds: case.depth_thresholds.clone(),
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::coverage::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalCoverageSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        regions: case.regions,
        depth_thresholds: case.depth_thresholds,
        expected_coverage_regime: case.expected_coverage_regime,
        expected_rows: case.expected_rows,
        plan,
    })
}

fn build_local_insert_size_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalInsertSizeSmokeCase,
) -> Result<LocalInsertSizeSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.insert_size BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_read_pairs == 0 {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must declare expected_read_pairs greater than zero",
            case.sample_id
        ));
    }
    if case.expected_min_insert_size == 0 || case.expected_max_insert_size == 0 {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected insert-size bounds greater than zero",
            case.sample_id
        ));
    }
    if case.expected_min_insert_size > case.expected_max_insert_size {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected min insert size less than or equal to expected max insert size",
            case.sample_id
        ));
    }
    if case.expected_mean_insert_size < case.expected_min_insert_size as f64
        || case.expected_mean_insert_size > case.expected_max_insert_size as f64
    {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected mean insert size within the declared bounds",
            case.sample_id
        ));
    }
    if case.expected_median_insert_size < case.expected_min_insert_size as f64
        || case.expected_median_insert_size > case.expected_max_insert_size as f64
    {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected median insert size within the declared bounds",
            case.sample_id
        ));
    }

    let params = CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        crate::tool_adapters::bam::insert_size::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalInsertSizeSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_read_pairs: case.expected_read_pairs,
        expected_median_insert_size: case.expected_median_insert_size,
        expected_mean_insert_size: case.expected_mean_insert_size,
        expected_min_insert_size: case.expected_min_insert_size,
        expected_max_insert_size: case.expected_max_insert_size,
        plan,
    })
}

fn build_local_gc_bias_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalGcBiasSmokeCase,
) -> Result<LocalGcBiasSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.gc_bias BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    let reference_abs = repo_root.join(&case.reference);
    if !reference_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.gc_bias reference fixture is missing: {}",
            reference_abs.display()
        ));
    }
    if case.window_size == 0 {
        return Err(anyhow!(
            "local-smoke bam.gc_bias case `{}` must declare window_size greater than zero",
            case.sample_id
        ));
    }
    if case.expected_rows.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.gc_bias case `{}` must declare at least one expected GC bin row",
            case.sample_id
        ));
    }

    let params = CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::gc_bias::plan(
        tool_spec,
        &case.bam,
        &case.reference,
        &out_dir,
        &params,
    )?;

    Ok(LocalGcBiasSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference: case.reference,
        window_size: case.window_size,
        expected_rows: case.expected_rows,
        plan,
    })
}

fn build_local_endogenous_content_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalEndogenousContentSmokeCase,
) -> Result<LocalEndogenousContentSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.host_reference_scope.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must declare a non-empty host_reference_scope",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads > case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` cannot declare mapped reads greater than total reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_endogenous_fraction) {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must keep expected_endogenous_fraction within [0, 1]",
            case.sample_id
        ));
    }
    if case.expected_method.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must declare a non-empty expected_method",
            case.sample_id
        ));
    }

    let params = bijux_dna_domain_bam::params::EndogenousContentEffectiveParams {
        regions: None,
        depth_thresholds: vec![1],
        host_reference_scope: case.host_reference_scope.clone(),
        host_reference_digest: None,
        refuse_without_host_reference: true,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::endogenous_content::plan(
        tool_spec, &case.bam, &out_dir, &params,
    )?;

    Ok(LocalEndogenousContentSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        host_reference_scope: case.host_reference_scope,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_endogenous_fraction: case.expected_endogenous_fraction,
        expected_method: case.expected_method,
        plan,
    })
}

fn build_local_overlap_correction_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalOverlapCorrectionSmokeCase,
) -> Result<LocalOverlapCorrectionSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_corrected_pairs > case.expected_pair_count {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction case `{}` cannot declare corrected pairs greater than pair count",
            case.sample_id
        ));
    }

    let params = FilterEffectiveParams {
        mapq_threshold: 0,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 0,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::overlap_correction::plan(
        tool_spec, &case.bam, &out_dir, &params,
    )?;

    Ok(LocalOverlapCorrectionSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_pair_count: case.expected_pair_count,
        expected_corrected_pairs: case.expected_corrected_pairs,
        expected_corrected_overlap_bases: case.expected_corrected_overlap_bases,
        plan,
    })
}

fn hydrate_smoke_threads(tool_spec: &mut ToolExecutionSpecV1, threads: Option<u32>) {
    if let Some(threads) = threads {
        tool_spec.resources.threads = threads.max(1);
    } else {
        tool_spec.resources.threads = tool_spec.resources.threads.max(1);
    }
}

fn ensure_unique_sample_ids(cases: &[LocalValidateSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.validate sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.validate sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_qc_pre_sample_ids(cases: &[LocalQcPreSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.qc_pre sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!("duplicate local-smoke bam.qc_pre sample_id `{}`", case.sample_id));
        }
    }
    Ok(())
}

fn ensure_unique_mapping_summary_sample_ids(cases: &[LocalMappingSummarySmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.mapping_summary sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.mapping_summary sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_filter_sample_ids(cases: &[LocalFilterSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.filter sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!("duplicate local-smoke bam.filter sample_id `{}`", case.sample_id));
        }
    }
    Ok(())
}

fn ensure_unique_mapq_filter_sample_ids(cases: &[LocalMapqFilterSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.mapq_filter sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.mapq_filter sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_length_filter_sample_ids(cases: &[LocalLengthFilterSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.length_filter sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.length_filter sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_markdup_sample_ids(cases: &[LocalMarkdupSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.markdup sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.markdup sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_duplication_metrics_sample_ids(
    cases: &[LocalDuplicationMetricsSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.duplication_metrics sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.duplication_metrics sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_complexity_sample_ids(cases: &[LocalComplexitySmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.complexity sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.complexity sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_coverage_sample_ids(cases: &[LocalCoverageSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.coverage sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.coverage sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_insert_size_sample_ids(cases: &[LocalInsertSizeSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.insert_size sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.insert_size sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_gc_bias_sample_ids(cases: &[LocalGcBiasSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.gc_bias sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.gc_bias sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_endogenous_content_sample_ids(
    cases: &[LocalEndogenousContentSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.endogenous_content sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.endogenous_content sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_overlap_correction_sample_ids(
    cases: &[LocalOverlapCorrectionSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.overlap_correction sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.overlap_correction sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_validate_smoke_config(repo_root: &Path) -> Result<LocalValidateSmokeConfig> {
    let path = repo_root.join(LOCAL_VALIDATE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalValidateSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_validate.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.validate schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.validate must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_qc_pre_smoke_config(repo_root: &Path) -> Result<LocalQcPreSmokeConfig> {
    let path = repo_root.join(LOCAL_QC_PRE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalQcPreSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_qc_pre.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.qc_pre schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.qc_pre must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_mapping_summary_smoke_config(
    repo_root: &Path,
) -> Result<LocalMappingSummarySmokeConfig> {
    let path = repo_root.join(LOCAL_MAPPING_SUMMARY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMappingSummarySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_mapping_summary.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.mapping_summary schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_filter_smoke_config(repo_root: &Path) -> Result<LocalFilterSmokeConfig> {
    let path = repo_root.join(LOCAL_FILTER_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalFilterSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_filter.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.filter schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.filter must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_mapq_filter_smoke_config(repo_root: &Path) -> Result<LocalMapqFilterSmokeConfig> {
    let path = repo_root.join(LOCAL_MAPQ_FILTER_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMapqFilterSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_mapq_filter.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.mapq_filter schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.mapq_filter must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_length_filter_smoke_config(repo_root: &Path) -> Result<LocalLengthFilterSmokeConfig> {
    let path = repo_root.join(LOCAL_LENGTH_FILTER_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalLengthFilterSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_length_filter.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.length_filter schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.length_filter must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_markdup_smoke_config(repo_root: &Path) -> Result<LocalMarkdupSmokeConfig> {
    let path = repo_root.join(LOCAL_MARKDUP_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMarkdupSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_markdup.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.markdup schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.markdup must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_duplication_metrics_smoke_config(
    repo_root: &Path,
) -> Result<LocalDuplicationMetricsSmokeConfig> {
    let path = repo_root.join(LOCAL_DUPLICATION_METRICS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDuplicationMetricsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_duplication_metrics.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.duplication_metrics schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_complexity_smoke_config(repo_root: &Path) -> Result<LocalComplexitySmokeConfig> {
    let path = repo_root.join(LOCAL_COMPLEXITY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalComplexitySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_complexity.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.complexity schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.complexity must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_coverage_smoke_config(repo_root: &Path) -> Result<LocalCoverageSmokeConfig> {
    let path = repo_root.join(LOCAL_COVERAGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalCoverageSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_coverage.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.coverage schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.coverage must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_insert_size_smoke_config(repo_root: &Path) -> Result<LocalInsertSizeSmokeConfig> {
    let path = repo_root.join(LOCAL_INSERT_SIZE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalInsertSizeSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_insert_size.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.insert_size schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.insert_size must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_gc_bias_smoke_config(repo_root: &Path) -> Result<LocalGcBiasSmokeConfig> {
    let path = repo_root.join(LOCAL_GC_BIAS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalGcBiasSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_gc_bias.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.gc_bias schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.gc_bias must declare at least one governed case"));
    }
    Ok(config)
}

fn load_local_endogenous_content_smoke_config(
    repo_root: &Path,
) -> Result<LocalEndogenousContentSmokeConfig> {
    let path = repo_root.join(LOCAL_ENDOGENOUS_CONTENT_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalEndogenousContentSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_endogenous_content.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.endogenous_content schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_overlap_correction_smoke_config(
    repo_root: &Path,
) -> Result<LocalOverlapCorrectionSmokeConfig> {
    let path = repo_root.join(LOCAL_OVERLAP_CORRECTION_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalOverlapCorrectionSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_overlap_correction.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.overlap_correction schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction must declare at least one governed case"
        ));
    }
    Ok(config)
}
