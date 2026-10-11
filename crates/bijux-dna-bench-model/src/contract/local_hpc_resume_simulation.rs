//! Semantic admission for persisted local HPC resume simulations.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcResumeSimulationJob, BenchmarkLocalHpcResumeSimulationReport,
    BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID,
};

/// Validate resume identities, dependency propagation, classification partitions, and proofs.
///
/// # Errors
/// Returns an error when persisted resume evidence is incomplete or inconsistent.
pub fn validate_benchmark_local_hpc_resume_simulation(
    report: &BenchmarkLocalHpcResumeSimulationReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1 {
        return invalid("unsupported local HPC resume-simulation schema");
    }
    for (value, field) in [
        (&report.manifest_path, "manifest_path"),
        (&report.output_path, "output_path"),
        (&report.simulation_root, "simulation_root"),
    ] {
        require_path(value, field)?;
    }
    let classified = report.valid_completed_job_count
        + report.failed_stage_result_job_count
        + report.invalid_stage_result_job_count
        + report.missing_stage_result_job_count
        + report.stale_partial_output_job_count;
    if report.job_count == 0
        || report.job_count != report.jobs.len()
        || report.job_count != report.benchmark_job_count + report.essential_pipeline_job_count
        || report.job_count != report.skip_job_count + report.rerun_job_count
        || report.job_count != classified
        || report.dependency_count
            != report.jobs.iter().map(|job| job.dependencies.len()).sum::<usize>()
        || report.dependency_rerun_job_count >= report.rerun_job_count
    {
        return invalid("local HPC resume-simulation aggregate does not close");
    }
    let ids = report.jobs.iter().map(|job| job.job_id_local.as_str()).collect::<BTreeSet<_>>();
    if ids.len() != report.jobs.len() {
        return invalid("local HPC resume simulation repeats a job identity");
    }
    let mut prior_ids = BTreeSet::new();
    let mut rerun_ids = BTreeSet::new();
    for job in &report.jobs {
        validate_job(job, &ids, &prior_ids, &rerun_ids)?;
        prior_ids.insert(job.job_id_local.as_str());
        if job.resume_action == "rerun" {
            rerun_ids.insert(job.job_id_local.as_str());
        }
    }
    let benchmark_jobs =
        report.jobs.iter().filter(|job| job.job_kind == "benchmark_result").count();
    let essential_pipeline_jobs =
        report.jobs.iter().filter(|job| job.job_kind == "essential_pipeline_node").count();
    let count_state =
        |state: &str| report.jobs.iter().filter(|job| job.completion_state == state).count();
    if report.benchmark_job_count != benchmark_jobs
        || report.essential_pipeline_job_count != essential_pipeline_jobs
        || report.valid_completed_job_count != count_state("valid_completed")
        || report.failed_stage_result_job_count != count_state("failed_stage_result_manifest")
        || report.invalid_stage_result_job_count != count_state("invalid_stage_result_manifest")
        || report.missing_stage_result_job_count != count_state("missing_stage_result_manifest")
        || report.stale_partial_output_job_count != count_state("stale_partial_outputs")
        || report.skip_job_count
            != report.jobs.iter().filter(|job| job.resume_action == "skip").count()
        || report.rerun_job_count != rerun_ids.len()
        || report.dependency_rerun_job_count
            != report.jobs.iter().filter(|job| job.reason == "upstream_dependency_rerun").count()
    {
        return invalid("local HPC resume-simulation counters do not match job rows");
    }
    validate_behavior(report)?;
    Ok(())
}

