//! Owner: bijux-dna-bench-model
//! Admission rules for persisted benchmark appraisal and hardening-queue records.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{
    AppraisalFinding, AppraisalReport, HardeningQueueEntry, HardeningQueueReport,
    APPRAISAL_REPORT_SCHEMA_V1, HARDENING_QUEUE_REPORT_SCHEMA_V1,
};

/// Validate appraisal identity, finding vocabulary, canonical order, and exact summary closure.
///
/// # Errors
/// Returns an error for unsupported schemas, incomplete findings, duplicates, or summary drift.
pub fn validate_appraisal_report(report: &AppraisalReport) -> Result<(), BenchError> {
    if report.schema_version != APPRAISAL_REPORT_SCHEMA_V1
        || report.campaign_id.trim().is_empty()
        || report.domain.trim().is_empty()
    {
        return Err(invalid("appraisal report identity is invalid"));
    }

    let mut identities = BTreeSet::new();
    let mut by_appraiser = BTreeMap::new();
    let mut by_severity = BTreeMap::new();
    for finding in &report.findings {
        validate_finding(finding)?;
        let identity = (
            finding.appraiser_id.as_str(),
            finding.row_id.as_str(),
            finding.failure_class.as_str(),
            finding.result_scope.as_str(),
        );
        if !identities.insert(identity) {
            return Err(invalid("appraisal report contains a duplicate finding"));
        }
        *by_appraiser.entry(finding.appraiser_id.clone()).or_insert(0_usize) += 1;
        *by_severity.entry(finding.severity.clone()).or_insert(0_usize) += 1;
    }
    if report.findings.windows(2).any(|pair| finding_order(&pair[0], &pair[1]).is_gt()) {
        return Err(invalid("appraisal findings are not in canonical order"));
    }
    if report.summary.total_findings != report.findings.len()
        || report.summary.by_appraiser != by_appraiser
        || report.summary.by_severity != by_severity
    {
        return Err(invalid("appraisal report summary drifted"));
    }
    Ok(())
}

/// Validate hardening-queue identity, deterministic queue allocation, and grouped finding links.
///
/// # Errors
/// Returns an error for unsupported schemas, malformed entries, or non-canonical ordering.
pub fn validate_hardening_queue_report(report: &HardeningQueueReport) -> Result<(), BenchError> {
    if report.schema_version != HARDENING_QUEUE_REPORT_SCHEMA_V1
        || report.campaign_id.trim().is_empty()
        || report.domain.trim().is_empty()
    {
        return Err(invalid("hardening queue report identity is invalid"));
    }
    for (index, entry) in report.entries.iter().enumerate() {
        if entry.queue_id != format!("hardening-{:04}", index + 1) {
            return Err(invalid("hardening queue identity is not contiguous"));
        }
        validate_queue_entry(entry)?;
    }
    if report.entries.windows(2).any(|pair| queue_order(&pair[0], &pair[1]).is_gt()) {
        return Err(invalid("hardening queue entries are not in canonical order"));
    }
    Ok(())
}

fn validate_finding(finding: &AppraisalFinding) -> Result<(), BenchError> {
    if [
        finding.appraiser_id.as_str(),
        finding.row_id.as_str(),
        finding.failure_class.as_str(),
        finding.summary.as_str(),
        finding.recommendation.as_str(),
    ]
    .into_iter()
    .any(|value| value.trim().is_empty())
        || !matches!(finding.severity.as_str(), "critical" | "warning" | "info")
        || !matches!(finding.confidence.as_str(), "high" | "medium" | "low")
        || !matches!(finding.result_scope.as_str(), "encrypted-results" | "encrypted-code")
    {
        return Err(invalid("appraisal finding is invalid"));
    }
    Ok(())
}

fn validate_queue_entry(entry: &HardeningQueueEntry) -> Result<(), BenchError> {
    if !matches!(entry.severity.as_str(), "critical" | "warning" | "info")
        || entry.failure_class.trim().is_empty()
        || entry.recommendation.trim().is_empty()
        || !is_sorted_unique_nonempty(&entry.affected_rows)
        || !is_sorted_unique_nonempty(&entry.source_appraisers)
    {
        return Err(invalid("hardening queue entry is invalid"));
    }
    Ok(())
}

fn is_sorted_unique_nonempty(values: &[String]) -> bool {
    !values.is_empty()
        && values.iter().all(|value| !value.trim().is_empty())
        && values.windows(2).all(|pair| pair[0] < pair[1])
}

fn finding_order(left: &AppraisalFinding, right: &AppraisalFinding) -> Ordering {
    severity_weight(&right.severity)
        .cmp(&severity_weight(&left.severity))
        .then_with(|| left.appraiser_id.cmp(&right.appraiser_id))
        .then_with(|| left.row_id.cmp(&right.row_id))
        .then_with(|| left.failure_class.cmp(&right.failure_class))
        .then_with(|| left.result_scope.cmp(&right.result_scope))
}

fn queue_order(left: &HardeningQueueEntry, right: &HardeningQueueEntry) -> Ordering {
    severity_weight(&right.severity)
        .cmp(&severity_weight(&left.severity))
        .then_with(|| left.failure_class.cmp(&right.failure_class))
        .then_with(|| left.recommendation.cmp(&right.recommendation))
}

fn severity_weight(value: &str) -> u8 {
    match value {
        "critical" => 3,
        "warning" => 2,
        _ => 1,
    }
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
