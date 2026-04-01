use anyhow::{anyhow, Result};
use bijux_dna_environment::api::{PlatformSpec, ToolImageCatalog};

/// QA hooks are a no-op in production API builds.
///
/// The QA crate carries heavy dependencies and is intentionally kept out of the
/// runtime dependency graph.
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
