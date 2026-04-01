use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::models::StageAuditIssue;
use super::publication_io::{
    csv_report_value, csv_required_value, csv_value, load_csv_rows, relative_to_docs_root,
};
use super::report_audit::append_stage_audit_issue;
use crate::commands::benchmark_workspace::CorpusBenchmarkContract;

pub(super) fn audit_tool_runtime_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    tool_runtime_summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_tools: &[String],
) -> Result<()> {
    if !tool_runtime_summary_path.is_file() || fs::metadata(tool_runtime_summary_path)?.len() == 0 {
        return Ok(());
    }
    let mut observed_tools = load_csv_rows(tool_runtime_summary_path)?
        .into_iter()
        .filter_map(|row| csv_required_value(&row, "tool"))
        .collect::<Vec<_>>();
    observed_tools.sort();
    if observed_tools != expected_tools {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "tool-runtime-summary-drift",
            format!(
                "{} tools={:?} expected {:?}",
                relative_to_docs_root(tool_runtime_summary_path, docs_root),
                observed_tools,
                expected_tools
            ),
            "error",
        );
    }
    Ok(())
}

pub(super) fn audit_cohort_runtime_summary(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    cohort_runtime_summary_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_cohort_counts: &BTreeMap<String, usize>,
) -> Result<()> {
    if !cohort_runtime_summary_path.is_file()
        || fs::metadata(cohort_runtime_summary_path)?.len() == 0
    {
        return Ok(());
    }
    let observed_cohorts = load_csv_rows(cohort_runtime_summary_path)?
        .into_iter()
        .filter(|row| {
            let dimension = csv_report_value(row, "dimension");
            dimension == "missing" || dimension == "era_layout"
        })
        .map(|row| csv_report_value(&row, "cohort"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected_cohorts = expected_cohort_counts.keys().cloned().collect::<Vec<_>>();
    if observed_cohorts != expected_cohorts {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "cohort-runtime-summary-drift",
            format!(
                "{} cohorts={:?} expected {:?}",
                relative_to_docs_root(cohort_runtime_summary_path, docs_root),
                observed_cohorts,
                expected_cohorts
            ),
            "error",
        );
    }
    Ok(())
}

pub(super) fn audit_sample_runtime_outliers(
    issues: &mut Vec<StageAuditIssue>,
    docs_root: &Path,
    sample_runtime_outliers_path: &Path,
    contract: &CorpusBenchmarkContract,
    expected_total: usize,
) -> Result<()> {
    if !sample_runtime_outliers_path.is_file()
        || fs::metadata(sample_runtime_outliers_path)?.len() == 0
    {
        return Ok(());
    }
    let unique_sample_ids = load_csv_rows(sample_runtime_outliers_path)?
        .into_iter()
        .map(|row| csv_value(&row, "sample_id"))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if unique_sample_ids.len() != expected_total {
        append_stage_audit_issue(
            issues,
            &contract.stage_id,
            "sample-runtime-outlier-coverage-drift",
            format!(
                "{} unique_samples={:?} expected {}",
                relative_to_docs_root(sample_runtime_outliers_path, docs_root),
                unique_sample_ids.len(),
                expected_total
            ),
            "error",
        );
    }
    Ok(())
}
