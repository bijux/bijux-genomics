use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::contract::{ContractVersion, ToolConstraints};
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
    ReadsR1,
    ReadsR2,
    ReadsMerged,
    ReadsUnmerged,
    MeanQBefore,
    MeanQAfter,
    MeanQ,
    MergeRate,
    DedupRate,
    KmerFixRate,
    ContaminationRate,
    ContaminationSummary,
    GcPercent,
    LengthHistogram,
    DeltaMetrics,
    AdapterPreset,
    AdapterBankId,
    AdapterBankHash,
    AdapterOverrides,
    QcRawDir,
    QcTrimmedDir,
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
            MetricId::ReadsR1 => "reads_r1",
            MetricId::ReadsR2 => "reads_r2",
            MetricId::ReadsMerged => "reads_merged",
            MetricId::ReadsUnmerged => "reads_unmerged",
            MetricId::MeanQBefore => "mean_q_before",
            MetricId::MeanQAfter => "mean_q_after",
            MetricId::MeanQ => "mean_q",
            MetricId::MergeRate => "merge_rate",
            MetricId::DedupRate => "dedup_rate",
            MetricId::KmerFixRate => "kmer_fix_rate",
            MetricId::ContaminationRate => "contamination_rate",
            MetricId::ContaminationSummary => "contamination_summary",
            MetricId::GcPercent => "gc_percent",
            MetricId::LengthHistogram => "length_histogram",
            MetricId::DeltaMetrics => "delta_metrics",
            MetricId::AdapterPreset => "adapter_preset",
            MetricId::AdapterBankId => "adapter_bank_id",
            MetricId::AdapterBankHash => "adapter_bank_hash",
            MetricId::AdapterOverrides => "adapter_overrides",
            MetricId::QcRawDir => "qc_raw_dir",
            MetricId::QcTrimmedDir => "qc_trimmed_dir",
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
        "reads_r1" => Some(MetricId::ReadsR1),
        "reads_r2" => Some(MetricId::ReadsR2),
        "reads_merged" => Some(MetricId::ReadsMerged),
        "reads_unmerged" => Some(MetricId::ReadsUnmerged),
        "mean_q_before" => Some(MetricId::MeanQBefore),
        "mean_q_after" => Some(MetricId::MeanQAfter),
        "mean_q" => Some(MetricId::MeanQ),
        "merge_rate" => Some(MetricId::MergeRate),
        "dedup_rate" => Some(MetricId::DedupRate),
        "kmer_fix_rate" => Some(MetricId::KmerFixRate),
        "contamination_rate" => Some(MetricId::ContaminationRate),
        "contamination_summary" => Some(MetricId::ContaminationSummary),
        "gc_percent" => Some(MetricId::GcPercent),
        "length_histogram" => Some(MetricId::LengthHistogram),
        "delta_metrics" => Some(MetricId::DeltaMetrics),
        "adapter_preset" => Some(MetricId::AdapterPreset),
        "adapter_bank_id" => Some(MetricId::AdapterBankId),
        "adapter_bank_hash" => Some(MetricId::AdapterBankHash),
        "adapter_overrides" => Some(MetricId::AdapterOverrides),
        "qc_raw_dir" => Some(MetricId::QcRawDir),
        "qc_trimmed_dir" => Some(MetricId::QcTrimmedDir),
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
