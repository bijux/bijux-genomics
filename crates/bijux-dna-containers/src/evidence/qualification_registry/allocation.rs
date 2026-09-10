//! Owner: bijux-dna-containers
//! Qualification registry per-tool run reservations and execution profile policy.

use super::identity::parse_run_id;
use super::{QualificationRunIdAuditRow, QualificationRunRegistryEntry};
use crate::evidence::tool_contract::ContainerCatalog;
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;

pub(super) struct DerivedRegistryRows {
    pub(super) entries: Vec<QualificationRunRegistryEntry>,
    pub(super) audit_rows: Vec<QualificationRunIdAuditRow>,
    pub(super) allocated_mapping_count: usize,
}

pub(super) fn derive_registry_rows(
    catalog: &ContainerCatalog,
    current: &BTreeMap<String, u64>,
    reservations: &BTreeMap<&str, &str>,
    historical_max_run_number: u64,
) -> Result<DerivedRegistryRows> {
    let highest_reserved = reservations
        .values()
        .map(|run_id| parse_run_id(run_id))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .max()
        .unwrap_or(0);
    let mut next_run_number = historical_max_run_number
        .max(highest_reserved)
        .checked_add(1)
        .context("qualification run number overflow")?;
    let missing_redistributable = catalog
        .tools
        .iter()
        .filter(|tool| tool.is_redistributable_image())
        .filter(|tool| {
            !current.contains_key(&tool.id) && !reservations.contains_key(tool.id.as_str())
        })
        .count();
    let mut next_external_run_number = next_run_number
        .checked_add(
            u64::try_from(missing_redistributable)
                .context("redistributable allocation count exceeds u64")?,
        )
        .context("qualification external-artifact run number overflow")?;
    let mut derived = DerivedRegistryRows {
        entries: Vec::with_capacity(catalog.tools.len()),
        audit_rows: Vec::with_capacity(catalog.tools.len()),
        allocated_mapping_count: 0,
    };
    for tool in &catalog.tools {
        let runnable = tool.is_product_supported();
        let (run_id, identity_source, resource_profile, acceptance_adapter) = if runnable {
            let (run_id, source) = if let Some(number) = current.get(&tool.id) {
                let current_run_id = format!("run-{number}");
                if reservations
                    .get(tool.id.as_str())
                    .is_some_and(|reserved| *reserved != current_run_id)
                {
                    bail!(
                        "checked-in final run for {} differs from its governed reservation",
                        tool.id
                    );
                }
                (current_run_id, "checked_in_final_run")
            } else if let Some(reserved) = reservations.get(tool.id.as_str()) {
                derived.allocated_mapping_count += 1;
                ((*reserved).to_string(), "allocated_after_historical_max")
            } else {
                let number = if tool.external_artifact_supported {
                    let number = next_external_run_number;
                    next_external_run_number = next_external_run_number
                        .checked_add(1)
                        .context("qualification external-artifact run number overflow")?;
                    number
                } else {
                    let number = next_run_number;
                    next_run_number = next_run_number
                        .checked_add(1)
                        .context("qualification run number overflow")?;
                    number
                };
                derived.allocated_mapping_count += 1;
                (format!("run-{number}"), "allocated_after_historical_max")
            };
            (
                Some(run_id),
                source,
                Some(resource_profile_for(&tool.real_test_input_kind)?.to_string()),
                Some(acceptance_adapter_for(&tool.real_test_stage).to_string()),
            )
        } else {
            bail!("tool {} is not product-supported", tool.id);
        };
        derived.audit_rows.push(QualificationRunIdAuditRow {
            tool_id: tool.id.clone(),
            product_support: tool.product_support,
            artifact_delivery: tool.artifact_delivery,
            redistribution_permitted: tool.redistribution_permitted,
            run_id: run_id.clone(),
            identity_source: identity_source.to_string(),
            stage_id: tool.real_test_stage.clone(),
            input_role: tool.real_test_input_kind.clone(),
        });
        derived.entries.push(QualificationRunRegistryEntry {
            tool_id: tool.id.clone(),
            product_support: tool.product_support,
            artifact_delivery: tool.artifact_delivery,
            redistribution_permitted: tool.redistribution_permitted,
            run_id,
            stage_id: tool.real_test_stage.clone(),
            input_role: tool.real_test_input_kind.clone(),
            resource_profile,
            acceptance_adapter,
        });
    }
    Ok(derived)
}

pub(super) fn resource_profile_for(input_role: &str) -> Result<&'static str> {
    match input_role {
        "fastq" => Ok("fastq-standard"),
        "bam" => Ok("bam-standard"),
        "bam_with_reference" => Ok("bam-reference"),
        "vcf" => Ok("vcf-standard"),
        "hap_ibd_segments" => Ok("ibd-segments"),
        other => bail!("qualification input role has no resource profile: {other}"),
    }
}

pub(super) fn acceptance_adapter_for(stage_id: &str) -> &'static str {
    if stage_id == "container.runtime" {
        "runtime-contract"
    } else {
        "stage-contract"
    }
}
