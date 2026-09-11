use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity as owner;
pub(crate) use owner::{BamDamageAuthenticityReadyReport, BamDamageAuthenticityReadyRow};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/damage-authenticity-ready.json";
#[cfg(test)]
const BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_damage_authenticity_ready.v1";
pub(crate) fn run_render_bam_damage_authenticity_ready(
    args: &parse::BenchReadinessRenderBamDamageAuthenticityReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_damage_authenticity_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_damage_authenticity_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamDamageAuthenticityReadyReport> {
    owner::render_bam_damage_authenticity_ready(
        &super::bam_readiness::context(repo_root),
        &output_path,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_damage_authenticity_ready, BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION,
        DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_damage_authenticity_ready_reports_governed_rows() {
        let root = repo_root();
        let report = render_bam_damage_authenticity_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH),
        )
        .expect("render BAM damage/authenticity readiness");

        assert_eq!(report.schema_version, BAM_DAMAGE_AUTHENTICITY_READY_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 8);
        assert_eq!(report.complete_row_count, 8);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.rows.len(), 8);
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

        let authenticity_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.authenticity" && row.tool_id == "authenticct")
            .expect("bam.authenticity authenticct row");
        assert_eq!(authenticity_row.expected_normalized_metrics_output_id, "authenticity_report");
        assert_eq!(authenticity_row.report_section_id, "ancient_signal");
        assert_eq!(authenticity_row.summary_table_id, "damage_authenticity");
        assert!(authenticity_row.required_local_smoke_fields.iter().any(|field| field == "score"));
        assert!(authenticity_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "mapping_summary"));
        assert!(authenticity_row.local_smoke_artifact_paths.iter().any(|path| {
            path == "artifacts/runs/bench/local-smoke/bam.authenticity/authenticity.json"
        }));

        let damage_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.damage" && row.tool_id == "mapdamage2")
            .expect("bam.damage mapdamage2 row");
        assert_eq!(damage_row.expected_normalized_metrics_output_id, "damage_report");
        assert_eq!(damage_row.report_section_id, "ancient_signal");
        assert_eq!(damage_row.summary_table_id, "damage_authenticity");
        assert!(damage_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "terminal_c_to_t_5p"));
        assert!(damage_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "damage_report"));
        assert!(damage_row
            .local_smoke_artifact_paths
            .iter()
            .any(|path| path == "artifacts/runs/bench/local-smoke/bam.damage/damage.json"));
    }
}
