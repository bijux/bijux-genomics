//! Owner: pure HPC scratch job projection, cleanup policy and layout admission.
use crate::{
    validate_benchmark_local_hpc_scratch_layout,
    BenchmarkLocalAllDomainSlurmSubmitJob as SubmitJob,
    BenchmarkLocalHpcScratchCleanupPolicy as CleanupPolicy,
    BenchmarkLocalHpcScratchInputLink as InputLink,
    BenchmarkLocalHpcScratchJobResources as Resources, BenchmarkLocalHpcScratchLayout as Layout,
    BenchmarkLocalHpcScratchLayoutJob as Job, BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
const CLEANUP_POLICY_ID: &str = "copy_back_outputs_remove_successful_scratch";
/// Pure current job inputs, with source observations supplied by the repository owner.
pub struct HpcScratchJobRequest<'a> {
    pub repository_root: &'a Path,
    pub job: &'a SubmitJob,
    pub scratch_root: PathBuf,
    pub input_links_root: PathBuf,
    pub input_links: Vec<InputLink>,
    pub resources: &'a Resources,
}
/// Derive a job layout without filesystem effects.
/// # Errors
/// Rejects missing output roots and divergent log roots.
pub fn project_hpc_scratch_job(request: HpcScratchJobRequest<'_>) -> Result<Job> {
    let HpcScratchJobRequest {
        repository_root: repo_root,
        job,
        scratch_root: job_scratch_root,
        input_links_root,
        input_links,
        resources,
    } = request;
    let result_id = job.result_id.clone();
    let pipeline_id = job.pipeline_id.clone();
    let node_id = job.node_id.clone();
    Ok(Job {
        job_id_local: job.job_id_local.clone(),
        job_kind: if result_id.is_some() {
            "benchmark_result".to_string()
        } else {
            "essential_pipeline_node".to_string()
        },
        result_id,
        pipeline_id,
        node_id,
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        scratch_root: path_relative_to_repo(repo_root, &job_scratch_root),
        input_links_root: path_relative_to_repo(repo_root, &input_links_root),
        output_root: output_root_for_job(&job.outputs)?,
        log_root: log_root_for_job(&job.stdout, &job.stderr)?,
        stdout_path: job.stdout.clone(),
        stderr_path: job.stderr.clone(),
        resources: Resources {
            cpus_per_task: resources.cpus_per_task,
            memory_mb: resources.memory_mb,
            time_limit: resources.time_limit.clone(),
            scratch_gb: resources.scratch_gb,
        },
        cleanup_policy: default_cleanup_policy(),
        input_links,
    })
}

/// Assemble and admit current jobs while preserving their governed order.
/// # Errors
/// Rejects invalid lineage, resources, source links, cleanup policy or root overlap.
pub fn assemble_hpc_scratch_layout(
    output_path: String,
    staging_root: String,
    scratch_root: String,
    jobs: Vec<Job>,
) -> Result<Layout> {
    let manifest = Layout {
        schema_version: BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1.to_string(),
        output_path,
        staging_root,
        scratch_root,
        selected_job_count: jobs.len(),
        benchmark_job_count: jobs.iter().filter(|job| job.result_id.is_some()).count(),
        essential_pipeline_job_count: jobs.iter().filter(|job| job.pipeline_id.is_some()).count(),
        input_link_count: jobs.iter().map(|job| job.input_links.len()).sum(),
        jobs,
    };
    validate_benchmark_local_hpc_scratch_layout(&manifest)?;
    Ok(manifest)
}
fn path_relative_to_repo(root: &Path, path: &Path) -> String {
    path_to_string(path.strip_prefix(root).unwrap_or(path))
}
fn output_root_for_job(outputs: &[String]) -> Result<String> {
    if outputs.is_empty() {
        return Err(anyhow!("HPC scratch layout job must keep at least one output path"));
    }
    if outputs.len() == 1 {
        return Path::new(&outputs[0]).parent().map(path_to_string).ok_or_else(|| {
            anyhow!("HPC scratch layout output path `{}` has no parent", outputs[0])
        });
    }
    common_path_prefix(outputs)
}

fn log_root_for_job(stdout: &str, stderr: &str) -> Result<String> {
    let stdout_parent = Path::new(stdout)
        .parent()
        .ok_or_else(|| anyhow!("stdout path `{stdout}` has no parent"))?;
    let stderr_parent = Path::new(stderr)
        .parent()
        .ok_or_else(|| anyhow!("stderr path `{stderr}` has no parent"))?;
    if stdout_parent != stderr_parent {
        return Err(anyhow!(
            "stdout and stderr roots diverged for scratch layout: `{stdout}` vs `{stderr}`"
        ));
    }
    Ok(path_to_string(stdout_parent))
}

fn common_path_prefix(paths: &[String]) -> Result<String> {
    let mut prefix = PathBuf::from(&paths[0]);
    while !paths.iter().all(|path| Path::new(path).starts_with(&prefix)) {
        if !prefix.pop() {
            return Err(anyhow!(
                "scratch layout could not derive a shared output root from `{}`",
                paths.join("`, `")
            ));
        }
    }
    Ok(path_to_string(&prefix))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn default_cleanup_policy() -> CleanupPolicy {
    CleanupPolicy {
        policy_id: CLEANUP_POLICY_ID.to_string(),
        remove_scratch_after_successful_sync: true,
        retain_scratch_on_failure: true,
        preserve_output_root: true,
        preserve_log_root: true,
        preserve_staged_inputs: true,
    }
}

/// Compare admitted persisted layout with current source observations.
/// # Errors
/// Rejects any field drift, including equal-count input substitutions.
pub fn compare_hpc_scratch_layout(
    manifest_path: &Path,
    observed: &Layout,
    expected: &Layout,
) -> Result<()> {
    if observed != expected {
        return Err(anyhow!(
            "HPC scratch layout `{}` drifted from the governed render",
            manifest_path.display()
        ));
    }
    Ok(())
}
