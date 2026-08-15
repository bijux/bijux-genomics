//! Benchmark configuration validation and path admission.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkConfig, BenchmarkPublicationConfig, BenchmarkWorkspaceConfig, CorpusBenchmarkContract,
    CorpusBenchmarkExclusion,
};

/// Expand `${NAME}` placeholders in raw benchmark configuration text from the process environment.
///
/// # Errors
/// Returns an error when a referenced environment variable is not set.
pub fn expand_benchmark_config_env_placeholders(raw: &str) -> Result<String> {
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

/// Parse and normalize one benchmark configuration document.
///
/// # Errors
/// Returns an error when environment expansion or TOML deserialization fails.
pub fn parse_benchmark_config(raw: &str) -> Result<BenchmarkConfig> {
    let expanded = expand_benchmark_config_env_placeholders(raw)?;
    let config = toml::from_str::<BenchmarkConfig>(&expanded).context("parse benchmark config")?;
    Ok(normalize_benchmark_config(config))
}

/// Normalize optional string fields in a benchmark configuration.
#[must_use]
pub fn normalize_benchmark_config(mut config: BenchmarkConfig) -> BenchmarkConfig {
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
    normalize_optional_string(&mut config.stage_inputs.fastq_deplete_host.reference_index_backend);
    normalize_optional_string(
        &mut config.stage_inputs.fastq_deplete_reference_contaminants.reference_index,
    );
    normalize_optional_string(
        &mut config.stage_inputs.fastq_deplete_reference_contaminants.reference_catalog_id,
    );
    normalize_optional_string(
        &mut config.stage_inputs.fastq_deplete_reference_contaminants.reference_index_backend,
    );
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_root);
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_catalog_id);
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_artifact_id);
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_namespace);
    normalize_optional_string(&mut config.stage_inputs.fastq_screen_taxonomy.database_scope);
    config
}

/// Convert a corpus identifier to its TOML publication-table key.
#[must_use]
pub fn benchmark_publication_corpus_key(corpus_id: &str) -> String {
    corpus_id.replace('-', "_")
}

/// Convert a TOML publication-table key to its corpus identifier.
#[must_use]
pub fn benchmark_publication_corpus_id(publication_key: &str) -> String {
    publication_key.replace('_', "-")
}

/// Resolve one configured corpus specification path relative to the workspace root.
///
/// # Errors
/// Returns an error when the corpus is undeclared or has no specification path.
pub fn benchmark_corpus_spec_path_from_config(
    cwd: &Path,
    config: &BenchmarkConfig,
    corpus_id: &str,
) -> Result<PathBuf> {
    config
        .corpora
        .get(corpus_id)
        .and_then(|row| row.spec_path.as_deref())
        .map(|path| absolutize(cwd, Path::new(path)))
        .ok_or_else(|| anyhow!("benchmark config is missing corpora.{corpus_id}.spec_path"))
}

