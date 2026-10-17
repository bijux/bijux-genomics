use super::super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::super::local_hpc_asset_staging_manifest::{
    collect_hpc_asset_staging_manifest, LocalHpcAssetStagingManifest,
    DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH,
};
use super::super::local_hpc_candidate_run_manifest::{
    collect_hpc_candidate_run_manifest_with_jobs, LocalHpcCandidateRunManifest,
    DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH,
};
use super::super::local_hpc_dependency_simulation::{
    collect_hpc_dependency_simulation_with_jobs, LocalHpcDependencySimulationReport,
    DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH,
};
use super::super::local_hpc_execution_resolver::{
    collect_hpc_execution_resolver_report_with_jobs, LocalHpcExecutionResolverReport,
    DEFAULT_HPC_EXECUTION_RESOLVER_PATH,
};
use super::super::local_hpc_pipeline_node_array::{
    collect_hpc_pipeline_node_array_with_jobs, LocalHpcPipelineNodeArrayReport,
    DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH,
};
use super::super::local_hpc_result_collection_simulation::{
    collect_hpc_result_collection_simulation_with_jobs, LocalHpcResultCollectionSimulationReport,
    DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH,
};
use super::super::local_hpc_resume_simulation::{
    collect_hpc_resume_simulation_with_jobs, LocalHpcResumeSimulationReport,
    DEFAULT_HPC_RESUME_SIMULATION_PATH,
};
use super::super::local_hpc_scratch_layout::{
    collect_hpc_scratch_layout_with_jobs, LocalHpcScratchLayout, DEFAULT_HPC_SCRATCH_LAYOUT_PATH,
};
use super::super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::super::local_hpc_stage_benchmark_array::{
    collect_hpc_stage_benchmark_array_with_jobs, LocalHpcStageBenchmarkArrayReport,
    DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH,
};
use super::super::{
    local_hpc_input_discovery::path_relative_to_repo,
    path_resolution::ensure_path_stays_within_benchmark_readiness_hpc_root,
};
use anyhow::{ensure, Result};
use bijux_dna_bench_hpc::dry_run::{
    ports::{DryRunEvidenceProvider, DryRunOutputPolicy},
    DryRunContext, DryRunInputPaths,
};
use std::cell::OnceCell;
use std::path::Path;
pub(super) struct LocalEvidence<'a> {
    repo_root: &'a Path,
    selected_jobs: OnceCell<Vec<BenchLocalAllDomainSlurmSubmitJob>>,
}
impl<'a> LocalEvidence<'a> {
    pub(super) fn new(repo_root: &'a Path) -> Self {
        Self { repo_root, selected_jobs: OnceCell::new() }
    }

    fn jobs(&self, repo_root: &Path) -> Result<&[BenchLocalAllDomainSlurmSubmitJob]> {
        self.jobs_from(repo_root, load_local_hpc_selected_jobs)
    }

