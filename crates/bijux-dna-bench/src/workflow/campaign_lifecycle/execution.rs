//! Owner: bijux-dna-bench
//! Deterministic construction of executable specifications from admitted authorities.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_defined_interface_campaign_alignment, validate_stage_campaign_registry,
    validate_stage_interface_catalog,
};
use bijux_dna_bench_model::{
    benchmark_run_directory_name, validate_campaign_plan, validate_execution_spec,
    BenchmarkAttemptIdentity, BenchmarkCampaignPlan, BenchmarkExecutionAcceptance,
    BenchmarkExecutionCachePolicy, BenchmarkExecutionCommands, BenchmarkExecutionDeterminism,
    BenchmarkExecutionDownstreamProbe, BenchmarkExecutionImage, BenchmarkExecutionInput,
    BenchmarkExecutionInterface, BenchmarkExecutionMeasurement, BenchmarkExecutionOutputs,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionResources,
    BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock,
    BenchmarkExecutionSpec, BenchmarkLifecycleStatus, BenchmarkStageCampaignRegistry,
    BenchmarkStageInterfaceCatalog, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
};
use bijux_dna_environment::api::{ContainerCatalog, CurrentApptainerImage};

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionMaterial {
    pub operator: String,
    pub prepared_at_utc: String,
    pub repository_path: PathBuf,
    pub benchmark_root: PathBuf,
    pub source_contract_sha256: BTreeMap<String, String>,
    pub inputs: Vec<BenchmarkExecutionInput>,
    pub warmup_command: Vec<String>,
    pub measured_command: Vec<String>,
    pub negative_command: Vec<String>,
    pub parameters: BenchmarkExecutionParameterEvidence,
    pub required_files: Vec<String>,
    pub account: String,
    pub partition: String,
    pub memory_mb: u64,
    pub walltime: String,
    pub scratch_mb: u64,
    pub random_seed: Option<i64>,
    pub determinism: BenchmarkExecutionDeterminism,
    pub acceptance: BenchmarkExecutionAcceptance,
}

