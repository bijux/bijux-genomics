use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use bijux_dna_containers::api::{
    clean_repository_head_commit, derive_apptainer_tool_contract, load_container_catalog,
    load_image_inventory_publication, render_runtime_verification_markdown,
    resolve_catalog_locked_apptainer_image, resolve_current_apptainer_contract_commit,
    run_apptainer_smoke, validate_container_evidence_chain, validate_container_inventory_catalog,
    validate_container_recipe_evidence_chain, validate_runtime_verification_publication,
    validate_tool_image_identity_evidence, verify_apptainer_tool_contract, ApptainerSmokeManifest,
    ApptainerSmokeRequest, ContainerRuntimeEvidence, ContainerToolContract, EvidenceSourceIdentity,
    RuntimeEvidenceStatus, RuntimeVerificationPublication,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2,
};

#[derive(Debug, Clone)]
pub struct RuntimeVerificationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub inventory_path: PathBuf,
    pub apptainer_executable: PathBuf,
    pub data_root: PathBuf,
    pub evidence_root: PathBuf,
    pub worker_count: usize,
    pub timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct RuntimeVerificationResult {
    pub publication: RuntimeVerificationPublication,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone)]
struct ToolTask {
    tool_id: String,
    version: String,
    upstream: String,
    help_command: String,
    contract: ContainerToolContract,
    image_path: PathBuf,
    image_sha256: String,
}

#[derive(Debug)]
struct ToolResult {
    tool_id: String,
    version: String,
    image_sha256: String,
    status: RuntimeEvidenceStatus,
    identity: EvidenceSourceIdentity,
    manifest: EvidenceSourceIdentity,
}

/// Execute bounded version/help probes for every current retained image and publish exact evidence.
///
/// Existing matching manifests are reused, making an interrupted campaign resumable without
/// replacing immutable evidence. All tools are attempted before execution errors are returned.
///
/// # Errors
/// Returns an error when source identities drift, selected images differ on disk, an immutable
/// evidence path conflicts, a worker fails, or output publication cannot be created atomically.
pub fn execute_runtime_verification(
    request: &RuntimeVerificationRequest,
    publication_path: &Path,
    markdown_path: &Path,
) -> Result<RuntimeVerificationResult> {
    validate_request(request, publication_path, markdown_path)?;
    let commit = clean_repository_head_commit(&request.repository_root)?;
    let inventory = load_image_inventory_publication(&request.inventory_path)?;
    if inventory.repository_commit != commit {
        bail!("runtime repository and inventory identify different commits");
    }
    let catalog = load_container_catalog(&request.repository_root)?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &request.repository_root.join("containers/contracts/catalog.toml"),
    )
    .context("hash runtime-verification container catalog")?;
    validate_container_inventory_catalog(&catalog_sha256, &catalog, &inventory)?;
    validate_container_recipe_evidence_chain(
        &request.repository_root,
        &commit,
        &catalog,
        &inventory,
    )?;
    let tasks = catalog
        .tools
        .iter()
        .filter(|tool| tool.support_disposition == "retained" && tool.apptainer_supported)
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
                upstream: tool.upstream.clone(),
                help_command: tool.help_command.clone(),
                contract,
                image_path: image.path,
                image_sha256: image.sha256,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if tasks.is_empty() {
        bail!("runtime verification has no retained Apptainer tools");
    }
    bijux_dna_infra::ensure_dir(&request.evidence_root)?;
    let results = execute_tasks(request, &commit, &catalog_sha256, &tasks)?;
    let mut tools = BTreeMap::new();
    let mut passed = 0;
    for result in results {
        passed += usize::from(result.status == RuntimeEvidenceStatus::Passed);
        tools.insert(
            result.tool_id,
            ContainerRuntimeEvidence {
                version: result.version,
                image_sha256: result.image_sha256,
                status: result.status,
                identity: Some(result.identity),
                manifest: Some(result.manifest),
            },
        );
    }
    let publication = RuntimeVerificationPublication {
        schema_version: CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2.to_string(),
        catalog: EvidenceSourceIdentity {
            path: "repository://containers/contracts/catalog.toml".to_string(),
            sha256: catalog_sha256.clone(),
        },
        inventory: EvidenceSourceIdentity {
            path: portable_path(&request.inventory_path, &request.study_root, "study")?,
            sha256: inventory.publication.raw_sha256.clone(),
        },
        repository_commit: commit,
        tools,
    };
    if passed == tasks.len() {
        validate_container_evidence_chain(&catalog_sha256, &catalog, &inventory, &publication)?;
    } else {
        validate_runtime_verification_publication(&publication)?;
    }
    ensure_immutable_json(publication_path, &publication)?;
    ensure_immutable_bytes(
        markdown_path,
        render_runtime_verification_markdown(&publication).as_bytes(),
    )?;
    Ok(RuntimeVerificationResult { publication, passed, failed: tasks.len() - passed })
}

