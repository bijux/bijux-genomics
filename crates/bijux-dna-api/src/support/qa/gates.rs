use anyhow::{anyhow, Result};
use bijux_dna_containers::api::{PlatformSpec, ToolImageCatalog};

/// Compatibility hooks validate stage identity without running image QA.
///
/// These hooks do not inspect QA records or execute probes. Image and tool QA
/// are explicit operations owned by `bijux-dna-containers-qa`.
pub fn ensure_image_qa_passed(
    stage: &str,
    _tools: &[String],
    _platform: &PlatformSpec,
    _catalog: &impl ToolImageCatalog,
) -> Result<()> {
    if stage.trim().is_empty() {
        return Err(anyhow!("stage id cannot be empty"));
    }
    Ok(())
}

/// See `ensure_image_qa_passed`.
pub fn ensure_tool_qa_passed(
    stage: &str,
    _tools: &[String],
    _platform: &PlatformSpec,
    _catalog: &impl ToolImageCatalog,
) -> Result<()> {
    if stage.trim().is_empty() {
        return Err(anyhow!("stage id cannot be empty"));
    }
    Ok(())
}
