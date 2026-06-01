use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ArtifactRef;
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_fastq::banks::{
    adapter_bank_provenance_json, resolve_effective_adapters, AdapterSelection,
    DEFAULT_ADAPTER_PRESET,
};
use bijux_dna_domain_fastq::params::correct::QualityEncoding;
use bijux_dna_domain_fastq::params::qc_post::{QcAggregationEngine, QcAggregationScope};
use bijux_dna_domain_fastq::params::umi::{UmiFailedExtractionPolicy, UmiReadNameTransform};
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS;
use bijux_dna_domain_fastq::stages::ids::STAGE_CORRECT_ERRORS;
use bijux_dna_domain_fastq::stages::ids::STAGE_DETECT_ADAPTERS;
use bijux_dna_domain_fastq::stages::ids::STAGE_DETECT_DUPLICATES_PREMERGE;
use bijux_dna_domain_fastq::stages::ids::STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN;
use bijux_dna_domain_fastq::stages::ids::STAGE_EXTRACT_UMIS;
use bijux_dna_domain_fastq::stages::ids::STAGE_FILTER_LOW_COMPLEXITY;
use bijux_dna_domain_fastq::stages::ids::STAGE_FILTER_READS;
use bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS;
use bijux_dna_domain_fastq::stages::ids::STAGE_MERGE_PAIRS;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_ABUNDANCE;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS;
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES;
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READS;
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS;
use bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_CHIMERAS;
use bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_POLYG_TAILS;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_READS;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE;
use bijux_dna_domain_fastq::stages::ids::STAGE_VALIDATE_READS;
use bijux_dna_domain_fastq::GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION;
use serde::Deserialize;

use crate::selection::{
    allowed_tools_for_stage, load_fastq_domain_tool_execution_spec, select_cluster_otus_tools,
    select_correct_tools, select_detect_adapters_tools, select_filter_low_complexity_tools,
    select_filter_tools, select_infer_asvs_tools, select_merge_tools,
    select_normalize_abundance_tools, select_normalize_primers_tools,
    select_profile_overrepresented_tools, select_profile_read_lengths_tools,
    select_remove_chimeras_tools, select_remove_duplicates_tools, select_stats_tools,
    select_trim_tools, select_umi_tools, select_validate_tools,
};
use crate::tool_adapters::fastq::cluster_otus::{
    plan_with_options as plan_cluster_otus_with_options, ClusterOtusPlanOptions,
};
use crate::tool_adapters::fastq::correct_errors::plan_correct_with_options;
use crate::tool_adapters::fastq::detect_adapters::plan_with_options as plan_detect_adapters;
use crate::tool_adapters::fastq::detect_duplicates_premerge::plan as plan_detect_duplicates_premerge;
use crate::tool_adapters::fastq::estimate_library_complexity_prealign::plan as plan_estimate_library_complexity_prealign;
use crate::tool_adapters::fastq::extract_umis::plan_umi_with_options;
use crate::tool_adapters::fastq::filter_low_complexity::{
    plan_low_complexity, LowComplexityPlanOptions,
};
use crate::tool_adapters::fastq::filter_reads::{plan_filter, FilterPlanOptions};
use crate::tool_adapters::fastq::infer_asvs::{
    plan_with_options as plan_infer_asvs_with_options, InferAsvsPlanOptions,
};
use crate::tool_adapters::fastq::merge_pairs::{plan_merge_with_options, MergePlanOptions};
use crate::tool_adapters::fastq::normalize_abundance::{
    plan_with_options as plan_normalize_abundance_with_options, NormalizeAbundancePlanOptions,
};
use crate::tool_adapters::fastq::normalize_primers::{
    plan_with_options as plan_normalize_primers, NormalizePrimersPlanOptions,
};
use crate::tool_adapters::fastq::profile_overrepresented_sequences::plan_with_options as plan_profile_overrepresented_sequences_with_options;
use crate::tool_adapters::fastq::profile_read_lengths::plan_with_options as plan_profile_read_lengths;
use crate::tool_adapters::fastq::profile_reads::plan_stats_with_threads;
use crate::tool_adapters::fastq::remove_chimeras::plan as plan_remove_chimeras;
use crate::tool_adapters::fastq::remove_duplicates::{
    dedup_mode_from_literal, plan_deduplicate_with_options, RemoveDuplicatesPlanOptions,
};
use crate::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs;
use crate::tool_adapters::fastq::trim_polyg_tails::{
    plan_trim_polyg_tails_with_options, TrimPolygPlanOptions,
};
use crate::tool_adapters::fastq::trim_reads::{
    plan_with_options as plan_trim_reads_with_options, TrimPlanOptions,
};
use crate::tool_adapters::fastq::trim_terminal_damage::{
    plan_trim_terminal_damage_with_options, TrimTerminalDamagePlanOptions,
};
use crate::tool_adapters::fastq::validate_reads::{
    default_plan_options_for_layout, plan_with_options, validation_mode_from_literal,
};

const LOCAL_DETECT_ADAPTERS_CONFIG_PATH: &str = "configs/bench/local/fastq-detect-adapters.toml";
const DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR: &str = "target/local-smoke/fastq.detect_adapters";
const LOCAL_CORRECT_ERRORS_CONFIG_PATH: &str = "configs/bench/local/fastq-correct-errors.toml";
const DEFAULT_LOCAL_CORRECT_ERRORS_OUTPUT_DIR: &str = "target/local-smoke/fastq.correct_errors";
const LOCAL_EXTRACT_UMIS_CONFIG_PATH: &str = "configs/bench/local/fastq-extract-umis.toml";
const DEFAULT_LOCAL_EXTRACT_UMIS_OUTPUT_DIR: &str = "target/local-smoke/fastq.extract_umis";
const LOCAL_DETECT_DUPLICATES_PREMERGE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-detect-duplicates-premerge.toml";
const DEFAULT_LOCAL_DETECT_DUPLICATES_PREMERGE_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.detect_duplicates_premerge";
const LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_CONFIG_PATH: &str =
    "configs/bench/local/fastq-estimate-library-complexity-prealign.toml";
const DEFAULT_LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.estimate_library_complexity_prealign";
const LOCAL_FILTER_LOW_COMPLEXITY_CONFIG_PATH: &str =
    "configs/bench/local/fastq-filter-low-complexity.toml";
