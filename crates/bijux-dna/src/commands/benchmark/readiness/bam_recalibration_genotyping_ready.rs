use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping as owner;
pub(crate) use owner::{BamRecalibrationGenotypingReadyReport, BamRecalibrationGenotypingReadyRow};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/recalibration-genotyping-ready.json";
#[cfg(test)]
const BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_recalibration_genotyping_ready.v1";
pub(crate) fn run_render_bam_recalibration_genotyping_ready(
    args: &parse::BenchReadinessRenderBamRecalibrationGenotypingReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_recalibration_genotyping_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_recalibration_genotyping_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamRecalibrationGenotypingReadyReport> {
    owner::render_bam_recalibration_genotyping_ready(
        &super::bam_readiness::context(repo_root),
        &output_path,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_recalibration_genotyping_ready,
        BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION,
        DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_recalibration_genotyping_ready_reports_governed_rows() {
        let report = render_bam_recalibration_genotyping_ready(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH),
        )
        .expect("render BAM recalibration/genotyping readiness");

        #[cfg(feature = "bam_downstream")]
        let expected_row_count = 2;
        #[cfg(not(feature = "bam_downstream"))]
        let expected_row_count = 1;

        assert_eq!(report.schema_version, BAM_RECALIBRATION_GENOTYPING_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH);
        assert_eq!(report.active_row_count, expected_row_count);
        assert_eq!(report.complete_row_count, expected_row_count);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, expected_row_count);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let recalibration_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.recalibration" && row.tool_id == "gatk")
            .expect("bam.recalibration gatk row");
        assert_eq!(recalibration_row.expected_normalized_metrics_output_id, "recal_report");
        assert_eq!(recalibration_row.expected_local_proof_kind, "local_smoke");
        assert_eq!(recalibration_row.local_proof_tool_id, "gatk");
        assert!(recalibration_row
            .required_local_proof_fields
            .iter()
            .any(|field| field == "known_sites"));
        assert!(recalibration_row
            .local_proof_declared_output_ids
            .iter()
            .any(|output| output == "recalibration_summary"));

        #[cfg(feature = "bam_downstream")]
        {
            let genotyping_row = report
                .rows
                .iter()
                .find(|row| row.stage_id == "bam.genotyping" && row.tool_id == "bcftools")
                .expect("bam.genotyping bcftools row");
            assert_eq!(genotyping_row.expected_normalized_metrics_output_id, "genotyping_report");
            assert_eq!(genotyping_row.expected_local_proof_kind, "local_ready_plan");
            assert_eq!(genotyping_row.local_proof_tool_id, "bcftools");
            assert!(genotyping_row
                .required_local_proof_fields
                .iter()
                .any(|field| field == "min_call_rate"));
            assert!(genotyping_row
                .local_proof_declared_output_ids
                .iter()
                .any(|output| output == "genotyping_bcf"));
        }
    }
}