/// Select all publication contracts for one declared corpus.
///
/// # Errors
/// Returns an error when the publication configuration has no entry for the corpus.
pub fn benchmark_publication_contracts_from_config(
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

/// Select one stage publication contract for a declared corpus.
///
/// # Errors
/// Returns an error when the corpus publication entry or requested stage contract is missing.
pub fn benchmark_publication_contract_from_config(
    publication: &BenchmarkPublicationConfig,
    corpus_id: &str,
    stage_id: &str,
) -> Result<CorpusBenchmarkContract> {
    benchmark_publication_contracts_from_config(publication, corpus_id)?
        .into_iter()
        .find(|row| row.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing {corpus_id} publication contract for {stage_id}"))
}

/// Select all scientific exclusions for one declared corpus.
///
/// # Errors
/// Returns an error when the publication configuration has no entry for the corpus.
pub fn benchmark_publication_exclusions_from_config(
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

/// Resolve one supported benchmark workspace configuration value.
///
/// # Errors
/// Returns an error for an unsupported key or a supported key whose value is absent.
pub fn benchmark_workspace_config_value(
    workspace: &BenchmarkWorkspaceConfig,
    key: &str,
) -> Result<String> {
    let value = match key {
        "local.results_root" => workspace.local.as_ref().and_then(|row| row.results_root.clone()),
        "local.cache_mirror_root" => {
            workspace.local.as_ref().and_then(|row| row.cache_mirror_root.clone())
        }
        "local.extra_data_root" => {
            workspace.local.as_ref().and_then(|row| row.extra_data_root.clone())
        }
        "local.reference_root" => {
            workspace.local.as_ref().and_then(|row| row.reference_root.clone())
        }
        "remote.ssh_host" => workspace.remote.as_ref().and_then(|row| row.ssh_host.clone()),
        "remote.repo_root" => workspace.remote.as_ref().and_then(|row| row.repo_root.clone()),
        "remote.cache_root" => workspace.remote.as_ref().and_then(|row| row.cache_root.clone()),
        "remote.corpus_root" => workspace.remote.as_ref().and_then(|row| row.corpus_root.clone()),
        "remote.results_root" => workspace.remote.as_ref().and_then(|row| row.results_root.clone()),
        "remote.extra_data_root" => {
            workspace.remote.as_ref().and_then(|row| row.extra_data_root.clone())
        }
        "remote.containers_root" => {
            workspace.remote.as_ref().and_then(|row| row.containers_root.clone())
        }
        "remote.reference_root" => {
            workspace.remote.as_ref().and_then(|row| row.reference_root.clone())
        }
        "sync.defaults.pull_base" => workspace_sync_value(workspace, |row| row.pull_base.clone()),
        "sync.defaults.pull_mode" => workspace_sync_value(workspace, |row| row.pull_mode.clone()),
        "sync.defaults.include_profile" => {
            workspace_sync_value(workspace, |row| row.include_profile.clone())
        }
        "sync.defaults.exclude_profile" => {
            workspace_sync_value(workspace, |row| row.exclude_profile.clone())
        }
        "sync.defaults.clean_context" => {
            workspace_sync_value(workspace, |row| row.clean_context.map(|value| value.to_string()))
        }
        "sync.defaults.allow_dirty" => {
            workspace_sync_value(workspace, |row| row.allow_dirty.map(|value| value.to_string()))
        }
        "sync.defaults.include_containers_manifest" => workspace_sync_value(workspace, |row| {
            row.include_containers_manifest.map(|value| value.to_string())
        }),
        "sync.defaults.data_manifest_glob" => {
            workspace_sync_value(workspace, |row| row.data_manifest_glob.clone())
        }
        other => return Err(anyhow!("unsupported benchmark workspace key `{other}`")),
    };
    value.ok_or_else(|| anyhow!("missing benchmark workspace value for `{key}`"))
}

/// Render one supported benchmark configuration section as pretty JSON.
///
/// # Errors
/// Returns an error for an unsupported section or failed serialization.
pub fn render_benchmark_config_section_json(
    config: &BenchmarkConfig,
    section: &str,
) -> Result<String> {
    match section {
        "full" => serde_json::to_string_pretty(config),
        "workspace" => serde_json::to_string_pretty(&config.workspace),
        "publication" => serde_json::to_string_pretty(&config.publication),
        "corpora" => serde_json::to_string_pretty(&config.corpora),
        "stage_inputs" => serde_json::to_string_pretty(&config.stage_inputs),
        other => {
            return Err(anyhow!(
                "unsupported benchmark config section `{other}`; expected one of: full, workspace, publication, corpora, stage_inputs"
            ));
        }
    }
    .context("serialize benchmark configuration section")
}

/// Inputs required to validate one loaded benchmark configuration.
pub struct BenchmarkConfigValidationRequest<'a> {
    pub cwd: &'a Path,
    pub config_path: &'a Path,
    pub config: &'a BenchmarkConfig,
    pub check_paths: bool,
}

/// Stable summary returned after benchmark configuration admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkConfigValidationSummary {
    pub corpus_count: usize,
    pub paths_checked: bool,
}

/// Validate benchmark workspace, publication, corpus, and required stage-input contracts.
///
/// # Errors
/// Returns all detected configuration defects in one error, including missing required values,
/// unmatched publication corpora, and missing paths when path checking is enabled.
pub fn validate_benchmark_config_contract(
    request: &BenchmarkConfigValidationRequest<'_>,
) -> Result<BenchmarkConfigValidationSummary> {
    let config = request.config;
    let mut errors = Vec::new();

    for (key, value) in [
        (
            "workspace.local.results_root",
            config.workspace.local.as_ref().and_then(|row| row.results_root.as_deref()),
        ),
        (
            "workspace.local.cache_mirror_root",
            config.workspace.local.as_ref().and_then(|row| row.cache_mirror_root.as_deref()),
        ),
        (
            "workspace.local.extra_data_root",
            config.workspace.local.as_ref().and_then(|row| row.extra_data_root.as_deref()),
        ),
        (
            "workspace.remote.ssh_host",
            config.workspace.remote.as_ref().and_then(|row| row.ssh_host.as_deref()),
        ),
        (
            "workspace.remote.repo_root",
            config.workspace.remote.as_ref().and_then(|row| row.repo_root.as_deref()),
        ),
        (
            "workspace.remote.cache_root",
            config.workspace.remote.as_ref().and_then(|row| row.cache_root.as_deref()),
        ),
        (
            "workspace.remote.corpus_root",
            config.workspace.remote.as_ref().and_then(|row| row.corpus_root.as_deref()),
        ),
        (
            "workspace.remote.results_root",
            config.workspace.remote.as_ref().and_then(|row| row.results_root.as_deref()),
        ),
        (
            "workspace.remote.extra_data_root",
            config.workspace.remote.as_ref().and_then(|row| row.extra_data_root.as_deref()),
        ),
        (
            "workspace.layout.stage_runs.remote_results_template",
            stage_run_value(config, |row| row.remote_results_template.as_deref()),
        ),
        (
            "workspace.layout.stage_runs.local_cache_results_template",
            stage_run_value(config, |row| row.local_cache_results_template.as_deref()),
        ),
        (
            "workspace.layout.stage_runs.local_archive_results_template",
            stage_run_value(config, |row| row.local_archive_results_template.as_deref()),
        ),
    ] {
        require_value(&mut errors, key, value);
    }

    if config.corpora.is_empty() {
        errors
            .push("benchmark config must declare at least one corpus under [corpora]".to_string());
    }
    for (corpus_id, corpus) in &config.corpora {
        require_value(
            &mut errors,
            &format!("corpora.{corpus_id}.spec_path"),
            corpus.spec_path.as_deref(),
        );
        let publication_key = benchmark_publication_corpus_key(corpus_id);
        if config
            .publication
            .corpora
            .get(&publication_key)
            .is_none_or(|row| row.contracts.is_empty())
        {
            errors.push(format!(
                "publication.{publication_key}.contracts is empty for declared corpus {corpus_id}"
            ));
        }
    }
    for publication_key in config.publication.corpora.keys() {
        let corpus_id = benchmark_publication_corpus_id(publication_key);
        if !config.corpora.contains_key(&corpus_id) {
            errors.push(format!(
                "publication.{publication_key} does not match any declared corpus under [corpora]"
            ));
        }
    }

    if request.check_paths {
        for (corpus_id, corpus) in &config.corpora {
            if let Some(path) = corpus
                .spec_path
                .as_deref()
                .map(|row| absolutize(request.cwd, Path::new(row)))
                .filter(|row| !row.is_file())
            {
                errors.push(format!("missing corpus spec for {corpus_id}: {}", path.display()));
            }
        }
        for (key, value) in [
            (
                "stage_inputs.fastq_deplete_rrna.rrna_db",
                config.stage_inputs.fastq_deplete_rrna.rrna_db.as_deref(),
            ),
            (
                "stage_inputs.fastq_deplete_host.reference_index",
                config.stage_inputs.fastq_deplete_host.reference_index.as_deref(),
            ),
            (
                "stage_inputs.fastq_deplete_reference_contaminants.reference_index",
                config.stage_inputs.fastq_deplete_reference_contaminants.reference_index.as_deref(),
            ),
            (
                "stage_inputs.fastq_screen_taxonomy.database_root",
                config.stage_inputs.fastq_screen_taxonomy.database_root.as_deref(),
            ),
        ] {
            require_existing_path(&mut errors, key, value, request.cwd);
        }
    }

    if errors.is_empty() {
        Ok(BenchmarkConfigValidationSummary {
            corpus_count: config.corpora.len(),
            paths_checked: request.check_paths,
        })
    } else {
        Err(anyhow!(
            "benchmark config validation failed for {}:\n{}",
            request.config_path.display(),
            errors.join("\n")
        ))
    }
}

fn stage_run_value<'a>(
    config: &'a BenchmarkConfig,
    project: impl FnOnce(&'a bijux_dna_bench_model::BenchmarkWorkspaceStageRuns) -> Option<&'a str>,
) -> Option<&'a str> {
    config.workspace.layout.as_ref().and_then(|row| row.stage_runs.as_ref()).and_then(project)
}

fn normalize_optional_string(value: &mut Option<String>) {
    let Some(raw) = value.take() else {
        return;
    };
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        *value = Some(trimmed.to_string());
    }
}

fn workspace_sync_value(
    workspace: &BenchmarkWorkspaceConfig,
    project: impl FnOnce(&bijux_dna_bench_model::BenchmarkWorkspaceSyncDefaults) -> Option<String>,
) -> Option<String> {
    workspace.sync.as_ref().and_then(|row| row.defaults.as_ref()).and_then(project)
}

fn require_value(errors: &mut Vec<String>, key: &str, value: Option<&str>) {
    if value.is_none_or(|row| row.trim().is_empty()) {
        errors.push(format!("missing required benchmark config key: {key}"));
    }
}

fn require_existing_path(errors: &mut Vec<String>, key: &str, value: Option<&str>, cwd: &Path) {
    let Some(raw) = value.map(str::trim).filter(|row| !row.is_empty()) else {
        errors.push(format!("missing required benchmark config key: {key}"));
        return;
    };
    let path = absolutize(cwd, Path::new(raw));
    if !path.exists() {
        errors.push(format!("missing configured path for {key}: {}", path.display()));
    }
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
