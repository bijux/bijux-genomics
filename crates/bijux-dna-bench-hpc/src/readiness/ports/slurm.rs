use super::ReadinessDomain;
use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalSlurmDependencyCheckReport, BenchmarkLocalSlurmDryRunReport,
    BenchmarkLocalSlurmScriptBodyReport, BenchmarkLocalSlurmShellSyntaxReport,
    BenchmarkLocalSlurmSubmitManifest,
};
use std::path::{Path, PathBuf};
/// Supply typed local slurm evidence; provider failures remain explicit readiness diagnostics.
pub trait SlurmArtifactEvidence {
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn render_local_slurm_scripts(
        &self,
        repo_root: &Path,
        domain: ReadinessDomain,
        output_root: PathBuf,
    ) -> Result<BenchmarkLocalSlurmDryRunReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_slurm_script_bodies(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmScriptBodyReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_slurm_shell_syntax(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmShellSyntaxReport>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn render_slurm_submit_manifest(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        output_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmSubmitManifest>;
    /// Obtain governed evidence for this local readiness operation.
    ///
    /// # Errors
    /// Returns the underlying admission, observation or publication failure.
    fn validate_slurm_dependencies(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        manifest_path: PathBuf,
        report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmDependencyCheckReport>;
}
