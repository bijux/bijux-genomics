use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;

use super::model::{
    ContainerPipelineContract, ContainerPipelineV1, ContainerPipelineV2, PipelineAcceptanceMetric,
    PipelineAcceptanceV1, PipelineAcceptanceV2, PipelineCommandsV1, PipelineCommandsV2,
    PipelineDeterminism, PipelineEvidenceKind, PipelineExecutionV1, PipelineExecutionV2,
    PipelineInputKind, PipelineInputV1, PipelineInputV2, PipelineInterfaceEvidence,
    PipelineMeasurement, PipelineParameterEvidence, PipelineRepository, PipelineResourcesV1,
    PipelineResourcesV2, PipelineRunV1, PipelineRunV2, CONTAINER_PIPELINE_SCHEMA_V1,
    CONTAINER_PIPELINE_SCHEMA_V2,
};

const PARAMETER_EVIDENCE_SCHEMA_V1: &str = "bijux.benchmark.parameter_evidence.v1";
const STAGE_INTERFACE_EVIDENCE_SCHEMA_V1: &str = "bijux.benchmark.stage_interface_evidence.v1";
const REPEATED_MEASUREMENT_SCHEMA_V1: &str = "bijux.benchmark.repeated_measurement.v1";

#[derive(Deserialize)]
struct PipelineSchemaProbe {
    schema_version: String,
}

/// Load and structurally admit a legacy or current container pipeline declaration.
///
/// This admission is intentionally free of repository, image, data, and scheduler effects. Those
/// identities are reverified by the execution-site admission layer before a run can be submitted.
///
/// # Errors
/// Returns an error when the declaration cannot be read, parsed, or admitted.
pub fn load_container_pipeline(path: &Path) -> Result<ContainerPipelineContract> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read container pipeline {}", path.display()))?;
    parse_container_pipeline_document(&raw, &path.display().to_string())
}

pub(super) fn parse_container_pipeline_document(
    raw: &str,
    source: &str,
) -> Result<ContainerPipelineContract> {
    let probe: PipelineSchemaProbe = serde_yaml::from_str(raw)
        .with_context(|| format!("parse container pipeline schema {source}"))?;
    let contract = match probe.schema_version.as_str() {
        CONTAINER_PIPELINE_SCHEMA_V1 => ContainerPipelineContract::Legacy(Box::new(
            serde_yaml::from_str(raw)
                .with_context(|| format!("parse legacy container pipeline {source}"))?,
        )),
        CONTAINER_PIPELINE_SCHEMA_V2 => ContainerPipelineContract::Current(Box::new(
            serde_yaml::from_str(raw)
                .with_context(|| format!("parse current container pipeline {source}"))?,
        )),
        schema => bail!("unsupported container pipeline schema: {schema}"),
    };
    validate_container_pipeline(&contract)
        .with_context(|| format!("validate container pipeline {source}"))?;
    Ok(contract)
}

/// Validate version-specific pipeline declaration invariants.
///
/// # Errors
/// Returns an error when a declaration violates its schema-version semantics.
pub fn validate_container_pipeline(contract: &ContainerPipelineContract) -> Result<()> {
    match contract {
        ContainerPipelineContract::Legacy(pipeline) => validate_legacy_pipeline(pipeline),
        ContainerPipelineContract::Current(pipeline) => validate_current_pipeline(pipeline),
    }
}

fn validate_legacy_pipeline(pipeline: &ContainerPipelineV1) -> Result<()> {
    if pipeline.schema_version != CONTAINER_PIPELINE_SCHEMA_V1 {
        bail!("legacy pipeline has an unsupported schema");
    }
    validate_run_v1(&pipeline.run)?;
    validate_repository(&pipeline.repository)?;
    validate_image(&pipeline.image.path, &pipeline.image.sha256)?;
    if pipeline.data.inputs.is_empty() {
        bail!("pipeline data inputs must not be empty");
    }
    for (index, input) in pipeline.data.inputs.iter().enumerate() {
        validate_input_v1(input, index)?;
    }
    validate_commands_v1(&pipeline.commands)?;
    require_absolute(&pipeline.outputs.result_root, "outputs.result_root")?;
    for path in &pipeline.outputs.required_files {
        validate_relative_path(path, "required output")?;
    }
    validate_resources_v1(&pipeline.resources)?;
    validate_execution_v1(&pipeline.execution)?;
    validate_acceptance_v1(&pipeline.acceptance)
}

