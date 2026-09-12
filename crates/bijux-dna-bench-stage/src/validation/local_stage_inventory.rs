//! Validation for persisted local benchmark stage inventories.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchmarkLocalAllDomainStageInventory, BenchmarkLocalStageInventory, StageContractError,
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};

/// Validate one domain-local stage inventory.
///
/// # Errors
/// Returns an error when identity, domain, or cardinality invariants are invalid.
pub fn validate_benchmark_local_stage_inventory(
    inventory: &BenchmarkLocalStageInventory,
) -> Result<(), StageContractError> {
    if inventory.schema_version != BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local stage-inventory schema `{}`",
            inventory.schema_version
        ));
    }
    let prefix = domain_prefix(&inventory.domain)?;
    require_text(&inventory.stage_matrix_schema_version, "stage_matrix_schema_version")?;
    require_text(&inventory.stage_matrix_path, "stage_matrix_path")?;
    if inventory.stage_count == 0 || inventory.stage_count != inventory.stages.len() {
        return invalid("stage_count must equal a non-empty stages array");
    }
    let mut stage_ids = BTreeSet::new();
    for stage in &inventory.stages {
        require_text(&stage.stage_id, "stages[].stage_id")?;
        if !stage.stage_id.starts_with(&prefix) {
            return invalid(format!(
                "stage `{}` is outside `{}` domain",
                stage.stage_id, inventory.domain
            ));
        }
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("stage inventory repeats stage `{}`", stage.stage_id));
        }
    }
    Ok(())
}

/// Validate the aggregate local stage inventory.
///
/// # Errors
/// Returns an error when selected-domain, count, or nested-inventory closure is invalid.
pub fn validate_benchmark_local_all_domain_stage_inventory(
    report: &BenchmarkLocalAllDomainStageInventory,
) -> Result<(), StageContractError> {
    if report.schema_version != BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1 {
        return invalid(format!(
            "unsupported all-domain local stage-inventory schema `{}`",
            report.schema_version
        ));
    }
    require_text(&report.output_path, "output_path")?;
    if report.selected_domains.is_empty()
        || report.selected_domains.len() != report.inventories.len()
        || report.selected_domains.len() != report.domain_counts.len()
    {
        return invalid("selected domains, inventories, and domain counts must have equal size");
    }
    let mut selected = BTreeSet::new();
    let mut expected_counts = BTreeMap::new();
    let mut total = 0usize;
    for (domain, inventory) in report.selected_domains.iter().zip(&report.inventories) {
        domain_prefix(domain)?;
        if !selected.insert(domain.as_str()) {
            return invalid(format!("all-domain inventory repeats domain `{domain}`"));
        }
        validate_benchmark_local_stage_inventory(inventory)?;
        if inventory.domain != *domain {
            return invalid(format!(
                "selected domain `{domain}` does not match nested `{}` inventory",
                inventory.domain
            ));
        }
        expected_counts.insert(domain.clone(), inventory.stage_count);
        let Some(next_total) = total.checked_add(inventory.stage_count) else {
            return invalid("total stage count overflowed");
        };
        total = next_total;
    }
    if report.domain_counts != expected_counts {
        return invalid("domain_counts do not match nested inventories");
    }
    if report.total_stage_count != total {
        return invalid("total_stage_count does not match nested inventories");
    }
    Ok(())
}

fn domain_prefix(domain: &str) -> Result<String, StageContractError> {
    match domain {
        "fastq" | "bam" | "vcf" => Ok(format!("{domain}.")),
        other => invalid(format!("unsupported local stage-inventory domain `{other}`")),
    }
}

fn require_text(value: &str, field: &str) -> Result<(), StageContractError> {
    if value.trim().is_empty() {
        return invalid(format!("local stage-inventory requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, StageContractError> {
    Err(StageContractError::invalid(message.into()))
}