const DEFAULT_LOCAL_FILTER_LOW_COMPLEXITY_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.filter_low_complexity";
const LOCAL_FILTER_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-filter-reads.toml";
const DEFAULT_LOCAL_FILTER_READS_OUTPUT_DIR: &str = "target/local-smoke/fastq.filter_reads";
const LOCAL_CLUSTER_OTUS_CONFIG_PATH: &str = "configs/bench/local/fastq-cluster-otus.toml";
const DEFAULT_LOCAL_CLUSTER_OTUS_OUTPUT_DIR: &str = "target/local-smoke/fastq.cluster_otus";
const LOCAL_INFER_ASVS_CONFIG_PATH: &str = "configs/bench/local/fastq-infer-asvs.toml";
const DEFAULT_LOCAL_INFER_ASVS_OUTPUT_DIR: &str = "target/local-smoke/fastq.infer_asvs";
const LOCAL_MERGE_PAIRS_CONFIG_PATH: &str = "configs/bench/local/fastq-merge-pairs.toml";
const DEFAULT_LOCAL_MERGE_PAIRS_OUTPUT_DIR: &str = "target/local-smoke/fastq.merge_pairs";
const LOCAL_NORMALIZE_ABUNDANCE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-normalize-abundance.toml";
const DEFAULT_LOCAL_NORMALIZE_ABUNDANCE_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.normalize_abundance";
const LOCAL_NORMALIZE_PRIMERS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-normalize-primers.toml";
const DEFAULT_LOCAL_NORMALIZE_PRIMERS_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.normalize_primers";
const LOCAL_PROFILE_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-profile-reads.toml";
const DEFAULT_LOCAL_PROFILE_READS_OUTPUT_DIR: &str = "target/local-smoke/fastq.profile_reads";
const LOCAL_REPORT_QC_CONFIG_PATH: &str = "configs/bench/local/fastq-report-qc.toml";
const DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR: &str = "target/local-smoke/fastq.report_qc";
const LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_CONFIG_PATH: &str =
    "configs/bench/local/fastq-profile-overrepresented-sequences.toml";
const DEFAULT_LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.profile_overrepresented_sequences";
const LOCAL_PROFILE_READ_LENGTHS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-profile-read-lengths.toml";
const DEFAULT_LOCAL_PROFILE_READ_LENGTHS_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.profile_read_lengths";
const LOCAL_REMOVE_DUPLICATES_CONFIG_PATH: &str =
    "configs/bench/local/fastq-remove-duplicates.toml";
const DEFAULT_LOCAL_REMOVE_DUPLICATES_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.remove_duplicates";
const LOCAL_REMOVE_CHIMERAS_CONFIG_PATH: &str = "configs/bench/local/fastq-remove-chimeras.toml";
const DEFAULT_LOCAL_REMOVE_CHIMERAS_OUTPUT_DIR: &str = "target/local-smoke/fastq.remove_chimeras";
const LOCAL_TRIM_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-trim-reads.toml";
const DEFAULT_LOCAL_TRIM_READS_OUTPUT_DIR: &str = "target/local-smoke/fastq.trim_reads";
const LOCAL_TRIM_POLYG_TAILS_CONFIG_PATH: &str = "configs/bench/local/fastq-trim-polyg-tails.toml";
const DEFAULT_LOCAL_TRIM_POLYG_TAILS_OUTPUT_DIR: &str = "target/local-smoke/fastq.trim_polyg_tails";
const LOCAL_TRIM_TERMINAL_DAMAGE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-trim-terminal-damage.toml";
const DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.trim_terminal_damage";
const LOCAL_VALIDATE_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-validate-reads.toml";
const DEFAULT_LOCAL_VALIDATE_READS_OUTPUT_DIR: &str = "target/local-smoke/fastq.validate_reads";

