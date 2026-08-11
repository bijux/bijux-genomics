use std::path::PathBuf;

use anyhow::Result;
pub use bijux_dna_api::v1::api::bench::{
    BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
    BenchmarkSurfaceMatch, BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};

use crate::commands::cli::BenchmarkMatrixArgs;
use crate::commands::hpc::campaign_dry_run;

fn workspace_root() -> Result<PathBuf> {
    crate::commands::support::workspace_root::resolve_repo_root()
}

pub fn benchmark_matrix(args: &BenchmarkMatrixArgs) -> Result<BenchmarkMatrixReport> {
    let dry_run =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    bijux_dna_api::v1::api::bench::derive_benchmark_matrix(
        &bijux_dna_api::v1::api::bench::BenchmarkMatrixRequest {
            repository_root: workspace_root()?,
            campaign_id: dry_run.campaign_id,
            domain: args.domain.clone(),
            corpora_root: PathBuf::from(dry_run.layout.corpora_root),
            databases_root: PathBuf::from(dry_run.layout.databases_root),
            images_root: PathBuf::from(dry_run.layout.images_root),
            generated_at: bijux_dna_api::v1::api::bench::current_benchmark_matrix_timestamp(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::workspace_root;
    use anyhow::Result;

    #[test]
    fn workspace_discovery_uses_product_catalog_authority() -> Result<()> {
        let root = workspace_root()?;
        assert!(root.join("domain").is_dir());
        assert!(root.join("containers/contracts/catalog.toml").is_file());
        Ok(())
    }
}
