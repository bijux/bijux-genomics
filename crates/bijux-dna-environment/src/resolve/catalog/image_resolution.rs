use super::super::{EnvError, PlatformSpec, ResolvedImage, ToolImageSpec};

/// Resolve an image reference for a tool and platform.
///
/// # Errors
/// Returns an error if the tool name violates image naming rules.
pub(super) fn resolve_image(
    tool: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage, EnvError> {
    if tool.tool.to_lowercase().contains("base") {
        return Err(EnvError::Image(format!(
            "tool image name must not reference base: {}",
            tool.tool
        )));
    }
    let full_name = if let Some(digest) = tool.digest.as_ref() {
        format!("{}/{}@{}", platform.image_prefix, tool.tool, digest)
    } else {
        format!("{}/{}:{}-{}", platform.image_prefix, tool.tool, tool.version, platform.arch)
    };
    Ok(ResolvedImage { full_name, arch: platform.arch.clone(), runner: platform.runner })
}
