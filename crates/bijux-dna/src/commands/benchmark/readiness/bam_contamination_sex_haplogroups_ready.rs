use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups as owner;
pub(crate) use owner::{
    BamContaminationSexHaplogroupsReadyReport, BamContaminationSexHaplogroupsReadyRow,
};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/contamination-sex-haplogroups-ready.json";
#[cfg(test)]
const BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_contamination_sex_haplogroups_ready.v1";
pub(crate) fn run_render_bam_contamination_sex_haplogroups_ready(
    args: &parse::BenchReadinessRenderBamContaminationSexHaplogroupsReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_contamination_sex_haplogroups_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_contamination_sex_haplogroups_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamContaminationSexHaplogroupsReadyReport> {
    owner::render_bam_contamination_sex_haplogroups_ready(
        &super::bam_readiness::context(repo_root),
        &output_path,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_contamination_sex_haplogroups_ready,
        BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_SCHEMA_VERSION,
        DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_contamination_sex_haplogroups_ready_reports_governed_rows() {
        let root = repo_root();
        let report = render_bam_contamination_sex_haplogroups_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH),
        )
        .expect("render BAM contamination/sex/haplogroups readiness");

        #[cfg(feature = "bam_downstream")]
        let expected_row_count = 6;
        #[cfg(not(feature = "bam_downstream"))]
        let expected_row_count = 5;

        #[cfg(feature = "bam_downstream")]
        let expected_stage_count = 3;
        #[cfg(not(feature = "bam_downstream"))]
        let expected_stage_count = 2;

        assert_eq!(report.schema_version, BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, expected_row_count);
        assert_eq!(report.complete_row_count, expected_row_count);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, expected_stage_count);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.rows.len(), expected_row_count);
        assert!(report.rows.iter().all(|row| {
            row.active_scope_ready
                && row.command_ready
                && row.output_ready
                && row.parser_ready
                && row.expected_result_ready
                && row.report_ready
                && row.schema_ready
                && row.local_proof_ready
                && row.coverage_status == "complete"
        }));

        let contamination_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.contamination" && row.tool_id == "verifybamid2")
            .expect("bam.contamination verifybamid2 row");
        assert_eq!(contamination_row.expected_normalized_metrics_output_id, "contamination_report");
        assert_eq!(contamination_row.report_section_id, "ancient_signal");
        assert_eq!(contamination_row.summary_table_id, "damage_authenticity");
        assert_eq!(contamination_row.expected_local_proof_kind, "local_ready_plan");
        assert_eq!(contamination_row.local_proof_tool_id, "verifybamid2");
        assert!(contamination_row
            .required_local_proof_fields
            .iter()
            .any(|field| field == "reference_panels"));
        assert!(contamination_row
            .local_proof_declared_output_ids
            .iter()
            .any(|output| output == "contamination_estimate"));

        let sex_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.sex" && row.tool_id == "rxy")
            .expect("bam.sex rxy row");
        assert_eq!(sex_row.expected_normalized_metrics_output_id, "sex_report");
        assert_eq!(sex_row.report_section_id, "sample_identity");
        assert_eq!(sex_row.summary_table_id, "identity_inference");
        assert_eq!(sex_row.expected_local_proof_kind, "local_smoke");
        assert_eq!(sex_row.local_proof_tool_id, "rxy");
        assert!(sex_row
            .required_local_proof_fields
            .iter()
            .any(|field| field == "autosomal_coverage"));
        assert!(sex_row
            .local_proof_artifact_paths
            .iter()
            .any(|path| path == "artifacts/runs/bench/local-smoke/bam.sex/sex.json"));

        #[cfg(feature = "bam_downstream")]
        {
            let haplogroups_row = report
                .rows
                .iter()
                .find(|row| row.stage_id == "bam.haplogroups" && row.tool_id == "yleaf")
                .expect("bam.haplogroups yleaf row");
            assert_eq!(haplogroups_row.expected_normalized_metrics_output_id, "haplogroups");
            assert_eq!(haplogroups_row.report_section_id, "sample_identity");
            assert_eq!(haplogroups_row.summary_table_id, "identity_inference");
            assert_eq!(haplogroups_row.expected_local_proof_kind, "local_ready_plan");
            assert_eq!(haplogroups_row.local_proof_tool_id, "yleaf");
            assert!(haplogroups_row
                .required_local_proof_fields
                .iter()
                .any(|field| field == "reference_panel_id"));
            assert!(haplogroups_row
                .local_proof_declared_output_ids
                .iter()
                .any(|output| output == "haplogroup_report"));
        }
    }
}
