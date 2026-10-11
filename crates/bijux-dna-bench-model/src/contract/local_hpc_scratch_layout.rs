//! Validation for persisted local HPC scratch layouts.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalHpcScratchInputLink, BenchmarkLocalHpcScratchLayout,
    BenchmarkLocalHpcScratchLayoutJob, BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
};

const CLEANUP_POLICY_ID: &str = "copy_back_outputs_remove_successful_scratch";

/// Validate scratch isolation, job lineage, resource bounds, and exact counters.
///
/// # Errors
/// Returns an error when persisted scratch-layout evidence is incomplete or inconsistent.
pub fn validate_benchmark_local_hpc_scratch_layout(
    layout: &BenchmarkLocalHpcScratchLayout,
) -> Result<(), BenchError> {
    if layout.schema_version != BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local HPC scratch-layout schema `{}`",
            layout.schema_version
        ));
    }
    require_repository_path(&layout.output_path, "output_path")?;
    require_repository_path(&layout.staging_root, "staging_root")?;
    require_repository_path(&layout.scratch_root, "scratch_root")?;
    let output_path = Path::new(&layout.output_path);
    let staging_root = Path::new(&layout.staging_root);
    let scratch_root = Path::new(&layout.scratch_root);
    if staging_root == scratch_root
        || staging_root.starts_with(scratch_root)
        || scratch_root.starts_with(staging_root)
        || output_path.starts_with(staging_root)
        || output_path.starts_with(scratch_root)
    {
        return invalid("local HPC staging, scratch, and manifest roots must remain independent");
    }
    if layout.selected_job_count != layout.jobs.len() {
        return invalid("selected_job_count drifted from jobs");
    }
    let observed_input_links = layout.jobs.iter().map(|job| job.input_links.len()).sum::<usize>();
    if layout.input_link_count != observed_input_links {
        return invalid("input_link_count aligned with input links invariant failed");
    }

    let mut job_ids = BTreeSet::new();
    let mut benchmark_job_count = 0usize;
    let mut essential_pipeline_job_count = 0usize;
    for job in &layout.jobs {
        validate_job(job, &layout.scratch_root, &layout.staging_root)?;
        match job.job_kind.as_str() {
            "benchmark_result" => benchmark_job_count += 1,
            "essential_pipeline_node" => essential_pipeline_job_count += 1,
            _ => {}
        }
        if !job_ids.insert(job.job_id_local.as_str()) {
            return invalid(format!("duplicate local scratch job `{}`", job.job_id_local));
        }
    }
    if layout.benchmark_job_count != benchmark_job_count
        || layout.essential_pipeline_job_count != essential_pipeline_job_count
    {
        return invalid("benchmark and essential-pipeline counts drifted from job kinds");
    }
    Ok(())
}

fn validate_job(
    job: &BenchmarkLocalHpcScratchLayoutJob,
    shared_scratch_root: &str,
    shared_staging_root: &str,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&job.job_id_local, "jobs[].job_id_local"),
        (&job.domain, "jobs[].domain"),
        (&job.stage_id, "jobs[].stage_id"),
        (&job.tool_id, "jobs[].tool_id"),
        (&job.corpus_id, "jobs[].corpus_id"),
        (&job.asset_profile_id, "jobs[].asset_profile_id"),
    ] {
        require_text(value, field)?;
    }
    for (value, field) in [
        (&job.scratch_root, "jobs[].scratch_root"),
        (&job.input_links_root, "jobs[].input_links_root"),
        (&job.output_root, "jobs[].output_root"),
        (&job.log_root, "jobs[].log_root"),
        (&job.stdout_path, "jobs[].stdout_path"),
        (&job.stderr_path, "jobs[].stderr_path"),
    ] {
        require_repository_path(value, field)?;
    }
    match job.job_kind.as_str() {
        "benchmark_result" => {
            require_optional_text(job.result_id.as_deref(), "jobs[].result_id")?;
            require_absent(job.pipeline_id.as_deref(), "pipeline_id", job)?;
            require_absent(job.node_id.as_deref(), "node_id", job)?;
        }
        "essential_pipeline_node" => {
            require_absent(job.result_id.as_deref(), "result_id", job)?;
            require_optional_text(job.pipeline_id.as_deref(), "jobs[].pipeline_id")?;
            require_optional_text(job.node_id.as_deref(), "jobs[].node_id")?;
        }
        kind => return invalid(format!("unsupported scratch job kind `{kind}`")),
    }
    if job.scratch_root == shared_scratch_root
        || !Path::new(&job.scratch_root).starts_with(shared_scratch_root)
        || !Path::new(&job.input_links_root).starts_with(&job.scratch_root)
        || !Path::new(&job.stdout_path).starts_with(&job.log_root)
        || !Path::new(&job.stderr_path).starts_with(&job.log_root)
        || Path::new(&job.output_root).starts_with(&job.scratch_root)
        || Path::new(&job.log_root).starts_with(&job.scratch_root)
    {
        return invalid(format!("scratch roots escaped job `{}` isolation", job.job_id_local));
    }
    if job.resources.cpus_per_task == 0
        || job.resources.memory_mb == 0
        || job.resources.scratch_gb == 0
    {
        return invalid(format!("scratch job `{}` has an unbounded resource", job.job_id_local));
    }
    require_text(&job.resources.time_limit, "jobs[].resources.time_limit")?;
    let policy = &job.cleanup_policy;
    if policy.policy_id != CLEANUP_POLICY_ID
        || !policy.remove_scratch_after_successful_sync
        || !policy.retain_scratch_on_failure
        || !policy.preserve_output_root
        || !policy.preserve_log_root
        || !policy.preserve_staged_inputs
    {
        return invalid(format!("scratch job `{}` has an unsafe cleanup policy", job.job_id_local));
    }
    if job.input_links.is_empty() {
        return invalid(format!("scratch job `{}` has no input links", job.job_id_local));
    }
    let mut link_paths = BTreeSet::new();
    for input in &job.input_links {
        validate_input_link(input, job, shared_staging_root)?;
        if !link_paths.insert(input.link_path.as_str()) {
            return invalid(format!(
                "scratch job `{}` repeats input link `{}`",
                job.job_id_local, input.link_path
            ));
        }
    }
    Ok(())
}

