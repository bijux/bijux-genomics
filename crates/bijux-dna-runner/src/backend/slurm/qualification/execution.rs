//! Owner: bijux-dna-runner
//! Native execution of admitted container qualification specifications.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    load_container_governance, load_container_pipeline, validate_container_governance_host,
    ContainerPipelineContract, CONTAINER_PIPELINE_SCHEMA_V2,
};
use regex::RegexBuilder;
use serde_json::{json, Value};

use super::contracts::{
    ContainerQualificationExecution, ContainerQualificationSlurmSpec,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3,
};
use super::validation::validate_specification;

const EXECUTION_EVIDENCE_SCHEMA_V1: &str = "bijux.container.evidence.v1";
const EXECUTION_EVIDENCE_SCHEMA_V2: &str = "bijux.container.evidence.v2";
const ISOLATED_RUNTIME_HOSTNAME: &str = "bijux-runtime";

struct NativeExecutionContract {
    pipeline_schema: String,
    image_path: PathBuf,
    image_sha256: String,
    inputs: Value,
    input_binds: Vec<(PathBuf, PathBuf)>,
    version_command: Vec<String>,
    help_command: Vec<String>,
    real_command: Vec<String>,
    negative_command: Vec<String>,
    output_mount: PathBuf,
    required_files: Vec<PathBuf>,
    version_regex: String,
    metrics: Vec<NativeMetric>,
    execution: Value,
    dependencies: Value,
    parameters: Value,
    parameter_files: Vec<PathBuf>,
    interface: Value,
}

struct NativeMetric {
    id: String,
    path: PathBuf,
    json_pointer: String,
    operator: String,
    expected: Value,
    unit: String,
}

struct NativeExecutionWorkspace {
    result_root: PathBuf,
    output_root: PathBuf,
    scratch: PathBuf,
    hostname: String,
}

struct NativeExecutionOutcome {
    started_at_utc: String,
    checks: BTreeMap<String, Value>,
    failures: Vec<String>,
    outputs: Vec<Value>,
    metrics: Vec<Value>,
}

/// Execute one v3 qualification specification with the pinned Rust operator.
///
/// # Errors
/// Returns an error when immutable authority validation, isolated runtime execution, output
/// inspection, or no-clobber evidence publication fails.
pub fn execute_container_qualification(
    spec: &ContainerQualificationSlurmSpec,
) -> Result<ContainerQualificationExecution> {
    validate_specification(spec)?;
    if spec.schema_version != CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3 {
        bail!("native qualification execution requires a v3 scheduler specification");
    }
    let governance = load_container_governance(&spec.governance_path)?;
    let contract = native_contract(load_container_pipeline(&spec.pipeline_path)?)?;
    let workspace = prepare_execution_workspace(spec, &contract, &governance)?;
    let outcome = execute_contract(spec, &contract, &workspace)?;
    publish_execution_evidence(spec, &governance, &contract, &workspace, outcome)
}

fn prepare_execution_workspace(
    spec: &ContainerQualificationSlurmSpec,
    contract: &NativeExecutionContract,
    governance: &bijux_dna_containers::api::ContainerGovernance,
) -> Result<NativeExecutionWorkspace> {
    let hostname = hostname();
    if hostname.is_empty() {
        bail!("execution hostname is unavailable");
    }
    validate_container_governance_host(governance, &hostname)?;
    let scratch = governed_scratch(governance)?;
    let result_parent =
        spec.result_root.parent().context("qualification result root has no parent")?;
    fs::create_dir_all(result_parent).with_context(|| {
        format!("create qualification result parent {}", result_parent.display())
    })?;
    fs::create_dir(&spec.result_root).with_context(|| {
        format!("refuse to replace qualification result root {}", spec.result_root.display())
    })?;
    let result_root =
        spec.result_root.canonicalize().context("canonicalize qualification result root")?;
    let output_root = if contract.pipeline_schema == CONTAINER_PIPELINE_SCHEMA_V2 {
        let output = result_root.join("outputs");
        fs::create_dir(&output)?;
        output
    } else {
        result_root.clone()
    };
    archive_contract(&spec.governance_path, &result_root.join("config-governance.yml"))?;
    archive_contract(&spec.pipeline_path, &result_root.join("config-pipeline.yml"))?;
    archive_contract(&spec.credential_reference_path, &result_root.join("config-credential.yml"))?;
    for parameter_file in &contract.parameter_files {
        let name =
            parameter_file.file_name().context("parameter evidence path has no file name")?;
        archive_contract(parameter_file, &result_root.join(name))?;
    }
    Ok(NativeExecutionWorkspace { result_root, output_root, scratch, hostname })
}