#[derive(Debug, Clone)]
pub struct LocalProfileReadLengthsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub histogram_bins: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalProfileReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalProfileOverrepresentedSequencesSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalDetectAdaptersSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalCorrectErrorsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub quality_encoding: QualityEncoding,
    pub conservative_mode: bool,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalExtractUmisSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub umi_pattern: String,
    pub read_name_transform: UmiReadNameTransform,
    pub failed_extraction_policy: UmiFailedExtractionPolicy,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalDetectDuplicatesPremergeSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalEstimateLibraryComplexityPrealignSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub kmer_size: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalFilterReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalFilterLowComplexitySmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub entropy_threshold: f64,
    pub polyx_threshold: Option<u32>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalInferAsvsSmokeCasePlan {
    pub sample_id: String,
    pub reads: PathBuf,
    pub denoising_method: String,
    pub pooling_mode: String,
    pub chimera_policy: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalClusterOtusSmokeCasePlan {
    pub sample_id: String,
    pub reads: PathBuf,
    pub otu_identity: f64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalMergePairsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub merge_overlap: u32,
    pub min_length: u32,
    pub unmerged_read_policy: bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalNormalizeAbundanceSmokeCasePlan {
    pub sample_id: String,
    pub abundance_table: PathBuf,
    pub method: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalRemoveDuplicatesSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub dedup_mode: bijux_dna_domain_fastq::params::remove_duplicates::DedupMode,
    pub keep_order: bool,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalRemoveChimerasSmokeCasePlan {
    pub sample_id: String,
    pub reads: PathBuf,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalNormalizePrimersSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalTrimTerminalDamageSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalTrimPolygTailsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub min_polyg_run: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalTrimReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub min_length: u32,
    pub quality_cutoff: Option<u32>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalValidateReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub validation_mode: ValidationMode,
    pub pair_sync_policy: PairSyncPolicy,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalReportQcSmokePlanConfig {
    schema_version: String,
    tool_id: String,
    fixture_root: PathBuf,
    manifest_template: PathBuf,
    #[serde(default)]
    aggregation_engine: Option<String>,
    #[serde(default)]
    aggregation_scope: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalValidateReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    validation_mode: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalValidateReadsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDetectAdaptersSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDetectAdaptersSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalCorrectErrorsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    quality_encoding: Option<String>,
    #[serde(default)]
    conservative_mode: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalCorrectErrorsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalExtractUmisSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    umi_pattern: Option<String>,
    #[serde(default)]
    extraction_location: Option<String>,
    #[serde(default)]
    read_name_transform: Option<String>,
    #[serde(default)]
    failed_extraction_policy: Option<String>,
    #[serde(default)]
    grouping_policy: Option<String>,
    #[serde(default)]
    downstream_dedup_policy: Option<String>,
    #[serde(default)]
    downstream_propagation: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalExtractUmisSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDetectDuplicatesPremergeSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDetectDuplicatesPremergeSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalEstimateLibraryComplexityPrealignSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    kmer_size: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalEstimateLibraryComplexityPrealignSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    max_n: Option<u32>,
    #[serde(default)]
    max_n_fraction: Option<f64>,
    #[serde(default)]
    max_n_count: Option<u32>,
    #[serde(default)]
    low_complexity_threshold: Option<f64>,
    #[serde(default)]
    entropy_threshold: Option<f64>,
    #[serde(default)]
    polyx_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalFilterReadsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterLowComplexitySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    entropy_threshold: Option<f64>,
    #[serde(default)]
    polyx_threshold: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalFilterLowComplexitySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalClusterOtusSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    otu_identity: Option<f64>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalClusterOtusSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalInferAsvsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    denoising_method: Option<String>,
    #[serde(default)]
    pooling_mode: Option<String>,
    #[serde(default)]
    chimera_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalInferAsvsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMergePairsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    merge_overlap: Option<u32>,
    #[serde(default)]
    min_length: Option<u32>,
    #[serde(default)]
    unmerged_read_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMergePairsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizeAbundanceSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalNormalizeAbundanceSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveDuplicatesSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    dedup_mode: Option<String>,
    #[serde(default)]
    keep_order: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalRemoveDuplicatesSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveChimerasSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalRemoveChimerasSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizePrimersSmokeConfig {
    schema_version: String,
    tool_id: String,
    primer_set_id: String,
    marker_id: String,
    primer_fasta: PathBuf,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    orientation_policy: Option<String>,
    #[serde(default)]
    max_mismatch_rate: Option<f64>,
    #[serde(default)]
    min_overlap_bp: Option<u32>,
    #[serde(default)]
    strict_5p_anchor: Option<bool>,
    #[serde(default)]
    allow_iupac_codes: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalNormalizePrimersSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalProfileReadsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileOverrepresentedSequencesSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    top_k: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalProfileOverrepresentedSequencesSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimTerminalDamageSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    damage_mode: String,
    #[serde(default)]
    execution_policy: Option<String>,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalTrimTerminalDamageSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimPolygTailsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    trim_polyg: Option<bool>,
    #[serde(default)]
    min_polyg_run: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalTrimPolygTailsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    min_length: Option<u32>,
    #[serde(default)]
    quality_cutoff: Option<u32>,
    #[serde(default)]
    n_policy: Option<String>,
    #[serde(default)]
    adapter_policy: Option<String>,
    #[serde(default)]
    adapter_preset: Option<String>,
    #[serde(default)]
    polyx_policy: Option<String>,
    #[serde(default)]
    contaminant_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalTrimReadsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDetectAdaptersSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalCorrectErrorsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalExtractUmisSmokeCase {
    sample_id: String,
    r1: PathBuf,
    r2: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalDetectDuplicatesPremergeSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalEstimateLibraryComplexityPrealignSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterLowComplexitySmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalClusterOtusSmokeCase {
    sample_id: String,
    reads: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalInferAsvsSmokeCase {
    sample_id: String,
    reads: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalMergePairsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    r2: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizeAbundanceSmokeCase {
    sample_id: String,
    abundance_table: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveDuplicatesSmokeCase {
    sample_id: String,
    r1: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveChimerasSmokeCase {
    sample_id: String,
    reads: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizePrimersSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileOverrepresentedSequencesSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimTerminalDamageSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimPolygTailsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileReadLengthsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    histogram_bins: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalProfileReadLengthsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileReadLengthsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalValidateReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_detect_adapters_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDetectAdaptersSmokeCasePlan>> {
    let config = load_local_detect_adapters_smoke_config(repo_root)?;
    ensure_unique_detect_adapters_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_DETECT_ADAPTERS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_detect_adapters_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.detect_adapters tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_detect_adapters_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_correct_errors_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalCorrectErrorsSmokeCasePlan>> {
    let config = load_local_correct_errors_smoke_config(repo_root)?;
    ensure_unique_correct_errors_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_CORRECT_ERRORS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_correct_tools(std::slice::from_ref(&config.tool_id), false)?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.correct_errors tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let quality_encoding =
        parse_local_correct_errors_quality_encoding(config.quality_encoding.as_deref())?;
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_CORRECT_ERRORS_OUTPUT_DIR));
    let mut plan_options = crate::CorrectErrorsStageParams::baseline();
    plan_options.threads = Some(tool_spec.resources.threads.max(1));
    plan_options.quality_encoding = quality_encoding.clone();
    plan_options.conservative_mode = config.conservative_mode.unwrap_or(false);

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_correct_errors_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_extract_umis_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalExtractUmisSmokeCasePlan>> {
    let config = load_local_extract_umis_smoke_config(repo_root)?;
    ensure_unique_extract_umis_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_EXTRACT_UMIS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_umi_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }
    if !crate::stage_api::tool_supports_input_layout(&stage_id, &tool_id, true) {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis tool_id `{}` does not support governed paired-end smoke inputs",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_EXTRACT_UMIS_OUTPUT_DIR));
    let plan_options = crate::ExtractUmisStageParams {
        threads: Some(tool_spec.resources.threads.max(1)),
        umi_pattern: config.umi_pattern,
        extraction_location: config.extraction_location,
        read_name_transform: config.read_name_transform,
        failed_extraction_policy: config.failed_extraction_policy,
        grouping_policy: config.grouping_policy,
        downstream_dedup_policy: config.downstream_dedup_policy,
        downstream_propagation: config.downstream_propagation,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_extract_umis_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_detect_duplicates_premerge_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDetectDuplicatesPremergeSmokeCasePlan>> {
    let config = load_local_detect_duplicates_premerge_smoke_config(repo_root)?;
    ensure_unique_detect_duplicates_premerge_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_DETECT_DUPLICATES_PREMERGE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if tool_id.as_str() != "bijux_dna" {
        return Err(anyhow!(
            "local-smoke fastq.detect_duplicates_premerge currently requires governed native tool_id `bijux_dna`, got `{}`",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DETECT_DUPLICATES_PREMERGE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_detect_duplicates_premerge_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_estimate_library_complexity_prealign_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalEstimateLibraryComplexityPrealignSmokeCasePlan>> {
    let config = load_local_estimate_library_complexity_prealign_smoke_config(repo_root)?;
    ensure_unique_estimate_library_complexity_prealign_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if tool_id.as_str() != "bijux_dna" {
        return Err(anyhow!(
            "local-smoke fastq.estimate_library_complexity_prealign currently requires governed native tool_id `bijux_dna`, got `{}`",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let kmer_size = config.kmer_size.unwrap_or(31).max(1);
    let output_root = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_OUTPUT_DIR)
    });

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_estimate_library_complexity_prealign_smoke_case(
                repo_root,
                &tool_spec,
                kmer_size,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_filter_reads_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalFilterReadsSmokeCasePlan>> {
    let config = load_local_filter_reads_smoke_config(repo_root)?;
    ensure_unique_filter_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_FILTER_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_filter_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_FILTER_READS_OUTPUT_DIR));
    let plan_options = FilterPlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        max_n: config.max_n,
        max_n_fraction: config.max_n_fraction,
        max_n_count: config.max_n_count,
        low_complexity_threshold: config.low_complexity_threshold,
        entropy_threshold: config.entropy_threshold,
        kmer_ref: None,
        redundant_filters: Vec::new(),
        polyx_policy: config.polyx_policy.clone(),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_filter_reads_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_filter_low_complexity_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalFilterLowComplexitySmokeCasePlan>> {
    let config = load_local_filter_low_complexity_smoke_config(repo_root)?;
    ensure_unique_filter_low_complexity_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_FILTER_LOW_COMPLEXITY.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools =
        select_filter_low_complexity_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.filter_low_complexity tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_FILTER_LOW_COMPLEXITY_OUTPUT_DIR));
    let plan_options = LowComplexityPlanOptions {
        entropy_threshold: Some(config.entropy_threshold.unwrap_or(0.5)),
        polyx_threshold: config.polyx_threshold,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_filter_low_complexity_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_infer_asvs_smoke_plans(repo_root: &Path) -> Result<Vec<LocalInferAsvsSmokeCasePlan>> {
    let config = load_local_infer_asvs_smoke_config(repo_root)?;
    ensure_unique_infer_asvs_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_INFER_ASVS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_infer_asvs_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }
    if !crate::stage_api::tool_supports_input_layout(&stage_id, &tool_id, false) {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs tool_id `{}` does not support governed single-end smoke inputs",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let mut plan_options = InferAsvsPlanOptions::baseline();
    plan_options.threads = Some(tool_spec.resources.threads.max(1));
    if let Some(denoising_method) = config.denoising_method {
        plan_options.denoising_method = denoising_method;
    }
    if let Some(pooling_mode) = config.pooling_mode {
        plan_options.pooling_mode = pooling_mode;
    }
    if let Some(chimera_policy) = config.chimera_policy {
        plan_options.chimera_policy = chimera_policy;
    }
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_INFER_ASVS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_infer_asvs_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_cluster_otus_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalClusterOtusSmokeCasePlan>> {
    let config = load_local_cluster_otus_smoke_config(repo_root)?;
    ensure_unique_cluster_otus_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_CLUSTER_OTUS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_cluster_otus_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.cluster_otus tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }
    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let mut plan_options = ClusterOtusPlanOptions::baseline();
    plan_options.threads = Some(tool_spec.resources.threads.max(1));
    if let Some(otu_identity) = config.otu_identity {
        plan_options.otu_identity = otu_identity;
    }
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_CLUSTER_OTUS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_cluster_otus_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_normalize_abundance_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalNormalizeAbundanceSmokeCasePlan>> {
    let config = load_local_normalize_abundance_smoke_config(repo_root)?;
    ensure_unique_normalize_abundance_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_NORMALIZE_ABUNDANCE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_normalize_abundance_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_abundance tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_NORMALIZE_ABUNDANCE_OUTPUT_DIR));
    let plan_options = NormalizeAbundancePlanOptions {
        method: config.method.unwrap_or_else(|| "relative_abundance".to_string()),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_normalize_abundance_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_merge_pairs_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMergePairsSmokeCasePlan>> {
    let config = load_local_merge_pairs_smoke_config(repo_root)?;
    ensure_unique_merge_pairs_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_MERGE_PAIRS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_merge_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MERGE_PAIRS_OUTPUT_DIR));
    let merge_overlap = config.merge_overlap.unwrap_or(10).max(1);
    let min_length = config.min_length.unwrap_or(30).max(1);
    let unmerged_read_policy =
        parse_local_merge_pairs_unmerged_read_policy(config.unmerged_read_policy.as_deref())?;
    let plan_options = MergePlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        merge_overlap: Some(merge_overlap),
        min_length: Some(min_length),
        unmerged_read_policy: unmerged_read_policy.clone(),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_merge_pairs_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_remove_duplicates_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalRemoveDuplicatesSmokeCasePlan>> {
    let config = load_local_remove_duplicates_smoke_config(repo_root)?;
    ensure_unique_remove_duplicates_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_REMOVE_DUPLICATES.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_remove_duplicates_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }
    if !crate::stage_api::tool_supports_input_layout(&stage_id, &tool_id, false) {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates tool_id `{}` does not support governed single-end smoke inputs",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let dedup_mode = dedup_mode_from_literal(config.dedup_mode.as_deref().unwrap_or("exact"))?;
    let keep_order = config.keep_order.unwrap_or(true);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REMOVE_DUPLICATES_OUTPUT_DIR));
    let plan_options = RemoveDuplicatesPlanOptions {
        dedup_mode: dedup_mode.clone(),
        keep_order,
        threads_override: Some(tool_spec.resources.threads.max(1)),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_remove_duplicates_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_remove_chimeras_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalRemoveChimerasSmokeCasePlan>> {
    let config = load_local_remove_chimeras_smoke_config(repo_root)?;
    ensure_unique_remove_chimeras_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_REMOVE_CHIMERAS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_remove_chimeras_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.remove_chimeras tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REMOVE_CHIMERAS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_remove_chimeras_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_normalize_primers_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalNormalizePrimersSmokeCasePlan>> {
    let config = load_local_normalize_primers_smoke_config(repo_root)?;
    ensure_unique_normalize_primers_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_NORMALIZE_PRIMERS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_normalize_primers_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let primer_fasta = repo_root.join(&config.primer_fasta);
    if !primer_fasta.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers primer_fasta is missing: {}",
            primer_fasta.display()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_NORMALIZE_PRIMERS_OUTPUT_DIR));
    let plan_options = NormalizePrimersPlanOptions {
        primer_set_id: config.primer_set_id,
        marker_id: Some(config.marker_id),
        primer_fasta: Some(config.primer_fasta),
        orientation_policy: config
            .orientation_policy
            .unwrap_or_else(|| "normalize_to_forward_primer".to_string()),
        max_mismatch_rate: config.max_mismatch_rate.unwrap_or(0.10),
        min_overlap_bp: config.min_overlap_bp.unwrap_or(10).max(1),
        strict_5p_anchor: config.strict_5p_anchor.unwrap_or(true),
        allow_iupac_codes: config.allow_iupac_codes.unwrap_or(true),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_normalize_primers_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_terminal_damage_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimTerminalDamageSmokeCasePlan>> {
    let config = load_local_trim_terminal_damage_smoke_config(repo_root)?;
    ensure_unique_trim_terminal_damage_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(&stage_id).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage tool_id `{}` is not admitted for {}",
            tool_id.as_str(),
            stage_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR));
    let damage_mode = config.damage_mode.parse().map_err(|error: String| {
        anyhow!(
            "invalid local-smoke fastq.trim_terminal_damage damage_mode `{}`: {error}",
            config.damage_mode
        )
    })?;
    let execution_policy =
        bijux_dna_domain_fastq::params::trim::parse_terminal_damage_execution_policy(
            config.execution_policy.as_deref().unwrap_or("policy_derived"),
        )
        .ok_or_else(|| {
            anyhow!(
                "invalid local-smoke fastq.trim_terminal_damage execution_policy `{:?}`",
                config.execution_policy
            )
        })?;
    let plan_options = TrimTerminalDamagePlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        damage_mode,
        execution_policy,
        trim_5p_bases: config.trim_5p_bases,
        trim_3p_bases: config.trim_3p_bases,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_trim_terminal_damage_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_polyg_tails_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimPolygTailsSmokeCasePlan>> {
    let config = load_local_trim_polyg_tails_smoke_config(repo_root)?;
    ensure_unique_trim_polyg_tails_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_TRIM_POLYG_TAILS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(&stage_id).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails tool_id `{}` is not admitted for {}",
            tool_id.as_str(),
            stage_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_POLYG_TAILS_OUTPUT_DIR));
    let trim_polyg = config.trim_polyg.unwrap_or(true);
    let min_polyg_run = config.min_polyg_run.unwrap_or(10).max(1);
    let plan_options = TrimPolygPlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        trim_polyg,
        min_polyg_run,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_trim_polyg_tails_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_reads_smoke_plans(repo_root: &Path) -> Result<Vec<LocalTrimReadsSmokeCasePlan>> {
    let config = load_local_trim_reads_smoke_config(repo_root)?;
    ensure_unique_trim_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_TRIM_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_trim_tools(std::slice::from_ref(&config.tool_id), false)?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.trim_reads tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let adapter_policy = config.adapter_policy.clone().unwrap_or_else(|| "none".to_string());
    let adapter_bank = if matches!(adapter_policy.as_str(), "bank" | "ancient_strict") {
        Some(load_local_trim_reads_adapter_bank_context(
            repo_root,
            config.adapter_preset.as_deref().unwrap_or(DEFAULT_ADAPTER_PRESET),
        )?)
    } else {
        None
    };

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_READS_OUTPUT_DIR));
    let plan_options = TrimPlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        min_length: Some(config.min_length.unwrap_or(30).max(1)),
        quality_cutoff: config.quality_cutoff,
        n_policy: config.n_policy,
        adapter_policy: Some(adapter_policy),
        polyx_policy: config.polyx_policy,
        contaminant_policy: config.contaminant_policy,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_trim_reads_smoke_case(
                repo_root,
                &tool_spec,
                adapter_bank.as_ref(),
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_detect_adapters_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDetectAdaptersSmokeCase,
) -> Result<LocalDetectAdaptersSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.detect_adapters r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.detect_adapters r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let mut options = crate::DetectAdaptersStageParams::default();
    options.threads = Some(tool_spec.resources.threads.max(1));
    let plan = plan_detect_adapters(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, &options)?;

    Ok(LocalDetectAdaptersSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn build_local_correct_errors_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &crate::CorrectErrorsStageParams,
    output_root: &Path,
    case: LocalCorrectErrorsSmokeCase,
) -> Result<LocalCorrectErrorsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.correct_errors r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.correct_errors r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        plan_correct_with_options(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, options)?;

    Ok(LocalCorrectErrorsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        quality_encoding: options.quality_encoding.clone(),
        conservative_mode: options.conservative_mode,
        plan,
    })
}

fn build_local_extract_umis_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &crate::ExtractUmisStageParams,
    output_root: &Path,
    case: LocalExtractUmisSmokeCase,
) -> Result<LocalExtractUmisSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    let r2_abs = repo_root.join(&case.r2);
    if !r2_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis r2 fixture is missing: {}",
            r2_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_umi_with_options(tool_spec, &case.r1, &case.r2, &out_dir, options)?;
    let effective_params = serde_json::from_value::<bijux_dna_domain_fastq::FastqUmiParams>(
        plan.effective_params.clone(),
    )
    .map_err(|error| anyhow!("decode local-smoke fastq.extract_umis effective params: {error}"))?;

    Ok(LocalExtractUmisSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        umi_pattern: effective_params.umi_pattern.unwrap_or_else(|| "NNNNNNNN".to_string()),
        read_name_transform: effective_params.read_name_transform,
        failed_extraction_policy: effective_params.failed_extraction_policy,
        plan,
    })
}

fn build_local_detect_duplicates_premerge_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDetectDuplicatesPremergeSmokeCase,
) -> Result<LocalDetectDuplicatesPremergeSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.detect_duplicates_premerge r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.detect_duplicates_premerge r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_detect_duplicates_premerge(tool_spec, &case.r1, case.r2.as_deref(), &out_dir)?;

    Ok(LocalDetectDuplicatesPremergeSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn build_local_estimate_library_complexity_prealign_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    kmer_size: u32,
    output_root: &Path,
    case: LocalEstimateLibraryComplexityPrealignSmokeCase,
) -> Result<LocalEstimateLibraryComplexityPrealignSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.estimate_library_complexity_prealign r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.estimate_library_complexity_prealign r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_estimate_library_complexity_prealign(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(kmer_size),
    )?;

    Ok(LocalEstimateLibraryComplexityPrealignSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        kmer_size,
        plan,
    })
}

fn build_local_filter_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    plan_options: &FilterPlanOptions,
    output_root: &Path,
    case: LocalFilterReadsSmokeCase,
) -> Result<LocalFilterReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.filter_reads r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_filter(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, plan_options)?;

    Ok(LocalFilterReadsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        max_n_count: plan_options.max_n_count.or(plan_options.max_n),
        low_complexity_threshold: plan_options
            .low_complexity_threshold
            .or(plan_options.entropy_threshold),
        plan,
    })
}

fn build_local_filter_low_complexity_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    plan_options: &LowComplexityPlanOptions,
    output_root: &Path,
    case: LocalFilterLowComplexitySmokeCase,
) -> Result<LocalFilterLowComplexitySmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.filter_low_complexity r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.filter_low_complexity r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        plan_low_complexity(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, plan_options)?;

    Ok(LocalFilterLowComplexitySmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        entropy_threshold: plan_options.entropy_threshold.unwrap_or(0.5),
        polyx_threshold: plan_options.polyx_threshold,
        plan,
    })
}

fn build_local_infer_asvs_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &InferAsvsPlanOptions,
    output_root: &Path,
    case: LocalInferAsvsSmokeCase,
) -> Result<LocalInferAsvsSmokeCasePlan> {
    let reads_abs = repo_root.join(&case.reads);
    if !reads_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs reads fixture is missing: {}",
            reads_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_infer_asvs_with_options(tool_spec, &case.reads, None, &out_dir, options)?;

    Ok(LocalInferAsvsSmokeCasePlan {
        sample_id: case.sample_id,
        reads: case.reads,
        denoising_method: options.denoising_method.clone(),
        pooling_mode: options.pooling_mode.clone(),
        chimera_policy: options.chimera_policy.clone(),
        plan,
    })
}

fn build_local_cluster_otus_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &ClusterOtusPlanOptions,
    output_root: &Path,
    case: LocalClusterOtusSmokeCase,
) -> Result<LocalClusterOtusSmokeCasePlan> {
    let reads_abs = repo_root.join(&case.reads);
    if !reads_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.cluster_otus reads fixture is missing: {}",
            reads_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_cluster_otus_with_options(tool_spec, &case.reads, None, &out_dir, options)?;

    Ok(LocalClusterOtusSmokeCasePlan {
        sample_id: case.sample_id,
        reads: case.reads,
        otu_identity: options.otu_identity,
        plan,
    })
}

fn build_local_normalize_primers_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &NormalizePrimersPlanOptions,
    output_root: &Path,
    case: LocalNormalizePrimersSmokeCase,
) -> Result<LocalNormalizePrimersSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.normalize_primers r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_normalize_primers(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, options)?;

    Ok(LocalNormalizePrimersSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn build_local_normalize_abundance_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &NormalizeAbundancePlanOptions,
    output_root: &Path,
    case: LocalNormalizeAbundanceSmokeCase,
) -> Result<LocalNormalizeAbundanceSmokeCasePlan> {
    let abundance_table_abs = repo_root.join(&case.abundance_table);
    if !abundance_table_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_abundance abundance_table fixture is missing: {}",
            abundance_table_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        plan_normalize_abundance_with_options(tool_spec, &case.abundance_table, &out_dir, options)?;

    Ok(LocalNormalizeAbundanceSmokeCasePlan {
        sample_id: case.sample_id,
        abundance_table: case.abundance_table,
        method: options.method.clone(),
        plan,
    })
}

