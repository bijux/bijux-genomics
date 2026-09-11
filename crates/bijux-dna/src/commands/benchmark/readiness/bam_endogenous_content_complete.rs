use crate::commands::cli::{parse, render};
use anyhow::Context;
use anyhow::Result;
pub(crate) use bijux_dna_bench_bam_proofs::endogenous_content::{
    BamEndogenousContentCompleteReport, DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_endogenous_content_complete(
    args: &parse::BenchReadinessRenderBamEndogenousContentCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_endogenous_content_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_endogenous_content_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamEndogenousContentCompleteReport> {
    bijux_dna_bench_bam_proofs::endogenous_content::render_bam_endogenous_content_complete(
        &super::bam_stage_proofs::context(repo_root),
        &output_path,
    )
}
#[cfg(test)]
const BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_endogenous_content_complete.v1";
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_endogenous_content_complete, BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_endogenous_content_complete_reports_governed_metrics() {
        let root = repo_root();
        let report = render_bam_endogenous_content_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH),
        )
        .expect("render BAM endogenous-content completion report");

        assert_eq!(report.schema_version, BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 1);
        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 11);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let row = report.rows.first().expect("endogenous-content row");
        assert_eq!(row.stage_id, "bam.endogenous_content");
        assert_eq!(row.tool_id, "samtools");
        assert_eq!(row.summary_schema_version, "bijux.bam.endogenous_content.v1");
        assert_eq!(
            row.normalized_metrics_schema_version,
            "bijux.bam.endogenous_content.local_smoke.metrics.v1"
        );
        assert_eq!(row.summary_mapped_reads, 3);
        assert_eq!(row.summary_endogenous_reads, 3);
        assert_eq!(row.summary_total_reads, 5);
        assert!((row.summary_endogenous_fraction - 0.6).abs() <= 1e-9);
        assert!(row.local_smoke_expectation_matched);
        assert!(row.summary_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.endogenous_metric_consistency_ready);
        assert_eq!(row.coverage_status, "complete");
        assert_eq!(row.contaminant_reads, None);
    }
}
