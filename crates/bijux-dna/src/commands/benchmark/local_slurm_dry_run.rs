use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_slurm_run_paths::{
    collect_local_slurm_run_paths, BenchLocalSlurmRunPaths,
};
use crate::commands::benchmark::local_stage_commands::{
    collect_local_stage_command_entries, BenchLocalStageCommandEntry,
};
use crate::commands::benchmark::local_stage_inventory::BenchLocalDomain;
use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_SLURM_DRY_RUN_SCHEMA_VERSION: &str = "bijux.bench.local_slurm_dry_run.v1";
const DEFAULT_SLURM_TIME_LIMIT: &str = "04:00:00";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmDryRunReport {
    pub(crate) schema_version: &'static str,
    pub(crate) domain: &'static str,
    pub(crate) output_root: String,
    pub(crate) script_count: usize,
    pub(crate) scripts: Vec<BenchLocalSlurmScriptEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmScriptEntry {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
    pub(crate) script_path: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) result_root: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) command: String,
}

pub(crate) fn run_render_slurm_scripts(
    args: &parse::BenchLocalRenderSlurmScriptsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let domain = match args.domain {
        parse::BenchLocalDomainArg::Fastq => BenchLocalDomain::Fastq,
        parse::BenchLocalDomainArg::Bam => BenchLocalDomain::Bam,
    };
    let output_root = args
        .output_root
        .clone()
        .unwrap_or_else(|| benchmark_paths.benchmark_slurm_dry_run_root().join(domain.as_str()));
    let report = render_local_slurm_scripts(&repo_root, domain, output_root)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_root);
    }
    Ok(())
}

pub(crate) fn render_local_slurm_scripts(
    repo_root: &Path,
    domain: BenchLocalDomain,
    output_root: PathBuf,
) -> Result<BenchLocalSlurmDryRunReport> {
    validate_slurm_dry_run_domain_support(domain)?;
    let command_entries = collect_local_stage_command_entries(repo_root, Some(domain))?;
    let absolute_output_root =
        if output_root.is_absolute() { output_root } else { repo_root.join(output_root) };
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_output_root,
        "slurm dry-run output",
    )?;
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;
    let slurm_root = slurm_dry_run_root_from_output_root(&absolute_output_root, domain);
    let run_paths = collect_local_slurm_run_paths(repo_root, domain, &slurm_root)?;

    let scripts = command_entries
        .into_iter()
        .map(|entry| {
            let stage_id = entry.stage_id.clone();
            let paths = run_paths
                .get(&stage_id)
                .cloned()
                .with_context(|| format!("missing slurm run paths for `{stage_id}`"))?;
            write_slurm_script(repo_root, &absolute_output_root, entry, paths)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BenchLocalSlurmDryRunReport {
        schema_version: LOCAL_SLURM_DRY_RUN_SCHEMA_VERSION,
        domain: domain.as_str(),
        output_root: path_relative_to_repo(repo_root, &absolute_output_root),
        script_count: scripts.len(),
        scripts,
    })
}

#[cfg(feature = "bam_downstream")]
fn validate_slurm_dry_run_domain_support(_domain: BenchLocalDomain) -> Result<()> {
    Ok(())
}

#[cfg(not(feature = "bam_downstream"))]
fn validate_slurm_dry_run_domain_support(domain: BenchLocalDomain) -> Result<()> {
    match domain {
        BenchLocalDomain::Fastq => Ok(()),
        BenchLocalDomain::Bam => Err(anyhow::anyhow!(
            "domain `bam` requires the `bam_downstream` feature; rerun with `cargo run -p bijux-dna --features bam_downstream -- bench local render-slurm-scripts --domain bam`"
        )),
        BenchLocalDomain::Vcf => Err(anyhow::anyhow!(
            "domain `vcf` is governed by dedicated VCF adapter and smoke-rendering surfaces, not the FASTQ/BAM local slurm dry-run path"
        )),
    }
}

