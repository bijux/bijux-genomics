use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.local_slurm_dependency_check.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmDependencyCheckReport {
    pub(crate) schema_version: &'static str,
    pub(crate) root_path: String,
    pub(crate) manifest_path: String,
    pub(crate) report_path: String,
    pub(crate) job_count: usize,
    pub(crate) manifest_dependency_count: usize,
    pub(crate) script_header_dependency_count: usize,
    pub(crate) findings_count: usize,
    pub(crate) ok: bool,
    pub(crate) report_findings: Vec<String>,
    pub(crate) jobs: Vec<BenchLocalSlurmDependencyCheckJob>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalSlurmDependencyCheckJob {
    pub(crate) job_name: String,
    pub(crate) stage_id: Option<String>,
    pub(crate) script_path: String,
    pub(crate) manifest_dependencies: Vec<String>,
    pub(crate) script_header_dependencies: Vec<String>,
    pub(crate) dependency_source: String,
    pub(crate) ok: bool,
    pub(crate) findings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BenchLocalSlurmSubmitManifestRecord {
    schema_version: String,
    jobs: Vec<BenchLocalSlurmSubmitJobRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct BenchLocalSlurmSubmitJobRecord {
    job_name: String,
    stage_id: Option<String>,
    script_path: String,
    dependencies: Vec<String>,
}

pub(crate) fn run_validate_slurm_dependencies(
    args: &parse::BenchLocalValidateSlurmDependenciesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let root_path =
        args.root.clone().unwrap_or_else(|| benchmark_paths.benchmark_slurm_dry_run_root());
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_slurm_dry_run_root().join("submit-manifest.json")
    });
    let report_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_slurm_dry_run_root().join("dependency-check.json")
    });
    let report = validate_slurm_dependencies(&repo_root, root_path, manifest_path, report_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_path);
    }
    Ok(())
}

pub(crate) fn validate_slurm_dependencies(
    repo_root: &Path,
    root_path: PathBuf,
    manifest_path: PathBuf,
    report_path: PathBuf,
) -> Result<BenchLocalSlurmDependencyCheckReport> {
    let absolute_root = if root_path.is_absolute() { root_path } else { repo_root.join(root_path) };
    let absolute_manifest =
        if manifest_path.is_absolute() { manifest_path } else { repo_root.join(manifest_path) };
    let absolute_report =
        if report_path.is_absolute() { report_path } else { repo_root.join(report_path) };
    ensure_path_stays_within_benchmark_runs_root(repo_root, &absolute_root, "slurm dry-run root")?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_manifest,
        "slurm submit manifest input",
    )?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_report,
        "slurm dependency report output",
    )?;

    let manifest_bytes = fs::read(&absolute_manifest)
        .with_context(|| format!("read {}", absolute_manifest.display()))?;
    let manifest: BenchLocalSlurmSubmitManifestRecord = serde_json::from_slice(&manifest_bytes)
        .with_context(|| format!("parse {}", absolute_manifest.display()))?;

    let mut report_findings = Vec::new();
    if manifest.schema_version != "bijux.bench.local_slurm_submit_manifest.v1" {
        report_findings.push(format!(
            "unexpected submit manifest schema `{}` in {}",
            manifest.schema_version,
            path_relative_to_repo(repo_root, &absolute_manifest)
        ));
    }

    let mut jobs = Vec::with_capacity(manifest.jobs.len());
    for job in manifest.jobs {
        jobs.push(inspect_job_dependency_sources(repo_root, &absolute_root, job)?);
    }

    let manifest_dependency_count =
        jobs.iter().map(|job| job.manifest_dependencies.len()).sum::<usize>();
    let script_header_dependency_count =
        jobs.iter().map(|job| job.script_header_dependencies.len()).sum::<usize>();
    let findings_count =
        report_findings.len() + jobs.iter().map(|job| job.findings.len()).sum::<usize>();
    let ok = findings_count == 0;

    let report = BenchLocalSlurmDependencyCheckReport {
        schema_version: LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_VERSION,
        root_path: path_relative_to_repo(repo_root, &absolute_root),
        manifest_path: path_relative_to_repo(repo_root, &absolute_manifest),
        report_path: path_relative_to_repo(repo_root, &absolute_report),
        job_count: jobs.len(),
        manifest_dependency_count,
        script_header_dependency_count,
        findings_count,
        ok,
        report_findings,
        jobs,
    };

    if let Some(parent) = absolute_report.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_report, &report)?;

    if report.ok {
        Ok(report)
    } else {
        Err(anyhow!("slurm dependency validation failed; see {}", report.report_path))
    }
}

