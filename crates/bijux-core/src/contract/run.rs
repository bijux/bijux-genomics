use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::contract::tooling::PathSpec;
use crate::ids::{RunId, StageId, ToolId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub container_runtime: String,
    pub default_threads: u32,
    pub default_mem_gb: u32,
    pub default_time_minutes: u32,
    pub run_base_dir: PathBuf,
    pub image_pull_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSpec {
    pub stage: StageId,
    pub tool: ToolId,
    pub paths: PathSpec,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

#[must_use]
pub fn run_dir(base_dir: &Path, run_id: &RunId, _stage: &StageId, _tool: &ToolId) -> PathBuf {
    base_dir.join(&run_id.0)
}
