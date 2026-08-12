//! Owner: bijux-dna-bench-model
//! Persisted validation evidence for governed local benchmark pipeline DAGs.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineReferenceContext {
    pub species_id: String,
    pub build_id: String,
    pub reference_bundle_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineExecutionContext {
    pub site_profile_id: String,
    #[serde(default)]
    pub default_campaign_config: Option<String>,
    #[serde(default)]
    pub frontend_surfaces: Vec<String>,
    #[serde(default)]
    pub compute_surfaces: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineProjectSource {
    pub project_id: String,
    pub metadata_url: String,
    pub retries: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineAncientDnaPolicy {
    pub aligner_mode: String,
    pub auto_bwa_aln_p75_threshold_bp: u32,
    pub adapter_cleanup_tool: String,
    pub merge_pairs_in_preprocess: bool,
    pub duplicate_stage_id: String,
    pub duplicate_tool: String,
    pub caller_tool: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineReferenceAssets {
    pub directory_name: String,
    pub genome_url: String,
    pub genome_filename: String,
    pub assembly_report_url: String,
    pub assembly_report_filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineVariantAssets {
    pub directory_name: String,
    pub source_vcf_url: String,
    pub source_vcf_filename: String,
    pub refseq_vcf_filename: String,
    pub annotated_vcf_filename: String,
    pub snpeff: BenchmarkLocalPipelineSnpEffAssets,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineSnpEffAssets {
    pub genome_id: String,
    pub database_url: String,
    pub database_filename: String,
    pub core_url: String,
    pub core_filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagValidationNode {
    pub node_id: String,
    pub stage_id: String,
    pub readiness_kind: String,
    pub dependency_count: usize,
    pub external_input_count: usize,
    pub upstream_input_count: usize,
    pub output_count: usize,
    pub depends_on: Vec<String>,
    pub external_inputs: Vec<String>,
    pub upstream_inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagValidationProfile {
    pub profile_id: String,
    pub check_count: usize,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalPipelineDagValidationReport {
    #[schemars(schema_with = "local_pipeline_dag_validation_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub output_path: String,
    pub pipeline_id: String,
    pub domain: String,
    pub default_corpus_id: String,
    pub summary: String,
    pub reference_context: Option<BenchmarkLocalPipelineReferenceContext>,
    pub execution_context: Option<BenchmarkLocalPipelineExecutionContext>,
    pub project_source_count: usize,
    pub project_sources: Vec<BenchmarkLocalPipelineProjectSource>,
    pub ancient_dna_policy: Option<BenchmarkLocalPipelineAncientDnaPolicy>,
    pub reference_assets: Option<BenchmarkLocalPipelineReferenceAssets>,
    pub variant_assets: Option<BenchmarkLocalPipelineVariantAssets>,
    pub node_count: usize,
    pub edge_count: usize,
    pub acyclic: bool,
    pub valid: bool,
    pub topological_order: Vec<String>,
    pub validation_profiles: Vec<BenchmarkLocalPipelineDagValidationProfile>,
    pub nodes: Vec<BenchmarkLocalPipelineDagValidationNode>,
}

#[must_use]
pub fn benchmark_local_pipeline_dag_validation_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalPipelineDagValidationReport)
}

fn local_pipeline_dag_validation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1
    })
}