fn execute_tasks(
    request: &RuntimeVerificationRequest,
    commit: &str,
    catalog_sha256: &str,
    tasks: &[ToolTask],
) -> Result<Vec<ToolResult>> {
    let next = AtomicUsize::new(0);
    let results = Mutex::new(Vec::with_capacity(tasks.len()));
    let errors = Mutex::new(Vec::new());
    let workers = request.worker_count.min(tasks.len());
    std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            handles.push(scope.spawn(|| -> Result<()> {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(task) = tasks.get(index) else {
                        break;
                    };
                    match execute_tool(request, commit, catalog_sha256, task) {
                        Ok(result) => {
                            results
                                .lock()
                                .map_err(|_| anyhow::anyhow!("runtime result lock poisoned"))?
                                .push(result);
                        }
                        Err(error) => {
                            errors
                                .lock()
                                .map_err(|_| anyhow::anyhow!("runtime error lock poisoned"))?
                                .push(format!("{}: {error:#}", task.tool_id));
                        }
                    }
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("runtime verification worker panicked"))??;
        }
        Ok(())
    })?;
    let errors = errors.into_inner().map_err(|_| anyhow::anyhow!("runtime error lock poisoned"))?;
    if !errors.is_empty() {
        bail!("runtime verification execution errors: {}", errors.join("; "));
    }
    let mut results =
        results.into_inner().map_err(|_| anyhow::anyhow!("runtime result lock poisoned"))?;
    results.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    Ok(results)
}

fn execute_tool(
    request: &RuntimeVerificationRequest,
    commit: &str,
    catalog_sha256: &str,
    task: &ToolTask,
) -> Result<ToolResult> {
    let identity_path =
        request.evidence_root.join("identities").join(format!("{}.json", task.tool_id));
    let identity = verify_apptainer_tool_contract(
        &request.apptainer_executable,
        &task.image_path,
        &task.image_sha256,
        &task.contract,
    )?;
    validate_tool_image_identity_evidence(&identity, &task.image_sha256, &task.contract)?;
    ensure_immutable_json(&identity_path, &identity)?;
    let manifest_path =
        request.evidence_root.join("manifests").join(format!("{}.json", task.tool_id));
    let manifest = if manifest_path.is_file() {
        let bytes = fs::read(&manifest_path)
            .with_context(|| format!("read runtime manifest {}", manifest_path.display()))?;
        let manifest: ApptainerSmokeManifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse runtime manifest {}", manifest_path.display()))?;
        validate_manifest(&manifest, task, commit, catalog_sha256)?;
        manifest
    } else {
        let results_root = request.evidence_root.join("results").join(&task.tool_id);
        let log_root = request.evidence_root.join("logs").join(&task.tool_id);
        bijux_dna_infra::ensure_dir(&results_root)?;
        let manifest = run_apptainer_smoke(&ApptainerSmokeRequest {
            executable: request.apptainer_executable.clone(),
            image_path: task.image_path.clone(),
            data_root: request.data_root.clone(),
            results_root,
            tool_id: task.tool_id.clone(),
            stage_id: "catalog.retained".to_string(),
            build_evidence_id: task.contract.build_evidence_id.clone(),
            repository_commit: commit.to_string(),
            registry_digest: catalog_sha256.to_string(),
            image_sha256: task.image_sha256.clone(),
            version_command: task.contract.version_command.clone(),
            expected_version_regex: task.contract.expected_version_regex.clone(),
            help_command: task.help_command.clone(),
            require_help: true,
            probe_commands: Vec::new(),
            java_heap_mb: None,
            upstream: task.upstream.clone(),
            evidence_root: Some(log_root),
            timeout: request.timeout,
        })?;
        ensure_immutable_json(&manifest_path, &manifest)?;
        manifest
    };
    let status = if manifest.status == "ok" {
        RuntimeEvidenceStatus::Passed
    } else {
        RuntimeEvidenceStatus::Failed
    };
    Ok(ToolResult {
        tool_id: task.tool_id.clone(),
        version: task.version.clone(),
        image_sha256: task.image_sha256.clone(),
        status,
        identity: EvidenceSourceIdentity {
            path: portable_path(&identity_path, &request.study_root, "study")?,
            sha256: bijux_dna_infra::hash_file_sha256(&identity_path)?,
        },
        manifest: EvidenceSourceIdentity {
            path: portable_path(&manifest_path, &request.study_root, "study")?,
            sha256: bijux_dna_infra::hash_file_sha256(&manifest_path)?,
        },
    })
}

