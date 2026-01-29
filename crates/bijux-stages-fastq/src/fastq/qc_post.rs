use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.qc_post";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QcPostPlan {
    pub tool: String,
    pub input: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

pub fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    normalize_tools_with_allowlist(tools, &allowed)
}

#[must_use]
pub fn aux_tool_ids() -> &'static [&'static str] {
    &["fastqc"]
}

/// Build a qc_post plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_qc_post(tool: &str, r1: &Path, out_dir: &Path) -> Result<QcPostPlan> {
    if !normalize_qc_post_tool_list(&[tool.to_string()])?.is_empty() {
        return Ok(QcPostPlan {
            tool: tool.to_string(),
            input: r1.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
        });
    }
    Err(anyhow!("unsupported qc_post tool: {tool}"))
}

impl StagePlan for QcPostPlan {
    fn stage_id(&self) -> StageId {
        StageId(STAGE_ID.to_string())
    }

    fn stage_version(&self) -> StageVersion {
        STAGE_VERSION
    }

    fn outputs(&self) -> StageIO {
        StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: self.input.clone(),
            }],
            outputs: Vec::new(),
        }
    }

    fn parameters_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tool": self.tool,
            "input": self.input,
            "out_dir": self.out_dir
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
