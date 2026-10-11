//! Owner: bijux-dna-containers
//! Deterministic allocation and admission of stable qualification-run identities.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::{
    QualificationHistoricalRunIdentity, QualificationRunIdAudit, QualificationRunIdAuditRow,
    QualificationRunIdTombstone, QualificationRunRegistry, QualificationRunRegistryEntry,
    QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V2, QUALIFICATION_RUN_REGISTRY_SCHEMA_V1,
};
use crate::evidence::tool_contract::{load_container_catalog, ContainerCatalog};

const EXPECTED_RETAINED_TOOLS: usize = 85;
const EXPECTED_UNSUPPORTED_TOOLS: usize = 4;

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

/// Derive one stable run identity per retained tool without reusing historical IDs.
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
        .map(|path| load_qualification_run_registry(&request.repository_root, path))
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
    let registry = QualificationRunRegistry {
        schema_version: QUALIFICATION_RUN_REGISTRY_SCHEMA_V1.to_string(),
        catalog_sha256: catalog_sha256.clone(),
        historical_max_run_number: existing
            .as_ref()
            .map_or(historical_max_run_number, |registry| registry.historical_max_run_number),
        tools: derived_rows.entries,
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
        retained_mapping_count: registry.tools.iter().filter(|row| row.run_id.is_some()).count(),
        unsupported_tool_count: registry.tools.iter().filter(|row| row.run_id.is_none()).count(),
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
    let mut derived = DerivedRegistryRows {
        entries: Vec::with_capacity(catalog.tools.len()),
        audit_rows: Vec::with_capacity(catalog.tools.len()),
        allocated_mapping_count: 0,
    };
    for tool in &catalog.tools {
        let retained = tool.support_disposition == "retained";
        let (run_id, identity_source, resource_profile, acceptance_adapter) = if retained {
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
                let number = next_run_number;
                next_run_number =
                    next_run_number.checked_add(1).context("qualification run number overflow")?;
                derived.allocated_mapping_count += 1;
                (format!("run-{number}"), "allocated_after_historical_max")
            };
            (
                Some(run_id),
                source,
                Some(resource_profile_for(&tool.real_test_input_kind)?.to_string()),
                Some(acceptance_adapter_for(&tool.real_test_stage).to_string()),
            )
        } else if tool.support_disposition == "unsupported" {
            (None, "unsupported", None, None)
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

/// Load and validate the governed qualification-run registry against the current catalog.
///
/// # Errors
/// Returns an error when bytes, TOML, catalog identity, counts, or mappings are invalid.
pub fn load_qualification_run_registry(
    repository_root: &Path,
    registry_path: &Path,
) -> Result<QualificationRunRegistry> {
    let bytes = fs::read(registry_path)
        .with_context(|| format!("read qualification registry {}", registry_path.display()))?;
    let registry: QualificationRunRegistry = toml::from_slice(&bytes)
        .with_context(|| format!("parse qualification registry {}", registry_path.display()))?;
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

/// Validate catalog closure, support disposition, run identity, stage, and adapter contracts.
///
/// # Errors
/// Returns an error when the registry is incomplete, ambiguous, stale, or malformed.
pub fn validate_qualification_run_registry(
    catalog: &ContainerCatalog,
    registry: &QualificationRunRegistry,
) -> Result<()> {
    if registry.schema_version != QUALIFICATION_RUN_REGISTRY_SCHEMA_V1 {
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
                if row.run_id.is_some()
                    || row.resource_profile.is_some()
                    || row.acceptance_adapter.is_some()
                {
                    bail!("unsupported tool {} has runnable qualification fields", row.tool_id);
                }
            }
            other => bail!("unexpected support disposition {other}"),
        }
    }
    if retained != EXPECTED_RETAINED_TOOLS || unsupported != EXPECTED_UNSUPPORTED_TOOLS {
        bail!("qualification registry must contain 85 retained and four unsupported tools");
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
