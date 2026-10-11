use crate::commands::benchmark_corpus_metadata::load_corpus_spec;
use crate::commands::benchmark_workspace::{
    benchmark_publication_contract, benchmark_runtime_corpus_dir_name,
    benchmark_stage_run_relative_root, benchmark_workspace_value, load_benchmark_config,
    load_benchmark_workspace_config,
};
use crate::commands::cli::{BenchCorpusFastqArgs, BenchWorkspaceValueArgs, Cli};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_campaign::request::{CorpusCampaignContract, CorpusCampaignRequest};
use std::path::{Path, PathBuf};
mod adapters;
mod defaults;
pub(crate) use bijux_dna_bench_campaign::artifacts::{
    artifact_bundle_size_bytes, sha256_artifact_bundle, sha256_file_hex,
};
pub(crate) fn print_benchmark_workspace_value(
    cwd: &Path,
    args: &BenchWorkspaceValueArgs,
) -> Result<()> {
    println!("{}", benchmark_workspace_value(cwd, args.config.as_deref(), &args.key)?);
    Ok(())
}

pub(crate) fn run_benchmark_corpus_fastq(cli: &Cli, args: &BenchCorpusFastqArgs) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let benchmark_config = load_benchmark_config(&repo_root, args.config.as_deref())?;
    let workspace_config = load_benchmark_workspace_config(&repo_root, args.config.as_deref())?;
    let workspace_config_path =
        crate::commands::benchmark_workspace::benchmark_workspace_config_path(
            &repo_root,
            args.config.as_deref(),
        );
    let publication_config_path =
        crate::commands::benchmark_workspace::benchmark_publication_config_path(
            &repo_root,
            args.publication_config.as_deref(),
        );
    let contract = benchmark_publication_contract(
        &repo_root,
        args.publication_config.as_deref(),
        &args.corpus_id,
        &args.stage,
    )?;
    let corpus_spec = load_corpus_spec(&repo_root, args.config.as_deref(), &args.corpus_id)?;
    let corpus_root = if let Some(path) = args.corpus_root.as_ref() {
        absolutize(&repo_root, path)
    } else {
        workspace_config
            .remote
            .as_ref()
            .and_then(|row| row.corpus_root.as_deref())
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("workspace config is missing remote.corpus_root"))?
    };
    let out_root = if let Some(path) = args.out_root.as_ref() {
        absolutize(&repo_root, path)
    } else {
        default_stage_out_root(&workspace_config, &corpus_spec.corpus_id, &args.stage)?
    };

    let request = CorpusCampaignRequest {
        corpus_id: args.corpus_id.clone(),
        source: corpus_spec,
        contract: CorpusCampaignContract {
            stage_id: contract.stage_id,
            scenario_id: contract.scenario_id,
            sample_scope: contract.sample_scope,
            tools: contract.tools,
        },
        repo_root,
        corpus_root,
        out_root,
        config_path: workspace_config_path,
        publication_config_path,
        program: std::env::current_exe().context("resolve bijux-dna executable")?,
        platform: resolve_benchmark_platform(cli.platform.as_deref())?,
        tools: args.tools.clone(),
        threads: args.threads,
        jobs: args.jobs,
        sample_jobs: args.sample_jobs,
        sample_limit: args.sample_limit,
        dry_run: args.dry_run,
        resume: args.resume,
        stage_args: args.stage_args.clone(),
        manifest_args: args.manifest_args.clone(),
        defaults: defaults::stage_defaults(&benchmark_config, &args.stage),
    };
    let manifest = bijux_dna_bench_campaign::run_corpus_campaign(
        &request,
        &adapters::ConfiguredPaths(&workspace_config),
        &adapters::NativeExecution,
        &adapters::PublicationClock,
    )?;
    println!("stage_id={}", manifest.stage_id);
    println!("corpus_root={}", request.corpus_root.display());
    println!("out_root={}", request.out_root.display());
    println!("manifest={}", request.out_root.join("run_manifest.json").display());
    println!("samples_total={}", manifest.samples_total);
    println!("samples_failed={}", manifest.samples_failed);
    if manifest.samples_failed > 0 {
        return Err(anyhow!(
            "corpus benchmark failures: {} samples failed",
            manifest.samples_failed
        ));
    }
    Ok(())
}

fn resolve_benchmark_platform(explicit_platform: Option<&str>) -> Result<String> {
    if let Some(platform) = explicit_platform {
        let trimmed = platform.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("benchmark platform must be a non-empty identifier"));
        }
        return Ok(trimmed.to_string());
    }
    std::env::var("BIJUX_PLATFORM")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow!("benchmark platform must be declared with --platform or BIJUX_PLATFORM")
        })
}

fn default_stage_out_root(
    workspace_config: &crate::commands::benchmark_workspace::BenchmarkWorkspaceConfig,
    corpus_id: &str,
    stage_id: &str,
) -> Result<PathBuf> {
    let results_root = workspace_config
        .remote
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .ok_or_else(|| anyhow!("workspace config is missing remote.results_root"))?;
    let corpus_dir_name = benchmark_runtime_corpus_dir_name(workspace_config, corpus_id)?;
    Ok(PathBuf::from(results_root).join(benchmark_stage_run_relative_root(
        workspace_config,
        "remote",
        &corpus_dir_name,
        stage_id,
    )?))
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[cfg(test)]
mod contracts;
