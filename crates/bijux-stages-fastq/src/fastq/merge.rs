use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.merge";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergePlan {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: std::path::PathBuf,
    pub output: std::path::PathBuf,
}

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a merge plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_merge(tool: &str, r1: &Path, r2: &Path, out_dir: &Path) -> Result<MergePlan> {
    let output_name =
        merge_output_name(tool).ok_or_else(|| anyhow!("unsupported merge tool: {tool}"))?;
    Ok(MergePlan {
        tool: tool.to_string(),
        r1: r1.to_path_buf(),
        r2: r2.to_path_buf(),
        output: out_dir.join(output_name),
    })
}

fn merge_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "pear" => Some("pear.fastq.gz"),
        "vsearch" => Some("vsearch.fastq.gz"),
        "bbmerge" => Some("bbmerge.fastq.gz"),
        "flash2" => Some("flash2.fastq.gz"),
        _ => None,
    }
}

impl StagePlan for MergePlan {
    fn stage_id(&self) -> StageId {
        StageId(STAGE_ID.to_string())
    }

    fn stage_version(&self) -> StageVersion {
        STAGE_VERSION
    }

    fn outputs(&self) -> StageIO {
        StageIO {
            inputs: vec![
                ArtifactRef {
                    name: "reads_r1".to_string(),
                    path: self.r1.clone(),
                },
                ArtifactRef {
                    name: "reads_r2".to_string(),
                    path: self.r2.clone(),
                },
            ],
            outputs: vec![ArtifactRef {
                name: "merged_reads".to_string(),
                path: self.output.clone(),
            }],
        }
    }

    fn parameters_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tool": self.tool,
            "r1": self.r1,
            "r2": self.r2,
            "output": self.output
        })
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