fn build_local_trim_terminal_damage_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &TrimTerminalDamagePlanOptions,
    output_root: &Path,
    case: LocalTrimTerminalDamageSmokeCase,
) -> Result<LocalTrimTerminalDamageSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.trim_terminal_damage r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_trim_terminal_damage_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        options,
    )?;

    Ok(LocalTrimTerminalDamageSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn build_local_trim_polyg_tails_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &TrimPolygPlanOptions,
    output_root: &Path,
    case: LocalTrimPolygTailsSmokeCase,
) -> Result<LocalTrimPolygTailsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.trim_polyg_tails r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_trim_polyg_tails_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        options,
    )?;

    Ok(LocalTrimPolygTailsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        min_polyg_run: options.min_polyg_run,
        plan,
    })
}

fn build_local_merge_pairs_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &MergePlanOptions,
    output_root: &Path,
    case: LocalMergePairsSmokeCase,
) -> Result<LocalMergePairsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    let r2_abs = repo_root.join(&case.r2);
    if !r2_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs r2 fixture is missing: {}",
            r2_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_merge_with_options(tool_spec, &case.r1, &case.r2, &out_dir, options)?;

    Ok(LocalMergePairsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        merge_overlap: options.merge_overlap.unwrap_or(10),
        min_length: options.min_length.unwrap_or(30),
        unmerged_read_policy: options.unmerged_read_policy.clone(),
        plan,
    })
}

