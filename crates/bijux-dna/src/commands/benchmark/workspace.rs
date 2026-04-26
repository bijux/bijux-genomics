#![allow(clippy::too_many_lines)]

mod config_loading;
mod config_paths;
mod config_queries;
mod contracts;
mod layout_normalization;
mod layout_status;
mod publication_contracts;
mod stage_run_layout;
mod value_queries;

pub(crate) use self::config_loading::{
    expand_env_placeholders, load_benchmark_config, load_benchmark_publication_config,
    load_benchmark_workspace_config, load_optional_benchmark_workspace_config,
};
pub(crate) use self::config_paths::{
    benchmark_config_path, benchmark_publication_config_path, benchmark_workspace_config_path,
};
pub(crate) use self::config_queries::{benchmark_corpus_spec_path, print_benchmark_config_json};
#[allow(unused_imports)]
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
pub(crate) use self::stage_run_layout::{
    benchmark_runtime_corpus_dir_name, benchmark_stage_run_relative_root,
    run_normalize_workspace_layout,
};
pub(crate) use self::value_queries::benchmark_workspace_value;

pub(crate) const BENCHMARK_CONFIG_ENV: &str = "BIJUX_BENCHMARK_CONFIG";
pub(crate) const BENCHMARK_CONFIG_JSON_ENV: &str = "BIJUX_BENCHMARK_CONFIG_JSON";
pub(crate) const DEFAULT_BENCHMARK_CONFIG: &str = "configs/bench/benchmark.toml";

pub(crate) fn benchmark_publication_corpus_key(corpus_id: &str) -> String {
    corpus_id.replace('-', "_")
}

pub(crate) fn benchmark_publication_corpus_id(publication_key: &str) -> String {
    publication_key.replace('_', "-")
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
    use std::ffi::{OsStr, OsString};
    use std::path::Path;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        value: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
            let guard = Self { key, value: std::env::var_os(key) };
            std::env::set_var(key, value.as_ref());
            guard
        }

        fn remove(key: &'static str) -> Self {
            let guard = Self { key, value: std::env::var_os(key) };
            std::env::remove_var(key);
            guard
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.value {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

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

        let _config_env = EnvVarGuard::set(BENCHMARK_CONFIG_ENV, config_path.as_os_str());
        let benchmark_path = benchmark_config_path(temp.path(), None);
        let workspace_path = benchmark_workspace_config_path(temp.path(), None);
        let publication_path = benchmark_publication_config_path(temp.path(), None);

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
        assert_eq!(actions.get("fastq.trim_reads"), Some(&"remove-legacy-duplicate".to_string()));
        assert_eq!(actions.get("fastq.filter_reads"), Some(&"move-legacy-entry".to_string()));
    }

    #[test]
    fn normalize_workspace_layout_report_converges_shared_and_archive_only_stage_ids() {
        let temp = tempfile::tempdir().expect("tempdir");
        let results_root = temp.path().join("archive");
        let cache_mirror_root = temp.path().join("mirror");
        let legacy_stage_root = results_root.join("benchmark_corpus").join("fastq.trim_reads");
        let canonical_stage_root =
            cache_mirror_root.join("results").join("benchmark_corpus").join("fastq.trim_reads");
        let archive_only_stage_root =
            results_root.join("benchmark_corpus").join("fastq.validate_reads");
        std::fs::create_dir_all(legacy_stage_root.join("cluster-apptainer")).expect("legacy stage");
        std::fs::create_dir_all(canonical_stage_root.join("cluster-apptainer"))
            .expect("canonical stage");
        std::fs::create_dir_all(archive_only_stage_root.join("cluster-apptainer"))
            .expect("archive stage");
        write_text(legacy_stage_root.join("cluster-apptainer/run_manifest.json"), "{}");
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
        assert_eq!(path, temp.path().join("configs/runtime/corpora/corpus-01.toml"));
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
                    spec_path: Some("configs/runtime/corpora/corpus-01.toml".to_string()),
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
        let _results_env = EnvVarGuard::set("BIJUX_TEST_RESULTS_ROOT", "/tmp/env-results");
        let config = load_benchmark_config(temp.path(), None).expect("load benchmark config");
        assert_eq!(
            config.workspace.local.and_then(|row| row.results_root),
            Some("/tmp/env-results".to_string())
        );
    }

    #[test]
    fn benchmark_config_treats_unset_placeholder_values_as_missing() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _corpus_env = EnvVarGuard::remove("BIJUX_TEST_CORPUS_ROOT");
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

        let _results_env = EnvVarGuard::set("BIJUX_TEST_RESULTS_ROOT", "/tmp/legacy-results");
        let _corpus_env = EnvVarGuard::set("BIJUX_TEST_CORPUS_ROOT", "/tmp/legacy-corpus");
        let config = load_benchmark_config(temp.path(), None).expect("load benchmark config");

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
        let _missing_env = EnvVarGuard::remove("BIJUX_MISSING_RESULTS_ROOT");
        let error = expand_env_placeholders("results_root = \"${BIJUX_MISSING_RESULTS_ROOT}\"\n")
            .expect_err("missing environment placeholder must fail");
        assert!(error
            .to_string()
            .contains("missing environment placeholder ${BIJUX_MISSING_RESULTS_ROOT}"));
    }
}
