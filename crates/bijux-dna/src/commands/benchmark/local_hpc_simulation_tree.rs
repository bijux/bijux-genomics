use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_job_completion::resolve_local_hpc_job_result_paths;
use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};

pub(crate) fn remap_job_to_simulation_root(
    repo_root: &Path,
    simulation_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<BenchLocalAllDomainSlurmSubmitJob> {
    Ok(BenchLocalAllDomainSlurmSubmitJob {
        job_id_local: job.job_id_local.clone(),
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        pipeline_id: job.pipeline_id.clone(),
        node_id: job.node_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        result_id: job.result_id.clone(),
        script_path: job.script_path.clone(),
        stdout: remap_repo_relative_path(repo_root, simulation_root, &job.stdout)?,
        stderr: remap_repo_relative_path(repo_root, simulation_root, &job.stderr)?,
        outputs: job
            .outputs
            .iter()
            .map(|path| remap_repo_relative_path(repo_root, simulation_root, path))
            .collect::<Result<Vec<_>>>()?,
        dependencies: job.dependencies.clone(),
        resources: job.resources.clone(),
    })
}

pub(crate) fn write_stage_result_manifest(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    status: BenchStageResultStatus,
) -> Result<()> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    fs::create_dir_all(&result_paths.result_root)
        .with_context(|| format!("create {}", result_paths.result_root.display()))?;
    fs::write(repo_root.join(&job.stdout), "stdout\n")
        .with_context(|| format!("write {}", repo_root.join(&job.stdout).display()))?;
    fs::write(repo_root.join(&job.stderr), "stderr\n")
        .with_context(|| format!("write {}", repo_root.join(&job.stderr).display()))?;
    for output_path in &result_paths.declared_output_paths {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(output_path, "artifact\n")
            .with_context(|| format!("write {}", output_path.display()))?;
    }
    let exit_code = i32::from(status != BenchStageResultStatus::Succeeded);
    let manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: job.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: job.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: format!("sbatch {}", job.script_path) },
        runtime: BenchStageResultRuntimeV1 {
            mode: "hpc_dry_run_simulation".to_string(),
            status,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::Estimated,
            memory_mb: Some(f64::from(job.resources.memory_mb)),
            cpu_threads: Some(job.resources.cpus_per_task),
        },
        outputs: result_paths
            .declared_output_paths
            .iter()
            .map(|path| BenchStageResultOutputV1 {
                artifact_id: path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("artifact")
                    .to_string(),
                declared_path: path_relative_to_repo(repo_root, path),
                realized_path: path_relative_to_repo(repo_root, path),
                role: "declared_output".to_string(),
                optional: false,
                exists: true,
            })
            .collect(),
    };
    validate_stage_result_manifest(&manifest)?;
    bijux_dna_infra::atomic_write_json(&result_paths.stage_result_manifest_path, &manifest)?;
    Ok(())
}

pub(crate) fn remove_stage_result_manifest(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<()> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    fs::remove_file(&result_paths.stage_result_manifest_path)
        .with_context(|| format!("remove {}", result_paths.stage_result_manifest_path.display()))?;
    Ok(())
}

pub(crate) fn remove_one_declared_output(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<PathBuf> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    let stale_output = result_paths.declared_output_paths.first().ok_or_else(|| {
        anyhow!(
            "HPC simulation stale-output seed requires at least one declared output for `{}`",
            job.job_id_local
        )
    })?;
    fs::remove_file(stale_output).with_context(|| format!("remove {}", stale_output.display()))?;
    Ok(stale_output.clone())
}

fn remap_repo_relative_path(
    repo_root: &Path,
    simulation_root: &Path,
    original: &str,
) -> Result<String> {
    let original_path = Path::new(original);
    let absolute_original = if original_path.is_absolute() {
        original_path.to_path_buf()
    } else {
        repo_root.join(original_path)
    };
    let repo_relative = absolute_original.strip_prefix(repo_root).map_err(|_| {
        anyhow!(
            "HPC simulation can only remap repo-owned paths, got `{}`",
            absolute_original.display()
        )
    })?;
    Ok(path_relative_to_repo(repo_root, &simulation_root.join(repo_relative)))
}
