use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::measure::ExecutionMetrics;
use crate::ToolConstraints;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankRefV1 {
    pub bank_id: String,
    pub bank_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricContextV1 {
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub params_hash: String,
    #[serde(default)]
    pub presets: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub banks: std::collections::BTreeMap<String, BankRefV1>,
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
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub runner_kind: String,
    pub platform: String,
    pub parameters_json: serde_json::Value,
    pub parameters_json_normalized: serde_json::Value,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDeltaMetricsV1 {
    pub read_retention: f64,
    pub base_retention: f64,
    pub mean_q_delta: f64,
    pub gc_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionReportMetricV1 {
    pub value: f64,
    pub numerator_reads: u64,
    pub denominator_reads: u64,
    pub numerator_bases: u64,
    pub denominator_bases: u64,
    pub definition: String,
    pub stage_boundary: String,
    pub conditions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqFilterMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_dropped: u64,
    #[serde(default)]
    pub reads_removed_by_n: u64,
    #[serde(default)]
    pub reads_removed_by_entropy: u64,
    #[serde(default)]
    pub reads_removed_low_complexity: u64,
    #[serde(default)]
    pub reads_removed_by_kmer: u64,
    #[serde(default)]
    pub reads_removed_contaminant_kmer: u64,
    #[serde(default)]
    pub reads_removed_by_length: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqMergeMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: u64,
    pub pairs_out: u64,
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqQcPostMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q: f64,
    pub contamination_rate: f64,
    #[serde(default)]
    pub raw_fastqc_dir: Option<String>,
    #[serde(default)]
    pub trimmed_fastqc_dir: Option<String>,
    #[serde(default)]
    pub multiqc_report: Option<String>,
    #[serde(default)]
    pub multiqc_data: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqCorrectMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqUmiMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqPreprocessMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
}
