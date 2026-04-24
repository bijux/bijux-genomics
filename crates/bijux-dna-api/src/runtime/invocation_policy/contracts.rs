use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use bijux_dna_core::contract::ExecutionStep;

use super::{Result, ToolInvocationRequest};

pub(super) fn canonicalize_existing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path.canonicalize().with_context(|| format!("canonicalize {}", path.display()));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().context("resolve cwd for relative path contracts")?.join(path)
    };

    let mut missing_components = Vec::new();
    let mut ancestor = absolute.as_path();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            anyhow!(
                "cannot resolve non-existent path without existing ancestor: {}",
                absolute.display()
            )
        })?;
        missing_components.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            anyhow!("cannot resolve parent for non-existent path: {}", absolute.display())
        })?;
    }

    let mut resolved = ancestor
        .canonicalize()
        .with_context(|| format!("canonicalize ancestor {}", ancestor.display()))?;
    for component in missing_components.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

pub(super) fn ensure_subpath(path: &Path, root: &Path, label: &str) -> Result<()> {
    let cpath = canonicalize_existing(path)?;
    let croot = canonicalize_existing(root)?;
    if !cpath.starts_with(&croot) {
        bail!("{label} path contract violated: {} not under {}", cpath.display(), croot.display());
    }
    Ok(())
}

pub(crate) fn enforce_path_contracts(req: &ToolInvocationRequest) -> Result<()> {
    ensure_subpath(&req.context.stage_root, &req.context.output_root, "stage_root")?;
    ensure_subpath(&req.context.tmp_root, &req.context.output_root, "tmp_root")?;
    for artifact in &req.step.io.outputs {
        ensure_subpath(&artifact.path, &req.context.output_root, "output")?;
    }
    for artifact in &req.step.io.inputs {
        ensure_subpath(&artifact.path, &req.context.input_root, "input")?;
    }
    Ok(())
}

pub(crate) fn validate_required_outputs(step: &ExecutionStep) -> Result<()> {
    for artifact in &step.io.outputs {
        if artifact.optional {
            continue;
        }
        if !artifact.path.exists() {
            bail!(
                "stage contract violation: required output '{}' was not produced at {}",
                artifact.name,
                artifact.path.display()
            );
        }
    }
    Ok(())
}

pub(crate) fn enforce_large_file_guard(
    output_root: &Path,
    outputs: &[bijux_dna_core::contract::ArtifactSpec],
) -> Result<()> {
    let max_bytes: u64 = std::env::var("BIJUX_MAX_UNEXPECTED_FILE_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5 * 1024 * 1024 * 1024);
    let allowed = outputs
        .iter()
        .map(|artifact| canonicalize_existing(&artifact.path))
        .collect::<Result<Vec<_>>>()?;
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(output_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = canonicalize_existing(entry.path())?;
        if allowed.iter().any(|allowed_path| allowed_path == &path) {
            continue;
        }
        let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        if size > max_bytes {
            violations.push(format!("{} ({} bytes)", path.display(), size));
        }
    }
    if !violations.is_empty() {
        bail!(
            "large-file guard violation: unexpected files outside contract exceeded limit: {}",
            violations.join(", ")
        );
    }
    Ok(())
}
