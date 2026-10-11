//! Owner: bijux-dna-bench
//! Deterministic completion of benchmark campaign, interface, and parameter declarations.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_executable_stage_campaign_registry, validate_interface_parameter_alignment,
    validate_pipeline_parameter_config, validate_portfolio_campaign_alignment,
    validate_stage_interface_catalog,
};
use bijux_dna_bench_model::{
    BenchmarkCachePolicy, BenchmarkCampaignMaterialDriver, BenchmarkComparisonMetricTolerance,
    BenchmarkEvidenceTier, BenchmarkExecutionRuntime, BenchmarkExecutionScheduler,
    BenchmarkPipelineParameterConfig, BenchmarkPortfolio, BenchmarkReportComparisonPresentation,
    BenchmarkReportDeterminismPresentation, BenchmarkReportMetricFormat,
    BenchmarkReportPerformanceColumn, BenchmarkReportPresentation, BenchmarkReportTitleClass,
    BenchmarkRunDefinition, BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec,
    BenchmarkStageDownstreamProbe, BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec,
    BenchmarkStageParameterConfig, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
};
use bijux_dna_stage_contract::{
    canonical_downstream_probes, canonical_tool_bindings, DownstreamProbe, ToolBinding,
    ToolBindingRegistry,
};
use serde::Deserialize;

use super::load_stage_campaign_registry;
use super::parameter_authority::{
    load_parameter_registry_entry, ParameterRegistryEntry, PIPELINE_PARAMETERS_PATH,
};
use super::suite_load::load_benchmark_portfolio_from_repo;

const CAMPAIGN_PATH: &str = "benchmarks/contracts/stage-campaigns.toml";
const INTERFACE_PATH: &str = "benchmarks/contracts/stage-interfaces.toml";
const PORTFOLIO_PATH: &str = "benchmarks/contracts/portfolio.toml";
const RUN_ROOT: &str = "benchmarks/runs";
const GENERATED_HEADER: &str =
    "# GENERATED FILE - reviewed material-driver declarations are preserved; do not edit derived entries\n";
