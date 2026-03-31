use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::{
    benchmark_config_path, benchmark_publication_config_path, benchmark_workspace_config_path,
    BenchmarkConfig, BenchmarkPublicationConfig, BenchmarkWorkspaceConfig,
};

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