fn write_slurm_script(
    repo_root: &Path,
    output_root: &Path,
    entry: BenchLocalStageCommandEntry,
    run_paths: BenchLocalSlurmRunPaths,
) -> Result<BenchLocalSlurmScriptEntry> {
    let script_path = output_root.join(format!("{}.sbatch", entry.stage_id));
    fs::create_dir_all(&run_paths.result_root)
        .with_context(|| format!("create {}", run_paths.result_root.display()))?;
    fs::write(&script_path, build_slurm_script(repo_root, &entry, &run_paths))
        .with_context(|| format!("write {}", script_path.display()))?;
    Ok(BenchLocalSlurmScriptEntry {
        stage_id: entry.stage_id,
        tool_id: entry.tool_id,
        readiness_kind: entry.readiness_kind.as_str().to_string(),
        cpus_per_task: entry.threads,
        memory_mb: entry.memory_mb,
        time_limit: DEFAULT_SLURM_TIME_LIMIT.to_string(),
        script_path: path_relative_to_repo(repo_root, &script_path),
        stdout_path: path_relative_to_repo(repo_root, &run_paths.stdout_path),
        stderr_path: path_relative_to_repo(repo_root, &run_paths.stderr_path),
        result_root: path_relative_to_repo(repo_root, &run_paths.result_root),
        stage_result_manifest_path: path_relative_to_repo(
            repo_root,
            &run_paths.stage_result_manifest_path,
        ),
        command: entry.command,
    })
}

fn build_slurm_script(
    repo_root: &Path,
    entry: &BenchLocalStageCommandEntry,
    run_paths: &BenchLocalSlurmRunPaths,
) -> String {
    let job_name = entry.stage_id.replace('.', "-");
    let repo_root_absolute = repo_root.to_string_lossy().replace('\\', "/");
    format!(
        "#!/usr/bin/env bash\n\
set -euo pipefail\n\
\n\
#SBATCH --job-name={job_name}\n\
#SBATCH --chdir={repo_root}\n\
#SBATCH --cpus-per-task={threads}\n\
#SBATCH --mem={memory_mb}M\n\
#SBATCH --time={time_limit}\n\
#SBATCH --output={stdout_path}\n\
#SBATCH --error={stderr_path}\n\
\n\
# Governed local benchmark dry-run script.\n\
# Domain readiness kind: {readiness_kind}\n\
# Tool: {tool_id}\n\
\n\
REPO_ROOT={repo_root}\n\
RESULT_ROOT={result_root}\n\
STAGE_RESULT_MANIFEST_PATH={stage_result_manifest_path}\n\
STDOUT_PATH={stdout_path}\n\
STDERR_PATH={stderr_path}\n\
cd \"$REPO_ROOT\"\n\
mkdir -p \"$RESULT_ROOT\"\n\
\n\
{command}\n",
        job_name = shell_quote(&job_name),
        repo_root = shell_quote(&repo_root_absolute),
        threads = entry.threads,
        memory_mb = entry.memory_mb,
        time_limit = DEFAULT_SLURM_TIME_LIMIT,
        stdout_path = shell_quote(&path_relative_to_repo(repo_root, &run_paths.stdout_path)),
        stderr_path = shell_quote(&path_relative_to_repo(repo_root, &run_paths.stderr_path)),
        result_root = shell_quote(&path_relative_to_repo(repo_root, &run_paths.result_root)),
        stage_result_manifest_path =
            shell_quote(&path_relative_to_repo(repo_root, &run_paths.stage_result_manifest_path,)),
        readiness_kind = entry.readiness_kind.as_str(),
        tool_id = entry.tool_id,
        command = entry.command,
    )
}

fn slurm_dry_run_root_from_output_root(output_root: &Path, domain: BenchLocalDomain) -> PathBuf {
    let matches_domain_dir =
        output_root.file_name().and_then(|segment| segment.to_str()) == Some(domain.as_str());
    if matches_domain_dir {
        output_root.parent().unwrap_or(output_root).to_path_buf()
    } else {
        output_root.to_path_buf()
    }
}

fn shell_quote(value: &str) -> String {
    if value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/')) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| path.to_string_lossy().replace('\\', "/"))
}
