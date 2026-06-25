#![allow(clippy::too_many_arguments, clippy::too_many_lines)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::models::{
    BenchmarkPublicationStatusReport, ExcludedStageEntry, PublicationCorpusSpec,
    PublicationStageReport, StageAuditIssue,
};
use super::{
    append_stage_audit_issue, audit_cohort_runtime_summary, audit_publication_summary,
    audit_sample_results, audit_sample_runtime_outliers, audit_tool_runtime_summary,
    expected_counts_for_scope, publication_method_file_name, relative_to_docs_root,
    relative_to_repo_root, sorted_strings, value_string, CorpusBenchmarkContract,
    CorpusBenchmarkExclusion,
};

pub(super) fn audit_publication_docs(
    repo_root: &Path,
    docs_root: &Path,
    corpus_id: &str,
    contracts: &[CorpusBenchmarkContract],
    exclusions: &[CorpusBenchmarkExclusion],
    corpus_spec: &PublicationCorpusSpec,
    supplemental_findings: &BTreeMap<String, Vec<StageAuditIssue>>,
    results_by_stage: &BTreeMap<String, serde_json::Value>,
    audit_warnings: &[String],
    supplemental_findings_generated_at_utc: Option<String>,
) -> Result<BenchmarkPublicationStatusReport> {
    let stages = contracts
        .iter()
        .map(|contract| {
            audit_publication_stage(
                docs_root,
                contract,
                corpus_id,
                corpus_spec,
                supplemental_findings.get(&contract.stage_id).cloned().ok_or_else(|| {
                    anyhow!(
                        "publication audit missing supplemental findings for stage `{}`",
                        contract.stage_id
                    )
                })?,
                results_by_stage.get(&contract.stage_id),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BenchmarkPublicationStatusReport {
        corpus_id: corpus_id.to_string(),
        docs_root: relative_to_repo_root(docs_root, repo_root),
        benchmarkable_stage_count: contracts.len() + exclusions.len(),
        applicable_stage_count: stages.len(),
        completed_stage_count: stages.iter().filter(|stage| stage.status == "complete").count(),
        incomplete_stage_count: stages.iter().filter(|stage| stage.status != "complete").count(),
        excluded_stage_count: exclusions.len(),
        issue_count: stages.iter().map(|stage| stage.issue_count).sum(),
        audit_warning_count: audit_warnings.len(),
        audit_warnings: audit_warnings.to_vec(),
        supplemental_findings_generated_at_utc,
        excluded_stages: exclusions
            .iter()
            .map(|exclusion| ExcludedStageEntry {
                stage_id: exclusion.stage_id.clone(),
                reason: exclusion.reason.clone(),
            })
            .collect(),
        stages,
    })
}

pub(super) fn audit_publication_stage(
    docs_root: &Path,
    contract: &CorpusBenchmarkContract,
    corpus_id: &str,
    corpus_spec: &PublicationCorpusSpec,
    supplemental_issues: Vec<StageAuditIssue>,
    results_stage: Option<&serde_json::Value>,
) -> Result<PublicationStageReport> {
    let (expected_total, expected_cohort_counts) =
        expected_counts_for_scope(corpus_spec, &contract.sample_scope)?;
    let stage_root = docs_root.join(&contract.stage_id);
    let method_path = stage_root.join(publication_method_file_name(corpus_id));
    let corpus_root = stage_root.join(corpus_id);
    let expected_tools = sorted_strings(&contract.tools);
    let mut issues = Vec::new();

    if !method_path.is_file() {
        append_stage_audit_issue(
            &mut issues,
            &contract.stage_id,
            "missing-method-doc",
            format!("missing {}", relative_to_docs_root(&method_path, docs_root)),
            "error",
        );
    }

    if corpus_root.is_dir() {
        for file_name in [
            "summary.json",
            "sample_results.csv",
            "tool_runtime_summary.csv",
            "cohort_runtime_summary.csv",
            "sample_runtime_outliers.csv",
            "benchmark.md",
        ] {
            let artifact_path = corpus_root.join(file_name);
            if !artifact_path.is_file() {
                append_stage_audit_issue(
                    &mut issues,
                    &contract.stage_id,
                    &format!("missing-{}", file_name.replace('.', "-")),
                    format!("missing {}", relative_to_docs_root(&artifact_path, docs_root)),
                    "error",
                );
                continue;
            }
            if fs::metadata(&artifact_path).is_ok_and(|metadata| metadata.len() == 0) {
                append_stage_audit_issue(
                    &mut issues,
                    &contract.stage_id,
                    &format!("empty-{}", file_name.replace('.', "-")),
                    format!("empty {}", relative_to_docs_root(&artifact_path, docs_root)),
                    "error",
                );
            }
        }

        audit_publication_summary(
            &mut issues,
            docs_root,
            &corpus_root.join("summary.json"),
            contract,
            &expected_tools,
            expected_total,
            &expected_cohort_counts,
        )?;
        audit_sample_results(
            &mut issues,
            docs_root,
            &corpus_root.join("sample_results.csv"),
            contract,
            &expected_tools,
            expected_total,
            &expected_cohort_counts,
        )?;
        audit_tool_runtime_summary(
            &mut issues,
            docs_root,
            &corpus_root.join("tool_runtime_summary.csv"),
            contract,
            &expected_tools,
        )?;
        audit_cohort_runtime_summary(
            &mut issues,
            docs_root,
            &corpus_root.join("cohort_runtime_summary.csv"),
            contract,
            &expected_cohort_counts,
        )?;
        audit_sample_runtime_outliers(
            &mut issues,
            docs_root,
            &corpus_root.join("sample_runtime_outliers.csv"),
            contract,
            expected_total,
        )?;
    } else {
        append_stage_audit_issue(
            &mut issues,
            &contract.stage_id,
            "missing-corpus-dir",
            format!("missing {}", relative_to_docs_root(&corpus_root, docs_root)),
            "error",
        );
    }

    issues.extend(supplemental_issues);
    let results_stage = results_stage.cloned().unwrap_or_else(|| serde_json::json!({}));
    Ok(PublicationStageReport {
        stage_id: contract.stage_id.clone(),
        scenario_id: contract.scenario_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        contract_tool_roster: contract.tools.clone(),
        expected_tool_roster: expected_tools,
        method_path: relative_to_docs_root(&method_path, docs_root),
        corpus_path: relative_to_docs_root(&corpus_root, docs_root),
        status: if issues.is_empty() { "complete".to_string() } else { "incomplete".to_string() },
        issue_count: issues.len(),
        results_status: value_string(&results_stage, "status").unwrap_or("missing").to_string(),
        results_issue_count: results_stage
            .get("issue_count")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(0),
        results_selected_run_root: value_string(&results_stage, "selected_run_root")
            .unwrap_or("missing")
            .to_string(),
        results_newest_available_run_root: value_string(
            &results_stage,
            "newest_available_run_root",
        )
        .unwrap_or("missing")
        .to_string(),
        results_selected_run_root_is_newest: results_stage
            .get("selected_run_root_is_newest")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        issues,
    })
}

pub(super) fn render_publication_docs_markdown(
    report: &BenchmarkPublicationStatusReport,
) -> String {
    let mut lines = vec![
        format!("# `{}` FASTQ benchmark publication status", report.corpus_id),
        String::new(),
        format!("- Benchmarkable governed stages: `{}`", report.benchmarkable_stage_count),
        format!("- Corpus-applicable publication stages: `{}`", report.applicable_stage_count),
        format!("- Completed stage dossiers: `{}`", report.completed_stage_count),
        format!("- Incomplete stage dossiers: `{}`", report.incomplete_stage_count),
        format!("- Excluded stages: `{}`", report.excluded_stage_count),
        format!("- Publication issues: `{}`", report.issue_count),
        format!("- Audit warnings: `{}`", report.audit_warning_count),
        String::new(),
        "## Stage status".to_string(),
        String::new(),
    ];
    for stage in &report.stages {
        lines.push(format!(
            "- `{}`: `{}` (`{}` publication issues, results `{}`, scope `{}`)",
            stage.stage_id,
            stage.status,
            stage.issue_count,
            stage.results_status,
            stage.sample_scope
        ));
        if !stage.results_selected_run_root.is_empty() {
            lines.push(format!(
                "  - selected mirrored run root: `{}`",
                stage.results_selected_run_root
            ));
        }
        if !stage.results_newest_available_run_root.is_empty() {
            lines.push(format!(
                "  - newest mirrored run root: `{}` (selected newest=`{}`)",
                stage.results_newest_available_run_root, stage.results_selected_run_root_is_newest
            ));
        }
        if stage.results_issue_count > 0 {
            lines.push(format!("  - mirrored result issues: `{}`", stage.results_issue_count));
        }
        for issue in &stage.issues {
            lines.push(format!("  - `{}`: {}", issue.issue_id, issue.detail));
        }
    }
    if !report.audit_warnings.is_empty() {
        lines.push(String::new());
        lines.push("## Audit Warnings".to_string());
        lines.push(String::new());
        for warning in &report.audit_warnings {
            lines.push(format!("- {warning}"));
        }
    }
    lines.push(String::new());
    lines.push("## Excluded Stages".to_string());
    lines.push(String::new());
    for exclusion in &report.excluded_stages {
        lines.push(format!("- `{}`: {}", exclusion.stage_id, exclusion.reason));
    }
    lines.push(String::new());
    lines.push("## Contract".to_string());
    lines.push(String::new());
    lines.push(format!(
        "A complete published corpus dossier requires `{}`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `benchmark.md`.",
        publication_method_file_name(&report.corpus_id)
    ));
    lines.push("Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.".to_string());
    lines.join("\n") + "\n"
}
