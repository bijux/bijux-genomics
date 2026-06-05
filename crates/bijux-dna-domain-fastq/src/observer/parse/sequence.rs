use anyhow::{anyhow, Context, Result};
use tracing::warn;

use super::SeqkitMetrics;

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_seqkit_stats(output: &str) -> Result<SeqkitMetrics> {
    let mut lines = output.lines();
    let header = lines.next().ok_or_else(|| anyhow!("empty seqkit output"))?;
    let data = lines.next().ok_or_else(|| anyhow!("missing seqkit data"))?;
    let header_fields: Vec<&str> = header.split('\t').collect();
    let data_fields: Vec<&str> = data.split('\t').collect();
    if header_fields.len() != data_fields.len() {
        return Err(anyhow!("seqkit header/data column mismatch"));
    }
    let canonical = |field: &str| {
        field
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .flat_map(char::to_lowercase)
            .collect::<String>()
    };
    let col = |name: &str| -> Result<&str> {
        let idx = header_fields
            .iter()
            .position(|field| canonical(field) == canonical(name))
            .ok_or_else(|| anyhow!("seqkit column missing: {name}"))?;
        data_fields.get(idx).copied().ok_or_else(|| anyhow!("seqkit data missing for {name}"))
    };
    let reads: u64 = col("num_seqs")?.parse().context("parse reads")?;
    let bases: u64 = col("sum_len")?.parse().context("parse bases")?;
    let mean_q = if header_fields.iter().any(|field| canonical(field) == canonical("avg_qual")) {
        col("avg_qual")?.parse().context("parse mean_q")?
    } else if header_fields.iter().any(|field| canonical(field) == canonical("mean_qual")) {
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

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_length_histogram(output: &str) -> Result<Vec<(u64, u64)>> {
    let mut counts: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
    for line in output.lines() {
        let len =
            line.rsplit('\t').next().ok_or_else(|| anyhow!("seqkit fx2tab length missing"))?;
        let cleaned: String = len.chars().filter(char::is_ascii_digit).collect();
        if cleaned.is_empty() {
            continue;
        }
        let length: u64 = cleaned.parse().context("parse length")?;
        *counts.entry(length).or_insert(0) += 1;
    }
    Ok(counts.into_iter().collect())
}

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_fastqvalidator_count(stdout: &str) -> Result<u64> {
    let line = stdout
        .lines()
        .find(|line| line.to_lowercase().contains("total reads"))
        .ok_or_else(|| anyhow!("fastqvalidator total reads line missing"))?;
    let count = line
        .split_once(':')
        .ok_or_else(|| anyhow!("fastqvalidator total reads format missing ':'"))?
        .1
        .trim();
    Ok(count.parse::<u64>()?)
}
