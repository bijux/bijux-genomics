use super::checks::check_known_sites_identity_ready;
use super::checks::check_local_smoke_ready;
use super::checks::check_recalibration_report_ready;
use super::checks::check_skip_behavior_ready;
use super::checks::check_stage_metrics_ready;
use super::checks::check_summary_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamRecalibrationCompleteRow;
use super::observations::LocalRecalibrationSmokeReport;
use super::projection::project;
use super::projection::RowProjection;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyRow;
use std::fs;
use std::path::Path;
pub(super) fn build_bam_recalibration_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    readiness_row: &BamRecalibrationGenotypingReadyRow,
    smoke_report: &LocalRecalibrationSmokeReport,
) -> Result<BamRecalibrationCompleteRow> {
    let summary_path = repo_root.join(&smoke_report.recalibration_summary);
    let stage_metrics_path = repo_root.join(&smoke_report.stage_metrics);
    let recalibration_report_path = repo_root.join(&smoke_report.recalibration_report);
    let recalibrated_bam_path = repo_root.join(&smoke_report.recalibrated_bam);

    let summary: bijux_dna_domain_bam::BamRecalibrationSummaryV1 =
        crate::input::read_json(&summary_path)?;
    let stage_metrics: serde_json::Value = crate::input::read_json(&stage_metrics_path)?;
    let recalibration_report = fs::read_to_string(&recalibration_report_path)
        .with_context(|| format!("read {}", recalibration_report_path.display()))?;

    let local_smoke_ready = check_local_smoke_ready(
        smoke_report,
        &summary_path,
        &stage_metrics_path,
        &recalibration_report_path,
        &recalibrated_bam_path,
    );

    let summary_ready = check_summary_ready(repo_root, smoke_report, &summary);

    let stage_metrics_ready = check_stage_metrics_ready(&stage_metrics);

    let skip_behavior_ready = check_skip_behavior_ready(smoke_report);

    let known_sites_identity_ready =
        check_known_sites_identity_ready(repo_root, smoke_report, &summary);

    let recalibration_report_ready = check_recalibration_report_ready(&recalibration_report);

    let checks = SurfaceChecks {
        local_smoke_ready,
        summary_ready,
        stage_metrics_ready,
        skip_behavior_ready,
        known_sites_identity_ready,
        recalibration_report_ready,
    };
    let (missing_surfaces, coverage_status, reason) = explain(readiness_row, checks);

    project(RowProjection {
        repo_root,
        smoke_path,
        readiness_row,
        smoke_report,
        stage_metrics_path,
        summary,
        stage_metrics,
        local_smoke_ready,
        summary_ready,
        stage_metrics_ready,
        skip_behavior_ready,
        known_sites_identity_ready,
        recalibration_report_ready,
        missing_surfaces,
        coverage_status,
        reason,
    })
}
