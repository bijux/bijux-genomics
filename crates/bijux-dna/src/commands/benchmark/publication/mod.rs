#![allow(clippy::too_many_lines)]

mod corpus_dossier;
mod docs_audit;
mod docs_status;
mod dossier_index;
mod entrypoint;
mod models;
mod publication_io;
mod remediation;
mod report_audit;
mod results_status;
mod runtime_audit;

use crate::commands::benchmark_workspace::{
    benchmark_corpus_spec_path, benchmark_publication_contracts, benchmark_publication_exclusions,
    benchmark_runtime_corpus_dir_name, benchmark_stage_run_relative_root, load_benchmark_config,
    BenchmarkWorkspaceConfig, CorpusBenchmarkContract, CorpusBenchmarkExclusion,
};

use self::corpus_dossier::render_corpus_fastq_dossier;
use self::docs_audit::{audit_publication_docs, render_publication_docs_markdown};
use self::docs_status::{expected_counts_for_scope, write_corpus_fastq_docs_status};
#[cfg(test)]
use self::docs_status::{load_publication_corpus_spec, load_supplemental_findings};
use self::dossier_index::write_corpus_fastq_dossier_index;
#[cfg(test)]
use self::dossier_index::{build_dossier_stage_entry, resolve_existing_dossier_path};
#[cfg(test)]
use self::entrypoint::corpus_fastq_publication_command;
pub(crate) use self::entrypoint::{
    print_benchmark_publication_targets, run_corpus_fastq_publication_status,
    run_corpus_fastq_published_dossiers, run_corpus_fastq_report,
};
#[cfg(test)]
use self::models::StageAuditIssue;
#[cfg(test)]
use self::models::StageRunRootCandidate;
#[cfg(test)]
use self::models::{BenchmarkPublicationStatusReport, ExcludedStageEntry, PublicationStageReport};
#[cfg(test)]
use self::models::{PublishedResultsStageReport, PublishedResultsStatusReport, StageResultIssue};
#[cfg(test)]
use self::models::{
    RemediationIssue, RemediationIssueGroup, RemediationQueue, RemediationStageEntry,
};
use self::publication_io::{
    absolutize, classify_run_root_source, configured_stage_run_roots, csv_report_value,
    json_string_array, load_json_value, localize_results_path, observed_tools_from_report,
    publication_artifact_file_name, publication_method_file_name, publication_stage_docs_root,
    relative_to_docs_root, relative_to_repo_root, select_stage_run_root, sorted_json_string_array,
    sorted_strings, summary_corpus_id, unique_existing_run_roots, value_string,
    workspace_local_cache_mirror_root, workspace_local_results_root, workspace_remote_corpus_root,
    workspace_remote_results_root,
};
use self::remediation::write_corpus_fastq_remediation_queue;
#[cfg(test)]
use self::remediation::{build_remediation_queue, render_remediation_queue_markdown};
use self::report_audit::{
    append_stage_audit_issue, audit_publication_summary, audit_sample_results,
};
use self::results_status::write_corpus_fastq_results_status;
#[cfg(test)]
use self::results_status::{
    audit_published_results, audit_published_results_stage, render_published_results_markdown,
};
use self::runtime_audit::{
    audit_cohort_runtime_summary, audit_sample_runtime_outliers, audit_tool_runtime_summary,
};

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    fn validate_reads_contract() -> crate::commands::benchmark_workspace::CorpusBenchmarkContract {
        crate::commands::benchmark_workspace::CorpusBenchmarkContract {
            stage_id: "fastq.validate_reads".to_string(),
            scenario_id: "validation_fairness".to_string(),
            sample_scope: "full".to_string(),
            tools: vec![
                "fastqvalidator".to_string(),
                "fastqc".to_string(),
                "fastq_scan".to_string(),
                "fqtools".to_string(),
                "seqtk".to_string(),
            ],
        }
    }

    fn sample_workspace(
        cache_root: &Path,
        archive_root: &Path,
        remote_root: &Path,
        remote_corpus_root: &Path,
    ) -> crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
        crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some(archive_root.display().to_string()),
                cache_mirror_root: Some(cache_root.display().to_string()),
                extra_data_root: Some(cache_root.join("extra-data").display().to_string()),
                reference_root: Some(cache_root.join("reference").display().to_string()),
            }),
            remote: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                corpus_root: Some(remote_corpus_root.display().to_string()),
                results_root: Some(remote_root.join("results").display().to_string()),
                ..Default::default()
            }),
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(
                    crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                        remote_results_template: Some(
                            "{corpus_id}/{stage_id}/cluster-apptainer".to_string(),
                        ),
                        local_cache_results_template: Some(
                            "results/{corpus_id}/{stage_id}/cluster-apptainer".to_string(),
                        ),
                        local_archive_results_template: Some(
                            "{corpus_id}/{stage_id}/cluster-apptainer".to_string(),
                        ),
                    },
                ),
            }),
            artifacts: BTreeMap::new(),
            sync: None,
        }
    }

    fn write_benchmark_config_for_corpus(repo_root: &Path, corpus_id: &str) {
        let config_path = repo_root.join("configs/bench/benchmark.toml");
        let corpus_spec_path = format!("configs/runtime/corpora/{corpus_id}.toml");
        fs::create_dir_all(config_path.parent().expect("config dir")).expect("config dir");
        fs::write(
            config_path,
            format!("[corpora.\"{corpus_id}\"]\nspec_path = \"{corpus_spec_path}\"\n"),
        )
        .expect("write benchmark config");
    }

    #[allow(clippy::needless_pass_by_value)]
    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, format!("{}\n", serde_json::to_string_pretty(&value).expect("json")))
            .expect("write json");
    }

    #[test]
    fn publication_command_maps_profile_overrepresented_stage_report() {
        assert_eq!(
            super::corpus_fastq_publication_command(
                "fastq.profile_overrepresented_sequences",
                "corpus-01",
                "report",
                None,
            )
            .expect("report command"),
            "bijux-dna bench corpus-fastq-report --stage fastq.profile_overrepresented_sequences --corpus-id corpus-01"
        );
    }

    #[test]
    fn publication_command_maps_merge_pairs_stage_run() {
        assert_eq!(
            super::corpus_fastq_publication_command("fastq.merge_pairs", "corpus-01", "run", None)
                .expect("run command"),
            "bijux-dna bench corpus-fastq --corpus-id corpus-01 --stage fastq.merge_pairs"
        );
    }

    #[test]
    fn publication_command_includes_config_path() {
        assert_eq!(
            super::corpus_fastq_publication_command(
                "fastq.filter_reads",
                "corpus-01",
                "report",
                Some(Path::new("configs/bench/benchmark.toml")),
            )
            .expect("report command"),
            "bijux-dna bench corpus-fastq-report --stage fastq.filter_reads --corpus-id corpus-01 --config configs/bench/benchmark.toml"
        );
    }

    #[test]
    fn corpus_fastq_report_docs_root_tracks_stage_contract() {
        let docs_root =
            super::absolutize(Path::new("/repo"), Path::new("docs/30-operations/benchmark"))
                .join("fastq.validate_reads")
                .join("corpus-01");
        assert_eq!(
            docs_root,
            Path::new("/repo/docs/30-operations/benchmark/fastq.validate_reads/corpus-01")
        );
    }

    #[test]
    fn resolve_existing_dossier_path_uses_benchmark_markdown_contract() {
        let temp = tempdir().expect("tempdir");
        let stage_docs_root =
            temp.path().join("docs/30-operations/benchmark/fastq.validate_reads/corpus-01");
        fs::create_dir_all(&stage_docs_root).expect("stage docs root");
        fs::write(stage_docs_root.join("legacy-site.md"), "# legacy\n").expect("legacy dossier");

        assert_eq!(
            super::resolve_existing_dossier_path(&stage_docs_root),
            stage_docs_root.join("benchmark.md")
        );
    }

    #[test]
    fn run_corpus_fastq_report_writes_governed_dossier_without_python_scripts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let cache_root = repo_root.join("cache-mirror");
        let archive_root = repo_root.join("archive");
        let remote_root = repo_root.join("remote");
        let remote_corpus_root = repo_root.join("benchmark_corpus");
        let config_path = repo_root.join("configs/bench/benchmark.toml");
        let corpus_spec_path = repo_root.join("configs/runtime/corpora/corpus-01.toml");
        fs::create_dir_all(config_path.parent().expect("config dir")).expect("config dir");
        fs::create_dir_all(corpus_spec_path.parent().expect("corpus spec dir"))
            .expect("corpus spec dir");
        fs::create_dir_all(remote_corpus_root.join("raw/DRR000001")).expect("raw dir");
        fs::create_dir_all(remote_corpus_root.join("normalized")).expect("normalized dir");
        fs::create_dir_all(cache_root.join("results")).expect("cache results dir");
        fs::create_dir_all(&archive_root).expect("archive dir");
        fs::create_dir_all(remote_root.join("results")).expect("remote results dir");

        fs::write(
            &config_path,
            format!(
                r#"[workspace.local]
results_root = "{}"
cache_mirror_root = "{}"
extra_data_root = "{}"
reference_root = "{}"

[workspace.remote]
corpus_root = "{}"
results_root = "{}"

[corpora."corpus-01"]
spec_path = "configs/runtime/corpora/corpus-01.toml"

[[publication.corpus_01.contracts]]
stage_id = "fastq.validate_reads"
scenario_id = "validation_fairness"
sample_scope = "full"
tools = ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"]
"#,
                archive_root.display(),
                cache_root.display(),
                cache_root.join("extra-data").display(),
                cache_root.join("reference").display(),
                remote_corpus_root.display(),
                remote_root.join("results").display(),
            ),
        )
        .expect("write benchmark config");
        fs::write(
            &corpus_spec_path,
            r#"schema_version = "bijux.corpus_spec.v1"
corpus_id = "corpus-01"
target_ancient_se = 0
target_ancient_pe = 0
target_modern_se = 1
target_modern_pe = 0

[[samples]]
accession = "DRR000001"
study_accession = "PRJ000001"
era = "modern"
layout = "se"
size_band = "under_100mb"
reason = "Compact validation fixture."
"#,
        )
        .expect("write corpus spec");

        let raw_fastq = remote_corpus_root.join("raw/DRR000001/reads.fastq.gz");
        let normalized_fastq = remote_corpus_root.join("normalized/sample_0001_R1.fastq.gz");
        fs::write(&raw_fastq, b"raw-fastq\n").expect("raw fastq");
        fs::write(&normalized_fastq, b"raw-fastq\n").expect("normalized fastq");
        write_json(
            &remote_corpus_root.join("MANIFEST.json"),
            serde_json::json!({
                "files": {
                    "raw/DRR000001/reads.fastq.gz": "sha256:fixture",
                    "normalized/sample_0001_R1.fastq.gz": "sha256:fixture"
                }
            }),
        );

        let run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report =
            run_root.join("bench").join("validate_reads").join("sample_0001").join("report.json");
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}, "execution": {"runtime_s": 1.2, "exit_code": 0}},
                    {"context": {"tool": "fastqc"}, "execution": {"runtime_s": 2.3, "exit_code": 0}},
                    {"context": {"tool": "fastq_scan"}, "execution": {"runtime_s": 0.9, "exit_code": 0}},
                    {"context": {"tool": "fqtools"}, "execution": {"runtime_s": 1.0, "exit_code": 0}},
                    {"context": {"tool": "seqtk"}, "execution": {"runtime_s": 1.1, "exit_code": 0}}
                ]
            }),
        );
        write_json(
            &run_root.join("run_manifest.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "platform": "cluster-apptainer",
                "samples_failed": 0,
                "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                "runs": [{
                    "sample_id": "sample_0001",
                    "report_json": sample_report
                }]
            }),
        );

        super::run_corpus_fastq_report(
            repo_root,
            &crate::commands::cli::BenchCorpusFastqReportArgs {
                stage: "fastq.validate_reads".to_string(),
                corpus_id: "corpus-01".to_string(),
                config: Some(PathBuf::from("configs/bench/benchmark.toml")),
                docs_root: PathBuf::from("docs/30-operations/benchmark"),
                run_root: Some(run_root.clone()),
            },
        )
        .expect("render dossier");

        let stage_docs_root = repo_root
            .join("docs/30-operations/benchmark")
            .join("fastq.validate_reads")
            .join("corpus-01");
        let summary = fs::read_to_string(stage_docs_root.join("summary.json")).expect("summary");
        let benchmark_md =
            fs::read_to_string(stage_docs_root.join("benchmark.md")).expect("benchmark md");
        assert!(summary.contains("\"stage_id\": \"fastq.validate_reads\""));
        assert!(summary.contains("\"samples_total\": 1"));
        assert!(benchmark_md.contains("generated directly by `bijux-dna`"));
        assert!(stage_docs_root.join("tool_runtime_summary.csv").is_file());
        assert!(stage_docs_root.join("cohort_runtime_summary.csv").is_file());
        assert!(stage_docs_root.join("sample_runtime_outliers.csv").is_file());
    }

    #[test]
    fn classify_run_root_source_prefers_local_cache_mirror() {
        assert_eq!(
            super::classify_run_root_source(
                Path::new(
                    "/bench/local/cache-mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/remote/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/local/cache-mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new(
                    "/bench/local/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                ),
                Path::new("/bench/remote/corpus/benchmark_corpus"),
            ),
            "local-cache-mirror"
        );
    }

    #[test]
    fn localize_results_path_does_not_translate_legacy_results_aliases() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/results".to_string()),
                cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                extra_data_root: None,
                reference_root: None,
            }),
            ..Default::default()
        };

        let localized = super::localize_results_path(
            "/bench/local/cache-mirror/bijux-dna-results/corpus_01/fastq.validate_reads/cluster-apptainer/run_manifest.json",
            Path::new("/bench/local/results"),
            &workspace,
        );

        assert_eq!(
            localized,
            PathBuf::from(
                "/bench/local/cache-mirror/bijux-dna-results/corpus_01/fastq.validate_reads/cluster-apptainer/run_manifest.json"
            )
        );
    }

    #[test]
    fn stage_run_relative_root_uses_workspace_local_cache_template() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(
                    crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                        local_cache_results_template: Some(
                            "results/{corpus_id}/{stage_id}/cluster".to_string(),
                        ),
                        ..Default::default()
                    },
                ),
            }),
            ..Default::default()
        };
        assert_eq!(
            crate::commands::benchmark_workspace::benchmark_stage_run_relative_root(
                &workspace,
                "local-cache",
                "benchmark_corpus",
                "fastq.validate_reads",
            )
            .expect("relative root"),
            Path::new("results/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn configured_stage_run_roots_only_publish_local_mirrors() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig {
            local: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                results_root: Some("/bench/local/archive".to_string()),
                cache_mirror_root: Some("/bench/local/cache-mirror".to_string()),
                extra_data_root: Some("/bench/local/extra-data".to_string()),
                reference_root: None,
            }),
            remote: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                results_root: Some("/bench/remote/results".to_string()),
                ..Default::default()
            }),
            layout: Some(crate::commands::benchmark_workspace::BenchmarkWorkspaceLayout {
                stage_runs: Some(
                    crate::commands::benchmark_workspace::BenchmarkWorkspaceStageRuns {
                        local_cache_results_template: Some(
                            "results/{corpus_id}/{stage_id}/cluster".to_string(),
                        ),
                        local_archive_results_template: Some(
                            "{corpus_id}/{stage_id}/cluster".to_string(),
                        ),
                        remote_results_template: Some(
                            "{corpus_id}/{stage_id}/remote-cluster".to_string(),
                        ),
                    },
                ),
            }),
            ..Default::default()
        };

        let roots = super::configured_stage_run_roots(
            &workspace,
            "benchmark_corpus",
            "fastq.validate_reads",
        )
        .expect("stage roots");
        assert_eq!(roots.len(), 2);
        assert_eq!(
            roots[0].path,
            PathBuf::from(
                "/bench/local/cache-mirror/results/benchmark_corpus/fastq.validate_reads/cluster"
            )
        );
        assert_eq!(
            roots[1].path,
            PathBuf::from("/bench/local/archive/benchmark_corpus/fastq.validate_reads/cluster")
        );
    }

    #[test]
    fn select_stage_run_root_requires_existing_mirrors() {
        let roots = vec![
            super::StageRunRootCandidate {
                path: PathBuf::from(
                    "/bench/local/cache-mirror/results/corpus_01/fastq.validate_reads/cluster-apptainer",
                ),
            },
            super::StageRunRootCandidate {
                path: PathBuf::from(
                    "/bench/local/archive/corpus_01/fastq.validate_reads/cluster-apptainer",
                ),
            },
        ];

        let selection = super::select_stage_run_root(&roots);

        assert!(selection.selected_path.as_os_str().is_empty());
        assert!(selection.newest_available_path.is_none());
    }

    #[test]
    fn dossier_stage_entry_uses_workspace_run_root_templates() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = remote_root.join("corpus_01");
        let workspace =
            sample_workspace(&cache_root, &archive_root, &remote_root, &remote_corpus_root);

        let entry = super::build_dossier_stage_entry(
            temp.path(),
            &docs_root,
            &workspace,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("dossier entry");

        assert_eq!(
            entry.expected_remote_run_root,
            remote_root
                .join("results")
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
        assert_eq!(
            entry.expected_local_cache_mirror_run_root,
            cache_root
                .join("results")
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
        assert_eq!(
            entry.expected_local_results_run_root,
            archive_root
                .join("corpus_01")
                .join("fastq.validate_reads")
                .join("cluster-apptainer")
                .display()
                .to_string()
        );
    }

    #[test]
    fn results_audit_tracks_missing_published_stage_summary() {
        let temp = tempdir().expect("tempdir");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace =
            sample_workspace(&cache_root, &archive_root, &remote_root, &remote_corpus_root);
        let report = super::audit_published_results(
            temp.path(),
            &workspace,
            &temp.path().join("docs").join("benchmark"),
            "corpus-01",
            &[validate_reads_contract()],
        )
        .expect("results audit");
        assert_eq!(report.applicable_stage_count, 1);
        assert!(report
            .stages
            .iter()
            .flat_map(|stage| stage.issues.iter())
            .any(|issue| issue.issue_id == "missing-published-summary"));
    }

    #[test]
    fn results_audit_requires_summary_corpus_root() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace =
            sample_workspace(&cache_root, &archive_root, &remote_root, &remote_corpus_root);
        write_json(
            &docs_root.join("fastq.validate_reads").join("corpus-01").join("summary.json"),
            serde_json::json!({
                "run_root": cache_root.join("results"),
            }),
        );

        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report.issues.iter().any(|issue| issue.issue_id == "missing-summary-corpus-root"));
    }

    #[test]
    fn results_audit_missing_local_run_root_reports_expected_mirror() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace =
            sample_workspace(&cache_root, &archive_root, &remote_root, &remote_corpus_root);
        let reported_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        write_json(
            &docs_root.join("fastq.validate_reads").join("corpus-01").join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": reported_run_root,
            }),
        );
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        let missing_issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_id == "missing-local-run-root")
            .expect("missing issue");
        assert!(missing_issue.detail.contains(&reported_run_root.display().to_string()));
        assert!(missing_issue.detail.contains("expected_local_mirror="));
        assert_eq!(report.reported_run_root, reported_run_root.display().to_string());
        assert!(report.available_run_roots.is_empty());
    }

    #[test]
    fn results_audit_flags_duplicate_local_run_roots() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace =
            sample_workspace(&cache_root, &archive_root, &remote_root, &remote_corpus_root);
        let canonical_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let legacy_run_root = archive_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root.join("fastq.validate_reads").join("corpus-01").join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": canonical_run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        for run_root in [&canonical_run_root, &legacy_run_root] {
            write_json(
                &run_root.join("run_manifest.json"),
                serde_json::json!({
                    "stage_id": "fastq.validate_reads",
                    "scenario_id": "validation_fairness",
                    "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                    "dry_run": false,
                    "sample_limit": serde_json::Value::Null,
                    "samples_failed": 0,
                    "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
                }),
            );
        }
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "duplicate-result-root-ambiguity"));
    }

    #[test]
    fn results_audit_flags_newer_available_duplicate_run_root() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("benchmark_corpus");
        let workspace =
            sample_workspace(&cache_root, &archive_root, &remote_root, &remote_corpus_root);
        let canonical_run_root = cache_root
            .join("results")
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let legacy_run_root = archive_root
            .join("benchmark_corpus")
            .join("fastq.validate_reads")
            .join("cluster-apptainer");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root.join("fastq.validate_reads").join("corpus-01").join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": canonical_run_root,
            }),
        );
        write_json(
            &sample_report,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"tool": "fastqc"}},
                    {"context": {"tool": "fastq_scan"}},
                    {"context": {"tool": "fqtools"}},
                    {"context": {"tool": "seqtk"}},
                ],
            }),
        );
        for (run_root, generated_at_utc) in [
            (&canonical_run_root, "2026-03-28T00:00:00Z"),
            (&legacy_run_root, "2026-03-29T00:00:00Z"),
        ] {
            write_json(
                &run_root.join("run_manifest.json"),
                serde_json::json!({
                    "stage_id": "fastq.validate_reads",
                    "scenario_id": "validation_fairness",
                    "generated_at_utc": generated_at_utc,
                    "tools": ["fastqvalidator", "fastqc", "fastq_scan", "fqtools", "seqtk"],
                    "dry_run": false,
                    "sample_limit": serde_json::Value::Null,
                    "samples_failed": 0,
                    "runs": [{"sample_id": "sample_0001", "report_json": sample_report}],
                }),
            );
        }
        let report = super::audit_published_results_stage(
            temp.path(),
            &workspace,
            &docs_root,
            "corpus-01",
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert_eq!(report.selected_run_root, canonical_run_root.display().to_string());
        assert_eq!(report.newest_available_run_root, legacy_run_root.display().to_string());
        assert!(!report.selected_run_root_is_newest);
        assert!(report.issues.iter().any(|issue| issue.issue_id == "newer-run-root-available"));
    }

    #[test]
    fn results_audit_markdown_lists_selected_and_available_run_roots() {
        let rendered =
            super::render_published_results_markdown(&super::PublishedResultsStatusReport {
                corpus_id: "corpus-01".to_string(),
                applicable_stage_count: 1,
                published_stage_count: 1,
                complete_stage_count: 0,
                incomplete_stage_count: 1,
                issue_count: 1,
                stages: vec![super::PublishedResultsStageReport {
                    stage_id: "fastq.validate_reads".to_string(),
                    status: "incomplete".to_string(),
                    issue_count: 1,
                    reported_run_root:
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                    selected_run_root:
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                    newest_available_run_root:
                        "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                    selected_run_root_is_newest: false,
                    available_run_roots: vec![
                        "/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                        "/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer"
                            .to_string(),
                    ],
                    issues: vec![super::StageResultIssue {
                        stage_id: "fastq.validate_reads".to_string(),
                        issue_id: "missing-local-run-root".to_string(),
                        detail: "missing local mirror".to_string(),
                    }],
                }],
            });
        assert!(rendered.contains("selected run root"));
        assert!(rendered
            .contains("/mirror/results/benchmark_corpus/fastq.validate_reads/cluster-apptainer"));
        assert!(
            rendered.contains("/archive/benchmark_corpus/fastq.validate_reads/cluster-apptainer")
        );
    }

    #[test]
    fn observed_tools_from_report_collects_nested_tool_literals() {
        let temp = tempdir().expect("tempdir");
        let report_path = temp.path().join("report.json");
        write_json(
            &report_path,
            serde_json::json!({
                "records": [
                    {"context": {"tool": "fastqvalidator"}},
                    {"context": {"parameters": {"tool": "seqtk"}}},
                    {"context": {"tool": "fastqvalidator"}},
                ],
            }),
        );
        let observed_tools = super::observed_tools_from_report(&report_path).expect("tools");
        assert_eq!(observed_tools, vec!["fastqvalidator", "seqtk"]);
    }

    #[test]
    fn publication_docs_report_missing_stage_artifacts() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 1\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 1\n",
            ),
        )
        .expect("write corpus spec");
        write_benchmark_config_for_corpus(repo_root, "corpus-01");
        let stage_root = docs_root.join("fastq.validate_reads");
        let corpus_root = stage_root.join("corpus-01");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        write_json(
            &corpus_root.join("summary.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
            }),
        );
        fs::write(corpus_root.join("sample_results.csv"), "sample_id,tool\n").expect("sample csv");
        let supplemental = BTreeMap::from([("fastq.validate_reads".to_string(), Vec::new())]);
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[validate_reads_contract()],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &supplemental,
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert_eq!(validate_report.status, "incomplete");
        assert!(validate_report.issue_count >= 4);
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "missing-benchmark-md"));
    }

    #[test]
    fn publication_docs_markdown_summarizes_completion_and_issue_count() {
        let markdown =
            super::render_publication_docs_markdown(&super::BenchmarkPublicationStatusReport {
                corpus_id: "corpus-01".to_string(),
                docs_root: "/bench/docs/30-operations/benchmark".to_string(),
                benchmarkable_stage_count: 3,
                applicable_stage_count: 2,
                completed_stage_count: 1,
                incomplete_stage_count: 1,
                excluded_stage_count: 1,
                issue_count: 3,
                audit_warning_count: 0,
                audit_warnings: Vec::new(),
                supplemental_findings_generated_at_utc: None,
                excluded_stages: vec![super::ExcludedStageEntry {
                    stage_id: "fastq.index_reference".to_string(),
                    reason: "reference bundle benchmark".to_string(),
                }],
                stages: vec![
                    super::PublicationStageReport {
                        stage_id: "fastq.validate_reads".to_string(),
                        scenario_id: "validation_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        contract_tool_roster: vec!["fastqvalidator".to_string()],
                        expected_tool_roster: vec!["fastqvalidator".to_string()],
                        method_path: "benchmark/fastq.validate_reads/corpus-01-method.md"
                            .to_string(),
                        corpus_path: "benchmark/fastq.validate_reads/corpus-01".to_string(),
                        status: "complete".to_string(),
                        issue_count: 0,
                        results_status: "complete".to_string(),
                        results_issue_count: 0,
                        results_selected_run_root:
                            "/bench/results/fastq.validate_reads/cluster-apptainer".to_string(),
                        results_newest_available_run_root:
                            "/bench/results/fastq.validate_reads/cluster-apptainer".to_string(),
                        results_selected_run_root_is_newest: true,
                        issues: Vec::new(),
                    },
                    super::PublicationStageReport {
                        stage_id: "fastq.trim_reads".to_string(),
                        scenario_id: "trim_fairness".to_string(),
                        sample_scope: "full".to_string(),
                        contract_tool_roster: vec!["fastp".to_string()],
                        expected_tool_roster: vec!["fastp".to_string()],
                        method_path: "benchmark/fastq.trim_reads/corpus-01-method.md".to_string(),
                        corpus_path: "benchmark/fastq.trim_reads/corpus-01".to_string(),
                        status: "incomplete".to_string(),
                        issue_count: 3,
                        results_status: "incomplete".to_string(),
                        results_issue_count: 2,
                        results_selected_run_root:
                            "/bench/results/fastq.trim_reads/cluster-apptainer".to_string(),
                        results_newest_available_run_root:
                            "/bench/archive/fastq.trim_reads/cluster-apptainer".to_string(),
                        results_selected_run_root_is_newest: false,
                        issues: vec![super::StageAuditIssue {
                            stage_id: "fastq.trim_reads".to_string(),
                            issue_id: "missing-corpus-dir".to_string(),
                            severity: "error".to_string(),
                            detail:
                                "missing docs/30-operations/benchmark/fastq.trim_reads/corpus-01"
                                    .to_string(),
                        }],
                    },
                ],
            });
        assert!(markdown.contains("Benchmarkable governed stages: `3`"));
        assert!(markdown.contains("Completed stage dossiers: `1`"));
        assert!(markdown.contains("Publication issues: `3`"));
        assert!(markdown.contains(
            "`fastq.trim_reads`: `incomplete` (`3` publication issues, results `incomplete`, scope `full`)"
        ));
        assert!(markdown.contains(
            "selected mirrored run root: `/bench/results/fastq.trim_reads/cluster-apptainer`"
        ));
        assert!(markdown.contains(
            "newest mirrored run root: `/bench/archive/fastq.trim_reads/cluster-apptainer` (selected newest=`false`)"
        ));
        assert!(markdown.contains("mirrored result issues: `2`"));
        assert!(markdown.contains("`fastq.index_reference`: reference bundle benchmark"));
    }

    #[test]
    fn publication_docs_append_supplemental_findings() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 0\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 0\n",
            ),
        )
        .expect("write corpus spec");
        write_benchmark_config_for_corpus(repo_root, "corpus-01");
        let stage_root = docs_root.join("fastq.validate_reads");
        fs::create_dir_all(stage_root.join("corpus-01")).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        let mut supplemental = BTreeMap::new();
        supplemental.insert(
            "fastq.validate_reads".to_string(),
            vec![super::StageAuditIssue {
                stage_id: "fastq.validate_reads".to_string(),
                issue_id: "fixture-integrity-gap".to_string(),
                severity: "error".to_string(),
                detail: "synthetic fixture does not represent a publishable benchmark lineage"
                    .to_string(),
            }],
        );
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                sample_scope: "paired".to_string(),
                ..validate_reads_contract()
            }],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &supplemental,
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "fixture-integrity-gap"));
    }

    #[test]
    fn load_supplemental_findings_warns_when_freshness_missing() {
        let temp = tempdir().expect("tempdir");
        let findings_path = temp.path().join("findings.json");
        write_json(
            &findings_path,
            serde_json::json!({
                "findings": [{
                    "stage_id": "fastq.validate_reads",
                    "issue_id": "fixture-gap",
                    "detail": "fixture gap",
                }],
            }),
        );
        let (findings, warnings, generated_at_utc) =
            super::load_supplemental_findings(&findings_path).expect("findings");
        assert!(findings.contains_key("fastq.validate_reads"));
        assert_eq!(generated_at_utc, None);
        assert!(warnings.iter().any(|warning| warning.contains("generated_at_utc")));
    }

    #[test]
    fn publication_docs_reject_missing_tool_coverage_in_sample_results() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();
        let docs_root = repo_root.join("docs").join("benchmark");
        fs::create_dir_all(repo_root.join("configs/runtime/corpora")).expect("corpora dir");
        fs::write(
            repo_root.join("configs/runtime/corpora/corpus-01.toml"),
            concat!(
                "corpus_id = \"corpus-01\"\n",
                "target_ancient_se = 1\n",
                "target_ancient_pe = 1\n",
                "target_modern_se = 1\n",
                "target_modern_pe = 1\n",
            ),
        )
        .expect("write corpus spec");
        write_benchmark_config_for_corpus(repo_root, "corpus-01");
        let stage_root = docs_root.join("fastq.validate_reads");
        let corpus_root = stage_root.join("corpus-01");
        fs::create_dir_all(&corpus_root).expect("corpus dir");
        fs::write(stage_root.join("corpus-01-method.md"), "# method\n").expect("method");
        write_json(
            &corpus_root.join("summary.json"),
            serde_json::json!({
                "stage_id": "fastq.validate_reads",
                "scenario_id": "validation_fairness",
                "tools": ["fastqvalidator", "seqtk"],
                "samples_total": 4,
                "samples_failed": 0,
                "cohort_counts": {
                    "ancient_pe": 1,
                    "ancient_se": 1,
                    "modern_pe": 1,
                    "modern_se": 1,
                },
                "tool_summary": [
                    {"tool": "fastqvalidator"},
                    {"tool": "seqtk"},
                ],
            }),
        );
        fs::write(
            corpus_root.join("sample_results.csv"),
            concat!(
                "sample_id,accession,era,layout,study_accession,size_band,tool\n",
                "sample_0001,ACC1,ancient,se,PRJ1,under_100mb,fastqvalidator\n",
                "sample_0002,ACC2,ancient,pe,PRJ2,under_100mb,fastqvalidator\n",
                "sample_0003,ACC3,modern,se,PRJ3,under_500mb,fastqvalidator\n",
                "sample_0004,ACC4,modern,pe,PRJ4,under_500mb,fastqvalidator\n",
            ),
        )
        .expect("sample csv");
        fs::write(corpus_root.join("tool_runtime_summary.csv"), "tool\nfastqvalidator\nseqtk\n")
            .expect("tool summary");
        fs::write(
            corpus_root.join("cohort_runtime_summary.csv"),
            "cohort\nancient_pe\nancient_se\nmodern_pe\nmodern_se\n",
        )
        .expect("cohort summary");
        fs::write(
            corpus_root.join("sample_runtime_outliers.csv"),
            "sample_id\nsample_0001\nsample_0002\nsample_0003\nsample_0004\n",
        )
        .expect("outliers");
        fs::write(corpus_root.join("benchmark.md"), "# dossier\n").expect("dossier");
        let supplemental = BTreeMap::from([("fastq.validate_reads".to_string(), Vec::new())]);
        let report = super::audit_publication_docs(
            repo_root,
            &docs_root,
            "corpus-01",
            &[crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                stage_id: "fastq.validate_reads".to_string(),
                scenario_id: "validation_fairness".to_string(),
                sample_scope: "full".to_string(),
                tools: vec!["fastqvalidator".to_string(), "seqtk".to_string()],
            }],
            &[],
            &super::load_publication_corpus_spec(repo_root, None, "corpus-01")
                .expect("corpus spec"),
            &supplemental,
            &BTreeMap::new(),
            &[],
            None,
        )
        .expect("publication report");
        let validate_report = report.stages.first().expect("stage");
        assert!(validate_report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "sample-results-tool-coverage-drift"));
    }

    #[test]
    fn remediation_queue_merges_publication_results_and_findings() {
        let queue = super::build_remediation_queue(
            "corpus-01",
            &[crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                stage_id: "fastq.validate_reads".to_string(),
                scenario_id: "governed-fixture".to_string(),
                sample_scope: "paired-subset".to_string(),
                tools: Vec::new(),
            }],
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-benchmark-md",
                        "severity": "error",
                        "detail": "missing docs dossier",
                    }],
                }],
            }),
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-local-run-root",
                        "severity": "error",
                        "detail": "missing local mirror root",
                    }],
                }],
            }),
            &serde_json::json!({
                "findings": [{
                    "stage_id": "fastq.validate_reads",
                    "issue_id": "publication-gap",
                    "detail": "supplemental finding",
                    "severity": "error",
                }],
            }),
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "generated_at_utc": "2026-03-28T00:00:00Z",
                    "run_root_source": "local-results-root",
                }],
            }),
        )
        .expect("remediation queue");

        let stage = queue.stages.first().expect("stage");
        assert_eq!(stage.stage_id, "fastq.validate_reads");
        assert_eq!(stage.status, "open");
        assert_eq!(stage.issue_count, 3);
        assert_eq!(stage.recommended_action, "sync-or-normalize-results");
        assert_eq!(stage.published_generated_at_utc.as_deref(), Some("2026-03-28T00:00:00Z"));
        assert_eq!(stage.run_root_source.as_deref(), Some("local-results-root"));
    }

    #[test]
    fn remediation_queue_markdown_uses_issue_groups() {
        let rendered = super::render_remediation_queue_markdown(&super::RemediationQueue {
            corpus_id: "corpus-01".to_string(),
            stage_count: 1,
            open_stage_count: 1,
            clear_stage_count: 0,
            stages: vec![super::RemediationStageEntry {
                stage_id: "fastq.validate_reads".to_string(),
                owner: "benchmark-governance".to_string(),
                status: "open".to_string(),
                issue_count: 2,
                issue_group_count: 1,
                recommended_action: "sync-or-normalize-results".to_string(),
                publication_status: "incomplete".to_string(),
                results_status: "incomplete".to_string(),
                sample_scope: "paired-subset".to_string(),
                published_generated_at_utc: Some("2026-03-28T00:00:00Z".to_string()),
                run_root_source: Some("local-cache-mirror".to_string()),
                issue_groups: vec![super::RemediationIssueGroup {
                    issue_id: "missing-localized-report-json".to_string(),
                    count: 2,
                    sources: vec!["results".to_string()],
                    severity: "error".to_string(),
                    example_details: vec![
                        "sample_0001 missing report.json".to_string(),
                        "sample_0002 missing report.json".to_string(),
                    ],
                    additional_detail_count: 0,
                }],
                issues: vec![
                    super::RemediationIssue {
                        issue_id: "missing-localized-report-json".to_string(),
                        detail: "sample_0001 missing report.json".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                    super::RemediationIssue {
                        issue_id: "missing-localized-report-json".to_string(),
                        detail: "sample_0002 missing report.json".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                ],
            }],
        });

        assert!(rendered.contains("issue group `missing-localized-report-json` x2"));
        assert!(rendered.contains("sample_0001 missing report.json"));
    }
}
