use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::VcfStatsMetricsV1;

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
            "snps" => metrics.snps = v.parse().unwrap_or(0),
            "indels" => metrics.indels = v.parse().unwrap_or(0),
            "ti_tv" => metrics.ti_tv = v.parse::<f64>().ok(),
            _ if k.starts_with("filter.") => {
                let key = k.trim_start_matches("filter.").to_string();
                let value = v.parse::<u64>().unwrap_or(0);
                metrics.filter_breakdown.insert(key, value);
            }
            _ => {}
        }
    }

    if metrics.variants_total == 0 && metrics.snps == 0 && metrics.indels == 0 {
        return Err(anyhow!("vcf.stats parser found no variant counters"));
    }
    if metrics.filter_breakdown.is_empty() {
        metrics
            .filter_breakdown
            .insert("PASS".to_string(), metrics.variants_total);
    }
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
        "snps": metrics.snps,
        "indels": metrics.indels,
        "ti_tv": metrics.ti_tv,
        "filter_breakdown": filters,
    })
}
