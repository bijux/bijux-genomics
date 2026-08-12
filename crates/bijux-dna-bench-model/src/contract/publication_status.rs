//! Semantic validation for persisted benchmark publication audit status.

use std::collections::BTreeSet;

use crate::{
    validate_utc_timestamp, BenchError, BenchmarkPublicationStageStatus,
    BenchmarkPublicationStatusReport, BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1,
};

/// Validate publication coverage, audit findings, result roots, and counters.
///
/// # Errors
/// Returns an error when the publication status is incomplete or internally inconsistent.
pub fn validate_benchmark_publication_status(
    report: &BenchmarkPublicationStatusReport,
) -> Result<(), BenchError> {
    validate_report_identity(report)?;
    let excluded_ids = validate_excluded_stages(report)?;
    validate_stage_rows(report, &excluded_ids)
}

fn validate_report_identity(report: &BenchmarkPublicationStatusReport) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1 {
        return invalid("unsupported benchmark publication status schema");
    }
    require_text(&report.corpus_id, "corpus_id")?;
    require_text(&report.docs_root, "docs_root")?;
    if report.benchmarkable_stage_count
        != report.applicable_stage_count + report.excluded_stage_count
    {
        return invalid("publication benchmarkable count does not partition applicable/excluded");
    }
    if report.applicable_stage_count != report.stages.len()
        || report.excluded_stage_count != report.excluded_stages.len()
    {
        return invalid("publication coverage counters do not match persisted rows");
    }
    if report.completed_stage_count + report.incomplete_stage_count != report.stages.len() {
        return invalid("publication completion counters do not partition stage rows");
    }
    if report.issue_count != report.stages.iter().map(|stage| stage.issue_count).sum::<usize>() {
        return invalid("publication issue_count does not match stage issues");
    }
    if report.audit_warning_count != report.audit_warnings.len() {
        return invalid("publication audit_warning_count does not match audit_warnings");
    }
    let warnings = report.audit_warnings.iter().collect::<BTreeSet<_>>();
    if warnings.len() != report.audit_warnings.len()
        || report.audit_warnings.iter().any(|warning| warning.trim().is_empty())
    {
        return invalid("publication audit warnings must be unique and non-empty");
    }
    if let Some(timestamp) = report.supplemental_findings_generated_at_utc.as_deref() {
        validate_utc_timestamp(timestamp, "supplemental_findings_generated_at_utc")?;
    }
    Ok(())
}

fn validate_excluded_stages(
    report: &BenchmarkPublicationStatusReport,
) -> Result<BTreeSet<&str>, BenchError> {
    let mut excluded_ids = BTreeSet::new();
    for stage in &report.excluded_stages {
        require_text(&stage.stage_id, "excluded_stages[].stage_id")?;
        require_text(&stage.reason, "excluded_stages[].reason")?;
        if !excluded_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("publication repeats excluded stage `{}`", stage.stage_id));
        }
    }
    Ok(excluded_ids)
}

fn validate_stage_rows(
    report: &BenchmarkPublicationStatusReport,
    excluded_ids: &BTreeSet<&str>,
) -> Result<(), BenchError> {
    let mut stage_ids = BTreeSet::new();
    let mut completed_count = 0usize;
    for stage in &report.stages {
        validate_stage_identity(stage)?;
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("publication repeats stage `{}`", stage.stage_id));
        }
        if excluded_ids.contains(stage.stage_id.as_str()) {
            return invalid(format!(
                "publication stage `{}` is both applicable and excluded",
                stage.stage_id
            ));
        }
        if stage.status == "complete" {
            completed_count += 1;
        }
        validate_stage_status(stage)?;
        validate_stage_results(stage)?;
        validate_stage_issues(stage)?;
    }
    if completed_count != report.completed_stage_count
        || report.stages.len() - completed_count != report.incomplete_stage_count
    {
        return invalid("publication completion counters do not match stage statuses");
    }
    Ok(())
}

