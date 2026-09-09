//! Aggregate and publish admitted local stage inventories.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::repo::{load_benchmark_local_stage_inventory, BenchmarkLocalStageInventoryDomain};
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_all_domain_stage_inventory, BenchmarkLocalAllDomainStageInventory,
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1,
};

/// Default report location for aggregate local stage inventory.
pub const DEFAULT_BENCHMARK_ALL_DOMAIN_STAGE_LIST_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domain-stage-list.json";

/// Publish a report for the requested domain order after every inventory is admitted.
///
/// # Errors
/// Returns an error for invalid/duplicate domain selections, inadmissible matrices, or output IO.
pub fn render_benchmark_all_domain_stage_inventory(
    repo_root: &Path,
    domains: &[BenchmarkLocalStageInventoryDomain],
    output_path: PathBuf,
) -> Result<BenchmarkLocalAllDomainStageInventory> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let inventories = domains
        .iter()
        .copied()
        .map(|domain| load_benchmark_local_stage_inventory(repo_root, domain))
        .collect::<Result<Vec<_>>>()?;
    let selected_domains =
        inventories.iter().map(|inventory| inventory.domain.clone()).collect::<Vec<_>>();
    let domain_counts = inventories
        .iter()
        .map(|inventory| (inventory.domain.clone(), inventory.stage_count))
        .collect::<BTreeMap<_, _>>();
    let total_stage_count = inventories.iter().map(|inventory| inventory.stage_count).sum();

    let report = BenchmarkLocalAllDomainStageInventory {
        schema_version: BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        selected_domains,
        domain_counts,
        total_stage_count,
        inventories,
    };
    validate_benchmark_local_all_domain_stage_inventory(&report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