fn validate_job<'a>(
    job: &'a BenchmarkLocalHpcResumeSimulationJob,
    ids: &BTreeSet<&str>,
    prior_ids: &BTreeSet<&'a str>,
    prior_reruns: &BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&job.job_id_local, "jobs[].job_id_local"),
        (&job.job_kind, "jobs[].job_kind"),
        (&job.stage_id, "jobs[].stage_id"),
        (&job.tool_id, "jobs[].tool_id"),
        (&job.completion_state, "jobs[].completion_state"),
        (&job.completion_detail, "jobs[].completion_detail"),
        (&job.resume_action, "jobs[].resume_action"),
        (&job.reason, "jobs[].reason"),
    ] {
        require_text(value, field)?;
    }
    let identity_valid = match job.job_kind.as_str() {
        "benchmark_result" => {
            job.result_id.is_some() && job.pipeline_id.is_none() && job.node_id.is_none()
        }
        "essential_pipeline_node" => {
            job.result_id.is_none() && job.pipeline_id.is_some() && job.node_id.is_some()
        }
        _ => false,
    };
    if !identity_valid || job.dependency_count != job.dependencies.len() {
        return invalid("local HPC resume job identity or dependency count does not close");
    }
    for dependency in &job.dependencies {
        if dependency == &job.job_id_local
            || !ids.contains(dependency.as_str())
            || !prior_ids.contains(dependency.as_str())
        {
            return invalid("local HPC resume job order is not topological");
        }
    }
    if job.rerun_dependencies
        != job
            .dependencies
            .iter()
            .filter(|dependency| prior_reruns.contains(dependency.as_str()))
            .cloned()
            .collect::<Vec<_>>()
    {
        return invalid("local HPC resume dependency propagation does not close");
    }
    if !matches!(
        job.completion_state.as_str(),
        "valid_completed"
            | "failed_stage_result_manifest"
            | "invalid_stage_result_manifest"
            | "missing_stage_result_manifest"
            | "stale_partial_outputs"
    ) || !matches!(job.resume_action.as_str(), "skip" | "rerun")
        || job.present_output_count > job.declared_output_count
        || job.missing_output_paths.len()
            != job.declared_output_count.saturating_sub(job.present_output_count)
    {
        return invalid("local HPC resume action and completion classification do not close");
    }
    let expected_reason = if job.completion_state != "valid_completed" {
        job.completion_state.as_str()
    } else if job.rerun_dependencies.is_empty() {
        "valid_completed"
    } else {
        "upstream_dependency_rerun"
    };
    if job.reason != expected_reason
        || (job.resume_action == "skip") != (expected_reason == "valid_completed")
    {
        return invalid("local HPC resume action does not follow completion and dependency state");
    }
    let completion_valid = match job.completion_state.as_str() {
        "valid_completed" => {
            job.manifest_valid
                && job.runtime_status.as_deref() == Some("succeeded")
                && job.present_output_count == job.declared_output_count
                && job.missing_output_paths.is_empty()
        }
        "failed_stage_result_manifest" => {
            job.manifest_valid && job.runtime_status.as_deref() == Some("failed")
        }
        "invalid_stage_result_manifest" => !job.manifest_valid,
        "missing_stage_result_manifest" => {
            !job.manifest_valid && job.runtime_status.is_none() && job.present_output_count == 0
        }
        "stale_partial_outputs" => {
            job.manifest_valid
                && job.runtime_status.as_deref() == Some("succeeded")
                && job.present_output_count < job.declared_output_count
        }
        _ => false,
    };
    if !completion_valid {
        return invalid("local HPC resume completion evidence is inconsistent");
    }
    require_path(&job.result_root, "jobs[].result_root")?;
    require_path(&job.stage_result_manifest_path, "jobs[].stage_result_manifest_path")?;
    for path in &job.missing_output_paths {
        require_path(path, "jobs[].missing_output_paths[]")?;
    }
    Ok(())
}

fn validate_behavior(report: &BenchmarkLocalHpcResumeSimulationReport) -> Result<(), BenchError> {
    let behavior = &report.behavior;
    if behavior.failed_stage_result_job_id != BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID
        || behavior.dependency_rerun_job_id != BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID
        || behavior.unrelated_valid_job_id != BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID
        || behavior.missing_manifest_job_id != BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID
        || behavior.stale_partial_output_job_id != BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID
        || behavior.valid_completed_job_id != BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID
        || !behavior.proven
        || !behavior.failed_stage_result_reruns
        || !behavior.dependency_rerun_propagates
        || !behavior.unrelated_valid_job_skips
        || !behavior.missing_manifest_reruns
        || !behavior.stale_partial_output_reruns
        || !behavior.valid_completed_job_skips
    {
        return invalid("local HPC resume-simulation behavior proof is incomplete");
    }
    let job = |id: &str| report.jobs.iter().find(|job| job.job_id_local == id);
    let failed = job(&behavior.failed_stage_result_job_id);
    let dependency = job(&behavior.dependency_rerun_job_id);
    let unrelated = job(&behavior.unrelated_valid_job_id);
    let missing = job(&behavior.missing_manifest_job_id);
    let stale = job(&behavior.stale_partial_output_job_id);
    let valid = job(&behavior.valid_completed_job_id);
    if !matches!(failed, Some(job) if job.completion_state == "failed_stage_result_manifest" && job.resume_action == "rerun")
        || !matches!(dependency, Some(job) if job.reason == "upstream_dependency_rerun" && job.resume_action == "rerun" && job.rerun_dependencies.iter().any(|id| id == BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID))
        || !matches!(unrelated, Some(job) if job.completion_state == "valid_completed" && job.resume_action == "skip")
        || !matches!(missing, Some(job) if job.completion_state == "missing_stage_result_manifest" && job.resume_action == "rerun")
        || !matches!(stale, Some(job) if job.completion_state == "stale_partial_outputs" && job.resume_action == "rerun")
        || !matches!(valid, Some(job) if job.completion_state == "valid_completed" && job.resume_action == "skip")
    {
        return invalid("local HPC resume-simulation proof flags do not match job evidence");
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC resume simulation requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local HPC resume simulation requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