/// Build one source- and attempt-locked executable specification.
///
/// # Errors
/// Returns an error when campaign, interface, plan, catalog, image, material, or immutable result
/// identity disagree.
pub fn build_execution_spec(
    registry: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
    catalog: &ContainerCatalog,
    plan: &BenchmarkCampaignPlan,
    identity: &BenchmarkAttemptIdentity,
    image: &CurrentApptainerImage,
    material: BenchmarkExecutionMaterial,
) -> Result<BenchmarkExecutionSpec> {
    validate_stage_campaign_registry(registry).context("validate stage campaign registry")?;
    validate_stage_interface_catalog(interfaces).context("validate stage interface catalog")?;
    validate_defined_interface_campaign_alignment(interfaces, registry)
        .context("align stage interfaces with campaign declarations")?;
    validate_campaign_plan(plan).context("validate benchmark campaign plan")?;
    if catalog.schema_version != "bijux.container.catalog.v1" {
        bail!("container catalog schema is unsupported");
    }
    let attempt = plan
        .cycle
        .attempts
        .iter()
        .find(|attempt| attempt.identity == *identity)
        .context("campaign plan does not contain the requested attempt identity")?;
    if attempt.transitions.len() != 1
        || attempt.current_status() != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("execution specifications can only be built for wholly planned attempts");
    }
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == identity.run.run_id)
        .context("campaign registry does not contain the attempt run")?;
    if campaign.domain != identity.run.domain
        || campaign.stage_id != identity.run.stage_id
        || !campaign.tool_ids.contains(&identity.tool_id)
    {
        bail!("campaign declaration differs from the planned attempt identity");
    }
    let interface = interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == campaign.stage_id)
        .context("stage interface catalog does not contain the campaign stage")?;
    let adapter = interface
        .adapters
        .get(&identity.tool_id)
        .context("stage interface does not define the attempt tool adapter")?;
    let tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == identity.tool_id)
        .context("container catalog does not contain the attempt tool")?;
    if tool.support_disposition != "retained"
        || !tool.apptainer_supported
        || tool.runtime_network
        || !tool.stage_ids.contains(&campaign.stage_id)
    {
        bail!("container catalog does not retain the attempt tool for this offline stage");
    }
    if image.tool_id != identity.tool_id
        || image.repository_commit != plan.repository_commit
        || image.catalog_sha256
            != required_source_lock(&material.source_contract_sha256, "container_catalog")?
    {
        bail!("selected image identity differs from source-locked campaign authority");
    }
    if required_source_lock(&material.source_contract_sha256, "campaign_registry")?
        != plan.campaign_registry_sha256
    {
        bail!("execution material campaign registry digest differs from the plan");
    }
    required_source_lock(&material.source_contract_sha256, "image_inventory")?;
    required_source_lock(&material.source_contract_sha256, "stage_interface_registry")?;

    let threads = campaign
        .threads_by_tool
        .get(&identity.tool_id)
        .copied()
        .or(campaign.threads)
        .context("campaign does not allocate threads for the attempt tool")?;
    let result_root = material
        .benchmark_root
        .join(benchmark_run_directory_name(&identity.run)?)
        .join(&identity.tool_id)
        .join("attempts")
        .join(&identity.campaign_id)
        .join(&identity.attempt_id);
    let cache_policy = match campaign.cache_policy {
        bijux_dna_bench_model::BenchmarkCachePolicy::Cold => BenchmarkExecutionCachePolicy::Cold,
        bijux_dna_bench_model::BenchmarkCachePolicy::Warm => BenchmarkExecutionCachePolicy::Warm,
    };
    let mut acceptance = material.acceptance;
    acceptance.version_regex.clone_from(&tool.expected_version_regex);
    let spec = BenchmarkExecutionSpec {
        schema_version: BENCHMARK_EXECUTION_SPEC_SCHEMA_V1.to_string(),
        identity: identity.clone(),
        operator: material.operator,
        prepared_at_utc: material.prepared_at_utc,
        source: BenchmarkExecutionSourceLock {
            repository_commit: plan.repository_commit.clone(),
            repository_path: material.repository_path.display().to_string(),
            contract_sha256: material.source_contract_sha256,
        },
        image: BenchmarkExecutionImage {
            path: image.path.display().to_string(),
            sha256: image.sha256.clone(),
        },
        inputs: material.inputs,
        commands: BenchmarkExecutionCommands {
            version: shell_command(&tool.version_command),
            help: shell_command(&tool.help_command),
            warmup: material.warmup_command,
            measured: material.measured_command,
            negative: material.negative_command,
        },
        parameters: material.parameters,
        interface: BenchmarkExecutionInterface {
            accepted_input_contracts: interface.accepted_input_contracts.clone(),
            output_contract: interface.output_contract.clone(),
            normalized_result_contract: interface.normalized_result_contract.clone(),
            adapter: adapter.clone(),
            downstream_probe: BenchmarkExecutionDownstreamProbe {
                stage_id: interface.downstream_probe.consumer_stage_id.clone(),
                input_contract: interface.downstream_probe.input_contract.clone(),
                source: interface.downstream_probe.source.clone(),
                artifact: "repetitions/repetition-1.report-qc.json".to_string(),
            },
        },
        measurement: BenchmarkExecutionMeasurement {
            warmup_repetitions: campaign.warmup_repetitions,
            measured_repetitions: campaign.measured_repetitions,
            resource_file_template: "repetitions/repetition-{repetition}.resource.tsv".to_string(),
            runtime_scope: "containerized_stage_end_to_end".to_string(),
        },
        outputs: BenchmarkExecutionOutputs {
            result_root: result_root.display().to_string(),
            mount_path: "/result".to_string(),
            required_files: material.required_files,
        },
        resources: BenchmarkExecutionResources {
            account: material.account,
            partition: material.partition,
            cpus: threads,
            memory_mb: material.memory_mb,
            walltime: material.walltime,
            scratch_mb: material.scratch_mb,
            retry: BenchmarkExecutionRetryPolicy {
                max_attempts: 1,
                backoff_seconds: 0,
                retryable_exit_codes: Vec::new(),
            },
        },
        runtime: BenchmarkExecutionRuntimePolicy {
            locale: "C".to_string(),
            timezone: "UTC".to_string(),
            runtime_network: false,
            threads,
            random_seed: material.random_seed,
            cache_policy,
            determinism: material.determinism,
        },
        acceptance,
    };
    validate_execution_spec(&spec).context("validate constructed benchmark execution spec")?;
    Ok(spec)
}

fn required_source_lock<'a>(locks: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    locks.get(name).map(String::as_str).with_context(|| format!("missing source lock {name}"))
}

fn shell_command(command: &str) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command.to_string()]
}
