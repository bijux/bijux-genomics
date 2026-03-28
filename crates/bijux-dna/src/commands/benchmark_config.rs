use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::commands::benchmark_workspace::{
    benchmark_config_path, benchmark_corpus_spec_path, load_benchmark_config,
};
use crate::commands::cli::BenchConfigValidateArgs;

pub(crate) fn validate_benchmark_config(cwd: &Path, args: &BenchConfigValidateArgs) -> Result<()> {
    let path = benchmark_config_path(cwd, args.config.as_deref());
    let config = load_benchmark_config(cwd, args.config.as_deref())?;
    let mut errors = Vec::new();

    require_value(
        &mut errors,
        "workspace.local.results_root",
        config
            .workspace
            .local
            .as_ref()
            .and_then(|row| row.results_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.ssh_host",
        config
            .workspace
            .remote
            .as_ref()
            .and_then(|row| row.ssh_host.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.repo_root",
        config
            .workspace
            .remote
            .as_ref()
            .and_then(|row| row.repo_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.corpus_root",
        config
            .workspace
            .remote
            .as_ref()
            .and_then(|row| row.corpus_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.results_root",
        config
            .workspace
            .remote
            .as_ref()
            .and_then(|row| row.results_root.as_deref()),
    );
    if config
        .publication
        .corpus_01
        .as_ref()
        .is_none_or(|row| row.contracts.is_empty())
    {
        errors.push("publication.corpus_01.contracts is empty".to_string());
    }

    let corpus_rows = if config.corpora.is_empty() {
        vec!["corpus-01".to_string()]
    } else {
        config.corpora.keys().cloned().collect()
    };
    for corpus_id in corpus_rows {
        let spec_path = benchmark_corpus_spec_path(cwd, args.config.as_deref(), &corpus_id)?;
        if args.check_paths && !spec_path.is_file() {
            errors.push(format!(
                "missing corpus spec for {corpus_id}: {}",
                spec_path.display()
            ));
        }
    }

    if args.check_paths {
        require_existing_path(
            &mut errors,
            "stage_inputs.fastq_deplete_rrna.rrna_db",
            config.stage_inputs.fastq_deplete_rrna.rrna_db.as_deref(),
            cwd,
        );
        require_existing_path(
            &mut errors,
            "stage_inputs.fastq_deplete_host.reference_index",
            config
                .stage_inputs
                .fastq_deplete_host
                .reference_index
                .as_deref(),
            cwd,
        );
        require_existing_path(
            &mut errors,
            "stage_inputs.fastq_deplete_reference_contaminants.reference_index",
            config
                .stage_inputs
                .fastq_deplete_reference_contaminants
                .reference_index
                .as_deref(),
            cwd,
        );
        require_existing_path(
            &mut errors,
            "stage_inputs.fastq_screen_taxonomy.database_root",
            config
                .stage_inputs
                .fastq_screen_taxonomy
                .database_root
                .as_deref(),
            cwd,
        );
    }

    if !errors.is_empty() {
        return Err(anyhow!(
            "benchmark config validation failed for {}:\n{}",
            path.display(),
            errors.join("\n")
        ));
    }

    println!("benchmark_config={}", path.display());
    println!("workspace=ok");
    println!("publication=ok");
    println!("corpora={}", config.corpora.len().max(1));
    println!("paths_checked={}", args.check_paths);
    Ok(())
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
        errors.push(format!(
            "missing configured path for {key}: {}",
            path.display()
        ));
    }
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}
