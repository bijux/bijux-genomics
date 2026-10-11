//! Owner: bijux-dna-containers
//! Qualification registry run protocol, citation and supersession projections.

use super::allocation::{acceptance_adapter_for, resource_profile_for};
use super::identity::{parse_run_id, RUN_NAMESPACE_NEXT, RUN_NAMESPACE_SIZE};
use super::{
    QualificationRunCitationRecord, QualificationRunDefinition, QualificationRunIdTombstone,
    QualificationRunRegistryEntry, QualificationRunRegistrySummary, QualificationRunState,
};
use crate::evidence::tool_contract::{ContainerCatalog, ContainerCatalogTool};
use anyhow::{bail, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn derive_run_definitions(
    catalog: &ContainerCatalog,
    tool_entries: &[QualificationRunRegistryEntry],
    current: &BTreeMap<String, u64>,
    historical: &BTreeMap<u64, String>,
    tombstones: &[QualificationRunIdTombstone],
) -> Result<Vec<QualificationRunDefinition>> {
    let catalog_tools =
        catalog.tools.iter().map(|tool| (tool.id.as_str(), tool)).collect::<BTreeMap<_, _>>();
    let mut tool_by_run = historical.clone();
    let mut allocated_runs = BTreeSet::new();
    for entry in tool_entries {
        let Some(run_id) = entry.run_id.as_deref() else {
            continue;
        };
        let number = parse_run_id(run_id)?;
        if let Some(existing) = tool_by_run.insert(number, entry.tool_id.clone()) {
            if existing != entry.tool_id {
                bail!("run {run_id} has conflicting historical and current tool ownership");
            }
        }
        if current.get(&entry.tool_id) != Some(&number) {
            allocated_runs.insert(number);
        }
    }
    let reserved = tombstones
        .iter()
        .map(|tombstone| Ok((parse_run_id(&tombstone.run_id)?, tombstone)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let mut per_tool_runs = BTreeMap::<&str, Vec<u64>>::new();
    for (number, tool_id) in &tool_by_run {
        per_tool_runs.entry(tool_id).or_default().push(*number);
    }
    let redistributable_tools = catalog
        .tools
        .iter()
        .filter(|tool| tool.is_redistributable_image())
        .map(|tool| tool.id.clone())
        .collect::<Vec<_>>();
    let mut runs = Vec::with_capacity(RUN_NAMESPACE_SIZE);
    runs.push(QualificationRunDefinition {
        run_id: "run-1000".to_string(),
        title: "Redistributable container image campaign".to_string(),
        purpose: "Build and admit the exact repository-distributable image roster from the governed catalog and recipes.".to_string(),
        owner: "bijux-dna-containers".to_string(),
        operation: "image_build_campaign".to_string(),
        stage_id: "container.image_campaign".to_string(),
        tool_ids: redistributable_tools,
        contract_ids: vec![
            "container-catalog".to_string(),
            "container-tool-contract".to_string(),
            "container-image-identity".to_string(),
        ],
        corpus_roles: vec!["recipe_source".to_string(), "tool_source".to_string()],
        acceptance_policy: "all redistributable-image tools publish digest-bound build and image identity evidence".to_string(),
        state: QualificationRunState::Campaign,
        state_reason: None,
        supersedes_run_ids: Vec::new(),
        superseded_by_run_id: None,
        citation: citation_record(
            1000,
            vec!["repo://containers/runs/run-1000-images/config-campaign.json".to_string()],
            Vec::new(),
        ),
    });

    for number in 1001..RUN_NAMESPACE_NEXT {
        if let Some(tombstone) = reserved.get(&number) {
            runs.push(reserved_run_definition(number, tombstone));
            continue;
        }
        let tool_id = tool_by_run
            .get(&number)
            .with_context(|| format!("run-{number} has no tool ownership or reservation"))?;
        let tool = catalog_tools
            .get(tool_id.as_str())
            .with_context(|| format!("run-{number} references unknown catalog tool {tool_id}"))?;
        let chain = per_tool_runs.get(tool_id.as_str()).context("tool run chain is absent")?;
        let position = chain
            .iter()
            .position(|candidate| *candidate == number)
            .context("tool run chain does not contain run")?;
        let is_current = tool.is_redistributable_image() && current.get(tool_id) == Some(&number);
        let is_allocated = allocated_runs.contains(&number)
            || (tool.external_artifact_supported && current.get(tool_id) == Some(&number));
        let state = if is_current {
            QualificationRunState::Active
        } else if is_allocated {
            QualificationRunState::Allocated
        } else {
            QualificationRunState::Historical
        };
        let supersedes_run_ids = position
            .checked_sub(1)
            .map(|previous| vec![format!("run-{}", chain[previous])])
            .unwrap_or_default();
        let superseded_by_run_id = chain.get(position + 1).map(|next| format!("run-{next}"));
        runs.push(tool_run_definition(
            number,
            tool,
            state,
            supersedes_run_ids,
            superseded_by_run_id,
            historical.contains_key(&number),
        )?);
    }
    Ok(runs)
}

fn tool_run_definition(
    number: u64,
    tool: &ContainerCatalogTool,
    state: QualificationRunState,
    supersedes_run_ids: Vec<String>,
    superseded_by_run_id: Option<String>,
    has_historical_evidence: bool,
) -> Result<QualificationRunDefinition> {
    let run_id = format!("run-{number}");
    let mut evidence_uris = Vec::new();
    if has_historical_evidence {
        evidence_uris.push(format!("repo://containers/runs/attempt-ledger.json#{run_id}"));
    }
    if state == QualificationRunState::Active {
        evidence_uris
            .push(format!("repo://containers/runs/{run_id}-{}/evidence.lock.json", tool.id));
    }
    let (state_reason, limitations) = match state {
        QualificationRunState::Active => (None, Vec::new()),
        QualificationRunState::Allocated => (
            Some(
                "Permanent run identity allocated; no admitted terminal attempt is selected."
                    .to_string(),
            ),
            vec!["The protocol has no selected publication until qualification completes."
                .to_string()],
        ),
        QualificationRunState::Historical => (
            Some(
                "Historical run identity retained without promotion to current evidence."
                    .to_string(),
            ),
            vec!["Historical evidence is not a current qualification publication.".to_string()],
        ),
        QualificationRunState::Campaign | QualificationRunState::Reserved => unreachable!(),
    };
    Ok(QualificationRunDefinition {
        run_id: run_id.clone(),
        title: format!("{} container qualification protocol", tool.display_name),
        purpose: format!(
            "Qualify {} {} for the {} operation using declared real-data outputs.",
            tool.display_name, tool.version, tool.real_test_stage
        ),
        owner: "bijux-dna-containers-qa".to_string(),
        operation: "container_qualification".to_string(),
        stage_id: tool.real_test_stage.clone(),
        tool_ids: vec![tool.id.clone()],
        contract_ids: vec![
            "container-catalog".to_string(),
            "container-tool-contract".to_string(),
            format!("stage-binding:{}", tool.real_test_stage),
        ],
        corpus_roles: vec![tool.real_test_input_kind.clone()],
        acceptance_policy: format!(
            "{} with {} resource accounting",
            acceptance_adapter_for(&tool.real_test_stage),
            resource_profile_for(&tool.real_test_input_kind)?
        ),
        state,
        state_reason,
        supersedes_run_ids,
        superseded_by_run_id,
        citation: citation_record(number, evidence_uris, limitations),
    })
}

fn reserved_run_definition(
    number: u64,
    tombstone: &QualificationRunIdTombstone,
) -> QualificationRunDefinition {
    QualificationRunDefinition {
        run_id: format!("run-{number}"),
        title: format!("Reserved container run {number}"),
        purpose: "Preserve an unallocated historical namespace position so it can never be reused."
            .to_string(),
        owner: "bijux-dna-containers".to_string(),
        operation: "run_id_reservation".to_string(),
        stage_id: "container.run_reservation".to_string(),
        tool_ids: Vec::new(),
        contract_ids: vec!["container-run-identity-policy".to_string()],
        corpus_roles: Vec::new(),
        acceptance_policy: "the run ID remains permanently unavailable for allocation".to_string(),
        state: QualificationRunState::Reserved,
        state_reason: Some(tombstone.reason.clone()),
        supersedes_run_ids: Vec::new(),
        superseded_by_run_id: None,
        citation: citation_record(
            number,
            vec!["repo://containers/contracts/qualification-run-history.toml".to_string()],
            vec!["This identity is a tombstone and has no executable publication.".to_string()],
        ),
    }
}

fn citation_record(
    number: u64,
    evidence_uris: Vec<String>,
    limitations: Vec<String>,
) -> QualificationRunCitationRecord {
    QualificationRunCitationRecord {
        citation_key: format!("bijux-container-run-{number}"),
        protocol_version: "1".to_string(),
        registry_anchor: format!(
            "repo://containers/contracts/qualification-runs.toml#run-{number}"
        ),
        evidence_uris,
        limitations,
    }
}

pub(super) fn summarize_run_definitions(
    runs: &[QualificationRunDefinition],
) -> QualificationRunRegistrySummary {
    QualificationRunRegistrySummary {
        run_count: runs.len(),
        campaign_count: runs
            .iter()
            .filter(|run| run.state == QualificationRunState::Campaign)
            .count(),
        active_count: runs.iter().filter(|run| run.state == QualificationRunState::Active).count(),
        allocated_count: runs
            .iter()
            .filter(|run| run.state == QualificationRunState::Allocated)
            .count(),
        historical_count: runs
            .iter()
            .filter(|run| run.state == QualificationRunState::Historical)
            .count(),
        reserved_count: runs
            .iter()
            .filter(|run| run.state == QualificationRunState::Reserved)
            .count(),
    }
}
