use anyhow::{Context, Result};
use bijux_engine::api::bench_tools_dir;
use bijux_engine::api::ResolvedImage;
use bijux_engine::api::{
    normalize_correct_tool_list as engine_normalize_correct_tool_list,
    normalize_filter_tool_list as engine_normalize_filter_tool_list,
    normalize_merge_tool_list as engine_normalize_merge_tool_list,
    normalize_qc2_tool_list as engine_normalize_qc2_tool_list,
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

pub use bijux_engine::api::ExecutionManifest;

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

pub(crate) fn normalize_qc2_tool_list(tools: &[String]) -> Result<Vec<String>> {
    engine_normalize_qc2_tool_list(tools)
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

pub(crate) fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    engine_resolve_image_for_run(spec, platform)
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn ratio_u64(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

pub(crate) fn normalize_inverted(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        return 1.0;
    }
    (max - value) / (max - min)
}

pub(crate) fn min_max(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for value in values {
        min = min.min(value);
        max = max.max(value);
    }
    if min == f64::INFINITY {
        (0.0, 0.0)
    } else {
        (min, max)
    }
}

pub(crate) fn format_optional(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".to_string(), |v| format!("{v:.3}"))
}
