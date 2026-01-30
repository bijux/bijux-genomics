use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};

pub const STAGE_ID: &str = "fastq.umi";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a UMI plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_umi(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    normalize_umi_tool_list(std::slice::from_ref(&tool.tool_id.0))?;
    let output_r1 = out_dir.join("reads_r1.fastq.gz");
    let output_r2 = out_dir.join("reads_r2.fastq.gz");
    Ok(StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![
                ArtifactRef {
                    name: "reads_r1".to_string(),
                    path: r1.to_path_buf(),
                },
                ArtifactRef {
                    name: "reads_r2".to_string(),
                    path: r2.to_path_buf(),
                },
            ],
            outputs: vec![
                ArtifactRef {
                    name: "dedup_reads_r1".to_string(),
                    path: output_r1.clone(),
                },
                ArtifactRef {
                    name: "dedup_reads_r2".to_string(),
                    path: output_r2.clone(),
                },
            ],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "r1": r1,
            "r2": r2,
            "out_dir": out_dir,
            "output_r1": output_r1,
            "output_r2": output_r2
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
