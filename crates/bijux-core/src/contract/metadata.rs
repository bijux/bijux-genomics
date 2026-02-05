use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunMetadataV1 {
    pub schema_version: String,
    pub run_id: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub platform: String,
    pub runner: String,
    pub hostname: String,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationMetadataV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub executed_command: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageMetadataV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExecutionMetadataV1 {
    pub schema_version: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: String,
    pub exit_code: i32,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
}
