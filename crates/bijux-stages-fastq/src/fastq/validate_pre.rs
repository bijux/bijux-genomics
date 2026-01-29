use std::path::Path;

use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.validate_pre";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatePrePlan {
    pub tool: String,
    pub input: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidatePreUserConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct ValidatePreEffectiveConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

pub fn plan(tool: &str, r1: &Path, out_dir: &Path) -> ValidatePrePlan {
    ValidatePrePlan {
        tool: tool.to_string(),
        input: r1.to_path_buf(),
        out_dir: out_dir.to_path_buf(),
    }
}

pub fn resolve_config(user: ValidatePreUserConfig) -> ValidatePreEffectiveConfig {
    ValidatePreEffectiveConfig {
        tool: user.tool,
        r1: user.r1,
        out_dir: user.out_dir,
    }
}

pub fn plan_from_config(config: &ValidatePreEffectiveConfig) -> ValidatePrePlan {
    plan(&config.tool, &config.r1, &config.out_dir)
}

impl StagePlan for ValidatePrePlan {
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