fn validate_stage_identity(stage: &BenchmarkPublicationStageStatus) -> Result<(), BenchError> {
    for (value, field) in [
        (&stage.stage_id, "stages[].stage_id"),
        (&stage.scenario_id, "stages[].scenario_id"),
        (&stage.sample_scope, "stages[].sample_scope"),
        (&stage.method_path, "stages[].method_path"),
        (&stage.corpus_path, "stages[].corpus_path"),
    ] {
        require_text(value, field)?;
    }
    if stage.contract_tool_roster.is_empty()
        || stage.contract_tool_roster.iter().any(|tool| tool.trim().is_empty())
        || stage.contract_tool_roster.iter().collect::<BTreeSet<_>>().len()
            != stage.contract_tool_roster.len()
    {
        return invalid(format!(
            "publication stage `{}` has an invalid tool roster",
            stage.stage_id
        ));
    }
    if stage.contract_tool_roster != stage.expected_tool_roster {
        return invalid(format!(
            "publication stage `{}` expected tool roster drifted from its contract",
            stage.stage_id
        ));
    }
    Ok(())
}

fn validate_stage_status(stage: &BenchmarkPublicationStageStatus) -> Result<(), BenchError> {
    if stage.issue_count != stage.issues.len() {
        return invalid(format!(
            "publication stage `{}` issue_count does not match issues",
            stage.stage_id
        ));
    }
    match stage.status.as_str() {
        "complete" if stage.issues.is_empty() => Ok(()),
        "incomplete" if !stage.issues.is_empty() => Ok(()),
        "complete" | "incomplete" => invalid(format!(
            "publication stage `{}` status contradicts issue evidence",
            stage.stage_id
        )),
        _ => invalid(format!(
            "publication stage `{}` has unknown status `{}`",
            stage.stage_id, stage.status
        )),
    }
}

fn validate_stage_results(stage: &BenchmarkPublicationStageStatus) -> Result<(), BenchError> {
    if !matches!(stage.results_status.as_str(), "complete" | "incomplete" | "missing") {
        return invalid(format!(
            "publication stage `{}` has unknown results status `{}`",
            stage.stage_id, stage.results_status
        ));
    }
    if stage.results_issue_count > stage.issue_count {
        return invalid(format!(
            "publication stage `{}` results issues exceed total issues",
            stage.stage_id
        ));
    }
    if stage.results_status == "complete" && stage.results_issue_count != 0 {
        return invalid(format!(
            "publication stage `{}` has complete results with result issues",
            stage.stage_id
        ));
    }
    let selected_available = root_is_available(&stage.results_selected_run_root);
    let newest_available = root_is_available(&stage.results_newest_available_run_root);
    if selected_available != newest_available {
        return invalid(format!(
            "publication stage `{}` has a partial result-root selection",
            stage.stage_id
        ));
    }
    if stage.results_selected_run_root_is_newest
        != (selected_available
            && stage.results_selected_run_root == stage.results_newest_available_run_root)
    {
        return invalid(format!(
            "publication stage `{}` result-root flag contradicts roots",
            stage.stage_id
        ));
    }
    if stage.results_status == "complete" && !stage.results_selected_run_root_is_newest {
        return invalid(format!(
            "publication stage `{}` complete results do not select the newest root",
            stage.stage_id
        ));
    }
    Ok(())
}

fn root_is_available(value: &str) -> bool {
    !value.is_empty() && value != "missing"
}

fn validate_stage_issues(stage: &BenchmarkPublicationStageStatus) -> Result<(), BenchError> {
    let mut issue_ids = BTreeSet::new();
    for issue in &stage.issues {
        if issue.stage_id != stage.stage_id {
            return invalid(format!(
                "publication stage `{}` contains an issue for `{}`",
                stage.stage_id, issue.stage_id
            ));
        }
        require_text(&issue.issue_id, "stages[].issues[].issue_id")?;
        require_text(&issue.detail, "stages[].issues[].detail")?;
        if !matches!(issue.severity.as_str(), "error" | "warning") {
            return invalid(format!(
                "publication stage `{}` has unknown issue severity `{}`",
                stage.stage_id, issue.severity
            ));
        }
        if !issue_ids.insert(issue.issue_id.as_str()) {
            return invalid(format!(
                "publication stage `{}` repeats issue `{}`",
                stage.stage_id, issue.issue_id
            ));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("benchmark publication status requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