fn validate_current_pipeline(pipeline: &ContainerPipelineV2) -> Result<()> {
    if pipeline.schema_version != CONTAINER_PIPELINE_SCHEMA_V2 {
        bail!("current pipeline has an unsupported schema");
    }
    validate_run_v2(&pipeline.run)?;
    validate_repository(&pipeline.repository)?;
    validate_image(&pipeline.image.path, &pipeline.image.sha256)?;
    validate_current_inputs(pipeline)?;
    validate_commands_v2(&pipeline.commands, pipeline.measurement.as_ref())?;
    validate_current_outputs(pipeline)?;
    validate_resources_v2(&pipeline.resources)?;
    validate_execution_v2(&pipeline.execution, pipeline.resources.cpus)?;
    validate_dependencies(pipeline)?;
    validate_acceptance_v2(&pipeline.acceptance)?;
    validate_optional_current_contracts(pipeline)
}

fn validate_run_v1(run: &PipelineRunV1) -> Result<()> {
    validate_run_id(&run.id)?;
    validate_nonempty_fields(&[
        ("run.campaign", &run.campaign),
        ("run.corpus", &run.corpus),
        ("run.sample", &run.sample),
        ("run.domain", &run.domain),
        ("run.stage", &run.stage),
        ("run.tool", &run.tool),
        ("run.operator", &run.operator),
    ])
}

fn validate_run_v2(run: &PipelineRunV2) -> Result<()> {
    validate_run_id(&run.id)?;
    validate_nonempty_fields(&[
        ("run.campaign", &run.campaign),
        ("run.corpus", &run.corpus),
        ("run.sample", &run.sample),
        ("run.domain", &run.domain),
        ("run.stage", &run.stage),
        ("run.tool", &run.tool),
        ("run.operator", &run.operator),
        ("run.submitted_at_utc", &run.submitted_at_utc),
    ])?;
    validate_snake_identifier(&run.tool, "run.tool")?;
    let timestamp = Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")?;
    if !timestamp.is_match(&run.submitted_at_utc) {
        bail!("run.submitted_at_utc must be a UTC timestamp");
    }
    Ok(())
}

fn validate_repository(repository: &PipelineRepository) -> Result<()> {
    require_absolute(&repository.path, "repository.path")?;
    validate_hex(&repository.commit, 40, "repository.commit")?;
    if !repository.clean {
        bail!("repository.clean must be true");
    }
    Ok(())
}

fn validate_image(path: &Path, sha256: &str) -> Result<()> {
    require_absolute(path, "image.path")?;
    validate_hex(sha256, 64, "image.sha256")
}

fn validate_input_v1(input: &PipelineInputV1, index: usize) -> Result<()> {
    if input.id.trim().is_empty() {
        bail!("data input {index} ID must be non-empty");
    }
    require_absolute(&input.path, "data input path")?;
    validate_hex(&input.sha256, 64, "data input digest")
}

fn validate_current_inputs(pipeline: &ContainerPipelineV2) -> Result<()> {
    if pipeline.data.inputs.is_empty() {
        bail!("pipeline data inputs must not be empty");
    }
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    for (index, input) in pipeline.data.inputs.iter().enumerate() {
        validate_input_v2(input, index)?;
        if !ids.insert(input.id.as_str()) {
            bail!("data input ID is duplicated: {}", input.id);
        }
        if !mounts.insert(input.mount_path.as_path()) {
            bail!("data input mount path is duplicated: {}", input.mount_path.display());
        }
    }
    Ok(())
}

