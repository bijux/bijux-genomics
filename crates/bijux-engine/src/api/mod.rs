//! Stable, intended-for-use engine interfaces.

use anyhow::{anyhow, Result};
use bijux_environment::api::{PlatformSpec, RunnerKind};

pub use crate::core::composer::{
    load_registry, normalize_correct_tool_list, normalize_filter_tool_list,
    normalize_merge_tool_list, normalize_qc_post_tool_list, normalize_screen_tool_list,
    normalize_stats_tool_list, normalize_trim_tool_list, normalize_umi_tool_list,
    normalize_validate_tool_list,
};
pub use crate::core::types::{
    default_domain_registry, trace_enabled, Capability, DataArtifact, Dependency, DomainRegistry,
    ExecutionContext, ExecutionManifest, ExecutionPlan, ExplainExclusion, ExplainPlan, MetricSet,
    PipelineSpec, Policy, ReadSet, RunPlan, SequenceCollection, StageGraph, StageNode,
    StageRequirement, ToolInvocation,
};
pub use crate::core::types::{init_logging, StdoutLogger};
pub use crate::core::validator::validate_execution_outputs;
pub use crate::services::composer::paths::{
    bench_base_dir, bench_tools_dir, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
};
pub use crate::services::composer::replay;
pub use crate::services::executor::{
    docker_logs, docker_rm, docker_stats_mb, docker_wait, docker_wait_timeout, parse_mem_to_mb,
    resolve_image_for_run, run_merge_container, run_merge_container_with_timeout,
    run_multiqc_container, run_multiqc_container_with_timeout, run_tool_container,
    run_tool_container_with_timeout, run_validate_container, run_validate_container_with_timeout,
    ExecutionOutput, MergeExecutionOutput,
};
pub use crate::services::observer::{
    hash_file_sha256, input_fastq_stats, length_histogram, output_fastq_stats,
    parse_fastqvalidator_count, write_explain_plan, SeqkitMetrics,
};
pub use crate::services::stage_exec::{execute_stage_plan, StagePlan, StageResult};
pub use bijux_environment::api::ResolvedImage;

pub fn ensure_bench_runner(
    platform: &PlatformSpec,
    runner_override: Option<RunnerKind>,
) -> Result<RunnerKind> {
    let runner = runner_override.unwrap_or(platform.runner);
    if runner != RunnerKind::Docker {
        return Err(anyhow!("benchmarking supports docker only for now"));
    }
    Ok(runner)
}

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
