use crate::commands::cli::{parse, render};
use anyhow::Context;
use anyhow::Result;
pub(crate) use bijux_dna_bench_bam_proofs::damage::{
    BamDamageCompleteReport, DEFAULT_BAM_DAMAGE_COMPLETE_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_damage_complete(
    args: &parse::BenchReadinessRenderBamDamageCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_damage_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_DAMAGE_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_damage_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamDamageCompleteReport> {
    bijux_dna_bench_bam_proofs::damage::render_bam_damage_complete(
        &super::bam_stage_proofs::context(repo_root),
        &output_path,
    )
}
#[cfg(test)]
const BAM_DAMAGE_COMPLETE_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_damage_complete.v1";
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_damage_complete, BAM_DAMAGE_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_DAMAGE_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_damage_complete_reports_governed_metrics() {
        let root = repo_root();
        let report =
            render_bam_damage_complete(&root, PathBuf::from(DEFAULT_BAM_DAMAGE_COMPLETE_PATH))
                .expect("render BAM damage completion report");

        assert_eq!(report.schema_version, BAM_DAMAGE_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 5);
        assert_eq!(report.complete_row_count, 5);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 13);
        assert_eq!(report.missing_tool_ids, Vec::<String>::new());
        assert_eq!(report.unexpected_tool_ids, Vec::<String>::new());
        assert_eq!(report.violation_count, 0);
        assert!(report.toolset_ready);
        assert!(report.ok);

        let row =
            report.rows.iter().find(|row| row.tool_id == "mapdamage2").expect("mapdamage2 row");
        assert_eq!(row.stage_id, "bam.damage");
        assert_eq!(row.summary_schema_version, "bijux.bam.damage_evidence.v1");
        assert_eq!(row.parser_output_schema_version, "bijux.bam.damage.parser_output.v1");
        assert_eq!(row.normalized_metrics_schema_version, "bijux.bam.damage.stage_metrics.v1");
        assert!((row.summary_terminal_c_to_t_5p - 0.18).abs() <= 1e-9);
        assert!((row.summary_terminal_g_to_a_3p - 0.11).abs() <= 1e-9);
        assert!((row.summary_short_fragment_fraction - 1.0).abs() <= 1e-9);
        assert_eq!(row.summary_damage_signal, "moderate");
        assert!(row.local_smoke_expectation_matched);
        assert!(row.summary_ready);
        assert!(row.unified_metrics_ready);
        assert!(row.parser_output_contract_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.damage_metric_consistency_ready);
        assert_eq!(row.coverage_status, "complete");
    }
}
