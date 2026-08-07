use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};

use crate::site_policy::{validate_container_governance_host, ContainerGovernance};

use super::model::{
    ContainerPipelineContract, PipelineEvidenceKind, PipelineInputKind, PipelineRepository,
};
use super::validation::load_container_pipeline;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPipelineSiteAdmissionRequest {
    pub governance: ContainerGovernance,
    pub hostname: String,
    pub pipeline_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPipelineInputAdmission {
    pub id: String,
    pub kind: PipelineInputKind,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPipelineSiteAdmission {
    pub schema_version: String,
    pub run_id: String,
    pub evidence_kind: PipelineEvidenceKind,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub image_path: PathBuf,
    pub image_sha256: String,
    pub inputs: Vec<ContainerPipelineInputAdmission>,
    pub result_root: PathBuf,
    pub account: String,
    pub partition: String,
}

struct PipelineSiteFields<'a> {
    schema_version: &'a str,
    current_schema: bool,
    run_id: &'a str,
    stage: &'a str,
    tool: &'a str,
    evidence_kind: PipelineEvidenceKind,
    repository: &'a PipelineRepository,
    image_path: &'a Path,
    image_sha256: &'a str,
    inputs: Vec<PipelineInputFields<'a>>,
    result_root: &'a Path,
    account: &'a str,
    partition: &'a str,
}

struct PipelineInputFields<'a> {
    id: &'a str,
    kind: PipelineInputKind,
    path: &'a Path,
    sha256: &'a str,
}

/// Admit one structurally valid pipeline against the concrete execution site.
///
/// This boundary verifies host policy and immutable filesystem identities. It does not execute the
/// pipeline, submit scheduler jobs, or interpret stage-interface semantics.
///
/// # Errors
/// Returns an error when site policy, repository state, image or input identity, output placement,
/// or scheduler resource selection differs from the governed declaration.
pub fn admit_container_pipeline_site(
    request: &ContainerPipelineSiteAdmissionRequest,
) -> Result<ContainerPipelineSiteAdmission> {
    validate_container_governance_host(&request.governance, &request.hostname)?;
    let contract = load_container_pipeline(&request.pipeline_path)?;
    let fields = pipeline_site_fields(&contract);
    admit_pipeline_fields(&request.governance, fields)
        .with_context(|| format!("admit pipeline site {}", request.pipeline_path.display()))
}

fn pipeline_site_fields(contract: &ContainerPipelineContract) -> PipelineSiteFields<'_> {
    match contract {
        ContainerPipelineContract::Legacy(pipeline) => PipelineSiteFields {
            schema_version: &pipeline.schema_version,
            current_schema: false,
            run_id: &pipeline.run.id,
            stage: &pipeline.run.stage,
            tool: &pipeline.run.tool,
            evidence_kind: PipelineEvidenceKind::Qualification,
            repository: &pipeline.repository,
            image_path: &pipeline.image.path,
            image_sha256: &pipeline.image.sha256,
            inputs: pipeline
                .data
                .inputs
                .iter()
                .map(|input| PipelineInputFields {
                    id: &input.id,
                    kind: input.kind.unwrap_or(PipelineInputKind::Data),
                    path: &input.path,
                    sha256: &input.sha256,
                })
                .collect(),
            result_root: &pipeline.outputs.result_root,
            account: &pipeline.resources.account,
            partition: &pipeline.resources.partition,
        },
        ContainerPipelineContract::Current(pipeline) => PipelineSiteFields {
            schema_version: &pipeline.schema_version,
            current_schema: true,
            run_id: &pipeline.run.id,
            stage: &pipeline.run.stage,
            tool: &pipeline.run.tool,
            evidence_kind: pipeline.run.kind.unwrap_or(PipelineEvidenceKind::Qualification),
            repository: &pipeline.repository,
            image_path: &pipeline.image.path,
            image_sha256: &pipeline.image.sha256,
            inputs: pipeline
                .data
                .inputs
                .iter()
                .map(|input| PipelineInputFields {
                    id: &input.id,
                    kind: input.kind.unwrap_or(PipelineInputKind::Data),
                    path: &input.path,
                    sha256: &input.sha256,
                })
                .collect(),
            result_root: &pipeline.outputs.result_root,
            account: &pipeline.resources.account,
            partition: &pipeline.resources.partition,
        },
    }
}

fn admit_pipeline_fields(
    governance: &ContainerGovernance,
    fields: PipelineSiteFields<'_>,
) -> Result<ContainerPipelineSiteAdmission> {
    ensure!(
        fields.account == governance.slurm.account,
        "resources.account is outside the governance contract"
    );
    ensure!(
        governance.slurm.partitions.iter().any(|partition| partition == fields.partition),
        "resources.partition is outside the governance contract"
    );

    let repository_path = admit_repository(governance, fields.repository)?;
    let image_path = admit_file_under(
        fields.image_path,
        &governance.paths.containers_root.join("apptainer"),
        fields.image_sha256,
        "image.path",
    )?;
    ensure!(
        image_path.extension().and_then(|extension| extension.to_str()) == Some("sif"),
        "image.path must be an immutable SIF: {}",
        image_path.display()
    );

    let mut inputs = Vec::with_capacity(fields.inputs.len());
    for input in fields.inputs {
        let root = if input.kind == PipelineInputKind::UpstreamResult {
            &governance.paths.results_root
        } else {
            &governance.paths.data_root
        };
        let path = admit_file_under(input.path, root, input.sha256, "data input path")?;
        inputs.push(ContainerPipelineInputAdmission {
            id: input.id.to_owned(),
            kind: input.kind,
            path,
            sha256: input.sha256.to_owned(),
        });
    }

    let result_root = admit_result_root(
        governance,
        fields.evidence_kind,
        fields.current_schema,
        fields.run_id,
        fields.stage,
        fields.tool,
        fields.result_root,
    )?;

    Ok(ContainerPipelineSiteAdmission {
        schema_version: fields.schema_version.to_owned(),
        run_id: fields.run_id.to_owned(),
        evidence_kind: fields.evidence_kind,
        repository_path,
        repository_commit: fields.repository.commit.clone(),
        image_path,
        image_sha256: fields.image_sha256.to_owned(),
        inputs,
        result_root,
        account: fields.account.to_owned(),
        partition: fields.partition.to_owned(),
    })
}

