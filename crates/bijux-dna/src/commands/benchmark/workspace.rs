#![allow(clippy::too_many_lines)]

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

mod config_loading;
mod config_paths;
mod contracts;
mod layout_normalization;
mod layout_status;
mod publication_contracts;

pub(crate) use self::config_loading::{
    expand_env_placeholders, load_benchmark_config, load_benchmark_publication_config,
    load_benchmark_workspace_config, load_optional_benchmark_workspace_config,
};
use self::config_paths::absolutize;
pub(crate) use self::config_paths::{
    benchmark_config_path, benchmark_publication_config_path, benchmark_workspace_config_path,
};
pub(crate) use self::contracts::{
    BenchmarkConfig, BenchmarkCorpusConfig, BenchmarkCorpusPublicationConfig,
    BenchmarkDepleteRrnaInputConfig, BenchmarkPublicationConfig, BenchmarkReferenceInputConfig,
    BenchmarkScreenTaxonomyInputConfig, BenchmarkStageInputConfig, BenchmarkWorkspaceArtifact,
    BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout, BenchmarkWorkspaceLocal,
    BenchmarkWorkspaceRemote, BenchmarkWorkspaceStageRuns, BenchmarkWorkspaceSync,
    BenchmarkWorkspaceSyncDefaults, CorpusBenchmarkContract, CorpusBenchmarkExclusion,
};
pub(crate) use self::layout_normalization::normalize_workspace_layout_report;
pub(crate) use self::layout_status::write_workspace_layout_status;
pub(crate) use self::publication_contracts::{
    benchmark_publication_contract, benchmark_publication_contracts,
    benchmark_publication_exclusions,
};

pub(crate) const BENCHMARK_CONFIG_ENV: &str = "BIJUX_BENCHMARK_CONFIG";
pub(crate) const BENCHMARK_CONFIG_JSON_ENV: &str = "BIJUX_BENCHMARK_CONFIG_JSON";
pub(crate) const DEFAULT_BENCHMARK_CONFIG: &str = "configs/bench/benchmark.toml";

pub(crate) fn benchmark_publication_corpus_key(corpus_id: &str) -> String {
    corpus_id.replace('-', "_")
}

pub(crate) fn benchmark_publication_corpus_id(publication_key: &str) -> String {
    publication_key.replace('_', "-")
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

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::layout_normalization::plan_root_convergence;
    use super::layout_status::{summarize_root_pair, workspace_layout_corpus_dir_name};
    use super::{
        benchmark_config_path, benchmark_corpus_spec_path, benchmark_publication_config_path,
        benchmark_publication_corpus_key, benchmark_runtime_corpus_dir_name,
        benchmark_stage_run_relative_root, benchmark_workspace_config_path,
        benchmark_workspace_value, expand_env_placeholders, load_benchmark_config,
        load_benchmark_publication_config, load_benchmark_workspace_config,
        load_optional_benchmark_workspace_config, normalize_workspace_layout_report,
        BenchmarkConfig, BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout,
        BenchmarkWorkspaceLocal, BenchmarkWorkspaceStageRuns, BENCHMARK_CONFIG_ENV,
    };
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_text(path: impl AsRef<Path>, content: &str) {
        bijux_dna_infra::write_bytes(path.as_ref(), content.as_bytes()).expect("write fixture");
    }

    fn write_workspace(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        write_text(
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
        );
    }

    fn write_publication(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqc"]
"#,
        );
    }

    fn write_unified_config(root: &Path) {
        let config_dir = root.join("configs/bench");
        std::fs::create_dir_all(&config_dir).expect("create bench config dir");
        write_text(
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
        );
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
    fn workspace_path_honors_explicit_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let config_path = temp.path().join("custom.toml");
        write_text(&config_path, "");
        let resolved = benchmark_workspace_config_path(temp.path(), Some(&config_path));
        assert_eq!(resolved, config_path);
    }

    #[test]
    fn benchmark_config_path_honors_benchmark_config_env_binding() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("tempdir");
        let config_path = temp.path().join("custom-benchmark.toml");
        write_text(&config_path, "");

        std::env::set_var(BENCHMARK_CONFIG_ENV, &config_path);
        let benchmark_path = benchmark_config_path(temp.path(), None);
        let workspace_path = benchmark_workspace_config_path(temp.path(), None);
        let publication_path = benchmark_publication_config_path(temp.path(), None);
        std::env::remove_var(BENCHMARK_CONFIG_ENV);

        assert_eq!(benchmark_path, config_path);
        assert_eq!(workspace_path, config_path);
        assert_eq!(publication_path, config_path);
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
        write_text(canonical_root.join("fastq.trim_reads/new.txt"), "fresh");
        write_text(legacy_root.join("fastq.trim_reads/old.txt"), "old");
        write_text(legacy_root.join("fastq.filter_reads/report.json"), "{}");
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
        write_text(
            legacy_stage_root.join("cluster-apptainer/run_manifest.json"),
            "{}",
        );
        write_text(
            canonical_stage_root.join("cluster-apptainer/run_manifest.json"),
            "{\"completed_at_utc\": \"2026-03-28T00:00:00Z\"}",
        );
        write_text(
            archive_only_stage_root.join("cluster-apptainer/run_manifest.json"),
            "{\"completed_at_utc\": \"2026-03-27T00:00:00Z\"}",
        );

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
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.local]
results_root = "${BIJUX_TEST_RESULTS_ROOT}"
"#,
        );
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
        write_text(
            config_dir.join("benchmark.toml"),
            r#"[workspace.remote]
corpus_root = "${BIJUX_TEST_CORPUS_ROOT}"
"#,
        );

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
        write_text(
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
        );

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
