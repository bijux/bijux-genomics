use super::DryRunContext;
use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalHpcAssetStagingManifest as LocalHpcAssetStagingManifest,
    BenchmarkLocalHpcCandidateRunManifest as LocalHpcCandidateRunManifest,
    BenchmarkLocalHpcDependencySimulationReport as LocalHpcDependencySimulationReport,
    BenchmarkLocalHpcExecutionResolverReport as LocalHpcExecutionResolverReport,
    BenchmarkLocalHpcPipelineNodeArrayReport as LocalHpcPipelineNodeArrayReport,
    BenchmarkLocalHpcResultCollectionSimulationReport as LocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResumeSimulationReport as LocalHpcResumeSimulationReport,
    BenchmarkLocalHpcScratchLayout as LocalHpcScratchLayout,
    BenchmarkLocalHpcStageBenchmarkArrayReport as LocalHpcStageBenchmarkArrayReport,
};

/// One acquisition result per provider; a value and an error cannot coexist.
pub(super) struct DryRunEvidence {
    pub(super) asset: Result<LocalHpcAssetStagingManifest>,
    pub(super) scratch: Result<LocalHpcScratchLayout>,
    pub(super) execution: Result<LocalHpcExecutionResolverReport>,
    pub(super) stage_benchmark: Result<LocalHpcStageBenchmarkArrayReport>,
    pub(super) pipeline_node: Result<LocalHpcPipelineNodeArrayReport>,
    pub(super) dependency: Result<LocalHpcDependencySimulationReport>,
    pub(super) resume: Result<LocalHpcResumeSimulationReport>,
    pub(super) result_collection: Result<LocalHpcResultCollectionSimulationReport>,
    pub(super) candidate: Result<LocalHpcCandidateRunManifest>,
}
impl DryRunEvidence {
    pub(super) fn acquire(context: &DryRunContext<'_>) -> Self {
        Self {
            asset: context.evidence.asset(context.repo_root),
            scratch: context.evidence.scratch(context.repo_root),
            execution: context.evidence.execution(context.repo_root),
            stage_benchmark: context.evidence.stage_benchmark(context.repo_root),
            pipeline_node: context.evidence.pipeline_node(context.repo_root),
            dependency: context.evidence.dependency(context.repo_root),
            resume: context.evidence.resume(context.repo_root),
            result_collection: context.evidence.result_collection(context.repo_root),
            candidate: context.evidence.candidate(context.repo_root),
        }
    }
}
