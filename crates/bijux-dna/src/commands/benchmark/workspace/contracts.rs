use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchmarkConfig {
    #[serde(default)]
    pub(crate) workspace: BenchmarkWorkspaceConfig,
    #[serde(default)]
    pub(crate) publication: BenchmarkPublicationConfig,
    #[serde(default)]
    pub(crate) corpora: BTreeMap<String, BenchmarkCorpusConfig>,
    #[serde(default)]
    pub(crate) stage_inputs: BenchmarkStageInputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceConfig {
    pub(crate) local: Option<BenchmarkWorkspaceLocal>,
    pub(crate) remote: Option<BenchmarkWorkspaceRemote>,
    pub(crate) layout: Option<BenchmarkWorkspaceLayout>,
    #[serde(default)]
    pub(crate) artifacts: BTreeMap<String, BenchmarkWorkspaceArtifact>,
    pub(crate) sync: Option<BenchmarkWorkspaceSync>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct BenchmarkWorkspaceLocal {
    pub(crate) results_root: Option<String>,
    pub(crate) cache_mirror_root: Option<String>,
    pub(crate) extra_data_root: Option<String>,
    pub(crate) reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceRemote {
    pub(crate) ssh_host: Option<String>,
    pub(crate) repo_root: Option<String>,
    pub(crate) cache_root: Option<String>,
    pub(crate) corpus_root: Option<String>,
    pub(crate) results_root: Option<String>,
    pub(crate) extra_data_root: Option<String>,
    pub(crate) containers_root: Option<String>,
    pub(crate) reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceLayout {
    pub(crate) stage_runs: Option<BenchmarkWorkspaceStageRuns>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct BenchmarkWorkspaceStageRuns {
    pub(crate) remote_results_template: Option<String>,
    pub(crate) local_cache_results_template: Option<String>,
    pub(crate) local_archive_results_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceArtifact {
    pub(crate) reference_index_template: Option<String>,
    pub(crate) database_root_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceSync {
    pub(crate) defaults: Option<BenchmarkWorkspaceSyncDefaults>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceSyncDefaults {
    pub(crate) pull_base: Option<String>,
    pub(crate) pull_mode: Option<String>,
    pub(crate) include_profile: Option<String>,
    pub(crate) exclude_profile: Option<String>,
    pub(crate) clean_context: Option<bool>,
    pub(crate) allow_dirty: Option<bool>,
    pub(crate) include_containers_manifest: Option<bool>,
    pub(crate) data_manifest_glob: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkPublicationConfig {
    #[serde(flatten)]
    pub(crate) corpora: BTreeMap<String, BenchmarkCorpusPublicationConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkCorpusConfig {
    pub(crate) spec_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct BenchmarkStageInputConfig {
    #[serde(default)]
    pub(crate) fastq_deplete_rrna: BenchmarkDepleteRrnaInputConfig,
    #[serde(default)]
    pub(crate) fastq_deplete_host: BenchmarkReferenceInputConfig,
    #[serde(default)]
    pub(crate) fastq_deplete_reference_contaminants: BenchmarkReferenceInputConfig,
    #[serde(default)]
    pub(crate) fastq_screen_taxonomy: BenchmarkScreenTaxonomyInputConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkDepleteRrnaInputConfig {
    pub(crate) rrna_db: Option<String>,
    pub(crate) rrna_bundle_id: Option<String>,
    pub(crate) min_identity: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct BenchmarkReferenceInputConfig {
    pub(crate) reference_index: Option<String>,
    pub(crate) reference_catalog_id: Option<String>,
    pub(crate) reference_index_backend: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
#[allow(clippy::struct_field_names)]
pub(crate) struct BenchmarkScreenTaxonomyInputConfig {
    pub(crate) database_root: Option<String>,
    pub(crate) database_catalog_id: Option<String>,
    pub(crate) database_artifact_id: Option<String>,
    pub(crate) database_namespace: Option<String>,
    pub(crate) database_scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkCorpusPublicationConfig {
    #[serde(default)]
    pub(crate) contracts: Vec<CorpusBenchmarkContract>,
    #[serde(default)]
    pub(crate) exclusions: Vec<CorpusBenchmarkExclusion>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusBenchmarkContract {
    pub(crate) stage_id: String,
    pub(crate) scenario_id: String,
    #[serde(default = "default_sample_scope")]
    pub(crate) sample_scope: String,
    #[serde(default)]
    pub(crate) tools: Vec<String>,
}

fn default_sample_scope() -> String {
    "full".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusBenchmarkExclusion {
    pub(crate) stage_id: String,
    pub(crate) reason: String,
}
