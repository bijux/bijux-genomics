use super::LocalReadinessAdapter;
use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    BenchmarkLocalSlurmDependencyCheckReport, BenchmarkLocalSlurmDryRunReport,
    BenchmarkLocalSlurmScriptBodyReport, BenchmarkLocalSlurmShellSyntaxReport,
    BenchmarkLocalSlurmSubmitManifest,
};
use bijux_dna_bench_hpc::readiness::ports::{ReadinessDomain, SlurmArtifactEvidence};
use std::path::{Path, PathBuf};
impl SlurmArtifactEvidence for LocalReadinessAdapter {
    fn render_local_slurm_scripts(
        &self,
        repo_root: &Path,
        domain: ReadinessDomain,
        output_root: PathBuf,
    ) -> Result<BenchmarkLocalSlurmDryRunReport> {
        crate::commands::benchmark::local_slurm_dry_run::render_local_slurm_scripts(
            repo_root,
            super::inventory_domain(domain),
            output_root,
        )
    }
    fn validate_slurm_script_bodies(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmScriptBodyReport> {
        crate::commands::benchmark::local_slurm_script_bodies::validate_slurm_script_bodies(
            repo_root,
            root_path,
            report_path,
        )
    }
    fn validate_slurm_shell_syntax(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmShellSyntaxReport> {
        crate::commands::benchmark::local_slurm_shell_syntax::validate_slurm_shell_syntax(
            repo_root,
            root_path,
            report_path,
        )
    }
    fn render_slurm_submit_manifest(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        output_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmSubmitManifest> {
        crate::commands::benchmark::local_slurm_submit_manifest::render_slurm_submit_manifest(
            repo_root,
            root_path,
            output_path,
        )
    }
    fn validate_slurm_dependencies(
        &self,
        repo_root: &Path,
        root_path: PathBuf,
        manifest_path: PathBuf,
        report_path: PathBuf,
    ) -> Result<BenchmarkLocalSlurmDependencyCheckReport> {
        crate::commands::benchmark::local_slurm_dependency_check::validate_slurm_dependencies(
            repo_root,
            root_path,
            manifest_path,
            report_path,
        )
    }
}
