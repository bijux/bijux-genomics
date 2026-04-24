#![allow(clippy::too_many_lines)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::commands::benchmark_workspace::{
    benchmark_config_path, benchmark_publication_corpus_id, benchmark_publication_corpus_key,
    load_benchmark_config,
};
use crate::commands::cli::BenchConfigValidateArgs;

pub(crate) fn validate_benchmark_config(cwd: &Path, args: &BenchConfigValidateArgs) -> Result<()> {
    let path = benchmark_config_path(cwd, args.config.as_deref());
    let config = load_benchmark_config(cwd, args.config.as_deref())?;
    let mut errors = Vec::new();

    require_value(
        &mut errors,
        "workspace.local.results_root",
        config.workspace.local.as_ref().and_then(|row| row.results_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.local.cache_mirror_root",
        config.workspace.local.as_ref().and_then(|row| row.cache_mirror_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.local.extra_data_root",
        config.workspace.local.as_ref().and_then(|row| row.extra_data_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.ssh_host",
        config.workspace.remote.as_ref().and_then(|row| row.ssh_host.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.repo_root",
        config.workspace.remote.as_ref().and_then(|row| row.repo_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.cache_root",
        config.workspace.remote.as_ref().and_then(|row| row.cache_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.corpus_root",
        config.workspace.remote.as_ref().and_then(|row| row.corpus_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.results_root",
        config.workspace.remote.as_ref().and_then(|row| row.results_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.remote.extra_data_root",
        config.workspace.remote.as_ref().and_then(|row| row.extra_data_root.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.layout.stage_runs.remote_results_template",
        config
            .workspace
            .layout
            .as_ref()
            .and_then(|row| row.stage_runs.as_ref())
            .and_then(|row| row.remote_results_template.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.layout.stage_runs.local_cache_results_template",
        config
            .workspace
            .layout
            .as_ref()
            .and_then(|row| row.stage_runs.as_ref())
            .and_then(|row| row.local_cache_results_template.as_deref()),
    );
    require_value(
        &mut errors,
        "workspace.layout.stage_runs.local_archive_results_template",
        config
            .workspace
            .layout
            .as_ref()
            .and_then(|row| row.stage_runs.as_ref())
            .and_then(|row| row.local_archive_results_template.as_deref()),
    );
    if config.corpora.is_empty() {
        errors
            .push("benchmark config must declare at least one corpus under [corpora]".to_string());
    }

    let corpus_rows = config.corpora.keys().cloned().collect::<Vec<_>>();
    for corpus_id in &corpus_rows {
        require_value(
            &mut errors,
            &format!("corpora.{corpus_id}.spec_path"),
            config.corpora.get(corpus_id).and_then(|row| row.spec_path.as_deref()),
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
    for corpus_id in corpus_rows {
        let spec_path = config
            .corpora
            .get(&corpus_id)
            .and_then(|row| row.spec_path.as_deref())
            .map(|row| absolutize(cwd, Path::new(row)));
        if let Some(spec_path) = spec_path.filter(|row| args.check_paths && !row.is_file()) {
            errors.push(format!("missing corpus spec for {corpus_id}: {}", spec_path.display()));
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
            config.stage_inputs.fastq_deplete_host.reference_index.as_deref(),
            cwd,
        );
        require_existing_path(
            &mut errors,
            "stage_inputs.fastq_deplete_reference_contaminants.reference_index",
            config.stage_inputs.fastq_deplete_reference_contaminants.reference_index.as_deref(),
            cwd,
        );
        require_existing_path(
            &mut errors,
            "stage_inputs.fastq_screen_taxonomy.database_root",
            config.stage_inputs.fastq_screen_taxonomy.database_root.as_deref(),
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
    println!("corpora={}", config.corpora.len());
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::validate_benchmark_config;

    fn write_required_stage_input_fixtures(root: &std::path::Path) {
        let rrna_db = root.join("inputs/rrna");
        let host_reference_index = root.join("inputs/reference/host");
        let contaminant_reference_index = root.join("inputs/reference/contaminants");
        let taxonomy_database = root.join("inputs/taxonomy/database");

        std::fs::create_dir_all(&rrna_db).expect("rrna fixture dir");
        std::fs::create_dir_all(&host_reference_index).expect("host reference fixture dir");
        std::fs::create_dir_all(&contaminant_reference_index)
            .expect("contaminant reference fixture dir");
        std::fs::create_dir_all(&taxonomy_database).expect("taxonomy fixture dir");

        bijux_dna_infra::write_bytes(rrna_db.join("rrna.db"), b"").expect("write rrna fixture");
        bijux_dna_infra::write_bytes(host_reference_index.join("index"), b"")
            .expect("write host reference fixture");
        bijux_dna_infra::write_bytes(contaminant_reference_index.join("index"), b"")
            .expect("write contaminant reference fixture");
        bijux_dna_infra::write_bytes(taxonomy_database.join("nodes.dmp"), b"")
            .expect("write taxonomy fixture");
    }

    fn write_text(path: impl AsRef<std::path::Path>, content: &str) {
        bijux_dna_infra::write_bytes(path.as_ref(), content.as_bytes()).expect("write fixture");
    }

    #[test]
    fn validate_benchmark_config_requires_declared_corpora() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"
cache_mirror_root = "/bench/local/cache-mirror"
extra_data_root = "/bench/local/extra-data"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
cache_root = "/bench/remote/cache"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"
extra_data_root = "/bench/remote/cache/extra-data"

[workspace.layout.stage_runs]
remote_results_template = "{corpus_id}/{stage_id}/cluster"
local_cache_results_template = "results/{corpus_id}/{stage_id}/cluster"
local_archive_results_template = "{corpus_id}/{stage_id}/cluster"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]
"#,
        );

        let error = validate_benchmark_config(
            temp.path(),
            &crate::commands::cli::BenchConfigValidateArgs { config: None, check_paths: false },
        )
        .expect_err("validator should reject missing corpora");

        assert!(error
            .to_string()
            .contains("benchmark config must declare at least one corpus under [corpora]"));
    }

    #[test]
    fn validate_benchmark_config_requires_publication_contract_for_declared_corpus() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"
cache_mirror_root = "/bench/local/cache-mirror"
extra_data_root = "/bench/local/extra-data"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
cache_root = "/bench/remote/cache"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"
extra_data_root = "/bench/remote/cache/extra-data"

[workspace.layout.stage_runs]
remote_results_template = "{corpus_id}/{stage_id}/cluster"
local_cache_results_template = "results/{corpus_id}/{stage_id}/cluster"
local_archive_results_template = "{corpus_id}/{stage_id}/cluster"

[corpora.corpus-01]
spec_path = "configs/runtime/corpora/corpus-01.toml"
"#,
        );

        let error = validate_benchmark_config(
            temp.path(),
            &crate::commands::cli::BenchConfigValidateArgs { config: None, check_paths: false },
        )
        .expect_err("validator should reject missing publication contracts");

        assert!(error
            .to_string()
            .contains("publication.corpus_01.contracts is empty for declared corpus corpus-01"));
    }

    #[test]
    fn validate_benchmark_config_requires_declared_corpus_spec_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"
cache_mirror_root = "/bench/local/cache-mirror"
extra_data_root = "/bench/local/extra-data"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
cache_root = "/bench/remote/cache"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"
extra_data_root = "/bench/remote/cache/extra-data"

[workspace.layout.stage_runs]
remote_results_template = "{corpus_id}/{stage_id}/cluster"
local_cache_results_template = "results/{corpus_id}/{stage_id}/cluster"
local_archive_results_template = "{corpus_id}/{stage_id}/cluster"

[corpora.corpus-01]

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]
"#,
        );

        let error = validate_benchmark_config(
            temp.path(),
            &crate::commands::cli::BenchConfigValidateArgs { config: None, check_paths: false },
        )
        .expect_err("validator should reject missing corpus spec path");

        assert!(error
            .to_string()
            .contains("missing required benchmark config key: corpora.corpus-01.spec_path"));
    }

    #[test]
    fn validate_benchmark_config_requires_workspace_layout_contracts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"

[corpora.corpus-01]
spec_path = "configs/runtime/corpora/corpus-01.toml"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]
"#,
        );

        let error = validate_benchmark_config(
            temp.path(),
            &crate::commands::cli::BenchConfigValidateArgs { config: None, check_paths: false },
        )
        .expect_err("validator should reject missing workspace contracts");

        let rendered = error.to_string();
        assert!(rendered
            .contains("missing required benchmark config key: workspace.local.cache_mirror_root"));
        assert!(rendered
            .contains("missing required benchmark config key: workspace.local.extra_data_root"));
        assert!(
            rendered.contains("missing required benchmark config key: workspace.remote.cache_root")
        );
        assert!(rendered
            .contains("missing required benchmark config key: workspace.remote.extra_data_root"));
        assert!(rendered.contains("missing required benchmark config key: workspace.layout.stage_runs.remote_results_template"));
        assert!(rendered.contains("missing required benchmark config key: workspace.layout.stage_runs.local_cache_results_template"));
        assert!(rendered.contains("missing required benchmark config key: workspace.layout.stage_runs.local_archive_results_template"));
    }

    #[test]
    fn validate_benchmark_config_rejects_undeclared_publication_corpus() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"
cache_mirror_root = "/bench/local/cache-mirror"
extra_data_root = "/bench/local/extra-data"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
cache_root = "/bench/remote/cache"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"
extra_data_root = "/bench/remote/cache/extra-data"

[workspace.layout.stage_runs]
remote_results_template = "{corpus_id}/{stage_id}/cluster"
local_cache_results_template = "results/{corpus_id}/{stage_id}/cluster"
local_archive_results_template = "{corpus_id}/{stage_id}/cluster"

[corpora.corpus-01]
spec_path = "configs/runtime/corpora/corpus-01.toml"

[[publication.study_42.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]
"#,
        );

        let error = validate_benchmark_config(
            temp.path(),
            &crate::commands::cli::BenchConfigValidateArgs { config: None, check_paths: false },
        )
        .expect_err("validator should reject undeclared publication corpus");

        assert!(error
            .to_string()
            .contains("publication.study_42 does not match any declared corpus under [corpora]"));
    }

    #[test]
    fn validate_benchmark_config_accepts_declared_corpora() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("config dir");
        std::fs::create_dir_all(temp.path().join("configs/runtime/corpora")).expect("corpus dir");
        write_required_stage_input_fixtures(temp.path());
        write_text(
            temp.path().join("configs/runtime/corpora/corpus-01.toml"),
            "corpus_id = \"corpus-01\"\n",
        );
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "/bench/local/results"
cache_mirror_root = "/bench/local/cache-mirror"
extra_data_root = "/bench/local/extra-data"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/bench/remote/repo"
cache_root = "/bench/remote/cache"
corpus_root = "/bench/remote/cache/benchmark_corpus"
results_root = "/bench/remote/cache/results"
extra_data_root = "/bench/remote/cache/extra-data"

[workspace.layout.stage_runs]
remote_results_template = "{corpus_id}/{stage_id}/cluster"
local_cache_results_template = "results/{corpus_id}/{stage_id}/cluster"
local_archive_results_template = "{corpus_id}/{stage_id}/cluster"

[stage_inputs.fastq_deplete_rrna]
rrna_db = "inputs/rrna/rrna.db"

[stage_inputs.fastq_deplete_host]
reference_index = "inputs/reference/host/index"

[stage_inputs.fastq_deplete_reference_contaminants]
reference_index = "inputs/reference/contaminants/index"

[stage_inputs.fastq_screen_taxonomy]
database_root = "inputs/taxonomy/database"

[corpora.corpus-01]
spec_path = "configs/runtime/corpora/corpus-01.toml"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator"]
"#,
        );

        validate_benchmark_config(
            temp.path(),
            &crate::commands::cli::BenchConfigValidateArgs { config: None, check_paths: true },
        )
        .expect("validator should accept declared corpus config");
    }
}
