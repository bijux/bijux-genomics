use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::readiness::insert_size_gc_bias as owner;
pub(crate) use owner::BamInsertSizeGcBiasReadyReport;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/insert-size-gc-bias-ready.json";
#[cfg(test)]
const BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_insert_size_gc_bias_ready.v1";
pub(crate) fn run_render_bam_insert_size_gc_bias_ready(
    args: &parse::BenchReadinessRenderBamInsertSizeGcBiasReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_insert_size_gc_bias_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_insert_size_gc_bias_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamInsertSizeGcBiasReadyReport> {
    owner::render_bam_insert_size_gc_bias_ready(
        &super::bam_readiness::context(repo_root),
        &output_path,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_insert_size_gc_bias_ready, BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION,
        DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_insert_size_gc_bias_ready_reports_governed_rows() {
        let root = repo_root();
        let report = render_bam_insert_size_gc_bias_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_INSERT_SIZE_GC_BIAS_READY_PATH),
        )
        .expect("render BAM insert-size/gc-bias readiness");

        assert_eq!(report.schema_version, BAM_INSERT_SIZE_GC_BIAS_READY_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 2);
        assert_eq!(report.complete_row_count, 2);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 8);
        assert_eq!(report.stage_count, 2);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert_eq!(report.rows.len(), 2);
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

        let insert_size_row = report
            .rows
            .iter()
            .find(|row| row.stage_id == "bam.insert_size")
            .expect("bam.insert_size row");
        assert_eq!(insert_size_row.expected_normalized_metrics_output_id, "insert_size_report");
        assert!(insert_size_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "read_pairs"));
        assert!(insert_size_row.local_smoke_artifact_paths.iter().any(
            |path| path == "artifacts/runs/bench/local-smoke/bam.insert_size/insert_size.json"
        ));

        let gc_bias_row =
            report.rows.iter().find(|row| row.stage_id == "bam.gc_bias").expect("bam.gc_bias row");
        assert_eq!(gc_bias_row.expected_normalized_metrics_output_id, "gc_bias_report");
        assert!(gc_bias_row
            .required_local_smoke_fields
            .iter()
            .any(|field| field == "gc_bias_score"));
        assert!(gc_bias_row
            .local_smoke_artifact_paths
            .iter()
            .any(|path| path.ends_with("/picard/gc_bias.tsv")));
    }
}
