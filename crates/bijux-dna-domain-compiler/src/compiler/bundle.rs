use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    anyhow, collect_yaml_files, ensure_dir, read_yaml, write_string, Context, DomainIndex, Result,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainRegistrySchemas {
    pub stage_schema_version: String,
    pub tool_schema_version: String,
    pub artifact_schema_version: String,
    pub metric_schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageParameterDefault {
    pub name: String,
    pub param_type: String,
    pub default_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryPort {
    pub name: String,
    pub data_type: String,
    pub cardinality: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageMetricContract {
    pub metric_id: String,
    pub meaning: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolContainerContract {
    pub image: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainStageContract {
    pub stage_id: String,
    pub status: String,
    pub evidence_status: String,
    pub scope: String,
    pub description: String,
    pub defaults_source: String,
    pub metrics_schema: String,
    pub inputs: Vec<RegistryPort>,
    pub outputs: Vec<RegistryPort>,
    pub required_inputs: Vec<String>,
    pub required_outputs: Vec<String>,
    pub parameters: Vec<StageParameterDefault>,
    pub metrics: Vec<StageMetricContract>,
    pub tool_capability_requirements: Vec<String>,
    pub compatible_tools: Vec<String>,
    pub assumptions: Vec<String>,
    pub invariants: Vec<String>,
    pub allowed_missingness: Vec<String>,
    pub bank_hooks: Vec<String>,
    pub planned_out_of_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainToolContract {
    pub tool_id: String,
    pub status: String,
    pub scope: String,
    pub role: String,
    pub default_version: String,
    pub upstream: String,
    pub pin_strategy: String,
    pub license: String,
    pub citation: String,
    pub version_cmd: String,
    pub help_cmd: String,
    pub stage_ids: Vec<String>,
    pub planned_stage_ids: Vec<String>,
    pub expected_artifacts: Vec<String>,
    pub capabilities: Vec<String>,
    pub metrics_schema_id: String,
    pub comparability_notes: String,
    pub container: Option<ToolContainerContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactRoleSnapshot {
    pub artifact_id: String,
    pub artifact_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainMetricEntry {
    pub metric_id: String,
    pub schema_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultSettingsContract {
    pub stage_id: String,
    pub tool_id: String,
    pub source: String,
    pub rationale: String,
    pub governance_status: String,
    pub override_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryFixtureBinding {
    pub stage_id: String,
    pub tool_id: String,
    pub fixture_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegistryDeprecationRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    pub deprecated_since: String,
    pub removal_after: String,
    pub rationale: String,
    pub replacement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledDomainRegistry {
    pub domain_id: String,
    pub domain_version: String,
    pub schemas: DomainRegistrySchemas,
    pub stage_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub governed_stage_ids: Vec<String>,
    pub governed_tool_ids: Vec<String>,
    pub stages: Vec<DomainStageContract>,
    pub tools: Vec<DomainToolContract>,
    pub metrics: Vec<DomainMetricEntry>,
    pub artifacts: Vec<ArtifactRoleSnapshot>,
    pub defaults: Vec<DefaultSettingsContract>,
    pub fixtures: Vec<RegistryFixtureBinding>,
    pub deprecations: Vec<RegistryDeprecationRecord>,
    pub checksum_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainRegistryReleaseBundle {
    pub schema_version: String,
    pub source_commit: String,
    pub bundle_checksum_sha256: String,
    pub domains: Vec<CompiledDomainRegistry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompiledDomainDefaultsSnapshot {
    pub domain_id: String,
    pub defaults: Vec<DefaultSettingsContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactContractSnapshot {
    pub domain_id: String,
    pub artifacts: Vec<ArtifactRoleSnapshot>,
    pub stage_outputs: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainMetricCatalog {
    pub domain_id: String,
    pub metrics: Vec<DomainMetricEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainDeprecationCatalog {
    pub domain_id: String,
    pub deprecations: Vec<RegistryDeprecationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainInvariantCatalog {
    pub domain_id: String,
    pub stage_invariants: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainRegistryQueryKind {
    Domains,
    Stages,
    Tools,
    Metrics,
    Artifacts,
    Defaults,
    Deprecations,
    Fixtures,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomainRegistryQuery {
    pub kind: DomainRegistryQueryKind,
    pub domain_id: Option<String>,
    pub stage_id: Option<String>,
    pub tool_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SchemaDoc {
    #[serde(default)]
    schema_version: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleStageParameterDoc {
    name: String,
    #[serde(default)]
    param_type: String,
    #[serde(default)]
    default: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleStageMetricDoc {
    name: String,
    #[serde(default)]
    meaning: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleStageDoc {
    #[serde(default)]
    stage_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    defaults_source: String,
    #[serde(default)]
    metrics_schema: String,
    #[serde(default)]
    inputs: Vec<RegistryPort>,
    #[serde(default)]
    outputs: Vec<RegistryPort>,
    #[serde(default)]
    required_inputs: Vec<String>,
    #[serde(default)]
    required_outputs: Vec<String>,
    #[serde(default)]
    parameters: Vec<BundleStageParameterDoc>,
    #[serde(default)]
    metrics: Vec<BundleStageMetricDoc>,
    #[serde(default)]
    tool_capability_requirements: Vec<String>,
    #[serde(default)]
    compatible_tools: Vec<String>,
    #[serde(default)]
    assumptions: Vec<String>,
    #[serde(default)]
    invariants: Vec<String>,
    #[serde(default)]
    allowed_missingness: Vec<String>,
    #[serde(default)]
    bank_hooks: Vec<String>,
    #[serde(default)]
    planned_out_of_scope: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleToolContainerDoc {
    #[serde(default)]
    image: String,
    #[serde(default)]
    digest: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BundleToolDoc {
    #[serde(default)]
    tool_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    default_version: String,
    #[serde(default)]
    upstream: String,
    #[serde(default)]
    pin_strategy: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    citation: String,
    #[serde(default)]
    version_cmd: String,
    #[serde(default)]
    help_cmd: String,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    planned_stage_ids: Vec<String>,
    #[serde(default)]
    expected_artifacts: Vec<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    metrics_schema_id: String,
    #[serde(default)]
    comparability_notes: String,
    #[serde(default)]
    container: Option<BundleToolContainerDoc>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactDoc {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    artifact_ids: Vec<String>,
    #[serde(default)]
    artifacts: Vec<ArtifactDocEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactDocEntry {
    id: String,
    #[serde(default, rename = "type")]
    artifact_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MetricDoc {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    metric_ids: Vec<String>,
    #[serde(default)]
    metrics: Vec<MetricDocEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct MetricDocEntry {
    id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DeprecationsDoc {
    #[serde(default)]
    deprecations: Vec<DeprecationDocEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DeprecationDocEntry {
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    tool_id: Option<String>,
    #[serde(default)]
    deprecated_since: String,
    #[serde(default)]
    removal_after: String,
    #[serde(default)]
    rationale: String,
    #[serde(default)]
    replacement: String,
}

impl Default for DomainRegistryQueryKind {
    fn default() -> Self {
        Self::Domains
    }
}

fn sort_and_dedup(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value).context("serialize registry json for checksum")?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn schema_version(path: &Path) -> Result<String> {
    let schema: SchemaDoc = read_yaml(path)?;
    Ok(schema.schema_version)
}

fn load_stage_docs(domain_dir: &Path, domain_id: &str) -> Result<Vec<BundleStageDoc>> {
    let mut stages = Vec::new();
    for path in collect_yaml_files(&domain_dir.join(domain_id).join("stages"))? {
        if path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.starts_with('_'))
        {
            continue;
        }
        let stage: BundleStageDoc = read_yaml(&path)?;
        stages.push(stage);
    }
    stages.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    Ok(stages)
}

fn load_tool_docs(domain_dir: &Path, domain_id: &str) -> Result<Vec<BundleToolDoc>> {
    let mut tools = Vec::new();
    for path in collect_yaml_files(&domain_dir.join(domain_id).join("tools"))? {
        if path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.starts_with('_'))
        {
            continue;
        }
        let tool: BundleToolDoc = read_yaml(&path)?;
        tools.push(tool);
    }
    tools.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    Ok(tools)
}

fn load_artifacts(domain_dir: &Path, domain_id: &str) -> Result<(String, Vec<ArtifactRoleSnapshot>)> {
    let path = domain_dir.join(domain_id).join("artifacts.yaml");
    let doc: ArtifactDoc = read_yaml(&path)?;
    if doc.domain != domain_id {
        return Err(anyhow!(
            "{} has domain {} but expected {}",
            path.display(),
            doc.domain,
            domain_id
        ));
    }
    let mut artifacts = if doc.artifacts.is_empty() {
        doc.artifact_ids
            .into_iter()
            .map(|artifact_id| ArtifactRoleSnapshot { artifact_id, artifact_type: None })
            .collect::<Vec<_>>()
    } else {
        doc.artifacts
            .into_iter()
            .map(|entry| ArtifactRoleSnapshot {
                artifact_id: entry.id,
                artifact_type: entry.artifact_type,
            })
            .collect::<Vec<_>>()
    };
    artifacts.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    Ok((doc.schema_version, artifacts))
}

fn load_metrics(domain_dir: &Path, domain_id: &str) -> Result<(String, Vec<DomainMetricEntry>)> {
    let path = domain_dir.join(domain_id).join("metrics.yaml");
    let doc: MetricDoc = read_yaml(&path)?;
    if doc.domain != domain_id {
        return Err(anyhow!(
            "{} has domain {} but expected {}",
            path.display(),
            doc.domain,
            domain_id
        ));
    }
    let mut metrics = if doc.metrics.is_empty() {
        doc.metric_ids
            .into_iter()
            .map(|metric_id| DomainMetricEntry {
                metric_id,
                schema_version: doc.schema_version.clone(),
            })
            .collect::<Vec<_>>()
    } else {
        doc.metrics
            .into_iter()
            .map(|entry| DomainMetricEntry {
                metric_id: entry.id,
                schema_version: doc.schema_version.clone(),
            })
            .collect::<Vec<_>>()
    };
    metrics.sort_by(|left, right| left.metric_id.cmp(&right.metric_id));
    Ok((doc.schema_version, metrics))
}

fn load_deprecations(workspace_root: &Path) -> Result<Vec<DeprecationDocEntry>> {
    let path = workspace_root.join("configs/ci/registry/deprecations.toml");
    let text = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let parsed: DeprecationsDoc =
        toml::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed.deprecations)
}

fn stage_outputs_by_stage(stages: &[DomainStageContract]) -> BTreeMap<String, Vec<String>> {
    stages
        .iter()
        .map(|stage| {
            (
                stage.stage_id.clone(),
                stage.outputs.iter().map(|port| port.name.clone()).collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn build_domain_defaults(
    index: &DomainIndex,
    stages_by_id: &BTreeMap<String, DomainStageContract>,
) -> Vec<DefaultSettingsContract> {
    let mut defaults = index
        .active_defaults
        .iter()
        .filter_map(|(stage_id, tool_id)| {
            stages_by_id.get(stage_id).map(|stage| DefaultSettingsContract {
                stage_id: stage_id.clone(),
                tool_id: tool_id.clone(),
                source: stage.defaults_source.clone(),
                rationale: index.active_default_rationale.get(stage_id).cloned().unwrap_or_default(),
                governance_status: if stage.status == "supported" {
                    "enforced".to_string()
                } else {
                    "advisory".to_string()
                },
                override_policy: "explicit profile override required".to_string(),
            })
        })
        .collect::<Vec<_>>();
    defaults.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    defaults
}

fn build_fixture_bindings(
    domain_dir: &Path,
    domain_id: &str,
    index: &DomainIndex,
    stages_by_id: &BTreeMap<String, DomainStageContract>,
) -> Vec<RegistryFixtureBinding> {
    let mut fixtures = Vec::new();
    for (stage_id, tool_ids) in &index.stage_tool_compatibility {
        let Some(stage) = stages_by_id.get(stage_id) else {
            continue;
        };
        if stage.status != "supported" {
            continue;
        }
        for tool_id in tool_ids {
            let fixture_path = domain_dir
                .join(domain_id)
                .join("fixtures")
                .join(stage_id)
                .join(format!("{tool_id}.txt"));
            if fixture_path.exists() {
                fixtures.push(RegistryFixtureBinding {
                    stage_id: stage_id.clone(),
                    tool_id: tool_id.clone(),
                    fixture_path: fixture_path.display().to_string(),
                });
            }
        }
    }
    fixtures.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    fixtures
}

fn build_deprecation_records(
    domain_id: &str,
    stage_ids: &[String],
    tool_ids: &[String],
    entries: &[DeprecationDocEntry],
) -> Vec<RegistryDeprecationRecord> {
    let stage_set = stage_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    let tool_set = tool_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    let mut out = entries
        .iter()
        .filter(|entry| {
            entry
                .stage
                .as_deref()
                .is_some_and(|stage_id| stage_id.starts_with(&format!("{domain_id}.")))
                || entry.tool_id.as_deref().is_some_and(|tool_id| tool_set.contains(tool_id))
        })
        .filter(|entry| {
            entry.stage.as_deref().is_none_or(|stage_id| stage_set.contains(stage_id))
                && entry.tool_id.as_deref().is_none_or(|tool_id| tool_set.contains(tool_id))
        })
        .map(|entry| RegistryDeprecationRecord {
            stage_id: entry.stage.clone(),
            tool_id: entry.tool_id.clone(),
            deprecated_since: entry.deprecated_since.clone(),
            removal_after: entry.removal_after.clone(),
            rationale: entry.rationale.clone(),
            replacement: entry.replacement.clone(),
        })
        .collect::<Vec<_>>();
    out.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    out
}

fn to_stage_contracts(
    stage_schema_version: &str,
    stages: Vec<BundleStageDoc>,
) -> Vec<DomainStageContract> {
    let mut out = stages
        .into_iter()
        .map(|stage| DomainStageContract {
            stage_id: stage.stage_id,
            status: stage.status.clone(),
            evidence_status: stage.status,
            scope: stage.scope,
            description: stage.description,
            defaults_source: stage.defaults_source,
            metrics_schema: stage.metrics_schema,
            inputs: stage.inputs,
            outputs: stage.outputs,
            required_inputs: stage.required_inputs,
            required_outputs: stage.required_outputs,
            parameters: stage
                .parameters
                .into_iter()
                .map(|parameter| StageParameterDefault {
                    name: parameter.name,
                    param_type: parameter.param_type,
                    default_value: parameter.default,
                })
                .collect(),
            metrics: stage
                .metrics
                .into_iter()
                .map(|metric| StageMetricContract {
                    metric_id: metric.name,
                    meaning: metric.meaning,
                    schema_version: stage_schema_version.to_string(),
                })
                .collect(),
            tool_capability_requirements: sort_and_dedup(stage.tool_capability_requirements),
            compatible_tools: sort_and_dedup(stage.compatible_tools),
            assumptions: stage.assumptions,
            invariants: stage.invariants,
            allowed_missingness: sort_and_dedup(stage.allowed_missingness),
            bank_hooks: sort_and_dedup(stage.bank_hooks),
            planned_out_of_scope: sort_and_dedup(stage.planned_out_of_scope),
        })
        .collect::<Vec<_>>();
    out.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    out
}

fn to_tool_contracts(tools: Vec<BundleToolDoc>) -> Vec<DomainToolContract> {
    let mut out = tools
        .into_iter()
        .map(|tool| DomainToolContract {
            tool_id: tool.tool_id,
            status: tool.status,
            scope: tool.scope,
            role: tool.role,
            default_version: tool.default_version,
            upstream: tool.upstream,
            pin_strategy: tool.pin_strategy,
            license: tool.license,
            citation: tool.citation,
            version_cmd: tool.version_cmd,
            help_cmd: tool.help_cmd,
            stage_ids: sort_and_dedup(tool.stage_ids),
            planned_stage_ids: sort_and_dedup(tool.planned_stage_ids),
            expected_artifacts: sort_and_dedup(tool.expected_artifacts),
            capabilities: sort_and_dedup(tool.capabilities),
            metrics_schema_id: tool.metrics_schema_id,
            comparability_notes: tool.comparability_notes,
            container: tool.container.map(|container| ToolContainerContract {
                image: container.image,
                digest: container.digest,
            }),
        })
        .collect::<Vec<_>>();
    out.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    out
}

fn build_single_domain_registry(
    domain_dir: &Path,
    domain_id: &str,
    source_commit: &str,
    deprecations: &[DeprecationDocEntry],
) -> Result<CompiledDomainRegistry> {
    let index_path = domain_dir.join(domain_id).join("index.yaml");
    let index: DomainIndex = read_yaml(&index_path)?;
    let stage_schema_version = schema_version(&domain_dir.join(domain_id).join("stages/_schema.yaml"))?;
    let tool_schema_version = schema_version(&domain_dir.join(domain_id).join("tools/_schema.yaml"))?;
    let stage_contracts = to_stage_contracts(&stage_schema_version, load_stage_docs(domain_dir, domain_id)?);
    let stages_by_id = stage_contracts
        .iter()
        .cloned()
        .map(|stage| (stage.stage_id.clone(), stage))
        .collect::<BTreeMap<_, _>>();
    let tool_contracts = to_tool_contracts(load_tool_docs(domain_dir, domain_id)?);
    let (artifact_schema_version, artifacts) = load_artifacts(domain_dir, domain_id)?;
    let (metric_schema_version, metrics) = load_metrics(domain_dir, domain_id)?;
    let defaults = build_domain_defaults(&index, &stages_by_id);
    let fixtures = build_fixture_bindings(domain_dir, domain_id, &index, &stages_by_id);
    let stage_ids = sort_and_dedup(index.stage_ids.clone());
    let tool_ids = sort_and_dedup(index.tool_ids.clone());
    let deprecations = build_deprecation_records(domain_id, &stage_ids, &tool_ids, deprecations);
    let mut registry = CompiledDomainRegistry {
        domain_id: domain_id.to_string(),
        domain_version: index.domain_version,
        schemas: DomainRegistrySchemas {
            stage_schema_version,
            tool_schema_version,
            artifact_schema_version,
            metric_schema_version,
        },
        stage_ids,
        tool_ids,
        governed_stage_ids: sort_and_dedup(index.active_defaults.keys().cloned().collect()),
        governed_tool_ids: sort_and_dedup(index.active_defaults.values().cloned().collect()),
        stages: stage_contracts,
        tools: tool_contracts,
        metrics,
        artifacts,
        defaults,
        fixtures,
        deprecations,
        checksum_sha256: String::new(),
    };
    registry.checksum_sha256 = sha256_json(&(
        &registry.domain_id,
        &registry.domain_version,
        &registry.schemas,
        &registry.stage_ids,
        &registry.tool_ids,
        &registry.governed_stage_ids,
        &registry.governed_tool_ids,
        &registry.stages,
        &registry.tools,
        &registry.metrics,
        &registry.artifacts,
        &registry.defaults,
        &registry.fixtures,
        &registry.deprecations,
        source_commit,
    ))?;
    Ok(registry)
}

pub fn build_domain_registry_bundle(
    domain_dir: &Path,
    source_commit: &str,
) -> Result<DomainRegistryReleaseBundle> {
    let workspace_root = domain_dir.parent().unwrap_or(domain_dir);
    let deprecations = load_deprecations(workspace_root)?;
    let mut domains = ["fastq", "bam", "vcf"]
        .into_iter()
        .map(|domain_id| build_single_domain_registry(domain_dir, domain_id, source_commit, &deprecations))
        .collect::<Result<Vec<_>>>()?;
    domains.sort_by(|left, right| left.domain_id.cmp(&right.domain_id));
    let bundle_checksum_sha256 = sha256_json(&domains)?;
    Ok(DomainRegistryReleaseBundle {
        schema_version: "bijux.domain.registry.release_bundle.v1".to_string(),
        source_commit: source_commit.to_string(),
        bundle_checksum_sha256,
        domains,
    })
}

pub fn domain_defaults_snapshot(
    bundle: &DomainRegistryReleaseBundle,
) -> Vec<CompiledDomainDefaultsSnapshot> {
    bundle
        .domains
        .iter()
        .map(|domain| CompiledDomainDefaultsSnapshot {
            domain_id: domain.domain_id.clone(),
            defaults: domain.defaults.clone(),
        })
        .collect()
}

pub fn domain_artifact_contract_snapshots(
    bundle: &DomainRegistryReleaseBundle,
) -> Vec<ArtifactContractSnapshot> {
    bundle
        .domains
        .iter()
        .map(|domain| ArtifactContractSnapshot {
            domain_id: domain.domain_id.clone(),
            artifacts: domain.artifacts.clone(),
            stage_outputs: stage_outputs_by_stage(&domain.stages),
        })
        .collect()
}

pub fn domain_metric_catalogs(bundle: &DomainRegistryReleaseBundle) -> Vec<DomainMetricCatalog> {
    bundle
        .domains
        .iter()
        .map(|domain| DomainMetricCatalog {
            domain_id: domain.domain_id.clone(),
            metrics: domain.metrics.clone(),
        })
        .collect()
}

pub fn domain_deprecation_catalogs(
    bundle: &DomainRegistryReleaseBundle,
) -> Vec<DomainDeprecationCatalog> {
    bundle
        .domains
        .iter()
        .map(|domain| DomainDeprecationCatalog {
            domain_id: domain.domain_id.clone(),
            deprecations: domain.deprecations.clone(),
        })
        .collect()
}

pub fn domain_invariant_catalogs(bundle: &DomainRegistryReleaseBundle) -> Vec<DomainInvariantCatalog> {
    bundle
        .domains
        .iter()
        .map(|domain| DomainInvariantCatalog {
            domain_id: domain.domain_id.clone(),
            stage_invariants: domain
                .stages
                .iter()
                .map(|stage| (stage.stage_id.clone(), stage.invariants.clone()))
                .collect(),
        })
        .collect()
}

pub fn write_domain_registry_bundle(
    configs_dir: &Path,
    bundle: &DomainRegistryReleaseBundle,
) -> Result<Vec<PathBuf>> {
    let registry_dir = configs_dir.join("ci").join("registry");
    ensure_dir(&registry_dir).with_context(|| format!("create {}", registry_dir.display()))?;

    let release_bundle_path = registry_dir.join("domain_registry_release_bundle.json");
    let defaults_path = registry_dir.join("domain_defaults_snapshot.json");
    let artifacts_path = registry_dir.join("domain_artifact_contract_snapshots.json");
    let metrics_path = registry_dir.join("domain_metric_catalogs.json");
    let deprecations_path = registry_dir.join("domain_deprecations_snapshot.json");
    let invariants_path = registry_dir.join("domain_invariant_catalogs.json");

    write_string(
        &release_bundle_path,
        &serde_json::to_string_pretty(bundle).context("serialize release bundle")?,
    )
    .with_context(|| format!("write {}", release_bundle_path.display()))?;
    write_string(
        &defaults_path,
        &serde_json::to_string_pretty(&domain_defaults_snapshot(bundle))
            .context("serialize defaults snapshot")?,
    )
    .with_context(|| format!("write {}", defaults_path.display()))?;
    write_string(
        &artifacts_path,
        &serde_json::to_string_pretty(&domain_artifact_contract_snapshots(bundle))
            .context("serialize artifact snapshots")?,
    )
    .with_context(|| format!("write {}", artifacts_path.display()))?;
    write_string(
        &metrics_path,
        &serde_json::to_string_pretty(&domain_metric_catalogs(bundle))
            .context("serialize metric catalogs")?,
    )
    .with_context(|| format!("write {}", metrics_path.display()))?;
    write_string(
        &deprecations_path,
        &serde_json::to_string_pretty(&domain_deprecation_catalogs(bundle))
            .context("serialize deprecation catalogs")?,
    )
    .with_context(|| format!("write {}", deprecations_path.display()))?;
    write_string(
        &invariants_path,
        &serde_json::to_string_pretty(&domain_invariant_catalogs(bundle))
            .context("serialize invariant catalogs")?,
    )
    .with_context(|| format!("write {}", invariants_path.display()))?;

    Ok(vec![
        release_bundle_path,
        defaults_path,
        artifacts_path,
        metrics_path,
        deprecations_path,
        invariants_path,
    ])
}

pub fn load_domain_registry_bundle(path: &Path) -> Result<DomainRegistryReleaseBundle> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub fn query_domain_registry_bundle(
    bundle: &DomainRegistryReleaseBundle,
    query: &DomainRegistryQuery,
) -> serde_json::Value {
    let domains = bundle
        .domains
        .iter()
        .filter(|domain| query.domain_id.as_deref().is_none_or(|wanted| domain.domain_id == wanted))
        .collect::<Vec<_>>();

    match query.kind {
        DomainRegistryQueryKind::Domains => {
            serde_json::json!(domains)
        }
        DomainRegistryQueryKind::Stages => serde_json::json!(
            domains
                .iter()
                .flat_map(|domain| domain.stages.iter())
                .filter(|stage| query.stage_id.as_deref().is_none_or(|wanted| stage.stage_id == wanted))
                .collect::<Vec<_>>()
        ),
        DomainRegistryQueryKind::Tools => serde_json::json!(
            domains
                .iter()
                .flat_map(|domain| domain.tools.iter())
                .filter(|tool| {
                    query.tool_id.as_deref().is_none_or(|wanted| tool.tool_id == wanted)
                        && query.stage_id.as_deref().is_none_or(|stage_id| {
                            tool.stage_ids.iter().any(|candidate| candidate == stage_id)
                                || tool.planned_stage_ids.iter().any(|candidate| candidate == stage_id)
                        })
                })
                .collect::<Vec<_>>()
        ),
        DomainRegistryQueryKind::Metrics => serde_json::json!(
            domains.iter().flat_map(|domain| domain.metrics.iter()).collect::<Vec<_>>()
        ),
        DomainRegistryQueryKind::Artifacts => serde_json::json!(
            domains.iter().flat_map(|domain| domain.artifacts.iter()).collect::<Vec<_>>()
        ),
        DomainRegistryQueryKind::Defaults => serde_json::json!(
            domains
                .iter()
                .flat_map(|domain| domain.defaults.iter())
                .filter(|default| query.stage_id.as_deref().is_none_or(|wanted| default.stage_id == wanted))
                .collect::<Vec<_>>()
        ),
        DomainRegistryQueryKind::Deprecations => serde_json::json!(
            domains
                .iter()
                .flat_map(|domain| domain.deprecations.iter())
                .collect::<Vec<_>>()
        ),
        DomainRegistryQueryKind::Fixtures => serde_json::json!(
            domains
                .iter()
                .flat_map(|domain| domain.fixtures.iter())
                .filter(|fixture| {
                    query.stage_id.as_deref().is_none_or(|wanted| fixture.stage_id == wanted)
                        && query.tool_id.as_deref().is_none_or(|wanted| fixture.tool_id == wanted)
                })
                .collect::<Vec<_>>()
        ),
    }
}
