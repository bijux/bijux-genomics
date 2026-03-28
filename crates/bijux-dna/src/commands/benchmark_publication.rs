use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
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
    write_corpus_fastq_results_status(cwd, args.config.as_deref(), &docs_root)?;
    for spec in publication_status_steps(cwd, &docs_root) {
        run_subprocess(cwd, &config_path, &spec)?;
    }
    write_corpus_fastq_remediation_queue(cwd, args.config.as_deref(), &docs_root)?;
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

#[derive(Debug, Serialize)]
struct PublishedResultsStatusReport {
    corpus_id: String,
    applicable_stage_count: usize,
    published_stage_count: usize,
    complete_stage_count: usize,
    incomplete_stage_count: usize,
    issue_count: usize,
    stages: Vec<PublishedResultsStageReport>,
}

#[derive(Debug, Serialize)]
struct PublishedResultsStageReport {
    stage_id: String,
    status: String,
    issue_count: usize,
    reported_run_root: String,
    selected_run_root: String,
    newest_available_run_root: String,
    selected_run_root_is_newest: bool,
    available_run_roots: Vec<String>,
    issues: Vec<StageResultIssue>,
}

#[derive(Debug, Serialize)]
struct StageResultIssue {
    stage_id: String,
    issue_id: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct RemediationQueue {
    corpus_id: String,
    stage_count: usize,
    open_stage_count: usize,
    clear_stage_count: usize,
    stages: Vec<RemediationStageEntry>,
}

#[derive(Debug, Serialize)]
struct RemediationStageEntry {
    stage_id: String,
    owner: String,
    status: String,
    issue_count: usize,
    issue_group_count: usize,
    recommended_action: String,
    publication_status: String,
    results_status: String,
    sample_scope: String,
    published_generated_at_utc: Option<String>,
    run_root_source: Option<String>,
    issue_groups: Vec<RemediationIssueGroup>,
    issues: Vec<RemediationIssue>,
}

#[derive(Debug, Clone, Serialize)]
struct RemediationIssue {
    issue_id: String,
    detail: String,
    severity: String,
    source: String,
}

#[derive(Debug, Serialize)]
struct RemediationIssueGroup {
    issue_id: String,
    count: usize,
    sources: Vec<String>,
    severity: String,
    example_details: Vec<String>,
    additional_detail_count: usize,
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

#[derive(Debug, Clone)]
struct StageRunRootCandidate {
    path: PathBuf,
}

#[derive(Debug)]
struct StageRunRootSelection {
    selected_path: PathBuf,
    newest_available_path: Option<PathBuf>,
}

fn write_corpus_fastq_results_status(
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
    let report = audit_published_results(workspace, docs_root, &contracts)?;
    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join("corpus-01-results-status.json");
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join("corpus-01-results-status.md");
    fs::write(&markdown_path, render_published_results_markdown(&report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

fn write_corpus_fastq_remediation_queue(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
) -> Result<()> {
    let publication_status = load_json_value(&docs_root.join("corpus-01-status.json"))?;
    let results_status = load_json_value(&docs_root.join("corpus-01-results-status.json"))?;
    let findings_payload = load_json_value(&docs_root.join("corpus-01-publication-findings.json"))?;
    let dossier_index = load_json_value(&docs_root.join("corpus-01-dossier-index.json"))?;
    let publication = load_benchmark_publication_config(cwd, explicit_config)?;
    let contracts = publication
        .corpus_01
        .map(|row| row.contracts)
        .unwrap_or_default();
    let queue = build_remediation_queue(
        &contracts,
        &publication_status,
        &results_status,
        &findings_payload,
        &dossier_index,
    );
    let json_path = docs_root.join("corpus-01-remediation-queue.json");
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&queue)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join("corpus-01-remediation-queue.md");
    fs::write(&markdown_path, render_remediation_queue_markdown(&queue))
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

fn load_json_value(path: &Path) -> Result<serde_json::Value> {
    serde_json::from_str(
        &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
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

fn audit_published_results(
    workspace: &BenchmarkWorkspaceConfig,
    docs_root: &Path,
    contracts: &[CorpusBenchmarkContract],
) -> Result<PublishedResultsStatusReport> {
    let stages = contracts
        .iter()
        .map(|contract| audit_published_results_stage(workspace, docs_root, contract))
        .collect::<Result<Vec<_>>>()?;
    Ok(PublishedResultsStatusReport {
        corpus_id: "corpus-01".to_string(),
        applicable_stage_count: contracts.len(),
        published_stage_count: contracts
            .iter()
            .filter(|contract| {
                docs_root
                    .join(&contract.stage_id)
                    .join("corpus-01")
                    .join("summary.json")
                    .is_file()
            })
            .count(),
        complete_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "complete")
            .count(),
        incomplete_stage_count: stages
            .iter()
            .filter(|stage| stage.status != "complete")
            .count(),
        issue_count: stages.iter().map(|stage| stage.issue_count).sum(),
        stages,
    })
}

fn audit_published_results_stage(
    workspace: &BenchmarkWorkspaceConfig,
    docs_root: &Path,
    contract: &CorpusBenchmarkContract,
) -> Result<PublishedResultsStageReport> {
    let stage_docs_root = docs_root.join(&contract.stage_id).join("corpus-01");
    let summary_path = stage_docs_root.join("summary.json");
    let mut issues = Vec::new();
    if !summary_path.is_file() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-published-summary",
            format!("missing {}", summary_path.display()),
        );
        return Ok(PublishedResultsStageReport {
            stage_id: contract.stage_id.clone(),
            status: "incomplete".to_string(),
            issue_count: issues.len(),
            reported_run_root: String::new(),
            selected_run_root: String::new(),
            newest_available_run_root: String::new(),
            selected_run_root_is_newest: false,
            available_run_roots: Vec::new(),
            issues,
        });
    }

    let summary = load_json_value(&summary_path)?;
    let summary_corpus_root = summary
        .get("corpus_root")
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
        .unwrap_or_default();
    let corpus_id = summary_corpus_id(&summary_corpus_root, workspace)
        .unwrap_or_else(|| "corpus_01".to_string());
    let expected_tools = sorted_strings(&contract.tools);
    let configured_roots = configured_stage_run_roots(workspace, &corpus_id, &contract.stage_id)?;
    let canonical_run_root = configured_roots[0].path.clone();
    let legacy_run_root = configured_roots[1].path.clone();
    let reported_run_root = summary
        .get("run_root")
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
        .unwrap_or_default();
    let selection = select_stage_run_root(&configured_roots);
    let selected_run_root = if reported_run_root.is_dir() {
        reported_run_root.clone()
    } else {
        selection.selected_path.clone()
    };
    let unique_existing_roots = unique_existing_run_roots(&reported_run_root, &configured_roots);
    if canonical_run_root.is_dir() && legacy_run_root.is_dir() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "duplicate-result-root-ambiguity",
            format!(
                "both {} and {} exist",
                canonical_run_root.display(),
                legacy_run_root.display()
            ),
        );
    }
    if reported_run_root != canonical_run_root && !reported_run_root.is_dir() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "summary-run-root-drift",
            format!(
                "summary run_root={} expected {}",
                reported_run_root.display(),
                canonical_run_root.display()
            ),
        );
    }
    if let Some(newest_available_run_root) = selection.newest_available_path.as_ref() {
        if newest_available_run_root != &selected_run_root {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "newer-run-root-available",
                format!(
                    "published dossier selected {} but newer mirrored run exists at {}",
                    selected_run_root.display(),
                    newest_available_run_root.display()
                ),
            );
        }
    }
    if !selected_run_root.is_dir() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-local-run-root",
            format!(
                "local mirror missing: selected={}; summary_run_root={}; expected_local_mirror={}",
                selected_run_root.display(),
                reported_run_root.display(),
                canonical_run_root.display()
            ),
        );
    } else {
        let polluting_files = find_polluting_ds_store_files(&selected_run_root);
        if !polluting_files.is_empty() {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "polluting-mirror-artifact",
                format!(
                    "mirror contains {} .DS_Store files under {}",
                    polluting_files.len(),
                    selected_run_root.display()
                ),
            );
        }
    }

    let stage_run_manifest = selected_run_root.join("run_manifest.json");
    if !stage_run_manifest.is_file() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-stage-run-manifest",
            format!("missing {}", stage_run_manifest.display()),
        );
    } else {
        let run_manifest = load_json_value(&stage_run_manifest)?;
        if value_string(&run_manifest, "stage_id") != Some(contract.stage_id.as_str()) {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-stage-id-drift",
                format!(
                    "run_manifest stage_id={:?}",
                    run_manifest
                        .get("stage_id")
                        .and_then(|value| value.as_str())
                ),
            );
        }
        if value_string(&run_manifest, "scenario_id") != Some(contract.scenario_id.as_str()) {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-scenario-id-drift",
                format!(
                    "run_manifest scenario_id={:?}",
                    run_manifest
                        .get("scenario_id")
                        .and_then(|value| value.as_str())
                ),
            );
        }
        if sorted_json_string_array(run_manifest.get("tools")) != expected_tools {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-tool-roster-drift",
                format!(
                    "run_manifest tools={:?} expected {:?}",
                    json_string_array(run_manifest.get("tools")),
                    expected_tools
                ),
            );
        }
        if run_manifest
            .get("dry_run")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-dry-run",
                "run_manifest recorded dry_run=true".to_string(),
            );
        }
        if run_manifest
            .get("sample_limit")
            .is_some_and(|value| !value.is_null())
        {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-sample-limit",
                format!(
                    "run_manifest sample_limit={:?}",
                    run_manifest.get("sample_limit")
                ),
            );
        }
        if run_manifest
            .get("samples_failed")
            .and_then(|value| value.as_i64())
            .unwrap_or(0)
            != 0
        {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "run-manifest-sample-failures",
                format!(
                    "run_manifest samples_failed={:?}",
                    run_manifest.get("samples_failed")
                ),
            );
        }

        let local_results_root = selected_run_root
            .ancestors()
            .nth(2)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| selected_run_root.clone());
        let mut missing_report_count = 0usize;
        let mut tool_roster_drift_samples = Vec::new();
        for run in run_manifest
            .get("runs")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
        {
            let Some(report_json) = run.get("report_json").and_then(|value| value.as_str()) else {
                missing_report_count += 1;
                continue;
            };
            let localized_report =
                localize_results_path(report_json, &local_results_root, workspace);
            if !localized_report.is_file() {
                missing_report_count += 1;
                continue;
            }
            let observed_tools = observed_tools_from_report(&localized_report)?;
            if observed_tools != expected_tools {
                tool_roster_drift_samples.push(format!(
                    "{} observed {:?}",
                    run.get("sample_id")
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown"),
                    observed_tools
                ));
            }
        }
        if missing_report_count > 0 {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "missing-localized-report-json",
                format!(
                    "{} run rows do not resolve to a local report.json",
                    missing_report_count
                ),
            );
        }
        if !tool_roster_drift_samples.is_empty() {
            let preview = tool_roster_drift_samples
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("; ");
            let detail = if tool_roster_drift_samples.len() > 3 {
                format!("{preview} (+{} more)", tool_roster_drift_samples.len() - 3)
            } else {
                preview
            };
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "report-tool-roster-drift",
                detail,
            );
        }
    }

    let newest_available_run_root = selection
        .newest_available_path
        .unwrap_or_else(|| selected_run_root.clone());
    let selected_run_root_is_newest = newest_available_run_root == selected_run_root;
    Ok(PublishedResultsStageReport {
        stage_id: contract.stage_id.clone(),
        status: if issues.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        },
        issue_count: issues.len(),
        reported_run_root: reported_run_root.display().to_string(),
        selected_run_root: selected_run_root.display().to_string(),
        newest_available_run_root: newest_available_run_root.display().to_string(),
        selected_run_root_is_newest,
        available_run_roots: unique_existing_roots
            .iter()
            .map(|root| root.display().to_string())
            .collect(),
        issues,
    })
}

