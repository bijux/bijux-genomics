use std::path::Path;

use anyhow::{anyhow, Context, Result};

pub mod artifacts;
mod parse;

pub use artifacts::*;
pub use parse::{parse_fastqvalidator_count, parse_length_histogram, parse_seqkit_stats};

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
