use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use wait_timeout::ChildExt as _;

use bijux_dna_containers::api::{
    admit_security_probe_baseline, clean_repository_head_commit, derive_apptainer_tool_contract,
    load_container_catalog, load_image_inventory_publication,
    resolve_catalog_locked_apptainer_image, resolve_current_apptainer_contract_commit,
    validate_container_inventory_catalog, validate_container_recipe_evidence_chain,
    verify_grype_database, verify_security_scanner_toolchain, AdmittedSecurityProbePublication,
    ContainerToolContract, SecurityFilesystemKind, SecurityFilesystemObservation,
    SecurityProbeArtifact, SecurityProbeBaselineAuthority, SecurityProbePublication,
    SecurityProbeStatus, SecurityRuntimeProbe, SecurityRuntimeReport, SecurityStructuralReport,
    ToolSecurityProbe, SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
};

const NON_ROOT_PROBE_SCRIPT: &str = "uid=$(id -u); printf '%s\\n' \"$uid\"; test \"$uid\" -ne 0";

#[derive(Debug, Clone)]
pub struct SecurityVerificationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub inventory_path: PathBuf,
    pub apptainer_executable: PathBuf,
    pub scanner_contract_path: PathBuf,
    pub scanner_root: PathBuf,
    pub database_contract_path: PathBuf,
    pub database_root: PathBuf,
    pub reused_publication_path: Option<PathBuf>,
    pub evidence_root: PathBuf,
    pub worker_count: usize,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct SecurityVerificationResult {
    pub publication: SecurityProbePublication,
    pub collected: usize,
    pub failed: usize,
    pub executed: usize,
    pub reused: usize,
}

#[derive(Debug, Clone)]
struct ToolTask {
    tool_id: String,
    version: String,
    image_path: PathBuf,
    image_sha256: String,
    contract: ContainerToolContract,
}