fn render_published_results_markdown(report: &PublishedResultsStatusReport) -> String {
    let mut lines = vec![
        "# `corpus-01` published result mirror status".to_string(),
        "".to_string(),
        format!(
            "- Governed publication stages: `{}`",
            report.applicable_stage_count
        ),
        format!(
            "- Published stages audited: `{}`",
            report.published_stage_count
        ),
        format!(
            "- Complete mirrored stages: `{}`",
            report.complete_stage_count
        ),
        format!(
            "- Incomplete mirrored stages: `{}`",
            report.incomplete_stage_count
        ),
        format!("- Mirror issues: `{}`", report.issue_count),
        "".to_string(),
        "## Stage status".to_string(),
        "".to_string(),
    ];
    for stage in &report.stages {
        lines.push(format!(
            "- `{}`: `{}` (`{}` issues)",
            stage.stage_id, stage.status, stage.issue_count
        ));
        if !stage.selected_run_root.is_empty() {
            lines.push(format!(
                "  - selected run root: `{}`",
                stage.selected_run_root
            ));
        }
        if !stage.newest_available_run_root.is_empty() {
            lines.push(format!(
                "  - newest available run root: `{}` (selected newest=`{}`)",
                stage.newest_available_run_root, stage.selected_run_root_is_newest
            ));
        }
        if !stage.available_run_roots.is_empty() {
            let roots = stage
                .available_run_roots
                .iter()
                .map(|root| format!("`{root}`"))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("  - available run roots: {roots}"));
        }
        for issue in &stage.issues {
            lines.push(format!("  - `{}`: {}", issue.issue_id, issue.detail));
        }
    }
    lines.join("\n") + "\n"
}