    fn jobs_from(
        &self,
        repo_root: &Path,
        acquire: impl FnOnce(&Path) -> Result<Vec<BenchLocalAllDomainSlurmSubmitJob>>,
    ) -> Result<&[BenchLocalAllDomainSlurmSubmitJob]> {
        ensure!(repo_root == self.repo_root, "HPC evidence belongs to a different request root");
        if let Some(jobs) = self.selected_jobs.get() {
            return Ok(jobs);
        }
        // Share only successful acquisition inside this assessment. A new render or
        // validation request derives its own jobs from the current governed inputs.
        let jobs = acquire(repo_root)?;
        Ok(self.selected_jobs.get_or_init(|| jobs))
    }
}
impl DryRunEvidenceProvider for LocalEvidence<'_> {
    fn asset(&self, repo_root: &Path) -> Result<LocalHpcAssetStagingManifest> {
        collect_hpc_asset_staging_manifest(repo_root)
    }
    fn scratch(&self, repo_root: &Path) -> Result<LocalHpcScratchLayout> {
        collect_hpc_scratch_layout_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn execution(&self, repo_root: &Path) -> Result<LocalHpcExecutionResolverReport> {
        collect_hpc_execution_resolver_report_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn stage_benchmark(&self, repo_root: &Path) -> Result<LocalHpcStageBenchmarkArrayReport> {
        collect_hpc_stage_benchmark_array_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn pipeline_node(&self, repo_root: &Path) -> Result<LocalHpcPipelineNodeArrayReport> {
        collect_hpc_pipeline_node_array_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn dependency(&self, repo_root: &Path) -> Result<LocalHpcDependencySimulationReport> {
        collect_hpc_dependency_simulation_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn resume(&self, repo_root: &Path) -> Result<LocalHpcResumeSimulationReport> {
        collect_hpc_resume_simulation_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn result_collection(
        &self,
        repo_root: &Path,
    ) -> Result<LocalHpcResultCollectionSimulationReport> {
        collect_hpc_result_collection_simulation_with_jobs(repo_root, || self.jobs(repo_root))
    }
    fn candidate(&self, repo_root: &Path) -> Result<LocalHpcCandidateRunManifest> {
        collect_hpc_candidate_run_manifest_with_jobs(repo_root, &|| self.jobs(repo_root))
    }
}
struct OutputPolicy;
impl DryRunOutputPolicy for OutputPolicy {
    fn admit(&self, repo_root: &Path, output: &Path) -> Result<()> {
        ensure_path_stays_within_benchmark_readiness_hpc_root(
            repo_root,
            output,
            "HPC dry-run readiness output",
        )
    }
    fn display(&self, repo_root: &Path, output: &Path) -> String {
        path_relative_to_repo(repo_root, output)
    }
}
pub(super) fn context<'a>(
    repo_root: &'a Path,
    evidence: &'a LocalEvidence<'_>,
) -> DryRunContext<'a> {
    DryRunContext {
        repo_root,
        evidence,
        output: &OutputPolicy,
        paths: DryRunInputPaths {
            asset: DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH,
            scratch: DEFAULT_HPC_SCRATCH_LAYOUT_PATH,
            execution: DEFAULT_HPC_EXECUTION_RESOLVER_PATH,
            stage_benchmark: DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH,
            pipeline_node: DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH,
            dependency: DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH,
            resume: DEFAULT_HPC_RESUME_SIMULATION_PATH,
            result_collection: DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH,
            candidate: DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::LocalEvidence;
    use std::{cell::Cell, path::Path};

    #[test]
    fn selected_jobs_are_acquired_once_per_request_and_fresh_for_the_next() -> anyhow::Result<()> {
        let root = Path::new("workspace");
        let acquisitions = Cell::new(0);
        let acquire = |_: &Path| {
            acquisitions.set(acquisitions.get() + 1);
            Ok(Vec::new())
        };
        let request = LocalEvidence::new(root);
        for _ in 0..9 {
            assert!(request.jobs_from(root, acquire)?.is_empty());
        }
        assert_eq!(acquisitions.get(), 1);
        assert!(LocalEvidence::new(root).jobs_from(root, acquire)?.is_empty());
        assert_eq!(acquisitions.get(), 2);
        Ok(())
    }

    #[test]
    fn failed_acquisition_remains_retryable_and_cannot_cross_request_roots() -> anyhow::Result<()> {
        let root = Path::new("workspace");
        let request = LocalEvidence::new(root);
        assert!(request.jobs_from(root, |_| anyhow::bail!("source unavailable")).is_err());
        let acquisitions = Cell::new(0);
        let acquire = |_: &Path| {
            acquisitions.set(acquisitions.get() + 1);
            Ok(Vec::new())
        };
        assert!(request.jobs_from(Path::new("another-workspace"), acquire).is_err());
        assert_eq!(acquisitions.get(), 0);
        assert!(request.jobs_from(root, acquire)?.is_empty());
        assert_eq!(acquisitions.get(), 1);
        Ok(())
    }
}
