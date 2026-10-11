use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use tracing::warn;

pub use crate::api::ResolvedImage;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SeqkitMetrics {
    pub reads: u64,
    pub bases: u64,
    pub mean_q: f64,
    pub gc_percent: f64,
}

pub fn input_fastq_stats(
    image: &ResolvedImage,
    mount_dir: &Path,
    fastq: &Path,
) -> Result<SeqkitMetrics> {
    seqkit_stats(image, mount_dir, fastq)
}

pub fn output_fastq_stats(
    image: &ResolvedImage,
    mount_dir: &Path,
    fastq: &Path,
) -> Result<SeqkitMetrics> {
    seqkit_stats(image, mount_dir, fastq)
}

pub fn adapter_hit_reads(
    image: &ResolvedImage,
    mount_dir: &Path,
    fastq: &Path,
    adapter: &str,
) -> Result<u64> {
    let mount_dir = mount_dir.canonicalize().context("resolve mount directory")?;
    let fastq = fastq.canonicalize().context("resolve fastq path")?;
    let fastq_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("fastq missing filename"))?
        .to_string_lossy()
        .to_string();
    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/data:ro", mount_dir.display()))
        .arg(&image.full_name)
        .arg("sh")
        .arg("-lc")
        .arg(format!("seqkit grep -s -p {adapter} /data/{fastq_name} | seqkit stats -a -T -"));
    let output = cmd.output().context("run seqkit grep stats")?;
    if !output.status.success() {
        return Err(anyhow!("seqkit adapter scan failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let metrics = parse_seqkit_stats(&stdout)?;
    Ok(metrics.reads)
}

pub fn ensure_gzip_integrity(path: &Path) -> Result<()> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let status = Command::new("gzip").arg("-t").arg(path).status().context("run gzip -t")?;
        if !status.success() {
            return Err(anyhow!("gzip integrity check failed: {}", path.display()));
        }
    }
    Ok(())
}

fn seqkit_stats(image: &ResolvedImage, mount_dir: &Path, fastq: &Path) -> Result<SeqkitMetrics> {
    let mount_dir = mount_dir.canonicalize().context("resolve mount directory")?;
    let fastq = fastq.canonicalize().context("resolve fastq path")?;
    let fastq_name = fastq
        .file_name()
        .ok_or_else(|| anyhow!("fastq missing filename"))?
        .to_string_lossy()
        .to_string();

    let mut cmd = Command::new("docker");
    cmd.arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/data:ro", mount_dir.display()))
        .arg(&image.full_name)
        .arg("seqkit")
        .arg("stats")
        .arg("-a")
        .arg("-T")
        .arg(format!("/data/{fastq_name}"));

    let output = cmd.output().context("run seqkit stats")?;
    if !output.status.success() {
        return Err(anyhow!("seqkit stats failed"));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_seqkit_stats(&stdout)
}

fn parse_seqkit_stats(output: &str) -> Result<SeqkitMetrics> {
    let mut lines = output.lines();
    let header = lines.next().ok_or_else(|| anyhow!("empty seqkit output"))?;
    let data = lines.next().ok_or_else(|| anyhow!("missing seqkit data"))?;
    let header_fields: Vec<&str> = header.split('\t').collect();
    let data_fields: Vec<&str> = data.split('\t').collect();
    if header_fields.len() != data_fields.len() {
        return Err(anyhow!("seqkit header/data column mismatch"));
    }
    let col = |name: &str| -> Result<&str> {
        let idx = header_fields
            .iter()
            .position(|field| field == &name)
            .ok_or_else(|| anyhow!("seqkit column missing: {name}"))?;
        data_fields.get(idx).copied().ok_or_else(|| anyhow!("seqkit data missing for {name}"))
    };
    let reads: u64 = col("num_seqs")?.parse().context("parse reads")?;
    let bases: u64 = col("sum_len")?.parse().context("parse bases")?;
    let mean_q = if header_fields.iter().any(|field| field == &"avg_qual") {
        col("avg_qual")?.parse().context("parse mean_q")?
    } else if header_fields.iter().any(|field| field == &"mean_qual") {
        col("mean_qual")?.parse().context("parse mean_q")?
    } else {
        warn!("seqkit avg_qual/mean_qual missing; defaulting mean_q to 0.0");
        0.0
    };
    let gc_percent = if let Some((idx, _)) =
        header_fields.iter().enumerate().find(|(_, field)| field.to_lowercase().starts_with("gc"))
    {
        data_fields
            .get(idx)
            .ok_or_else(|| anyhow!("seqkit data missing for gc"))?
            .parse()
            .context("parse gc_percent")?
    } else {
        warn!("seqkit gc column missing; defaulting gc_percent to 0.0");
        0.0
    };
    Ok(SeqkitMetrics { reads, bases, mean_q, gc_percent })
}
