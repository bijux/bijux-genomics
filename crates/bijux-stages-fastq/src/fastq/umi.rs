use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.umi";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UmiPlan {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
    pub output_r1: std::path::PathBuf,
    pub output_r2: std::path::PathBuf,
}

pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a UMI plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_umi(tool: &str, r1: &Path, r2: &Path, out_dir: &Path) -> Result<UmiPlan> {
    normalize_umi_tool_list(&[tool.to_string()])?;
    Ok(UmiPlan {
        tool: tool.to_string(),
        r1: r1.to_path_buf(),
        r2: r2.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        output_r1: out_dir.join("reads_r1.fastq.gz"),
        output_r2: out_dir.join("reads_r2.fastq.gz"),
    })
}

impl StagePlan for UmiPlan {
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
            outputs: vec![
                ArtifactRef {
                    name: "dedup_reads_r1".to_string(),
                    path: self.output_r1.clone(),
                },
                ArtifactRef {
                    name: "dedup_reads_r2".to_string(),
                    path: self.output_r2.clone(),
                },
            ],
        }
    }

    fn parameters_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tool": self.tool,
            "r1": self.r1,
            "r2": self.r2,
            "out_dir": self.out_dir,
            "output_r1": self.output_r1,
            "output_r2": self.output_r2
        })
    }
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
