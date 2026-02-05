use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionManifest {
    pub run_id: String,
    pub stage: String,
    pub tool: String,
    pub tool_version: String,
    pub image_digest: String,
    pub command: String,
    pub input_hashes: Vec<String>,
    pub input_files: Vec<String>,
    pub output_dir: String,
    pub runner: String,
    pub platform: String,
    pub arch: String,
}
