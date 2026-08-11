//! Owner: bijux-dna-bench-model
//! Admission rules for persisted domain benchmark campaign records.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{
    AppraisalFinding, BamBenchmarkCampaignReport, DomainBenchmarkCampaignSummary,
    DomainBenchmarkGoalEntry, FastqBenchmarkCampaignReport, HardeningBenchmarkCampaignReport,
    HardeningQueueEntry, VcfBenchmarkCampaignReport, BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1,
    FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1, HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1,
    VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};

/// Validate a persisted FASTQ campaign and its exact nested evidence closure.
///
/// # Errors
/// Returns an error for identity, ordering, nested record, status, or summary drift.
pub fn validate_fastq_benchmark_campaign_report(
    report: &FastqBenchmarkCampaignReport,
) -> Result<(), BenchError> {
    validate_domain_campaign(DomainCampaignView {
        label: "FASTQ",
        expected_schema: FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1,
        expected_domain: "fastq",
        schema_version: &report.schema_version,
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        selected_goals: &report.selected_goals,
        summary: &report.summary,
        entries: &report.entries,
    })
}

/// Validate a persisted BAM campaign and its exact nested evidence closure.
///
/// # Errors
/// Returns an error for identity, ordering, nested record, status, or summary drift.
pub fn validate_bam_benchmark_campaign_report(
    report: &BamBenchmarkCampaignReport,
) -> Result<(), BenchError> {
    validate_domain_campaign(DomainCampaignView {
        label: "BAM",
        expected_schema: BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1,
        expected_domain: "bam",
        schema_version: &report.schema_version,
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        selected_goals: &report.selected_goals,
        summary: &report.summary,
        entries: &report.entries,
    })
}

/// Validate a persisted VCF campaign and its exact nested evidence closure.
///
/// # Errors
/// Returns an error for identity, ordering, nested record, status, or summary drift.
pub fn validate_vcf_benchmark_campaign_report(
    report: &VcfBenchmarkCampaignReport,
) -> Result<(), BenchError> {
    validate_domain_campaign(DomainCampaignView {
        label: "VCF",
        expected_schema: VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1,
        expected_domain: "vcf",
        schema_version: &report.schema_version,
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        selected_goals: &report.selected_goals,
        summary: &report.summary,
        entries: &report.entries,
    })
}

/// Validate a persisted hardening campaign and its exact nested evidence closure.
///
/// # Errors
/// Returns an error for identity, ordering, nested record, status, or summary drift.
pub fn validate_hardening_benchmark_campaign_report(
    report: &HardeningBenchmarkCampaignReport,
) -> Result<(), BenchError> {
    validate_domain_campaign(DomainCampaignView {
        label: "hardening",
        expected_schema: HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1,
        expected_domain: "all",
        schema_version: &report.schema_version,
        campaign_id: &report.campaign_id,
        domain: &report.domain,
        selected_goals: &report.selected_goals,
        summary: &report.summary,
        entries: &report.entries,
    })
}

#[derive(Clone, Copy)]
struct DomainCampaignView<'a> {
    label: &'a str,
    expected_schema: &'a str,
    expected_domain: &'a str,
    schema_version: &'a str,
    campaign_id: &'a str,
    domain: &'a str,
    selected_goals: &'a [String],
    summary: &'a DomainBenchmarkCampaignSummary,
    entries: &'a [DomainBenchmarkGoalEntry],
}

fn validate_domain_campaign(view: DomainCampaignView<'_>) -> Result<(), BenchError> {
    let DomainCampaignView {
        label,
        expected_schema,
        expected_domain,
        schema_version,
        campaign_id,
        domain,
        selected_goals,
        summary,
        entries,
    } = view;
    if schema_version != expected_schema
        || campaign_id.trim().is_empty()
        || domain != expected_domain
        || selected_goals.is_empty()
    {
        return Err(invalid(label, "campaign identity is invalid"));
    }
    validate_nonempty_unique(label, selected_goals, "selected goals")?;
    if selected_goals.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid(label, "selected goals are not in canonical order"));
    }
    let entry_goals = entries.iter().map(|entry| entry.goal_id.clone()).collect::<Vec<_>>();
    if entry_goals != selected_goals {
        return Err(invalid(label, "entries do not close over selected goals"));
    }

    let mut status_counts = BTreeMap::new();
    let mut total_rows = 0_usize;
    let mut total_findings = 0_usize;
    let mut total_queue_entries = 0_usize;
    for entry in entries {
        validate_goal_entry(label, entry)?;
        *status_counts.entry(entry.status.clone()).or_insert(0_usize) += 1;
        total_rows += entry.matched_rows.len();
        total_findings += entry.appraisal_findings.len();
        total_queue_entries += entry.hardening_entries.len();
    }
    if summary.total_goals != entries.len()
        || summary.total_rows != total_rows
        || summary.total_findings != total_findings
        || summary.total_queue_entries != total_queue_entries
        || summary.status_counts != status_counts
    {
        return Err(invalid(label, "campaign summary drifted"));
    }
    Ok(())
}

