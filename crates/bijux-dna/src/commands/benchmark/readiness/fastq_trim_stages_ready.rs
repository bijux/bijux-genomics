use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::proofs::trim_stages::FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::proofs::trim_stages::{
    FastqTrimStagesReadyReport, DEFAULT_FASTQ_TRIM_STAGES_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_trim_stages_ready(
    args: &parse::BenchReadinessRenderFastqTrimStagesReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_trim_stages_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TRIM_STAGES_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_trim_stages_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqTrimStagesReadyReport> {
    bijux_dna_bench_fastq::proofs::trim_stages::render_fastq_trim_stages_ready(
        &super::fastq_proof_evidence::schema(repo_root),
        &output_path,
    )
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_trim_stages_ready, DEFAULT_FASTQ_TRIM_STAGES_READY_PATH,
        FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_fastq_trim_stages_ready_reports_complete_trim_bindings() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let report = render_fastq_trim_stages_ready(
            &root,
            PathBuf::from(DEFAULT_FASTQ_TRIM_STAGES_READY_PATH),
        )
        .expect("render FASTQ trim stages readiness");

        assert_eq!(report.schema_version, FASTQ_TRIM_STAGES_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_TRIM_STAGES_READY_PATH);
        assert_eq!(report.active_row_count, 19);
        assert_eq!(report.complete_row_count, 19);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 7);
        assert_eq!(report.stage_count, 3);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.coverage_status_counts.get("complete"), Some(&19));

        let row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.trim_polyg_tails" && row.tool_id == "fastp")
            .expect("fastp trim_polyg_tails row");
        assert!(row.active_scope_ready);
        assert!(row.command_ready);
        assert!(row.output_ready);
        assert!(row.parser_ready);
        assert!(row.expected_result_ready);
        assert!(row.report_ready);
        assert!(row.schema_ready);
        assert_eq!(row.report_section_id, "read_cleanup");
        assert_eq!(row.summary_table_id, "cleanup_retention");
        assert_eq!(row.normalized_metrics_output_id.as_deref(), Some("report_json"));
        assert!(row.schema_required_fields.iter().any(|field| field == "trimmed_tail_count"));
        assert!(row.schema_required_fields.iter().any(|field| field == "bases_trimmed_polyg"));
    }
}
