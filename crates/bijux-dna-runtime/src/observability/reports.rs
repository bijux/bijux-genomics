use serde::{Deserialize, Serialize};

use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub metrics_path: String,
    pub tool_invocation_path: String,
    pub effective_config_path: String,
    #[serde(default)]
    pub effective_config_hash: Option<String>,
    pub facts_row_id: Option<String>,
    pub summary: serde_json::Value,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<InvariantResultV1>,
    #[serde(default)]
    pub verdict: Option<StageVerdictV1>,
    pub outputs: Vec<String>,
    pub subreports: Vec<String>,
    pub log_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub boundary: String,
    pub numerator: serde_json::Value,
    pub denominator: serde_json::Value,
    pub units: String,
    pub scope: String,
    pub condition: serde_json::Value,
    pub parameters_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrimReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub bases_trimmed: u64,
    pub per_adapter_counts: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    pub adapter_preset: Option<String>,
    #[serde(default)]
    pub adapter_bank_id: Option<String>,
    #[serde(default)]
    pub adapter_bank_hash: Option<String>,
    #[serde(default)]
    pub adapter_overrides: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidateReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub integrity_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QcPostReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub raw_fastqc_dir: Option<String>,
    pub trimmed_fastqc_dir: Option<String>,
    pub multiqc_report: Option<String>,
    pub multiqc_data: Option<String>,
    pub fastqc_raw_modules: serde_json::Value,
    pub fastqc_trimmed_modules: serde_json::Value,
    #[serde(default)]
    pub fastqc_metrics_v2_path: Option<String>,
    #[serde(default)]
    pub suggested_adapters_path: Option<String>,
    #[serde(default)]
    pub suggested_preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilterReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_removed_total: u64,
    pub reads_removed_by_n: u64,
    pub reads_removed_by_entropy: u64,
    #[serde(default)]
    pub reads_removed_low_complexity: u64,
    pub reads_removed_by_kmer: u64,
    #[serde(default)]
    pub reads_removed_contaminant_kmer: u64,
    pub reads_removed_by_length: u64,
    #[serde(default)]
    pub entropy_distribution: serde_json::Value,
    pub conditions: serde_json::Value,
    #[serde(default)]
    pub redundant_filters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MergeReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchemaV1 {
    pub schema_version: String,
    pub contract: ReportContractV1,
    pub run_id: String,
    pub completeness: ReportCompletenessV1,
    pub stages: Vec<ReportStageSummaryV1>,
    pub provenance: Vec<ReportProvenanceV1>,
    pub retention_definition: Vec<RetentionDefinitionV1>,
    pub retention_context: Vec<RetentionContextV1>,
    pub assets_provenance: Vec<AssetsProvenanceV1>,
    pub metric_semantics: Vec<MetricSemanticsV1>,
    #[serde(default)]
    pub telemetry: serde_json::Value,
    #[serde(default)]
    pub qc_improvement: serde_json::Value,
    #[serde(default)]
    pub final_qc_summary: serde_json::Value,
    #[serde(default)]
    pub filter_interpretation: serde_json::Value,
    #[serde(default)]
    pub adapter_inference: serde_json::Value,
    #[serde(default)]
    pub pipeline_verdict: Option<PipelineVerdictV1>,
    #[serde(default)]
    pub sections: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineVerdictV1 {
    pub verdict: InvariantStatusV1,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportContractV1 {
    pub schema_version: String,
    pub required_sections: Vec<String>,
    pub required_provenance_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportCompletenessV1 {
    pub status: String,
    pub missing_metrics: Vec<String>,
    pub missing_reports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportStageSummaryV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hash: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub metrics_path: String,
    pub tool_invocation_path: String,
    pub effective_config_path: String,
    pub stage_report_path: String,
    #[serde(default)]
    pub retention_report_path: Option<String>,
    #[serde(default)]
    pub bank_report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportProvenanceV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub trace_id: String,
    pub span_id: String,
    pub params_hash: String,
    pub bank_hashes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionDefinitionV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub numerator: String,
    pub denominator: String,
    pub conditions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionContextV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub definition: String,
    pub conditions: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetsProvenanceV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub banks: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSemanticsV1 {
    pub metric_id: String,
    pub direction: String,
    pub units: String,
    pub range: String,
    pub missing_data_policy: String,
    #[serde(default)]
    pub influencing_params: Vec<String>,
}
