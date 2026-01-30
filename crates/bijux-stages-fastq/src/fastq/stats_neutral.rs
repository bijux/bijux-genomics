use std::path::Path;

use anyhow::Result;
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.stats_neutral";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatsNeutralPlan {
    pub tool: String,
    pub input: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

/// Build a stats_neutral plan.
///
/// # Errors
/// Returns an error if planning fails.
pub fn plan_stats_neutral(tool: &str, r1: &Path, out_dir: &Path) -> Result<StatsNeutralPlan> {
    Ok(StatsNeutralPlan {
        tool: tool.to_string(),
        input: r1.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
    })
}

impl StagePlan for StatsNeutralPlan {
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
