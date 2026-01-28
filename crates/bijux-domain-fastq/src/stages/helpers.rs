use anyhow::{Context, Result};
use bijux_engine::api::bench_tools_dir;
use bijux_engine::api::ResolvedImage;
use bijux_engine::api::{
    normalize_correct_tool_list as engine_normalize_correct_tool_list,
    normalize_filter_tool_list as engine_normalize_filter_tool_list,
    normalize_merge_tool_list as engine_normalize_merge_tool_list,
    normalize_qc_post_tool_list as engine_normalize_qc_post_tool_list,
    normalize_screen_tool_list as engine_normalize_screen_tool_list,
    normalize_stats_tool_list as engine_normalize_stats_tool_list,
    normalize_trim_tool_list as engine_normalize_trim_tool_list,
    normalize_umi_tool_list as engine_normalize_umi_tool_list,
    normalize_validate_tool_list as engine_normalize_validate_tool_list,
    resolve_image_for_run as engine_resolve_image_for_run,
};
use bijux_environment::api::{PlatformSpec, ToolImageSpec};
use sha2::Digest;
use std::path::{Path, PathBuf};

use bijux_core::ToolRole;

use bijux_analyze::BenchmarkRecord;

use crate::contracts::RawFailure;

pub use bijux_engine::api::ExecutionManifest;
pub use bijux_engine::api::{ExplainExclusion, ExplainPlan};

#[derive(Debug)]
pub struct BenchOutcome<M: bijux_analyze::StageMetricSchema> {
    pub records: Vec<BenchmarkRecord<M>>,
    pub failures: Vec<RawFailure>,
    pub bench_dir: PathBuf,
    pub explain: bool,
}

#[derive(Debug)]
pub(crate) struct RunDirs {
    pub(crate) artifacts_dir: PathBuf,
    pub(crate) logs_dir: PathBuf,
    pub(crate) manifest_path: PathBuf,
    pub(crate) metrics_path: PathBuf,
}