/// Collect complete raw security probes for every current retained image.
///
/// Collection status never implies a policy pass. Scientific and release consumers must admit a
/// separate policy-owned security publication derived from these exact artifacts.
///
/// # Errors
/// Returns an error for authority drift, invalid scanner/database evidence, unsafe output paths,
/// worker failure, or conflicting immutable publications.
pub fn execute_security_verification(
    request: &SecurityVerificationRequest,
    publication_path: &Path,
) -> Result<SecurityVerificationResult> {
    validate_request(request, publication_path)?;
    let commit = clean_repository_head_commit(&request.repository_root)?;
    let inventory = load_image_inventory_publication(&request.inventory_path)?;
    let catalog = load_container_catalog(&request.repository_root)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &request.repository_root.join("containers/contracts/catalog.toml"),
    )?;
    let inventory_sha256 = bijux_dna_infra::hash_file_sha256(&request.inventory_path)?;
    let scanner_completion_sha256 =
        bijux_dna_infra::hash_file_sha256(&request.scanner_root.join("completion.json"))?;
    let database_completion_sha256 =
        bijux_dna_infra::hash_file_sha256(&request.database_root.join("completion.json"))?;
    validate_container_inventory_catalog(&catalog_sha256, &catalog, &inventory)?;
    validate_container_recipe_evidence_chain(
        &request.repository_root,
        &commit,
        &catalog,
        &inventory,
    )?;
    verify_security_scanner_toolchain(&request.scanner_contract_path, &request.scanner_root)?;
    verify_grype_database(
        &request.database_contract_path,
        &request.scanner_root,
        &request.database_root,
    )?;
    let baseline_authority = SecurityProbeBaselineAuthority {
        catalog_sha256: catalog_sha256.clone(),
        scanner_completion_sha256: scanner_completion_sha256.clone(),
        database_completion_sha256: database_completion_sha256.clone(),
    };
    let reused = request
        .reused_publication_path
        .as_ref()
        .map(|path| admit_security_probe_baseline(path, &request.study_root, &baseline_authority))
        .transpose()?;
    let tasks = catalog
        .tools
        .iter()
        .filter(|tool| tool.is_redistributable_image() && tool.apptainer_supported)
        .map(|tool| {
            let image = resolve_catalog_locked_apptainer_image(
                &inventory,
                &catalog_sha256,
                &catalog,
                &commit,
                &request.study_root,
                &tool.id,
            )?;
            let contract_commit = resolve_current_apptainer_contract_commit(
                &request.repository_root,
                &commit,
                &inventory,
                tool,
            )?;
            let contract = derive_apptainer_tool_contract(
                &request.repository_root,
                &tool.id,
                &contract_commit,
                &catalog.apptainer_architecture,
            )?;
            Ok(ToolTask {
                tool_id: tool.id.clone(),
                version: tool.version.clone(),
                image_path: image.path,
                image_sha256: image.sha256,
                contract,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if tasks.is_empty() {
        bail!("security verification has no retained Apptainer images");
    }
    bijux_dna_infra::ensure_dir(&request.evidence_root)?;
    let execution = execute_tasks(request, &tasks, reused.as_ref())?;
    let collected = execution
        .tools
        .values()
        .filter(|tool| tool.status == SecurityProbeStatus::Collected)
        .count();
    let publication = SecurityProbePublication {
        schema_version: SECURITY_PROBE_PUBLICATION_SCHEMA_V1.to_string(),
        repository_commit: commit,
        catalog_sha256,
        inventory_sha256,
        scanner_completion_sha256,
        database_completion_sha256,
        reused_publication_sha256: reused.map(|publication| publication.publication_sha256),
        tools: execution.tools,
    };
    ensure_immutable_json(publication_path, &publication)?;
    let failed = tasks.len() - collected;
    Ok(SecurityVerificationResult {
        publication,
        collected,
        failed,
        executed: execution.executed,
        reused: execution.reused,
    })
}

struct SecurityTaskResults {
    tools: BTreeMap<String, ToolSecurityProbe>,
    executed: usize,
    reused: usize,
}

fn execute_tasks(
    request: &SecurityVerificationRequest,
    tasks: &[ToolTask],
    reused: Option<&AdmittedSecurityProbePublication>,
) -> Result<SecurityTaskResults> {
    let next = AtomicUsize::new(0);
    let executed = AtomicUsize::new(0);
    let reused_count = AtomicUsize::new(0);
    let results = Mutex::new(BTreeMap::new());
    std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::new();
        for _ in 0..request.worker_count.min(tasks.len()) {
            handles.push(scope.spawn(|| -> Result<()> {
                while let Some(task) = tasks.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let (result, was_reused) = execute_or_resume_tool(request, task, reused)?;
                    if was_reused {
                        reused_count.fetch_add(1, Ordering::Relaxed);
                    } else {
                        executed.fetch_add(1, Ordering::Relaxed);
                    }
                    results
                        .lock()
                        .map_err(|_| anyhow::anyhow!("security result lock poisoned"))?
                        .insert(task.tool_id.clone(), result);
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle.join().map_err(|_| anyhow::anyhow!("security worker panicked"))??;
        }
        Ok(())
    })?;
    Ok(SecurityTaskResults {
        tools: results
            .into_inner()
            .map_err(|_| anyhow::anyhow!("security result lock poisoned"))?,
        executed: executed.load(Ordering::Relaxed),
        reused: reused_count.load(Ordering::Relaxed),
    })
}

fn execute_or_resume_tool(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    reused: Option<&AdmittedSecurityProbePublication>,
) -> Result<(ToolSecurityProbe, bool)> {
    let tool_root = request.evidence_root.join("tools").join(&task.tool_id);
    let result_path = tool_root.join("result.json");
    if result_path.is_file() {
        let result: ToolSecurityProbe = serde_json::from_slice(&fs::read(&result_path)?)?;
        validate_resumed_result(request, task, &result)?;
        return Ok((result, false));
    }
    if let Some(previous) = reusable_baseline_tool(request, task, reused)? {
        return Ok((previous.clone(), true));
    }
    bijux_dna_infra::ensure_dir(&tool_root)?;
    let result = collect_tool(request, task, &tool_root);
    let result = match result {
        Ok(result) => result,
        Err(error) => failed_result(request, task, &tool_root, &format!("{error:#}"))?,
    };
    bijux_dna_infra::atomic_create_json(&result_path, &result)?;
    Ok((result, false))
}

fn reusable_baseline_tool<'a>(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    baseline: Option<&'a AdmittedSecurityProbePublication>,
) -> Result<Option<&'a ToolSecurityProbe>> {
    let Some(previous) =
        baseline.and_then(|baseline| baseline.publication.tools.get(&task.tool_id))
    else {
        return Ok(None);
    };
    if !baseline_identity_matches(task, previous)? {
        return Ok(None);
    }
    validate_resumed_result(request, task, previous)?;
    Ok(Some(previous))
}

fn baseline_identity_matches(task: &ToolTask, previous: &ToolSecurityProbe) -> Result<bool> {
    if previous.status != SecurityProbeStatus::Collected
        || previous.tool_id != task.tool_id
        || previous.version != task.version
        || previous.image_sha256 != task.image_sha256
        || previous.build_evidence_id != task.contract.build_evidence_id
    {
        return Ok(false);
    }
    let contract_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&task.contract)?);
    if previous.tool_contract_sha256 != contract_sha256 {
        return Ok(false);
    }
    Ok(true)
}

