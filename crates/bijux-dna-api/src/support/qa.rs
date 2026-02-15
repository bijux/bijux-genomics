use std::collections::HashMap;

use anyhow::{anyhow, Result};
use bijux_dna_environment::api::{PlatformSpec, ToolImageSpec};

/// QA hooks are a no-op in production API builds.
///
/// The QA crate carries heavy dependencies and is intentionally kept out of the
/// runtime dependency graph.
pub fn ensure_image_qa_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    _tools: &[String],
    _platform: &PlatformSpec,
    _catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    if stage.trim().is_empty() {
        return Err(anyhow!("stage id cannot be empty"));
    }
    Ok(())
}

/// See `ensure_image_qa_passed`.
pub fn ensure_tool_qa_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    _tools: &[String],
    _platform: &PlatformSpec,
    _catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    if stage.trim().is_empty() {
        return Err(anyhow!("stage id cannot be empty"));
    }
    Ok(())
}

/// Stubbed image QA runner for API surface parity.
///
/// # Errors
/// Returns `Ok(())` in production builds; QA enforcement lives in the QA crate.
pub fn run_image_qa(_platform_name: Option<&str>) -> Result<()> {
    Ok(())
}
