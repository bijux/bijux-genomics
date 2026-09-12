//! Compose domain matrix adapters before owner publication.
use crate::repo::{load_benchmark_local_stage_inventory, BenchmarkLocalStageInventoryDomain};
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalAllDomainStageInventory;
use std::path::{Path, PathBuf};
/// Publish the requested inventories in declared domain order.
///
/// # Errors
/// Propagates domain admission or publication failures.
pub fn render_benchmark_all_domain_stage_inventory(
    repo_root: &Path,
    domains: &[BenchmarkLocalStageInventoryDomain],
    output_path: PathBuf,
) -> Result<BenchmarkLocalAllDomainStageInventory> {
    let inventories = domains
        .iter()
        .copied()
        .map(|domain| load_benchmark_local_stage_inventory(repo_root, domain))
        .collect::<Result<Vec<_>>>()?;
    bijux_dna_bench_stage::repository::publish_inventories(repo_root, inventories, &output_path)
}
