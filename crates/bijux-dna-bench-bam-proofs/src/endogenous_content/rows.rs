use super::checks::check_endogenous_metric_consistency_ready;
use super::checks::check_normalized_metrics_ready;
use super::checks::check_summary_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamEndogenousContentCompleteRow;
use super::models::LocalEndogenousContentSmokeMetrics;
use super::observations::LocalEndogenousContentSmokeReport;
use super::projection::project;
use super::projection::RowProjection;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyRow;
use std::path::Path;
pub(super) fn build_endogenous_content_complete_row(
    repo_root: &Path,
    endogenous_row: BamOverlapEndogenousReadyRow,
) -> Result<BamEndogenousContentCompleteRow> {
    let local_smoke_report_path = repo_root.join(&endogenous_row.local_smoke_proof_path);
    let local_smoke_report: LocalEndogenousContentSmokeReport =
        crate::input::read_json(&local_smoke_report_path)?;

    let summary_path = repo_root.join(&local_smoke_report.endogenous_summary);
    let summary: bijux_dna_domain_bam::BamEndogenousContentEstimateV1 =
        crate::input::read_json(&summary_path)?;

    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalEndogenousContentSmokeMetrics =
        crate::input::read_json(&stage_metrics_path)?;

    let summary_ready = check_summary_ready(&summary);

    let normalized_metrics_ready = check_normalized_metrics_ready(&normalized_metrics);

    let endogenous_metric_consistency_ready = check_endogenous_metric_consistency_ready(
        &local_smoke_report,
        &summary,
        &normalized_metrics,
    );

    let checks = SurfaceChecks {
        summary_ready,
        normalized_metrics_ready,
        endogenous_metric_consistency_ready,
    };
    let (missing_surfaces, coverage_status, reason) = explain(&endogenous_row, checks);

    Ok(project(RowProjection {
        endogenous_row,
        local_smoke_report,
        summary,
        normalized_metrics,
        summary_ready,
        normalized_metrics_ready,
        endogenous_metric_consistency_ready,
        missing_surfaces,
        coverage_status,
        reason,
    }))
}
