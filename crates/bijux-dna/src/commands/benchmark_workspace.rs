use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

pub(crate) const BENCHMARK_WORKSPACE_CONFIG_ENV: &str = "BIJUX_FASTQ_CORPUS_CONFIG";
pub(crate) const DEFAULT_BENCHMARK_WORKSPACE_CONFIG: &str = "configs/bench/workspace.toml";
pub(crate) const DEFAULT_BENCHMARK_PUBLICATION_CONFIG: &str = "configs/bench/publication.toml";

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceConfig {
    pub(crate) local: Option<BenchmarkWorkspaceLocal>,
    pub(crate) remote: Option<BenchmarkWorkspaceRemote>,
    pub(crate) layout: Option<BenchmarkWorkspaceLayout>,
    #[serde(default)]
    pub(crate) artifacts: BTreeMap<String, BenchmarkWorkspaceArtifact>,
    pub(crate) sync: Option<BenchmarkWorkspaceSync>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceLocal {
    pub(crate) results_root: Option<String>,
    pub(crate) cache_mirror_root: Option<String>,
    pub(crate) extra_data_root: Option<String>,
    pub(crate) reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceRemote {
    pub(crate) ssh_host: Option<String>,
    pub(crate) repo_root: Option<String>,
    pub(crate) cache_root: Option<String>,
    pub(crate) corpus_root: Option<String>,
    pub(crate) results_root: Option<String>,
    pub(crate) results_legacy_root: Option<String>,
    pub(crate) extra_data_root: Option<String>,
    pub(crate) containers_root: Option<String>,
    pub(crate) reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceLayout {
    pub(crate) stage_runs: Option<BenchmarkWorkspaceStageRuns>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceStageRuns {
    pub(crate) remote_results_template: Option<String>,
    pub(crate) local_cache_results_template: Option<String>,
    pub(crate) local_archive_results_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceArtifact {
    pub(crate) reference_index_template: Option<String>,
    pub(crate) database_root_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkWorkspaceSync {
    pub(crate) defaults: Option<BenchmarkWorkspaceSyncDefaults>,
}

#[derive(Debug, Clone, Deserialize, Default)]
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

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct BenchmarkPublicationConfig {
    pub(crate) corpus_01: Option<Corpus01PublicationConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct Corpus01PublicationConfig {
    #[serde(default)]
    pub(crate) contracts: Vec<CorpusBenchmarkContract>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

fn config_path_env_override() -> Option<PathBuf> {
    std::env::var_os(BENCHMARK_WORKSPACE_CONFIG_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn resolve_config_path(cwd: &Path, explicit_path: Option<&Path>, default_rel: &str) -> PathBuf {
    if let Some(path) = explicit_path {
        return if path.is_absolute() {
            path.to_path_buf()
        } else {
            cwd.join(path)
        };
    }
    if default_rel == DEFAULT_BENCHMARK_WORKSPACE_CONFIG {
        if let Some(path) = config_path_env_override() {
            return if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            };
        }
    }
    cwd.join(default_rel)
}

pub(crate) fn benchmark_workspace_config_path(cwd: &Path, explicit_path: Option<&Path>) -> PathBuf {
    resolve_config_path(cwd, explicit_path, DEFAULT_BENCHMARK_WORKSPACE_CONFIG)
}

pub(crate) fn benchmark_publication_config_path(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> PathBuf {
    resolve_config_path(cwd, explicit_path, DEFAULT_BENCHMARK_PUBLICATION_CONFIG)
}

pub(crate) fn load_optional_benchmark_workspace_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<Option<BenchmarkWorkspaceConfig>> {
    let path = benchmark_workspace_config_path(cwd, explicit_path);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let parsed = toml::from_str::<BenchmarkWorkspaceConfig>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(parsed))
}

pub(crate) fn load_benchmark_workspace_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkWorkspaceConfig> {
    let path = benchmark_workspace_config_path(cwd, explicit_path);
    load_optional_benchmark_workspace_config(cwd, explicit_path)?
        .ok_or_else(|| anyhow!("missing benchmark workspace config: {}", path.display()))
}

pub(crate) fn load_benchmark_publication_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkPublicationConfig> {
    let path = benchmark_publication_config_path(cwd, explicit_path);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<BenchmarkPublicationConfig>(&raw)
        .with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn benchmark_workspace_value(
    cwd: &Path,
    explicit_path: Option<&Path>,
    key: &str,
) -> Result<String> {
    let workspace = load_benchmark_workspace_config(cwd, explicit_path)?;
    match key {
        "local.results_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.results_root.clone()),
        "local.cache_mirror_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.cache_mirror_root.clone()),
        "local.extra_data_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.extra_data_root.clone()),
        "local.reference_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.reference_root.clone()),
        "remote.ssh_host" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.ssh_host.clone()),
        "remote.repo_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.repo_root.clone()),
        "remote.frontend_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.repo_root.as_ref())
            .and_then(|row| Path::new(row).parent())
            .map(|row| row.display().to_string()),
        "remote.cache_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.cache_root.clone()),
        "remote.corpus_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.corpus_root.clone()),
        "remote.results_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.results_root.clone()),
        "remote.results_legacy_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.results_legacy_root.clone()),
        "remote.extra_data_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.extra_data_root.clone()),
        "remote.containers_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.containers_root.clone()),
        "remote.reference_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.reference_root.clone()),
        "sync.defaults.pull_base" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.pull_base.clone()),
        "sync.defaults.pull_mode" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.pull_mode.clone()),
        "sync.defaults.include_profile" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.include_profile.clone()),
        "sync.defaults.exclude_profile" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.exclude_profile.clone()),
        "sync.defaults.clean_context" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.clean_context.map(|value| value.to_string())),
        "sync.defaults.allow_dirty" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.allow_dirty.map(|value| value.to_string())),
        "sync.defaults.include_containers_manifest" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| {
                row.include_containers_manifest
                    .map(|value| value.to_string())
            }),
        "sync.defaults.data_manifest_glob" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.data_manifest_glob.clone()),
        other => return Err(anyhow!("unsupported benchmark workspace key `{other}`")),
    }
    .ok_or_else(|| anyhow!("missing benchmark workspace value for `{key}`"))
}

pub(crate) fn corpus_01_publication_contract(
    cwd: &Path,
    explicit_path: Option<&Path>,
    stage_id: &str,
) -> Result<CorpusBenchmarkContract> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    publication
        .corpus_01
        .unwrap_or_default()
        .contracts
        .into_iter()
        .find(|row| row.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing corpus-01 publication contract for {stage_id}"))
}

#[cfg(test)]
mod tests {
    use super::{
        benchmark_publication_config_path, benchmark_workspace_config_path,
        benchmark_workspace_value, corpus_01_publication_contract, load_benchmark_workspace_config,
        load_optional_benchmark_workspace_config,
    };
    use std::path::Path;

    fn write_workspace(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        std::fs::write(
            config_dir.join("workspace.toml"),
            r#"[local]
results_root = "/tmp/local-results"
cache_mirror_root = "/tmp/local-results/cache"

[remote]
ssh_host = "cluster"
repo_root = "/srv/repo"
cache_root = "/srv/cache"
corpus_root = "/srv/cache/corpus_01"
results_root = "/srv/cache/results"
containers_root = "/srv/cache/containers"

[sync.defaults]
pull_mode = "results"
"#,
        )
        .expect("write workspace");
    }

    fn write_publication(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        std::fs::write(
            config_dir.join("publication.toml"),
            r#"[[corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqc"]
"#,
        )
        .expect("write publication");
    }

    #[test]
    fn workspace_path_honors_explicit_override() {
        let temp = tempfile::tempdir().expect("tempdir");
        let override_path = temp.path().join("custom.toml");
        std::fs::write(&override_path, "").expect("write override");
        let resolved = benchmark_workspace_config_path(temp.path(), Some(&override_path));
        assert_eq!(resolved, override_path);
    }

    #[test]
    fn optional_workspace_load_returns_none_when_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config = load_optional_benchmark_workspace_config(temp.path(), None)
            .expect("optional workspace load");
        assert!(config.is_none());
    }

    #[test]
    fn workspace_value_reads_governed_contract_keys() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_workspace(temp.path());
        let value =
            benchmark_workspace_value(temp.path(), None, "remote.corpus_root").expect("value");
        assert_eq!(value, "/srv/cache/corpus_01");
    }

    #[test]
    fn workspace_config_load_reads_default_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_workspace(temp.path());
        let config = load_benchmark_workspace_config(temp.path(), None).expect("workspace config");
        assert_eq!(
            config.remote.and_then(|row| row.repo_root),
            Some("/srv/repo".to_string())
        );
    }

    #[test]
    fn publication_contract_loads_stage_contract() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_publication(temp.path());
        let contract = corpus_01_publication_contract(temp.path(), None, "fastq.validate_reads")
            .expect("publication contract");
        assert_eq!(contract.scenario_id, "validation_fairness");
        assert_eq!(contract.tools, vec!["fastqc"]);
    }

    #[test]
    fn publication_path_defaults_under_configs_bench() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = benchmark_publication_config_path(temp.path(), None);
        assert_eq!(path, temp.path().join("configs/bench/publication.toml"));
    }
}