fn collect_tool(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    root: &Path,
) -> Result<ToolSecurityProbe> {
    let gitleaks = request.scanner_root.join("bin/gitleaks");
    let (sbom, vulnerability) = collect_package_reports(request, task, root)?;
    let signature_stdout = root.join("signature.stdout");
    let signature_stderr = root.join("signature.stderr");
    let signature_exit = run_bounded(
        &request.apptainer_executable,
        &[OsString::from("verify"), task.image_path.clone().into_os_string()],
        &[],
        &signature_stdout,
        &signature_stderr,
        request.timeout,
    )?;
    let sandbox = root.join("sandbox");
    let sandbox_stdout = root.join("sandbox-build.stdout");
    let sandbox_stderr = root.join("sandbox-build.stderr");
    require_success(
        run_bounded(
            &request.apptainer_executable,
            &[
                OsString::from("build"),
                OsString::from("--sandbox"),
                sandbox.clone().into_os_string(),
                task.image_path.clone().into_os_string(),
            ],
            &[],
            &sandbox_stdout,
            &sandbox_stderr,
            request.timeout,
        )?,
        "Apptainer sandbox extraction",
    )?;
    let structural = inspect_structure(&sandbox)?;
    let structural_path = root.join("structural.json");
    bijux_dna_infra::atomic_create_json(&structural_path, &structural)?;
    let secret = root.join("gitleaks.json");
    let gitleaks_stdout = root.join("gitleaks.stdout");
    let gitleaks_stderr = root.join("gitleaks.stderr");
    let gitleaks_exit = run_bounded(
        &gitleaks,
        &[
            OsString::from("detect"),
            OsString::from("--source"),
            sandbox.clone().into_os_string(),
            OsString::from("--no-git"),
            OsString::from("--redact"),
            OsString::from("--report-format"),
            OsString::from("json"),
            OsString::from("--report-path"),
            secret.clone().into_os_string(),
            OsString::from("--exit-code"),
            OsString::from("0"),
        ],
        &[],
        &gitleaks_stdout,
        &gitleaks_stderr,
        request.timeout,
    )?;
    require_success(gitleaks_exit, "Gitleaks")?;
    fs::remove_dir_all(&sandbox)
        .with_context(|| format!("remove security sandbox {}", sandbox.display()))?;
    let runtime_path = root.join("runtime.json");
    collect_runtime(request, task, root, &runtime_path)?;
    let contract_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&task.contract)?);
    let raw_manifest = write_raw_manifest(request, root)?;
    Ok(ToolSecurityProbe {
        tool_id: task.tool_id.clone(),
        version: task.version.clone(),
        image_sha256: task.image_sha256.clone(),
        tool_contract_sha256: contract_sha256,
        build_evidence_id: task.contract.build_evidence_id.clone(),
        status: SecurityProbeStatus::Collected,
        sbom: Some(artifact(request, &sbom)?),
        vulnerability_report: Some(artifact(request, &vulnerability)?),
        secret_report: Some(artifact(request, &secret)?),
        signature_stdout: Some(artifact(request, &signature_stdout)?),
        signature_stderr: Some(artifact(request, &signature_stderr)?),
        signature_verified: signature_exit == 0,
        structural_report: Some(artifact(request, &structural_path)?),
        runtime_report: Some(artifact(request, &runtime_path)?),
        raw_manifest,
        error: None,
    })
}

