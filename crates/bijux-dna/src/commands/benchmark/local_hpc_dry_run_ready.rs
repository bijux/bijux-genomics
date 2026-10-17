mod adapters;
use super::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::BenchmarkLocalHpcDryRunReadinessReport;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_HPC_DRY_RUN_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/hpc/HPC_DRY_RUN_LOCAL_READY.json";
pub(crate) type LocalHpcDryRunReadyReport = BenchmarkLocalHpcDryRunReadinessReport;
pub(crate) fn run_render_hpc_dry_run_ready(
    args: &parse::BenchLocalRenderHpcDryRunReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("HPC_DRY_RUN_LOCAL_READY.json")
    });
    let report = render_hpc_dry_run_ready(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn run_validate_hpc_dry_run_ready(
    args: &parse::BenchLocalValidateHpcDryRunReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("HPC_DRY_RUN_LOCAL_READY.json")
    });
    let report = validate_hpc_dry_run_ready_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_hpc_dry_run_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcDryRunReadyReport> {
    let evidence = adapters::LocalEvidence::new(repo_root);
    bijux_dna_bench_hpc::dry_run::render_hpc_dry_run_ready(
        &adapters::context(repo_root, &evidence),
        output_path,
    )
}
pub(crate) fn validate_hpc_dry_run_ready_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcDryRunReadyReport> {
    let evidence = adapters::LocalEvidence::new(repo_root);
    bijux_dna_bench_hpc::dry_run::validate_hpc_dry_run_ready_path(
        &adapters::context(repo_root, &evidence),
        manifest_path,
    )
}
#[cfg(test)]
mod tests {
    use anyhow::Context;
    #[test]
    fn hpc_readiness_rejects_malformed_input_before_acquiring_evidence() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let directory = root.path().join("artifacts/benchmark-smoke/readiness/hpc");
        std::fs::create_dir_all(&directory)?;
        let path = directory.join("report.json");
        std::fs::write(&path, b"null")?;
        let error = super::validate_hpc_dry_run_ready_path(root.path(), &path)
            .err()
            .context("malformed report must reject")?;
        assert!(format!("{error:#}").contains("parse HPC dry-run readiness report"));
        assert_eq!(std::fs::read_dir(directory)?.count(), 1);
        assert_eq!(std::fs::read(path)?, b"null");
        Ok(())
    }
}
