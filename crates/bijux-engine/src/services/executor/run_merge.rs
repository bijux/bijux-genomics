use std::path::Path;

use anyhow::Result;
use bijux_env_runtime::api::ResolvedImage;

use super::plan::plan_merge_execution;
use super::tool_runner::{DockerToolRunner, ToolRunner};

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

pub fn run_merge_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<MergeExecutionOutput> {
    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let r2_name = r2
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r2 filename missing"))?
        .to_string_lossy()
        .to_string();
    let r1_path = format!("/data/input/{r1_name}");
    let r2_path = format!("/data/input/{r2_name}");
    let plan = plan_merge_execution(tool, &r1_path, &r2_path, out_dir)?;
    let runner = DockerToolRunner;
    let output = runner.run(&plan, image, r1_dir, out_dir, container_name)?;
    let outputs = plan.expected_outputs.clone();
    let merged_fastq = outputs
        .first()
        .cloned()
        .unwrap_or_else(|| out_dir.join("merged.fastq"));
    let unmerged_r1 = outputs
        .get(1)
        .cloned()
        .unwrap_or_else(|| out_dir.join("unmerged_r1.fastq"));
    let unmerged_r2 = outputs
        .get(2)
        .cloned()
        .unwrap_or_else(|| out_dir.join("unmerged_r2.fastq"));
    Ok(MergeExecutionOutput {
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
        merged_fastq,
        unmerged_r1,
        unmerged_r2,
        command: output.command,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn run_merge_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    r2: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<MergeExecutionOutput> {
    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let r2_name = r2
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r2 filename missing"))?
        .to_string_lossy()
        .to_string();
    let r1_path = format!("/data/input/{r1_name}");
    let r2_path = format!("/data/input/{r2_name}");
    let plan = plan_merge_execution(tool, &r1_path, &r2_path, out_dir)?;
    let runner = DockerToolRunner;
    let output = runner.run_with_timeout(&plan, image, r1_dir, out_dir, container_name, timeout)?;
    let outputs = plan.expected_outputs.clone();
    let merged_fastq = outputs
        .first()
        .cloned()
        .unwrap_or_else(|| out_dir.join("merged.fastq"));
    let unmerged_r1 = outputs
        .get(1)
        .cloned()
        .unwrap_or_else(|| out_dir.join("unmerged_r1.fastq"));
    let unmerged_r2 = outputs
        .get(2)
        .cloned()
        .unwrap_or_else(|| out_dir.join("unmerged_r2.fastq"));
    Ok(MergeExecutionOutput {
        exit_code: output.exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
        merged_fastq,
        unmerged_r1,
        unmerged_r2,
        command: output.command,
    })
}
