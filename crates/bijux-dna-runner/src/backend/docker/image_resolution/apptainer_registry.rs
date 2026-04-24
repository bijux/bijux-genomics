use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_environment::api::{PlatformSpec, ResolvedImage, ToolImageSpec};

pub(super) fn resolve_apptainer_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
    image: ResolvedImage,
) -> Result<ResolvedImage> {
    let candidates = apptainer_image_candidates(spec, platform);
    let image_path = candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());
    let resolved = ResolvedImage {
        full_name: image_path.display().to_string(),
        arch: image.arch,
        runner: image.runner,
    };
    if std::env::var("BIJUX_SKIP_IMAGE_CHECK").is_ok() || image_path.is_file() {
        return Ok(resolved);
    }
    Err(anyhow!(
        "apptainer image not found for tool {}. checked: {}",
        spec.tool,
        candidates.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
    ))
}

fn apptainer_image_candidates(spec: &ToolImageSpec, platform: &PlatformSpec) -> Vec<PathBuf> {
    let registry_root = apptainer_registry_root(&platform.container_dir);
    let mut candidates = vec![platform.container_dir.join(format!("{}.sif", spec.tool))];
    if let Some(digest) = spec.digest.as_deref() {
        let normalized_digest = digest.strip_prefix("sha256:").unwrap_or(digest);
        candidates.push(registry_root.join(&spec.tool).join(format!("{normalized_digest}.sif")));
        candidates.push(registry_root.join(&spec.tool).join(format!("{digest}.sif")));
    } else if let Some(unique_sif) = unique_registry_sif(&registry_root, &spec.tool) {
        candidates.push(unique_sif);
    }
    candidates.dedup();
    candidates
}

fn unique_registry_sif(registry_root: &Path, tool: &str) -> Option<PathBuf> {
    let tool_dir = registry_root.join(tool);
    let entries = fs::read_dir(&tool_dir).ok()?;
    let mut sifs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sif"))
        .collect::<Vec<_>>();
    sifs.sort();
    if sifs.len() == 1 {
        return sifs.into_iter().next();
    }
    None
}

pub(crate) fn apptainer_registry_root(container_dir: &Path) -> PathBuf {
    let parent = container_dir.parent();
    let grandparent = parent.and_then(Path::parent);
    let is_flat_apptainer_sif_dir =
        container_dir.file_name().and_then(|name| name.to_str()).is_some_and(|name| name == "sif")
            && parent
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "apptainer");
    if is_flat_apptainer_sif_dir {
        return grandparent.unwrap_or(container_dir).to_path_buf();
    }
    container_dir.to_path_buf()
}