fn inspect_job_dependency_sources(
    repo_root: &Path,
    root_path: &Path,
    job: BenchLocalSlurmSubmitJobRecord,
) -> Result<BenchLocalSlurmDependencyCheckJob> {
    let script_path = resolve_script_path(repo_root, root_path, &job.script_path);
    let script_body = fs::read_to_string(&script_path)
        .with_context(|| format!("read {}", script_path.display()))?;
    let manifest_dependencies = sorted_unique(job.dependencies);
    let script_header_dependencies = sorted_unique(parse_slurm_header_dependencies(&script_body));
    let dependency_source =
        classify_dependency_source(&manifest_dependencies, &script_header_dependencies).to_string();

    let mut findings = Vec::new();
    if !manifest_dependencies.is_empty() && !script_header_dependencies.is_empty() {
        findings
            .push("dependencies are split across submit manifest and script header".to_string());
    }

    for dependency in union_dependencies(&manifest_dependencies, &script_header_dependencies) {
        let mut locations = 0;
        if manifest_dependencies.contains(&dependency) {
            locations += 1;
        }
        if script_header_dependencies.contains(&dependency) {
            locations += 1;
        }
        if locations > 1 {
            findings.push(format!(
                "dependency `{dependency}` appears in both submit manifest and script header"
            ));
        }
    }

    Ok(BenchLocalSlurmDependencyCheckJob {
        job_name: job.job_name,
        stage_id: job.stage_id,
        script_path: path_relative_to_repo(repo_root, &script_path),
        manifest_dependencies,
        script_header_dependencies,
        dependency_source,
        ok: findings.is_empty(),
        findings,
    })
}

fn resolve_script_path(repo_root: &Path, root_path: &Path, script_path: &str) -> PathBuf {
    let script_path = PathBuf::from(script_path);
    if script_path.is_absolute() {
        return script_path;
    }
    let from_repo_root = repo_root.join(&script_path);
    if from_repo_root.exists() {
        return from_repo_root;
    }
    root_path.join(script_path)
}

fn parse_slurm_header_dependencies(body: &str) -> Vec<String> {
    let mut dependencies = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("#SBATCH") {
            continue;
        }
        let payload = trimmed.trim_start_matches("#SBATCH").trim();
        let dependency_value = if let Some(value) = payload.strip_prefix("--dependency=") {
            Some(value.trim())
        } else if let Some(value) = payload.strip_prefix("--dependency") {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        } else {
            None
        };

        if let Some(value) = dependency_value {
            dependencies.extend(parse_dependency_value(value));
        }
    }
    dependencies
}

fn parse_dependency_value(value: &str) -> Vec<String> {
    value
        .split(',')
        .flat_map(|clause| {
            let clause = clause.trim();
            if clause.is_empty() {
                return Vec::new();
            }
            if let Some((_, jobs)) = clause.split_once(':') {
                jobs.split(':')
                    .map(str::trim)
                    .filter(|job| !job.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            } else {
                vec![clause.to_string()]
            }
        })
        .collect()
}

fn sorted_unique(values: Vec<String>) -> Vec<String> {
    let mut values = values.into_iter().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    values.sort();
    values
}

fn union_dependencies(left: &[String], right: &[String]) -> Vec<String> {
    left.iter().chain(right.iter()).cloned().collect::<BTreeSet<_>>().into_iter().collect()
}

fn classify_dependency_source(
    manifest_dependencies: &[String],
    script_header_dependencies: &[String],
) -> &'static str {
    match (manifest_dependencies.is_empty(), script_header_dependencies.is_empty()) {
        (true, true) => "none",
        (false, true) => "submit_manifest",
        (true, false) => "script_header",
        (false, false) => "mixed",
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}
