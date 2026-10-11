//! Owner: benchmark model dependency-source interpretation and diagnostic construction.
use crate::{
    BenchmarkLocalSlurmDependencyCheckJob, BenchmarkLocalSlurmDependencyCheckReport,
    BenchmarkLocalSlurmSubmitJob, BenchmarkLocalSlurmSubmitManifest,
    BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
};
use anyhow::{ensure, Context, Result};
use std::collections::BTreeSet;

/// Interpret explicit dependency sources without filesystem access or scheduler execution.
#[must_use]
pub fn inspect_benchmark_local_slurm_job_dependencies(
    job: &BenchmarkLocalSlurmSubmitJob,
    script_path: String,
    script_body: &str,
) -> BenchmarkLocalSlurmDependencyCheckJob {
    let manifest_dependencies = sorted_unique(job.dependencies.clone());
    let script_header_dependencies = sorted_unique(parse_slurm_header_dependencies(script_body));
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

    BenchmarkLocalSlurmDependencyCheckJob {
        job_name: job.job_name.clone(),
        stage_id: job.stage_id.clone(),
        script_path,
        manifest_dependencies,
        script_header_dependencies,
        dependency_source,
        ok: findings.is_empty(),
        findings,
    }
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

/// Build a complete diagnostic, preserving negative findings as report evidence.
/// This examines dependency ownership; it does not authorize submission or prove a DAG valid.
///
/// # Errors
/// Rejects missing/misbound job observations, inconsistent counts and malformed output evidence.
pub fn build_benchmark_local_slurm_dependency_report(
    root_path: String,
    manifest_path: String,
    report_path: String,
    manifest: &BenchmarkLocalSlurmSubmitManifest,
    jobs: Vec<BenchmarkLocalSlurmDependencyCheckJob>,
) -> Result<BenchmarkLocalSlurmDependencyCheckReport> {
    ensure!(
        manifest.job_count == manifest.jobs.len() && jobs.len() == manifest.jobs.len(),
        "dependency inspection requires the complete declared job inventory"
    );
    for (observed, declared) in jobs.iter().zip(&manifest.jobs) {
        ensure!(
            observed.job_name == declared.job_name
                && observed.stage_id == declared.stage_id
                && observed.manifest_dependencies == sorted_unique(declared.dependencies.clone()),
            "dependency observation differs from its declared job"
        );
    }
    let mut report_findings = Vec::new();
    if manifest.schema_version != BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1 {
        report_findings.push(format!(
            "unexpected submit manifest schema `{}` in {manifest_path}",
            manifest.schema_version
        ));
    }
    let findings_count =
        checked_sum(jobs.iter().map(|job| job.findings.len()).chain([report_findings.len()]))?;
    let report = BenchmarkLocalSlurmDependencyCheckReport {
        schema_version: BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1.to_string(),
        root_path,
        manifest_path,
        report_path,
        job_count: jobs.len(),
        manifest_dependency_count: checked_sum(
            jobs.iter().map(|job| job.manifest_dependencies.len()),
        )?,
        script_header_dependency_count: checked_sum(
            jobs.iter().map(|job| job.script_header_dependencies.len()),
        )?,
        findings_count,
        ok: findings_count == 0,
        report_findings,
        jobs,
    };
    crate::validate_benchmark_local_slurm_dependency_check(&report)?;
    Ok(report)
}

fn checked_sum(values: impl Iterator<Item = usize>) -> Result<usize> {
    values.into_iter().try_fold(0_usize, |total, value| {
        total.checked_add(value).context("dependency diagnostic count overflow")
    })
}
