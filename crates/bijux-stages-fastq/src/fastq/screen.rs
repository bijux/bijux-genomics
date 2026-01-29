use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.screen";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenPlan {
    pub tool: String,
    pub input: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
    pub report: std::path::PathBuf,
}

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
pub fn plan_screen(tool: &str, r1: &Path, out_dir: &Path) -> Result<ScreenPlan> {
    normalize_screen_tool_list(&[tool.to_string()])?;
    let report = out_dir.join("screen_report.tsv");
    Ok(ScreenPlan {
        tool: tool.to_string(),
        input: r1.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
        report,
    })
}

impl StagePlan for ScreenPlan {
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
            outputs: vec![ArtifactRef {
                name: "screen_report_tsv".to_string(),
                path: self.report.clone(),
            }],
        }
    }

    fn parameters_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tool": self.tool,
            "input": self.input,
            "out_dir": self.out_dir,
            "report": self.report
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
