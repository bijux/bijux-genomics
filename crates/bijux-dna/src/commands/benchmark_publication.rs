use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark_workspace::{
    benchmark_config_path, load_benchmark_config, load_benchmark_publication_config,
    write_workspace_layout_status, BenchmarkWorkspaceConfig, CorpusBenchmarkContract,
    BENCHMARK_CONFIG_ENV,
};
use crate::commands::cli::{
    BenchCorpusFastqPublicationStatusArgs, BenchCorpusFastqPublishedDossiersArgs,
    BenchCorpusFastqReportArgs, BenchPublicationTargetsArgs,
};

pub(crate) fn print_benchmark_publication_targets(
    cwd: &Path,
    args: &BenchPublicationTargetsArgs,
) -> Result<()> {
    let publication = load_benchmark_publication_config(cwd, args.config.as_deref())?;
    let Some(corpus_01) = publication.corpus_01 else {
        println!();
        return Ok(());
    };
    let targets = corpus_01
        .contracts
        .into_iter()
        .map(|contract| benchmark_make_target(&contract.stage_id, &args.kind))
        .collect::<Vec<_>>();
    println!("{}", targets.join(" "));
    Ok(())
}

pub(crate) fn run_corpus_fastq_publication_status(
    cwd: &Path,
    args: &BenchCorpusFastqPublicationStatusArgs,
) -> Result<()> {
    let config_path = benchmark_config_path(cwd, args.config.as_deref());
    let docs_root = absolutize(cwd, &args.docs_root);
    write_corpus_fastq_dossier_index(cwd, args.config.as_deref(), &docs_root)?;
    write_workspace_layout_status(cwd, args.config.as_deref(), &docs_root)?;
    for spec in publication_status_steps(cwd, &docs_root) {
        run_subprocess(cwd, &config_path, &spec)?;
    }
    Ok(())
}

pub(crate) fn run_corpus_fastq_published_dossiers(
    cwd: &Path,
    args: &BenchCorpusFastqPublishedDossiersArgs,
) -> Result<()> {
    let publication = load_benchmark_publication_config(cwd, args.config.as_deref())?;
    if let Some(corpus_01) = publication.corpus_01 {
        for contract in corpus_01.contracts {
            run_corpus_fastq_report(
                cwd,
                &BenchCorpusFastqReportArgs {
                    stage: contract.stage_id,
                    config: args.config.clone(),
                    docs_root: args.docs_root.clone(),
                    run_root: args.run_root.clone(),
                },
            )?;
        }
    }
    run_corpus_fastq_publication_status(
        cwd,
        &BenchCorpusFastqPublicationStatusArgs {
            config: args.config.clone(),
            docs_root: args.docs_root.clone(),
        },
    )?;
    Ok(())
}

pub(crate) fn run_corpus_fastq_report(cwd: &Path, args: &BenchCorpusFastqReportArgs) -> Result<()> {
    let config_path = benchmark_config_path(cwd, args.config.as_deref());
    let stage_docs_root = absolutize(cwd, &args.docs_root)
        .join(&args.stage)
        .join("corpus-01");
    let report_spec = corpus_fastq_stage_render_step(
        cwd,
        &args.stage,
        &stage_docs_root,
        args.run_root.as_deref(),
    )?;
    run_subprocess(cwd, &config_path, &report_spec)?;
    let briefing_spec = corpus_fastq_stage_briefing_step(cwd, &args.stage, &stage_docs_root)?;
    run_subprocess(cwd, &config_path, &briefing_spec)?;
    Ok(())
}