pub(crate) fn params_hash(params: &serde_json::Value) -> Result<String> {
    let bytes = serde_json::to_vec(params).context("serialize params")?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

pub(crate) fn compute_run_id(
    stage: &str,
    tool: &str,
    image_digest: &str,
    input_hash: &str,
    params_hash: &str,
) -> String {
    let seed = format!("{stage}|{tool}|{image_digest}|{input_hash}|{params_hash}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn prepare_tool_run_dirs(
    tools_root: &Path,
    tool: &str,
    run_id: &str,
) -> Result<RunDirs> {
    let tool_dir = tools_root.join(tool);
    let run_dir = tool_dir.join("run").join(run_id);
    let artifacts_dir = run_dir.join("artifacts");
    let logs_dir = run_dir.join("logs");
    std::fs::create_dir_all(&artifacts_dir).context("create artifacts dir")?;
    std::fs::create_dir_all(&logs_dir).context("create logs dir")?;
    Ok(RunDirs {
        artifacts_dir,
        logs_dir: logs_dir.clone(),
        manifest_path: run_dir.join("manifest.json"),
        metrics_path: run_dir.join("metrics.json"),
    })
}

#[allow(dead_code)]
pub(crate) fn tool_run_artifacts_dir(
    out: &Path,
    stage: &str,
    sample_id: &str,
    tool: &str,
    run_id: &str,
) -> PathBuf {
    bench_tools_dir(out, stage, sample_id)
        .join(tool)
        .join("run")
        .join(run_id)
        .join("artifacts")
}

pub(crate) fn write_execution_logs(run_dirs: &RunDirs, stdout: &str, stderr: &str) -> Result<()> {
    let log_path = run_dirs.logs_dir.join("tool.log");
    if stderr.is_empty() {
        std::fs::write(&log_path, stdout).context("write tool.log")?;
    } else {
        std::fs::write(&log_path, format!("{stdout}\n--- stderr ---\n{stderr}"))
            .context("write tool.log")?;
    }
    Ok(())
}

pub(crate) fn write_metrics_json<T: serde::Serialize>(
    run_dirs: &RunDirs,
    execution: &bijux_measure::ExecutionMetrics,
    metrics: &T,
) -> Result<()> {
    let payload = serde_json::json!({
        "execution": execution,
        "metrics": metrics
    });
    std::fs::write(&run_dirs.metrics_path, serde_json::to_vec_pretty(&payload)?)
        .context("write metrics.json")?;
    Ok(())
}

pub(crate) fn write_explain_md(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    excluded: &[String],
    policy: Option<bijux_engine::api::Policy>,
) -> Result<()> {
    let path = base_dir.join("explain.md");
    let mut lines = Vec::new();
    lines.push(format!("# Explain: {stage}"));
    if let Some(policy) = policy {
        lines.push(format!("\nPolicy: `{policy:?}`"));
    }
    lines.push("\n## Selected tools".to_string());
    for tool in selected {
        lines.push(format!("- {tool}"));
    }
    if !excluded.is_empty() {
        lines.push("\n## Excluded tools".to_string());
        for tool in excluded {
            lines.push(format!("- {tool}"));
        }
    }
    std::fs::write(&path, lines.join("\n")).context("write explain.md")?;
    Ok(())
}

pub(crate) fn write_explain_plan_json(
    base_dir: &Path,
    stage: &str,
    selected: &[String],
    registry: &bijux_core::ToolRegistry,
    policy: Option<bijux_engine::api::Policy>,
) -> Result<()> {
    let mut excluded = Vec::new();
    for tool in registry.tools_for_stage(stage) {
        if !selected.iter().any(|t| t == &tool.tool_id) {
            excluded.push(ExplainExclusion {
                tool: tool.tool_id.clone(),
                reason: "not selected".to_string(),
            });
        }
    }
    let invariants = vec![
        "stage_contract".to_string(),
        "header_inspection".to_string(),
        "output_normalization".to_string(),
    ];
    let plan = ExplainPlan {
        stage: stage.to_string(),
        selected_tools: selected.to_vec(),
        excluded_tools: excluded,
        policy,
        invariants,
    };
    let path = base_dir.join("explain_plan.json");
    bijux_engine::api::write_explain_plan(&path, &plan)
}

pub(crate) fn normalize_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_trim_tool_list(tools)
}

pub(crate) fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_validate_tool_list(tools)
}

pub(crate) fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_filter_tool_list(tools)
}

pub(crate) fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_merge_tool_list(tools)
}

pub(crate) fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_correct_tool_list(tools)
}

pub(crate) fn normalize_qc_post_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_qc_post_tool_list(tools)
}

pub(crate) fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_umi_tool_list(tools)
}

pub(crate) fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_screen_tool_list(tools)
}

pub(crate) fn normalize_stats_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_stats_tool_list(tools)
}

pub(crate) fn filter_tools_by_role(
    stage_id: &str,
    tools: &[String],
    registry: &bijux_core::ToolRegistry,
    strict: bool,
) -> Result<Vec<String>> {
    let allow_experimental = std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_ok();
    let mut filtered = Vec::new();
    for tool in tools {
        let manifest = registry
            .tool_by_id(stage_id, tool)
            .ok_or_else(|| anyhow::anyhow!("tool {tool} missing from manifests"))?;
        match manifest.role {
            ToolRole::Authoritative => filtered.push(tool.clone()),
            ToolRole::Diagnostic => {
                if strict {
                    return Err(anyhow::anyhow!(
                        "strict mode requires authoritative tools; {tool} is diagnostic"
                    ));
                }
                filtered.push(tool.clone());
            }
            ToolRole::Experimental => {
                if !allow_experimental {
                    return Err(anyhow::anyhow!(
                        "experimental tool {tool} requires BIJUX_EXPERIMENTAL_TOOLS=1"
                    ));
                }
                filtered.push(tool.clone());
            }
        }
    }
    if filtered.is_empty() {
        return Err(anyhow::anyhow!("no tools available after role filtering"));
    }
    Ok(filtered)
}

pub(crate) fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    engine_resolve_image_for_run(spec, platform)
}
