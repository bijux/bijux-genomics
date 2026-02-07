//! Local execution spec helpers.

use std::collections::BTreeMap;

use anyhow::Result;
use bijux_core::prelude::ToolExecutionSpecV1;

use crate::runner_core::invocation_hash;

#[must_use]
pub fn module_id() -> &'static str {
    "bijux-runner-local-execution-spec"
}

/// Compute a stable invocation hash for a local execution spec.
///
/// # Errors
/// Returns an error if canonical serialization fails.
pub fn invocation_hash_for_spec(
    spec: &ToolExecutionSpecV1,
    env: &BTreeMap<String, String>,
    input_hashes: &[String],
) -> Result<String> {
    let image_digest = spec
        .image
        .digest
        .clone()
        .unwrap_or_else(|| spec.image.image.clone());
    invocation_hash(&spec.command.template, env, &image_digest, input_hashes)
}
