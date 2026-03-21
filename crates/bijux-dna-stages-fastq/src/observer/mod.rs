use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};

pub mod artifacts;
mod parse;

pub use artifacts::*;
pub use parse::{
    parse_deduplicate_report, parse_fastqvalidator_count, parse_length_histogram,
    parse_low_complexity_report, parse_multiqc_general_stats_metrics, parse_seqkit_stats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverSpecializationBinding {
    pub stage_id: &'static str,
    pub tool_id: &'static str,
    pub semantic_surface: &'static str,
}

const OBSERVER_SPECIALIZATION_BINDINGS: &[ObserverSpecializationBinding] = &[
    ObserverSpecializationBinding {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqvalidator",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.validate_reads",
        tool_id: "seqtk",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.validate_reads",
        tool_id: "fqtools",
        semantic_surface: "validation_report",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.profile_read_lengths",
        tool_id: "seqkit_stats",
        semantic_surface: "length_distribution_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.detect_adapters",
        tool_id: "fastqc",
        semantic_surface: "adapter_evidence_dir",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.profile_overrepresented_sequences",
        tool_id: "fastqc",
        semantic_surface: "overrepresented_sequences_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.profile_reads",
        tool_id: "seqkit_stats",
        semantic_surface: "qc_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.report_qc",
        tool_id: "multiqc",
        semantic_surface: "multiqc_data",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.remove_duplicates",
        tool_id: "fastuniq",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.remove_duplicates",
        tool_id: "clumpify",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: "cutadapt",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: "seqkit",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.trim_polyg_tails",
        tool_id: "fastp",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.trim_polyg_tails",
        tool_id: "bbduk",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.correct_errors",
        tool_id: "rcorrector",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.correct_errors",
        tool_id: "musket",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.correct_errors",
        tool_id: "lighter",
        semantic_surface: "report_json",
    },
    ObserverSpecializationBinding {
        stage_id: "fastq.correct_errors",
        tool_id: "bayeshammer",
        semantic_surface: "report_json",
    },
];

#[must_use]
pub fn observer_specialized_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    OBSERVER_SPECIALIZATION_BINDINGS
        .iter()
        .map(|binding| {
            (
                StageId::from_static(binding.stage_id),
                ToolId::from_static(binding.tool_id),
            )
        })
        .collect()
}

#[must_use]
pub fn is_observer_specialized_stage_tool(stage_id: &StageId, tool_id: &ToolId) -> bool {
    OBSERVER_SPECIALIZATION_BINDINGS.iter().any(|binding| {
        binding.stage_id == stage_id.as_str() && binding.tool_id == tool_id.as_str()
    })
}

#[derive(Debug, Clone)]
pub struct ObserverCommandSpec {
    pub image: String,
    pub mount_dir: std::path::PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ObserverCommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub enum ObserverCommandKind {
    SeqkitStats,
    SeqkitLengthHistogram,
}

pub fn input_fastq_stats(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    seqkit_stats_command(mount_dir, fastq)
}

pub fn output_fastq_stats(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    seqkit_stats_command(mount_dir, fastq)
}

fn seqkit_stats_command(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    let mount_dir = mount_dir
        .canonicalize()
        .context("resolve mount directory")?;
    let fastq = fastq.canonicalize().context("resolve fastq path")?;
    let fastq_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("fastq missing filename"))?
        .to_string_lossy()
        .to_string();

    Ok(ObserverCommandSpec {
        image: "seqkit".to_string(),
        mount_dir,
        args: vec![
            "seqkit".to_string(),
            "stats".to_string(),
            "-a".to_string(),
            "-T".to_string(),
            format!("/data/{fastq_name}"),
        ],
    })
}

pub fn length_histogram_command(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    let mount_dir = mount_dir
        .canonicalize()
        .context("resolve mount directory")?;
    let fastq = fastq.canonicalize().context("resolve fastq path")?;
    let fastq_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("fastq missing filename"))?
        .to_string_lossy()
        .to_string();

    Ok(ObserverCommandSpec {
        image: "seqkit".to_string(),
        mount_dir,
        args: vec![
            "seqkit".to_string(),
            "fx2tab".to_string(),
            "-l".to_string(),
            format!("/data/{fastq_name}"),
        ],
    })
}
