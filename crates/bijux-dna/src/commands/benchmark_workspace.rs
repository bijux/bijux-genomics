#![allow(clippy::struct_field_names, clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub(crate) const BENCHMARK_CONFIG_ENV: &str = "BIJUX_BENCHMARK_CONFIG";
pub(crate) const BENCHMARK_CONFIG_JSON_ENV: &str = "BIJUX_BENCHMARK_CONFIG_JSON";
pub(crate) const DEFAULT_BENCHMARK_CONFIG: &str = "configs/bench/benchmark.toml";

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
pub(crate) struct BenchmarkReferenceInputConfig {
    pub(crate) reference_index: Option<String>,
    pub(crate) reference_catalog_id: Option<String>,
    pub(crate) reference_index_backend: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
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

pub(crate) fn benchmark_publication_corpus_key(corpus_id: &str) -> String {
    corpus_id.replace('-', "_")
}

pub(crate) fn benchmark_publication_corpus_id(publication_key: &str) -> String {
    publication_key.replace('_', "-")
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct CorpusBenchmarkExclusion {
    pub(crate) stage_id: String,
    pub(crate) reason: String,
}

fn benchmark_config_path_env_override() -> Option<PathBuf> {
    std::env::var_os(BENCHMARK_CONFIG_ENV)
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
    toml::from_str::<T>(&expand_env_placeholders(&raw)?)
        .with_context(|| format!("parse {}", path.display()))
}

fn normalize_optional_string(value: &mut Option<String>) {
    let Some(raw) = value.take() else {
        return;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    *value = Some(trimmed.to_string());
}

fn normalize_benchmark_config(mut config: BenchmarkConfig) -> BenchmarkConfig {
    if let Some(local) = config.workspace.local.as_mut() {
        normalize_optional_string(&mut local.results_root);
        normalize_optional_string(&mut local.cache_mirror_root);
        normalize_optional_string(&mut local.extra_data_root);
        normalize_optional_string(&mut local.reference_root);
    }
    if let Some(remote) = config.workspace.remote.as_mut() {
        normalize_optional_string(&mut remote.ssh_host);
        normalize_optional_string(&mut remote.repo_root);
        normalize_optional_string(&mut remote.cache_root);
        normalize_optional_string(&mut remote.corpus_root);
        normalize_optional_string(&mut remote.results_root);
        normalize_optional_string(&mut remote.extra_data_root);
        normalize_optional_string(&mut remote.containers_root);
        normalize_optional_string(&mut remote.reference_root);
    }
    if let Some(layout) = config.workspace.layout.as_mut() {
        if let Some(stage_runs) = layout.stage_runs.as_mut() {
            normalize_optional_string(&mut stage_runs.remote_results_template);
            normalize_optional_string(&mut stage_runs.local_cache_results_template);
            normalize_optional_string(&mut stage_runs.local_archive_results_template);
        }
    }
    for artifact in config.workspace.artifacts.values_mut() {
        normalize_optional_string(&mut artifact.reference_index_template);
        normalize_optional_string(&mut artifact.database_root_template);
    }
    if let Some(sync) = config.workspace.sync.as_mut() {
        if let Some(defaults) = sync.defaults.as_mut() {
            normalize_optional_string(&mut defaults.pull_base);
            normalize_optional_string(&mut defaults.pull_mode);
            normalize_optional_string(&mut defaults.include_profile);
            normalize_optional_string(&mut defaults.exclude_profile);
        }
    }
    for corpus in config.corpora.values_mut() {
        normalize_optional_string(&mut corpus.spec_path);
    }
    normalize_optional_string(&mut config.stage_inputs.fastq_deplete_rrna.rrna_db);
    normalize_optional_string(&mut config.stage_inputs.fastq_deplete_rrna.rrna_bundle_id);
    normalize_optional_string(&mut config.stage_inputs.fastq_deplete_rrna.min_identity);
    normalize_optional_string(&mut config.stage_inputs.fastq_deplete_host.reference_index);
    normalize_optional_string(&mut config.stage_inputs.fastq_deplete_host.reference_catalog_id);
    normalize_optional_string(
        &mut config
            .stage_inputs
            .fastq_deplete_host
            .reference_index_backend,
    );
    normalize_optional_string(
        &mut config
            .stage_inputs
            .fastq_deplete_reference_contaminants
            .reference_index,
    );
    normalize_optional_string(
        &mut config
            .stage_inputs
            .fastq_deplete_reference_contaminants
            .reference_catalog_id,
    );
    normalize_optional_string(
        &mut config
            .stage_inputs
            .fastq_deplete_reference_contaminants
            .reference_index_backend,
    );
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_root);
    normalize_optional_string(
        &mut config
            .stage_inputs
            .fastq_screen_taxonomy
            .database_catalog_id,
    );
    normalize_optional_string(
        &mut config
            .stage_inputs
            .fastq_screen_taxonomy
            .database_artifact_id,
    );
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_namespace);
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_scope);
    config
}

pub(crate) fn expand_env_placeholders(raw: &str) -> Result<String> {
    let mut expanded = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut name = String::new();
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            let value = std::env::var(&name)
                .with_context(|| format!("missing environment placeholder ${{{name}}}"))?;
            expanded.push_str(&value);
            continue;
        }
        expanded.push(ch);
    }
    Ok(expanded)
}

pub(crate) fn load_benchmark_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkConfig> {
    let path = benchmark_config_path(cwd, explicit_path);
    if !path.is_file() {
        return Err(anyhow!("missing benchmark config: {}", path.display()));
    }
    Ok(normalize_benchmark_config(load_toml::<BenchmarkConfig>(
        &path,
    )?))
}

