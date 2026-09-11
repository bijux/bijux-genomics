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
use std::path::Path;
/// Obtain the governed local evidence once per assessment. Each operation may
/// materialize its own local artifacts; failures remain individual diagnostics.
pub trait DryRunEvidenceProvider {
    /// Acquire canonical asset evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn asset(&self, repo_root: &Path) -> Result<LocalHpcAssetStagingManifest>;
    /// Acquire canonical scratch evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn scratch(&self, repo_root: &Path) -> Result<LocalHpcScratchLayout>;
    /// Acquire canonical execution evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn execution(&self, repo_root: &Path) -> Result<LocalHpcExecutionResolverReport>;
    /// Acquire canonical stage benchmark evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn stage_benchmark(&self, repo_root: &Path) -> Result<LocalHpcStageBenchmarkArrayReport>;
    /// Acquire canonical pipeline node evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn pipeline_node(&self, repo_root: &Path) -> Result<LocalHpcPipelineNodeArrayReport>;
    /// Acquire canonical dependency evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn dependency(&self, repo_root: &Path) -> Result<LocalHpcDependencySimulationReport>;
    /// Acquire canonical resume evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn resume(&self, repo_root: &Path) -> Result<LocalHpcResumeSimulationReport>;
    /// Acquire canonical result collection evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn result_collection(
        &self,
        repo_root: &Path,
    ) -> Result<LocalHpcResultCollectionSimulationReport>;
    /// Acquire canonical candidate evidence.
    ///
    /// # Errors
    /// Returns the original evidence acquisition or validation error.
    fn candidate(&self, repo_root: &Path) -> Result<LocalHpcCandidateRunManifest>;
}
/// Composition supplies the existing application path policy without duplicating it.
pub trait DryRunOutputPolicy {
    /// Admit an absolute readiness output before any evidence acquisition or write.
    ///
    /// # Errors
    /// Rejects paths outside the governed local readiness destination.
    fn admit(&self, repo_root: &Path, output: &Path) -> Result<()>;
    /// Preserve the application-relative report path representation.
    fn display(&self, repo_root: &Path, output: &Path) -> String;
}
