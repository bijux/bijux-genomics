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
    DuplicateReads,
    MeanQBefore,
    MeanQAfter,
    MeanQ,
    MergeRate,
    ReadsWithUmi,
    DedupRate,
    KmerFixRate,
    CandidateAdapterCount,
    AdapterTrimmedFraction,
    DetectionConfidence,
    DetectionThreshold,
    PrimerTrimmedFraction,
    OrientationForwardFraction,
    HostFractionRemoved,
    ContaminantFractionRemoved,
    RrnaFractionRemoved,
    DepletionSummary,
    ContaminationRate,
    ContaminationSummary,
    GcPercent,
    LengthHistogram,
    ReadCount,
    MinReadLength,
    MeanReadLength,
    MedianReadLength,
    MaxReadLength,
    DistinctLengths,
    ReferenceBytes,
    IndexBytes,
    IndexFileCount,
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
    DedupMode,
    KeepOrder,
    DuplicateClassCount,
    DuplicateProvenanceJson,
    AdapterPreset,
    AdapterBankId,
    AdapterBankHash,
    AdapterOverrides,
    AdapterReport,
    DetectedAdapterIds,
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

const fn metric_id_execution_str(metric_id: MetricId) -> Option<&'static str> {
    match metric_id {
        MetricId::RuntimeS => Some("runtime_s"),
        MetricId::MemoryMb => Some("memory_mb"),
        MetricId::ExitCode => Some("exit_code"),
        MetricId::ReadsIn => Some("reads_in"),
        MetricId::ReadsOut => Some("reads_out"),
        MetricId::ReadsDropped => Some("reads_dropped"),
        MetricId::ReadsRemovedByN => Some("reads_removed_by_n"),
        MetricId::ReadsRemovedByEntropy => Some("reads_removed_by_entropy"),
        MetricId::ReadsRemovedLowComplexity => Some("reads_removed_low_complexity"),
        MetricId::ReadsRemovedByKmer => Some("reads_removed_by_kmer"),
        MetricId::ReadsRemovedContaminantKmer => Some("reads_removed_contaminant_kmer"),
        MetricId::ReadsRemovedByLength => Some("reads_removed_by_length"),
        MetricId::ReadsTotal => Some("reads_total"),
        MetricId::ReadsValid => Some("reads_valid"),
        MetricId::ReadsInvalid => Some("reads_invalid"),
        MetricId::BasesIn => Some("bases_in"),
        MetricId::BasesOut => Some("bases_out"),
        MetricId::BasesTotal => Some("bases_total"),
        MetricId::PairsIn => Some("pairs_in"),
        MetricId::PairsOut => Some("pairs_out"),
        MetricId::Threads => Some("threads"),
        MetricId::ReadsR1 => Some("reads_r1"),
        MetricId::ReadsR2 => Some("reads_r2"),
        MetricId::ReadsMerged => Some("reads_merged"),
        MetricId::ReadsUnmerged => Some("reads_unmerged"),
        MetricId::DuplicateReads => Some("duplicate_reads"),
        _ => None,
    }
}

