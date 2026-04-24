use bijux_dna_core::contract::MetricProvenanceV1;
use serde::{Deserialize, Serialize};

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

impl FactsRowV1 {
    #[must_use]
    pub fn effective_metric_provenance(&self) -> MetricProvenanceV1 {
        MetricProvenanceV1 {
            run_id: self.run_id.clone(),
            stage_id: self.stage_id.clone(),
            tool_id: self.tool_id.clone(),
            tool_version: self.tool_version.clone(),
            params_hash: self.params_hash.clone(),
            input_artifact_hashes: vec![self.input_hash.clone()],
            manifest_hash: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunProvenanceV1 {
    pub schema_version: String,
    pub tool_image_digest: Option<String>,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hashes: Vec<String>,
    pub reference_genome: Option<String>,
    pub pipeline_id: String,
    pub git_commit: String,
    pub build_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunContextV1 {
    Local,
    Hpc { site: String, scratch: String, slurm: bool },
}