fn validate_input_link(
    input: &BenchmarkLocalHpcScratchInputLink,
    job: &BenchmarkLocalHpcScratchLayoutJob,
    shared_staging_root: &str,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&input.source_path, "jobs[].input_links[].source_path"),
        (&input.staged_path, "jobs[].input_links[].staged_path"),
        (&input.link_path, "jobs[].input_links[].link_path"),
    ] {
        require_repository_path(value, field)?;
    }
    require_text(&input.source_kind, "jobs[].input_links[].source_kind")?;
    validate_optional_text(input.artifact_id.as_deref(), "jobs[].input_links[].artifact_id")?;
    validate_optional_text(input.artifact_role.as_deref(), "jobs[].input_links[].artifact_role")?;
    if !matches!(input.source_kind.as_str(), "file" | "directory" | "prefix_bundle") {
        return invalid(format!(
            "scratch job `{}` has unsupported input source kind `{}`",
            job.job_id_local, input.source_kind
        ));
    }
    if input.artifact_id.is_some() != input.artifact_role.is_some() {
        return invalid(format!(
            "scratch job `{}` has incomplete artifact identity",
            job.job_id_local
        ));
    }
    if input.staged_path == shared_staging_root
        || !Path::new(&input.staged_path).starts_with(shared_staging_root)
        || !Path::new(&input.link_path).starts_with(&job.input_links_root)
        || input.source_path == input.link_path
        || input.source_path == input.staged_path
        || input.staged_path == input.link_path
    {
        return invalid(format!(
            "scratch job `{}` has invalid input-link lineage",
            job.job_id_local
        ));
    }
    if input.checksum_sha256.len() != 64
        || !input.checksum_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return invalid(format!(
            "scratch job `{}` has an invalid input checksum",
            job.job_id_local
        ));
    }
    if input.member_count == 0 {
        return invalid(format!("scratch job `{}` has an empty input asset", job.job_id_local));
    }
    Ok(())
}

fn require_absent(
    value: Option<&str>,
    field: &str,
    job: &BenchmarkLocalHpcScratchLayoutJob,
) -> Result<(), BenchError> {
    if value.is_some() {
        return invalid(format!("scratch job `{}` cannot declare {field}", job.job_id_local));
    }
    Ok(())
}

fn validate_optional_text(value: Option<&str>, field: &str) -> Result<(), BenchError> {
    if let Some(value) = value {
        require_text(value, field)?;
    }
    Ok(())
}

fn require_optional_text<'a>(value: Option<&'a str>, field: &str) -> Result<&'a str, BenchError> {
    let value = value.ok_or_else(|| {
        BenchError::InvalidPolicy(format!("local HPC scratch layout requires {field}"))
    })?;
    require_text(value, field)?;
    Ok(value)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local HPC scratch layout requires non-empty {field}"));
    }
    Ok(())
}

fn require_repository_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local HPC scratch layout requires repository-relative {field} without traversal"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
