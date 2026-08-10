//! Owner: bijux-dna-containers
//! Deterministic allocation and admission of stable qualification-run identities.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::{
    ContainerRunIndex, ContainerRunIndexSource, QualificationHistoricalRunIdentity,
    QualificationRunCitationRecord, QualificationRunDefinition, QualificationRunIdAudit,
    QualificationRunIdAuditRow, QualificationRunIdTombstone, QualificationRunNamespacePolicy,
    QualificationRunRegistry, QualificationRunRegistryEntry, QualificationRunRegistrySummary,
    QualificationRunState, CONTAINER_RUN_INDEX_SCHEMA_V1, QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V2,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V2,
};
use crate::evidence::tool_contract::{
    load_container_catalog, ContainerCatalog, ContainerCatalogTool,
};

const EXPECTED_RETAINED_TOOLS: usize = 85;
const EXPECTED_UNSUPPORTED_TOOLS: usize = 4;
const QUALIFICATION_RUN_REGISTRY_SCHEMA_V1: &str = "bijux.container.qualification_run_registry.v1";
const RUN_NAMESPACE_MINIMUM: u64 = 1000;
const RUN_NAMESPACE_MAXIMUM: u64 = 1999;
const RUN_NAMESPACE_NEXT: u64 = 1159;
const RUN_NAMESPACE_SIZE: usize = 159;

