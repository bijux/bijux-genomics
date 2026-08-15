//! Owner: bijux-dna-bench-model
//! Persisted benchmark workspace, corpus, publication, and stage-input configuration.

use std::collections::BTreeMap;

use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkConfig {
    #[serde(default)]
    pub workspace: BenchmarkWorkspaceConfig,
    #[serde(default)]
    pub publication: BenchmarkPublicationConfig,
    #[serde(default)]
    pub corpora: BTreeMap<String, BenchmarkCorpusConfig>,
    #[serde(default)]
    pub stage_inputs: BenchmarkStageInputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkWorkspaceConfig {
    pub local: Option<BenchmarkWorkspaceLocal>,
    pub remote: Option<BenchmarkWorkspaceRemote>,
    pub layout: Option<BenchmarkWorkspaceLayout>,
    #[serde(default)]
    pub artifacts: BTreeMap<String, BenchmarkWorkspaceArtifact>,
    pub sync: Option<BenchmarkWorkspaceSync>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct BenchmarkWorkspaceLocal {
    pub results_root: Option<String>,
    pub cache_mirror_root: Option<String>,
    pub extra_data_root: Option<String>,
    pub reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkWorkspaceRemote {
    pub ssh_host: Option<String>,
    pub repo_root: Option<String>,
    pub cache_root: Option<String>,
    pub corpus_root: Option<String>,
    pub results_root: Option<String>,
    pub extra_data_root: Option<String>,
    pub containers_root: Option<String>,
    pub reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkWorkspaceLayout {
    pub stage_runs: Option<BenchmarkWorkspaceStageRuns>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct BenchmarkWorkspaceStageRuns {
    pub remote_results_template: Option<String>,
    pub local_cache_results_template: Option<String>,
    pub local_archive_results_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkWorkspaceArtifact {
    pub reference_index_template: Option<String>,
    pub database_root_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkWorkspaceSync {
    pub defaults: Option<BenchmarkWorkspaceSyncDefaults>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkWorkspaceSyncDefaults {
    pub pull_base: Option<String>,
    pub pull_mode: Option<String>,
    pub include_profile: Option<String>,
    pub exclude_profile: Option<String>,
    pub clean_context: Option<bool>,
    pub allow_dirty: Option<bool>,
    pub include_containers_manifest: Option<bool>,
    pub data_manifest_glob: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkPublicationConfig {
    #[serde(flatten)]
    pub corpora: BTreeMap<String, BenchmarkCorpusPublicationConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkCorpusConfig {
    pub spec_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct BenchmarkStageInputConfig {
    #[serde(default)]
    pub fastq_deplete_rrna: BenchmarkDepleteRrnaInputConfig,
    #[serde(default)]
    pub fastq_deplete_host: BenchmarkReferenceInputConfig,
    #[serde(default)]
    pub fastq_deplete_reference_contaminants: BenchmarkReferenceInputConfig,
    #[serde(default)]
    pub fastq_screen_taxonomy: BenchmarkScreenTaxonomyInputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkDepleteRrnaInputConfig {
    pub rrna_db: Option<String>,
    pub rrna_bundle_id: Option<String>,
    pub min_identity: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct BenchmarkReferenceInputConfig {
    pub reference_index: Option<String>,
    pub reference_catalog_id: Option<String>,
    pub reference_index_backend: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub struct BenchmarkScreenTaxonomyInputConfig {
    pub database_root: Option<String>,
    pub database_catalog_id: Option<String>,
    pub database_artifact_id: Option<String>,
    pub database_namespace: Option<String>,
    pub database_scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default, PartialEq, Eq)]
pub struct BenchmarkCorpusPublicationConfig {
    #[serde(default)]
    pub contracts: Vec<CorpusBenchmarkContract>,
    #[serde(default)]
    pub exclusions: Vec<CorpusBenchmarkExclusion>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct CorpusBenchmarkContract {
    pub stage_id: String,
    pub scenario_id: String,
    #[serde(default = "default_sample_scope")]
    pub sample_scope: String,
    #[serde(default)]
    pub tools: Vec<String>,
}

fn default_sample_scope() -> String {
    "full".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub struct CorpusBenchmarkExclusion {
    pub stage_id: String,
    pub reason: String,
}

#[must_use]
pub fn benchmark_config_schema() -> Schema {
    schemars::schema_for!(BenchmarkConfig)
}
