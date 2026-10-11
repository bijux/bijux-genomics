use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalAllDomainSlurmSubmitJob, BenchmarkLocalHpcAssetStagingManifest,
    BenchmarkLocalHpcExecutionResolverReport, HpcJobResourceHints,
};
use std::path::{Path, PathBuf};
/// Result locations observed by the existing job-completion adapter. These are
/// request inputs, not a second persisted artifact schema.
#[derive(Debug, Clone)]
pub struct CandidateResultPaths {
    pub result_root: PathBuf,
    pub stage_result_manifest_path: PathBuf,
}
/// Local evidence acquisition for candidate selection. There is no submission operation.
pub trait CandidateEvidenceProvider {
    /// Acquire canonical assets evidence.
    ///
    /// # Errors
    /// Returns source acquisition or admission failure.
    fn assets(&self, repo_root: &Path) -> Result<BenchmarkLocalHpcAssetStagingManifest>;
    /// Acquire canonical execution evidence.
    ///
    /// # Errors
    /// Returns source acquisition or admission failure.
    fn execution(&self, repo_root: &Path) -> Result<BenchmarkLocalHpcExecutionResolverReport>;
    /// Acquire canonical jobs evidence.
    ///
    /// # Errors
    /// Returns source acquisition or admission failure.
    fn jobs(&self, repo_root: &Path) -> Result<Vec<BenchmarkLocalAllDomainSlurmSubmitJob>>;
    /// Acquire canonical resources evidence.
    ///
    /// # Errors
    /// Returns source acquisition or admission failure.
    fn resources(&self, repo_root: &Path) -> Result<HpcJobResourceHints>;
    /// Resolve governed result locations once for an otherwise eligible job.
    ///
    /// # Errors
    /// Rejects inconsistent or unresolvable result paths.
    fn result_paths(
        &self,
        repo_root: &Path,
        job: &BenchmarkLocalAllDomainSlurmSubmitJob,
    ) -> Result<CandidateResultPaths>;
    /// Admit output confinement before any local evidence acquisition.
    ///
    /// # Errors
    /// Rejects an output outside the governed readiness destination.
    fn admit_output(&self, repo_root: &Path, output: &Path) -> Result<()>;
}
