use std::path::Path;

use anyhow::Context;

use crate::metrics::{
    AlignmentCountsV1, ContaminationMetricsV1, CoverageMetricsV1, DamageMetricsV1,
    FragmentLengthSummaryV1, IdxstatsContigV1, IdxstatsSummaryV1, MapqSummaryV1,
    SexConfidenceClass, SexInferenceV1,
};

fn parse_first_int(text: &str) -> Option<u64> {
    text.split_whitespace().next()?.parse().ok()
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

/// # Errors
/// Returns an error if the flagstat file cannot be read.
pub fn parse_samtools_flagstat(path: &Path) -> anyhow::Result<AlignmentCountsV1> {
    let raw = std::fs::read_to_string(path).context("read flagstat")?;
    let mut counts = AlignmentCountsV1::empty();
    for line in raw.lines() {
        let value = parse_first_int(line).unwrap_or(0);
        if line.contains("in total") {
            counts.total = value;
        } else if line.contains("primary") {
            counts.primary = value;
        } else if line.contains("mapped") && !line.contains("mate mapped") {
            counts.mapped = value;
        } else if line.contains("properly paired") {
            counts.proper_pair = value;
        } else if line.contains("duplicates") {
            counts.duplicates = value;
        }
    }
    if counts.primary == 0 && counts.total > 0 {
        counts.primary = counts.total;
    }
    Ok(counts)
}

/// # Errors
/// Returns an error if the stats file cannot be read.
pub fn parse_samtools_stats(
    path: &Path,
) -> anyhow::Result<(FragmentLengthSummaryV1, MapqSummaryV1)> {
    let raw = std::fs::read_to_string(path).context("read samtools stats")?;
    let mut length_hist: Vec<(u32, u64)> = Vec::new();
    let mut mapq_hist: Vec<(u8, u64)> = Vec::new();
    for line in raw.lines() {
        if line.starts_with("RL") {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                if let (Ok(len), Ok(count)) = (parts[1].parse::<u32>(), parts[2].parse::<u64>()) {
                    length_hist.push((len, count));
                }
            }
        } else if line.starts_with("MQ") {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                if let (Ok(score), Ok(count)) = (parts[1].parse::<u8>(), parts[2].parse::<u64>()) {
                    mapq_hist.push((score, count));
                }
            }
        }
    }

    let fragment = summarize_length_hist(&length_hist);
    let mapq = summarize_mapq_hist(&mapq_hist);
    Ok((fragment, mapq))
}

fn summarize_length_hist(hist: &[(u32, u64)]) -> FragmentLengthSummaryV1 {
    if hist.is_empty() {
        return FragmentLengthSummaryV1::empty();
    }
    let total: u64 = hist.iter().map(|(_, c)| *c).sum();
    let mean = hist
        .iter()
        .map(|(len, c)| f64::from(*len) * u64_to_f64(*c))
        .sum::<f64>()
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
    let short_fraction = hist
        .iter()
        .filter(|(len, _)| *len <= 35)
        .map(|(_, c)| u64_to_f64(*c))
        .sum::<f64>()
        / u64_to_f64(total);
    FragmentLengthSummaryV1 {
        mean,
        median,
        p10,
        p90,
        short_fraction,
    }
}

fn summarize_mapq_hist(hist: &[(u8, u64)]) -> MapqSummaryV1 {
    if hist.is_empty() {
        return MapqSummaryV1::empty();
    }
    let total: u64 = hist.iter().map(|(_, c)| *c).sum();
    let mean = hist
        .iter()
        .map(|(q, c)| f64::from(*q) * u64_to_f64(*c))
        .sum::<f64>()
        / u64_to_f64(total);
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
    MapqSummaryV1 {
        mean,
        median,
        p10,
        p90,
        histogram: hist.to_vec(),
    }
}

