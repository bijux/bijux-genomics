//! Model-owned normalization of admitted benchmark observations into collector rows.
use crate::{
    AllDomainExpectedBenchmarkResultsReport, AllDomainFailureClassificationReport,
    AllDomainFakeFailuresManifest, AllDomainFakeRunsReport, AllDomainMissingResultStatus,
    AllDomainMissingResultTestReport, FullBenchmarkResultCollectorRow, FullBenchmarkResultStatus,
    FullBenchmarkResultSurfaceKind, RealSmokeCoreSubsetExecutionKind, RealSmokeCoreSubsetReport,
};
use anyhow::{ensure, Result};

/// Borrow source evidence with its original kind; simulation cannot become observed execution.
#[derive(Debug, Clone, Copy)]
pub enum FullBenchmarkSourceObservation<'a> {
    Expected(&'a AllDomainExpectedBenchmarkResultsReport),
    Simulated(&'a AllDomainFakeRunsReport),
    SimulatedFailures(&'a AllDomainFakeFailuresManifest),
    MissingResultAudit(&'a AllDomainMissingResultTestReport),
    SmokeSubset(&'a RealSmokeCoreSubsetReport),
    FailureClassification(&'a AllDomainFailureClassificationReport),
}

/// Project supported observation families after complete existing owner admission.
/// Source-specific identifiers, missingness, evidence paths and category ordering are retained.
/// Smoke admission checks its declared subset; it does not independently establish fresh tool
/// execution or scientific metric compatibility. Final collector closure remains a separate check.
///
/// # Errors
/// Rejects malformed source reports and unsuccessful declared behavior checks before producing rows.
pub fn project_full_benchmark_observation_rows(
    observation: FullBenchmarkSourceObservation<'_>,
) -> Result<Vec<FullBenchmarkResultCollectorRow>> {
    Ok(match observation {
        FullBenchmarkSourceObservation::Expected(report) => {
            crate::validate_all_domain_expected_benchmark_results(report)?;
            collect_expected_rows(report)
        }
        FullBenchmarkSourceObservation::Simulated(report) => {
            crate::validate_all_domain_simulation_report(report)?;
            collect_fake_run_rows(report)
        }
        FullBenchmarkSourceObservation::SimulatedFailures(report) => {
            crate::validate_all_domain_failure_simulation_manifest(report)?;
            collect_fake_failure_rows(report)
        }
        FullBenchmarkSourceObservation::MissingResultAudit(report) => {
            crate::validate_all_domain_missing_result_audit(report)?;
            collect_missing_result_rows(report)
        }
        FullBenchmarkSourceObservation::SmokeSubset(report) => {
            ensure!(report.passes_behavior_test, "smoke subset has not passed its behavior checks");
            // The existing admission consumes a four-row report; preserve the caller's evidence.
            crate::admit_real_smoke_core_subset_report(report.clone())?;
            collect_real_smoke_rows(report)
        }
        FullBenchmarkSourceObservation::FailureClassification(report) => {
            ensure!(
                report.passes_behavior_test,
                "failure classification has not passed its behavior checks"
            );
            // Admission normalizes the eight-class verdict, so validate an owned observation copy.
            crate::validate_all_domain_failure_classification(&mut report.clone())?;
            let mut rows = collect_insufficient_data_rows(report);
            rows.extend(collect_unsupported_pair_rows(report));
            rows
        }
    })
}

fn collect_expected_rows(
    report: &AllDomainExpectedBenchmarkResultsReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("expected:{}", row.result_id),
            source_surface: "all_domain_expected_benchmark_results".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::BenchmarkExpected,
            result_status: FullBenchmarkResultStatus::Expected,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: Some(row.report_section.clone()),
            evidence_path: report.output_path.clone(),
            manifest_path: None,
            declared_output_count: row.expected_outputs.len(),
            normalized_metric_count: row.expected_metrics.len(),
            detail: "canonical benchmark-ready result binding".to_string(),
        })
        .collect()
}

fn collect_fake_run_rows(report: &AllDomainFakeRunsReport) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .results
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("fake-run:{}", row.result_id),
            source_surface: "all_domain_fake_runs".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::FakeRun,
            result_status: FullBenchmarkResultStatus::Succeeded,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: None,
            evidence_path: row.metrics_path.clone(),
            manifest_path: Some(row.stage_result_path.clone()),
            declared_output_count: row.declared_output_count,
            normalized_metric_count: row.expected_metric_count,
            detail: format!("fake-run materialized via `{}`", row.command_source),
        })
        .collect()
}

