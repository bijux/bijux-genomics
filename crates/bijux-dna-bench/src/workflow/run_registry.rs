//! Owner: bijux-dna-bench
//! Deterministic permanent benchmark run registry derivation and publication.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_benchmark_portfolio, validate_benchmark_run_registry,
    validate_defined_campaign_portfolio_alignment, validate_stage_campaign_registry,
    validate_stage_interface_catalog,
};
use bijux_dna_bench_model::{
    benchmark_run_registry_schema, BenchmarkPortfolio, BenchmarkRunNamespacePolicy,
    BenchmarkRunRegistry, BenchmarkRunRegistryRow, BenchmarkRunRegistryState,
    BenchmarkRunRegistrySummary, BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog,
};
use bijux_dna_containers::api::{load_container_catalog, ContainerCatalog};

use super::stage_coverage::runnable_stage_tools;
use super::{
    load_benchmark_portfolio, load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
};

const REGISTRY_JSON: &str = "benchmarks/contracts/run-registry.json";
const REGISTRY_CHECKSUM: &str = "benchmarks/contracts/run-registry.sha256";
const REGISTRY_MARKDOWN: &str = "benchmarks/reports/run-registry.md";
const REGISTRY_SCHEMA: &str = "benchmarks/contracts/schemas/run-registry.schema.json";
const INTRODUCED_AT_COMMIT: &str = "3d8daa382e46074fc21e486131b7bd334b7e943d";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkRunRegistryWriteMode {
    Write,
    Check,
}

