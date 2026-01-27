use anyhow::{anyhow, Result};
use bijux_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, ToolImageSpec,
};
use tracing::warn;

pub fn normalize_trim_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

pub fn normalize_validate_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "seqtk",
        "fastqc",
        "fastqvalidator",
        "fastqvalidator_official",
        "fqtools",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_merge_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["pear", "vsearch", "bbmerge", "flash2"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_correct_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["rcorrector", "spades", "bayeshammer", "lighter", "musket"];
    let mut allowlist = allowed.to_vec();
    if std::env::var("BIJUX_EXPERIMENTAL_TOOLS").is_err() {
        allowlist.retain(|tool| *tool == "rcorrector");
    }
    normalize_tools_with_allowlist(tools, &allowlist)
}

pub fn normalize_qc2_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["fastqc", "multiqc"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_umi_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["umi_tools"];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_screen_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = [
        "kraken2",
        "centrifuge",
        "metaphlan",
        "kaiju",
        "fastq_screen",
    ];
    normalize_tools_with_allowlist(tools, &allowed)
}

pub fn normalize_stats_tool_list(tools: &[String]) -> Result<Vec<String>> {
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

pub fn resolve_image_for_run(
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
