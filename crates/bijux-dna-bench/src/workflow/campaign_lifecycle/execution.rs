//! Owner: bijux-dna-bench
//! Deterministic construction of executable specifications from admitted authorities.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

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
use bijux_dna_containers::api::{ContainerCatalog, ContainerCatalogTool, CurrentApptainerImage};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::ids::{ArtifactId, StepId};
use bijux_dna_core::prelude::ContainerImageRefV1;
use bijux_dna_stage_contract::{
    build_binding_stage_plan, project_tool_parameters, BindingStagePlanRequest, StagePlanV1,
    ToolBinding, ToolBindingV1, ToolParameterProjectionRequest,
};

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
    pub stage_plan_units: Option<Vec<BenchmarkStagePlanUnitMaterial>>,
}

/// One binding-owned input path for an independently executable benchmark unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkStagePlanInputMaterial {
    pub binding_name: String,
    pub path: PathBuf,
    pub role: ArtifactRole,
}

/// One binding-owned output projected beneath a repetition execution-unit directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkStagePlanOutputMaterial {
    pub binding_name: String,
    pub relative_path: PathBuf,
    pub role: ArtifactRole,
}

/// Scientific, input-derived, and operator authority for one benchmark execution unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkStagePlanUnitMaterial {
    pub execution_unit_id: String,
    pub inputs: Vec<BenchmarkStagePlanInputMaterial>,
    pub outputs: Vec<BenchmarkStagePlanOutputMaterial>,
    pub scientific_defaults: BTreeMap<String, String>,
    pub input_derived: BTreeMap<String, String>,
    pub operator_overrides: BTreeMap<String, String>,
    pub immutable_parameters: BTreeSet<String>,
}

/// Build the complete measured-repetition by execution-unit stage-plan matrix.
///
/// The lifecycle supplies only material paths and parameter provenance. Stage/tool identity,
/// artifact contracts, command rendering, resources, executable, version, and image identity come
/// from the already admitted execution specification, canonical binding, and container catalog.
///
/// # Errors
/// Returns an error when an execution unit, path, binding, catalog tool, parameter projection, or
/// resource allocation differs from immutable execution authority.
pub fn build_benchmark_stage_plans(
    spec: &BenchmarkExecutionSpec,
    binding: &ToolBindingV1,
    catalog_tool: &ContainerCatalogTool,
    units: &[BenchmarkStagePlanUnitMaterial],
) -> Result<Vec<StagePlanV1>> {
    validate_execution_spec(spec).context("validate benchmark stage-plan execution authority")?;
    if binding.stage_id().as_str() != spec.identity.run.stage_id
        || binding.tool_id().as_str() != spec.identity.tool_id
        || catalog_tool.id != spec.identity.tool_id
        || catalog_tool.expected_bin.trim().is_empty()
        || catalog_tool.version.trim().is_empty()
        || !supports_offline_stage(catalog_tool, &spec.identity.run.stage_id)
    {
        bail!("benchmark stage-plan binding or catalog identity differs from execution authority");
    }
    if units.is_empty() {
        bail!("benchmark stage-plan material has no execution units");
    }

    let mut units = units.iter().collect::<Vec<_>>();
    units.sort_by(|left, right| left.execution_unit_id.cmp(&right.execution_unit_id));
    let mut unit_ids = BTreeSet::new();
    for unit in &units {
        validate_execution_unit_id(&unit.execution_unit_id)?;
        if !unit_ids.insert(unit.execution_unit_id.as_str()) {
            bail!("benchmark stage-plan execution unit is duplicated");
        }
        for output in &unit.outputs {
            validate_relative_output_path(&output.relative_path)?;
        }
    }

    let attempt_directory = Path::new(&spec.outputs.result_root);
    let container_result_root = Path::new(&spec.outputs.mount_path);
    let image = ContainerImageRefV1 {
        image: spec.image.path.clone(),
        digest: Some(format!("sha256:{}", spec.image.sha256)),
    };
    let mut plans =
        Vec::with_capacity(usize::try_from(spec.measurement.measured_repetitions)? * units.len());
    for repetition in 1..=spec.measurement.measured_repetitions {
        for unit in &units {
            let instance = format!("measured-repetition-{repetition}--{}", unit.execution_unit_id);
            let out_dir = attempt_directory.join("repetitions").join(&instance);
            let container_out_dir = container_result_root.join("repetitions").join(&instance);
            let effective = project_tool_parameters(&ToolParameterProjectionRequest {
                binding,
                scientific_defaults: &unit.scientific_defaults,
                input_derived: &unit.input_derived,
                operator_overrides: &unit.operator_overrides,
                immutable_parameters: &unit.immutable_parameters,
                allocated_cpus: spec.resources.cpus,
            })
            .context("project benchmark stage-plan parameters")?;
            let inputs = material_inputs(binding, &unit.inputs)?;
            let outputs = material_outputs(binding, &unit.outputs, &container_out_dir)?;
            plans.push(build_binding_stage_plan(BindingStagePlanRequest {
                binding,
                effective_parameters: &effective,
                image: image.clone(),
                executable: catalog_tool.expected_bin.clone(),
                tool_version: catalog_tool.version.clone(),
                stage_instance_id: Some(StepId::new(instance)),
                inputs,
                outputs,
                out_dir,
            })?);
        }
    }
    Ok(plans)
}