fn validate_input_v2(input: &PipelineInputV2, index: usize) -> Result<()> {
    validate_snake_identifier(&input.id, &format!("data input {index} ID"))?;
    require_absolute(&input.path, "data input path")?;
    if input.mount_path.parent() != Some(Path::new("/input"))
        || input.mount_path.file_name().and_then(|name| name.to_str()).is_none_or(|name| {
            name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        })
    {
        bail!("data input {index} mount path must be one safe filename under /input");
    }
    validate_hex(&input.sha256, 64, "data input digest")
}

fn validate_commands_v1(commands: &PipelineCommandsV1) -> Result<()> {
    for (label, argv) in [
        ("version", &commands.version),
        ("help", &commands.help),
        ("real", &commands.real),
        ("negative", &commands.negative),
    ] {
        validate_argv(argv, label)?;
    }
    Ok(())
}

fn validate_commands_v2(
    commands: &PipelineCommandsV2,
    measurement: Option<&PipelineMeasurement>,
) -> Result<()> {
    for (label, argv) in [
        ("version", &commands.version),
        ("help", &commands.help),
        ("real", &commands.real),
        ("negative", &commands.negative),
    ] {
        validate_argv(argv, label)?;
    }
    if let Some(measurement) = measurement {
        let warmup =
            commands.warmup.as_ref().context("measured pipeline requires a warmup command")?;
        validate_argv(warmup, "warmup")?;
        if !commands.real.iter().any(|token| token.contains("{repetition}")) {
            bail!("measured command must identify its repetition");
        }
        validate_measurement(measurement)?;
    } else if commands.warmup.is_some() {
        bail!("warmup command requires a measurement contract");
    }
    Ok(())
}

fn validate_current_outputs(pipeline: &ContainerPipelineV2) -> Result<()> {
    require_absolute(&pipeline.outputs.result_root, "outputs.result_root")?;
    if pipeline.outputs.mount_path != Path::new("/result") {
        bail!("outputs.mount_path must equal /result");
    }
    if pipeline.outputs.required_files.is_empty() {
        bail!("outputs.required_files must not be empty");
    }
    for path in &pipeline.outputs.required_files {
        validate_relative_path(path, "required output")?;
    }
    if pipeline.outputs.collection_rule != "required_files_and_metrics" {
        bail!("outputs.collection_rule must be required_files_and_metrics");
    }
    Ok(())
}

fn validate_resources_v1(resources: &PipelineResourcesV1) -> Result<()> {
    validate_resources(
        &resources.account,
        &resources.partition,
        resources.cpus,
        resources.memory_mb,
        &resources.walltime,
        resources.scratch_mb,
    )
}

fn validate_resources_v2(resources: &PipelineResourcesV2) -> Result<()> {
    validate_resources(
        &resources.account,
        &resources.partition,
        resources.cpus,
        resources.memory_mb,
        &resources.walltime,
        resources.scratch_mb,
    )?;
    if resources.retry.max_attempts != 1 || !resources.retry.retryable_exit_codes.is_empty() {
        bail!("pipeline retries require one attempt and no implicit retryable exit codes");
    }
    Ok(())
}

fn validate_resources(
    account: &str,
    partition: &str,
    cpus: u64,
    memory_mb: u64,
    walltime: &str,
    scratch_mb: u64,
) -> Result<()> {
    validate_nonempty_fields(&[
        ("resources.account", account),
        ("resources.partition", partition),
    ])?;
    if cpus == 0 || memory_mb == 0 || scratch_mb == 0 {
        bail!("pipeline resource quantities must be positive");
    }
    if !Regex::new(r"^[0-9]{2,3}:[0-5][0-9]:[0-5][0-9]$")?.is_match(walltime) {
        bail!("resources.walltime must use HH:MM:SS");
    }
    Ok(())
}

fn validate_execution_v1(execution: &PipelineExecutionV1) -> Result<()> {
    validate_frozen_environment(&execution.locale, &execution.timezone, execution.runtime_network)
}

fn validate_execution_v2(execution: &PipelineExecutionV2, cpus: u64) -> Result<()> {
    validate_frozen_environment(&execution.locale, &execution.timezone, execution.runtime_network)?;
    if execution.threads == 0 || execution.threads > cpus {
        bail!("execution.threads must fit within allocated CPUs");
    }
    if execution.determinism == PipelineDeterminism::Seeded && execution.random_seed.is_none() {
        bail!("seeded execution requires execution.random_seed");
    }
    Ok(())
}

