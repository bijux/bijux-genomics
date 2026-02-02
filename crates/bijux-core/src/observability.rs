use serde::{Deserialize, Serialize};

use crate::{
    metrics::{AdapterBankProvenanceV1, MetricContextV1},
    ToolConstraints,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageObservabilityContractV1 {
    pub required_artifacts: Vec<String>,
    pub required_metadata_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageObservabilityContextV1 {
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub input_hash: String,
    pub params_hash: String,
    pub parameters_json: serde_json::Value,
    pub metric_context: MetricContextV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveConfigV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub stage_version: i32,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub runner: String,
    pub platform: String,
    pub resources: ToolConstraints,
    pub parameters_json: serde_json::Value,
    pub parameters_json_normalized: serde_json::Value,
    #[serde(default)]
    pub effective_params_json: serde_json::Value,
    #[serde(default)]
    pub effective_params_json_normalized: serde_json::Value,
    #[serde(default)]
    pub adapter_bank: Option<AdapterBankProvenanceV1>,
    #[serde(default)]
    pub banks: Option<serde_json::Value>,
    #[serde(default)]
    pub bank_assets: Option<serde_json::Value>,
}

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
    pub scope: String,
    pub condition: serde_json::Value,
    pub parameters_json: serde_json::Value,
    #[serde(default)]
    pub retention: Option<crate::metrics::RetentionReportMetricV1>,
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
#[serde(deny_unknown_fields)]
pub struct TelemetryEventV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub event_name: String,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
    pub status: String,
    pub trace_id: String,
    pub span_id: String,
    pub attrs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactsRowV1 {
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub trace_id: String,
    pub span_id: String,
    pub params_hash: String,
    pub input_hash: String,
    pub output_hashes: Vec<String>,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub bank_hashes: serde_json::Value,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub metrics: serde_json::Value,
    pub reports: serde_json::Value,
    pub artifacts: serde_json::Value,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum InvariantStatusV1 {
    Pass,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantResultV1 {
    pub id: String,
    pub status: InvariantStatusV1,
    pub message: String,
    #[serde(default)]
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageVerdictV1 {
    pub stage_id: String,
    pub verdict: InvariantStatusV1,
    pub reasons: Vec<String>,
    pub key_metrics: serde_json::Value,
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
}

pub fn canonicalize_json_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut ordered = serde_json::Map::new();
            for key in keys {
                let val = map.get(key).unwrap_or(&serde_json::Value::Null);
                ordered.insert(key.clone(), canonicalize_json_value(val));
            }
            serde_json::Value::Object(ordered)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(canonicalize_json_value).collect())
        }
        _ => value.clone(),
    }
}

#[must_use]
pub fn parameters_json_canonicalization(value: &serde_json::Value) -> serde_json::Value {
    fn normalize_numbers(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Number(num) => {
                if let Some(f) = num.as_f64() {
                    serde_json::Number::from_f64(f).map_or_else(
                        || serde_json::Value::Number(num.clone()),
                        serde_json::Value::Number,
                    )
                } else {
                    serde_json::Value::Number(num.clone())
                }
            }
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(normalize_numbers).collect())
            }
            serde_json::Value::Object(map) => {
                let mut ordered = serde_json::Map::new();
                for (key, val) in map {
                    ordered.insert(key.clone(), normalize_numbers(val));
                }
                serde_json::Value::Object(ordered)
            }
            _ => value.clone(),
        }
    }

    let canonical = canonicalize_json_value(value);
    normalize_numbers(&canonical)
}
