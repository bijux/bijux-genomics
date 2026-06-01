use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_stage_commands::{
    collect_local_stage_command_entries, BenchLocalStageCommandEntry,
};
use crate::commands::benchmark::local_stage_inventory::BenchLocalDomain;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_SLURM_DRY_RUN_SCHEMA_VERSION: &str = "bijux.bench.local_slurm_dry_run.v1";
const DEFAULT_SLURM_DRY_RUN_ROOT: &str = "target/slurm-dry-run";
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
    pub(crate) command: String,
}

pub(crate) fn run_render_slurm_scripts(
    args: &parse::BenchLocalRenderSlurmScriptsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let domain = match args.domain {
        parse::BenchLocalDomainArg::Fastq => BenchLocalDomain::Fastq,
        parse::BenchLocalDomainArg::Bam => BenchLocalDomain::Bam,
    };
    let output_root = args
        .output_root
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SLURM_DRY_RUN_ROOT).join(domain.as_str()));
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
    let command_entries = collect_local_stage_command_entries(repo_root, Some(domain))?;
    let absolute_output_root =
        if output_root.is_absolute() { output_root } else { repo_root.join(output_root) };
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;

    let scripts = command_entries
        .into_iter()
        .map(|entry| write_slurm_script(repo_root, &absolute_output_root, entry))
        .collect::<Result<Vec<_>>>()?;

    Ok(BenchLocalSlurmDryRunReport {
        schema_version: LOCAL_SLURM_DRY_RUN_SCHEMA_VERSION,
        domain: domain.as_str(),
        output_root: path_relative_to_repo(repo_root, &absolute_output_root),
        script_count: scripts.len(),
        scripts,
    })
}

fn write_slurm_script(
    repo_root: &Path,
    output_root: &Path,
    entry: BenchLocalStageCommandEntry,
) -> Result<BenchLocalSlurmScriptEntry> {
    let script_path = output_root.join(format!("{}.sbatch", entry.stage_id));
    fs::write(&script_path, build_slurm_script(repo_root, &entry))
        .with_context(|| format!("write {}", script_path.display()))?;
    Ok(BenchLocalSlurmScriptEntry {
        stage_id: entry.stage_id,
        tool_id: entry.tool_id,
        readiness_kind: entry.readiness_kind.as_str().to_string(),
        cpus_per_task: entry.threads,
        memory_mb: entry.memory_mb,
        time_limit: DEFAULT_SLURM_TIME_LIMIT.to_string(),
        script_path: path_relative_to_repo(repo_root, &script_path),
        command: entry.command,
    })
}

fn build_slurm_script(repo_root: &Path, entry: &BenchLocalStageCommandEntry) -> String {
    let job_name = entry.stage_id.replace('.', "-");
    format!(
        "#!/usr/bin/env bash\n\
set -euo pipefail\n\
\n\
#SBATCH --job-name={job_name}\n\
#SBATCH --cpus-per-task={threads}\n\
#SBATCH --mem={memory_mb}M\n\
#SBATCH --time={time_limit}\n\
\n\
# Governed local benchmark dry-run script.\n\
# Domain readiness kind: {readiness_kind}\n\
# Tool: {tool_id}\n\
\n\
REPO_ROOT={repo_root}\n\
cd \"$REPO_ROOT\"\n\
\n\
{command}\n",
        job_name = shell_quote(&job_name),
        threads = entry.threads,
        memory_mb = entry.memory_mb,
        time_limit = DEFAULT_SLURM_TIME_LIMIT,
        readiness_kind = entry.readiness_kind.as_str(),
        tool_id = entry.tool_id,
        repo_root = shell_quote(&repo_root.display().to_string()),
        command = entry.command,
    )
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
