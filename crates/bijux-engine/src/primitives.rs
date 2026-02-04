//! Owner: bijux-engine
//! Low-level engine primitives. Prefer bijux-api for orchestration.

use anyhow::{anyhow, Result};
use bijux_core::ToolRole;
use std::path::Path;

pub use crate::core::composer::load_registry;
pub use crate::core::types::{
    default_domain_registry, trace_enabled, Capability, DataArtifact, Dependency, DomainRegistry,
    ExecutionContext, ExecutionManifest, ExplainExclusion, ExplainPlan, MetricSet, Policy, ReadSet,
    RunPlan, SequenceCollection, StageGraph, StageNode, StageRequirement, ToolInvocation,
};
pub use crate::core::types::{init_logging, StdoutLogger};
pub use crate::core::validator::validate_execution_outputs;
pub use crate::services::composer::paths::{
    bench_base_dir, bench_tools_dir, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
};
pub use crate::services::run_artifacts::{
    compute_run_id, prepare_tool_run_dirs, run_artifacts_dir_for_out,
    tool_run_artifacts_dir, write_execution_logs, write_metrics_envelope, write_metrics_json,
    write_retention_report_placeholder, write_run_manifest, write_scientific_provenance,
    write_stage_plan_json,
    MetricsEnvelopeV1, RunArtifactInput, RunDirs,
};
pub use bijux_core::params_hash;
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
    _stage_id: &str,
    _tool_id: &str,
    _registry: &bijux_core::ToolRegistry,
    _catalog: &std::collections::HashMap<String, ToolImageSpec, S>,
    _platform: &PlatformSpec,
) -> Result<bijux_core::ToolExecutionSpecV1> {
    Err(anyhow!(
        "build_tool_execution_spec moved to bijux-runner-docker"
    ))
}
