use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::models::{
    RemediationIssue, RemediationIssueGroup, RemediationQueue, RemediationStageEntry,
};
use super::{
    benchmark_publication_contracts, load_json_value, publication_artifact_file_name,
    CorpusBenchmarkContract,
};

pub(super) fn write_corpus_fastq_remediation_queue(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let publication_status =
        load_json_value(&docs_root.join(publication_artifact_file_name(corpus_id, "status.json")))?;
    let results_status = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "results-status.json",
    )))?;
    let findings_payload = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "publication-findings.json",
    )))?;
    let dossier_index = load_json_value(&docs_root.join(publication_artifact_file_name(
        corpus_id,
        "dossier-index.json",
    )))?;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    let queue = build_remediation_queue(
        corpus_id,
        &contracts,
        &publication_status,
        &results_status,
        &findings_payload,
        &dossier_index,
    )?;
    let json_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "remediation-queue.json",
    ));
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(&queue)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(
        corpus_id,
        "remediation-queue.md",
    ));
    fs::write(&markdown_path, render_remediation_queue_markdown(&queue))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

pub(super) fn build_remediation_queue(
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
    publication_status: &serde_json::Value,
    results_status: &serde_json::Value,
    findings_payload: &serde_json::Value,
    dossier_index: &serde_json::Value,
) -> Result<RemediationQueue> {
    let publication_by_stage = stage_value_lookup(publication_status);
    let results_by_stage = stage_value_lookup(results_status);
    let dossier_by_stage = stage_value_lookup(dossier_index);
    let findings_by_stage = findings_lookup(findings_payload);

    let stages = contracts
        .iter()
        .map(|contract| -> Result<RemediationStageEntry> {
            let publication_stage = publication_by_stage.get(&contract.stage_id);
            let results_stage = results_by_stage.get(&contract.stage_id);
            let dossier_stage = dossier_by_stage.get(&contract.stage_id);

            let mut issues = collect_stage_issues(publication_stage, "publication");
            issues.extend(collect_stage_issues(results_stage, "results"));
            issues.extend(
                findings_by_stage
                    .get(&contract.stage_id)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow!(
                            "remediation queue missing findings for stage `{}`",
                            contract.stage_id
                        )
                    })?,
            );
            let issue_groups = summarize_issue_groups(&issues);
            let issue_ids = issues
                .iter()
                .map(|issue| issue.issue_id.clone())
                .collect::<Vec<_>>();

            Ok(RemediationStageEntry {
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
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(RemediationQueue {
        corpus_id: corpus_id.to_string(),
        stage_count: stages.len(),
        open_stage_count: stages.iter().filter(|stage| stage.status == "open").count(),
        clear_stage_count: stages
            .iter()
            .filter(|stage| stage.status == "clear")
            .count(),
        stages,
    })
}

fn stage_value_lookup(payload: &serde_json::Value) -> BTreeMap<String, &serde_json::Value> {
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

fn declared_issue_field<'a>(value: &'a serde_json::Value, field: &str) -> Option<&'a str> {
    value
        .get(field)
        .and_then(|entry| entry.as_str())
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
}

fn findings_lookup(payload: &serde_json::Value) -> BTreeMap<String, Vec<RemediationIssue>> {
    let mut findings_by_stage = BTreeMap::new();
    for finding in payload
        .get("findings")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
    {
        let Some(stage_id) = declared_issue_field(finding, "stage_id") else {
            continue;
        };
        let Some(issue_id) = declared_issue_field(finding, "issue_id") else {
            continue;
        };
        let Some(detail) = declared_issue_field(finding, "detail") else {
            continue;
        };
        let Some(severity) = declared_issue_field(finding, "severity") else {
            continue;
        };
        findings_by_stage
            .entry(stage_id.to_string())
            .or_insert_with(Vec::new)
            .push(RemediationIssue {
                issue_id: issue_id.to_string(),
                detail: detail.to_string(),
                severity: severity.to_string(),
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
        .filter_map(|issue| {
            let issue_id = declared_issue_field(issue, "issue_id")?;
            let detail = declared_issue_field(issue, "detail")?;
            let severity = declared_issue_field(issue, "severity")?;
            Some(RemediationIssue {
                issue_id: issue_id.to_string(),
                detail: detail.to_string(),
                severity: severity.to_string(),
                source: source.to_string(),
            })
        })
        .collect()
}

fn summarize_issue_groups(issues: &[RemediationIssue]) -> Vec<RemediationIssueGroup> {
    let mut grouped = BTreeMap::<String, (usize, Vec<String>, Vec<String>, String)>::new();
    for issue in issues {
        let group = grouped
            .entry(issue.issue_id.clone())
            .or_insert_with(|| (0, Vec::new(), Vec::new(), issue.severity.clone()));
        group.0 += 1;
        if !group.1.iter().any(|source| source == &issue.source) {
            group.1.push(issue.source.clone());
        }
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
                sources,
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
        "missing-benchmark-md",
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

pub(super) fn render_remediation_queue_markdown(queue: &RemediationQueue) -> String {
    let mut lines = vec![
        format!("# `{}` FASTQ remediation queue", queue.corpus_id),
        String::new(),
        format!("- Governed publication stages: `{}`", queue.stage_count),
        format!("- Open stages: `{}`", queue.open_stage_count),
        format!("- Clear stages: `{}`", queue.clear_stage_count),
        String::new(),
        "## Stage queue".to_string(),
        String::new(),
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
                stage.run_root_source.as_deref().unwrap_or("missing")
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