const fn metric_id_quality_str(metric_id: MetricId) -> Option<&'static str> {
    match metric_id {
        MetricId::MeanQBefore => Some("mean_q_before"),
        MetricId::MeanQAfter => Some("mean_q_after"),
        MetricId::MeanQ => Some("mean_q"),
        MetricId::MergeRate => Some("merge_rate"),
        MetricId::ReadsWithUmi => Some("reads_with_umi"),
        MetricId::DedupRate => Some("dedup_rate"),
        MetricId::KmerFixRate => Some("kmer_fix_rate"),
        MetricId::CandidateAdapterCount => Some("candidate_adapter_count"),
        MetricId::AdapterTrimmedFraction => Some("adapter_trimmed_fraction"),
        MetricId::DetectionConfidence => Some("detection_confidence"),
        MetricId::DetectionThreshold => Some("detection_threshold"),
        MetricId::PrimerTrimmedFraction => Some("primer_trimmed_fraction"),
        MetricId::OrientationForwardFraction => Some("orientation_forward_fraction"),
        MetricId::HostFractionRemoved => Some("host_fraction_removed"),
        MetricId::ContaminantFractionRemoved => Some("contaminant_fraction_removed"),
        MetricId::RrnaFractionRemoved => Some("rrna_fraction_removed"),
        MetricId::DepletionSummary => Some("depletion_summary"),
        MetricId::ContaminationRate => Some("contamination_rate"),
        MetricId::ContaminationSummary => Some("contamination_summary"),
        MetricId::GcPercent => Some("gc_percent"),
        MetricId::LengthHistogram => Some("length_histogram"),
        MetricId::ReadCount => Some("read_count"),
        MetricId::MinReadLength => Some("min_read_length"),
        MetricId::MeanReadLength => Some("mean_read_length"),
        MetricId::MedianReadLength => Some("median_read_length"),
        MetricId::MaxReadLength => Some("max_read_length"),
        MetricId::DistinctLengths => Some("distinct_lengths"),
        MetricId::ReferenceBytes => Some("reference_bytes"),
        MetricId::IndexBytes => Some("index_bytes"),
        MetricId::IndexFileCount => Some("index_file_count"),
        MetricId::SequenceCount => Some("sequence_count"),
        MetricId::FlaggedSequences => Some("flagged_sequences"),
        MetricId::TopFraction => Some("top_fraction"),
        MetricId::DeltaMetrics => Some("delta_metrics"),
        _ => None,
    }
}

const fn metric_id_policy_str(metric_id: MetricId) -> Option<&'static str> {
    match metric_id {
        MetricId::PairedMode => Some("paired_mode"),
        MetricId::AdapterPolicy => Some("adapter_policy"),
        MetricId::PolyxPolicy => Some("polyx_policy"),
        MetricId::NPolicy => Some("n_policy"),
        MetricId::ContaminantPolicy => Some("contaminant_policy"),
        MetricId::RawBackendReportFormat => Some("raw_backend_report_format"),
        MetricId::DedupMode => Some("dedup_mode"),
        MetricId::KeepOrder => Some("keep_order"),
        MetricId::DuplicateClassCount => Some("duplicate_class_count"),
        MetricId::DuplicateProvenanceJson => Some("duplicate_provenance_json"),
        MetricId::AdapterPreset => Some("adapter_preset"),
        MetricId::AdapterBankId => Some("adapter_bank_id"),
        MetricId::AdapterBankHash => Some("adapter_bank_hash"),
        MetricId::AdapterOverrides => Some("adapter_overrides"),
        MetricId::AdapterReport => Some("adapter_report"),
        MetricId::DetectedAdapterIds => Some("detected_adapter_ids"),
        MetricId::ValidatedInputs => Some("validated_inputs"),
        MetricId::ValidatedPairs => Some("validated_pairs"),
        MetricId::PairSyncChecked => Some("pair_sync_checked"),
        MetricId::PairSyncPass => Some("pair_sync_pass"),
        MetricId::PairCountMatch => Some("pair_count_match"),
        MetricId::StrictPass => Some("strict_pass"),
        MetricId::FailureClass => Some("failure_class"),
        MetricId::Tool => Some("tool"),
        _ => None,
    }
}

