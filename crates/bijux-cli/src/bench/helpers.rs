use anyhow::{anyhow, Result};
use bijux_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, ToolImageSpec,
};
use serde::Serialize;
use tracing::warn;

#[derive(Debug, Serialize)]
pub(crate) struct ExecutionManifest {
    pub(crate) tool: String,
    pub(crate) tool_version: String,
    pub(crate) image_digest: String,
    pub(crate) command: String,
    pub(crate) input_hashes: Vec<String>,
    pub(crate) input_files: Vec<String>,
    pub(crate) runner: String,
    pub(crate) platform: String,
    pub(crate) arch: String,
}

pub(crate) fn normalize_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "fastp",
        "cutadapt",
        "bbduk",
        "adapterremoval",
        "trimmomatic",
        "trim_galore",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub(crate) fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["seqtk", "fastqc", "fastqvalidator", "fqtools"];
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
