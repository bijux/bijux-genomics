use anyhow::{anyhow, Result};

const ATROPOS_MIN_THREADS: u32 = 2;

#[derive(Debug, Clone, Default)]
pub struct TrimPlanOptions {
    pub threads: Option<u32>,
    pub min_length: Option<u32>,
    pub quality_cutoff: Option<u32>,
    pub n_policy: Option<String>,
    pub adapter_policy: Option<String>,
    pub polyx_policy: Option<String>,
    pub contaminant_policy: Option<String>,
}

impl TrimPlanOptions {
    pub(super) fn resolved_threads(&self, default_threads: u32) -> u32 {
        self.threads.unwrap_or(default_threads).max(1)
    }

    pub(super) fn resolved_min_length(&self) -> u32 {
        self.min_length.unwrap_or(30)
    }

    pub(super) fn resolved_adapter_policy(&self) -> String {
        self.adapter_policy.clone().unwrap_or_else(|| "none".to_string())
    }

    pub(super) fn resolved_polyx_policy(&self) -> String {
        self.polyx_policy.clone().unwrap_or_else(|| "none".to_string())
    }

    pub(super) fn resolved_n_policy(&self) -> String {
        self.n_policy.clone().unwrap_or_else(|| "retain".to_string())
    }

    pub(super) fn resolved_contaminant_policy(&self) -> String {
        self.contaminant_policy.clone().unwrap_or_else(|| "none".to_string())
    }
}

pub(super) fn normalize_trim_threads(tool_id: &str, threads: u32) -> u32 {
    if tool_id == "atropos" {
        threads.max(ATROPOS_MIN_THREADS)
    } else {
        threads.max(1)
    }
}

#[derive(Debug, Clone)]
pub struct TrimUserConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub out_dir: std::path::PathBuf,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct TrimEffectiveConfig {
    pub tool: String,
    pub r1: std::path::PathBuf,
    pub r2: Option<std::path::PathBuf>,
    pub out_dir: std::path::PathBuf,
    pub adapter_bank: Option<serde_json::Value>,
    pub polyx_bank: Option<serde_json::Value>,
    pub contaminant_bank: Option<serde_json::Value>,
}

#[must_use]
pub fn trim_output_name(tool: &str) -> Option<&'static str> {
    match tool {
        "fastp" => Some("fastp.fastq.gz"),
        "cutadapt" => Some("cutadapt.fastq.gz"),
        "atropos" => Some("atropos.fastq.gz"),
        "bbduk" => Some("bbduk.fastq.gz"),
        "adapterremoval" => Some("adapterremoval.fastq.gz"),
        "trimmomatic" => Some("trimmomatic.fastq.gz"),
        "trim_galore" => Some("trimmed_trimmed.fq.gz"),
        "prinseq" => Some("prinseq_good.fastq"),
        "seqkit" => Some("seqkit.fastq.gz"),
        "seqpurge" => Some("seqpurge.fastq.gz"),
        "skewer" => Some("skewer.fastq.gz"),
        "leehom" => Some("leehom.fastq.gz"),
        "alientrimmer" => Some("alientrimmer.fastq.gz"),
        "fastx_clipper" => Some("fastx_clipper.fastq.gz"),
        _ => None,
    }
}

#[must_use]
pub fn resolve_config(user: TrimUserConfig) -> TrimEffectiveConfig {
    TrimEffectiveConfig {
        tool: user.tool,
        r1: user.r1,
        r2: user.r2,
        out_dir: user.out_dir,
        adapter_bank: user.adapter_bank,
        polyx_bank: user.polyx_bank,
        contaminant_bank: user.contaminant_bank,
    }
}

/// # Errors
/// Returns an error when any selected trim backend cannot execute the requested trim surface.
pub fn validate_trim_toolset_support(
    tool_ids: &[String],
    paired_layout: bool,
    options: &TrimPlanOptions,
) -> Result<()> {
    let mut incompatibilities = Vec::new();
    for tool_id in tool_ids {
        if tool_id == "seqpurge" && !paired_layout {
            incompatibilities.push(format!("{tool_id}: requires paired-end reads"));
            continue;
        }
        if let Err(error) = ensure_trim_option_support(tool_id, options) {
            incompatibilities.push(format!("{tool_id}: {error}"));
        }
    }
    if incompatibilities.is_empty() {
        return Ok(());
    }
    Err(anyhow!(
        "trim request is incompatible with selected tools: {}",
        incompatibilities.join("; ")
    ))
}

pub(super) fn ensure_trim_option_support(tool_id: &str, options: &TrimPlanOptions) -> Result<()> {
    if let Some(policy) = options.n_policy.as_deref() {
        if !matches!(
            (policy, tool_id),
            ("retain", _) | ("drop", "fastp" | "cutadapt" | "prinseq" | "bbduk")
        ) {
            return Err(anyhow!(
                "trim planning does not yet support n_policy={policy} for {tool_id}"
            ));
        }
    }
    if let Some(policy) = options.adapter_policy.as_deref() {
        match policy {
            "none" | "auto" | "bank" | "ancient_strict" => {}
            _ => {
                return Err(anyhow!(
                    "trim planning does not yet support adapter_policy={policy} for {tool_id}"
                ));
            }
        }
    }
    if let Some(policy) = options.polyx_policy.as_deref() {
        match policy {
            "none" | "trim" | "bank" if tool_id == "fastp" => {}
            "none" => {}
            _ => {
                return Err(anyhow!(
                    "trim planning does not yet support polyx_policy={policy} for {tool_id}"
                ));
            }
        }
    }
    if let Some(policy) = options.contaminant_policy.as_deref() {
        if !matches!((policy, tool_id), ("none", _) | ("bank", "bbduk")) {
            return Err(anyhow!(
                "trim planning does not execute contaminant_policy={policy} for {tool_id}; use fastq.deplete_reference_contaminants"
            ));
        }
    }
    if matches!(options.adapter_policy.as_deref(), Some("bank" | "ancient_strict")) {
        match tool_id {
            "fastp" | "cutadapt" | "atropos" | "adapterremoval" | "alientrimmer"
            | "trim_galore" | "fastx_clipper" | "skewer" | "leehom" => {}
            _ => {
                return Err(anyhow!(
                    "trim planning does not yet execute adapter bank policies for {tool_id}"
                ));
            }
        }
    }
    let uses_length_or_quality = options.min_length.is_some() || options.quality_cutoff.is_some();
    if !uses_length_or_quality {
        return Ok(());
    }
    match tool_id {
        "fastp" | "cutadapt" | "atropos" | "bbduk" | "adapterremoval" | "trimmomatic"
        | "alientrimmer" | "trim_galore" | "skewer" | "prinseq" => Ok(()),
        "seqkit" | "seqpurge" if options.quality_cutoff.is_none() => Ok(()),
        _ => Err(anyhow!("trim planning does not yet map min_length/quality_cutoff for {tool_id}")),
    }
}