const fn metric_id_damage_taxonomy_str(metric_id: MetricId) -> Option<&'static str> {
    match metric_id {
        MetricId::TrimPolyg => Some("trim_polyg"),
        MetricId::MinPolygRun => Some("min_polyg_run"),
        MetricId::BasesTrimmedPolyg => Some("bases_trimmed_polyg"),
        MetricId::PolyxBankId => Some("polyx_bank_id"),
        MetricId::PolyxBankHash => Some("polyx_bank_hash"),
        MetricId::PolyxPreset => Some("polyx_preset"),
        MetricId::DamageMode => Some("damage_mode"),
        MetricId::ExecutionPolicy => Some("execution_policy"),
        MetricId::RequestedTrim5pBases => Some("requested_trim_5p_bases"),
        MetricId::RequestedTrim3pBases => Some("requested_trim_3p_bases"),
        MetricId::UdgClassification => Some("udg_classification"),
        MetricId::CtGaAsymmetryPre => Some("ct_ga_asymmetry_pre"),
        MetricId::CtGaAsymmetryPost => Some("ct_ga_asymmetry_post"),
        MetricId::ClassifiedFraction => Some("classified_fraction"),
        MetricId::UnclassifiedFraction => Some("unclassified_fraction"),
        MetricId::Classifier => Some("classifier"),
        MetricId::ReportFormat => Some("report_format"),
        MetricId::DatabaseCatalogId => Some("database_catalog_id"),
        MetricId::DatabaseArtifactId => Some("database_artifact_id"),
        MetricId::MinimumConfidence => Some("minimum_confidence"),
        MetricId::EmitUnclassified => Some("emit_unclassified"),
        MetricId::TopTaxa => Some("top_taxa"),
        _ => None,
    }
}

const fn metric_id_reporting_str(metric_id: MetricId) -> Option<&'static str> {
    match metric_id {
        MetricId::QcRawDir => Some("qc_raw_dir"),
        MetricId::QcTrimmedDir => Some("qc_trimmed_dir"),
        MetricId::AggregationEngine => Some("aggregation_engine"),
        MetricId::AggregationScope => Some("aggregation_scope"),
        MetricId::GovernedQcInputCount => Some("governed_qc_input_count"),
        MetricId::GovernedQcContributorStageIds => Some("governed_qc_contributor_stage_ids"),
        MetricId::GovernedQcContributorToolIds => Some("governed_qc_contributor_tool_ids"),
        MetricId::GovernedQcLineageHash => Some("governed_qc_lineage_hash"),
        MetricId::MultiqcSampleCount => Some("multiqc_sample_count"),
        MetricId::MultiqcModuleCount => Some("multiqc_module_count"),
        MetricId::MultiqcReport => Some("multiqc_report"),
        MetricId::MultiqcData => Some("multiqc_data"),
        _ => None,
    }
}