fn append_stage_result_issue(
    issues: &mut Vec<StageResultIssue>,
    stage_id: &str,
    issue_id: &str,
    detail: String,
) {
    issues.push(StageResultIssue {
        stage_id: stage_id.to_string(),
        issue_id: issue_id.to_string(),
        detail,
    });
}

fn summary_corpus_id(
    summary_corpus_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
) -> Option<String> {
    summary_corpus_root
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned)
        .or_else(|| {
            workspace
                .remote
                .as_ref()
                .and_then(|row| row.corpus_root.as_deref())
                .map(Path::new)
                .and_then(|path| path.file_name())
                .and_then(|value| value.to_str())
                .map(ToOwned::to_owned)
        })
}

fn configured_stage_run_roots(
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    stage_id: &str,
) -> Result<Vec<StageRunRootCandidate>> {
    Ok(vec![
        StageRunRootCandidate {
            path: workspace_local_cache_mirror_root(workspace)?.join(stage_run_relative_root(
                workspace,
                "local-cache",
                corpus_id,
                stage_id,
            )),
        },
        StageRunRootCandidate {
            path: workspace_local_results_root(workspace)?.join(stage_run_relative_root(
                workspace,
                "local-archive",
                corpus_id,
                stage_id,
            )),
        },
        StageRunRootCandidate {
            path: workspace_remote_results_root(workspace)?.join(stage_run_relative_root(
                workspace, "remote", corpus_id, stage_id,
            )),
        },
    ])
}

