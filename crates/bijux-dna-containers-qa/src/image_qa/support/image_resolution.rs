use anyhow::{anyhow, Result};
use tracing::warn;

pub use crate::api::ResolvedImage;
use crate::api::{docker_image_exists, resolve_image, PlatformSpec, ToolImageSpec};

pub fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!("digest image missing locally; falling back to tag {}", fallback.full_name);
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
}

/// Keep the configured pin key only when that exact primary image was admitted.
pub(crate) fn resolved_image_evidence_identity(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
    image: &ResolvedImage,
) -> Result<String> {
    let primary = resolve_image(spec, platform)?;
    Ok(if image.full_name == primary.full_name {
        spec.digest.as_ref().map_or_else(|| image.full_name.clone(), ToString::to_string)
    } else {
        image.full_name.clone()
    })
}
