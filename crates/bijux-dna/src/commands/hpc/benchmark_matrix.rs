use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
pub use bijux_dna_bench::{
    BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
    BenchmarkSurfaceMatch, BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};

use crate::commands::cli::BenchmarkMatrixArgs;
use crate::commands::hpc::campaign_dry_run;

fn workspace_root() -> Result<PathBuf> {
    let mut cursor = std::env::current_dir().context("resolve current directory")?;
    loop {
        if cursor.join("domain").is_dir()
            && cursor.join("containers/contracts/catalog.toml").is_file()
        {
            return Ok(cursor);
        }
        let Some(parent) = cursor.parent() else {
            break;
        };
        cursor = parent.to_path_buf();
    }
    Err(anyhow!(
        "unable to locate workspace root containing domain/ and containers/contracts/catalog.toml"
    ))
}

pub fn benchmark_matrix(args: &BenchmarkMatrixArgs) -> Result<BenchmarkMatrixReport> {
    let dry_run =
        campaign_dry_run(&args.config, args.env_file.as_deref(), args.user_policies.as_deref())?;
    bijux_dna_bench::derive_benchmark_matrix(&bijux_dna_bench::BenchmarkMatrixRequest {
        repository_root: workspace_root()?,
        campaign_id: dry_run.campaign_id,
        domain: args.domain.clone(),
        corpora_root: PathBuf::from(dry_run.layout.corpora_root),
        databases_root: PathBuf::from(dry_run.layout.databases_root),
        images_root: PathBuf::from(dry_run.layout.images_root),
        generated_at: bijux_dna_bench::current_benchmark_matrix_timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::workspace_root;

    #[test]
    fn workspace_discovery_uses_product_catalog_authority() {
        let root = workspace_root().expect("workspace root");
        assert!(root.join("domain").is_dir());
        assert!(root.join("containers/contracts/catalog.toml").is_file());
    }
}
