use super::bam_evidence::BamEvidenceAdapter;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/BAM_ACTIVE_ROW_CONSISTENCY.json";
pub(crate) type BamActiveRowConsistencySurfaceCheck =
    bijux_dna_bench_bam_evidence::consistency::BamActiveRowConsistencySurfaceCheck;
pub(crate) type BamActiveRowConsistencyReport =
    bijux_dna_bench_bam_evidence::consistency::BamActiveRowConsistencyReport;
pub(crate) fn run_render_bam_active_row_consistency(
    args: &parse::BenchReadinessRenderBamActiveRowConsistencyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_active_row_consistency(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_active_row_consistency(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamActiveRowConsistencyReport> {
    bijux_dna_bench_bam_evidence::consistency::render_bam_active_row_consistency(
        &BamEvidenceAdapter.context(repo_root),
        &output_path,
    )
}