fn validate_goal_entry(label: &str, entry: &DomainBenchmarkGoalEntry) -> Result<(), BenchError> {
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
        return Err(invalid(label, "goal entry is invalid"));
    }
    validate_nonempty_unique(label, &entry.stage_ids, "goal stage identities")?;
    validate_nonempty_unique(label, &entry.goal_checks, "goal checks")?;

    let mut scenarios = BTreeSet::new();
    for scenario in &entry.scenarios {
        if scenario.scenario_id.trim().is_empty()
            || scenario.focus.trim().is_empty()
            || !scenarios.insert(scenario.scenario_id.as_str())
        {
            return Err(invalid(label, "scenario is invalid"));
        }
    }

    let mut rows = BTreeSet::new();
    for row in &entry.matched_rows {
        let repetitions_match_readiness = match row.readiness_class.as_str() {
            "ready" | "degraded" => row.repetitions > 0,
            "refuse" => row.repetitions == 0,
            _ => false,
        };
        if row.row_id.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || !repetitions_match_readiness
            || !rows.insert(row.row_id.as_str())
        {
            return Err(invalid(label, "matrix row reference is invalid"));
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
        return Err(invalid(label, "goal status does not match its evidence"));
    }

    let mut findings = BTreeSet::new();
    for finding in &entry.appraisal_findings {
        validate_finding(label, finding, &rows)?;
        if !findings.insert((
            finding.appraiser_id.as_str(),
            finding.row_id.as_str(),
            finding.failure_class.as_str(),
            finding.result_scope.as_str(),
        )) {
            return Err(invalid(label, "appraisal finding is duplicated"));
        }
    }
    let mut queue_ids = BTreeSet::new();
    for queue_entry in &entry.hardening_entries {
        validate_queue_entry(label, queue_entry, &rows)?;
        if !queue_ids.insert(queue_entry.queue_id.as_str()) {
            return Err(invalid(label, "hardening entry is duplicated"));
        }
    }
    Ok(())
}

fn validate_finding(
    label: &str,
    finding: &AppraisalFinding,
    rows: &BTreeSet<&str>,
) -> Result<(), BenchError> {
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
        return Err(invalid(label, "appraisal finding is invalid"));
    }
    Ok(())
}

fn validate_queue_entry(
    label: &str,
    entry: &HardeningQueueEntry,
    rows: &BTreeSet<&str>,
) -> Result<(), BenchError> {
    if entry.queue_id.trim().is_empty()
        || !matches!(entry.severity.as_str(), "critical" | "warning" | "info")
        || entry.failure_class.trim().is_empty()
        || entry.recommendation.trim().is_empty()
        || entry.affected_rows.is_empty()
        || entry.source_appraisers.is_empty()
        || entry.affected_rows.iter().any(|row| !rows.contains(row.as_str()))
    {
        return Err(invalid(label, "hardening entry is invalid"));
    }
    validate_nonempty_unique(label, &entry.affected_rows, "hardening affected rows")?;
    validate_nonempty_unique(label, &entry.source_appraisers, "hardening source appraisers")?;
    Ok(())
}

fn validate_nonempty_unique(label: &str, values: &[String], field: &str) -> Result<(), BenchError> {
    let mut seen = BTreeSet::new();
    if values.iter().any(|value| value.trim().is_empty() || !seen.insert(value.as_str())) {
        return Err(invalid(label, format!("{field} are invalid")));
    }
    Ok(())
}

fn invalid(label: &str, message: impl AsRef<str>) -> BenchError {
    BenchError::InvalidPolicy(format!("{label} benchmark {message}", message = message.as_ref()))
}
