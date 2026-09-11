use super::bam_evidence::BamEvidenceAdapter;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_REPORT_MAP_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-report-map.tsv";
pub(crate) type BamReportMapRow = bijux_dna_bench_bam_evidence::report_map::BamReportMapRow;
pub(crate) type BamReportMapReport = bijux_dna_bench_bam_evidence::report_map::BamReportMapReport;
pub(crate) fn run_render_bam_report_map(
    args: &parse::BenchReadinessRenderBamReportMapArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_report_map(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_report_map(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamReportMapReport> {
    bijux_dna_bench_bam_evidence::report_map::render_bam_report_map(
        &BamEvidenceAdapter.context(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_bam_report_map_rows(repo_root: &Path) -> Result<Vec<BamReportMapRow>> {
    bijux_dna_bench_bam_evidence::report_map::collect_bam_report_map_rows(
        &BamEvidenceAdapter.context(repo_root),
    )
}
#[cfg(test)]
use bijux_dna_bench_bam_evidence::report_map::BAM_REPORT_MAP_SCHEMA_VERSION;
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_report_map, BAM_REPORT_MAP_SCHEMA_VERSION, DEFAULT_BAM_REPORT_MAP_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn bam_report_map_tracks_governed_result_bindings() {
        let repo_root = repo_root();
        let report = render_bam_report_map(&repo_root, PathBuf::from(DEFAULT_BAM_REPORT_MAP_PATH))
            .expect("render BAM report map");

        assert_eq!(report.schema_version, BAM_REPORT_MAP_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_REPORT_MAP_PATH);
        assert_eq!(report.domain, "bam");
        assert!(report.expected_result_row_count > 0);
        assert_eq!(report.row_count, report.expected_result_row_count);
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.tool_count, 26);
        assert_eq!(report.section_count, 7);
        assert_eq!(report.summary_table_count, 7);

        let align = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.align" && row.tool_id == "bwa")
            .expect("align row");
        assert_eq!(align.fixture_id, "corpus-01-mini");
        assert_eq!(align.support_status, "supported");
        assert_eq!(align.report_section_id, "alignment_intake");
        assert_eq!(align.summary_table_id, "alignment_baseline");
        assert_eq!(align.anchor_tool_id, "bwa");

        let contamination = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.contamination" && row.tool_id == "schmutzi")
            .expect("contamination row");
        assert_eq!(contamination.anchor_tool_id, "schmutzi");
        assert_eq!(contamination.support_status, "supported");
        assert_eq!(contamination.workflow_branch_id.as_deref(), Some("ancient_dna_authenticity"));
        assert_eq!(contamination.report_section_id, "ancient_signal");
        assert_eq!(contamination.summary_table_id, "damage_authenticity");

        let sex = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.sex" && row.tool_id == "rxy")
            .expect("sex row");
        assert_eq!(sex.anchor_tool_id, "rxy");
        assert_eq!(
            sex.scientific_context_required,
            vec!["chromosome_system".to_string(), "minimum_y_sites".to_string()]
        );
        assert_eq!(sex.report_section_id, "sample_identity");

        let recalibration = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.recalibration" && row.tool_id == "gatk")
            .expect("recalibration row");
        assert_eq!(recalibration.criticality, "essential");
        assert_eq!(recalibration.support_status, "supported");
        assert_eq!(recalibration.report_section_id, "downstream_readiness");
        assert_eq!(recalibration.summary_table_id, "variant_readiness");
    }
}
