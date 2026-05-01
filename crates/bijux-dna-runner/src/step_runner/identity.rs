use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_environment::api::RuntimeKind;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

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
    if path.is_file() {
        return Ok(bijux_dna_infra::hash_file_sha256(path)?);
    }
    if path.is_dir() {
        return hash_directory(path);
    }
    Err(anyhow!("unsupported hash input path type: {}", path.display()))
}

fn hash_directory(root: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut entries = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("walk directory for hashing")?;
    entries.sort_by(|left, right| left.path().cmp(right.path()));

    for entry in entries {
        let path = entry.path();
        if path == root {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("strip directory prefix for {}", path.display()))?;
        hasher.update(relative.to_string_lossy().as_bytes());
        if entry.file_type().is_dir() {
            hasher.update(b"\0dir\0");
            continue;
        }
        if entry.file_type().is_symlink() {
            hasher.update(b"\0symlink\0");
            let target = std::fs::read_link(path)
                .with_context(|| format!("read directory hash symlink {}", path.display()))?;
            hasher.update(target.to_string_lossy().as_bytes());
            continue;
        }
        hasher.update(b"\0file\0");
        hasher.update(bijux_dna_infra::hash_file_sha256(path)?.as_bytes());
    }

    Ok(sha256_hex(hasher.finalize()))
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

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
