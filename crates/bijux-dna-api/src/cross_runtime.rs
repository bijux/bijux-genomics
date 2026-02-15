use anyhow::{anyhow, Result};
use bijux_dna_environment::api::RuntimeKind;

use crate::execution_kernel::{invoke_tool, ToolInvocationRequest};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeParityResult {
    pub primary_runtime: String,
    pub secondary_runtime: String,
    pub primary_exit: i32,
    pub secondary_exit: i32,
    pub matched: bool,
}

fn normalized_first_line(stdout: &str) -> String {
    stdout.lines().next().unwrap_or_default().trim().to_string()
}

/// # Errors
/// Returns an error if either invocation fails or if parity does not hold.
pub fn check_invocation_parity(
    request: &ToolInvocationRequest,
    secondary_runtime: RuntimeKind,
) -> Result<RuntimeParityResult> {
    let primary = invoke_tool(request)?;
    let mut secondary_request = request.clone();
    secondary_request.runner = secondary_runtime;
    secondary_request.context.stage_root = request
        .context
        .stage_root
        .join(format!("parity_{}", secondary_runtime));
    secondary_request.context.tmp_root = secondary_request.context.stage_root.join("tmp");
    let secondary = invoke_tool(&secondary_request)?;
    let matched = primary.stage_result.exit_code == secondary.stage_result.exit_code
        && normalized_first_line(&primary.stage_result.stdout)
            == normalized_first_line(&secondary.stage_result.stdout);
    if !matched {
        return Err(anyhow!(
            "cross-runtime parity mismatch for {} ({} vs {})",
            request.context.tool_id,
            request.runner,
            secondary_runtime
        ));
    }
    Ok(RuntimeParityResult {
        primary_runtime: request.runner.to_string(),
        secondary_runtime: secondary_runtime.to_string(),
        primary_exit: primary.stage_result.exit_code,
        secondary_exit: secondary.stage_result.exit_code,
        matched,
    })
}
