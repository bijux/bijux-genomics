use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRole, StageId, StageVersion, ToolExecutionSpecV1,
};
use bijux_dna_domain_fastq::params::{filter::FilterEffectiveParams, PairedMode};
use bijux_dna_domain_fastq::STAGE_FILTER;
use bijux_dna_stage_contract::{ArtifactRef, StageIO, StagePlanV1};

pub const STAGE_ID: StageId = STAGE_FILTER;
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, Default)]
pub struct FilterPlanOptions {
    pub max_n: Option<u32>,
    pub max_n_fraction: Option<f64>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub entropy_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
    pub redundant_filters: Vec<String>,
    pub polyx_policy: Option<String>,
}

pub fn normalize_filter_tool_list(tools: &[String]) -> Result<Vec<String>> {
    let allowlist = crate::selection::allowed_tools_for_stage(&STAGE_ID);
    normalize_tools_with_allowlist(tools, &allowlist)
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
        .map(|path| path.display().to_string());
    let effective_params = FilterEffectiveParams {
        paired_mode: PairedMode::SingleEnd,
        threads: tool.resources.threads,
        max_n: options.max_n,
        max_n_fraction: options.max_n_fraction,
        max_n_count: options.max_n_count.or(options.max_n),
        low_complexity_threshold: options.low_complexity_threshold,
        entropy_threshold: options.entropy_threshold,
        contaminant_db: kmer_ref.clone(),
        n_policy: None,
        polyx_policy: options.polyx_policy.clone(),
        damage_mode: None,
    };
    Ok(StagePlanV1 {
        stage_id: STAGE_ID.clone(),
        stage_version: STAGE_VERSION,
        tool_id: tool.tool_id.clone(),
        tool_version: tool.tool_version.clone(),
        image: tool.image.clone(),
        command: tool.command.clone(),
        resources: tool.resources.clone(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads_r1"),
                r1.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("filtered_reads"),
                output.clone(),
                ArtifactRole::Reads,
            )],
        },
        out_dir: out_dir.to_path_buf(),
        params: serde_json::json!({
            "tool": tool.tool_id.0,
            "input": r1,
            "output": output,
            "max_n": options.max_n,
            "max_n_fraction": options.max_n_fraction,
            "max_n_count": options.max_n_count,
            "low_complexity_threshold": options.low_complexity_threshold,
            "entropy_threshold": options.entropy_threshold,
            "kmer_ref": kmer_ref,
            "redundant_filters": options.redundant_filters,
            "polyx_policy": options.polyx_policy,
        }),
        effective_params: serde_json::to_value(&effective_params)
            .map_err(|error| anyhow!("serialize filter effective params: {error}"))?,
        aux_images: std::collections::BTreeMap::new(),
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
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

pub fn default_kmer_ref() -> Option<PathBuf> {
    let dir = bijux_dna_domain_fastq::contaminant_references_dir();
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

fn normalize_tools_with_allowlist(
    tools: &[String],
    allowlist: &[bijux_dna_core::ids::ToolId],
) -> Result<Vec<String>> {
    let mut normalized: Vec<String> = tools.iter().map(|tool| tool.to_lowercase()).collect();
    normalized.sort();
    normalized.dedup();
    for tool in &normalized {
        if !allowlist.iter().any(|allowed| allowed.as_str() == tool) {
            return Err(anyhow!("unsupported tool {tool}"));
        }
    }
    Ok(normalized)
}
