//! Owner: bijux-dna-runner
//! Native execution of admitted container qualification specifications.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
const MAX_EVIDENCE_PUBLICATION_RESERVE: Duration = Duration::from_mins(2);

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
    let execution_budget = qualification_execution_budget(&spec.resources.walltime)?;
    let deadline = Instant::now()
        .checked_add(execution_budget)
        .context("qualification execution deadline overflow")?;
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
    let outcome = execute_contract(spec, &contract, &workspace, deadline)?;
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
    deadline: Instant,
) -> Result<NativeExecutionOutcome> {
    let started_at_utc = utc_now();
    let result_root = &workspace.result_root;
    let output_root = &workspace.output_root;
    let scratch = &workspace.scratch;
    let binds = qualification_binds(spec, contract, output_root, scratch);
    let checks = run_qualification_checks(contract, &binds, scratch, result_root, deadline)?;
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
    deadline: Instant,
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
                    deadline.checked_duration_since(Instant::now()).unwrap_or_default(),
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
    timeout: Duration,
) -> Result<Value> {
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
        .env("TZ", "UTC");
    run_supervised_check(command, arguments, result_root, name, timeout)
}

fn run_supervised_check(
    mut command: Command,
    arguments: &[String],
    result_root: &Path,
    name: &str,
    timeout: Duration,
) -> Result<Value> {
    let stdout_path = result_root.join(format!("{name}.stdout.log"));
    let stderr_path = result_root.join(format!("{name}.stderr.log"));
    let stdout = File::create(&stdout_path)
        .with_context(|| format!("create {name} qualification stdout"))?;
    let stderr = File::create(&stderr_path)
        .with_context(|| format!("create {name} qualification stderr"))?;
    command.stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    crate::process_supervision::isolate_process_group(&mut command);
    let mut child =
        command.spawn().with_context(|| format!("execute {name} qualification check"))?;
    let outcome = crate::process_supervision::wait_for_status(
        &mut child,
        timeout,
        &format!("{name} qualification check"),
    )?;
    let timeout_milliseconds = u64::try_from(timeout.as_millis()).unwrap_or(u64::MAX);
    Ok(json!({
        "argv": arguments,
        "exit_code": if outcome.timed_out { 124 } else { outcome.status.code().unwrap_or(128) },
        "timed_out": outcome.timed_out,
        "timeout_milliseconds": timeout_milliseconds,
        "stdout_sha256": bijux_dna_infra::hash_file_sha256(&stdout_path)?,
        "stderr_sha256": bijux_dna_infra::hash_file_sha256(&stderr_path)?,
    }))
}

fn qualification_execution_budget(walltime: &str) -> Result<Duration> {
    let mut fields = walltime.split(':');
    let hours = fields.next().context("qualification walltime omits hours")?;
    let minutes = fields.next().context("qualification walltime omits minutes")?;
    let seconds = fields.next().context("qualification walltime omits seconds")?;
    if fields.next().is_some() || hours.len() < 2 || hours.len() > 3 {
        bail!("qualification walltime must use HH:MM:SS");
    }
    let hours = hours.parse::<u64>().context("parse qualification walltime hours")?;
    let minutes = minutes.parse::<u64>().context("parse qualification walltime minutes")?;
    let seconds = seconds.parse::<u64>().context("parse qualification walltime seconds")?;
    if minutes > 59 || seconds > 59 {
        bail!("qualification walltime is outside supported bounds");
    }
    let total_seconds = hours
        .checked_mul(60 * 60)
        .and_then(|value| value.checked_add(minutes * 60))
        .and_then(|value| value.checked_add(seconds))
        .context("qualification walltime overflow")?;
    if total_seconds == 0 {
        bail!("qualification walltime must be positive");
    }
    let reserve_seconds =
        (total_seconds / 5).max(1).min(MAX_EVIDENCE_PUBLICATION_RESERVE.as_secs());
    Ok(Duration::from_secs(total_seconds - reserve_seconds))
}

fn archive_contract(source: &Path, target: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("inspect execution contract {}", source.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("execution contract is not a regular file: {}", source.display());
    }
    if let Ok(target_metadata) = fs::symlink_metadata(target) {
        if target_metadata.file_type().is_symlink() || !target_metadata.is_file() {
            bail!("archived execution contract is not a regular file: {}", target.display());
        }
        if metadata.dev() == target_metadata.dev() && metadata.ino() == target_metadata.ino() {
            return Ok(());
        }
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

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::time::Duration;

    use super::{archive_contract, qualification_execution_budget, run_supervised_check};

    #[test]
    fn archive_contract_preserves_an_already_archived_file() -> anyhow::Result<()> {
        let root = bijux_dna_infra::temp_dir("qualification-contract-self-archive")?;
        let contract = root.path().join("config-pipeline.yml");
        let expected = b"schema_version: bijux.container.pipeline.v2\n";
        std::fs::write(&contract, expected)?;

        archive_contract(&contract, &contract)?;

        assert_eq!(std::fs::read(contract)?, expected);
        Ok(())
    }

    #[test]
    fn qualification_budget_reserves_evidence_publication_time() -> anyhow::Result<()> {
        assert_eq!(qualification_execution_budget("00:30:00")?, Duration::from_mins(28));
        assert_eq!(qualification_execution_budget("00:05:00")?, Duration::from_mins(4));
        assert_eq!(qualification_execution_budget("00:00:01")?, Duration::ZERO);
        assert!(qualification_execution_budget("00:60:00").is_err());
        assert!(qualification_execution_budget("00:00:00").is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn supervised_qualification_check_records_nonzero_exit() -> anyhow::Result<()> {
        let root = bijux_dna_infra::temp_dir("qualification-nonzero-check")?;
        let arguments = vec!["-c".to_string(), "printf quota >&2; exit 28".to_string()];
        let mut command = Command::new("sh");
        command.args(&arguments);

        let result =
            run_supervised_check(command, &arguments, root.path(), "real", Duration::from_secs(2))?;

        assert_eq!(result["exit_code"], 28);
        assert_eq!(result["timed_out"], false);
        assert_eq!(std::fs::read_to_string(root.path().join("real.stderr.log"))?, "quota");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn supervised_qualification_check_kills_descendants_at_deadline() -> anyhow::Result<()> {
        let root = bijux_dna_infra::temp_dir("qualification-timeout-check")?;
        let arguments = vec!["-c".to_string(), "printf started >&2; tail -f /dev/null".to_string()];
        let mut command = Command::new("sh");
        command.args(&arguments);

        let result = run_supervised_check(
            command,
            &arguments,
            root.path(),
            "real",
            Duration::from_millis(50),
        )?;

        assert_eq!(result["exit_code"], 124);
        assert_eq!(result["timed_out"], true);
        assert_eq!(std::fs::read_to_string(root.path().join("real.stderr.log"))?, "started");
        Ok(())
    }
}
