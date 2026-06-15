use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_VERSION: &str = "bijux.bench.local_slurm_shell_syntax.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmShellSyntaxReport {
    pub(crate) schema_version: &'static str,
    pub(crate) root_path: String,
    pub(crate) report_path: String,
    pub(crate) script_count: usize,
    pub(crate) findings_count: usize,
    pub(crate) ok: bool,
    pub(crate) report_findings: Vec<String>,
    pub(crate) scripts: Vec<BenchLocalSlurmShellSyntaxEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmShellSyntaxEntry {
    pub(crate) script_path: String,
    pub(crate) ok: bool,
    pub(crate) findings: Vec<String>,
}

pub(crate) fn run_validate_slurm_shell_syntax(
    args: &parse::BenchLocalValidateSlurmShellSyntaxArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let root_path =
        args.root.clone().unwrap_or_else(|| benchmark_paths.benchmark_slurm_dry_run_root());
    let report_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_slurm_dry_run_root().join("bash-n-report.json")
    });
    let report = validate_slurm_shell_syntax(&repo_root, root_path, report_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_path);
    }
    Ok(())
}

pub(crate) fn validate_slurm_shell_syntax(
    repo_root: &Path,
    root_path: PathBuf,
    report_path: PathBuf,
) -> Result<BenchLocalSlurmShellSyntaxReport> {
    let absolute_root = if root_path.is_absolute() { root_path } else { repo_root.join(root_path) };
    let absolute_report =
        if report_path.is_absolute() { report_path } else { repo_root.join(report_path) };
    ensure_path_stays_within_benchmark_runs_root(repo_root, &absolute_root, "slurm dry-run root")?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_report,
        "slurm shell syntax report output",
    )?;

    let mut script_paths = Vec::new();
    collect_sbatch_paths(&absolute_root, &mut script_paths)?;
    script_paths.sort();

    let scripts = script_paths
        .iter()
        .map(|path| inspect_slurm_script(repo_root, path))
        .collect::<Result<Vec<_>>>()?;

    let mut report_findings = Vec::new();
    if script_paths.is_empty() {
        report_findings.push(format!(
            "no .sbatch files found under {}",
            path_relative_to_repo(repo_root, &absolute_root)
        ));
    }

    let findings_count =
        report_findings.len() + scripts.iter().map(|entry| entry.findings.len()).sum::<usize>();
    let ok = findings_count == 0;

    let report = BenchLocalSlurmShellSyntaxReport {
        schema_version: LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_VERSION,
        root_path: path_relative_to_repo(repo_root, &absolute_root),
        report_path: path_relative_to_repo(repo_root, &absolute_report),
        script_count: scripts.len(),
        findings_count,
        ok,
        report_findings,
        scripts,
    };

    if let Some(parent) = absolute_report.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_report, &report)?;

    if report.ok {
        Ok(report)
    } else {
        Err(anyhow!("slurm shell syntax validation failed; see {}", report.report_path))
    }
}

fn inspect_slurm_script(
    repo_root: &Path,
    script_path: &Path,
) -> Result<BenchLocalSlurmShellSyntaxEntry> {
    let syntax = Command::new("bash")
        .arg("-n")
        .arg(script_path)
        .output()
        .with_context(|| format!("run bash -n on {}", script_path.display()))?;

    let mut findings = Vec::new();
    if !syntax.status.success() {
        let stderr = String::from_utf8_lossy(&syntax.stderr).trim().to_string();
        if stderr.is_empty() {
            findings.push("bash -n failed without stderr output".to_string());
        } else {
            findings.push(stderr);
        }
    }

    Ok(BenchLocalSlurmShellSyntaxEntry {
        script_path: path_relative_to_repo(repo_root, script_path),
        ok: findings.is_empty(),
        findings,
    })
}

fn collect_sbatch_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("read {}", root.display()))? {
        let entry = entry.with_context(|| format!("read entry in {}", root.display()))?;
        let path = entry.path();
        let file_type =
            entry.file_type().with_context(|| format!("read file type for {}", path.display()))?;
        if file_type.is_dir() {
            collect_sbatch_paths(&path, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("sbatch") {
            paths.push(path);
        }
    }
    Ok(())
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}
