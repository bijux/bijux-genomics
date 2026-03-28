use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

pub(crate) const BENCHMARK_CONFIG_ENV: &str = "BIJUX_BENCHMARK_CONFIG";
pub(crate) const LEGACY_BENCHMARK_WORKSPACE_CONFIG_ENV: &str = "BIJUX_FASTQ_CORPUS_CONFIG";
pub(crate) const DEFAULT_BENCHMARK_CONFIG: &str = "configs/bench/benchmark.toml";
pub(crate) const DEFAULT_BENCHMARK_WORKSPACE_CONFIG: &str = "configs/bench/workspace.toml";
pub(crate) const DEFAULT_BENCHMARK_PUBLICATION_CONFIG: &str = "configs/bench/publication.toml";

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceConfig {
    pub(crate) local: Option<BenchmarkWorkspaceLocal>,
    pub(crate) remote: Option<BenchmarkWorkspaceRemote>,
    pub(crate) layout: Option<BenchmarkWorkspaceLayout>,
    #[serde(default)]
    pub(crate) artifacts: BTreeMap<String, BenchmarkWorkspaceArtifact>,
    pub(crate) sync: Option<BenchmarkWorkspaceSync>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceLocal {
    pub(crate) results_root: Option<String>,
    pub(crate) cache_mirror_root: Option<String>,
    pub(crate) extra_data_root: Option<String>,
    pub(crate) reference_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceLayout {
    pub(crate) stage_runs: Option<BenchmarkWorkspaceStageRuns>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceStageRuns {
    pub(crate) remote_results_template: Option<String>,
    pub(crate) local_cache_results_template: Option<String>,
    pub(crate) local_archive_results_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceArtifact {
    pub(crate) reference_index_template: Option<String>,
    pub(crate) database_root_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkWorkspaceSync {
    pub(crate) defaults: Option<BenchmarkWorkspaceSyncDefaults>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkPublicationConfig {
    pub(crate) corpus_01: Option<Corpus01PublicationConfig>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkCorpusConfig {
    pub(crate) spec_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
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

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkDepleteRrnaInputConfig {
    pub(crate) rrna_db: Option<String>,
    pub(crate) rrna_bundle_id: Option<String>,
    pub(crate) min_identity: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkReferenceInputConfig {
    pub(crate) reference_index: Option<String>,
    pub(crate) reference_catalog_id: Option<String>,
    pub(crate) reference_index_backend: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct BenchmarkScreenTaxonomyInputConfig {
    pub(crate) database_root: Option<String>,
    pub(crate) database_catalog_id: Option<String>,
    pub(crate) database_artifact_id: Option<String>,
    pub(crate) database_namespace: Option<String>,
    pub(crate) database_scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
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

fn benchmark_config_path_env_override() -> Option<PathBuf> {
    std::env::var_os(BENCHMARK_CONFIG_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn legacy_workspace_config_path_env_override() -> Option<PathBuf> {
    std::env::var_os(LEGACY_BENCHMARK_WORKSPACE_CONFIG_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn resolve_config_path(cwd: &Path, explicit_path: Option<&Path>, default_rel: &str) -> PathBuf {
    if let Some(path) = explicit_path {
        return absolutize(cwd, path);
    }
    if default_rel == DEFAULT_BENCHMARK_CONFIG {
        if let Some(path) = benchmark_config_path_env_override() {
            return absolutize(cwd, &path);
        }
        if let Some(path) = legacy_workspace_config_path_env_override() {
            return absolutize(cwd, &path);
        }
    }
    if default_rel == DEFAULT_BENCHMARK_WORKSPACE_CONFIG {
        if let Some(path) = legacy_workspace_config_path_env_override() {
            return absolutize(cwd, &path);
        }
    }
    cwd.join(default_rel)
}

pub(crate) fn benchmark_config_path(cwd: &Path, explicit_path: Option<&Path>) -> PathBuf {
    resolve_config_path(cwd, explicit_path, DEFAULT_BENCHMARK_CONFIG)
}

pub(crate) fn benchmark_workspace_config_path(cwd: &Path, explicit_path: Option<&Path>) -> PathBuf {
    benchmark_config_path(cwd, explicit_path)
}

pub(crate) fn benchmark_publication_config_path(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> PathBuf {
    benchmark_config_path(cwd, explicit_path)
}

fn load_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<T>(&expand_env_placeholders(&raw))
        .with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn expand_env_placeholders(raw: &str) -> String {
    let mut expanded = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut name = String::new();
            while let Some(next) = chars.next() {
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            expanded.push_str(&std::env::var(&name).unwrap_or_default());
            continue;
        }
        expanded.push(ch);
    }
    expanded
}

fn synthesize_legacy_benchmark_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkConfig> {
    let workspace_path =
        resolve_config_path(cwd, explicit_path, DEFAULT_BENCHMARK_WORKSPACE_CONFIG);
    let publication_path =
        resolve_config_path(cwd, explicit_path, DEFAULT_BENCHMARK_PUBLICATION_CONFIG);
    let workspace = if workspace_path.is_file() {
        load_toml::<BenchmarkWorkspaceConfig>(&workspace_path)?
    } else {
        BenchmarkWorkspaceConfig::default()
    };
    let publication = if publication_path.is_file() {
        load_toml::<BenchmarkPublicationConfig>(&publication_path)?
    } else {
        BenchmarkPublicationConfig::default()
    };
    Ok(BenchmarkConfig {
        workspace,
        publication,
        corpora: BTreeMap::new(),
        stage_inputs: BenchmarkStageInputConfig::default(),
    })
}

pub(crate) fn load_benchmark_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkConfig> {
    let path = benchmark_config_path(cwd, explicit_path);
    if path.is_file() {
        if let Ok(config) = load_toml::<BenchmarkConfig>(&path) {
            return Ok(config);
        }
        if let Ok(workspace) = load_toml::<BenchmarkWorkspaceConfig>(&path) {
            return Ok(BenchmarkConfig {
                workspace,
                publication: synthesize_legacy_benchmark_config(cwd, explicit_path)?.publication,
                corpora: BTreeMap::new(),
                stage_inputs: BenchmarkStageInputConfig::default(),
            });
        }
        if let Ok(publication) = load_toml::<BenchmarkPublicationConfig>(&path) {
            return Ok(BenchmarkConfig {
                workspace: synthesize_legacy_benchmark_config(cwd, explicit_path)?.workspace,
                publication,
                corpora: BTreeMap::new(),
                stage_inputs: BenchmarkStageInputConfig::default(),
            });
        }
    }
    synthesize_legacy_benchmark_config(cwd, explicit_path)
}

pub(crate) fn load_optional_benchmark_workspace_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<Option<BenchmarkWorkspaceConfig>> {
    let config = load_benchmark_config(cwd, explicit_path)?;
    if config.workspace == BenchmarkWorkspaceConfig::default() {
        return Ok(None);
    }
    Ok(Some(config.workspace))
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
    let config = load_benchmark_config(cwd, explicit_path)?;
    if config.publication == BenchmarkPublicationConfig::default() {
        return Err(anyhow!(
            "missing benchmark publication config: {}",
            path.display()
        ));
    }
    Ok(config.publication)
}

pub(crate) fn benchmark_corpus_spec_path(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
) -> Result<PathBuf> {
    let config = load_benchmark_config(cwd, explicit_path)?;
    if let Some(path) = config
        .corpora
        .get(corpus_id)
        .and_then(|row| row.spec_path.as_deref())
    {
        return Ok(absolutize(cwd, Path::new(path)));
    }
    Ok(cwd
        .join("configs")
        .join("runtime")
        .join("corpora")
        .join(format!("{corpus_id}.toml")))
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
        benchmark_config_path, benchmark_corpus_spec_path, benchmark_publication_config_path,
        benchmark_workspace_config_path, benchmark_workspace_value, corpus_01_publication_contract,
        load_benchmark_config, load_benchmark_workspace_config,
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

    fn write_unified_config(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/tmp/local-results"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/srv/repo"
corpus_root = "/srv/cache/corpus_01"
results_root = "/srv/cache/results"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]

[corpora.corpus-01]
spec_path = "configs/runtime/corpora/corpus-01.toml"
"#,
        )
        .expect("write unified config");
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
        assert_eq!(path, temp.path().join("configs/bench/benchmark.toml"));
    }

    #[test]
    fn unified_benchmark_config_is_preferred_when_present() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_workspace(temp.path());
        write_publication(temp.path());
        write_unified_config(temp.path());

        let config = load_benchmark_config(temp.path(), None).expect("benchmark config");
        assert_eq!(
            config.workspace.remote.and_then(|row| row.corpus_root),
            Some("/srv/cache/corpus_01".to_string())
        );
        assert_eq!(
            config
                .publication
                .corpus_01
                .expect("corpus publication")
                .contracts
                .len(),
            1
        );
        assert_eq!(
            benchmark_config_path(temp.path(), None),
            temp.path().join("configs/bench/benchmark.toml")
        );
    }

    #[test]
    fn corpus_spec_path_comes_from_benchmark_config() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_unified_config(temp.path());
        let path = benchmark_corpus_spec_path(temp.path(), None, "corpus-01")
            .expect("configured corpus spec path");
        assert_eq!(
            path,
            temp.path().join("configs/runtime/corpora/corpus-01.toml")
        );
    }

    #[test]
    fn benchmark_config_expands_environment_placeholders() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create config dir");
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "${BIJUX_TEST_RESULTS_ROOT}"
"#,
        )
        .expect("write config");
        std::env::set_var("BIJUX_TEST_RESULTS_ROOT", "/tmp/env-results");
        let config = load_benchmark_config(temp.path(), None).expect("load benchmark config");
        std::env::remove_var("BIJUX_TEST_RESULTS_ROOT");
        assert_eq!(
            config.workspace.local.and_then(|row| row.results_root),
            Some("/tmp/env-results".to_string())
        );
    }
}
