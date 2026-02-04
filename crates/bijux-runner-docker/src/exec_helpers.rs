use anyhow::Result;
use bijux_env_runtime::api::ResolvedImage;

use crate::executor::{
    docker_rm, docker_stats_mb, run_filter_container, run_merge_container, run_multiqc_container,
    run_tool_container, run_validate_container, ExecutionOutput, MergeExecutionOutput,
};

pub fn run_tool_execution(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &std::path::Path,
    r1: &std::path::Path,
    out_dir: &std::path::Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    run_tool_container(tool, image, r1_dir, r1, out_dir, container_name)
}

pub fn run_filter_execution(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &std::path::Path,
    r1: &std::path::Path,
    out_dir: &std::path::Path,
    container_name: &str,
    params: &serde_json::Value,
) -> Result<ExecutionOutput> {
    run_filter_container(tool, image, r1_dir, r1, out_dir, container_name, params)
}

pub fn run_validate_execution(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &std::path::Path,
    r1: &std::path::Path,
    out_dir: &std::path::Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    run_validate_container(tool, image, r1_dir, r1, out_dir, container_name)
}

pub fn run_multiqc_execution(
    image: &ResolvedImage,
    input_dir: &std::path::Path,
    out_dir: &std::path::Path,
    container_name: &str,
) -> Result<ExecutionOutput> {
    run_multiqc_container(image, input_dir, out_dir, container_name)
}

pub fn run_merge_execution(
    tool: &str,
    image: &ResolvedImage,
    r1_dir: &std::path::Path,
    r1: &std::path::Path,
    r2: &std::path::Path,
    out_dir: &std::path::Path,
    container_name: &str,
) -> Result<MergeExecutionOutput> {
    run_merge_container(tool, image, r1_dir, r1, r2, out_dir, container_name)
}

pub fn execution_memory_mb(container_name: &str) -> Result<f64> {
    docker_stats_mb(container_name)
}

pub fn cleanup_execution(container_name: &str) -> Result<()> {
    docker_rm(container_name)
}
