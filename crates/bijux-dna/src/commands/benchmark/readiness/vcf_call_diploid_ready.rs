use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_genotypes::readiness::call_diploid::{
    VcfCallDiploidReadyReport, DEFAULT_VCF_CALL_DIPLOID_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_call_diploid_ready(
    args: &parse::BenchReadinessRenderVcfCallDiploidReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_call_diploid_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_CALL_DIPLOID_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_call_diploid_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfCallDiploidReadyReport> {
    let (commands, snapshot) =
        super::vcf_stage_readiness_support::collect_vcf_readiness_evidence(repo_root)?;
    render_vcf_call_diploid_ready_with_evidence(repo_root, output_path, &commands, &snapshot)
}

pub(crate) fn render_vcf_call_diploid_ready_with_evidence(
    repo_root: &Path,
    output_path: PathBuf,
    commands: &bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport,
    snapshot: &bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceSnapshot,
) -> Result<VcfCallDiploidReadyReport> {
    let rows = crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| row.stage_id == "vcf.call_diploid")
        .collect();
    let inputs = bijux_dna_bench_genotypes::inputs::GenotypeBenchmarkInputs::new(rows)?;
    let command_paths = bijux_dna_bench_genotypes::readiness::ReadinessCommandPaths {
        output_path: commands.output_path.clone(),
        argv_output_path: commands.argv_output_path.clone(),
    };
    bijux_dna_bench_genotypes::readiness::call_diploid::render_vcf_call_diploid_ready(
        &inputs,
        snapshot,
        &command_paths,
        repo_root,
        &output_path,
    )
}
