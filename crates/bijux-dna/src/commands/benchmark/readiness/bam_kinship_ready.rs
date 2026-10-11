use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness::kinship as owner;
pub(crate) use owner::BamKinshipReadyReport;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_KINSHIP_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/kinship-ready.json";
#[cfg(test)]
const BAM_KINSHIP_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_kinship_ready.v1";
pub(crate) fn run_render_bam_kinship_ready(
    args: &parse::BenchReadinessRenderBamKinshipReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_kinship_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_KINSHIP_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_kinship_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamKinshipReadyReport> {
    owner::render_bam_kinship_ready(&super::bam_readiness::context(repo_root), &output_path)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_kinship_ready, BAM_KINSHIP_READY_SCHEMA_VERSION, DEFAULT_BAM_KINSHIP_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_kinship_ready_reports_governed_rows() {
        let report =
            render_bam_kinship_ready(&repo_root(), PathBuf::from(DEFAULT_BAM_KINSHIP_READY_PATH))
                .expect("render BAM kinship readiness");

        #[cfg(feature = "bam_downstream")]
        let expected_row_count = 1;
        #[cfg(not(feature = "bam_downstream"))]
        let expected_row_count = 0;

        assert_eq!(report.schema_version, BAM_KINSHIP_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_KINSHIP_READY_PATH);
        assert_eq!(report.active_row_count, expected_row_count);
        assert_eq!(report.complete_row_count, expected_row_count);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, usize::from(expected_row_count > 0));
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        #[cfg(feature = "bam_downstream")]
        {
            let king_row = report
                .rows
                .iter()
                .find(|row| row.stage_id == "bam.kinship" && row.tool_id == "king")
                .expect("bam.kinship king row");
            assert_eq!(king_row.expected_normalized_metrics_output_id, "kinship_report");
            assert_eq!(king_row.expected_local_proof_kind, "local_smoke");
            assert_eq!(king_row.local_proof_tool_id, "king");
            assert_eq!(king_row.local_proof_sample_id, "sample-set");
            assert!(king_row
                .local_proof_observed_fields
                .iter()
                .any(|field| field == "overlap_snps"));
            assert!(king_row
                .local_proof_declared_output_ids
                .iter()
                .any(|output| output == "stage_metrics"));
        }
    }
}