fn build_local_remove_duplicates_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &RemoveDuplicatesPlanOptions,
    output_root: &Path,
    case: LocalRemoveDuplicatesSmokeCase,
) -> Result<LocalRemoveDuplicatesSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_deduplicate_with_options(tool_spec, &case.r1, None, &out_dir, options)?;

    Ok(LocalRemoveDuplicatesSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        dedup_mode: options.dedup_mode.clone(),
        keep_order: options.keep_order,
        plan,
    })
}

fn build_local_remove_chimeras_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalRemoveChimerasSmokeCase,
) -> Result<LocalRemoveChimerasSmokeCasePlan> {
    let reads_abs = repo_root.join(&case.reads);
    if !reads_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.remove_chimeras reads fixture is missing: {}",
            reads_abs.display()
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_remove_chimeras(tool_spec, &case.reads, None, &out_dir)?;

    Ok(LocalRemoveChimerasSmokeCasePlan { sample_id: case.sample_id, reads: case.reads, plan })
}

fn build_local_trim_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
    output_root: &Path,
    case: LocalTrimReadsSmokeCase,
) -> Result<LocalTrimReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.trim_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.trim_reads r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_trim_reads_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        adapter_bank,
        None,
        None,
        options,
    )?;

    Ok(LocalTrimReadsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        min_length: options.min_length.unwrap_or(30),
        quality_cutoff: options.quality_cutoff,
        plan,
    })
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_validate_reads_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalValidateReadsSmokeCasePlan>> {
    let config = load_local_validate_reads_smoke_config(repo_root)?;
    ensure_unique_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_VALIDATE_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_validate_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.validate_reads tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let validation_mode =
        validation_mode_from_literal(config.validation_mode.as_deref().unwrap_or("strict"))?;
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_VALIDATE_READS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_validate_reads_smoke_case(
                repo_root,
                &tool_spec,
                &validation_mode,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_profile_reads_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileReadsSmokeCasePlan>> {
    let config = load_local_profile_reads_smoke_config(repo_root)?;
    ensure_unique_profile_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_PROFILE_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_stats_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.profile_reads tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_PROFILE_READS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_profile_reads_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the governed QC manifest cannot
/// be resolved against the fixture bundle, or the stage plan cannot be built.
pub fn local_report_qc_smoke_plan(
    repo_root: &Path,
) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_report_qc_smoke_plan_config(repo_root)?;
    let stage_id = StageId::new("fastq.report_qc".to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = crate::tool_adapters::fastq::report_qc::normalize_qc_post_tool_list(
        std::slice::from_ref(&config.tool_id),
    )?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.report_qc tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let manifest_path = repo_root.join(&config.manifest_template);
    let raw_manifest = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let mut manifest: bijux_dna_domain_fastq::GovernedQcInputsManifestV1 =
        serde_json::from_str(&raw_manifest)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
    if manifest.schema_version != GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required",
            manifest_path.display(),
            manifest.schema_version,
            GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION
        ));
    }

    let fixture_root = repo_root.join(&config.fixture_root);
    if !fixture_root.is_dir() {
        return Err(anyhow!(
            "local-smoke fastq.report_qc fixture_root is missing: {}",
            fixture_root.display()
        ));
    }

    rebase_governed_qc_inputs(&mut manifest.qc_inputs, &fixture_root)?;
    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, None);
    let aggregation_engine =
        parse_local_report_qc_aggregation_engine(config.aggregation_engine.as_deref())?;
    let aggregation_scope =
        parse_local_report_qc_aggregation_scope(config.aggregation_scope.as_deref())?;
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR));

    plan_qc_post_with_qc_inputs(
        &tool_spec,
        &manifest.qc_inputs,
        &output_root,
        std::collections::BTreeMap::new(),
        PairedMode::SingleEnd,
        aggregation_engine,
        aggregation_scope,
        None,
        None,
    )
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_profile_overrepresented_sequences_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileOverrepresentedSequencesSmokeCasePlan>> {
    let config = load_local_profile_overrepresented_sequences_smoke_config(repo_root)?;
    ensure_unique_profile_overrepresented_sequences_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_PROFILE_OVERREPRESENTED_SEQUENCES.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools =
        select_profile_overrepresented_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.profile_overrepresented_sequences tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let top_k = config.top_k;
    let output_root = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_OUTPUT_DIR)
    });

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_profile_overrepresented_sequences_smoke_case(
                repo_root,
                &tool_spec,
                top_k,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_profile_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalProfileReadsSmokeCase,
) -> Result<LocalProfileReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.profile_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.profile_reads r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_stats_with_threads(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(tool_spec.resources.threads.max(1)),
    )?;

    Ok(LocalProfileReadsSmokeCasePlan { sample_id: case.sample_id, r1: case.r1, r2: case.r2, plan })
}