impl MetricId {
    #[must_use]
    /// # Panics
    /// Panics only if a newly added `MetricId` variant is not assigned a stable string mapping.
    pub const fn as_str(self) -> &'static str {
        if let Some(metric_name) = metric_id_execution_str(self) {
            metric_name
        } else if let Some(metric_name) = metric_id_quality_str(self) {
            metric_name
        } else if let Some(metric_name) = metric_id_policy_str(self) {
            metric_name
        } else if let Some(metric_name) = metric_id_damage_taxonomy_str(self) {
            metric_name
        } else if let Some(metric_name) = metric_id_reporting_str(self) {
            metric_name
        } else {
            panic!("unhandled metric id")
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

fn parse_execution_metric_id(value: &str) -> Option<MetricId> {
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
        "duplicate_reads" => Some(MetricId::DuplicateReads),
        _ => None,
    }
}

fn parse_quality_metric_id(value: &str) -> Option<MetricId> {
    match value {
        "mean_q_before" => Some(MetricId::MeanQBefore),
        "mean_q_after" => Some(MetricId::MeanQAfter),
        "mean_q" => Some(MetricId::MeanQ),
        "merge_rate" => Some(MetricId::MergeRate),
        "reads_with_umi" => Some(MetricId::ReadsWithUmi),
        "dedup_rate" => Some(MetricId::DedupRate),
        "kmer_fix_rate" => Some(MetricId::KmerFixRate),
        "candidate_adapter_count" => Some(MetricId::CandidateAdapterCount),
        "adapter_trimmed_fraction" => Some(MetricId::AdapterTrimmedFraction),
        "detection_confidence" => Some(MetricId::DetectionConfidence),
        "detection_threshold" => Some(MetricId::DetectionThreshold),
        "primer_trimmed_fraction" => Some(MetricId::PrimerTrimmedFraction),
        "orientation_forward_fraction" => Some(MetricId::OrientationForwardFraction),
        "host_fraction_removed" => Some(MetricId::HostFractionRemoved),
        "contaminant_fraction_removed" => Some(MetricId::ContaminantFractionRemoved),
        "rrna_fraction_removed" => Some(MetricId::RrnaFractionRemoved),
        "depletion_summary" => Some(MetricId::DepletionSummary),
        "contamination_rate" => Some(MetricId::ContaminationRate),
        "contamination_summary" => Some(MetricId::ContaminationSummary),
        "gc_percent" => Some(MetricId::GcPercent),
        "length_histogram" => Some(MetricId::LengthHistogram),
        "read_count" => Some(MetricId::ReadCount),
        "mean_read_length" => Some(MetricId::MeanReadLength),
        "max_read_length" => Some(MetricId::MaxReadLength),
        "distinct_lengths" => Some(MetricId::DistinctLengths),
        "reference_bytes" => Some(MetricId::ReferenceBytes),
        "index_bytes" => Some(MetricId::IndexBytes),
        "index_file_count" => Some(MetricId::IndexFileCount),
        "sequence_count" => Some(MetricId::SequenceCount),
        "flagged_sequences" => Some(MetricId::FlaggedSequences),
        "top_fraction" => Some(MetricId::TopFraction),
        "delta_metrics" => Some(MetricId::DeltaMetrics),
        _ => None,
    }
}

fn parse_policy_metric_id(value: &str) -> Option<MetricId> {
    match value {
        "paired_mode" => Some(MetricId::PairedMode),
        "adapter_policy" => Some(MetricId::AdapterPolicy),
        "polyx_policy" => Some(MetricId::PolyxPolicy),
        "n_policy" => Some(MetricId::NPolicy),
        "contaminant_policy" => Some(MetricId::ContaminantPolicy),
        "raw_backend_report_format" => Some(MetricId::RawBackendReportFormat),
        "dedup_mode" => Some(MetricId::DedupMode),
        "keep_order" => Some(MetricId::KeepOrder),
        "duplicate_class_count" => Some(MetricId::DuplicateClassCount),
        "duplicate_provenance_json" => Some(MetricId::DuplicateProvenanceJson),
        "adapter_preset" => Some(MetricId::AdapterPreset),
        "adapter_bank_id" => Some(MetricId::AdapterBankId),
        "adapter_bank_hash" => Some(MetricId::AdapterBankHash),
        "adapter_overrides" => Some(MetricId::AdapterOverrides),
        "adapter_report" => Some(MetricId::AdapterReport),
        "detected_adapter_ids" => Some(MetricId::DetectedAdapterIds),
        "validated_inputs" => Some(MetricId::ValidatedInputs),
        "validated_pairs" => Some(MetricId::ValidatedPairs),
        "pair_sync_checked" => Some(MetricId::PairSyncChecked),
        "pair_sync_pass" => Some(MetricId::PairSyncPass),
        "pair_count_match" => Some(MetricId::PairCountMatch),
        "strict_pass" => Some(MetricId::StrictPass),
        "failure_class" => Some(MetricId::FailureClass),
        "tool" => Some(MetricId::Tool),
        _ => None,
    }
}

fn parse_damage_taxonomy_metric_id(value: &str) -> Option<MetricId> {
    match value {
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
        _ => None,
    }
}

fn parse_reporting_metric_id(value: &str) -> Option<MetricId> {
    match value {
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
pub fn parse_metric_id(value: &str) -> Option<MetricId> {
    parse_execution_metric_id(value)
        .or_else(|| parse_quality_metric_id(value))
        .or_else(|| parse_policy_metric_id(value))
        .or_else(|| parse_damage_taxonomy_metric_id(value))
        .or_else(|| parse_reporting_metric_id(value))
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
        Self { metrics_schema, version, metrics }
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
