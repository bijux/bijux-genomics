use super::checks::check_corrected_overlap_metrics_ready;
use super::checks::check_normalized_metrics_ready;
use super::checks::check_summary_ready;
use super::explanation::explain;
use super::explanation::SurfaceChecks;
use super::models::BamOverlapCorrectionCompleteRow;
use super::models::LocalOverlapCorrectionSmokeMetrics;
use super::observations::LocalOverlapCorrectionSmokeReport;
use super::projection::project;
use super::projection::RowProjection;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyRow;
use std::path::Path;
pub(super) fn build_overlap_correction_complete_row(
    repo_root: &Path,
    overlap_row: BamOverlapEndogenousReadyRow,
) -> Result<BamOverlapCorrectionCompleteRow> {
    let local_smoke_report_path = repo_root.join(&overlap_row.local_smoke_proof_path);
    let local_smoke_report: LocalOverlapCorrectionSmokeReport =
        crate::input::read_json(&local_smoke_report_path)?;
    let summary_path = repo_root.join(&local_smoke_report.overlap_correction_summary);
    let summary: bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1 =
        crate::input::read_json(&summary_path)?;
    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalOverlapCorrectionSmokeMetrics =
        crate::input::read_json(&stage_metrics_path)?;

    let overlap_corrected_bai_path = overlap_row
        .local_smoke_artifact_paths
        .iter()
        .find(|path| path.ends_with("overlap_corrected.bam.bai"))
        .cloned()
        .ok_or_else(|| anyhow!("missing governed overlap-corrected BAM index path"))?;

    let summary_ready = check_summary_ready(&summary);
    let normalized_metrics_ready =
        check_normalized_metrics_ready(&local_smoke_report, &normalized_metrics);
    let corrected_overlap_metrics_ready =
        check_corrected_overlap_metrics_ready(&local_smoke_report, &summary, &normalized_metrics);

    let checks =
        SurfaceChecks { summary_ready, normalized_metrics_ready, corrected_overlap_metrics_ready };
    let (missing_surfaces, coverage_status, reason) = explain(&overlap_row, checks);

    Ok(project(RowProjection {
        repo_root,
        overlap_row,
        local_smoke_report,
        summary,
        normalized_metrics,
        overlap_corrected_bai_path,
        summary_ready,
        normalized_metrics_ready,
        corrected_overlap_metrics_ready,
        missing_surfaces,
        coverage_status,
        reason,
    }))
}
