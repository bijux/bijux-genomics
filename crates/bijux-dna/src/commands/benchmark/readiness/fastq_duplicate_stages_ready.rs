use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::proofs::duplicate_stages::FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::proofs::duplicate_stages::{
    FastqDuplicateStagesReadyReport, DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_duplicate_stages_ready(
    args: &parse::BenchReadinessRenderFastqDuplicateStagesReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_duplicate_stages_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_duplicate_stages_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqDuplicateStagesReadyReport> {
    bijux_dna_bench_fastq::proofs::duplicate_stages::render_fastq_duplicate_stages_ready(
        &super::fastq_proof_evidence::schema(repo_root),
        &output_path,
    )
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_duplicate_stages_ready, DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH,
        FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION,
    };
    use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_fastq_duplicate_stages_ready_reports_complete_duplicate_bindings() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let report = render_fastq_duplicate_stages_ready(
            &root,
            PathBuf::from(DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH),
        )
        .expect("render FASTQ duplicate stages readiness");

        assert_eq!(report.schema_version, FASTQ_DUPLICATE_STAGES_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_DUPLICATE_STAGES_READY_PATH);
        assert_eq!(report.active_row_count, 3);
        assert_eq!(report.complete_row_count, 3);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 7);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.coverage_status_counts.get("complete"), Some(&3));

        let detect_row = report
            .rows
            .iter()
            .find(|row| {
                row.stage_id == "fastq.detect_duplicates_premerge" && row.tool_id == "bijux_dna"
            })
            .expect("detect_duplicates_premerge row");
        assert!(detect_row.active_scope_ready);
        assert!(detect_row.command_ready);
        assert!(detect_row.output_ready);
        assert!(detect_row.parser_ready);
        assert!(detect_row.expected_result_ready);
        assert!(detect_row.report_ready);
        assert!(detect_row.schema_ready);
        assert_eq!(detect_row.report_section_id, "quality_profiling");
        assert_eq!(detect_row.summary_table_id, "premerge_complexity");
        assert_eq!(
            detect_row.normalized_metrics_output_id.as_deref(),
            Some("duplicate_signal_report")
        );
        assert!(detect_row
            .schema_required_fields
            .iter()
            .any(|field| field == "inspected_pair_count"));
    }
}