fn unique_existing_run_roots(
    reported_run_root: &Path,
    configured_roots: &[StageRunRootCandidate],
) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for root in std::iter::once(reported_run_root).chain(
        configured_roots
            .iter()
            .map(|candidate| candidate.path.as_path()),
    ) {
        if !root.is_dir() || roots.iter().any(|existing| existing == root) {
            continue;
        }
        roots.push(root.to_path_buf());
    }
    roots
}

fn select_stage_run_root(candidates: &[StageRunRootCandidate]) -> StageRunRootSelection {
    let existing_candidates = candidates
        .iter()
        .filter(|candidate| candidate.path.is_dir())
        .cloned()
        .collect::<Vec<_>>();
    if existing_candidates.is_empty() {
        return StageRunRootSelection {
            selected_path: candidates
                .first()
                .map(|candidate| candidate.path.clone())
                .unwrap_or_default(),
            newest_available_path: None,
        };
    }
    let mut freshest_path = existing_candidates[0].path.clone();
    let mut freshest_timestamp = run_root_freshness_timestamp(&freshest_path);
    for candidate in existing_candidates.iter().skip(1) {
        let candidate_timestamp = run_root_freshness_timestamp(&candidate.path);
        if candidate_timestamp.is_some()
            && (freshest_timestamp.is_none() || candidate_timestamp > freshest_timestamp)
        {
            freshest_path = candidate.path.clone();
            freshest_timestamp = candidate_timestamp;
        }
    }
    StageRunRootSelection {
        selected_path: freshest_path.clone(),
        newest_available_path: Some(freshest_path),
    }
}

fn run_root_freshness_timestamp(run_root: &Path) -> Option<DateTime<Utc>> {
    let manifest_path = run_root.join("run_manifest.json");
    if manifest_path.is_file() {
        let manifest = load_json_value(&manifest_path).ok()?;
        for key in [
            "completed_at_utc",
            "generated_at_utc",
            "finished_at_utc",
            "started_at_utc",
        ] {
            if let Some(parsed) =
                parse_utc_timestamp(manifest.get(key).and_then(|value| value.as_str()))
            {
                return Some(parsed);
            }
        }
    }
    None
}

fn run_root_observed_timestamp(run_root: &Path) -> Option<DateTime<Utc>> {
    run_root_freshness_timestamp(run_root).or_else(|| {
        fs::metadata(run_root)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .map(DateTime::<Utc>::from)
    })
}

fn parse_utc_timestamp(raw: Option<&str>) -> Option<DateTime<Utc>> {
    let normalized = raw?.trim().replace('Z', "+00:00");
    if normalized.is_empty() {
        return None;
    }
    DateTime::parse_from_rfc3339(&normalized)
        .map(|value| value.with_timezone(&Utc))
        .ok()
}

fn find_polluting_ds_store_files(root: &Path) -> Vec<PathBuf> {
    let mut polluting_files = Vec::new();
    let Ok(entries) = fs::read_dir(root) else {
        return polluting_files;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            polluting_files.extend(find_polluting_ds_store_files(&path));
        } else if path.file_name().and_then(|value| value.to_str()) == Some(".DS_Store") {
            polluting_files.push(path);
        }
    }
    polluting_files.sort();
    polluting_files
}

