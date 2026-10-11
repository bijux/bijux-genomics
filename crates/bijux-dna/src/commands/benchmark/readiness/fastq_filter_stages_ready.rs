use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::proofs::filter_stages::FASTQ_FILTER_STAGES_READY_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::proofs::filter_stages::{
    FastqFilterStagesReadyReport, DEFAULT_FASTQ_FILTER_STAGES_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_filter_stages_ready(
    args: &parse::BenchReadinessRenderFastqFilterStagesReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_filter_stages_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_FILTER_STAGES_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_filter_stages_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqFilterStagesReadyReport> {
    bijux_dna_bench_fastq::proofs::filter_stages::render_fastq_filter_stages_ready(
        &super::fastq_proof_evidence::schema(repo_root),
        &output_path,
    )
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_filter_stages_ready, DEFAULT_FASTQ_FILTER_STAGES_READY_PATH,
        FASTQ_FILTER_STAGES_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_fastq_filter_stages_ready_reports_complete_filter_bindings() {
        let root = repo_root();
        let report = render_fastq_filter_stages_ready(
            &root,
            PathBuf::from(DEFAULT_FASTQ_FILTER_STAGES_READY_PATH),
        )
        .expect("render FASTQ filter stages readiness");

        assert_eq!(report.schema_version, FASTQ_FILTER_STAGES_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_FILTER_STAGES_READY_PATH);
        assert_eq!(report.active_row_count, 7);
        assert_eq!(report.complete_row_count, 7);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 7);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.coverage_status_counts.get("complete"), Some(&7));

        let row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.filter_low_complexity" && row.tool_id == "bbduk")
            .expect("bbduk filter_low_complexity row");
        assert!(row.active_scope_ready);
        assert!(row.command_ready);
        assert!(row.output_ready);
        assert!(row.parser_ready);
        assert!(row.expected_result_ready);
        assert!(row.report_ready);
        assert!(row.schema_ready);
        assert_eq!(row.report_section_id, "read_cleanup");
        assert_eq!(row.summary_table_id, "cleanup_retention");
        assert_eq!(row.normalized_metrics_output_id.as_deref(), Some("filter_report_json"));
        assert!(row.schema_required_fields.iter().any(|field| field == "reads_retained"));
        assert!(row.schema_required_fields.iter().any(|field| field == "reads_removed"));
        assert_eq!(row.expected_normalized_metrics_output_id, "filter_report_json");
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.filter_low_complexity"
                && row.tool_id == "fastp"
                && row.coverage_status == "complete"
        }));
    }
}