/// Derive all permanent stage protocols from the portfolio and runnable catalog closure.
///
/// # Errors
/// Returns an error when source contracts drift, a stage lacks a profile, or lifecycle closure is
/// inconsistent with the available campaign and interface contracts.
pub fn derive_benchmark_run_registry(
    portfolio: &BenchmarkPortfolio,
    campaigns: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
    catalog: &ContainerCatalog,
    source_contract_sha256: BTreeMap<String, String>,
) -> Result<BenchmarkRunRegistry> {
    validate_benchmark_portfolio(portfolio).context("validate benchmark portfolio")?;
    validate_stage_campaign_registry(campaigns).context("validate benchmark campaigns")?;
    validate_defined_campaign_portfolio_alignment(campaigns, portfolio)
        .context("align campaigns with permanent reservations")?;
    validate_stage_interface_catalog(interfaces).context("validate benchmark interfaces")?;

    let runnable = runnable_stage_tools(catalog)?;
    if runnable.keys().collect::<Vec<_>>() != portfolio.stage_runs.keys().collect::<Vec<_>>() {
        bail!("benchmark run registry does not close over runnable stages");
    }
    let profiles = portfolio
        .profiles
        .iter()
        .map(|profile| (profile.id.as_str(), profile))
        .collect::<BTreeMap<_, _>>();
    let assignments = portfolio
        .assignments
        .iter()
        .flat_map(|assignment| {
            assignment
                .stage_ids
                .iter()
                .map(move |stage| (stage.as_str(), assignment.profile_id.as_str()))
        })
        .collect::<BTreeMap<_, _>>();
    let campaigns = campaigns
        .campaigns
        .iter()
        .map(|campaign| (campaign.stage_id.as_str(), campaign))
        .collect::<BTreeMap<_, _>>();
    let interfaces = interfaces
        .interfaces
        .iter()
        .map(|interface| (interface.stage_id.as_str(), interface))
        .collect::<BTreeMap<_, _>>();

    let mut runs = Vec::with_capacity(portfolio.stage_runs.len());
    for (stage_id, run_id) in &portfolio.stage_runs {
        let domain = stage_id
            .split_once('.')
            .map(|(domain, _)| domain)
            .context("benchmark stage lacks a domain prefix")?;
        let profile_id = assignments
            .get(stage_id.as_str())
            .context("benchmark stage lacks a profile assignment")?;
        let profile =
            profiles.get(profile_id).context("benchmark profile assignment is unresolved")?;
        let interface = interfaces.get(stage_id.as_str()).copied();
        let campaign = campaigns.get(stage_id.as_str()).copied();
        if interface.is_some() != campaign.is_some() {
            bail!("benchmark stage has only one half of campaign/interface closure: {stage_id}");
        }
        if let (Some(interface), Some(campaign)) = (interface, campaign) {
            if interface.tool_ids != campaign.tool_ids {
                bail!("benchmark stage campaign/interface tool closure drifted: {stage_id}");
            }
        }
        let tools = runnable
            .get(stage_id)
            .context("benchmark stage has no runnable tools")?
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let (state, state_reason) = if interface.is_some() {
            (BenchmarkRunRegistryState::Active, None)
        } else {
            (
                BenchmarkRunRegistryState::Reserved,
                Some("Permanent identity reserved; a closed stage interface and executable campaign are not yet defined.".to_string()),
            )
        };
        runs.push(BenchmarkRunRegistryRow {
            run_id: run_id.clone(),
            stage_id: stage_id.clone(),
            domain: domain.to_string(),
            title: format!("{} {} benchmark protocol", domain.to_ascii_uppercase(), stage_label(stage_id)),
            slug: stage_id.replace(['.', '_'], "-"),
            owner: "bijux-dna-bench".to_string(),
            purpose: format!(
                "Compare the runnable {stage_id} tools using {}; downstream substitution: {}.",
                profile.accuracy_strategy, profile.downstream_probe
            ),
            profile_id: (*profile_id).to_string(),
            tool_ids: tools,
            tool_exclusion_policy: "Include tools bound to this stage as retained packages or governed external artifacts; exclude every other catalog tool.".to_string(),
            required_corpus_roles: profile.required_corpus_roles.clone(),
            accepted_input_contracts: interface
                .map(|value| value.accepted_input_contracts.clone())
                .unwrap_or_default(),
            output_contract: interface.map(|value| value.output_contract.clone()),
            normalized_result_contract: interface
                .map(|value| value.normalized_result_contract.clone()),
            measurement_policy_id: "benchmark-measurement-v1".to_string(),
            acceptance_policy_id: format!("{}-acceptance-v1", profile.id),
            state,
            state_reason,
            supersedes_run_ids: Vec::new(),
            superseded_by_run_id: None,
            introduced_at_commit: INTRODUCED_AT_COMMIT.to_string(),
        });
    }
    runs.sort_by_key(|row| row.run_id.clone());
    let summary = BenchmarkRunRegistrySummary {
        run_count: runs.len(),
        active_count: runs
            .iter()
            .filter(|row| row.state == BenchmarkRunRegistryState::Active)
            .count(),
        reserved_count: runs
            .iter()
            .filter(|row| row.state == BenchmarkRunRegistryState::Reserved)
            .count(),
        retired_count: runs
            .iter()
            .filter(|row| row.state == BenchmarkRunRegistryState::Retired)
            .count(),
    };
    let registry = BenchmarkRunRegistry::v1(
        BenchmarkRunNamespacePolicy {
            prefix: "run-".to_string(),
            minimum: 2001,
            maximum: 2999,
            next_run_number: 2072,
            allocation_policy: "monotonic_only".to_string(),
            identity_policy: "existing_ids_immutable".to_string(),
        },
        source_contract_sha256,
        summary,
        runs,
    );
    validate_benchmark_run_registry(&registry)
        .context("validate derived benchmark run registry")?;
    Ok(registry)
}