fn observed_tools_from_report(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pattern = Regex::new(r#""tool"\s*:\s*"([^"]+)""#).expect("tool regex");
    let tools = pattern
        .captures_iter(&text)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<BTreeSet<_>>();
    Ok(tools.into_iter().collect())
}

fn localize_results_path(
    path_str: &str,
    local_results_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
) -> PathBuf {
    let path = PathBuf::from(path_str);
    if path.exists() {
        return path;
    }

    let cache_mirror_root = workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from);
    let bijux_dna_results_root = cache_mirror_root.as_ref().and_then(|cache_root| {
        (local_results_root
            .file_name()
            .and_then(|value| value.to_str())
            == Some("results")
            && local_results_root.parent() == Some(cache_root.as_path()))
        .then(|| cache_root.join("bijux-dna-results"))
    });
    let mut root_mappings = vec![("/results/", vec![local_results_root.to_path_buf()])];
    if let Some(bijux_root) = bijux_dna_results_root {
        root_mappings.push((
            "/bijux-dna-results/",
            vec![bijux_root, local_results_root.to_path_buf()],
        ));
    } else {
        root_mappings.push((
            "/bijux-dna-results/",
            vec![local_results_root.to_path_buf()],
        ));
    }
    if let Some(extra_data_root) = workspace
        .local
        .as_ref()
        .and_then(|row| row.extra_data_root.as_deref())
        .map(PathBuf::from)
    {
        root_mappings.push(("/extra-data/", vec![extra_data_root]));
    }
    if let Some(reference_root) = workspace
        .local
        .as_ref()
        .and_then(|row| row.reference_root.as_deref())
        .map(PathBuf::from)
    {
        root_mappings.push(("/reference/", vec![reference_root]));
    }

    let mut fallback_path = None;
    for (marker, mapped_roots) in root_mappings {
        if !path_str.contains(marker) {
            continue;
        }
        let suffix = path_str
            .split_once(marker)
            .map(|(_, tail)| tail)
            .unwrap_or_default();
        for mapped_root in mapped_roots {
            let localized = mapped_root.join(suffix);
            if localized.exists() {
                return localized;
            }
            if fallback_path.is_none() {
                fallback_path = Some(localized);
            }
        }
    }
    fallback_path.unwrap_or(path)
}

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn sorted_json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    let mut values = json_string_array(value);
    values.sort();
    values
}

fn json_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str().map(ToOwned::to_owned))
        .collect()
}

fn value_string<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(|entry| entry.as_str())
}

fn build_remediation_queue(
    contracts: &[CorpusBenchmarkContract],
    publication_status: &serde_json::Value,
    results_status: &serde_json::Value,
    findings_payload: &serde_json::Value,
    dossier_index: &serde_json::Value,
) -> RemediationQueue {
    let publication_by_stage = stage_value_lookup(publication_status);
    let results_by_stage = stage_value_lookup(results_status);
    let dossier_by_stage = stage_value_lookup(dossier_index);
    let findings_by_stage = findings_lookup(findings_payload);

    let stages = contracts
        .iter()
        .map(|contract| {
            let publication_stage = publication_by_stage.get(&contract.stage_id);
            let results_stage = results_by_stage.get(&contract.stage_id);
            let dossier_stage = dossier_by_stage.get(&contract.stage_id);

            let mut issues = collect_stage_issues(publication_stage, "publication");
            issues.extend(collect_stage_issues(results_stage, "results"));
            issues.extend(
                findings_by_stage
                    .get(&contract.stage_id)
                    .cloned()
                    .unwrap_or_default(),
            );
            let issue_groups = summarize_issue_groups(&issues);
            let issue_ids = issues
                .iter()
                .map(|issue| issue.issue_id.clone())
                .collect::<Vec<_>>();

            RemediationStageEntry {
                stage_id: contract.stage_id.clone(),
                owner: "benchmark-governance".to_string(),
                status: if issues.is_empty() {
                    "clear".to_string()
                } else {
                    "open".to_string()
                },
                issue_count: issues.len(),
                issue_group_count: issue_groups.len(),
                recommended_action: if issues.is_empty() {
                    "none".to_string()
                } else {
                    classify_recommended_action(&issue_ids)
                },
                publication_status: stage_value_string(publication_stage, "status", "missing"),
                results_status: stage_value_string(results_stage, "status", "missing"),
                sample_scope: contract.sample_scope.clone(),
                published_generated_at_utc: stage_value_optional_string(
                    dossier_stage,
                    "generated_at_utc",
                ),
                run_root_source: stage_value_optional_string(dossier_stage, "run_root_source"),
                issue_groups,
                issues,
            }
        })
        .collect::<Vec<_>>();

    RemediationQueue {
        corpus_id: "corpus-01".to_string(),
        stage_count: stages.len(),
        open_stage_count: stages.iter().filter(|stage| stage.status == "open").count(),
        clear_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "clear")
            .count(),
        stages,
    }
}