fn benchmark_make_target(stage_id: &str, kind: &str) -> String {
    let stage_suffix = match stage_id {
        "fastq.validate_reads" => "validate",
        "fastq.detect_adapters" => "detect-adapters",
        "fastq.profile_reads" => "profile-reads",
        "fastq.profile_read_lengths" => "profile-read-lengths",
        "fastq.profile_overrepresented_sequences" => "profile-overrepresented",
        "fastq.normalize_primers" => "normalize-primers",
        "fastq.trim_polyg_tails" => "trim-polyg",
        "fastq.trim_reads" => "trim-reads",
        "fastq.filter_reads" => "filter-reads",
        "fastq.filter_low_complexity" => "filter-low-complexity",
        "fastq.deplete_rrna" => "deplete-rrna",
        "fastq.merge_pairs" => "merge",
        "fastq.remove_duplicates" => "remove-duplicates",
        "fastq.deplete_host" => "deplete-host",
        "fastq.deplete_reference_contaminants" => "deplete-reference-contaminants",
        "fastq.correct_errors" => "correct-errors",
        "fastq.extract_umis" => "extract-umis",
        "fastq.screen_taxonomy" => "screen-taxonomy",
        "fastq.trim_terminal_damage" => "trim-terminal-damage",
        "fastq.report_qc" => "report-qc",
        other => panic!("unsupported corpus benchmark publication stage: {other}"),
    };
    match kind {
        "run" => format!("_benchmark-{stage_suffix}-corpus-01"),
        "report" => format!("_benchmark-{stage_suffix}-corpus-01-report"),
        other => panic!("unsupported benchmark publication target kind: {other}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubprocessSpec {
    program: &'static str,
    args: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DossierIndex {
    corpus_id: String,
    stage_count: usize,
    published_stage_count: usize,
    missing_stage_count: usize,
    stages: Vec<DossierStageEntry>,
}

#[derive(Debug, Serialize)]
struct DossierStageEntry {
    stage_id: String,
    sample_scope: String,
    status: String,
    summary_path: String,
    dossier_path: String,
    expected_remote_run_root: String,
    expected_remote_legacy_run_root: String,
    expected_local_cache_mirror_run_root: String,
    expected_local_results_run_root: String,
    generated_at_utc: Option<String>,
    platform: Option<String>,
    corpus_root: Option<String>,
    run_root: Option<String>,
    run_root_source: Option<String>,
}

fn publication_status_steps(repo_root: &Path, docs_root: &Path) -> Vec<SubprocessSpec> {
    let repo_root_string = repo_root.display().to_string();
    let docs_root_string = docs_root.display().to_string();
    vec![
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/benchmark_tooling_repo_checks.py")
                    .display()
                    .to_string(),
                "--repo-root".to_string(),
                repo_root_string.clone(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/audit_corpus_01_fastq_benchmark_docs.py")
                    .display()
                    .to_string(),
                "--repo-root".to_string(),
                repo_root_string.clone(),
                "--docs-root".to_string(),
                docs_root_string.clone(),
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-status.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root.join("corpus-01-status.md").display().to_string(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/audit_published_corpus_01_fastq_results.py")
                    .display()
                    .to_string(),
                "--repo-root".to_string(),
                repo_root_string,
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-results-status.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root
                    .join("corpus-01-results-status.md")
                    .display()
                    .to_string(),
            ],
        },
        SubprocessSpec {
            program: "python3",
            args: vec![
                repo_root
                    .join("makes/bin/build_corpus_01_benchmark_remediation_queue.py")
                    .display()
                    .to_string(),
                "--status-json".to_string(),
                docs_root
                    .join("corpus-01-status.json")
                    .display()
                    .to_string(),
                "--results-json".to_string(),
                docs_root
                    .join("corpus-01-results-status.json")
                    .display()
                    .to_string(),
                "--findings-json".to_string(),
                docs_root
                    .join("corpus-01-publication-findings.json")
                    .display()
                    .to_string(),
                "--dossier-index-json".to_string(),
                docs_root
                    .join("corpus-01-dossier-index.json")
                    .display()
                    .to_string(),
                "--json-out".to_string(),
                docs_root
                    .join("corpus-01-remediation-queue.json")
                    .display()
                    .to_string(),
                "--markdown-out".to_string(),
                docs_root
                    .join("corpus-01-remediation-queue.md")
                    .display()
                    .to_string(),
            ],
        },
    ]
}

fn write_corpus_fastq_dossier_index(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    let workspace = &config.workspace;
    let contracts = config
        .publication
        .corpus_01
        .map(|row| row.contracts)
        .unwrap_or_default();

    let stages = contracts
        .iter()
        .map(|contract| build_dossier_stage_entry(docs_root, workspace, contract))
        .collect::<Result<Vec<_>>>()?;
    let index = DossierIndex {
        corpus_id: "corpus-01".to_string(),
        stage_count: stages.len(),
        published_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "published")
            .count(),
        missing_stage_count: stages
            .iter()
            .filter(|stage| stage.status != "published")
            .count(),
        stages,
    };

    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join("corpus-01-dossier-index.json");
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&index)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;

    let markdown_path = docs_root.join("corpus-01-dossier-index.md");
    fs::write(&markdown_path, render_dossier_index_markdown(&index))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn build_dossier_stage_entry(
    docs_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    contract: &CorpusBenchmarkContract,
) -> Result<DossierStageEntry> {
    let stage_docs_root = docs_root.join(&contract.stage_id).join("corpus-01");
    let summary_path = stage_docs_root.join("summary.json");
    let dossier_path = resolve_existing_dossier_path(&stage_docs_root);

    let remote_corpus_root = workspace_remote_corpus_root(workspace)?;
    let remote_corpus_id = remote_corpus_root
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow!("invalid workspace.remote.corpus_root"))?;
    let expected_remote_run_root = workspace_remote_results_root(workspace)?.join(
        stage_run_relative_root(workspace, "remote", remote_corpus_id, &contract.stage_id),
    );
    let expected_local_cache_mirror_run_root =
        workspace_local_cache_mirror_root(workspace)?.join(stage_run_relative_root(
            workspace,
            "local-cache",
            remote_corpus_id,
            &contract.stage_id,
        ));
    let expected_local_results_run_root =
        workspace_local_results_root(workspace)?.join(stage_run_relative_root(
            workspace,
            "local-archive",
            remote_corpus_id,
            &contract.stage_id,
        ));
    let expected_remote_legacy_run_root = workspace_remote_results_legacy_root(workspace)?
        .join(remote_corpus_id)
        .join(&contract.stage_id)
        .join("lunarc");

    let mut entry = DossierStageEntry {
        stage_id: contract.stage_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        status: "missing".to_string(),
        summary_path: summary_path.display().to_string(),
        dossier_path: dossier_path.display().to_string(),
        expected_remote_run_root: expected_remote_run_root.display().to_string(),
        expected_remote_legacy_run_root: expected_remote_legacy_run_root.display().to_string(),
        expected_local_cache_mirror_run_root: expected_local_cache_mirror_run_root
            .display()
            .to_string(),
        expected_local_results_run_root: expected_local_results_run_root.display().to_string(),
        generated_at_utc: None,
        platform: None,
        corpus_root: None,
        run_root: None,
        run_root_source: None,
    };

    if !summary_path.is_file() {
        return Ok(entry);
    }

    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    let run_root = summary
        .get("run_root")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);

    entry.status = "published".to_string();
    entry.generated_at_utc = summary
        .get("generated_at_utc")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    entry.platform = summary
        .get("platform")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    entry.corpus_root = summary
        .get("corpus_root")
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned);
    entry.run_root = run_root.as_ref().map(|value| value.display().to_string());
    entry.run_root_source = run_root.as_ref().map(|path| {
        classify_run_root_source(
            path,
            &expected_remote_run_root,
            &expected_remote_legacy_run_root,
            &expected_local_cache_mirror_run_root,
            &expected_local_results_run_root,
            &remote_corpus_root,
        )
    });
    Ok(entry)
}

