//! Owner: bijux-dna-containers
//! Qualification registry pure catalog, run, history and publication closure.

use super::allocation::{acceptance_adapter_for, resource_profile_for};
use super::identity::{
    parse_run_id, validate_sha256, EXPECTED_EXTERNAL_ARTIFACT_TOOLS,
    EXPECTED_REDISTRIBUTABLE_TOOLS, RUN_NAMESPACE_MAXIMUM, RUN_NAMESPACE_MINIMUM,
    RUN_NAMESPACE_NEXT, RUN_NAMESPACE_SIZE,
};
use super::protocol::summarize_run_definitions;
use super::{
    ContainerRunIndex, QualificationRunDefinition, QualificationRunIdTombstone,
    QualificationRunRegistry, QualificationRunState, CONTAINER_RUN_INDEX_SCHEMA_V1,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V3,
};
use crate::evidence::tool_contract::{
    ContainerArtifactDelivery, ContainerCatalog, ContainerProductSupport,
};
use anyhow::{bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate_qualification_run_reservations(
    registry: &QualificationRunRegistry,
) -> Result<()> {
    if registry.schema_version != QUALIFICATION_RUN_REGISTRY_SCHEMA_V3 {
        bail!("unsupported qualification-run registry schema");
    }
    validate_sha256(&registry.catalog_sha256, "qualification registry catalog")?;
    let mut tool_ids = BTreeSet::new();
    let mut run_ids = BTreeSet::new();
    for row in &registry.tools {
        if row.tool_id.is_empty() || !tool_ids.insert(row.tool_id.as_str()) {
            bail!("qualification registry contains an empty or duplicate tool ID");
        }
        if row.product_support != ContainerProductSupport::Supported {
            bail!("qualification registry contains a non-supported tool {}", row.tool_id);
        }
        let run_id = row.run_id.as_deref().context("supported tool has no run ID")?;
        parse_run_id(run_id)?;
        if !run_ids.insert(run_id) {
            bail!("qualification registry contains duplicate run ID {run_id}");
        }
        if row.resource_profile.is_none() || row.acceptance_adapter.is_none() {
            bail!("supported tool {} has incomplete execution fields", row.tool_id);
        }
    }
    Ok(())
}

/// Validate catalog closure, product support, delivery, run identity, stage, and adapters.
///
/// # Errors
/// Returns an error when the registry is incomplete, ambiguous, stale, or malformed.
pub fn validate_qualification_run_registry(
    catalog: &ContainerCatalog,
    registry: &QualificationRunRegistry,
) -> Result<()> {
    if registry.schema_version != QUALIFICATION_RUN_REGISTRY_SCHEMA_V3 {
        bail!("unsupported qualification-run registry schema");
    }
    validate_sha256(&registry.catalog_sha256, "qualification registry catalog")?;
    if catalog.tools.len() != EXPECTED_REDISTRIBUTABLE_TOOLS + EXPECTED_EXTERNAL_ARTIFACT_TOOLS
        || registry.tools.len() != catalog.tools.len()
    {
        bail!("qualification registry does not close over the 89-tool catalog");
    }
    let mut tool_ids = BTreeSet::new();
    let mut run_ids = BTreeSet::new();
    let mut supported = 0;
    let mut redistributable = 0;
    let mut external_artifact = 0;
    for (tool, row) in catalog.tools.iter().zip(&registry.tools) {
        if row.tool_id != tool.id
            || row.product_support != tool.product_support
            || row.artifact_delivery != tool.artifact_delivery
            || row.redistribution_permitted != tool.redistribution_permitted
            || row.stage_id != tool.real_test_stage
            || row.input_role != tool.real_test_input_kind
        {
            bail!("qualification registry row differs from catalog tool {}", tool.id);
        }
        if !tool_ids.insert(row.tool_id.as_str()) {
            bail!("qualification registry contains duplicate tool {}", row.tool_id);
        }
        if !tool.is_product_supported() {
            bail!("qualification registry tool {} is not product-supported", row.tool_id);
        }
        supported += 1;
        match tool.artifact_delivery {
            ContainerArtifactDelivery::RedistributableImage => redistributable += 1,
            ContainerArtifactDelivery::ExternalArtifact => {
                external_artifact += 1;
                if !tool.external_artifact_supported {
                    bail!("external-artifact tool {} lacks governed admission", row.tool_id);
                }
            }
            ContainerArtifactDelivery::PrivateSiteBuild | ContainerArtifactDelivery::SiteModule => {
            }
        }
        let run_id = row.run_id.as_deref().context("supported tool has no permanent run ID")?;
        parse_run_id(run_id)?;
        if !run_ids.insert(run_id) {
            bail!("qualification registry contains duplicate run ID {run_id}");
        }
        if row.resource_profile.as_deref() != Some(resource_profile_for(&row.input_role)?)
            || row.acceptance_adapter.as_deref() != Some(acceptance_adapter_for(&row.stage_id))
        {
            bail!("qualification registry adapters differ for tool {}", row.tool_id);
        }
    }
    if supported != catalog.tools.len()
        || redistributable != EXPECTED_REDISTRIBUTABLE_TOOLS
        || external_artifact != EXPECTED_EXTERNAL_ARTIFACT_TOOLS
    {
        bail!("qualification registry must contain 89 supported tools across governed delivery classes");
    }
    validate_run_definitions(catalog, registry)?;
    Ok(())
}

fn validate_run_definitions(
    catalog: &ContainerCatalog,
    registry: &QualificationRunRegistry,
) -> Result<()> {
    let namespace = &registry.namespace;
    if namespace.minimum != RUN_NAMESPACE_MINIMUM
        || namespace.maximum != RUN_NAMESPACE_MAXIMUM
        || namespace.next_run_number != RUN_NAMESPACE_NEXT
        || namespace.allocation_policy != "monotonic_only"
        || namespace.identity_policy != "existing_ids_immutable_attempts_reuse_run_id"
    {
        bail!("qualification run namespace policy is invalid");
    }
    if registry.runs.len() != RUN_NAMESPACE_SIZE {
        bail!("qualification run definitions do not close the allocated namespace");
    }
    let catalog_tools = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<BTreeSet<_>>();
    let redistributable_tools = catalog
        .tools
        .iter()
        .filter(|tool| tool.is_redistributable_image())
        .map(|tool| tool.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut citation_keys = BTreeSet::new();
    let mut run_tools = BTreeMap::new();
    for (offset, run) in registry.runs.iter().enumerate() {
        let expected_number = RUN_NAMESPACE_MINIMUM
            + u64::try_from(offset).context("qualification run offset exceeds u64")?;
        validate_run_definition_row(
            expected_number,
            run,
            &catalog_tools,
            &redistributable_tools,
            &mut citation_keys,
            &mut run_tools,
        )?;
    }
    for run in &registry.runs {
        for superseded in &run.supersedes_run_ids {
            if parse_run_id(superseded)? >= parse_run_id(&run.run_id)?
                || run_tools.get(superseded) != run_tools.get(&run.run_id)
            {
                bail!("qualification run supersession lineage is invalid: {}", run.run_id);
            }
        }
        if let Some(successor) = run.superseded_by_run_id.as_deref() {
            if parse_run_id(successor)? <= parse_run_id(&run.run_id)?
                || run_tools.get(successor) != run_tools.get(&run.run_id)
            {
                bail!("qualification run successor lineage is invalid: {}", run.run_id);
            }
        }
    }
    let summary = summarize_run_definitions(&registry.runs);
    if registry.summary != summary
        || summary.run_count != RUN_NAMESPACE_SIZE
        || summary.campaign_count != 1
        || summary.allocated_count != 21
        || summary.reserved_count != 2
    {
        bail!("qualification run registry summary is invalid");
    }
    Ok(())
}

fn validate_run_definition_row(
    expected_number: u64,
    run: &QualificationRunDefinition,
    catalog_tools: &BTreeSet<&str>,
    redistributable_tools: &BTreeSet<&str>,
    citation_keys: &mut BTreeSet<String>,
    run_tools: &mut BTreeMap<String, String>,
) -> Result<()> {
    if run.run_id != format!("run-{expected_number}")
        || run.title.trim().is_empty()
        || run.purpose.trim().is_empty()
        || run.owner.trim().is_empty()
        || run.operation.trim().is_empty()
        || run.stage_id.trim().is_empty()
        || run.contract_ids.is_empty()
        || run.acceptance_policy.trim().is_empty()
        || run.citation.protocol_version != "1"
        || run.citation.citation_key != format!("bijux-container-run-{expected_number}")
        || run.citation.registry_anchor
            != format!("repo://containers/contracts/qualification-runs.toml#run-{expected_number}")
        || !citation_keys.insert(run.citation.citation_key.clone())
    {
        bail!("qualification run definition is incomplete: {}", run.run_id);
    }
    if run.tool_ids.iter().any(|tool| !catalog_tools.contains(tool.as_str())) {
        bail!("qualification run references a tool outside the catalog: {}", run.run_id);
    }
    match run.state {
        QualificationRunState::Campaign => {
            if expected_number != 1000
                || run.owner != "bijux-dna-containers"
                || run.tool_ids.iter().map(String::as_str).collect::<BTreeSet<_>>()
                    != *redistributable_tools
                || run.state_reason.is_some()
            {
                bail!("container image campaign run definition is malformed");
            }
        }
        QualificationRunState::Reserved => {
            if !matches!(expected_number, 1081 | 1085)
                || !run.tool_ids.is_empty()
                || !run.corpus_roles.is_empty()
                || run.state_reason.as_deref().is_none_or(str::is_empty)
            {
                bail!("qualification run tombstone is malformed: {}", run.run_id);
            }
        }
        QualificationRunState::Active
        | QualificationRunState::Allocated
        | QualificationRunState::Historical => {
            if run.owner != "bijux-dna-containers-qa"
                || run.tool_ids.len() != 1
                || run.corpus_roles.len() != 1
                || (run.state == QualificationRunState::Active && run.state_reason.is_some())
                || (run.state != QualificationRunState::Active
                    && run.state_reason.as_deref().is_none_or(str::is_empty))
            {
                bail!("tool qualification run definition is malformed: {}", run.run_id);
            }
            run_tools.insert(run.run_id.clone(), run.tool_ids[0].clone());
        }
    }
    Ok(())
}

pub(super) fn validate_historical_ownership(
    registry: &QualificationRunRegistry,
    historical: &BTreeMap<u64, String>,
    tombstones: &[QualificationRunIdTombstone],
) -> Result<()> {
    let reserved = tombstones
        .iter()
        .map(|tombstone| parse_run_id(&tombstone.run_id))
        .collect::<Result<BTreeSet<_>>>()?;
    for row in &registry.tools {
        let Some(run_id) = &row.run_id else {
            continue;
        };
        let number = parse_run_id(run_id)?;
        if reserved.contains(&number) {
            bail!("{run_id} is permanently reserved by a qualification run-ID tombstone");
        }
        if let Some(owner) = historical.get(&number) {
            if owner != &row.tool_id {
                bail!("{run_id} historically belongs to {owner}, not {}", row.tool_id);
            }
        } else if number <= registry.historical_max_run_number {
            bail!("new qualification run ID does not exceed the historical maximum");
        }
    }
    Ok(())
}

/// Validate namespace, summary, run ordering, and citation closure for a container run index.
///
/// # Errors
/// Returns an error when the index is incomplete or contradicts permanent run identity.
pub fn validate_container_run_index(index: &ContainerRunIndex) -> Result<()> {
    if index.schema_version != CONTAINER_RUN_INDEX_SCHEMA_V1
        || index.source_registry.uri != "repo://containers/contracts/qualification-runs.toml"
        || index.source_registry.size_bytes == 0
    {
        bail!("container run index source identity is invalid");
    }
    validate_sha256(&index.source_registry.sha256, "container run index source")?;
    validate_sha256(&index.catalog_sha256, "container run index catalog")?;
    if index.namespace.minimum != RUN_NAMESPACE_MINIMUM
        || index.namespace.maximum != RUN_NAMESPACE_MAXIMUM
        || index.namespace.next_run_number != RUN_NAMESPACE_NEXT
        || index.runs.len() != RUN_NAMESPACE_SIZE
        || index.summary != summarize_run_definitions(&index.runs)
    {
        bail!("container run index namespace or summary is invalid");
    }
    let mut citations = BTreeSet::new();
    for (offset, run) in index.runs.iter().enumerate() {
        let number = RUN_NAMESPACE_MINIMUM
            + u64::try_from(offset).context("container run index offset exceeds u64")?;
        if run.run_id != format!("run-{number}")
            || run.citation.citation_key != format!("bijux-container-run-{number}")
            || run.citation.registry_anchor
                != format!("repo://containers/contracts/qualification-runs.toml#run-{number}")
            || !citations.insert(run.citation.citation_key.as_str())
            || run.title.trim().is_empty()
            || run.owner.trim().is_empty()
            || run.operation.trim().is_empty()
            || run.stage_id.trim().is_empty()
            || run.acceptance_policy.trim().is_empty()
        {
            bail!("container run index entry is invalid: {}", run.run_id);
        }
        if run.citation.evidence_uris.is_empty() && run.citation.limitations.is_empty() {
            bail!("container run index entry hides absent evidence: {}", run.run_id);
        }
    }
    Ok(())
}
