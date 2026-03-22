use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::{
    is_observer_specialized_stage_tool as domain_is_observer_specialized_stage_tool,
    observer_specialized_stage_tool_bindings as domain_observer_specialized_stage_tool_bindings,
};

pub mod artifacts;
mod parse;

pub use artifacts::*;
pub use parse::{
    parse_bbduk_reads_removed, parse_deduplicate_report, parse_duplicate_classes_tsv,
    parse_fastp_metrics, parse_fastqvalidator_count, parse_length_histogram,
    parse_low_complexity_report, parse_multiqc_general_stats_metrics,
    parse_merge_pairs_report,
    parse_normalize_abundance_report,
    parse_normalize_primers_report,
    parse_profile_overrepresented_report, parse_profile_read_lengths_report,
    parse_profile_reads_report,
    parse_remove_chimeras_report, parse_remove_duplicates_provenance,
    parse_remove_duplicates_report, parse_report_qc_report, parse_screen_summary_tsv,
    parse_screen_taxonomy_report, parse_seqkit_stats, parse_terminal_damage_report,
    parse_trim_polyg_report, parse_trim_reads_report, parse_validated_reads_manifest,
    parse_validation_report,
};

#[must_use]
pub fn observer_specialized_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    domain_observer_specialized_stage_tool_bindings()
}

#[must_use]
pub fn is_observer_specialized_stage_tool(stage_id: &StageId, tool_id: &ToolId) -> bool {
    domain_is_observer_specialized_stage_tool(stage_id, tool_id)
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