fn build_local_profile_overrepresented_sequences_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    top_k: Option<u32>,
    output_root: &Path,
    case: LocalProfileOverrepresentedSequencesSmokeCase,
) -> Result<LocalProfileOverrepresentedSequencesSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.profile_overrepresented_sequences r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.profile_overrepresented_sequences r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_profile_overrepresented_sequences_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(tool_spec.resources.threads.max(1)),
        top_k,
    )?;

    Ok(LocalProfileOverrepresentedSequencesSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn build_local_validate_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    validation_mode: &ValidationMode,
    output_root: &Path,
    case: LocalValidateReadsSmokeCase,
) -> Result<LocalValidateReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.validate_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.validate_reads r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let mut options = default_plan_options_for_layout(case.r2.as_deref());
    options.threads = Some(tool_spec.resources.threads.max(1));
    options.validation_mode = validation_mode.clone();
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_with_options(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, &options)?;

    Ok(LocalValidateReadsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        validation_mode: options.validation_mode,
        pair_sync_policy: options.pair_sync_policy,
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

fn parse_local_report_qc_aggregation_engine(value: Option<&str>) -> Result<QcAggregationEngine> {
    match value.unwrap_or("multiqc") {
        "auto" | "multiqc" => Ok(QcAggregationEngine::Multiqc),
        other => {
            Err(anyhow!("unsupported local-smoke fastq.report_qc aggregation_engine `{other}`"))
        }
    }
}

fn parse_local_report_qc_aggregation_scope(value: Option<&str>) -> Result<QcAggregationScope> {
    match value.unwrap_or("governed_qc_artifacts") {
        "governed_qc_artifacts" => Ok(QcAggregationScope::GovernedQcArtifacts),
        other => {
            Err(anyhow!("unsupported local-smoke fastq.report_qc aggregation_scope `{other}`"))
        }
    }
}

fn rebase_governed_qc_inputs(qc_inputs: &mut [ArtifactRef], fixture_root: &Path) -> Result<()> {
    for qc_input in qc_inputs {
        let source = fixture_root.join(&qc_input.path);
        if !source.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.report_qc governed QC input is missing: {}",
                source.display()
            ));
        }
        qc_input.path = source;
    }
    Ok(())
}