fn validate_frozen_environment(locale: &str, timezone: &str, network: bool) -> Result<()> {
    if locale != "C" || timezone != "UTC" || network {
        bail!("pipeline execution must freeze locale C, timezone UTC, and disable network");
    }
    Ok(())
}

fn validate_dependencies(pipeline: &ContainerPipelineV2) -> Result<()> {
    let mut run_ids = BTreeSet::new();
    let mut dependency_digests = BTreeSet::new();
    for dependency in &pipeline.dependencies {
        validate_run_id(&dependency.run_id)?;
        if !run_ids.insert(dependency.run_id.as_str()) {
            bail!("dependency run ID is duplicated: {}", dependency.run_id);
        }
        if dependency.slurm_job_id.is_empty()
            || !dependency.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
        {
            bail!("dependency Slurm job ID must be numeric");
        }
        if dependency.required_outputs.is_empty() {
            bail!("dependency required outputs must not be empty");
        }
        for digest in &dependency.required_outputs {
            validate_hex(digest, 64, "dependency output digest")?;
            if !dependency_digests.insert(digest.as_str()) {
                bail!("dependency output digest is duplicated: {digest}");
            }
        }
    }
    let upstream_digests = pipeline
        .data
        .inputs
        .iter()
        .filter(|input| input.kind == Some(PipelineInputKind::UpstreamResult))
        .map(|input| input.sha256.as_str())
        .collect::<BTreeSet<_>>();
    if dependency_digests != upstream_digests {
        bail!("dependency output digests must exactly match upstream-result inputs");
    }
    Ok(())
}

fn validate_acceptance_v1(acceptance: &PipelineAcceptanceV1) -> Result<()> {
    validate_acceptance_base(&acceptance.version_regex, acceptance.negative_exit_nonzero)
}

fn validate_acceptance_v2(acceptance: &PipelineAcceptanceV2) -> Result<()> {
    validate_acceptance_base(&acceptance.version_regex, acceptance.negative_exit_nonzero)?;
    if acceptance.metrics.is_empty() {
        bail!("acceptance.metrics must not be empty");
    }
    let mut ids = BTreeSet::new();
    for metric in &acceptance.metrics {
        validate_acceptance_metric(metric)?;
        if !ids.insert(metric.id.as_str()) {
            bail!("acceptance metric ID is duplicated: {}", metric.id);
        }
    }
    Ok(())
}

fn validate_acceptance_base(version_regex: &str, negative_exit_nonzero: bool) -> Result<()> {
    if version_regex.trim().is_empty() {
        bail!("acceptance.version_regex must be non-empty");
    }
    Regex::new(version_regex).context("acceptance.version_regex is invalid")?;
    if !negative_exit_nonzero {
        bail!("acceptance must require a nonzero negative probe");
    }
    Ok(())
}

fn validate_acceptance_metric(metric: &PipelineAcceptanceMetric) -> Result<()> {
    validate_snake_identifier(&metric.id, "acceptance metric ID")?;
    validate_relative_path(&metric.path, "acceptance metric")?;
    if !metric.json_pointer.starts_with('/') {
        bail!("acceptance metric JSON pointer must start with /");
    }
    if !metric.expected.is_number() && !metric.expected.is_string() {
        bail!("acceptance metric expected value must be a number or string");
    }
    if metric.unit.trim().is_empty() {
        bail!("acceptance metric unit must be non-empty");
    }
    Ok(())
}

fn validate_optional_current_contracts(pipeline: &ContainerPipelineV2) -> Result<()> {
    let kind = pipeline.run.kind.unwrap_or(PipelineEvidenceKind::Qualification);
    if kind == PipelineEvidenceKind::Benchmark && pipeline.parameters.is_none() {
        bail!("benchmark pipeline requires parameter evidence");
    }
    if let Some(parameters) = &pipeline.parameters {
        validate_parameters(parameters)?;
    }
    if let Some(interface) = &pipeline.interface {
        validate_interface(interface)?;
    }
    if pipeline.measurement.is_some() && kind != PipelineEvidenceKind::Benchmark {
        bail!("measurement is valid only for benchmark pipelines");
    }
    Ok(())
}

