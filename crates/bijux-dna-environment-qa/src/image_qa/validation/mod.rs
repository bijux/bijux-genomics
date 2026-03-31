mod input_inventory;
mod pass_requirements;

use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::api::{PlatformSpec, ToolImageSpec};
use input_inventory::expected_input_hashes;
use pass_requirements::ensure_image_qa_inputs_passed;

/// Ensure required image QA results exist for a stage/tool set.
///
/// # Errors
/// Returns an error if QA results are missing or cannot be read.
pub fn ensure_image_qa_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    if std::env::var("BIJUX_SKIP_QA").is_ok() {
        return Ok(());
    }
    let expected_inputs = expected_input_hashes(stage, platform)?;
    ensure_image_qa_inputs_passed(stage, tools, platform, catalog, &expected_inputs)
}

/// Ensure tool-level QA results exist for a stage/tool set.
///
/// # Errors
/// Returns an error if QA results are missing or cannot be read.
pub fn ensure_tool_qa_passed<S: ::std::hash::BuildHasher>(
    stage: &str,
    tools: &[String],
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec, S>,
) -> Result<()> {
    ensure_image_qa_passed(stage, tools, platform, catalog).map_err(|err| {
        anyhow!(
            "tool QA failed or missing for {stage}; run `bijux image-qa --platform {}`: {err}",
            platform.name
        )
    })
}
