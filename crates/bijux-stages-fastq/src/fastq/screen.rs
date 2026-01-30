use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlan, StageVersion, ToolExecutionSpecV1};

pub const STAGE_ID: &str = "fastq.screen";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a screen plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_screen(tool: &ToolExecutionSpecV1, r1: &Path, out_dir: &Path) -> Result<StagePlan> {
    normalize_screen_tool_list(std::slice::from_ref(&tool.tool_id.0))?;
    let report = out_dir.join("screen_report.tsv");
    Ok(StagePlan {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs: vec![ArtifactRef {
                name: "screen_report_tsv".to_string(),
                path: report.clone(),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "out_dir": out_dir,
            "report": report
        }),
        aux_images: std::collections::BTreeMap::new(),
    })
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}
