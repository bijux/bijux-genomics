use anyhow::{Context, Result};

use crate::fastq_router::jobs::normalize_tool_spec_for_jobs;
use crate::fastq_router::summary::StageExecutionSummary;
use bijux_engine::api::{
    bench_tools_dir, build_tool_execution_spec, execute_plan, PlatformSpec, ToolImageSpec,
};

pub struct SmartPipelineResult {
    pub adapter_inference: Option<serde_json::Value>,
    pub adapter_bank_preset_override: Option<String>,
    pub(crate) preplanned_stage_runs: Vec<StageExecutionSummary>,
    pub pipeline_stages: Vec<String>,
    pub pipeline_tools: Vec<String>,
    pub stage_skips: Vec<serde_json::Value>,
}

/// Apply automatic pipeline decisions such as adapter detection and stage skipping.
///
/// # Errors
/// Returns an error if a required stage plan cannot be built or executed, or if
/// the adapter detection output directory cannot be created.
#[allow(clippy::too_many_arguments)]
pub fn apply_smart_pipeline_decisions<S: ::std::hash::BuildHasher>(
    catalog: &std::collections::HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
    registry: &bijux_core::ToolRegistry,
    args: &bijux_stages_fastq::args::BenchFastqPreprocessArgs,
    jobs: usize,
    pipeline_stages: Vec<String>,
    pipeline_tools: Vec<String>,
) -> Result<SmartPipelineResult> {
    let user_specified_adapters = args.adapter_bank_preset.is_some()
        || args.adapter_bank.is_some()
        || args.adapter_bank_file.is_some()
        || !args.enable_adapters.is_empty()
        || !args.disable_adapters.is_empty();
    let mut adapter_inference: Option<serde_json::Value> = None;
    let mut adapter_bank_preset_override: Option<String> = None;
    let mut preplanned_stage_runs: Vec<StageExecutionSummary> = Vec::new();
    let mut pipeline_stages = pipeline_stages;
    let mut pipeline_tools = pipeline_tools;

    if !user_specified_adapters {
        if let Some(idx) = pipeline_stages
            .iter()
            .position(|stage| stage == "fastq.detect_adapters")
        {
            let tool_id = pipeline_tools
                .get(idx)
                .cloned()
                .unwrap_or_else(|| "fastqc".to_string());
            let spec = build_tool_execution_spec(
                "fastq.detect_adapters",
                &tool_id,
                registry,
                catalog,
                platform,
            )?;
            let spec = normalize_tool_spec_for_jobs(&spec, jobs);
            let stage_root = bench_tools_dir(&args.out, "detect_adapters", &args.sample_id);
            let out_dir = stage_root.join(&spec.tool_id.0);
            std::fs::create_dir_all(&out_dir).context("create detect_adapters output dir")?;
            let plan = bijux_stages_fastq::fastq::detect_adapters::plan(&spec, &args.r1, &out_dir);
            let execution = execute_plan(&plan, platform.runner, None)?;
            if execution.exit_code == 0 {
                let candidates_path = out_dir
                    .join("run_artifacts")
                    .join("reports")
                    .join("adapter_candidates.json");
                let suggested_preset = std::fs::read_to_string(&candidates_path)
                    .ok()
                    .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                    .and_then(|value| {
                        value
                            .get("suggested_preset")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                    });
                adapter_bank_preset_override.clone_from(&suggested_preset);
                adapter_inference = Some(serde_json::json!({
                    "stage_id": "fastq.detect_adapters",
                    "tool_id": spec.tool_id.0,
                    "suggested_preset": suggested_preset,
                    "candidates_path": candidates_path.display().to_string(),
                    "reason": "auto-detected adapters from fastqc",
                }));
            }
            preplanned_stage_runs.push(StageExecutionSummary {
                plan,
                result: execution,
            });
            pipeline_stages.remove(idx);
            pipeline_tools.remove(idx);
        }
    }

    let mut stage_skips: Vec<serde_json::Value> = Vec::new();
    if let (Some(trim_idx), Some(filter_idx)) = (
        pipeline_stages
            .iter()
            .position(|stage| stage == "fastq.trim"),
        pipeline_stages
            .iter()
            .position(|stage| stage == "fastq.filter"),
    ) {
        let trim_tool = pipeline_tools.get(trim_idx).map(String::as_str);
        let filter_tool = pipeline_tools.get(filter_idx).map(String::as_str);
        if trim_tool == Some("fastp") && filter_tool == Some("fastp") {
            let skipped_stage = pipeline_stages.remove(filter_idx);
            let skipped_tool = pipeline_tools.remove(filter_idx);
            stage_skips.push(serde_json::json!({
                "stage_id": skipped_stage,
                "tool_id": skipped_tool,
                "reason": "fastp trimming already performs quality filtering; filter stage skipped",
                "equivalent_params": {
                    "quality_filtering": true
                }
            }));
        }
    }

    Ok(SmartPipelineResult {
        adapter_inference,
        adapter_bank_preset_override,
        preplanned_stage_runs,
        pipeline_stages,
        pipeline_tools,
        stage_skips,
    })
}
