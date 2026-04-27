use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct ObserverCommandSpec {
    pub image: String,
    pub mount_dir: PathBuf,
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

/// # Errors
/// Returns an error if the mount directory or input FASTQ path cannot be canonicalized.
pub fn input_fastq_stats(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    seqkit_stats_command(mount_dir, fastq)
}

/// # Errors
/// Returns an error if the mount directory or input FASTQ path cannot be canonicalized.
pub fn output_fastq_stats(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    seqkit_stats_command(mount_dir, fastq)
}

fn seqkit_stats_command(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    let mount_dir = mount_dir.canonicalize().context("resolve mount directory")?;
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

/// # Errors
/// Returns an error if the mount directory or input FASTQ path cannot be canonicalized.
pub fn length_histogram_command(mount_dir: &Path, fastq: &Path) -> Result<ObserverCommandSpec> {
    let mount_dir = mount_dir.canonicalize().context("resolve mount directory")?;
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
