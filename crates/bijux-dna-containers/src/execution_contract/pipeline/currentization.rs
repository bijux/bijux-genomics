use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};

use crate::evidence::{
    load_image_inventory_publication, resolve_current_apptainer_image, ContainerSupportDisposition,
};
use crate::site_policy::{validate_container_governance_host, ContainerGovernance};

use super::model::ContainerPipelineContract;
use super::validation::parse_container_pipeline_document;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPipelineCurrentizationRequest {
    pub governance: ContainerGovernance,
    pub hostname: String,
    pub pipelines_root: PathBuf,
    pub inventory_path: PathBuf,
    pub repository_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPipelineCurrentizationEntry {
    pub path: PathBuf,
    pub run_id: String,
    pub tool_id: String,
    pub source_sha256: String,
    pub current_sha256: String,
    pub image_path: PathBuf,
    pub image_sha256: String,
    pub disposition: ContainerPipelineCurrentizationDisposition,
    pub changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerPipelineCurrentizationDisposition {
    Currentized,
    AlreadyCurrent,
    ExternalArtifactRequired,
    PublishedEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPipelineCurrentizationPlan {
    pub inventory_sha256: String,
    pub repository_path: PathBuf,
    pub repository_commit: String,
    pub entries: Vec<ContainerPipelineCurrentizationEntry>,
}

struct RenderedPipeline {
    entry: ContainerPipelineCurrentizationEntry,
    bytes: Vec<u8>,
}

/// Derive immutable repository and image identity updates for every tracked pipeline declaration.
///
/// The plan is non-mutating and binds every source and rendered document by SHA-256.
///
/// # Errors
/// Returns an error when governance, inventory, repository identity, pipeline structure, or current
/// image resolution is invalid.
pub fn derive_container_pipeline_currentization(
    request: &ContainerPipelineCurrentizationRequest,
) -> Result<ContainerPipelineCurrentizationPlan> {
    let rendered = render_currentization(request)?;
    plan_from_rendered(request, &rendered)
}

/// Atomically apply an unchanged currentization plan.
///
/// # Errors
/// Returns an error when any input has drifted since planning or an atomic write fails.
pub fn apply_container_pipeline_currentization(
    request: &ContainerPipelineCurrentizationRequest,
    plan: &ContainerPipelineCurrentizationPlan,
) -> Result<()> {
    let rendered = render_currentization(request)?;
    let inventory_sha256 = bijux_dna_infra::hash_file_sha256(&request.inventory_path)
        .context("hash currentization inventory")?;
    ensure!(
        plan.inventory_sha256 == inventory_sha256
            && plan.repository_commit == request.repository_commit
            && plan.repository_path
                == request.governance.paths.frozen_repositories.join(&request.repository_commit)
            && plan.entries.len() == rendered.len(),
        "container pipeline currentization plan identity changed after planning"
    );
    for (planned, document) in plan.entries.iter().zip(rendered) {
        ensure!(
            planned.path == document.entry.path
                && planned.run_id == document.entry.run_id
                && planned.tool_id == document.entry.tool_id
                && planned.current_sha256 == document.entry.current_sha256
                && planned.image_path == document.entry.image_path
                && planned.image_sha256 == document.entry.image_sha256
                && is_external_artifact_disposition(planned.disposition)
                    == is_external_artifact_disposition(document.entry.disposition),
            "container pipeline currentization target changed after planning"
        );
        let observed = document.entry.source_sha256;
        if observed == planned.current_sha256 {
            continue;
        }
        ensure!(
            observed == planned.source_sha256,
            "container pipeline currentization source changed after planning"
        );
        bijux_dna_infra::atomic_write_bytes(&planned.path, &document.bytes)
            .with_context(|| format!("write current pipeline {}", planned.path.display()))?;
    }
    Ok(())
}

fn render_currentization(
    request: &ContainerPipelineCurrentizationRequest,
) -> Result<Vec<RenderedPipeline>> {
    validate_container_governance_host(&request.governance, &request.hostname)?;
    validate_commit(&request.repository_commit)?;
    let inventory = load_image_inventory_publication(&request.inventory_path)?;
    ensure!(
        inventory.repository_commit == request.repository_commit,
        "inventory repository commit differs from currentization commit"
    );
    let repository_path =
        request.governance.paths.frozen_repositories.join(&request.repository_commit);
    ensure!(repository_path.is_dir(), "currentization repository freeze is absent");
    ensure!(request.pipelines_root.is_dir(), "pipeline declaration root is absent");

    let mut paths = std::fs::read_dir(&request.pipelines_root)
        .with_context(|| format!("read pipeline root {}", request.pipelines_root.display()))?
        .map(|entry| entry.map(|entry| entry.path().join("config-pipeline.yml")))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.is_file());
    paths.sort();
    ensure!(!paths.is_empty(), "pipeline declaration root contains no contracts");

    let mut rendered = Vec::with_capacity(paths.len());
    for path in paths {
        let source = std::fs::read_to_string(&path)
            .with_context(|| format!("read pipeline declaration {}", path.display()))?;
        let contract = parse_container_pipeline_document(&source, &path.display().to_string())?;
        let (run_id, tool_id) = pipeline_identity(&contract);
        if path.parent().is_some_and(|parent| parent.join("evidence.lock.json").is_file()) {
            rendered.push(preserve_pipeline_document(
                path,
                source,
                &contract,
                ContainerPipelineCurrentizationDisposition::PublishedEvidence,
            ));
            continue;
        }
        let inventory_tool = inventory
            .tools
            .iter()
            .find(|tool| tool.tool == tool_id)
            .with_context(|| format!("image inventory does not declare tool {tool_id}"))?;
        if inventory_tool.support_disposition == ContainerSupportDisposition::Unsupported {
            rendered.push(preserve_pipeline_document(
                path,
                source,
                &contract,
                ContainerPipelineCurrentizationDisposition::ExternalArtifactRequired,
            ));
            continue;
        }
        ensure!(
            inventory_tool.support_disposition == ContainerSupportDisposition::Retained,
            "pipeline tool {tool_id} is outside retained and external-artifact inventory policy"
        );
        let image = resolve_current_apptainer_image(
            &inventory,
            &request.governance.paths.study_root,
            tool_id,
        )?;
        let current = rewrite_pipeline_identities(
            &source,
            &repository_path,
            &request.repository_commit,
            &image.path,
            &image.sha256,
        )?;
        parse_container_pipeline_document(&current, &format!("rendered {}", path.display()))?;
        let source_sha256 = bijux_dna_infra::hash_bytes_sha256(source.as_bytes());
        let current_sha256 = bijux_dna_infra::hash_bytes_sha256(current.as_bytes());
        let changed = source_sha256 != current_sha256;
        rendered.push(RenderedPipeline {
            entry: ContainerPipelineCurrentizationEntry {
                path,
                run_id: run_id.to_owned(),
                tool_id: tool_id.to_owned(),
                source_sha256: source_sha256.clone(),
                current_sha256,
                image_path: image.path,
                image_sha256: image.sha256,
                disposition: if changed {
                    ContainerPipelineCurrentizationDisposition::Currentized
                } else {
                    ContainerPipelineCurrentizationDisposition::AlreadyCurrent
                },
                changed,
            },
            bytes: current.into_bytes(),
        });
    }
    Ok(rendered)
}

fn preserve_pipeline_document(
    path: PathBuf,
    source: String,
    contract: &ContainerPipelineContract,
    disposition: ContainerPipelineCurrentizationDisposition,
) -> RenderedPipeline {
    let (run_id, tool_id) = pipeline_identity(contract);
    let (image_path, image_sha256) = pipeline_image_identity(contract);
    let source_sha256 = bijux_dna_infra::hash_bytes_sha256(source.as_bytes());
    RenderedPipeline {
        entry: ContainerPipelineCurrentizationEntry {
            path,
            run_id: run_id.to_owned(),
            tool_id: tool_id.to_owned(),
            source_sha256: source_sha256.clone(),
            current_sha256: source_sha256,
            image_path: image_path.to_path_buf(),
            image_sha256: image_sha256.to_owned(),
            disposition,
            changed: false,
        },
        bytes: source.into_bytes(),
    }
}

fn plan_from_rendered(
    request: &ContainerPipelineCurrentizationRequest,
    rendered: &[RenderedPipeline],
) -> Result<ContainerPipelineCurrentizationPlan> {
    Ok(ContainerPipelineCurrentizationPlan {
        inventory_sha256: bijux_dna_infra::hash_file_sha256(&request.inventory_path)
            .context("hash currentization inventory")?,
        repository_path: request
            .governance
            .paths
            .frozen_repositories
            .join(&request.repository_commit),
        repository_commit: request.repository_commit.clone(),
        entries: rendered.iter().map(|document| document.entry.clone()).collect(),
    })
}

fn pipeline_identity(contract: &ContainerPipelineContract) -> (&str, &str) {
    match contract {
        ContainerPipelineContract::Legacy(pipeline) => (&pipeline.run.id, &pipeline.run.tool),
        ContainerPipelineContract::Current(pipeline) => (&pipeline.run.id, &pipeline.run.tool),
    }
}

fn pipeline_image_identity(contract: &ContainerPipelineContract) -> (&Path, &str) {
    match contract {
        ContainerPipelineContract::Legacy(pipeline) => {
            (&pipeline.image.path, &pipeline.image.sha256)
        }
        ContainerPipelineContract::Current(pipeline) => {
            (&pipeline.image.path, &pipeline.image.sha256)
        }
    }
}

fn is_external_artifact_disposition(
    disposition: ContainerPipelineCurrentizationDisposition,
) -> bool {
    disposition == ContainerPipelineCurrentizationDisposition::ExternalArtifactRequired
}

fn rewrite_pipeline_identities(
    source: &str,
    repository_path: &Path,
    repository_commit: &str,
    image_path: &Path,
    image_sha256: &str,
) -> Result<String> {
    let source = replace_section_scalar(
        source,
        "repository",
        "path",
        &repository_path.display().to_string(),
    )?;
    let source = replace_section_scalar(&source, "repository", "commit", repository_commit)?;
    let source =
        replace_section_scalar(&source, "image", "path", &image_path.display().to_string())?;
    replace_section_scalar(&source, "image", "sha256", image_sha256)
}

fn replace_section_scalar(source: &str, section: &str, field: &str, value: &str) -> Result<String> {
    ensure!(
        !value.is_empty()
            && value.bytes().all(|byte| !matches!(byte, b'\n' | b'\r' | b'#' | b'\'' | b'"')),
        "currentization scalar is not safe plain YAML"
    );
    let section_line = format!("{section}:");
    let inline_section_prefix = format!("{section}: {{");
    let field_prefix = format!("  {field}:");
    let inline_field_prefix = format!("{field}: ");
    let mut inside = false;
    let mut replaced = 0;
    let mut lines = Vec::new();
    for line in source.lines() {
        if line.starts_with(&inline_section_prefix) && line.ends_with('}') {
            let fields = &line[inline_section_prefix.len()..line.len() - 1];
            let fields = fields
                .split(", ")
                .map(|entry| {
                    if entry.starts_with(&inline_field_prefix) {
                        replaced += 1;
                        format!("{inline_field_prefix}{value}")
                    } else {
                        entry.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("{inline_section_prefix}{fields}}}"));
            inside = false;
            continue;
        }
        if !line.starts_with(' ') {
            inside = line == section_line;
        }
        if inside && line.starts_with(&field_prefix) {
            lines.push(format!("{field_prefix} {value}"));
            replaced += 1;
        } else {
            lines.push(line.to_owned());
        }
    }
    if replaced != 1 {
        bail!("pipeline section {section:?} must contain exactly one scalar {field:?}");
    }
    let mut rendered = lines.join("\n");
    if source.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

fn validate_commit(commit: &str) -> Result<()> {
    ensure!(
        commit.len() == 40
            && commit.bytes().all(|byte| byte.is_ascii_hexdigit())
            && !commit.bytes().any(|byte| byte.is_ascii_uppercase()),
        "currentization repository commit must be full lowercase Git identity"
    );
    Ok(())
}