fn collect_package_reports(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    root: &Path,
) -> Result<(PathBuf, PathBuf)> {
    let sbom = root.join("sbom.cyclonedx.json");
    require_success(
        run_bounded(
            &request.scanner_root.join("bin/syft"),
            &[
                OsString::from("scan"),
                OsString::from(format!("singularity:{}", task.image_path.display())),
                OsString::from("-o"),
                OsString::from("cyclonedx-json"),
            ],
            &[("SYFT_CHECK_FOR_APP_UPDATE", PathBuf::from("false"))],
            &sbom,
            &root.join("syft.stderr"),
            request.timeout,
        )?,
        "Syft",
    )?;
    let vulnerability = root.join("grype.json");
    require_success(
        run_bounded(
            &request.scanner_root.join("bin/grype"),
            &[
                OsString::from(format!("sbom:{}", sbom.display())),
                OsString::from("-o"),
                OsString::from("json"),
            ],
            &[
                ("GRYPE_DB_CACHE_DIR", request.database_root.join("database")),
                ("GRYPE_DB_AUTO_UPDATE", PathBuf::from("false")),
                ("GRYPE_CHECK_FOR_APP_UPDATE", PathBuf::from("false")),
            ],
            &vulnerability,
            &root.join("grype.stderr"),
            request.timeout,
        )?,
        "Grype",
    )?;
    Ok((sbom, vulnerability))
}

fn failed_result(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    root: &Path,
    error: &str,
) -> Result<ToolSecurityProbe> {
    let sandbox = root.join("sandbox");
    if sandbox.exists() {
        fs::remove_dir_all(&sandbox)?;
    }
    let contract_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&task.contract)?);
    Ok(ToolSecurityProbe {
        tool_id: task.tool_id.clone(),
        version: task.version.clone(),
        image_sha256: task.image_sha256.clone(),
        tool_contract_sha256: contract_sha256,
        build_evidence_id: task.contract.build_evidence_id.clone(),
        status: SecurityProbeStatus::Failed,
        sbom: optional_artifact(request, &root.join("sbom.cyclonedx.json"))?,
        vulnerability_report: optional_artifact(request, &root.join("grype.json"))?,
        secret_report: optional_artifact(request, &root.join("gitleaks.json"))?,
        signature_stdout: optional_artifact(request, &root.join("signature.stdout"))?,
        signature_stderr: optional_artifact(request, &root.join("signature.stderr"))?,
        signature_verified: false,
        structural_report: optional_artifact(request, &root.join("structural.json"))?,
        runtime_report: optional_artifact(request, &root.join("runtime.json"))?,
        raw_manifest: write_raw_manifest(request, root)?,
        error: Some(error.to_string()),
    })
}

fn collect_runtime(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    root: &Path,
    output: &Path,
) -> Result<()> {
    let probes = [
        ("non_root_uid", NON_ROOT_PROBE_SCRIPT),
        ("system_read_only", "test ! -w /usr"),
        ("network_isolated", "test ! -e /sys/class/net/eth0"),
    ];
    let mut report = BTreeMap::new();
    for (name, script) in probes {
        let stdout = root.join(format!("runtime-{name}.stdout"));
        let stderr = root.join(format!("runtime-{name}.stderr"));
        let exit_code = run_bounded(
            &request.apptainer_executable,
            &[
                OsString::from("exec"),
                OsString::from("--containall"),
                OsString::from("--cleanenv"),
                OsString::from("--net"),
                OsString::from("--network"),
                OsString::from("none"),
                task.image_path.clone().into_os_string(),
                OsString::from("sh"),
                OsString::from("-c"),
                OsString::from(script),
            ],
            &[],
            &stdout,
            &stderr,
            request.timeout,
        )?;
        report.insert(
            name,
            SecurityRuntimeProbe {
                exit_code,
                stdout: artifact(request, &stdout)?,
                stderr: artifact(request, &stderr)?,
            },
        );
    }
    let report = SecurityRuntimeReport {
        network_isolated: report.remove("network_isolated").context("network probe missing")?,
        non_root_uid: report.remove("non_root_uid").context("non-root probe missing")?,
        system_read_only: report.remove("system_read_only").context("read-only probe missing")?,
    };
    bijux_dna_infra::atomic_create_json(output, &report)?;
    Ok(())
}

