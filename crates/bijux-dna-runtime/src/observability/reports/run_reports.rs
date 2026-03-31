use serde::{Deserialize, Serialize};

use bijux_dna_core::prelude::invariants::InvariantStatusV1;

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
