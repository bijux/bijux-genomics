use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::{
    VcfCallSummaryMetricsV1, VcfFilterBreakdownMetricsV1, VcfStatsMetricsV1,
};

use crate::vcf_io::read_vcf_text;

fn parse_record_fields(line: &str) -> Option<Vec<&str>> {
    if line.trim().is_empty() || line.starts_with('#') {
        return None;
    }
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() < 8 {
        return None;
    }
    Some(fields)
}

#[must_use]
pub fn parse_depth_from_info(info: &str) -> Option<u32> {
    info.split(';').find_map(|field| {
        let (key, value) = field.split_once('=')?;
        if key == "DP" {
            value.parse::<u32>().ok()
        } else {
            None
        }
    })
}

/// # Errors
/// Returns an error when no VCF records can be parsed.
pub fn parse_vcf_call_summary(path: &Path, sample_name: &str) -> Result<VcfCallSummaryMetricsV1> {
    let raw = read_vcf_text(path)?;
    let mut metrics = VcfCallSummaryMetricsV1::empty(sample_name.to_string());
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        metrics.variants_called += 1;
        let r = fields[3];
        let a = fields[4];
        if r.len() == 1 && a.len() == 1 && !a.contains(',') {
            metrics.snps += 1;
        } else {
            metrics.indels += 1;
        }
    }
    if metrics.variants_called == 0 {
        return Err(anyhow!("vcf.call parser found no variants"));
    }
    Ok(metrics)
}

/// # Errors
/// Returns an error when no VCF records can be parsed.
pub fn parse_vcf_filter_breakdown(
    path: &Path,
    sample_name: &str,
) -> Result<VcfFilterBreakdownMetricsV1> {
    let raw = read_vcf_text(path)?;
    let mut metrics = VcfFilterBreakdownMetricsV1::empty(sample_name.to_string());
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        metrics.variants_in += 1;
        let filter = fields[6].to_string();
        *metrics.filter_breakdown.entry(filter.clone()).or_insert(0) += 1;
        if filter == "PASS" || filter == "." {
            metrics.variants_pass += 1;
        } else {
            metrics.variants_filtered += 1;
        }
    }
    if metrics.variants_in == 0 {
        return Err(anyhow!("vcf.filter parser found no variants"));
    }
    Ok(metrics)
}

/// # Errors
/// Returns an error when the stats file cannot be read or required counters are missing.
pub fn parse_vcf_stats(path: &Path) -> Result<VcfStatsMetricsV1> {
    let raw = std::fs::read_to_string(path)?;
    let mut metrics = VcfStatsMetricsV1::empty();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('\t') else {
            continue;
        };
        match k {
            "variants_total" => metrics.variants_total = v.parse().unwrap_or(0),
            "sample_count" => metrics.sample_count = v.parse().unwrap_or(0),
            "snps" => metrics.snps = v.parse().unwrap_or(0),
            "indels" => metrics.indels = v.parse().unwrap_or(0),
            "ti_tv" => metrics.ti_tv = v.parse::<f64>().ok(),
            "missingness_post" => metrics.missingness_post = v.parse::<f64>().ok(),
            "heterozygosity_ratio" => metrics.heterozygosity_ratio = v.parse::<f64>().ok(),
            "annotation_coverage" => metrics.annotation_coverage = v.parse::<f64>().ok(),
            "sample_name" => metrics.sample_name = v.to_string(),
            _ if k.starts_with("filter.") => {
                let key = k.trim_start_matches("filter.").to_string();
                let value = v.parse::<u64>().unwrap_or(0);
                metrics.filter_breakdown.insert(key, value);
            }
            _ if k.starts_with("depth.") => {
                let key = k.trim_start_matches("depth.").to_string();
                let value = v.parse::<u64>().unwrap_or(0);
                metrics.depth_distribution.insert(key, value);
            }
            _ => {}
        }
    }

    if metrics.variants_total == 0 && metrics.snps == 0 && metrics.indels == 0 {
        return Err(anyhow!("vcf.stats parser found no variant counters"));
    }
    if metrics.filter_breakdown.is_empty() {
        metrics.filter_breakdown.insert("PASS".to_string(), metrics.variants_total);
    }
    metrics.call_summary = VcfCallSummaryMetricsV1 {
        schema_version: "bijux.vcf.call_summary.v1".to_string(),
        sample_name: metrics.sample_name.clone(),
        variants_called: metrics.variants_total,
        snps: metrics.snps,
        indels: metrics.indels,
    };
    metrics.filter_summary = VcfFilterBreakdownMetricsV1 {
        schema_version: "bijux.vcf.filter_breakdown.v1".to_string(),
        sample_name: metrics.sample_name.clone(),
        variants_in: metrics.variants_total,
        variants_pass: *metrics.filter_breakdown.get("PASS").unwrap_or(&0),
        variants_filtered: metrics
            .variants_total
            .saturating_sub(*metrics.filter_breakdown.get("PASS").unwrap_or(&0)),
        filter_breakdown: metrics.filter_breakdown.clone(),
    };
    Ok(metrics)
}

#[must_use]
pub fn summarize_vcf_metrics(metrics: &VcfStatsMetricsV1) -> serde_json::Value {
    let mut filters = BTreeMap::new();
    for (k, v) in &metrics.filter_breakdown {
        filters.insert(k.clone(), serde_json::Value::from(*v));
    }
    serde_json::json!({
        "schema_version": metrics.schema_version,
        "variants_total": metrics.variants_total,
        "sample_name": metrics.sample_name,
        "sample_count": metrics.sample_count,
        "call_summary": metrics.call_summary,
        "filter_summary": metrics.filter_summary,
        "snps": metrics.snps,
        "indels": metrics.indels,
        "ti_tv": metrics.ti_tv,
        "missingness_post": metrics.missingness_post,
        "heterozygosity_ratio": metrics.heterozygosity_ratio,
        "annotation_coverage": metrics.annotation_coverage,
        "filter_breakdown": filters,
        "depth_distribution": metrics.depth_distribution,
    })
}
