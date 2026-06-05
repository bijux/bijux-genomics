use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::metrics::{FragmentLengthSummaryV1, MapqSummaryV1};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlignmentCountsV1 {
    pub total: u64,
    pub primary: u64,
    pub mapped: u64,
    pub proper_pair: u64,
    pub duplicates: u64,
}

impl AlignmentCountsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self { total: 0, primary: 0, mapped: 0, proper_pair: 0, duplicates: 0 }
    }
}

fn parse_first_int(text: &str) -> Option<u64> {
    text.split_whitespace().next()?.parse().ok()
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

/// # Errors
/// Returns an error if the flagstat file cannot be read.
pub fn parse_samtools_flagstat(path: &std::path::Path) -> anyhow::Result<AlignmentCountsV1> {
    let raw = std::fs::read_to_string(path).context("read flagstat")?;
    let mut counts = AlignmentCountsV1::empty();
    let mut saw_total = false;
    let mut saw_mapped = false;
    for line in raw.lines() {
        let value = parse_first_int(line).unwrap_or(0);
        if line.contains("in total") {
            counts.total = value;
            saw_total = true;
        } else if line.contains("primary") {
            counts.primary = value;
        } else if line.contains("mapped") && !line.contains("mate mapped") {
            counts.mapped = value;
            saw_mapped = true;
        } else if line.contains("properly paired") {
            counts.proper_pair = value;
        } else if line.contains("duplicates") {
            counts.duplicates = value;
        }
    }
    if !saw_total {
        anyhow::bail!("flagstat summary missing `in total` line");
    }
    if !saw_mapped {
        anyhow::bail!("flagstat summary missing `mapped` line");
    }
    if counts.primary == 0 && counts.total > 0 {
        counts.primary = counts.total;
    }
    Ok(counts)
}

/// # Errors
/// Returns an error if the stats file cannot be read.
pub fn parse_samtools_stats(
    path: &std::path::Path,
) -> anyhow::Result<(FragmentLengthSummaryV1, MapqSummaryV1)> {
    let raw = std::fs::read_to_string(path).context("read samtools stats")?;
    let mut length_hist: Vec<(u32, u64)> = Vec::new();
    let mut mapq_hist: Vec<(u8, u64)> = Vec::new();
    for (line_no, line) in raw.lines().enumerate() {
        if line.starts_with("RL") {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 3 {
                anyhow::bail!("samtools stats RL line {} has {} columns", line_no + 1, parts.len());
            }
            length_hist.push((
                parts[1].parse::<u32>().with_context(|| format!("parse RL length on line {}", line_no + 1))?,
                parts[2].parse::<u64>().with_context(|| format!("parse RL count on line {}", line_no + 1))?,
            ));
        } else if line.starts_with("MQ") {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 3 {
                anyhow::bail!("samtools stats MQ line {} has {} columns", line_no + 1, parts.len());
            }
            mapq_hist.push((
                parts[1].parse::<u8>().with_context(|| format!("parse MQ score on line {}", line_no + 1))?,
                parts[2].parse::<u64>().with_context(|| format!("parse MQ count on line {}", line_no + 1))?,
            ));
        }
    }
    if length_hist.is_empty() && mapq_hist.is_empty() {
        anyhow::bail!("samtools stats report contains no RL or MQ rows");
    }

    let fragment = summarize_length_hist(&length_hist);
    let mapq = summarize_mapq_hist(&mapq_hist);
    Ok((fragment, mapq))
}

pub(crate) fn summarize_length_hist(hist: &[(u32, u64)]) -> FragmentLengthSummaryV1 {
    if hist.is_empty() {
        return FragmentLengthSummaryV1::empty();
    }
    let total: u64 = hist.iter().map(|(_, c)| *c).sum();
    let mean = hist.iter().map(|(len, c)| f64::from(*len) * u64_to_f64(*c)).sum::<f64>()
        / u64_to_f64(total);
    let mut ordered = hist.to_vec();
    ordered.sort_by_key(|(len, _)| *len);
    let mut cumulative = 0_u64;
    let mut median = 0.0;
    let mut p10 = 0.0;
    let mut p90 = 0.0;
    for (len, count) in ordered {
        cumulative += count;
        let frac = u64_to_f64(cumulative) / u64_to_f64(total);
        if p10 == 0.0 && frac >= 0.1 {
            p10 = f64::from(len);
        }
        if median == 0.0 && frac >= 0.5 {
            median = f64::from(len);
        }
        if p90 == 0.0 && frac >= 0.9 {
            p90 = f64::from(len);
            break;
        }
    }
    let short_fraction =
        hist.iter().filter(|(len, _)| *len <= 35).map(|(_, c)| u64_to_f64(*c)).sum::<f64>()
            / u64_to_f64(total);
    FragmentLengthSummaryV1 { mean, median, p10, p90, short_fraction }
}

pub(crate) fn summarize_mapq_hist(hist: &[(u8, u64)]) -> MapqSummaryV1 {
    if hist.is_empty() {
        return MapqSummaryV1::empty();
    }
    let total: u64 = hist.iter().map(|(_, c)| *c).sum();
    let mean =
        hist.iter().map(|(q, c)| f64::from(*q) * u64_to_f64(*c)).sum::<f64>() / u64_to_f64(total);
    let mut ordered = hist.to_vec();
    ordered.sort_by_key(|(q, _)| *q);
    let mut cumulative = 0_u64;
    let mut median = 0.0;
    let mut p10 = 0.0;
    let mut p90 = 0.0;
    for (q, count) in ordered {
        cumulative += count;
        let frac = u64_to_f64(cumulative) / u64_to_f64(total);
        if p10 == 0.0 && frac >= 0.1 {
            p10 = f64::from(q);
        }
        if median == 0.0 && frac >= 0.5 {
            median = f64::from(q);
        }
        if p90 == 0.0 && frac >= 0.9 {
            p90 = f64::from(q);
            break;
        }
    }
    MapqSummaryV1 { mean, median, p10, p90, histogram: hist.to_vec() }
}