fn resolve_existing_dossier_path(stage_docs_root: &Path) -> PathBuf {
    let preferred = stage_docs_root.join("benchmark.md");
    if preferred.is_file() {
        return preferred;
    }
    let legacy = stage_docs_root.join("lunarc.md");
    if legacy.is_file() {
        return legacy;
    }
    preferred
}

fn render_dossier_index_markdown(index: &DossierIndex) -> String {
    let mut lines = vec![
        "# `corpus-01` FASTQ dossier index".to_string(),
        "".to_string(),
        format!("- Governed publication stages: `{}`", index.stage_count),
        format!("- Published summaries: `{}`", index.published_stage_count),
        format!("- Missing summaries: `{}`", index.missing_stage_count),
        "".to_string(),
        "## Stage index".to_string(),
        "".to_string(),
    ];
    for stage in &index.stages {
        if stage.status == "published" {
            lines.push(format!(
                "- `{}`: `{}` from `{}`",
                stage.stage_id,
                stage.generated_at_utc.as_deref().unwrap_or("unknown"),
                stage.run_root_source.as_deref().unwrap_or("missing")
            ));
            lines.push(format!(
                "  - published run root: `{}`",
                stage.run_root.as_deref().unwrap_or("")
            ));
            lines.push(format!(
                "  - expected remote run root: `{}`",
                stage.expected_remote_run_root
            ));
            lines.push(format!(
                "  - expected local cache mirror run root: `{}`",
                stage.expected_local_cache_mirror_run_root
            ));
        } else {
            lines.push(format!("- `{}`: `missing`", stage.stage_id));
            lines.push(format!(
                "  - expected remote run root: `{}`",
                stage.expected_remote_run_root
            ));
        }
    }
    lines.join("\n") + "\n"
}

