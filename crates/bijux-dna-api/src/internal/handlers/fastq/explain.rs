use std::fmt::Write as _;
use std::path::Path;

use crate::explain::{ExplainExclusion, ExplainPlan, ExplainSelectionNote};
use anyhow::{Context, Result};

fn read_domain_snapshot_hash() -> Option<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let registry = root.join("configs/ci/registry/tool_registry.toml");
    let raw = std::fs::read_to_string(registry).ok()?;
    for line in raw.lines().take(8) {
        if let Some(rest) = line.strip_prefix("# source_commit: ") {
            let hash = rest.trim();
            if hash.len() == 40 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return Some(hash.to_string());
            }
        }
    }
    None
}

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
    bijux_dna_infra::atomic_write_bytes(&path, contents.as_bytes()).context("write explain.md")?;
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
    registry: &bijux_dna_core::contract::ToolRegistry,
    _policy: Option<&str>,
) -> Result<()> {
    let mut excluded = Vec::new();
    let stage_id = bijux_dna_core::ids::StageId::try_from(stage)
        .map_err(|err| anyhow::anyhow!("invalid stage id: {err}"))?;
    for tool in registry.tools_for_stage(&stage_id) {
        let tool_id = tool.tool_id.to_string();
        if !selected.iter().any(|t| t == &tool_id) {
            excluded.push(ExplainExclusion {
                tool: tool_id,
                reason: "not selected".to_string(),
            });
        }
    }
    let invariants = vec![
        "stage_contract".to_string(),
        "header_inspection".to_string(),
        "output_normalization".to_string(),
    ];
    let selection = selected
        .iter()
        .map(|tool| ExplainSelectionNote {
            tool: tool.clone(),
            reason: "selected by planner defaults and stage constraints".to_string(),
            provenance_notes: vec![
                format!("stage={stage}"),
                "registry=bijux_dna_core::contract::ToolRegistry".to_string(),
            ],
            comparability_notes: vec![
                "compare metrics only when stage outputs use the same schema".to_string(),
            ],
        })
        .collect();
    let plan = ExplainPlan {
        stage: stage.to_string(),
        domain_snapshot_hash: read_domain_snapshot_hash(),
        selected_tools: selected.to_vec(),
        tool_selection: selection,
        excluded_tools: excluded,
        policy: None,
        invariants,
    };
    let path = base_dir.join("explain_plan.json");
    let payload = serde_json::to_vec_pretty(&plan)?;
    bijux_dna_infra::atomic_write_bytes(&path, &payload).context("write explain_plan.json")?;
    Ok(())
}
