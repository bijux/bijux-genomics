//! Semantic admission for persisted local Slurm submission manifests.

use std::collections::BTreeSet;

use crate::{
    BenchError, BenchmarkLocalAllDomainSlurmSubmitJob, BenchmarkLocalAllDomainSlurmSubmitManifest,
    BenchmarkLocalSlurmSubmitJob, BenchmarkLocalSlurmSubmitManifest,
    BenchmarkLocalStageArtifactEntry, BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
};

/// Validate an all-domain Slurm submission manifest and its dependency closure.
///
/// # Errors
/// Returns an error when identities, job kinds, resources, paths, or dependencies drift.
pub fn validate_benchmark_local_all_domain_slurm_submit_manifest(
    manifest: &BenchmarkLocalAllDomainSlurmSubmitManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1 {
        return invalid("unsupported all-domain Slurm submit-manifest schema");
    }
    for (field, value) in [
        ("root_path", manifest.root_path.as_str()),
        ("manifest_path", manifest.manifest_path.as_str()),
        ("run_id", manifest.run_id.as_str()),
    ] {
        require_text(value, field)?;
    }
    let benchmark_jobs = manifest.jobs.iter().filter(|job| job.result_id.is_some()).count();
    let pipeline_jobs = manifest.jobs.len().saturating_sub(benchmark_jobs);
    if !has_extension(&manifest.manifest_path, "json")
        || manifest.jobs.is_empty()
        || manifest.job_count != manifest.jobs.len()
        || manifest.benchmark_job_count != benchmark_jobs
        || manifest.essential_pipeline_job_count != pipeline_jobs
        || manifest.dependency_count
            != manifest.jobs.iter().map(|job| job.dependencies.len()).sum::<usize>()
    {
        return invalid("all-domain Slurm submit-manifest aggregate does not close");
    }

    let mut job_ids = BTreeSet::new();
    let mut script_paths = BTreeSet::new();
    for job in &manifest.jobs {
        validate_all_domain_job(job)?;
        if !job_ids.insert(job.job_id_local.as_str())
            || !script_paths.insert(job.script_path.as_str())
        {
            return invalid("all-domain Slurm submit manifest repeats a persisted identity");
        }
    }
    for job in &manifest.jobs {
        let mut dependencies = BTreeSet::new();
        for dependency in &job.dependencies {
            require_text(dependency, "jobs[].dependencies")?;
            if dependency == &job.job_id_local
                || !dependencies.insert(dependency.as_str())
                || !job_ids.contains(dependency.as_str())
            {
                return invalid(
                    "all-domain Slurm submit manifest has an invalid dependency closure",
                );
            }
        }
    }
    Ok(())
}

fn validate_all_domain_job(job: &BenchmarkLocalAllDomainSlurmSubmitJob) -> Result<(), BenchError> {
    for (field, value) in [
        ("jobs[].job_id_local", job.job_id_local.as_str()),
        ("jobs[].domain", job.domain.as_str()),
        ("jobs[].stage_id", job.stage_id.as_str()),
        ("jobs[].tool_id", job.tool_id.as_str()),
        ("jobs[].corpus_id", job.corpus_id.as_str()),
        ("jobs[].asset_profile_id", job.asset_profile_id.as_str()),
        ("jobs[].script_path", job.script_path.as_str()),
        ("jobs[].stdout", job.stdout.as_str()),
        ("jobs[].stderr", job.stderr.as_str()),
    ] {
        require_text(value, field)?;
    }
    if !matches!(job.domain.as_str(), "fastq" | "bam" | "vcf")
        || job.stage_id.split_once('.').map(|(domain, _)| domain) != Some(job.domain.as_str())
    {
        return invalid("all-domain Slurm submit job has an invalid domain identity");
    }
    validate_optional_text(job.pipeline_id.as_deref(), "jobs[].pipeline_id")?;
    validate_optional_text(job.node_id.as_deref(), "jobs[].node_id")?;
    validate_optional_text(job.result_id.as_deref(), "jobs[].result_id")?;
    match (&job.pipeline_id, &job.node_id, &job.result_id) {
        (None, None, Some(_)) if job.job_id_local.starts_with("benchmark:") => {
            if !job.dependencies.is_empty() {
                return invalid("all-domain benchmark jobs cannot declare dependencies");
            }
        }
        (Some(pipeline_id), Some(node_id), None)
            if job.job_id_local == format!("pipeline:{pipeline_id}:{node_id}") => {}
        _ => return invalid("all-domain Slurm submit job kind does not close"),
    }
    if job.resources.cpus_per_task == 0 || job.resources.memory_mb == 0 {
        return invalid("all-domain Slurm submit job resources must be positive");
    }
    validate_time_limit(&job.resources.time_limit)?;
    if !has_extension(&job.script_path, "sbatch")
        || job.stdout == job.stderr
        || !has_file_name(&job.stdout, "stdout.log")
        || !has_file_name(&job.stderr, "stderr.log")
    {
        return invalid("all-domain Slurm submit job paths do not close");
    }
    if job.outputs.is_empty() {
        return invalid("all-domain Slurm submit job requires declared outputs");
    }
    let mut outputs = BTreeSet::new();
    for output in &job.outputs {
        require_text(output, "jobs[].outputs")?;
        if !outputs.insert(output.as_str()) {
            return invalid("all-domain Slurm submit job repeats an output path");
        }
    }
    Ok(())
}

