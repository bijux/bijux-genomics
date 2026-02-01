//! Owner: bijux-engine
//! Stable, intended-for-use engine interfaces.

use anyhow::{anyhow, Result};
use bijux_core::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1, ToolId, ToolRole};
use std::collections::HashMap;
use std::path::Path;

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
    resolve_image_for_run, run_filter_container, run_merge_container,
    run_merge_container_with_timeout, run_multiqc_container, run_multiqc_container_with_timeout,
    run_tool_container, run_tool_container_with_timeout, run_validate_container,
    run_validate_container_with_timeout, ExecutionOutput, MergeExecutionOutput,
};
pub use crate::services::observer::{
    hash_file_sha256, input_fastq_stats, length_histogram, output_fastq_stats,
    parse_fastqvalidator_count, write_explain_plan, Observer, SeqkitMetrics,
};
pub use crate::services::run_artifacts::{
    compute_run_id, params_hash, prepare_tool_run_dirs, run_artifacts_dir_for_out,
    tool_run_artifacts_dir, write_execution_logs, write_metrics_envelope, write_metrics_json,
    write_retention_report_placeholder, write_run_manifest, write_stage_plan_json,
    MetricsEnvelopeV1, RunArtifactInput, RunDirs,
};
pub use crate::services::stage_exec::{execute_stage_plan, StageResultV1};
pub use crate::services::telemetry::{build_telemetry_adapter, TelemetryAdapter, TelemetrySpan};
pub use bijux_core::StagePlanV1;
pub use bijux_core::{
    EffectiveConfigV1, FactsRowV1, RetentionReportV1, StageReportV1, TelemetryEventV1,
};
pub use bijux_env_builder::image_qa::{ensure_image_qa_passed, ensure_tool_qa_passed};
pub use bijux_env_runtime::api::{PlatformSpec, ResolvedImage, RunnerKind, ToolImageSpec};

pub fn write_telemetry_event(path: &Path, event: &TelemetryEventV1) -> Result<()> {
    crate::services::run_artifacts::write_telemetry_event(path, event)
}

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

pub fn filter_tools_by_role(
    stage_id: &str,
    tools: &[String],
    registry: &bijux_core::ToolRegistry,
    strict: bool,
) -> Result<Vec<String>> {
    let allow_silver = std::env::var("BIJUX_ALLOW_SILVER").is_ok();
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let mut filtered = Vec::new();
    for tool in tools {
        let manifest = registry
            .tool_by_id(stage_id, tool)
            .ok_or_else(|| anyhow!("tool {tool} missing from manifests"))?;
        let tier = match manifest.role {
            ToolRole::Authoritative => "gold",
            ToolRole::Diagnostic => "silver",
            ToolRole::Experimental => "experimental",
        };
        let allowed = match tier {
            "gold" => true,
            "silver" => allow_silver || allow_experimental,
            "experimental" => allow_experimental,
            _ => false,
        };
        if allowed {
            filtered.push(tool.clone());
        } else if strict {
            return Err(anyhow!(
                "tool {tool} is {tier}; enable --allow-silver or --allow-experimental"
            ));
        }
    }
    if filtered.is_empty() {
        return Err(anyhow!("no tools available after role filtering"));
    }
    Ok(filtered)
}

pub fn build_tool_execution_spec<S: ::std::hash::BuildHasher>(
    stage_id: &str,
    tool_id: &str,
    registry: &bijux_core::ToolRegistry,
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<ToolExecutionSpecV1> {
    let manifest = registry
        .tool_by_id(stage_id, tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from manifest for {stage_id}"))?;
    let spec = catalog
        .get(tool_id)
        .ok_or_else(|| anyhow!("tool {tool_id} missing from images.yaml"))?;
    let image = resolve_image_for_run(spec, platform)?;
    Ok(ToolExecutionSpecV1 {
        tool_id: ToolId(tool_id.to_string()),
        tool_version: spec.version.clone(),
        image: ContainerImageRefV1 {
            image: image.full_name,
            digest: spec.digest.clone(),
        },
        command: CommandSpecV1 {
            template: manifest.command_template.clone(),
        },
        resources: manifest.constraints.clone(),
    })
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

/// Execute a stage plan through the public engine API.
///
/// # Errors
/// Returns an error if the execution fails or the plan is invalid.
pub fn execute_plan(
    plan: &StagePlanV1,
    runner: RunnerKind,
    observer: Option<&mut dyn Observer>,
) -> Result<StageResultV1> {
    execute_stage_plan(plan, runner, observer)
}
