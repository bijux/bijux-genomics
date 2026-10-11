//! Semantic admission for full benchmark result collector reports.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::{
    BenchError, FullBenchmarkResultCollectorReport, FullBenchmarkResultCollectorRow,
    FullBenchmarkResultStatus, FullBenchmarkResultSurfaceKind,
    FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FullBenchmarkResultBinding {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullBenchmarkResultCollectorAdmissionContext {
    pub expected_pipeline_fake_run_row_count: usize,
    pub expected_real_smoke_row_count: usize,
    pub expected_insufficient_data_row_count: usize,
    pub expected_missing_result_row_count: usize,
    pub expected_domains: BTreeSet<String>,
    pub expected_unsupported_bindings: BTreeSet<FullBenchmarkResultBinding>,
}

/// Validate and admit a collector report against its governed source closure.
///
/// # Errors
/// Returns an error when schema identity, source counts, row identity, or evidence paths drift.
pub fn validate_full_benchmark_result_collector(
    report: &mut FullBenchmarkResultCollectorReport,
    context: &FullBenchmarkResultCollectorAdmissionContext,
) -> Result<(), BenchError> {
    if report.schema_version != FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1 {
        return invalid("unsupported full benchmark result collector schema");
    }
    require_path(&report.output_path, "output_path")?;

    let expected_total_row_count = report.benchmark_expected_row_count
        + report.pipeline_fake_run_row_count
        + report.fake_run_row_count
        + report.fake_failure_row_count
        + report.missing_result_audit_row_count
        + report.real_smoke_row_count
        + report.insufficient_data_row_count
        + report.unsupported_pair_row_count;
    if report.row_count != report.rows.len() || report.row_count != expected_total_row_count {
        return invalid("full benchmark result collector source aggregates do not close");
    }

    validate_rows(report)?;
    validate_aggregates(report)?;

    if report.pipeline_fake_run_row_count != context.expected_pipeline_fake_run_row_count
        || report.real_smoke_row_count != context.expected_real_smoke_row_count
        || report.insufficient_data_row_count != context.expected_insufficient_data_row_count
        || report.unsupported_pair_row_count != context.expected_unsupported_bindings.len()
    {
        return invalid("full benchmark result collector governed source counts drifted");
    }
    if report.benchmark_expected_row_count != report.fake_run_row_count
        || report.benchmark_expected_row_count != report.fake_failure_row_count
        || report.benchmark_expected_row_count != report.missing_result_audit_row_count
    {
        return invalid(
            "full benchmark result collector expected, fake-run, failure, and audit surfaces drifted",
        );
    }

    let expected_succeeded_count = report.pipeline_fake_run_row_count
        + report.fake_run_row_count
        + report.real_smoke_row_count;
    let expected_present_count = report
        .benchmark_expected_row_count
        .saturating_sub(context.expected_missing_result_row_count);
    if status_count(report, FullBenchmarkResultStatus::Expected)
        != report.benchmark_expected_row_count
        || status_count(report, FullBenchmarkResultStatus::Succeeded) != expected_succeeded_count
        || status_count(report, FullBenchmarkResultStatus::Failed) != report.fake_failure_row_count
        || status_count(report, FullBenchmarkResultStatus::Present) != expected_present_count
        || report.missing_result_status_count != context.expected_missing_result_row_count
        || report.insufficient_data_status_count != context.expected_insufficient_data_row_count
        || report.unsupported_pair_status_count != context.expected_unsupported_bindings.len()
    {
        return invalid("full benchmark result collector status counts drifted");
    }

    let domains = report.domain_counts.keys().cloned().collect::<BTreeSet<_>>();
    if domains != context.expected_domains || report.domain_counts.values().any(|count| *count == 0)
    {
        return invalid("full benchmark result collector domain coverage drifted");
    }

    let missing_rows = report
        .rows
        .iter()
        .filter(|row| row.result_status == FullBenchmarkResultStatus::MissingResult)
        .collect::<Vec<_>>();
    if missing_rows.len() != context.expected_missing_result_row_count
        || missing_rows
            .iter()
            .any(|row| row.surface_kind != FullBenchmarkResultSurfaceKind::MissingResultAudit)
    {
        return invalid("full benchmark result collector missing-result evidence drifted");
    }
    let insufficient_rows = report
        .rows
        .iter()
        .filter(|row| row.result_status == FullBenchmarkResultStatus::InsufficientData)
        .collect::<Vec<_>>();
    if insufficient_rows.len() != context.expected_insufficient_data_row_count
        || insufficient_rows
            .iter()
            .any(|row| row.surface_kind != FullBenchmarkResultSurfaceKind::FailureClassification)
    {
        return invalid("full benchmark result collector insufficient-data evidence drifted");
    }

    let unsupported_bindings = report
        .rows
        .iter()
        .filter(|row| row.result_status == FullBenchmarkResultStatus::UnsupportedPair)
        .map(binding)
        .collect::<BTreeSet<_>>();
    if unsupported_bindings != context.expected_unsupported_bindings
        || report.rows.iter().any(|row| {
            row.result_status == FullBenchmarkResultStatus::UnsupportedPair
                && row.surface_kind != FullBenchmarkResultSurfaceKind::UnsupportedPair
        })
    {
        return invalid("full benchmark result collector unsupported-pair evidence drifted");
    }
    if missing_rows.iter().any(|row| context.expected_unsupported_bindings.contains(&binding(row)))
    {
        return invalid(
            "full benchmark result collector collapsed missing and unsupported evidence",
        );
    }

    report.passes_behavior_test = true;
    Ok(())
}

fn validate_rows(report: &FullBenchmarkResultCollectorReport) -> Result<(), BenchError> {
    let record_ids = report.rows.iter().map(|row| row.record_id.as_str()).collect::<BTreeSet<_>>();
    if record_ids.len() != report.rows.len() {
        return invalid("full benchmark result collector record identities are duplicated");
    }
    for row in &report.rows {
        for (value, field) in [
            (&row.record_id, "rows[].record_id"),
            (&row.source_surface, "rows[].source_surface"),
            (&row.domain, "rows[].domain"),
            (&row.stage_id, "rows[].stage_id"),
            (&row.tool_id, "rows[].tool_id"),
            (&row.evidence_path, "rows[].evidence_path"),
            (&row.detail, "rows[].detail"),
        ] {
            require_text(value, field)?;
        }
        require_path(&row.evidence_path, "rows[].evidence_path")?;
        if let Some(path) = &row.manifest_path {
            require_path(path, "rows[].manifest_path")?;
        }
        for (value, field) in [
            (row.pipeline_id.as_deref(), "rows[].pipeline_id"),
            (row.node_id.as_deref(), "rows[].node_id"),
            (row.result_id.as_deref(), "rows[].result_id"),
            (row.execution_id.as_deref(), "rows[].execution_id"),
            (row.corpus_id.as_deref(), "rows[].corpus_id"),
            (row.asset_profile_id.as_deref(), "rows[].asset_profile_id"),
            (row.report_section.as_deref(), "rows[].report_section"),
        ] {
            if value.is_some_and(|value| value.trim().is_empty()) {
                return invalid(format!(
                    "full benchmark result collector requires non-empty optional {field}"
                ));
            }
        }
        if row.stage_id.split_once('.').map(|(domain, _)| domain) != Some(row.domain.as_str())
            || row.source_surface != expected_source_surface(row.surface_kind)
            || !surface_accepts_status(row.surface_kind, row.result_status)
        {
            return invalid("full benchmark result collector row semantics are inconsistent");
        }
    }
    Ok(())
}

fn expected_source_surface(surface: FullBenchmarkResultSurfaceKind) -> &'static str {
    match surface {
        FullBenchmarkResultSurfaceKind::BenchmarkExpected => {
            "all_domain_expected_benchmark_results"
        }
        FullBenchmarkResultSurfaceKind::PipelineFakeRun => "essential_pipeline_fake_runs",
        FullBenchmarkResultSurfaceKind::FakeRun => "all_domain_fake_runs",
        FullBenchmarkResultSurfaceKind::FakeFailure => "all_domain_fake_failures",
        FullBenchmarkResultSurfaceKind::MissingResultAudit => "all_domain_missing_result_audit",
        FullBenchmarkResultSurfaceKind::RealSmoke => "real_smoke_core_subset",
        FullBenchmarkResultSurfaceKind::FailureClassification
        | FullBenchmarkResultSurfaceKind::UnsupportedPair => "all_domain_failure_classification",
    }
}