pub(crate) fn load_optional_benchmark_workspace_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<Option<BenchmarkWorkspaceConfig>> {
    let path = benchmark_config_path(cwd, explicit_path);
    if !path.is_file() {
        return Ok(None);
    }
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
    Err(anyhow!(
        "benchmark config is missing corpora.{corpus_id}.spec_path"
    ))
}

pub(crate) fn benchmark_runtime_corpus_dir_name(
    workspace: &BenchmarkWorkspaceConfig,
    _corpus_id: &str,
) -> Result<String> {
    if let Some(dir_name) = workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(dir_name.to_string());
    }
    Err(anyhow!(
        "benchmark config is missing workspace.remote.corpus_root"
    ))
}

pub(crate) fn benchmark_stage_run_relative_root(
    workspace: &BenchmarkWorkspaceConfig,
    scope: &str,
    corpus_dir_name: &str,
    stage_id: &str,
) -> Result<PathBuf> {
    let template = workspace
        .layout
        .as_ref()
        .and_then(|row| row.stage_runs.as_ref())
        .and_then(|row| match scope {
            "remote" => row.remote_results_template.as_deref(),
            "local-cache" => row.local_cache_results_template.as_deref(),
            "local-archive" => row.local_archive_results_template.as_deref(),
            _ => None,
        })
        .ok_or_else(|| anyhow!("benchmark config is missing workspace.layout.stage_runs template for scope `{scope}`"))?;
    Ok(PathBuf::from(
        template
            .replace("{corpus_id}", corpus_dir_name)
            .replace("{stage_id}", stage_id),
    ))
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

pub(crate) fn print_benchmark_config_json(
    cwd: &Path,
    args: &crate::commands::cli::BenchConfigJsonArgs,
) -> Result<()> {
    let config = load_benchmark_config(cwd, args.config.as_deref())?;
    match args.section.as_str() {
        "full" => println!("{}", serde_json::to_string_pretty(&config)?),
        "workspace" => println!("{}", serde_json::to_string_pretty(&config.workspace)?),
        "publication" => println!("{}", serde_json::to_string_pretty(&config.publication)?),
        "corpora" => println!("{}", serde_json::to_string_pretty(&config.corpora)?),
        "stage_inputs" => println!("{}", serde_json::to_string_pretty(&config.stage_inputs)?),
        other => {
            return Err(anyhow!(
                "unsupported benchmark config section `{other}`; expected one of: full, workspace, publication, corpora, stage_inputs"
            ))
        }
    }
    Ok(())
}

pub(crate) fn run_normalize_workspace_layout(
    cwd: &Path,
    args: &crate::commands::cli::BenchNormalizeWorkspaceLayoutArgs,
) -> Result<()> {
    let workspace = load_benchmark_workspace_config(cwd, args.config.as_deref())?;
    let corpus_dir_name = benchmark_runtime_corpus_dir_name(&workspace, &args.corpus_id)?;
    let report = normalize_workspace_layout_report(
        &workspace,
        &args.corpus_id,
        &corpus_dir_name,
        args.confirm,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceEntrySummary {
    exists: bool,
    kind: String,
    file_count: u64,
    total_size_bytes: u64,
    mtime: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceConvergenceAction {
    action: String,
    entry_name: String,
    source: String,
    target: String,
    legacy_summary: WorkspaceEntrySummary,
    canonical_summary: WorkspaceEntrySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceConvergencePlan {
    canonical_root: String,
    legacy_root: String,
    actions: Vec<WorkspaceConvergenceAction>,
    removable_legacy_root: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    legacy_root_removed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceStageNormalizationReport {
    stage_id: String,
    action: String,
    source: String,
    target: String,
    legacy_summary: WorkspaceEntrySummary,
    canonical_summary: WorkspaceEntrySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceNormalizationReport {
    corpus_id: String,
    corpus_dir_name: String,
    canonical_corpus_root: String,
    legacy_corpus_root: String,
    archive_stage_ids: Vec<String>,
    cache_stage_ids: Vec<String>,
    shared_stage_ids: Vec<String>,
    archive_only_stage_ids: Vec<String>,
    cache_only_stage_ids: Vec<String>,
    mode: String,
    status: String,
    stage_reports: Vec<WorkspaceStageNormalizationReport>,
    moved_stage_ids: Vec<String>,
    removed_duplicate_stage_ids: Vec<String>,
    manual_review_stage_ids: Vec<String>,
}

fn normalize_workspace_layout_report(
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    corpus_dir_name: &str,
    confirm: bool,
) -> Result<WorkspaceNormalizationReport> {
    let local_results_root = workspace_local_results_root(workspace)?;
    let local_cache_mirror_root = workspace_local_cache_mirror_root(workspace)?;
    let legacy_corpus_root = local_results_root.join(corpus_dir_name);
    let canonical_corpus_root = local_cache_mirror_root
        .join("results")
        .join(corpus_dir_name);

    let archive_stage_ids = stage_directory_names(&legacy_corpus_root);
    let cache_stage_ids = stage_directory_names(&canonical_corpus_root);
    let archive_set = archive_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let cache_set = cache_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let shared_stage_ids = archive_set
        .intersection(&cache_set)
        .cloned()
        .collect::<Vec<_>>();
    let archive_only_stage_ids = archive_set
        .difference(&cache_set)
        .cloned()
        .collect::<Vec<_>>();
    let cache_only_stage_ids = cache_set
        .difference(&archive_set)
        .cloned()
        .collect::<Vec<_>>();

    let plan = plan_root_convergence(&canonical_corpus_root, &legacy_corpus_root)?;
    let convergence_report = if confirm {
        apply_root_convergence(&plan)?
    } else {
        plan
    };

    if confirm && legacy_corpus_root.exists() {
        remove_empty_parents(&legacy_corpus_root, &local_results_root)?;
    }

    let stage_reports = convergence_report
        .actions
        .iter()
        .map(|action| WorkspaceStageNormalizationReport {
            stage_id: action.entry_name.clone(),
            action: action.action.clone(),
            source: action.source.clone(),
            target: action.target.clone(),
            legacy_summary: action.legacy_summary.clone(),
            canonical_summary: action.canonical_summary.clone(),
            status: action.status.clone(),
        })
        .collect::<Vec<_>>();
    let moved_stage_ids = stage_reports
        .iter()
        .filter(|report| report.action == "move-legacy-entry")
        .map(|report| report.stage_id.clone())
        .collect::<Vec<_>>();
    let removed_duplicate_stage_ids = stage_reports
        .iter()
        .filter(|report| report.action == "remove-legacy-duplicate")
        .map(|report| report.stage_id.clone())
        .collect::<Vec<_>>();
    let manual_review_stage_ids = stage_reports
        .iter()
        .filter(|report| report.action == "manual-review-required")
        .map(|report| report.stage_id.clone())
        .collect::<Vec<_>>();

    Ok(WorkspaceNormalizationReport {
        corpus_id: corpus_id.to_string(),
        corpus_dir_name: corpus_dir_name.to_string(),
        canonical_corpus_root: canonical_corpus_root.display().to_string(),
        legacy_corpus_root: legacy_corpus_root.display().to_string(),
        archive_stage_ids,
        cache_stage_ids,
        shared_stage_ids,
        archive_only_stage_ids,
        cache_only_stage_ids,
        mode: if confirm { "confirm" } else { "dry-run" }.to_string(),
        status: if manual_review_stage_ids.is_empty() {
            "clear"
        } else {
            "needs-review"
        }
        .to_string(),
        stage_reports,
        moved_stage_ids,
        removed_duplicate_stage_ids,
        manual_review_stage_ids,
    })
}

fn workspace_local_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))
}

fn workspace_local_cache_mirror_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))
}

fn stage_directory_names(root: &Path) -> Vec<String> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut entries = root
        .read_dir()
        .ok()
        .into_iter()
        .flat_map(|iter| iter.filter_map(Result::ok))
        .filter_map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => Some(entry.file_name().to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn remove_empty_parents(root: &Path, stop_at: &Path) -> Result<()> {
    let mut current = root.to_path_buf();
    while current != stop_at && current.exists() {
        match fs::remove_dir(&current) {
            Ok(()) => {
                current = current
                    .parent()
                    .ok_or_else(|| anyhow!("missing parent for {}", root.display()))?
                    .to_path_buf();
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => break,
            Err(err) => {
                if current.exists()
                    && fs::read_dir(&current)
                        .map(|mut entries| entries.next().is_some())
                        .unwrap_or(false)
                {
                    break;
                }
                return Err(err).with_context(|| format!("remove dir {}", current.display()));
            }
        }
    }
    Ok(())
}

fn plan_root_convergence(
    canonical_root: &Path,
    legacy_root: &Path,
) -> Result<WorkspaceConvergencePlan> {
    let mut actions = Vec::new();
    if legacy_root.exists() {
        let mut legacy_entries = legacy_root
            .read_dir()
            .with_context(|| format!("read {}", legacy_root.display()))?
            .filter_map(|entry| entry.ok().map(|item| item.path()))
            .collect::<Vec<_>>();
        legacy_entries.sort();

        for legacy_entry in legacy_entries {
            let entry_name = legacy_entry
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| anyhow!("invalid legacy entry {}", legacy_entry.display()))?
                .to_string();
            let canonical_entry = canonical_root.join(&entry_name);
            let legacy_summary = workspace_entry_summary(&legacy_entry)?;
            let canonical_summary = workspace_entry_summary(&canonical_entry)?;
            let action = if !canonical_entry.exists() {
                "move-legacy-entry"
            } else if canonical_summary.file_count >= legacy_summary.file_count
                && canonical_summary.total_size_bytes >= legacy_summary.total_size_bytes
                && canonical_summary.mtime.unwrap_or(0) >= legacy_summary.mtime.unwrap_or(0)
            {
                "remove-legacy-duplicate"
            } else {
                "manual-review-required"
            };
            actions.push(WorkspaceConvergenceAction {
                action: action.to_string(),
                entry_name,
                source: legacy_entry.display().to_string(),
                target: canonical_entry.display().to_string(),
                legacy_summary,
                canonical_summary,
                status: None,
            });
        }
    }

    Ok(WorkspaceConvergencePlan {
        canonical_root: canonical_root.display().to_string(),
        legacy_root: legacy_root.display().to_string(),
        removable_legacy_root: actions
            .iter()
            .all(|action| action.action != "manual-review-required"),
        actions,
        legacy_root_removed: None,
    })
}

fn workspace_entry_summary(path: &Path) -> Result<WorkspaceEntrySummary> {
    if !path.exists() {
        return Ok(WorkspaceEntrySummary {
            exists: false,
            kind: "missing".to_string(),
            file_count: 0,
            total_size_bytes: 0,
            mtime: None,
        });
    }

    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if metadata.is_file() {
        return Ok(WorkspaceEntrySummary {
            exists: true,
            kind: "file".to_string(),
            file_count: 1,
            total_size_bytes: metadata.len(),
            mtime: modified_secs(&metadata),
        });
    }

    let mut file_count = 0u64;
    let mut total_size_bytes = 0u64;
    let mut newest_mtime = modified_secs(&metadata).unwrap_or(0);
    for child in walkdir::WalkDir::new(path) {
        let child = child.with_context(|| format!("walk {}", path.display()))?;
        let child_metadata = child
            .metadata()
            .with_context(|| format!("stat {}", child.path().display()))?;
        if let Some(mtime) = modified_secs(&child_metadata) {
            newest_mtime = newest_mtime.max(mtime);
        }
        if child.file_type().is_file() {
            file_count += 1;
            total_size_bytes += child_metadata.len();
        }
    }

    Ok(WorkspaceEntrySummary {
        exists: true,
        kind: "directory".to_string(),
        file_count,
        total_size_bytes,
        mtime: Some(newest_mtime),
    })
}

fn modified_secs(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn apply_root_convergence(plan: &WorkspaceConvergencePlan) -> Result<WorkspaceConvergencePlan> {
    let mut applied_actions = Vec::new();
    for action in &plan.actions {
        let source = PathBuf::from(&action.source);
        let target = PathBuf::from(&action.target);
        let status = match action.action.as_str() {
            "move-legacy-entry" => {
                move_workspace_path(&source, &target)?;
                "applied"
            }
            "remove-legacy-duplicate" => {
                remove_workspace_path(&source)?;
                "applied"
            }
            "manual-review-required" => "pending-manual-review",
            other => return Err(anyhow!("unsupported convergence action `{other}`")),
        };
        applied_actions.push(WorkspaceConvergenceAction {
            status: Some(status.to_string()),
            ..action.clone()
        });
    }

    let legacy_root = PathBuf::from(&plan.legacy_root);
    let legacy_root_removed = if plan.removable_legacy_root && legacy_root.is_dir() {
        let mut entries = legacy_root
            .read_dir()
            .with_context(|| format!("read {}", legacy_root.display()))?;
        if entries.next().is_none() {
            fs::remove_dir(&legacy_root)
                .with_context(|| format!("remove dir {}", legacy_root.display()))?;
            true
        } else {
            false
        }
    } else {
        false
    };

    Ok(WorkspaceConvergencePlan {
        actions: applied_actions,
        legacy_root_removed: Some(legacy_root_removed),
        ..plan.clone()
    })
}

fn move_workspace_path(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::rename(source, target)
        .with_context(|| format!("move {} -> {}", source.display(), target.display()))
}

fn remove_workspace_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("remove dir {}", path.display()))
    } else {
        fs::remove_file(path).with_context(|| format!("remove file {}", path.display()))
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkspaceLayoutReport {
    local_results_root: String,
    local_cache_mirror_root: String,
    remote_workspace_root: String,
    authoritative_roots: WorkspaceAuthoritativeRoots,
    root_pairs: Vec<WorkspaceRootPairSummary>,
    unexpected_remote_siblings: Vec<String>,
    local_stage_layout: WorkspaceLocalStageLayout,
    status: String,
    issue_count: usize,
    issues: Vec<WorkspaceLayoutIssue>,
}

#[derive(Debug, Serialize)]
struct WorkspaceAuthoritativeRoots {
    remote_results_root: String,
    remote_reference_root: String,
    local_stage_root: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceRootPairSummary {
    scope: String,
    canonical_root: String,
    legacy_root: String,
    canonical_exists: bool,
    legacy_exists: bool,
    canonical_entries: Vec<String>,
    legacy_entries: Vec<String>,
    shared_entries: Vec<String>,
    canonical_only_entries: Vec<String>,
    legacy_only_entries: Vec<String>,
    status: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceLocalStageLayout {
    archive_corpus_root: String,
    cache_corpus_root: String,
    archive_stage_ids: Vec<String>,
    cache_stage_ids: Vec<String>,
    shared_stage_ids: Vec<String>,
    archive_only_stage_ids: Vec<String>,
    cache_only_stage_ids: Vec<String>,
    authoritative_stage_root: String,
}

#[derive(Debug, Serialize)]
struct WorkspaceLayoutIssue {
    issue_id: String,
    detail: String,
}

pub(crate) fn write_workspace_layout_status(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    let report = workspace_layout_report(&config)?;
    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join("workspace-layout-status.json");
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join("workspace-layout-status.md");
    fs::write(&markdown_path, render_workspace_layout_markdown(&report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn workspace_layout_report(config: &BenchmarkConfig) -> Result<WorkspaceLayoutReport> {
    let workspace = &config.workspace;
    let local_results_root = workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))?;
    let local_cache_mirror_root = workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))?;
    let remote_workspace_root = local_cache_mirror_root
        .parent()
        .ok_or_else(|| anyhow!("invalid local.cache_mirror_root"))?
        .to_path_buf();
    let corpus_dir_name = workspace_layout_corpus_dir_name(config)?;

    let root_pairs = vec![
        summarize_root_pair(
            "remote-results",
            &local_cache_mirror_root.join("results"),
            &local_cache_mirror_root.join("bijux-dna-results"),
        ),
        summarize_root_pair(
            "remote-reference",
            &local_cache_mirror_root.join("reference"),
            &local_cache_mirror_root.join("bijux-reference"),
        ),
    ];

    let mut issues = Vec::new();
    for pair in &root_pairs {
        if pair.status == "duplicate" {
            issues.push(WorkspaceLayoutIssue {
                issue_id: format!("duplicate-{}-root", pair.scope),
                detail: format!(
                    "both {} and {} exist",
                    pair.canonical_root, pair.legacy_root
                ),
            });
        }
    }

    let mut unexpected_remote_siblings = Vec::new();
    for sibling in ["results", corpus_dir_name.as_str(), "extra-data"] {
        let sibling_root = remote_workspace_root.join(sibling);
        if sibling_root.exists() {
            let detail = sibling_root.display().to_string();
            unexpected_remote_siblings.push(detail.clone());
            issues.push(WorkspaceLayoutIssue {
                issue_id: "unexpected-remote-workspace-sibling".to_string(),
                detail: format!("unexpected sibling beside .cache: {detail}"),
            });
        }
    }

    let local_stage_layout = summarize_local_stage_layout(
        &local_results_root,
        &local_cache_mirror_root,
        &corpus_dir_name,
    );
    for stage_id in &local_stage_layout.shared_stage_ids {
        issues.push(WorkspaceLayoutIssue {
            issue_id: "duplicate-local-stage-root".to_string(),
            detail: format!(
                "both {}/{} and {}/{} exist",
                local_stage_layout.cache_corpus_root,
                stage_id,
                local_stage_layout.archive_corpus_root,
                stage_id,
            ),
        });
    }
    for stage_id in &local_stage_layout.archive_only_stage_ids {
        issues.push(WorkspaceLayoutIssue {
            issue_id: "archive-only-local-stage-root".to_string(),
            detail: format!(
                "{}/{} exists outside the governed cache mirror root {}",
                local_stage_layout.archive_corpus_root,
                stage_id,
                local_stage_layout.cache_corpus_root,
            ),
        });
    }

    Ok(WorkspaceLayoutReport {
        local_results_root: local_results_root.display().to_string(),
        local_cache_mirror_root: local_cache_mirror_root.display().to_string(),
        remote_workspace_root: remote_workspace_root.display().to_string(),
        authoritative_roots: WorkspaceAuthoritativeRoots {
            remote_results_root: local_cache_mirror_root
                .join("results")
                .display()
                .to_string(),
            remote_reference_root: local_cache_mirror_root
                .join("reference")
                .display()
                .to_string(),
            local_stage_root: local_stage_layout.authoritative_stage_root.clone(),
        },
        root_pairs,
        unexpected_remote_siblings,
        local_stage_layout,
        status: if issues.is_empty() {
            "clear".to_string()
        } else {
            "incomplete".to_string()
        },
        issue_count: issues.len(),
        issues,
    })
}

fn summarize_root_pair(
    scope: &str,
    canonical_root: &Path,
    legacy_root: &Path,
) -> WorkspaceRootPairSummary {
    let canonical_entries = entry_names(canonical_root);
    let legacy_entries = entry_names(legacy_root);
    let canonical_set = canonical_entries.iter().cloned().collect::<BTreeSet<_>>();
    let legacy_set = legacy_entries.iter().cloned().collect::<BTreeSet<_>>();
    WorkspaceRootPairSummary {
        scope: scope.to_string(),
        canonical_root: canonical_root.display().to_string(),
        legacy_root: legacy_root.display().to_string(),
        canonical_exists: canonical_root.exists(),
        legacy_exists: legacy_root.exists(),
        canonical_entries: canonical_entries.clone(),
        legacy_entries: legacy_entries.clone(),
        shared_entries: canonical_set.intersection(&legacy_set).cloned().collect(),
        canonical_only_entries: canonical_set.difference(&legacy_set).cloned().collect(),
        legacy_only_entries: legacy_set.difference(&canonical_set).cloned().collect(),
        status: if canonical_root.exists() && legacy_root.exists() {
            "duplicate".to_string()
        } else {
            "clear".to_string()
        },
    }
}

fn summarize_local_stage_layout(
    local_results_root: &Path,
    local_cache_mirror_root: &Path,
    corpus_dir_name: &str,
) -> WorkspaceLocalStageLayout {
    let archive_corpus_root = local_results_root.join(corpus_dir_name);
    let cache_corpus_root = local_cache_mirror_root
        .join("results")
        .join(corpus_dir_name);
    let archive_stage_ids = entry_names(&archive_corpus_root);
    let cache_stage_ids = entry_names(&cache_corpus_root);
    let archive_set = archive_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let cache_set = cache_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    WorkspaceLocalStageLayout {
        archive_corpus_root: archive_corpus_root.display().to_string(),
        cache_corpus_root: cache_corpus_root.display().to_string(),
        archive_stage_ids: archive_stage_ids.clone(),
        cache_stage_ids: cache_stage_ids.clone(),
        shared_stage_ids: archive_set.intersection(&cache_set).cloned().collect(),
        archive_only_stage_ids: archive_set.difference(&cache_set).cloned().collect(),
        cache_only_stage_ids: cache_set.difference(&archive_set).cloned().collect(),
        authoritative_stage_root: cache_corpus_root.display().to_string(),
    }
}

fn entry_names(root: &Path) -> Vec<String> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut entries = root
        .read_dir()
        .ok()
        .into_iter()
        .flat_map(|iter| iter.filter_map(Result::ok))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn render_workspace_layout_markdown(report: &WorkspaceLayoutReport) -> String {
    let mut lines = vec![
        "# Benchmark Workspace Layout Status".to_string(),
        String::new(),
        format!("- Local results root: `{}`", report.local_results_root),
        format!(
            "- Local cache mirror root: `{}`",
            report.local_cache_mirror_root
        ),
        format!(
            "- Mirrored remote workspace root: `{}`",
            report.remote_workspace_root
        ),
        format!(
            "- Authoritative remote results root: `{}`",
            report.authoritative_roots.remote_results_root
        ),
        format!(
            "- Authoritative remote reference root: `{}`",
            report.authoritative_roots.remote_reference_root
        ),
        format!(
            "- Authoritative local publication root: `{}`",
            report.authoritative_roots.local_stage_root
        ),
        format!("- Status: `{}`", report.status),
        format!("- Issues: `{}`", report.issue_count),
        String::new(),
        "## Root Pairs".to_string(),
        String::new(),
    ];
    for pair in &report.root_pairs {
        lines.push(format!(
            "- `{}`: `{}` (canonical `{}`, legacy `{}`)",
            pair.scope, pair.status, pair.canonical_root, pair.legacy_root
        ));
        if !pair.canonical_entries.is_empty() {
            lines.push(format!(
                "  - canonical entries: {}",
                pair.canonical_entries
                    .iter()
                    .map(|entry| format!("`{entry}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !pair.legacy_entries.is_empty() {
            lines.push(format!(
                "  - legacy entries: {}",
                pair.legacy_entries
                    .iter()
                    .map(|entry| format!("`{entry}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !pair.shared_entries.is_empty() {
            lines.push(format!(
                "  - shared entries: {}",
                pair.shared_entries
                    .iter()
                    .map(|entry| format!("`{entry}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Local Stage Layout".to_string(),
        String::new(),
        format!(
            "- Archive corpus root: `{}`",
            report.local_stage_layout.archive_corpus_root
        ),
        format!(
            "- Cache corpus root: `{}`",
            report.local_stage_layout.cache_corpus_root
        ),
    ]);
    if !report.local_stage_layout.shared_stage_ids.is_empty() {
        lines.push(format!(
            "- Shared stage ids: {}",
            report
                .local_stage_layout
                .shared_stage_ids
                .iter()
                .map(|entry| format!("`{entry}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !report.local_stage_layout.archive_only_stage_ids.is_empty() {
        lines.push(format!(
            "- Archive-only stage ids: {}",
            report
                .local_stage_layout
                .archive_only_stage_ids
                .iter()
                .map(|entry| format!("`{entry}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    if !report.local_stage_layout.cache_only_stage_ids.is_empty() {
        lines.push(format!(
            "- Cache-only stage ids: {}",
            report
                .local_stage_layout
                .cache_only_stage_ids
                .iter()
                .map(|entry| format!("`{entry}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.join("\n") + "\n"
}

fn workspace_layout_corpus_dir_name(config: &BenchmarkConfig) -> Result<String> {
    if let Some(dir_name) = config
        .workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
    {
        return Ok(dir_name.to_string());
    }
    if let Some(corpus_id) = config.corpora.keys().next() {
        return Ok(corpus_id.replace('-', "_"));
    }
    Err(anyhow!(
        "benchmark config must declare workspace.remote.corpus_root or at least one [corpora] entry"
    ))
}

pub(crate) fn benchmark_publication_contract(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
    stage_id: &str,
) -> Result<CorpusBenchmarkContract> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_contracts_from_config(&publication, corpus_id)?
        .into_iter()
        .find(|row| row.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing {corpus_id} publication contract for {stage_id}"))
}

pub(crate) fn benchmark_publication_contracts(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkContract>> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_contracts_from_config(&publication, corpus_id)
}

pub(crate) fn benchmark_publication_exclusions(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkExclusion>> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_exclusions_from_config(&publication, corpus_id)
}

fn benchmark_publication_contracts_from_config(
    publication: &BenchmarkPublicationConfig,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkContract>> {
    let key = benchmark_publication_corpus_key(corpus_id);
    publication
        .corpora
        .get(&key)
        .cloned()
        .map(|entry| entry.contracts)
        .ok_or_else(|| anyhow!("benchmark publication config is missing [{key}]"))
}

fn benchmark_publication_exclusions_from_config(
    publication: &BenchmarkPublicationConfig,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkExclusion>> {
    let key = benchmark_publication_corpus_key(corpus_id);
    publication
        .corpora
        .get(&key)
        .cloned()
        .map(|entry| entry.exclusions)
        .ok_or_else(|| anyhow!("benchmark publication config is missing [{key}]"))
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        benchmark_config_path, benchmark_corpus_spec_path, benchmark_publication_config_path,
        benchmark_publication_corpus_key, benchmark_runtime_corpus_dir_name,
        benchmark_stage_run_relative_root, benchmark_workspace_config_path,
        benchmark_workspace_value, expand_env_placeholders, load_benchmark_config,
        load_benchmark_publication_config, load_benchmark_workspace_config,
        load_optional_benchmark_workspace_config, normalize_workspace_layout_report,
        plan_root_convergence, summarize_root_pair, workspace_layout_corpus_dir_name,
        BenchmarkConfig, BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout,
        BenchmarkWorkspaceLocal, BenchmarkWorkspaceStageRuns, BENCHMARK_CONFIG_ENV,
    };
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_workspace(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"
cache_mirror_root = "/bench/local/cache-mirror"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
cache_root = "/bench/remote/cache"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"
containers_root = "/bench/remote/cache/containers"

[workspace.sync.defaults]
pull_mode = "results"
"#,
        )
        .expect("write workspace");
    }

    fn write_publication(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[[publication.corpus_01.contracts]]
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
results_root = "/bench/local/results"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]

[[publication.corpus_01.exclusions]]
stage_id = "fastq.index_reference"
reason = "reference indexing does not benchmark corpus execution"

[corpora.corpus-01]
spec_path = "configs/runtime/corpora/corpus-01.toml"
"#,
        )
        .expect("write unified config");
    }

    fn sample_workspace(results_root: &Path, cache_mirror_root: &Path) -> BenchmarkWorkspaceConfig {
        BenchmarkWorkspaceConfig {
            local: Some(BenchmarkWorkspaceLocal {
                results_root: Some(results_root.display().to_string()),
                cache_mirror_root: Some(cache_mirror_root.display().to_string()),
                extra_data_root: None,
                reference_root: None,
            }),
            remote: None,
            layout: None,
            artifacts: BTreeMap::default(),
            sync: None,
        }
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
    fn benchmark_config_path_honors_benchmark_config_env_override() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let override_path = temp.path().join("custom-benchmark.toml");
        std::fs::write(&override_path, "").expect("write override");

        std::env::set_var(BENCHMARK_CONFIG_ENV, &override_path);
        let benchmark_path = benchmark_config_path(temp.path(), None);
        let workspace_path = benchmark_workspace_config_path(temp.path(), None);
        let publication_path = benchmark_publication_config_path(temp.path(), None);
        std::env::remove_var(BENCHMARK_CONFIG_ENV);

        assert_eq!(benchmark_path, override_path);
        assert_eq!(workspace_path, override_path);
        assert_eq!(publication_path, override_path);
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
        assert_eq!(value, "/bench/remote/cache/benchmark_corpus");
    }

    #[test]
    fn workspace_config_load_reads_default_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_workspace(temp.path());
        let config = load_benchmark_workspace_config(temp.path(), None).expect("workspace config");
        assert_eq!(
            config.remote.and_then(|row| row.repo_root),
            Some("/bench/remote/repo".to_string())
        );
    }

    #[test]
    fn publication_contract_loads_stage_contract() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_publication(temp.path());
        let contract = super::benchmark_publication_contract(
            temp.path(),
            None,
            "corpus-01",
            "fastq.validate_reads",
        )
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
    fn summarize_root_pair_marks_duplicate_when_both_roots_exist() {
        let temp = tempfile::tempdir().expect("tempdir");
        let canonical = temp.path().join("canonical");
        let legacy = temp.path().join("legacy");
        std::fs::create_dir_all(canonical.join("results")).expect("create canonical");
        std::fs::create_dir_all(legacy.join("results")).expect("create legacy");

        let summary = summarize_root_pair("remote-results", &canonical, &legacy);
        assert_eq!(summary.status, "duplicate");
        assert!(summary.shared_entries.contains(&"results".to_string()));
    }

    #[test]
    fn plan_root_convergence_moves_unique_entries_and_drops_stale_duplicates() {
        let temp = tempfile::tempdir().expect("tempdir");
        let canonical_root = temp.path().join("results");
        let legacy_root = temp.path().join("bijux-dna-results");
        std::fs::create_dir_all(canonical_root.join("fastq.trim_reads")).expect("canonical root");
        std::fs::create_dir_all(legacy_root.join("fastq.trim_reads")).expect("legacy shared");
        std::fs::create_dir_all(legacy_root.join("fastq.filter_reads")).expect("legacy archive");
        std::fs::write(canonical_root.join("fastq.trim_reads/new.txt"), "fresh")
            .expect("write canonical file");
        std::fs::write(legacy_root.join("fastq.trim_reads/old.txt"), "old")
            .expect("write legacy duplicate");
        std::fs::write(legacy_root.join("fastq.filter_reads/report.json"), "{}")
            .expect("write unique legacy");
        let stale_time = filetime::FileTime::from_unix_time(1, 0);
        filetime::set_file_times(
            legacy_root.join("fastq.trim_reads/old.txt"),
            stale_time,
            stale_time,
        )
        .expect("stale legacy file");

        let plan = plan_root_convergence(&canonical_root, &legacy_root).expect("plan");
        let actions = plan
            .actions
            .into_iter()
            .map(|action| (action.entry_name, action.action))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            actions.get("fastq.trim_reads"),
            Some(&"remove-legacy-duplicate".to_string())
        );
        assert_eq!(
            actions.get("fastq.filter_reads"),
            Some(&"move-legacy-entry".to_string())
        );
    }

    #[test]
    fn normalize_workspace_layout_report_converges_shared_and_archive_only_stage_ids() {
        let temp = tempfile::tempdir().expect("tempdir");
        let results_root = temp.path().join("archive");
        let cache_mirror_root = temp.path().join("mirror");
        let legacy_stage_root = results_root
            .join("benchmark_corpus")
            .join("fastq.trim_reads");
        let canonical_stage_root = cache_mirror_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.trim_reads");
        let archive_only_stage_root = results_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads");
        std::fs::create_dir_all(legacy_stage_root.join("cluster-apptainer")).expect("legacy stage");
        std::fs::create_dir_all(canonical_stage_root.join("cluster-apptainer"))
            .expect("canonical stage");
        std::fs::create_dir_all(archive_only_stage_root.join("cluster-apptainer"))
            .expect("archive stage");
        std::fs::write(
            legacy_stage_root.join("cluster-apptainer/run_manifest.json"),
            "{}",
        )
        .expect("write legacy manifest");
        std::fs::write(
            canonical_stage_root.join("cluster-apptainer/run_manifest.json"),
            "{\"completed_at_utc\": \"2026-03-28T00:00:00Z\"}",
        )
        .expect("write canonical manifest");
        std::fs::write(
            archive_only_stage_root.join("cluster-apptainer/run_manifest.json"),
            "{\"completed_at_utc\": \"2026-03-27T00:00:00Z\"}",
        )
        .expect("write archive manifest");

        let report = normalize_workspace_layout_report(
            &sample_workspace(&results_root, &cache_mirror_root),
            "corpus-01",
            "benchmark_corpus",
            true,
        )
        .expect("report");

        assert!(!legacy_stage_root.exists());
        assert!(canonical_stage_root.exists());
        assert!(!archive_only_stage_root.exists());
        assert!(cache_mirror_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer")
            .exists());
        assert_eq!(report.status, "clear");
        assert_eq!(report.moved_stage_ids, vec!["fastq.validate_reads"]);
        assert_eq!(report.removed_duplicate_stage_ids, vec!["fastq.trim_reads"]);
        assert!(report.manual_review_stage_ids.is_empty());
    }

    #[test]
    fn benchmark_config_loads_workspace_and_publication_sections() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_unified_config(temp.path());

        let config = load_benchmark_config(temp.path(), None).expect("benchmark config");
        assert_eq!(
            config.workspace.remote.and_then(|row| row.corpus_root),
            Some("/bench/remote/cache/benchmark_corpus".to_string())
        );
        assert_eq!(
            config
                .publication
                .corpora
                .get(&benchmark_publication_corpus_key("corpus-01"))
                .cloned()
                .expect("corpus publication")
                .contracts
                .len(),
            1
        );
        assert_eq!(
            load_benchmark_publication_config(temp.path(), None)
                .expect("publication config")
                .corpora
                .get(&benchmark_publication_corpus_key("corpus-01"))
                .cloned()
                .expect("corpus publication")
                .exclusions
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
    fn corpus_spec_path_requires_declared_benchmark_contract() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_workspace(temp.path());
        let error = benchmark_corpus_spec_path(temp.path(), None, "corpus-01")
            .expect_err("missing corpus spec path must fail");
        assert!(error
            .to_string()
            .contains("benchmark config is missing corpora.corpus-01.spec_path"));
    }

    #[test]
    fn benchmark_runtime_corpus_dir_name_prefers_remote_corpus_root_basename() {
        let temp = tempfile::tempdir().expect("tempdir");
        write_workspace(temp.path());
        let workspace = load_benchmark_workspace_config(temp.path(), None).expect("workspace");
        assert_eq!(
            benchmark_runtime_corpus_dir_name(&workspace, "corpus-01").expect("dir name"),
            "benchmark_corpus"
        );
    }

    #[test]
    fn workspace_layout_corpus_dir_name_falls_back_to_declared_corpus_id() {
        let config = BenchmarkConfig {
            corpora: [(
                "corpus-42".to_string(),
                super::BenchmarkCorpusConfig {
                    spec_path: Some("configs/runtime/corpora/corpus-42.toml".to_string()),
                },
            )]
            .into_iter()
            .collect(),
            ..BenchmarkConfig::default()
        };

        assert_eq!(
            workspace_layout_corpus_dir_name(&config).expect("corpus dir name"),
            "corpus_42"
        );
    }

    #[test]
    fn benchmark_runtime_corpus_dir_name_requires_declared_remote_corpus_root() {
        let error =
            benchmark_runtime_corpus_dir_name(&BenchmarkWorkspaceConfig::default(), "corpus-01")
                .expect_err("missing remote corpus root must fail");
        assert!(error
            .to_string()
            .contains("benchmark config is missing workspace.remote.corpus_root"));
    }

    #[test]
    fn benchmark_stage_run_relative_root_requires_declared_templates() {
        let error = benchmark_stage_run_relative_root(
            &BenchmarkWorkspaceConfig::default(),
            "remote",
            "benchmark_corpus",
            "fastq.validate_reads",
        )
        .expect_err("missing templates must fail");
        assert!(error
            .to_string()
            .contains("benchmark config is missing workspace.layout.stage_runs template"));
    }

    #[test]
    fn benchmark_stage_run_relative_root_uses_workspace_templates() {
        let workspace = BenchmarkWorkspaceConfig {
            layout: Some(BenchmarkWorkspaceLayout {
                stage_runs: Some(BenchmarkWorkspaceStageRuns {
                    remote_results_template: Some("{corpus_id}/{stage_id}/cluster".to_string()),
                    local_cache_results_template: Some(
                        "results/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                    local_archive_results_template: Some(
                        "archive/{corpus_id}/{stage_id}/cluster".to_string(),
                    ),
                }),
            }),
            ..BenchmarkWorkspaceConfig::default()
        };
        assert_eq!(
            benchmark_stage_run_relative_root(
                &workspace,
                "remote",
                "benchmark_corpus",
                "fastq.validate_reads"
            )
            .expect("remote path"),
            std::path::PathBuf::from("benchmark_corpus/fastq.validate_reads/cluster")
        );
        assert_eq!(
            benchmark_stage_run_relative_root(
                &workspace,
                "local-cache",
                "benchmark_corpus",
                "fastq.validate_reads"
            )
            .expect("cache path"),
            std::path::PathBuf::from("results/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn benchmark_config_expands_environment_placeholders() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
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

    #[test]
    fn benchmark_config_treats_unset_placeholder_values_as_missing() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create config dir");
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[workspace.remote]
corpus_root = "${BIJUX_TEST_CORPUS_ROOT}"
"#,
        )
        .expect("write config");

        let error = load_benchmark_config(temp.path(), None)
            .expect_err("unset placeholder must fail during expansion");
        assert!(error
            .to_string()
            .contains("missing environment placeholder ${BIJUX_TEST_CORPUS_ROOT}"));
    }

    #[test]
    fn benchmark_config_expands_workspace_and_publication_placeholders() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create config dir");
        std::fs::write(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "${BIJUX_TEST_RESULTS_ROOT}"

[workspace.remote]
corpus_root = "${BIJUX_TEST_CORPUS_ROOT}"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
tools = ["fastqc"]
"#,
        )
        .expect("write benchmark config");

        std::env::set_var("BIJUX_TEST_RESULTS_ROOT", "/tmp/legacy-results");
        std::env::set_var("BIJUX_TEST_CORPUS_ROOT", "/tmp/legacy-corpus");
        let config = load_benchmark_config(temp.path(), None).expect("load benchmark config");
        std::env::remove_var("BIJUX_TEST_RESULTS_ROOT");
        std::env::remove_var("BIJUX_TEST_CORPUS_ROOT");

        assert_eq!(
            config.workspace.local.and_then(|row| row.results_root),
            Some("/tmp/legacy-results".to_string())
        );
        assert_eq!(
            config.workspace.remote.and_then(|row| row.corpus_root),
            Some("/tmp/legacy-corpus".to_string())
        );
        assert_eq!(
            config
                .publication
                .corpora
                .get(&benchmark_publication_corpus_key("corpus-01"))
                .cloned()
                .expect("corpus publication")
                .contracts
                .len(),
            1
        );
    }

    #[test]
    fn benchmark_config_rejects_missing_environment_placeholders() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let error = expand_env_placeholders("results_root = \"${BIJUX_MISSING_RESULTS_ROOT}\"\n")
            .expect_err("missing environment placeholder must fail");
        assert!(error
            .to_string()
            .contains("missing environment placeholder ${BIJUX_MISSING_RESULTS_ROOT}"));
    }
}
