#[derive(Debug)]
pub(crate) struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_fastq: Option<std::path::PathBuf>,
    pub command: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct TrimExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_r1: std::path::PathBuf,
    pub output_r2: Option<std::path::PathBuf>,
    pub command: String,
}
