use crate::commands::cli::{parse, render};
use anyhow::Result;
pub(crate) use bijux_dna_bench_variants::readiness::qc::{
    VcfQcReadyReport, DEFAULT_VCF_QC_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_qc_ready(
    args: &parse::BenchReadinessRenderVcfQcReadyArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_vcf_qc_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_QC_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_qc_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfQcReadyReport> {
    let (commands, snapshot) =
        super::vcf_stage_readiness_support::collect_vcf_readiness_evidence(repo_root)?;
    render_vcf_qc_ready_with_evidence(repo_root, output_path, &commands, &snapshot)
}

pub(crate) fn render_vcf_qc_ready_with_evidence(
    repo_root: &Path,
    output_path: PathBuf,
    commands: &bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport,
    snapshot: &bijux_dna_bench_vcf_evidence::snapshot::VcfEvidenceSnapshot,
) -> Result<VcfQcReadyReport> {
    let bindings =
        crate::commands::benchmark::vcf_benchmark_bindings::collect_vcf_qc_benchmark_binding_rows(
        )?;
    let inputs =
        bijux_dna_bench_variants::inputs::VariantBenchmarkInputs::new(Vec::new(), bindings)?;
    let command_paths = bijux_dna_bench_variants::readiness::ReadinessCommandPaths {
        output_path: commands.output_path.clone(),
        argv_output_path: commands.argv_output_path.clone(),
    };
    bijux_dna_bench_variants::readiness::qc::render_vcf_qc_ready(
        &inputs,
        snapshot,
        &command_paths,
        repo_root,
        &output_path,
    )
}
