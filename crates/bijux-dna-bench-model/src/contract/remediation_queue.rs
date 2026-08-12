//! Semantic validation for persisted benchmark remediation queues.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    validate_utc_timestamp, BenchError, BenchmarkRemediationIssue, BenchmarkRemediationIssueGroup,
    BenchmarkRemediationQueue, BenchmarkRemediationStageEntry,
    BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1,
};

/// Validate remediation identity, counters, evidence groups, and recommended actions.
///
/// # Errors
/// Returns an error when the persisted queue contradicts its issue evidence.
pub fn validate_benchmark_remediation_queue(
    queue: &BenchmarkRemediationQueue,
) -> Result<(), BenchError> {
    if queue.schema_version != BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1 {
        return invalid("unsupported benchmark remediation queue schema");
    }
    require_text(&queue.corpus_id, "corpus_id")?;
    if queue.stage_count == 0 || queue.stage_count != queue.stages.len() {
        return invalid("remediation stage_count must equal a non-empty stage queue");
    }
    if queue.open_stage_count + queue.clear_stage_count != queue.stage_count {
        return invalid("remediation open/clear counters do not partition the stage queue");
    }

    let mut stage_ids = BTreeSet::new();
    let mut open_count = 0usize;
    for stage in &queue.stages {
        validate_stage(stage, &mut stage_ids)?;
        if stage.status == "open" {
            open_count += 1;
        }
    }
    if open_count != queue.open_stage_count
        || queue.stages.len() - open_count != queue.clear_stage_count
    {
        return invalid("remediation counters do not match stage statuses");
    }
    Ok(())
}

fn validate_stage<'a>(
    stage: &'a BenchmarkRemediationStageEntry,
    stage_ids: &mut BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&stage.stage_id, "stages[].stage_id"),
        (&stage.owner, "stages[].owner"),
        (&stage.sample_scope, "stages[].sample_scope"),
    ] {
        require_text(value, field)?;
    }
    if !stage_ids.insert(stage.stage_id.as_str()) {
        return invalid(format!("remediation repeats stage `{}`", stage.stage_id));
    }
    if stage.owner != "benchmark-governance" {
        return invalid(format!(
            "remediation stage `{}` has unknown owner `{}`",
            stage.stage_id, stage.owner
        ));
    }
    validate_stage_status(stage)?;
    validate_stage_publication(stage)?;
    validate_stage_issues(stage)?;
    validate_issue_groups(stage)
}

fn validate_stage_status(stage: &BenchmarkRemediationStageEntry) -> Result<(), BenchError> {
    if stage.issue_count != stage.issues.len()
        || stage.issue_group_count != stage.issue_groups.len()
    {
        return invalid(format!(
            "remediation stage `{}` counters do not match issue evidence",
            stage.stage_id
        ));
    }
    match stage.status.as_str() {
        "clear" if stage.issues.is_empty() && stage.recommended_action == "none" => Ok(()),
        "open" if !stage.issues.is_empty() && stage.recommended_action != "none" => Ok(()),
        "clear" | "open" => invalid(format!(
            "remediation stage `{}` status contradicts its issues or action",
            stage.stage_id
        )),
        _ => invalid(format!(
            "remediation stage `{}` has unknown status `{}`",
            stage.stage_id, stage.status
        )),
    }?;
    if !matches!(
        stage.recommended_action.as_str(),
        "none"
            | "sync-or-normalize-results"
            | "render-or-publish-dossier"
            | "rerun-benchmark-stage"
            | "repair-benchmark-contract"
    ) {
        return invalid(format!(
            "remediation stage `{}` has unknown action `{}`",
            stage.stage_id, stage.recommended_action
        ));
    }
    let expected_action = expected_remediation_action(&stage.issues);
    if stage.recommended_action != expected_action {
        return invalid(format!(
            "remediation stage `{}` action `{}` contradicts issue evidence; expected `{expected_action}`",
            stage.stage_id, stage.recommended_action
        ));
    }
    Ok(())
}

fn expected_remediation_action(issues: &[BenchmarkRemediationIssue]) -> &'static str {
    if issues.is_empty() {
        return "none";
    }
    let issue_ids = issues.iter().map(|issue| issue.issue_id.as_str()).collect::<Vec<_>>();
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
    if issue_ids.iter().any(|issue_id| sync_issue_ids.contains(issue_id)) {
        return "sync-or-normalize-results";
    }
    if issue_ids.iter().any(|issue_id| publish_issue_ids.contains(issue_id)) {
        return "render-or-publish-dossier";
    }
    if issue_ids.iter().any(|issue_id| {
        ["sample-failures", "dry-run", "sample-limit"].iter().any(|part| issue_id.contains(part))
    }) {
        return "rerun-benchmark-stage";
    }
    "repair-benchmark-contract"
}

