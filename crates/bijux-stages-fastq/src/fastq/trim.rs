use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{ArtifactRef, StageIO, StageId, StagePlanV1, StageVersion, ToolExecutionSpecV1};

pub const STAGE_ID: &str = "fastq.trim";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone)]
pub struct TrimUserConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct TrimEffectiveConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub out_dir: std::path::PathBuf,
}

pub fn trim_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "cutadapt" => Some("cutadapt.fastq.gz"),
        "atropos" => Some("atropos.fastq.gz"),
        "bbduk" => Some("bbduk.fastq.gz"),
        "adapterremoval" => Some("adapterremoval.fastq.gz"),
        "trimmomatic" => Some("trimmomatic.fastq.gz"),
        "trim_galore" => Some("trimmed_trimmed.fq.gz"),
        "seqpurge" => Some("seqpurge.fastq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        _ => None,
    }
}

pub fn resolve_config(user: TrimUserConfig) -> TrimEffectiveConfig {
    TrimEffectiveConfig {
        tool: user.tool,
        r1: user.r1,
        out_dir: user.out_dir,
    }
}

/// Build a trim command plan.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan(
    tool: &ToolExecutionSpecV1,
    r1: &Path,
    out_dir: &Path,
    adapter_bank: Option<&serde_json::Value>,
    polyx_bank: Option<&serde_json::Value>,
    contaminant_bank: Option<&serde_json::Value>,
) -> Result<StagePlanV1> {
    let output_name =
        trim_output_name(&tool.tool_id.0).ok_or_else(|| anyhow!("unsupported trim tool"))?;
    let output = out_dir.join(output_name);
    let mut params = serde_json::json!({
        "tool": tool.tool_id.0,
        "input": r1,
        "output": output
    });
    if let Some(adapter_bank) = adapter_bank {
        if let Some(map) = params.as_object_mut() {
            map.insert("adapter_bank".to_string(), adapter_bank.clone());
        }
    }
    if let Some(polyx_bank) = polyx_bank {
        if let Some(map) = params.as_object_mut() {
            map.insert("polyx_bank".to_string(), polyx_bank.clone());
        }
    }
    if let Some(contaminant_bank) = contaminant_bank {
        if let Some(map) = params.as_object_mut() {
            map.insert("contaminant_bank".to_string(), contaminant_bank.clone());
        }
    }
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
                name: "trimmed_reads".to_string(),
                path: output.clone(),
            }],
        },
        out_dir: out_dir.to_path_buf(),
        params,
        aux_images: std::collections::BTreeMap::new(),
    })
}

/// Build a trim plan from resolved config.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_from_config(
    tool: &ToolExecutionSpecV1,
    config: &TrimEffectiveConfig,
) -> Result<StagePlanV1> {
    plan(tool, &config.r1, &config.out_dir, None, None, None)
}
