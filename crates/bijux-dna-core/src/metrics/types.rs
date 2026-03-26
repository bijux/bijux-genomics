use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contract::{ContractVersion, MetricProvenanceV1, ToolConstraints};
use crate::foundation::{measure::ExecutionMetrics, BijuxError, Result};
use crate::ids::{StageId, ToolId};
use crate::metrics::MetricContextV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricId {
    RuntimeS,
    MemoryMb,
    ExitCode,
    ReadsIn,
    ReadsOut,
    ReadsDropped,
    ReadsRemovedByN,
    ReadsRemovedByEntropy,
    ReadsRemovedLowComplexity,
    ReadsRemovedByKmer,
    ReadsRemovedContaminantKmer,
    ReadsRemovedByLength,
    ReadsTotal,
    ReadsValid,
    ReadsInvalid,
    BasesIn,
    BasesOut,
    BasesTotal,
    PairsIn,
    PairsOut,
    Threads,
    ReadsR1,
    ReadsR2,
    ReadsMerged,
    ReadsUnmerged,
    MeanQBefore,
    MeanQAfter,
    MeanQ,
    MergeRate,
    ReadsWithUmi,
    DedupRate,
    KmerFixRate,
    CandidateAdapterCount,
    AdapterTrimmedFraction,
    ContaminationRate,
    ContaminationSummary,
    GcPercent,
    LengthHistogram,
    ReadCount,
    MeanReadLength,
    MaxReadLength,
    DistinctLengths,
    SequenceCount,
    FlaggedSequences,
    TopFraction,
    DeltaMetrics,
    PairedMode,
    AdapterPolicy,
    PolyxPolicy,
    NPolicy,
    ContaminantPolicy,
    RawBackendReportFormat,
    AdapterPreset,
    AdapterBankId,
    AdapterBankHash,
    AdapterOverrides,
    ValidatedInputs,
    ValidatedPairs,
    PairSyncChecked,
    PairSyncPass,
    PairCountMatch,
    StrictPass,
    FailureClass,
    Tool,
    TrimPolyg,
    MinPolygRun,
    BasesTrimmedPolyg,
    PolyxBankId,
    PolyxBankHash,
    PolyxPreset,
    DamageMode,
    ExecutionPolicy,
    RequestedTrim5pBases,
    RequestedTrim3pBases,
    UdgClassification,
    CtGaAsymmetryPre,
    CtGaAsymmetryPost,
    ClassifiedFraction,
    UnclassifiedFraction,
    Classifier,
    ReportFormat,
    DatabaseCatalogId,
    DatabaseArtifactId,
    MinimumConfidence,
    EmitUnclassified,
    TopTaxa,
    QcRawDir,
    QcTrimmedDir,
    AggregationEngine,
    AggregationScope,
    GovernedQcInputCount,
    GovernedQcContributorStageIds,
    GovernedQcContributorToolIds,
    GovernedQcLineageHash,
    MultiqcSampleCount,
    MultiqcModuleCount,
    MultiqcReport,
    MultiqcData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DerivedMetricId {
    ReadRetention,
    BaseRetention,
    MergeEfficiency,
    ErrorReductionProxy,
}

impl MetricId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            MetricId::RuntimeS => "runtime_s",
            MetricId::MemoryMb => "memory_mb",
            MetricId::ExitCode => "exit_code",
            MetricId::ReadsIn => "reads_in",
            MetricId::ReadsOut => "reads_out",
            MetricId::ReadsDropped => "reads_dropped",
            MetricId::ReadsRemovedByN => "reads_removed_by_n",
            MetricId::ReadsRemovedByEntropy => "reads_removed_by_entropy",
            MetricId::ReadsRemovedLowComplexity => "reads_removed_low_complexity",
            MetricId::ReadsRemovedByKmer => "reads_removed_by_kmer",
            MetricId::ReadsRemovedContaminantKmer => "reads_removed_contaminant_kmer",
            MetricId::ReadsRemovedByLength => "reads_removed_by_length",
            MetricId::ReadsTotal => "reads_total",
            MetricId::ReadsValid => "reads_valid",
            MetricId::ReadsInvalid => "reads_invalid",
            MetricId::BasesIn => "bases_in",
            MetricId::BasesOut => "bases_out",
            MetricId::BasesTotal => "bases_total",
            MetricId::PairsIn => "pairs_in",
            MetricId::PairsOut => "pairs_out",
            MetricId::Threads => "threads",
            MetricId::ReadsR1 => "reads_r1",
            MetricId::ReadsR2 => "reads_r2",
            MetricId::ReadsMerged => "reads_merged",
            MetricId::ReadsUnmerged => "reads_unmerged",
            MetricId::MeanQBefore => "mean_q_before",
            MetricId::MeanQAfter => "mean_q_after",
            MetricId::MeanQ => "mean_q",
            MetricId::MergeRate => "merge_rate",
            MetricId::ReadsWithUmi => "reads_with_umi",
            MetricId::DedupRate => "dedup_rate",
            MetricId::KmerFixRate => "kmer_fix_rate",
            MetricId::CandidateAdapterCount => "candidate_adapter_count",
            MetricId::AdapterTrimmedFraction => "adapter_trimmed_fraction",
            MetricId::ContaminationRate => "contamination_rate",
            MetricId::ContaminationSummary => "contamination_summary",
            MetricId::GcPercent => "gc_percent",
            MetricId::LengthHistogram => "length_histogram",
            MetricId::ReadCount => "read_count",
            MetricId::MeanReadLength => "mean_read_length",
            MetricId::MaxReadLength => "max_read_length",
            MetricId::DistinctLengths => "distinct_lengths",
            MetricId::SequenceCount => "sequence_count",
            MetricId::FlaggedSequences => "flagged_sequences",
            MetricId::TopFraction => "top_fraction",
            MetricId::DeltaMetrics => "delta_metrics",
            MetricId::PairedMode => "paired_mode",
            MetricId::AdapterPolicy => "adapter_policy",
            MetricId::PolyxPolicy => "polyx_policy",
            MetricId::NPolicy => "n_policy",
            MetricId::ContaminantPolicy => "contaminant_policy",
            MetricId::RawBackendReportFormat => "raw_backend_report_format",
            MetricId::AdapterPreset => "adapter_preset",
            MetricId::AdapterBankId => "adapter_bank_id",
            MetricId::AdapterBankHash => "adapter_bank_hash",
            MetricId::AdapterOverrides => "adapter_overrides",
            MetricId::ValidatedInputs => "validated_inputs",
            MetricId::ValidatedPairs => "validated_pairs",
            MetricId::PairSyncChecked => "pair_sync_checked",
            MetricId::PairSyncPass => "pair_sync_pass",
            MetricId::PairCountMatch => "pair_count_match",
            MetricId::StrictPass => "strict_pass",
            MetricId::FailureClass => "failure_class",
            MetricId::Tool => "tool",
            MetricId::TrimPolyg => "trim_polyg",
            MetricId::MinPolygRun => "min_polyg_run",
            MetricId::BasesTrimmedPolyg => "bases_trimmed_polyg",
            MetricId::PolyxBankId => "polyx_bank_id",
            MetricId::PolyxBankHash => "polyx_bank_hash",
            MetricId::PolyxPreset => "polyx_preset",
            MetricId::DamageMode => "damage_mode",
            MetricId::ExecutionPolicy => "execution_policy",
            MetricId::RequestedTrim5pBases => "requested_trim_5p_bases",
            MetricId::RequestedTrim3pBases => "requested_trim_3p_bases",
            MetricId::UdgClassification => "udg_classification",
            MetricId::CtGaAsymmetryPre => "ct_ga_asymmetry_pre",
            MetricId::CtGaAsymmetryPost => "ct_ga_asymmetry_post",
            MetricId::ClassifiedFraction => "classified_fraction",
            MetricId::UnclassifiedFraction => "unclassified_fraction",
            MetricId::Classifier => "classifier",
            MetricId::ReportFormat => "report_format",
            MetricId::DatabaseCatalogId => "database_catalog_id",
            MetricId::DatabaseArtifactId => "database_artifact_id",
            MetricId::MinimumConfidence => "minimum_confidence",
            MetricId::EmitUnclassified => "emit_unclassified",
            MetricId::TopTaxa => "top_taxa",
            MetricId::QcRawDir => "qc_raw_dir",
            MetricId::QcTrimmedDir => "qc_trimmed_dir",
            MetricId::AggregationEngine => "aggregation_engine",
            MetricId::AggregationScope => "aggregation_scope",
            MetricId::GovernedQcInputCount => "governed_qc_input_count",
            MetricId::GovernedQcContributorStageIds => "governed_qc_contributor_stage_ids",
            MetricId::GovernedQcContributorToolIds => "governed_qc_contributor_tool_ids",
            MetricId::GovernedQcLineageHash => "governed_qc_lineage_hash",
            MetricId::MultiqcSampleCount => "multiqc_sample_count",
            MetricId::MultiqcModuleCount => "multiqc_module_count",
            MetricId::MultiqcReport => "multiqc_report",
            MetricId::MultiqcData => "multiqc_data",
        }
    }
}

