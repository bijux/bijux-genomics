use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StageResultV1 {
    pub run_id: String,
    pub exit_code: i32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub outputs: Vec<PathBuf>,
    pub metrics_path: Option<PathBuf>,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}