/// Validate a local Slurm submission manifest and its dependency closure.
///
/// # Errors
/// Returns an error when identities, resources, paths, samples, or dependencies drift.
pub fn validate_benchmark_local_slurm_submit_manifest(
    manifest: &BenchmarkLocalSlurmSubmitManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1 {
        return invalid("unsupported local Slurm submit-manifest schema");
    }
    for (field, value) in [
        ("root_path", manifest.root_path.as_str()),
        ("manifest_path", manifest.manifest_path.as_str()),
        ("run_id", manifest.run_id.as_str()),
    ] {
        require_text(value, field)?;
    }
    if !has_extension(&manifest.manifest_path, "json")
        || manifest.jobs.is_empty()
        || manifest.job_count != manifest.jobs.len()
        || manifest.dependency_count
            != manifest.jobs.iter().map(|job| job.dependencies.len()).sum::<usize>()
    {
        return invalid("local Slurm submit-manifest aggregate does not close");
    }

    let mut job_names = BTreeSet::new();
    let mut stage_ids = BTreeSet::new();
    let mut script_paths = BTreeSet::new();
    let mut result_roots = BTreeSet::new();
    for job in &manifest.jobs {
        validate_job(job)?;
        let stage_id = job
            .stage_id
            .as_deref()
            .ok_or_else(|| BenchError::InvalidPolicy("local Slurm job requires stage_id".into()))?;
        if !job_names.insert(job.job_name.as_str())
            || !stage_ids.insert(stage_id)
            || !script_paths.insert(job.script_path.as_str())
            || !result_roots.insert(job.result_root.as_str())
        {
            return invalid("local Slurm submit manifest repeats a persisted identity");
        }
    }
    for job in &manifest.jobs {
        let mut dependencies = BTreeSet::new();
        for dependency in &job.dependencies {
            require_text(dependency, "jobs[].dependencies")?;
            if dependency == &job.job_name
                || !dependencies.insert(dependency.as_str())
                || !job_names.contains(dependency.as_str())
            {
                return invalid("local Slurm submit manifest has an invalid dependency closure");
            }
        }
    }
    Ok(())
}