fn validate_stage_publication(stage: &BenchmarkRemediationStageEntry) -> Result<(), BenchError> {
    for (value, field) in [
        (&stage.publication_status, "publication_status"),
        (&stage.results_status, "results_status"),
    ] {
        if !matches!(value.as_str(), "complete" | "incomplete" | "missing") {
            return invalid(format!(
                "remediation stage `{}` has unknown {field} `{value}`",
                stage.stage_id
            ));
        }
    }
    if stage.status == "clear"
        && (stage.publication_status != "complete" || stage.results_status != "complete")
    {
        return invalid(format!(
            "remediation stage `{}` is clear without complete publication and results",
            stage.stage_id
        ));
    }
    match (stage.published_generated_at_utc.as_deref(), stage.run_root_source.as_deref()) {
        (None, None) if stage.publication_status != "complete" => Ok(()),
        (None, None) => invalid(format!(
            "remediation stage `{}` has complete publication without dossier metadata",
            stage.stage_id
        )),
        (Some(timestamp), Some(source)) => {
            validate_utc_timestamp(timestamp, "stages[].published_generated_at_utc")?;
            if !matches!(
                source,
                "local-cache-mirror"
                    | "local-results-root"
                    | "remote-results-root"
                    | "remote-custom"
                    | "custom"
            ) {
                return invalid(format!(
                    "remediation stage `{}` has unknown run-root source `{source}`",
                    stage.stage_id
                ));
            }
            Ok(())
        }
        _ => invalid(format!(
            "remediation stage `{}` has partial publication metadata",
            stage.stage_id
        )),
    }
}

fn validate_stage_issues(stage: &BenchmarkRemediationStageEntry) -> Result<(), BenchError> {
    for issue in &stage.issues {
        require_text(&issue.issue_id, "stages[].issues[].issue_id")?;
        require_text(&issue.detail, "stages[].issues[].detail")?;
        if !matches!(issue.severity.as_str(), "error" | "warning") {
            return invalid(format!(
                "remediation stage `{}` has unknown issue severity `{}`",
                stage.stage_id, issue.severity
            ));
        }
        if !matches!(issue.source.as_str(), "publication" | "results" | "findings") {
            return invalid(format!(
                "remediation stage `{}` has unknown issue source `{}`",
                stage.stage_id, issue.source
            ));
        }
    }
    Ok(())
}

fn validate_issue_groups(stage: &BenchmarkRemediationStageEntry) -> Result<(), BenchError> {
    let mut groups = BTreeMap::new();
    for group in &stage.issue_groups {
        require_text(&group.issue_id, "stages[].issue_groups[].issue_id")?;
        if groups.insert(group.issue_id.as_str(), group).is_some() {
            return invalid(format!(
                "remediation stage `{}` repeats issue group `{}`",
                stage.stage_id, group.issue_id
            ));
        }
        validate_issue_group(stage, group)?;
    }
    let issue_ids =
        stage.issues.iter().map(|issue| issue.issue_id.as_str()).collect::<BTreeSet<_>>();
    if issue_ids != groups.keys().copied().collect::<BTreeSet<_>>() {
        return invalid(format!(
            "remediation stage `{}` issue groups do not cover its issues",
            stage.stage_id
        ));
    }
    Ok(())
}

fn validate_issue_group(
    stage: &BenchmarkRemediationStageEntry,
    group: &BenchmarkRemediationIssueGroup,
) -> Result<(), BenchError> {
    let matching =
        stage.issues.iter().filter(|issue| issue.issue_id == group.issue_id).collect::<Vec<_>>();
    if matching.is_empty() || group.count != matching.len() {
        return invalid(format!(
            "remediation stage `{}` group `{}` has the wrong count",
            stage.stage_id, group.issue_id
        ));
    }
    validate_group_sources_and_severity(stage, group, &matching)?;
    validate_group_details(stage, group, &matching)
}

fn validate_group_sources_and_severity(
    stage: &BenchmarkRemediationStageEntry,
    group: &BenchmarkRemediationIssueGroup,
    matching: &[&BenchmarkRemediationIssue],
) -> Result<(), BenchError> {
    let expected_sources =
        matching.iter().map(|issue| issue.source.as_str()).collect::<BTreeSet<_>>();
    let actual_sources = group.sources.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if group.sources.iter().any(|source| source.trim().is_empty())
        || actual_sources.len() != group.sources.len()
        || expected_sources != actual_sources
    {
        return invalid(format!(
            "remediation stage `{}` group `{}` has source drift",
            stage.stage_id, group.issue_id
        ));
    }
    let severities = matching.iter().map(|issue| issue.severity.as_str()).collect::<BTreeSet<_>>();
    if severities.len() != 1 || !severities.contains(group.severity.as_str()) {
        return invalid(format!(
            "remediation stage `{}` group `{}` has severity drift",
            stage.stage_id, group.issue_id
        ));
    }
    Ok(())
}

fn validate_group_details(
    stage: &BenchmarkRemediationStageEntry,
    group: &BenchmarkRemediationIssueGroup,
    matching: &[&BenchmarkRemediationIssue],
) -> Result<(), BenchError> {
    let details = matching.iter().map(|issue| issue.detail.as_str()).collect::<BTreeSet<_>>();
    let examples = group.example_details.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if group.example_details.is_empty()
        || group.example_details.len() > 3
        || examples.len() != group.example_details.len()
        || !examples.is_subset(&details)
        || group.additional_detail_count + group.example_details.len() != details.len()
    {
        return invalid(format!(
            "remediation stage `{}` group `{}` has detail-summary drift",
            stage.stage_id, group.issue_id
        ));
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("benchmark remediation queue requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
