use std::path::Path;

use anyhow::Result;
use bijux_environment::api::ResolvedImage;

use super::plan::plan_validate_execution;
use super::run_tool::ExecutionOutput;
use super::tool_runner::{DockerToolRunner, ToolRunner};

pub fn run_validate_container(
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
    let plan = plan_validate_execution(tool, &input_path, out_dir)?;
    let runner = DockerToolRunner;
    runner.run(&plan, image, r1_dir, out_dir, container_name)
}

pub fn run_validate_container_with_timeout(
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
    let plan = plan_validate_execution(tool, &input_path, out_dir)?;
    let runner = DockerToolRunner;
    runner.run_with_timeout(&plan, image, r1_dir, out_dir, container_name, timeout)
}

pub fn run_multiqc_container(
    image: &ResolvedImage,
    input_dir: &Path,
    out_dir: &Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    let plan = plan_validate_execution("multiqc", "/data/input", out_dir)?;
    let runner = DockerToolRunner;
    runner.run(&plan, image, input_dir, out_dir, container_name)
}

pub fn run_multiqc_container_with_timeout(
    image: &ResolvedImage,
    input_dir: &Path,
    out_dir: &Path,
    container_name: &str,
    timeout: std::time::Duration,
) -> Result<ExecutionOutput> {
    let plan = plan_validate_execution("multiqc", "/data/input", out_dir)?;
    let runner = DockerToolRunner;
    runner.run_with_timeout(&plan, image, input_dir, out_dir, container_name, timeout)
}
