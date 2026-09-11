use crate::commands::cli::{parse, render};
use anyhow::Context;
use anyhow::Result;
pub(crate) use bijux_dna_bench_bam_proofs::authenticity::{
    BamAuthenticityCompleteReport, DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_authenticity_complete(
    args: &parse::BenchReadinessRenderBamAuthenticityCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_authenticity_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_authenticity_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamAuthenticityCompleteReport> {
    bijux_dna_bench_bam_proofs::authenticity::render_bam_authenticity_complete(
        &super::bam_stage_proofs::context(repo_root),
        &output_path,
    )
}
#[cfg(test)]
const BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_authenticity_complete.v1";
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_authenticity_complete, BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_authenticity_complete_reports_governed_metrics() {
        let root = repo_root();
        let report = render_bam_authenticity_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH),
        )
        .expect("render BAM authenticity completion report");

        assert_eq!(report.schema_version, BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 3);
        assert_eq!(report.complete_row_count, 3);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 12);
        assert_eq!(report.missing_tool_ids, Vec::<String>::new());
        assert_eq!(report.unexpected_tool_ids, Vec::<String>::new());
        assert_eq!(report.violation_count, 0);
        assert!(report.toolset_ready);
        assert!(report.ok);

        let row =
            report.rows.iter().find(|row| row.tool_id == "authenticct").expect("authenticct row");
        assert_eq!(row.stage_id, "bam.authenticity");
        assert_eq!(row.summary_schema_version, "bijux.bam.authenticity_advisory.v1");
        assert_eq!(row.composition_schema_version, "bijux.bam.authenticity.composition.v1");
        assert_eq!(
            row.normalized_metrics_schema_version,
            "bijux.bam.authenticity.local_smoke.metrics.v1"
        );
        assert!((row.summary.score - 0.533_333_333_333_333_3).abs() <= 1e-9);
        assert!((row.summary.confidence - 0.813_333_333_333_333_4).abs() <= 1e-9);
        assert_eq!(row.summary.status, "pass");
        assert!(row.local_smoke_report.expectation_matched);
        assert!(row.authenticity_report_contract_ready);
        assert!(row.summary_ready);
        assert!(row.composition_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.evidence_consumption_ready);
        assert_eq!(row.coverage_status, "complete");
    }
}
