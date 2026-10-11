//! Typed inputs for the existing local benchmark pipeline configuration protocol.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::report::{
    BenchmarkLocalPipelineAncientDnaPolicy, BenchmarkLocalPipelineExecutionContext,
    BenchmarkLocalPipelineReferenceAssets, BenchmarkLocalPipelineReferenceContext,
    BenchmarkLocalPipelineVariantAssets,
};
use crate::contract::BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1;
use crate::model::BenchmarkLocalStageReadinessKind;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkLocalPipelineDagDomain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

impl BenchmarkLocalPipelineDagDomain {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
            Self::Vcf => "vcf",
            Self::Cross => "cross",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub pipeline_id: String,
    pub domain: BenchmarkLocalPipelineDagDomain,
    pub summary: String,
    pub default_corpus_id: String,
    #[serde(default)]
    pub reference_context: Option<BenchmarkLocalPipelineReferenceContext>,
    #[serde(default)]
    pub execution_context: Option<BenchmarkLocalPipelineExecutionContext>,
    #[serde(default)]
    pub project_sources: Vec<BenchmarkLocalPipelineProjectSourceConfig>,
    #[serde(default)]
    pub ancient_dna_policy: Option<BenchmarkLocalPipelineAncientDnaPolicy>,
    #[serde(default)]
    pub reference_assets: Option<BenchmarkLocalPipelineReferenceAssets>,
    #[serde(default)]
    pub variant_assets: Option<BenchmarkLocalPipelineVariantAssets>,
    pub nodes: Vec<BenchmarkLocalPipelineDagNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagMetadataConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub pipeline_id: String,
    pub domain: BenchmarkLocalPipelineDagDomain,
    pub summary: String,
    pub default_corpus_id: String,
    #[serde(default)]
    pub reference_context: Option<BenchmarkLocalPipelineReferenceContext>,
    #[serde(default)]
    pub execution_context: Option<BenchmarkLocalPipelineExecutionContext>,
    #[serde(default)]
    pub project_sources: Vec<BenchmarkLocalPipelineProjectSourceConfig>,
    #[serde(default)]
    pub ancient_dna_policy: Option<BenchmarkLocalPipelineAncientDnaPolicy>,
    #[serde(default)]
    pub reference_assets: Option<BenchmarkLocalPipelineReferenceAssets>,
    #[serde(default)]
    pub variant_assets: Option<BenchmarkLocalPipelineVariantAssets>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagStagesConfig {
    pub nodes: Vec<BenchmarkLocalPipelineDagNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineProjectSourceConfig {
    pub project_id: String,
    pub metadata_url: String,
    #[serde(default = "default_local_pipeline_project_retries")]
    pub retries: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagNode {
    pub node_id: String,
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub summary: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub external_inputs: Vec<String>,
    #[serde(default)]
    pub upstream_inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
}

fn default_local_pipeline_project_retries() -> u32 {
    3
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1})
}

#[must_use]
pub fn benchmark_local_pipeline_dag_config_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalPipelineDagConfig)
}

#[must_use]
pub fn benchmark_local_pipeline_dag_metadata_config_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalPipelineDagMetadataConfig)
}

#[must_use]
pub fn benchmark_local_pipeline_dag_stages_config_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalPipelineDagStagesConfig)
}
