use crate::commands::cli::{parse, render};
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_bam_proofs::kinship as owner;
pub(crate) use owner::{BamKinshipCompleteReport, DEFAULT_BAM_KINSHIP_COMPLETE_PATH};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_kinship_complete(
    args: &parse::BenchReadinessRenderBamKinshipCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_kinship_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_KINSHIP_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
#[cfg(feature = "bam_downstream")]
pub(crate) fn render_bam_kinship_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamKinshipCompleteReport> {
    owner::render_bam_kinship_complete(
        &super::bam_downstream_proofs::context(repo_root),
        &output_path,
    )
}
#[cfg(not(feature = "bam_downstream"))]
pub(crate) fn render_bam_kinship_complete(
    _repo_root: &Path,
    _output_path: PathBuf,
) -> Result<BamKinshipCompleteReport> {
    owner::unavailable()
}
