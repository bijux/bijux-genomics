//! Semantic validation for published benchmark result mirror status.

use std::collections::BTreeSet;

use crate::{
    BenchError, PublishedResultsStageReport, PublishedResultsStatusReport,
    BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1,
};

/// Validate one persisted published-results status report.
///
/// # Errors
/// Returns an error when identities, counters, statuses, roots, or issues are inconsistent.
pub fn validate_benchmark_published_results_status(
    report: &PublishedResultsStatusReport,
) -> Result<(), BenchError> {
    validate_report_counters(report)?;
    validate_stage_rows(report)
}

fn validate_report_counters(report: &PublishedResultsStatusReport) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1 {
        return invalid("unsupported published-results status schema");
    }
    require_text(&report.corpus_id, "corpus_id")?;
    if report.applicable_stage_count != report.stages.len() {
        return invalid("published-results applicable_stage_count does not match stage rows");
    }
    if report.published_stage_count > report.applicable_stage_count {
        return invalid("published-results published_stage_count exceeds applicable stages");
    }
    let published_stage_count = report
        .stages
        .iter()
        .filter(|stage| {
            !stage.issues.iter().any(|issue| issue.issue_id == "missing-published-summary")
        })
        .count();
    if report.published_stage_count != published_stage_count {
        return invalid(
            "published-results published_stage_count does not match missing-summary evidence",
        );
    }
    if report.complete_stage_count > report.published_stage_count {
        return invalid("published-results complete_stage_count exceeds published stages");
    }
    if report.complete_stage_count + report.incomplete_stage_count != report.stages.len() {
        return invalid("published-results completion counters do not partition stage rows");
    }
    if report.issue_count != report.stages.iter().map(|stage| stage.issue_count).sum::<usize>() {
        return invalid("published-results issue_count does not match stage issues");
    }
    Ok(())
}

fn validate_stage_rows(report: &PublishedResultsStatusReport) -> Result<(), BenchError> {
    let mut stage_ids = BTreeSet::new();
    let mut complete_count = 0usize;
    for stage in &report.stages {
        require_text(&stage.stage_id, "stages[].stage_id")?;
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("published-results repeats stage `{}`", stage.stage_id));
        }
        if stage.issue_count != stage.issues.len() {
            return invalid(format!(
                "published-results stage `{}` issue_count does not match issues",
                stage.stage_id
            ));
        }
        match stage.status.as_str() {
            "complete" => {
                complete_count += 1;
                if !stage.issues.is_empty() {
                    return invalid(format!(
                        "published-results complete stage `{}` contains issues",
                        stage.stage_id
                    ));
                }
                for (value, field) in [
                    (&stage.reported_run_root, "reported_run_root"),
                    (&stage.selected_run_root, "selected_run_root"),
                    (&stage.newest_available_run_root, "newest_available_run_root"),
                ] {
                    require_text(value, field)?;
                }
                if !stage.selected_run_root_is_newest {
                    return invalid(format!(
                        "published-results complete stage `{}` does not select the newest root",
                        stage.stage_id
                    ));
                }
                if !stage.available_run_roots.contains(&stage.selected_run_root) {
                    return invalid(format!(
                        "published-results complete stage `{}` omits its selected root",
                        stage.stage_id
                    ));
                }
            }
            "incomplete" => {
                if stage.issues.is_empty() {
                    return invalid(format!(
                        "published-results incomplete stage `{}` has no issue evidence",
                        stage.stage_id
                    ));
                }
            }
            _ => {
                return invalid(format!(
                    "published-results stage `{}` has unknown status `{}`",
                    stage.stage_id, stage.status
                ));
            }
        }
        validate_stage_roots(stage)?;
        validate_stage_issues(stage)?;
    }
    if complete_count != report.complete_stage_count {
        return invalid("published-results complete_stage_count does not match stage statuses");
    }
    if report.stages.len() - complete_count != report.incomplete_stage_count {
        return invalid("published-results incomplete_stage_count does not match stage statuses");
    }
    Ok(())
}

fn validate_stage_roots(stage: &PublishedResultsStageReport) -> Result<(), BenchError> {
    if stage.selected_run_root.is_empty() != stage.newest_available_run_root.is_empty() {
        return invalid(format!(
            "published-results stage `{}` has a partial run-root selection",
            stage.stage_id
        ));
    }
    if stage.selected_run_root_is_newest
        != (!stage.selected_run_root.is_empty()
            && stage.selected_run_root == stage.newest_available_run_root)
    {
        return invalid(format!(
            "published-results stage `{}` newest-root flag contradicts roots",
            stage.stage_id
        ));
    }
    let unique_roots = stage.available_run_roots.iter().collect::<BTreeSet<_>>();
    if unique_roots.len() != stage.available_run_roots.len()
        || stage.available_run_roots.iter().any(|root| root.trim().is_empty())
    {
        return invalid(format!(
            "published-results stage `{}` has invalid available run roots",
            stage.stage_id
        ));
    }
    Ok(())
}

fn validate_stage_issues(stage: &PublishedResultsStageReport) -> Result<(), BenchError> {
    let mut issue_ids = BTreeSet::new();
    for issue in &stage.issues {
        if issue.stage_id != stage.stage_id {
            return invalid(format!(
                "published-results stage `{}` contains an issue for `{}`",
                stage.stage_id, issue.stage_id
            ));
        }
        require_text(&issue.issue_id, "stages[].issues[].issue_id")?;
        require_text(&issue.detail, "stages[].issues[].detail")?;
        if !issue_ids.insert(issue.issue_id.as_str()) {
            return invalid(format!(
                "published-results stage `{}` repeats issue `{}`",
                stage.stage_id, issue.issue_id
            ));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("published-results requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
