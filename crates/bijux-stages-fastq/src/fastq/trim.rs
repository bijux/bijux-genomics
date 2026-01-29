use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, StageVersion};

use crate::plan::{ArtifactRef, StageIO, StagePlan};

pub const STAGE_ID: &str = "fastq.trim";
pub const STAGE_VERSION: StageVersion = StageVersion(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimPlan {
    pub tool: String,
    pub input: std::path::PathBuf,
    pub output: std::path::PathBuf,
}

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
pub fn plan(tool: &str, r1: &Path, out_dir: &Path) -> Result<TrimPlan> {
    let output_name =
        trim_output_name(tool).ok_or_else(|| anyhow!("unsupported trim tool: {tool}"))?;
    Ok(TrimPlan {
        tool: tool.to_string(),
        input: r1.to_path_buf(),
        output: out_dir.join(output_name),
    })
}

/// Build a trim plan from resolved config.
///
/// # Errors
/// Returns an error if the tool is unsupported.
pub fn plan_from_config(config: &TrimEffectiveConfig) -> Result<TrimPlan> {
    plan(&config.tool, &config.r1, &config.out_dir)
}

impl StagePlan for TrimPlan {
    fn stage_id(&self) -> StageId {
        StageId(STAGE_ID.to_string())
    }

    fn stage_version(&self) -> StageVersion {
        STAGE_VERSION
    }

    fn outputs(&self) -> StageIO {
        StageIO {
            inputs: vec![ArtifactRef {
                name: "reads_r1".to_string(),
                path: self.input.clone(),
            }],
            outputs: vec![ArtifactRef {
                name: "trimmed_reads".to_string(),
                path: self.output.clone(),
            }],
        }
    }

    fn parameters_json(&self) -> serde_json::Value {
        serde_json::json!({
            "tool": self.tool,
            "input": self.input,
            "output": self.output
        })
    }
}
