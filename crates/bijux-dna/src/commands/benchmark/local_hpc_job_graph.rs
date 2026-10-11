use std::path::Path;

use anyhow::Result;
use bijux_dna_api::v1::api::bench::build_benchmark_local_hpc_job_graph;
pub(crate) use bijux_dna_api::v1::api::bench::{
    BenchmarkLocalHpcJobGraph as LocalHpcJobGraph,
    BenchmarkLocalHpcJobGraphNode as LocalHpcJobGraphNode,
};

use super::local_all_domain_slurm_submit_manifest::DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH;
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;

pub(crate) fn collect_local_hpc_job_graph(repo_root: &Path) -> Result<LocalHpcJobGraph> {
    build_benchmark_local_hpc_job_graph(
        DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
        load_local_hpc_selected_jobs(repo_root)?,
    )
}