fn inspect_structure(root: &Path) -> Result<SecurityStructuralReport> {
    let mut pending = vec![root.to_path_buf()];
    let mut suid_sgid_observations = Vec::new();
    let mut world_writable_observations = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            }
            let mode = metadata.permissions().mode();
            let relative = format!("/{}", entry.path().strip_prefix(root)?.display());
            let kind = if metadata.is_dir() {
                SecurityFilesystemKind::Directory
            } else if metadata.is_file() {
                SecurityFilesystemKind::File
            } else {
                SecurityFilesystemKind::Other
            };
            let observation =
                SecurityFilesystemObservation { path: relative, mode: mode & 0o7777, kind };
            if mode & 0o6000 != 0 {
                suid_sgid_observations.push(observation.clone());
            }
            if mode & 0o002 != 0 {
                world_writable_observations.push(observation);
            }
        }
    }
    suid_sgid_observations.sort_by(|left, right| left.path.cmp(&right.path));
    world_writable_observations.sort_by(|left, right| left.path.cmp(&right.path));
    let suid_sgid_paths =
        suid_sgid_observations.iter().map(|observation| observation.path.clone()).collect();
    let world_writable_paths =
        world_writable_observations.iter().map(|observation| observation.path.clone()).collect();
    Ok(SecurityStructuralReport {
        suid_sgid_paths,
        world_writable_paths,
        suid_sgid_observations,
        world_writable_observations,
    })
}

fn run_bounded(
    executable: &Path,
    args: &[OsString],
    environment: &[(&str, PathBuf)],
    stdout_path: &Path,
    stderr_path: &Path,
    timeout: Duration,
) -> Result<i32> {
    let stdout = fs::File::create(stdout_path)?;
    let stderr = fs::File::create(stderr_path)?;
    let mut command = Command::new(executable);
    command.args(args).stdout(Stdio::from(stdout)).stderr(Stdio::from(stderr));
    for (key, value) in environment {
        command.env(key, value);
    }
    let mut child = command
        .spawn()
        .with_context(|| format!("execute security probe {}", executable.display()))?;
    if let Some(status) = child.wait_timeout(timeout)? {
        return Ok(status.code().unwrap_or(128));
    }
    child.kill()?;
    let _status = child.wait()?;
    bail!("security probe timed out after {} seconds", timeout.as_secs());
}

fn require_success(exit_code: i32, label: &str) -> Result<()> {
    if exit_code != 0 {
        bail!("{label} exited with status {exit_code}");
    }
    Ok(())
}

fn write_raw_manifest(
    request: &SecurityVerificationRequest,
    root: &Path,
) -> Result<SecurityProbeArtifact> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.file_name() != OsStr::new("result.json")
            && entry.file_name() != OsStr::new("raw-manifest.json")
        {
            files.insert(
                entry.file_name().to_string_lossy().to_string(),
                bijux_dna_infra::hash_file_sha256(&entry.path())?,
            );
        }
    }
    let path = root.join("raw-manifest.json");
    bijux_dna_infra::atomic_create_json(&path, &files)?;
    artifact(request, &path)
}

fn artifact(request: &SecurityVerificationRequest, path: &Path) -> Result<SecurityProbeArtifact> {
    let relative = path
        .strip_prefix(&request.study_root)
        .with_context(|| format!("security artifact outside study root: {}", path.display()))?;
    Ok(SecurityProbeArtifact {
        uri: format!("study://{}", relative.display()),
        sha256: bijux_dna_infra::hash_file_sha256(path)?,
    })
}

fn optional_artifact(
    request: &SecurityVerificationRequest,
    path: &Path,
) -> Result<Option<SecurityProbeArtifact>> {
    path.is_file().then(|| artifact(request, path)).transpose()
}

fn validate_resumed_result(
    request: &SecurityVerificationRequest,
    task: &ToolTask,
    result: &ToolSecurityProbe,
) -> Result<()> {
    if result.tool_id != task.tool_id
        || result.version != task.version
        || result.image_sha256 != task.image_sha256
        || result.build_evidence_id != task.contract.build_evidence_id
    {
        bail!("resumed security result identity differs for {}", task.tool_id);
    }
    let contract_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&task.contract)?);
    if result.tool_contract_sha256 != contract_sha256 {
        bail!("resumed security tool contract differs for {}", task.tool_id);
    }
    let artifacts = [
        result.sbom.as_ref(),
        result.vulnerability_report.as_ref(),
        result.secret_report.as_ref(),
        result.signature_stdout.as_ref(),
        result.signature_stderr.as_ref(),
        result.structural_report.as_ref(),
        result.runtime_report.as_ref(),
        Some(&result.raw_manifest),
    ];
    for artifact in artifacts.into_iter().flatten() {
        let path = request
            .study_root
            .join(artifact.uri.strip_prefix("study://").context("invalid security artifact URI")?);
        if bijux_dna_infra::hash_file_sha256(&path)? != artifact.sha256 {
            bail!("resumed security artifact digest differs for {}", task.tool_id);
        }
    }
    Ok(())
}

