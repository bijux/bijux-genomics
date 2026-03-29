use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::models::{PublishedResultsStageReport, PublishedResultsStatusReport, StageResultIssue};
use super::{
    benchmark_publication_contracts, configured_stage_run_roots, find_polluting_ds_store_files,
    json_string_array, load_benchmark_config, load_json_value, localize_results_path,
    observed_tools_from_report, publication_artifact_file_name, publication_stage_docs_root,
    relative_to_repo_root, select_stage_run_root, sorted_json_string_array, sorted_strings,
    summary_corpus_id, unique_existing_run_roots, value_string, BenchmarkWorkspaceConfig,
    CorpusBenchmarkContract,
};

pub(super) fn write_corpus_fastq_results_status(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    let workspace = &config.workspace;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    let report = audit_published_results(cwd, workspace, docs_root, corpus_id, &contracts)?;
    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.json",
    ));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&report)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.md",
    ));
    fs::write(&markdown_path, render_published_results_markdown(&report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

pub(super) fn audit_published_results(
    repo_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    docs_root: &Path,
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
) -> Result<PublishedResultsStatusReport> {
    let stages = contracts
        .iter()
        .map(|contract| {
            audit_published_results_stage(repo_root, workspace, docs_root, corpus_id, contract)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(PublishedResultsStatusReport {
        corpus_id: corpus_id.to_string(),
        applicable_stage_count: contracts.len(),
        published_stage_count: contracts
            .iter()
            .filter(|contract| {
                publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id)
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

pub(super) fn audit_published_results_stage(
    repo_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    docs_root: &Path,
    corpus_id: &str,
    contract: &CorpusBenchmarkContract,
) -> Result<PublishedResultsStageReport> {
    let stage_docs_root = publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id);
    let summary_path = stage_docs_root.join("summary.json");
    let mut issues = Vec::new();
    if !summary_path.is_file() {
        append_stage_result_issue(
            &mut issues,
            &contract.stage_id,
            "missing-published-summary",
            format!("missing {}", relative_to_repo_root(&summary_path, repo_root)),
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
    let summary_corpus_root = match summary
        .get("corpus_root")
        .and_then(|value| value.as_str())
        .map(PathBuf::from)
    {
        Some(path) => path,
        None => {
            append_stage_result_issue(
                &mut issues,
                &contract.stage_id,
                "missing-summary-corpus-root",
                format!(
                    "summary {} must declare corpus_root",
                    relative_to_repo_root(&summary_path, repo_root)
                ),
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
    };
    let corpus_dir_name = summary_corpus_id(&summary_corpus_root)?;
    let expected_tools = sorted_strings(&contract.tools);
    let configured_roots =
        configured_stage_run_roots(workspace, &corpus_dir_name, &contract.stage_id)?;
    let canonical_run_root = configured_roots[0].path.clone();
    let legacy_run_root = configured_roots[1].path.clone();
    let reported_run_root = summary
        .get("run_root")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(PathBuf::new);
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
                    run_manifest.get("stage_id").and_then(|value| value.as_str())
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
                    run_manifest.get("scenario_id").and_then(|value| value.as_str())
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
                format!("run_manifest sample_limit={:?}", run_manifest.get("sample_limit")),
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
                let Some(sample_id) = run.get("sample_id").and_then(|value| value.as_str()) else {
                    continue;
                };
                tool_roster_drift_samples.push(format!(
                    "{} observed {:?}",
                    sample_id,
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

pub(super) fn render_published_results_markdown(report: &PublishedResultsStatusReport) -> String {
    let mut lines = vec![
        format!("# `{}` published result mirror status", report.corpus_id),
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