impl DerivedMetricId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DerivedMetricId::ReadRetention => "read_retention",
            DerivedMetricId::BaseRetention => "base_retention",
            DerivedMetricId::MergeEfficiency => "merge_efficiency",
            DerivedMetricId::ErrorReductionProxy => "error_reduction_proxy",
        }
    }
}

#[must_use]
pub fn parse_metric_id(value: &str) -> Option<MetricId> {
    match value {
        "runtime_s" => Some(MetricId::RuntimeS),
        "memory_mb" => Some(MetricId::MemoryMb),
        "exit_code" => Some(MetricId::ExitCode),
        "reads_in" => Some(MetricId::ReadsIn),
        "reads_out" => Some(MetricId::ReadsOut),
        "reads_dropped" => Some(MetricId::ReadsDropped),
        "reads_removed_by_n" => Some(MetricId::ReadsRemovedByN),
        "reads_removed_by_entropy" => Some(MetricId::ReadsRemovedByEntropy),
        "reads_removed_low_complexity" => Some(MetricId::ReadsRemovedLowComplexity),
        "reads_removed_by_kmer" => Some(MetricId::ReadsRemovedByKmer),
        "reads_removed_contaminant_kmer" => Some(MetricId::ReadsRemovedContaminantKmer),
        "reads_removed_by_length" => Some(MetricId::ReadsRemovedByLength),
        "reads_total" => Some(MetricId::ReadsTotal),
        "reads_valid" => Some(MetricId::ReadsValid),
        "reads_invalid" => Some(MetricId::ReadsInvalid),
        "bases_in" => Some(MetricId::BasesIn),
        "bases_out" => Some(MetricId::BasesOut),
        "bases_total" => Some(MetricId::BasesTotal),
        "pairs_in" => Some(MetricId::PairsIn),
        "pairs_out" => Some(MetricId::PairsOut),
        "threads" => Some(MetricId::Threads),
        "reads_r1" => Some(MetricId::ReadsR1),
        "reads_r2" => Some(MetricId::ReadsR2),
        "reads_merged" => Some(MetricId::ReadsMerged),
        "reads_unmerged" => Some(MetricId::ReadsUnmerged),
        "mean_q_before" => Some(MetricId::MeanQBefore),
        "mean_q_after" => Some(MetricId::MeanQAfter),
        "mean_q" => Some(MetricId::MeanQ),
        "merge_rate" => Some(MetricId::MergeRate),
        "reads_with_umi" => Some(MetricId::ReadsWithUmi),
        "dedup_rate" => Some(MetricId::DedupRate),
        "kmer_fix_rate" => Some(MetricId::KmerFixRate),
        "candidate_adapter_count" => Some(MetricId::CandidateAdapterCount),
        "adapter_trimmed_fraction" => Some(MetricId::AdapterTrimmedFraction),
        "contamination_rate" => Some(MetricId::ContaminationRate),
        "contamination_summary" => Some(MetricId::ContaminationSummary),
        "gc_percent" => Some(MetricId::GcPercent),
        "length_histogram" => Some(MetricId::LengthHistogram),
        "read_count" => Some(MetricId::ReadCount),
        "mean_read_length" => Some(MetricId::MeanReadLength),
        "max_read_length" => Some(MetricId::MaxReadLength),
        "distinct_lengths" => Some(MetricId::DistinctLengths),
        "sequence_count" => Some(MetricId::SequenceCount),
        "flagged_sequences" => Some(MetricId::FlaggedSequences),
        "top_fraction" => Some(MetricId::TopFraction),
        "delta_metrics" => Some(MetricId::DeltaMetrics),
        "paired_mode" => Some(MetricId::PairedMode),
        "adapter_policy" => Some(MetricId::AdapterPolicy),
        "polyx_policy" => Some(MetricId::PolyxPolicy),
        "n_policy" => Some(MetricId::NPolicy),
        "contaminant_policy" => Some(MetricId::ContaminantPolicy),
        "raw_backend_report_format" => Some(MetricId::RawBackendReportFormat),
        "adapter_preset" => Some(MetricId::AdapterPreset),
        "adapter_bank_id" => Some(MetricId::AdapterBankId),
        "adapter_bank_hash" => Some(MetricId::AdapterBankHash),
        "adapter_overrides" => Some(MetricId::AdapterOverrides),
        "validated_inputs" => Some(MetricId::ValidatedInputs),
        "validated_pairs" => Some(MetricId::ValidatedPairs),
        "pair_sync_checked" => Some(MetricId::PairSyncChecked),
        "pair_sync_pass" => Some(MetricId::PairSyncPass),
        "pair_count_match" => Some(MetricId::PairCountMatch),
        "strict_pass" => Some(MetricId::StrictPass),
        "failure_class" => Some(MetricId::FailureClass),
        "tool" => Some(MetricId::Tool),
        "trim_polyg" => Some(MetricId::TrimPolyg),
        "min_polyg_run" => Some(MetricId::MinPolygRun),
        "bases_trimmed_polyg" => Some(MetricId::BasesTrimmedPolyg),
        "polyx_bank_id" => Some(MetricId::PolyxBankId),
        "polyx_bank_hash" => Some(MetricId::PolyxBankHash),
        "polyx_preset" => Some(MetricId::PolyxPreset),
        "damage_mode" => Some(MetricId::DamageMode),
        "execution_policy" => Some(MetricId::ExecutionPolicy),
        "requested_trim_5p_bases" => Some(MetricId::RequestedTrim5pBases),
        "requested_trim_3p_bases" => Some(MetricId::RequestedTrim3pBases),
        "udg_classification" => Some(MetricId::UdgClassification),
        "ct_ga_asymmetry_pre" => Some(MetricId::CtGaAsymmetryPre),
        "ct_ga_asymmetry_post" => Some(MetricId::CtGaAsymmetryPost),
        "classified_fraction" => Some(MetricId::ClassifiedFraction),
        "unclassified_fraction" => Some(MetricId::UnclassifiedFraction),
        "classifier" => Some(MetricId::Classifier),
        "report_format" => Some(MetricId::ReportFormat),
        "database_catalog_id" => Some(MetricId::DatabaseCatalogId),
        "database_artifact_id" => Some(MetricId::DatabaseArtifactId),
        "minimum_confidence" => Some(MetricId::MinimumConfidence),
        "emit_unclassified" => Some(MetricId::EmitUnclassified),
        "top_taxa" => Some(MetricId::TopTaxa),
        "qc_raw_dir" => Some(MetricId::QcRawDir),
        "qc_trimmed_dir" => Some(MetricId::QcTrimmedDir),
        "aggregation_engine" => Some(MetricId::AggregationEngine),
        "aggregation_scope" => Some(MetricId::AggregationScope),
        "governed_qc_input_count" => Some(MetricId::GovernedQcInputCount),
        "governed_qc_contributor_stage_ids" => Some(MetricId::GovernedQcContributorStageIds),
        "governed_qc_contributor_tool_ids" => Some(MetricId::GovernedQcContributorToolIds),
        "governed_qc_lineage_hash" => Some(MetricId::GovernedQcLineageHash),
        "multiqc_sample_count" => Some(MetricId::MultiqcSampleCount),
        "multiqc_module_count" => Some(MetricId::MultiqcModuleCount),
        "multiqc_report" => Some(MetricId::MultiqcReport),
        "multiqc_data" => Some(MetricId::MultiqcData),
        _ => None,
    }
}

