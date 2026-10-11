use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_vcf::evidence::adapters::{ShellSyntaxObservation, VcfAdapterReadinessPorts};
pub(crate) use bijux_dna_bench_vcf::evidence::adapters::{
    VcfAdaptersReadyReport, DEFAULT_VCF_ADAPTERS_READY_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_vcf_adapters_ready(
    args: &parse::BenchReadinessRenderVcfAdaptersReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_adapters_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADAPTERS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_adapters_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdaptersReadyReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations::new(
        &inputs, repo_root,
    );
    let ports = VcfAdapterReadinessPorts {
        active_bindings: super::vcf_active_stage_tool_matrix::active_keys,
        inspect_shell: inspect_shell_syntax,
    };
    bijux_dna_bench_vcf::evidence::adapters::render_vcf_adapters_ready(
        crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID,
        &observations,
        &ports,
        &output_path,
    )
}
fn inspect_shell_syntax(repo_root: &Path, script: &Path) -> Result<ShellSyntaxObservation> {
    let output = bijux_dna_api::v1::api::run::run_command_with_context(
        "bash",
        &["-n".to_string(), script.display().to_string()],
        Some(repo_root),
        None,
    )?;
    Ok(ShellSyntaxObservation { exit_code: output.exit_code, stderr: output.stderr })
}
