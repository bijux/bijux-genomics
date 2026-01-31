use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};

pub const STAGE_ID: &str = "fastq.filter";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default)]
pub struct FilterPlanOptions {
    pub max_n: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
    pub redundant_filters: Vec<String>,
}

pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowed = ["prinseq", "fastp", "seqkit", "bbduk"];
    normalize_tools_with_allowlist(tools, &allowed)
}

/// Build a filter plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_filter(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
    options: &FilterPlanOptions,
) -> Result<StagePlanV1> {
    let output_name =
        filter_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported filter tool"))?;
    let output = out_dir.join(output_name);
    let kmer_ref = options
        .kmer_ref
        .clone()
        .or_else(default_kmer_ref)
        .map(|path| path.display().to_string());
    Ok(StagePlanV1 {
        stage_id: StageId(STAGE_ID.to_string()),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: r1.to_path_buf(),
            }],
            outputs: vec![ArtifactRef {
                name: "filtered_reads".to_string(),
                path: output.clone(),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "output": output,
            "max_n": options.max_n,
            "low_complexity_threshold": options.low_complexity_threshold,
            "kmer_ref": kmer_ref,
            "redundant_filters": options.redundant_filters,
        }),
        aux_images: std::collections::BTreeMap::new(),
    })
}

fn filter_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        "bbduk" => Some("bbduk.fastq.gz"),
        _ => None,
    }
}

fn default_kmer_ref() -> Option<PathBuf> {
    let dir = crate::contaminant_references_dir();
    let entries = std::fs::read_dir(dir).ok()?;
    let mut fasta = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("fasta") {
            fasta.push(path);
        }
    }
    fasta.sort();
    fasta.into_iter().next()
}

fn normalize_tools_with_allowlist(tools: &[String], allowlist: &[&str]) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.contains(&tool.as_str()) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}
