use super::CandidateContext;
use anyhow::Result;
use bijux_dna_bench_model::{
    validate_benchmark_local_hpc_asset_staging_manifest,
    validate_benchmark_local_hpc_execution_resolver,
    BenchmarkLocalAllDomainSlurmSubmitJob as BenchLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcAssetStagingManifest as LocalHpcAssetStagingManifest,
    BenchmarkLocalHpcExecutionResolverReport as LocalHpcExecutionResolverReport,
    HpcJobResourceHints,
};
/// Canonical acquired reports and job declarations for one request.
pub(super) struct CandidateInputs {
    pub assets: LocalHpcAssetStagingManifest,
    pub execution: LocalHpcExecutionResolverReport,
    pub jobs: Vec<BenchLocalAllDomainSlurmSubmitJob>,
    pub resources: HpcJobResourceHints,
}
impl CandidateInputs {
    pub(super) fn acquire(context: &CandidateContext<'_>) -> Result<Self> {
        let assets = context.evidence.assets(context.repo_root)?;
        let execution = context.evidence.execution(context.repo_root)?;
        let jobs = context.evidence.jobs(context.repo_root)?;
        let resources = context.evidence.resources(context.repo_root)?;
        validate_benchmark_local_hpc_asset_staging_manifest(&assets)?;
        validate_benchmark_local_hpc_execution_resolver(&execution)?;
        Ok(Self { assets, execution, jobs, resources })
    }
}