#[must_use]
pub fn parse_derived_metric_id(value: &str) -> Option<DerivedMetricId> {
    match value {
        "read_retention" => Some(DerivedMetricId::ReadRetention),
        "base_retention" => Some(DerivedMetricId::BaseRetention),
        "merge_efficiency" => Some(DerivedMetricId::MergeEfficiency),
        "error_reduction_proxy" => Some(DerivedMetricId::ErrorReductionProxy),
        _ => None,
    }
}

/// # Errors
/// Returns an error if the metric id is unknown.
pub fn validate_metric_id_str(value: &str) -> Result<()> {
    parse_metric_id(value)
        .ok_or_else(|| BijuxError::validation(format!("unknown metric id {value}")))?;
    Ok(())
}

/// # Errors
/// Returns an error if the derived metric id is unknown.
/// # Errors
/// Returns an error if the derived metric id is unknown.
pub fn validate_derived_metric_id_str(value: &str) -> Result<()> {
    parse_derived_metric_id(value)
        .ok_or_else(|| BijuxError::validation(format!("unknown derived metric id {value}")))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSet<T> {
    pub metrics_schema: String,
    pub version: i32,
    pub metrics: T,
}

impl<T> MetricSet<T> {
    #[must_use]
    pub fn new(metrics_schema: String, version: i32, metrics: T) -> Self {
        Self {
            metrics_schema,
            version,
            metrics,
        }
    }
}

pub type MetricEnvelope<T> = MetricSet<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsEnvelope<T> {
    pub schema_version: String,
    #[serde(default = "ContractVersion::v1")]
    pub contract_version: ContractVersion,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub parameters_fingerprint: String,
    pub input_fingerprint: String,
    #[serde(default)]
    pub parameters_json_normalized: serde_json::Value,
    #[serde(default)]
    pub input_hashes: Vec<String>,
    #[serde(default)]
    pub metric_provenance: Option<MetricProvenanceV1>,
    pub metrics: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageMetricsV1<T> {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub context: MetricContextV1,
    #[serde(default)]
    pub metric_provenance: Option<MetricProvenanceV1>,
    pub execution: ExecutionMetrics,
    #[serde(default)]
    pub failure_class: Option<String>,
    #[serde(default)]
    pub failure_reason: Option<String>,
    pub metrics: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationV1 {
    pub schema_version: String,
    pub contract_version: ContractVersion,
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub tool_version: String,
    #[serde(default)]
    pub resolved_tool_version: Option<String>,
    pub image_digest: String,
    pub runner_kind: String,
    pub platform: String,
    pub parameters_json: serde_json::Value,
    pub parameters_json_normalized: serde_json::Value,
    #[serde(default)]
    pub effective_params_json: serde_json::Value,
    #[serde(default)]
    pub effective_params_json_normalized: serde_json::Value,
    #[serde(default)]
    pub params_provenance: serde_json::Value,
    #[serde(default)]
    pub params_provenance_normalized: serde_json::Value,
    #[serde(default)]
    pub adapter_bank: Option<AdapterBankProvenanceV1>,
    #[serde(default)]
    pub banks: Option<serde_json::Value>,
    #[serde(default)]
    pub bank_assets: Option<serde_json::Value>,
    pub resources: ToolConstraints,
    pub environment: BTreeMap<String, String>,
    pub input_hashes: Vec<String>,
    pub output_hashes: Vec<String>,
    #[serde(default)]
    pub executed_command: Option<String>,
}

impl ToolInvocationV1 {
    #[must_use]
    pub fn new(spec: ToolInvocationSpecV1) -> Self {
        Self {
            schema_version: spec.schema_version,
            contract_version: spec.contract_version,
            stage_id: spec.stage_id,
            tool_id: spec.tool_id,
            tool_version: spec.tool_version,
            resolved_tool_version: spec.resolved_tool_version,
            image_digest: spec.image_digest,
            runner_kind: spec.runner_kind,
            platform: spec.platform,
            parameters_json: spec.parameters_json,
            parameters_json_normalized: spec.parameters_json_normalized,
            effective_params_json: spec.effective_params_json,
            effective_params_json_normalized: spec.effective_params_json_normalized,
            params_provenance: spec.params_provenance,
            params_provenance_normalized: spec.params_provenance_normalized,
            adapter_bank: None,
            banks: None,
            bank_assets: None,
            resources: spec.resources,
            environment: spec.environment,
            input_hashes: spec.input_hashes,
            output_hashes: spec.output_hashes,
            executed_command: spec.executed_command,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationSpecV1 {
    pub schema_version: String,
    pub contract_version: ContractVersion,
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub tool_version: String,
    #[serde(default)]
    pub resolved_tool_version: Option<String>,
    pub image_digest: String,
    pub runner_kind: String,
    pub platform: String,
    pub parameters_json: serde_json::Value,
    pub parameters_json_normalized: serde_json::Value,
    #[serde(default)]
    pub effective_params_json: serde_json::Value,
    #[serde(default)]
    pub effective_params_json_normalized: serde_json::Value,
    #[serde(default)]
    pub params_provenance: serde_json::Value,
    #[serde(default)]
    pub params_provenance_normalized: serde_json::Value,
    pub resources: ToolConstraints,
    pub environment: BTreeMap<String, String>,
    pub input_hashes: Vec<String>,
    pub output_hashes: Vec<String>,
    #[serde(default)]
    pub executed_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBankProvenanceV1 {
    pub bank_id: String,
    pub bank_version: String,
    pub bank_hash: String,
    pub presets_hash: String,
    pub preset: String,
    pub preset_hash: String,
    pub enabled_categories: Vec<String>,
    pub disabled_categories: Vec<String>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    #[serde(default)]
    pub enabled_entries: Vec<BankEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankEntryV1 {
    pub id: String,
    pub sequence: String,
    pub rationale: String,
    pub source: String,
}