fn classify_run_root_source(
    run_root: &Path,
    expected_remote_run_root: &Path,
    expected_remote_legacy_run_root: &Path,
    expected_local_cache_mirror_run_root: &Path,
    expected_local_results_run_root: &Path,
    remote_corpus_root: &Path,
) -> String {
    if run_root == expected_local_cache_mirror_run_root {
        return "local-cache-mirror".to_string();
    }
    if run_root == expected_local_results_run_root {
        return "local-results-root".to_string();
    }
    if run_root == expected_remote_run_root {
        return "remote-results-root".to_string();
    }
    if run_root == expected_remote_legacy_run_root {
        return "remote-results-legacy-root".to_string();
    }
    if remote_corpus_root
        .parent()
        .is_some_and(|root| run_root.starts_with(root))
    {
        return "remote-custom".to_string();
    }
    "custom".to_string()
}

fn stage_run_relative_root(
    workspace: &BenchmarkWorkspaceConfig,
    scope: &str,
    corpus_id: &str,
    stage_id: &str,
) -> PathBuf {
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
        .unwrap_or(match scope {
            "remote" => "{corpus_id}/{stage_id}/lunarc",
            "local-cache" => "results/{corpus_id}/{stage_id}/lunarc",
            "local-archive" => "{corpus_id}/{stage_id}/lunarc",
            _ => "{corpus_id}/{stage_id}/lunarc",
        });
    PathBuf::from(
        template
            .replace("{corpus_id}", corpus_id)
            .replace("{stage_id}", stage_id),
    )
}

fn workspace_remote_corpus_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.corpus_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.corpus_root"))
}

fn workspace_remote_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_root"))
}

fn workspace_remote_results_legacy_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .remote
        .as_ref()
        .and_then(|row| row.results_legacy_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_legacy_root"))
}

fn workspace_local_cache_mirror_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))
}

fn workspace_local_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))
}

fn corpus_fastq_stage_render_step(
    repo_root: &Path,
    stage_id: &str,
    stage_docs_root: &Path,
    run_root: Option<&Path>,
) -> Result<SubprocessSpec> {
    let script_path = corpus_fastq_script_path(repo_root, "report");
    if !script_path.is_file() {
        return Err(anyhow!(
            "missing corpus benchmark render script for {stage_id}: {}",
            script_path.display()
        ));
    }
    let mut args = vec![
        script_path.display().to_string(),
        "--stage".to_string(),
        stage_id.to_string(),
        "--repo-root".to_string(),
        repo_root.display().to_string(),
        "--docs-root".to_string(),
        stage_docs_root.display().to_string(),
    ];
    if let Some(run_root) = run_root {
        args.push("--run-root".to_string());
        args.push(absolutize(repo_root, run_root).display().to_string());
    }
    Ok(SubprocessSpec {
        program: "python3",
        args,
    })
}

fn corpus_fastq_stage_briefing_step(
    repo_root: &Path,
    stage_id: &str,
    stage_docs_root: &Path,
) -> Result<SubprocessSpec> {
    let script_path = corpus_fastq_script_path(repo_root, "briefing");
    if !script_path.is_file() {
        return Err(anyhow!(
            "missing corpus benchmark briefing script for {stage_id}: {}",
            script_path.display()
        ));
    }
    Ok(SubprocessSpec {
        program: "python3",
        args: vec![
            script_path.display().to_string(),
            "--stage".to_string(),
            stage_id.to_string(),
            "--docs-root".to_string(),
            stage_docs_root.display().to_string(),
        ],
    })
}

fn corpus_fastq_script_path(repo_root: &Path, kind: &str) -> PathBuf {
    repo_root.join(format!("makes/bin/render_corpus_01_fastq_{kind}.py"))
}

