use super::super::local_hpc_asset_staging_manifest::{
    collect_hpc_asset_staging_manifest, LocalHpcAssetStagingManifest,
    DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH,
};
use super::super::local_hpc_candidate_run_manifest::{
    collect_hpc_candidate_run_manifest, LocalHpcCandidateRunManifest,
    DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH,
};
use super::super::local_hpc_dependency_simulation::{
    collect_hpc_dependency_simulation, LocalHpcDependencySimulationReport,
    DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH,
};
use super::super::local_hpc_execution_resolver::{
    collect_hpc_execution_resolver_report, LocalHpcExecutionResolverReport,
    DEFAULT_HPC_EXECUTION_RESOLVER_PATH,
};
use super::super::local_hpc_pipeline_node_array::{
    collect_hpc_pipeline_node_array, LocalHpcPipelineNodeArrayReport,
    DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH,
};
use super::super::local_hpc_result_collection_simulation::{
    collect_hpc_result_collection_simulation, LocalHpcResultCollectionSimulationReport,
    DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH,
};
use super::super::local_hpc_resume_simulation::{
    collect_hpc_resume_simulation, LocalHpcResumeSimulationReport,
    DEFAULT_HPC_RESUME_SIMULATION_PATH,
};
use super::super::local_hpc_scratch_layout::{
    collect_hpc_scratch_layout, LocalHpcScratchLayout, DEFAULT_HPC_SCRATCH_LAYOUT_PATH,
};
use super::super::local_hpc_stage_benchmark_array::{
    collect_hpc_stage_benchmark_array, LocalHpcStageBenchmarkArrayReport,
    DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH,
};
use super::super::{
    local_hpc_input_discovery::path_relative_to_repo,
    path_resolution::ensure_path_stays_within_benchmark_readiness_hpc_root,
};
use anyhow::Result;
use bijux_dna_bench_hpc::dry_run::{
    ports::{DryRunEvidenceProvider, DryRunOutputPolicy},
    DryRunContext, DryRunInputPaths,
};
use std::path::Path;
struct LocalEvidence;
impl DryRunEvidenceProvider for LocalEvidence {
    fn asset(&self, repo_root: &Path) -> Result<LocalHpcAssetStagingManifest> {
        collect_hpc_asset_staging_manifest(repo_root)
    }
    fn scratch(&self, repo_root: &Path) -> Result<LocalHpcScratchLayout> {
        collect_hpc_scratch_layout(repo_root)
    }
    fn execution(&self, repo_root: &Path) -> Result<LocalHpcExecutionResolverReport> {
        collect_hpc_execution_resolver_report(repo_root)
    }
    fn stage_benchmark(&self, repo_root: &Path) -> Result<LocalHpcStageBenchmarkArrayReport> {
        collect_hpc_stage_benchmark_array(repo_root)
    }
    fn pipeline_node(&self, repo_root: &Path) -> Result<LocalHpcPipelineNodeArrayReport> {
        collect_hpc_pipeline_node_array(repo_root)
    }
    fn dependency(&self, repo_root: &Path) -> Result<LocalHpcDependencySimulationReport> {
        collect_hpc_dependency_simulation(repo_root)
    }
    fn resume(&self, repo_root: &Path) -> Result<LocalHpcResumeSimulationReport> {
        collect_hpc_resume_simulation(repo_root)
    }
    fn result_collection(
        &self,
        repo_root: &Path,
    ) -> Result<LocalHpcResultCollectionSimulationReport> {
        collect_hpc_result_collection_simulation(repo_root)
    }
    fn candidate(&self, repo_root: &Path) -> Result<LocalHpcCandidateRunManifest> {
        collect_hpc_candidate_run_manifest(repo_root)
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
pub(super) fn context(repo_root: &Path) -> DryRunContext<'_> {
    DryRunContext {
        repo_root,
        evidence: &LocalEvidence,
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