fn validate_job(job: &BenchmarkLocalSlurmSubmitJob) -> Result<(), BenchError> {
    for (field, value) in [
        ("jobs[].job_name", job.job_name.as_str()),
        ("jobs[].domain", job.domain.as_str()),
        ("jobs[].tool_id", job.tool_id.as_str()),
        ("jobs[].script_path", job.script_path.as_str()),
        ("jobs[].result_root", job.result_root.as_str()),
        ("jobs[].compatibility_kind", job.compatibility_kind.as_str()),
        ("jobs[].compatibility_note", job.compatibility_note.as_str()),
        ("jobs[].logs.stdout_path", job.logs.stdout_path.as_str()),
        ("jobs[].logs.stderr_path", job.logs.stderr_path.as_str()),
    ] {
        require_text(value, field)?;
    }
    if !matches!(job.domain.as_str(), "fastq" | "bam") {
        return invalid("local Slurm submit job has an unsupported domain");
    }
    let stage_id = job
        .stage_id
        .as_deref()
        .ok_or_else(|| BenchError::InvalidPolicy("local Slurm job requires stage_id".into()))?;
    require_text(stage_id, "jobs[].stage_id")?;
    if stage_id.split_once('.').map(|(domain, _)| domain) != Some(job.domain.as_str())
        || job.pipeline_id.is_some()
        || job.job_name != stage_id.replace('.', "-")
    {
        return invalid("local Slurm submit job identity does not close");
    }
    if !matches!(job.compatibility_kind.as_str(), "fixture" | "asset_backed") {
        return invalid("local Slurm submit job has an unsupported compatibility kind");
    }
    validate_optional_text(job.corpus_id.as_deref(), "jobs[].corpus_id")?;
    validate_optional_text(job.sample_id.as_deref(), "jobs[].sample_id")?;
    validate_samples(job)?;
    if job.resources.cpus_per_task == 0 || job.resources.memory_mb == 0 {
        return invalid("local Slurm submit job resources must be positive");
    }
    validate_time_limit(&job.resources.time_limit)?;
    if !has_extension(&job.script_path, "sbatch")
        || job.logs.stdout_path == job.logs.stderr_path
        || job.logs.stdout_path != format!("{}/stdout.log", job.result_root.trim_end_matches('/'))
        || job.logs.stderr_path != format!("{}/stderr.log", job.result_root.trim_end_matches('/'))
    {
        return invalid("local Slurm submit job paths do not close");
    }
    validate_artifacts(&job.outputs)?;
    Ok(())
}

fn validate_samples(job: &BenchmarkLocalSlurmSubmitJob) -> Result<(), BenchError> {
    let mut sample_ids = BTreeSet::new();
    for sample_id in &job.sample_ids {
        require_text(sample_id, "jobs[].sample_ids")?;
        if !sample_ids.insert(sample_id.as_str()) {
            return invalid("local Slurm submit job repeats a sample identity");
        }
    }
    match (job.sample_id.as_deref(), job.sample_ids.as_slice()) {
        (Some(sample_id), [only]) if sample_id == only => Ok(()),
        (None, [] | [_, _, ..]) => Ok(()),
        _ => invalid("local Slurm submit job sample scope does not close"),
    }
}

fn validate_artifacts(artifacts: &[BenchmarkLocalStageArtifactEntry]) -> Result<(), BenchError> {
    if artifacts.is_empty() {
        return invalid("local Slurm submit job requires declared outputs");
    }
    let mut identities = BTreeSet::new();
    for artifact in artifacts {
        require_text(&artifact.artifact_id, "jobs[].outputs[].artifact_id")?;
        require_text(&artifact.path, "jobs[].outputs[].path")?;
        require_text(&artifact.role, "jobs[].outputs[].role")?;
        if !identities.insert((artifact.artifact_id.as_str(), artifact.path.as_str())) {
            return invalid("local Slurm submit job repeats an output artifact");
        }
    }
    Ok(())
}

fn validate_time_limit(value: &str) -> Result<(), BenchError> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.len() != 2 || !part.bytes().all(|byte| byte.is_ascii_digit()))
        || parts[1].parse::<u8>().map_or(true, |minutes| minutes >= 60)
        || parts[2].parse::<u8>().map_or(true, |seconds| seconds >= 60)
    {
        return invalid("local Slurm submit time limit must use HH:MM:SS");
    }
    Ok(())
}

fn has_extension(path: &str, expected: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn has_file_name(path: &str, expected: &str) -> bool {
    std::path::Path::new(path).file_name() == Some(std::ffi::OsStr::new(expected))
}

fn validate_optional_text(value: Option<&str>, field: &str) -> Result<(), BenchError> {
    if let Some(value) = value {
        require_text(value, field)?;
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local Slurm submit manifest requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