fn run_subprocess(repo_root: &Path, config_path: &Path, spec: &SubprocessSpec) -> Result<()> {
    let status = Command::new(spec.program)
        .args(&spec.args)
        .current_dir(repo_root)
        .env(BENCHMARK_CONFIG_ENV, config_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("run {}", format_command(spec)))?;
    if status.success() {
        return Ok(());
    }
    Err(anyhow!(
        "{} exited with {}",
        format_command(spec),
        status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "signal".to_string())
    ))
}

fn format_command(spec: &SubprocessSpec) -> String {
    std::iter::once(spec.program.to_string())
        .chain(spec.args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn absolutize(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn publication_target_maps_profile_overrepresented_stage() {
        assert_eq!(
            super::benchmark_make_target("fastq.profile_overrepresented_sequences", "report"),
            "_benchmark-profile-overrepresented-corpus-01-report"
        );
    }

    #[test]
    fn publication_target_maps_merge_pairs_stage() {
        assert_eq!(
            super::benchmark_make_target("fastq.merge_pairs", "run"),
            "_benchmark-merge-corpus-01"
        );
    }

    #[test]
    fn publication_target_maps_filter_reads_stage() {
        assert_eq!(
            super::benchmark_make_target("fastq.filter_reads", "report"),
            "_benchmark-filter-reads-corpus-01-report"
        );
    }

    #[test]
    fn corpus_fastq_report_script_path_matches_stage_contract() {
        assert_eq!(
            super::corpus_fastq_script_path(Path::new("/repo"), "report"),
            Path::new("/repo/makes/bin/render_corpus_01_fastq_report.py")
        );
    }

    #[test]
    fn corpus_fastq_report_docs_root_tracks_stage_contract() {
        let docs_root = super::absolutize(Path::new("/repo"), Path::new("docs/benchmark"))
            .join("fastq.validate_reads")
            .join("corpus-01");
        assert_eq!(
            docs_root,
            Path::new("/repo/docs/benchmark/fastq.validate_reads/corpus-01")
        );
    }

    #[test]
    fn corpus_fastq_render_step_passes_stage_to_dispatcher() {
        let step = super::corpus_fastq_stage_render_step(
            Path::new("/repo"),
            "fastq.validate_reads",
            Path::new("/repo/docs/benchmark/fastq.validate_reads/corpus-01"),
            None,
        );
        let err = step.expect_err("dispatcher should not exist in unit test cwd");
        assert!(err.to_string().contains("render_corpus_01_fastq_report.py"));
    }

    #[test]
    fn classify_run_root_source_prefers_local_cache_mirror() {
        assert_eq!(
            super::classify_run_root_source(
                Path::new(
                    "/archive/srv/cluster/.cache/results/corpus_01/fastq.validate_reads/lunarc"
                ),
                Path::new("/srv/cluster/.cache/results/corpus_01/fastq.validate_reads/lunarc"),
                Path::new("/srv/cluster/results/corpus_01/fastq.validate_reads/lunarc"),
                Path::new(
                    "/archive/srv/cluster/.cache/results/corpus_01/fastq.validate_reads/lunarc"
                ),
                Path::new("/archive/corpus_01/fastq.validate_reads/lunarc"),
                Path::new("/srv/cluster/.cache/corpus_01"),
            ),
            "local-cache-mirror"
        );
    }

    #[test]
    fn stage_run_relative_root_uses_default_local_cache_template() {
        let workspace = crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig::default();
        assert_eq!(
            super::stage_run_relative_root(
                &workspace,
                "local-cache",
                "corpus_01",
                "fastq.validate_reads",
            ),
            Path::new("results/corpus_01/fastq.validate_reads/lunarc")
        );
    }

    #[test]
    fn publication_status_steps_write_to_docs_root_outputs() {
        let steps =
            super::publication_status_steps(Path::new("/repo"), Path::new("/repo/docs/benchmark"));
        let last = steps.last().expect("status steps");
        assert_eq!(last.program, "python3");
        assert!(last
            .args
            .contains(&"/repo/docs/benchmark/corpus-01-remediation-queue.json".to_string()));
        assert!(last
            .args
            .contains(&"/repo/docs/benchmark/corpus-01-results-status.json".to_string()));
    }
}