fn execute_contract(
    spec: &ContainerQualificationSlurmSpec,
    contract: &NativeExecutionContract,
    workspace: &NativeExecutionWorkspace,
) -> Result<NativeExecutionOutcome> {
    let started_at_utc = utc_now();
    let result_root = &workspace.result_root;
    let output_root = &workspace.output_root;
    let scratch = &workspace.scratch;
    let binds = qualification_binds(spec, contract, output_root, scratch);
    let checks = run_qualification_checks(contract, &binds, scratch, result_root)?;
    let mut failures = assess_checks(contract, &checks, result_root)?;
    let outputs = inspect_required_outputs(contract, output_root, result_root, &mut failures)?;
    let metrics = evaluate_metrics(output_root, &contract.metrics, &mut failures)?;
    Ok(NativeExecutionOutcome { started_at_utc, checks, failures, outputs, metrics })
}

fn run_qualification_checks(
    contract: &NativeExecutionContract,
    binds: &[(PathBuf, PathBuf, &'static str)],
    scratch: &Path,
    result_root: &Path,
) -> Result<BTreeMap<String, Value>> {
    [
        ("version", contract.version_command.as_slice()),
        ("help", contract.help_command.as_slice()),
        ("real", contract.real_command.as_slice()),
        ("negative", contract.negative_command.as_slice()),
    ]
    .into_iter()
    .map(|(name, arguments)| {
        Ok((
            name.to_string(),
            run_check(&contract.image_path, arguments, binds, scratch, result_root, name)?,
        ))
    })
    .collect()
}

fn assess_checks(
    contract: &NativeExecutionContract,
    checks: &BTreeMap<String, Value>,
    result_root: &Path,
) -> Result<Vec<String>> {
    let mut failures = Vec::new();
    let exit_code = |name: &str| {
        checks
            .get(name)
            .and_then(|record| record.get("exit_code"))
            .and_then(Value::as_i64)
            .unwrap_or(i64::MAX)
    };
    if exit_code("version") != 0 {
        failures.push("version command failed".to_string());
    }
    let version_text = format!(
        "{}{}",
        fs::read_to_string(result_root.join("version.stdout.log")).unwrap_or_default(),
        fs::read_to_string(result_root.join("version.stderr.log")).unwrap_or_default()
    );
    if !RegexBuilder::new(&contract.version_regex)
        .case_insensitive(true)
        .build()?
        .is_match(version_text.trim_end())
    {
        failures.push("version output did not match the catalog contract".to_string());
    }
    if exit_code("help") != 0 {
        failures.push("help command failed".to_string());
    }
    if exit_code("real") != 0 {
        failures.push("real command failed".to_string());
    }
    if exit_code("negative") == 0 {
        failures.push("negative command unexpectedly succeeded".to_string());
    }
    Ok(failures)
}

fn inspect_required_outputs(
    contract: &NativeExecutionContract,
    output_root: &Path,
    result_root: &Path,
    failures: &mut Vec<String>,
) -> Result<Vec<Value>> {
    let mut outputs = Vec::new();
    for relative in &contract.required_files {
        let output = output_root.join(relative);
        match fs::symlink_metadata(&output) {
            Ok(metadata) if metadata.is_file() && metadata.len() > 0 => outputs.push(json!({
                "path": output.strip_prefix(result_root)?.to_string_lossy(),
                "size_bytes": metadata.len(),
                "sha256": bijux_dna_infra::hash_file_sha256(&output)?,
            })),
            _ => {
                failures
                    .push(format!("required output is absent or empty: {}", relative.display()));
            }
        }
    }
    Ok(outputs)
}

fn publish_execution_evidence(
    spec: &ContainerQualificationSlurmSpec,
    governance: &bijux_dna_containers::api::ContainerGovernance,
    contract: &NativeExecutionContract,
    workspace: &NativeExecutionWorkspace,
    outcome: NativeExecutionOutcome,
) -> Result<ContainerQualificationExecution> {
    let NativeExecutionOutcome { started_at_utc, checks, failures, outputs, metrics } = outcome;
    let passed = failures.is_empty();
    let image_size = fs::metadata(&contract.image_path)?.len();
    let execution_evidence = json!({
        "schema_version": if contract.pipeline_schema == CONTAINER_PIPELINE_SCHEMA_V2 {
            EXECUTION_EVIDENCE_SCHEMA_V2
        } else {
            EXECUTION_EVIDENCE_SCHEMA_V1
        },
        "pipeline_schema_version": contract.pipeline_schema,
        "run_id": spec.run_id,
        "evidence_kind": "qualification",
        "status": if passed { "passed" } else { "failed" },
        "failure_class": if passed { Value::Null } else { Value::String("qualification_execution".to_string()) },
        "failures": failures,
        "repository_commit": spec.repository_commit,
        "repository_path": spec.repository_path,
        "repository_clean": true,
        "governance_sha256": spec.governance_sha256,
        "site": {"id": governance.site.id, "host": workspace.hostname},
        "pipeline_sha256": spec.pipeline_sha256,
        "credential_reference_sha256": spec.credential_reference_sha256,
        "parameters": contract.parameters,
        "interface": contract.interface,
        "measurement": Value::Null,
        "image": {
            "path": contract.image_path,
            "sha256": contract.image_sha256,
            "size_bytes": image_size,
        },
        "inputs": contract.inputs,
        "dependencies": contract.dependencies,
        "checks": checks,
        "outputs": outputs,
        "metrics": metrics,
        "execution": contract.execution,
        "runtime": {
            "host": workspace.hostname,
            "slurm_job_id": std::env::var("SLURM_JOB_ID").unwrap_or_default(),
            "cpus": std::env::var("SLURM_CPUS_PER_TASK").unwrap_or_default(),
            "started_at_utc": started_at_utc,
            "finished_at_utc": utc_now(),
            "apptainer_version": apptainer_version(),
            "scratch_path": workspace.scratch,
        },
    });
    let evidence_path = workspace.result_root.join("evidence.json");
    bijux_dna_infra::atomic_create_json(&evidence_path, &execution_evidence)?;
    Ok(ContainerQualificationExecution {
        passed,
        evidence_sha256: bijux_dna_infra::hash_file_sha256(&evidence_path)?,
        evidence_path,
    })
}

fn native_contract(contract: ContainerPipelineContract) -> Result<NativeExecutionContract> {
    match contract {
        ContainerPipelineContract::Legacy(pipeline) => Ok(NativeExecutionContract {
            pipeline_schema: pipeline.schema_version,
            image_path: pipeline.image.path,
            image_sha256: pipeline.image.sha256,
            inputs: serde_json::to_value(&pipeline.data.inputs)?,
            input_binds: pipeline
                .data
                .inputs
                .iter()
                .map(|input| (input.path.clone(), input.path.clone()))
                .collect(),
            version_command: pipeline.commands.version,
            help_command: pipeline.commands.help,
            real_command: pipeline.commands.real,
            negative_command: pipeline.commands.negative,
            output_mount: pipeline.outputs.result_root,
            required_files: pipeline.outputs.required_files,
            version_regex: pipeline.acceptance.version_regex,
            metrics: Vec::new(),
            execution: serde_json::to_value(pipeline.execution)?,
            dependencies: json!([]),
            parameters: Value::Null,
            parameter_files: Vec::new(),
            interface: Value::Null,
        }),
        ContainerPipelineContract::Current(pipeline) => {
            if pipeline.measurement.is_some() {
                bail!("qualification execution cannot carry benchmark measurement authority");
            }
            let metrics = pipeline
                .acceptance
                .metrics
                .iter()
                .map(|metric| -> Result<NativeMetric> {
                    Ok(NativeMetric {
                        id: metric.id.clone(),
                        path: metric.path.clone(),
                        json_pointer: metric.json_pointer.clone(),
                        operator: serde_json::to_value(metric.operator)?
                            .as_str()
                            .context("serialize metric operator")?
                            .to_string(),
                        expected: metric.expected.clone(),
                        unit: metric.unit.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            let parameter_files =
                pipeline.parameters.as_ref().map_or_else(Vec::new, |parameters| {
                    vec![parameters.overrides_file.clone(), parameters.effective_file.clone()]
                });
            Ok(NativeExecutionContract {
                pipeline_schema: pipeline.schema_version,
                image_path: pipeline.image.path,
                image_sha256: pipeline.image.sha256,
                inputs: serde_json::to_value(&pipeline.data.inputs)?,
                input_binds: pipeline
                    .data
                    .inputs
                    .iter()
                    .map(|input| (input.path.clone(), input.mount_path.clone()))
                    .collect(),
                version_command: pipeline.commands.version,
                help_command: pipeline.commands.help,
                real_command: pipeline.commands.real,
                negative_command: pipeline.commands.negative,
                output_mount: pipeline.outputs.mount_path,
                required_files: pipeline.outputs.required_files,
                version_regex: pipeline.acceptance.version_regex,
                metrics,
                execution: serde_json::to_value(pipeline.execution)?,
                dependencies: serde_json::to_value(pipeline.dependencies)?,
                parameters: serde_json::to_value(pipeline.parameters)?,
                parameter_files,
                interface: serde_json::to_value(pipeline.interface)?,
            })
        }
    }
}

fn governed_scratch(
    governance: &bijux_dna_containers::api::ContainerGovernance,
) -> Result<PathBuf> {
    let root = if let Some(variable) = &governance.slurm.scratch_environment {
        PathBuf::from(
            std::env::var(variable)
                .with_context(|| format!("{variable} is absent from the execution environment"))?,
        )
    } else {
        governance.slurm.scratch_root.clone().context("governance lacks a scratch root")?
    }
    .canonicalize()
    .context("canonicalize governed scratch root")?;
    let scratch = PathBuf::from(std::env::var("TMPDIR").context("TMPDIR is absent")?)
        .canonicalize()
        .context("canonicalize job scratch")?;
    if scratch == root || !scratch.starts_with(&root) || !scratch.is_dir() {
        bail!("job scratch is outside the governed scratch root");
    }
    Ok(scratch)
}

fn qualification_binds(
    spec: &ContainerQualificationSlurmSpec,
    contract: &NativeExecutionContract,
    output_root: &Path,
    scratch: &Path,
) -> Vec<(PathBuf, PathBuf, &'static str)> {
    contract
        .input_binds
        .iter()
        .map(|(source, target)| (source.clone(), target.clone(), "ro"))
        .chain([
            (spec.repository_path.clone(), PathBuf::from("/repository"), "ro"),
            (output_root.to_path_buf(), contract.output_mount.clone(), "rw"),
            (scratch.to_path_buf(), PathBuf::from("/scratch"), "rw"),
        ])
        .collect()
}

fn run_check(
    image_path: &Path,
    arguments: &[String],
    binds: &[(PathBuf, PathBuf, &'static str)],
    scratch: &Path,
    result_root: &Path,
    name: &str,
) -> Result<Value> {
    let stdout_path = result_root.join(format!("{name}.stdout.log"));
    let stderr_path = result_root.join(format!("{name}.stderr.log"));
    let stdout = File::create(&stdout_path)?;
    let stderr = File::create(&stderr_path)?;
    let mut command = Command::new("apptainer");
    command.args([
        "exec",
        "--cleanenv",
        "--containall",
        "--hostname",
        ISOLATED_RUNTIME_HOSTNAME,
        "--net",
        "--network",
        "none",
    ]);
    for (source, target, mode) in binds {
        command.arg("--bind").arg(format!("{}:{}:{mode}", source.display(), target.display()));
    }
    command
        .arg(image_path)
        .args(arguments)
        .env_clear()
        .env("APPTAINERENV_TMPDIR", "/scratch")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("TMPDIR", scratch)
        .env("TZ", "UTC")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let status = command.status().with_context(|| format!("execute {name} qualification check"))?;
    Ok(json!({
        "argv": arguments,
        "exit_code": status.code().unwrap_or(128),
        "stdout_sha256": bijux_dna_infra::hash_file_sha256(&stdout_path)?,
        "stderr_sha256": bijux_dna_infra::hash_file_sha256(&stderr_path)?,
    }))
}

fn evaluate_metrics(
    output_root: &Path,
    metrics: &[NativeMetric],
    failures: &mut Vec<String>,
) -> Result<Vec<Value>> {
    metrics
        .iter()
        .map(|metric| -> Result<Value> {
            let evaluation = evaluate_metric(output_root, metric);
            let (observed, passed) = match evaluation {
                Ok((observed, true)) => (observed, true),
                Ok((observed, false)) => {
                    failures.push(format!(
                        "metric {} rejected {} {} {}",
                        metric.id, observed, metric.operator, metric.expected
                    ));
                    (observed, false)
                }
                Err(error) => {
                    failures.push(format!("metric {} cannot be evaluated: {error:#}", metric.id));
                    (Value::Null, false)
                }
            };
            Ok(json!({
                "id": metric.id,
                "path": metric.path,
                "json_pointer": metric.json_pointer,
                "operator": metric.operator,
                "expected": metric.expected,
                "observed": observed,
                "unit": metric.unit,
                "status": if passed { "passed" } else { "failed" },
            }))
        })
        .collect()
}

fn evaluate_metric(output_root: &Path, metric: &NativeMetric) -> Result<(Value, bool)> {
    let source = output_root.join(&metric.path);
    let metadata = fs::symlink_metadata(&source)
        .with_context(|| format!("inspect metric source {}", source.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("metric source is not a regular file: {}", source.display());
    }
    let payload: Value =
        serde_json::from_slice(&bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)?)?;
    let observed = payload
        .pointer(&metric.json_pointer)
        .with_context(|| format!("metric JSON pointer does not resolve: {}", metric.id))?
        .clone();
    let passed = compare_metric(&observed, &metric.operator, &metric.expected)?;
    Ok((observed, passed))
}

fn compare_metric(observed: &Value, operator: &str, expected: &Value) -> Result<bool> {
    if operator == "eq" {
        return Ok(observed == expected);
    }
    let observed = observed.as_f64().context("ordered metric observation is not numeric")?;
    let expected = expected.as_f64().context("ordered metric expectation is not numeric")?;
    match operator {
        "ge" => Ok(observed >= expected),
        "gt" => Ok(observed > expected),
        "le" => Ok(observed <= expected),
        "lt" => Ok(observed < expected),
        _ => bail!("metric operator is unsupported: {operator}"),
    }
}

fn archive_contract(source: &Path, target: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("inspect execution contract {}", source.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("execution contract is not a regular file: {}", source.display());
    }
    fs::copy(source, target)
        .with_context(|| format!("archive execution contract {}", source.display()))?;
    Ok(())
}

fn hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

fn apptainer_version() -> String {
    Command::new("apptainer")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_default()
}

fn utc_now() -> String {
    bijux_dna_runtime::run_layout::now_string()
}