fn validate_manifest(
    manifest: &ApptainerSmokeManifest,
    task: &ToolTask,
    commit: &str,
    catalog_sha256: &str,
) -> Result<()> {
    if manifest.tool_id != task.tool_id
        || manifest.repository_commit != commit
        || manifest.registry_digest != catalog_sha256
        || manifest.sif_sha256 != task.image_sha256
        || manifest.build_evidence_id != task.contract.build_evidence_id
        || manifest.version_cmd != task.contract.version_command
        || manifest.help_cmd != task.help_command
        || manifest.expected_version_regex != task.contract.expected_version_regex
    {
        bail!("runtime manifest identity differs from requested tool evidence");
    }
    Ok(())
}

fn validate_request(
    request: &RuntimeVerificationRequest,
    publication_path: &Path,
    markdown_path: &Path,
) -> Result<()> {
    if request.worker_count == 0 || request.timeout.is_zero() {
        bail!("runtime worker count and timeout must be positive");
    }
    if !request.repository_root.is_dir()
        || !request.study_root.is_dir()
        || !request.inventory_path.is_file()
        || !request.data_root.is_dir()
    {
        bail!("runtime verification source roots are incomplete");
    }
    if publication_path == markdown_path {
        bail!("runtime JSON and Markdown outputs must be distinct");
    }
    if publication_path.try_exists()? || markdown_path.try_exists()? {
        bail!("refusing to replace runtime-verification publication");
    }
    Ok(())
}

fn portable_path(path: &Path, root: &Path, scheme: &str) -> Result<String> {
    let root = root.canonicalize().with_context(|| format!("canonicalize {scheme} root"))?;
    let path = path
        .canonicalize()
        .with_context(|| format!("canonicalize governed evidence {}", path.display()))?;
    let relative = path
        .strip_prefix(&root)
        .with_context(|| format!("evidence path is outside governed {scheme} root"))?;
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(component, Component::CurDir | Component::ParentDir | Component::RootDir)
        })
    {
        bail!("governed evidence path is not portable");
    }
    Ok(format!("{scheme}://{}", relative.to_string_lossy().replace('\\', "/")))
}

fn ensure_immutable_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure_immutable_bytes(path, &bytes)
}

fn ensure_immutable_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    if path.try_exists()? {
        if fs::read(path)? != bytes {
            bail!("immutable runtime evidence differs at {}", path.display());
        }
        return Ok(());
    }
    let parent = path.parent().context("runtime evidence path has no parent")?;
    bijux_dna_infra::ensure_dir(parent)?;
    bijux_dna_infra::atomic_create_bytes(path, bytes)?;
    Ok(())
}
