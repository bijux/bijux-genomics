#[allow(dead_code)]
#[derive(Debug)]
pub struct MergeExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub merged_fastq: std::path::PathBuf,
    pub unmerged_r1: std::path::PathBuf,
    pub unmerged_r2: std::path::PathBuf,
    pub command: String,
}