fn validate_aggregates(report: &FullBenchmarkResultCollectorReport) -> Result<(), BenchError> {
    let mut surface_counts = BTreeMap::<String, usize>::new();
    let mut status_counts = BTreeMap::<String, usize>::new();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &report.rows {
        *surface_counts.entry(row.surface_kind.as_str().to_string()).or_default() += 1;
        *status_counts.entry(row.result_status.as_str().to_string()).or_default() += 1;
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }
    if report.surface_kind_counts != surface_counts
        || report.result_status_counts != status_counts
        || report.domain_counts != domain_counts
        || report.benchmark_expected_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::BenchmarkExpected)
        || report.pipeline_fake_run_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::PipelineFakeRun)
        || report.fake_run_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::FakeRun)
        || report.fake_failure_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::FakeFailure)
        || report.missing_result_audit_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::MissingResultAudit)
        || report.real_smoke_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::RealSmoke)
        || report.insufficient_data_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::FailureClassification)
        || report.unsupported_pair_row_count
            != surface_count(report, FullBenchmarkResultSurfaceKind::UnsupportedPair)
        || report.missing_result_status_count
            != status_count(report, FullBenchmarkResultStatus::MissingResult)
        || report.insufficient_data_status_count
            != status_count(report, FullBenchmarkResultStatus::InsufficientData)
        || report.unsupported_pair_status_count
            != status_count(report, FullBenchmarkResultStatus::UnsupportedPair)
    {
        return invalid("full benchmark result collector row aggregates drifted");
    }
    Ok(())
}

