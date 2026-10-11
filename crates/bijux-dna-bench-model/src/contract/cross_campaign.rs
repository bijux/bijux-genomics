//! Owner: bijux-dna-bench-model
//! Admission rules for persisted cross-domain benchmark campaign records.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{
    AppraisalFinding, CrossBenchmarkCampaignReport, CrossBenchmarkGoalEntry, HardeningQueueEntry,
    CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};

/// Validate campaign identity, selected-goal closure, nested records, and exact summary totals.
///
/// # Errors
/// Returns an error for an unsupported schema, malformed or duplicate nested identities, an
/// invalid status vocabulary, or aggregate drift.
pub fn validate_cross_benchmark_campaign_report(
    report: &CrossBenchmarkCampaignReport,
) -> Result<(), BenchError> {
    if report.schema_version != CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1
        || report.campaign_id.trim().is_empty()
        || report.domain != "cross"
        || report.selected_goals.is_empty()
    {
        return Err(invalid("cross-benchmark campaign identity is invalid"));
    }
    validate_nonempty_unique(&report.selected_goals, "selected goals")?;
    if report.selected_goals.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid("cross-benchmark selected goals are not in canonical order"));
    }

    let entry_goals = report.entries.iter().map(|entry| entry.goal_id.clone()).collect::<Vec<_>>();
    if entry_goals != report.selected_goals {
        return Err(invalid("cross-benchmark entries do not close over selected goals"));
    }

    let mut status_counts = BTreeMap::new();
    let mut total_rows = 0_usize;
    let mut total_findings = 0_usize;
    let mut total_queue_entries = 0_usize;
    for entry in &report.entries {
        validate_goal_entry(entry)?;
        *status_counts.entry(entry.status.clone()).or_insert(0_usize) += 1;
        total_rows += entry.matched_rows.len();
        total_findings += entry.appraisal_findings.len();
        total_queue_entries += entry.hardening_entries.len();
    }

    if report.summary.total_goals != report.entries.len()
        || report.summary.total_rows != total_rows
        || report.summary.total_findings != total_findings
        || report.summary.total_queue_entries != total_queue_entries
        || report.summary.status_counts != status_counts
    {
        return Err(invalid("cross-benchmark campaign summary drifted"));
    }
    Ok(())
}

fn validate_goal_entry(entry: &CrossBenchmarkGoalEntry) -> Result<(), BenchError> {
    if entry.goal_id.trim().is_empty()
        || entry.title.trim().is_empty()
        || entry.stage_ids.is_empty()
        || entry.scenarios.is_empty()
        || entry.goal_checks.is_empty()
        || !matches!(
            entry.status.as_str(),
            "missing-stage-binding" | "requires-hardening" | "ready-for-benchmark-run"
        )
    {
        return Err(invalid("cross-benchmark goal entry is invalid"));
    }
    validate_nonempty_unique(&entry.stage_ids, "goal stage identities")?;
    validate_nonempty_unique(&entry.goal_checks, "goal checks")?;

    let mut scenarios = BTreeSet::new();
    for scenario in &entry.scenarios {
        if scenario.scenario_id.trim().is_empty()
            || scenario.focus.trim().is_empty()
            || !scenarios.insert(scenario.scenario_id.as_str())
        {
            return Err(invalid("cross-benchmark scenario is invalid"));
        }
    }

    let mut rows = BTreeSet::new();
    for row in &entry.matched_rows {
        if row.row_id.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || !matches!(row.readiness_class.as_str(), "ready" | "degraded" | "refuse")
            || !rows.insert(row.row_id.as_str())
        {
            return Err(invalid("cross-benchmark matrix row reference is invalid"));
        }
    }
    let expected_status = if entry.matched_rows.is_empty() {
        "missing-stage-binding"
    } else if entry.matched_rows.iter().any(|row| row.readiness_class == "refuse")
        || !entry.hardening_entries.is_empty()
    {
        "requires-hardening"
    } else {
        "ready-for-benchmark-run"
    };
    if entry.status != expected_status {
        return Err(invalid("cross-benchmark goal status does not match its evidence"));
    }

    let mut findings = BTreeSet::new();
    for finding in &entry.appraisal_findings {
        validate_finding(finding, &rows)?;
        if !findings.insert((
            finding.appraiser_id.as_str(),
            finding.row_id.as_str(),
            finding.failure_class.as_str(),
            finding.result_scope.as_str(),
        )) {
            return Err(invalid("cross-benchmark appraisal finding is duplicated"));
        }
    }
    let mut queue_ids = BTreeSet::new();
    for queue_entry in &entry.hardening_entries {
        validate_queue_entry(queue_entry, &rows)?;
        if !queue_ids.insert(queue_entry.queue_id.as_str()) {
            return Err(invalid("cross-benchmark hardening entry is duplicated"));
        }
    }
    Ok(())
}

fn validate_finding(finding: &AppraisalFinding, rows: &BTreeSet<&str>) -> Result<(), BenchError> {
    if finding.appraiser_id.trim().is_empty()
        || finding.row_id.trim().is_empty()
        || !rows.contains(finding.row_id.as_str())
        || !matches!(finding.severity.as_str(), "critical" | "warning" | "info")
        || !matches!(finding.confidence.as_str(), "high" | "medium" | "low")
        || finding.failure_class.trim().is_empty()
        || !matches!(finding.result_scope.as_str(), "encrypted-results" | "encrypted-code")
        || finding.summary.trim().is_empty()
        || finding.recommendation.trim().is_empty()
    {
        return Err(invalid("cross-benchmark appraisal finding is invalid"));
    }
    Ok(())
}

fn validate_queue_entry(
    entry: &HardeningQueueEntry,
    rows: &BTreeSet<&str>,
) -> Result<(), BenchError> {
    if entry.queue_id.trim().is_empty()
        || !matches!(entry.severity.as_str(), "critical" | "warning" | "info")
        || entry.failure_class.trim().is_empty()
        || entry.recommendation.trim().is_empty()
        || entry.affected_rows.is_empty()
        || entry.source_appraisers.is_empty()
        || entry.affected_rows.iter().all(|row| !rows.contains(row.as_str()))
    {
        return Err(invalid("cross-benchmark hardening entry is invalid"));
    }
    validate_nonempty_unique(&entry.affected_rows, "hardening affected rows")?;
    validate_nonempty_unique(&entry.source_appraisers, "hardening source appraisers")?;
    Ok(())
}

fn validate_nonempty_unique(values: &[String], label: &str) -> Result<(), BenchError> {
    let mut seen = BTreeSet::new();
    if values.iter().any(|value| value.trim().is_empty() || !seen.insert(value.as_str())) {
        return Err(invalid(format!("cross-benchmark {label} are invalid")));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