fn admit_repository(
    governance: &ContainerGovernance,
    repository: &PipelineRepository,
) -> Result<PathBuf> {
    let repository_path = canonical_path(&repository.path, "repository.path")?;
    let active_repository =
        canonical_path(&governance.paths.active_repository, "paths.active_repository")?;
    let frozen_repositories =
        canonical_path(&governance.paths.frozen_repositories, "paths.frozen_repositories")?;
    let is_active = repository_path == active_repository;
    let is_frozen = repository_path.parent() == Some(frozen_repositories.as_path())
        && repository_path.file_name().and_then(|name| name.to_str())
            == Some(repository.commit.as_str());
    ensure!(
        is_active || is_frozen,
        "repository.path must identify the active checkout or its commit-addressed frozen snapshot"
    );

    let head = git_output(&repository_path, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    ensure!(head == repository.commit, "repository HEAD differs from pipeline commit");
    let status =
        git_output(&repository_path, &["status", "--porcelain=v1", "--untracked-files=normal"])?;
    ensure!(status.is_empty(), "repository worktree is not clean");
    Ok(repository_path)
}

fn admit_file_under(
    path: &Path,
    root: &Path,
    expected_sha256: &str,
    label: &str,
) -> Result<PathBuf> {
    let root = canonical_path(root, label)?;
    let path = canonical_path(path, label)?;
    ensure!(path.starts_with(&root) && path != root, "{label} is outside governed root");
    ensure!(path.is_file(), "{label} is not a file: {}", path.display());
    let observed = sha256_file(&path)?;
    ensure!(observed == expected_sha256, "{label} digest differs from pipeline declaration");
    Ok(path)
}

fn admit_result_root(
    governance: &ContainerGovernance,
    evidence_kind: PipelineEvidenceKind,
    current_schema: bool,
    run_id: &str,
    stage: &str,
    tool: &str,
    declared: &Path,
) -> Result<PathBuf> {
    let governed_root = canonical_path(&governance.paths.results_root, "paths.results_root")?;
    let admitted_path = resolve_path(declared, "outputs.result_root")?;
    ensure!(
        admitted_path.starts_with(&governed_root) && admitted_path != governed_root,
        "outputs.result_root is outside the governed results root"
    );
    match evidence_kind {
        PipelineEvidenceKind::Benchmark => {
            let expected = governed_root
                .join("benchmarks")
                .join(format!("{run_id}-{}", stage_slug(stage)))
                .join(tool);
            ensure!(
                admitted_path == expected,
                "benchmark outputs.result_root must equal {}",
                expected.display()
            );
        }
        PipelineEvidenceKind::Qualification => {
            let qualification_root = governed_root.join("container-validation");
            ensure!(
                admitted_path.starts_with(&qualification_root)
                    && admitted_path != qualification_root,
                "qualification outputs.result_root is outside {}",
                qualification_root.display()
            );
            let expected_name =
                if current_schema { format!("{run_id}-{tool}") } else { run_id.to_owned() };
            ensure!(
                admitted_path.file_name().and_then(|name| name.to_str())
                    == Some(expected_name.as_str()),
                "qualification outputs.result_root must end with {expected_name:?}"
            );
        }
    }
    Ok(admitted_path)
}

fn stage_slug(stage: &str) -> String {
    let mut slug = String::new();
    let mut separated = false;
    for byte in stage.bytes() {
        if byte.is_ascii_alphanumeric() {
            slug.push((byte as char).to_ascii_lowercase());
            separated = false;
        } else if !slug.is_empty() && !separated {
            slug.push('-');
            separated = true;
        }
    }
    slug.trim_end_matches('-').to_owned()
}

fn canonical_path(path: &Path, label: &str) -> Result<PathBuf> {
    ensure_safe_absolute(path, label)?;
    path.canonicalize().with_context(|| format!("resolve {label} {}", path.display()))
}

fn resolve_path(path: &Path, label: &str) -> Result<PathBuf> {
    ensure_safe_absolute(path, label)?;
    let mut ancestor = path;
    let mut suffix = Vec::<OsString>::new();
    while !ancestor.exists() {
        let name = ancestor
            .file_name()
            .with_context(|| format!("resolve existing ancestor for {label}"))?;
        suffix.push(name.to_owned());
        ancestor = ancestor.parent().with_context(|| format!("resolve parent for {label}"))?;
    }
    let mut resolved = ancestor
        .canonicalize()
        .with_context(|| format!("resolve {label} ancestor {}", ancestor.display()))?;
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn ensure_safe_absolute(path: &Path, label: &str) -> Result<()> {
    ensure!(path.is_absolute(), "{label} must be absolute");
    ensure!(
        path.components().all(|component| {
            matches!(component, Component::RootDir | Component::Normal(_) | Component::Prefix(_))
        }),
        "{label} contains unsafe path components: {}",
        path.display()
    );
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let count = file.read(&mut buffer).with_context(|| format!("read {}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex_digest(&hasher.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn git_output(repository: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()
        .with_context(|| format!("execute git in {}", repository.display()))?;
    ensure!(
        output.status.success(),
        "git {} failed in {}: {}",
        arguments.join(" "),
        repository.display(),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