fn validate_request(request: &SecurityVerificationRequest, publication_path: &Path) -> Result<()> {
    if request.worker_count == 0 || request.timeout.is_zero() {
        bail!("security worker count and timeout must be positive");
    }
    let study = request.study_root.canonicalize()?;
    let evidence = request.evidence_root.canonicalize().or_else(|_| {
        bijux_dna_infra::ensure_dir(&request.evidence_root)?;
        request.evidence_root.canonicalize().map_err(anyhow::Error::from)
    })?;
    let publication_parent =
        publication_path.parent().context("security publication has no parent")?;
    let publication_parent = publication_parent.canonicalize()?;
    if !evidence.starts_with(&study)
        || !publication_parent.starts_with(&study)
        || evidence.parent() != Some(publication_parent.as_path())
    {
        bail!("security evidence outputs must remain under the governed study root");
    }
    Ok(())
}

fn ensure_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if path.exists() {
        let existing: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
        let expected = serde_json::to_value(value)?;
        if existing == expected {
            return Ok(());
        }
        bail!("immutable security publication conflicts at {}", path.display());
    }
    bijux_dna_infra::atomic_create_json(path, value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::time::Duration;

    use anyhow::Result;

    use super::{
        baseline_identity_matches, ensure_immutable_json, inspect_structure, run_bounded,
        validate_request, ContainerToolContract, SecurityFilesystemKind, SecurityProbeArtifact,
        SecurityProbeStatus, SecurityVerificationRequest, ToolSecurityProbe, ToolTask,
        NON_ROOT_PROBE_SCRIPT,
    };

    #[test]
    fn bounded_process_preserves_complete_streams() -> Result<()> {
        let root = bijux_dna_testkit::tempdir_for("security-probe-streams");
        let stdout = root.path().join("stdout");
        let stderr = root.path().join("stderr");
        let exit = run_bounded(
            std::path::Path::new("/bin/sh"),
            &[
                OsString::from("-c"),
                OsString::from("printf complete-output; printf complete-error >&2"),
            ],
            &[],
            &stdout,
            &stderr,
            Duration::from_secs(2),
        )?;

        assert_eq!(exit, 0);
        assert_eq!(fs::read_to_string(stdout)?, "complete-output");
        assert_eq!(fs::read_to_string(stderr)?, "complete-error");
        Ok(())
    }

    #[test]
    fn baseline_reuse_requires_exact_image_and_tool_contract_identity() -> Result<()> {
        let contract = ContainerToolContract {
            schema_version: "bijux.container.tool_contract.v2".to_string(),
            tool_id: "samtools".to_string(),
            version: "1.20".to_string(),
            support_disposition: "retained".to_string(),
            catalog_entry_sha256: "1".repeat(64),
            catalog_sha256: "2".repeat(64),
            recipe_path: "containers/tools/samtools/samtools".to_string(),
            recipe_sha256: "3".repeat(64),
            source_sha256: None,
            pinned_commit: None,
            repository_commit: "4".repeat(40),
            architecture: "linux/amd64".to_string(),
            build_evidence_id: "5".repeat(64),
            expected_binary: "samtools".to_string(),
            version_command: "samtools --version".to_string(),
            expected_version_regex: "1\\.20".to_string(),
            interface_sha256: "6".repeat(64),
        };
        let task = ToolTask {
            tool_id: contract.tool_id.clone(),
            version: contract.version.clone(),
            image_path: "/images/samtools.sif".into(),
            image_sha256: "7".repeat(64),
            contract,
        };
        let mut previous = ToolSecurityProbe {
            tool_id: task.tool_id.clone(),
            version: task.version.clone(),
            image_sha256: task.image_sha256.clone(),
            tool_contract_sha256: bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(
                &task.contract,
            )?),
            build_evidence_id: task.contract.build_evidence_id.clone(),
            status: SecurityProbeStatus::Collected,
            sbom: None,
            vulnerability_report: None,
            secret_report: None,
            signature_stdout: None,
            signature_stderr: None,
            signature_verified: false,
            structural_report: None,
            runtime_report: None,
            raw_manifest: SecurityProbeArtifact {
                uri: "study://security/raw-manifest.json".to_string(),
                sha256: "8".repeat(64),
            },
            error: None,
        };

        assert!(baseline_identity_matches(&task, &previous)?);
        previous.image_sha256 = "9".repeat(64);
        assert!(!baseline_identity_matches(&task, &previous)?);
        previous.image_sha256 = task.image_sha256.clone();
        previous.tool_contract_sha256 = "a".repeat(64);
        assert!(!baseline_identity_matches(&task, &previous)?);
        Ok(())
    }

    #[test]
    fn structural_probe_records_privileged_and_world_writable_paths() -> Result<()> {
        use std::os::unix::fs::PermissionsExt as _;

        let root = bijux_dna_testkit::tempdir_for("security-structural-probe");
        let privileged = root.path().join("privileged");
        let sticky = root.path().join("sticky");
        let writable = root.path().join("writable");
        fs::write(&privileged, b"fixture")?;
        fs::create_dir(&sticky)?;
        fs::write(&writable, b"fixture")?;
        fs::set_permissions(&privileged, fs::Permissions::from_mode(0o4755))?;
        fs::set_permissions(&sticky, fs::Permissions::from_mode(0o1777))?;
        fs::set_permissions(&writable, fs::Permissions::from_mode(0o666))?;

        let report = inspect_structure(root.path())?;

        assert_eq!(report.suid_sgid_paths, ["/privileged"]);
        assert_eq!(report.suid_sgid_observations[0].mode, 0o4755);
        assert_eq!(report.world_writable_paths, ["/sticky", "/writable"]);
        assert_eq!(report.world_writable_observations[0].mode, 0o1777);
        assert_eq!(report.world_writable_observations[0].kind, SecurityFilesystemKind::Directory);
        assert_eq!(report.world_writable_observations[1].mode, 0o666);
        assert_eq!(report.world_writable_observations[1].kind, SecurityFilesystemKind::File);
        Ok(())
    }

    #[test]
    fn non_root_probe_emits_the_observed_uid() {
        assert!(NON_ROOT_PROBE_SCRIPT.contains("printf '%s\\n'"));
        assert!(NON_ROOT_PROBE_SCRIPT.ends_with("test \"$uid\" -ne 0"));
    }

    #[test]
    fn immutable_publication_accepts_exact_replay_and_rejects_conflict() -> Result<()> {
        let root = bijux_dna_testkit::tempdir_for("security-publication-replay");
        let publication = root.path().join("publication.json");

        ensure_immutable_json(&publication, &serde_json::json!({"evidence": "exact"}))?;
        ensure_immutable_json(&publication, &serde_json::json!({"evidence": "exact"}))?;

        let Err(error) =
            ensure_immutable_json(&publication, &serde_json::json!({"evidence": "conflicting"}))
        else {
            anyhow::bail!("conflicting immutable publication was accepted");
        };
        assert!(error.to_string().contains("immutable security publication conflicts"));
        Ok(())
    }

    #[test]
    fn publication_and_evidence_use_governed_sibling_layout() -> Result<()> {
        let study = bijux_dna_testkit::tempdir_for("security-publication-path");
        let attempt = study.path().join("attempt");
        let evidence = attempt.join("evidence");
        fs::create_dir_all(&evidence)?;
        fs::create_dir(study.path().join("escaped"))?;
        let request = SecurityVerificationRequest {
            repository_root: study.path().to_path_buf(),
            study_root: study.path().to_path_buf(),
            inventory_path: study.path().join("inventory.json"),
            apptainer_executable: study.path().join("apptainer"),
            scanner_contract_path: study.path().join("scanner.toml"),
            scanner_root: study.path().join("scanner"),
            database_contract_path: study.path().join("database.toml"),
            database_root: study.path().join("database"),
            reused_publication_path: None,
            evidence_root: evidence.clone(),
            worker_count: 1,
            timeout: Duration::from_secs(1),
        };

        validate_request(&request, &attempt.join("publication.json"))?;

        let Err(error) = validate_request(&request, &study.path().join("escaped/publication.json"))
        else {
            anyhow::bail!("escaped publication path was accepted");
        };
        assert!(error.to_string().contains("governed study root"));
        Ok(())
    }
}
