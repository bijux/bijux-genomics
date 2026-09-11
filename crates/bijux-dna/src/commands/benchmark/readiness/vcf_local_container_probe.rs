use crate::commands::cli::{parse, render};
use anyhow::{anyhow, Context, Result};
pub(crate) use bijux_dna_bench_vcf::evidence::smoke::{
    VcfLocalContainerSmokeReport, VcfLocalContainerSmokeRow, DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
pub(crate) fn run_render_vcf_local_container_smoke(
    args: &parse::BenchReadinessRenderVcfLocalContainerSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_local_container_smoke(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_vcf_local_container_smoke(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfLocalContainerSmokeReport> {
    with_inputs(repo_root, |inputs| {
        bijux_dna_bench_vcf::evidence::smoke::render_vcf_local_container_smoke(
            repo_root,
            &output_path,
            inputs,
        )
    })
}
pub(crate) fn collect_vcf_local_container_smoke_rows(
    repo_root: &Path,
) -> Result<Vec<VcfLocalContainerSmokeRow>> {
    with_inputs(repo_root, |inputs| {
        bijux_dna_bench_vcf::evidence::smoke::collect_vcf_local_container_smoke_rows(
            repo_root, inputs,
        )
    })
}
fn with_inputs<T>(
    repo_root: &Path,
    run: impl FnOnce(&bijux_dna_bench_vcf::evidence::smoke::VcfSmokeInputs<'_>) -> Result<T>,
) -> Result<T> {
    let bindings = super::vcf_adapter_inputs::load(repo_root)?;
    let retained =
        super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows(repo_root)?;
    let defaults =
        crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows()?
            .into_iter()
            .map(|row| (row.stage_id, row.tool_id))
            .collect::<BTreeMap<_, _>>();
    let external = super::catalog::load_external_artifact_tool_contracts(repo_root)?
        .into_iter()
        .map(|contract| contract.tool_id)
        .collect::<BTreeSet<_>>();
    run(&bijux_dna_bench_vcf::evidence::smoke::VcfSmokeInputs {
        bindings: &bindings,
        retained: &retained,
        stage_defaults: &defaults,
        external_tool_ids: &external,
        host_support: host_smoke_source_path,
    })
}
fn host_smoke_source_path(repo_root: &Path, stage_id: &str) -> Result<Option<String>> {
    let suffix = stage_id
        .strip_prefix("vcf.")
        .ok_or_else(|| anyhow!("VCF local-container smoke expected a `vcf.*` stage id"))?;
    let relative_path = PathBuf::from(format!(
        "crates/bijux-dna/src/commands/benchmark/local_vcf_{suffix}_smoke.rs"
    ));
    let absolute_path = repo_root.join(&relative_path);
    if !absolute_path.is_file() {
        return Ok(None);
    }
    Ok(Some(path_relative_to_repo(repo_root, &absolute_path)))
}
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