/// Write or check the checked-in permanent run registry and its review surfaces.
///
/// # Errors
/// Returns an error when source contracts are invalid or generated outputs drift.
pub fn publish_checked_in_benchmark_run_registry(
    repository_root: &Path,
    mode: BenchmarkRunRegistryWriteMode,
) -> Result<BenchmarkRunRegistry> {
    let portfolio_path = repository_root.join("benchmarks/contracts/portfolio.toml");
    let campaign_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let interface_path = repository_root.join("benchmarks/contracts/stage-interfaces.toml");
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    let portfolio = load_benchmark_portfolio(&portfolio_path)?;
    let campaigns = load_stage_campaign_registry(&campaign_path)?;
    let interfaces = load_stage_interface_catalog_from_repo(&interface_path, repository_root)?;
    let catalog = load_container_catalog(repository_root)?;
    let mut source_contract_sha256 = BTreeMap::new();
    for (relative, path) in [
        ("benchmarks/contracts/portfolio.toml", portfolio_path),
        ("benchmarks/contracts/stage-campaigns.toml", campaign_path),
        ("benchmarks/contracts/stage-interfaces.toml", interface_path),
        ("containers/contracts/catalog.toml", catalog_path),
    ] {
        let bytes =
            fs::read(&path).with_context(|| format!("read run registry source {relative}"))?;
        source_contract_sha256
            .insert(relative.to_string(), bijux_dna_infra::hash_bytes_sha256(&bytes));
    }
    let registry = derive_benchmark_run_registry(
        &portfolio,
        &campaigns,
        &interfaces,
        &catalog,
        source_contract_sha256,
    )?;
    let json = serde_json::to_string_pretty(&registry)? + "\n";
    let digest = bijux_dna_infra::hash_bytes_sha256(json.as_bytes());
    let checksum = format!("{digest}  run-registry.json\n");
    let markdown = render_benchmark_run_registry_markdown(&registry, &digest);
    let schema = serde_json::to_string_pretty(&benchmark_run_registry_schema())? + "\n";
    for (relative, content) in [
        (REGISTRY_JSON, json.as_bytes()),
        (REGISTRY_CHECKSUM, checksum.as_bytes()),
        (REGISTRY_MARKDOWN, markdown.as_bytes()),
        (REGISTRY_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkRunRegistryWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write generated run registry {relative}"))?;
            }
            BenchmarkRunRegistryWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read generated run registry {relative}"))?;
                if current != content {
                    bail!("generated benchmark run registry drifted: {relative}");
                }
            }
        }
    }
    Ok(registry)
}

#[must_use]
pub fn render_benchmark_run_registry_markdown(
    registry: &BenchmarkRunRegistry,
    registry_sha256: &str,
) -> String {
    let summary = &registry.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::run_registry -->".to_string(),
        "# Permanent benchmark run registry".to_string(),
        String::new(),
        "Run IDs identify durable stage protocols, not attempts or publications. Existing IDs are immutable; additions allocate monotonically from the declared frontier.".to_string(),
        String::new(),
        format!("- Registry SHA-256: `{registry_sha256}`"),
        format!("- Registered protocols: **{}**", summary.run_count),
        format!("- Active protocols: **{}**", summary.active_count),
        format!("- Reserved protocols: **{}**", summary.reserved_count),
        format!("- Next allocation: **run-{:04}**", registry.namespace.next_run_number),
        String::new(),
        "| Run | Stage | Profile | Tools | State | Interface |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    for row in &registry.runs {
        lines.push(format!(
            "| `{}` | `{}` | `{}` | {} | `{}` | {} |",
            row.run_id,
            row.stage_id,
            row.profile_id,
            row.tool_ids.iter().map(|tool| format!("`{tool}`")).collect::<Vec<_>>().join(", "),
            match row.state {
                BenchmarkRunRegistryState::Active => "active",
                BenchmarkRunRegistryState::Reserved => "reserved",
                BenchmarkRunRegistryState::Retired => "retired",
            },
            row.output_contract.as_deref().map_or("-".to_string(), |value| format!("`{value}`")),
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn stage_label(stage_id: &str) -> String {
    stage_id.split_once('.').map_or(stage_id, |(_, stage)| stage).replace('_', " ")
}
