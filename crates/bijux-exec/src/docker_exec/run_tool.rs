use std::path::Path;

use anyhow::Result;
use bijux_environment::api::ResolvedImage;

use super::plan::{plan_filter_execution, plan_tool_execution};
use super::tool_runner::{DockerToolRunner, ToolRunner};

#[derive(Debug)]
pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub output_fastq: Option<std::path::PathBuf>,
    pub command: String,
}

pub fn run_tool_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");
    let plan = plan_tool_execution(tool, &input_path, out_dir)?;
    let runner = DockerToolRunner;
    runner.run(&plan, image, r1_dir, out_dir, container_name)
}

pub fn run_filter_container(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
    params: &serde_json::Value,
) -> Result<ExecutionOutput> {
    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");
    let plan = plan_filter_execution(tool, &input_path, out_dir, params)?;
    let runner = DockerToolRunner;
    runner.run(&plan, image, r1_dir, out_dir, container_name)
}

pub fn run_tool_container_with_timeout(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &Path,
    r1: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<ExecutionOutput> {
    let r1_name = r1
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("r1 filename missing"))?
        .to_string_lossy()
        .to_string();
    let input_path = format!("/data/input/{r1_name}");
    let plan = plan_tool_execution(tool, &input_path, out_dir)?;
    let runner = DockerToolRunner;
    runner.run_with_timeout(&plan, image, r1_dir, out_dir, container_name, timeout)
}
