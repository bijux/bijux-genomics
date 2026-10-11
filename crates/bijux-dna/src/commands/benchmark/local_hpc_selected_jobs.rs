use std::path::Path;

use anyhow::{Context, Result};

use super::local_all_domain_slurm_scripts::DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT;
use super::local_all_domain_slurm_submit_manifest::{
    render_all_domain_slurm_submit_manifest, BenchLocalAllDomainSlurmSubmitJob,
    DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
};

pub(crate) fn load_local_hpc_selected_jobs(
    repo_root: &Path,
) -> Result<Vec<BenchLocalAllDomainSlurmSubmitJob>> {
    let path = repo_root.join(DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH);
    let manifest = render_all_domain_slurm_submit_manifest(
        repo_root,
        DEFAULT_ALL_DOMAIN_SLURM_DRY_RUN_ROOT.into(),
        DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH.into(),
    )
    .with_context(|| format!("render {}", path.display()))?;
    Ok(manifest.jobs)
}
