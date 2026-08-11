use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_containers::api::RuntimeKind;
use bijux_dna_core::contract::ExecutionStep;

pub(super) fn hash_inputs(inputs: &[PathBuf]) -> Result<Vec<String>> {
    if inputs.is_empty() {
        return Ok(Vec::new());
    }
    let mut hashes = Vec::with_capacity(inputs.len());
    for path in inputs {
        if !path.exists() {
            return Err(anyhow!("declared input path does not exist: {}", path.display()));
        }
        hashes.push(hash_path(path)?);
    }
    Ok(hashes)
}

pub(super) fn hash_path(path: &Path) -> Result<String> {
    Ok(bijux_dna_infra::hash_path_sha256(path)?)
}

pub(super) fn execution_pipeline_identity(step: &ExecutionStep) -> String {
    std::env::var("BIJUX_PIPELINE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| step.stage_id.to_string())
}

pub(super) fn execution_sample_identity(step: &ExecutionStep) -> String {
    std::env::var("BIJUX_SAMPLE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| step.step_id.to_string())
}

pub(super) fn runtime_platform_identity(runner: RuntimeKind) -> String {
    std::env::var("BIJUX_PLATFORM")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| match runner {
            RuntimeKind::Local => "local".to_string(),
            RuntimeKind::Docker => "docker".to_string(),
            RuntimeKind::Apptainer => "apptainer".to_string(),
            RuntimeKind::Singularity => "singularity".to_string(),
        })
}

pub(super) fn infer_tool_version_from_image(image: &str) -> String {
    let without_digest = image.split('@').next().unwrap_or(image);
    if let Some((_, tag)) = without_digest.rsplit_once(':') {
        if !tag.is_empty() && tag != "latest" && !tag.contains('/') {
            return tag.to_string();
        }
    }
    "unknown".to_string()
}
