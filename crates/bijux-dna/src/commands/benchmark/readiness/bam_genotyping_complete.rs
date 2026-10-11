use crate::commands::cli::{parse, render};
use anyhow::Result;
use bijux_dna_bench_bam_proofs::genotyping as owner;
pub(crate) use owner::{BamGenotypingCompleteReport, DEFAULT_BAM_GENOTYPING_COMPLETE_PATH};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_genotyping_complete(
    args: &parse::BenchReadinessRenderBamGenotypingCompleteArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_bam_genotyping_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_GENOTYPING_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
#[cfg(feature = "bam_downstream")]
pub(crate) fn render_bam_genotyping_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamGenotypingCompleteReport> {
    owner::render_bam_genotyping_complete(
        &super::bam_downstream_proofs::context(repo_root),
        &output_path,
    )
}
#[cfg(not(feature = "bam_downstream"))]
pub(crate) fn render_bam_genotyping_complete(
    _repo_root: &Path,
    _output_path: PathBuf,
) -> Result<BamGenotypingCompleteReport> {
    owner::unavailable()
}