fn stage_value_lookup<'a>(
    payload: &'a serde_json::Value,
) -> BTreeMap<String, &'a serde_json::Value> {
    payload
        .get("stages")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|stage| {
            stage
                .get("stage_id")
                .and_then(|value| value.as_str())
                .map(|stage_id| (stage_id.to_string(), stage))
        })
        .collect()
}

fn findings_lookup(payload: &serde_json::Value) -> BTreeMap<String, Vec<RemediationIssue>> {
    let mut findings_by_stage = BTreeMap::new();
    for finding in payload
        .get("findings")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let Some(stage_id) = finding.get("stage_id").and_then(|value| value.as_str()) else {
            continue;
        };
        findings_by_stage
            .entry(stage_id.to_string())
            .or_insert_with(Vec::new)
            .push(RemediationIssue {
                issue_id: finding
                    .get("issue_id")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                detail: finding
                    .get("detail")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_string(),
                severity: finding
                    .get("severity")
                    .and_then(|value| value.as_str())
                    .unwrap_or("error")
                    .to_string(),
                source: "findings".to_string(),
            });
    }
    findings_by_stage
}

fn collect_stage_issues(stage: Option<&&serde_json::Value>, source: &str) -> Vec<RemediationIssue> {
    stage
        .and_then(|value| value.get("issues"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .map(|issue| RemediationIssue {
            issue_id: issue
                .get("issue_id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_string(),
            detail: issue
                .get("detail")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string(),
            severity: issue
                .get("severity")
                .and_then(|value| value.as_str())
                .unwrap_or("error")
                .to_string(),
            source: source.to_string(),
        })
        .collect()
}

fn summarize_issue_groups(issues: &[RemediationIssue]) -> Vec<RemediationIssueGroup> {
    let mut grouped = BTreeMap::<String, (usize, BTreeMap<String, ()>, Vec<String>, String)>::new();
    for issue in issues {
        let group = grouped
            .entry(issue.issue_id.clone())
            .or_insert_with(|| (0, BTreeMap::new(), Vec::new(), issue.severity.clone()));
        group.0 += 1;
        group.1.insert(issue.source.clone(), ());
        let detail = issue.detail.trim();
        if !detail.is_empty() && !group.2.iter().any(|existing| existing == detail) {
            group.2.push(detail.to_string());
        }
    }
    grouped
        .into_iter()
        .map(
            |(issue_id, (count, sources, details, severity))| RemediationIssueGroup {
                issue_id,
                count,
                sources: sources.into_keys().collect(),
                severity,
                example_details: details.iter().take(3).cloned().collect(),
                additional_detail_count: details.len().saturating_sub(3),
            },
        )
        .collect()
}

fn classify_recommended_action(issue_ids: &[String]) -> String {
    let sync_issue_ids = [
        "missing-local-run-root",
        "missing-stage-run-manifest",
        "missing-localized-report-json",
        "duplicate-result-root-ambiguity",
    ];
    let publish_issue_ids = [
        "missing-published-summary",
        "missing-corpus-dir",
        "missing-summary-json",
        "missing-lunarc-md",
        "missing-sample-results-csv",
        "missing-tool-runtime-summary-csv",
        "missing-cohort-runtime-summary-csv",
        "missing-sample-runtime-outliers-csv",
    ];
    let rerun_issue_fragments = ["sample-failures", "dry-run", "sample-limit"];
    if issue_ids
        .iter()
        .any(|issue_id| sync_issue_ids.contains(&issue_id.as_str()))
    {
        return "sync-or-normalize-results".to_string();
    }
    if issue_ids
        .iter()
        .any(|issue_id| publish_issue_ids.contains(&issue_id.as_str()))
    {
        return "render-or-publish-dossier".to_string();
    }
    if issue_ids.iter().any(|issue_id| {
        rerun_issue_fragments
            .iter()
            .any(|fragment| issue_id.contains(fragment))
    }) {
        return "rerun-benchmark-stage".to_string();
    }
    "repair-benchmark-contract".to_string()
}

fn render_remediation_queue_markdown(queue: &RemediationQueue) -> String {
    let mut lines = vec![
        "# `corpus-01` FASTQ remediation queue".to_string(),
        "".to_string(),
        format!("- Governed publication stages: `{}`", queue.stage_count),
        format!("- Open stages: `{}`", queue.open_stage_count),
        format!("- Clear stages: `{}`", queue.clear_stage_count),
        "".to_string(),
        "## Stage queue".to_string(),
        "".to_string(),
    ];
    for stage in &queue.stages {
        lines.push(format!(
            "- `{}`: `{}` via `{}`",
            stage.stage_id, stage.status, stage.recommended_action
        ));
        lines.push(format!(
            "  - publication `{}`, results `{}`, owner `{}`",
            stage.publication_status, stage.results_status, stage.owner
        ));
        if let Some(generated_at) = stage.published_generated_at_utc.as_deref() {
            lines.push(format!(
                "  - dossier `{}` from `{}`",
                generated_at,
                stage.run_root_source.as_deref().unwrap_or("unknown")
            ));
        }
        for group in &stage.issue_groups {
            lines.push(format!(
                "  - issue group `{}` x{} from {}",
                group.issue_id,
                group.count,
                group.sources.join(", ")
            ));
            for detail in &group.example_details {
                lines.push(format!("    - {detail}"));
            }
            if group.additional_detail_count > 0 {
                lines.push(format!(
                    "    - (+{} more detail rows)",
                    group.additional_detail_count
                ));
            }
        }
    }
    lines.join("\n") + "\n"
}

fn stage_value_string(stage: Option<&&serde_json::Value>, key: &str, default: &str) -> String {
    stage
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .unwrap_or(default)
        .to_string()
}

fn stage_value_optional_string(stage: Option<&&serde_json::Value>, key: &str) -> Option<String> {
    stage
        .and_then(|value| value.get(key))
        .and_then(|value| value.as_str())
        .map(ToOwned::to_owned)
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
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

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
            local: Some(
                crate::commands::benchmark_workspace::BenchmarkWorkspaceLocal {
                    results_root: Some(archive_root.display().to_string()),
                    cache_mirror_root: Some(cache_root.display().to_string()),
                    extra_data_root: Some(cache_root.join("extra-data").display().to_string()),
                    reference_root: Some(cache_root.join("reference").display().to_string()),
                },
            ),
            remote: Some(
                crate::commands::benchmark_workspace::BenchmarkWorkspaceRemote {
                    corpus_root: Some(remote_corpus_root.display().to_string()),
                    results_root: Some(remote_root.join("results").display().to_string()),
                    results_legacy_root: Some(
                        remote_root.join("legacy-results").display().to_string(),
                    ),
                    ..Default::default()
                },
            ),
            layout: None,
            artifacts: BTreeMap::new(),
            sync: None,
        }
    }

    fn write_json(path: &Path, value: serde_json::Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(
            path,
            format!("{}\n", serde_json::to_string_pretty(&value).expect("json")),
        )
        .expect("write json");
    }

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
    fn publication_status_steps_stop_before_rust_owned_outputs() {
        let steps =
            super::publication_status_steps(Path::new("/repo"), Path::new("/repo/docs/benchmark"));
        let last = steps.last().expect("status steps");
        assert_eq!(last.program, "python3");
        assert!(last
            .args
            .iter()
            .any(|arg| arg == "/repo/makes/bin/audit_corpus_01_fastq_benchmark_docs.py"));
        assert!(last
            .args
            .contains(&"/repo/docs/benchmark/corpus-01-status.json".to_string()));
        assert!(!steps.iter().flat_map(|step| step.args.iter()).any(|arg| {
            arg.contains("audit_published_corpus_01_fastq_results.py")
                || arg.contains("corpus-01-results-status.json")
                || arg.contains("corpus-01-results-status.md")
                || arg.contains("build_corpus_01_benchmark_remediation_queue.py")
                || arg.contains("corpus-01-remediation-queue.json")
                || arg.contains("corpus-01-remediation-queue.md")
        }));
    }

    #[test]
    fn results_audit_tracks_missing_published_stage_summary() {
        let temp = tempdir().expect("tempdir");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("corpus_01");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let report = super::audit_published_results(
            &workspace,
            &temp.path().join("docs").join("benchmark"),
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
    fn results_audit_missing_local_run_root_reports_expected_mirror() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("corpus_01");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let reported_run_root = cache_root
            .join("results")
            .join("corpus_01")
            .join("fastq.validate_reads")
            .join("lunarc");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": reported_run_root,
            }),
        );
        let report = super::audit_published_results_stage(
            &workspace,
            &docs_root,
            &validate_reads_contract(),
        )
        .expect("stage report");
        let missing_issue = report
            .issues
            .iter()
            .find(|issue| issue.issue_id == "missing-local-run-root")
            .expect("missing issue");
        assert!(missing_issue
            .detail
            .contains(&reported_run_root.display().to_string()));
        assert!(missing_issue.detail.contains("expected_local_mirror="));
        assert_eq!(
            report.reported_run_root,
            reported_run_root.display().to_string()
        );
        assert!(report.available_run_roots.is_empty());
    }

    #[test]
    fn results_audit_flags_duplicate_local_run_roots() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("corpus_01");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let canonical_run_root = cache_root
            .join("results")
            .join("corpus_01")
            .join("fastq.validate_reads")
            .join("lunarc");
        let legacy_run_root = archive_root
            .join("corpus_01")
            .join("fastq.validate_reads")
            .join("lunarc");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
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
            &workspace,
            &docs_root,
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
        let remote_corpus_root = cache_root.join("corpus_01");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let canonical_run_root = cache_root
            .join("results")
            .join("corpus_01")
            .join("fastq.validate_reads")
            .join("lunarc");
        let legacy_run_root = archive_root
            .join("corpus_01")
            .join("fastq.validate_reads")
            .join("lunarc");
        let sample_report = canonical_run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
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
            &workspace,
            &docs_root,
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert_eq!(
            report.selected_run_root,
            canonical_run_root.display().to_string()
        );
        assert_eq!(
            report.newest_available_run_root,
            legacy_run_root.display().to_string()
        );
        assert!(!report.selected_run_root_is_newest);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "newer-run-root-available"));
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
                    reported_run_root: "/mirror/results/corpus_01/fastq.validate_reads/lunarc"
                        .to_string(),
                    selected_run_root: "/mirror/results/corpus_01/fastq.validate_reads/lunarc"
                        .to_string(),
                    newest_available_run_root: "/archive/corpus_01/fastq.validate_reads/lunarc"
                        .to_string(),
                    selected_run_root_is_newest: false,
                    available_run_roots: vec![
                        "/mirror/results/corpus_01/fastq.validate_reads/lunarc".to_string(),
                        "/archive/corpus_01/fastq.validate_reads/lunarc".to_string(),
                    ],
                    issues: vec![super::StageResultIssue {
                        stage_id: "fastq.validate_reads".to_string(),
                        issue_id: "missing-local-run-root".to_string(),
                        detail: "missing local mirror".to_string(),
                    }],
                }],
            });
        assert!(rendered.contains("selected run root"));
        assert!(rendered.contains("/mirror/results/corpus_01/fastq.validate_reads/lunarc"));
        assert!(rendered.contains("/archive/corpus_01/fastq.validate_reads/lunarc"));
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
    fn results_audit_flags_polluting_mirror_artifacts() {
        let temp = tempdir().expect("tempdir");
        let docs_root = temp.path().join("docs").join("benchmark");
        let cache_root = temp.path().join("cache-mirror");
        let archive_root = temp.path().join("archive");
        let remote_root = temp.path().join("remote");
        let remote_corpus_root = cache_root.join("corpus_01");
        let workspace = sample_workspace(
            &cache_root,
            &archive_root,
            &remote_root,
            &remote_corpus_root,
        );
        let run_root = temp
            .path()
            .join("mirror")
            .join("corpus_01")
            .join("fastq.validate_reads")
            .join("lunarc");
        let sample_report = run_root
            .join("bench")
            .join("validate_reads")
            .join("sample_0001")
            .join("report.json");
        fs::create_dir_all(run_root.join("bench")).expect("create bench");
        fs::write(run_root.join("bench").join(".DS_Store"), "").expect("write ds store");
        write_json(
            &docs_root
                .join("fastq.validate_reads")
                .join("corpus-01")
                .join("summary.json"),
            serde_json::json!({
                "corpus_root": remote_corpus_root,
                "run_root": run_root,
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
        let report = super::audit_published_results_stage(
            &workspace,
            &docs_root,
            &validate_reads_contract(),
        )
        .expect("stage report");
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.issue_id == "polluting-mirror-artifact"));
    }

    #[test]
    fn remediation_queue_merges_publication_results_and_findings() {
        let queue = super::build_remediation_queue(
            &[
                crate::commands::benchmark_workspace::CorpusBenchmarkContract {
                    stage_id: "fastq.validate_reads".to_string(),
                    scenario_id: "governed-fixture".to_string(),
                    sample_scope: "paired-subset".to_string(),
                    tools: Vec::new(),
                },
            ],
            &serde_json::json!({
                "stages": [{
                    "stage_id": "fastq.validate_reads",
                    "status": "incomplete",
                    "issues": [{
                        "issue_id": "missing-benchmark-md",
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
        );

        let stage = queue.stages.first().expect("stage");
        assert_eq!(stage.stage_id, "fastq.validate_reads");
        assert_eq!(stage.status, "open");
        assert_eq!(stage.issue_count, 3);
        assert_eq!(stage.recommended_action, "sync-or-normalize-results");
        assert_eq!(
            stage.published_generated_at_utc.as_deref(),
            Some("2026-03-28T00:00:00Z")
        );
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
