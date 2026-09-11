use crate::commands::cli::{parse, render};
use anyhow::Context;
use anyhow::Result;
pub(crate) use bijux_dna_bench_bam_proofs::contamination::{
    BamContaminationCompleteReport, DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_bam_contamination_complete(
    args: &parse::BenchReadinessRenderBamContaminationCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_contamination_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_contamination_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamContaminationCompleteReport> {
    bijux_dna_bench_bam_proofs::contamination::render_bam_contamination_complete(
        &super::bam_stage_proofs::context(repo_root),
        &output_path,
    )
}
