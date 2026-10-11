use super::super::{
    local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob,
    local_hpc_asset_staging_manifest::{
        collect_hpc_asset_staging_manifest, LocalHpcAssetStagingManifest,
    },
    local_hpc_execution_resolver::{
        collect_hpc_execution_resolver_report, LocalHpcExecutionResolverReport,
    },
    local_hpc_job_completion::resolve_local_hpc_job_result_paths,
    local_hpc_job_resources::{load_local_hpc_job_resource_hints, LocalHpcJobResourceHint},
    local_hpc_selected_jobs::load_local_hpc_selected_jobs,
    path_resolution::ensure_path_stays_within_benchmark_readiness_hpc_root,
};
use anyhow::Result;
use bijux_dna_bench_hpc::candidate::{
    ports::{CandidateEvidenceProvider, CandidateResultPaths},
    CandidateContext,
};
use std::{collections::BTreeMap, path::Path};
struct LocalEvidence;
impl CandidateEvidenceProvider for LocalEvidence {
    fn assets(&self, root: &Path) -> Result<LocalHpcAssetStagingManifest> {
        collect_hpc_asset_staging_manifest(root)
    }
    fn execution(&self, root: &Path) -> Result<LocalHpcExecutionResolverReport> {
        collect_hpc_execution_resolver_report(root)
    }
    fn jobs(&self, root: &Path) -> Result<Vec<BenchLocalAllDomainSlurmSubmitJob>> {
        load_local_hpc_selected_jobs(root)
    }
    fn resources(
        &self,
        root: &Path,
    ) -> Result<BTreeMap<(String, String, String), LocalHpcJobResourceHint>> {
        load_local_hpc_job_resource_hints(root)
    }
    fn result_paths(
        &self,
        root: &Path,
        job: &BenchLocalAllDomainSlurmSubmitJob,
    ) -> Result<CandidateResultPaths> {
        let observed = resolve_local_hpc_job_result_paths(root, job)?;
        Ok(CandidateResultPaths {
            result_root: observed.result_root,
            stage_result_manifest_path: observed.stage_result_manifest_path,
        })
    }
    fn admit_output(&self, root: &Path, output: &Path) -> Result<()> {
        ensure_path_stays_within_benchmark_readiness_hpc_root(
            root,
            output,
            "HPC candidate run manifest output",
        )
    }
}
pub(super) fn context(repo_root: &Path) -> CandidateContext<'_> {
    CandidateContext { repo_root, evidence: &LocalEvidence }
}