fn surface_accepts_status(
    surface: FullBenchmarkResultSurfaceKind,
    status: FullBenchmarkResultStatus,
) -> bool {
    matches!(
        (surface, status),
        (FullBenchmarkResultSurfaceKind::BenchmarkExpected, FullBenchmarkResultStatus::Expected)
            | (
                FullBenchmarkResultSurfaceKind::PipelineFakeRun
                    | FullBenchmarkResultSurfaceKind::FakeRun
                    | FullBenchmarkResultSurfaceKind::RealSmoke,
                FullBenchmarkResultStatus::Succeeded
            )
            | (FullBenchmarkResultSurfaceKind::FakeFailure, FullBenchmarkResultStatus::Failed)
            | (
                FullBenchmarkResultSurfaceKind::MissingResultAudit,
                FullBenchmarkResultStatus::Present | FullBenchmarkResultStatus::MissingResult
            )
            | (
                FullBenchmarkResultSurfaceKind::FailureClassification,
                FullBenchmarkResultStatus::InsufficientData
            )
            | (
                FullBenchmarkResultSurfaceKind::UnsupportedPair,
                FullBenchmarkResultStatus::UnsupportedPair
            )
    )
}

fn surface_count(
    report: &FullBenchmarkResultCollectorReport,
    surface: FullBenchmarkResultSurfaceKind,
) -> usize {
    report.surface_kind_counts.get(surface.as_str()).copied().unwrap_or_default()
}

fn status_count(
    report: &FullBenchmarkResultCollectorReport,
    status: FullBenchmarkResultStatus,
) -> usize {
    report.result_status_counts.get(status.as_str()).copied().unwrap_or_default()
}

fn binding(row: &FullBenchmarkResultCollectorRow) -> FullBenchmarkResultBinding {
    FullBenchmarkResultBinding {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("full benchmark result collector requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "full benchmark result collector requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
