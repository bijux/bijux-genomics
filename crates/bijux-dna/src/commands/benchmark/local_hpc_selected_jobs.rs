use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use super::local_all_domain_slurm_scripts::DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT;
use super::local_all_domain_slurm_submit_manifest::{
    render_all_domain_slurm_submit_manifest, BenchLocalAllDomainSlurmSubmitJob,
    BenchLocalAllDomainSlurmSubmitResources, DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
};

#[derive(Debug, Clone, Deserialize)]
struct LoadedAllDomainSlurmSubmitManifest {
    jobs: Vec<LoadedAllDomainSlurmSubmitJob>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedAllDomainSlurmSubmitJob {
    job_id_local: String,
    domain: String,
    stage_id: String,
    pipeline_id: Option<String>,
    node_id: Option<String>,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
    result_id: Option<String>,
    script_path: String,
    stdout: String,
    stderr: String,
    outputs: Vec<String>,
    dependencies: Vec<String>,
    resources: LoadedAllDomainSlurmSubmitResources,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedAllDomainSlurmSubmitResources {
    cpus_per_task: u32,
    memory_mb: u32,
    time_limit: String,
}

pub(crate) fn load_local_hpc_selected_jobs(
    repo_root: &Path,
) -> Result<Vec<BenchLocalAllDomainSlurmSubmitJob>> {
    let path = repo_root.join(DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH);
    if !path.is_file() {
        render_all_domain_slurm_submit_manifest(
            repo_root,
            DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT.into(),
            DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH.into(),
        )
        .with_context(|| format!("render {}", path.display()))?;
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let loaded = serde_json::from_str::<LoadedAllDomainSlurmSubmitManifest>(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(loaded
        .jobs
        .into_iter()
        .map(|job| BenchLocalAllDomainSlurmSubmitJob {
            job_id_local: job.job_id_local,
            domain: job.domain,
            stage_id: job.stage_id,
            pipeline_id: job.pipeline_id,
            node_id: job.node_id,
            tool_id: job.tool_id,
            corpus_id: job.corpus_id,
            asset_profile_id: job.asset_profile_id,
            result_id: job.result_id,
            script_path: job.script_path,
            stdout: job.stdout,
            stderr: job.stderr,
            outputs: job.outputs,
            dependencies: job.dependencies,
            resources: BenchLocalAllDomainSlurmSubmitResources {
                cpus_per_task: job.resources.cpus_per_task,
                memory_mb: job.resources.memory_mb,
                time_limit: job.resources.time_limit,
            },
        })
        .collect())
}
