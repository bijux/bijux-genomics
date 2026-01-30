use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlan, StageVersion, ToolExecutionSpecV1};

pub const STAGE_ID: &str = "fastq.filter";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a filter plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_filter(tool: &ToolExecutionSpecV1, r1: &Path, out_dir: &Path) -> Result<StagePlan> {
    let output_name =
        filter_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported filter tool"))?;
    let output = out_dir.join(output_name);
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
                name: "filtered_reads".to_string(),
                path: output.clone(),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "output": output
        }),
        aux_images: std::collections::BTreeMap::new(),
    })
}

fn filter_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        _ => None,
    }
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}
