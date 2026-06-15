use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_result_paths::LOCAL_ALL_DOMAIN_SLURM_RUN_ID;
use super::local_all_domain_slurm_submit_manifest::{
    render_all_domain_slurm_submit_manifest, BenchLocalAllDomainSlurmSubmitJob,
};
use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_ALL_DOMAIN_SLURM_PATH_CONVENTION_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_slurm_path_convention.v1";
const DEFAULT_ALL_DOMAIN_SLURM_PATH_CONVENTION_REPORT_PATH: &str =
    "runs/bench/slurm-dry-run/all-domains/path-convention-check.json";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmPathConventionReport {
    pub(crate) schema_version: &'static str,
    pub(crate) root_path: String,
    pub(crate) manifest_path: String,
    pub(crate) report_path: String,
    pub(crate) job_count: usize,
    pub(crate) checked_path_count: usize,
    pub(crate) finding_count: usize,
    pub(crate) ok: bool,
    pub(crate) jobs: Vec<BenchLocalAllDomainSlurmPathConventionJob>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BenchLocalAllDomainSlurmPathConventionJob {
    pub(crate) job_id_local: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) sample_scope: Option<String>,
    pub(crate) checked_path_count: usize,
    pub(crate) ok: bool,
    pub(crate) findings: Vec<String>,
}

pub(crate) fn run_validate_all_domain_slurm_result_paths(
    args: &parse::BenchLocalValidateAllDomainSlurmResultPathsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let root_path = args
        .root
        .clone()
        .unwrap_or_else(|| benchmark_paths.benchmark_slurm_dry_run_root().join("all-domains"));
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_slurm_dry_run_root().join("all-domains/submit-manifest.json")
    });
    let report_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths
            .benchmark_slurm_dry_run_root()
            .join("all-domains/path-convention-check.json")
    });
    let report =
        validate_all_domain_slurm_result_paths(&repo_root, root_path, manifest_path, report_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_path);
    }
    Ok(())
}

pub(crate) fn validate_all_domain_slurm_result_paths(
    repo_root: &Path,
    root_path: PathBuf,
    manifest_path: PathBuf,
    report_path: PathBuf,
) -> Result<BenchLocalAllDomainSlurmPathConventionReport> {
    let absolute_root = repo_relative_path(repo_root, &root_path);
    let absolute_manifest = repo_relative_path(repo_root, &manifest_path);
    let absolute_report = repo_relative_path(repo_root, &report_path);
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_root,
        "all-domain slurm dry-run root",
    )?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_manifest,
        "all-domain slurm submit manifest output",
    )?;
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &absolute_report,
        "all-domain slurm path convention report output",
    )?;

    let manifest = render_all_domain_slurm_submit_manifest(
        repo_root,
        absolute_root.clone(),
        absolute_manifest.clone(),
    )?;

    let jobs = manifest.jobs.iter().map(inspect_job_paths).collect::<Result<Vec<_>>>()?;
    let checked_path_count = jobs.iter().map(|job| job.checked_path_count).sum::<usize>();
    let finding_count = jobs.iter().map(|job| job.findings.len()).sum::<usize>();
    let ok = finding_count == 0;

    let report = BenchLocalAllDomainSlurmPathConventionReport {
        schema_version: LOCAL_ALL_DOMAIN_SLURM_PATH_CONVENTION_SCHEMA_VERSION,
        root_path: path_relative_to_repo(repo_root, &absolute_root),
        manifest_path: path_relative_to_repo(repo_root, &absolute_manifest),
        report_path: path_relative_to_repo(repo_root, &absolute_report),
        job_count: jobs.len(),
        checked_path_count,
        finding_count,
        ok,
        jobs,
    };

    if let Some(parent) = absolute_report.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_report, &report)?;

    if report.ok {
        Ok(report)
    } else {
        Err(anyhow!(
            "all-domain slurm result path convention validation failed; see {}",
            report.report_path
        ))
    }
}

