use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_environment::api::{PlatformSpec, RunnerKind};
use serde::{Deserialize, Serialize};

mod logging;
pub use logging::{init_logging, StdoutLogger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub platform: PlatformSpec,
    pub runner_override: Option<RunnerKind>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub stage_id: String,
    pub tool_id: String,
    pub inputs: Vec<PathBuf>,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPlan {
    pub invocation: ToolInvocation,
    pub image_digest: String,
    pub runner: RunnerKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub invocation: ToolInvocation,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSet {
    pub schema: String,
    pub metrics: serde_json::Value,
}

pub fn trace_enabled() -> bool {
    std::env::var("BIJUX_TRACE_ENGINE").is_ok()
}
