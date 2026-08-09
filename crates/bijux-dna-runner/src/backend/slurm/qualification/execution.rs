//! Owner: bijux-dna-runner
//! Native execution of admitted container qualification specifications.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    load_container_governance, load_container_pipeline, validate_container_governance_host,
    CONTAINER_PIPELINE_SCHEMA_V2,
};
use bijux_dna_containers_qa::api::{
    evaluate_container_qualification, plan_container_qualification,
    ContainerQualificationAssessment, ContainerQualificationPlan, QualificationProbeObservation,
};
use serde_json::{json, Value};

use super::contracts::{
    ContainerQualificationExecution, ContainerQualificationSlurmSpec,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3, CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4,
    CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5,
};
use super::validation::{canonical_directory, validate_specification};

const EXECUTION_EVIDENCE_SCHEMA_V1: &str = "bijux.container.evidence.v1";
const EXECUTION_EVIDENCE_SCHEMA_V2: &str = "bijux.container.evidence.v2";
const ISOLATED_RUNTIME_HOSTNAME: &str = "bijux-runtime";

struct NativeExecutionWorkspace {
    result_root: PathBuf,
    output_root: PathBuf,
    scratch: PathBuf,
    hostname: String,
}

struct NativeExecutionOutcome {
    started_at_utc: String,
    checks: BTreeMap<String, Value>,
    assessment: ContainerQualificationAssessment,
}

/// Execute one supported qualification specification with the pinned Rust operator.
///
/// # Errors
/// Returns an error when immutable authority validation, isolated runtime execution, output
/// inspection, or no-clobber evidence publication fails.
pub fn execute_container_qualification(
    spec: &ContainerQualificationSlurmSpec,
) -> Result<ContainerQualificationExecution> {
    validate_specification(spec)?;
    if !matches!(
        spec.schema_version.as_str(),
        CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V3
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V4
            | CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5
    ) {
        bail!("native qualification execution requires a native scheduler specification");
    }
    let governance = load_container_governance(&spec.governance_path)?;
    let contract = plan_container_qualification(load_container_pipeline(&spec.pipeline_path)?)?;
    let workspace = prepare_execution_workspace(spec, &contract, &governance)?;
    let outcome = execute_contract(spec, &contract, &workspace)?;
    publish_execution_evidence(spec, &governance, &contract, &workspace, outcome)
}

fn prepare_execution_workspace(
    spec: &ContainerQualificationSlurmSpec,
    contract: &ContainerQualificationPlan,
    governance: &bijux_dna_containers::api::ContainerGovernance,
) -> Result<NativeExecutionWorkspace> {
    let hostname = hostname();
    if hostname.is_empty() {
        bail!("execution hostname is unavailable");
    }
    validate_container_governance_host(governance, &hostname)?;
    let scratch = governed_scratch(governance)?;
    if spec.schema_version != CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5 {
        let result_parent =
            spec.result_root.parent().context("qualification result root has no parent")?;
        fs::create_dir_all(result_parent).with_context(|| {
            format!("create qualification result parent {}", result_parent.display())
        })?;
        fs::create_dir(&spec.result_root).with_context(|| {
            format!("refuse to replace qualification result root {}", spec.result_root.display())
        })?;
    }
    let result_root =
        spec.result_root.canonicalize().context("canonicalize qualification result root")?;
    let output_root = if spec.schema_version == CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5 {
        canonical_directory(&result_root.join("outputs"), "qualification output root")?
    } else if contract.pipeline_schema == CONTAINER_PIPELINE_SCHEMA_V2 {
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
    contract: &ContainerQualificationPlan,
    workspace: &NativeExecutionWorkspace,
) -> Result<NativeExecutionOutcome> {
    let started_at_utc = utc_now();
    let result_root = &workspace.result_root;
    let output_root = &workspace.output_root;
    let scratch = &workspace.scratch;
    let binds = qualification_binds(spec, contract, output_root, scratch);
    let checks = run_qualification_checks(contract, &binds, scratch, result_root)?;
    let observations = probe_observations(contract, &checks, result_root)?;
    let assessment =
        evaluate_container_qualification(contract, &observations, result_root, output_root)?;
    Ok(NativeExecutionOutcome { started_at_utc, checks, assessment })
}

fn run_qualification_checks(
    contract: &ContainerQualificationPlan,
    binds: &[(PathBuf, PathBuf, &'static str)],
    scratch: &Path,
    result_root: &Path,
) -> Result<BTreeMap<String, Value>> {
    contract
        .probes
        .iter()
        .map(|probe| {
            let name = probe.kind.as_str();
            Ok((
                name.to_string(),
                run_check(
                    &contract.image_path,
                    &probe.arguments,
                    binds,
                    scratch,
                    result_root,
                    name,
                )?,
            ))
        })
        .collect()
}

fn probe_observations(
    contract: &ContainerQualificationPlan,
    checks: &BTreeMap<String, Value>,
    result_root: &Path,
) -> Result<Vec<QualificationProbeObservation>> {
    contract
        .probes
        .iter()
        .map(|probe| {
            let name = probe.kind.as_str();
            let exit_code = checks
                .get(name)
                .and_then(|record| record.get("exit_code"))
                .and_then(Value::as_i64)
                .context("qualification check omits its exit code")?;
            Ok(QualificationProbeObservation {
                kind: probe.kind,
                exit_code: i32::try_from(exit_code).context("qualification exit code overflows")?,
                stdout: fs::read_to_string(result_root.join(format!("{name}.stdout.log")))?,
                stderr: fs::read_to_string(result_root.join(format!("{name}.stderr.log")))?,
            })
        })
        .collect()
}

fn publish_execution_evidence(
    spec: &ContainerQualificationSlurmSpec,
    governance: &bijux_dna_containers::api::ContainerGovernance,
    contract: &ContainerQualificationPlan,
    workspace: &NativeExecutionWorkspace,
    outcome: NativeExecutionOutcome,
) -> Result<ContainerQualificationExecution> {
    let NativeExecutionOutcome { started_at_utc, checks, assessment } = outcome;
    let passed = assessment.passed();
    let failures = assessment.failure_messages();
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
        "failure_details": assessment.failures,
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
        "outputs": assessment.outputs,
        "metrics": assessment.metrics,
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
    contract: &ContainerQualificationPlan,
    output_root: &Path,
    scratch: &Path,
) -> Vec<(PathBuf, PathBuf, &'static str)> {
    contract
        .input_binds
        .iter()
        .map(|bind| (bind.source.clone(), bind.target.clone(), "ro"))
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
