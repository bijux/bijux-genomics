use std::collections::HashMap;

use anyhow::Result;
use bijux_environment::api::{PlatformSpec, ToolImageSpec};

/// QA hooks are a no-op in production API builds.
///
/// The QA crate carries heavy dependencies and is intentionally kept out of the
/// runtime dependency graph.
#[allow(clippy::unnecessary_wraps)]
pub fn ensure_image_qa_passed<S: ::std::hash::BuildHasher>(
    _stage: &str,
    _tools: &[String],
    _platform: &PlatformSpec,
    _catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    Ok(())
}

/// See `ensure_image_qa_passed`.
#[allow(clippy::unnecessary_wraps)]
pub fn ensure_tool_qa_passed<S: ::std::hash::BuildHasher>(
    _stage: &str,
    _tools: &[String],
    _platform: &PlatformSpec,
    _catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    Ok(())
}

/// Stubbed image QA runner for API surface parity.
///
/// # Errors
/// Returns `Ok(())` in production builds; QA enforcement lives in the QA crate.
pub fn run_image_qa(_platform_name: Option<&str>) -> Result<()> {
    Ok(())
}
