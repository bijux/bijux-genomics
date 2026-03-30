use std::path::PathBuf;

use super::{ResolvedImage, RuntimeKind};

#[must_use]
pub(super) fn cache_dir(runner: RuntimeKind) -> PathBuf {
    let cache_root = base_cache_root();
    match runner {
        RuntimeKind::Docker => cache_root.join("bijux").join("docker").join("images"),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            cache_root.join("bijux").join("apptainer").join("sif")
        }
    }
}

fn base_cache_root() -> PathBuf {
    std::env::var_os("BIJUX_CACHE_ROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_CACHE_HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map_or_else(|| PathBuf::from("."), PathBuf::from)
                .join(".cache")
        })
}

pub(super) fn docker_image_exists_with<F>(image: &ResolvedImage, runner: F) -> bool
where
    F: Fn(&[&str]) -> bool,
{
    runner(&["image", "inspect", &image.full_name])
}

#[must_use]
pub(super) fn apptainer_sif_path(image: &ResolvedImage) -> PathBuf {
    let cache = cache_dir(RuntimeKind::Apptainer);
    let tool = extract_tool_name(&image.full_name);
    let version_or_digest = extract_version_or_digest(&image.full_name, &image.arch);
    cache.join(format!("{}-{}-{}.sif", tool, version_or_digest, image.arch))
}

pub(super) fn reference_cache_dir() -> PathBuf {
    base_cache_root().join("bijux").join("references")
}

fn extract_tool_name(full_name: &str) -> String {
    let without_prefix = full_name.rsplit_once('/').map_or(full_name, |(_, t)| t);
    let tool = without_prefix
        .split(['@', ':'])
        .next()
        .unwrap_or(without_prefix);
    tool.to_string()
}

fn extract_version_or_digest(full_name: &str, arch: &str) -> String {
    if let Some((_, digest)) = full_name.rsplit_once('@') {
        digest.to_string()
    } else if let Some((_, tag)) = full_name.rsplit_once(':') {
        tag.strip_suffix(&format!("-{arch}"))
            .unwrap_or(tag)
            .to_string()
    } else {
        "unknown".to_string()
    }
}
