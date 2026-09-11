use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous as owner;
pub(crate) use owner::{BamOverlapEndogenousReadyReport, BamOverlapEndogenousReadyRow};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/overlap-endogenous-ready.json";
#[cfg(test)]
const BAM_OVERLAP_ENDOGENOUS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_overlap_endogenous_ready.v1";
pub(crate) fn run_render_bam_overlap_endogenous_ready(
    args: &parse::BenchReadinessRenderBamOverlapEndogenousReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_overlap_endogenous_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_overlap_endogenous_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamOverlapEndogenousReadyReport> {
    owner::render_bam_overlap_endogenous_ready(
        &super::bam_readiness::context(repo_root),
        &output_path,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_overlap_endogenous_ready, BAM_OVERLAP_ENDOGENOUS_READY_SCHEMA_VERSION,
        DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_overlap_endogenous_ready_reports_governed_rows() {
        let root = repo_root();
        let report = render_bam_overlap_endogenous_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH),
        )
        .expect("render BAM overlap/endogenous readiness");

        assert_eq!(report.schema_version, BAM_OVERLAP_ENDOGENOUS_READY_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 2);
        assert_eq!(report.complete_row_count, 2);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.rows.len(), 2);
        assert!(report.rows.iter().all(|row| {
            row.active_scope_ready
                && row.command_ready
                && row.output_ready
                && row.parser_ready
                && row.expected_result_ready
                && row.report_ready
                && row.schema_ready
                && row.local_smoke_ready
                && row.coverage_status == "complete"
        }));

        let endogenous_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.endogenous_content")
            .expect("bam.endogenous_content row");
        assert_eq!(endogenous_row.expected_normalized_metrics_output_id, "endogenous_report");
        assert_eq!(endogenous_row.report_section_id, "coverage_quality");
        assert_eq!(endogenous_row.summary_table_id, "coverage_bias_qc");
        assert!(endogenous_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "endogenous_fraction"));
        assert!(endogenous_row.local_smoke_artifact_paths.iter().any(|path| path
            == "artifacts/runs/bench/local-smoke/bam.endogenous_content/endogenous_content.json"));

        let overlap_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.overlap_correction")
            .expect("bam.overlap_correction row");
        assert_eq!(overlap_row.expected_normalized_metrics_output_id, "summary");
        assert_eq!(overlap_row.report_section_id, "alignment_refinement");
        assert_eq!(overlap_row.summary_table_id, "filter_retention");
        assert!(overlap_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "corrected_overlap_bases"));
        assert!(overlap_row.local_smoke_artifact_paths.iter().any(|path| path
            == "artifacts/runs/bench/local-smoke/bam.overlap_correction/overlap_correction.json"));
        assert!(overlap_row.local_smoke_artifact_paths.iter().any(|path| {
            path
            == "artifacts/runs/bench/local-smoke/bam.overlap_correction/overlap_corrected.bam.bai"
        }));
    }
}
