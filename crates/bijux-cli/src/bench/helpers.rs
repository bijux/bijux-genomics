use anyhow::{anyhow, Context, Result};
use bijux_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, ToolImageSpec,
};
use serde::Serialize;
use sha2::Digest;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::utils::bench_tools_dir;

#[derive(Debug, Serialize, serde::Deserialize)]
pub(crate) struct ExecutionManifest {
    pub(crate) run_id: String,
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) tool_version: String,
    pub(crate) image_digest: String,
    pub(crate) command: String,
    pub(crate) input_hashes: Vec<String>,
    pub(crate) input_files: Vec<String>,
    pub(crate) output_dir: String,
    pub(crate) runner: String,
    pub(crate) platform: String,
    pub(crate) arch: String,
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
    execution: &bijux_bench::ExecutionMetrics,
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

pub(crate) fn normalize_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "fastp",
        "cutadapt",
        "bbduk",
        "adapterremoval",
        "trimmomatic",
        "trim_galore",
        "atropos",
        "seqpurge",
    ];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool != "seqpurge");
    }
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub(crate) fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "seqtk",
        "fastqc",
        "fastqvalidator",
        "fastqvalidator_official",
        "fqtools",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["rcorrector", "spades", "bayeshammer", "lighter", "musket"];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool == "rcorrector");
    }
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub(crate) fn normalize_qc2_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_stats_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["seqkit_stats"];
    normalize_tools_with_allowlist(tools, &allowed)
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err(anyhow!("no tools specified"));
    }
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool: {tool}"));
        }
    }
    Ok(normalized)
}

pub(crate) fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!(
                "digest image missing locally; falling back to tag {}",
                fallback.full_name
            );
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
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

pub(crate) fn find_first_fastq(dir: &Path) -> Result<PathBuf> {
    let entries = std::fs::read_dir(dir)
        .map_err(|err| anyhow!("failed to read output directory {}: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if ext.eq_ignore_ascii_case("fq")
                || ext.eq_ignore_ascii_case("fastq")
                || ext.eq_ignore_ascii_case("gz")
            {
                return Ok(path);
            }
        }
    }
    Err(anyhow!("no FASTQ output found in {}", dir.display()))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct DeltaMetrics {
    pub(crate) delta_mean_q: f64,
    pub(crate) delta_gc: f64,
    pub(crate) read_retention: f64,
    pub(crate) base_retention: f64,
}

impl DeltaMetrics {
    pub(crate) fn validate(&self) -> Result<()> {
        if !self.delta_mean_q.is_finite() {
            return Err(anyhow!("delta_mean_q must be finite"));
        }
        if !self.delta_gc.is_finite() {
            return Err(anyhow!("delta_gc must be finite"));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(anyhow!("read_retention must be within [0, 1]"));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(anyhow!("base_retention must be within [0, 1]"));
        }
        Ok(())
    }
}

pub(crate) fn delta_metrics(
    before: crate::utils::SeqkitMetrics,
    after: crate::utils::SeqkitMetrics,
) -> DeltaMetrics {
    let read_retention = if before.reads > 0 {
        ratio_u64(after.reads, before.reads)
    } else {
        0.0
    };
    let base_retention = if before.bases > 0 {
        ratio_u64(after.bases, before.bases)
    } else {
        0.0
    };
    DeltaMetrics {
        delta_mean_q: after.mean_q - before.mean_q,
        delta_gc: after.gc_percent - before.gc_percent,
        read_retention,
        base_retention,
    }
}