#[derive(Debug, Clone)]
pub struct QualificationRegistryDerivationRequest {
    pub repository_root: PathBuf,
    pub current_runs_root: PathBuf,
    pub attempt_ledger_path: PathBuf,
    pub supplemental_history_path: PathBuf,
    pub existing_registry_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DerivedQualificationRunRegistry {
    pub registry: QualificationRunRegistry,
    pub audit: QualificationRunIdAudit,
}

struct DerivedRegistryRows {
    entries: Vec<QualificationRunRegistryEntry>,
    audit_rows: Vec<QualificationRunIdAuditRow>,
    allocated_mapping_count: usize,
}

#[derive(Debug, Deserialize)]
struct CurrentRunEvidence {
    schema_version: String,
    run_id: String,
    status: String,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AttemptLedger {
    schema_version: String,
    attempts: Vec<AttemptLedgerRow>,
    summary: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AttemptLedgerRow {
    run_id: String,
    tool: String,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementalRunHistory {
    schema_version: String,
    identities: Vec<SupplementalRunIdentity>,
    #[serde(default)]
    tombstones: Vec<QualificationRunIdTombstone>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementalRunIdentity {
    run_id: String,
    tool_id: String,
    terminal_state: String,
    sources: Vec<SupplementalRunIdentitySource>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SupplementalRunIdentitySource {
    uri: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyQualificationRunRegistry {
    schema_version: String,
    catalog_sha256: String,
    historical_max_run_number: u64,
    tools: Vec<QualificationRunRegistryEntry>,
}

/// Derive one stable run identity per runnable tool without reusing historical IDs.
///
/// # Errors
/// Returns an error when catalog counts, current final runs, historical attempts, or run-ID
/// ownership are malformed or ambiguous.
pub fn derive_qualification_run_registry(
    request: &QualificationRegistryDerivationRequest,
) -> Result<DerivedQualificationRunRegistry> {
    let catalog = load_container_catalog(&request.repository_root)?;
    let catalog_path = request.repository_root.join("containers/contracts/catalog.toml");
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(&catalog_path)?;
    let mut historical = BTreeMap::new();
    let current = scan_current_runs(&request.current_runs_root, &mut historical)?;
    scan_attempt_ledger(&request.attempt_ledger_path, &mut historical)?;
    let tombstones =
        scan_supplemental_history(&request.supplemental_history_path, &mut historical)?;
    let historical_max_run_number = historical.keys().next_back().copied().unwrap_or(0).max(
        tombstones
            .iter()
            .map(|tombstone| parse_run_id(&tombstone.run_id))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .max()
            .unwrap_or(0),
    );
    let existing = request
        .existing_registry_path
        .as_deref()
        .filter(|path| path.is_file())
        .map(load_qualification_run_registry_for_derivation)
        .transpose()?;
    let reservations = existing
        .as_ref()
        .map(|registry| {
            registry
                .tools
                .iter()
                .filter_map(|row| {
                    row.run_id.as_deref().map(|run_id| (row.tool_id.as_str(), run_id))
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let catalog_tools = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<BTreeSet<_>>();
    if current.keys().any(|tool| !catalog_tools.contains(tool.as_str())) {
        bail!("current qualification runs contain a tool absent from the catalog");
    }

    let derived_rows =
        derive_registry_rows(&catalog, &current, &reservations, historical_max_run_number)?;
    let runs = derive_run_definitions(
        &catalog,
        &derived_rows.entries,
        &current,
        &historical,
        &tombstones,
    )?;
    let registry = QualificationRunRegistry {
        schema_version: QUALIFICATION_RUN_REGISTRY_SCHEMA_V2.to_string(),
        catalog_sha256: catalog_sha256.clone(),
        historical_max_run_number: existing
            .as_ref()
            .map_or(historical_max_run_number, |registry| registry.historical_max_run_number),
        namespace: QualificationRunNamespacePolicy {
            minimum: RUN_NAMESPACE_MINIMUM,
            maximum: RUN_NAMESPACE_MAXIMUM,
            next_run_number: RUN_NAMESPACE_NEXT,
            allocation_policy: "monotonic_only".to_string(),
            identity_policy: "existing_ids_immutable_attempts_reuse_run_id".to_string(),
        },
        summary: summarize_run_definitions(&runs),
        tools: derived_rows.entries,
        runs,
    };
    validate_qualification_run_registry(&catalog, &registry)?;
    validate_historical_ownership(&registry, &historical, &tombstones)?;
    let audit = QualificationRunIdAudit {
        schema_version: QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V2.to_string(),
        catalog_sha256,
        current_run_root: portable_repository_path(
            &request.current_runs_root,
            &request.repository_root,
        )?,
        attempt_ledger: portable_repository_path(
            &request.attempt_ledger_path,
            &request.repository_root,
        )?,
        supplemental_history: portable_repository_path(
            &request.supplemental_history_path,
            &request.repository_root,
        )?,
        observed_historical_run_count: historical.len(),
        reserved_run_count: tombstones.len(),
        historical_max_run_number,
        retained_mapping_count: disposition_count(&registry, "retained"),
        unsupported_tool_count: disposition_count(&registry, "unsupported"),
        allocated_mapping_count: derived_rows.allocated_mapping_count,
        historical_identities: historical
            .iter()
            .map(|(number, tool_id)| QualificationHistoricalRunIdentity {
                run_id: format!("run-{number}"),
                tool_id: tool_id.clone(),
            })
            .collect(),
        reserved_run_ids: tombstones,
        rows: derived_rows.audit_rows,
    };
    Ok(DerivedQualificationRunRegistry { registry, audit })
}

fn disposition_count(registry: &QualificationRunRegistry, disposition: &str) -> usize {
    registry.tools.iter().filter(|row| row.support_disposition == disposition).count()
}

fn derive_registry_rows(
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
    let missing_retained = catalog
        .tools
        .iter()
        .filter(|tool| tool.support_disposition == "retained")
        .filter(|tool| {
            !current.contains_key(&tool.id) && !reservations.contains_key(tool.id.as_str())
        })
        .count();
    let mut next_external_run_number = next_run_number
        .checked_add(
            u64::try_from(missing_retained).context("retained allocation count exceeds u64")?,
        )
        .context("qualification external-artifact run number overflow")?;
    let mut derived = DerivedRegistryRows {
        entries: Vec::with_capacity(catalog.tools.len()),
        audit_rows: Vec::with_capacity(catalog.tools.len()),
        allocated_mapping_count: 0,
    };
    for tool in &catalog.tools {
        let runnable = tool.support_disposition == "retained" || tool.external_artifact_supported;
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
            bail!("tool {} has unsupported disposition {}", tool.id, tool.support_disposition);
        };
        derived.audit_rows.push(QualificationRunIdAuditRow {
            tool_id: tool.id.clone(),
            support_disposition: tool.support_disposition.clone(),
            run_id: run_id.clone(),
            identity_source: identity_source.to_string(),
            stage_id: tool.real_test_stage.clone(),
            input_role: tool.real_test_input_kind.clone(),
        });
        derived.entries.push(QualificationRunRegistryEntry {
            tool_id: tool.id.clone(),
            support_disposition: tool.support_disposition.clone(),
            run_id,
            stage_id: tool.real_test_stage.clone(),
            input_role: tool.real_test_input_kind.clone(),
            resource_profile,
            acceptance_adapter,
        });
    }
    Ok(derived)
}

fn derive_run_definitions(
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
    let retained_tools = catalog
        .tools
        .iter()
        .filter(|tool| tool.support_disposition == "retained")
        .map(|tool| tool.id.clone())
        .collect::<Vec<_>>();
    let mut runs = Vec::with_capacity(RUN_NAMESPACE_SIZE);
    runs.push(QualificationRunDefinition {
        run_id: "run-1000".to_string(),
        title: "Retained container image campaign".to_string(),
        purpose: "Build and admit the exact retained container image roster from the governed catalog and recipes.".to_string(),
        owner: "bijux-dna-containers".to_string(),
        operation: "image_build_campaign".to_string(),
        stage_id: "container.image_campaign".to_string(),
        tool_ids: retained_tools,
        contract_ids: vec![
            "container-catalog".to_string(),
            "container-tool-contract".to_string(),
            "container-image-identity".to_string(),
        ],
        corpus_roles: vec!["recipe_source".to_string(), "tool_source".to_string()],
        acceptance_policy: "all retained tools publish digest-bound build and image identity evidence".to_string(),
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
        let is_current =
            tool.support_disposition == "retained" && current.get(tool_id) == Some(&number);
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

fn summarize_run_definitions(
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

/// Load and validate the governed qualification-run registry against the current catalog.
///
/// # Errors
/// Returns an error when bytes, TOML, catalog identity, counts, or mappings are invalid.
pub fn load_qualification_run_registry(
    repository_root: &Path,
    registry_path: &Path,
) -> Result<QualificationRunRegistry> {
    let registry = load_qualification_run_registry_for_derivation(registry_path)?;
    let catalog = load_container_catalog(repository_root)?;
    validate_qualification_run_registry(&catalog, &registry)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("containers/contracts/catalog.toml"),
    )?;
    if registry.catalog_sha256 != catalog_sha256 {
        bail!("qualification registry catalog identity is stale");
    }
    Ok(registry)
}

fn load_qualification_run_registry_for_derivation(
    registry_path: &Path,
) -> Result<QualificationRunRegistry> {
    let bytes = fs::read(registry_path)
        .with_context(|| format!("read qualification registry {}", registry_path.display()))?;
    let value: toml::Value = toml::from_slice(&bytes)
        .with_context(|| format!("parse qualification registry {}", registry_path.display()))?;
    let schema_version = value
        .get("schema_version")
        .and_then(toml::Value::as_str)
        .context("qualification registry lacks a schema version")?;
    let registry = if schema_version == QUALIFICATION_RUN_REGISTRY_SCHEMA_V1 {
        let legacy: LegacyQualificationRunRegistry =
            value.try_into().context("parse legacy qualification registry")?;
        QualificationRunRegistry {
            schema_version: legacy.schema_version,
            catalog_sha256: legacy.catalog_sha256,
            historical_max_run_number: legacy.historical_max_run_number,
            namespace: QualificationRunNamespacePolicy {
                minimum: RUN_NAMESPACE_MINIMUM,
                maximum: RUN_NAMESPACE_MAXIMUM,
                next_run_number: RUN_NAMESPACE_NEXT,
                allocation_policy: "monotonic_only".to_string(),
                identity_policy: "existing_ids_immutable_attempts_reuse_run_id".to_string(),
            },
            summary: QualificationRunRegistrySummary {
                run_count: 0,
                campaign_count: 0,
                active_count: 0,
                allocated_count: 0,
                historical_count: 0,
                reserved_count: 0,
            },
            tools: legacy.tools,
            runs: Vec::new(),
        }
    } else {
        value.try_into().context("parse qualification registry")?
    };
    validate_qualification_run_reservations(&registry)?;
    Ok(registry)
}

fn validate_qualification_run_reservations(registry: &QualificationRunRegistry) -> Result<()> {
    if !matches!(
        registry.schema_version.as_str(),
        QUALIFICATION_RUN_REGISTRY_SCHEMA_V1 | QUALIFICATION_RUN_REGISTRY_SCHEMA_V2
    ) {
        bail!("unsupported qualification-run registry schema");
    }
    validate_sha256(&registry.catalog_sha256, "qualification registry catalog")?;
    let mut tool_ids = BTreeSet::new();
    let mut run_ids = BTreeSet::new();
    for row in &registry.tools {
        if row.tool_id.is_empty() || !tool_ids.insert(row.tool_id.as_str()) {
            bail!("qualification registry contains an empty or duplicate tool ID");
        }
        match row.support_disposition.as_str() {
            "retained" => {
                let run_id = row.run_id.as_deref().context("retained tool has no run ID")?;
                parse_run_id(run_id)?;
                if !run_ids.insert(run_id) {
                    bail!("qualification registry contains duplicate run ID {run_id}");
                }
                if row.resource_profile.is_none() || row.acceptance_adapter.is_none() {
                    bail!("retained tool {} has incomplete execution fields", row.tool_id);
                }
            }
            "unsupported" => {
                let fields = [
                    row.run_id.is_some(),
                    row.resource_profile.is_some(),
                    row.acceptance_adapter.is_some(),
                ];
                if fields.iter().any(|present| *present) && fields.iter().any(|present| !*present) {
                    bail!(
                        "external-artifact tool {} has incomplete qualification fields",
                        row.tool_id
                    );
                }
            }
            other => bail!("unexpected support disposition {other}"),
        }
    }
    Ok(())
}

/// Validate catalog closure, support disposition, run identity, stage, and adapter contracts.
///
/// # Errors
/// Returns an error when the registry is incomplete, ambiguous, stale, or malformed.
pub fn validate_qualification_run_registry(
    catalog: &ContainerCatalog,
    registry: &QualificationRunRegistry,
) -> Result<()> {
    if registry.schema_version != QUALIFICATION_RUN_REGISTRY_SCHEMA_V2 {
        bail!("unsupported qualification-run registry schema");
    }
    validate_sha256(&registry.catalog_sha256, "qualification registry catalog")?;
    if catalog.tools.len() != EXPECTED_RETAINED_TOOLS + EXPECTED_UNSUPPORTED_TOOLS
        || registry.tools.len() != catalog.tools.len()
    {
        bail!("qualification registry does not close over the 89-tool catalog");
    }
    let mut tool_ids = BTreeSet::new();
    let mut run_ids = BTreeSet::new();
    let mut retained = 0;
    let mut unsupported = 0;
    for (tool, row) in catalog.tools.iter().zip(&registry.tools) {
        if row.tool_id != tool.id
            || row.support_disposition != tool.support_disposition
            || row.stage_id != tool.real_test_stage
            || row.input_role != tool.real_test_input_kind
        {
            bail!("qualification registry row differs from catalog tool {}", tool.id);
        }
        if !tool_ids.insert(row.tool_id.as_str()) {
            bail!("qualification registry contains duplicate tool {}", row.tool_id);
        }
        match tool.support_disposition.as_str() {
            "retained" => {
                retained += 1;
                let run_id = row.run_id.as_deref().context("retained tool has no run ID")?;
                parse_run_id(run_id)?;
                if !run_ids.insert(run_id) {
                    bail!("qualification registry contains duplicate run ID {run_id}");
                }
                if row.resource_profile.as_deref() != Some(resource_profile_for(&row.input_role)?)
                    || row.acceptance_adapter.as_deref()
                        != Some(acceptance_adapter_for(&row.stage_id))
                {
                    bail!(
                        "qualification registry adapters differ for retained tool {}",
                        row.tool_id
                    );
                }
            }
            "unsupported" => {
                unsupported += 1;
                if !tool.external_artifact_supported {
                    bail!(
                        "unsupported tool {} lacks governed external-artifact support",
                        row.tool_id
                    );
                }
                let run_id = row
                    .run_id
                    .as_deref()
                    .context("external-artifact tool has no permanent run ID")?;
                parse_run_id(run_id)?;
                if !run_ids.insert(run_id) {
                    bail!("qualification registry contains duplicate run ID {run_id}");
                }
                if row.resource_profile.as_deref() != Some(resource_profile_for(&row.input_role)?)
                    || row.acceptance_adapter.as_deref()
                        != Some(acceptance_adapter_for(&row.stage_id))
                {
                    bail!(
                        "qualification registry adapters differ for external-artifact tool {}",
                        row.tool_id
                    );
                }
            }
            other => bail!("unexpected support disposition {other}"),
        }
    }
    if retained != EXPECTED_RETAINED_TOOLS || unsupported != EXPECTED_UNSUPPORTED_TOOLS {
        bail!("qualification registry must contain 85 retained and four unsupported tools");
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
    let retained_tools = catalog
        .tools
        .iter()
        .filter(|tool| tool.support_disposition == "retained")
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
            &retained_tools,
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
    retained_tools: &BTreeSet<&str>,
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
                    != *retained_tools
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

/// Render deterministic checked-in TOML.
///
/// # Errors
/// Returns an error when registry serialization fails.
pub fn render_qualification_run_registry(registry: &QualificationRunRegistry) -> Result<String> {
    Ok(format!(
        "# GENERATED FILE - DO NOT EDIT\n{}",
        toml::to_string_pretty(registry).context("serialize qualification-run registry")?
    ))
}

/// Derive the complete paper-facing index from the canonical checked-in registry.
///
/// # Errors
/// Returns an error when the source registry, catalog closure, path, or index identity is invalid.
pub fn derive_container_run_index(
    repository_root: &Path,
    registry_path: &Path,
) -> Result<ContainerRunIndex> {
    let expected = repository_root.join("containers/contracts/qualification-runs.toml");
    if registry_path != expected {
        bail!("container run index requires the canonical qualification registry");
    }
    let registry = load_qualification_run_registry(repository_root, registry_path)?;
    let bytes = fs::read(registry_path)
        .with_context(|| format!("read qualification registry {}", registry_path.display()))?;
    let index = ContainerRunIndex {
        schema_version: CONTAINER_RUN_INDEX_SCHEMA_V1.to_string(),
        source_registry: ContainerRunIndexSource {
            uri: "repo://containers/contracts/qualification-runs.toml".to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
            size_bytes: u64::try_from(bytes.len()).context("qualification registry exceeds u64")?,
        },
        catalog_sha256: registry.catalog_sha256,
        historical_max_run_number: registry.historical_max_run_number,
        namespace: registry.namespace,
        summary: registry.summary,
        runs: registry.runs,
    };
    validate_container_run_index(&index)?;
    Ok(index)
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

/// Render deterministic container run-index JSON.
///
/// # Errors
/// Returns an error when serialization fails.
pub fn render_container_run_index_json(index: &ContainerRunIndex) -> Result<String> {
    validate_container_run_index(index)?;
    Ok(serde_json::to_string_pretty(index)? + "\n")
}

#[must_use]
pub fn render_container_run_index_markdown(index: &ContainerRunIndex) -> String {
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-containers::evidence::qualification_registry -->".to_string(),
        "# Container run index".to_string(),
        String::new(),
        "Run IDs are permanent protocols; scheduler jobs and retries are attempts under those IDs."
            .to_string(),
        String::new(),
        format!("- Registered protocols: **{}**", index.summary.run_count),
        format!("- Active protocols: **{}**", index.summary.active_count),
        format!("- Allocated protocols: **{}**", index.summary.allocated_count),
        format!("- Historical protocols: **{}**", index.summary.historical_count),
        format!("- Reserved IDs: **{}**", index.summary.reserved_count),
        String::new(),
        "| Run | State | Stage | Tools | Citation | Evidence |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    for run in &index.runs {
        let state = match run.state {
            QualificationRunState::Campaign => "campaign",
            QualificationRunState::Active => "active",
            QualificationRunState::Allocated => "allocated",
            QualificationRunState::Historical => "historical",
            QualificationRunState::Reserved => "reserved",
        };
        let evidence =
            if run.citation.evidence_uris.is_empty() { "not selected" } else { "indexed" };
        lines.push(format!(
            "| `{}` | `{state}` | `{}` | {} | `{}` | {evidence} |",
            run.run_id,
            run.stage_id,
            run.tool_ids.iter().map(|tool| format!("`{tool}`")).collect::<Vec<_>>().join(", "),
            run.citation.citation_key,
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn scan_current_runs(
    root: &Path,
    historical: &mut BTreeMap<u64, String>,
) -> Result<BTreeMap<String, u64>> {
    let mut current = BTreeMap::new();
    for entry in
        fs::read_dir(root).with_context(|| format!("read current runs {}", root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let Some((number, tool)) = parse_run_directory(&name)? else {
            continue;
        };
        let evidence_path = entry.path().join("evidence.json");
        if !evidence_path.is_file() {
            continue;
        }
        let evidence: CurrentRunEvidence = serde_json::from_slice(&fs::read(&evidence_path)?)
            .with_context(|| format!("parse final run evidence {}", evidence_path.display()))?;
        let _ = evidence.remaining;
        if !matches!(
            evidence.schema_version.as_str(),
            "bijux.container.evidence.v1" | "bijux.container.evidence.v2"
        ) {
            bail!("checked-in final run {name} has an unsupported evidence schema");
        }
        if evidence.run_id != format!("run-{number}") {
            bail!("checked-in final run {name} disagrees with its evidence run ID");
        }
        if evidence.status != "passed" {
            bail!("checked-in final run {name} is not passed");
        }
        insert_historical(historical, number, tool)?;
        if current.insert(tool.to_string(), number).is_some() {
            bail!("multiple checked-in final runs exist for tool {tool}");
        }
    }
    Ok(current)
}

fn scan_attempt_ledger(path: &Path, historical: &mut BTreeMap<u64, String>) -> Result<()> {
    let ledger: AttemptLedger = serde_json::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse qualification attempt ledger {}", path.display()))?;
    if ledger.schema_version != "bijux.container.run_attempt_ledger.v1" {
        bail!("unsupported qualification attempt ledger schema");
    }
    let _ = ledger.summary;
    for row in ledger.attempts {
        let number = parse_run_id(&row.run_id)?;
        if row.tool.trim().is_empty() {
            bail!("qualification attempt ledger contains an empty tool ID");
        }
        let _ = row.remaining;
        insert_historical(historical, number, &row.tool)?;
    }
    Ok(())
}

fn scan_supplemental_history(
    path: &Path,
    historical: &mut BTreeMap<u64, String>,
) -> Result<Vec<QualificationRunIdTombstone>> {
    let history: SupplementalRunHistory = toml::from_slice(&fs::read(path)?)
        .with_context(|| format!("parse supplemental qualification history {}", path.display()))?;
    if history.schema_version != "bijux.container.qualification_run_history.v2" {
        bail!("unsupported supplemental qualification history schema");
    }
    let mut seen_identities = BTreeSet::new();
    for identity in history.identities {
        let number = parse_run_id(&identity.run_id)?;
        if identity.tool_id.trim().is_empty() || identity.terminal_state != "submission_failed" {
            bail!("supplemental qualification identity is malformed");
        }
        if !seen_identities.insert(number) {
            bail!("supplemental qualification history contains a duplicate run ID");
        }
        if identity.sources.is_empty() {
            bail!("supplemental qualification identity has no immutable sources");
        }
        let expected_path_segment = format!("/{}-{}/", identity.run_id, identity.tool_id);
        let mut seen_sources = BTreeSet::new();
        for source in identity.sources {
            if !source.uri.starts_with("study://")
                || !source.uri.contains(&expected_path_segment)
                || !seen_sources.insert(source.uri.clone())
            {
                bail!("supplemental qualification source identity is malformed");
            }
            validate_sha256(&source.sha256, "supplemental qualification source")?;
        }
        insert_historical(historical, number, &identity.tool_id)?;
    }
    let mut seen_tombstones = BTreeSet::new();
    for tombstone in &history.tombstones {
        let number = parse_run_id(&tombstone.run_id)?;
        if !(1000..2000).contains(&number)
            || tombstone.disposition != "reserved_never_reuse"
            || tombstone.reason.trim().is_empty()
            || !seen_tombstones.insert(number)
        {
            bail!("supplemental qualification run-ID tombstone is malformed");
        }
        if historical.contains_key(&number) {
            bail!("supplemental qualification run-ID tombstone conflicts with a real run");
        }
    }
    Ok(history.tombstones)
}

fn insert_historical(
    historical: &mut BTreeMap<u64, String>,
    number: u64,
    tool: &str,
) -> Result<()> {
    if let Some(existing) = historical.insert(number, tool.to_string()) {
        if existing != tool {
            bail!("historical run-{number} is assigned to both {existing} and {tool}");
        }
    }
    Ok(())
}

fn validate_historical_ownership(
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

fn parse_run_directory(value: &str) -> Result<Option<(u64, &str)>> {
    let Some(rest) = value.strip_prefix("run-") else {
        return Ok(None);
    };
    let (number, tool) = rest
        .split_once('-')
        .with_context(|| format!("malformed qualification run directory {value}"))?;
    if tool.is_empty() {
        bail!("malformed qualification run directory {value}");
    }
    Ok(Some((number.parse().context("parse qualification run number")?, tool)))
}

fn parse_run_id(value: &str) -> Result<u64> {
    let number = value
        .strip_prefix("run-")
        .with_context(|| format!("qualification run ID is malformed: {value}"))?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("qualification run ID is malformed: {value}");
    }
    number.parse().context("parse qualification run ID")
}

fn resource_profile_for(input_role: &str) -> Result<&'static str> {
    match input_role {
        "fastq" => Ok("fastq-standard"),
        "bam" => Ok("bam-standard"),
        "bam_with_reference" => Ok("bam-reference"),
        "vcf" => Ok("vcf-standard"),
        "hap_ibd_segments" => Ok("ibd-segments"),
        other => bail!("qualification input role has no resource profile: {other}"),
    }
}

fn acceptance_adapter_for(stage_id: &str) -> &'static str {
    if stage_id == "container.runtime" {
        "runtime-contract"
    } else {
        "stage-contract"
    }
}

fn portable_repository_path(path: &Path, repository_root: &Path) -> Result<String> {
    Ok(format!(
        "repository://{}",
        path.canonicalize()?
            .strip_prefix(repository_root.canonicalize()?)
            .context("qualification registry input is outside the repository")?
            .to_string_lossy()
            .replace('\\', "/")
    ))
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} SHA-256 is malformed");
    }
    Ok(())
}
