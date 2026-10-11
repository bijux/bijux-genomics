use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_fastq::readiness::active::{
    FastqActiveStageToolMatrixCollection, FastqActiveStageToolMatrixReport,
    FastqActiveStageToolMatrixRow, DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH,
};
use bijux_dna_bench_fastq::readiness::inputs::{FastqAdapterCoverage, FastqScopeRow};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_active_stage_tool_matrix(
    args: &parse::BenchReadinessRenderFastqActiveStageToolMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_active_stage_tool_matrix(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_active_stage_tool_matrix(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqActiveStageToolMatrixReport> {
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::active::render_fastq_active_stage_tool_matrix(
        repo_root,
        &output_path,
        input,
    )
}
pub(crate) fn collect_fastq_active_stage_tool_matrix_rows(
    repo_root: &Path,
) -> Result<FastqActiveStageToolMatrixCollection> {
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::active::collect_fastq_active_stage_tool_matrix_rows(
        input.0, input.1,
    )
}
fn inputs(repo_root: &Path) -> Result<(FastqAdapterCoverage, Vec<FastqScopeRow>)> {
    let coverage =
        super::fastq_command_adapter_coverage::collect_fastq_command_adapter_coverage_rows(
            repo_root,
        )?;
    let active = super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter().map(|row| bijux_dna_bench_fastq::readiness::inputs::FastqScopeRow {
            domain: row.domain, stage_id: row.stage_id, tool_id: row.tool_id,
            corpus_id: row.corpus_id, asset_profile_id: row.asset_profile_id,
            adapter_id: row.adapter_id, parser_id: row.parser_id, schema_id: row.schema_id,
        }).collect();
    Ok((coverage, active))
}
