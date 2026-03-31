use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::models::StageAuditIssue;
use super::publication_io::{
    csv_report_value, csv_required_value, json_string_array, load_csv_rows, load_json_value,
    relative_to_docs_root, sort_count_map, sorted_json_string_array, value_string,
};
use crate::commands::benchmark_workspace::CorpusBenchmarkContract;

pub(super) fn audit_publication_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
    expected_total: usize,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !summary_path.is_file() || fs::metadata(summary_path)?.len() == 0 {
        return Ok(());
    }
    let summary = load_json_value(summary_path)?;
    if value_string(&summary, "stage_id") != Some(contract.stage_id.as_str()) {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-stage-id-drift",
            format!(
                "{} stage_id={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("stage_id").and_then(|value| value.as_str())
            ),
            "error",
        );
    }
    if value_string(&summary, "scenario_id") != Some(contract.scenario_id.as_str()) {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-scenario-id-drift",
            format!(
                "{} scenario_id={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("scenario_id").and_then(|value| value.as_str())
            ),
            "error",
        );
    }
    if sorted_json_string_array(summary.get("tools")) != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-tool-roster-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                json_string_array(summary.get("tools")),
                expected_tools
            ),
            "error",
        );
    }
    if summary
        .get("samples_total")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
        != expected_total
    {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-sample-count-drift",
            format!(
                "{} samples_total={:?} expected {}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("samples_total"),
                expected_total
            ),
            "error",
        );
    }
    if summary
        .get("samples_failed")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        != 0
    {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-sample-failures",
            format!(
                "{} samples_failed={:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("samples_failed")
            ),
            "error",
        );
    }
    if sort_count_map(summary.get("cohort_counts"))? != *expected_cohort_counts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-cohort-count-drift",
            format!(
                "{} cohort_counts={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary.get("cohort_counts"),
                expected_cohort_counts
            ),
            "error",
        );
    }
    let summary_tool_ids = summary
        .get("tool_summary")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("tool").and_then(|value| value.as_str()))
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if summary_tool_ids != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "summary-tool-summary-drift",
            format!(
                "{} tool_summary tools={:?} expected {:?}",
                relative_to_docs_root(summary_path, docs_root),
                summary_tool_ids,
                expected_tools
            ),
            "error",
        );
    }
    Ok(())
}

pub(super) fn audit_sample_results(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    sample_results_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
    expected_total: usize,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !sample_results_path.is_file() || fs::metadata(sample_results_path)?.len() == 0 {
        return Ok(());
    }
    let sample_rows = load_csv_rows(sample_results_path)?;
    if sample_rows.is_empty() {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "empty-sample-results-rows",
            format!(
                "no CSV rows in {}",
                relative_to_docs_root(sample_results_path, docs_root)
            ),
            "error",
        );
        return Ok(());
    }

    let mut per_sample_tools = BTreeMap::<String, Vec<String>>::new();
    let mut sample_metadata = BTreeMap::<String, (String, String, String, String, String)>::new();
    let mut cohort_counts_by_rows = BTreeMap::<String, usize>::new();
    let mut observed_tools = BTreeSet::new();

    for row in &sample_rows {
        let (Some(sample_id), Some(tool)) = (
            csv_required_value(row, "sample_id"),
            csv_required_value(row, "tool"),
        ) else {
            append_stage_audit_issue(
                issues,
                &contract.stage_id,
                "sample-results-missing-sample-or-tool",
                format!(
                    "invalid row in {}",
                    relative_to_docs_root(sample_results_path, docs_root)
                ),
                "error",
            );
            continue;
        };
        observed_tools.insert(tool.clone());
        per_sample_tools
            .entry(sample_id.clone())
            .or_default()
            .push(tool);
        let metadata_tuple = (
            csv_report_value(row, "accession"),
            csv_report_value(row, "era"),
            csv_report_value(row, "layout"),
            csv_report_value(row, "study_accession"),
            csv_report_value(row, "size_band"),
        );
        if let Some(existing) = sample_metadata.get(&sample_id) {
            if existing != &metadata_tuple {
                append_stage_audit_issue(
                    issues,
                    &contract.stage_id,
                    "sample-results-metadata-drift",
                    format!(
                        "{} sample {} metadata differs across rows",
                        relative_to_docs_root(sample_results_path, docs_root),
                        sample_id
                    ),
                    "error",
                );
            }
        } else {
            *cohort_counts_by_rows
                .entry(format!("{}_{}", metadata_tuple.1, metadata_tuple.2))
                .or_default() += 1;
            sample_metadata.insert(sample_id, metadata_tuple);
        }
    }

    let observed_tools = observed_tools.into_iter().collect::<Vec<_>>();
    if observed_tools != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-tool-roster-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(sample_results_path, docs_root),
                observed_tools,
                expected_tools
            ),
            "error",
        );
    }
    if sample_metadata.len() != expected_total {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-sample-count-drift",
            format!(
                "{} unique_samples={:?} expected {}",
                relative_to_docs_root(sample_results_path, docs_root),
                sample_metadata.len(),
                expected_total
            ),
            "error",
        );
    }
    if cohort_counts_by_rows != *expected_cohort_counts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-cohort-count-drift",
            format!(
                "{} cohort_counts={:?} expected {:?}",
                relative_to_docs_root(sample_results_path, docs_root),
                cohort_counts_by_rows,
                expected_cohort_counts
            ),
            "error",
        );
    }
    for (sample_id, tools) in &per_sample_tools {
        let mut sample_tools = tools.clone();
        sample_tools.sort();
        if sample_tools != expected_tools {
            append_stage_audit_issue(
                issues,
                &contract.stage_id,
                "sample-results-tool-coverage-drift",
                format!(
                    "{} sample {} tools={:?} expected {:?}",
                    relative_to_docs_root(sample_results_path, docs_root),
                    sample_id,
                    sample_tools,
                    expected_tools
                ),
                "error",
            );
        }
    }
    let expected_row_count = expected_total * expected_tools.len();
    if sample_rows.len() != expected_row_count {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-results-row-count-drift",
            format!(
                "{} row_count={:?} expected {}",
                relative_to_docs_root(sample_results_path, docs_root),
                sample_rows.len(),
                expected_row_count
            ),
            "error",
        );
    }
    Ok(())
}

pub(super) fn append_stage_audit_issue(
    issues: &mut Vec<StageAuditIssue>,
    stage_id: &str,
    issue_id: &str,
    detail: String,
    severity: &str,
) {
    issues.push(StageAuditIssue {
        stage_id: stage_id.to_string(),
        issue_id: issue_id.to_string(),
        severity: severity.to_string(),
        detail,
    });
}