/// # Errors
/// Returns an error if the idxstats file cannot be read or parsed.
pub fn parse_samtools_idxstats(path: &Path) -> anyhow::Result<IdxstatsSummaryV1> {
    let raw = std::fs::read_to_string(path).context("read samtools idxstats")?;
    let mut contigs = Vec::new();
    let mut total_mapped = 0_u64;
    let mut total_unmapped = 0_u64;
    let mut has_unmapped_row = false;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            continue;
        }
        let contig = parts[0].to_string();
        let length = parts[1].parse::<u64>().unwrap_or(0);
        let mapped = parts[2].parse::<u64>().unwrap_or(0);
        let unmapped = parts[3].parse::<u64>().unwrap_or(0);
        if contig == "*" {
            has_unmapped_row = true;
        }
        total_mapped += mapped;
        total_unmapped += unmapped;
        contigs.push(IdxstatsContigV1 {
            contig,
            length,
            mapped,
            unmapped,
        });
    }
    let reference_mismatch = has_unmapped_row
        || contigs
            .iter()
            .all(|contig| contig.length == 0 && contig.mapped == 0);
    Ok(IdxstatsSummaryV1 {
        contigs,
        total_mapped,
        total_unmapped,
        reference_mismatch,
    })
}

/// # Errors
/// Returns an error if the mosdepth summary cannot be read.
pub fn parse_mosdepth_summary(path: &Path) -> anyhow::Result<CoverageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read mosdepth summary")?;
    let mut mean = 0.0;
    let mut breadth_1x = 0.0;
    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        if parts[0] == "total" || parts[0] == "genome" || parts[0] == "all" {
            let length = parts[1].parse::<f64>().unwrap_or(0.0);
            let bases_covered = parts[2].parse::<f64>().unwrap_or(0.0);
            mean = parts[3].parse::<f64>().unwrap_or(0.0);
            if length > 0.0 {
                breadth_1x = (bases_covered / length).clamp(0.0, 1.0);
            }
            break;
        }
    }
    Ok(CoverageMetricsV1 {
        mean,
        median: mean,
        breadth_1x,
        breadth_3x: 0.0,
        breadth_5x: 0.0,
    })
}

/// # Errors
/// Returns an error if the preseq output cannot be read.
pub fn parse_preseq_estimates(path: &Path) -> anyhow::Result<crate::metrics::ComplexityMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read preseq output")?;
    let mut points = Vec::new();
    for line in raw.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let (Ok(x), Ok(y)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                points.push((x, y));
            }
        }
    }
    Ok(crate::metrics::ComplexityMetricsV1 {
        observed_reads: points.first().map_or(0, |(_, y)| *y),
        projected_reads: points,
    })
}

/// # Errors
/// Returns an error if the `PyDamage` JSON cannot be read or parsed.
pub fn parse_pydamage_json(path: &Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read pydamage json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let c_to_t = value
        .get("ct_5p")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let g_to_a = value
        .get("ga_3p")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    Ok(DamageMetricsV1 {
        c_to_t_5p: c_to_t,
        g_to_a_3p: g_to_a,
        pmd_score_histogram: Vec::new(),
    })
}

/// # Errors
/// Returns an error if the `DamageProfiler` JSON cannot be read or parsed.
pub fn parse_damageprofiler_json(path: &Path) -> anyhow::Result<DamageMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read damageprofiler json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let c_to_t = value
        .get("c_to_t_5p")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let g_to_a = value
        .get("g_to_a_3p")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    Ok(DamageMetricsV1 {
        c_to_t_5p: c_to_t,
        g_to_a_3p: g_to_a,
        pmd_score_histogram: Vec::new(),
    })
}

/// # Errors
/// Returns an error if the contamination JSON cannot be read or parsed.
pub fn parse_contamination_json(path: &Path) -> anyhow::Result<ContaminationMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read contamination json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(ContaminationMetricsV1 {
        method: value
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        estimate: value
            .get("estimate")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        ci_low: value
            .get("ci_low")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        ci_high: value
            .get("ci_high")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        assumptions: value
            .get("assumptions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// # Errors
/// Returns an error if the sex JSON cannot be read or parsed.
pub fn parse_sex_json(path: &Path) -> anyhow::Result<SexInferenceV1> {
    let raw = std::fs::read_to_string(path).context("read sex json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let x_to_y = value
        .get("x_to_y_ratio")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let confidence = value
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let sufficient = confidence >= 0.6 && x_to_y > 0.0;
    let classification = if !sufficient {
        SexConfidenceClass::Insufficient
    } else if x_to_y <= 0.6 {
        SexConfidenceClass::Male
    } else if x_to_y >= 1.5 {
        SexConfidenceClass::Female
    } else {
        SexConfidenceClass::Ambiguous
    };
    Ok(SexInferenceV1 {
        x_to_y_ratio: x_to_y,
        confidence,
        method: value
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        classification,
        sufficient_data: sufficient,
    })
}