fn validate_measurement(measurement: &PipelineMeasurement) -> Result<()> {
    if measurement.schema_version != REPEATED_MEASUREMENT_SCHEMA_V1
        || measurement.warmup_repetitions != 1
        || !(1..=10).contains(&measurement.measured_repetitions)
        || measurement.resource_file_template != "repetitions/repetition-{repetition}.resource.tsv"
        || measurement.runtime_scope != "containerized_stage_end_to_end"
    {
        bail!("measurement contract is unsupported");
    }
    Ok(())
}

fn validate_parameters(parameters: &PipelineParameterEvidence) -> Result<()> {
    if parameters.schema_version != PARAMETER_EVIDENCE_SCHEMA_V1 {
        bail!("parameter evidence schema is unsupported");
    }
    validate_nonempty_fields(&[
        ("parameter_schema_version", &parameters.parameter_schema_version),
        ("defaults_provider", &parameters.defaults_provider),
    ])?;
    if parameters.overrides_file != Path::new("config-pipeline-parameters.yml")
        || parameters.effective_file != Path::new("effective-parameters.json")
    {
        bail!("parameter evidence file names are unsupported");
    }
    validate_hex(&parameters.overrides_sha256, 64, "parameter overrides digest")?;
    validate_hex(&parameters.effective_sha256, 64, "effective parameters digest")
}

fn validate_interface(interface: &PipelineInterfaceEvidence) -> Result<()> {
    if interface.schema_version != STAGE_INTERFACE_EVIDENCE_SCHEMA_V1 {
        bail!("stage interface evidence schema is unsupported");
    }
    if interface.accepted_input_contracts.is_empty() {
        bail!("stage interface accepted inputs must not be empty");
    }
    validate_nonempty_fields(&[
        ("interface.output_contract", &interface.output_contract),
        ("interface.adapter", &interface.adapter),
        ("downstream_probe.stage_id", &interface.downstream_probe.stage_id),
        ("downstream_probe.input_contract", &interface.downstream_probe.input_contract),
        ("downstream_probe.source", &interface.downstream_probe.source),
    ])?;
    validate_relative_path(&interface.normalized_result_contract, "normalized result contract")?;
    validate_relative_path(&interface.downstream_probe.artifact, "downstream probe artifact")
}

fn validate_argv(argv: &[String], label: &str) -> Result<()> {
    if argv.is_empty() || argv.iter().any(String::is_empty) {
        bail!("commands.{label} must be a non-empty argv with non-empty tokens");
    }
    Ok(())
}

fn validate_nonempty_fields(fields: &[(&str, &str)]) -> Result<()> {
    for (label, value) in fields {
        if value.trim().is_empty() {
            bail!("{label} must be non-empty");
        }
    }
    Ok(())
}

fn validate_run_id(value: &str) -> Result<()> {
    if !value.strip_prefix("run-").is_some_and(|number| {
        number.len() >= 4
            && !number.starts_with('0')
            && number.bytes().all(|byte| byte.is_ascii_digit())
    }) {
        bail!("run identity is invalid: {value}");
    }
    Ok(())
}

fn validate_snake_identifier(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        bail!("{label} must use lowercase snake case");
    }
    Ok(())
}

fn validate_hex(value: &str, length: usize, label: &str) -> Result<()> {
    if value.len() != length || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} must be {length} lowercase hexadecimal characters");
    }
    if value.bytes().any(|byte| byte.is_ascii_uppercase()) {
        bail!("{label} must use lowercase hexadecimal characters");
    }
    Ok(())
}

fn require_absolute(path: &Path, label: &str) -> Result<()> {
    if !path.is_absolute() {
        bail!("{label} must be absolute");
    }
    Ok(())
}

fn validate_relative_path(path: &Path, label: &str) -> Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        bail!("{label} must be a safe relative path: {}", path.display());
    }
    Ok(())
}