fn material_inputs(
    binding: &impl ToolBinding,
    materials: &[BenchmarkStagePlanInputMaterial],
) -> Result<Vec<ArtifactRef>> {
    materials
        .iter()
        .map(|material| {
            if !material.path.is_absolute()
                || material
                    .path
                    .components()
                    .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
                || !material.role.is_typed()
            {
                bail!("benchmark stage-plan input path or role is invalid");
            }
            let authority = binding
                .inputs()
                .iter()
                .find(|input| input.name == material.binding_name)
                .with_context(|| {
                    format!(
                        "benchmark stage-plan input `{}` is not declared by the binding",
                        material.binding_name
                    )
                })?;
            Ok(ArtifactRef {
                name: ArtifactId::new(&material.binding_name),
                path: material.path.clone(),
                role: material.role,
                artifact_contract_id: Some(authority.artifact_contract_id.clone()),
                optional: false,
            })
        })
        .collect()
}

fn material_outputs(
    binding: &impl ToolBinding,
    materials: &[BenchmarkStagePlanOutputMaterial],
    out_dir: &Path,
) -> Result<Vec<ArtifactRef>> {
    materials
        .iter()
        .map(|material| {
            if !material.role.is_typed() {
                bail!("benchmark stage-plan output role is invalid");
            }
            let authority = binding
                .outputs()
                .iter()
                .find(|output| output.name == material.binding_name)
                .with_context(|| {
                    format!(
                        "benchmark stage-plan output `{}` is not declared by the binding",
                        material.binding_name
                    )
                })?;
            Ok(ArtifactRef {
                name: ArtifactId::new(&material.binding_name),
                path: out_dir.join(&material.relative_path),
                role: material.role,
                artifact_contract_id: Some(authority.artifact_contract_id.clone()),
                optional: false,
            })
        })
        .collect()
}

fn validate_execution_unit_id(value: &str) -> Result<()> {
    if matches!(value, "" | "." | "..")
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        bail!("benchmark stage-plan execution unit identity is not canonical");
    }
    Ok(())
}

fn validate_relative_output_path(path: &Path) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::CurDir | Component::ParentDir | Component::RootDir)
        })
    {
        bail!("benchmark stage-plan output path escapes its execution-unit directory");
    }
    Ok(())
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
    mut material: BenchmarkExecutionMaterial,
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
    if !supports_offline_stage(tool, &campaign.stage_id) {
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
    let stage_plan_units = material
        .stage_plan_units
        .as_ref()
        .context("campaign execution material omits stage-plan units")?;
    material
        .required_files
        .extend(stage_plan_required_files(stage_plan_units, campaign.measured_repetitions)?);
    if material.required_files.iter().collect::<BTreeSet<_>>().len()
        != material.required_files.len()
    {
        bail!("benchmark required outputs collide with stage-plan output authority");
    }
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

fn stage_plan_required_files(
    units: &[BenchmarkStagePlanUnitMaterial],
    measured_repetitions: u32,
) -> Result<Vec<String>> {
    let mut units = units.iter().collect::<Vec<_>>();
    units.sort_by(|left, right| left.execution_unit_id.cmp(&right.execution_unit_id));
    let mut required = Vec::new();
    for repetition in 1..=measured_repetitions {
        for unit in &units {
            validate_execution_unit_id(&unit.execution_unit_id)?;
            let root =
                format!("repetitions/measured-repetition-{repetition}--{}", unit.execution_unit_id);
            for output in &unit.outputs {
                validate_relative_output_path(&output.relative_path)?;
                required.push(
                    Path::new(&root)
                        .join(&output.relative_path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
            required.push(format!("{root}/resource.tsv"));
        }
    }
    Ok(required)
}

fn required_source_lock<'a>(locks: &'a BTreeMap<String, String>, name: &str) -> Result<&'a str> {
    locks.get(name).map(String::as_str).with_context(|| format!("missing source lock {name}"))
}

fn shell_command(command: &str) -> Vec<String> {
    vec!["sh".to_string(), "-c".to_string(), command.to_string()]
}

fn supports_offline_stage(
    tool: &bijux_dna_containers::api::ContainerCatalogTool,
    stage_id: &str,
) -> bool {
    tool.is_product_supported()
        && ((tool.is_redistributable_image() && tool.apptainer_supported)
            || tool.permits_external_artifact())
        && !tool.runtime_network
        && tool.stage_ids.iter().any(|candidate| candidate == stage_id)
}

#[cfg(test)]
mod tests {
    use bijux_dna_containers::api::ContainerCatalog;

    use super::supports_offline_stage;

    #[test]
    fn licensed_external_tools_remain_executable_stage_candidates() -> anyhow::Result<()> {
        let catalog: ContainerCatalog =
            toml::from_str(include_str!("../../../../../containers/contracts/catalog.toml"))?;
        for (tool_id, stage_id) in [
            ("angsd", "bam.kinship"),
            ("fastuniq", "fastq.remove_duplicates"),
            ("impute5", "vcf.impute"),
            ("ngsbriggs", "bam.damage"),
        ] {
            let tool = catalog
                .tools
                .iter()
                .find(|tool| tool.id == tool_id)
                .ok_or_else(|| anyhow::anyhow!("catalog omits {tool_id}"))?;
            assert!(!tool.redistribution_permitted);
            assert!(supports_offline_stage(tool, stage_id));
        }
        Ok(())
    }
}
