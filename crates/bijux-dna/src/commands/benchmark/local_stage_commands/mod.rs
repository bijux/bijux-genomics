//! Command adaptation for canonical stage plans and benchmark owner services.
#[cfg(test)]
mod contracts;
mod materialization;
mod planning;
mod qc;
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};
use crate::commands::cli::{parse, render};
use anyhow::Result;
use bijux_dna_bench_stage::execution::{build_local_stage_command_entry, publish_stage_commands};
pub(crate) use bijux_dna_bench_stage::execution::{
    materialize_declared_output, rendered_stage_materialize_argv,
    StagePlanBundle as BenchLocalStagePlanBundle,
};
#[cfg(all(test, feature = "bam_downstream"))]
use bijux_dna_bench_stage::BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3;
pub(crate) use bijux_dna_bench_stage::{
    BenchmarkLocalStageArtifactEntry as BenchLocalStageArtifactEntry,
    BenchmarkLocalStageCommandEntry as BenchLocalStageCommandEntry,
    BenchmarkLocalStageCommandManifest as BenchLocalStageCommandManifest,
};
pub(crate) use materialization::materialize_local_stage;
pub(crate) use planning::local_stage_plans;
#[cfg(test)]
use qc::materialize_local_report_qc_smoke_report;
use serde::Serialize;
use std::path::{Path, PathBuf};
const DEFAULT_RENDERED_STAGE_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.sh";
#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalStageMaterialization {
    pub(crate) stage_id: String,
    pub(crate) artifact_path: String,
}

pub(crate) fn run_materialize_stage(args: &parse::BenchLocalMaterializeStageArgs) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let artifact_path = materialize_local_stage(&repo_root, &args.stage_id)?;
    if args.json {
        render::json::print_pretty(&BenchLocalStageMaterialization {
            stage_id: args.stage_id.clone(),
            artifact_path: path_relative_to_repo(&repo_root, &artifact_path),
        })?;
    } else {
        println!("{}", path_relative_to_repo(&repo_root, &artifact_path));
    }
    Ok(())
}

pub(crate) fn run_bam_stage_smoke(args: &parse::BenchLocalRunBamStageSmokeArgs) -> Result<()> {
    super::local_bam_stage_smoke::run_bam_stage_smoke(args)
}

pub(crate) fn run_render_stage_commands(
    args: &parse::BenchLocalRenderStageCommandsArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let manifest = render_local_stage_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_RENDERED_STAGE_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.script_output_path);
    }
    Ok(())
}

pub(crate) fn bam_stage_smoke_command(stage_id: &str) -> Result<String> {
    super::local_bam_stage_smoke::bam_stage_smoke_command(stage_id)
}

pub(crate) fn bam_stage_smoke_support_path(
    repo_root: &Path,
    stage_id: &str,
) -> Result<Option<String>> {
    super::local_bam_stage_smoke::bam_stage_smoke_support_path(repo_root, stage_id)
}

pub(crate) fn governed_bam_local_smoke_tool_id(
    repo_root: &Path,
    stage_id: &str,
) -> Result<Option<String>> {
    super::local_bam_stage_smoke::governed_bam_local_smoke_tool_id(repo_root, stage_id)
}

pub(crate) fn has_bam_local_ready_only_contract(stage_id: &str) -> bool {
    super::local_bam_stage_smoke::has_bam_local_ready_only_contract(stage_id)
}

pub(crate) fn collect_local_stage_command_entries(
    repo_root: &Path,
    domain: Option<BenchLocalDomain>,
) -> Result<Vec<BenchLocalStageCommandEntry>> {
    collect_local_stage_plan_bundles(repo_root, domain)?
        .into_iter()
        .map(|bundle| build_local_stage_command_entry(repo_root, bundle))
        .collect()
}

pub(crate) fn collect_local_stage_plan_bundles(
    repo_root: &Path,
    domain: Option<BenchLocalDomain>,
) -> Result<Vec<BenchLocalStagePlanBundle>> {
    let domains = match domain {
        Some(domain) => vec![domain],
        None => vec![BenchLocalDomain::Fastq, BenchLocalDomain::Bam],
    };

    domains
        .into_iter()
        .map(|selected_domain| load_local_stage_inventory(repo_root, selected_domain))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|inventory| inventory.stages)
        .map(|stage| {
            Ok(BenchLocalStagePlanBundle {
                stage_id: stage.stage_id.clone(),
                readiness_kind: stage.readiness_kind,
                plans: local_stage_plans(repo_root, &stage.stage_id)?,
            })
        })
        .collect()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

pub(crate) fn render_local_stage_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BenchLocalStageCommandManifest> {
    publish_stage_commands(
        repo_root,
        &output_path,
        collect_local_stage_command_entries(repo_root, None)?,
    )
}
