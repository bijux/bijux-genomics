use crate::paths::{path_relative_to_repo, resolve_repo_pathbuf as repo_relative_path};
use crate::{
    validate_benchmark_local_all_domain_stage_inventory, BenchmarkLocalAllDomainStageInventory,
    BenchmarkLocalStageInventory, BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1,
};
use anyhow::{Context, Result};
use std::{collections::BTreeMap, path::Path};

/// Publish a complete admitted inventory selection.
///
/// # Errors
/// Refuses inconsistent inventories before creating output directories.
pub fn publish_inventories(
    repo_root: &Path,
    inventories: Vec<BenchmarkLocalStageInventory>,
    output_path: &Path,
) -> Result<BenchmarkLocalAllDomainStageInventory> {
    let absolute_output_path = repo_relative_path(repo_root, output_path);
    let selected_domains =
        inventories.iter().map(|inventory| inventory.domain.clone()).collect::<Vec<_>>();
    let domain_counts = inventories
        .iter()
        .map(|inventory| (inventory.domain.clone(), inventory.stage_count))
        .collect::<BTreeMap<_, _>>();
    let total_stage_count = inventories.iter().try_fold(0usize, |sum, inventory| {
        sum.checked_add(inventory.stage_count)
            .ok_or_else(|| anyhow::anyhow!("total stage count overflowed"))
    })?;

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
