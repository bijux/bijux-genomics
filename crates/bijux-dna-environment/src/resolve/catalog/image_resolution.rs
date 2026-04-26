use super::super::{EnvError, PlatformSpec, ResolvedImage, ToolImageSpec};

/// Resolve an image reference for a tool and platform.
///
/// # Errors
/// Returns an error if the tool name violates image naming rules.
pub(super) fn resolve_image(
    tool: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage, EnvError> {
    if tool.tool.trim().is_empty() {
        return Err(EnvError::Image("tool image name is empty".to_string()));
    }
    if platform.image_prefix.trim().is_empty() {
        return Err(EnvError::Image("platform image_prefix is empty".to_string()));
    }
    if platform.arch.trim().is_empty() {
        return Err(EnvError::Image("platform arch is empty".to_string()));
    }
    if tool.tool.to_lowercase().contains("base") {
        return Err(EnvError::Image(format!(
            "tool image name must not reference base: {}",
            tool.tool
        )));
    }
    let full_name = if let Some(digest) = tool.digest.as_ref() {
        if digest.trim().is_empty() {
            return Err(EnvError::Image(format!("empty digest for tool {}", tool.tool)));
        }
        format!("{}/{}@{}", platform.image_prefix, tool.tool, digest)
    } else {
        if tool.version.trim().is_empty() {
            return Err(EnvError::Image(format!("empty version for tool {}", tool.tool)));
        }
        format!("{}/{}:{}-{}", platform.image_prefix, tool.tool, tool.version, platform.arch)
    };
    Ok(ResolvedImage { full_name, arch: platform.arch.clone(), runner: platform.runner })
}