const PERFORMANCE_METRICS: &[&str] = &[
    "wall_seconds",
    "cpu_seconds",
    "cpu_utilization_percent",
    "peak_rss_kib",
    "filesystem_input_operations",
    "filesystem_output_operations",
    "input_bytes",
    "output_bytes",
    "throughput_input_bytes_per_second",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkProtocolPortfolioWriteMode {
    Write,
    Check,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkProtocolPortfolio {
    pub campaigns: BenchmarkStageCampaignRegistry,
    pub interfaces: BenchmarkStageInterfaceCatalog,
    pub parameters: BenchmarkPipelineParameterConfig,
    pub reviewed_stage_count: usize,
    pub derived_stage_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum CatalogRole {
    Corpus,
    Reference,
    Database,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DomainStageDeclaration {
    #[serde(default)]
    extends: Option<String>,
    stage_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    benchmark_disposition: Option<String>,
    #[serde(default)]
    benchmark_disposition_reason: Option<String>,
    #[serde(default)]
    single_tool_justification: Option<String>,
    #[serde(default)]
    inputs: Vec<DomainStagePort>,
    #[serde(default)]
    outputs: Vec<DomainStagePort>,
    #[serde(default)]
    parameters: Vec<DomainStageParameter>,
    #[serde(default)]
    metrics: Vec<DomainStageMetric>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    mutates_fastq: Option<bool>,
    #[serde(default)]
    report_only: Option<bool>,
    #[serde(default)]
    may_change_read_count: Option<bool>,
    #[serde(default)]
    image_requirements: Option<serde_yaml::Value>,
    #[serde(default)]
    required_inputs: Vec<String>,
    #[serde(default)]
    required_outputs: Vec<String>,
    #[serde(default)]
    tool_capability_requirements: Vec<String>,
    #[serde(default)]
    invariants: Vec<String>,
    #[serde(default)]
    allowed_missingness: Vec<String>,
    #[serde(default)]
    compatible_tools: Vec<String>,
    #[serde(default)]
    defaults_source: Option<String>,
    #[serde(default)]
    assumptions: Vec<String>,
    #[serde(default)]
    bank_hooks: Vec<String>,
    #[serde(default)]
    metrics_schema: Option<String>,
    #[serde(default)]
    planned_out_of_scope: Vec<String>,
    #[serde(default)]
    schema_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DomainStagePort {
    name: String,
    data_type: String,
    cardinality: String,
    #[serde(default)]
    meaning: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DomainStageParameter {
    name: String,
    param_type: String,
    #[serde(default)]
    default: Option<serde_yaml::Value>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    meaning: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DomainStageMetric {
    name: String,
    #[serde(default)]
    meaning: Option<String>,
    #[serde(default)]
    scientific_threshold: Option<DomainScientificThreshold>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct DomainScientificThreshold {
    pass_direction: String,
    tolerance_kind: String,
    tolerance_value: f64,
    insufficiency_policy: String,
}

/// Derive the complete 71-stage benchmark protocol portfolio from permanent run identities,
/// canonical bindings, domain stage contracts, data roles, and reviewed executable overrides.
///
/// # Errors
/// Returns an error when any authority is missing, duplicated, or disagrees on stage/tool identity.
pub fn derive_benchmark_protocol_portfolio(
    repository_root: &Path,
) -> Result<BenchmarkProtocolPortfolio> {
    let portfolio =
        load_benchmark_portfolio_from_repo(&repository_root.join(PORTFOLIO_PATH), repository_root)?;
    let seed_campaigns = load_stage_campaign_registry(&repository_root.join(CAMPAIGN_PATH))?;
    let seed_interfaces = load_seed_interfaces(&repository_root.join(INTERFACE_PATH))?;
    let seed_parameters = load_parameter_config(repository_root)?;
    let definitions = load_run_definitions(repository_root)?;
    let bindings = canonical_tool_bindings()?;
    let probes = canonical_downstream_probes()?;
    let catalog_roles = catalog_role_authority(&seed_campaigns)?;

    let reviewed_stage_ids = seed_campaigns
        .campaigns
        .iter()
        .filter(|campaign| campaign.material_driver.is_some())
        .map(|campaign| campaign.stage_id.clone())
        .collect::<BTreeSet<_>>();
    let reviewed_campaigns = seed_campaigns
        .campaigns
        .iter()
        .filter(|campaign| reviewed_stage_ids.contains(&campaign.stage_id))
        .map(|campaign| (campaign.stage_id.clone(), campaign.clone()))
        .collect::<BTreeMap<_, _>>();
    let reviewed_interfaces = seed_interfaces
        .interfaces
        .iter()
        .filter(|interface| reviewed_stage_ids.contains(&interface.stage_id))
        .map(|interface| (interface.stage_id.clone(), interface.clone()))
        .collect::<BTreeMap<_, _>>();

    let profiles = portfolio
        .profiles
        .iter()
        .map(|profile| (profile.id.as_str(), profile))
        .collect::<BTreeMap<_, _>>();
    let corpora = portfolio
        .corpora
        .iter()
        .map(|corpus| (corpus.id.as_str(), corpus))
        .collect::<BTreeMap<_, _>>();
    let definitions = definitions
        .into_iter()
        .map(|definition| (definition.identity.stage_id.clone(), definition))
        .collect::<BTreeMap<_, _>>();

    let mut campaigns = Vec::with_capacity(portfolio.stage_runs.len());
    let mut interfaces = Vec::with_capacity(portfolio.stage_runs.len());
    let mut parameters = BTreeMap::new();
    let mut stages = portfolio.stage_runs.iter().collect::<Vec<_>>();
    stages.sort_by(|left, right| left.1.cmp(right.1));
    for (stage_id, run_id) in stages {
        let definition = definitions
            .get(stage_id)
            .with_context(|| format!("permanent run definition is absent for {stage_id}"))?;
        if &definition.identity.run_id != run_id {
            bail!("permanent run definition identity drift for {stage_id}");
        }
        let profile = profiles
            .get(definition.profile_id.as_str())
            .with_context(|| format!("scientific profile is absent for {stage_id}"))?;
        let stage = load_domain_stage(repository_root, stage_id)?;
        let stage_bindings = bindings_for_stage(&bindings, stage_id, &definition.tool_ids)?;
        let parameter_entry = load_parameter_registry_entry(repository_root, stage_id)?;

        let campaign = if let Some(reviewed) = reviewed_campaigns.get(stage_id) {
            reviewed.clone()
        } else {
            derive_campaign(definition, profile, &corpora, &stage, &stage_bindings, &catalog_roles)?
        };
        let interface = if let Some(reviewed) = reviewed_interfaces.get(stage_id) {
            reviewed.clone()
        } else {
            let probe = probes
                .probe_for_stage(stage_bindings[0].stage_id())
                .with_context(|| format!("canonical downstream probe is absent for {stage_id}"))?;
            derive_interface(definition, &stage, &stage_bindings, probe, &parameter_entry)?
        };
        let parameter = if reviewed_stage_ids.contains(stage_id) {
            seed_parameters
                .stages
                .get(stage_id)
                .with_context(|| {
                    format!("reviewed parameter declaration is absent for {stage_id}")
                })?
                .clone()
        } else {
            BenchmarkStageParameterConfig {
                parameter_schema_version: parameter_entry.schema_version.clone(),
                defaults_provider: interface.default_provider.clone(),
                overrides: BTreeMap::new(),
            }
        };
        campaigns.push(campaign);
        interfaces.push(interface);
        parameters.insert(stage_id.clone(), parameter);
    }

    let campaigns = BenchmarkStageCampaignRegistry {
        schema_version: BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1.to_string(),
        campaigns,
    };
    let interfaces = BenchmarkStageInterfaceCatalog {
        schema_version: BENCHMARK_STAGE_INTERFACES_SCHEMA_V1.to_string(),
        interfaces,
    };
    let parameters = BenchmarkPipelineParameterConfig {
        schema_version: BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1.to_string(),
        profile: "governed-benchmark-portfolio".to_string(),
        stages: parameters,
    };
    validate_complete_portfolio(repository_root, &portfolio, &campaigns, &interfaces, &parameters)?;
    Ok(BenchmarkProtocolPortfolio {
        campaigns,
        interfaces,
        parameters,
        reviewed_stage_count: reviewed_stage_ids.len(),
        derived_stage_count: portfolio.stage_runs.len() - reviewed_stage_ids.len(),
    })
}

/// Write or check the complete checked-in benchmark protocol declarations.
///
/// # Errors
/// Returns an error when derivation fails or checked-in bytes differ in check mode.
pub fn publish_checked_in_benchmark_protocol_portfolio(
    repository_root: &Path,
    mode: BenchmarkProtocolPortfolioWriteMode,
) -> Result<BenchmarkProtocolPortfolio> {
    let portfolio = derive_benchmark_protocol_portfolio(repository_root)?;
    let campaign_bytes = format!(
        "{GENERATED_HEADER}{}",
        toml::to_string_pretty(&portfolio.campaigns).context("render campaign portfolio")?
    )
    .into_bytes();
    let interface_bytes = format!(
        "{GENERATED_HEADER}{}",
        toml::to_string_pretty(&portfolio.interfaces).context("render interface portfolio")?
    )
    .into_bytes();
    let parameter_bytes = serde_json::to_vec_pretty(&portfolio.parameters)?;
    let mut parameter_bytes = parameter_bytes;
    parameter_bytes.push(b'\n');
    for (relative, bytes) in [
        (CAMPAIGN_PATH, campaign_bytes),
        (INTERFACE_PATH, interface_bytes),
        (PIPELINE_PARAMETERS_PATH, parameter_bytes),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkProtocolPortfolioWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, &bytes)
                    .with_context(|| format!("write benchmark protocol declaration {relative}"))?;
            }
            BenchmarkProtocolPortfolioWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read benchmark protocol declaration {relative}"))?;
                if current != bytes {
                    bail!("generated benchmark protocol declaration drifted: {relative}");
                }
            }
        }
    }
    Ok(portfolio)
}

fn derive_campaign(
    definition: &BenchmarkRunDefinition,
    profile: &bijux_dna_bench_model::BenchmarkScientificProfile,
    corpora: &BTreeMap<&str, &bijux_dna_bench_model::BenchmarkCorpusContract>,
    stage: &DomainStageDeclaration,
    bindings: &[&bijux_dna_stage_contract::ToolBindingV1],
    catalog_roles: &BTreeMap<String, CatalogRole>,
) -> Result<BenchmarkStageCampaignSpec> {
    let corpus_ids = definition
        .corpus_roles
        .iter()
        .flat_map(|role| role.corpus_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let catalog_ids = corpus_ids
        .iter()
        .flat_map(|corpus_id| {
            corpora
                .get(corpus_id.as_str())
                .into_iter()
                .flat_map(|corpus| corpus.catalog_ids.clone())
        })
        .collect::<BTreeSet<_>>();
    let requires_reference = bindings.iter().flat_map(|binding| binding.inputs()).any(|input| {
        input.required && input.source == bijux_dna_stage_contract::BindingInputSourceV1::Reference
    });
    let requires_database = bindings.iter().flat_map(|binding| binding.inputs()).any(|input| {
        input.required && input.source == bijux_dna_stage_contract::BindingInputSourceV1::Database
    });
    let corpus_catalog_ids: Vec<String> = catalog_ids
        .iter()
        .filter(|catalog_id| {
            catalog_roles.get(*catalog_id).copied().unwrap_or(CatalogRole::Corpus)
                == CatalogRole::Corpus
        })
        .cloned()
        .collect();
    let reference_catalog_ids: Vec<String> = if requires_reference {
        catalog_ids
            .iter()
            .filter(|catalog_id| catalog_roles.get(*catalog_id) == Some(&CatalogRole::Reference))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let database_catalog_ids: Vec<String> = if requires_database {
        catalog_ids
            .iter()
            .filter(|catalog_id| catalog_roles.get(*catalog_id) == Some(&CatalogRole::Database))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let mut limitations = corpus_ids
        .iter()
        .flat_map(|corpus_id| {
            corpora
                .get(corpus_id.as_str())
                .into_iter()
                .flat_map(|corpus| corpus.limitations.clone())
        })
        .chain(bindings.iter().flat_map(|binding| binding.limitations().iter().cloned()))
        .collect::<BTreeSet<_>>();
    limitations.insert(
        "Publication remains blocked until every declared corpus, artifact, scientific metric, and downstream probe is admitted for this exact protocol identity."
            .to_string(),
    );
    let input_size_keys = bindings
        .iter()
        .flat_map(|binding| binding.inputs())
        .map(|input| format!("{}_input_bytes", input.name))
        .collect::<BTreeSet<_>>();
    let scientific_metrics = stage.metrics.iter().map(|metric| metric.name.clone()).collect();
    let comparison_metrics = if bindings.len() > 1 {
        stage.metrics.iter().map(comparison_metric).collect()
    } else {
        Vec::new()
    };
    let threads_by_tool = bindings
        .iter()
        .map(|binding| (binding.tool_id().as_str().to_string(), binding.resources().threads.max(1)))
        .collect();
    Ok(BenchmarkStageCampaignSpec {
        run_id: definition.identity.run_id.clone(),
        slug: definition.identity.stage_id.replace(['.', '_'], "-"),
        domain: definition.identity.domain.clone(),
        stage_id: definition.identity.stage_id.clone(),
        material_driver: None::<BenchmarkCampaignMaterialDriver>,
        evidence_tier: if bindings.len() > 1 {
            BenchmarkEvidenceTier::Comparative
        } else {
            BenchmarkEvidenceTier::Qualification
        },
        profile_id: definition.profile_id.clone(),
        corpus_ids: corpus_ids.into_iter().collect(),
        scientific_question: format!(
            "How accurately, reproducibly, and efficiently do all product-supported {} implementations satisfy the governed {} contract?",
            definition.identity.stage_id, profile.id
        ),
        comparison_scope: format!(
            "{}; downstream criterion: {}",
            profile.interchangeability, profile.downstream_probe
        ),
        tool_ids: definition.tool_ids.clone(),
        corpus_catalog_ids,
        reference_catalog_ids,
        database_catalog_ids,
        input_size_keys: input_size_keys.into_iter().collect(),
        result_artifact: "normalized-result.json".to_string(),
        runtime: BenchmarkExecutionRuntime::Apptainer,
        scheduler: BenchmarkExecutionScheduler::Slurm,
        container_only: true,
        threads: None,
        threads_by_tool,
        warmup_repetitions: 1,
        measured_repetitions: 3,
        cache_policy: BenchmarkCachePolicy::Warm,
        result_root_template: "results/benchmarks/{run_id}-{slug}/{tool_id}".to_string(),
        accuracy_claim: profile.accuracy_strategy.clone(),
        interpretation: format!(
            "The campaign evaluates {} through the canonical normalized result and downstream probe. Tool agreement alone is not truth; claims are limited to admitted corpus roles and declared controls.",
            profile.accuracy_strategy
        ),
        performance_metrics: PERFORMANCE_METRICS.iter().map(|metric| (*metric).to_string()).collect(),
        scientific_metrics,
        comparison_metrics,
        limitations: limitations.into_iter().collect(),
    })
}

fn catalog_role_authority(
    campaigns: &BenchmarkStageCampaignRegistry,
) -> Result<BTreeMap<String, CatalogRole>> {
    let mut roles = BTreeMap::new();
    for campaign in campaigns.campaigns.iter().filter(|campaign| campaign.material_driver.is_some())
    {
        for (ids, role) in [
            (&campaign.corpus_catalog_ids, CatalogRole::Corpus),
            (&campaign.reference_catalog_ids, CatalogRole::Reference),
            (&campaign.database_catalog_ids, CatalogRole::Database),
        ] {
            for catalog_id in ids {
                if roles.insert(catalog_id.clone(), role).is_some_and(|existing| existing != role) {
                    bail!("reviewed campaigns assign conflicting roles to catalog {catalog_id}");
                }
            }
        }
    }
    Ok(roles)
}

fn comparison_metric(metric: &DomainStageMetric) -> BenchmarkComparisonMetricTolerance {
    let (absolute_tolerance, relative_tolerance) =
        metric.scientific_threshold.as_ref().map_or((0.0, 0.0), |threshold| {
            match threshold.tolerance_kind.as_str() {
                "relative_fraction" => (0.0, threshold.tolerance_value),
                "absolute_delta" => (threshold.tolerance_value, 0.0),
                _ => (0.0, 0.0),
            }
        });
    BenchmarkComparisonMetricTolerance {
        id: metric.name.clone(),
        absolute_tolerance,
        relative_tolerance,
    }
}

fn derive_interface(
    definition: &BenchmarkRunDefinition,
    stage: &DomainStageDeclaration,
    bindings: &[&bijux_dna_stage_contract::ToolBindingV1],
    probe: &bijux_dna_stage_contract::DownstreamProbeV1,
    parameter_entry: &ParameterRegistryEntry,
) -> Result<BenchmarkStageInterfaceSpec> {
    let accepted_input_contracts = bindings
        .iter()
        .flat_map(|binding| binding.inputs())
        .map(|input| input.artifact_contract_id.clone())
        .collect::<BTreeSet<_>>();
    let output_contracts = bindings
        .iter()
        .map(|binding| binding.normalized_output_contract_id().to_string())
        .collect::<BTreeSet<_>>();
    if output_contracts.len() != 1 {
        bail!(
            "canonical bindings disagree on normalized output for {}",
            definition.identity.stage_id
        );
    }
    let output_contract = output_contracts
        .into_iter()
        .next()
        .context("validated normalized output closure is empty")?;
    let adapters = bindings
        .iter()
        .map(|binding| {
            (binding.tool_id().as_str().to_string(), binding.normalizer_id().to_string())
        })
        .collect();
    let immutable_parameters = std::iter::once("schema_version".to_string())
        .chain(parameter_entry.parameters.iter().cloned())
        .chain(stage.parameters.iter().map(|parameter| parameter.name.clone()))
        .collect::<BTreeSet<_>>();
    let stage_source = domain_stage_path(&definition.identity.stage_id)?;
    Ok(BenchmarkStageInterfaceSpec {
        stage_id: definition.identity.stage_id.clone(),
        accepted_input_contracts: accepted_input_contracts.into_iter().collect(),
        output_contract,
        normalized_result_contract: format!(
            "benchmarks/schemas/{}-normalized-metrics.v1.json",
            definition.identity.domain
        ),
        parameter_schema_version: parameter_entry.schema_version.clone(),
        default_provider: format!(
            "{}#{}",
            if definition.identity.domain == "vcf" {
                "configs/ci/params/param_registry_downstream.toml"
            } else {
                "configs/ci/params/param_registry.toml"
            },
            definition.identity.stage_id
        ),
        parameter_config: PIPELINE_PARAMETERS_PATH.to_string(),
        planner_contract: None,
        operator_override_keys: Vec::new(),
        input_derived_parameters: Vec::new(),
        immutable_parameters: immutable_parameters.into_iter().collect(),
        effective_parameters_artifact: "effective-parameters.json".to_string(),
        tool_ids: definition.tool_ids.clone(),
        provenance_sources: vec![
            stage_source,
            "crates/bijux-dna-stage-contract/src/tool_binding/catalog.rs".to_string(),
            format!("benchmarks/schemas/{}-normalized-metrics.v1.json", definition.identity.domain),
        ],
        adapters,
        parameter_projections: BTreeMap::new(),
        downstream_probe: BenchmarkStageDownstreamProbe {
            consumer_stage_id: probe.probe_id().to_string(),
            input_contract: probe.input_contract_id().to_string(),
            source: "crates/bijux-dna-stage-contract/src/downstream_probe/registry.rs".to_string(),
            assertions: vec![
                "normalized stage and tool identity must match the selected binding".to_string(),
                "stage-owned scientific metrics must be present and semantically valid".to_string(),
                "declared output artifacts must be complete and identity-addressed".to_string(),
                "stage-owned scientific invariants must pass before admission".to_string(),
            ],
        },
        report_presentation: BenchmarkReportPresentation {
            report_file_stem: format!(
                "{}-{}",
                definition.identity.run_id,
                definition.identity.stage_id.replace(['.', '_'], "-")
            ),
            generated_source: "bijux-dna-bench::workflow::campaign-engine::reporting".to_string(),
            title_class: if bindings.len() > 1 {
                BenchmarkReportTitleClass::Comparative
            } else {
                BenchmarkReportTitleClass::Qualification
            },
            separate_accuracy_claim: true,
            include_comparison_closure: true,
            determinism: BenchmarkReportDeterminismPresentation::DetailedEvidence,
            comparison: BenchmarkReportComparisonPresentation::ToolMeans,
            performance_columns: vec![
                performance_column(
                    "wall_seconds",
                    "Wall mean (range), s",
                    BenchmarkReportMetricFormat::Seconds3,
                    true,
                ),
                performance_column(
                    "cpu_seconds",
                    "CPU mean, s",
                    BenchmarkReportMetricFormat::Seconds3,
                    false,
                ),
                performance_column(
                    "peak_rss_kib",
                    "Peak RSS mean, MiB",
                    BenchmarkReportMetricFormat::Mebibytes1,
                    false,
                ),
                performance_column(
                    "input_bytes",
                    "Input bytes",
                    BenchmarkReportMetricFormat::Number0,
                    false,
                ),
                performance_column(
                    "output_bytes",
                    "Output mean, bytes",
                    BenchmarkReportMetricFormat::Number1,
                    false,
                ),
            ],
        },
    })
}

fn performance_column(
    metric: &str,
    heading: &str,
    format: BenchmarkReportMetricFormat,
    include_range: bool,
) -> BenchmarkReportPerformanceColumn {
    BenchmarkReportPerformanceColumn {
        metric: metric.to_string(),
        heading: heading.to_string(),
        format,
        include_range,
    }
}

fn validate_complete_portfolio(
    repository_root: &Path,
    portfolio: &BenchmarkPortfolio,
    campaigns: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
    parameters: &BenchmarkPipelineParameterConfig,
) -> Result<()> {
    if campaigns.campaigns.len() != 71
        || interfaces.interfaces.len() != 71
        || parameters.stages.len() != 71
    {
        bail!(
            "benchmark protocol closure must contain exactly 71 campaigns, interfaces, and parameter declarations"
        );
    }
    validate_executable_stage_campaign_registry(campaigns)?;
    validate_portfolio_campaign_alignment(campaigns, portfolio)?;
    validate_stage_interface_catalog(interfaces)?;
    validate_pipeline_parameter_config(parameters)?;
    for interface in &interfaces.interfaces {
        validate_interface_parameter_alignment(interface, parameters)?;
        for relative in std::iter::once(interface.normalized_result_contract.as_str())
            .chain(interface.planner_contract.as_deref())
            .chain(std::iter::once(interface.downstream_probe.source.as_str()))
            .chain(interface.provenance_sources.iter().map(String::as_str))
        {
            if !repository_root.join(relative).is_file() {
                bail!(
                    "benchmark interface source is absent for {}: {relative}",
                    interface.stage_id
                );
            }
        }
    }
    let campaign_tools = campaigns
        .campaigns
        .iter()
        .map(|campaign| (campaign.stage_id.as_str(), campaign.tool_ids.as_slice()))
        .collect::<BTreeMap<_, _>>();
    for interface in &interfaces.interfaces {
        if campaign_tools.get(interface.stage_id.as_str()).copied()
            != Some(interface.tool_ids.as_slice())
        {
            bail!("campaign/interface tool closure differs for {}", interface.stage_id);
        }
    }
    Ok(())
}

fn bindings_for_stage<'a>(
    bindings: &'a ToolBindingRegistry,
    stage_id: &str,
    declared_tools: &[String],
) -> Result<Vec<&'a bijux_dna_stage_contract::ToolBindingV1>> {
    let stage_id = bijux_dna_core::ids::StageId::new(stage_id);
    let stage_bindings = bindings.bindings_for_stage(&stage_id);
    let actual_tools = stage_bindings
        .iter()
        .map(|binding| binding.tool_id().as_str().to_string())
        .collect::<Vec<_>>();
    if actual_tools != declared_tools {
        bail!(
            "permanent protocol tool closure differs from canonical bindings for {stage_id}: definition={declared_tools:?}, bindings={actual_tools:?}"
        );
    }
    if stage_bindings.is_empty() {
        bail!("canonical binding closure is empty for {stage_id}");
    }
    Ok(stage_bindings)
}

fn load_run_definitions(repository_root: &Path) -> Result<Vec<BenchmarkRunDefinition>> {
    let root = repository_root.join(RUN_ROOT);
    let mut paths = fs::read_dir(&root)
        .with_context(|| format!("read benchmark run root {}", root.display()))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut definitions = Vec::new();
    for path in paths {
        if !path.is_dir()
            || !path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("run-"))
        {
            continue;
        }
        let definition_path = path.join("definition.json");
        let bytes = fs::read(&definition_path)
            .with_context(|| format!("read benchmark definition {}", definition_path.display()))?;
        let definition: BenchmarkRunDefinition = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse benchmark definition {}", definition_path.display()))?;
        definitions.push(definition);
    }
    if definitions.len() != 71 {
        bail!("benchmark run definition closure must contain exactly 71 protocols");
    }
    Ok(definitions)
}

fn load_seed_interfaces(path: &Path) -> Result<BenchmarkStageInterfaceCatalog> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read seed interface catalog {}", path.display()))?;
    let catalog: BenchmarkStageInterfaceCatalog = toml::from_str(&raw)
        .with_context(|| format!("parse seed interface catalog {}", path.display()))?;
    validate_stage_interface_catalog(&catalog)?;
    Ok(catalog)
}

fn load_parameter_config(repository_root: &Path) -> Result<BenchmarkPipelineParameterConfig> {
    let path = repository_root.join(PIPELINE_PARAMETERS_PATH);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read benchmark parameter config {}", path.display()))?;
    let config: BenchmarkPipelineParameterConfig = serde_yaml::from_str(&raw)
        .with_context(|| format!("parse benchmark parameter config {}", path.display()))?;
    validate_pipeline_parameter_config(&config)?;
    Ok(config)
}

fn load_domain_stage(repository_root: &Path, stage_id: &str) -> Result<DomainStageDeclaration> {
    let relative = domain_stage_path(stage_id)?;
    let path = repository_root.join(&relative);
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read domain stage declaration {}", path.display()))?;
    let stage: DomainStageDeclaration = serde_yaml::from_str(&raw)
        .with_context(|| format!("parse domain stage declaration {}", path.display()))?;
    if stage.stage_id != stage_id || stage.metrics.is_empty() {
        bail!("domain stage declaration is incomplete for {stage_id}");
    }
    Ok(stage)
}

fn domain_stage_path(stage_id: &str) -> Result<String> {
    let (domain, stage) = stage_id
        .split_once('.')
        .with_context(|| format!("benchmark stage ID has no domain: {stage_id}"))?;
    if !matches!(domain, "fastq" | "bam" | "vcf")
        || !stage.chars().all(|character| character.is_ascii_lowercase() || character == '_')
    {
        bail!("benchmark stage ID is invalid: {stage_id}");
    }
    Ok(format!("domain/{domain}/stages/{stage}.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_authorities_derive_complete_protocol_portfolio() -> Result<()> {
        let repository_root =
            bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"));
        let portfolio = derive_benchmark_protocol_portfolio(&repository_root)?;

        assert_eq!(portfolio.campaigns.campaigns.len(), 71);
        assert_eq!(portfolio.interfaces.interfaces.len(), 71);
        assert_eq!(portfolio.parameters.stages.len(), 71);
        assert_eq!(portfolio.reviewed_stage_count, 13);
        assert_eq!(portfolio.derived_stage_count, 58);
        let licensed_pairs = portfolio
            .campaigns
            .campaigns
            .iter()
            .flat_map(|campaign| {
                campaign.tool_ids.iter().filter(move |tool| {
                    matches!(tool.as_str(), "angsd" | "fastuniq" | "impute5" | "ngsbriggs")
                })
            })
            .count();
        assert_eq!(licensed_pairs, 8);
        let role_assignments = portfolio
            .campaigns
            .campaigns
            .iter()
            .flat_map(|campaign| {
                [
                    (&campaign.corpus_catalog_ids, CatalogRole::Corpus),
                    (&campaign.reference_catalog_ids, CatalogRole::Reference),
                    (&campaign.database_catalog_ids, CatalogRole::Database),
                ]
                .into_iter()
                .flat_map(|(catalog_ids, role)| {
                    catalog_ids.iter().map(move |catalog_id| (catalog_id.clone(), role))
                })
            })
            .collect::<BTreeSet<_>>();
        let role_counts = role_assignments.into_iter().fold(
            BTreeMap::<String, BTreeSet<CatalogRole>>::new(),
            |mut counts, (catalog_id, role)| {
                counts.entry(catalog_id).or_default().insert(role);
                counts
            },
        );
        assert!(role_counts.values().all(|roles| roles.len() == 1));
        let correct_errors = portfolio
            .campaigns
            .campaigns
            .iter()
            .find(|campaign| campaign.stage_id == "fastq.correct_errors")
            .context("derived correction campaign is absent")?;
        assert!(!correct_errors.corpus_catalog_ids.contains(&"grch38-reference".to_string()));
        assert!(correct_errors.reference_catalog_ids.is_empty());
        let deplete_host = portfolio
            .campaigns
            .campaigns
            .iter()
            .find(|campaign| campaign.stage_id == "fastq.deplete_host")
            .context("derived host-depletion campaign is absent")?;
        assert_eq!(deplete_host.reference_catalog_ids, ["grch38-reference"]);
        let authenticity = portfolio
            .interfaces
            .interfaces
            .iter()
            .find(|interface| interface.stage_id == "bam.authenticity")
            .context("derived authenticity interface is absent")?;
        assert_eq!(
            authenticity.adapters.get("authenticct").map(String::as_str),
            Some("bijux_dna_planner_bam::tool_adapters::stages_adna::authenticity::plan")
        );
        Ok(())
    }
}
