use std::path::PathBuf;

use super::cache_dir;
use crate::resolve::{ResolvedImage, RuntimeKind};

pub(in crate::resolve) fn docker_image_exists_with<F>(image: &ResolvedImage, runner: F) -> bool
where
    F: Fn(&[&str]) -> bool,
{
    if image.runner != RuntimeKind::Docker {
        return false;
    }
    runner(&["image", "inspect", &image.full_name])
}

#[must_use]
pub(in crate::resolve) fn apptainer_sif_path(image: &ResolvedImage) -> PathBuf {
    let cache = cache_dir(RuntimeKind::Apptainer);
    let tool = extract_tool_name(&image.full_name);
    let version_or_digest = extract_version_or_digest(&image.full_name, &image.arch);
    cache.join(format!("{}-{}-{}.sif", tool, version_or_digest, image.arch))
}

fn extract_tool_name(full_name: &str) -> String {
    let without_prefix = full_name.rsplit_once('/').map_or(full_name, |(_, tool)| tool);
    let tool = without_prefix.split(['@', ':']).next().unwrap_or(without_prefix);
    tool.to_string()
}

fn extract_version_or_digest(full_name: &str, arch: &str) -> String {
    if let Some((_, digest)) = full_name.rsplit_once('@') {
        digest.to_string()
    } else if let Some((_, tag)) = full_name.rsplit_once(':') {
        tag.strip_suffix(&format!("-{arch}")).unwrap_or(tag).to_string()
    } else {
        "unknown".to_string()
    }
}
