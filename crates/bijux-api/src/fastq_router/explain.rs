use std::fmt::Write as _;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_core::{ExplainExclusion, ExplainPlan};

/// Write a human-readable plan explanation.
///
/// # Errors
/// Returns an error if the markdown file cannot be written.
pub fn write_explain_md(
    base_dir: &Path,
    stage: &str,
    selected_tools: &[String],
    excluded_tools: &[String],
    policy: Option<&str>,
) -> Result<()> {
    let path = base_dir.join("explain.md");
    let mut contents = String::new();
    writeln!(contents, "# Explain: {stage}\n")?;
    if let Some(policy) = policy {
        writeln!(contents, "policy: {policy}\n")?;
    }
    contents.push_str("selected tools:\n");
    for tool in selected_tools {
        writeln!(contents, "- {tool}")?;
    }
    if !excluded_tools.is_empty() {
        contents.push_str("\nexcluded tools:\n");
        for tool in excluded_tools {
            writeln!(contents, "- {tool}")?;
        }
    }
    bijux_infra::atomic_write_bytes(&path, contents.as_bytes()).context("write explain.md")?;
    Ok(())
}

/// Write the JSON explain plan.
///
/// # Errors
/// Returns an error if the json plan cannot be written.
pub fn write_explain_plan_json(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    registry: &bijux_core::ToolRegistry,
    _policy: Option<&str>,
) -> Result<()> {
    let mut excluded = Vec::new();
    for tool in registry.tools_for_stage(stage) {
        if !selected.iter().any(|t| t == &tool.tool_id) {
            excluded.push(ExplainExclusion {
                tool: tool.tool_id.clone(),
                reason: "not selected".to_string(),
            });
        }
    }
    let invariants = vec![
        "stage_contract".to_string(),
        "header_inspection".to_string(),
        "output_normalization".to_string(),
    ];
    let plan = ExplainPlan {
        stage: stage.to_string(),
        selected_tools: selected.to_vec(),
        excluded_tools: excluded,
        policy: None,
        invariants,
    };
    let path = base_dir.join("explain_plan.json");
    let payload = serde_json::to_vec_pretty(&plan)?;
    bijux_infra::atomic_write_bytes(&path, &payload).context("write explain_plan.json")?;
    Ok(())
}