fn collect_real_smoke_rows(
    report: &RealSmokeCoreSubsetReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("real-smoke:{}", row.execution_id),
            source_surface: "real_smoke_core_subset".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::RealSmoke,
            result_status: FullBenchmarkResultStatus::Succeeded,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: None,
            execution_id: Some(row.execution_id.clone()),
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: row.stage_result_manifest_path.clone(),
            declared_output_count: 0,
            normalized_metric_count: row.normalized_metric_count,
            detail: match row.execution_kind {
                RealSmokeCoreSubsetExecutionKind::Stage => {
                    "governed real-smoke stage execution".to_string()
                }
                RealSmokeCoreSubsetExecutionKind::PipelineBridge => {
                    "governed real-smoke pipeline bridge execution".to_string()
                }
            },
        })
        .collect()
}

fn collect_insufficient_data_rows(
    report: &AllDomainFailureClassificationReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .filter(|row| row.class_id == "insufficient_data")
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("insufficient-data:{}:{}:{}", row.domain, row.stage_id, row.tool_id),
            source_surface: "all_domain_failure_classification".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::FailureClassification,
            result_status: FullBenchmarkResultStatus::InsufficientData,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: row.result_id.clone(),
            execution_id: None,
            corpus_id: None,
            asset_profile_id: None,
            report_section: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: None,
            declared_output_count: 0,
            normalized_metric_count: 1,
            detail: row.detail.clone(),
        })
        .collect()
}

fn collect_unsupported_pair_rows(
    report: &AllDomainFailureClassificationReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .filter(|row| row.class_id == "unsupported_pair")
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("unsupported-pair:{}:{}:{}", row.domain, row.stage_id, row.tool_id),
            source_surface: "all_domain_failure_classification".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::UnsupportedPair,
            result_status: FullBenchmarkResultStatus::UnsupportedPair,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: row.result_id.clone(),
            execution_id: None,
            corpus_id: None,
            asset_profile_id: None,
            report_section: None,
            evidence_path: row.evidence_path.clone(),
            manifest_path: None,
            declared_output_count: 0,
            normalized_metric_count: 0,
            detail: row.detail.clone(),
        })
        .collect()
}

fn collect_fake_failure_rows(
    report: &AllDomainFakeFailuresManifest,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .failures
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("fake-failure:{}", row.result_id),
            source_surface: "all_domain_fake_failures".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::FakeFailure,
            result_status: FullBenchmarkResultStatus::Failed,
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: None,
            evidence_path: row.failure_record_path.clone(),
            manifest_path: None,
            declared_output_count: row.failed_output_count,
            normalized_metric_count: 0,
            detail: format!("fake failure recorded exit_code={}", row.exit_code),
        })
        .collect()
}

fn collect_missing_result_rows(
    report: &AllDomainMissingResultTestReport,
) -> Vec<FullBenchmarkResultCollectorRow> {
    report
        .rows
        .iter()
        .map(|row| FullBenchmarkResultCollectorRow {
            record_id: format!("missing-audit:{}", row.result_id),
            source_surface: "all_domain_missing_result_audit".to_string(),
            surface_kind: FullBenchmarkResultSurfaceKind::MissingResultAudit,
            result_status: match row.result_status {
                AllDomainMissingResultStatus::Present => FullBenchmarkResultStatus::Present,
                AllDomainMissingResultStatus::MissingResult => {
                    FullBenchmarkResultStatus::MissingResult
                }
            },
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            pipeline_id: None,
            node_id: None,
            result_id: Some(row.result_id.clone()),
            execution_id: None,
            corpus_id: Some(row.corpus_id.clone()),
            asset_profile_id: Some(row.asset_profile_id.clone()),
            report_section: Some(row.report_section.clone()),
            evidence_path: row.audit_manifest_path.clone(),
            manifest_path: Some(row.audit_manifest_path.clone()),
            declared_output_count: row.expected_output_artifact_ids.len(),
            normalized_metric_count: row.expected_metrics.len(),
            detail: row.reason.clone(),
        })
        .collect()
}
