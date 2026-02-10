//! Owner: bijux-dna-stages-fastq
//! Observer parsers for tool stdout and report artifacts.
//! Supported formats must be deterministic and stable across versions.

use anyhow::{anyhow, Context, Result};
use tracing::warn;

use bijux_dna_core::prelude::measure::SeqkitMetrics;

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
    let col = |name: &str| -> Result<&str> {
        let idx = header_fields
            .iter()
            .position(|field| field == &name)
            .ok_or_else(|| anyhow!("seqkit column missing: {name}"))?;
        data_fields
            .get(idx)
            .copied()
            .ok_or_else(|| anyhow!("seqkit data missing for {name}"))
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
    let gc_percent = if let Some((idx, _)) = header_fields
        .iter()
        .enumerate()
        .find(|(_, field)| field.to_lowercase().starts_with("gc"))
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
    Ok(SeqkitMetrics {
        reads,
        bases,
        mean_q,
        gc_percent,
    })
}

/// # Errors
/// Returns an error if stdout cannot be parsed.
pub fn parse_length_histogram(output: &str) -> Result<Vec<(u64, u64)>> {
    let mut counts: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
    for line in output.lines() {
        let mut parts = line.split('\t');
        let _id = parts.next();
        let len = parts
            .next()
            .ok_or_else(|| anyhow!("seqkit fx2tab length missing"))?;
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

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_deduplicate_report(report_json: &str) -> Result<(u64, u64)> {
    let reads_in = parse_report_u64_field(report_json, "reads_in")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_in"))?;
    let reads_out = parse_report_u64_field(report_json, "reads_out")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_out"))?;
    Ok((reads_in, reads_out))
}

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_low_complexity_report(report_json: &str) -> Result<u64> {
    parse_report_u64_field(report_json, "reads_removed_low_complexity")
        .ok_or_else(|| anyhow!("low-complexity report missing reads_removed_low_complexity"))
}

fn parse_report_u64_field(raw: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(raw).ok().and_then(|value| {
        value
            .get(field)
            .and_then(serde_json::Value::as_u64)
            .or_else(|| {
                value
                    .as_object()
                    .and_then(|obj| obj.get(field))
                    .and_then(serde_json::Value::as_str)
                    .and_then(|s| s.parse::<u64>().ok())
            })
    }).or_else(|| parse_kv_u64_field(raw, field))
}

fn parse_kv_u64_field(raw: &str, field: &str) -> Option<u64> {
    raw.lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(k, v)| {
            if k.trim() == field {
                v.trim().parse::<u64>().ok()
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::{
        parse_deduplicate_report, parse_fastqvalidator_count, parse_length_histogram,
        parse_low_complexity_report, parse_seqkit_stats,
    };
    use anyhow::Result;

    #[test]
    fn parse_fastqvalidator_count_parses_fixture() -> Result<()> {
        let stdout =
            include_str!("../../tests/fixtures/fastqvalidator/default/fastqvalidator_v1.txt");
        let count = parse_fastqvalidator_count(stdout)?;
        assert_eq!(count, 12345);
        Ok(())
    }

    #[test]
    fn parse_fastqvalidator_count_rejects_missing_marker() {
        let stdout = "fastqvalidator output without total reads";
        assert!(parse_fastqvalidator_count(stdout).is_err());
    }

    #[test]
    fn parse_seqkit_stats_parses_fixture() -> Result<()> {
        let stdout = include_str!("../../tests/fixtures/seqkit/default/seqkit_stats_v1.txt");
        let metrics = parse_seqkit_stats(stdout)?;
        assert_eq!(metrics.reads, 1000);
        assert_eq!(metrics.bases, 100_000);
        Ok(())
    }

    #[test]
    fn parse_length_histogram_parses_fixture() -> Result<()> {
        let stdout = "readA\t100\nreadB\t100\nreadC\t50\n";
        let metrics = parse_length_histogram(stdout)?;
        assert_eq!(metrics.len(), 2);
        Ok(())
    }

    #[test]
    fn parse_deduplicate_report_parses_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/deduplicate/default/deduplicate_report_v1.json");
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert_eq!(reads_in, 1000);
        assert_eq!(reads_out, 820);
        Ok(())
    }

    #[test]
    fn parse_low_complexity_report_parses_fixture() -> Result<()> {
        let raw = include_str!("../../tests/fixtures/low_complexity/default/low_complexity_report_v1.json");
        let removed = parse_low_complexity_report(raw)?;
        assert_eq!(removed, 137);
        Ok(())
    }

    #[test]
    fn parse_deduplicate_report_parses_key_value_fixture() -> Result<()> {
        let raw =
            include_str!("../../tests/fixtures/stage_output_bank/default/fastq.deduplicate.fastuniq.txt");
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert_eq!(reads_in, 1000);
        assert_eq!(reads_out, 820);
        Ok(())
    }

    #[test]
    fn parse_low_complexity_report_parses_key_value_fixture() -> Result<()> {
        let raw =
            include_str!("../../tests/fixtures/stage_output_bank/default/fastq.low_complexity.bbduk.txt");
        let removed = parse_low_complexity_report(raw)?;
        assert_eq!(removed, 137);
        Ok(())
    }
}