fn load_local_report_qc_smoke_plan_config(
    repo_root: &Path,
) -> Result<LocalReportQcSmokePlanConfig> {
    let config_path = repo_root.join(LOCAL_REPORT_QC_CONFIG_PATH);
    let raw = std::fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: LocalReportQcSmokePlanConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_report_qc.v1" {
        return Err(anyhow!(
            "{} declares unexpected schema_version `{}`",
            config_path.display(),
            config.schema_version
        ));
    }
    Ok(config)
}

fn ensure_unique_sample_ids(cases: &[LocalValidateReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.validate_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.validate_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_correct_errors_sample_ids(cases: &[LocalCorrectErrorsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.correct_errors sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.correct_errors sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_extract_umis_sample_ids(cases: &[LocalExtractUmisSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.extract_umis sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.extract_umis sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn parse_local_correct_errors_quality_encoding(value: Option<&str>) -> Result<QualityEncoding> {
    match value.unwrap_or("phred33") {
        "phred33" => Ok(QualityEncoding::Phred33),
        "phred64" => Ok(QualityEncoding::Phred64),
        other => {
            Err(anyhow!("unsupported local-smoke fastq.correct_errors quality_encoding `{other}`"))
        }
    }
}

fn load_local_correct_errors_smoke_config(
    repo_root: &Path,
) -> Result<LocalCorrectErrorsSmokeConfig> {
    let path = repo_root.join(LOCAL_CORRECT_ERRORS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalCorrectErrorsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_correct_errors.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.correct_errors schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.correct_errors must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_extract_umis_smoke_config(repo_root: &Path) -> Result<LocalExtractUmisSmokeConfig> {
    let path = repo_root.join(LOCAL_EXTRACT_UMIS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalExtractUmisSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_extract_umis.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.extract_umis schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_validate_reads_smoke_config(
    repo_root: &Path,
) -> Result<LocalValidateReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_VALIDATE_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalValidateReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_validate_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.validate_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.validate_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_profile_read_lengths_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileReadLengthsSmokeCasePlan>> {
    let config = load_local_profile_read_lengths_smoke_config(repo_root)?;
    ensure_unique_profile_read_lengths_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_PROFILE_READ_LENGTHS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools =
        select_profile_read_lengths_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.profile_read_lengths tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let histogram_bins = config.histogram_bins.unwrap_or(100).max(1);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_PROFILE_READ_LENGTHS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_profile_read_lengths_smoke_case(
                repo_root,
                &tool_spec,
                histogram_bins,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_profile_read_lengths_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    histogram_bins: u32,
    output_root: &Path,
    case: LocalProfileReadLengthsSmokeCase,
) -> Result<LocalProfileReadLengthsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.profile_read_lengths r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.profile_read_lengths r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_profile_read_lengths(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(tool_spec.resources.threads.max(1)),
        Some(histogram_bins),
    )?;

    Ok(LocalProfileReadLengthsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        histogram_bins,
        plan,
    })
}

fn ensure_unique_profile_read_lengths_sample_ids(
    cases: &[LocalProfileReadLengthsSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.profile_read_lengths sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.profile_read_lengths sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_profile_reads_sample_ids(cases: &[LocalProfileReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.profile_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.profile_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_profile_overrepresented_sequences_sample_ids(
    cases: &[LocalProfileOverrepresentedSequencesSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.profile_overrepresented_sequences sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.profile_overrepresented_sequences sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_estimate_library_complexity_prealign_sample_ids(
    cases: &[LocalEstimateLibraryComplexityPrealignSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.estimate_library_complexity_prealign sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.estimate_library_complexity_prealign sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_infer_asvs_sample_ids(cases: &[LocalInferAsvsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.infer_asvs sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.infer_asvs sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_cluster_otus_sample_ids(cases: &[LocalClusterOtusSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.cluster_otus sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.cluster_otus sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_normalize_abundance_sample_ids(
    cases: &[LocalNormalizeAbundanceSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.normalize_abundance sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.normalize_abundance sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_merge_pairs_sample_ids(cases: &[LocalMergePairsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.merge_pairs sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.merge_pairs sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_remove_duplicates_sample_ids(
    cases: &[LocalRemoveDuplicatesSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.remove_duplicates sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.remove_duplicates sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_remove_chimeras_sample_ids(cases: &[LocalRemoveChimerasSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.remove_chimeras sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.remove_chimeras sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_normalize_primers_sample_ids(
    cases: &[LocalNormalizePrimersSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.normalize_primers sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.normalize_primers sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_trim_terminal_damage_sample_ids(
    cases: &[LocalTrimTerminalDamageSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.trim_terminal_damage sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.trim_terminal_damage sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_trim_polyg_tails_sample_ids(cases: &[LocalTrimPolygTailsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.trim_polyg_tails sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.trim_polyg_tails sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_trim_reads_sample_ids(cases: &[LocalTrimReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.trim_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.trim_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_filter_reads_sample_ids(cases: &[LocalFilterReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.filter_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.filter_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_filter_low_complexity_sample_ids(
    cases: &[LocalFilterLowComplexitySmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.filter_low_complexity sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.filter_low_complexity sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_detect_duplicates_premerge_sample_ids(
    cases: &[LocalDetectDuplicatesPremergeSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.detect_duplicates_premerge sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.detect_duplicates_premerge sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_detect_adapters_sample_ids(cases: &[LocalDetectAdaptersSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.detect_adapters sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.detect_adapters sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_estimate_library_complexity_prealign_smoke_config(
    repo_root: &Path,
) -> Result<LocalEstimateLibraryComplexityPrealignSmokeConfig> {
    let path = repo_root.join(LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalEstimateLibraryComplexityPrealignSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_estimate_library_complexity_prealign.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.estimate_library_complexity_prealign schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.estimate_library_complexity_prealign must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_infer_asvs_smoke_config(repo_root: &Path) -> Result<LocalInferAsvsSmokeConfig> {
    let path = repo_root.join(LOCAL_INFER_ASVS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalInferAsvsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_infer_asvs.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.infer_asvs schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_cluster_otus_smoke_config(repo_root: &Path) -> Result<LocalClusterOtusSmokeConfig> {
    let path = repo_root.join(LOCAL_CLUSTER_OTUS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalClusterOtusSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_cluster_otus.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.cluster_otus schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.cluster_otus must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_normalize_abundance_smoke_config(
    repo_root: &Path,
) -> Result<LocalNormalizeAbundanceSmokeConfig> {
    let path = repo_root.join(LOCAL_NORMALIZE_ABUNDANCE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalNormalizeAbundanceSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_normalize_abundance.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.normalize_abundance schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_abundance must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_merge_pairs_smoke_config(repo_root: &Path) -> Result<LocalMergePairsSmokeConfig> {
    let path = repo_root.join(LOCAL_MERGE_PAIRS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMergePairsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_merge_pairs.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.merge_pairs schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_remove_duplicates_smoke_config(
    repo_root: &Path,
) -> Result<LocalRemoveDuplicatesSmokeConfig> {
    let path = repo_root.join(LOCAL_REMOVE_DUPLICATES_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalRemoveDuplicatesSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_remove_duplicates.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.remove_duplicates schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_remove_chimeras_smoke_config(
    repo_root: &Path,
) -> Result<LocalRemoveChimerasSmokeConfig> {
    let path = repo_root.join(LOCAL_REMOVE_CHIMERAS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalRemoveChimerasSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_remove_chimeras.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.remove_chimeras schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.remove_chimeras must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_filter_reads_smoke_config(repo_root: &Path) -> Result<LocalFilterReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_FILTER_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalFilterReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_filter_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.filter_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_filter_low_complexity_smoke_config(
    repo_root: &Path,
) -> Result<LocalFilterLowComplexitySmokeConfig> {
    let path = repo_root.join(LOCAL_FILTER_LOW_COMPLEXITY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalFilterLowComplexitySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_filter_low_complexity.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.filter_low_complexity schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.filter_low_complexity must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_normalize_primers_smoke_config(
    repo_root: &Path,
) -> Result<LocalNormalizePrimersSmokeConfig> {
    let path = repo_root.join(LOCAL_NORMALIZE_PRIMERS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalNormalizePrimersSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_normalize_primers.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.normalize_primers schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_trim_terminal_damage_smoke_config(
    repo_root: &Path,
) -> Result<LocalTrimTerminalDamageSmokeConfig> {
    let path = repo_root.join(LOCAL_TRIM_TERMINAL_DAMAGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalTrimTerminalDamageSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_trim_terminal_damage.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.trim_terminal_damage schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_trim_polyg_tails_smoke_config(
    repo_root: &Path,
) -> Result<LocalTrimPolygTailsSmokeConfig> {
    let path = repo_root.join(LOCAL_TRIM_POLYG_TAILS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalTrimPolygTailsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_trim_polyg_tails.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.trim_polyg_tails schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_trim_reads_smoke_config(repo_root: &Path) -> Result<LocalTrimReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_TRIM_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalTrimReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_trim_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.trim_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_trim_reads_adapter_bank_context(
    repo_root: &Path,
    preset_name: &str,
) -> Result<serde_json::Value> {
    let bank_path = repo_root.join(bijux_dna_domain_fastq::adapter_bank_path());
    let presets_path = repo_root.join(bijux_dna_domain_fastq::adapter_presets_path());
    let bank = bijux_dna_domain_fastq::load_adapter_bank(&bank_path)
        .with_context(|| format!("load {}", bank_path.display()))?;
    let presets = bijux_dna_domain_fastq::load_adapter_presets(&presets_path, &bank)
        .with_context(|| format!("load {}", presets_path.display()))?;
    let selection = AdapterSelection {
        bank,
        presets,
        preset_name: preset_name.to_string(),
        bank_checksum: bijux_dna_infra::hash_file_sha256(&bank_path)?,
        presets_checksum: bijux_dna_infra::hash_file_sha256(&presets_path)?,
    };
    let effective = resolve_effective_adapters(&selection, &[], &[])?;
    Ok(adapter_bank_provenance_json(&selection, &effective, &[], &[]))
}

fn load_local_detect_duplicates_premerge_smoke_config(
    repo_root: &Path,
) -> Result<LocalDetectDuplicatesPremergeSmokeConfig> {
    let path = repo_root.join(LOCAL_DETECT_DUPLICATES_PREMERGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDetectDuplicatesPremergeSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_detect_duplicates_premerge.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.detect_duplicates_premerge schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.detect_duplicates_premerge must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_detect_adapters_smoke_config(
    repo_root: &Path,
) -> Result<LocalDetectAdaptersSmokeConfig> {
    let path = repo_root.join(LOCAL_DETECT_ADAPTERS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDetectAdaptersSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_detect_adapters.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.detect_adapters schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.detect_adapters must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_profile_read_lengths_smoke_config(
    repo_root: &Path,
) -> Result<LocalProfileReadLengthsSmokeConfig> {
    let path = repo_root.join(LOCAL_PROFILE_READ_LENGTHS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalProfileReadLengthsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_profile_read_lengths.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.profile_read_lengths schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.profile_read_lengths must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_profile_reads_smoke_config(repo_root: &Path) -> Result<LocalProfileReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_PROFILE_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalProfileReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_profile_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.profile_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.profile_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_profile_overrepresented_sequences_smoke_config(
    repo_root: &Path,
) -> Result<LocalProfileOverrepresentedSequencesSmokeConfig> {
    let path = repo_root.join(LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalProfileOverrepresentedSequencesSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_profile_overrepresented_sequences.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.profile_overrepresented_sequences schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.profile_overrepresented_sequences must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn parse_local_merge_pairs_unmerged_read_policy(
    raw: Option<&str>,
) -> Result<bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy> {
    match raw.unwrap_or("emit_unmerged_pairs") {
        "emit_unmerged_pairs" => {
            Ok(bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy::EmitUnmergedPairs)
        }
        "omit_unmerged_pairs" => {
            Ok(bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy::OmitUnmergedPairs)
        }
        other => {
            Err(anyhow!("unsupported local-smoke fastq.merge_pairs unmerged_read_policy `{other}`"))
        }
    }
}