fn inspect_job_paths(
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<BenchLocalAllDomainSlurmPathConventionJob> {
    let mut findings = Vec::new();
    let mut checked_path_count = 0usize;
    let mut sample_scope = None;

    let stdout_scope = inspect_stdout_or_stderr_path(job, &job.stdout, "stdout", &mut findings)?;
    checked_path_count += 1;
    sample_scope = sample_scope.or(stdout_scope);

    let stderr_scope = inspect_stdout_or_stderr_path(job, &job.stderr, "stderr", &mut findings)?;
    checked_path_count += 1;
    sample_scope = sample_scope.or(stderr_scope);

    for output_path in &job.outputs {
        let output_scope = inspect_output_path(job, output_path, &mut findings)?;
        checked_path_count += 1;
        sample_scope = sample_scope.or(output_scope);
    }

    Ok(BenchLocalAllDomainSlurmPathConventionJob {
        job_id_local: job.job_id_local.clone(),
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        pipeline_id: job.pipeline_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        sample_scope,
        checked_path_count,
        ok: findings.is_empty(),
        findings,
    })
}

fn inspect_stdout_or_stderr_path(
    job: &BenchLocalAllDomainSlurmSubmitJob,
    path: &str,
    kind: &str,
    findings: &mut Vec<String>,
) -> Result<Option<String>> {
    let parsed = parse_run_path(job, path, kind, findings)?;
    let expected_name = if kind == "stdout" { "stdout.log" } else { "stderr.log" };
    if parsed.tail.first().map(String::as_str) != Some(expected_name) {
        findings.push(format!(
            "{kind} path for `{}` must end with `{expected_name}`, found `{path}`",
            job.job_id_local
        ));
    }
    Ok(parsed.sample_scope)
}

fn inspect_output_path(
    job: &BenchLocalAllDomainSlurmSubmitJob,
    path: &str,
    findings: &mut Vec<String>,
) -> Result<Option<String>> {
    let parsed = parse_run_path(job, path, "output", findings)?;
    if job.pipeline_id.is_some() {
        if parsed.tail.first().map(String::as_str) != Some("outputs") {
            findings.push(format!(
                "pipeline output path for `{}` must live under `outputs/`, found `{path}`",
                job.job_id_local
            ));
        }
    } else if parsed.tail.first().map(String::as_str) == Some("stage-result.json") {
        if parsed.tail.len() != 1 {
            findings.push(format!(
                "benchmark stage-result path for `{}` must not carry extra nested segments, found `{path}`",
                job.job_id_local
            ));
        }
    } else if parsed.tail.first().map(String::as_str) != Some("declared-outputs") {
        findings.push(format!(
            "benchmark output path for `{}` must live under `declared-outputs/` or be `stage-result.json`, found `{path}`",
            job.job_id_local
        ));
    }
    Ok(parsed.sample_scope)
}

struct ParsedRunPath {
    sample_scope: Option<String>,
    tail: Vec<String>,
}

fn parse_run_path(
    job: &BenchLocalAllDomainSlurmSubmitJob,
    path: &str,
    kind: &str,
    findings: &mut Vec<String>,
) -> Result<ParsedRunPath> {
    let expected_prefix =
        format!("runs/bench/slurm-dry-run/all-domains/runs/{LOCAL_ALL_DOMAIN_SLURM_RUN_ID}/");
    if !path.starts_with(&expected_prefix) {
        findings.push(format!(
            "{kind} path for `{}` must start with `{expected_prefix}`, found `{path}`",
            job.job_id_local
        ));
        return Ok(ParsedRunPath { sample_scope: None, tail: Vec::new() });
    }

    let suffix = &path[expected_prefix.len()..];
    let segments = suffix
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    match job.pipeline_id.as_deref() {
        Some(pipeline_id) => {
            parse_pipeline_run_path(job, kind, path, pipeline_id, &segments, findings)
        }
        None => parse_benchmark_run_path(job, kind, path, &segments, findings),
    }
}

fn parse_benchmark_run_path(
    job: &BenchLocalAllDomainSlurmSubmitJob,
    kind: &str,
    path: &str,
    segments: &[String],
    findings: &mut Vec<String>,
) -> Result<ParsedRunPath> {
    let minimum_len = 6usize;
    if segments.len() < minimum_len {
        findings.push(format!(
            "{kind} path for `{}` must include domain, stage, tool, corpus, scope, and file segments, found `{path}`",
            job.job_id_local
        ));
        return Ok(ParsedRunPath { sample_scope: None, tail: Vec::new() });
    }
    if segments[0] != job.domain {
        findings.push(format!(
            "{kind} path for `{}` must include domain `{}`, found `{}` in `{path}`",
            job.job_id_local, job.domain, segments[0]
        ));
    }
    if segments[1] != job.stage_id {
        findings.push(format!(
            "{kind} path for `{}` must include stage `{}`, found `{}` in `{path}`",
            job.job_id_local, job.stage_id, segments[1]
        ));
    }
    if segments[2] != job.tool_id {
        findings.push(format!(
            "{kind} path for `{}` must include tool `{}`, found `{}` in `{path}`",
            job.job_id_local, job.tool_id, segments[2]
        ));
    }
    if segments[3] != job.corpus_id {
        findings.push(format!(
            "{kind} path for `{}` must include corpus `{}`, found `{}` in `{path}`",
            job.job_id_local, job.corpus_id, segments[3]
        ));
    }

    let scope = segments[4].clone();
    if matches!(job.domain.as_str(), "fastq" | "bam") && scope.is_empty() {
        findings.push(format!(
            "{kind} path for `{}` must include a sample scope in `{path}`",
            job.job_id_local
        ));
    }
    if job.domain == "vcf" && scope != job.asset_profile_id {
        findings.push(format!(
            "{kind} path for `{}` must include asset profile `{}` for VCF benchmark rows, found `{}` in `{path}`",
            job.job_id_local, job.asset_profile_id, scope
        ));
    }

    Ok(ParsedRunPath {
        sample_scope: matches!(job.domain.as_str(), "fastq" | "bam").then_some(scope),
        tail: segments[5..].to_vec(),
    })
}

fn parse_pipeline_run_path(
    job: &BenchLocalAllDomainSlurmSubmitJob,
    kind: &str,
    path: &str,
    pipeline_id: &str,
    segments: &[String],
    findings: &mut Vec<String>,
) -> Result<ParsedRunPath> {
    let minimum_len = 7usize;
    if segments.len() < minimum_len {
        findings.push(format!(
            "{kind} path for `{}` must include domain, pipeline, node, tool, corpus, sample scope, and file segments, found `{path}`",
            job.job_id_local
        ));
        return Ok(ParsedRunPath { sample_scope: None, tail: Vec::new() });
    }
    if segments[0] != job.domain {
        findings.push(format!(
            "{kind} path for `{}` must include domain `{}`, found `{}` in `{path}`",
            job.job_id_local, job.domain, segments[0]
        ));
    }
    if segments[1] != pipeline_id {
        findings.push(format!(
            "{kind} path for `{}` must include pipeline `{pipeline_id}`, found `{}` in `{path}`",
            job.job_id_local, segments[1]
        ));
    }
    if segments[2] != job.stage_id {
        findings.push(format!(
            "{kind} path for `{}` must include node `{}`, found `{}` in `{path}`",
            job.job_id_local, job.stage_id, segments[2]
        ));
    }
    if segments[3] != job.tool_id {
        findings.push(format!(
            "{kind} path for `{}` must include tool `{}`, found `{}` in `{path}`",
            job.job_id_local, job.tool_id, segments[3]
        ));
    }
    if segments[4] != job.corpus_id {
        findings.push(format!(
            "{kind} path for `{}` must include corpus `{}`, found `{}` in `{path}`",
            job.job_id_local, job.corpus_id, segments[4]
        ));
    }
    if segments[5].is_empty() {
        findings.push(format!(
            "{kind} path for `{}` must include a pipeline sample scope in `{path}`",
            job.job_id_local
        ));
    }

    Ok(ParsedRunPath { sample_scope: Some(segments[5].clone()), tail: segments[6..].to_vec() })
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}
