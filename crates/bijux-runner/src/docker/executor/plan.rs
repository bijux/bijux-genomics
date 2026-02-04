use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StageExecutionPlan {
    pub tool: String,
    pub container_args: Vec<String>,
    pub expected_outputs: Vec<PathBuf>,
    pub output_fastq: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}
