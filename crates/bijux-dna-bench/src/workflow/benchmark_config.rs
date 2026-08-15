//! Benchmark configuration validation and path admission.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_bench_model::BenchmarkConfig;

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
        let publication_key = corpus_id.replace('-', "_");
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
        let corpus_id = publication_key.replace('_', "-");
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
