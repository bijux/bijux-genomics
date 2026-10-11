use crate::commands::cli::{parse, render};
use anyhow::Context;
use anyhow::Result;
pub(crate) use bijux_dna_bench_bam_proofs::overlap_correction::{
    BamOverlapCorrectionCompleteReport, DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_overlap_correction_complete(
    args: &parse::BenchReadinessRenderBamOverlapCorrectionCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_overlap_correction_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_overlap_correction_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamOverlapCorrectionCompleteReport> {
    bijux_dna_bench_bam_proofs::overlap_correction::render_bam_overlap_correction_complete(
        &super::bam_stage_proofs::context(repo_root),
        &output_path,
    )
}
#[cfg(test)]
const BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_overlap_correction_complete.v1";
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_overlap_correction_complete, BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_overlap_correction_complete_reports_governed_metrics() {
        let root = repo_root();
        let report = render_bam_overlap_correction_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH),
        )
        .expect("render BAM overlap-correction completion report");

        assert_eq!(report.schema_version, BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 1);
        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 11);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let row = report.rows.first().expect("overlap-correction row");
        assert_eq!(row.stage_id, "bam.overlap_correction");
        assert_eq!(row.tool_id, "bamutil");
        assert_eq!(row.summary_schema_version, "bijux.bam.overlap_correction.v1");
        assert_eq!(
            row.normalized_metrics_schema_version,
            "bijux.bam.overlap_correction.local_smoke.metrics.v1"
        );
        assert_eq!(row.summary_pair_count, 2);
        assert_eq!(row.summary_corrected_pairs, 1);
        assert_eq!(row.summary_corrected_overlap_bases, 7);
        assert_eq!(row.normalized_metrics.expected_pair_count, 2);
        assert_eq!(row.normalized_metrics.pair_count, 2);
        assert_eq!(row.normalized_metrics.expected_corrected_pairs, 1);
        assert_eq!(row.normalized_metrics.corrected_pairs, 1);
        assert_eq!(row.normalized_metrics.expected_corrected_overlap_bases, 7);
        assert_eq!(row.normalized_metrics.corrected_overlap_bases, 7);
        assert!(row.local_smoke_expectation_matched);
        assert!(row.summary_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.corrected_overlap_metrics_ready);
        assert_eq!(row.coverage_status, "complete");
    }
}
